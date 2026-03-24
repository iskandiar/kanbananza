# Calendar Sync Error Banner — Design Spec
_Date: 2026-03-24_

## Overview

When Google Calendar becomes disconnected (token expired, revoked, or auth failure), the main board shows a persistent amber banner below the header so the user knows syncing is broken and can navigate to Settings to reconnect.

---

## Behaviour

- The banner is **persistent** — it stays visible until the next successful calendar sync clears the error.
- It appears on the main board page only (`/board`).
- It does **not** have a manual dismiss button.
- It is **not** shown for non-calendar sync errors (GitLab, Linear, etc.).

---

## State — `boardStore`

Add one new field:

```typescript
calendarSyncError: string | null = null;
```

### Setting the error

Update the existing `calendar://synced` listener in `loadCurrentWeek` to read the full payload:

```typescript
listen<{ count: number; error: string | null }>('calendar://synced', (event) => {
  if (event.payload.error) {
    this.calendarSyncError = event.payload.error;
  } else {
    this.calendarSyncError = null;
    this._loadCards();
  }
})
```

Also add a listener for `calendar://error` (emitted when the OAuth flow itself fails):

```typescript
listen<{ message: string }>('calendar://error', (event) => {
  this.calendarSyncError = event.payload.message;
})
```

Both listeners are set up once in `loadCurrentWeek`, guarded the same way as the existing `_calendarUnlisten` pattern.

### Clearing the error

- Cleared to `null` when `calendar://synced` arrives with `error: null`.
- Not explicitly cleared on disconnect — the banner disappears naturally on app restart or next successful sync.

---

## UI — Banner component

A slim inline banner rendered in `src/routes/board/+page.svelte`, placed **above** `<WeekBoard>` and only when `boardStore.calendarSyncError` is non-null.

### Visual

- Background: amber-toned (`bg-amber-900/40 border-amber-700/40`)
- Single line, full width
- Content: `⚠ Google Calendar disconnected — {truncated error} · Settings →`
- "Settings →" is an `<a href="/settings">` link
- Error message truncated to ~60 characters with CSS `truncate` to prevent overflow

### Markup (inline in `board/+page.svelte`, no new component needed)

```svelte
{#if boardStore.calendarSyncError}
  <div class="flex items-center gap-2 px-4 py-1.5 text-xs border-b bg-amber-900/40 border-amber-700/40 text-amber-200">
    <span>⚠</span>
    <span class="truncate flex-1">Google Calendar disconnected — {boardStore.calendarSyncError}</span>
    <a href="/settings" class="shrink-0 underline hover:text-amber-100 transition-colors">Settings →</a>
  </div>
{/if}
```

---

## Files changed

| File | Change |
|---|---|
| `src/lib/stores/board.svelte.ts` | Add `calendarSyncError` field; update `calendar://synced` listener to read payload; add `calendar://error` listener |
| `src/routes/board/+page.svelte` | Render banner above `<WeekBoard>` when `calendarSyncError` is set |

---

## Out of scope

- Banners for GitLab, Linear, or other integrations.
- Manual dismiss button.
- Persisting the error across app restarts (in-memory only).
