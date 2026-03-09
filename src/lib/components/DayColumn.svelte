<script lang="ts">
  import { onMount, onDestroy, untrack } from 'svelte';
  import type { Card, TimeEntry } from '$lib/types';
  import { sumHours } from '$lib/utils';
  import { dndzone } from 'svelte-dnd-action';
  import CardComponent from './Card.svelte';
  import LoadIndicator from './LoadIndicator.svelte';
  import QuickAdd from './QuickAdd.svelte';
  import * as timeApi from '$lib/api/time_entries';

  type DndCard = Card & { dragDisabled?: boolean };

  let {
    label,
    date,
    displayDate,
    dayOfWeek,
    weekId,
    meetings = [],
    tasks = [],
    availableHours,
    isToday = false,
    onAddCard,
    onMoveCard,
    onMarkDone,
    onCardCreated,
    onMoveToNextWeek,
    onClockedUpdate
  }: {
    label: string;
    date: string;       // ISO YYYY-MM-DD — used for ALL API calls
    displayDate: string; // "Mar 6" etc — used only for display
    dayOfWeek: number;
    weekId: number | null;
    meetings: Card[];
    tasks: Card[];
    availableHours: number;
    isToday?: boolean;
    onAddCard: (title: string) => void;
    onMoveCard: (cardId: number, weekId: number | null, dayOfWeek: number | null, position: number) => void;
    onMarkDone: (cardId: number) => void;
    onCardCreated?: (card: Card) => void;
    onMoveToNextWeek?: (id: number) => void;
    onClockedUpdate?: (hours: number) => void;
  } = $props();

  // Include both tasks and meetings in load calculation
  const doneHoursByType = $derived(
    sumHours([...tasks.filter(t => t.status === 'done'), ...meetings.filter(m => m.status === 'done')])
  );
  const plannedHoursByType = $derived(
    sumHours([...tasks.filter(t => t.status !== 'done'), ...meetings.filter(m => m.status !== 'done')])
  );

  // Merged pending items for DnD: tasks (draggable) + meetings (non-draggable), sorted by position
  let localPendingItems = $state<DndCard[]>([]);
  $effect(() => {
    const pendingT = tasks.filter(t => t.status !== 'done');
    const pendingM = meetings.filter(m => m.status !== 'done').map(m => ({ ...m, dragDisabled: true }));
    localPendingItems = [...pendingT, ...pendingM].sort((a, b) => (a.position ?? 0) - (b.position ?? 0));
  });

  // Done tasks/meetings for collapsed section
  const doneTasks = $derived(tasks.filter(t => t.status === 'done'));
  const doneMeetings = $derived(meetings.filter(m => m.status === 'done'));
  let showDone = $state(false);

  function handleDndConsider(e: CustomEvent<{ items: DndCard[] }>) {
    localPendingItems = e.detail.items;
  }

  function handleDndFinalize(e: CustomEvent<{ items: DndCard[] }>) {
    localPendingItems = e.detail.items;
    localPendingItems.forEach((card, i) => {
      if (card.card_type === 'meeting') return; // meetings keep their own position
      // Use column's weekId (not card's) so backlog→column drops get the right week assigned
      if (card.day_of_week !== dayOfWeek || card.week_id !== weekId || card.position !== i) {
        onMoveCard(card.id, weekId, dayOfWeek, i);
      }
    });
  }

  // Clock state (only active for today's column)
  let activeEntry = $state<TimeEntry | null>(null);
  let entries = $state<TimeEntry[]>([]);
  let showLog = $state(false);
  let elapsedSeconds = $state(0);
  let clockTick: ReturnType<typeof setInterval> | null = null;
  let addingEntry = $state(false);
  let newEntryStart = $state('');
  let newEntryEnd = $state('');
  let addError = $state<string | null>(null);
  let confirmingEntryId = $state<number | null>(null);

  // Today's date in YYYY-MM-DD format
  const todayDate = new Date().toISOString().slice(0, 10);

  $effect(() => {
    const hours = totalLoggedHours(); // reads `entries` — tracked
    untrack(() => onClockedUpdate?.(hours)); // callback not tracked — no loop
  });

  function formatElapsed(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  onMount(async () => {
    entries = await timeApi.listTimeEntries(date);
    if (isToday) {
      activeEntry = entries.find(e => e.end_time === null) ?? null;

      if (activeEntry) {
        const startMs = parseSqliteUtc(activeEntry.start_time).getTime();
        elapsedSeconds = Math.floor((Date.now() - startMs) / 1000);
      }

      clockTick = setInterval(() => {
        if (activeEntry) {
          const startMs = parseSqliteUtc(activeEntry.start_time).getTime();
          elapsedSeconds = Math.floor((Date.now() - startMs) / 1000);
        }
      }, 1000);
    }
  });

  onDestroy(() => {
    if (clockTick) clearInterval(clockTick);
  });

  async function handleClockIn() {
    activeEntry = await timeApi.clockIn(date);
    entries = await timeApi.listTimeEntries(date);
    elapsedSeconds = 0;
  }

  async function handleClockOut() {
    if (!activeEntry) return;
    activeEntry = await timeApi.clockOut(activeEntry.id);
    entries = await timeApi.listTimeEntries(date);
    activeEntry = null;
    elapsedSeconds = 0;
  }

  async function handleDeleteEntry(id: number) {
    await timeApi.deleteTimeEntry(id);
    entries = await timeApi.listTimeEntries(date);
    if (activeEntry?.id === id) {
      activeEntry = null;
      elapsedSeconds = 0;
    }
  }

  async function handleUpdateEntryTime(id: number, field: 'start' | 'end', localTime: string) {
    if (!localTime) return;
    const utcDt = toUtcDatetime(localTime);
    if (field === 'start') {
      await timeApi.updateTimeEntry(id, utcDt, undefined, undefined);
    } else {
      await timeApi.updateTimeEntry(id, undefined, utcDt, undefined);
    }
    entries = await timeApi.listTimeEntries(date);
    if (activeEntry) {
      activeEntry = entries.find(e => e.end_time === null) ?? null;
    }
  }

  async function handleAddManualEntry() {
    if (!newEntryStart) return;
    addError = null;
    const startUtc = toUtcDatetime(newEntryStart);
    const endUtc = newEntryEnd ? toUtcDatetime(newEntryEnd) : undefined;
    try {
      await timeApi.createManualTimeEntry(date, startUtc, endUtc);
      entries = await timeApi.listTimeEntries(date);
      addingEntry = false;
      newEntryStart = '';
      newEntryEnd = '';
    } catch (e) {
      addError = e instanceof Error ? e.message : String(e);
    }
  }

  function totalLoggedHours(): number {
    return entries.reduce((sum, e) => {
      const start = parseSqliteUtc(e.start_time).getTime();
      const end = e.end_time ? parseSqliteUtc(e.end_time).getTime() : Date.now();
      return sum + (end - start) / 3600000;
    }, 0);
  }

  // Parse SQLite UTC datetime string (no timezone) to JS Date correctly
  function parseSqliteUtc(s: string): Date {
    return new Date(s.replace(' ', 'T') + 'Z');
  }

  // Get local "HH:MM" from a SQLite UTC datetime string
  function toLocalTime(s: string): string {
    const d = parseSqliteUtc(s);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false });
  }

  // Convert local "HH:MM" + a given ISO date to SQLite UTC datetime string
  function toUtcDatetime(localTime: string, isoDate: string = date): string {
    const d = new Date(`${isoDate}T${localTime}:00`);
    return d.toISOString().slice(0, 19).replace('T', ' ');
  }
</script>

<div
  class="flex flex-col min-w-0 flex-1 border-r border-[var(--color-glass-border)] last:border-r-0 backdrop-blur-[2px] border-t-2 {isToday ? 'bg-[var(--color-glass-bg)] border-t-[var(--color-accent)]' : 'border-t-transparent'}"
>
  <!-- Static header -->
  <div class="px-3 pt-3 pb-2 shrink-0">
    <div class="flex items-start justify-between gap-2">
      <div class="min-w-0">
        <p
          class="text-xs uppercase tracking-wide {isToday ? 'font-bold text-[var(--color-text)]' : 'font-semibold text-[var(--color-text-muted)]'}"
        >{label}</p>
        <p
          class="text-xs text-[var(--color-muted)]"
          class:text-[var(--color-accent)]={isToday}
        >{displayDate}{#if entries.length > 0} · {totalLoggedHours().toFixed(1)}h{/if}</p>
      </div>

      <div class="flex items-center gap-1 shrink-0 mt-0.5">
        {#if isToday}
          {#if activeEntry}
            <span class="text-xs tabular-nums font-mono text-[var(--color-accent)]">{formatElapsed(elapsedSeconds)}</span>
            <button
              onclick={handleClockOut}
              class="text-xs px-1 py-px rounded bg-red-500/15 text-red-300 hover:bg-red-500/25 transition-colors leading-none"
              title="Clock out"
            >■</button>
          {:else}
            <button
              onclick={handleClockIn}
              class="text-xs px-1 py-px rounded bg-[var(--color-accent)]/10 text-[var(--color-muted)] hover:text-[var(--color-accent)] hover:bg-[var(--color-accent)]/15 transition-colors leading-none"
              title="Clock in"
            >▶</button>
          {/if}
        {/if}
        <button
          onclick={() => { showLog = !showLog; confirmingEntryId = null; }}
          class="text-xs text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors px-0.5"
          title="Time log"
        >≡</button>
      </div>
    </div>

    {#if showLog}
      <div class="mt-2 p-2 bg-[var(--color-bg)] border border-[var(--color-glass-border)] rounded text-xs">
        <div class="text-[var(--color-muted)] mb-1.5">{isToday ? "Today's" : displayDate} log · {totalLoggedHours().toFixed(1)}h</div>
        {#if entries.length === 0}
          <p class="text-[var(--color-muted)] italic">No entries yet</p>
        {:else}
          {#each entries as entry (entry.id)}
            <div class="flex items-center gap-1 py-0.5">
              <input
                type="time"
                value={toLocalTime(entry.start_time)}
                onchange={(e) => handleUpdateEntryTime(entry.id, 'start', e.currentTarget.value)}
                class="text-[0.65rem] bg-transparent border-b border-[var(--color-border)] text-[var(--color-text-muted)] focus:outline-none focus:border-[var(--color-accent)] w-14 tabular-nums"
              />
              <span class="text-[var(--color-muted)]">–</span>
              <input
                type="time"
                value={entry.end_time ? toLocalTime(entry.end_time) : ''}
                onchange={(e) => handleUpdateEntryTime(entry.id, 'end', e.currentTarget.value)}
                placeholder="now"
                class="text-[0.65rem] bg-transparent border-b border-[var(--color-border)] text-[var(--color-text-muted)] focus:outline-none focus:border-[var(--color-accent)] w-14 tabular-nums"
              />
              {#if isToday}
                <button
                  onclick={() => handleDeleteEntry(entry.id)}
                  class="ml-auto text-red-400/50 hover:text-red-400 transition-colors"
                >×</button>
              {:else if confirmingEntryId === entry.id}
                <span class="ml-auto text-[0.6rem] text-rose-400">Delete?</span>
                <button
                  onclick={async () => { await handleDeleteEntry(entry.id); confirmingEntryId = null; }}
                  class="text-rose-400 hover:text-rose-300 transition-colors text-xs leading-none"
                  title="Confirm delete"
                >✓</button>
                <button
                  onclick={() => (confirmingEntryId = null)}
                  class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors text-xs leading-none"
                  title="Cancel"
                >×</button>
              {:else}
                <button
                  onclick={() => (confirmingEntryId = entry.id)}
                  class="ml-auto text-red-400/50 hover:text-red-400 transition-colors"
                  title="Delete entry"
                >×</button>
              {/if}
            </div>
          {/each}
        {/if}
        {#if !isToday}
          {#if addingEntry}
            <div class="flex items-center gap-1 pt-1 border-t border-[var(--color-border)] mt-1">
              <input
                type="time"
                bind:value={newEntryStart}
                class="text-[0.65rem] bg-transparent border-b border-[var(--color-border)] text-[var(--color-text-muted)] focus:outline-none focus:border-[var(--color-accent)] w-14 tabular-nums"
              />
              <span class="text-[var(--color-muted)]">–</span>
              <input
                type="time"
                bind:value={newEntryEnd}
                class="text-[0.65rem] bg-transparent border-b border-[var(--color-border)] text-[var(--color-text-muted)] focus:outline-none focus:border-[var(--color-accent)] w-14 tabular-nums"
              />
              <button
                onclick={handleAddManualEntry}
                disabled={!newEntryStart}
                class="ml-auto text-[0.6rem] px-1 py-px rounded bg-[var(--color-accent)]/10 text-[var(--color-accent)] hover:bg-[var(--color-accent)]/20 disabled:opacity-40 transition-colors"
              >Save</button>
              <button
                onclick={() => { addingEntry = false; newEntryStart = ''; newEntryEnd = ''; addError = null; }}
                class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors text-xs"
              >×</button>
            </div>
            {#if addError}
              <p class="text-[0.6rem] text-rose-400 mt-0.5">{addError}</p>
            {/if}
          {:else}
            <button
              onclick={() => (addingEntry = true)}
              class="mt-1 text-[0.6rem] text-[var(--color-muted)] hover:text-[var(--color-accent)] transition-colors"
            >+ Add entry</button>
          {/if}
        {/if}
      </div>
    {/if}
  </div>

  <div class="px-3 pb-2 shrink-0">
    <LoadIndicator doneHours={doneHoursByType} plannedHours={plannedHoursByType} {availableHours} clockedHours={entries.length > 0 ? totalLoggedHours() : undefined} />
  </div>

  <!-- Scrollable card area -->
  <div class="flex-1 overflow-y-auto min-h-0 px-3 py-2">
    <div
      class="flex flex-col gap-1.5 min-h-[2rem] mb-3"
      use:dndzone={{
        items: localPendingItems,
        flipDurationMs: 150,
        dropTargetStyle: { outline: 'none', background: 'rgba(61,126,255,0.07)', 'border-radius': '6px' }
      }}
      onconsider={handleDndConsider}
      onfinalize={handleDndFinalize}
    >
      {#each localPendingItems as card (card.id)}
        <CardComponent {card} {onMarkDone} {onMoveToNextWeek} {isToday} />
      {/each}
      {#if isToday && localPendingItems.length === 0}
        <p class="text-xs text-[var(--color-muted)] text-center py-2">Drag from backlog or add a task ↓</p>
      {/if}
    </div>

    {#if doneTasks.length + doneMeetings.length > 0}
      <div>
        <button
          onclick={() => (showDone = !showDone)}
          class="text-xs text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
        >
          {showDone ? '▾' : '▸'} {doneTasks.length + doneMeetings.length} done · {doneHoursByType.toFixed(1)}h consumed
        </button>
        {#if showDone}
          <div class="flex flex-col gap-1.5 mt-1.5 pb-1">
            {#each doneMeetings as card (card.id)}
              <CardComponent {card} {onMarkDone} {isToday} />
            {/each}
            {#each doneTasks as card (card.id)}
              <CardComponent {card} {onMarkDone} {isToday} />
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- QuickAdd pinned to bottom -->
  <div class="px-3 py-2 shrink-0 border-t border-[var(--color-glass-border)]">
    <QuickAdd onAdd={onAddCard} {weekId} {dayOfWeek} {onCardCreated} />
  </div>
</div>
