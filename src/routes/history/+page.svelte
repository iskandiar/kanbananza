<script lang="ts">
  import { onMount } from 'svelte';
  import type { Week, CardTypeHours, DayTypeHours } from '$lib/types';
  import { invoke } from '@tauri-apps/api/core';
  import { summariseWeek } from '$lib/api/ai';
  import { listCardsByWeek } from '$lib/api/cards';
  import { listCardEntriesForWeek, listDayEntriesForWeek } from '$lib/api/card_time_entries';
  import { formatDateRange } from '$lib/utils';
  import { toastStore } from '$lib/stores/toast.svelte';
  import KLogo from '$lib/components/KLogo.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';

  type WeekRow = Week & {
    cardCount: number;
    summarising: boolean;
    clockedBreakdown: CardTypeHours[];
    dayBreakdown: DayTypeHours[];
  };

  let weeks = $state<WeekRow[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const raw: Week[] = await invoke('list_weeks');
      const rows = await Promise.all(
        raw.map(async (w) => {
          const [cards, clockedBreakdown, dayBreakdown] = await Promise.all([
            listCardsByWeek(w.id),
            listCardEntriesForWeek(w.id),
            listDayEntriesForWeek(w.id),
          ]);
          return { ...w, cardCount: cards.length, summarising: false, clockedBreakdown, dayBreakdown };
        })
      );
      weeks = rows;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  async function summarise(weekId: number) {
    const idx = weeks.findIndex((w) => w.id === weekId);
    if (idx === -1) return;
    weeks[idx].summarising = true;
    try {
      const summary = await summariseWeek(weekId);
      weeks[idx].summary = summary;
      toastStore.add('Summary generated', 'success');
    } catch (e) {
      error = String(e);
    } finally {
      weeks[idx].summarising = false;
    }
  }

  function totalClocked(breakdown: CardTypeHours[]): number {
    return breakdown.reduce((sum, b) => sum + b.hours, 0);
  }

  const typeColors: Record<string, string> = {
    task: '#10b981',
    meeting: '#3b82f6',
    mr: '#a855f7',
    thread: '#eab308',
    review: '#64748b',
    documentation: '#64748b',
  };

  function typeColor(type: string): string {
    return typeColors[type] ?? '#6b7280';
  }

  const DAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri'];

  function dayDate(weekStart: string, dayIndex: number): string {
    const d = new Date(weekStart + 'T00:00:00Z');
    d.setUTCDate(d.getUTCDate() + dayIndex);
    return d.toISOString().slice(0, 10);
  }

  type DayBar = {
    label: string;
    date: string;
    hours: number;
    segments: { card_type: string; hours: number; color: string }[];
  };

  function buildDayBars(weekStart: string, dayBreakdown: DayTypeHours[]): DayBar[] {
    return DAY_LABELS.map((label, i) => {
      const date = dayDate(weekStart, i);
      const entries = dayBreakdown.filter(d => d.date === date);
      const hours = entries.reduce((s, e) => s + e.hours, 0);
      return {
        label,
        date,
        hours,
        segments: entries.map(e => ({ card_type: e.card_type, hours: e.hours, color: typeColor(e.card_type) })),
      };
    });
  }
</script>

<div class="min-h-screen bg-[var(--color-background)] text-[var(--color-text)] flex flex-col">

  <!-- Header -->
  <header class="flex items-center justify-between px-6 py-3 border-b border-[var(--color-border)] shrink-0">
    <div class="flex items-center gap-3">
      <KLogo size={26} theme={themeStore.current} />
      <span class="border-l border-[var(--color-border)] h-4 self-center"></span>
      <a
        href="/board"
        class="text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
      >← Board</a>
    </div>
    <h1 class="text-sm font-medium text-[var(--color-text)] absolute left-1/2 -translate-x-1/2">
      History
    </h1>
    <div class="w-14"></div>
  </header>

  <main class="flex-1 overflow-y-auto px-6 py-6 max-w-2xl mx-auto w-full">

    {#if loading}
      <p class="text-sm text-[var(--color-text-muted)]">Loading…</p>
    {:else if error}
      <p class="text-sm text-red-400">{error}</p>
    {:else if weeks.length === 0}
      <p class="text-sm text-[var(--color-text-muted)]">No weeks yet.</p>
    {:else}
      <div class="flex flex-col gap-0">
        {#each weeks as week (week.id)}
          {@const dayBars = buildDayBars(week.start_date, week.dayBreakdown)}
          {@const maxDayHours = Math.max(1, ...dayBars.map(d => d.hours))}
          {@const BAR_HEIGHT = 56}
          <div class="py-5 border-b border-[var(--color-border)]">
            <!-- Week header row -->
            <div class="flex items-center justify-between gap-4 mb-3">
              <div>
                <p class="text-sm font-medium text-[var(--color-text)]">
                  {formatDateRange(week.start_date)}
                </p>
                <p class="text-xs text-[var(--color-text-muted)] mt-0.5">
                  {week.cardCount} card{week.cardCount === 1 ? '' : 's'}
                  {#if totalClocked(week.clockedBreakdown) > 0}
                    · {totalClocked(week.clockedBreakdown).toFixed(1)}h clocked
                  {/if}
                </p>
                {#if week.summary}
                  <p class="text-xs text-[var(--color-text-muted)] mt-2 leading-relaxed max-w-xl">
                    {week.summary}
                  </p>
                {/if}
              </div>
              <button
                onclick={() => summarise(week.id)}
                disabled={week.summarising || week.cardCount === 0}
                class="shrink-0 text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)] disabled:opacity-40 disabled:cursor-not-allowed transition-colors whitespace-nowrap"
              >
                {week.summarising ? '…' : '✦ Summarize'}
              </button>
            </div>

            <!-- Day bar chart -->
            <div class="flex gap-2 items-end">
              {#each dayBars as day (day.date)}
                {@const barH = day.hours > 0 ? Math.max(4, Math.round((day.hours / maxDayHours) * BAR_HEIGHT)) : 0}
                <div class="flex flex-col items-center gap-1 flex-1 min-w-0">
                  <!-- Stacked bar -->
                  <div class="w-full flex flex-col-reverse rounded-sm overflow-hidden" style="height: {BAR_HEIGHT}px; background: var(--color-surface);">
                    {#if day.hours > 0}
                      {#each day.segments as seg (seg.card_type)}
                        {@const segH = Math.max(2, Math.round((seg.hours / day.hours) * barH))}
                        <div
                          style="height: {segH}px; background: {seg.color};"
                          title="{seg.card_type}: {seg.hours.toFixed(1)}h"
                        ></div>
                      {/each}
                    {/if}
                  </div>
                  <!-- Hours label -->
                  <p class="text-[0.6rem] tabular-nums text-[var(--color-text-muted)]">
                    {day.hours > 0 ? day.hours.toFixed(1) + 'h' : '–'}
                  </p>
                  <!-- Day label -->
                  <p class="text-[0.6rem] text-[var(--color-muted)] uppercase tracking-wide">{day.label}</p>
                </div>
              {/each}
            </div>

            <!-- Legend -->
            {#if week.clockedBreakdown.length > 0}
              <div class="flex items-center gap-3 mt-2 flex-wrap">
                {#each week.clockedBreakdown as b (b.card_type)}
                  <span class="text-[0.6rem] text-[var(--color-text-muted)] flex items-center gap-1">
                    <span class="inline-block w-2 h-2 rounded-sm" style="background: {typeColor(b.card_type)}"></span>
                    {b.card_type}
                  </span>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

  </main>
</div>
