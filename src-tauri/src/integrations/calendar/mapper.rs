use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rusqlite::{Connection, OptionalExtension};

use crate::db::DbState;
use crate::integrations::calendar::auth::get_valid_token;
use crate::integrations::calendar::client::{fetch_events, list_calendars, GCalEvent};

/// Syncs Google Calendar events for the next 10 days (today … today+9) into
/// the local `cards` table via upsert on `external_id`.
///
/// The window is always computed from today (UTC midnight), so each sync is
/// independent of which week the user currently has open in the UI.
/// Events that fall on different ISO weeks are each assigned to the correct
/// `weeks` row, creating new rows as needed.
///
/// The DB lock is never held across an `.await` point.
/// Returns the total number of events upserted across all calendars.
pub async fn sync_events(db_state: &DbState) -> Result<usize, String> {
    // --- async phase: no DB lock held ---

    // Step 1: get a valid (non-expired) access token.
    let access_token = get_valid_token(db_state).await?;

    // Step 2: build a rolling 10-day window from today (UTC).
    let today = Utc::now().date_naive();
    let end = today + chrono::Duration::days(9); // inclusive — 10 days total

    let time_min = format!("{}T00:00:00Z", today.format("%Y-%m-%d"));
    let time_max = format!("{}T23:59:59Z", end.format("%Y-%m-%d"));

    // Step 3: fetch the list of all accessible calendars.
    let calendars = list_calendars(&access_token).await?;

    // Step 4: for each calendar, fetch events and upsert into DB.
    let mut total_count = 0usize;

    for calendar in &calendars {
        // Each HTTP call is awaited without holding the DB lock.
        let events = match fetch_events(&access_token, &time_min, &time_max, &calendar.id).await {
            Ok(evts) => evts,
            Err(e) => {
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
            if upsert_event(&event, &db)? {
                total_count += 1;
            }
        }
        // Lock released here, before the next HTTP call.
    }

    // Step 5: reorder all synced calendar cards by start_time within each day,
    // then update last_synced_at.
    {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        reorder_calendar_events(&db)?;
        db.execute(
            "UPDATE integrations SET last_synced_at=datetime('now') WHERE id='calendar'",
            [],
        )
        .map_err(|e| format!("failed to update last_synced_at: {e}"))?;
    }

    Ok(total_count)
}

/// Finds or creates the `weeks` row for the ISO week that contains `date`.
/// Returns the `weeks.id`.
fn get_or_create_week(db: &Connection, date: NaiveDate) -> Result<i64, String> {
    let iso = date.iso_week();
    let year = iso.year() as i64;
    let week_number = iso.week() as i64;

    // Monday of this ISO week.
    let days_from_monday = (date.weekday().number_from_monday() - 1) as i64;
    let monday = date - chrono::Duration::days(days_from_monday);
    let start_date = monday.format("%Y-%m-%d").to_string();

    // Try to find an existing row first.
    let existing: Option<i64> = db
        .query_row(
            "SELECT id FROM weeks WHERE year=? AND week_number=?",
            rusqlite::params![year, week_number],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(id) = existing {
        return Ok(id);
    }

    // Insert a new week row.
    db.execute(
        "INSERT INTO weeks (year, week_number, start_date) VALUES (?, ?, ?)",
        rusqlite::params![year, week_number, start_date],
    )
    .map_err(|e| format!("failed to create week row: {e}"))?;

    Ok(db.last_insert_rowid())
}

/// Inserts or updates a single calendar event as a `cards` row.
/// Returns `true` when the row was written, `false` when the event was
/// skipped (e.g. weekend or missing dateTime).
fn upsert_event(event: &GCalEvent, db: &Connection) -> Result<bool, String> {
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

    // Get or create the week row for this event's UTC date.
    let week_id = get_or_create_week(db, start.date_naive())?;

    let title = event.summary.as_deref().unwrap_or("(no title)");
    let metadata = serde_json::json!({
        "start_time": event.start.date_time,
        "end_time": event.end.date_time,
    })
    .to_string();

    // Compute duration in hours for time_estimate (e.g. 0.5 for a 30-min meeting).
    let time_estimate: Option<f64> = event.end.date_time.as_deref().and_then(|end_iso| {
        let end: DateTime<Utc> = end_iso.parse().ok()?;
        let mins = (end - start).num_minutes();
        if mins > 0 { Some(mins as f64 / 60.0) } else { None }
    });

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
             time_estimate = ?, updated_at = datetime('now') WHERE id = ?",
            rusqlite::params![title, metadata, day_of_week, week_id, time_estimate, card_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        db.execute(
            "INSERT INTO cards \
             (title, card_type, status, source, external_id, metadata, week_id, day_of_week, \
              position, time_estimate) \
             VALUES (?, 'meeting', 'planned', 'calendar', ?, ?, ?, ?, 0, ?)",
            rusqlite::params![title, event.id, metadata, week_id, day_of_week, time_estimate],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(true)
}

/// Assigns `position` values to all calendar cards so that within each
/// (week_id, day_of_week) group they appear in chronological order.
///
/// Position = number of other calendar cards on the same day whose
/// `start_time` is strictly earlier — giving 0-indexed ordering.
fn reorder_calendar_events(db: &Connection) -> Result<(), String> {
    db.execute(
        "UPDATE cards SET position = (
            SELECT COUNT(*) FROM cards c2
            WHERE c2.source = 'calendar'
              AND c2.week_id = cards.week_id
              AND c2.day_of_week = cards.day_of_week
              AND json_extract(c2.metadata, '$.start_time')
                < json_extract(cards.metadata, '$.start_time')
        )
        WHERE source = 'calendar'
          AND week_id IS NOT NULL
          AND day_of_week IS NOT NULL",
        [],
    )
    .map_err(|e| format!("failed to reorder calendar events: {e}"))?;
    Ok(())
}
