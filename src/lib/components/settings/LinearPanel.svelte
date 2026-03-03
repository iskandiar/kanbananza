<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getSecret, saveLinearApiKey, syncLinear, disconnectLinear } from '$lib/api/settings';
  import IntegrationCard from '../IntegrationCard.svelte';
  import SyncReviewModal from '$lib/components/SyncReviewModal.svelte';

  let connected = $state(false);
  let editing = $state(false);
  let keyInput = $state('');
  let saved = $state(false);
  let syncing = $state(false);
  let error = $state<string | null>(null);
  let syncMessage = $state<string | null>(null);
  let showReviewModal = $state(false);
  let unlistenSynced: (() => void) | null = null;

  onMount(async () => {
    connected = (await getSecret('kanbananza', 'linear_api_key')) !== null;

    unlistenSynced = await listen<{ count: number; error: string | null }>('linear://synced', (event) => {
      syncing = false;
      if (event.payload.error) {
        error = event.payload.error;
        syncMessage = null;
      } else {
        error = null;
        syncMessage = `Synced ${event.payload.count} issue${event.payload.count === 1 ? '' : 's'}`;
      }
    });
  });

  onDestroy(() => {
    unlistenSynced?.();
  });

  async function saveKey() {
    if (!keyInput.trim()) return;
    try {
      await saveLinearApiKey(keyInput.trim());
      connected = true;
      editing = false;
      keyInput = '';
      saved = true;
      setTimeout(() => { saved = false; }, 1500);
    } catch (e) {
      error = `Failed to save key: ${String(e)}`;
    }
  }

  async function sync() {
    error = null;
    syncMessage = null;
    syncing = true;
    try {
      await syncLinear();
    } catch (e) {
      error = String(e);
      syncing = false;
    }
  }

  async function disconnect() {
    await disconnectLinear();
    connected = false;
    error = null;
    syncMessage = null;
  }
</script>

<div>
  <IntegrationCard
    name="Linear"
    description="Import assigned issues"
    status={connected ? 'connected' : 'not_connected'}
    onConnect={() => { keyInput = ''; }}
  />
  {#if !connected}
    <div class="flex items-center gap-2 px-0 py-2 border-b border-[var(--color-border)]">
      <input
        type="password"
        bind:value={keyInput}
        placeholder="lin_api_…"
        class="flex-1 bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-indigo-500"
        onkeydown={(e) => { if (e.key === 'Enter') saveKey(); }}
      />
      <button
        onclick={saveKey}
        disabled={!keyInput.trim()}
        class="px-3 py-1.5 rounded bg-indigo-600/80 hover:bg-indigo-600 disabled:opacity-40 disabled:cursor-not-allowed text-sm text-white transition-colors"
      >
        Save
      </button>
      {#if saved}
        <span class="text-xs text-emerald-500">Saved</span>
      {/if}
    </div>
  {/if}
  {#if connected && !editing}
    <div class="flex items-center gap-4 px-0 py-2 border-b border-[var(--color-border)]">
      <button
        onclick={() => showReviewModal = true}
        disabled={syncing}
        class="text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-text)] disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
      >
        {syncing ? 'Syncing…' : 'Sync now'}
      </button>
      {#if syncMessage}
        <span class="text-xs text-emerald-500/80">{syncMessage}</span>
      {/if}
      <div class="flex items-center gap-3 ml-auto">
        <button
          onclick={() => { editing = true; error = null; syncMessage = null; }}
          class="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
        >
          Replace key
        </button>
        <button
          onclick={disconnect}
          class="text-xs text-red-400/70 hover:text-red-400 transition-colors"
        >
          Disconnect
        </button>
      </div>
    </div>
  {/if}
  {#if editing}
    <div class="flex items-center gap-2 px-0 py-2 border-b border-[var(--color-border)]">
      <input
        type="password"
        bind:value={keyInput}
        placeholder="lin_api_…"
        class="flex-1 bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-indigo-500"
        onkeydown={(e) => { if (e.key === 'Enter') saveKey(); if (e.key === 'Escape') { editing = false; keyInput = ''; } }}
      />
      <button
        onclick={saveKey}
        disabled={!keyInput.trim()}
        class="px-3 py-1.5 rounded bg-indigo-600/80 hover:bg-indigo-600 disabled:opacity-40 disabled:cursor-not-allowed text-sm text-white transition-colors"
      >
        Save
      </button>
      <button
        onclick={() => { editing = false; keyInput = ''; }}
        class="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
      >
        Cancel
      </button>
    </div>
  {/if}
  {#if error}
    <p class="text-xs text-red-400/90 py-2 border-b border-[var(--color-border)]">{error}</p>
  {/if}
</div>

{#if showReviewModal}
  <SyncReviewModal
    source="linear"
    onClose={() => showReviewModal = false}
    onSynced={(n) => {
      syncMessage = `Imported ${n} item${n === 1 ? '' : 's'}`;
      showReviewModal = false;
    }}
  />
{/if}
