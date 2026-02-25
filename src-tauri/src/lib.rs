pub mod commands;
pub mod db;
pub mod types;

use commands::{cards::*, keychain::*, rollover::*, settings::*, weeks::*};
use db::DbState;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("kanbananza.db");
            let conn = Connection::open(&db_path).expect("failed to open db");
            conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
                .expect("failed to run migrations");
            app.manage(DbState(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_cards_by_week,
            create_card,
            update_card,
            delete_card,
            get_or_create_week,
            get_week_by_date,
            list_weeks,
            update_week_summary,
            rollover_week,
            get_settings,
            update_settings,
            store_secret,
            get_secret,
            delete_secret,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
