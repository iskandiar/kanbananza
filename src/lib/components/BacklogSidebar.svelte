<script lang="ts">
  import type { Card } from '$lib/types';
  import { dndzone } from 'svelte-dnd-action';
  import CardComponent from './Card.svelte';
  import QuickAdd from './QuickAdd.svelte';

  let {
    cards = [],
    isOpen,
    onAddCard,
    onClose,
    onMoveCard,
    onMarkDone
  }: {
    cards: Card[];
    isOpen: boolean;
    onAddCard: (title: string) => void;
    onClose: () => void;
    onMoveCard: (cardId: number, weekId: number | null, dayOfWeek: number | null, position: number) => void;
    onMarkDone: (cardId: number) => void;
  } = $props();

  let localCards = $state<Card[]>([]);
  $effect(() => { localCards = cards; });

  function handleDndConsider(e: CustomEvent<{ items: Card[] }>) {
    localCards = e.detail.items;
  }

  function handleDndFinalize(e: CustomEvent<{ items: Card[] }>) {
    localCards = e.detail.items;
    localCards.forEach((card, i) => {
      if (card.week_id !== null || card.position !== i) {
        onMoveCard(card.id, null, null, i);
      }
    });
  }
</script>

<aside
  class="flex-shrink-0 flex flex-col bg-[var(--color-background)] overflow-hidden transition-[width] duration-200"
  class:border-l={isOpen}
  class:border-[var(--color-border)]={isOpen}
  style:width={isOpen ? '18rem' : '0'}
>
  <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)]">
    <span class="text-sm font-medium text-[var(--color-text)]">Backlog ({cards.length})</span>
    <button
      onclick={onClose}
      class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors text-lg leading-none"
      aria-label="Close backlog"
    >×</button>
  </div>

  {#if isOpen}
    <div class="px-3 py-2 border-b border-[var(--color-border)]">
      <QuickAdd onAdd={onAddCard} />
    </div>
  {/if}

  <div
    class="flex flex-col gap-2 p-3 overflow-y-auto flex-1 min-h-[2rem]"
    use:dndzone={{
      items: localCards,
      flipDurationMs: 150,
      dropTargetStyle: { outline: 'none', background: 'rgba(99, 102, 241, 0.07)', 'border-radius': '6px' }
    }}
    onconsider={handleDndConsider}
    onfinalize={handleDndFinalize}
  >
    {#each localCards as card (card.id)}
      <CardComponent {card} {onMarkDone} />
    {/each}
  </div>
</aside>
