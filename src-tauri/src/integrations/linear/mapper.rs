use rusqlite::{Connection, OptionalExtension};

use crate::db::DbState;
use crate::integrations::linear::client::{get_issue_by_identifier, get_viewer_issues, LinearIssue};
use crate::types::{Card, CardStatus, CardType, Source};

// ---------------------------------------------------------------------------
// Card deserialization helper (mirrors commands/cards.rs)
// ---------------------------------------------------------------------------

fn row_to_card(row: &rusqlite::Row) -> rusqlite::Result<Card> {
    Ok(Card {
        id: row.get(0)?,
        title: row.get(1)?,
        card_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(2)?))
            .unwrap_or(CardType::Task),
        status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?))
            .unwrap_or(CardStatus::Planned),
        impact: row.get::<_, Option<String>>(4)?.and_then(|s| {
            serde_json::from_str(&format!("\"{s}\"")).ok()
        }),
        time_estimate: row.get(5)?,
        url: row.get(6)?,
        week_id: row.get(7)?,
        day_of_week: row.get(8)?,
        position: row.get(9)?,
        source: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(10)?))
            .unwrap_or(Source::Manual),
        external_id: row.get(11)?,
        notes: row.get(12)?,
        metadata: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

const SELECT: &str =
    "SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,\
     position,source,external_id,notes,metadata,created_at,updated_at FROM cards";

// ---------------------------------------------------------------------------
// Public sync entry-point
// ---------------------------------------------------------------------------

/// Pulls all "started" Linear issues assigned to the user and upserts them as
/// `task` cards in the local backlog.
///
/// The DB lock is never held across an `.await` point.
pub async fn sync_issues(db_state: &DbState) -> Result<usize, String> {
    // Phase 1: read the API key — lock acquired and immediately released.
    let api_key = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT value FROM secrets WHERE key = 'linear_api_key'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Linear API key not configured".to_string())?
    }; // DB lock released

    // Phase 2: fetch issues from Linear (no DB lock held).
    let issues = get_viewer_issues(&api_key).await?;

    // Phase 3: compute external IDs for the open set before re-acquiring the lock.
    let open_ids: Vec<String> = issues
        .iter()
        .map(|i| format!("linear:{}", i.identifier))
        .collect();

    // Phase 4: acquire DB lock and upsert all issues.
    let mut total_count = 0usize;
    let mut card_ids: Vec<i64> = Vec::new();
    {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        for issue in &issues {
            if let Some(id) = upsert_issue(issue, None, None, &db)? {
                total_count += 1;
                card_ids.push(id);
            }
        }

        // Mark any Linear cards no longer in the open set as done.
        mark_closed_done(&open_ids, &db)?;

        // Record the successful sync timestamp.
        db.execute(
            "INSERT OR REPLACE INTO integrations (id, last_synced_at) \
             VALUES ('linear', datetime('now'))",
            [],
        )
        .map_err(|e| format!("failed to update linear last_synced_at: {e}"))?;
    } // DB lock released

    // Phase 5: AI evaluation for new/updated cards (no DB lock held during await).
    for card_id in card_ids {
        if let Err(e) = crate::ai::evaluate_card(card_id, db_state).await {
            log::warn!("[sync_linear] AI eval failed for card {card_id}: {e}");
        }
    }

    Ok(total_count)
}

// ---------------------------------------------------------------------------
// Single-issue fetch-and-upsert
// ---------------------------------------------------------------------------

/// Fetches a single Linear issue by its identifier (e.g. `"ENG-42"`), upserts
/// it as a card, and returns the full `Card` struct.
///
/// `week_id` and `day_of_week` are applied only on initial insert; existing
/// placement is preserved on subsequent calls.
pub async fn create_single_issue_card(
    db_state: &DbState,
    identifier: &str,
    week_id: Option<i64>,
    day_of_week: Option<i64>,
) -> Result<Card, String> {
    // Phase 1: read the API key.
    let api_key = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT value FROM secrets WHERE key = 'linear_api_key'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Linear API key not configured".to_string())?
    }; // DB lock released

    // Phase 2: fetch the individual issue (no DB lock held).
    let issue = get_issue_by_identifier(&api_key, identifier).await?;

    // Phase 3: upsert and read back the card.
    let (card_id, card) = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        let id = upsert_issue(&issue, week_id, day_of_week, &db)?
            .ok_or_else(|| format!("failed to upsert Linear issue '{identifier}'"))?;
        let card = db
            .query_row(&format!("{SELECT} WHERE id=?"), [id], row_to_card)
            .map_err(|e| e.to_string())?;
        (id, card)
    }; // DB lock released

    // Phase 4: AI evaluation (no DB lock held during await).
    if let Err(e) = crate::ai::evaluate_card(card_id, db_state).await {
        log::warn!("[create_single_issue_card] AI eval failed for card {card_id}: {e}");
    }

    Ok(card)
}

// ---------------------------------------------------------------------------
// DB helpers
// ---------------------------------------------------------------------------

/// Inserts or updates a Linear issue as a `cards` row.
///
/// - `week_id` / `day_of_week` are only applied on **insert** (new cards land
///   in Backlog by default when both are `None`).  Existing placement is
///   preserved on update.
/// - AI metadata keys (`ai_title`, `ai_description`, `ai_impact`, `ai_hours`)
///   are carried over from the existing row on update so AI content survives
///   re-syncs.
///
/// Returns `Some(card_id)` when a row was written, `None` on skip.
fn upsert_issue(
    issue: &LinearIssue,
    week_id: Option<i64>,
    day_of_week: Option<i64>,
    db: &Connection,
) -> Result<Option<i64>, String> {
    let external_id = format!("linear:{}", issue.identifier);

    let metadata = serde_json::json!({
        "priority": issue.priority,
        "description": issue.description,
        "estimate": issue.estimate,
        "state": issue.state.name,
    });

    let existing: Option<(i64, Option<String>)> = db
        .query_row(
            "SELECT id, metadata FROM cards WHERE external_id = ? AND source = 'linear'",
            rusqlite::params![external_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some((card_id, existing_meta_str)) = existing {
        // Update mutable fields; preserve week_id / day_of_week.
        // Carry forward any ai_* keys from the stored metadata.
        let mut merged = metadata;
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
            rusqlite::params![issue.title, issue.url, update_metadata, card_id],
        )
        .map_err(|e| format!("failed to update Linear issue card {card_id}: {e}"))?;
        Ok(Some(card_id))
    } else {
        // New issue — insert with requested placement (defaults to Backlog).
        db.execute(
            "INSERT INTO cards \
             (title, card_type, status, source, external_id, url, metadata, \
              week_id, day_of_week, position) \
             VALUES (?, 'task', 'planned', 'linear', ?, ?, ?, ?, ?, 0)",
            rusqlite::params![
                issue.title,
                external_id,
                issue.url,
                metadata.to_string(),
                week_id,
                day_of_week,
            ],
        )
        .map_err(|e| format!("failed to insert Linear issue card: {e}"))?;
        Ok(Some(db.last_insert_rowid()))
    }
}

/// Sets `status = 'done'` for every Linear card whose `external_id` is not
/// in `open_ids` (i.e. the issue is no longer in the "started" state).
fn mark_closed_done(open_ids: &[String], db: &Connection) -> Result<(), String> {
    let mut stmt = db
        .prepare(
            "SELECT id, external_id FROM cards \
             WHERE source = 'linear' AND status != 'done'",
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
            .map_err(|e| format!("failed to mark Linear card {card_id} as done: {e}"))?;
        }
    }

    Ok(())
}
