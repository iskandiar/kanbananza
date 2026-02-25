<script lang="ts">
  let {
    weekLabel,
    onPrev,
    onNext
  }: { weekLabel: string; onPrev: () => void; onNext: () => void } = $props();
</script>

<header class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)]">
  <button
    onclick={onPrev}
    class="p-1.5 rounded hover:bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
    aria-label="Previous week"
  >
    ←
  </button>
  <span class="text-sm font-medium text-[var(--color-text)]">{weekLabel}</span>
  <button
    onclick={onNext}
    class="p-1.5 rounded hover:bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
    aria-label="Next week"
  >
    →
  </button>
</header>
