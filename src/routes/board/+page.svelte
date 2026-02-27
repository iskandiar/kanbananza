<script lang="ts">
  import { onMount } from 'svelte';
  import WeekBoard from '$lib/components/WeekBoard.svelte';
  import { boardStore } from '$lib/stores/board.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Card } from '$lib/types';

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

    // Compute today's ISO week start date
    const today = new Date();
    const todayMonday = new Date(today);
    todayMonday.setDate(today.getDate() - ((today.getDay() + 6) % 7));
    const todayWeekStart = [
      todayMonday.getFullYear(),
      String(todayMonday.getMonth() + 1).padStart(2, '0'),
      String(todayMonday.getDate()).padStart(2, '0')
    ].join('-');
    const isCurrentWeek = w.start_date === todayWeekStart;
    // today.getDay(): 0=Sun,1=Mon,...,6=Sat → DOW 1-5: ((day+6)%7)+1
    const todayDOW = isCurrentWeek ? ((today.getDay() + 6) % 7) + 1 : null;

    return DAY_LABELS.map((label, i) => {
      const d = new Date(monday);
      d.setDate(monday.getDate() + i);
      const date = d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
      return {
        label,
        date,
        dayOfWeek: i + 1,
        weekId: w.id,
        isToday: todayDOW === i + 1,
        meetings: boardStore.meetingsByDay.get(i + 1) ?? [],
        tasks: boardStore.tasksByDay.get(i + 1) ?? []
      };
    });
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
    days={days()}
    backlogCards={boardStore.backlog}
    availableHours={settingsStore.availableHours}
    onPrevWeek={() => boardStore.navigateWeek(-1)}
    onNextWeek={() => boardStore.navigateWeek(1)}
    onAddCard={handleAddCard}
    onMoveCard={handleMoveCard}
    onMarkDone={(id) => boardStore.markDone(id)}
    onRollover={() => boardStore.rollover()}
    onCardCreated={handleCardCreated}
  />
{/if}
