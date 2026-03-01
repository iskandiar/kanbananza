<script module lang="ts">
  import type { Card } from '$lib/types';

  export function filterCards(cards: Card[], query: string): Card[] {
    const q = query.trim().toLowerCase();
    if (!q) return cards;
    return cards.filter(c => c.title.toLowerCase().includes(q));
  }
</script>

<script lang="ts">
  import { dndzone } from 'svelte-dnd-action';
  import { projectsStore } from '$lib/stores/projects.svelte';
  import CardComponent from './Card.svelte';
  import QuickAdd from './QuickAdd.svelte';

  let {
    cards = [],
    isOpen,
    onAddCard,
    onClose,
    onMoveCard,
    onMarkDone,
    onCardCreated
  }: {
    cards: Card[];
    isOpen: boolean;
    onAddCard: (title: string) => void;
    onClose: () => void;
    onMoveCard: (cardId: number, weekId: number | null, dayOfWeek: number | null, position: number) => void;
    onMarkDone: (cardId: number) => void;
    onCardCreated?: (card: Card) => void;
  } = $props();

  // Local mutable copy for svelte-dnd-action; initialized empty and synced via $effect
  let localCards = $state<Card[]>([]);
  $effect(() => { localCards = cards; });

  let searchQuery = $state('');

  type ProjectFilter = 'all' | 'unassigned' | number;
  let projectFilter = $state<ProjectFilter>('all');

  const filteredByText = $derived(filterCards(localCards, searchQuery));
  const filteredCards = $derived(filteredByText.filter(card => {
    if (projectFilter === 'all') return true;
    if (projectFilter === 'unassigned') return card.project_id === null;
    return card.project_id === projectFilter;
  }));

  $effect(() => {
    if (isOpen && projectsStore.projects.length === 0) {
      projectsStore.loadProjects();
    }
  });

  function handleDndConsider(e: CustomEvent<{ items: Card[] }>) {
    if (searchQuery.trim()) return; // No-op during filtered view
    localCards = e.detail.items;
  }

  function handleDndFinalize(e: CustomEvent<{ items: Card[] }>) {
    if (searchQuery.trim()) return; // No-op during filtered view
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
      <input type="text" bind:value={searchQuery} placeholder="Search backlog…"
        class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1.5 text-xs text-[var(--color-text)] placeholder-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]" />
    </div>
  {/if}

  {#if isOpen && projectsStore.projects.length > 0}
    <div class="px-3 py-1.5 border-b border-[var(--color-border)]">
      <select
        bind:value={projectFilter}
        class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-xs text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
      >
        <option value="all">All projects</option>
        <option value="unassigned">Unassigned</option>
        {#each projectsStore.projects as project (project.id)}
          <option value={project.id}>[{project.slug}] {project.name}</option>
        {/each}
      </select>
    </div>
  {/if}

  <div
    class="flex flex-col gap-2 p-3 overflow-y-auto flex-1 min-h-[2rem]"
    use:dndzone={{
      items: filteredCards,
      flipDurationMs: 150,
      dropTargetStyle: { outline: 'none', background: 'rgba(61,126,255,0.07)', 'border-radius': '6px' }
    }}
    onconsider={handleDndConsider}
    onfinalize={handleDndFinalize}
  >
    {#each filteredCards as card (card.id)}
      <CardComponent {card} {onMarkDone} />
    {/each}
  </div>

  {#if isOpen}
    <div class="px-3 py-2 border-t border-[var(--color-border)]">
      <QuickAdd onAdd={onAddCard} weekId={null} dayOfWeek={null} {onCardCreated} />
    </div>
  {/if}
</aside>
