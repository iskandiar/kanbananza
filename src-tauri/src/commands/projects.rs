use crate::ai::client::OpenAiClient;
use crate::db::{
    DbState, db_archive_project, db_create_project, db_list_cards_by_project,
    db_list_projects, db_update_project,
};
use crate::types::{Card, Project};
use rusqlite::OptionalExtension;
use tauri::State;

#[tauri::command]
pub async fn list_projects(state: State<'_, DbState>, archived: bool) -> Result<Vec<Project>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_list_projects(&db, archived)
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, DbState>,
    name: String,
    slug: String,
    color: String,
) -> Result<Project, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_create_project(&db, &name, &slug, &color)
}

#[tauri::command]
pub async fn update_project(
    state: State<'_, DbState>,
    id: i64,
    name: Option<String>,
    slug: Option<String>,
    color: Option<String>,
) -> Result<Project, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_update_project(&db, id, name.as_deref(), slug.as_deref(), color.as_deref())
}

#[tauri::command]
pub async fn archive_project(state: State<'_, DbState>, id: i64) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_archive_project(&db, id)
}

#[tauri::command]
pub async fn list_cards_by_project(
    state: State<'_, DbState>,
    project_id: i64,
) -> Result<Vec<Card>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_list_cards_by_project(&db, project_id)
}

#[tauri::command]
pub async fn generate_project_slug(
    state: State<'_, DbState>,
    name: String,
) -> Result<String, String> {
    let client = {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        let key: Option<String> = db
            .query_row(
                "SELECT value FROM secrets WHERE key = 'openai_api_key'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        key.map(|k| OpenAiClient::new(&k))
            .ok_or_else(|| "OpenAI API key not configured".to_string())?
    };

    let slug = client
        .complete(
            "Reply with only 2-3 uppercase letters, nothing else.",
            &format!("Generate a 2-3 uppercase letter abbreviation/slug for a project named '{name}'."),
        )
        .await?;

    Ok(slug.trim().to_uppercase())
}

#[tauri::command]
pub async fn summarise_project(
    state: State<'_, DbState>,
    project_id: i64,
) -> Result<String, String> {
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
            .prepare(
                "SELECT card_type, title, done_at FROM cards WHERE project_id=? AND status='done'",
            )
            .map_err(|e| e.to_string())?;
        let cards: Vec<(String, String, Option<String>)> = stmt
            .query_map(rusqlite::params![project_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        (cards, client)
    };

    if cards.is_empty() {
        return Err("No completed cards found for this project".to_string());
    }

    let user_msg = cards
        .iter()
        .map(|(t, title, done_at)| {
            let date = done_at.as_deref().unwrap_or("unknown date");
            format!("{t}: {title} [done: {date}]")
        })
        .collect::<Vec<_>>()
        .join("\n");

    client
        .complete(
            "Summarise this project work for an HR/personal log. Include: main focus areas, split by task type, approximate timeline based on done dates. Write 3-5 paragraphs.",
            &user_msg,
        )
        .await
}
