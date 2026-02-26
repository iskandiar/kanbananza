<script lang="ts">
  import { onMount } from 'svelte';
  import WeekBoard from '$lib/components/WeekBoard.svelte';
  import { boardStore } from '$lib/stores/board.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  const DAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri'];

  onMount(async () => {
    await Promise.all([boardStore.loadCurrentWeek(), settingsStore.load()]);
  });

  const weekLabel = $derived(() => {
    const w = boardStore.currentWeek;
    if (!w) return '…';
    return `W${w.week_number} · ${w.start_date}`;
  });

  const days = $derived(() => {
    const w = boardStore.currentWeek;
    if (!w) return [];
    const monday = new Date(w.start_date);
    return DAY_LABELS.map((label, i) => {
      const d = new Date(monday);
      d.setDate(monday.getDate() + i);
      const date = d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
      return {
        label,
        date,
        dayOfWeek: i + 1,
        weekId: w.id,
        meetings: boardStore.meetingsByDay.get(i + 1) ?? [],
        tasks: boardStore.tasksByDay.get(i + 1) ?? []
      };
    });
  });

  async function handleAddCard(dayOfWeek: number | null, title: string) {
    await boardStore.addCard(title, boardStore.currentWeek?.id ?? null, dayOfWeek);
  }

  async function handleMoveCard(
    cardId: number,
    weekId: number | null,
    dayOfWeek: number | null,
    position: number
  ) {
    await boardStore.moveCard(cardId, weekId, dayOfWeek, position);
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
    days={days()}
    backlogCards={boardStore.backlog}
    availableHours={settingsStore.availableHours}
    onPrevWeek={() => boardStore.navigateWeek(-1)}
    onNextWeek={() => boardStore.navigateWeek(1)}
    onAddCard={handleAddCard}
    onMoveCard={handleMoveCard}
    onMarkDone={(id) => boardStore.markDone(id)}
    onRollover={() => boardStore.rollover()}
  />
{/if}
