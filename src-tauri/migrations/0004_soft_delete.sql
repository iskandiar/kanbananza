-- 0004: soft delete for cards, sync_skip table, time_entries table

-- Soft delete column (ignored if already exists — ALTER TABLE is idempotent via lib.rs pattern)
-- Added via ALTER TABLE in lib.rs below

-- Sync skip table: tracks items user chose to never import
CREATE TABLE IF NOT EXISTS sync_skip (
    id INTEGER PRIMARY KEY,
    external_id TEXT NOT NULL UNIQUE,
    skipped_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Time entries table: clock in/out sessions
CREATE TABLE IF NOT EXISTS time_entries (
    id INTEGER PRIMARY KEY,
    date TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
