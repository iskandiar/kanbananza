<script lang="ts">
  import { onMount } from 'svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { getSecret, storeSecret, updateSettings, backupDatabase, restoreDatabase, clearAllData } from '$lib/api/settings';
  import { save, open } from '@tauri-apps/plugin-dialog';
  import { toastStore } from '$lib/stores/toast.svelte';
  import CalendarPanel from '$lib/components/settings/CalendarPanel.svelte';
  import GitLabPanel from '$lib/components/settings/GitLabPanel.svelte';
  import LinearPanel from '$lib/components/settings/LinearPanel.svelte';
  import NotionPanel from '$lib/components/settings/NotionPanel.svelte';
  import SlackPanel from '$lib/components/settings/SlackPanel.svelte';
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
  let autoAiEnabled = $state(false);
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

  async function toggleAutoAi() {
    autoAiEnabled = !autoAiEnabled;
    await updateSettings({ autoAi: autoAiEnabled });
  }

  // --- Backup / restore / clear state ---
  let backupStatus = $state<'idle' | 'success' | 'error'>('idle');
  let backupError = $state<string | null>(null);
  let clearConfirming = $state(false);

  async function handleRestore() {
    const path = await open({ filters: [{ name: 'Database', extensions: ['db'] }] });
    if (path) {
      await restoreDatabase(path as string);
      // app restarts automatically
    }
  }

  async function backupDb() {
    backupStatus = 'idle';
    backupError = null;
    const path = await save({ filters: [{ name: 'SQLite', extensions: ['db'] }], defaultPath: 'kanbananza.db' });
    if (!path) return;
    try {
      await backupDatabase(path);
      backupStatus = 'success';
      toastStore.add('Database backed up', 'success');
      setTimeout(() => { backupStatus = 'idle'; }, 2000);
    } catch (e) {
      backupStatus = 'error';
      backupError = String(e);
    }
  }

  onMount(async () => {
    await settingsStore.load();
    availableHours = settingsStore.availableHours;
    selectedProvider = settingsStore.settings?.ai_provider ?? 'openai';
    if (selectedProvider === 'anthropic') selectedProvider = 'openai';
    autoAiEnabled = settingsStore.settings?.auto_ai ?? false;
    await loadKeyStatus(selectedProvider);
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
        <button
          onclick={() => selectProvider('openai')}
          class="px-3 py-1 rounded text-sm transition-colors {selectedProvider === 'openai'
            ? 'bg-indigo-600 text-white'
            : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
        >
          OpenAI
        </button>
        <button
          disabled
          class="px-3 py-1 rounded text-sm transition-colors text-[var(--color-text-muted)] opacity-50 cursor-not-allowed relative"
        >
          Anthropic
          <span class="ml-1.5 text-[10px] bg-[var(--color-border)] px-1 rounded">Soon</span>
        </button>
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
        <p class="text-xs text-[var(--color-text-muted)] mt-2">
          Using <span class="font-mono">gpt-4o-mini</span> — fast &amp; affordable
        </p>
      </div>

      <!-- Auto-evaluate toggle -->
      <div class="flex items-center justify-between mt-4">
        <div>
          <p class="text-sm text-[var(--color-text)]">Auto-evaluate cards</p>
          <p class="text-xs text-[var(--color-text-muted)] mt-0.5">Automatically fills in title, description, impact and time estimate when cards are added</p>
        </div>
        <button
          onclick={toggleAutoAi}
          class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none {autoAiEnabled ? 'bg-indigo-600' : 'bg-[var(--color-border)]'}"
          role="switch"
          aria-checked={autoAiEnabled}
          aria-label="Auto-evaluate cards"
        >
          <span
            class="pointer-events-none inline-block h-4 w-4 rounded-full bg-white shadow transform transition duration-200 {autoAiEnabled ? 'translate-x-4' : 'translate-x-0'}"
          ></span>
        </button>
      </div>
    </section>

    <!-- Section 3: Integrations -->
    <section class="border-t border-[var(--color-border)] mt-6 pt-6">
      <p class="text-xs uppercase tracking-wide text-[var(--color-muted)] mb-3">Integrations</p>

      <div class="flex flex-col gap-0">
        <CalendarPanel />
        <GitLabPanel />
        <LinearPanel />
        <NotionPanel />
        <SlackPanel />
      </div>
    </section>

    <!-- Section 4: Data -->
    <section class="border-t border-[var(--color-border)] mt-6 pt-6">
      <p class="text-xs uppercase tracking-wide text-[var(--color-muted)] mb-3">Data</p>

      <div class="flex flex-col gap-4">
        <!-- Backup row -->
        <div class="flex items-center gap-4">
          <button
            onclick={backupDb}
            class="text-sm px-3 py-1.5 rounded border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)] transition-colors"
          >
            Backup database
          </button>
          {#if backupStatus === 'success'}
            <span class="text-xs text-emerald-500">Backup saved</span>
          {/if}
          {#if backupStatus === 'error'}
            <span class="text-xs text-red-400">{backupError}</span>
          {/if}
        </div>

        <!-- Restore row -->
        <div class="flex items-center gap-4">
          <button
            onclick={handleRestore}
            class="text-sm px-3 py-1.5 rounded border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)] transition-colors"
          >
            Restore from backup
          </button>
          <span class="text-xs text-[var(--color-muted)]">The app will restart after restore.</span>
        </div>

        <!-- Clear all data row -->
        <div class="flex items-center gap-4">
          {#if clearConfirming}
            <span class="text-sm text-[var(--color-text-muted)]">Are you sure?</span>
            <button
              onclick={async () => { await clearAllData(); window.location.reload(); }}
              class="text-sm px-3 py-1.5 rounded border border-red-500 text-red-400 hover:bg-red-500/10 transition-colors"
            >
              Confirm
            </button>
            <button
              onclick={() => { clearConfirming = false; }}
              class="text-sm px-3 py-1.5 rounded border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)] transition-colors"
            >
              Cancel
            </button>
          {:else}
            <button
              onclick={() => { clearConfirming = true; }}
              class="text-sm px-3 py-1.5 rounded border border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-red-400 hover:border-red-400 transition-colors"
            >
              Clear all data
            </button>
          {/if}
        </div>
      </div>
    </section>

  </main>
</div>
