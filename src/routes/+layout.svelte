<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  onMount(() => {
    themeStore.init();
  });
</script>

{@render children()}
<Toast />
