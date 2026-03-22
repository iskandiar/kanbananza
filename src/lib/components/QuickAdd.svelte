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
        const { url: extractedUrl, cardType, impact } = extractFromTitle(trimmed);
        const card = await createCardFromUrl(extractedUrl ?? trimmed, weekId, dayOfWeek);
        // Apply token overrides — tags win over integration defaults
        if (cardType !== null || impact !== null) {
          const updated = await updateCard(card.id, {
            ...(cardType !== null ? { cardType } : {}),
            ...(impact !== null ? { impact } : {}),
          });
          boardStore.cards = boardStore.cards.map(c => c.id === updated.id ? updated : c);
          onCardCreated?.(updated);
        } else {
          onCardCreated?.(card);
        }
        value = '';
        active = false;
      } catch {
        // Known URL format but sync failed — fall back to generic card + edit modal
        try {
          const { cleanedTitle, timeEstimate, url: extractedUrl, impact, cardType } = extractFromTitle(trimmed);
          const finalUrl = extractedUrl || trimmed;
          const card = await createCard(cleanedTitle, cardType ?? inferTypeFromUrl(finalUrl), weekId, dayOfWeek, undefined, finalUrl);

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
        const { cleanedTitle, timeEstimate, impact, cardType } = extractFromTitle(trimmed);
        const card = await createCard(cleanedTitle, cardType ?? inferTypeFromUrl(trimmed), weekId, dayOfWeek, undefined, trimmed);

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
      // Regular text input — extract time, URL, type, priority before creating
      const { cleanedTitle, timeEstimate, url: extractedUrl, impact, cardType } = extractFromTitle(trimmed);

      if (extractedUrl) {
        // If URL was extracted, create as URL card with extracted values
        isLoading = true;
        error = null;
        try {
          const card = await createCard(cleanedTitle || trimmed, cardType ?? inferTypeFromUrl(extractedUrl), weekId, dayOfWeek, undefined, extractedUrl);

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
      } else if (timeEstimate !== null || impact !== null || cardType !== null) {
        // No URL but time, priority, or type was extracted — create card directly with extracted values
        isLoading = true;
        error = null;
        try {
          const card = await createCard(cleanedTitle, cardType ?? 'task', weekId, dayOfWeek);
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
      placeholder="Title, URL, 1h, #mr, #high…"
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
    cardType: CardType | null;
  } {
    let cleaned = title;
    let timeEstimate: number | null = null;
    let extractedUrl: string | null = null;
    let impact: Impact | null = null;
    let cardType: CardType | null = null;

    // 1. Extract URL first — any #tokens inside URL fragments are consumed here
    const urlMatch = cleaned.match(/https?:\/\/\S+/);
    if (urlMatch) {
      extractedUrl = urlMatch[0];
      cleaned = cleaned.replace(urlMatch[0], '').trim();
    }

    // 2. Extract time (H:MM format first, then unit-based)
    const hmMatch = cleaned.match(/(?<!\.)(\b\d+):([0-5]\d)\b/);
    if (hmMatch) {
      timeEstimate = parseInt(hmMatch[1]) + parseInt(hmMatch[2]) / 60;
      cleaned = cleaned.replace(hmMatch[0], '').trim();
    } else {
      const timeMatch = cleaned.match(/\b(\d+(?:\.\d+)?)\s*(h|hr|hrs?|hours?|m|min|mins?|minutes?)\b/i);
      if (timeMatch) {
        const val = parseFloat(timeMatch[1]);
        const unit = timeMatch[2].toLowerCase();
        timeEstimate = (unit === 'm' || unit.startsWith('min')) ? val / 60 : val;
        cleaned = cleaned.replace(timeMatch[0], '').trim();
      }
    }

    // 3. Scan #tokens — split on whitespace, classify each token exactly
    const TYPE_MAP: Record<string, CardType> = {
      '#task': 'task', '#todo': 'task',
      '#meeting': 'meeting', '#meet': 'meeting',
      '#mr': 'mr',
      '#thread': 'thread',
      '#review': 'review',
      '#doc': 'documentation', '#documentation': 'documentation',
    };
    const PRIORITY_MAP: Record<string, Impact> = {
      '#high': 'high', '#h': 'high',
      '#mid': 'mid', '#m': 'mid',
      '#low': 'low', '#l': 'low',
    };

    const tokens = cleaned.split(/\s+/);
    const kept: string[] = [];
    for (const token of tokens) {
      const lower = token.toLowerCase();
      if (TYPE_MAP[lower] !== undefined) {
        cardType = TYPE_MAP[lower];         // last wins
      } else if (PRIORITY_MAP[lower] !== undefined) {
        impact = PRIORITY_MAP[lower];       // last wins
      } else {
        kept.push(token);
      }
    }
    cleaned = kept.join(' ').trim();

    return { cleanedTitle: cleaned, timeEstimate, url: extractedUrl, impact, cardType };
  }

  function focus(node: HTMLElement) { node.focus(); }
</script>
