use serde::Deserialize;

const BASE: &str = "https://slack.com/api";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A Slack thread with its most relevant messages.
pub struct SlackThread {
    /// Total number of replies (does not include the root message).
    pub reply_count: usize,
    /// Full text of the root message (`messages[0]`).
    pub first_message: String,
    /// Last 3 messages joined by `"\n---\n"`.
    pub thread_preview: String,
}

// ---------------------------------------------------------------------------
// Internal deserialization types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ConversationsInfoResponse {
    ok: bool,
    error: Option<String>,
    channel: Option<ChannelInfo>,
}

#[derive(Deserialize)]
struct ChannelInfo {
    name: String,
}

#[derive(Deserialize)]
struct ConversationsRepliesResponse {
    ok: bool,
    error: Option<String>,
    messages: Option<Vec<SlackMessage>>,
}

#[derive(Deserialize)]
struct SlackMessage {
    #[serde(default)]
    text: String,
    #[serde(default)]
    reply_count: Option<usize>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns the human-readable name of a Slack channel via
/// `GET /conversations.info?channel={channel_id}`.
pub async fn get_channel_info(token: &str, channel_id: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{BASE}/conversations.info"))
        .header("Authorization", format!("Bearer {token}"))
        .query(&[("channel", channel_id)])
        .send()
        .await
        .map_err(|e| format!("Slack conversations.info fetch failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Slack API HTTP error {status}: {body}"));
    }

    let body: ConversationsInfoResponse = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse Slack conversations.info response: {e}"))?;

    if !body.ok {
        let err = body.error.unwrap_or_else(|| "unknown error".to_string());
        return Err(format!("Slack conversations.info error: {err}"));
    }

    let name = body
        .channel
        .map(|c| c.name)
        .ok_or_else(|| "Slack conversations.info: missing channel object".to_string())?;

    Ok(name)
}

/// Fetches thread messages via
/// `GET /conversations.replies?channel={channel_id}&ts={thread_ts}`.
///
/// Returns a [`SlackThread`] containing the reply count, the first message
/// text, and a preview of the last 3 messages.
pub async fn get_thread_replies(
    token: &str,
    channel_id: &str,
    thread_ts: &str,
) -> Result<SlackThread, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{BASE}/conversations.replies"))
        .header("Authorization", format!("Bearer {token}"))
        .query(&[("channel", channel_id), ("ts", thread_ts)])
        .send()
        .await
        .map_err(|e| format!("Slack conversations.replies fetch failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Slack API HTTP error {status}: {body}"));
    }

    let body: ConversationsRepliesResponse = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse Slack conversations.replies response: {e}"))?;

    if !body.ok {
        let err = body.error.unwrap_or_else(|| "unknown error".to_string());
        return Err(format!("Slack conversations.replies error: {err}"));
    }

    let messages = body.messages.unwrap_or_default();

    // The first element is the root (parent) message; subsequent ones are replies.
    let first_message = messages
        .first()
        .map(|m| m.text.clone())
        .unwrap_or_default();

    // reply_count is carried on the root message object.
    let reply_count = messages
        .first()
        .and_then(|m| m.reply_count)
        .unwrap_or_else(|| messages.len().saturating_sub(1));

    // Thread preview = last 3 messages (excluding root) joined by separator.
    let replies: Vec<&SlackMessage> = messages.iter().skip(1).collect();
    let last_three: Vec<&str> = replies
        .iter()
        .rev()
        .take(3)
        .rev()
        .map(|m| m.text.as_str())
        .collect();
    let thread_preview = last_three.join("\n---\n");

    Ok(SlackThread {
        reply_count,
        first_message,
        thread_preview,
    })
}
