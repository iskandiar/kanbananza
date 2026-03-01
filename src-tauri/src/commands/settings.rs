use crate::db::DbState;
use crate::types::Settings;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn get_settings(state: State<DbState>) -> Result<Settings, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    let settings = db
        .query_row(
            "SELECT id,available_hours,ai_provider,auto_ai FROM settings WHERE id=1",
            [],
            |row| Ok(Settings {
                id: row.get(0)?,
                available_hours: row.get(1)?,
                ai_provider: row.get(2)?,
                auto_ai: row.get::<_, i64>(3).map(|v| v != 0).unwrap_or(false),
            }),
        )
        .map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub fn update_settings(
    available_hours: Option<f64>,
    ai_provider: Option<String>,
    auto_ai: Option<bool>,
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
    if let Some(v) = auto_ai {
        db.execute("UPDATE settings SET auto_ai=? WHERE id=1", [v as i64])
            .map_err(|e| e.to_string())?;
    }
    let settings = db
        .query_row(
            "SELECT id,available_hours,ai_provider,auto_ai FROM settings WHERE id=1",
            [],
            |row| Ok(Settings {
                id: row.get(0)?,
                available_hours: row.get(1)?,
                ai_provider: row.get(2)?,
                auto_ai: row.get::<_, i64>(3).map(|v| v != 0).unwrap_or(false),
            }),
        )
        .map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub fn backup_database(state: State<DbState>, path: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let escaped = path.replace('\'', "''");
    conn.execute_batch(&format!("VACUUM INTO '{}'", escaped))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_database(state: State<DbState>, app: AppHandle, path: String) -> Result<(), String> {
    let _guard = state.0.lock().map_err(|e| e.to_string())?;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_path = data_dir.join("kanbananza.db");
    std::fs::copy(&path, &db_path).map_err(|e| e.to_string())?;
    app.restart();
    #[allow(unreachable_code)]
    Ok(())
}

#[tauri::command]
pub fn clear_all_data(state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute_batch(
        "PRAGMA foreign_keys=OFF;
        DELETE FROM cards;
        DELETE FROM projects;
        DELETE FROM weeks;
        PRAGMA foreign_keys=ON;",
    )
    .map_err(|e| e.to_string())
}
