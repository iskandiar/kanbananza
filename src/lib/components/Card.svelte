<script lang="ts">
  import type { Card, Impact } from '$lib/types';
  import { boardStore } from '$lib/stores/board.svelte';
  import { openUrl } from '$lib/api/shell';

  let { card, onMarkDone }: { card: Card; onMarkDone: (id: number) => void } = $props();

  const typeBadge: Record<string, string> = {
    meeting: 'bg-blue-500/15 text-blue-300',
    mr:      'bg-purple-500/15 text-purple-300',
    thread:  'bg-yellow-500/15 text-yellow-300',
    task:    'bg-emerald-500/15 text-emerald-300',
    review:  'bg-slate-500/15 text-slate-300'
  };

  const impactBadge: Record<string, string> = {
    low:  'text-[var(--color-impact-low)]  bg-[var(--color-impact-low-bg)]  px-1.5 py-0.5 rounded',
    mid:  'text-[var(--color-impact-mid)]  bg-[var(--color-impact-mid-bg)]  px-1.5 py-0.5 rounded',
    high: 'text-[var(--color-impact-high)] bg-[var(--color-impact-high-bg)] px-1.5 py-0.5 rounded',
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

  const meetingTime = $derived.by(() => {
    if (card.card_type !== 'meeting' || !card.metadata) return null;
    try {
      const m = JSON.parse(card.metadata) as { start_time: string; end_time: string };
      return new Date(m.start_time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } catch { return null; }
  });

  // Editing state
  let isEditing = $state(false);
  let editTitle = $state('');
  let editImpact = $state('');
  let editHours = $state('');
  let editUrl = $state('');

  function startEdit() {
    editTitle = card.title;
    editImpact = displayImpact ?? '';
    editHours = card.time_estimate != null ? String(card.time_estimate) : '';
    editUrl = card.url ?? '';
    isEditing = true;
  }

  function cancelEdit() {
    isEditing = false;
  }

  async function saveEdit() {
    await boardStore.updateCard(card.id, {
      title: editTitle,
      ...(editImpact ? { impact: editImpact as Impact } : {}),
      ...(editHours !== '' ? { timeEstimate: Number(editHours) } : {}),
      ...(editUrl ? { url: editUrl } : {})
    });
    isEditing = false;
  }

  function focus(node: HTMLElement) {
    node.focus();
  }
</script>

<div
  class="group rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 hover:border-[var(--color-accent)]/40 hover:bg-[var(--color-surface-raised)] transition-colors"
  class:cursor-grab={!isEditing}
  class:active:cursor-grabbing={!isEditing}
  class:cursor-default={isEditing}
  class:opacity-40={card.status === 'done'}
>
  {#if meetingTime}
    <span class="text-xs text-[var(--color-muted)] mb-1 block">{meetingTime}</span>
  {/if}
  <p class="text-sm text-[var(--color-text)] leading-snug">{card.title}</p>
  {#if aiFields.description}
    <p class="text-xs text-[var(--color-muted)] mt-0.5 line-clamp-2 leading-snug">{aiFields.description}</p>
  {/if}
  <div class="mt-1.5 flex items-center gap-1.5 flex-wrap">
    <span class="text-xs px-1.5 py-0.5 rounded {typeBadge[card.card_type]}">{card.card_type}</span>
    {#if displayImpact}
      <span class="text-xs {impactBadge[displayImpact]}">{displayImpact}</span>
    {/if}
    {#if card.time_estimate}
      <span class="text-xs text-[var(--color-muted)]">{card.time_estimate}h</span>
    {/if}
    <div class="ml-auto flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
      {#if card.url}
        <button
          onclick={(e) => { e.stopPropagation(); if (card.url) openUrl(card.url); }}
          class="text-xs text-[var(--color-muted)] hover:text-[var(--color-accent-hover)]"
          aria-label="Open link"
          title="Open link"
        >↗</button>
      {/if}
      {#if !isEditing}
        <button
          onclick={startEdit}
          class="text-xs text-[var(--color-muted)] hover:text-[var(--color-accent-hover)] transition-colors"
          aria-label="Edit card"
          title="Edit card"
        >✎</button>
        {#if card.status !== 'done'}
          <button
            onclick={() => onMarkDone(card.id)}
            class="text-xs text-[var(--color-muted)] hover:text-[var(--color-done)] transition-colors"
            aria-label="Mark done"
            title="Mark done"
          >✓</button>
        {:else}
          <button
            onclick={() => boardStore.updateCard(card.id, { status: 'planned' })}
            class="text-xs text-[var(--color-muted)] hover:text-[var(--color-impact-mid)] transition-colors"
            aria-label="Undo done"
            title="Undo"
          >↩</button>
        {/if}
      {/if}
    </div>
  </div>

  {#if isEditing}
    <form
      data-no-dnd="true"
      class="mt-2 flex flex-col gap-1.5"
      onsubmit={(e) => { e.preventDefault(); saveEdit(); }}
    >
      <input
        type="text"
        bind:value={editTitle}
        use:focus
        class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-xs text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
        onkeydown={(e) => { if (e.key === 'Escape') cancelEdit(); }}
      />
      <div class="flex gap-1.5">
        {#each ['low', 'mid', 'high'] as level}
          <button
            type="button"
            onclick={() => { editImpact = editImpact === level ? '' : level; }}
            class="flex-1 text-xs py-0.5 rounded border transition-colors {editImpact === level ? 'border-[var(--color-accent)]/60 text-[var(--color-accent)] bg-[var(--color-accent)]/10' : 'border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)]'}"
          >{level}</button>
        {/each}
      </div>
      <input
        type="number"
        bind:value={editHours}
        step="0.5"
        min="0"
        placeholder="hours"
        class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-xs text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
        onkeydown={(e) => { if (e.key === 'Escape') cancelEdit(); }}
      />
      <input
        type="text"
        bind:value={editUrl}
        placeholder="https://…"
        class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-xs text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
        onkeydown={(e) => { if (e.key === 'Escape') cancelEdit(); }}
      />
      <div class="flex gap-1.5">
        <button
          type="submit"
          class="flex-1 text-xs py-1 rounded bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white transition-colors"
        >Save</button>
        <button
          type="button"
          onclick={cancelEdit}
          class="flex-1 text-xs py-1 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
        >Cancel</button>
      </div>
    </form>
  {/if}
</div>
