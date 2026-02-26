<script lang="ts">
  let { scheduledHours, availableHours }: { scheduledHours: number; availableHours: number } = $props();

  const ratio = $derived(availableHours > 0 ? scheduledHours / availableHours : 0);
  const pct = $derived(Math.min(ratio * 100, 100));
  // green → amber → red as day fills
  const barColor = $derived(
    ratio > 0.9
      ? 'bg-[var(--color-impact-high)]'
      : ratio > 0.7
        ? 'bg-[var(--color-impact-mid)]'
        : 'bg-[var(--color-done)]'
  );
</script>

<div class="flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
  <div class="h-1 flex-1 rounded-full bg-[var(--color-border)]">
    <div class="h-1 rounded-full transition-all {barColor}" style="width: {pct}%"></div>
  </div>
  <span>{scheduledHours.toFixed(1)}h / {availableHours}h</span>
</div>
