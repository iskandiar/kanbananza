use crate::db::DbState;
use crate::types::{Card, CardStatus, CardType, Source};
use rusqlite::types::Value;
use tauri::State;

fn row_to_card(row: &rusqlite::Row) -> rusqlite::Result<Card> {
    Ok(Card {
        id: row.get(0)?,
        title: row.get(1)?,
        card_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(2)?))
            .unwrap_or(CardType::Task),
        status: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?))
            .unwrap_or(CardStatus::Planned),
        impact: row.get::<_, Option<String>>(4)?.and_then(|s| {
            serde_json::from_str(&format!("\"{s}\"")).ok()
        }),
        time_estimate: row.get(5)?,
        url: row.get(6)?,
        week_id: row.get(7)?,
        day_of_week: row.get(8)?,
        position: row.get(9)?,
        source: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(10)?))
            .unwrap_or(Source::Manual),
        external_id: row.get(11)?,
        notes: row.get(12)?,
        metadata: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

const SELECT: &str = "SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,position,source,external_id,notes,metadata,created_at,updated_at FROM cards";

#[tauri::command]
pub fn list_cards_by_week(week_id: Option<i64>, state: State<DbState>) -> Result<Vec<Card>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    let (sql, params): (&str, Box<dyn rusqlite::types::ToSql>) = match week_id {
        Some(id) => (
            &format!("{SELECT} WHERE week_id=? ORDER BY day_of_week,position"),
            Box::new(id),
        ),
        None => (
            &format!("{SELECT} WHERE week_id IS ? ORDER BY position"),
            Box::new(rusqlite::types::Null),
        ),
    };
    let mut stmt = db.prepare(sql).map_err(|e| e.to_string())?;
    let cards = stmt
        .query_map([params], row_to_card)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(cards)
}

#[tauri::command]
pub async fn create_card(
    title: String,
    card_type: CardType, // validated enum — rejects unknown types at the boundary
    week_id: Option<i64>,
    day_of_week: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Card, String> {
    let (card_id, card) = {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        let position: i64 = db
            .query_row(
                "SELECT COALESCE(MAX(position),0)+1 FROM cards WHERE week_id IS ? AND day_of_week IS ?",
                rusqlite::params![week_id, day_of_week],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        // Serialize enum to the DB string value ("task", "meeting", etc.)
        let card_type_str = serde_json::to_value(&card_type)
            .map_err(|e| e.to_string())?
            .as_str()
            .unwrap_or("task")
            .to_string();
        db.execute(
            "INSERT INTO cards (title,card_type,week_id,day_of_week,position) VALUES (?,?,?,?,?)",
            rusqlite::params![title, card_type_str, week_id, day_of_week, position],
        )
        .map_err(|e| e.to_string())?;
        let id = db.last_insert_rowid();
        let card = db
            .query_row(&format!("{SELECT} WHERE id=?"), [id], row_to_card)
            .map_err(|e| e.to_string())?;
        (id, card)
    }; // DB lock released

    if let Err(e) = crate::ai::evaluate_card(card_id, &state).await {
        log::warn!("[create_card] AI eval failed for card {card_id}: {e}");
    }

    Ok(card)
}

#[tauri::command]
pub fn update_card(
    id: i64,
    title: Option<String>,
    status: Option<String>,
    impact: Option<String>,
    time_estimate: Option<f64>,
    url: Option<String>,
    week_id: Option<i64>,
    day_of_week: Option<i64>,
    position: Option<i64>,
    notes: Option<String>,
    // When true, sets week_id=NULL and day_of_week=NULL (move to backlog).
    // Needed because JSON null and absent field both deserialize to Option::None.
    clear_week: Option<bool>,
    state: State<DbState>,
) -> Result<Card, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;

    let mut parts: Vec<String> = Vec::new();
    let mut vals: Vec<Value> = Vec::new();

    if let Some(v) = title        { parts.push("title=?".into());         vals.push(Value::Text(v)); }
    if let Some(v) = status       { parts.push("status=?".into());        vals.push(Value::Text(v)); }
    if let Some(v) = impact       { parts.push("impact=?".into());        vals.push(Value::Text(v)); }
    if let Some(v) = time_estimate { parts.push("time_estimate=?".into()); vals.push(Value::Real(v)); }
    if let Some(v) = url          { parts.push("url=?".into());           vals.push(Value::Text(v)); }
    if let Some(v) = position     { parts.push("position=?".into());      vals.push(Value::Integer(v)); }
    if let Some(v) = notes        { parts.push("notes=?".into());         vals.push(Value::Text(v)); }

    if clear_week == Some(true) {
        // Embed NULLs directly — no placeholder needed
        parts.push("week_id=NULL".into());
        parts.push("day_of_week=NULL".into());
    } else {
        if let Some(v) = week_id    { parts.push("week_id=?".into());    vals.push(Value::Integer(v)); }
        if let Some(v) = day_of_week { parts.push("day_of_week=?".into()); vals.push(Value::Integer(v)); }
    }

    if !parts.is_empty() {
        parts.push("updated_at=datetime('now')".into());
        let sql = format!("UPDATE cards SET {} WHERE id=?", parts.join(","));
        vals.push(Value::Integer(id));
        // Single statement — atomically updates all fields at once
        db.execute(&sql, rusqlite::params_from_iter(vals.into_iter()))
            .map_err(|e| e.to_string())?;
    }

    db.query_row(&format!("{SELECT} WHERE id=?"), [id], row_to_card)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_card(id: i64, state: State<DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM cards WHERE id=?", [id]).map_err(|e| e.to_string())?;
    Ok(())
}
