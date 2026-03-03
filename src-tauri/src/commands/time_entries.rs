use crate::db::DbState;
use tauri::State;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimeEntry {
    pub id: i64,
    pub date: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

fn row_to_time_entry(row: &rusqlite::Row) -> rusqlite::Result<TimeEntry> {
    Ok(TimeEntry {
        id: row.get(0)?,
        date: row.get(1)?,
        start_time: row.get(2)?,
        end_time: row.get(3)?,
        notes: row.get(4)?,
        created_at: row.get(5)?,
    })
}

#[tauri::command]
pub fn clock_in(date: String, state: State<DbState>) -> Result<TimeEntry, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO time_entries (date, start_time) VALUES (?, datetime('now'))",
        [&date],
    )
    .map_err(|e| e.to_string())?;
    let id = db.last_insert_rowid();
    db.query_row(
        "SELECT id, date, start_time, end_time, notes, created_at FROM time_entries WHERE id=?",
        [id],
        row_to_time_entry,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clock_out(entry_id: i64, state: State<DbState>) -> Result<TimeEntry, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE time_entries SET end_time=datetime('now') WHERE id=? AND end_time IS NULL",
        [entry_id],
    )
    .map_err(|e| e.to_string())?;
    db.query_row(
        "SELECT id, date, start_time, end_time, notes, created_at FROM time_entries WHERE id=?",
        [entry_id],
        row_to_time_entry,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_time_entries(date: String, state: State<DbState>) -> Result<Vec<TimeEntry>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare(
            "SELECT id, date, start_time, end_time, notes, created_at \
             FROM time_entries WHERE date=? ORDER BY start_time",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_map([&date], row_to_time_entry)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_time_entry(
    id: i64,
    start_time: Option<String>,
    end_time: Option<String>,
    notes: Option<String>,
    state: State<DbState>,
) -> Result<TimeEntry, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(ref st) = start_time {
        db.execute(
            "UPDATE time_entries SET start_time=? WHERE id=?",
            rusqlite::params![st, id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref et) = end_time {
        db.execute(
            "UPDATE time_entries SET end_time=? WHERE id=?",
            rusqlite::params![et, id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref n) = notes {
        db.execute(
            "UPDATE time_entries SET notes=? WHERE id=?",
            rusqlite::params![n, id],
        )
        .map_err(|e| e.to_string())?;
    }
    db.query_row(
        "SELECT id, date, start_time, end_time, notes, created_at FROM time_entries WHERE id=?",
        [id],
        row_to_time_entry,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_time_entry(id: i64, state: State<DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM time_entries WHERE id=?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
