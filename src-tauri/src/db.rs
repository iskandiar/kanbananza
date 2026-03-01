/// Wraps a rusqlite `Connection` in a `Mutex` so Tauri can share it across
/// command handlers as managed state. All commands lock this mutex for the
/// duration of their query — there is no connection pool.
use rusqlite::Connection;
use std::sync::Mutex;
use crate::types::{Card, Project};

pub struct DbState(pub Mutex<Connection>);

pub fn row_to_card(row: &rusqlite::Row) -> rusqlite::Result<Card> {
    use crate::types::{Card, CardStatus, CardType, Source};
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
        project_id: row.get(16)?,
        done_at: row.get(17)?,
    })
}

pub fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        slug: row.get(2)?,
        color: row.get(3)?,
        archived: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
    })
}

pub fn db_list_projects(conn: &Connection, archived: bool) -> Result<Vec<Project>, String> {
    let mut stmt = conn.prepare(
        "SELECT id,name,slug,color,archived,created_at FROM projects WHERE archived=? ORDER BY created_at"
    ).map_err(|e| e.to_string())?;
    stmt.query_map([archived as i64], row_to_project)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

pub fn db_create_project(conn: &Connection, name: &str, slug: &str, color: &str) -> Result<Project, String> {
    conn.execute(
        "INSERT INTO projects(name,slug,color) VALUES(?,?,?)",
        rusqlite::params![name, slug, color],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id,name,slug,color,archived,created_at FROM projects WHERE id=?",
        [id],
        row_to_project,
    ).map_err(|e| e.to_string())
}

pub fn db_update_project(conn: &Connection, id: i64, name: Option<&str>, slug: Option<&str>, color: Option<&str>) -> Result<Project, String> {
    conn.execute(
        "UPDATE projects SET name=COALESCE(?,name),slug=COALESCE(?,slug),color=COALESCE(?,color) WHERE id=?",
        rusqlite::params![name, slug, color, id],
    ).map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id,name,slug,color,archived,created_at FROM projects WHERE id=?",
        [id],
        row_to_project,
    ).map_err(|e| e.to_string())
}

pub fn db_archive_project(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("UPDATE projects SET archived=1 WHERE id=?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn db_list_cards_by_project(conn: &Connection, project_id: i64) -> Result<Vec<Card>, String> {
    let select = "SELECT id,title,card_type,status,impact,time_estimate,url,week_id,day_of_week,position,source,external_id,notes,metadata,created_at,updated_at,project_id,done_at FROM cards";
    let mut stmt = conn.prepare(&format!("{select} WHERE project_id=? ORDER BY status,position"))
        .map_err(|e| e.to_string())?;
    stmt.query_map([project_id], row_to_card)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}
