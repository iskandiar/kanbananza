<script lang="ts">
  import { dndzone } from 'svelte-dnd-action';
  import type { Project, Card } from '$lib/types';
  import { projectsStore } from '$lib/stores/projects.svelte';
  import { boardStore } from '$lib/stores/board.svelte';
  import ProjectCard from './ProjectCard.svelte';
  import ProjectSummaryModal from './ProjectSummaryModal.svelte';

  let {
    project,
    isExpanded,
    onToggle
  }: {
    project: Project;
    isExpanded: boolean;
    onToggle: () => void;
  } = $props();

  let showSummary = $state(false);
  let confirmingArchive = $state(false);

  let localBacklog = $state<Card[]>([]);
  let localInProgress = $state<Card[]>([]);
  let localDone = $state<Card[]>([]);
  let addingCard = $state(false);
  let newCardTitle = $state('');

  $effect(() => {
    if (isExpanded) {
      projectsStore.loadProjectCards(project.id);
    }
  });

  $effect(() => {
    localBacklog = projectsStore.backlogFor(project.id);
  });
  $effect(() => {
    localInProgress = projectsStore.inProgressFor(project.id);
  });
  $effect(() => {
    localDone = projectsStore.doneFor(project.id);
  });

  const dndOptions = {
    flipDurationMs: 150,
    dropTargetStyle: {
      outline: 'none',
      background: 'rgba(61,126,255,0.07)',
      'border-radius': '6px'
    }
  };

  function handleBacklogConsider(e: CustomEvent<{ items: Card[] }>) {
    localBacklog = e.detail.items;
  }

  function handleBacklogFinalize(e: CustomEvent<{ items: Card[] }>) {
    localBacklog = e.detail.items;
  }

  function handleInProgressConsider(e: CustomEvent<{ items: Card[] }>) {
    localInProgress = e.detail.items;
  }

  // Returns today's ISO day of week (1=Mon … 5=Fri); weekends clamp to 1.
  function todayDow(): number {
    const d = new Date().getDay(); // 0=Sun, 6=Sat
    const iso = d === 0 ? 7 : d;  // 1=Mon … 7=Sun
    return iso > 5 ? 1 : iso;
  }

  async function handleInProgressFinalize(e: CustomEvent<{ items: Card[] }>) {
    localInProgress = e.detail.items;
    const dow = todayDow();
    for (const card of localInProgress) {
      if (card.week_id === null && boardStore.currentWeek) {
        await projectsStore.moveToInProgress(card.id, boardStore.currentWeek.id, dow);
      }
    }
    for (const card of localInProgress) {
      if (card.status === 'done') {
        await projectsStore.moveFromDoneToBacklog(card.id);
        await projectsStore.moveToInProgress(card.id, boardStore.currentWeek?.id ?? 0, dow);
      }
    }
  }

  function handleDoneConsider(e: CustomEvent<{ items: Card[] }>) {
    localDone = e.detail.items;
  }

  async function handleDoneFinalize(e: CustomEvent<{ items: Card[] }>) {
    localDone = e.detail.items;
    for (const card of localDone) {
      if (card.status !== 'done') {
        await projectsStore.markDone(card.id);
      }
    }
  }

  async function addCard() {
    const title = newCardTitle.trim();
    if (!title) return;
    newCardTitle = '';
    addingCard = false;
    await projectsStore.addCardToProject(title, project.id);
  }

  function cancelAdd() {
    addingCard = false;
    newCardTitle = '';
  }

  async function handleArchive() {
    await projectsStore.archiveProject(project.id);
    confirmingArchive = false;
  }

  function handleRowClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.closest('[data-no-toggle]')) return;
    onToggle();
  }
</script>

<!-- Header row -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="flex items-center gap-3 px-4 py-2.5 cursor-pointer hover:bg-[var(--color-surface)]/40 transition-colors select-none"
  onclick={handleRowClick}
>
  <span class="text-[var(--color-muted)] text-xs w-3 shrink-0">{isExpanded ? '▾' : '▸'}</span>

  <span
    class="inline-flex items-center justify-center text-xs font-bold rounded px-1.5 py-0.5 shrink-0"
    style="background-color: {project.color}20; color: {project.color};"
  >{project.slug}</span>

  <span class="text-sm text-[var(--color-text)] flex-1 min-w-0 truncate">{project.name}</span>

  {#if projectsStore.cardsLoadedFor(project.id)}
    <span class="text-xs text-[var(--color-muted)] shrink-0">
      {projectsStore.backlogFor(project.id).length} backlog ·
      {projectsStore.inProgressFor(project.id).length} in progress ·
      {projectsStore.doneFor(project.id).length} done
    </span>
  {/if}

  <div class="flex items-center gap-1.5 shrink-0" data-no-toggle>
    <button
      onclick={() => (showSummary = true)}
      class="text-xs px-2 py-0.5 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-accent)] transition-colors"
    >Summarise</button>
    {#if !confirmingArchive}
      <button
        onclick={() => (confirmingArchive = true)}
        class="text-xs px-2 py-0.5 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-rose-400 hover:border-rose-400/50 transition-colors"
      >Archive</button>
    {:else}
      <span class="text-xs text-rose-400">Archive?</span>
      <button
        onclick={handleArchive}
        class="text-xs px-2 py-0.5 rounded border border-rose-500/60 text-rose-400 hover:bg-rose-500/10 transition-colors"
      >Confirm</button>
      <button
        onclick={() => (confirmingArchive = false)}
        class="text-xs px-2 py-0.5 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
      >Cancel</button>
    {/if}
  </div>
</div>

<!-- Expanded kanban -->
{#if isExpanded}
  <div class="border-t border-[var(--color-border)]/50">
    {#if !projectsStore.cardsLoadedFor(project.id)}
      <div class="flex items-center justify-center h-24">
        <span class="text-xs text-[var(--color-muted)] animate-pulse">Loading…</span>
      </div>
    {:else}
      <div class="flex overflow-hidden" style="min-height: 10rem;">
        <!-- Backlog column -->
        <div class="flex flex-col flex-1 border-r border-[var(--color-border)] px-4 py-3 gap-2">
          <p class="text-xs font-semibold text-[var(--color-muted)] uppercase tracking-wide">
            Backlog ({localBacklog.length})
          </p>
          <div
            class="flex flex-col gap-2 flex-1 min-h-[3rem]"
            use:dndzone={{ ...dndOptions, items: localBacklog }}
            onconsider={handleBacklogConsider}
            onfinalize={handleBacklogFinalize}
          >
            {#each localBacklog as card (card.id)}
              <ProjectCard
                {card}
                onMarkDone={(id) => projectsStore.markDone(id)}
                onMoveToBacklog={(id) => projectsStore.moveToBacklog(id)}
                onDelete={(id) => projectsStore.deleteCard(id)}
              />
            {/each}
          </div>
          {#if addingCard}
            <!-- svelte-ignore a11y_autofocus -->
            <input
              type="text"
              bind:value={newCardTitle}
              placeholder="Card title…"
              autofocus
              class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1.5 text-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
              onkeydown={(e) => {
                if (e.key === 'Enter') addCard();
                if (e.key === 'Escape') cancelAdd();
              }}
              onblur={() => { if (newCardTitle.trim()) addCard(); else cancelAdd(); }}
            />
          {:else}
            <button
              onclick={() => (addingCard = true)}
              class="w-full text-left text-xs text-[var(--color-muted)] hover:text-[var(--color-text)] py-1 px-1 transition-colors"
            >+ Add card…</button>
          {/if}
        </div>

        <!-- In Progress column -->
        <div class="flex flex-col flex-1 border-r border-[var(--color-border)] px-4 py-3 gap-2">
          <p class="text-xs font-semibold text-[var(--color-muted)] uppercase tracking-wide">
            In Progress ({localInProgress.length})
          </p>
          <div
            class="flex flex-col gap-2 flex-1 min-h-[3rem]"
            use:dndzone={{ ...dndOptions, items: localInProgress }}
            onconsider={handleInProgressConsider}
            onfinalize={handleInProgressFinalize}
          >
            {#each localInProgress as card (card.id)}
              <ProjectCard
                {card}
                onMarkDone={(id) => projectsStore.markDone(id)}
                onMoveToBacklog={(id) => projectsStore.moveToBacklog(id)}
                onDelete={(id) => projectsStore.deleteCard(id)}
              />
            {/each}
          </div>
        </div>

        <!-- Done column -->
        <div class="flex flex-col flex-1 px-4 py-3 gap-2">
          <p class="text-xs font-semibold text-[var(--color-muted)] uppercase tracking-wide">
            Done ({localDone.length})
          </p>
          <div
            class="flex flex-col gap-2 flex-1 min-h-[3rem]"
            use:dndzone={{ ...dndOptions, items: localDone }}
            onconsider={handleDoneConsider}
            onfinalize={handleDoneFinalize}
          >
            {#each localDone as card (card.id)}
              <ProjectCard
                {card}
                onMarkDone={(id) => projectsStore.markDone(id)}
                onMoveToBacklog={(id) => projectsStore.moveFromDoneToBacklog(id)}
                onDelete={(id) => projectsStore.deleteCard(id)}
              />
            {/each}
          </div>
        </div>
      </div>
    {/if}
  </div>
{/if}

{#if showSummary}
  <ProjectSummaryModal {project} onClose={() => (showSummary = false)} />
{/if}
