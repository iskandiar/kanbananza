# Tray Revamp — Design Spec

**Date:** 2026-03-23
**Status:** Approved
**Amends:** `2026-03-23-menu-bar-icon-design.md`

---

## Overview

Two targeted changes to the existing tray implementation:

1. **Clock In/Out switches to day-level timer** — uses `time_entries` (same table as the day play button in the board), not `card_time_entries`.
2. **Colored dot badges on both icons** — tray icon shows green dot (running) or red dot (not running); dock badge clears when clocked in, shows remaining planned card count when clocked out.

---

## Change 1 — Day-Level Clock In/Out

### Behaviour

| State | Menu item | Click action |
|---|---|---|
| Not clocked in | `▶  Clock In` | Insert row into `time_entries` |
| Clocked in | `⏹  Clock Out · 42m` | Update open `time_entries` row, set `end_time` |

- Clock Out label shows elapsed minutes only — no card title (day timer has no associated card).
- If already clocked in when menu is opened, shows Clock Out (existing toggle logic unchanged).

### Queries

```sql
-- Detect active day timer
SELECT id, start_time FROM time_entries WHERE end_time IS NULL LIMIT 1;

-- Clock In
INSERT INTO time_entries (date, start_time) VALUES (?, datetime('now'));

-- Clock Out
UPDATE time_entries SET end_time = datetime('now') WHERE id = ?;
```

### Code changes in `src-tauri/src/tray.rs`

- `query_active_timer` — rewrite to query `time_entries` instead of `card_time_entries`. Return type changes from `Option<(String, i64)>` to `Option<i64>` (elapsed minutes only).
- `rebuild_menu` — Clock Out label becomes `format!("Clock Out · {}m", elapsed)`.
- `"clock_in"` event handler — replace `db_card_clock_in` call with direct SQL insert into `time_entries`.
- `"clock_out"` event handler — replace `card_time_entries` query with `time_entries WHERE end_time IS NULL`.
- Remove `query_most_recent_planned_card` (no longer needed). Remove its import of `db_card_clock_in` from `card_time_entries`.

---

## Change 2 — Colored Dot Badges

### Tray Icon (menu bar)

Two pre-rendered PNG variants replace the current placeholders:

| File | State | Appearance |
|---|---|---|
| `src-tauri/icons/tray-active.png` | Timer running | Icon + **green dot** (bottom-right corner) |
| `src-tauri/icons/tray-active@2x.png` | Timer running | Retina variant |
| `src-tauri/icons/tray-default.png` | Timer not running | Icon + **orange/red dot** (bottom-right corner) |
| `src-tauri/icons/tray-default@2x.png` | Timer not running | Retina variant |

Icons are 16×16 (standard) / 32×32 (@2x). Dots are ~4px diameter. The existing `update_badge_and_icon` swap logic is unchanged — only the image files change.

### Dock Badge

| State | Dock badge |
|---|---|
| Timer running | `set_badge_count(None)` — badge cleared |
| Timer not running | `set_badge_count(Some(n))` where n = today's planned card count |

This is the **inverse** of the current behaviour (which always shows the count regardless of timer state). Logic change is in `update_badge_and_icon` in `tray.rs`.

---

## Files Modified

| File | Change |
|---|---|
| `src-tauri/src/tray.rs` | Rewrite `query_active_timer`; update Clock In/Out handlers; update dock badge logic; remove `query_most_recent_planned_card` |
| `src-tauri/icons/tray-default.png` | Replace placeholder with red-dot variant |
| `src-tauri/icons/tray-default@2x.png` | Replace placeholder |
| `src-tauri/icons/tray-active.png` | Replace placeholder with green-dot variant |
| `src-tauri/icons/tray-active@2x.png` | Replace placeholder |

No schema changes. No new dependencies.

---

## Testing

- Manual: Clock In from tray → tray icon turns green dot, dock badge clears
- Manual: Clock Out from tray → tray icon turns red dot, dock badge shows remaining count
- Manual: Clock In from day play button, then open tray → shows Clock Out (timer already active)
- Rust unit test: update `active_timer_returns_card_title_when_clocked_in` → query `time_entries` instead; assert elapsed ≥ 0
