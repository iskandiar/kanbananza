<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { getSecret, storeSecret } from '$lib/api/settings';
  import IntegrationCard from '$lib/components/IntegrationCard.svelte';
  import type { AiProvider } from '$lib/types';

  // --- Workload state ---
  let availableHours = $state(8);
  let hoursSaved = $state(false);
  let hoursSaveTimer: ReturnType<typeof setTimeout> | null = null;

  async function onHoursBlur() {
    await settingsStore.updateAvailableHours(availableHours);
    hoursSaved = true;
    if (hoursSaveTimer) clearTimeout(hoursSaveTimer);
    hoursSaveTimer = setTimeout(() => { hoursSaved = false; }, 1500);
  }

  // --- AI provider state ---
  let selectedProvider = $state<AiProvider>('anthropic');
  let apiKeyInput = $state('');
  let keyIsSaved = $state(false);
  let keySaved = $state(false);
  let keySaveTimer: ReturnType<typeof setTimeout> | null = null;

  async function loadKeyStatus(provider: AiProvider) {
    const existing = await getSecret('kanbananza', `${provider}_api_key`);
    keyIsSaved = existing !== null;
    apiKeyInput = '';
  }

  async function selectProvider(provider: AiProvider) {
    selectedProvider = provider;
    await settingsStore.updateAiProvider(provider);
    await loadKeyStatus(provider);
  }

  async function saveApiKey() {
    if (!apiKeyInput.trim()) return;
    await storeSecret('kanbananza', `${selectedProvider}_api_key`, apiKeyInput.trim());
    keyIsSaved = true;
    apiKeyInput = '';
    keySaved = true;
    if (keySaveTimer) clearTimeout(keySaveTimer);
    keySaveTimer = setTimeout(() => { keySaved = false; }, 1500);
  }

  // --- Calendar integration state ---
  let calendarConnected = $state(false);
  let calendarSyncing = $state(false);
  let calendarError = $state<string | null>(null);
  let calendarSyncMessage = $state<string | null>(null);

  async function syncCalendar() {
    calendarError = null;
    calendarSyncMessage = null;
    calendarSyncing = true;
    try {
      await invoke('sync_calendar');
    } catch (e) {
      calendarError = String(e);
    } finally {
      calendarSyncing = false;
    }
  }

  async function connectCalendar() {
    calendarError = null;
    calendarSyncMessage = null;
    await invoke('get_calendar_auth_url');
  }

  async function disconnectCalendar() {
    await invoke('disconnect_calendar');
    calendarConnected = false;
    calendarError = null;
    calendarSyncMessage = null;
  }

  // --- GitLab integration state ---
  let gitlabConnected = $state(false);
  let gitlabEditingPat = $state(false);
  let gitlabPatInput = $state('');
  let gitlabPatSaved = $state(false);
  let gitlabSyncing = $state(false);
  let gitlabError = $state<string | null>(null);
  let gitlabSyncMessage = $state<string | null>(null);
  let unlistenGitlabSynced: (() => void) | null = null;

  async function saveGitlabPat() {
    if (!gitlabPatInput.trim()) return;
    try {
      await storeSecret('kanbananza', 'gitlab_pat', gitlabPatInput.trim());
      gitlabConnected = true;
      gitlabEditingPat = false;
      gitlabPatInput = '';
      gitlabPatSaved = true;
      setTimeout(() => { gitlabPatSaved = false; }, 1500);
    } catch (e) {
      gitlabError = `Failed to save PAT: ${String(e)}`;
    }
  }

  async function syncGitlab() {
    gitlabError = null;
    gitlabSyncMessage = null;
    gitlabSyncing = true;
    try {
      await invoke('sync_gitlab');
    } catch (e) {
      gitlabError = String(e);
    } finally {
      gitlabSyncing = false;
    }
  }

  async function disconnectGitlab() {
    await invoke('disconnect_gitlab');
    gitlabConnected = false;
    gitlabError = null;
    gitlabSyncMessage = null;
  }

  // --- Integrations list ---
  const integrations = [
    { id: 'linear',   name: 'Linear',           description: 'Import assigned issues',   status: 'coming_soon' as const },
    { id: 'slack',    name: 'Slack',            description: 'Import threads',           status: 'coming_soon' as const },
    { id: 'notion',   name: 'Notion',           description: 'Import action items',      status: 'coming_soon' as const },
  ];

  // --- Bootstrap ---
  let unlistenConnected: (() => void) | null = null;
  let unlistenSynced: (() => void) | null = null;
  let unlistenError: (() => void) | null = null;

  onMount(async () => {
    await settingsStore.load();
    availableHours = settingsStore.availableHours;
    selectedProvider = settingsStore.settings?.ai_provider ?? 'anthropic';
    await loadKeyStatus(selectedProvider);

    calendarConnected = await invoke<boolean>('get_calendar_status');
    gitlabConnected = (await getSecret('kanbananza', 'gitlab_pat')) !== null;

    unlistenGitlabSynced = await listen<{ count: number; error: string | null }>('gitlab://synced', (event) => {
      gitlabSyncing = false;
      if (event.payload.error) {
        gitlabError = event.payload.error;
        gitlabSyncMessage = null;
      } else {
        gitlabError = null;
        gitlabSyncMessage = `Synced ${event.payload.count} MR${event.payload.count === 1 ? '' : 's'}`;
      }
    });

    // OAuth flow completed — Rust side already triggers the first sync
    unlistenConnected = await listen('calendar://connected', () => {
      calendarConnected = true;
    });

    // Sync result (success or error) from any sync source
    unlistenSynced = await listen<{ count: number; error: string | null }>('calendar://synced', (event) => {
      calendarSyncing = false;
      if (event.payload.error) {
        calendarError = event.payload.error;
        calendarSyncMessage = null;
      } else {
        calendarError = null;
        calendarSyncMessage = `Synced ${event.payload.count} event${event.payload.count === 1 ? '' : 's'}`;
      }
    });

    // OAuth or token-exchange errors
    unlistenError = await listen<{ message: string }>('calendar://error', (event) => {
      calendarError = event.payload.message;
    });
  });

  onDestroy(() => {
    unlistenConnected?.();
    unlistenSynced?.();
    unlistenError?.();
    unlistenGitlabSynced?.();
  });
</script>

<div class="min-h-screen bg-[var(--color-background)] text-[var(--color-text)] flex flex-col">

  <!-- Header bar -->
  <header class="flex items-center justify-between px-6 py-3 border-b border-[var(--color-border)] shrink-0">
    <a
      href="/board"
      class="text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors flex items-center gap-1"
    >
      ← Board
    </a>
    <h1 class="text-sm font-medium text-[var(--color-text)] absolute left-1/2 -translate-x-1/2">
      Settings
    </h1>
    <!-- spacer to balance the back link -->
    <div class="w-14"></div>
  </header>

  <!-- Scrollable body -->
  <main class="flex-1 overflow-y-auto px-6 py-6 max-w-xl mx-auto w-full">

    <!-- Section 1: Workload -->
    <section>
      <p class="text-xs uppercase tracking-wide text-[var(--color-muted)] mb-3">Workload</p>

      <div class="flex items-center gap-3">
        <label for="available-hours" class="text-sm text-[var(--color-text-muted)] whitespace-nowrap">
          Available hours per day
        </label>
        <input
          id="available-hours"
          type="number"
          step="0.5"
          min="1"
          max="16"
          bind:value={availableHours}
          onblur={onHoursBlur}
          class="w-20 bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1 text-sm text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-indigo-500 text-center"
        />
        {#if hoursSaved}
          <span class="text-xs text-emerald-500 transition-opacity">Saved</span>
        {/if}
      </div>
    </section>

    <!-- Section 2: AI -->
    <section class="border-t border-[var(--color-border)] mt-6 pt-6">
      <p class="text-xs uppercase tracking-wide text-[var(--color-muted)] mb-3">AI</p>

      <!-- Provider pill toggle -->
      <div class="flex items-center gap-1 p-0.5 rounded-md bg-[var(--color-surface)] border border-[var(--color-border)] w-fit mb-4">
        {#each (['anthropic', 'openai'] as AiProvider[]) as provider}
          <button
            onclick={() => selectProvider(provider)}
            class="px-3 py-1 rounded text-sm transition-colors {selectedProvider === provider
              ? 'bg-indigo-600 text-white'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
          >
            {provider === 'anthropic' ? 'Anthropic' : 'OpenAI'}
          </button>
        {/each}
      </div>

      <!-- API key input -->
      <div class="flex flex-col gap-2">
        <label for="api-key" class="text-sm text-[var(--color-text-muted)]">
          {selectedProvider === 'anthropic' ? 'Anthropic' : 'OpenAI'} API key
        </label>
        <div class="flex gap-2">
          <input
            id="api-key"
            type="password"
            bind:value={apiKeyInput}
            placeholder={keyIsSaved ? 'Key saved — enter to replace' : 'sk-…'}
            class="flex-1 bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-indigo-500"
            onkeydown={(e) => { if (e.key === 'Enter') saveApiKey(); }}
          />
          <button
            onclick={saveApiKey}
            disabled={!apiKeyInput.trim()}
            class="px-3 py-1.5 rounded bg-indigo-600/80 hover:bg-indigo-600 disabled:opacity-40 disabled:cursor-not-allowed text-sm text-white transition-colors"
          >
            Save
          </button>
        </div>
        {#if keySaved}
          <span class="text-xs text-emerald-500">Saved</span>
        {/if}
      </div>
    </section>

    <!-- Section 3: Integrations -->
    <section class="border-t border-[var(--color-border)] mt-6 pt-6">
      <p class="text-xs uppercase tracking-wide text-[var(--color-muted)] mb-3">Integrations</p>

      <div>
        <!-- Google Calendar — live status -->
        <IntegrationCard
          name="Google Calendar"
          description="Import meetings as cards"
          status={calendarConnected ? 'connected' : 'not_connected'}
          onConnect={connectCalendar}
        />
        {#if calendarConnected}
          <div class="flex items-center gap-4 px-0 py-2 border-b border-[var(--color-border)]">
            <button
              onclick={syncCalendar}
              disabled={calendarSyncing}
              class="text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)] disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >
              {calendarSyncing ? 'Syncing…' : 'Sync now'}
            </button>
            {#if calendarSyncMessage}
              <span class="text-xs text-emerald-500/80">{calendarSyncMessage}</span>
            {/if}
            <button
              onclick={disconnectCalendar}
              class="text-xs text-red-400/70 hover:text-red-400 transition-colors ml-auto"
            >
              Disconnect
            </button>
          </div>
        {/if}
        {#if calendarError}
          <p class="text-xs text-red-400/90 py-2 border-b border-[var(--color-border)]">{calendarError}</p>
        {/if}

        <!-- GitLab -->
        <IntegrationCard
          name="GitLab"
          description="Import open and review MRs"
          status={gitlabConnected ? 'connected' : 'not_connected'}
          onConnect={() => { gitlabPatInput = ''; }}
        />
        {#if !gitlabConnected}
          <div class="flex items-center gap-2 px-0 py-2 border-b border-[var(--color-border)]">
            <input
              type="password"
              bind:value={gitlabPatInput}
              placeholder="glpat-…"
              class="flex-1 bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-indigo-500"
              onkeydown={(e) => { if (e.key === 'Enter') saveGitlabPat(); }}
            />
            <button
              onclick={saveGitlabPat}
              disabled={!gitlabPatInput.trim()}
              class="px-3 py-1.5 rounded bg-indigo-600/80 hover:bg-indigo-600 disabled:opacity-40 disabled:cursor-not-allowed text-sm text-white transition-colors"
            >
              Save
            </button>
            {#if gitlabPatSaved}
              <span class="text-xs text-emerald-500">Saved</span>
            {/if}
          </div>
        {/if}
        {#if gitlabConnected && !gitlabEditingPat}
          <div class="flex items-center gap-4 px-0 py-2 border-b border-[var(--color-border)]">
            <button
              onclick={syncGitlab}
              disabled={gitlabSyncing}
              class="text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)] disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >
              {gitlabSyncing ? 'Syncing…' : 'Sync now'}
            </button>
            {#if gitlabSyncMessage}
              <span class="text-xs text-emerald-500/80">{gitlabSyncMessage}</span>
            {/if}
            <div class="flex items-center gap-3 ml-auto">
              <button
                onclick={() => { gitlabEditingPat = true; gitlabError = null; gitlabSyncMessage = null; }}
                class="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
              >
                Replace key
              </button>
              <button
                onclick={disconnectGitlab}
                class="text-xs text-red-400/70 hover:text-red-400 transition-colors"
              >
                Disconnect
              </button>
            </div>
          </div>
        {/if}
        {#if gitlabEditingPat}
          <div class="flex items-center gap-2 px-0 py-2 border-b border-[var(--color-border)]">
            <input
              type="password"
              bind:value={gitlabPatInput}
              placeholder="glpat-…"
              class="flex-1 bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-indigo-500"
              onkeydown={(e) => { if (e.key === 'Enter') saveGitlabPat(); if (e.key === 'Escape') { gitlabEditingPat = false; gitlabPatInput = ''; } }}
            />
            <button
              onclick={saveGitlabPat}
              disabled={!gitlabPatInput.trim()}
              class="px-3 py-1.5 rounded bg-indigo-600/80 hover:bg-indigo-600 disabled:opacity-40 disabled:cursor-not-allowed text-sm text-white transition-colors"
            >
              Save
            </button>
            <button
              onclick={() => { gitlabEditingPat = false; gitlabPatInput = ''; }}
              class="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
            >
              Cancel
            </button>
          </div>
        {/if}
        {#if gitlabError}
          <p class="text-xs text-red-400/90 py-2 border-b border-[var(--color-border)]">{gitlabError}</p>
        {/if}

        <!-- Other integrations — coming soon -->
        {#each integrations as integration (integration.id)}
          <IntegrationCard
            name={integration.name}
            description={integration.description}
            status={integration.status}
          />
        {/each}
      </div>
    </section>

  </main>
</div>
