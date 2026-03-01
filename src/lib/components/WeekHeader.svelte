<script lang="ts">
  import { themeStore } from '$lib/stores/theme.svelte';

  let {
    weekLabel,
    unfinishedCount,
    isCurrentWeek = true,
    onPrev,
    onNext,
    onJumpToToday,
    onRollover
  }: { weekLabel: string; unfinishedCount: number; isCurrentWeek: boolean; onPrev: () => void; onNext: () => void; onJumpToToday: () => void; onRollover: () => void } = $props();

  let rolloverConfirming = $state(false);
  let showShortcuts = $state(false);

  function handleWindowClick(e: MouseEvent) {
    if (showShortcuts && !(e.target as Element).closest('.shortcuts-container')) {
      showShortcuts = false;
    }
  }
</script>

<svelte:window onclick={handleWindowClick} />

<header class="relative z-10 flex items-center justify-between px-4 py-3 border-b border-[var(--color-glass-border)] bg-[var(--color-glass-header)] backdrop-blur-md">
  <div class="flex items-center gap-1">
    <button
      onclick={onPrev}
      class="p-1.5 rounded hover:bg-[var(--color-surface-raised)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
      aria-label="Previous week"
    >
      ←
    </button>
    {#if weekLabel.includes(' · ')}
      {@const parts = weekLabel.split(' · ')}
      <span class="text-sm font-medium text-[var(--color-text)] px-2">{parts[0]} <span class="text-[var(--color-muted)] font-normal text-xs">· {parts[1]}</span></span>
    {:else}
      <span class="text-sm font-medium text-[var(--color-text)] px-2">{weekLabel}</span>
    {/if}
    <button
      onclick={onNext}
      class="p-1.5 rounded hover:bg-[var(--color-surface-raised)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
      aria-label="Next week"
    >
      →
    </button>
    {#if !isCurrentWeek}
      <button
        onclick={onJumpToToday}
        class="text-xs px-2 py-0.5 rounded border border-[var(--color-accent)]/60 text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 transition-colors"
        title="Jump to current week"
      >Today</button>
    {/if}
  </div>

  <div class="flex items-center gap-2">
    {#if isCurrentWeek}
      {#if !rolloverConfirming}
        <button
          onclick={() => (rolloverConfirming = true)}
          class="text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-accent)] transition-colors"
          title="Move unfinished cards to backlog"
        >
          Rollover
        </button>
      {:else}
        <div class="flex items-center gap-1.5">
          <span class="text-xs text-[var(--color-muted)]">Move {unfinishedCount} cards?</span>
          <button
            onclick={() => { onRollover(); rolloverConfirming = false; }}
            class="text-xs px-2 py-0.5 rounded border border-amber-500/60 text-amber-400 hover:bg-amber-500/10 transition-colors"
          >
            Confirm
          </button>
          <button
            onclick={() => (rolloverConfirming = false)}
            class="text-xs px-2 py-0.5 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] transition-colors"
          >
            Cancel
          </button>
        </div>
      {/if}
    {/if}
    <a
      href="/projects"
      class="text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-accent)] transition-colors"
      title="View projects"
    >
      Projects
    </a>
    <a
      href="/history"
      class="text-xs px-2.5 py-1 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-accent)] transition-colors"
      title="View history"
    >
      History
    </a>
    <div class="shortcuts-container relative">
      <button
        onclick={(e) => { e.stopPropagation(); showShortcuts = !showShortcuts; }}
        class="p-1.5 rounded hover:bg-[var(--color-surface-raised)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
        title="Keyboard shortcuts"
        aria-label="Keyboard shortcuts"
      >?</button>
      {#if showShortcuts}
        <div class="absolute right-0 top-full mt-1 bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-xs text-[var(--color-text-muted)] z-20 whitespace-nowrap shadow-lg">
          <div class="space-y-1.5">
            <div class="flex gap-4">
              <kbd class="font-mono">[</kbd>
              <span>Prev week</span>
            </div>
            <div class="flex gap-4">
              <kbd class="font-mono">]</kbd>
              <span>Next week</span>
            </div>
            <div class="flex gap-4">
              <kbd class="font-mono">B</kbd>
              <span>Toggle backlog</span>
            </div>
            <div class="flex gap-4">
              <kbd class="font-mono">⌘K</kbd>
              <span>Search</span>
            </div>
          </div>
        </div>
      {/if}
    </div>
    <button
      onclick={themeStore.toggle}
      class="p-1.5 rounded hover:bg-[var(--color-surface-raised)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
      aria-label={themeStore.current === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
      title={themeStore.current === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
    >
      {themeStore.current === 'dark' ? '☀' : '☾'}
    </button>
    <a
      href="/settings"
      class="p-1.5 rounded hover:bg-[var(--color-surface-raised)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
      aria-label="Settings"
      title="Settings"
    >
      ⚙
    </a>
  </div>
</header>
