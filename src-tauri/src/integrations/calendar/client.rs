use serde::Deserialize;

/// Represents a single event returned by the Google Calendar Events API.
#[derive(Deserialize)]
pub struct GCalEvent {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub start: GCalTime,
    pub end: GCalTime,
    #[serde(rename = "eventType", default)]
    pub event_type: Option<String>,
    pub attendees: Option<Vec<GCalAttendee>>,
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

#[derive(Deserialize)]
pub struct GCalAttendee {
    #[serde(rename = "self", default)]
    pub is_self: Option<bool>,
    #[serde(rename = "responseStatus", default)]
    pub response_status: Option<String>,
}

/// Wraps the `items` array in a Google Calendar list-events response.
#[derive(Deserialize)]
struct EventsResponse {
    #[serde(default)]
    items: Vec<GCalEvent>,
}

/// A calendar entry from the user's calendar list.
#[derive(Deserialize)]
pub struct GCalCalendar {
    pub id: String,
    pub summary: Option<String>,
    pub selected: Option<bool>,
    pub primary: Option<bool>,
    /// "owner", "writer", "reader", or "freeBusyReader"
    #[serde(rename = "accessRole")]
    pub access_role: Option<String>,
}

/// Wraps the `items` array in a Google Calendar calendarList response.
#[derive(Deserialize)]
struct CalendarListResponse {
    #[serde(default)]
    items: Vec<GCalCalendar>,
}

/// Fetches the user's calendar list and returns all calendars where the user
/// has at least reader access (i.e. `access_role != "freeBusyReader"`).
pub async fn list_calendars(access_token: &str) -> Result<Vec<GCalCalendar>, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://www.googleapis.com/calendar/v3/users/me/calendarList")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("calendarList fetch failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("calendarList API error {status}: {text}"));
    }

    let body: CalendarListResponse = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse calendarList response: {e}"))?;

    let accessible: Vec<GCalCalendar> = body
        .items
        .into_iter()
        .filter(|c| c.access_role.as_deref() != Some("freeBusyReader"))
        .collect();

    Ok(accessible)
}

/// Fetches timed (non-all-day) calendar events from the specified calendar
/// for the given time window.
///
/// `calendar_id` — the calendar identifier (e.g. `"primary"` or an email address).
/// `week_start` / `week_end` — RFC 3339 strings such as
/// `"2025-01-06T00:00:00Z"` and `"2025-01-10T23:59:59Z"`.
pub async fn fetch_events(
    access_token: &str,
    week_start: &str,
    week_end: &str,
    calendar_id: &str,
) -> Result<Vec<GCalEvent>, String> {
    let client = reqwest::Client::new();

    // URL-encode the calendar_id in case it contains special characters (e.g. '@').
    let encoded_id = url::form_urlencoded::byte_serialize(calendar_id.as_bytes())
        .collect::<String>();
    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{encoded_id}/events"
    );

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&[
            ("timeMin", week_start),
            ("timeMax", week_end),
            ("singleEvents", "true"),
            ("orderBy", "startTime"),
        ])
        .send()
        .await
        .map_err(|e| format!("calendar fetch failed for '{calendar_id}': {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!(
            "calendar API error {status} for '{calendar_id}': {text}"
        ));
    }

    let body: EventsResponse = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse calendar response for '{calendar_id}': {e}"))?;

    let timed: Vec<GCalEvent> = body
        .items
        .into_iter()
        // Only timed events (all-day have no dateTime)
        .filter(|e| e.start.date_time.is_some())
        // Skip non-default event types (focus time, OOO, working location)
        .filter(|e| {
            e.event_type.as_deref().map_or(true, |t| t == "default" || t == "fromGmail")
        })
        // Skip if user declined or hasn't responded (only when attendees list is present)
        .filter(|e| {
            let Some(attendees) = &e.attendees else { return true }; // solo event, keep
            if attendees.is_empty() { return true; }
            attendees.iter().any(|a| {
                a.is_self == Some(true)
                    && a.response_status.as_deref() == Some("accepted")
            })
        })
        // Skip by title keyword (case-insensitive)
        .filter(|e| {
            let title = e.summary.as_deref().unwrap_or("").to_lowercase();
            !["focus time", "out of office", "ooo", "lunch"]
                .iter()
                .any(|kw| title.contains(kw))
        })
        .collect();

    Ok(timed)
}
