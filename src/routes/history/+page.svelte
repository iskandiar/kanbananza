<script lang="ts">
  import { onMount } from 'svelte';
  import type { Week, CardTypeHours } from '$lib/types';
  import { invoke } from '@tauri-apps/api/core';
  import { summariseWeek } from '$lib/api/ai';
  import { listCardsByWeek } from '$lib/api/cards';
  import { listCardEntriesForWeek } from '$lib/api/card_time_entries';
  import { formatDateRange } from '$lib/utils';
  import { toastStore } from '$lib/stores/toast.svelte';
  import KLogo from '$lib/components/KLogo.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';

  type WeekRow = Week & { cardCount: number; summarising: boolean; clockedBreakdown: CardTypeHours[] };

  let weeks = $state<WeekRow[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const raw: Week[] = await invoke('list_weeks');
      const rows = await Promise.all(
        raw.map(async (w) => {
          const [cards, clockedBreakdown] = await Promise.all([
            listCardsByWeek(w.id),
            listCardEntriesForWeek(w.id),
          ]);
          return { ...w, cardCount: cards.length, summarising: false, clockedBreakdown };
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

  const maxClocked = $derived(
    Math.max(1, ...weeks.map((w) => totalClocked(w.clockedBreakdown)))
  );

  type Segment = { x: number; w: number; color: string; type: string; hours: number };

  function buildSegments(breakdown: CardTypeHours[], barWidth: number): Segment[] {
    let x = 0;
    return breakdown.map((b) => {
      const total = totalClocked(breakdown);
      const segW = Math.round((b.hours / total) * barWidth);
      const seg: Segment = { x, w: segW, color: typeColor(b.card_type), type: b.card_type, hours: b.hours };
      x += segW;
      return seg;
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
          <div class="py-4 border-b border-[var(--color-border)]">
            <div class="flex items-start justify-between gap-4">
              <!-- Left: date + count -->
              <div class="min-w-0">
                <p class="text-sm font-medium text-[var(--color-text)]">
                  {formatDateRange(week.start_date)}
                </p>
                <p class="text-xs text-[var(--color-text-muted)] mt-0.5">
                  {week.cardCount} card{week.cardCount === 1 ? '' : 's'}
                </p>
                {#if week.summary}
                  <p class="text-xs text-[var(--color-text-muted)] mt-2 leading-relaxed max-w-xl">
                    {week.summary}
                  </p>
                {/if}

                {#if week.clockedBreakdown.length > 0}
                  {@const barWidth = Math.round((totalClocked(week.clockedBreakdown) / maxClocked) * 200)}
                  {@const segments = buildSegments(week.clockedBreakdown, barWidth)}
                  <div class="mt-2">
                    <p class="text-xs text-[var(--color-text-muted)]">{totalClocked(week.clockedBreakdown).toFixed(1)}h clocked</p>
                    <div class="mt-1 flex items-center gap-2">
                      <svg
                        width={barWidth}
                        height="8"
                        class="rounded-full overflow-hidden"
                      >
                        {#each segments as seg (seg.type)}
                          <rect x={seg.x} y="0" width={seg.w} height="8" fill={seg.color}>
                            <title>{seg.type}: {seg.hours.toFixed(1)}h</title>
                          </rect>
                        {/each}
                      </svg>
                      <div class="flex items-center gap-2 flex-wrap">
                        {#each week.clockedBreakdown as b (b.card_type)}
                          <span class="text-[0.6rem] text-[var(--color-text-muted)] flex items-center gap-1">
                            <span class="inline-block w-2 h-2 rounded-sm" style="background: {typeColor(b.card_type)}"></span>
                            {b.card_type} {b.hours.toFixed(1)}h
                          </span>
                        {/each}
                      </div>
                    </div>
                  </div>
                {/if}
              </div>

              <!-- Right: summarize button -->
              <button
                onclick={() => summarise(week.id)}
                disabled={week.summarising || week.cardCount === 0}
                class="shrink-0 text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)] disabled:opacity-40 disabled:cursor-not-allowed transition-colors whitespace-nowrap"
              >
                {week.summarising ? '…' : '✦ Summarize'}
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

  </main>
</div>
