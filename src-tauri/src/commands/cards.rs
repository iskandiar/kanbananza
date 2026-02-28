use crate::db::DbState;
use crate::types::{Card, CardStatus, CardType, Source};
use rusqlite::types::Value;
use rusqlite::Connection;
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

// ---------------------------------------------------------------------------
// Pure-DB helpers — take &Connection directly so tests can call them without
// constructing Tauri State.
// ---------------------------------------------------------------------------

pub(crate) fn db_list_cards_by_week(
    db: &Connection,
    week_id: Option<i64>,
) -> Result<Vec<Card>, String> {
    let sql = match week_id {
        Some(_) => format!("{SELECT} WHERE week_id=? ORDER BY day_of_week,position"),
        None => format!("{SELECT} WHERE week_id IS NULL ORDER BY position"),
    };
    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let cards = match week_id {
        Some(id) => stmt
            .query_map([id], row_to_card)
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?,
        None => stmt
            .query_map([], row_to_card)
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?,
    };
    Ok(cards)
}

pub(crate) fn db_create_card(
    db: &Connection,
    title: &str,
    card_type: &CardType,
    week_id: Option<i64>,
    day_of_week: Option<i64>,
) -> Result<Card, String> {
    let position: i64 = db
        .query_row(
            "SELECT COALESCE(MAX(position),0)+1 FROM cards WHERE week_id IS ? AND day_of_week IS ?",
            rusqlite::params![week_id, day_of_week],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let card_type_str = serde_json::to_value(card_type)
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
    db.query_row(&format!("{SELECT} WHERE id=?"), [id], row_to_card)
        .map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn db_update_card(
    db: &Connection,
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
    clear_week: Option<bool>,
) -> Result<Card, String> {
    let mut parts: Vec<String> = Vec::new();
    let mut vals: Vec<Value> = Vec::new();

    if let Some(v) = title         { parts.push("title=?".into());          vals.push(Value::Text(v)); }
    if let Some(v) = status        { parts.push("status=?".into());         vals.push(Value::Text(v)); }
    if let Some(v) = impact        { parts.push("impact=?".into());         vals.push(Value::Text(v)); }
    if let Some(v) = time_estimate { parts.push("time_estimate=?".into());  vals.push(Value::Real(v)); }
    if let Some(v) = url           { parts.push("url=?".into());            vals.push(Value::Text(v)); }
    if let Some(v) = position      { parts.push("position=?".into());       vals.push(Value::Integer(v)); }
    if let Some(v) = notes         { parts.push("notes=?".into());          vals.push(Value::Text(v)); }

    if clear_week == Some(true) {
        // Embed NULLs directly — no placeholder needed
        parts.push("week_id=NULL".into());
        parts.push("day_of_week=NULL".into());
    } else {
        if let Some(v) = week_id     { parts.push("week_id=?".into());     vals.push(Value::Integer(v)); }
        if let Some(v) = day_of_week { parts.push("day_of_week=?".into()); vals.push(Value::Integer(v)); }
    }

    if !parts.is_empty() {
        parts.push("updated_at=datetime('now')".into());
        let sql = format!("UPDATE cards SET {} WHERE id=?", parts.join(","));
        vals.push(Value::Integer(id));
        db.execute(&sql, rusqlite::params_from_iter(vals.into_iter()))
            .map_err(|e| e.to_string())?;
    }

    db.query_row(&format!("{SELECT} WHERE id=?"), [id], row_to_card)
        .map_err(|e| e.to_string())
}

pub(crate) fn db_delete_card(db: &Connection, id: i64) -> Result<(), String> {
    db.execute("DELETE FROM cards WHERE id=?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri commands — thin wrappers that lock the mutex and delegate to helpers.
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_cards_by_week(week_id: Option<i64>, state: State<DbState>) -> Result<Vec<Card>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_list_cards_by_week(&db, week_id)
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
        let card = db_create_card(&db, &title, &card_type, week_id, day_of_week)?;
        (card.id, card)
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
    db_update_card(
        &db, id, title, status, impact, time_estimate, url, week_id, day_of_week, position,
        notes, clear_week,
    )
}

#[tauri::command]
pub fn delete_card(id: i64, state: State<DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_delete_card(&db, id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_test_db() -> Connection {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("../../migrations/0001_initial.sql"))
            .unwrap();
        db.execute_batch(include_str!("../../migrations/0002_auto_ai.sql"))
            .unwrap();
        db
    }

    // A created card must be returned by list_cards_by_week with a matching week.
    #[test]
    fn create_then_list_round_trip() {
        let db = open_test_db();
        db.execute(
            "INSERT OR IGNORE INTO weeks (year, week_number, start_date) VALUES (2026, 9, '2026-02-23')",
            [],
        )
        .unwrap();
        let week_id: i64 = db
            .query_row("SELECT id FROM weeks WHERE year=2026 AND week_number=9", [], |r| r.get(0))
            .unwrap();

        let card = db_create_card(&db, "Test card", &CardType::Task, Some(week_id), Some(1))
            .unwrap();

        assert_eq!(card.title, "Test card");
        assert_eq!(card.week_id, Some(week_id));
        assert_eq!(card.day_of_week, Some(1));

        let listed = db_list_cards_by_week(&db, Some(week_id)).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, card.id);
    }

    // update_card must persist new status and title values.
    #[test]
    fn update_status_and_title_persists() {
        let db = open_test_db();
        let card = db_create_card(&db, "Original", &CardType::Task, None, None).unwrap();

        let updated = db_update_card(
            &db,
            card.id,
            Some("Renamed".to_string()),
            Some("done".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(updated.title, "Renamed");
        // Re-read to confirm persistence through a second query.
        let re_read = db_list_cards_by_week(&db, None).unwrap();
        assert_eq!(re_read.len(), 1);
        assert_eq!(re_read[0].title, "Renamed");
        // Status round-trips through the serde serialisation path.
        let status_str = serde_json::to_value(&updated.status).unwrap();
        assert_eq!(status_str.as_str().unwrap(), "done");
    }

    // Passing clear_week: true must set week_id and day_of_week to NULL.
    #[test]
    fn update_card_clear_week_nullifies_week() {
        let db = open_test_db();
        db.execute(
            "INSERT OR IGNORE INTO weeks (year, week_number, start_date) VALUES (2026, 9, '2026-02-23')",
            [],
        )
        .unwrap();
        let week_id: i64 = db
            .query_row("SELECT id FROM weeks WHERE year=2026 AND week_number=9", [], |r| r.get(0))
            .unwrap();

        let card =
            db_create_card(&db, "Placed card", &CardType::Task, Some(week_id), Some(2)).unwrap();
        assert_eq!(card.week_id, Some(week_id));

        let moved = db_update_card(
            &db, card.id, None, None, None, None, None, None, None, None, None,
            Some(true), // clear_week
        )
        .unwrap();

        assert!(
            moved.week_id.is_none(),
            "week_id must be NULL after clear_week=true"
        );
        assert!(
            moved.day_of_week.is_none(),
            "day_of_week must be NULL after clear_week=true"
        );
    }

    // Deleting a card must leave list_cards_by_week empty.
    #[test]
    fn delete_card_removes_from_list() {
        let db = open_test_db();
        let card = db_create_card(&db, "To delete", &CardType::Task, None, None).unwrap();

        db_delete_card(&db, card.id).unwrap();

        let backlog = db_list_cards_by_week(&db, None).unwrap();
        assert!(backlog.is_empty(), "backlog must be empty after deletion");
    }

    // Three cards inserted for the same (week_id, day_of_week) must receive
    // consecutive positions 1, 2, 3.
    #[test]
    fn auto_increment_position_for_multiple_cards_same_day() {
        let db = open_test_db();
        db.execute(
            "INSERT OR IGNORE INTO weeks (year, week_number, start_date) VALUES (2026, 9, '2026-02-23')",
            [],
        )
        .unwrap();
        let week_id: i64 = db
            .query_row("SELECT id FROM weeks WHERE year=2026 AND week_number=9", [], |r| r.get(0))
            .unwrap();

        let c1 = db_create_card(&db, "First",  &CardType::Task, Some(week_id), Some(3)).unwrap();
        let c2 = db_create_card(&db, "Second", &CardType::Task, Some(week_id), Some(3)).unwrap();
        let c3 = db_create_card(&db, "Third",  &CardType::Task, Some(week_id), Some(3)).unwrap();

        assert_eq!(c1.position, 1, "first card position must be 1");
        assert_eq!(c2.position, 2, "second card position must be 2");
        assert_eq!(c3.position, 3, "third card position must be 3");
    }
}
