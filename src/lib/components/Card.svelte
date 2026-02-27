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

  const typeIcon: Record<string, string> = {
    meeting:       '⊙',
    mr:            '⎇',
    thread:        '⌘',
    task:          '◻',
    review:        '◈',
    documentation: '≡'
  };

  const sourceLabel: Record<string, string> = {
    calendar: 'GCal',
    gitlab:   'GL',
    linear:   'LN',
    slack:    'SL',
    notion:   'NT'
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
  let confirmingDelete = $state(false);

  function startEdit() {
    editTitle = card.title;
    editImpact = displayImpact ?? '';
    editHours = card.time_estimate != null ? String(card.time_estimate) : '';
    editUrl = card.url ?? '';
    isEditing = true;
  }

  function cancelEdit() {
    isEditing = false;
    confirmingDelete = false;
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

  function handleToggleDone() {
    if (card.status !== 'done') {
      onMarkDone(card.id);
    } else {
      boardStore.updateCard(card.id, { status: 'planned' });
    }
  }
</script>

<div
  class="group flex flex-row gap-2 items-start rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 hover:border-[var(--color-accent)]/40 hover:bg-[var(--color-surface-raised)] transition-colors"
  class:cursor-grab={!isEditing}
  class:active:cursor-grabbing={!isEditing}
  class:cursor-default={isEditing}
  class:opacity-40={card.status === 'done'}
>
  <button
    data-no-dnd="true"
    onclick={handleToggleDone}
    class="flex-shrink-0 mt-0.5 w-4 h-4 rounded-full border transition-colors flex items-center justify-center {card.status === 'done' ? 'border-[var(--color-done)] bg-[var(--color-done)]/20 text-[var(--color-done)]' : 'border-[var(--color-border)] text-transparent hover:border-[var(--color-done)]/60'}"
    aria-label={card.status === 'done' ? 'Undo done' : 'Mark done'}
  ><span class="text-xs">✓</span></button>

  <div class="flex-1 min-w-0">
    {#if meetingTime}
      <span class="text-xs text-[var(--color-muted)] mb-1 block">{meetingTime}</span>
    {/if}
    <p class="text-sm text-[var(--color-text)] leading-snug">{isEditing ? editTitle : card.title}</p>
    {#if aiFields.description}
      <p
        data-no-dnd="true"
        class="text-xs text-[var(--color-muted)] mt-0.5 leading-snug overflow-hidden max-h-[2.4em] hover:max-h-40 transition-[max-height] duration-300 ease-in-out cursor-default"
      >{aiFields.description}</p>
    {/if}
    <div class="mt-1.5 flex items-center gap-1.5 flex-wrap">
      <span class="text-xs px-1.5 py-0.5 rounded {typeBadge[card.card_type]}">{typeIcon[card.card_type] ?? ''} {card.card_type}</span>
      {#if card.source !== 'manual' && sourceLabel[card.source]}
        <span
          class="text-[10px] px-1 py-0.5 rounded bg-[var(--color-surface-raised)] text-[var(--color-muted)] border border-[var(--color-border)]"
          title="Synced from {card.source}"
        >{sourceLabel[card.source]}</span>
      {/if}
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
        <p class="text-xs text-[var(--color-muted)]">Priority</p>
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
          step="any"
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
        {#if !confirmingDelete}
          <button
            type="button"
            onclick={() => (confirmingDelete = true)}
            class="text-xs text-[var(--color-muted)] hover:text-rose-400 transition-colors text-left"
          >Delete card</button>
        {:else}
          <div class="flex items-center gap-1.5">
            <span class="text-xs text-[var(--color-muted)]">Delete card?</span>
            <button
              type="button"
              onclick={async () => { await boardStore.deleteCard(card.id); isEditing = false; }}
              class="text-xs px-2 py-0.5 rounded border border-rose-500/60 text-rose-400 hover:bg-rose-500/10 transition-colors"
            >Yes</button>
            <button
              type="button"
              onclick={() => (confirmingDelete = false)}
              class="text-xs px-2 py-0.5 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
            >No</button>
          </div>
        {/if}
      </form>
    {/if}
  </div>
</div>
