<script lang="ts">
  import type { Project } from '$lib/types';
  import * as projectsApi from '$lib/api/projects';
  import { X } from 'lucide-svelte';

  let {
    onCreated,
    onClose
  }: {
    onCreated: (project: Project) => void;
    onClose: () => void;
  } = $props();

  const PRESET_COLORS = [
    { label: 'Indigo', value: '#6366f1' },
    { label: 'Blue', value: '#3b82f6' },
    { label: 'Emerald', value: '#10b981' },
    { label: 'Amber', value: '#f59e0b' },
    { label: 'Rose', value: '#f43f5e' },
    { label: 'Purple', value: '#a855f7' },
    { label: 'Teal', value: '#14b8a6' },
    { label: 'Orange', value: '#f97316' }
  ];

  let name = $state('');
  let slug = $state('');
  let color = $state('#6366f1');
  let isGeneratingSlug = $state(false);
  let isCreating = $state(false);
  let error = $state<string | null>(null);
  let slugDebounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  let slugManuallyEdited = $state(false);

  function onNameInput() {
    if (slugManuallyEdited) return;
    if (slugDebounceTimer) clearTimeout(slugDebounceTimer);
    if (!name.trim()) {
      slug = '';
      return;
    }
    slugDebounceTimer = setTimeout(async () => {
      isGeneratingSlug = true;
      try {
        slug = await projectsApi.generateProjectSlug(name);
      } catch {
        slug = name.slice(0, 3).toUpperCase();
      } finally {
        isGeneratingSlug = false;
      }
    }, 600);
  }

  async function handleCreate() {
    if (!name.trim()) {
      error = 'Name is required';
      return;
    }
    if (!slug.trim()) {
      error = 'Slug is required';
      return;
    }
    isCreating = true;
    error = null;
    try {
      const project = await projectsApi.createProject(
        name.trim(),
        slug.trim().toUpperCase().slice(0, 4),
        color
      );
      onCreated(project);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isCreating = false;
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleOverlayClick}
>
  <div
    class="bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg shadow-xl p-4 w-80"
  >
    <div class="flex items-center justify-between mb-4">
      <span class="text-sm font-medium text-[var(--color-text)]">New Project</span>
      <button
        onclick={onClose}
        class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
        aria-label="Close"><X size={14} /></button
      >
    </div>

    <div class="flex flex-col gap-3">
      <div>
        <label for="project-name" class="text-xs text-[var(--color-muted)] block mb-1">Name</label>
        <input
          id="project-name"
          type="text"
          bind:value={name}
          oninput={onNameInput}
          placeholder="My Project"
          class="w-full text-sm bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1.5 text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
        />
      </div>

      <div>
        <label for="project-slug" class="text-xs text-[var(--color-muted)] block mb-1">
          Slug
          {#if isGeneratingSlug}<span class="text-[var(--color-muted)]/60">(generating…)</span
            >{/if}
        </label>
        <input
          id="project-slug"
          type="text"
          bind:value={slug}
          oninput={() => (slugManuallyEdited = true)}
          maxlength={4}
          placeholder="ABC"
          class="w-full text-sm bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1.5 text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)] uppercase font-mono tracking-widest"
          style="text-transform: uppercase"
        />
      </div>

      <div>
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="text-xs text-[var(--color-muted)] block mb-1">Color</label>
        <div class="flex gap-1.5 flex-wrap">
          {#each PRESET_COLORS as preset (preset.value)}
            <button
              onclick={() => (color = preset.value)}
              class="w-6 h-6 rounded-full border-2 transition-all {color === preset.value
                ? 'border-white scale-110'
                : 'border-transparent'}"
              style="background-color: {preset.value};"
              title={preset.label}
              aria-label={preset.label}
            ></button>
          {/each}
        </div>
        <!-- Preview -->
        {#if name}
          <div class="mt-2 flex items-center gap-2">
            <span
              class="inline-flex items-center justify-center text-xs font-bold rounded px-1.5 py-0.5"
              style="background-color: {color}20; color: {color};">{slug || '…'}</span
            >
            <span class="text-sm text-[var(--color-text)]">{name}</span>
          </div>
        {/if}
      </div>

      {#if error}
        <p class="text-xs text-rose-400">{error}</p>
      {/if}

      <div class="flex gap-2 justify-end mt-1">
        <button
          onclick={onClose}
          class="text-xs px-3 py-1.5 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
          >Cancel</button
        >
        <button
          onclick={handleCreate}
          disabled={isCreating || !name.trim()}
          class="text-xs px-3 py-1.5 rounded border border-[var(--color-accent)] text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >{isCreating ? 'Creating…' : 'Create'}</button
        >
      </div>
    </div>
  </div>
</div>
