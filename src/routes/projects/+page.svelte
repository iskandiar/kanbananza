<script lang="ts">
  import { onMount } from 'svelte';
  import { projectsStore } from '$lib/stores/projects.svelte';
  import ProjectAccordionRow from '$lib/components/ProjectAccordionRow.svelte';
  import CreateProjectModal from '$lib/components/CreateProjectModal.svelte';

  let showCreateModal = $state(false);
  let expandedSet = $state(new Set<number>());

  onMount(async () => {
    await projectsStore.loadProjects();
    if (projectsStore.projects.length > 0) {
      expandedSet = new Set([projectsStore.projects[0].id]);
    }
  });

  function toggleProject(id: number) {
    const next = new Set(expandedSet);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expandedSet = next;
  }
</script>

<div class="flex flex-col min-h-screen">
  <!-- Header -->
  <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-glass-border)] bg-[var(--color-glass-header)] backdrop-blur-md">
    <div class="flex items-center gap-3">
      <a
        href="/board"
        class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors text-sm"
      >← Board</a>
      <h1 class="text-sm font-medium text-[var(--color-text)]">Projects</h1>
    </div>
    <button
      onclick={() => (showCreateModal = true)}
      class="text-sm px-3 py-1.5 rounded border border-[var(--color-accent)] text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 transition-colors"
    >+ New Project</button>
  </div>

  <!-- Project list -->
  <div class="flex-1 overflow-y-auto divide-y divide-[var(--color-glass-border)]">
    {#each projectsStore.projects as project (project.id)}
      <ProjectAccordionRow
        {project}
        isExpanded={expandedSet.has(project.id)}
        onToggle={() => toggleProject(project.id)}
      />
    {/each}
    {#if projectsStore.projects.length === 0 && !projectsStore.isLoading}
      <div class="flex flex-col items-center justify-center h-48 gap-3">
        <p class="text-[var(--color-muted)] text-sm">No projects yet.</p>
        <button
          onclick={() => (showCreateModal = true)}
          class="text-sm px-3 py-1.5 rounded border border-[var(--color-accent)] text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 transition-colors"
        >Create your first project</button>
      </div>
    {/if}
  </div>
</div>

{#if showCreateModal}
  <CreateProjectModal
    onCreated={(_p) => { showCreateModal = false; projectsStore.loadProjects(); }}
    onClose={() => (showCreateModal = false)}
  />
{/if}
