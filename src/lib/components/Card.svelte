<script module lang="ts">
  /**
   * Extract meeting time range from metadata
   * Returns "HH:MM" for start-only, "HH:MM – HH:MM" for start+end, null otherwise
   */
  export function getMeetingTimeRange(metadata: string | null): string | null {
    if (!metadata) return null;
    try {
      const m = JSON.parse(metadata) as Record<string, unknown>;
      const opts = { hour: '2-digit', minute: '2-digit' } as const;
      const start = new Date(m.start_time as string ?? '').toLocaleTimeString([], opts);
      if (!m.end_time) return start;
      const end = new Date(m.end_time as string).toLocaleTimeString([], opts);
      return `${start} – ${end}`;
    } catch { return null; }
  }
</script>

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { Card, Impact, CardTimeEntry } from '$lib/types';
  import { boardStore } from '$lib/stores/board.svelte';
  import { projectsStore } from '$lib/stores/projects.svelte';
  import { openUrl } from '$lib/api/shell';
  import { hoursToHHMM } from '$lib/utils';
  import * as cardTimeApi from '$lib/api/card_time_entries';
  import { ExternalLink, Trash2, Check, Users, GitPullRequest, MessageSquare, ListTodo, Eye, FileText, X, GripVertical } from 'lucide-svelte';
  import EditCardModal from './EditCardModal.svelte';

  let {
    card,
    onMarkDone,
    onMoveToNextWeek,
    onScheduleToday,
    isToday = false
  }: {
    card: Card;
    onMarkDone: (id: number) => void;
    onMoveToNextWeek?: (id: number) => void;
    onScheduleToday?: (id: number) => void;
    isToday?: boolean;
  } = $props();

  const cardProject = $derived(
    card.project_id != null
      ? projectsStore.projects.find(p => p.id === card.project_id) ?? null
      : null
  );

  const typeBadge: Record<string, string> = {
    meeting: 'bg-blue-500/15 text-blue-300',
    mr:      'bg-purple-500/15 text-purple-300',
    thread:  'bg-yellow-500/15 text-yellow-300',
    task:    'bg-emerald-500/15 text-emerald-300',
    review:  'bg-slate-500/15 text-slate-300',
    documentation: 'bg-slate-500/15 text-slate-300'
  };

  const typeLabel: Record<string, string> = {
    meeting: 'meeting',
    mr: 'MR',
    thread: 'thread',
    task: 'task',
    review: 'review',
    documentation: 'doc',
  };

  const sourceLabel: Record<string, string> = {
    calendar: 'gcal',
    gitlab: 'gitlab',
    linear: 'linear',
    slack: 'slack',
    notion: 'notion',
  };

  const impactBadge: Record<string, string> = {
    low:  'text-[var(--color-impact-low)]  bg-[var(--color-impact-low-bg)]  px-1 py-px rounded',
    mid:  'text-[var(--color-impact-mid)]  bg-[var(--color-impact-mid-bg)]  px-1 py-px rounded',
    high: 'text-[var(--color-impact-high)] bg-[var(--color-impact-high-bg)] px-1 py-px rounded',
  };

  const aiFields = $derived.by(() => {
    if (!card.metadata) return { description: null, impact: null };
    try {
      const m = JSON.parse(card.metadata) as Record<string, unknown>;
      const rawImpact = m.ai_impact as string | undefined;
      return {
        description: (m.ai_description as string) ?? null,
        impact: rawImpact === 'medium' ? 'mid' : (rawImpact ?? null) as Impact | null,
      };
    } catch { return { description: null, impact: null }; }
  });

  const displayImpact = $derived((card.impact ?? aiFields.impact) as Impact | null);

  const meetingTime = $derived(card.card_type === 'meeting' ? getMeetingTimeRange(card.metadata) : null);

  // Card-level time tracking state
  let activeCardEntry = $state<CardTimeEntry | null>(null);
  let cardEntries = $state<CardTimeEntry[]>([]);
  let cardElapsedSeconds = $state(0);
  let cardClockTick: ReturnType<typeof setInterval> | null = null;

  function parseSqliteUtc(s: string): Date {
    return new Date(s.replace(' ', 'T') + 'Z');
  }

  function formatCardElapsed(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  const totalCardClocked = $derived(
    cardEntries
      .filter(e => e.end_time !== null)
      .reduce((sum, e) => {
        const start = parseSqliteUtc(e.start_time).getTime();
        const end = parseSqliteUtc(e.end_time!).getTime();
        return sum + (end - start) / 3600000;
      }, 0)
  );

  onMount(async () => {
    if (!card.week_id) return; // backlog cards don't track time
    try {
      [activeCardEntry, cardEntries] = await Promise.all([
        cardTimeApi.getActiveCardEntry(card.id),
        cardTimeApi.listCardTimeEntries(card.id),
      ]);
      if (activeCardEntry) {
        const startMs = parseSqliteUtc(activeCardEntry.start_time).getTime();
        cardElapsedSeconds = Math.floor((Date.now() - startMs) / 1000);
        cardClockTick = setInterval(() => {
          if (activeCardEntry) {
            const startMs = parseSqliteUtc(activeCardEntry.start_time).getTime();
            cardElapsedSeconds = Math.floor((Date.now() - startMs) / 1000);
          }
        }, 1000);
      }
    } catch { /* non-critical */ }
  });

  onDestroy(() => {
    if (cardClockTick) clearInterval(cardClockTick);
  });

  async function handleCardClockIn() {
    const today = new Date().toISOString().slice(0, 10);
    activeCardEntry = await cardTimeApi.cardClockIn(card.id, today);
    cardEntries = await cardTimeApi.listCardTimeEntries(card.id);
    cardElapsedSeconds = 0;
    cardClockTick = setInterval(() => {
      if (activeCardEntry) {
        const startMs = parseSqliteUtc(activeCardEntry.start_time).getTime();
        cardElapsedSeconds = Math.floor((Date.now() - startMs) / 1000);
      }
    }, 1000);
  }

  async function handleCardClockOut() {
    if (!activeCardEntry) return;
    if (cardClockTick) { clearInterval(cardClockTick); cardClockTick = null; }
    await cardTimeApi.cardClockOut(activeCardEntry.id);
    cardEntries = await cardTimeApi.listCardTimeEntries(card.id);
    activeCardEntry = null;
    cardElapsedSeconds = 0;
  }

  // Editing state
  let isPopoverOpen = $state(false);
  let confirmingDelete = $state(false);
  let saveError = $state<string | null>(null);

  function openPopover() {
    isPopoverOpen = true;
  }

  function closePopover() {
    isPopoverOpen = false;
  }

  function cancelDelete() {
    confirmingDelete = false;
  }

  async function deleteCard() {
    try {
      await boardStore.deleteCard(card.id);
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleToggleDone() {
    if (card.status !== 'done') {
      if (activeCardEntry) await handleCardClockOut();
      if (cardEntries.filter(e => e.end_time !== null).length > 0) {
        await cardTimeApi.finalizeCardTime(card.id);
      }
      onMarkDone(card.id);
    } else {
      boardStore.updateCard(card.id, { status: 'planned' });
    }
  }
</script>

<div
  class="glass-card group relative flex flex-row gap-2 items-start rounded-md border border-[var(--color-glass-border)] bg-[var(--color-glass-bg)] px-3 hover:border-[var(--color-accent)]/40 hover:bg-[var(--color-glass-bg-raised)] backdrop-blur-sm transition-colors"
  class:py-2={card.card_type !== 'meeting'}
  class:py-1={card.card_type === 'meeting'}
  class:cursor-grab={!isPopoverOpen}
  class:active:cursor-grabbing={!isPopoverOpen}
  class:cursor-default={isPopoverOpen}
  class:opacity-40={card.status === 'done'}
  role="article"
>
  <div class="flex flex-col items-center justify-between flex-shrink-0 self-stretch">
    <button
      data-no-dnd="true"
      onclick={handleToggleDone}
      class="flex-shrink-0 w-4 h-4 rounded-full border transition-colors flex items-center justify-center {card.status === 'done' ? 'border-[var(--color-done)] bg-[var(--color-done)]/20 text-[var(--color-done)]' : 'border-[var(--color-border)] text-transparent hover:border-[var(--color-done)]/60'}"
      aria-label={card.status === 'done' ? 'Undo done' : 'Mark done'}
    >{#if card.status === 'done'}<Check size={10} />{/if}</button>

    {#if !isPopoverOpen}
      <div class="text-[var(--color-muted)] opacity-30 group-hover:opacity-70 cursor-grab mt-auto">
        <GripVertical size={14} />
      </div>
    {/if}
  </div>

  <div
    class="flex-1 min-w-0"
    class:cursor-pointer={!isPopoverOpen}
    class:cursor-default={isPopoverOpen}
    role="button"
    tabindex="0"
    onclick={openPopover}
    onkeydown={(e) => { if ((e.key === 'Enter' || e.key === ' ') && e.target === e.currentTarget) { e.preventDefault(); openPopover(); } }}
  >
    {#if meetingTime}
      <span class="text-xs text-[var(--color-muted)] mb-1 block">{meetingTime}</span>
    {/if}

    <p class="text-sm font-medium text-[var(--color-text)] leading-snug">{card.title}</p>

    <div class="mt-1.5 flex items-center gap-1.5 flex-wrap">
      <!-- Project slug badge -->
      {#if cardProject}
        <span
          class="text-xs px-1 py-px rounded border font-mono font-semibold"
          style="border-color: {cardProject.color}40; background-color: {cardProject.color}15; color: {cardProject.color};"
        >{cardProject.slug}</span>
      {/if}

      <!-- Type badge: icon + expanding label on hover -->
      <span
        class="text-xs px-1 py-px rounded border border-[var(--color-border)] flex items-center gap-1 {typeBadge[card.card_type] ?? 'bg-slate-500/15 text-slate-300'}"
        title={card.source !== 'manual' ? `${typeLabel[card.card_type]} · ${sourceLabel[card.source]}` : typeLabel[card.card_type]}
      >
        {#if card.card_type === 'meeting'}<Users size={10} />{/if}
        {#if card.card_type === 'mr'}<GitPullRequest size={10} />{/if}
        {#if card.card_type === 'thread'}<MessageSquare size={10} />{/if}
        {#if card.card_type === 'task'}<ListTodo size={10} />{/if}
        {#if card.card_type === 'review'}<Eye size={10} />{/if}
        {#if card.card_type === 'documentation'}<FileText size={10} />{/if}
        <span class="max-w-0 overflow-hidden group-hover:max-w-[4rem] transition-all duration-200 whitespace-nowrap text-[0.65rem]">{typeLabel[card.card_type]}</span>
      </span>

      <!-- Impact badge -->
      {#if displayImpact}
        <span class="text-xs border border-[var(--color-border)] {impactBadge[displayImpact]}">{displayImpact}</span>
      {/if}

      <!-- Time estimate -->
      {#if card.time_estimate}
        <span class="text-xs text-[var(--color-muted)]">{hoursToHHMM(card.time_estimate)}</span>
      {/if}

      <!-- Note indicator -->
      {#if card.notes}
        <span class="text-xs text-[var(--color-muted)]" title="Has notes">•</span>
      {/if}

      <!-- Card-level time tracking display (only when on board) -->
      {#if card.week_id}
        {#if activeCardEntry}
          <span class="text-xs tabular-nums font-mono text-[var(--color-accent)]">{formatCardElapsed(cardElapsedSeconds)}</span>
        {:else if totalCardClocked > 0}
          <span class="text-xs text-[var(--color-muted)]">{totalCardClocked.toFixed(1)}h clocked</span>
        {/if}
      {/if}

      <!-- URL link (always visible) -->
      {#if card.url}
        <button
          data-no-dnd="true"
          onclick={(e) => { e.stopPropagation(); openUrl(card.url!); }}
          class="text-[var(--color-muted)] hover:text-[var(--color-accent)] transition-colors"
          aria-label="Open link"
          title="Open link"
        ><ExternalLink size={10} /></button>
      {/if}
    </div>

    {#if !isPopoverOpen}
      <div class="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity mt-0.5">
        <button
          data-no-dnd="true"
          onclick={(e) => { e.stopPropagation(); boardStore.duplicateCard(card.id); }}
          class="text-[var(--color-muted)] hover:text-[var(--color-accent-hover)] transition-colors text-xs leading-none"
          aria-label="Duplicate card"
          title="Duplicate card"
        >⧉</button>
        {#if isToday && card.card_type !== 'meeting'}
          {#if activeCardEntry}
            <button
              data-no-dnd="true"
              onclick={(e) => { e.stopPropagation(); handleCardClockOut(); }}
              class="text-red-300 hover:text-red-200 transition-colors text-xs leading-none"
              title="Pause timer"
            >⏸</button>
          {:else}
            <button
              data-no-dnd="true"
              onclick={(e) => { e.stopPropagation(); handleCardClockIn(); }}
              class="text-[var(--color-muted)] hover:text-[var(--color-accent)] transition-colors text-xs leading-none"
              title="Start timer"
            >▶</button>
          {/if}
        {/if}
        {#if onMoveToNextWeek}
          <button
            data-no-dnd="true"
            onclick={(e) => { e.stopPropagation(); onMoveToNextWeek!(card.id); }}
            class="text-[var(--color-muted)] hover:text-[var(--color-accent-hover)] transition-colors text-xs leading-none"
            aria-label="Move to next week"
            title="Move to next week"
          >»</button>
        {/if}
        {#if onScheduleToday}
          <button
            data-no-dnd="true"
            onclick={(e) => { e.stopPropagation(); onScheduleToday!(card.id); }}
            class="text-[var(--color-muted)] hover:text-[var(--color-accent-hover)] transition-colors text-xs leading-none whitespace-nowrap"
            aria-label="Schedule for today"
            title="Schedule for today"
          >→ Today</button>
        {/if}
        {#if !confirmingDelete}
          <button
            data-no-dnd="true"
            onclick={(e) => { e.stopPropagation(); confirmingDelete = true; }}
            class="text-[var(--color-muted)] hover:text-rose-400 transition-colors"
            aria-label="Delete card"
            title="Delete card"
          ><Trash2 size={12} /></button>
        {:else}
          <span class="text-xs text-rose-400">Delete?</span>
          <button
            data-no-dnd="true"
            onclick={(e) => { e.stopPropagation(); cancelDelete(); }}
            class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
            aria-label="Cancel delete"
          ><X size={12} /></button>
          <button
            data-no-dnd="true"
            onclick={(e) => { e.stopPropagation(); deleteCard(); }}
            class="text-rose-400 hover:text-rose-300 transition-colors"
            aria-label="Confirm delete"
          ><Check size={12} /></button>
        {/if}
      </div>
    {/if}
  </div>

</div>

{#if isPopoverOpen}
  <EditCardModal {card} onClose={closePopover} />
{/if}
