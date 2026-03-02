use crate::db::DbState;
use rusqlite::Connection;
use tauri::State;

// ---------------------------------------------------------------------------
// Pure-DB helper
// ---------------------------------------------------------------------------

/// Moves all `planned` cards from `week_id` to the global backlog (week_id=NULL).
/// `done` cards are left untouched.
/// Returns the number of rows moved.
pub(crate) fn db_rollover_week(db: &Connection, week_id: i64) -> Result<i64, String> {
    let count = db
        .execute(
            "UPDATE cards SET week_id=NULL, day_of_week=NULL, updated_at=datetime('now') \
             WHERE week_id=? AND status='planned'",
            [week_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(count as i64)
}

// ---------------------------------------------------------------------------
// Tauri command
// ---------------------------------------------------------------------------

/// Moves all planned (non-done) cards from a week to the global backlog.
/// Returns the count of cards rolled over.
#[tauri::command]
pub fn rollover_week(week_id: i64, state: State<DbState>) -> Result<i64, String> {
    let db = state.0.lock().map_err(|e| e.to_string())?;
    db_rollover_week(&db, week_id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::cards::db_create_card;
    use crate::commands::weeks::db_get_or_create_week;
    use crate::types::CardType;
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
        db
    }

    // Only planned cards must move to backlog; done cards stay in their week.
    #[test]
    fn rollover_moves_planned_leaves_done() {
        let db = open_test_db();
        let week = db_get_or_create_week(&db, 2026, 9, "2026-02-23").unwrap();

        // Create one planned card and one done card in the week.
        let planned =
            db_create_card(&db, "Planned task", &CardType::Task, Some(week.id), Some(1), None, None).unwrap();
        let done_card =
            db_create_card(&db, "Done task", &CardType::Task, Some(week.id), Some(1), None, None).unwrap();

        // Mark the second card as done directly in the DB.
        db.execute(
            "UPDATE cards SET status='done' WHERE id=?",
            [done_card.id],
        )
        .unwrap();

        let moved = db_rollover_week(&db, week.id).unwrap();
        assert_eq!(moved, 1, "exactly one planned card must move");

        // Planned card must now be in the backlog (week_id IS NULL).
        let (planned_week_id,): (Option<i64>,) = db
            .query_row(
                "SELECT week_id FROM cards WHERE id=?",
                [planned.id],
                |r| Ok((r.get(0)?,)),
            )
            .unwrap();
        assert!(
            planned_week_id.is_none(),
            "planned card must have week_id=NULL after rollover"
        );

        // Done card must remain in the week.
        let (done_week_id,): (Option<i64>,) = db
            .query_row(
                "SELECT week_id FROM cards WHERE id=?",
                [done_card.id],
                |r| Ok((r.get(0)?,)),
            )
            .unwrap();
        assert_eq!(
            done_week_id,
            Some(week.id),
            "done card must stay in its week after rollover"
        );
    }

    // Rolling over a week with no planned cards must return 0.
    #[test]
    fn rollover_empty_week_returns_zero() {
        let db = open_test_db();
        let week = db_get_or_create_week(&db, 2026, 10, "2026-03-02").unwrap();

        let moved = db_rollover_week(&db, week.id).unwrap();
        assert_eq!(moved, 0);
    }
}
