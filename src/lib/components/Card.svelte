<script lang="ts">
  import type { Card, MeetingMetadata } from '$lib/types';

  let { card, onMarkDone }: { card: Card; onMarkDone: (id: number) => void } = $props();

  const typeBadge: Record<string, string> = {
    meeting: 'bg-blue-500/20 text-blue-300',
    mr: 'bg-purple-500/20 text-purple-300',
    thread: 'bg-yellow-500/20 text-yellow-300',
    task: 'bg-emerald-500/20 text-emerald-300',
    review: 'bg-slate-500/20 text-slate-300'
  };

  const impactBadge: Record<string, string> = {
    low: 'text-[var(--color-muted)]',
    mid: 'text-amber-400',
    high: 'text-rose-400'
  };

  const meetingTime = $derived.by(() => {
    if (card.card_type !== 'meeting' || !card.metadata) return null;
    try {
      const m = JSON.parse(card.metadata) as MeetingMetadata;
      return new Date(m.start_time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } catch { return null; }
  });
</script>

<div
  class="group rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 cursor-grab active:cursor-grabbing hover:border-indigo-500/40 transition-colors"
  class:opacity-50={card.status === 'done'}
>
  {#if meetingTime}
    <span class="text-xs text-[var(--color-muted)] mb-1 block">{meetingTime}</span>
  {/if}
  <p class="text-sm text-[var(--color-text)] leading-snug">{card.title}</p>
  <div class="mt-1.5 flex items-center gap-1.5 flex-wrap">
    <span class="text-xs px-1.5 py-0.5 rounded {typeBadge[card.card_type]}">{card.card_type}</span>
    {#if card.impact}
      <span class="text-xs {impactBadge[card.impact]}">{card.impact}</span>
    {/if}
    {#if card.time_estimate}
      <span class="text-xs text-[var(--color-muted)]">{card.time_estimate}h</span>
    {/if}
    <div class="ml-auto flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
      {#if card.url}
        <a
          href={card.url}
          target="_blank"
          class="text-xs text-[var(--color-muted)] hover:text-indigo-400"
          onclick={(e) => e.stopPropagation()}
        >↗</a>
      {/if}
      {#if card.status !== 'done'}
        <button
          onclick={() => onMarkDone(card.id)}
          class="text-xs text-[var(--color-muted)] hover:text-emerald-400 transition-colors"
          aria-label="Mark done"
          title="Mark done"
        >✓</button>
      {/if}
    </div>
  </div>
</div>
