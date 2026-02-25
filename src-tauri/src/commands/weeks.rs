use crate::db::DbState;
use crate::types::Week;
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

#[tauri::command]
pub fn get_or_create_week(
    year: i64,
    week_number: i64,
    start_date: String,
    state: State<DbState>,
) -> Result<Week, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR IGNORE INTO weeks (year, week_number, start_date) VALUES (?, ?, ?)",
        rusqlite::params![year, week_number, start_date],
    )
    .map_err(|e| e.to_string())?;
    let week = db
        .query_row(
            "SELECT id,year,week_number,start_date,summary FROM weeks WHERE year=? AND week_number=?",
            rusqlite::params![year, week_number],
            row_to_week,
        )
        .map_err(|e| e.to_string())?;
    Ok(week)
}

#[tauri::command]
pub fn get_week_by_date(date: String, state: State<DbState>) -> Result<Option<Week>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
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
