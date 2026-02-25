use crate::db::DbState;
use crate::types::{Card, CardStatus, CardType, Source};
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

#[tauri::command]
pub fn list_cards_by_week(week_id: Option<i64>, state: State<DbState>) -> Result<Vec<Card>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    let (sql, params): (&str, Box<dyn rusqlite::types::ToSql>) = match week_id {
        Some(id) => ("SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,position,source,external_id,notes,metadata,created_at,updated_at FROM cards WHERE week_id=? ORDER BY day_of_week,position", Box::new(id)),
        None => ("SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,position,source,external_id,notes,metadata,created_at,updated_at FROM cards WHERE week_id IS NULL ORDER BY position", Box::new(rusqlite::types::Null)),
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
pub fn create_card(
    title: String,
    card_type: String,
    week_id: Option<i64>,
    day_of_week: Option<i64>,
    state: State<DbState>,
) -> Result<Card, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    let position: i64 = db
        .query_row(
            "SELECT COALESCE(MAX(position),0)+1 FROM cards WHERE week_id IS ? AND day_of_week IS ?",
            rusqlite::params![week_id, day_of_week],
            |r| r.get(0),
        )
        .unwrap_or(0);
    db.execute(
        "INSERT INTO cards (title,card_type,week_id,day_of_week,position) VALUES (?,?,?,?,?)",
        rusqlite::params![title, card_type, week_id, day_of_week, position],
    )
    .map_err(|e| e.to_string())?;
    let id = db.last_insert_rowid();
    let card = db
        .query_row(
            "SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,position,source,external_id,notes,metadata,created_at,updated_at FROM cards WHERE id=?",
            [id],
            row_to_card,
        )
        .map_err(|e| e.to_string())?;
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
    day_of_week: Option<i64>,
    week_id: Option<i64>,
    position: Option<i64>,
    notes: Option<String>,
    state: State<DbState>,
) -> Result<Card, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(v) = title { db.execute("UPDATE cards SET title=?,updated_at=datetime('now') WHERE id=?", rusqlite::params![v, id]).map_err(|e| e.to_string())?; }
    if let Some(v) = status { db.execute("UPDATE cards SET status=?,updated_at=datetime('now') WHERE id=?", rusqlite::params![v, id]).map_err(|e| e.to_string())?; }
    if let Some(v) = impact { db.execute("UPDATE cards SET impact=?,updated_at=datetime('now') WHERE id=?", rusqlite::params![v, id]).map_err(|e| e.to_string())?; }
    if let Some(v) = time_estimate { db.execute("UPDATE cards SET time_estimate=?,updated_at=datetime('now') WHERE id=?", rusqlite::params![v, id]).map_err(|e| e.to_string())?; }
    if let Some(v) = url { db.execute("UPDATE cards SET url=?,updated_at=datetime('now') WHERE id=?", rusqlite::params![v, id]).map_err(|e| e.to_string())?; }
    if let Some(v) = day_of_week { db.execute("UPDATE cards SET day_of_week=?,updated_at=datetime('now') WHERE id=?", rusqlite::params![v, id]).map_err(|e| e.to_string())?; }
    if let Some(v) = week_id { db.execute("UPDATE cards SET week_id=?,updated_at=datetime('now') WHERE id=?", rusqlite::params![v, id]).map_err(|e| e.to_string())?; }
    if let Some(v) = position { db.execute("UPDATE cards SET position=?,updated_at=datetime('now') WHERE id=?", rusqlite::params![v, id]).map_err(|e| e.to_string())?; }
    if let Some(v) = notes { db.execute("UPDATE cards SET notes=?,updated_at=datetime('now') WHERE id=?", rusqlite::params![v, id]).map_err(|e| e.to_string())?; }
    let card = db
        .query_row(
            "SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,position,source,external_id,notes,metadata,created_at,updated_at FROM cards WHERE id=?",
            [id],
            row_to_card,
        )
        .map_err(|e| e.to_string())?;
    Ok(card)
}

#[tauri::command]
pub fn delete_card(id: i64, state: State<DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM cards WHERE id=?", [id]).map_err(|e| e.to_string())?;
    Ok(())
}
