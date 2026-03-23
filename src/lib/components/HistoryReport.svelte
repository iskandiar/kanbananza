<!-- src/lib/components/HistoryReport.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import type { Card, Week, DayTypeHours } from '$lib/types';
  import { listDayEntriesForWeek } from '$lib/api/card_time_entries';
  import { listTimeEntriesForWeek, type DayTimeEntry } from '$lib/api/time_entries';

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

  function toLocalHHMM(utcDatetime: string): string {
    const d = new Date(utcDatetime.replace(' ', 'T') + 'Z');
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false });
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
        const first = toLocalHHMM(daySessions[0].start_time);
        const last = toLocalHHMM(daySessions[daySessions.length - 1].end_time!);
        sessionLabel = `${first}–${last}`;
      }
      return { label, date, hours, segments: entries.map(e => ({ card_type: e.card_type, hours: e.hours, color: typeColor(e.card_type) })), sessionLabel };
    });
  });

  const maxOverflowH = $derived(
    Math.min(28, Math.max(0, ...dayBars.map(d => {
      const ah = Math.max(1, availableHours ?? 8);
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

      <!-- Items by card type -->
      {#each cardsByType as [type, cards] (type)}
        <section>
          <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-muted)] mb-2 capitalize">{type}</h3>
          <div class="flex flex-col gap-1">
            {#each cards as card (card.id)}
              <div
                class="text-sm px-3 py-1.5 rounded border border-[var(--color-border)] {card.status === 'done' ? 'opacity-40' : ''}"
              >
                <span class="text-[var(--color-text)]">{card.title}</span>
                {#if card.time_estimate && card.time_estimate > 0}
                  <span class="text-xs text-[var(--color-muted)] ml-2">{card.time_estimate.toFixed(1)}h</span>
                {/if}
              </div>
            {/each}
          </div>
        </section>
      {/each}

      <!-- Stacked bar chart -->
      {#if sessionTotalHours > 0 || dayBars.some(d => d.hours > 0)}
        {@const ah = Math.max(1, availableHours ?? 8)}
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
      {/if}

      <!-- Pie chart -->
      {#if pieSlices.length > 0}
        <section>
          <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-muted)] mb-2">Breakdown</h3>
          <div class="flex items-center gap-4">
            <svg width="80" height="80" viewBox="0 0 80 80" class="shrink-0">
              {#each pieSlices as slice (slice.label)}
                <path
                  d={pieArcPath(40, 40, 36, slice.startAngle, slice.endAngle)}
                  fill={slice.color}
                  opacity="0.85"
                />
              {/each}
              <circle cx="40" cy="40" r="18" fill="var(--color-background)" />
              <text x="40" y="44" text-anchor="middle" fill="var(--color-text-muted)" font-size="8">{pieTotal.toFixed(1)}h</text>
            </svg>
            <div class="flex flex-col gap-1">
              {#each pieSlices as slice (slice.label)}
                <span class="text-[0.6rem] text-[var(--color-text-muted)] flex items-center gap-1.5">
                  <span class="inline-block w-2 h-2 rounded-sm shrink-0" style="background: {slice.color}"></span>
                  <span class="capitalize">{slice.label}</span>
                  <span class="tabular-nums text-[var(--color-muted)]">{slice.hours.toFixed(1)}h · {slice.pct}%</span>
                </span>
              {/each}
            </div>
          </div>
        </section>
      {/if}

    </div>
  {/if}
</div>
