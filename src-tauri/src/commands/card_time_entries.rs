use crate::db::DbState;
use rusqlite::Connection;
use tauri::State;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CardTimeEntry {
    pub id: i64,
    pub card_id: i64,
    pub date: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub created_at: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CardTypeHours {
    pub card_type: String,
    pub hours: f64,
}

fn row_to_card_time_entry(row: &rusqlite::Row) -> rusqlite::Result<CardTimeEntry> {
    Ok(CardTimeEntry {
        id: row.get(0)?,
        card_id: row.get(1)?,
        date: row.get(2)?,
        start_time: row.get(3)?,
        end_time: row.get(4)?,
        created_at: row.get(5)?,
    })
}

// ---------------------------------------------------------------------------
// Pure-DB helpers — take &Connection directly so tests can call them without
// constructing Tauri State.
// ---------------------------------------------------------------------------

pub(crate) fn db_card_clock_in(
    db: &Connection,
    card_id: i64,
    date: &str,
) -> Result<CardTimeEntry, String> {
    db.execute(
        "INSERT INTO card_time_entries (card_id, date, start_time) VALUES (?, ?, datetime('now'))",
        rusqlite::params![card_id, date],
    )
    .map_err(|e| e.to_string())?;
    let id = db.last_insert_rowid();
    db.query_row(
        "SELECT id, card_id, date, start_time, end_time, created_at FROM card_time_entries WHERE id=?",
        [id],
        row_to_card_time_entry,
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn db_card_clock_out(
    db: &Connection,
    entry_id: i64,
) -> Result<CardTimeEntry, String> {
    db.execute(
        "UPDATE card_time_entries SET end_time=datetime('now') WHERE id=? AND end_time IS NULL",
        [entry_id],
    )
    .map_err(|e| e.to_string())?;
    db.query_row(
        "SELECT id, card_id, date, start_time, end_time, created_at FROM card_time_entries WHERE id=?",
        [entry_id],
        row_to_card_time_entry,
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn db_get_active_card_entry(
    db: &Connection,
    card_id: i64,
) -> Result<Option<CardTimeEntry>, String> {
    use rusqlite::OptionalExtension;
    db.query_row(
        "SELECT id, card_id, date, start_time, end_time, created_at FROM card_time_entries WHERE card_id=? AND end_time IS NULL LIMIT 1",
        [card_id],
        row_to_card_time_entry,
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub(crate) fn db_list_card_time_entries(
    db: &Connection,
    card_id: i64,
) -> Result<Vec<CardTimeEntry>, String> {
    let mut stmt = db
        .prepare(
            "SELECT id, card_id, date, start_time, end_time, created_at \
             FROM card_time_entries WHERE card_id=? ORDER BY start_time",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_map([card_id], row_to_card_time_entry)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

pub(crate) fn db_finalize_card_time(db: &Connection, card_id: i64) -> Result<(), String> {
    let total_hours: Option<f64> = db
        .query_row(
            "SELECT SUM((julianday(end_time) - julianday(start_time)) * 24.0) \
             FROM card_time_entries WHERE card_id=? AND end_time IS NOT NULL",
            [card_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let Some(hours) = total_hours else {
        return Ok(());
    };

    db.execute(
        "UPDATE cards SET time_estimate=? WHERE id=?",
        rusqlite::params![hours, card_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) fn db_list_card_entries_for_week(
    db: &Connection,
    week_id: i64,
) -> Result<Vec<CardTypeHours>, String> {
    let mut stmt = db
        .prepare(
            "SELECT c.card_type, SUM((julianday(cte.end_time) - julianday(cte.start_time)) * 24.0) as hours \
             FROM card_time_entries cte \
             JOIN cards c ON c.id = cte.card_id \
             WHERE c.week_id=? AND cte.end_time IS NOT NULL \
             GROUP BY c.card_type",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_map([week_id], |row| {
        Ok(CardTypeHours {
            card_type: row.get(0)?,
            hours: row.get(1)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Tauri commands — thin wrappers that lock the mutex and delegate to helpers.
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn card_clock_in(
    card_id: i64,
    date: String,
    state: State<DbState>,
) -> Result<CardTimeEntry, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_card_clock_in(&db, card_id, &date)
}

#[tauri::command]
pub fn card_clock_out(
    entry_id: i64,
    state: State<DbState>,
) -> Result<CardTimeEntry, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_card_clock_out(&db, entry_id)
}

#[tauri::command]
pub fn get_active_card_entry(
    card_id: i64,
    state: State<DbState>,
) -> Result<Option<CardTimeEntry>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_get_active_card_entry(&db, card_id)
}

#[tauri::command]
pub fn list_card_time_entries(
    card_id: i64,
    state: State<DbState>,
) -> Result<Vec<CardTimeEntry>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_list_card_time_entries(&db, card_id)
}

#[tauri::command]
pub fn finalize_card_time(card_id: i64, state: State<DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_finalize_card_time(&db, card_id)
}

#[tauri::command]
pub fn list_card_entries_for_week(
    week_id: i64,
    state: State<DbState>,
) -> Result<Vec<CardTypeHours>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_list_card_entries_for_week(&db, week_id)
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
        db.execute_batch(include_str!("../../migrations/0003_projects.sql"))
            .unwrap();
        let _ = db.execute(
            "ALTER TABLE cards ADD COLUMN project_id INTEGER REFERENCES projects(id)",
            [],
        );
        let _ = db.execute("ALTER TABLE cards ADD COLUMN done_at TEXT", []);
        let _ = db.execute("ALTER TABLE cards ADD COLUMN deleted_at TEXT", []);
        db.execute_batch(include_str!(
            "../../migrations/0005_card_time_tracking.sql"
        ))
        .unwrap();
        db
    }

    fn insert_test_card(db: &Connection) -> i64 {
        db.execute(
            "INSERT INTO cards (title, card_type, status, position) VALUES ('Test', 'task', 'planned', 1)",
            [],
        )
        .unwrap();
        db.last_insert_rowid()
    }

    // Clocking in must create an active entry returned by get_active_card_entry.
    #[test]
    fn clock_in_creates_active_entry() {
        let db = open_test_db();
        let card_id = insert_test_card(&db);

        let entry = db_card_clock_in(&db, card_id, "2026-03-04").unwrap();
        assert_eq!(entry.card_id, card_id);
        assert!(entry.end_time.is_none(), "new entry must have no end_time");

        let active = db_get_active_card_entry(&db, card_id).unwrap();
        assert!(active.is_some(), "active entry must be present after clock-in");
        assert_eq!(active.unwrap().id, entry.id);
    }

    // Clocking out must clear the active entry so get_active_card_entry returns None.
    #[test]
    fn clock_out_clears_active_entry() {
        let db = open_test_db();
        let card_id = insert_test_card(&db);

        let entry = db_card_clock_in(&db, card_id, "2026-03-04").unwrap();
        db_card_clock_out(&db, entry.id).unwrap();

        let active = db_get_active_card_entry(&db, card_id).unwrap();
        assert!(
            active.is_none(),
            "active entry must be absent after clock-out"
        );
    }

    // finalize_card_time must sum completed entries and write to cards.time_estimate.
    #[test]
    fn finalize_card_time_updates_time_estimate() {
        let db = open_test_db();
        let card_id = insert_test_card(&db);

        // Insert a completed entry spanning 2 hours directly.
        db.execute(
            "INSERT INTO card_time_entries (card_id, date, start_time, end_time) \
             VALUES (?, '2026-03-04', datetime('2026-03-04 10:00:00'), datetime('2026-03-04 12:00:00'))",
            [card_id],
        )
        .unwrap();

        db_finalize_card_time(&db, card_id).unwrap();

        let time_estimate: Option<f64> = db
            .query_row(
                "SELECT time_estimate FROM cards WHERE id=?",
                [card_id],
                |r| r.get(0),
            )
            .unwrap();

        let hours = time_estimate.expect("time_estimate must be set after finalize");
        assert!(
            (hours - 2.0).abs() < 0.001,
            "time_estimate must be approximately 2.0, got {hours}"
        );
    }

    // finalize_card_time with no entries must return Ok(()) without touching the card.
    #[test]
    fn finalize_card_time_no_entries_is_noop() {
        let db = open_test_db();
        let card_id = insert_test_card(&db);

        let result = db_finalize_card_time(&db, card_id);
        assert!(result.is_ok(), "finalize with no entries must return Ok(())");

        let time_estimate: Option<f64> = db
            .query_row(
                "SELECT time_estimate FROM cards WHERE id=?",
                [card_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            time_estimate.is_none(),
            "time_estimate must remain NULL when there are no entries"
        );
    }
}
