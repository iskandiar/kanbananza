pub mod ai;
pub mod commands;
pub mod db;
pub mod error;
pub mod integrations;
pub mod tray;
pub mod types;
#[cfg(test)]
pub mod tests;

pub use error::AppError;

use commands::{
    ai::summarise_week,
    cards::*,
    integrations::{disconnect_calendar, disconnect_gitlab, get_calendar_auth_url, get_calendar_status, sync_calendar, sync_gitlab, sync_linear, disconnect_linear, create_card_from_url},
    keychain::*,
    projects::{
        list_projects, create_project, update_project, archive_project,
        list_cards_by_project, generate_project_slug, summarise_project,
    },
    rollover::*,
    settings::*,
    shell::open_url,
    sync_review::{
        fetch_calendar_preview, confirm_calendar_sync, skip_sync_item,
        fetch_linear_preview, confirm_linear_sync,
        fetch_gitlab_preview, confirm_gitlab_sync,
    },
    card_time_entries::{
        card_clock_in, card_clock_out, get_active_card_entry, list_card_time_entries,
        finalize_card_time, list_card_entries_for_week, list_day_entries_for_week,
    },
    time_entries::{clock_in, clock_out, list_time_entries, update_time_entry, delete_time_entry, create_manual_time_entry, list_time_entries_for_week},
    tray::refresh_tray,
    weeks::*,
};
use db::DbState;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // ---------------------------------------------------------------
            // Database setup
            // ---------------------------------------------------------------
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("kanbananza.db");
            let conn = Connection::open(&db_path)?;
            // WAL mode: writes survive crashes; readers never block writers
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
            conn.execute_batch(include_str!("../migrations/0001_initial.sql"))?;
            // 0002 adds a column via ALTER TABLE — ignore if already applied.
            if let Err(e) = conn.execute_batch(include_str!("../migrations/0002_auto_ai.sql")) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(format!("failed to run auto_ai migration: {e}").into());
                }
            }
            // 0003 adds projects table and new card columns — ignore if already applied.
            if let Err(e) = conn.execute_batch(include_str!("../migrations/0003_projects.sql")) {
                if !e.to_string().contains("already exists") {
                    return Err(format!("failed to run projects migration: {e}").into());
                }
            }
            let _ = conn.execute("ALTER TABLE cards ADD COLUMN project_id INTEGER REFERENCES projects(id)", []);
            let _ = conn.execute("ALTER TABLE cards ADD COLUMN done_at TEXT", []);
            // 0004 adds soft delete, sync_skip, and time_entries.
            if let Err(e) = conn.execute_batch(include_str!("../migrations/0004_soft_delete.sql")) {
                if !e.to_string().contains("already exists") {
                    return Err(format!("failed to run soft_delete migration: {e}").into());
                }
            }
            let _ = conn.execute("ALTER TABLE cards ADD COLUMN deleted_at TEXT", []);
            // 0005 adds card_time_entries table.
            if let Err(e) = conn.execute_batch(include_str!("../migrations/0005_card_time_tracking.sql")) {
                if !e.to_string().contains("already exists") {
                    return Err(format!("failed to run card_time_tracking migration: {e}").into());
                }
            }
            app.manage(DbState(Mutex::new(conn)));

            // Tray icon setup
            crate::tray::setup_tray(app.handle())?;

            // OAuth callback is handled via loopback TCP in get_calendar_auth_url —
            // no deep-link handler needed.

            // ---------------------------------------------------------------
            // Background polling — sync once per hour when connected
            // ---------------------------------------------------------------
            let poll_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tauri::async_runtime::spawn_blocking(|| {
                        std::thread::sleep(std::time::Duration::from_secs(3600));
                    })
                    .await
                    .ok();

                    let db_state = poll_handle.state::<DbState>();

                    // Check connection status with a short-lived lock.
                    let is_conn = {
                        match db_state.0.lock() {
                            Ok(db) => integrations::calendar::is_connected(&db),
                            Err(e) => {
                                log::error!(
                                    "[poll] db lock error checking connection: {e}"
                                );
                                false
                            }
                        }
                    }; // lock released before await

                    if !is_conn {
                        continue;
                    }

                    match integrations::calendar::sync_events(&db_state).await {
                        Ok(count) => {
                            let _ = poll_handle.emit(
                                "calendar://synced",
                                serde_json::json!({ "count": count, "error": null }),
                            );
                        }
                        Err(e) => {
                            log::error!("[poll] sync error: {e}");
                            let _ = poll_handle.emit(
                                "calendar://synced",
                                serde_json::json!({ "count": 0, "error": e }),
                            );
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Cards
            list_cards_by_week,
            create_card,
            update_card,
            delete_card,
            duplicate_card,
            search_cards,
            // Weeks
            get_or_create_week,
            get_week_by_date,
            list_weeks,
            update_week_summary,
            // Rollover
            rollover_week,
            // Settings
            get_settings,
            update_settings,
            // Keychain
            store_secret,
            get_secret,
            delete_secret,
            // Calendar integration
            get_calendar_auth_url,
            sync_calendar,
            disconnect_calendar,
            get_calendar_status,
            // GitLab integration
            sync_gitlab,
            disconnect_gitlab,
            // Linear integration
            sync_linear,
            disconnect_linear,
            // Universal URL-to-card
            create_card_from_url,
            // Projects
            list_projects,
            create_project,
            update_project,
            archive_project,
            list_cards_by_project,
            generate_project_slug,
            summarise_project,
            // AI
            summarise_week,
            // Shell
            open_url,
            // Data
            backup_database,
            restore_database,
            clear_all_data,
            // Time entries
            clock_in,
            clock_out,
            list_time_entries,
            update_time_entry,
            delete_time_entry,
            create_manual_time_entry,
            list_time_entries_for_week,
            // Card time entries
            card_clock_in,
            card_clock_out,
            get_active_card_entry,
            list_card_time_entries,
            finalize_card_time,
            list_card_entries_for_week,
            list_day_entries_for_week,
            // Sync review
            fetch_calendar_preview,
            confirm_calendar_sync,
            skip_sync_item,
            fetch_linear_preview,
            confirm_linear_sync,
            fetch_gitlab_preview,
            confirm_gitlab_sync,
            // Tray
            refresh_tray,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("fatal: {e}");
            std::process::exit(1);
        })
}
