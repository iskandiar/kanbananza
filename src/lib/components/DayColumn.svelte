<script lang="ts">
  import type { Card } from '$lib/types';
  import { dndzone } from 'svelte-dnd-action';
  import CardComponent from './Card.svelte';
  import LoadIndicator from './LoadIndicator.svelte';
  import QuickAdd from './QuickAdd.svelte';

  let {
    label,
    date,
    dayOfWeek,
    weekId,
    meetings = [],
    tasks = [],
    availableHours,
    onAddCard,
    onMoveCard,
    onMarkDone
  }: {
    label: string;
    date: string;
    dayOfWeek: number;
    weekId: number | null;
    meetings: Card[];
    tasks: Card[];
    availableHours: number;
    onAddCard: (title: string) => void;
    onMoveCard: (cardId: number, weekId: number | null, dayOfWeek: number | null, position: number) => void;
    onMarkDone: (cardId: number) => void;
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

  // Local copy for optimistic DnD reordering
  let localTasks = $state<Card[]>([]);
  $effect(() => { localTasks = tasks; });

  function handleDndConsider(e: CustomEvent<{ items: Card[] }>) {
    localTasks = e.detail.items;
  }

  function handleDndFinalize(e: CustomEvent<{ items: Card[] }>) {
    localTasks = e.detail.items;
    localTasks.forEach((card, i) => {
      // Use column's weekId (not card's) so backlog→column drops get the right week assigned
      if (card.day_of_week !== dayOfWeek || card.week_id !== weekId || card.position !== i) {
        onMoveCard(card.id, weekId, dayOfWeek, i);
      }
    });
  }
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
        <CardComponent {card} {onMarkDone} />
      {/each}
    </div>
  {/if}

  <div
    class="flex flex-col gap-1.5 flex-1 min-h-[2rem]"
    use:dndzone={{
      items: localTasks,
      flipDurationMs: 150,
      dropTargetStyle: { outline: 'none', background: 'rgba(61,126,255,0.07)', 'border-radius': '6px' }
    }}
    onconsider={handleDndConsider}
    onfinalize={handleDndFinalize}
  >
    {#each localTasks as card (card.id)}
      <CardComponent {card} {onMarkDone} />
    {/each}
  </div>

  <QuickAdd onAdd={onAddCard} />
</div>
