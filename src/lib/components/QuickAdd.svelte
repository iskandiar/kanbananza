<script lang="ts">
  import type { Card, CardType } from '$lib/types';
  import { createCard, createCardFromUrl } from '$lib/api/cards';
  import EditCardModal from '$lib/components/EditCardModal.svelte';

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
  let pendingEditCard = $state<Card | null>(null);

  const IS_URL = /^https?:\/\//;
  const LINEAR_URL = /https:\/\/linear\.app\/[^/]+\/issue\/([A-Z]+-\d+)/;
  const NOTION_URL = /https?:\/\/(www\.)?notion\.(so|com)\/.*([a-f0-9]{32})/;
  const SLACK_URL = /https?:\/\/[^.]+\.slack\.com\/archives\/[A-Z0-9]+\/p\d+/;

  const detectedSource = $derived(
    LINEAR_URL.test(value) ? 'Linear issue'
    : NOTION_URL.test(value) ? 'Notion page'
    : SLACK_URL.test(value) ? 'Slack thread'
    : null
  );

  function inferTypeFromUrl(url: string): CardType {
    try {
      const host = new URL(url).hostname;
      if (host.endsWith('.slack.com')) return 'thread';
      if (host === 'linear.app') return 'task';
      if (host === 'notion.so' || host === 'notion.com') return 'documentation';
      if (host === 'github.com' || host === 'gitlab.com' || host.includes('.gitlab.')) return 'mr';
    } catch {}
    return 'task';
  }

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
      } catch {
        // Known URL format but sync failed — fall back to generic card + edit modal
        try {
          const card = await createCard(trimmed, inferTypeFromUrl(trimmed), weekId, dayOfWeek, undefined, trimmed);
          value = '';
          active = false;
          onCardCreated?.(card);
          pendingEditCard = card;
        } catch (e2) {
          error = String(e2);
        }
      } finally {
        isLoading = false;
      }
    } else if (IS_URL.test(trimmed)) {
      isLoading = true;
      error = null;
      try {
        const card = await createCard(trimmed, inferTypeFromUrl(trimmed), weekId, dayOfWeek, undefined, trimmed);
        value = '';
        active = false;
        onCardCreated?.(card);
        pendingEditCard = card;
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

{#if pendingEditCard}
  <EditCardModal card={pendingEditCard} onClose={() => (pendingEditCard = null)} />
{/if}

<script module lang="ts">
  function focus(node: HTMLElement) { node.focus(); }
</script>
