use crate::db::DbState;
use crate::types::Settings;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<DbState>) -> Result<Settings, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    let settings = db
        .query_row(
            "SELECT id,available_hours,ai_provider FROM settings WHERE id=1",
            [],
            |row| Ok(Settings { id: row.get(0)?, available_hours: row.get(1)?, ai_provider: row.get(2)? }),
        )
        .map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub fn update_settings(
    available_hours: Option<f64>,
    ai_provider: Option<String>,
    state: State<DbState>,
) -> Result<Settings, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(h) = available_hours {
        db.execute("UPDATE settings SET available_hours=? WHERE id=1", [h])
            .map_err(|e| e.to_string())?;
    }
    if let Some(p) = ai_provider {
        db.execute("UPDATE settings SET ai_provider=? WHERE id=1", [p])
            .map_err(|e| e.to_string())?;
    }
    let settings = db
        .query_row(
            "SELECT id,available_hours,ai_provider FROM settings WHERE id=1",
            [],
            |row| Ok(Settings { id: row.get(0)?, available_hours: row.get(1)?, ai_provider: row.get(2)? }),
        )
        .map_err(|e| e.to_string())?;
    Ok(settings)
}
