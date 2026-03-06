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
    isCurrentWeek = true,
    sidebarOpen = false,
    onPrevWeek,
    onNextWeek,
    onJumpToToday,
    onAddCard,
    onMoveCard,
    onMarkDone,
    onRollover,
    onToggleSidebar,
    onCardCreated,
    onMoveCardToNextWeek,
    onScheduleToday
  }: {
    weekLabel: string;
    days: Array<{ label: string; date: string; displayDate: string; dayOfWeek: number; weekId: number | null; isToday: boolean; meetings: Card[]; tasks: Card[] }>;
    backlogCards: Card[];
    availableHours: number;
    isCurrentWeek: boolean;
    sidebarOpen?: boolean;
    onPrevWeek: () => void;
    onNextWeek: () => void;
    onJumpToToday: () => void;
    onAddCard: (dayOfWeek: number | null, title: string) => void;
    onMoveCard: (cardId: number, weekId: number | null, dayOfWeek: number | null, position: number) => void;
    onMarkDone: (cardId: number) => void;
    onRollover: () => void;
    onToggleSidebar?: () => void;
    onCardCreated?: (card: Card) => void;
    onMoveCardToNextWeek?: (id: number) => void;
    onScheduleToday?: (id: number) => void;
  } = $props();

  let clockedByDay = $state<Record<string, number>>({});
  const weeklyClocked = $derived(Object.values(clockedByDay).reduce((s, h) => s + h, 0));

  const unfinishedCount = $derived(
    days.flatMap(d => d.tasks).filter(c => c.status === 'planned').length
  );
</script>

<div class="flex flex-col h-screen text-[var(--color-text)]">
  <WeekHeader {weekLabel} onPrev={onPrevWeek} onNext={onNextWeek} {isCurrentWeek} {onJumpToToday} {onRollover} {unfinishedCount} clockedHours={weeklyClocked} />

  <div class="flex flex-1 min-h-0">
    <div class="flex flex-1 min-w-0 overflow-x-auto">
      {#each days as day (day.date)}
        <DayColumn
          label={day.label}
          date={day.date}
          displayDate={day.displayDate}
          dayOfWeek={day.dayOfWeek}
          weekId={day.weekId}
          isToday={day.isToday}
          meetings={day.meetings}
          tasks={day.tasks}
          {availableHours}
          onAddCard={(title: string) => onAddCard(day.dayOfWeek, title)}
          {onMoveCard}
          {onMarkDone}
          {onCardCreated}
          onMoveToNextWeek={onMoveCardToNextWeek}
          onClockedUpdate={(hours) => { clockedByDay = { ...clockedByDay, [day.date]: hours }; }}
        />
      {/each}
    </div>

    <BacklogSidebar
      cards={backlogCards}
      isOpen={sidebarOpen}
      onAddCard={(title) => onAddCard(null, title)}
      onClose={() => onToggleSidebar?.()}
      {onMoveCard}
      {onMarkDone}
      {onCardCreated}
      {onScheduleToday}
    />

    {#if !sidebarOpen}
      <button
        onclick={() => onToggleSidebar?.()}
        class="flex-shrink-0 w-12 border-l border-[var(--color-border)] flex flex-col items-center justify-center text-[var(--color-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface)] transition-colors gap-0.5"
        title="Open backlog"
      >
        <span class="text-xs">≡</span>
        <span class="text-xs">{backlogCards.length}</span>
      </button>
    {/if}
  </div>
</div>
