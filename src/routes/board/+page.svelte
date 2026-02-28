<script lang="ts">
  import { onMount } from 'svelte';
  import WeekBoard from '$lib/components/WeekBoard.svelte';
  import { boardStore } from '$lib/stores/board.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Card } from '$lib/types';

  onMount(async () => {
    await Promise.all([boardStore.loadCurrentWeek(), settingsStore.load()]);
  });

  const weekLabel = $derived(() => {
    const w = boardStore.currentWeek;
    if (!w) return '…';
    return `W${w.week_number} · ${w.start_date}`;
  });

  async function handleAddCard(dayOfWeek: number | null, title: string) {
    // Backlog cards have no week assignment (week_id = null)
    const weekId = dayOfWeek !== null ? (boardStore.currentWeek?.id ?? null) : null;
    await boardStore.addCard(title, weekId, dayOfWeek);
  }

  async function handleMoveCard(
    cardId: number,
    weekId: number | null,
    dayOfWeek: number | null,
    position: number
  ) {
    await boardStore.moveCard(cardId, weekId, dayOfWeek, position);
  }

  function handleCardCreated(card: Card) {
    boardStore.cards = [...boardStore.cards, card];
  }
</script>

{#if boardStore.error}
  <div class="flex items-center justify-center h-screen bg-[var(--color-background)] text-rose-400 text-sm">
    {boardStore.error}
  </div>
{:else if boardStore.isLoading && !boardStore.currentWeek}
  <div class="flex items-center justify-center h-screen bg-[var(--color-background)] text-[var(--color-muted)] text-sm">
    Loading…
  </div>
{:else}
  <WeekBoard
    weekLabel={weekLabel()}
    days={boardStore.days}
    backlogCards={boardStore.backlog}
    availableHours={settingsStore.availableHours}
    isCurrentWeek={boardStore.isCurrentWeek}
    onPrevWeek={() => boardStore.navigateWeek(-1)}
    onNextWeek={() => boardStore.navigateWeek(1)}
    onJumpToToday={() => boardStore.loadCurrentWeek()}
    onAddCard={handleAddCard}
    onMoveCard={handleMoveCard}
    onMarkDone={(id) => boardStore.markDone(id)}
    onRollover={() => boardStore.rollover()}
    onCardCreated={handleCardCreated}
  />
{/if}
