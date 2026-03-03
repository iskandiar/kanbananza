<script lang="ts">
  let { doneHours, plannedHours, availableHours, clockedHours }: {
    doneHours: number;
    plannedHours: number;
    availableHours: number;
    clockedHours?: number;
  } = $props();

  const total = $derived(doneHours + plannedHours);
  const donePct = $derived(availableHours > 0 ? Math.min((doneHours / availableHours) * 100, 100) : 0);
  const plannedPct = $derived(availableHours > 0 ? Math.min((plannedHours / availableHours) * 100, Math.max(0, 100 - donePct)) : 0);
  const clockedPct = $derived(
    clockedHours != null && availableHours > 0
      ? Math.min((clockedHours / availableHours) * 100, 100)
      : null
  );
</script>

<div>
  <div class="flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
    <div class="relative h-1 flex-1 rounded-full bg-[var(--color-border)]">
      <div class="flex h-1 rounded-full overflow-hidden">
        <div
          class="h-1 bg-emerald-500 transition-all"
          style="width: {donePct}%"
        ></div>
        <div
          class="h-1 bg-amber-500 transition-all"
          style="width: {plannedPct}%"
        ></div>
      </div>
      {#if clockedPct != null}
        <div
          class="absolute top-1/2 -translate-y-1/2 w-0.5 h-3 bg-[var(--color-accent)] rounded-sm pointer-events-none"
          style="left: calc({clockedPct}% - 1px)"
        ></div>
      {/if}
    </div>
    <span>{total.toFixed(1)}h / {availableHours}h</span>
  </div>
  {#if availableHours > 0 && total > availableHours}
    <p class="text-xs text-rose-400 mt-0.5">⚠ {(total - availableHours).toFixed(1)}h over</p>
  {/if}
</div>
