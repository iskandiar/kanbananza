<script lang="ts">
  let { onAdd }: { onAdd: (title: string) => void } = $props();

  let active = $state(false);
  let value = $state('');

  function submit() {
    const title = value.trim();
    if (title) { onAdd(title); }
    value = '';
    active = false;
  }

  function cancel() {
    value = '';
    active = false;
  }
</script>

{#if active}
  <input
    class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1.5 text-sm text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-indigo-500"
    placeholder="Card title..."
    bind:value
    onkeydown={(e) => { if (e.key === 'Enter') submit(); if (e.key === 'Escape') cancel(); }}
    use:focus
  />
{:else}
  <button
    onclick={() => (active = true)}
    class="w-full text-left text-xs text-[var(--color-muted)] hover:text-[var(--color-text-muted)] py-1.5 px-1 transition-colors"
  >
    + Add card...
  </button>
{/if}

<script module lang="ts">
  function focus(node: HTMLElement) { node.focus(); }
</script>
