# UX Improvements — Implementation Plan

## Context
A UX audit identified 9 improvements across the board view. All are frontend-only Svelte changes — no Rust/SQLite changes needed.

## Approved Changes

1. Done cards — Linear collapse with time consumed
2. Rollover confirmation dialog with card count
3. Always-visible checkbox for mark-done (left edge of card)
4. Backlog pinning (always-visible sidebar)
5. Highlight today's column
6. "Impact" → "Priority" label in edit form UI
7. LoadIndicator: split bar — solid (meetings) + hatched (tasks)
8. Backlog count shown on closed toggle button
9. AI description expands on hover (hover:line-clamp-none)

---

## Critical Files

- `src/lib/components/Card.svelte` — items 3, 6, 9
- `src/lib/components/DayColumn.svelte` — items 1, 5, 7
- `src/lib/components/LoadIndicator.svelte` — item 7
- `src/lib/components/WeekBoard.svelte` — items 2, 4, 8
- `src/lib/components/BacklogSidebar.svelte` — item 4
- `src/lib/components/WeekHeader.svelte` — item 2
- `src/routes/board/+page.svelte` — item 5

---

## Commit 1: Today highlight + backlog count on toggle

### board/+page.svelte
Add `isToday: boolean` to each day in the `days()` derived:
```ts
const todayDOW = () => {
  const w = boardStore.currentWeek;
  if (!w) return null;
  const today = new Date();
  const monday = new Date(w.start_date);
  const diff = Math.floor((today.getTime() - monday.getTime()) / 86400000);
  return diff >= 0 && diff < 5 ? diff + 1 : null; // 1-5, null if not this week
};
// In days array:
isToday: todayDOW() === i + 1
```
Pass `isToday` through `WeekBoard` → `DayColumn` props (add to both).

### DayColumn.svelte
Accept new `isToday: boolean = false` prop.
- Column background: `class:bg-[var(--color-surface)]/30={isToday}` (very subtle)
- Date label (`date` text): `class:text-[var(--color-accent)]={isToday}` instead of muted
- Day label (`MON`/`TUE`): `class:text-[var(--color-text)]={isToday}` instead of text-muted

### WeekBoard.svelte — closed sidebar button
Change from `≡` to `≡ ({backlogCards.length})` showing count when closed.

---

## Commit 2: Card — checkbox + priority label + AI description expand

### Card.svelte — layout restructure for checkbox

Restructure outer layout to flex-row:
```html
<div class="group flex items-start gap-2 rounded-md border … px-2 py-2">
  <!-- Left: checkbox, always visible -->
  <button
    data-no-dnd="true"
    onclick={handleToggleDone}
    class="flex-shrink-0 mt-0.5 w-4 h-4 rounded-full border …"
    aria-label={card.status === 'done' ? 'Undo done' : 'Mark done'}
  >
    {#if card.status === 'done'}✓{/if}
  </button>
  <!-- Right: existing content, fades when done -->
  <div class="flex-1 min-w-0" class:opacity-40={card.status === 'done'}>
    … (all existing card content)
  </div>
</div>
```

Checkbox styling:
- Planned: `border-[var(--color-border)] text-transparent` (empty circle, opacity-40 normally, opacity-70 on group-hover)
- Done: `border-[var(--color-done)] bg-[var(--color-done-bg)] text-[var(--color-done)]` (filled green)

Add `handleToggleDone()`:
```ts
function handleToggleDone() {
  if (card.status === 'done') {
    boardStore.updateCard(card.id, { status: 'planned' });
  } else {
    onMarkDone(card.id);
  }
}
```
Remove the ✓ and ↩ buttons from the hover-only actions row (checkbox replaces them). Keep ✎ edit and ↗ open link.

### Card.svelte — Priority label in edit form
Add `<p class="text-xs text-[var(--color-muted)]">Priority</p>` above the impact toggle buttons in the edit form.

### Card.svelte — AI description expand on hover
Change description paragraph from:
```html
<p class="text-xs … line-clamp-2 …">
```
to:
```html
<p class="text-xs … line-clamp-2 hover:line-clamp-none transition-all cursor-default" data-no-dnd="true">
```

---

## Commit 3: LoadIndicator split bar + Done card collapse

### LoadIndicator.svelte — new props + segmented bar

Change props from `scheduledHours` to `meetingHours` and `taskHours`:
```ts
let { meetingHours, taskHours, availableHours } = $props();
const total = $derived(meetingHours + taskHours);
const ratio = $derived(availableHours > 0 ? total / availableHours : 0);
const meetingPct = $derived(Math.min((meetingHours / availableHours) * 100, 100));
const taskPct = $derived(Math.min((taskHours / availableHours) * 100, Math.max(0, 100 - meetingPct)));
```

New bar layout (two segments):
```html
<div class="h-1.5 flex-1 rounded-full bg-[var(--color-border)] overflow-hidden flex">
  <!-- Meetings: solid -->
  <div class="h-full {barColor} transition-all rounded-l-full" style="width: {meetingPct}%"></div>
  <!-- Tasks: same color + hatched overlay -->
  <div
    class="h-full {barColor} opacity-70 transition-all"
    style="width: {taskPct}%; background-image: repeating-linear-gradient(45deg, transparent, transparent 2px, rgba(0,0,0,0.2) 2px, rgba(0,0,0,0.2) 4px)"
  ></div>
</div>
<span>{total.toFixed(1)}h / {availableHours}h</span>
```

### DayColumn.svelte — pass new props + done collapse

Update `LoadIndicator` call: pass `{meetingHours}` and `{taskHours}` instead of `{scheduledHours}`.

Replace `localTasks` with `localPendingTasks` — done tasks are derived from the `tasks` prop directly and are excluded from DnD:

```ts
let showDone = $state(false);

// Done tasks: derived from props (stable, not part of DnD)
const doneTasks = $derived(tasks.filter(c => c.status === 'done'));
const doneHours = $derived(doneTasks.reduce((s, c) => s + (c.time_estimate ?? 0), 0));

// Pending tasks: local optimistic copy for DnD reordering
let localPendingTasks = $state<Card[]>([]);
$effect(() => { localPendingTasks = tasks.filter(c => c.status !== 'done'); });

function handleDndConsider(e: CustomEvent<{ items: Card[] }>) {
  localPendingTasks = e.detail.items;
}
function handleDndFinalize(e: CustomEvent<{ items: Card[] }>) {
  localPendingTasks = e.detail.items;
  localPendingTasks.forEach((card, i) => {
    if (card.day_of_week !== dayOfWeek || card.week_id !== weekId || card.position !== i) {
      onMoveCard(card.id, weekId, dayOfWeek, i);
    }
  });
}
```

DnD zone `items` binds to `localPendingTasks`. Done tasks are rendered in a separate section below.

Below the DnD zone, add collapse section:
```html
{#if doneTasks.length > 0}
  <div>
    <button
      onclick={() => (showDone = !showDone)}
      class="text-xs text-[var(--color-muted)] hover:text-[var(--color-text)] flex items-center gap-1 py-0.5"
    >
      {showDone ? '▾' : '▸'}
      {doneTasks.length} done
      {#if doneHours > 0}· {doneHours.toFixed(1)}h consumed{/if}
    </button>
    {#if showDone}
      <div class="flex flex-col gap-1.5 mt-1">
        {#each doneTasks as card (card.id)}
          <CardComponent {card} {onMarkDone} />
        {/each}
      </div>
    {/if}
  </div>
{/if}
```

---

## Commit 4: Rollover confirmation + Backlog pin

### WeekHeader.svelte — rollover confirmation

Add `unfinishedCount: number` prop.
Add `rolloverConfirming = $state(false)` local state.

Replace single rollover button with two-state UI:
```html
{#if !rolloverConfirming}
  <button onclick={() => (rolloverConfirming = true)} …>Rollover</button>
{:else}
  <span class="text-xs text-[var(--color-muted)]">Move {unfinishedCount} cards?</span>
  <button onclick={confirmRollover} class="… text-amber-400 …">Confirm</button>
  <button onclick={() => (rolloverConfirming = false)} class="…">Cancel</button>
{/if}
```
`confirmRollover` calls `onRollover()` then resets `rolloverConfirming = false`.

### WeekBoard.svelte — compute unfinished count + pass to WeekHeader

```ts
const unfinishedCount = $derived(
  days.flatMap(d => d.tasks).filter(c => c.status === 'planned').length
);
```
Pass to `WeekHeader`: `{unfinishedCount}`.

### WeekBoard.svelte — backlog pin state

Add `backlogPinned = $state(false)`.

- When `backlogPinned` is true: `backlogOpen` is forced true
- Pass `isPinned`, `onPin`, `onUnpin` to `BacklogSidebar`
- When pinned, hide the `≡` toggle button (sidebar is always visible)

### BacklogSidebar.svelte — pin UI

Add props:
```ts
isPinned: boolean;
onPin: () => void;
onUnpin: () => void;
```

Header buttons (right side of header):
- **Pin button** (always shown): text pin icon — active (accent color) when pinned, muted when not
- **Close button** (only shown when not pinned): `×`

---

## Verification

1. `pnpm tauri dev` — full app test
2. Today highlight: today's column has accent-colored date label, others do not
3. Done collapse: mark 2 tasks done → disappear from list → "2 done · Xh consumed" appears → click to expand → both show
4. Checkbox: click empty circle → card done + green check + content fades; click again → reverts to planned
5. AI description hover: hover over clamped description → it expands fully
6. Load bar: day with meetings shows solid left segment + hatched right segment for tasks
7. Rollover: click → "Move N cards?" → Confirm triggers rollover; Cancel resets
8. Backlog pin: click pin → sidebar stays open; click again → sidebar can be closed
9. Backlog count: close sidebar → toggle button shows `≡ (N)`

---

## Commit labels

- `[boilerplate]` — Pure visual/layout changes with no data logic (items 5, 6, 8, 9)
- `[logic]` — Changes involving computed state or new interactions (items 1, 2, 3, 4, 7)
