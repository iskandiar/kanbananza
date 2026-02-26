use serde::Deserialize;

/// Represents a single event returned by the Google Calendar Events API.
#[derive(Deserialize)]
pub struct GCalEvent {
    pub id: String,
    pub summary: Option<String>,
    pub start: GCalTime,
    pub end: GCalTime,
}

/// The `start` or `end` field of a Google Calendar event.
/// `date_time` is present for timed events; `date` is present for all-day events.
#[derive(Deserialize)]
pub struct GCalTime {
    /// ISO 8601 timestamp — absent for all-day events.
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>,
    /// "YYYY-MM-DD" string — present only for all-day events.
    pub date: Option<String>,
}

/// Wraps the `items` array in a Google Calendar list-events response.
#[derive(Deserialize)]
struct EventsResponse {
    #[serde(default)]
    items: Vec<GCalEvent>,
}

/// Fetches timed (non-all-day) calendar events from the user's primary
/// calendar for the given time window.
///
/// `week_start` / `week_end` should be RFC 3339 strings such as
/// `"2025-01-06T00:00:00Z"` and `"2025-01-10T23:59:59Z"`.
pub async fn fetch_events(
    access_token: &str,
    week_start: &str,
    week_end: &str,
) -> Result<Vec<GCalEvent>, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://www.googleapis.com/calendar/v3/calendars/primary/events")
        .bearer_auth(access_token)
        .query(&[
            ("timeMin", week_start),
            ("timeMax", week_end),
            ("singleEvents", "true"),
            ("orderBy", "startTime"),
        ])
        .send()
        .await
        .map_err(|e| format!("calendar fetch failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("calendar API error {status}: {text}"));
    }

    let body: EventsResponse = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse calendar response: {e}"))?;

    // Only return timed events (skip all-day events which have no dateTime).
    let timed: Vec<GCalEvent> = body
        .items
        .into_iter()
        .filter(|e| e.start.date_time.is_some())
        .collect();

    Ok(timed)
}
