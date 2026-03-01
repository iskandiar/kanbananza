<script module lang="ts">
  /**
   * Extract meeting time range from metadata
   * Returns "HH:MM" for start-only, "HH:MM – HH:MM" for start+end, null otherwise
   */
  export function getMeetingTimeRange(metadata: string | null): string | null {
    if (!metadata) return null;
    try {
      const m = JSON.parse(metadata) as Record<string, unknown>;
      const opts = { hour: '2-digit', minute: '2-digit' } as const;
      const start = new Date(m.start_time as string ?? '').toLocaleTimeString([], opts);
      if (!m.end_time) return start;
      const end = new Date(m.end_time as string).toLocaleTimeString([], opts);
      return `${start} – ${end}`;
    } catch { return null; }
  }
</script>

<script lang="ts">
  import type { Card, Impact } from '$lib/types';
  import { boardStore } from '$lib/stores/board.svelte';
  import { projectsStore } from '$lib/stores/projects.svelte';
  import { openUrl } from '$lib/api/shell';
  import { Pencil, ExternalLink, Trash2, Check, Users, GitPullRequest, MessageSquare, ListTodo, Eye, FileText, X, GripVertical } from 'lucide-svelte';
  import EditCardModal from './EditCardModal.svelte';

  let { card, onMarkDone }: { card: Card; onMarkDone: (id: number) => void } = $props();

  const cardProject = $derived(
    card.project_id != null
      ? projectsStore.projects.find(p => p.id === card.project_id) ?? null
      : null
  );

  const typeBadge: Record<string, string> = {
    meeting: 'bg-blue-500/15 text-blue-300',
    mr:      'bg-purple-500/15 text-purple-300',
    thread:  'bg-yellow-500/15 text-yellow-300',
    task:    'bg-emerald-500/15 text-emerald-300',
    review:  'bg-slate-500/15 text-slate-300',
    documentation: 'bg-slate-500/15 text-slate-300'
  };

  const typeLabel: Record<string, string> = {
    meeting: 'meeting',
    mr: 'MR',
    thread: 'thread',
    task: 'task',
    review: 'review',
    documentation: 'doc',
  };

  const sourceLabel: Record<string, string> = {
    calendar: 'gcal',
    gitlab: 'gitlab',
    linear: 'linear',
    slack: 'slack',
    notion: 'notion',
  };

  const impactBadge: Record<string, string> = {
    low:  'text-[var(--color-impact-low)]  bg-[var(--color-impact-low-bg)]  px-1 py-px rounded',
    mid:  'text-[var(--color-impact-mid)]  bg-[var(--color-impact-mid-bg)]  px-1 py-px rounded',
    high: 'text-[var(--color-impact-high)] bg-[var(--color-impact-high-bg)] px-1 py-px rounded',
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

  const meetingTime = $derived(card.card_type === 'meeting' ? getMeetingTimeRange(card.metadata) : null);

  // Editing state
  let isTitleEditing = $state(false);
  let editTitle = $state('');
  let isPopoverOpen = $state(false);
  let confirmingDelete = $state(false);
  let saveError = $state<string | null>(null);

  function startTitleEdit() {
    editTitle = card.title;
    isTitleEditing = true;
  }

  function cancelTitleEdit() {
    isTitleEditing = false;
  }

  async function saveTitleEdit() {
    if (editTitle !== card.title) {
      try {
        await boardStore.updateCard(card.id, { title: editTitle });
        saveError = null;
      } catch (e) {
        saveError = e instanceof Error ? e.message : String(e);
      }
    }
    isTitleEditing = false;
  }

  function openPopover() {
    isPopoverOpen = true;
  }

  function closePopover() {
    isPopoverOpen = false;
  }

  function cancelDelete() {
    confirmingDelete = false;
  }

  async function deleteCard() {
    try {
      await boardStore.deleteCard(card.id);
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e);
    }
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
  class="group relative flex flex-row gap-2 items-start rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 hover:border-[var(--color-accent)]/40 hover:bg-[var(--color-surface-raised)] transition-colors"
  class:cursor-grab={!isTitleEditing && !isPopoverOpen}
  class:active:cursor-grabbing={!isTitleEditing && !isPopoverOpen}
  class:cursor-default={isTitleEditing || isPopoverOpen}
  class:opacity-40={card.status === 'done'}
  role="article"
>
  <div class="flex flex-col items-center justify-between flex-shrink-0 self-stretch">
    <button
      data-no-dnd="true"
      onclick={handleToggleDone}
      class="flex-shrink-0 w-4 h-4 rounded-full border transition-colors flex items-center justify-center {card.status === 'done' ? 'border-[var(--color-done)] bg-[var(--color-done)]/20 text-[var(--color-done)]' : 'border-[var(--color-border)] text-transparent hover:border-[var(--color-done)]/60'}"
      aria-label={card.status === 'done' ? 'Undo done' : 'Mark done'}
    >{#if card.status === 'done'}<Check size={10} />{/if}</button>

    {#if !isTitleEditing && !isPopoverOpen}
      <div class="text-[var(--color-muted)] opacity-30 group-hover:opacity-70 cursor-grab mt-auto">
        <GripVertical size={14} />
      </div>
    {/if}
  </div>

  <div class="flex-1 min-w-0">
    {#if meetingTime}
      <span class="text-xs text-[var(--color-muted)] mb-1 block">{meetingTime}</span>
    {/if}

    {#if isTitleEditing}
      <input
        type="text"
        bind:value={editTitle}
        use:focus
        class="w-full text-sm font-medium text-[var(--color-text)] bg-transparent border-0 border-b-2 border-[var(--color-accent)] px-0 py-0 focus:outline-none focus:ring-0"
        onkeydown={(e) => { if (e.key === 'Enter') saveTitleEdit(); if (e.key === 'Escape') cancelTitleEdit(); }}
        onblur={saveTitleEdit}
      />
    {:else}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <p
        class="text-sm font-medium text-[var(--color-text)] leading-snug cursor-text hover:text-[var(--color-accent)]/80 transition-colors"
        onclick={startTitleEdit}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') startTitleEdit(); }}
        title="Click to edit title"
      >{card.title}</p>
    {/if}

    {#if aiFields.description && !isTitleEditing && !isPopoverOpen}
      <p
        data-no-dnd="true"
        class="text-xs text-[var(--color-muted)] leading-snug mt-1 line-clamp-2 group-hover:line-clamp-none transition-all cursor-default"
        title={aiFields.description}
      >{aiFields.description}</p>
    {/if}

    <div class="mt-1.5 flex items-center gap-1.5 flex-wrap">
      <!-- Project slug badge -->
      {#if cardProject}
        <span
          class="text-xs px-1 py-px rounded border font-mono font-semibold"
          style="border-color: {cardProject.color}40; background-color: {cardProject.color}15; color: {cardProject.color};"
        >{cardProject.slug}</span>
      {/if}

      <!-- Type badge: icon + expanding label on hover -->
      <span
        class="text-xs px-1 py-px rounded border border-[var(--color-border)] flex items-center gap-1 {typeBadge[card.card_type] ?? 'bg-slate-500/15 text-slate-300'}"
        title={card.source !== 'manual' ? `${typeLabel[card.card_type]} · ${sourceLabel[card.source]}` : typeLabel[card.card_type]}
      >
        {#if card.card_type === 'meeting'}<Users size={10} />{/if}
        {#if card.card_type === 'mr'}<GitPullRequest size={10} />{/if}
        {#if card.card_type === 'thread'}<MessageSquare size={10} />{/if}
        {#if card.card_type === 'task'}<ListTodo size={10} />{/if}
        {#if card.card_type === 'review'}<Eye size={10} />{/if}
        {#if card.card_type === 'documentation'}<FileText size={10} />{/if}
        <span class="max-w-0 overflow-hidden group-hover:max-w-[4rem] transition-all duration-200 whitespace-nowrap text-[0.65rem]">{typeLabel[card.card_type]}</span>
      </span>

      <!-- Impact badge -->
      {#if displayImpact}
        <span class="text-xs border border-[var(--color-border)] {impactBadge[displayImpact]}">{displayImpact}</span>
      {/if}

      <!-- Time estimate -->
      {#if card.time_estimate}
        <span class="text-xs text-[var(--color-muted)]">{card.time_estimate}h</span>
      {/if}
    </div>

    {#if !isTitleEditing && !isPopoverOpen}
      <div class="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity mt-0.5">
        {#if card.url}
          <button
            data-no-dnd="true"
            onclick={(e) => { e.stopPropagation(); if (card.url) openUrl(card.url); }}
            class="text-[var(--color-muted)] hover:text-[var(--color-accent-hover)] transition-colors"
            aria-label="Open link"
            title="Open link"
          ><ExternalLink size={12} /></button>
        {/if}
        <button
          data-no-dnd="true"
          onclick={openPopover}
          class="text-[var(--color-muted)] hover:text-[var(--color-accent-hover)] transition-colors"
          aria-label="Edit card"
          title="Edit card"
        ><Pencil size={12} /></button>
        {#if !confirmingDelete}
          <button
            data-no-dnd="true"
            onclick={() => (confirmingDelete = true)}
            class="text-[var(--color-muted)] hover:text-rose-400 transition-colors"
            aria-label="Delete card"
            title="Delete card"
          ><Trash2 size={12} /></button>
        {:else}
          <span class="text-xs text-rose-400">Delete?</span>
          <button
            data-no-dnd="true"
            onclick={cancelDelete}
            class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
            aria-label="Cancel delete"
          ><X size={12} /></button>
          <button
            data-no-dnd="true"
            onclick={deleteCard}
            class="text-rose-400 hover:text-rose-300 transition-colors"
            aria-label="Confirm delete"
          ><Check size={12} /></button>
        {/if}
      </div>
    {/if}
  </div>

</div>

{#if isPopoverOpen}
  <EditCardModal {card} onClose={closePopover} />
{/if}
