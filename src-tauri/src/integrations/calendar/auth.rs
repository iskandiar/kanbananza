use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use keyring::Entry;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const SERVICE: &str = "kanbananza";
const CLIENT_ID: &str = env!("GCAL_CLIENT_ID");
const CLIENT_SECRET: &str = env!("GCAL_CLIENT_SECRET");
const REDIRECT_URI: &str = "kanbananza://auth";
const SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";

// --- keychain key constants ---
const KEY_ACCESS_TOKEN: &str = "calendar_access_token";
const KEY_REFRESH_TOKEN: &str = "calendar_refresh_token";
const KEY_TOKEN_EXPIRY: &str = "calendar_token_expiry";

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
// Public API
// ---------------------------------------------------------------------------

/// Builds the full Google OAuth 2.0 authorization URL for this integration.
///
/// The caller is responsible for generating and persisting the `verifier`
/// (64 random bytes encoded as base64url) between this call and
/// [`exchange_code`].
pub fn get_auth_url(verifier: &str) -> String {
    let challenge = pkce_challenge(verifier);
    url::form_urlencoded::Serializer::new(format!("{AUTH_ENDPOINT}?"))
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("response_type", "code")
        .append_pair("scope", SCOPE)
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", &challenge)
        .finish()
}

/// Exchanges an authorization `code` for access + refresh tokens and stores
/// them in the system keychain.
pub async fn exchange_code(code: &str, verifier: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("code", code),
        ("code_verifier", verifier),
        ("grant_type", "authorization_code"),
        ("redirect_uri", REDIRECT_URI),
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
        .ok_or("missing access_token in response")?;

    let refresh_token = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or("missing refresh_token in response")?;

    let expires_in = body
        .get("expires_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(3600);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let expiry = (now + expires_in).to_string();

    store_keychain(KEY_ACCESS_TOKEN, access_token)?;
    store_keychain(KEY_REFRESH_TOKEN, refresh_token)?;
    store_keychain(KEY_TOKEN_EXPIRY, &expiry)?;

    Ok(())
}

/// Returns a valid access token, refreshing it transparently if it is about
/// to expire (within 60 seconds).
pub async fn get_valid_token() -> Result<String, String> {
    let access_token = read_keychain(KEY_ACCESS_TOKEN)
        .ok_or("no access token stored — please connect Google Calendar")?;

    let expiry_str = read_keychain(KEY_TOKEN_EXPIRY).unwrap_or_default();
    let expiry: u64 = expiry_str.parse().unwrap_or(0);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    if expiry > now + 60 {
        return Ok(access_token);
    }

    // Token expired or about to expire — refresh.
    refresh_token().await
}

/// Uses the stored refresh token to obtain a new access token and updates the
/// keychain entries.  Returns the new access token.
pub async fn refresh_token() -> Result<String, String> {
    let refresh = read_keychain(KEY_REFRESH_TOKEN)
        .ok_or("no refresh token stored — please reconnect Google Calendar")?;

    let client = reqwest::Client::new();
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh.as_str()),
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
        .ok_or("missing access_token in refresh response")?;

    let expires_in = body
        .get("expires_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(3600);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let expiry = (now + expires_in).to_string();

    store_keychain(KEY_ACCESS_TOKEN, access_token)?;
    store_keychain(KEY_TOKEN_EXPIRY, &expiry)?;

    Ok(access_token.to_string())
}

/// Removes all calendar credentials from the system keychain.
/// Errors are silently ignored so disconnect is always a no-op for missing entries.
pub fn disconnect() {
    for key in [KEY_ACCESS_TOKEN, KEY_REFRESH_TOKEN, KEY_TOKEN_EXPIRY] {
        if let Ok(entry) = Entry::new(SERVICE, key) {
            let _ = entry.delete_credential();
        }
    }
}

/// Returns `true` if a refresh token is present in the keychain (i.e. the
/// user has previously completed the OAuth flow).
pub fn is_connected() -> bool {
    read_keychain(KEY_REFRESH_TOKEN)
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Keychain helpers (private)
// ---------------------------------------------------------------------------

fn store_keychain(key: &str, value: &str) -> Result<(), String> {
    Entry::new(SERVICE, key)
        .map_err(|e| e.to_string())?
        .set_password(value)
        .map_err(|e| e.to_string())
}

fn read_keychain(key: &str) -> Option<String> {
    Entry::new(SERVICE, key)
        .ok()?
        .get_password()
        .ok()
        .filter(|v| !v.is_empty())
}
