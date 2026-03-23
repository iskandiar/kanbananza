use crate::tray::{rebuild_menu, update_badge_and_icon};
use tauri::AppHandle;

#[tauri::command]
pub fn refresh_tray(app: AppHandle) {
    update_badge_and_icon(&app);
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = rebuild_menu(&app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
