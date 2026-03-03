<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { getCalendarAuthUrl } from '$lib/api/settings';
  import IntegrationCard from '../IntegrationCard.svelte';
  import SyncReviewModal from '$lib/components/SyncReviewModal.svelte';

  let connected = $state(false);
  let syncing = $state(false);
  let error = $state<string | null>(null);
  let syncMessage = $state<string | null>(null);
  let showReviewModal = $state(false);
  let unlistenConnected: (() => void) | null = null;
  let unlistenSynced: (() => void) | null = null;
  let unlistenError: (() => void) | null = null;

  onMount(async () => {
    connected = await invoke<boolean>('get_calendar_status');

    unlistenConnected = await listen('calendar://connected', () => {
      connected = true;
    });

    unlistenSynced = await listen<{ count: number; error: string | null }>('calendar://synced', (event) => {
      syncing = false;
      if (event.payload.error) {
        error = event.payload.error;
        syncMessage = null;
      } else {
        error = null;
        syncMessage = `Synced ${event.payload.count} event${event.payload.count === 1 ? '' : 's'}`;
      }
    });

    unlistenError = await listen<{ message: string }>('calendar://error', (event) => {
      error = event.payload.message;
    });
  });

  onDestroy(() => {
    unlistenConnected?.();
    unlistenSynced?.();
    unlistenError?.();
  });

  async function sync() {
    error = null;
    syncMessage = null;
    syncing = true;
    try {
      await invoke('sync_calendar');
    } catch (e) {
      error = String(e);
      syncing = false;
    }
  }

  async function connect() {
    error = null;
    syncMessage = null;
    await getCalendarAuthUrl();
  }

  async function disconnect() {
    await invoke('disconnect_calendar');
    connected = false;
    error = null;
    syncMessage = null;
  }
</script>

<div>
  <IntegrationCard
    name="Google Calendar"
    description="Import meetings as cards"
    status={connected ? 'connected' : 'not_connected'}
    onConnect={connect}
  />
  {#if connected}
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
      <button
        onclick={disconnect}
        class="text-xs text-red-400/70 hover:text-red-400 transition-colors ml-auto"
      >
        Disconnect
      </button>
    </div>
  {/if}
  {#if error}
    <p class="text-xs text-red-400/90 py-2 border-b border-[var(--color-border)]">{error}</p>
  {/if}
</div>

{#if showReviewModal}
  <SyncReviewModal
    source="calendar"
    onClose={() => showReviewModal = false}
    onSynced={(n) => {
      syncMessage = `Imported ${n} item${n === 1 ? '' : 's'}`;
      showReviewModal = false;
    }}
  />
{/if}
