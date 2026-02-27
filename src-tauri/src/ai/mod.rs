pub mod client;

use crate::db::DbState;
use client::OpenAiClient;
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
    let (card_type, title, metadata_str, time_estimate, api_key) = {
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
                "SELECT card_type, title, metadata, time_estimate FROM cards WHERE id=?",
                rusqlite::params![card_id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, Option<f64>>(3)?,
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

        (card.0, card.1, card.2, card.3, key)
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
    let system = "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences), \
                  ai_impact (high|medium|low), ai_hours (number, omit for meetings).";

    let metadata_val: serde_json::Value = metadata_str
        .as_deref()
        .and_then(|m| serde_json::from_str(m).ok())
        .unwrap_or(serde_json::json!({}));

    let user = match card_type.as_str() {
        "mr" => {
            let desc = metadata_val
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let loc = metadata_val
                .get("changes_additions")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            format!("MR: {title}\nDescription: {desc}\nChanged lines: {loc}")
        }
        "meeting" => {
            let desc = metadata_val
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let hours = time_estimate.unwrap_or(0.0);
            format!("Meeting: {title}\nDescription: {desc}\nDuration: {hours}h")
        }
        _ => format!("Task: {title}"),
    };

    let client = OpenAiClient::new(&api_key);
    let response = client.complete(system, &user).await?;

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

    // Phase C: save merged metadata
    {
        let db = db_state.0.lock().map_err(|e| e.to_string())?;
        db.execute(
            "UPDATE cards SET metadata=?, updated_at=datetime('now') WHERE id=?",
            rusqlite::params![new_metadata, card_id],
        )
        .map_err(|e| format!("evaluate_card: failed to update card {card_id}: {e}"))?;
    }

    Ok(())
}
