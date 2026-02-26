<script lang="ts">
  import type { Card } from '$lib/types';
  import WeekHeader from './WeekHeader.svelte';
  import DayColumn from './DayColumn.svelte';
  import BacklogSidebar from './BacklogSidebar.svelte';

  let {
    weekLabel,
    days,
    backlogCards = [],
    availableHours,
    onPrevWeek,
    onNextWeek,
    onAddCard,
    onMoveCard,
    onMarkDone,
    onRollover
  }: {
    weekLabel: string;
    days: Array<{ label: string; date: string; dayOfWeek: number; weekId: number | null; meetings: Card[]; tasks: Card[] }>;
    backlogCards: Card[];
    availableHours: number;
    onPrevWeek: () => void;
    onNextWeek: () => void;
    onAddCard: (dayOfWeek: number | null, title: string) => void;
    onMoveCard: (cardId: number, weekId: number | null, dayOfWeek: number | null, position: number) => void;
    onMarkDone: (cardId: number) => void;
    onRollover: () => void;
  } = $props();

  let backlogOpen = $state(false);
</script>

<div class="flex flex-col h-screen bg-[var(--color-background)] text-[var(--color-text)]">
  <WeekHeader {weekLabel} onPrev={onPrevWeek} onNext={onNextWeek} {onRollover} />

  <div class="flex flex-1 min-h-0">
    <div class="flex flex-1 min-w-0 overflow-x-auto">
      {#each days as day (day.date)}
        <DayColumn
          label={day.label}
          date={day.date}
          dayOfWeek={day.dayOfWeek}
          weekId={day.weekId}
          meetings={day.meetings}
          tasks={day.tasks}
          {availableHours}
          onAddCard={(title) => onAddCard(day.dayOfWeek, title)}
          {onMoveCard}
          {onMarkDone}
        />
      {/each}
    </div>

    <BacklogSidebar
      cards={backlogCards}
      isOpen={backlogOpen}
      onAddCard={(title) => onAddCard(null, title)}
      onClose={() => (backlogOpen = false)}
      {onMoveCard}
      {onMarkDone}
    />

    {#if !backlogOpen}
      <button
        onclick={() => (backlogOpen = true)}
        class="flex-shrink-0 w-8 border-l border-[var(--color-border)] flex items-center justify-center text-[var(--color-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface)] transition-colors text-xs"
        title="Open backlog"
      >
        ≡
      </button>
    {/if}
  </div>
</div>
