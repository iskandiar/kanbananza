<script lang="ts">
  import type { Card } from '$lib/types';
  import CardComponent from './Card.svelte';
  import LoadIndicator from './LoadIndicator.svelte';
  import QuickAdd from './QuickAdd.svelte';

  let {
    label,
    date,
    meetings = [],
    tasks = [],
    availableHours,
    onAddCard
  }: {
    label: string;
    date: string;
    meetings: Card[];
    tasks: Card[];
    availableHours: number;
    onAddCard: (title: string) => void;
  } = $props();

  const meetingHours = $derived(
    meetings.reduce((sum, m) => {
      if (!m.metadata) return sum;
      try {
        const meta = JSON.parse(m.metadata);
        const start = new Date(meta.start_time);
        const end = new Date(meta.end_time);
        return sum + (end.getTime() - start.getTime()) / 3_600_000;
      } catch { return sum; }
    }, 0)
  );

  const taskHours = $derived(tasks.reduce((sum, t) => sum + (t.time_estimate ?? 0), 0));
  const scheduledHours = $derived(meetingHours + taskHours);
</script>

<div class="flex flex-col min-w-0 flex-1 border-r border-[var(--color-border)] last:border-r-0 px-3 py-3 gap-3">
  <div>
    <p class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wide">{label}</p>
    <p class="text-xs text-[var(--color-muted)]">{date}</p>
  </div>
  <LoadIndicator {scheduledHours} {availableHours} />

  {#if meetings.length}
    <div class="flex flex-col gap-1.5">
      {#each meetings as card (card.id)}
        <CardComponent {card} />
      {/each}
    </div>
  {/if}

  <div class="flex flex-col gap-1.5 flex-1">
    {#each tasks as card (card.id)}
      <CardComponent {card} />
    {/each}
    <QuickAdd onAdd={onAddCard} />
  </div>
</div>
