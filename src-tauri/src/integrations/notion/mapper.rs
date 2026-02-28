use rusqlite::OptionalExtension;

use crate::db::{DbState, row_to_card};
use crate::integrations::notion::client;
use crate::types::Card;

const SELECT: &str =
    "SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,\
     position,source,external_id,notes,metadata,created_at,updated_at FROM cards";

// ---------------------------------------------------------------------------
// Page-ID extraction
// ---------------------------------------------------------------------------

/// Extracts the Notion page ID (32 hex characters, no hyphens) from a
/// `notion.so` URL.
///
/// Handles the two common patterns:
/// - `https://notion.so/{workspace-slug}/{title}-{32hex}`
/// - `https://notion.so/{32hex}`
///
/// Any hyphens inside the hex segment are stripped before validation.
fn extract_page_id(url: &str) -> Result<String, String> {
    // Split on '?' to drop query params, then take the path part.
    let path = url.split('?').next().unwrap_or(url);
    // The last non-empty path segment is the one containing the ID.
    let last_segment = path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .ok_or_else(|| format!("cannot parse page ID from URL: {url}"))?;

    // The page ID is always the last 32 hex chars of the segment (after any
    // title slug separated by a hyphen). Strip all hyphens first, then take
    // the trailing 32 hex characters.
    let stripped: String = last_segment
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();

    if stripped.len() < 32 {
        return Err(format!(
            "could not find a 32-char hex page ID in URL segment '{last_segment}'"
        ));
    }

    // Take the last 32 hex characters (handles slug-prefixed segments).
    Ok(stripped[stripped.len() - 32..].to_string())
}

// ---------------------------------------------------------------------------
// Public mapper
// ---------------------------------------------------------------------------

/// Fetches a Notion page by URL and upserts it into the `cards` table as a
/// `documentation` card (source = `notion`).
///
/// The DB lock is never held across an `.await` point.
/// Returns the upserted `Card`.
pub async fn create_card_from_url(
    db_state: &DbState,
    url: String,
    week_id: Option<i64>,
    day_of_week: Option<i64>,
) -> Result<Card, String> {
    // Step 1 — extract page ID from URL (no I/O needed).
    let page_id = extract_page_id(&url)?;

    // Step 2 — read API key from DB (lock briefly).
    let api_key = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT value FROM secrets WHERE key = 'notion_api_key'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Notion API key not configured".to_string())?
    }; // DB lock released before any await

    // Step 3 — fetch page title (no lock held).
    let page = client::fetch_page(&api_key, &page_id).await?;

    // Step 4 — fetch block content (no lock held).
    let content = client::fetch_blocks(&api_key, &page_id).await?;

    // Step 5 — build metadata.
    let word_count: usize = content.split_whitespace().count();
    let content_preview: String = content.chars().take(500).collect();
    let metadata = serde_json::json!({
        "source_url": url,
        "page_id": page_id,
        "word_count": word_count,
        "content_preview": content_preview,
    })
    .to_string();

    let external_id = format!("notion:{page_id}");

    // Step 6 — upsert card (lock briefly).
    let card_id: i64 = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;

        let existing: Option<(i64, Option<String>)> = db
            .query_row(
                "SELECT id, metadata FROM cards WHERE external_id = ? AND source = 'notion'",
                rusqlite::params![external_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some((id, existing_meta_str)) = existing {
            // Update mutable fields; preserve week placement if caller did not
            // supply one. Merge ai_* keys from the existing metadata row so
            // that AI-generated content survives re-fetches.
            let mut new_meta = serde_json::json!({
                "source_url": url,
                "page_id": page_id,
                "word_count": word_count,
                "content_preview": content_preview,
            });
            if let Some(s) = existing_meta_str {
                if let Ok(existing_val) = serde_json::from_str::<serde_json::Value>(&s) {
                    if let (Some(new_obj), Some(existing_obj)) =
                        (new_meta.as_object_mut(), existing_val.as_object())
                    {
                        for key in ["ai_title", "ai_description", "ai_impact", "ai_hours"] {
                            if let Some(v) = existing_obj.get(key) {
                                new_obj.insert(key.to_string(), v.clone());
                            }
                        }
                    }
                }
            }
            let update_metadata = new_meta.to_string();

            // Only update week placement when explicitly provided by caller.
            match (week_id, day_of_week) {
                (Some(wid), Some(dow)) => {
                    db.execute(
                        "UPDATE cards SET title=?, metadata=?, week_id=?, day_of_week=?, \
                         updated_at=datetime('now') WHERE id=?",
                        rusqlite::params![page.title, update_metadata, wid, dow, id],
                    )
                    .map_err(|e| format!("failed to update Notion card {id}: {e}"))?;
                }
                _ => {
                    db.execute(
                        "UPDATE cards SET title=?, metadata=?, \
                         updated_at=datetime('now') WHERE id=?",
                        rusqlite::params![page.title, update_metadata, id],
                    )
                    .map_err(|e| format!("failed to update Notion card {id}: {e}"))?;
                }
            }
            id
        } else {
            // New card — land in Backlog (week_id=NULL) unless caller supplies placement.
            db.execute(
                "INSERT INTO cards \
                 (title, card_type, status, source, external_id, url, metadata, \
                  week_id, day_of_week, position) \
                 VALUES (?, 'documentation', 'planned', 'notion', ?, ?, ?, ?, ?, 0)",
                rusqlite::params![
                    page.title,
                    external_id,
                    url,
                    metadata,
                    week_id,
                    day_of_week,
                ],
            )
            .map_err(|e| format!("failed to insert Notion card: {e}"))?;
            db.last_insert_rowid()
        }
    }; // DB lock released

    // Step 7 — AI evaluation (fire-and-forget; respects auto_ai toggle internally).
    if let Err(e) = crate::ai::evaluate_card(card_id, db_state).await {
        log::warn!("[notion] AI eval failed for card {card_id}: {e}");
    }

    // Step 8 — read back the final card row.
    let card = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            &format!("{SELECT} WHERE id=?"),
            [card_id],
            row_to_card,
        )
        .map_err(|e| format!("failed to read back Notion card {card_id}: {e}"))?
    };

    Ok(card)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::extract_page_id;

    // Plain 32-char hex ID — no title slug, no query params.
    #[test]
    fn plain_32_hex_id() {
        let url = "https://notion.so/0123456789abcdef0123456789abcdef";
        assert_eq!(
            extract_page_id(url).unwrap(),
            "0123456789abcdef0123456789abcdef"
        );
    }

    // The ID is the trailing hex portion after a human-readable title slug.
    #[test]
    fn id_after_title_slug() {
        let url = "https://www.notion.so/myworkspace/My-Document-Title-0123456789abcdef0123456789abcdef";
        assert_eq!(
            extract_page_id(url).unwrap(),
            "0123456789abcdef0123456789abcdef"
        );
    }

    // A trailing slash must not confuse the segment splitter.
    #[test]
    fn trailing_slash() {
        let url = "https://notion.so/0123456789abcdef0123456789abcdef/";
        assert_eq!(
            extract_page_id(url).unwrap(),
            "0123456789abcdef0123456789abcdef"
        );
    }

    // Everything after '?' must be stripped before parsing.
    #[test]
    fn query_params_stripped() {
        let url = "https://notion.so/0123456789abcdef0123456789abcdef?pvs=4&foo=bar";
        assert_eq!(
            extract_page_id(url).unwrap(),
            "0123456789abcdef0123456789abcdef"
        );
    }

    // A hex-like segment with fewer than 32 hex chars must return an Err.
    #[test]
    fn short_id_returns_err() {
        let url = "https://notion.so/0123456789abcdef"; // only 16 hex chars
        assert!(
            extract_page_id(url).is_err(),
            "expected Err for a segment with fewer than 32 hex chars"
        );
    }
}
