use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rusqlite::{Connection, OptionalExtension};

use crate::db::DbState;
use crate::integrations::calendar::auth::get_valid_token;
use crate::integrations::calendar::client::{fetch_events, list_calendars, GCalEvent};

/// Syncs Google Calendar events for the given ISO week into the local
/// `cards` table via upsert on `external_id`.
///
/// `week_start_date` — Monday of the week in `"YYYY-MM-DD"` format.
/// `week_id`         — the corresponding `weeks.id` row in SQLite.
///
/// The function acquires the DB lock only during the synchronous upsert
/// phase so the `MutexGuard` is never held across an `.await` point.
///
/// Syncs events from ALL user calendars (filtered to those where the user
/// has at least reader access).  Updates `integrations.last_synced_at`
/// after every successful sync.
///
/// Returns the total number of events upserted across all calendars.
pub async fn sync_events(
    week_start_date: &str,
    week_id: i64,
    db_state: &DbState,
) -> Result<usize, String> {
    // --- async phase: no DB lock held ---

    // Step 1: get a valid (non-expired) access token.
    let access_token = get_valid_token(db_state).await?;

    let monday = NaiveDate::parse_from_str(week_start_date, "%Y-%m-%d")
        .map_err(|e| format!("invalid week_start_date '{week_start_date}': {e}"))?;
    let friday = monday + chrono::Duration::days(4);

    let week_start = format!("{monday}T00:00:00Z");
    let week_end = format!("{friday}T23:59:59Z");

    // Step 2: fetch the list of all accessible calendars.
    let calendars = list_calendars(&access_token).await?;

    // Step 3: for each calendar, fetch events and upsert into DB.
    let mut total_count = 0usize;

    for calendar in &calendars {
        // Each HTTP call is awaited without holding the DB lock.
        let events = match fetch_events(&access_token, &week_start, &week_end, &calendar.id).await {
            Ok(evts) => evts,
            Err(e) => {
                // Log the per-calendar error but continue with remaining calendars.
                log::warn!(
                    "[calendar] failed to fetch events for '{}': {e}",
                    calendar.id
                );
                continue;
            }
        };

        // Acquire lock only for the synchronous DB writes.
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        for event in events {
            if upsert_event(&event, week_id, &db)? {
                total_count += 1;
            }
        }
        // Lock released here, before the next HTTP call.
    }

    // Step 4: update last_synced_at in the integrations table.
    {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.execute(
            "UPDATE integrations SET last_synced_at=datetime('now') WHERE id='calendar'",
            [],
        )
        .map_err(|e| format!("failed to update last_synced_at: {e}"))?;
    }

    Ok(total_count)
}

/// Inserts or updates a single calendar event as a `cards` row.
/// Returns `true` when the row was written, `false` when the event was
/// skipped (e.g. weekend or missing dateTime).
fn upsert_event(
    event: &GCalEvent,
    week_id: i64,
    db: &Connection,
) -> Result<bool, String> {
    let start_iso = match &event.start.date_time {
        Some(s) => s,
        // All-day events were already filtered out in client.rs, but be safe.
        None => return Ok(false),
    };

    let start: DateTime<Utc> = start_iso
        .parse()
        .map_err(|e| format!("failed to parse event start time '{start_iso}': {e}"))?;

    // Chrono: Monday = 1 … Sunday = 7.
    let day_of_week = start.weekday().number_from_monday() as i64;
    if day_of_week > 5 {
        return Ok(false); // skip weekend events
    }

    let title = event.summary.as_deref().unwrap_or("(no title)");
    let metadata = serde_json::json!({
        "start_time": event.start.date_time,
        "end_time": event.end.date_time,
    })
    .to_string();

    let existing_id: Option<i64> = db
        .query_row(
            "SELECT id FROM cards WHERE external_id = ? AND source = 'calendar'",
            rusqlite::params![event.id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(card_id) = existing_id {
        db.execute(
            "UPDATE cards SET title = ?, metadata = ?, day_of_week = ?, week_id = ?, \
             updated_at = datetime('now') WHERE id = ?",
            rusqlite::params![title, metadata, day_of_week, week_id, card_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        db.execute(
            "INSERT INTO cards \
             (title, card_type, status, source, external_id, metadata, week_id, day_of_week, position) \
             VALUES (?, 'meeting', 'planned', 'calendar', ?, ?, ?, ?, 0)",
            rusqlite::params![title, event.id, metadata, week_id, day_of_week],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(true)
}
