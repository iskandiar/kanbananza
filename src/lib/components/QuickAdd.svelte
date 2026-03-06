<script lang="ts">
  import type { Card, CardType, Impact } from '$lib/types';
  import { createCard, createCardFromUrl, updateCard } from '$lib/api/cards';
  import { boardStore } from '$lib/stores/board.svelte';
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
          const { cleanedTitle, timeEstimate, url: extractedUrl, impact } = extractFromTitle(trimmed);
          const finalUrl = extractedUrl || trimmed;
          const card = await createCard(cleanedTitle, inferTypeFromUrl(finalUrl), weekId, dayOfWeek, undefined, finalUrl);

          // Update with extracted time and priority if present
          if (timeEstimate !== null || impact !== null) {
            const updatedCard = await updateCard(card.id, {
              timeEstimate: timeEstimate ?? undefined,
              impact: impact ?? undefined
            });
            value = '';
            active = false;
            onCardCreated?.(updatedCard);
            boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
          } else {
            value = '';
            active = false;
            onCardCreated?.(card);
          }
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
        const { cleanedTitle, timeEstimate, impact } = extractFromTitle(trimmed);
        const card = await createCard(cleanedTitle, inferTypeFromUrl(trimmed), weekId, dayOfWeek, undefined, trimmed);

        // Update with extracted time and priority if present
        if (timeEstimate !== null || impact !== null) {
          const updatedCard = await updateCard(card.id, {
            timeEstimate: timeEstimate ?? undefined,
            impact: impact ?? undefined
          });
          value = '';
          active = false;
          onCardCreated?.(updatedCard);
          boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
        } else {
          value = '';
          active = false;
          onCardCreated?.(card);
        }
        pendingEditCard = card;
      } catch (e) {
        error = String(e);
      } finally {
        isLoading = false;
      }
    } else {
      // Regular text input — extract time, URL, priority before creating
      const { cleanedTitle, timeEstimate, url: extractedUrl, impact } = extractFromTitle(trimmed);

      if (extractedUrl) {
        // If URL was extracted, create as URL card with extracted values
        isLoading = true;
        error = null;
        try {
          const card = await createCard(cleanedTitle || trimmed, inferTypeFromUrl(extractedUrl), weekId, dayOfWeek, undefined, extractedUrl);

          // Update with extracted time and priority if present
          if (timeEstimate !== null || impact !== null) {
            const updatedCard = await updateCard(card.id, {
              timeEstimate: timeEstimate ?? undefined,
              impact: impact ?? undefined
            });
            value = '';
            active = false;
            onCardCreated?.(updatedCard);
            boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
          } else {
            value = '';
            active = false;
            onCardCreated?.(card);
          }
          pendingEditCard = card;
        } catch (e) {
          error = String(e);
        } finally {
          isLoading = false;
        }
      } else if (timeEstimate !== null || impact !== null) {
        // No URL but time or priority was extracted — create card directly with extracted values
        isLoading = true;
        error = null;
        try {
          const card = await createCard(cleanedTitle, 'task', weekId, dayOfWeek);

          // Update with extracted time and priority
          const updatedCard = await updateCard(card.id, {
            timeEstimate: timeEstimate ?? undefined,
            impact: impact ?? undefined
          });
          value = '';
          active = false;
          onCardCreated?.(updatedCard);
          boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
        } catch (e) {
          error = String(e);
        } finally {
          isLoading = false;
        }
      } else {
        // No extraction needed — use regular add flow with cleaned title
        onAdd(cleanedTitle);
        value = '';
        active = false;
      }
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
      placeholder="Title, URL, 1h or 0:30…"
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
  export function extractFromTitle(title: string): {
    cleanedTitle: string;
    timeEstimate: number | null;
    url: string | null;
    impact: Impact | null;
  } {
    let cleaned = title;
    let timeEstimate: number | null = null;
    let extractedUrl: string | null = null;
    let impact: Impact | null = null;

    // Extract URL (http:// or https://)
    const urlMatch = cleaned.match(/https?:\/\/\S+/);
    if (urlMatch) {
      extractedUrl = urlMatch[0];
      cleaned = cleaned.replace(urlMatch[0], '').trim();
    }

    // Try H:MM format first (e.g. 0:30, 1:30, 2:00) — require valid minutes 00-59
    const hmMatch = cleaned.match(/(?<!\.)(\b\d+):([0-5]\d)\b/);
    if (hmMatch) {
      timeEstimate = parseInt(hmMatch[1]) + parseInt(hmMatch[2]) / 60;
      cleaned = cleaned.replace(hmMatch[0], '').trim();
    } else {
      // Fall back to unit-based formats (1h, 30m, 1.5 hours, etc.)
      const timeMatch = cleaned.match(/\b(\d+(?:\.\d+)?)\s*(h|hr|hrs?|hours?|m|min|mins?|minutes?)\b/i);
      if (timeMatch) {
        const val = parseFloat(timeMatch[1]);
        const unit = timeMatch[2].toLowerCase();
        if (unit === 'm' || unit === 'min' || unit === 'mins' || unit === 'minute' || unit === 'minutes') {
          timeEstimate = val / 60;
        } else {
          timeEstimate = val;
        }
        cleaned = cleaned.replace(timeMatch[0], '').trim();
      }
    }

    // Extract priority (!high, !mid, !low, !h, !m, !l)
    const priorityMatch = cleaned.match(/!(high|mid|low|h|m|l)\b/i);
    if (priorityMatch) {
      const p = priorityMatch[1].toLowerCase();
      if (p === 'h') impact = 'high';
      else if (p === 'm') impact = 'mid';
      else if (p === 'l') impact = 'low';
      else impact = (p as Impact);
      cleaned = cleaned.replace(priorityMatch[0], '').trim();
    }

    return { cleanedTitle: cleaned, timeEstimate, url: extractedUrl, impact };
  }

  function focus(node: HTMLElement) { node.focus(); }
</script>
