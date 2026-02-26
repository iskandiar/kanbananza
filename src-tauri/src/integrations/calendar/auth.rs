use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::DbState;

const CLIENT_ID: &str = env!("GCAL_CLIENT_ID");
const CLIENT_SECRET: &str = env!("GCAL_CLIENT_SECRET");
const SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";

// ---------------------------------------------------------------------------
// PKCE helpers
// ---------------------------------------------------------------------------

/// Computes the PKCE `code_challenge` from a plain-text `verifier`.
/// Uses S256 method: base64url_nopad(SHA-256(verifier)).
fn pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let digest = hasher.finalize();
    URL_SAFE_NO_PAD.encode(digest)
}

// ---------------------------------------------------------------------------
// SQLite token storage helpers (private)
// ---------------------------------------------------------------------------

/// JSON shape stored in `integrations.config` for id='calendar'.
#[derive(serde::Serialize, serde::Deserialize)]
struct TokenConfig {
    access_token: String,
    refresh_token: String,
    token_expiry: u64,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Builds the full Google OAuth 2.0 authorization URL for this integration.
///
/// The caller is responsible for generating and persisting the `verifier`
/// (64 random bytes encoded as base64url) between this call and
/// [`exchange_code_http`] + [`store_tokens`].
pub fn get_auth_url(verifier: &str, redirect_uri: &str) -> String {
    let challenge = pkce_challenge(verifier);
    url::form_urlencoded::Serializer::new(format!("{AUTH_ENDPOINT}?"))
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", SCOPE)
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", &challenge)
        .finish()
}

/// Exchanges an authorization `code` for tokens via HTTP only — no DB access.
/// Returns `(access_token, refresh_token, expiry_unix_secs)`.
///
/// The caller must subsequently call [`store_tokens`] with the DB lock held
/// to persist the tokens. Separating HTTP from DB means we never hold a
/// `MutexGuard<Connection>` across an `.await`.
pub async fn exchange_code_http(
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<(String, String, u64), String> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("code", code),
        ("code_verifier", verifier),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];

    let resp = client
        .post(TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("token request failed: {e}"))?;

    let status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse token response: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "token exchange error {status}: {}",
            body.get("error_description")
                .or_else(|| body.get("error"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
        ));
    }

    let access_token = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("missing access_token in response")?
        .to_string();

    let refresh_token = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or("missing refresh_token in response")?
        .to_string();

    let expires_in = body
        .get("expires_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(3600);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let expiry = now + expires_in;

    Ok((access_token, refresh_token, expiry))
}

/// Stores all three token values atomically in the `integrations` table.
/// Sets `enabled=1` so that `is_connected` returns true.
pub fn store_tokens(
    db: &Connection,
    access_token: &str,
    refresh_token: &str,
    expiry: u64,
) -> Result<(), String> {
    let config = serde_json::to_string(&TokenConfig {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.to_string(),
        token_expiry: expiry,
    })
    .map_err(|e| e.to_string())?;

    db.execute(
        "INSERT OR REPLACE INTO integrations (id, enabled, config) VALUES ('calendar', 1, ?)",
        rusqlite::params![config],
    )
    .map_err(|e| format!("failed to store calendar tokens: {e}"))?;

    Ok(())
}

/// Reads all three token values from the `integrations` table.
/// Returns `None` if not connected (no row, disabled, or missing/invalid JSON).
pub fn read_tokens(db: &Connection) -> Option<(String, String, u64)> {
    let config_json: String = db
        .query_row(
            "SELECT config FROM integrations WHERE id='calendar' AND enabled=1",
            [],
            |r| r.get(0),
        )
        .ok()?;

    let cfg: TokenConfig = serde_json::from_str(&config_json).ok()?;
    Some((cfg.access_token, cfg.refresh_token, cfg.token_expiry))
}

/// Updates only the `access_token` and `token_expiry` in the stored config,
/// preserving the `refresh_token`.  Used after a token refresh.
pub fn update_access_token(db: &Connection, access_token: &str, expiry: u64) -> Result<(), String> {
    let config_json: String = db
        .query_row(
            "SELECT config FROM integrations WHERE id='calendar' AND enabled=1",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("no calendar token row to update: {e}"))?;

    let mut cfg: TokenConfig =
        serde_json::from_str(&config_json).map_err(|e| format!("corrupt token config: {e}"))?;

    cfg.access_token = access_token.to_string();
    cfg.token_expiry = expiry;

    let new_json = serde_json::to_string(&cfg).map_err(|e| e.to_string())?;

    db.execute(
        "UPDATE integrations SET config=? WHERE id='calendar'",
        rusqlite::params![new_json],
    )
    .map_err(|e| format!("failed to update access token: {e}"))?;

    Ok(())
}

/// Returns `true` if the user has a stored, enabled Google Calendar refresh
/// token (i.e. the OAuth flow has been completed at least once).
pub fn is_connected(db: &Connection) -> bool {
    match read_tokens(db) {
        Some((_, refresh, _)) => !refresh.is_empty(),
        None => false,
    }
}

/// Disables the calendar integration and clears stored tokens.
pub fn disconnect(db: &Connection) -> Result<(), String> {
    db.execute(
        "UPDATE integrations SET enabled=0, config=NULL WHERE id='calendar'",
        [],
    )
    .map_err(|e| format!("failed to disconnect calendar: {e}"))?;
    Ok(())
}

/// Returns a valid access token, refreshing it transparently if it is about
/// to expire (within 60 seconds).
///
/// Acquires the DB lock only briefly to read/write tokens; the HTTP refresh
/// call is made without holding the lock.
pub async fn get_valid_token(db_state: &DbState) -> Result<String, String> {
    // Phase 1: read tokens under a short-lived lock.
    let (access_token, refresh_token, expiry) = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        read_tokens(&db)
            .ok_or("no calendar tokens stored — please connect Google Calendar")?
    }; // lock released here

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    if expiry > now + 60 {
        return Ok(access_token);
    }

    // Phase 2: HTTP refresh — no lock held.
    let (new_access_token, new_expiry) = do_refresh_http(&refresh_token).await?;

    // Phase 3: persist updated tokens under a new short-lived lock.
    {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        update_access_token(&db, &new_access_token, new_expiry)?;
    } // lock released

    Ok(new_access_token)
}

/// Performs the HTTP token-refresh call only.  Returns `(new_access_token, new_expiry_unix)`.
/// Does not touch the database.
async fn do_refresh_http(refresh_token: &str) -> Result<(String, u64), String> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let resp = client
        .post(TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("refresh request failed: {e}"))?;

    let status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse refresh response: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "token refresh error {status}: {}",
            body.get("error_description")
                .or_else(|| body.get("error"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
        ));
    }

    let access_token = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("missing access_token in refresh response")?
        .to_string();

    let expires_in = body
        .get("expires_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(3600);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    Ok((access_token, now + expires_in))
}
