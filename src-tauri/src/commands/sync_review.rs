use chrono::{Duration, Utc};
use rusqlite::OptionalExtension;
use tauri::State;

use crate::db::DbState;
use crate::integrations::calendar::auth::get_valid_token;
use crate::integrations::calendar::client::{fetch_events, list_calendars};
use crate::integrations::linear::client::get_viewer_issues;
use crate::integrations::gitlab::client::{
    fetch_authored_mrs, fetch_review_mrs, fetch_mr_lines_changed, get_current_user,
};

// ---------------------------------------------------------------------------
// Shared preview type
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CardPreview {
    pub external_id: String,
    pub title: String,
    pub card_type: String,
    pub date: String,
    pub start_time: Option<String>,
    pub source: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true when a non-deleted card already exists for this external_id.
fn card_exists(db: &rusqlite::Connection, external_id: &str) -> Result<bool, String> {
    let id: Option<i64> = db
        .query_row(
            "SELECT id FROM cards WHERE external_id=? AND deleted_at IS NULL",
            rusqlite::params![external_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(id.is_some())
}

/// Returns true when this external_id has been added to sync_skip.
fn is_skipped(db: &rusqlite::Connection, external_id: &str) -> Result<bool, String> {
    let id: Option<i64> = db
        .query_row(
            "SELECT id FROM sync_skip WHERE external_id=?",
            rusqlite::params![external_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(id.is_some())
}

/// Computes time_min / time_max strings for "today" or "tomorrow" (UTC).
fn date_range_bounds(date_range: &str) -> Result<(String, String, String), String> {
    let today = Utc::now().date_naive();
    let date = match date_range {
        "today" => today,
        "tomorrow" => today + Duration::days(1),
        other => return Err(format!("unknown date_range '{other}': expected 'today' or 'tomorrow'")),
    };
    let date_str = date.format("%Y-%m-%d").to_string();
    let time_min = format!("{date_str}T00:00:00Z");
    let time_max = format!("{date_str}T23:59:59Z");
    Ok((date_str, time_min, time_max))
}

// ---------------------------------------------------------------------------
// Calendar preview / confirm
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn fetch_calendar_preview(
    date_range: String,
    state: State<'_, DbState>,
) -> Result<Vec<CardPreview>, String> {
    // Phase 1: get valid OAuth token (acquires and releases DB lock internally).
    let access_token = get_valid_token(&state).await?;

    let (date_str, time_min, time_max) = date_range_bounds(&date_range)?;

    // Phase 2: fetch calendar list — no DB lock held.
    let calendars = list_calendars(&access_token).await?;

    let mut previews: Vec<CardPreview> = Vec::new();

    for calendar in &calendars {
        // Phase 3: fetch events for this calendar — no DB lock held.
        let events = match fetch_events(&access_token, &time_min, &time_max, &calendar.id).await {
            Ok(evts) => evts,
            Err(e) => {
                log::warn!("[fetch_calendar_preview] failed for '{}': {e}", calendar.id);
                continue;
            }
        };

        // Phase 4: DB check — acquire lock, do all checks, release immediately.
        {
            let db = state.0.lock().map_err(|e| e.to_string())?;
            for event in events {
                let external_id = event.id.clone();

                if card_exists(&db, &external_id)? {
                    continue;
                }
                if is_skipped(&db, &external_id)? {
                    continue;
                }

                // Derive the date from the event's start time when available.
                let event_date = event
                    .start
                    .date_time
                    .as_deref()
                    .and_then(|dt| dt.get(..10))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| date_str.clone());

                previews.push(CardPreview {
                    external_id,
                    title: event.summary.unwrap_or_else(|| "(no title)".to_string()),
                    card_type: "meeting".to_string(),
                    date: event_date,
                    start_time: event.start.date_time,
                    source: "calendar".to_string(),
                });
            }
        } // DB lock released before next HTTP call.
    }

    Ok(previews)
}

#[tauri::command]
pub async fn confirm_calendar_sync(
    external_ids: Vec<String>,
    date_range: String,
    state: State<'_, DbState>,
) -> Result<usize, String> {
    if external_ids.is_empty() {
        return Ok(0);
    }

    // Phase 1: get valid OAuth token.
    let access_token = get_valid_token(&state).await?;

    let (_date_str, time_min, time_max) = date_range_bounds(&date_range)?;

    // Phase 2: fetch calendar list — no DB lock held.
    let calendars = list_calendars(&access_token).await?;

    let mut total_count = 0usize;
    let mut card_ids: Vec<i64> = Vec::new();

    for calendar in &calendars {
        // Phase 3: fetch events — no DB lock held.
        let events = match fetch_events(&access_token, &time_min, &time_max, &calendar.id).await {
            Ok(evts) => evts,
            Err(e) => {
                log::warn!("[confirm_calendar_sync] failed for '{}': {e}", calendar.id);
                continue;
            }
        };

        // Phase 4: upsert only the selected events.
        {
            let db = state.0.lock().map_err(|e| e.to_string())?;
            for event in events {
                if !external_ids.contains(&event.id) {
                    continue;
                }

                // Delegate to the calendar mapper's upsert logic by reproducing
                // the slim insert path here (mapper::upsert_event is private).
                if let Some(id) = upsert_calendar_event(&event, &db)? {
                    total_count += 1;
                    card_ids.push(id);
                }
            }
        } // DB lock released before next HTTP call.
    }

    // Phase 5: AI evaluation — no DB lock held during await.
    for card_id in card_ids {
        if let Err(e) = crate::ai::evaluate_card(card_id, &state).await {
            log::warn!("[confirm_calendar_sync] AI eval failed for card {card_id}: {e}");
        }
    }

    Ok(total_count)
}

/// Slim upsert for a single calendar event, mirroring the logic in
/// `integrations::calendar::mapper::upsert_event` but without the
/// `reorder_calendar_events` step (which is done by the full sync).
fn upsert_calendar_event(
    event: &crate::integrations::calendar::client::GCalEvent,
    db: &rusqlite::Connection,
) -> Result<Option<i64>, String> {
    use chrono::{DateTime, Datelike, Utc};

    let start_iso = match &event.start.date_time {
        Some(s) => s,
        None => return Ok(None),
    };

    let start: DateTime<Utc> = start_iso
        .parse()
        .map_err(|e| format!("failed to parse event start time '{start_iso}': {e}"))?;

    let day_of_week = start.weekday().number_from_monday() as i64;
    if day_of_week > 5 {
        return Ok(None);
    }

    let week_id = get_or_create_week(db, start.date_naive())?;

    let title = event.summary.as_deref().unwrap_or("(no title)");
    let metadata = serde_json::json!({
        "start_time": event.start.date_time,
        "end_time": event.end.date_time,
        "description": event.description,
    })
    .to_string();

    let time_estimate: Option<f64> = event.end.date_time.as_deref().and_then(|end_iso| {
        let end: DateTime<Utc> = end_iso.parse().ok()?;
        let mins = (end - start).num_minutes();
        if mins > 0 { Some(mins as f64 / 60.0) } else { None }
    });

    let existing: Option<(i64, Option<String>, Option<String>)> = db
        .query_row(
            "SELECT id, metadata, deleted_at FROM cards \
             WHERE external_id = ? AND source = 'calendar'",
            rusqlite::params![event.id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some((card_id, existing_meta_str, deleted_at)) = existing {
        if deleted_at.is_some() {
            return Ok(None);
        }
        let mut new_meta = serde_json::json!({
            "start_time": event.start.date_time,
            "end_time": event.end.date_time,
            "description": event.description,
        });
        if let Some(s) = existing_meta_str {
            if let Ok(existing_val) = serde_json::from_str::<serde_json::Value>(&s) {
                if let (Some(new_obj), Some(existing_obj)) =
                    (new_meta.as_object_mut(), existing_val.as_object())
                {
                    for key in ["ai_title", "ai_description", "ai_impact", "ai_hours"] {
                        if let Some(v) = existing_obj.get(key) {
                            new_obj.insert(key.to_string(), v.clone());
                        }
                    }
                }
            }
        }
        let update_metadata = new_meta.to_string();
        db.execute(
            "UPDATE cards SET title=?, metadata=?, day_of_week=?, week_id=?, \
             time_estimate=?, updated_at=datetime('now') WHERE id=?",
            rusqlite::params![title, update_metadata, day_of_week, week_id, time_estimate, card_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(Some(card_id))
    } else {
        db.execute(
            "INSERT INTO cards \
             (title, card_type, status, source, external_id, metadata, week_id, day_of_week, \
              position, time_estimate) \
             VALUES (?, 'meeting', 'planned', 'calendar', ?, ?, ?, ?, 0, ?)",
            rusqlite::params![title, event.id, metadata, week_id, day_of_week, time_estimate],
        )
        .map_err(|e| e.to_string())?;
        Ok(Some(db.last_insert_rowid()))
    }
}

/// Finds or creates the `weeks` row for the ISO week containing `date`.
/// Returns the `weeks.id`. Mirrors the private helper in calendar/mapper.rs.
fn get_or_create_week(db: &rusqlite::Connection, date: chrono::NaiveDate) -> Result<i64, String> {
    use chrono::Datelike;

    let iso = date.iso_week();
    let year = iso.year() as i64;
    let week_number = iso.week() as i64;

    let days_from_monday = (date.weekday().number_from_monday() - 1) as i64;
    let monday = date - Duration::days(days_from_monday);
    let start_date = monday.format("%Y-%m-%d").to_string();

    let existing: Option<i64> = db
        .query_row(
            "SELECT id FROM weeks WHERE year=? AND week_number=?",
            rusqlite::params![year, week_number],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(id) = existing {
        return Ok(id);
    }

    db.execute(
        "INSERT INTO weeks (year, week_number, start_date) VALUES (?, ?, ?)",
        rusqlite::params![year, week_number, start_date],
    )
    .map_err(|e| format!("failed to create week row: {e}"))?;

    Ok(db.last_insert_rowid())
}

// ---------------------------------------------------------------------------
// skip_sync_item
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn skip_sync_item(external_id: String, state: State<DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO sync_skip (external_id, skipped_at) VALUES (?, datetime('now'))",
        rusqlite::params![external_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Linear preview / confirm
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn fetch_linear_preview(
    state: State<'_, DbState>,
) -> Result<Vec<CardPreview>, String> {
    // Phase 1: read the Linear API key — acquire and immediately release lock.
    let api_key = {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT value FROM secrets WHERE key='linear_api_key'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or("Linear API key not configured")?
    }; // DB lock released

    // Phase 2: fetch issues — no DB lock held.
    let issues = get_viewer_issues(&api_key).await?;

    // Phase 3: filter to issues not already in DB and not skipped.
    let mut previews: Vec<CardPreview> = Vec::new();
    {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        for issue in &issues {
            let external_id = format!("linear:{}", issue.identifier);

            if card_exists(&db, &external_id)? {
                continue;
            }
            if is_skipped(&db, &external_id)? {
                continue;
            }

            previews.push(CardPreview {
                external_id,
                title: issue.title.clone(),
                card_type: "task".to_string(),
                date: Utc::now().format("%Y-%m-%d").to_string(),
                start_time: None,
                source: "linear".to_string(),
            });
        }
    } // DB lock released

    Ok(previews)
}

#[tauri::command]
pub async fn confirm_linear_sync(
    external_ids: Vec<String>,
    state: State<'_, DbState>,
) -> Result<usize, String> {
    if external_ids.is_empty() {
        return Ok(0);
    }

    // Phase 1: read the Linear API key.
    let api_key = {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT value FROM secrets WHERE key='linear_api_key'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or("Linear API key not configured")?
    }; // DB lock released

    // Phase 2: fetch issues — no DB lock held.
    let issues = get_viewer_issues(&api_key).await?;

    let mut total_count = 0usize;
    let mut card_ids: Vec<i64> = Vec::new();

    // Phase 3: upsert selected issues.
    {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        for issue in &issues {
            let external_id = format!("linear:{}", issue.identifier);
            if !external_ids.contains(&external_id) {
                continue;
            }

            let metadata = serde_json::json!({
                "priority": issue.priority,
                "description": issue.description,
                "estimate": issue.estimate,
                "state": issue.state.name,
            });

            let existing: Option<(i64, Option<String>)> = db
                .query_row(
                    "SELECT id, deleted_at FROM cards \
                     WHERE external_id=? AND source='linear'",
                    rusqlite::params![external_id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(|e| e.to_string())?;

            let card_id = if let Some((id, deleted_at)) = existing {
                if deleted_at.is_some() {
                    continue;
                }
                db.execute(
                    "UPDATE cards SET title=?, url=?, metadata=?, \
                     updated_at=datetime('now') WHERE id=?",
                    rusqlite::params![issue.title, issue.url, metadata.to_string(), id],
                )
                .map_err(|e| e.to_string())?;
                id
            } else {
                db.execute(
                    "INSERT INTO cards \
                     (title, card_type, status, source, external_id, url, metadata, \
                      week_id, day_of_week, position) \
                     VALUES (?, 'task', 'planned', 'linear', ?, ?, ?, NULL, NULL, 0)",
                    rusqlite::params![
                        issue.title,
                        external_id,
                        issue.url,
                        metadata.to_string(),
                    ],
                )
                .map_err(|e| e.to_string())?;
                db.last_insert_rowid()
            };

            total_count += 1;
            card_ids.push(card_id);
        }
    } // DB lock released

    // Phase 4: AI evaluation — no DB lock held during await.
    for card_id in card_ids {
        if let Err(e) = crate::ai::evaluate_card(card_id, &state).await {
            log::warn!("[confirm_linear_sync] AI eval failed for card {card_id}: {e}");
        }
    }

    Ok(total_count)
}

// ---------------------------------------------------------------------------
// GitLab preview / confirm
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn fetch_gitlab_preview(
    state: State<'_, DbState>,
) -> Result<Vec<CardPreview>, String> {
    // Phase 1: read GitLab PAT — acquire and immediately release lock.
    let pat = {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT value FROM secrets WHERE key='gitlab_pat'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or("GitLab PAT not configured")?
    }; // DB lock released

    // Phase 2: fetch MRs — no DB lock held.
    let current_user = get_current_user(&pat).await?;
    let authored_mrs = fetch_authored_mrs(&pat).await?;
    let review_mrs = fetch_review_mrs(&pat, current_user.id).await?;

    // Merge and dedup: "author" role wins on collision.
    let mut mr_map: std::collections::HashMap<i64, (&crate::integrations::gitlab::client::GitLabMR, &str)> =
        std::collections::HashMap::new();
    for mr in &review_mrs {
        mr_map.insert(mr.id, (mr, "reviewer"));
    }
    for mr in &authored_mrs {
        mr_map.insert(mr.id, (mr, "author"));
    }

    // Phase 3: DB check — acquire lock, filter, release.
    let mut previews: Vec<CardPreview> = Vec::new();
    {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        for (mr, _role) in mr_map.values() {
            let external_id = format!("gitlab:{}:{}", mr.project_id, mr.iid);

            if card_exists(&db, &external_id)? {
                continue;
            }
            if is_skipped(&db, &external_id)? {
                continue;
            }

            previews.push(CardPreview {
                external_id,
                title: mr.title.clone(),
                card_type: "mr".to_string(),
                date: Utc::now().format("%Y-%m-%d").to_string(),
                start_time: None,
                source: "gitlab".to_string(),
            });
        }
    } // DB lock released

    Ok(previews)
}

#[tauri::command]
pub async fn confirm_gitlab_sync(
    external_ids: Vec<String>,
    state: State<'_, DbState>,
) -> Result<usize, String> {
    if external_ids.is_empty() {
        return Ok(0);
    }

    // Phase 1: read GitLab PAT.
    let pat = {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT value FROM secrets WHERE key='gitlab_pat'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or("GitLab PAT not configured")?
    }; // DB lock released

    // Phase 2: fetch MRs — no DB lock held.
    let current_user = get_current_user(&pat).await?;
    let authored_mrs = fetch_authored_mrs(&pat).await?;
    let review_mrs = fetch_review_mrs(&pat, current_user.id).await?;

    // Merge and dedup.
    let mut mr_map: std::collections::HashMap<i64, (&crate::integrations::gitlab::client::GitLabMR, &str)> =
        std::collections::HashMap::new();
    for mr in &review_mrs {
        mr_map.insert(mr.id, (mr, "reviewer"));
    }
    for mr in &authored_mrs {
        mr_map.insert(mr.id, (mr, "author"));
    }

    // Build selected set with line counts; fetch lines only for selected MRs.
    let mut selected: Vec<(&crate::integrations::gitlab::client::GitLabMR, &str, i64)> = Vec::new();
    for (mr, role) in mr_map.values() {
        let external_id = format!("gitlab:{}:{}", mr.project_id, mr.iid);
        if external_ids.contains(&external_id) {
            let lines = fetch_mr_lines_changed(&pat, mr.project_id, mr.iid).await;
            selected.push((mr, role, lines));
        }
    }

    let mut total_count = 0usize;
    let mut card_ids: Vec<i64> = Vec::new();

    // Phase 3: upsert selected MRs.
    {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        for (mr, role, lines_changed) in &selected {
            let external_id = format!("gitlab:{}:{}", mr.project_id, mr.iid);

            let metadata = serde_json::json!({
                "author": mr.author.username,
                "mr_iid": mr.iid,
                "role": role,
                "description": mr.description,
                "lines_changed": lines_changed,
            })
            .to_string();

            let existing: Option<(i64, Option<String>)> = db
                .query_row(
                    "SELECT id, deleted_at FROM cards \
                     WHERE external_id=? AND source='gitlab'",
                    rusqlite::params![external_id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(|e| e.to_string())?;

            let card_id = if let Some((id, deleted_at)) = existing {
                if deleted_at.is_some() {
                    continue;
                }
                db.execute(
                    "UPDATE cards SET title=?, url=?, metadata=?, \
                     updated_at=datetime('now') WHERE id=?",
                    rusqlite::params![mr.title, mr.web_url, metadata, id],
                )
                .map_err(|e| e.to_string())?;
                id
            } else {
                db.execute(
                    "INSERT INTO cards \
                     (title, card_type, status, source, external_id, url, metadata, \
                      week_id, day_of_week, position) \
                     VALUES (?, 'mr', 'planned', 'gitlab', ?, ?, ?, NULL, NULL, 0)",
                    rusqlite::params![mr.title, external_id, mr.web_url, metadata],
                )
                .map_err(|e| e.to_string())?;
                db.last_insert_rowid()
            };

            total_count += 1;
            card_ids.push(card_id);
        }
    } // DB lock released

    // Phase 4: AI evaluation — no DB lock held during await.
    for card_id in card_ids {
        if let Err(e) = crate::ai::evaluate_card(card_id, &state).await {
            log::warn!("[confirm_gitlab_sync] AI eval failed for card {card_id}: {e}");
        }
    }

    Ok(total_count)
}
