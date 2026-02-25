<script lang="ts">
  import type { Card } from '$lib/types';
  import CardComponent from './Card.svelte';
  import QuickAdd from './QuickAdd.svelte';

  let {
    cards = [],
    isOpen,
    onAddCard,
    onClose
  }: {
    cards: Card[];
    isOpen: boolean;
    onAddCard: (title: string) => void;
    onClose: () => void;
  } = $props();
</script>

{#if isOpen}
  <aside class="w-72 flex-shrink-0 border-l border-[var(--color-border)] flex flex-col bg-[var(--color-background)]">
    <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)]">
      <span class="text-sm font-medium text-[var(--color-text)]">Backlog ({cards.length})</span>
      <button
        onclick={onClose}
        class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors text-lg leading-none"
        aria-label="Close backlog"
      >×</button>
    </div>
    <div class="flex flex-col gap-2 p-3 overflow-y-auto flex-1">
      <QuickAdd onAdd={onAddCard} />
      {#each cards as card (card.id)}
        <CardComponent {card} />
      {/each}
    </div>
  </aside>
{/if}
