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
  import type { Card, CardType, Impact } from '$lib/types';
  import { boardStore } from '$lib/stores/board.svelte';
  import { openUrl } from '$lib/api/shell';
  import { Pencil, ExternalLink, Trash2, Check, Users, GitPullRequest, MessageSquare, ListTodo, Eye, FileText, X, ChevronDown, GripVertical } from 'lucide-svelte';
  import IconCalendar from './icons/IconCalendar.svelte';
  import IconGitLab from './icons/IconGitLab.svelte';
  import IconLinear from './icons/IconLinear.svelte';
  import IconSlack from './icons/IconSlack.svelte';
  import IconNotion from './icons/IconNotion.svelte';

  let { card, onMarkDone }: { card: Card; onMarkDone: (id: number) => void } = $props();

  const typeBadge: Record<string, string> = {
    meeting: 'bg-blue-500/15 text-blue-300',
    mr:      'bg-purple-500/15 text-purple-300',
    thread:  'bg-yellow-500/15 text-yellow-300',
    task:    'bg-emerald-500/15 text-emerald-300',
    review:  'bg-slate-500/15 text-slate-300',
    documentation: 'bg-slate-500/15 text-slate-300'
  };

  const typeIconCmp = {
    meeting:       Users,
    mr:            GitPullRequest,
    thread:        MessageSquare,
    task:          ListTodo,
    review:        Eye,
    documentation: FileText,
  } as const;

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

  const meetingTime = $derived(card.card_type === 'meeting' ? getMeetingTimeRange(card.metadata) : null);

  // Editing state
  let isEditing = $state(false);
  let editTitle = $state('');
  let editCardType = $state<CardType>('task');
  let editImpact = $state('');
  let editHours = $state('');
  let editUrl = $state('');
  let editNotes = $state('');
  let confirmingDelete = $state(false);
  let saveError = $state<string | null>(null);

  $effect(() => {
    if (isEditing) {
      editCardType = card.card_type;
    }
  });

  function startEdit() {
    editTitle = card.title;
    editCardType = card.card_type;
    editImpact = displayImpact ?? '';
    editHours = card.time_estimate != null ? String(card.time_estimate) : '';
    editUrl = card.url ?? '';
    editNotes = card.notes ?? '';
    isEditing = true;
  }

  function cancelEdit() {
    isEditing = false;
    confirmingDelete = false;
  }

  async function saveField(fields: Parameters<typeof boardStore.updateCard>[1]) {
    try {
      await boardStore.updateCard(card.id, fields);
      saveError = null;
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e);
    }
  }

  async function saveAndClose() {
    if (editTitle !== card.title) {
      await saveField({ title: editTitle });
    }
    cancelEdit();
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
  class="group flex flex-row gap-2 items-start rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 hover:border-[var(--color-accent)]/40 hover:bg-[var(--color-surface-raised)] transition-colors"
  class:cursor-grab={!isEditing}
  class:active:cursor-grabbing={!isEditing}
  class:cursor-default={isEditing}
  class:opacity-40={card.status === 'done'}
  role="article"
>
  <button
    data-no-dnd="true"
    onclick={handleToggleDone}
    class="flex-shrink-0 mt-0.5 w-4 h-4 rounded-full border transition-colors flex items-center justify-center {card.status === 'done' ? 'border-[var(--color-done)] bg-[var(--color-done)]/20 text-[var(--color-done)]' : 'border-[var(--color-border)] text-transparent hover:border-[var(--color-done)]/60'}"
    aria-label={card.status === 'done' ? 'Undo done' : 'Mark done'}
  >{#if card.status === 'done'}<Check size={10} />{/if}</button>

  {#if !isEditing}
    <div class="flex-shrink-0 mt-0.5 text-[var(--color-muted)] opacity-30 group-hover:opacity-70 cursor-grab">
      <GripVertical size={14} />
    </div>
  {/if}

  <div class="flex-1 min-w-0">
    {#if meetingTime}
      <span class="text-xs text-[var(--color-muted)] mb-1 block">{meetingTime}</span>
    {/if}
    <p
      class="text-sm text-[var(--color-text)] leading-snug {!isEditing ? 'cursor-text' : ''}"
      ondblclick={!isEditing ? startEdit : undefined}
      title={!isEditing ? 'Double-click to edit' : undefined}
    >{isEditing ? editTitle : card.title}</p>
    {#if aiFields.description}
      <div data-no-dnd="true" class="flex items-start gap-1">
        <p
          class="text-xs text-[var(--color-muted)] mt-0.5 leading-snug overflow-hidden max-h-[2.4em] hover:max-h-40 transition-[max-height] duration-300 ease-in-out cursor-default flex-1"
          title={aiFields.description}
        >{aiFields.description}</p>
        <span class="text-[var(--color-muted)] mt-0.5 group-hover:hidden"><ChevronDown size={12} /></span>
      </div>
    {/if}
    <div class="mt-1.5 flex items-center gap-1.5 flex-wrap">
      <span class="text-xs px-1.5 py-0.5 rounded border border-[var(--color-border)] flex items-center gap-1 {typeBadge[card.card_type] ?? 'bg-slate-500/15 text-slate-300'}">
        {#if card.card_type === 'meeting'}<Users size={10} />{/if}
        {#if card.card_type === 'mr'}<GitPullRequest size={10} />{/if}
        {#if card.card_type === 'thread'}<MessageSquare size={10} />{/if}
        {#if card.card_type === 'task'}<ListTodo size={10} />{/if}
        {#if card.card_type === 'review'}<Eye size={10} />{/if}
        {#if card.card_type === 'documentation'}<FileText size={10} />{/if}
        {typeLabel[card.card_type]}
      </span>
      {#if card.source !== 'manual'}
        <span class="flex items-center gap-1 px-1.5 py-0.5 rounded border border-[var(--color-border)] text-[var(--color-muted)] text-xs" title="Synced from {card.source}">
          {#if card.source === 'calendar'}<IconCalendar />{/if}
          {#if card.source === 'gitlab'}<IconGitLab />{/if}
          {#if card.source === 'linear'}<IconLinear />{/if}
          {#if card.source === 'slack'}<IconSlack />{/if}
          {#if card.source === 'notion'}<IconNotion />{/if}
          <span>{sourceLabel[card.source]}</span>
        </span>
      {/if}
      {#if displayImpact}
        <span class="text-xs border border-[var(--color-border)] {impactBadge[displayImpact]}">{displayImpact}</span>
      {/if}
      {#if displayImpact && card.time_estimate}
        <span class="text-xs text-[var(--color-muted)]">·</span>
      {/if}
      {#if card.time_estimate}
        <span class="text-xs text-[var(--color-muted)]">{card.time_estimate}h</span>
      {/if}
    </div>

    {#if !isEditing}
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
        {#if !isEditing}
          <button
            data-no-dnd="true"
            onclick={startEdit}
            class="text-[var(--color-muted)] hover:text-[var(--color-accent-hover)] transition-colors"
            aria-label="Edit card"
            title="Edit card"
          ><Pencil size={12} /></button>
        {/if}
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
            onclick={() => (confirmingDelete = false)}
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

    {#if isEditing}
      <div data-no-dnd="true" class="mt-2 flex flex-col gap-1.5">
        <input
          type="text"
          bind:value={editTitle}
          use:focus
          class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-xs text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
          onkeydown={(e) => { if (e.key === 'Escape') cancelEdit(); }}
        />
        <p class="text-xs text-[var(--color-muted)]">Type</p>
        <div class="flex gap-1.5">
          {#each ['task', 'meeting', 'mr', 'thread', 'review', 'documentation'] as type (type)}
            <button
              type="button"
              onclick={() => {
                editCardType = type as CardType;
                saveField({ cardType: type as CardType });
              }}
              class="flex-1 text-xs py-0.5 rounded border transition-colors {editCardType === type ? 'border-[var(--color-accent)]/60 text-[var(--color-accent)] bg-[var(--color-accent)]/10' : 'border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)]'}"
            >{typeLabel[type as CardType]}</button>
          {/each}
        </div>
        <p class="text-xs text-[var(--color-muted)]">Priority</p>
        <div class="flex gap-1.5">
          {#each ['low', 'mid', 'high'] as level (level)}
            <button
              type="button"
              onclick={() => {
                const newImpact = editImpact === level ? '' : level;
                editImpact = newImpact;
                if (newImpact === '') {
                  saveField({});
                } else {
                  saveField({ impact: newImpact as Impact });
                }
              }}
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
          onblur={() => { editHours !== '' && saveField({ timeEstimate: Number(editHours) }); }}
          onkeydown={(e) => { if (e.key === 'Escape') cancelEdit(); }}
        />
        <input
          type="text"
          bind:value={editUrl}
          placeholder="https://…"
          class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-xs text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
          onblur={() => saveField(editUrl ? { url: editUrl } : {})}
          onkeydown={(e) => { if (e.key === 'Escape') cancelEdit(); }}
        />
        <textarea
          bind:value={editNotes}
          placeholder="Notes…"
          rows="2"
          class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-xs text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)] resize-none"
          onblur={() => saveField(editNotes ? { notes: editNotes } : {})}
          onkeydown={(e) => { if (e.key === 'Escape') cancelEdit(); }}
        ></textarea>
        <button
          type="button"
          onclick={saveAndClose}
          class="text-xs py-1 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
        >Save</button>
        {#if saveError}
          <p class="text-xs text-rose-400">{saveError}</p>
        {/if}
      </div>
    {/if}
  </div>
</div>
