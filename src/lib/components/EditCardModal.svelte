<script lang="ts">
  import type { Card, CardType, Impact } from '$lib/types';
  import { boardStore } from '$lib/stores/board.svelte';
  import { projectsStore } from '$lib/stores/projects.svelte';
  import * as cardsApi from '$lib/api/cards';
  import { Users, GitPullRequest, MessageSquare, ListTodo, Eye, FileText, X } from 'lucide-svelte';
  import { portal } from '$lib/actions/portal';

  let { card, onClose }: { card: Card; onClose: () => void } = $props();

  // Maps (mirrors Card.svelte)
  const typeBadge: Record<string, string> = {
    meeting: 'bg-blue-500/15 text-blue-300',
    mr:      'bg-purple-500/15 text-purple-300',
    thread:  'bg-yellow-500/15 text-yellow-300',
    task:    'bg-emerald-500/15 text-emerald-300',
    review:  'bg-slate-500/15 text-slate-300',
    documentation: 'bg-slate-500/15 text-slate-300',
  };

  const typeIconCmp = {
    meeting:       Users,
    mr:            GitPullRequest,
    thread:        MessageSquare,
    task:          ListTodo,
    review:        Eye,
    documentation: FileText,
  } as const;

  const typeLabel: Record<string, string> = {
    meeting: 'meeting',
    mr: 'MR',
    thread: 'thread',
    task: 'task',
    review: 'review',
    documentation: 'doc',
  };

  const impactBadge: Record<string, string> = {
    low:  'text-[var(--color-impact-low)]  bg-[var(--color-impact-low-bg)]  px-1 py-px rounded border-transparent',
    mid:  'text-[var(--color-impact-mid)]  bg-[var(--color-impact-mid-bg)]  px-1 py-px rounded border-transparent',
    high: 'text-[var(--color-impact-high)] bg-[var(--color-impact-high-bg)] px-1 py-px rounded border-transparent',
  };

  // Local state — synced from card prop via $effect
  let popoverCardType = $state<CardType>('task');
  let popoverImpact = $state('');
  let popoverHours = $state('');
  let popoverUrl = $state('');
  let popoverNotes = $state('');
  let popoverProjectId = $state<number | null>(null);
  let saveError = $state<string | null>(null);

  // Initialize and re-sync whenever the card prop changes
  $effect(() => {
    popoverCardType = card.card_type;
    popoverImpact = card.impact ?? '';
    popoverHours = card.time_estimate != null ? String(card.time_estimate) : '';
    popoverUrl = card.url ?? '';
    popoverNotes = card.notes ?? '';
    popoverProjectId = card.project_id ?? null;
  });

  // Load projects if not already loaded
  $effect(() => {
    if (projectsStore.projects.length === 0) {
      projectsStore.loadProjects();
    }
  });

  async function savePopoverField(fields: Parameters<typeof cardsApi.updateCard>[1]) {
    try {
      const updatedCard = await cardsApi.updateCard(card.id, fields);
      // Sync updated card into both stores
      boardStore.cards = boardStore.cards.map(c => c.id === updatedCard.id ? updatedCard : c);
      projectsStore.syncCard(updatedCard);
      saveError = null;
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e);
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  use:portal
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleOverlayClick}
>
  <div class="bg-[var(--color-glass-bg-raised)] border border-[var(--color-glass-border)] rounded-lg shadow-2xl backdrop-blur-xl p-4 w-72">
    <!-- Header -->
    <div class="flex items-center justify-between mb-3">
      <span class="text-sm font-medium text-[var(--color-text)]">Edit card</span>
      <button
        type="button"
        onclick={onClose}
        class="text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
        aria-label="Close"
      ><X size={14} /></button>
    </div>

    <div class="flex flex-col gap-2">
      <!-- Type selector -->
      <div>
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="text-xs text-[var(--color-muted)] block mb-1">Type</label>
        <div class="flex gap-1 flex-wrap">
          {#each ['task', 'meeting', 'mr', 'thread', 'review', 'documentation'] as type (type)}
            {@const TypeIcon = typeIconCmp[type as CardType]}
            <button
              type="button"
              onclick={() => {
                popoverCardType = type as CardType;
                savePopoverField({ cardType: type as CardType });
              }}
              class="flex items-center gap-1 text-xs px-2 py-1 rounded border transition-colors {
                popoverCardType === type
                  ? (typeBadge[type] ?? '') + ' border-transparent'
                  : 'border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)]'
              }"
            >
              <TypeIcon size={10} />
              {typeLabel[type as CardType]}
            </button>
          {/each}
        </div>
      </div>

      <!-- Impact/Priority selector -->
      <div>
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="text-xs text-[var(--color-muted)] block mb-1">Priority</label>
        <div class="flex gap-1">
          {#each ['low', 'mid', 'high'] as level (level)}
            <button
              type="button"
              onclick={() => {
                const newImpact = popoverImpact === level ? '' : level;
                popoverImpact = newImpact;
                if (newImpact === '') {
                  savePopoverField({});
                } else {
                  savePopoverField({ impact: newImpact as Impact });
                }
              }}
              class="flex-1 text-xs py-0.5 rounded border transition-colors {
                popoverImpact === level
                  ? (impactBadge[level] ?? '')
                  : 'border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)]'
              }"
            >{level}</button>
          {/each}
        </div>
      </div>

      <!-- Hours input -->
      <div>
        <label for="modal-hours" class="text-xs text-[var(--color-muted)] block mb-1">Hours</label>
        <input
          id="modal-hours"
          type="number"
          bind:value={popoverHours}
          step="any"
          min="0"
          placeholder="0"
          class="w-full text-xs bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
          onblur={() => {
            if (popoverHours !== '' && popoverHours !== '0') {
              savePopoverField({ timeEstimate: Number(popoverHours) });
            } else {
              savePopoverField({});
            }
          }}
        />
      </div>

      <!-- URL input -->
      <div>
        <label for="modal-url" class="text-xs text-[var(--color-muted)] block mb-1">URL</label>
        <input
          id="modal-url"
          type="text"
          bind:value={popoverUrl}
          placeholder="https://…"
          class="w-full text-xs bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
          onblur={() => savePopoverField(popoverUrl ? { url: popoverUrl } : {})}
        />
      </div>

      <!-- Notes textarea -->
      <div>
        <label for="modal-notes" class="text-xs text-[var(--color-muted)] block mb-1">Notes</label>
        <textarea
          id="modal-notes"
          bind:value={popoverNotes}
          placeholder="Add notes…"
          rows="2"
          class="w-full text-xs bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)] resize-none"
          onblur={() => savePopoverField(popoverNotes ? { notes: popoverNotes } : {})}
        ></textarea>
      </div>

      <!-- Project selector -->
      {#if projectsStore.projects.length > 0}
        <div>
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label class="text-xs text-[var(--color-muted)] block mb-1">Project</label>
          <select
            value={popoverProjectId ?? ''}
            onchange={(e) => {
              const val = (e.currentTarget as HTMLSelectElement).value;
              const newId = val === '' ? null : Number(val);
              popoverProjectId = newId;
              if (newId === null) {
                savePopoverField({ clearProjectId: true });
              } else {
                savePopoverField({ projectId: newId });
              }
            }}
            class="w-full text-xs bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
          >
            <option value="">None</option>
            {#each projectsStore.projects as project (project.id)}
              <option value={project.id}>[{project.slug}] {project.name}</option>
            {/each}
          </select>
        </div>
      {/if}

      {#if saveError}
        <p class="text-xs text-rose-400">{saveError}</p>
      {/if}
    </div>
  </div>
</div>
