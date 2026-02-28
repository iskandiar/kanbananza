/// Wraps a rusqlite `Connection` in a `Mutex` so Tauri can share it across
/// command handlers as managed state. All commands lock this mutex for the
/// duration of their query — there is no connection pool.
use rusqlite::Connection;
use std::sync::Mutex;
use crate::types::Card;

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
    })
}
