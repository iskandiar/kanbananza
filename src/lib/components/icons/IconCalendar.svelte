<script lang="ts">
  import { CalendarDays } from 'lucide-svelte';

  let { class: className = '' } = $props();
</script>

<CalendarDays size={13} class={className} />
