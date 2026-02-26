<script lang="ts">
  let {
    name,
    description,
    status,
    onConnect
  }: {
    name: string;
    description: string;
    status: 'connected' | 'not_connected' | 'coming_soon';
    onConnect?: () => void;
  } = $props();
</script>

<div class="flex items-center justify-between px-0 py-3 border-b border-[var(--color-border)] last:border-b-0">
  <div class="flex flex-col gap-0.5">
    <span class="text-sm font-medium text-[var(--color-text)]">{name}</span>
    <span class="text-xs text-[var(--color-text-muted)]">{description}</span>
  </div>

  <div class="flex items-center gap-2 ml-4 shrink-0">
    {#if status === 'connected'}
      <span class="flex items-center gap-1.5 text-xs text-emerald-500/80">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 inline-block"></span>
        Connected
      </span>
    {:else if status === 'not_connected'}
      <button
        onclick={onConnect}
        class="text-xs px-2.5 py-1 rounded border border-indigo-500/50 text-indigo-400 hover:bg-indigo-500/10 transition-colors"
      >
        Connect
      </button>
    {:else}
      <span class="text-xs px-2 py-0.5 rounded bg-[var(--color-surface)] text-[var(--color-muted)] border border-[var(--color-border)]">
        Coming soon
      </span>
    {/if}
  </div>
</div>
