use serde::Deserialize;

/// A merge request returned by the GitLab MRs API.
#[derive(Deserialize)]
pub struct GitLabMR {
    pub id: i64,
    pub iid: i64,
    pub title: String,
    pub web_url: String,
    pub state: String,
    pub project_id: i64,
    pub author: GitLabUser,
}

/// Minimal user object embedded in a GitLab MR response.
#[derive(Deserialize)]
pub struct GitLabUser {
    pub username: String,
}

/// The authenticated user returned by `GET /user`.
#[derive(Deserialize)]
pub struct GitLabCurrentUser {
    pub id: i64,
    pub username: String,
}

const BASE: &str = "https://gitlab.com/api/v4";

/// Fetches the currently authenticated GitLab user via `GET /user`.
///
/// Returns `Err` with the HTTP status and body on a non-2xx response, or a
/// network/parse error message otherwise.
pub async fn get_current_user(pat: &str) -> Result<GitLabCurrentUser, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{BASE}/user"))
        .header("PRIVATE-TOKEN", pat)
        .send()
        .await
        .map_err(|e| format!("GitLab /user fetch failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GitLab API error {status}: {body}"));
    }

    resp.json::<GitLabCurrentUser>()
        .await
        .map_err(|e| format!("failed to parse GitLab /user response: {e}"))
}

/// Fetches all open MRs created by the authenticated user
/// (`GET /merge_requests?state=opened&scope=created_by_me&per_page=100`).
pub async fn fetch_authored_mrs(pat: &str) -> Result<Vec<GitLabMR>, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{BASE}/merge_requests"))
        .header("PRIVATE-TOKEN", pat)
        .query(&[
            ("state", "opened"),
            ("scope", "created_by_me"),
            ("per_page", "100"),
        ])
        .send()
        .await
        .map_err(|e| format!("GitLab authored MRs fetch failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GitLab API error {status}: {body}"));
    }

    resp.json::<Vec<GitLabMR>>()
        .await
        .map_err(|e| format!("failed to parse GitLab authored MRs response: {e}"))
}

/// Fetches all open MRs where the authenticated user is a reviewer
/// (`GET /merge_requests?state=opened&reviewer_id={user_id}&per_page=100`).
pub async fn fetch_review_mrs(pat: &str, user_id: i64) -> Result<Vec<GitLabMR>, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{BASE}/merge_requests"))
        .header("PRIVATE-TOKEN", pat)
        .query(&[
            ("state", "opened"),
            ("reviewer_id", &user_id.to_string()),
            ("per_page", "100"),
        ])
        .send()
        .await
        .map_err(|e| format!("GitLab review MRs fetch failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GitLab API error {status}: {body}"));
    }

    resp.json::<Vec<GitLabMR>>()
        .await
        .map_err(|e| format!("failed to parse GitLab review MRs response: {e}"))
}
