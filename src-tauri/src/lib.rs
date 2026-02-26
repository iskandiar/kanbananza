pub mod commands;
pub mod db;
pub mod integrations;
pub mod types;

use commands::{
    cards::*,
    integrations::{
        disconnect_calendar, get_calendar_auth_url, get_calendar_status, sync_calendar, PkceState,
    },
    keychain::*,
    rollover::*,
    settings::*,
    weeks::*,
};
use db::DbState;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // ---------------------------------------------------------------
            // Database setup
            // ---------------------------------------------------------------
            let data_dir = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("kanbananza.db");
            let conn = Connection::open(&db_path).expect("failed to open db");
            // WAL mode: writes survive crashes; readers never block writers
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
                .expect("failed to set WAL mode");
            conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
                .expect("failed to run migrations");
            app.manage(DbState(Mutex::new(conn)));

            // ---------------------------------------------------------------
            // PKCE state (shared between auth-URL generation and deep-link
            // callback)
            // ---------------------------------------------------------------
            app.manage(PkceState(Mutex::new(None)));

            // ---------------------------------------------------------------
            // Deep-link handler — receives kanbananza://auth?code=...
            // ---------------------------------------------------------------
            let deep_link_app = app.handle().clone();
            app.deep_link()
                .on_open_url(move |event: tauri_plugin_deep_link::OpenUrlEvent| {
                    for raw_url in event.urls() {
                        let url_str = raw_url.as_str();

                        // Parse the URL and extract the `code` query parameter.
                        let parsed = match url::Url::parse(url_str) {
                            Ok(u) => u,
                            Err(e) => {
                                eprintln!("[deep-link] failed to parse URL '{url_str}': {e}");
                                continue;
                            }
                        };

                        let code = parsed
                            .query_pairs()
                            .find(|(k, _)| k == "code")
                            .map(|(_, v)| v.into_owned());

                        let code = match code {
                            Some(c) => c,
                            None => {
                                eprintln!("[deep-link] no 'code' param in URL '{url_str}'");
                                continue;
                            }
                        };

                        // Retrieve and clear the stored PKCE verifier.
                        let pkce_state = deep_link_app.state::<PkceState>();
                        let verifier = {
                            let mut guard = match pkce_state.0.lock() {
                                Ok(g) => g,
                                Err(e) => {
                                    eprintln!("[deep-link] PkceState lock poisoned: {e}");
                                    continue;
                                }
                            };
                            guard.take()
                        };

                        let verifier = match verifier {
                            Some(v) => v,
                            None => {
                                eprintln!(
                                    "[deep-link] no PKCE verifier stored — ignoring callback"
                                );
                                continue;
                            }
                        };

                        // Perform token exchange + initial sync on the async
                        // runtime (deep-link callback is synchronous).
                        let handle = deep_link_app.clone();
                        tauri::async_runtime::spawn(async move {
                            match integrations::calendar::exchange_code(&code, &verifier).await {
                                Ok(()) => {
                                    let _ = handle.emit("calendar://connected", ());

                                    // Kick off an immediate sync for the current week.
                                    let db_state = handle.state::<DbState>();
                                    let week_info = {
                                        match db_state.0.lock() {
                                            Ok(guard) => guard
                                                .query_row(
                                                    "SELECT id, start_date FROM weeks \
                                                     ORDER BY year DESC, week_number DESC LIMIT 1",
                                                    [],
                                                    |r| {
                                                        Ok((
                                                            r.get::<_, i64>(0)?,
                                                            r.get::<_, String>(1)?,
                                                        ))
                                                    },
                                                )
                                                .ok(),
                                            Err(e) => {
                                                eprintln!("[deep-link] db lock error: {e}");
                                                None
                                            }
                                        }
                                        // guard dropped here — lock released before await
                                    };

                                    if let Some((week_id, start_date)) = week_info {
                                        match integrations::calendar::sync_events(
                                            &start_date,
                                            week_id,
                                            &db_state,
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                                let _ = handle.emit("calendar://synced", ());
                                            }
                                            Err(e) => {
                                                eprintln!("[deep-link] initial sync error: {e}");
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[deep-link] token exchange failed: {e}");
                                }
                            }
                        });
                    }
                });

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

                    if !integrations::calendar::is_connected() {
                        continue;
                    }

                    let db_state = poll_handle.state::<DbState>();

                    // Fetch the most recent week metadata with a short-lived lock.
                    let week_info = {
                        match db_state.0.lock() {
                            Ok(guard) => guard
                                .query_row(
                                    "SELECT id, start_date FROM weeks \
                                     ORDER BY year DESC, week_number DESC LIMIT 1",
                                    [],
                                    |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
                                )
                                .ok(),
                            Err(e) => {
                                eprintln!("[poll] db lock error: {e}");
                                None
                            }
                        }
                        // guard dropped here — lock released before await
                    };

                    if let Some((week_id, start_date)) = week_info {
                        match integrations::calendar::sync_events(
                            &start_date,
                            week_id,
                            &db_state,
                        )
                        .await
                        {
                            Ok(_) => {
                                let _ = poll_handle.emit("calendar://synced", ());
                            }
                            Err(e) => eprintln!("[poll] sync error: {e}"),
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
