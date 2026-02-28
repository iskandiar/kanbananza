<script lang="ts">
  import { onMount } from 'svelte';
  import { getSecret, saveNotionApiKey, deleteSecret } from '$lib/api/settings';
  import IntegrationCard from '../IntegrationCard.svelte';

  let connected = $state(false);
  let keyInput = $state('');
  let saved = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    connected = (await getSecret('kanbananza', 'notion_api_key')) !== null;
  });

  async function saveKey() {
    if (!keyInput.trim()) return;
    try {
      await saveNotionApiKey(keyInput.trim());
      connected = true;
      keyInput = '';
      saved = true;
      setTimeout(() => { saved = false; }, 1500);
    } catch (e) {
      error = `Failed to save token: ${String(e)}`;
    }
  }

  async function disconnect() {
    await deleteSecret('kanbananza', 'notion_api_key');
    connected = false;
    error = null;
  }
</script>

<div>
  <IntegrationCard
    name="Notion"
    description="Import action items"
    status={connected ? 'connected' : 'not_connected'}
    onConnect={() => { keyInput = ''; }}
  />
  {#if !connected}
    <div class="flex flex-col gap-2 px-0 py-2 border-b border-[var(--color-border)]">
      <div class="flex items-center gap-2">
        <input
          type="password"
          bind:value={keyInput}
          placeholder="secret_…"
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
      <p class="text-xs text-[var(--color-text-muted)]">Share pages with your integration in Notion to allow access</p>
    </div>
  {/if}
  {#if connected}
    <div class="flex items-center gap-4 px-0 py-2 border-b border-[var(--color-border)]">
      <p class="text-xs text-[var(--color-text-muted)]">Share pages with your integration in Notion to allow access</p>
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
