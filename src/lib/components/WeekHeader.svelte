<script lang="ts">
  let {
    weekLabel,
    onPrev,
    onNext,
    onRollover
  }: { weekLabel: string; onPrev: () => void; onNext: () => void; onRollover: () => void } = $props();
</script>

<header class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)]">
  <div class="flex items-center gap-1">
    <button
      onclick={onPrev}
      class="p-1.5 rounded hover:bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
      aria-label="Previous week"
    >
      ←
    </button>
    <span class="text-sm font-medium text-[var(--color-text)] px-2">{weekLabel}</span>
    <button
      onclick={onNext}
      class="p-1.5 rounded hover:bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
      aria-label="Next week"
    >
      →
    </button>
  </div>

  <div class="flex items-center gap-2">
    <button
      onclick={onRollover}
      class="text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] hover:border-indigo-500/50 transition-colors"
      title="Move unfinished cards to backlog"
    >
      Rollover
    </button>
    <a
      href="/settings"
      class="p-1.5 rounded hover:bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
      aria-label="Settings"
      title="Settings"
    >
      ⚙
    </a>
  </div>
</header>
