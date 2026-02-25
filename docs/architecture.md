# Architecture

## Overview

Rust (Tauri) backend owns all I/O: SQLite reads/writes, OS keychain access, and card/week business logic. The Svelte frontend is purely presentational — it renders state and fires commands, it does not compute or persist anything itself.

## Rust ↔ JS Boundary

All `invoke()` calls must go through wrappers in `src/lib/api/`. Direct `invoke()` calls in components are forbidden — the api layer is the single choke point for argument shaping and error normalisation.

> `src/lib/api/` wrappers are not yet implemented (M4 milestone).

## Database

- Engine: rusqlite (SQLite), single file at `{app_data_dir}/kanbananza.db`
- State: `DbState(Mutex<Connection>)` registered with `app.manage()` on startup. Every command receives it via `State<DbState>` and locks the mutex for the duration of the query.
- Migrations: `conn.execute_batch(include_str!("../migrations/0001_initial.sql"))` runs on every startup — SQL must be idempotent (`CREATE TABLE IF NOT EXISTS`, `INSERT OR IGNORE`, etc.)

## Keychain

Credentials stored via the `keyring` crate → OS secret store (macOS Keychain, Linux libsecret, Windows Credential Manager).

Naming convention: `kanbananza.{service}.{key}`
Example: `kanbananza.linear.api_token`

`get_secret` returns `Ok(None)` on a missing entry so callers treat "not found" as "not configured" without separate error handling.

## Card Backlog Rule

`week_id IS NULL` = global backlog. `rollover_week` moves all `status='planned'` cards for a given week to backlog by setting `week_id=NULL, day_of_week=NULL`. Backlog cards persist until the user schedules them onto a day.

## Component Hierarchy

```
WeekBoard
├── WeekHeader              ← week date range + prev/next navigation
├── DayColumn × 5           ← one per working day (Mon–Fri)
│   ├── LoadIndicator       ← scheduled vs available hours bar
│   ├── Card × N (meetings) ← time-anchored, not draggable
│   ├── Card × N (tasks)    ← draggable
│   └── QuickAdd            ← inline card creation
└── BacklogSidebar          ← slide-in panel
    ├── QuickAdd
    └── Card × N
```

`Card` is a pure display component. Meeting cards parse `metadata` JSON to extract `start_time`; all other types ignore `metadata`.
