/// Multi-command integration scenarios exercising the full happy-path flow
/// across cards, weeks, and rollover using a single shared in-memory DB.

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::commands::cards::{db_create_card, db_list_cards_by_week, db_update_card};
    use crate::commands::rollover::db_rollover_week;
    use crate::commands::weeks::db_get_or_create_week;
    use crate::types::CardType;

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

    // Scenario 1 — create week → add card to week → rollover → card appears in backlog.
    #[test]
    fn scenario_create_week_card_rollover_backlog() {
        let db = open_test_db();

        // Step 1: create a week.
        let week = db_get_or_create_week(&db, 2026, 9, "2026-02-23").unwrap();

        // Step 2: create a planned card in that week.
        let card =
            db_create_card(&db, "Ship feature", &CardType::Task, Some(week.id), Some(1), None).unwrap();
        assert_eq!(card.week_id, Some(week.id));

        // Verify the card appears in that week's list.
        let in_week = db_list_cards_by_week(&db, Some(week.id)).unwrap();
        assert_eq!(in_week.len(), 1);

        // Step 3: rollover the week.
        let moved = db_rollover_week(&db, week.id).unwrap();
        assert_eq!(moved, 1);

        // Step 4: the week is now empty.
        let still_in_week = db_list_cards_by_week(&db, Some(week.id)).unwrap();
        assert!(still_in_week.is_empty(), "week must be empty after rollover");

        // Step 5: the card appears in the backlog (week_id IS NULL).
        let backlog = db_list_cards_by_week(&db, None).unwrap();
        assert_eq!(backlog.len(), 1);
        assert_eq!(backlog[0].id, card.id);
        assert!(
            backlog[0].week_id.is_none(),
            "rolled-over card must have no week_id"
        );
    }

    // Scenario 2 — create 3 cards same day → verify ascending positions →
    // reorder one → verify new order is reflected in the DB.
    #[test]
    fn scenario_position_sequence_and_reorder() {
        let db = open_test_db();

        let week = db_get_or_create_week(&db, 2026, 9, "2026-02-23").unwrap();

        // Three cards inserted for the same day must get positions 1, 2, 3.
        let c1 = db_create_card(&db, "Alpha", &CardType::Task, Some(week.id), Some(1), None).unwrap();
        let c2 = db_create_card(&db, "Beta",  &CardType::Task, Some(week.id), Some(1), None).unwrap();
        let c3 = db_create_card(&db, "Gamma", &CardType::Task, Some(week.id), Some(1), None).unwrap();

        assert_eq!(c1.position, 1);
        assert_eq!(c2.position, 2);
        assert_eq!(c3.position, 3);

        // Reorder: move Gamma to position 1 (front).
        db_update_card(
            &db,
            c3.id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(1), // new position
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        // Read back and confirm Gamma is now position 1.
        let cards = db_list_cards_by_week(&db, Some(week.id)).unwrap();
        let gamma = cards.iter().find(|c| c.id == c3.id).unwrap();
        assert_eq!(gamma.position, 1, "Gamma must be at position 1 after reorder");

        // Alpha and Beta retain their original values (no cascading shift in
        // this app — the frontend manages that separately via explicit updates).
        let alpha = cards.iter().find(|c| c.id == c1.id).unwrap();
        let beta  = cards.iter().find(|c| c.id == c2.id).unwrap();
        assert_eq!(alpha.position, 1); // unchanged by this test's update
        assert_eq!(beta.position,  2);
    }
}
