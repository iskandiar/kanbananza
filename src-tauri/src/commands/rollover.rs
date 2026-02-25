use crate::db::DbState;
use tauri::State;

/// Moves all planned (non-done) cards from a week to the global backlog.
/// Returns the count of cards rolled over.
#[tauri::command]
pub fn rollover_week(week_id: i64, state: State<DbState>) -> Result<i64, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    let count = db
        .execute(
            "UPDATE cards SET week_id=NULL, day_of_week=NULL, updated_at=datetime('now') WHERE week_id=? AND status='planned'",
            [week_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(count as i64)
}
