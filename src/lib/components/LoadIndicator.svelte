<script lang="ts">
  let { scheduledHours, availableHours }: { scheduledHours: number; availableHours: number } = $props();

  const ratio = $derived(availableHours > 0 ? scheduledHours / availableHours : 0);
  const pct = $derived(Math.min(ratio * 100, 100));
  const color = $derived(
    ratio > 0.9 ? 'bg-rose-500' : ratio > 0.7 ? 'bg-amber-400' : 'bg-emerald-500'
  );
</script>

<div class="flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
  <div class="h-1 flex-1 rounded-full bg-[var(--color-border)]">
    <div class="h-1 rounded-full transition-all {color}" style="width: {pct}%"></div>
  </div>
  <span>{scheduledHours.toFixed(1)}h / {availableHours}h</span>
</div>
