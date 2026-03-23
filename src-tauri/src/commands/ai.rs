use crate::ai::client::OpenAiClient;
use crate::ai::prompts::SYSTEM_PROMPT_WEEK_SUMMARY;
use crate::db::DbState;
use rusqlite::OptionalExtension;
use tauri::State;

#[tauri::command]
pub async fn summarise_week(week_id: i64, notes: Option<String>, state: State<'_, DbState>) -> Result<String, String> {
    // Phase A: fetch cards + API key (no lock across await)
    let (cards, clocked_hours, client) = {
        let db = state.0.lock().map_err(|e| e.to_string())?;

        let key: Option<String> = db
            .query_row(
                "SELECT value FROM secrets WHERE key = 'openai_api_key'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        let client = key
            .map(|k| OpenAiClient::new(&k))
            .ok_or_else(|| "OpenAI API key not configured".to_string())?;

        let mut stmt = db
            .prepare("SELECT card_type, title, status, metadata FROM cards WHERE week_id=?")
            .map_err(|e| e.to_string())?;

        let cards: Vec<(String, String, String, Option<String>)> = stmt
            .query_map(rusqlite::params![week_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let mut hours_stmt = db
            .prepare(
                "SELECT c.card_type, \
                 SUM((julianday(cte.end_time) - julianday(cte.start_time)) * 24.0) as hours \
                 FROM card_time_entries cte \
                 JOIN cards c ON c.id = cte.card_id \
                 WHERE c.week_id=? AND cte.end_time IS NOT NULL \
                 GROUP BY c.card_type \
                 ORDER BY hours DESC",
            )
            .map_err(|e| e.to_string())?;

        let clocked_hours: Vec<(String, f64)> = hours_stmt
            .query_map(rusqlite::params![week_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        (cards, clocked_hours, client)
    }; // DB lock released

    if cards.is_empty() {
        return Err("No cards found for this week".to_string());
    }

    // Phase B: AI call
    let time_section = if clocked_hours.is_empty() {
        String::new()
    } else {
        let total: f64 = clocked_hours.iter().map(|(_, h)| h).sum();
        let lines = clocked_hours
            .iter()
            .map(|(t, h)| format!("  {t}: {h:.1}h ({:.0}%)", h / total * 100.0))
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\nActual clocked time by category:\n{lines}\nTotal: {total:.1}h")
    };

    let user_msg = cards
        .iter()
        .map(|(t, title, status, metadata)| {
            let mut line = format!("{t}: {title} [{status}]");
            if let Some(meta_json) = metadata {
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(meta_json) {
                    if let Some(desc) = meta.get("ai_description").and_then(|v| v.as_str()) {
                        line.push_str(&format!("\n  Description: {desc}"));
                    }
                    if let Some(hours) = meta.get("ai_hours").and_then(|v| v.as_f64()) {
                        line.push_str(&format!("\n  Estimate: {hours}h"));
                    }
                }
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
        + &time_section;

    let user_msg = if let Some(ref n) = notes {
        format!("{user_msg}\n\nGuidance notes from user:\n{n}")
    } else {
        user_msg
    };

    let summary = client
        .complete(SYSTEM_PROMPT_WEEK_SUMMARY, &user_msg)
        .await?;

    // Phase C: save summary
    {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        db.execute(
            "UPDATE weeks SET summary=? WHERE id=?",
            rusqlite::params![summary, week_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(summary)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    /// Test helper: formats a user message with optional notes appended.
    fn format_user_msg_with_notes(base_msg: &str, notes: Option<&str>) -> String {
        let user_msg = base_msg;
        if let Some(n) = notes {
            format!("{user_msg}\n\nGuidance notes from user:\n{n}")
        } else {
            user_msg.to_string()
        }
    }

    #[test]
    fn user_msg_without_notes_unchanged() {
        let base = "Task: Implement feature [planned]\nMeeting: Standup [done]";
        let result = format_user_msg_with_notes(base, None);
        assert_eq!(result, base);
    }

    #[test]
    fn user_msg_with_notes_appended() {
        let base = "Task: Implement feature [planned]";
        let notes = "Focus on edge cases";
        let result = format_user_msg_with_notes(base, Some(notes));
        assert_eq!(result, "Task: Implement feature [planned]\n\nGuidance notes from user:\nFocus on edge cases");
    }

    #[test]
    fn user_msg_with_multiline_notes() {
        let base = "Tasks summary";
        let notes = "Line 1\nLine 2\nLine 3";
        let result = format_user_msg_with_notes(base, Some(notes));
        assert_eq!(result, "Tasks summary\n\nGuidance notes from user:\nLine 1\nLine 2\nLine 3");
    }
}
