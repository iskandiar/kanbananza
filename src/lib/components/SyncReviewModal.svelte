<script lang="ts">
  import { X, Loader2 } from 'lucide-svelte';
  import { portal } from '$lib/actions/portal';
  import * as syncApi from '$lib/api/sync';
  import type { CardPreview } from '$lib/api/sync';

  let {
    source,
    onClose,
    onSynced,
  }: {
    source: 'calendar' | 'linear' | 'gitlab';
    onClose: () => void;
    onSynced?: (count: number) => void;
  } = $props();

  // ── State ──────────────────────────────────────────────────────────────────
  let loading = $state(true);
  let items = $state<CardPreview[]>([]);
  let checked = $state<Set<string>>(new Set());
  let dateRange = $state<'today' | 'tomorrow'>('today');
  let confirming = $state(false);
  let syncedCount = $state<number | null>(null);
  let error = $state<string | null>(null);

  // ── Derived ────────────────────────────────────────────────────────────────
  const sourceLabel = $derived(
    source === 'calendar' ? 'Calendar' : source === 'linear' ? 'Linear' : 'GitLab'
  );

  const checkedCount = $derived(checked.size);

  // ── Type badge colours (mirrors EditCardModal / Card.svelte) ───────────────
  const typeBadge: Record<string, string> = {
    meeting: 'bg-blue-500/15 text-blue-300',
    mr:      'bg-purple-500/15 text-purple-300',
    task:    'bg-emerald-500/15 text-emerald-300',
    thread:  'bg-yellow-500/15 text-yellow-300',
    review:  'bg-slate-500/15 text-slate-300',
  };

  // ── Data loading ───────────────────────────────────────────────────────────
  async function loadPreview() {
    loading = true;
    error = null;
    syncedCount = null;
    try {
      let result: CardPreview[];
      if (source === 'calendar') {
        result = await syncApi.fetchCalendarPreview(dateRange);
      } else if (source === 'linear') {
        result = await syncApi.fetchLinearPreview();
      } else {
        result = await syncApi.fetchGitlabPreview();
      }
      items = result;
      checked = new Set(result.map(i => i.external_id));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      items = [];
      checked = new Set();
    } finally {
      loading = false;
    }
  }

  // Reload when dateRange changes (calendar only) or on initial mount
  $effect(() => {
    // Depend on dateRange so the effect re-runs when it changes
    void dateRange;
    loadPreview();
  });

  // ── Interactions ───────────────────────────────────────────────────────────
  function toggleItem(id: string) {
    const next = new Set(checked);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    checked = next;
  }

  async function skipItem(externalId: string) {
    // Optimistic: remove from list immediately
    items = items.filter(i => i.external_id !== externalId);
    const next = new Set(checked);
    next.delete(externalId);
    checked = next;
    try {
      await syncApi.skipSyncItem(externalId);
    } catch {
      // Non-fatal — item is already removed from the UI
    }
  }

  async function confirmSync() {
    if (checkedCount === 0) return;
    confirming = true;
    error = null;
    try {
      const ids = Array.from(checked);
      let count: number;
      if (source === 'calendar') {
        count = await syncApi.confirmCalendarSync(ids, dateRange);
      } else if (source === 'linear') {
        count = await syncApi.confirmLinearSync(ids);
      } else {
        count = await syncApi.confirmGitlabSync(ids);
      }
      syncedCount = count;
      onSynced?.(count);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      confirming = false;
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }

  // ── Helpers ────────────────────────────────────────────────────────────────
  function formatDate(dateStr: string): string {
    const d = new Date(dateStr + 'T00:00:00');
    return d.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' });
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  use:portal
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleOverlayClick}
>
  <div class="bg-[var(--color-glass-bg-raised)] border border-[var(--color-glass-border)] rounded-lg shadow-2xl backdrop-blur-xl w-full max-w-md mx-4 flex flex-col max-h-[80vh]">

    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)] shrink-0">
      <span class="text-sm font-medium text-[var(--color-text)]">Sync {sourceLabel}</span>
      <button
        type="button"
        onclick={onClose}
        class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
        aria-label="Close"
      ><X size={14} /></button>
    </div>

    <!-- Date tabs (calendar only) -->
    {#if source === 'calendar'}
      <div class="flex gap-1 px-4 pt-3 shrink-0">
        {#each ['today', 'tomorrow'] as range (range)}
          <button
            type="button"
            onclick={() => { dateRange = range as 'today' | 'tomorrow'; }}
            class="text-xs px-3 py-1 rounded border transition-colors {
              dateRange === range
                ? 'border-[var(--color-accent)] text-[var(--color-accent)] bg-[var(--color-accent)]/10'
                : 'border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)]'
            }"
          >{range === 'today' ? 'Today' : 'Tomorrow'}</button>
        {/each}
      </div>
    {/if}

    <!-- Body: loading / error / empty / list -->
    <div class="flex-1 overflow-y-auto px-4 py-3 min-h-0">
      {#if loading}
        <div class="flex items-center justify-center gap-2 py-8 text-[var(--color-muted)]">
          <Loader2 size={16} class="animate-spin" />
          <span class="text-xs">Fetching preview…</span>
        </div>
      {:else if error}
        <p class="text-xs text-rose-400 py-4 text-center">{error}</p>
      {:else if items.length === 0}
        <p class="text-xs text-[var(--color-muted)] py-8 text-center">No new items to import.</p>
      {:else}
        <ul class="flex flex-col gap-0.5">
          {#each items as item (item.external_id)}
            {@const isChecked = checked.has(item.external_id)}
            <li class="flex flex-col gap-0.5 py-2 border-b border-[var(--color-border)] last:border-0">
              <label class="flex items-start gap-2.5 cursor-pointer group">
                <input
                  type="checkbox"
                  checked={isChecked}
                  onchange={() => toggleItem(item.external_id)}
                  class="mt-0.5 shrink-0 accent-[var(--color-accent)]"
                />
                <span class="flex items-center gap-1.5 flex-wrap flex-1 min-w-0">
                  <!-- Type badge -->
                  <span class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0 {typeBadge[item.card_type] ?? 'bg-slate-500/15 text-slate-300'}">
                    {item.card_type}
                  </span>
                  <!-- Title -->
                  <span class="text-xs text-[var(--color-text)] truncate flex-1 min-w-0 group-hover:text-white transition-colors">
                    {item.title}
                  </span>
                  <!-- Date + time -->
                  <span class="text-[10px] text-[var(--color-muted)] shrink-0 tabular-nums">
                    {formatDate(item.date)}{item.start_time ? '  ' + item.start_time : ''}
                  </span>
                </span>
              </label>
              <!-- Skip forever link -->
              <button
                type="button"
                onclick={() => skipItem(item.external_id)}
                class="self-start ml-7 text-[10px] text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
              >Skip forever</button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-between gap-3 px-4 py-3 border-t border-[var(--color-border)] shrink-0">
      <div class="flex items-center gap-3">
        <button
          type="button"
          onclick={confirmSync}
          disabled={confirming || loading || checkedCount === 0}
          class="text-xs px-3 py-1.5 rounded bg-[var(--color-accent)]/80 hover:bg-[var(--color-accent)] disabled:opacity-40 disabled:cursor-not-allowed text-white transition-colors"
        >
          {confirming ? 'Importing…' : `Import ${checkedCount} selected`}
        </button>
        {#if syncedCount !== null}
          <span class="text-xs text-emerald-500/80">
            Imported {syncedCount} item{syncedCount === 1 ? '' : 's'}
          </span>
        {/if}
      </div>
      <button
        type="button"
        onclick={onClose}
        class="text-xs text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
      >Cancel</button>
    </div>

  </div>
</div>
