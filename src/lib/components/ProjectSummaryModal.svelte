<script lang="ts">
  import { onMount } from 'svelte';
  import type { Project } from '$lib/types';
  import { summariseProject } from '$lib/api/projects';
  import { X, Copy } from 'lucide-svelte';

  let { project, onClose }: { project: Project; onClose: () => void } = $props();

  let summary = $state<string | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let copied = $state(false);

  onMount(async () => {
    try {
      summary = await summariseProject(project.id);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isLoading = false;
    }
  });

  async function copySummary() {
    if (!summary) return;
    await navigator.clipboard.writeText(summary);
    copied = true;
    setTimeout(() => (copied = false), 2000);
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
    class="bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg shadow-xl p-4 w-[32rem] max-h-[80vh] flex flex-col"
  >
    <div class="flex items-center justify-between mb-3 flex-shrink-0">
      <div class="flex items-center gap-2">
        <span
          class="inline-flex items-center justify-center text-xs font-bold rounded px-1.5 py-0.5"
          style="background-color: {project.color}20; color: {project.color};"
          >{project.slug}</span
        >
        <span class="text-sm font-medium text-[var(--color-text)]"
          >{project.name} — Summary</span
        >
      </div>
      <button
        onclick={onClose}
        class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
        aria-label="Close"><X size={14} /></button
      >
    </div>

    <div class="flex-1 overflow-y-auto min-h-0">
      {#if isLoading}
        <div class="flex items-center justify-center h-32">
          <span class="text-sm text-[var(--color-muted)] animate-pulse">Generating summary…</span>
        </div>
      {:else if error}
        <p class="text-sm text-rose-400">{error}</p>
      {:else if summary}
        <pre
          class="text-sm text-[var(--color-text)] whitespace-pre-wrap font-sans leading-relaxed bg-[var(--color-surface)] rounded p-3">{summary}</pre>
      {/if}
    </div>

    {#if summary && !isLoading}
      <div class="flex justify-end mt-3 flex-shrink-0">
        <button
          onclick={copySummary}
          class="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
        >
          <Copy size={12} />
          {copied ? 'Copied!' : 'Copy to clipboard'}
        </button>
      </div>
    {/if}
  </div>
</div>
