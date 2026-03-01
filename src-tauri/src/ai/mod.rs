pub mod client;
pub mod prompts;

use crate::db::DbState;
use client::OpenAiClient;
use prompts::*;
use rusqlite::OptionalExtension;

/// Builds an OpenAiClient from the stored secret if ai_provider = 'openai'
/// and the openai_api_key secret is set. Returns None otherwise.
pub fn build_client(db_state: &DbState) -> Option<OpenAiClient> {
    let db = db_state.0.lock().ok()?;
    let key: Option<String> = db
        .query_row(
            "SELECT value FROM secrets WHERE key = 'openai_api_key'",
            [],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten();
    key.map(|k| OpenAiClient::new(&k))
}

pub async fn evaluate_card(card_id: i64, db_state: &DbState) -> Result<(), String> {
    // Phase A: read settings + card data (no lock held across await)
    let (card_type, source, title, metadata_str, time_estimate, api_key) = {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;

        // Check auto_ai toggle
        let auto_ai: bool = db
            .query_row("SELECT auto_ai FROM settings WHERE id=1", [], |r| r.get::<_, i64>(0))
            .map(|v| v != 0)
            .unwrap_or(false);
        if !auto_ai {
            return Ok(());
        }

        // Fetch card fields
        let card = db
            .query_row(
                "SELECT card_type, source, title, metadata, time_estimate FROM cards WHERE id=?",
                rusqlite::params![card_id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, Option<f64>>(4)?,
                    ))
                },
            )
            .map_err(|e| format!("evaluate_card: failed to fetch card {card_id}: {e}"))?;

        // Read API key
        let key: Option<String> = db
            .query_row(
                "SELECT value FROM secrets WHERE key = 'openai_api_key'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        (card.0, card.1, card.2, card.3, card.4, key)
    }; // DB lock released

    // Skip if no API key
    let api_key = match api_key {
        Some(k) => k,
        None => return Ok(()),
    };

    // Skip if already evaluated
    if let Some(meta) = &metadata_str {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(meta) {
            if v.get("ai_title").is_some() {
                return Ok(());
            }
        }
    }

    // Phase B: AI call (no DB lock held)
    let metadata_val: serde_json::Value = metadata_str
        .as_deref()
        .and_then(|m| serde_json::from_str(m).ok())
        .unwrap_or(serde_json::json!({}));

    // Build system prompt and user message based on card type / source.
    let (system, user) = match (card_type.as_str(), source.as_str()) {
        ("task", "linear") => {
            let priority = metadata_val
                .get("priority")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let priority_label = match priority {
                1 => "Urgent",
                2 => "High",
                3 => "Medium",
                4 => "Low",
                _ => "No priority",
            };
            let description = metadata_val
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let estimate = metadata_val
                .get("estimate")
                .and_then(|v| v.as_f64());

            let ai_impact = match priority {
                1 | 2 => "high",
                3 => "mid",
                _ => "low",
            };

            let mut msg = format!(
                "Linear Issue: {title}\nPriority: {priority_label}\nDescription: {description}"
            );
            if let Some(pts) = estimate {
                msg.push_str(&format!("\nStory points: {pts}"));
            }

            let hours_hint = estimate
                .map(|pts| format!(" Use {:.1} hours as a starting estimate (story points * 0.5).", pts * 0.5))
                .unwrap_or_default();

            let sys = build_linear_system_prompt(ai_impact, &hours_hint);
            (sys, msg)
        }
        ("documentation", "notion") => {
            let word_count = metadata_val
                .get("word_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let content_preview = metadata_val
                .get("content_preview")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let sys = SYSTEM_PROMPT_NOTION.to_string();
            let msg = format!(
                "Notion Document: {title}\nWord count: {word_count}\n\
                 Content preview:\n{content_preview}"
            );
            (sys, msg)
        }
        ("thread", "slack") => {
            let channel_name = metadata_val
                .get("channel_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let reply_count = metadata_val
                .get("reply_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let first_message = metadata_val
                .get("first_message")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let thread_preview = metadata_val
                .get("thread_preview")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let sys = SYSTEM_PROMPT_SLACK.to_string();
            let msg = format!(
                "Slack thread in #{channel_name}\nReply count: {reply_count}\n\
                 First message:\n{first_message}\nRecent replies:\n{thread_preview}"
            );
            (sys, msg)
        }
        ("mr", _) => {
            let desc = metadata_val
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let lines = metadata_val
                .get("lines_changed")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let msg = if desc.is_empty() {
                format!("MR: {title}\nLines changed: {lines}")
            } else {
                format!("MR: {title}\nLines changed: {lines}\nDescription: {desc}")
            };
            let sys = SYSTEM_PROMPT_MR.to_string();
            (sys, msg)
        }
        ("meeting", _) => {
            let desc = metadata_val
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let hours = time_estimate.unwrap_or(0.0);
            let sys = SYSTEM_PROMPT_MEETING.to_string();
            let msg = format!("Meeting: {title}\nDescription: {desc}\nDuration: {hours}h");
            (sys, msg)
        }
        _ => {
            let sys = SYSTEM_PROMPT_GENERIC.to_string();
            let msg = format!("Task: {title}");
            (sys, msg)
        }
    };

    let client = OpenAiClient::new(&api_key);
    let response = client.complete(&system, &user).await?;

    // Strip markdown code fences if present (```json ... ```)
    let json_str = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let ai_fields: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("failed to parse AI response as JSON: {e}\nResponse: {response}"))?;

    // Merge ai_* keys into existing metadata
    let mut merged = metadata_val;
    if let (Some(obj), Some(ai_obj)) = (merged.as_object_mut(), ai_fields.as_object()) {
        for (k, v) in ai_obj {
            obj.insert(k.clone(), v.clone());
        }
    }

    let new_metadata = serde_json::to_string(&merged).map_err(|e| e.to_string())?;

    // Phase C: save merged metadata; seed time_estimate from ai_hours when not yet set.
    // Meetings are excluded — their duration comes from the calendar event.
    let ai_hours = ai_fields.get("ai_hours").and_then(|v| v.as_f64());
    {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.execute(
            "UPDATE cards SET metadata=?, updated_at=datetime('now') WHERE id=?",
            rusqlite::params![new_metadata, card_id],
        )
        .map_err(|e| format!("evaluate_card: failed to update card {card_id}: {e}"))?;
        if card_type != "meeting" {
            if let Some(hours) = ai_hours {
                db.execute(
                    "UPDATE cards SET time_estimate=? WHERE id=? AND time_estimate IS NULL",
                    rusqlite::params![hours, card_id],
                )
                .map_err(|e| format!("evaluate_card: failed to seed time_estimate for card {card_id}: {e}"))?;
            }
        }
    }

    Ok(())
}
