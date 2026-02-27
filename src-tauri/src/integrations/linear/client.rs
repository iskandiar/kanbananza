use serde::Deserialize;

/// A Linear issue returned by the GraphQL API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearIssue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    /// 0 = No priority, 1 = Urgent, 2 = High, 3 = Medium, 4 = Low
    pub priority: i64,
    pub estimate: Option<f64>,
    pub url: String,
    pub state: LinearIssueState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearIssueState {
    pub name: String,
}

// ---------------------------------------------------------------------------
// Internal GraphQL response shapes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GqlResponse<T> {
    data: T,
}

#[derive(Deserialize)]
struct IssuesData {
    issues: IssuesNodes,
}

#[derive(Deserialize)]
struct IssuesNodes {
    nodes: Vec<LinearIssue>,
}

#[derive(Deserialize)]
struct SingleIssueData {
    issues: SingleIssueNodes,
}

#[derive(Deserialize)]
struct SingleIssueNodes {
    nodes: Vec<LinearIssue>,
}

const ENDPOINT: &str = "https://api.linear.app/graphql";

/// Fetches all "started" issues assigned to the authenticated user.
///
/// Linear API keys are passed directly in the `Authorization` header
/// (no "Bearer" prefix).
pub async fn get_viewer_issues(api_key: &str) -> Result<Vec<LinearIssue>, String> {
    let query = r#"
        query {
          issues(filter: {
            state: { type: { nin: ["completed", "cancelled"] } },
            assignee: { isMe: { eq: true } }
          }) {
            nodes {
              id
              identifier
              title
              description
              priority
              estimate
              url
              state { name }
            }
          }
        }
    "#;

    let body = serde_json::json!({ "query": query });

    let client = reqwest::Client::new();
    let resp = client
        .post(ENDPOINT)
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Linear API request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Linear API error {status}: {text}"));
    }

    let parsed = resp
        .json::<GqlResponse<IssuesData>>()
        .await
        .map_err(|e| format!("failed to parse Linear issues response: {e}"))?;

    Ok(parsed.data.issues.nodes)
}

/// Fetches a single Linear issue by its human-readable identifier (e.g. "ENG-42").
///
/// Returns `Err` if the issue is not found or the API call fails.
pub async fn get_issue_by_identifier(
    api_key: &str,
    identifier: &str,
) -> Result<LinearIssue, String> {
    let query = r#"
        query($identifier: String!) {
          issues(filter: { identifier: { eq: $identifier } }) {
            nodes {
              id
              identifier
              title
              description
              priority
              estimate
              url
              state { name }
            }
          }
        }
    "#;

    let body = serde_json::json!({
        "query": query,
        "variables": { "identifier": identifier }
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(ENDPOINT)
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Linear API request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Linear API error {status}: {text}"));
    }

    let parsed = resp
        .json::<GqlResponse<SingleIssueData>>()
        .await
        .map_err(|e| format!("failed to parse Linear issue response: {e}"))?;

    parsed
        .data
        .issues
        .nodes
        .into_iter()
        .next()
        .ok_or_else(|| format!("Linear issue '{identifier}' not found"))
}
