<script lang="ts">
  import type { Card } from '$lib/types';
  import {
    Eye,
    FileText,
    GitPullRequest,
    ListTodo,
    MessageSquare,
    Pencil,
    Trash2,
    Users,
    X,
    Check
  } from 'lucide-svelte';
  import EditCardModal from './EditCardModal.svelte';

  let {
    card,
    onMarkDone,
    onMoveToBacklog,
    onDelete
  }: {
    card: Card;
    onMarkDone: (id: number) => void;
    onMoveToBacklog: (id: number) => void;
    onDelete: (id: number) => void;
  } = $props();

  const typeBadge: Record<string, string> = {
    meeting: 'bg-blue-500/15 text-blue-300',
    mr: 'bg-purple-500/15 text-purple-300',
    thread: 'bg-yellow-500/15 text-yellow-300',
    task: 'bg-emerald-500/15 text-emerald-300',
    review: 'bg-slate-500/15 text-slate-300',
    documentation: 'bg-slate-500/15 text-slate-300'
  };

  const typeLabel: Record<string, string> = {
    meeting: 'meeting',
    mr: 'MR',
    thread: 'thread',
    task: 'task',
    review: 'review',
    documentation: 'doc'
  };

  const impactBadge: Record<string, string> = {
    low: 'text-[var(--color-impact-low)]  bg-[var(--color-impact-low-bg)]  px-1 py-px rounded',
    mid: 'text-[var(--color-impact-mid)]  bg-[var(--color-impact-mid-bg)]  px-1 py-px rounded',
    high: 'text-[var(--color-impact-high)] bg-[var(--color-impact-high-bg)] px-1 py-px rounded'
  };

  let isEditing = $state(false);
  let confirmingDelete = $state(false);

  const doneDate = $derived.by(() => {
    if (!card.done_at) return null;
    try {
      return new Date(card.done_at).toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric'
      });
    } catch {
      return null;
    }
  });
</script>

<div
  class="group relative flex flex-col gap-1.5 rounded-md border border-[var(--color-glass-border)] bg-[var(--color-glass-bg)] px-3 py-2 hover:border-[var(--color-accent)]/40 hover:bg-[var(--color-glass-bg-raised)] transition-colors cursor-grab active:cursor-grabbing glass-card backdrop-blur-sm"
  class:opacity-50={card.status === 'done'}
  role="article"
>
  <p class="text-sm font-medium text-[var(--color-text)] leading-snug">{card.title}</p>

  <div class="flex items-center gap-1.5 flex-wrap">
    <span
      class="text-xs px-1 py-px rounded border border-[var(--color-border)] flex items-center gap-1 {typeBadge[
        card.card_type
      ] ?? 'bg-slate-500/15 text-slate-300'}"
    >
      {#if card.card_type === 'meeting'}<Users size={10} />{/if}
      {#if card.card_type === 'mr'}<GitPullRequest size={10} />{/if}
      {#if card.card_type === 'thread'}<MessageSquare size={10} />{/if}
      {#if card.card_type === 'task'}<ListTodo size={10} />{/if}
      {#if card.card_type === 'review'}<Eye size={10} />{/if}
      {#if card.card_type === 'documentation'}<FileText size={10} />{/if}
      <span class="text-[0.65rem]">{typeLabel[card.card_type]}</span>
    </span>
    {#if card.impact}
      <span class="text-xs border border-[var(--color-border)] {impactBadge[card.impact]}"
        >{card.impact}</span
      >
    {/if}
    {#if doneDate}
      <span class="text-xs text-[var(--color-muted)]">Done {doneDate}</span>
    {/if}
  </div>

  <div class="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
    <button
      data-no-dnd="true"
      onclick={() => (isEditing = true)}
      class="text-[var(--color-muted)] hover:text-[var(--color-accent-hover)] transition-colors"
      aria-label="Edit"
    ><Pencil size={12} /></button>
    {#if card.status === 'done'}
      <button
        data-no-dnd="true"
        onclick={() => onMoveToBacklog(card.id)}
        class="text-xs text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
        title="Move to backlog"
      >↩</button>
    {:else}
      <button
        data-no-dnd="true"
        onclick={() => onMarkDone(card.id)}
        class="text-[var(--color-muted)] hover:text-[var(--color-done)] transition-colors"
        aria-label="Mark done"
        title="Mark done"
      ><Check size={12} /></button>
    {/if}
    {#if !confirmingDelete}
      <button
        data-no-dnd="true"
        onclick={() => (confirmingDelete = true)}
        class="text-[var(--color-muted)] hover:text-rose-400 transition-colors"
        aria-label="Delete"
      ><Trash2 size={12} /></button>
    {:else}
      <span class="text-xs text-rose-400">Delete?</span>
      <button
        data-no-dnd="true"
        onclick={() => (confirmingDelete = false)}
        class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
      ><X size={12} /></button>
      <button
        data-no-dnd="true"
        onclick={() => onDelete(card.id)}
        class="text-rose-400 hover:text-rose-300 transition-colors"
      ><Check size={12} /></button>
    {/if}
  </div>
</div>

{#if isEditing}
  <EditCardModal {card} onClose={() => (isEditing = false)} />
{/if}
