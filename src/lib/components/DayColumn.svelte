<script lang="ts">
  import type { Card } from '$lib/types';
  import { sumHours } from '$lib/utils';
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
    isToday = false,
    onAddCard,
    onMoveCard,
    onMarkDone,
    onCardCreated
  }: {
    label: string;
    date: string;
    dayOfWeek: number;
    weekId: number | null;
    meetings: Card[];
    tasks: Card[];
    availableHours: number;
    isToday?: boolean;
    onAddCard: (title: string) => void;
    onMoveCard: (cardId: number, weekId: number | null, dayOfWeek: number | null, position: number) => void;
    onMarkDone: (cardId: number) => void;
    onCardCreated?: (card: Card) => void;
  } = $props();

  // Include both tasks and meetings in load calculation
  const doneHoursByType = $derived(
    sumHours([...tasks.filter(t => t.status === 'done'), ...meetings.filter(m => m.status === 'done')])
  );
  const plannedHoursByType = $derived(
    sumHours([...tasks.filter(t => t.status !== 'done'), ...meetings.filter(m => m.status !== 'done')])
  );

  // Pending tasks for DnD zone — initialized empty; kept mutable for svelte-dnd-action
  // The $effect syncs updates, preventing stale cached state when parent re-renders.
  let localPendingTasks = $state<Card[]>([]);
  $effect(() => { localPendingTasks = tasks.filter(t => t.status !== 'done'); });

  // Done and pending tasks/meetings for collapsed section
  const doneTasks = $derived(tasks.filter(t => t.status === 'done'));
  const doneMeetings = $derived(meetings.filter(m => m.status === 'done'));
  const pendingMeetings = $derived(meetings.filter(m => m.status !== 'done'));
  let showDone = $state(false);

  function handleDndConsider(e: CustomEvent<{ items: Card[] }>) {
    localPendingTasks = e.detail.items;
  }

  function handleDndFinalize(e: CustomEvent<{ items: Card[] }>) {
    localPendingTasks = e.detail.items;
    localPendingTasks.forEach((card, i) => {
      // Use column's weekId (not card's) so backlog→column drops get the right week assigned
      if (card.day_of_week !== dayOfWeek || card.week_id !== weekId || card.position !== i) {
        onMoveCard(card.id, weekId, dayOfWeek, i);
      }
    });
  }
</script>

<div
  class="flex flex-col min-w-0 flex-1 border-r border-[var(--color-glass-border)] last:border-r-0 px-3 py-3 gap-3 backdrop-blur-[2px] {isToday ? 'bg-[var(--color-glass-bg)]' : ''}"
>
  <div>
    <p
      class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wide"
      class:text-[var(--color-text)]={isToday}
    >{label}</p>
    <p
      class="text-xs text-[var(--color-muted)]"
      class:text-[var(--color-accent)]={isToday}
    >{date}</p>
  </div>
  <LoadIndicator doneHours={doneHoursByType} plannedHours={plannedHoursByType} {availableHours} />

  {#if pendingMeetings.length}
    <div class="flex flex-col gap-1.5">
      {#each pendingMeetings as card (card.id)}
        <CardComponent {card} {onMarkDone} />
      {/each}
    </div>
  {/if}

  <div
    class="flex flex-col gap-1.5 flex-1 min-h-[2rem]"
    use:dndzone={{
      items: localPendingTasks,
      flipDurationMs: 150,
      dropTargetStyle: { outline: 'none', background: 'rgba(61,126,255,0.07)', 'border-radius': '6px' }
    }}
    onconsider={handleDndConsider}
    onfinalize={handleDndFinalize}
  >
    {#each localPendingTasks as card (card.id)}
      <CardComponent {card} {onMarkDone} />
    {/each}
  </div>

  {#if doneTasks.length + doneMeetings.length > 0}
    <div>
      <button
        onclick={() => (showDone = !showDone)}
        class="text-xs text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
      >
        {showDone ? '▾' : '▸'} {doneTasks.length + doneMeetings.length} done · {doneHoursByType.toFixed(1)}h consumed
      </button>
      {#if showDone}
        <div class="flex flex-col gap-1.5 mt-1.5">
          {#each doneMeetings as card (card.id)}
            <CardComponent {card} {onMarkDone} />
          {/each}
          {#each doneTasks as card (card.id)}
            <CardComponent {card} {onMarkDone} />
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <QuickAdd onAdd={onAddCard} {weekId} {dayOfWeek} {onCardCreated} />
</div>
