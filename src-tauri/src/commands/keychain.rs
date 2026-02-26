use rusqlite::OptionalExtension;
use tauri::State;

use crate::db::DbState;

#[tauri::command]
pub fn store_secret(
    state: State<'_, DbState>,
    _service: String,
    key: String,
    value: String,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO secrets (key, value) VALUES (?, ?)",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_secret(
    state: State<'_, DbState>,
    _service: String,
    key: String,
) -> Result<Option<String>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.query_row(
        "SELECT value FROM secrets WHERE key = ?",
        rusqlite::params![key],
        |r| r.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_secret(
    state: State<'_, DbState>,
    _service: String,
    key: String,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM secrets WHERE key = ?", rusqlite::params![key])
        .map_err(|e| e.to_string())?;
    Ok(())
}
