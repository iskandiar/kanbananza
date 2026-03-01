<script lang="ts">
  import { onMount } from 'svelte';
  import type { Week } from '$lib/types';
  import { invoke } from '@tauri-apps/api/core';
  import { summariseWeek } from '$lib/api/ai';
  import { listCardsByWeek } from '$lib/api/cards';
  import { formatDateRange } from '$lib/utils';
  import { toastStore } from '$lib/stores/toast.svelte';

  type WeekRow = Week & { cardCount: number; summarising: boolean };

  let weeks = $state<WeekRow[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const raw: Week[] = await invoke('list_weeks');
      const rows = await Promise.all(
        raw.map(async (w) => {
          const cards = await listCardsByWeek(w.id);
          return { ...w, cardCount: cards.length, summarising: false };
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
</script>

<div class="min-h-screen bg-[var(--color-background)] text-[var(--color-text)] flex flex-col">

  <!-- Header -->
  <header class="flex items-center justify-between px-6 py-3 border-b border-[var(--color-border)] shrink-0">
    <a
      href="/board"
      class="text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
    >
      ← Board
    </a>
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
