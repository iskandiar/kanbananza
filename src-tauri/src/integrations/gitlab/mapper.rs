use rusqlite::{Connection, OptionalExtension};
use std::collections::HashMap;

use crate::db::DbState;
use crate::integrations::gitlab::client::{
    fetch_authored_mrs, fetch_mr_lines_changed, fetch_review_mrs, get_current_user, GitLabMR,
};

/// Syncs GitLab MRs (both authored and assigned-for-review) into the local
/// `cards` table via upsert on `external_id`.
///
/// The DB lock is never held across an `.await` point.
/// Returns the total number of MR cards written (inserted or updated).
pub async fn sync_mrs(db_state: &DbState) -> Result<usize, String> {
    // --- async phase: no DB lock held ---

    // Step 1: read the PAT from local storage.
    let pat = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT value FROM secrets WHERE key = 'gitlab_pat'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "GitLab PAT not configured".to_string())?
    }; // DB lock released before any await

    // Step 2: resolve the authenticated user (needed for reviewer_id lookup).
    let current_user = get_current_user(&pat).await?;

    // Step 3: fetch authored and review MRs sequentially.
    // (tokio is compiled without the `macros` feature, so tokio::join! is not
    // available; these calls are infrequent enough that sequential is fine.)
    let authored_mrs = fetch_authored_mrs(&pat).await?;
    let review_mrs = fetch_review_mrs(&pat, current_user.id).await?;

    // Step 4: merge and dedup by MR `id`.
    // "author" role wins when the same MR appears in both lists (user opened
    // an MR and is also listed as a reviewer on it).
    //
    // We use a LinkedHashMap-style approach: insert review MRs first, then
    // overwrite with authored MRs so that "author" always wins on collision.
    let mut mr_map: HashMap<i64, (&GitLabMR, &str)> = HashMap::new();

    for mr in &review_mrs {
        mr_map.insert(mr.id, (mr, "reviewer"));
    }
    for mr in &authored_mrs {
        mr_map.insert(mr.id, (mr, "author"));
    }

    // Collect the deduplicated list.
    let merged: Vec<(&GitLabMR, &str)> = mr_map.into_values().collect();

    // Fetch line counts for each deduplicated MR (sequential; fire-and-forget on error).
    let mut merged_with_lines: Vec<(&GitLabMR, &str, i64)> = Vec::with_capacity(merged.len());
    for (mr, role) in &merged {
        let lines = fetch_mr_lines_changed(&pat, mr.project_id, mr.iid).await;
        merged_with_lines.push((mr, role, lines));
    }

    let open_ids: Vec<String> = merged_with_lines
        .iter()
        .map(|(mr, _, _)| format!("gitlab:{}:{}", mr.project_id, mr.iid))
        .collect();

    // Step 5: acquire DB lock and upsert each MR.
    let mut total_count = 0usize;
    let mut card_ids: Vec<i64> = Vec::new();
    {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        for (mr, role, lines_changed) in &merged_with_lines {
            if let Some(id) = upsert_mr(mr, role, *lines_changed, &db)? {
                total_count += 1;
                card_ids.push(id);
            }
        }

        // Step 6: mark any previously-synced MR cards that are no longer open
        // as done.
        mark_closed_done(&open_ids, &db)?;

        // Step 7: record the successful sync timestamp.
        db.execute(
            "INSERT OR REPLACE INTO integrations (id, enabled, last_synced_at) \
             VALUES ('gitlab', 1, datetime('now'))",
            [],
        )
        .map_err(|e| format!("failed to update gitlab last_synced_at: {e}"))?;
    } // DB lock released

    for card_id in card_ids {
        if let Err(e) = crate::ai::evaluate_card(card_id, db_state).await {
            log::warn!("[sync_gitlab] AI eval failed for card {card_id}: {e}");
        }
    }

    Ok(total_count)
}

/// Inserts or updates a single GitLab MR as a `cards` row.
///
/// On update, only mutable metadata fields (title, url, metadata,
/// updated_at) are touched — user placement (week_id, day_of_week) is
/// preserved.
///
/// Returns `Some(card_id)` when the row was written, `None` on skip.
fn upsert_mr(mr: &GitLabMR, role: &str, lines_changed: i64, db: &Connection) -> Result<Option<i64>, String> {
    let external_id = format!("gitlab:{}:{}", mr.project_id, mr.iid);

    let metadata = serde_json::json!({
        "author": mr.author.username,
        "mr_iid": mr.iid,
        "role": role,
        "description": mr.description,
        "lines_changed": lines_changed,
    })
    .to_string();

    let existing: Option<(i64, Option<String>)> = db
        .query_row(
            "SELECT id, metadata FROM cards WHERE external_id = ? AND source = 'gitlab'",
            rusqlite::params![external_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some((card_id, existing_meta_str)) = existing {
        // Update mutable fields; preserve week_id / day_of_week.
        // Merge ai_* fields from existing metadata so AI content survives re-syncs.
        let mut merged = serde_json::json!({
            "author": mr.author.username,
            "mr_iid": mr.iid,
            "role": role,
            "description": mr.description,
            "lines_changed": lines_changed,
        });
        if let Some(s) = existing_meta_str {
            if let Ok(existing_val) = serde_json::from_str::<serde_json::Value>(&s) {
                if let (Some(new_obj), Some(existing_obj)) =
                    (merged.as_object_mut(), existing_val.as_object())
                {
                    for key in ["ai_title", "ai_description", "ai_impact", "ai_hours"] {
                        if let Some(v) = existing_obj.get(key) {
                            new_obj.insert(key.to_string(), v.clone());
                        }
                    }
                }
            }
        }
        let update_metadata = merged.to_string();
        db.execute(
            "UPDATE cards SET title = ?, url = ?, metadata = ?, \
             updated_at = datetime('now') WHERE id = ?",
            rusqlite::params![mr.title, mr.web_url, update_metadata, card_id],
        )
        .map_err(|e| format!("failed to update GitLab MR card {card_id}: {e}"))?;
        Ok(Some(card_id))
    } else {
        // New MR — land in Backlog (week_id = NULL, day_of_week = NULL).
        db.execute(
            "INSERT INTO cards \
             (title, card_type, status, source, external_id, url, metadata, \
              week_id, day_of_week, position) \
             VALUES (?, 'mr', 'planned', 'gitlab', ?, ?, ?, NULL, NULL, 0)",
            rusqlite::params![mr.title, external_id, mr.web_url, metadata],
        )
        .map_err(|e| format!("failed to insert GitLab MR card: {e}"))?;
        Ok(Some(db.last_insert_rowid()))
    }
}

/// For every GitLab card that is not in `open_ids` (i.e. the MR is no longer
/// open on GitLab), sets `status = 'done'`.
///
/// This handles MRs that were merged or closed since the last sync.
fn mark_closed_done(open_ids: &[String], db: &Connection) -> Result<(), String> {
    // Fetch all non-done GitLab cards so we can diff against open_ids.
    let mut stmt = db
        .prepare(
            "SELECT id, external_id FROM cards \
             WHERE source = 'gitlab' AND status != 'done'",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(i64, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (card_id, ext_id) in rows {
        if !open_ids.contains(&ext_id) {
            db.execute(
                "UPDATE cards SET status = 'done', updated_at = datetime('now') \
                 WHERE id = ?",
                rusqlite::params![card_id],
            )
            .map_err(|e| format!("failed to mark GitLab card {card_id} as done: {e}"))?;
        }
    }

    Ok(())
}
