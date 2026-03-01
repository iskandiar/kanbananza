<script lang="ts">
  import { toastStore } from '$lib/stores/toast.svelte';
</script>

{#if toastStore.toasts.length > 0}
  <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 items-end pointer-events-none">
    {#each toastStore.toasts as toast (toast.id)}
      <div
        class="pointer-events-auto flex items-center gap-3 px-3.5 py-2.5 rounded-lg border backdrop-blur-md shadow-lg text-sm max-w-xs animate-slide-in
          {toast.type === 'success'
            ? 'bg-emerald-900/60 border-emerald-700/50 text-emerald-200'
            : toast.type === 'error'
              ? 'bg-rose-900/60 border-rose-700/50 text-rose-200'
              : 'bg-[var(--color-glass-header)] border-[var(--color-glass-border)] text-[var(--color-text)]'}"
      >
        <span class="flex-1">{toast.message}</span>
        <button
          onclick={() => toastStore.dismiss(toast.id)}
          class="flex-shrink-0 opacity-60 hover:opacity-100 transition-opacity text-xs leading-none"
          aria-label="Dismiss"
        >×</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  @keyframes slide-in {
    from { opacity: 0; transform: translateX(1rem); }
    to   { opacity: 1; transform: translateX(0); }
  }
  .animate-slide-in {
    animation: slide-in 0.18s ease-out both;
  }
</style>
