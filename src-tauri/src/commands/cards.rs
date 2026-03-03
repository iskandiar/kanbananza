use crate::db::{DbState, row_to_card};
use crate::types::{Card, CardType};
use rusqlite::types::Value;
use rusqlite::Connection;
use tauri::State;

const SELECT: &str = "SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,position,source,external_id,notes,metadata,created_at,updated_at,project_id,done_at FROM cards";

// ---------------------------------------------------------------------------
// Pure-DB helpers — take &Connection directly so tests can call them without
// constructing Tauri State.
// ---------------------------------------------------------------------------

pub(crate) fn db_list_cards_by_week(
    db: &Connection,
    week_id: Option<i64>,
) -> Result<Vec<Card>, String> {
    let sql = match week_id {
        Some(_) => format!("{SELECT} WHERE week_id=? AND deleted_at IS NULL ORDER BY day_of_week,position"),
        None => format!("{SELECT} WHERE week_id IS NULL AND deleted_at IS NULL AND status != 'done' ORDER BY position"),
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
    project_id: Option<i64>,
    url: Option<String>,
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
        "INSERT INTO cards (title,card_type,week_id,day_of_week,position,project_id,url) VALUES (?,?,?,?,?,?,?)",
        rusqlite::params![title, card_type_str, week_id, day_of_week, position, project_id, url],
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
    card_type: Option<String>,
    project_id: Option<i64>,
    clear_project_id: Option<bool>,
) -> Result<Card, String> {
    let mut parts: Vec<String> = Vec::new();
    let mut vals: Vec<Value> = Vec::new();

    if let Some(v) = title         { parts.push("title=?".into());          vals.push(Value::Text(v)); }
    if let Some(v) = status.clone() { parts.push("status=?".into());        vals.push(Value::Text(v)); }
    if let Some(v) = impact        { parts.push("impact=?".into());         vals.push(Value::Text(v)); }
    if let Some(v) = time_estimate { parts.push("time_estimate=?".into());  vals.push(Value::Real(v)); }
    if let Some(v) = url           { parts.push("url=?".into());            vals.push(Value::Text(v)); }
    if let Some(v) = position      { parts.push("position=?".into());       vals.push(Value::Integer(v)); }
    if let Some(v) = notes         { parts.push("notes=?".into());          vals.push(Value::Text(v)); }
    if let Some(v) = card_type     { parts.push("card_type=?".into());      vals.push(Value::Text(v)); }
    if clear_project_id == Some(true) {
        parts.push("project_id=NULL".into());
    } else if let Some(v) = project_id {
        parts.push("project_id=?".into()); vals.push(Value::Integer(v));
    }

    if let Some(ref s) = status {
        if s == "done" {
            parts.push("done_at=datetime('now')".into());
        } else {
            parts.push("done_at=NULL".into());
        }
    }

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
    db.execute(
        "UPDATE cards SET deleted_at=datetime('now'), updated_at=datetime('now') WHERE id=?",
        [id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn db_duplicate_card(db: &Connection, id: i64) -> Result<Card, String> {
    // 1. Fetch source card
    let src: Card = db
        .query_row(&format!("{SELECT} WHERE id=?"), [id], row_to_card)
        .map_err(|e| e.to_string())?;
    // 2. Compute next position in same slot
    let position: i64 = db
        .query_row(
            "SELECT COALESCE(MAX(position),0)+1 FROM cards WHERE week_id IS ? AND day_of_week IS ?",
            rusqlite::params![src.week_id, src.day_of_week],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    // 3. Convert card_type enum to its snake_case string via serde (matches db_create_card pattern)
    let card_type_str = serde_json::to_value(&src.card_type)
        .map_err(|e| e.to_string())?
        .as_str()
        .unwrap_or("task")
        .to_string();
    // 4. Insert copy (external_id not copied)
    db.execute(
        "INSERT INTO cards (title,card_type,impact,time_estimate,url,notes,metadata,week_id,day_of_week,position,project_id,status,source)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,'planned','manual')",
        rusqlite::params![
            src.title,
            card_type_str,
            src.impact.as_ref().and_then(|i| serde_json::to_value(i).ok()).and_then(|v| v.as_str().map(|s| s.to_string())),
            src.time_estimate,
            src.url,
            src.notes,
            src.metadata,
            src.week_id,
            src.day_of_week,
            position,
            src.project_id
        ],
    )
    .map_err(|e| e.to_string())?;
    // 5. Return new card
    db.query_row(
        &format!("{SELECT} WHERE id=?"),
        [db.last_insert_rowid()],
        row_to_card,
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn db_search_cards(db: &Connection, query: &str) -> Result<Vec<Card>, String> {
    let pattern = format!("%{}%", query);
    let sql = format!(
        "{SELECT} WHERE (title LIKE ?1 OR notes LIKE ?1) AND deleted_at IS NULL ORDER BY updated_at DESC LIMIT 50"
    );
    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let cards = stmt
        .query_map([&pattern], row_to_card)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(cards)
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
    project_id: Option<i64>,
    url: Option<String>,
    state: State<'_, DbState>,
) -> Result<Card, String> {
    let (card_id, card) = {
        let db = state.0.lock().map_err(|e| e.to_string())?;
        let card = db_create_card(&db, &title, &card_type, week_id, day_of_week, project_id, url)?;
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
    card_type: Option<String>,
    project_id: Option<i64>,
    // When true, sets project_id=NULL (unassign from project).
    clear_project_id: Option<bool>,
    state: State<DbState>,
) -> Result<Card, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_update_card(
        &db, id, title, status, impact, time_estimate, url, week_id, day_of_week, position,
        notes, clear_week, card_type, project_id, clear_project_id,
    )
}

#[tauri::command]
pub fn delete_card(id: i64, state: State<DbState>) -> Result<(), String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_delete_card(&db, id)
}

#[tauri::command]
pub fn duplicate_card(id: i64, state: State<DbState>) -> Result<Card, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_duplicate_card(&db, id)
}

#[tauri::command]
pub fn search_cards(query: String, state: State<DbState>) -> Result<Vec<Card>, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_search_cards(&db, &query)
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
        db.execute_batch(include_str!("../../migrations/0003_projects.sql"))
            .unwrap();
        let _ = db.execute("ALTER TABLE cards ADD COLUMN project_id INTEGER REFERENCES projects(id)", []);
        let _ = db.execute("ALTER TABLE cards ADD COLUMN done_at TEXT", []);
        let _ = db.execute("ALTER TABLE cards ADD COLUMN deleted_at TEXT", []);
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

        let card = db_create_card(&db, "Test card", &CardType::Task, Some(week_id), Some(1), None, None)
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
        let card = db_create_card(&db, "Original", &CardType::Task, None, None, None, None).unwrap();

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
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(updated.title, "Renamed");
        // Re-read by id to confirm persistence — done cards are excluded from
        // the backlog query, so we query directly.
        let re_read = db
            .query_row(&format!("{SELECT} WHERE id=?"), [card.id], row_to_card)
            .unwrap();
        assert_eq!(re_read.title, "Renamed");
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
            db_create_card(&db, "Placed card", &CardType::Task, Some(week_id), Some(2), None, None).unwrap();
        assert_eq!(card.week_id, Some(week_id));

        let moved = db_update_card(
            &db, card.id, None, None, None, None, None, None, None, None, None,
            Some(true), // clear_week
            None,
            None,
            None,
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
        let card = db_create_card(&db, "To delete", &CardType::Task, None, None, None, None).unwrap();

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

        let c1 = db_create_card(&db, "First",  &CardType::Task, Some(week_id), Some(3), None, None).unwrap();
        let c2 = db_create_card(&db, "Second", &CardType::Task, Some(week_id), Some(3), None, None).unwrap();
        let c3 = db_create_card(&db, "Third",  &CardType::Task, Some(week_id), Some(3), None, None).unwrap();

        assert_eq!(c1.position, 1, "first card position must be 1");
        assert_eq!(c2.position, 2, "second card position must be 2");
        assert_eq!(c3.position, 3, "third card position must be 3");
    }

    // Duplicating a card must produce a new card with the same fields but a new id and
    // a position one greater than the original in the same slot.
    #[test]
    fn duplicate_card_creates_copy_with_incremented_position() {
        let db = open_test_db();
        db.execute(
            "INSERT OR IGNORE INTO weeks (year, week_number, start_date) VALUES (2026, 9, '2026-02-23')",
            [],
        )
        .unwrap();
        let week_id: i64 = db
            .query_row("SELECT id FROM weeks WHERE year=2026 AND week_number=9", [], |r| r.get(0))
            .unwrap();

        let src = db_create_card(&db, "Original", &CardType::Task, Some(week_id), Some(1), None, None)
            .unwrap();
        assert_eq!(src.position, 1);

        let dup = db_duplicate_card(&db, src.id).unwrap();

        assert_ne!(dup.id, src.id, "duplicate must have a new id");
        assert_eq!(dup.title, src.title);
        assert_eq!(dup.card_type, src.card_type);
        assert_eq!(dup.week_id, src.week_id);
        assert_eq!(dup.day_of_week, src.day_of_week);
        assert_eq!(dup.position, 2, "duplicate must be placed after the original");
        assert_eq!(dup.notes, src.notes, "notes must be copied");
        assert!(dup.external_id.is_none(), "external_id must not be copied");
    }

    // Searching by title must return matching cards ordered by updated_at DESC.
    #[test]
    fn search_cards_matches_title_substring() {
        let db = open_test_db();
        db_create_card(&db, "Fix login bug", &CardType::Task, None, None, None, None).unwrap();
        db_create_card(&db, "Write documentation", &CardType::Task, None, None, None, None).unwrap();
        db_create_card(&db, "Fix signup bug", &CardType::Task, None, None, None, None).unwrap();

        let results = db_search_cards(&db, "Fix").unwrap();
        assert_eq!(results.len(), 2, "should return both 'Fix' cards");
        assert!(
            results.iter().all(|c| c.title.contains("Fix")),
            "all results must contain 'Fix'"
        );
    }

    // Searching with no matching term must return an empty list.
    #[test]
    fn search_cards_no_match_returns_empty() {
        let db = open_test_db();
        db_create_card(&db, "Refactor auth", &CardType::Task, None, None, None, None).unwrap();

        let results = db_search_cards(&db, "nonexistent").unwrap();
        assert!(results.is_empty(), "no results expected for unmatched query");
    }

    // Passing clear_project_id: true must set project_id to NULL.
    #[test]
    fn clear_project_id_nullifies_project() {
        let db = open_test_db();
        db.execute("INSERT INTO projects (name, slug, color) VALUES ('P','P1','#fff')", []).unwrap();
        let pid: i64 = db
            .query_row("SELECT id FROM projects WHERE slug='P1'", [], |r| r.get(0))
            .unwrap();
        let card = db_create_card(&db, "card", &CardType::Task, None, None, Some(pid), None).unwrap();
        assert_eq!(card.project_id, Some(pid));

        let updated = db_update_card(
            &db, card.id, None, None, None, None, None, None, None, None, None, None, None,
            None,       // project_id
            Some(true), // clear_project_id
        )
        .unwrap();

        assert!(
            updated.project_id.is_none(),
            "project_id must be NULL after clear_project_id=true"
        );
    }

    // Updating a card's type must persist the new card type.
    #[test]
    fn update_card_type_persists() {
        let db = open_test_db();
        let card = db_create_card(&db, "My task", &CardType::Task, None, None, None, None).unwrap();
        assert_eq!(card.card_type, CardType::Task);

        let meeting_str = serde_json::to_value(&CardType::Meeting)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let updated = db_update_card(
            &db,
            card.id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(meeting_str),
            None,
            None,
        )
        .unwrap();

        assert_eq!(updated.card_type, CardType::Meeting);
        // Re-read to confirm persistence through a second query.
        let re_read = db_list_cards_by_week(&db, None).unwrap();
        assert_eq!(re_read.len(), 1);
        assert_eq!(re_read[0].card_type, CardType::Meeting);
    }
}
