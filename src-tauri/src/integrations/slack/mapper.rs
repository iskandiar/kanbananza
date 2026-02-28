use rusqlite::OptionalExtension;

use crate::db::{DbState, row_to_card};
use crate::integrations::slack::client;
use crate::types::Card;

const SELECT: &str =
    "SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,\
     position,source,external_id,notes,metadata,created_at,updated_at FROM cards";

// ---------------------------------------------------------------------------
// URL parsing
// ---------------------------------------------------------------------------

/// Parses a Slack thread URL of the form:
/// `https://{workspace}.slack.com/archives/{channel_id}/p{thread_ts_raw}`
///
/// Returns `(channel_id, thread_ts)` where `thread_ts` has a dot inserted
/// 10 characters from the start (e.g. `p1234567890123456` → `1234567890.123456`).
fn parse_slack_url(url: &str) -> Result<(String, String), String> {
    // Strip query params / fragments.
    let path_part = url.split('?').next().unwrap_or(url);
    let path_part = path_part.split('#').next().unwrap_or(path_part);

    // Split path into segments, ignoring empty ones.
    let segments: Vec<&str> = path_part
        .trim_end_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    // Expected path structure: ...slack.com / archives / {channel_id} / p{ts_raw}
    // Find the "archives" sentinel and take the two segments after it.
    let archives_pos = segments
        .iter()
        .position(|&s| s == "archives")
        .ok_or_else(|| format!("Slack URL missing 'archives' segment: {url}"))?;

    let channel_id = segments
        .get(archives_pos + 1)
        .copied()
        .ok_or_else(|| format!("Slack URL missing channel_id segment: {url}"))?
        .to_string();

    let ts_segment = segments
        .get(archives_pos + 2)
        .copied()
        .ok_or_else(|| format!("Slack URL missing thread_ts segment: {url}"))?;

    // ts_segment starts with 'p' followed by digits — strip the leading 'p'.
    let ts_raw = ts_segment
        .strip_prefix('p')
        .ok_or_else(|| format!("Slack thread_ts segment does not start with 'p': {ts_segment}"))?;

    // Insert a dot 10 characters from the start: "1234567890123456" → "1234567890.123456"
    if ts_raw.len() < 11 {
        return Err(format!(
            "Slack thread_ts too short to insert dot: {ts_raw}"
        ));
    }
    let thread_ts = format!("{}.{}", &ts_raw[..10], &ts_raw[10..]);

    Ok((channel_id, thread_ts))
}

// ---------------------------------------------------------------------------
// Public mapper
// ---------------------------------------------------------------------------

/// Fetches a Slack thread by URL and upserts it into the `cards` table as a
/// `thread` card (source = `slack`).
///
/// The DB lock is never held across an `.await` point.
/// Returns the upserted `Card`.
pub async fn create_card_from_url(
    db_state: &DbState,
    url: String,
    week_id: Option<i64>,
    day_of_week: Option<i64>,
) -> Result<Card, String> {
    // Step 1 — parse channel_id and thread_ts from the URL (no I/O).
    let (channel_id, thread_ts) = parse_slack_url(&url)?;

    // Step 2 — read API token from DB (lock briefly).
    let token = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT value FROM secrets WHERE key = 'slack_api_key'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Slack API token not configured".to_string())?
    }; // DB lock released before any await

    // Step 3 — fetch channel name (no lock held).
    let channel_name = client::get_channel_info(&token, &channel_id).await?;

    // Step 4 — fetch thread messages (no lock held).
    let thread = client::get_thread_replies(&token, &channel_id, &thread_ts).await?;

    // Step 5 — build metadata.
    let metadata = serde_json::json!({
        "source_url": url,
        "channel_id": channel_id,
        "channel_name": channel_name,
        "thread_ts": thread_ts,
        "reply_count": thread.reply_count,
        "first_message": thread.first_message,
        "thread_preview": thread.thread_preview,
    })
    .to_string();

    let external_id = format!("slack:{channel_id}:{thread_ts}");
    let title = format!("Thread in #{channel_name}");

    // Step 6 — upsert card (lock briefly).
    let card_id: i64 = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;

        let existing: Option<(i64, Option<String>)> = db
            .query_row(
                "SELECT id, metadata FROM cards WHERE external_id = ? AND source = 'slack'",
                rusqlite::params![external_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some((id, existing_meta_str)) = existing {
            // Rebuild metadata, preserving any ai_* fields from the existing row.
            let mut new_meta = serde_json::json!({
                "source_url": url,
                "channel_id": channel_id,
                "channel_name": channel_name,
                "thread_ts": thread_ts,
                "reply_count": thread.reply_count,
                "first_message": thread.first_message,
                "thread_preview": thread.thread_preview,
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

            match (week_id, day_of_week) {
                (Some(wid), Some(dow)) => {
                    db.execute(
                        "UPDATE cards SET title=?, metadata=?, week_id=?, day_of_week=?, \
                         updated_at=datetime('now') WHERE id=?",
                        rusqlite::params![title, update_metadata, wid, dow, id],
                    )
                    .map_err(|e| format!("failed to update Slack card {id}: {e}"))?;
                }
                _ => {
                    db.execute(
                        "UPDATE cards SET title=?, metadata=?, \
                         updated_at=datetime('now') WHERE id=?",
                        rusqlite::params![title, update_metadata, id],
                    )
                    .map_err(|e| format!("failed to update Slack card {id}: {e}"))?;
                }
            }
            id
        } else {
            // New card — land in Backlog (week_id=NULL) unless caller supplies placement.
            db.execute(
                "INSERT INTO cards \
                 (title, card_type, status, source, external_id, url, metadata, \
                  week_id, day_of_week, position) \
                 VALUES (?, 'thread', 'planned', 'slack', ?, ?, ?, ?, ?, 0)",
                rusqlite::params![
                    title,
                    external_id,
                    url,
                    metadata,
                    week_id,
                    day_of_week,
                ],
            )
            .map_err(|e| format!("failed to insert Slack card: {e}"))?;
            db.last_insert_rowid()
        }
    }; // DB lock released

    // Step 7 — AI evaluation (fire-and-forget; respects auto_ai toggle internally).
    if let Err(e) = crate::ai::evaluate_card(card_id, db_state).await {
        log::warn!("[slack] AI eval failed for card {card_id}: {e}");
    }

    // Step 8 — read back the final card row.
    let card = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            &format!("{SELECT} WHERE id=?"),
            [card_id],
            row_to_card,
        )
        .map_err(|e| format!("failed to read back Slack card {card_id}: {e}"))?
    };

    Ok(card)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::parse_slack_url;

    // A standard Slack thread URL.
    #[test]
    fn normal_url() {
        let url = "https://myworkspace.slack.com/archives/C01234567/p1234567890123456";
        let (channel_id, thread_ts) = parse_slack_url(url).unwrap();
        assert_eq!(channel_id, "C01234567");
        assert_eq!(thread_ts, "1234567890.123456");
    }

    // The 'p' prefix is stripped and a dot is inserted at position 10.
    #[test]
    fn p_prefixed_ts_dot_insertion() {
        let url = "https://workspace.slack.com/archives/CABC123/p9876543210654321";
        let (channel_id, thread_ts) = parse_slack_url(url).unwrap();
        assert_eq!(channel_id, "CABC123");
        assert_eq!(thread_ts, "9876543210.654321");
    }

    // Query params after '?' must be stripped before parsing.
    #[test]
    fn query_params_stripped() {
        let url = "https://myworkspace.slack.com/archives/C01234567/p1234567890123456?thread_ts=1234567890.123456&cid=C01234567";
        let (channel_id, thread_ts) = parse_slack_url(url).unwrap();
        assert_eq!(channel_id, "C01234567");
        assert_eq!(thread_ts, "1234567890.123456");
    }

    // Fragment (#...) after the path must also be stripped.
    #[test]
    fn fragment_stripped() {
        let url = "https://myworkspace.slack.com/archives/C01234567/p1234567890123456#top";
        let (channel_id, thread_ts) = parse_slack_url(url).unwrap();
        assert_eq!(channel_id, "C01234567");
        assert_eq!(thread_ts, "1234567890.123456");
    }

    // A URL without the 'archives' path segment must return an Err.
    #[test]
    fn missing_archives_returns_err() {
        let url = "https://myworkspace.slack.com/messages/C01234567/p1234567890123456";
        assert!(
            parse_slack_url(url).is_err(),
            "expected Err when 'archives' segment is absent"
        );
    }
}
