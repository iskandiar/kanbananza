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

      <!-- Charts and summary added in Tasks 6–8 -->

    </div>
  {/if}
</div>
