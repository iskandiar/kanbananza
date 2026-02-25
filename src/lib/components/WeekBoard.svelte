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
    onAddCard
  }: {
    weekLabel: string;
    days: Array<{ label: string; date: string; meetings: Card[]; tasks: Card[] }>;
    backlogCards: Card[];
    availableHours: number;
    onPrevWeek: () => void;
    onNextWeek: () => void;
    onAddCard: (dayIndex: number | null, title: string) => void;
  } = $props();

  let backlogOpen = $state(false);
</script>

<div class="flex flex-col h-screen bg-[var(--color-background)] text-[var(--color-text)]">
  <WeekHeader {weekLabel} onPrev={onPrevWeek} onNext={onNextWeek} />

  <div class="flex flex-1 min-h-0">
    <div class="flex flex-1 min-w-0 overflow-x-auto">
      {#each days as day, i (day.date)}
        <DayColumn
          label={day.label}
          date={day.date}
          meetings={day.meetings}
          tasks={day.tasks}
          {availableHours}
          onAddCard={(title) => onAddCard(i, title)}
        />
      {/each}
    </div>

    <BacklogSidebar
      cards={backlogCards}
      isOpen={backlogOpen}
      onAddCard={(title) => onAddCard(null, title)}
      onClose={() => (backlogOpen = false)}
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
