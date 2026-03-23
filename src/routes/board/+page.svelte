<script lang="ts">
  import { onMount } from 'svelte';
  import WeekBoard from '$lib/components/WeekBoard.svelte';
  import SearchModal from '$lib/components/SearchModal.svelte';
  import { boardStore } from '$lib/stores/board.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Card } from '$lib/types';

  onMount(async () => {
    await Promise.all([boardStore.loadCurrentWeek(), settingsStore.load()]);
  });

  const weekLabel = $derived(() => {
    const w = boardStore.currentWeek;
    if (!w) return '…';
    const monday = new Date(w.start_date);
    const friday = new Date(monday);
    friday.setDate(monday.getDate() + 4);
    const fmt = (d: Date) => d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
    return `${fmt(monday)} – ${fmt(friday)} · W${w.week_number}`;
  });

  let sidebarOpen = $state(false);
  let searchOpen = $state(false);

  // Today's day-of-week (Mon=1 … Fri=5), clamped to 1–5
  let todayDayOfWeek = $state<number>(1);
  $effect(() => {
    const d = new Date().getDay(); // 0=Sun, 1=Mon … 6=Sat
    todayDayOfWeek = d === 0 ? 5 : d === 6 ? 5 : d; // clamp Sat/Sun → Fri
  });

  const todayPendingCount = $derived(
    (boardStore.tasksByDay.get(todayDayOfWeek) ?? []).filter(t => t.status !== 'done').length
  );

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') { e.preventDefault(); searchOpen = true; return; }

    const target = e.target as HTMLElement;
    if (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable
    ) return;

    if (e.key === '[') boardStore.navigateWeek(-1);
    else if (e.key === ']') boardStore.navigateWeek(1);
    else if (e.key === 'b' || e.key === 'B') sidebarOpen = !sidebarOpen;
    else if (e.key === 'h' || e.key === 'H') {
      boardStore.viewMode = boardStore.viewMode === 'board' ? 'history' : 'board';
    }
  }

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

<svelte:window onkeydown={handleKeydown} />

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
    {sidebarOpen}
    viewMode={boardStore.viewMode}
    isPastWeek={boardStore.isPastWeek}
    onToggleMode={() => {
      boardStore.viewMode = boardStore.viewMode === 'board' ? 'history' : 'board';
    }}
    currentWeek={boardStore.currentWeek}
    weekCards={boardStore.cards.filter(c => c.week_id === boardStore.currentWeek?.id)}
    onPrevWeek={() => boardStore.navigateWeek(-1)}
    onNextWeek={() => boardStore.navigateWeek(1)}
    onJumpToToday={() => boardStore.loadCurrentWeek()}
    onAddCard={handleAddCard}
    onMoveCard={handleMoveCard}
    onMarkDone={(id) => boardStore.markDone(id)}
    onRollover={() => boardStore.rollover()}
    onToggleSidebar={() => (sidebarOpen = !sidebarOpen)}
    onCardCreated={handleCardCreated}
    onMoveCardToNextWeek={boardStore.isCurrentWeek
      ? (id) => boardStore.moveToNextWeek(id)
      : undefined}
    onScheduleToday={boardStore.isCurrentWeek
      ? (id) => boardStore.moveCard(id, boardStore.currentWeek?.id ?? null, todayDayOfWeek, todayPendingCount)
      : undefined}
  />
{/if}

{#if searchOpen}
  <SearchModal onClose={() => (searchOpen = false)} />
{/if}
