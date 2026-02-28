use rusqlite::Connection;
use rusqlite::OptionalExtension;
use tauri::State;

use crate::db::DbState;

// ---------------------------------------------------------------------------
// Pure-DB helpers
// ---------------------------------------------------------------------------

pub(crate) fn db_store_secret(db: &Connection, key: &str, value: &str) -> Result<(), String> {
    db.execute(
        "INSERT OR REPLACE INTO secrets (key, value) VALUES (?, ?)",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn db_get_secret(db: &Connection, key: &str) -> Result<Option<String>, String> {
    db.query_row(
        "SELECT value FROM secrets WHERE key = ?",
        rusqlite::params![key],
        |r| r.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub(crate) fn db_delete_secret(db: &Connection, key: &str) -> Result<(), String> {
    db.execute("DELETE FROM secrets WHERE key = ?", rusqlite::params![key])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn store_secret(
    state: State<'_, DbState>,
    _service: String,
    key: String,
    value: String,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_store_secret(&db, &key, &value)
}

#[tauri::command]
pub fn get_secret(
    state: State<'_, DbState>,
    _service: String,
    key: String,
) -> Result<Option<String>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_get_secret(&db, &key)
}

#[tauri::command]
pub fn delete_secret(
    state: State<'_, DbState>,
    _service: String,
    key: String,
) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_delete_secret(&db, &key)
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

    // store_secret then get_secret must return the stored value.
    #[test]
    fn store_then_get_round_trip() {
        let db = open_test_db();
        db_store_secret(&db, "openai_api_key", "sk-test-value").unwrap();
        let val = db_get_secret(&db, "openai_api_key").unwrap();
        assert_eq!(val, Some("sk-test-value".to_string()));
    }

    // A second store_secret with the same key must overwrite (upsert).
    #[test]
    fn second_store_overwrites_previous_value() {
        let db = open_test_db();
        db_store_secret(&db, "openai_api_key", "sk-first").unwrap();
        db_store_secret(&db, "openai_api_key", "sk-second").unwrap();
        let val = db_get_secret(&db, "openai_api_key").unwrap();
        assert_eq!(val, Some("sk-second".to_string()), "second write must win");
    }

    // After delete_secret, get_secret must return None.
    #[test]
    fn delete_secret_then_get_returns_none() {
        let db = open_test_db();
        db_store_secret(&db, "notion_api_key", "secret-xyz").unwrap();
        db_delete_secret(&db, "notion_api_key").unwrap();
        let val = db_get_secret(&db, "notion_api_key").unwrap();
        assert!(val.is_none(), "get_secret must return None after deletion");
    }

    // get_secret for a key that was never stored must return None (not an error).
    #[test]
    fn get_nonexistent_secret_returns_none() {
        let db = open_test_db();
        let val = db_get_secret(&db, "does_not_exist").unwrap();
        assert!(val.is_none());
    }
}
