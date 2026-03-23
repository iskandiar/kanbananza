//! Tray icon lifecycle — setup, menu rebuild, badge updates.
//! All Tauri-specific functions are `pub(crate)`.
//! DB query helpers are `pub(crate)` so tests can call them directly.

use crate::commands::weeks::db_get_week_by_date;
use chrono::{Datelike, Local};
use rusqlite::Connection;

// ---------------------------------------------------------------------------
// DB query helpers
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub(crate) struct TodayContext {
    pub week_id: Option<i64>,
    pub day_of_week: i64,
}

/// Resolve the current week_id and day_of_week (0=Mon … 6=Sun) from today's date.
pub(crate) fn today_context(db: &Connection) -> TodayContext {
    let now = Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let day_of_week = now.weekday().num_days_from_monday() as i64;
    let week_id = db_get_week_by_date(db, &today)
        .ok()
        .flatten()
        .map(|w| w.id);
    TodayContext { week_id, day_of_week }
}

/// Today's meeting cards (card_type='meeting', status='planned'). Returns titles.
pub(crate) fn query_today_meetings(db: &Connection, ctx: &TodayContext) -> Vec<String> {
    let week_id = match ctx.week_id {
        Some(id) => id,
        None => return vec![],
    };
    let mut stmt = match db.prepare(
        "SELECT title FROM cards \
         WHERE card_type='meeting' AND status='planned' \
           AND week_id=? AND day_of_week=? AND deleted_at IS NULL \
         ORDER BY position",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map(rusqlite::params![week_id, ctx.day_of_week], |row| {
        row.get::<_, String>(0)
    })
    .map(|rows| {
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .unwrap_or_default()
    })
    .unwrap_or_default()
}

/// Today's high-priority cards (impact='high', status='planned'). Returns titles.
pub(crate) fn query_today_high_priority(db: &Connection, ctx: &TodayContext) -> Vec<String> {
    let week_id = match ctx.week_id {
        Some(id) => id,
        None => return vec![],
    };
    let mut stmt = match db.prepare(
        "SELECT title FROM cards \
         WHERE impact='high' AND status='planned' \
           AND week_id=? AND day_of_week=? AND deleted_at IS NULL \
         ORDER BY position",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map(rusqlite::params![week_id, ctx.day_of_week], |row| {
        row.get::<_, String>(0)
    })
    .map(|rows| {
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .unwrap_or_default()
    })
    .unwrap_or_default()
}

/// Returns (card_title, elapsed_minutes) for the active timer, or None.
pub(crate) fn query_active_timer(db: &Connection) -> Option<(String, i64)> {
    db.query_row(
        "SELECT c.title, cte.start_time \
         FROM card_time_entries cte \
         JOIN cards c ON c.id = cte.card_id \
         WHERE cte.end_time IS NULL \
         LIMIT 1",
        [],
        |row| {
            let title: String = row.get(0)?;
            let start_time: String = row.get(1)?;
            Ok((title, start_time))
        },
    )
    .ok()
    .map(|(title, start_time)| {
        let elapsed = elapsed_minutes(&start_time);
        (title, elapsed)
    })
}

/// Count of today's incomplete planned cards (for dock badge).
pub(crate) fn query_badge_count(db: &Connection, ctx: &TodayContext) -> i64 {
    let week_id = match ctx.week_id {
        Some(id) => id,
        None => return 0,
    };
    db.query_row(
        "SELECT COUNT(*) FROM cards \
         WHERE status='planned' AND week_id=? AND day_of_week=? AND deleted_at IS NULL",
        rusqlite::params![week_id, ctx.day_of_week],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
}

/// Returns the card_id of the most-recently-updated planned card for today (for Clock In).
pub(crate) fn query_most_recent_planned_card(db: &Connection, ctx: &TodayContext) -> Option<i64> {
    let week_id = ctx.week_id?;
    db.query_row(
        "SELECT id FROM cards \
         WHERE status='planned' AND week_id=? AND day_of_week=? AND deleted_at IS NULL \
         ORDER BY updated_at DESC LIMIT 1",
        rusqlite::params![week_id, ctx.day_of_week],
        |row| row.get::<_, i64>(0),
    )
    .ok()
}

/// Parse a SQLite datetime string (e.g. "2026-03-23 10:30:00") and return elapsed minutes.
fn elapsed_minutes(start_time: &str) -> i64 {
    use chrono::NaiveDateTime;
    let fmt = "%Y-%m-%d %H:%M:%S";
    let start = NaiveDateTime::parse_from_str(start_time, fmt).ok();
    match start {
        Some(s) => {
            let now = Local::now().naive_utc();
            (now - s).num_minutes().max(0)
        }
        None => 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{
        card_time_entries::db_card_clock_in,
        cards::db_create_card,
        weeks::db_get_or_create_week,
    };
    use crate::types::CardType;
    use rusqlite::Connection;

    fn open_test_db() -> Connection {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        db.execute_batch(include_str!("../migrations/0002_auto_ai.sql"))
            .unwrap();
        db.execute_batch(include_str!("../migrations/0003_projects.sql"))
            .unwrap();
        let _ = db.execute(
            "ALTER TABLE cards ADD COLUMN project_id INTEGER REFERENCES projects(id)",
            [],
        );
        let _ = db.execute("ALTER TABLE cards ADD COLUMN done_at TEXT", []);
        let _ = db.execute("ALTER TABLE cards ADD COLUMN deleted_at TEXT", []);
        db.execute_batch(include_str!("../migrations/0004_soft_delete.sql"))
            .unwrap();
        db.execute_batch(include_str!("../migrations/0005_card_time_tracking.sql"))
            .unwrap();
        db
    }

    // Seed a week+card and return (week_id, card_id)
    fn seed_card(
        db: &Connection,
        card_type: &CardType,
        impact: Option<&str>,
        day: i64,
    ) -> (i64, i64) {
        let week = db_get_or_create_week(db, 2026, 12, "2026-03-16").unwrap();
        let card =
            db_create_card(db, "Test card", card_type, Some(week.id), Some(day), None, None)
                .unwrap();
        if let Some(imp) = impact {
            db.execute(
                "UPDATE cards SET impact=? WHERE id=?",
                rusqlite::params![imp, card.id],
            )
            .unwrap();
        }
        (week.id, card.id)
    }

    #[test]
    fn meetings_returns_meeting_cards_for_today() {
        let db = open_test_db();
        let (week_id, _) = seed_card(&db, &CardType::Meeting, None, 0);
        let ctx = TodayContext {
            week_id: Some(week_id),
            day_of_week: 0,
        };
        let meetings = query_today_meetings(&db, &ctx);
        assert_eq!(meetings.len(), 1);
        assert_eq!(meetings[0], "Test card");
    }

    #[test]
    fn meetings_excludes_non_meeting_cards() {
        let db = open_test_db();
        let (week_id, _) = seed_card(&db, &CardType::Task, None, 0);
        let ctx = TodayContext {
            week_id: Some(week_id),
            day_of_week: 0,
        };
        assert!(query_today_meetings(&db, &ctx).is_empty());
    }

    #[test]
    fn high_priority_returns_high_impact_cards() {
        let db = open_test_db();
        let (week_id, _) = seed_card(&db, &CardType::Task, Some("high"), 0);
        let ctx = TodayContext {
            week_id: Some(week_id),
            day_of_week: 0,
        };
        let hp = query_today_high_priority(&db, &ctx);
        assert_eq!(hp.len(), 1);
    }

    #[test]
    fn high_priority_excludes_low_impact_cards() {
        let db = open_test_db();
        let (week_id, _) = seed_card(&db, &CardType::Task, Some("low"), 0);
        let ctx = TodayContext {
            week_id: Some(week_id),
            day_of_week: 0,
        };
        assert!(query_today_high_priority(&db, &ctx).is_empty());
    }

    #[test]
    fn badge_count_returns_planned_card_count() {
        let db = open_test_db();
        let (week_id, _) = seed_card(&db, &CardType::Task, None, 0);
        seed_card(&db, &CardType::Task, None, 0);
        let ctx = TodayContext {
            week_id: Some(week_id),
            day_of_week: 0,
        };
        assert_eq!(query_badge_count(&db, &ctx), 2);
    }

    #[test]
    fn badge_count_zero_when_no_week() {
        let db = open_test_db();
        let ctx = TodayContext {
            week_id: None,
            day_of_week: 0,
        };
        assert_eq!(query_badge_count(&db, &ctx), 0);
    }

    #[test]
    fn active_timer_returns_none_when_no_entry() {
        let db = open_test_db();
        assert!(query_active_timer(&db).is_none());
    }

    #[test]
    fn active_timer_returns_card_title_when_clocked_in() {
        let db = open_test_db();
        let (_, card_id) = seed_card(&db, &CardType::Task, None, 0);
        db_card_clock_in(&db, card_id, "2026-03-23").unwrap();
        let result = query_active_timer(&db);
        assert!(result.is_some());
        let (title, elapsed) = result.unwrap();
        assert_eq!(title, "Test card");
        assert!(elapsed >= 0);
    }

    #[test]
    fn most_recent_planned_card_returns_card_id() {
        let db = open_test_db();
        let (week_id, card_id) = seed_card(&db, &CardType::Task, None, 0);
        let ctx = TodayContext {
            week_id: Some(week_id),
            day_of_week: 0,
        };
        assert_eq!(query_most_recent_planned_card(&db, &ctx), Some(card_id));
    }
}
