use crate::db::DbState;
use crate::integrations::calendar;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use tauri::{AppHandle, Emitter, State};

/// Holds the PKCE code verifier between `get_calendar_auth_url` (which
/// generates it) and the deep-link handler in lib.rs that calls
/// `exchange_code`.
pub struct PkceState(pub std::sync::Mutex<Option<String>>);

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Generates a PKCE verifier, stores it in managed state, and opens the
/// Google OAuth 2.0 authorization URL in the system default browser.
#[tauri::command]
pub fn get_calendar_auth_url(pkce: State<PkceState>) -> Result<(), String> {
    // 64 random bytes → base64url (no padding) → ~86-char verifier.
    let mut bytes = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);

    *pkce.0.lock().map_err(|e| e.to_string())? = Some(verifier.clone());

    let url = calendar::get_auth_url(&verifier);
    open::that(&url).map_err(|e| format!("failed to open browser: {e}"))
}

/// Triggers an immediate calendar sync for the current (most-recent) week.
/// Emits `"calendar://synced"` on success.
#[tauri::command]
pub async fn sync_calendar(
    state: State<'_, DbState>,
    app: AppHandle,
) -> Result<(), String> {
    // Fetch the most recent week's metadata while holding the lock briefly.
    let (week_id, start_date) = {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT id, start_date FROM weeks ORDER BY year DESC, week_number DESC LIMIT 1",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
        )
        .map_err(|e| format!("no weeks found — open the board first: {e}"))?
    }; // lock released here, before any await

    // sync_events acquires the lock internally, after the async HTTP call.
    calendar::sync_events(&start_date, week_id, &state).await?;

    app.emit("calendar://synced", ())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Removes all Google Calendar credentials from the system keychain,
/// effectively disconnecting the integration.
#[tauri::command]
pub fn disconnect_calendar() -> Result<(), String> {
    calendar::disconnect();
    Ok(())
}

/// Returns `true` if the user has a stored Google Calendar refresh token
/// (i.e. the OAuth flow has been completed at least once).
#[tauri::command]
pub fn get_calendar_status() -> Result<bool, String> {
    Ok(calendar::is_connected())
}
