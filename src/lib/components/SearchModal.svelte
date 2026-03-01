<script lang="ts">
  import { onMount } from 'svelte';
  import type { Card, Week } from '$lib/types';
  import * as cardsApi from '$lib/api/cards';
  import * as weeksApi from '$lib/api/weeks';
  import { boardStore } from '$lib/stores/board.svelte';

  let { onClose }: { onClose: () => void } = $props();

  const typeBadge: Record<string, string> = {
    meeting: 'bg-blue-500/15 text-blue-300',
    mr:      'bg-purple-500/15 text-purple-300',
    thread:  'bg-yellow-500/15 text-yellow-300',
    task:    'bg-emerald-500/15 text-emerald-300',
    review:  'bg-slate-500/15 text-slate-300',
    documentation: 'bg-slate-500/15 text-slate-300',
  };

  const typeLabel: Record<string, string> = {
    meeting: 'meeting',
    mr: 'MR',
    thread: 'thread',
    task: 'task',
    review: 'review',
    documentation: 'doc',
  };

  const DAY_NAMES = ['', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri'];

  function fmtDate(dateStr: string): string {
    const d = new Date(dateStr + 'T00:00:00');
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  }

  let query = $state('');
  let results = $state<Card[]>([]);
  let loading = $state(false);
  let weeks = $state<Map<number, Week>>(new Map());
  let searchInput: HTMLInputElement | null = null;

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  function onQueryChange() {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (query.length < 2) { results = []; return; }
    debounceTimer = setTimeout(async () => {
      loading = true;
      try {
        results = await cardsApi.searchCards(query);
      } finally {
        loading = false;
      }
    }, 200);
  }

  onMount(async () => {
    if (searchInput) searchInput.focus();
    try {
      const weekList = await weeksApi.listWeeks();
      const map = new Map<number, Week>();
      for (const w of weekList) map.set(w.id, w);
      weeks = map;
    } catch {
      // Non-fatal — context strings will fall back gracefully
    }
  });

  function handleOverlayClick() {
    onClose();
  }

  function handlePanelClick(e: MouseEvent) {
    e.stopPropagation();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }

  function cardContext(card: Card): string {
    if (card.week_id == null) return 'Backlog';
    const week = weeks.get(card.week_id);
    if (!week) return `W? · Day ${card.day_of_week ?? '?'}`;
    const monFmt = fmtDate(week.start_date);
    const dayName = DAY_NAMES[card.day_of_week ?? 0] ?? '?';
    return `W${week.week_number} · ${monFmt} · ${dayName}`;
  }

  async function handleResultClick(card: Card) {
    if (card.week_id != null) {
      const week = weeks.get(card.week_id);
      if (week) {
        await boardStore.goToWeek(week.start_date);
      }
    }
    onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm"
  onclick={handleOverlayClick}
>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="max-w-lg w-full mx-auto mt-24 rounded-xl border border-[var(--color-glass-border)] bg-[var(--color-glass-header)] backdrop-blur-md shadow-2xl overflow-hidden"
    onclick={handlePanelClick}
  >
    <!-- Search input -->
    <div class="px-4 py-3 border-b border-[var(--color-glass-border)]">
      <input
        bind:this={searchInput}
        bind:value={query}
        oninput={onQueryChange}
        type="text"
        placeholder="Search cards…"
        class="w-full bg-transparent text-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none"
      />
    </div>

    <!-- Results area -->
    <div class="max-h-[60vh] overflow-y-auto">
      {#if loading}
        <div class="flex items-center justify-center py-8">
          <span class="text-xs text-[var(--color-muted)] animate-pulse">Searching…</span>
        </div>
      {:else if query.length >= 2 && results.length === 0}
        <div class="flex items-center justify-center py-8">
          <span class="text-xs text-[var(--color-muted)]">No results for &laquo;{query}&raquo;</span>
        </div>
      {:else if query.length < 2}
        <div class="flex items-center justify-center py-8">
          <span class="text-xs text-[var(--color-muted)]">Type to search all cards…</span>
        </div>
      {:else}
        <ul>
          {#each results as card (card.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <li>
              <button
                type="button"
                class="w-full text-left px-4 py-3 flex flex-col gap-1 hover:bg-[var(--color-glass-bg-raised)] transition-colors border-b border-[var(--color-glass-border)] last:border-b-0 {card.status === 'done' ? 'opacity-40' : ''}"
                onclick={() => handleResultClick(card)}
              >
                <div class="flex items-start gap-2">
                  <span
                    class="shrink-0 text-[0.65rem] px-1 py-px rounded border border-[var(--color-border)] {typeBadge[card.card_type] ?? 'bg-slate-500/15 text-slate-300'}"
                  >{typeLabel[card.card_type] ?? card.card_type}</span>
                  <span class="text-sm font-semibold text-[var(--color-text)] leading-snug">{card.title}</span>
                </div>
                <span class="text-xs text-[var(--color-muted)] pl-[calc(1rem+0.25rem)]">{cardContext(card)}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
</div>
