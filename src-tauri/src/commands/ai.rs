use crate::ai::client::OpenAiClient;
use crate::ai::prompts::SYSTEM_PROMPT_WEEK_SUMMARY;
use crate::db::DbState;
use rusqlite::OptionalExtension;
use tauri::State;

#[tauri::command]
pub async fn summarise_week(week_id: i64, state: State<'_, DbState>) -> Result<String, String> {
    // Phase A: fetch cards + API key (no lock across await)
    let (cards, client) = {
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
            .prepare("SELECT card_type, title, status FROM cards WHERE week_id=?")
            .map_err(|e| e.to_string())?;

        let cards: Vec<(String, String, String)> = stmt
            .query_map(rusqlite::params![week_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        (cards, client)
    }; // DB lock released

    if cards.is_empty() {
        return Err("No cards found for this week".to_string());
    }

    // Phase B: AI call
    let user_msg = cards
        .iter()
        .map(|(t, title, status)| format!("{t}: {title} [{status}]"))
        .collect::<Vec<_>>()
        .join("\n");

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
