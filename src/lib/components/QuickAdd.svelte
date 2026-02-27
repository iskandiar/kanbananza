<script lang="ts">
  import type { Card } from '$lib/types';
  import { createCardFromUrl } from '$lib/api/cards';

  let {
    weekId = null,
    dayOfWeek = null,
    onAdd,
    onCardCreated
  }: {
    weekId?: number | null;
    dayOfWeek?: number | null;
    onAdd: (title: string) => void;
    onCardCreated?: (card: Card) => void;
  } = $props();

  let active = $state(false);
  let value = $state('');
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  const LINEAR_URL = /https:\/\/linear\.app\/[^/]+\/issue\/([A-Z]+-\d+)/;
  const NOTION_URL = /https?:\/\/(www\.)?notion\.(so|com)\/.*([a-f0-9]{32})/;
  const SLACK_URL = /https?:\/\/[^.]+\.slack\.com\/archives\/[A-Z0-9]+\/p\d+/;

  const detectedSource = $derived(
    LINEAR_URL.test(value) ? 'Linear issue'
    : NOTION_URL.test(value) ? 'Notion page'
    : SLACK_URL.test(value) ? 'Slack thread'
    : null
  );

  async function submit() {
    const trimmed = value.trim();
    if (!trimmed) return;

    if (detectedSource) {
      isLoading = true;
      error = null;
      try {
        const card = await createCardFromUrl(trimmed, weekId, dayOfWeek);
        value = '';
        active = false;
        onCardCreated?.(card);
      } catch (e) {
        error = String(e);
      } finally {
        isLoading = false;
      }
    } else {
      onAdd(trimmed);
      value = '';
      active = false;
    }
  }

  function cancel() {
    value = '';
    error = null;
    active = false;
  }

  function handleInput() {
    if (error) error = null;
  }
</script>

{#if active}
  <div class="flex flex-col gap-1">
    <input
      class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1.5 text-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
      placeholder="Card title or URL..."
      bind:value
      disabled={isLoading}
      oninput={handleInput}
      onkeydown={(e) => { if (e.key === 'Enter') submit(); if (e.key === 'Escape') cancel(); }}
      use:focus
    />
    {#if detectedSource && !error}
      <p class="text-xs text-[var(--color-muted)] pl-0.5">↳ Detected: {detectedSource}</p>
    {/if}
    {#if error}
      <p class="text-xs text-red-400/90 pl-0.5">{error}</p>
    {/if}
  </div>
{:else}
  <button
    onclick={() => (active = true)}
    class="w-full text-left text-xs text-[var(--color-muted)] hover:text-[var(--color-text-muted)] py-1.5 px-1 transition-colors"
  >
    + Add card...
  </button>
{/if}

<script module lang="ts">
  function focus(node: HTMLElement) { node.focus(); }
</script>
