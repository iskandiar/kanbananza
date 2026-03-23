<!-- src/lib/components/HistoryReport.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import type { Card, Week, DayTypeHours } from '$lib/types';
  import { listDayEntriesForWeek } from '$lib/api/card_time_entries';
  import { listTimeEntriesForWeek, type DayTimeEntry } from '$lib/api/time_entries';
  import { summariseWeek } from '$lib/api/ai';
  import { toastStore } from '$lib/stores/toast.svelte';

  let { week, weekCards, availableHours, isPastWeek }: {
    week: Week | null;
    weekCards: Card[];
    availableHours: number;
    isPastWeek: boolean;
  } = $props();

  let dayBreakdown = $state<DayTypeHours[]>([]);
  let sessionEntries = $state<DayTimeEntry[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    if (!week || !isPastWeek) return;
    loading = true;
    try {
      [dayBreakdown, sessionEntries] = await Promise.all([
        listDayEntriesForWeek(week.id),
        listTimeEntriesForWeek(week.start_date),
      ]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  // Group cards by card_type — returns array of [type, cards] tuples for Svelte {#each}
  const CARD_TYPE_ORDER = ['task', 'meeting', 'mr', 'thread', 'review', 'documentation'] as const;

  const cardsByType = $derived.by((): [string, Card[]][] => {
    const result: [string, Card[]][] = [];
    for (const type of CARD_TYPE_ORDER) {
      const group = weekCards.filter(c => c.card_type === type);
      if (group.length > 0) result.push([type, group]);
    }
    return result;
  });

  // Total session hours for the week
  const sessionTotalHours = $derived(
    sessionEntries
      .filter(e => e.end_time !== null)
      .reduce((sum, e) => {
        const start = new Date(e.start_time.replace(' ', 'T') + 'Z').getTime();
        const end = new Date(e.end_time!.replace(' ', 'T') + 'Z').getTime();
        return sum + (end - start) / 3_600_000;
      }, 0)
  );

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
    label: string; date: string; hours: number;
    segments: { card_type: string; hours: number; color: string }[];
    sessionLabel: string;
  };

  const BAR_HEIGHT = 56;

  const dayBars = $derived.by((): DayBar[] => {
    if (!week) return [];
    const today = new Date().toISOString().slice(0, 10);
    return DAY_LABELS.map((label, i) => {
      const date = dayDate(week!.start_date, i);
      const isFuture = date > today;
      const entries = isFuture ? [] : dayBreakdown.filter(d => d.date === date);
      const hours = entries.reduce((s, e) => s + e.hours, 0);
      const daySessions = sessionEntries.filter(s => s.date === date && s.end_time !== null);
      let sessionLabel = '–';
      if (daySessions.length > 0) {
        const totalH = daySessions.reduce((sum, s) => {
          const start = new Date(s.start_time.replace(' ', 'T') + 'Z').getTime();
          const end = new Date(s.end_time!.replace(' ', 'T') + 'Z').getTime();
          return sum + (end - start) / 3_600_000;
        }, 0);
        const h = Math.floor(totalH);
        const m = Math.round((totalH - h) * 60);
        sessionLabel = m > 0 ? `${h}h${m}m` : `${h}h`;
      }
      return { label, date, hours, segments: entries.map(e => ({ card_type: e.card_type, hours: e.hours, color: typeColor(e.card_type) })), sessionLabel };
    });
  });

  const ah = $derived(Math.max(1, availableHours ?? 8));

  const maxOverflowH = $derived(
    Math.min(28, Math.max(0, ...dayBars.map(d => {
      return d.hours > ah ? Math.round(((d.hours - ah) / ah) * BAR_HEIGHT) : 0;
    })))
  );

  // Pie chart
  function buildPieInput(
    cards: { card_type: string; time_estimate: number | null }[],
    sessionTotal: number
  ): { card_type: string; hours: number }[] {
    const map: Record<string, number> = {};
    for (const c of cards) {
      map[c.card_type] = (map[c.card_type] ?? 0) + (c.time_estimate ?? 0);
    }
    const estimateTotal = Object.values(map).reduce((s, h) => s + h, 0);
    const other = Math.max(0, sessionTotal - estimateTotal);
    return [
      ...Object.entries(map).filter(([, h]) => h > 0).map(([card_type, hours]) => ({ card_type, hours })),
      ...(other > 0.01 ? [{ card_type: 'other', hours: other }] : []),
    ];
  }

  type PieSlice = { label: string; hours: number; pct: number; color: string; startAngle: number; endAngle: number };

  function buildPieSlices(input: { card_type: string; hours: number }[]): PieSlice[] {
    const total = input.reduce((s, b) => s + b.hours, 0);
    if (total === 0) return [];
    const threshold = total * 0.08;
    const main = input.filter(b => b.hours >= threshold);
    const otherHours = input.filter(b => b.hours < threshold).reduce((s, b) => s + b.hours, 0);
    const items: { label: string; hours: number; color: string }[] = [
      ...main.map(b => ({ label: b.card_type, hours: b.hours, color: typeColor(b.card_type) })),
      ...(otherHours > 0 ? [{ label: 'other', hours: otherHours, color: '#6b7280' }] : []),
    ];
    let angle = -Math.PI / 2;
    return items.map(item => {
      const sweep = (item.hours / total) * 2 * Math.PI;
      const slice: PieSlice = {
        label: item.label,
        hours: item.hours,
        pct: Math.round((item.hours / total) * 100),
        color: item.color,
        startAngle: angle,
        endAngle: angle + sweep,
      };
      angle += sweep;
      return slice;
    });
  }

  function pieArcPath(cx: number, cy: number, r: number, startAngle: number, endAngle: number): string {
    const x1 = cx + r * Math.cos(startAngle);
    const y1 = cy + r * Math.sin(startAngle);
    const x2 = cx + r * Math.cos(endAngle);
    const y2 = cy + r * Math.sin(endAngle);
    const largeArc = endAngle - startAngle > Math.PI ? 1 : 0;
    return `M ${cx} ${cy} L ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2} Z`;
  }

  const pieSlices = $derived.by(() => {
    if (!week) return [];
    const input = buildPieInput(weekCards, sessionTotalHours);
    return buildPieSlices(input);
  });
  const pieTotal = $derived(pieSlices.reduce((s, sl) => s + sl.hours, 0));

  let notesInput = $state('');
  let summarising = $state(false);
  let summaryError = $state<string | null>(null);
  let localSummary = $state<string | null>(null);

  const displaySummary = $derived(localSummary ?? week?.summary ?? null);

  async function handleSummarise() {
    if (!week) return;
    summarising = true;
    summaryError = null;
    try {
      const result = await summariseWeek(week.id, notesInput || undefined);
      localSummary = result;
      toastStore.add('Summary generated', 'success');
    } catch (e) {
      summaryError = String(e);
    } finally {
      summarising = false;
    }
  }
</script>

<div class="flex-1 overflow-y-auto">
  {#if !isPastWeek}
    <div class="flex items-center justify-center h-full text-sm text-[var(--color-muted)]">
      History will be available once this week is complete.
    </div>
  {:else if loading}
    <div class="flex items-center justify-center h-full text-sm text-[var(--color-muted)]">Loading…</div>
  {:else if error}
    <div class="flex items-center justify-center h-full text-sm text-red-400">{error}</div>
  {:else}
    <div class="max-w-2xl mx-auto px-6 py-6 flex flex-col gap-6">

      <!-- Stacked bar chart — always shown -->
      <section>
        <div class="flex items-center justify-between mb-2">
          <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-muted)]">Time</h3>
          <span class="text-xs tabular-nums text-[var(--color-muted)]">{sessionTotalHours.toFixed(1)}h clocked</span>
        </div>
        <div class="flex gap-2 items-end">
            {#each dayBars as day (day.date)}
              {@const barH = Math.min(BAR_HEIGHT, Math.round((day.hours / ah) * BAR_HEIGHT))}
              {@const overflowH = day.hours > ah ? Math.round(((day.hours - ah) / ah) * BAR_HEIGHT) : 0}
              {@const spacerH = maxOverflowH - overflowH}
              <div class="flex flex-col items-center gap-1 flex-1 min-w-0">
                {#if maxOverflowH > 0}
                  <div class="w-full flex flex-col-reverse rounded-sm overflow-hidden" style="height: {maxOverflowH}px;">
                    {#if overflowH > 0}<div style="height: {overflowH}px; background: #f97316;"></div>{/if}
                    {#if spacerH > 0}<div style="height: {spacerH}px; background: transparent;"></div>{/if}
                  </div>
                {/if}
                <div class="w-full flex flex-col-reverse rounded-sm overflow-hidden" style="height: {BAR_HEIGHT}px; background: var(--color-surface);">
                  {#if day.hours > 0}
                    {#each day.segments as seg (seg.card_type)}
                      {@const segH = Math.max(2, Math.round((seg.hours / day.hours) * barH))}
                      <div style="height: {segH}px; background: {seg.color};" title="{seg.card_type}: {seg.hours.toFixed(1)}h"></div>
                    {/each}
                  {/if}
                </div>
                <p class="text-[0.6rem] tabular-nums text-[var(--color-text-muted)]">{day.sessionLabel}</p>
                <p class="text-[0.6rem] text-[var(--color-muted)] uppercase tracking-wide">{day.label}</p>
              </div>
            {/each}
          </div>
        </section>

      <!-- AI Summary -->
      <section>
        <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-muted)] mb-2">Summary</h3>
        {#if displaySummary}
          <p class="text-sm text-[var(--color-text-muted)] leading-relaxed mb-3">{displaySummary}</p>
        {/if}
        <div class="flex gap-2 items-start">
          <textarea
            bind:value={notesInput}
            placeholder="Add guiding notes (optional)…"
            rows="2"
            class="flex-1 text-xs bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1.5 text-[var(--color-text)] placeholder:text-[var(--color-muted)] resize-none focus:outline-none focus:border-[var(--color-accent)]"
          ></textarea>
          <button
            onclick={handleSummarise}
            disabled={summarising || weekCards.length === 0}
            class="shrink-0 text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)] disabled:opacity-40 disabled:cursor-not-allowed transition-colors whitespace-nowrap"
          >
            {summarising ? '…' : '✦ Summarize'}
          </button>
        </div>
        {#if summaryError}
          <p class="text-xs text-red-400 mt-1">{summaryError}</p>
        {/if}
      </section>

      <!-- Breakdown (pie chart) -->
      {#if pieSlices.length > 0}
        <section>
          <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-muted)] mb-2">Breakdown</h3>
          <div class="flex items-center gap-6">
            <svg width="120" height="120" viewBox="0 0 120 120" class="shrink-0">
              {#each pieSlices as slice (slice.label)}
                <path
                  d={pieArcPath(60, 60, 54, slice.startAngle, slice.endAngle)}
                  fill={slice.color}
                  opacity="0.85"
                />
              {/each}
              <circle cx="60" cy="60" r="26" fill="var(--color-background)" />
              <text x="60" y="65" text-anchor="middle" fill="var(--color-text-muted)" font-size="10">{pieTotal.toFixed(1)}h</text>
            </svg>
            <div class="flex flex-col gap-1.5">
              {#each pieSlices as slice (slice.label)}
                <span class="text-xs text-[var(--color-text-muted)] flex items-center gap-2">
                  <span class="inline-block w-2.5 h-2.5 rounded-sm shrink-0" style="background: {slice.color}"></span>
                  <span class="capitalize">{slice.label}</span>
                  <span class="tabular-nums text-[var(--color-muted)]">{slice.hours.toFixed(1)}h · {slice.pct}%</span>
                </span>
              {/each}
            </div>
          </div>
        </section>
      {/if}

      <!-- Items by card type — last -->
      {#each cardsByType as [type, cards] (type)}
        {@const doneCount = cards.filter(c => c.status === 'done').length}
        <section>
          <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-muted)] mb-2 capitalize flex items-baseline gap-2">
            {type}
            <span class="font-normal normal-case tracking-normal">{cards.length} · {doneCount} done</span>
          </h3>
          <ul class="flex flex-col gap-0.5">
            {#each cards as card (card.id)}
              <li class="flex items-baseline gap-1.5 {card.status === 'done' ? 'opacity-40' : ''}">
                <span class="text-[var(--color-muted)] text-xs shrink-0">·</span>
                <span class="text-sm text-[var(--color-text)]">{card.title}</span>
                {#if card.time_estimate && card.time_estimate > 0}
                  <span class="text-xs text-[var(--color-muted)] shrink-0">{card.time_estimate.toFixed(1)}h</span>
                {/if}
              </li>
            {/each}
          </ul>
        </section>
      {/each}

    </div>
  {/if}
</div>
