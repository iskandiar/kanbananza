# Tray Revamp Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Switch tray Clock In/Out to the day-level `time_entries` timer and add colored status dot badges to the tray and dock icons.

**Architecture:** All changes are in `src-tauri/src/tray.rs` (Rust). `query_active_timer` is rewritten to query `time_entries` instead of `card_time_entries`, returning `Option<(i64, i64)>` (row id + elapsed minutes). Icon files are replaced with pre-rendered variants that include a colored status dot. No schema changes, no new dependencies.

**Tech Stack:** Rust · rusqlite · Tauri v2 · Python 3 (Pillow) for icon generation

---

## File Structure

| File | Change |
|---|---|
| `src-tauri/src/tray.rs` | Rewrite `query_active_timer`; update handlers and badge logic; remove `query_most_recent_planned_card` |
| `src-tauri/icons/tray-default.png` | Replace with red-dot variant (16×16) |
| `src-tauri/icons/tray-default@2x.png` | Replace with red-dot variant (32×32) |
| `src-tauri/icons/tray-active.png` | Replace with green-dot variant (16×16) |
| `src-tauri/icons/tray-active@2x.png` | Replace with green-dot variant (32×32) |

---

## Background: how time tracking works in this app

There are **two separate time-tracking tables**:

- `time_entries` (migration 0004) — day-level timer, no card. `date TEXT, start_time TEXT, end_time TEXT`. This is what the ▶ play button in the day column uses.
- `card_time_entries` (migration 0005) — card-level timer, has `card_id`. Used for per-card time logging.

The tray currently (incorrectly) uses `card_time_entries`. This plan switches it to `time_entries`.

---

## Task 1: Rewrite `query_active_timer` to use `time_entries`

**Files:**
- Modify: `src-tauri/src/tray.rs` (functions: `query_active_timer`, `rebuild_menu`, test `active_timer_returns_card_title_when_clocked_in`, test `most_recent_planned_card_returns_card_id`)

The test DB already has `time_entries` — migration 0004 is run in `open_test_db()`.

- [ ] **Step 1: Rewrite the existing timer test to use `time_entries`**

In the `#[cfg(test)]` block at the bottom of `src-tauri/src/tray.rs`, replace the `active_timer_returns_card_title_when_clocked_in` test:

```rust
#[test]
fn active_timer_returns_entry_when_clocked_in() {
    let db = open_test_db();
    db.execute(
        "INSERT INTO time_entries (date, start_time) VALUES (?, datetime('now'))",
        ["2026-03-23"],
    )
    .unwrap();
    let inserted_id = db.last_insert_rowid();
    let result = query_active_timer(&db);
    assert!(result.is_some());
    let (entry_id, elapsed) = result.unwrap();
    assert_eq!(entry_id, inserted_id);
    assert!(elapsed >= 0);
}
```

Also **delete** the `most_recent_planned_card_returns_card_id` test entirely (the function it tests will be removed).

- [ ] **Step 2: Run the tests to confirm they fail**

```bash
cd src-tauri && cargo test tray 2>&1 | tail -20
```

Expected: `active_timer_returns_entry_when_clocked_in` FAILS (function returns wrong type), `most_recent_planned_card_returns_card_id` is gone.

- [ ] **Step 3: Rewrite `query_active_timer`**

Replace the existing `query_active_timer` function (lines ~86–105 in `tray.rs`):

```rust
/// Returns (entry_id, elapsed_minutes) for the active day timer, or None.
pub(crate) fn query_active_timer(db: &Connection) -> Option<(i64, i64)> {
    db.query_row(
        "SELECT id, start_time FROM time_entries WHERE end_time IS NULL LIMIT 1",
        [],
        |row| {
            let id: i64 = row.get(0)?;
            let start_time: String = row.get(1)?;
            Ok((id, start_time))
        },
    )
    .ok()
    .map(|(id, start_time)| {
        let elapsed = elapsed_minutes(&start_time);
        (id, elapsed)
    })
}
```

- [ ] **Step 4: Fix `rebuild_menu` to use the new return type**

`query_active_timer` now returns `Option<(i64, i64)>` instead of `Option<(String, i64)>`. Find the Clock In/Out section in `rebuild_menu` (~line 185) and update:

```rust
// Clock In / Clock Out
let clock_item = if let Some((_entry_id, elapsed)) = &timer {
    let label = format!("Clock Out · {}m", elapsed);
    MenuItemBuilder::with_id("clock_out", label).build(app)?
} else {
    MenuItemBuilder::with_id("clock_in", "Clock In").build(app)?
};
```

(The `_entry_id` is prefixed with `_` because `rebuild_menu` doesn't need it — only the event handler does.)

- [ ] **Step 5: Remove `query_most_recent_planned_card`**

Delete the entire `query_most_recent_planned_card` function (~lines 122–133). It will be unused after Task 2.

- [ ] **Step 6: Run all tray tests**

```bash
cd src-tauri && cargo test tray 2>&1 | tail -20
```

Expected: all 9 tests pass (one renamed, one deleted, 7 unchanged). If any fail, read the error carefully before changing anything.

- [ ] **Step 7: Cargo check**

```bash
cd src-tauri && cargo check 2>&1 | tail -10
```

Expected: `Finished` with no errors. Fix any compile errors before proceeding.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/tray.rs
git commit -m "refactor: query_active_timer uses time_entries, remove most_recent_planned_card [logic]"
```

---

## Task 2: Update Clock In / Clock Out event handlers

**Files:**
- Modify: `src-tauri/src/tray.rs` (the `"clock_in"` and `"clock_out"` arms inside `setup_tray`)

The event handlers are inside `on_menu_event(|app, event| { match event.id().as_ref() { ... } })` in `setup_tray`.

- [ ] **Step 1: Replace the `"clock_in"` handler**

Find the `"clock_in" =>` arm (currently ~line 282). Replace it entirely:

```rust
"clock_in" => {
    let db_state = app.state::<DbState>();
    {
        let Ok(db) = db_state.0.lock() else { return };
        // Guard: only insert if no open entry exists
        let already_open: bool = db
            .query_row(
                "SELECT COUNT(*) FROM time_entries WHERE end_time IS NULL",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if !already_open {
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let _ = db.execute(
                "INSERT INTO time_entries (date, start_time) VALUES (?, datetime('now'))",
                [&today],
            );
        }
    } // DB lock released here
    update_badge_and_icon(app);
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = rebuild_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
```

- [ ] **Step 2: Replace the `"clock_out"` handler**

Find the `"clock_out" =>` arm (currently ~line 308). Replace it entirely:

```rust
"clock_out" => {
    let db_state = app.state::<DbState>();
    {
        let Ok(db) = db_state.0.lock() else { return };
        let entry_id: Option<i64> = db
            .query_row(
                "SELECT id FROM time_entries WHERE end_time IS NULL LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();
        if let Some(id) = entry_id {
            let _ = db.execute(
                "UPDATE time_entries SET end_time = datetime('now') WHERE id = ?",
                [id],
            );
        }
    } // DB lock released here
    update_badge_and_icon(app);
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = rebuild_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
```

- [ ] **Step 3: Remove the now-unused import**

At the top of the `"clock_in"` handler, the old code imported `crate::commands::card_time_entries::db_card_clock_in`. Since we no longer call it, remove that usage. Also check the top of the file for any `use crate::commands::card_time_entries` import and remove it if it's no longer referenced anywhere in the file.

- [ ] **Step 4: Cargo check**

```bash
cd src-tauri && cargo check 2>&1 | tail -10
```

Expected: `Finished`. Fix any compile errors.

- [ ] **Step 5: Run tray tests**

```bash
cd src-tauri && cargo test tray 2>&1 | tail -15
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/tray.rs
git commit -m "feat: tray clock in/out uses day-level time_entries [logic]"
```

---

## Task 3: Invert dock badge logic

**Files:**
- Modify: `src-tauri/src/tray.rs` (`update_badge_and_icon` function, ~lines 238–263)

Currently the dock badge shows the planned card count regardless of timer state. The new behaviour: clear badge when clocked in, show count when not clocked in.

- [ ] **Step 1: Update `update_badge_and_icon`**

Find the badge update section inside `update_badge_and_icon`:

```rust
// Current code (to replace):
if let Some(win) = app.get_webview_window("main") {
    let _ = win.set_badge_count(if count > 0 { Some(count) } else { None });
}
```

Replace with:

```rust
// New: clear badge when clocked in; show remaining count when not clocked in
if let Some(win) = app.get_webview_window("main") {
    let badge = if timer_active {
        None
    } else if count > 0 {
        Some(count)
    } else {
        None
    };
    let _ = win.set_badge_count(badge);
}
```

- [ ] **Step 2: Cargo check**

```bash
cd src-tauri && cargo check 2>&1 | tail -10
```

Expected: `Finished`.

- [ ] **Step 3: Run all Rust tests**

```bash
cd src-tauri && cargo test 2>&1 | tail -15
```

Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tray.rs
git commit -m "feat: dock badge clears when clocked in, shows count when not [logic]"
```

---

## Task 4: Replace icon files with colored dot variants

**Files:**
- Replace: `src-tauri/icons/tray-default.png` (16×16, red dot)
- Replace: `src-tauri/icons/tray-default@2x.png` (32×32, red dot)
- Replace: `src-tauri/icons/tray-active.png` (16×16, green dot)
- Replace: `src-tauri/icons/tray-active@2x.png` (32×32, green dot)

The existing placeholder PNGs are already loaded by `update_badge_and_icon`. We just replace the files; no Rust code changes needed.

The script reads each existing icon and overlays a 4px (8px @2x) filled circle in the bottom-right corner:
- `tray-active` variants: **green** `(0, 200, 0, 255)`
- `tray-default` variants: **orange-red** `(220, 80, 0, 255)`

- [ ] **Step 1: Check Pillow is available**

```bash
python3 -c "from PIL import Image, ImageDraw; print('ok')"
```

If this fails, install it: `pip3 install Pillow`

- [ ] **Step 2: Run the icon generation script**

Create and run this script (you can paste it directly into your terminal):

```python
python3 << 'EOF'
from PIL import Image, ImageDraw
import os

base = "src-tauri/icons"

configs = [
    ("tray-active.png",    16, (0, 200, 0, 255)),
    ("tray-active@2x.png", 32, (0, 200, 0, 255)),
    ("tray-default.png",    16, (220, 80, 0, 255)),
    ("tray-default@2x.png", 32, (220, 80, 0, 255)),
]

for filename, size, color in configs:
    path = os.path.join(base, filename)
    # Load existing icon (preserve whatever is already there)
    img = Image.open(path).convert("RGBA")
    # Ensure correct size
    if img.size != (size, size):
        img = img.resize((size, size), Image.LANCZOS)
    # Draw filled circle dot in bottom-right corner
    dot_radius = 2 if size == 16 else 4
    margin = 1
    x1 = size - dot_radius * 2 - margin
    y1 = size - dot_radius * 2 - margin
    x2 = size - margin
    y2 = size - margin
    draw = ImageDraw.Draw(img)
    draw.ellipse([x1, y1, x2, y2], fill=color)
    img.save(path)
    print(f"Updated {filename}")

print("Done.")
EOF
```

Expected output:
```
Updated tray-active.png
Updated tray-active@2x.png
Updated tray-default.png
Updated tray-default@2x.png
Done.
```

- [ ] **Step 3: Verify the icons visually**

```bash
open src-tauri/icons/tray-active.png
open src-tauri/icons/tray-default.png
```

Confirm each icon has a small colored dot in the bottom-right corner. Active = green, Default = orange-red.

- [ ] **Step 4: Cargo check (icons are compiled in via `include_bytes!`, verify no issues)**

```bash
cd src-tauri && cargo check 2>&1 | tail -5
```

Expected: `Finished`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/icons/tray-active.png src-tauri/icons/tray-active@2x.png \
        src-tauri/icons/tray-default.png src-tauri/icons/tray-default@2x.png
git commit -m "feat: tray icon colored dot badges — green (active) / red (inactive) [boilerplate]"
```

---

## Manual Verification

After all tasks:

1. `pnpm tauri dev` — app launches with tray icon visible in menu bar
2. Tray icon shows red/orange dot (not clocked in)
3. Click tray → "Clock In" visible
4. Click Clock In → icon switches to green dot, dock badge clears
5. Click tray → "Clock Out · Xm" visible
6. Click Clock Out → icon switches back to red/orange dot, dock badge shows remaining count
7. Use the ▶ play button in the day column of the board → tray should reflect clocked-in state after next `refresh_tray` call (triggered automatically by `card_time_entries.ts`... but wait — the play button calls `clockIn` from `time_entries.ts`, not `card_time_entries.ts`. Verify `refreshTray()` is called after `clockIn` in `src/lib/api/time_entries.ts` — if not, add it.)

> **Note on step 7:** `src/lib/api/time_entries.ts` does NOT call `refreshTray()` — this is fixed in Task 5.

---

## Task 5: Call `refreshTray` from `time_entries.ts`

**Files:**
- Modify: `src/lib/api/time_entries.ts`

When the user clocks in/out using the ▶ play button in the board (which calls `clockIn`/`clockOut` from `time_entries.ts`), the tray must update to reflect the new state. Currently it doesn't because `refreshTray()` is never called from this file.

The pattern to follow is already established in `src/lib/api/card_time_entries.ts` — fire-and-forget after the await.

- [ ] **Step 1: Add `refreshTray` import and calls to `time_entries.ts`**

Open `src/lib/api/time_entries.ts`. Add the import at the top and fire-and-forget calls to `clockIn` and `clockOut`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { TimeEntry } from '$lib/types';
import { refreshTray } from '$lib/api/tray';

export async function clockIn(date: string): Promise<TimeEntry> {
  const entry = await invoke<TimeEntry>('clock_in', { date });
  refreshTray();
  return entry;
}

export async function clockOut(entryId: number): Promise<TimeEntry> {
  const entry = await invoke<TimeEntry>('clock_out', { entryId });
  refreshTray();
  return entry;
}
```

Leave all other functions (`listTimeEntries`, `updateTimeEntry`, `deleteTimeEntry`, `createManualTimeEntry`, `listTimeEntriesForWeek`) unchanged.

- [ ] **Step 2: TypeScript check**

```bash
pnpm check 2>&1 | tail -10
```

Expected: no errors. Fix any type errors before proceeding.

- [ ] **Step 3: Commit**

```bash
git add src/lib/api/time_entries.ts
git commit -m "feat: refreshTray after day-level clock in/out [logic]"
```
