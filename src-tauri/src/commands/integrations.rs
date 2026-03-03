use crate::db::DbState;
use crate::integrations::calendar;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ---------------------------------------------------------------------------
// Shared event payload
// ---------------------------------------------------------------------------

/// Payload emitted with the `"calendar://synced"` event.
/// `error` is `Some(message)` when the sync failed, `None` on success.
#[derive(serde::Serialize, Clone)]
struct SyncResult {
    count: usize,
    error: Option<String>,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Starts the Google OAuth2 PKCE flow:
/// 1. Binds a loopback TCP listener on a random port (http://127.0.0.1:PORT)
/// 2. Opens the Google consent page in the system browser
/// 3. Awaits the redirect callback in the background
/// 4. Exchanges the code for tokens, stores them in the DB, then emits
///    "calendar://connected" and triggers an immediate sync.
///
/// Using loopback instead of a custom URL scheme (kanbananza://) because
/// Google's OAuth policy only allows loopback redirects for desktop apps.
#[tauri::command]
pub async fn get_calendar_auth_url(app: AppHandle) -> Result<(), String> {
    // 64 random bytes → base64url (no padding) → ~86-char verifier.
    let mut bytes = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);

    // Bind on port 0 — OS picks a free port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("failed to bind loopback listener: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| e.to_string())?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}");

    let auth_url = calendar::get_auth_url(&verifier, &redirect_uri);
    open::that(&auth_url).map_err(|e| format!("failed to open browser: {e}"))?;

    // Await the OAuth callback on the async runtime — returns immediately to
    // the caller so the UI is not blocked.
    tauri::async_runtime::spawn(async move {
        let result: Result<(), String> = async {
            let code = await_loopback_callback(listener).await?;

            // Step 1: HTTP token exchange — no DB lock held.
            let (access_token, refresh_token, expiry) =
                calendar::exchange_code_http(&code, &verifier, &redirect_uri).await?;

            // Step 2: persist tokens in DB.
            {
                let db_state = app.state::<DbState>();
                let db = db_state.0.lock().map_err(|e| e.to_string())?;
                calendar::store_tokens(&db, &access_token, &refresh_token, expiry)?;
            } // db lock released

            app.emit("calendar://connected", ())
                .map_err(|e| e.to_string())?;

            // Step 3: immediate sync for the next 10 days.
            let db_state = app.state::<DbState>();
            match calendar::sync_events(&db_state).await {
                Ok(count) => {
                    let _ = app.emit(
                        "calendar://synced",
                        SyncResult { count, error: None },
                    );
                }
                Err(e) => {
                    let _ = app.emit(
                        "calendar://synced",
                        SyncResult {
                            count: 0,
                            error: Some(e),
                        },
                    );
                }
            }

            Ok(())
        }
        .await;

        if let Err(e) = result {
            // Surface the error to the frontend so it can show a notification.
            let _ = app.emit(
                "calendar://error",
                serde_json::json!({ "message": e }),
            );
        }
    });

    Ok(())
}

/// Accepts one HTTP connection on the listener, parses `?code=` from the
/// request path, sends a "you can close this tab" response, and returns
/// the code string.
async fn await_loopback_callback(listener: tokio::net::TcpListener) -> Result<String, String> {
    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| format!("accept error: {e}"))?;

    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("read error: {e}"))?;

    let request = String::from_utf8_lossy(&buf[..n]);

    // First line: "GET /?code=...&... HTTP/1.1"
    let path = request
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .ok_or("malformed HTTP request")?;

    let code = url::Url::parse(&format!("http://localhost{path}"))
        .ok()
        .and_then(|u| {
            u.query_pairs()
                .find(|(k, _)| k == "code")
                .map(|(_, v)| v.into_owned())
        })
        .ok_or("no 'code' param in callback URL")?;

    let body = "<html><body style='font-family:sans-serif;text-align:center;padding:4rem'>\
        <h2>Connected to Google Calendar</h2>\
        <p>You can close this tab and return to Kanbananza.</p>\
        </body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;

    Ok(code)
}

/// Triggers an immediate calendar sync for the current (most-recent) week.
/// Emits `"calendar://synced"` with a [`SyncResult`] payload on completion,
/// including any error message so the frontend can display it.
#[tauri::command]
pub async fn sync_calendar(
    state: State<'_, DbState>,
    app: AppHandle,
) -> Result<(), String> {
    match calendar::sync_events(&state).await {
        Ok(count) => {
            app.emit("calendar://synced", SyncResult { count, error: None })
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            // Emit the error payload so the UI can surface it, then also
            // return it from the command so invoke() rejects on the JS side.
            let _ = app.emit(
                "calendar://synced",
                SyncResult {
                    count: 0,
                    error: Some(e.clone()),
                },
            );
            Err(e)
        }
    }
}

/// Disables the calendar integration and clears stored tokens from the DB.
#[tauri::command]
pub async fn disconnect_calendar(state: State<'_, DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    calendar::disconnect(&db)
}

/// Returns `true` if the user has a stored Google Calendar refresh token
/// in the DB (i.e. the OAuth flow has been completed at least once).
#[tauri::command]
pub async fn get_calendar_status(state: State<'_, DbState>) -> Result<bool, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    Ok(calendar::is_connected(&db))
}

// ---------------------------------------------------------------------------
// GitLab integration commands
// ---------------------------------------------------------------------------

use crate::integrations::gitlab;

/// Fetches open MRs (authored by user + assigned for review) from GitLab
/// and upserts them as `mr` cards in the local backlog.
/// Auto-marks merged/closed MRs as Done.
/// Emits `"gitlab://synced"` with { count, error } on completion.
#[tauri::command]
pub async fn sync_gitlab(
    state: State<'_, DbState>,
    app: AppHandle,
) -> Result<(), String> {
    match gitlab::sync_mrs(&state).await {
        Ok(count) => {
            app.emit("gitlab://synced", SyncResult { count, error: None })
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            let _ = app.emit(
                "gitlab://synced",
                SyncResult { count: 0, error: Some(e.clone()) },
            );
            Err(e)
        }
    }
}

/// Removes the GitLab PAT from local storage and clears the integrations row.
#[tauri::command]
pub async fn disconnect_gitlab(state: State<'_, DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM secrets WHERE key = 'gitlab_pat'", [])
        .map_err(|e| e.to_string())?;
    db.execute("DELETE FROM integrations WHERE id='gitlab'", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Linear integration commands
// ---------------------------------------------------------------------------

use crate::integrations::linear;

/// Pulls open Linear issues assigned to the user and upserts them as `task`
/// cards in the local backlog.
/// Emits `"linear://synced"` with { count, error } on completion.
#[tauri::command]
pub async fn sync_linear(
    state: State<'_, DbState>,
    app: AppHandle,
) -> Result<(), String> {
    match linear::sync_issues(&state).await {
        Ok(count) => {
            app.emit("linear://synced", SyncResult { count, error: None })
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            let _ = app.emit(
                "linear://synced",
                SyncResult { count: 0, error: Some(e.clone()) },
            );
            Err(e)
        }
    }
}

/// Removes the Linear API key from local storage and clears the integrations row.
#[tauri::command]
pub async fn disconnect_linear(state: State<'_, DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM secrets WHERE key = 'linear_api_key'", [])
        .map_err(|e| e.to_string())?;
    db.execute("DELETE FROM integrations WHERE id='linear'", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Universal URL-to-card command
// ---------------------------------------------------------------------------

use crate::integrations::{notion, slack};
use crate::types::Card;

/// Creates a card from a recognised integration URL.
///
/// Supported URL patterns (checked in order):
/// 1. `linear.app` + `/issue/` — fetches the Linear issue by identifier
/// 2. `notion.so` or `notion.com` — fetches the Notion page
/// 3. `*.slack.com/archives/` — fetches the Slack thread
///
/// Returns the upserted `Card` on success.
#[tauri::command]
pub async fn create_card_from_url(
    state: State<'_, DbState>,
    _app: AppHandle,
    url: String,
    week_id: Option<i64>,
    day_of_week: Option<i64>,
) -> Result<Card, String> {
    if url.contains("linear.app") && url.contains("/issue/") {
        // Extract the last non-empty path segment as the issue identifier.
        let identifier = url
            .split('?')
            .next()
            .unwrap_or(&url)
            .trim_end_matches('/')
            .rsplit('/')
            .find(|s| !s.is_empty())
            .ok_or_else(|| format!("could not extract Linear issue identifier from URL: {url}"))?
            .to_string();
        linear::create_single_issue_card(&state, &identifier, week_id, day_of_week).await
    } else if url.contains("/-/merge_requests/") {
        // GitLab MR URL: https://gitlab.com/<namespace>/<project>/-/merge_requests/<iid>
        gitlab::create_single_mr_card(&state, &url, week_id, day_of_week).await
    } else if url.contains("notion.so") || url.contains("notion.com") {
        notion::create_card_from_url(&state, url, week_id, day_of_week).await
    } else if url.contains(".slack.com/archives/") {
        slack::create_card_from_url(&state, url, week_id, day_of_week).await
    } else {
        Err("Unsupported URL type".to_string())
    }
}
