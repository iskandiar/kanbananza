use crate::db::DbState;
use crate::types::Week;
use rusqlite::Connection;
use tauri::State;

fn row_to_week(row: &rusqlite::Row) -> rusqlite::Result<Week> {
    Ok(Week {
        id: row.get(0)?,
        year: row.get(1)?,
        week_number: row.get(2)?,
        start_date: row.get(3)?,
        summary: row.get(4)?,
    })
}

// ---------------------------------------------------------------------------
// Pure-DB helpers
// ---------------------------------------------------------------------------

pub(crate) fn db_get_or_create_week(
    db: &Connection,
    year: i64,
    week_number: i64,
    start_date: &str,
) -> Result<Week, String> {
    db.execute(
        "INSERT OR IGNORE INTO weeks (year, week_number, start_date) VALUES (?, ?, ?)",
        rusqlite::params![year, week_number, start_date],
    )
    .map_err(|e| e.to_string())?;
    db.query_row(
        "SELECT id,year,week_number,start_date,summary FROM weeks WHERE year=? AND week_number=?",
        rusqlite::params![year, week_number],
        row_to_week,
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn db_get_week_by_date(db: &Connection, date: &str) -> Result<Option<Week>, String> {
    let result = db.query_row(
        "SELECT id,year,week_number,start_date,summary FROM weeks WHERE start_date <= ? ORDER BY start_date DESC LIMIT 1",
        [date],
        row_to_week,
    );
    match result {
        Ok(week) => Ok(Some(week)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_or_create_week(
    year: i64,
    week_number: i64,
    start_date: String,
    state: State<DbState>,
) -> Result<Week, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_get_or_create_week(&db, year, week_number, &start_date)
}

#[tauri::command]
pub fn get_week_by_date(date: String, state: State<DbState>) -> Result<Option<Week>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_get_week_by_date(&db, &date)
}

#[tauri::command]
pub fn list_weeks(state: State<DbState>) -> Result<Vec<Week>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id,year,week_number,start_date,summary FROM weeks ORDER BY year DESC, week_number DESC")
        .map_err(|e| e.to_string())?;
    let weeks = stmt
        .query_map([], row_to_week)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(weeks)
}

#[tauri::command]
pub fn update_week_summary(id: i64, summary: String, state: State<DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute("UPDATE weeks SET summary=? WHERE id=?", rusqlite::params![summary, id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_test_db() -> Connection {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("../../migrations/0001_initial.sql"))
            .unwrap();
        db.execute_batch(include_str!("../../migrations/0002_auto_ai.sql"))
            .unwrap();
        db
    }

    // Calling get_or_create_week twice with the same (year, week_number) must
    // return the same row id both times (INSERT OR IGNORE idempotency).
    #[test]
    fn get_or_create_week_is_idempotent() {
        let db = open_test_db();

        let first = db_get_or_create_week(&db, 2026, 9, "2026-02-23").unwrap();
        let second = db_get_or_create_week(&db, 2026, 9, "2026-02-23").unwrap();

        assert_eq!(
            first.id, second.id,
            "second call must return the same row id"
        );
        assert_eq!(first.year, 2026);
        assert_eq!(first.week_number, 9);
        assert_eq!(first.start_date, "2026-02-23");
    }

    // get_week_by_date must return the week whose start_date is on or before
    // the queried date, preferring the latest one.
    #[test]
    fn get_week_by_date_returns_correct_week() {
        let db = open_test_db();

        // Insert two consecutive weeks.
        db_get_or_create_week(&db, 2026, 8, "2026-02-16").unwrap();
        let w9 = db_get_or_create_week(&db, 2026, 9, "2026-02-23").unwrap();

        // A date inside week 9 must resolve to week 9.
        let found = db_get_week_by_date(&db, "2026-02-25").unwrap();
        assert!(found.is_some(), "expected Some(week) for a date inside a known week");
        assert_eq!(
            found.unwrap().id,
            w9.id,
            "must resolve to week 9, not week 8"
        );
    }

    // get_week_by_date must return None when no week exists before the date.
    #[test]
    fn get_week_by_date_returns_none_when_empty() {
        let db = open_test_db();
        let result = db_get_week_by_date(&db, "2026-01-01").unwrap();
        assert!(result.is_none());
    }
}
