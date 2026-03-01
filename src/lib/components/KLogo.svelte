<script lang="ts">
  let { size = 28, theme = 'dark' }: { size?: number; theme?: 'dark' | 'light' } = $props();

  // Unique gradient IDs so multiple instances never clash in the DOM
  const uid = Math.random().toString(36).slice(2, 7);

  // Exact colors from app.css variables
  const dark = {
    bg: '#161A22',        // --color-surface
    bgEnd: '#0D0F14',     // --color-background
    kTop: '#5A94FF',      // --color-accent-hover
    kBot: '#3D7EFF',      // --color-accent
    border: 'rgba(255,255,255,0.09)', // --color-glass-border
  };
  const light = {
    bg: '#F0EDE6',        // --color-surface (light)
    bgEnd: '#E6E2D8',     // --color-surface-raised (light)
    kTop: '#3D7EFF',      // --color-accent (same both themes)
    kBot: '#2B6AEE',      // --color-accent-hover (light)
    border: 'rgba(150,120,80,0.13)', // --color-glass-border (light)
  };

  const c = $derived(theme === 'light' ? light : dark);

  // K geometry — generous padding, K occupies ~55% of icon
  // Bar: x=[116,180], y=[120,392] → center Y=256
  // Arms taper naturally (60px at bar, 58px at tip)
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 512 512"
  xmlns="http://www.w3.org/2000/svg"
  aria-label="Kanbananza"
  role="img"
>
  <defs>
    <linearGradient id="kl-bg-{uid}" x1="0" y1="0" x2="1" y2="1" gradientUnits="objectBoundingBox">
      <stop offset="0%"   stop-color={c.bg}/>
      <stop offset="100%" stop-color={c.bgEnd}/>
    </linearGradient>

    <!-- Subtle 2-stop gradient on K — just enough dimension, no gloss -->
    <linearGradient id="kl-k-{uid}" x1="256" y1="120" x2="256" y2="392" gradientUnits="userSpaceOnUse">
      <stop offset="0%"   stop-color={c.kTop}/>
      <stop offset="100%" stop-color={c.kBot}/>
    </linearGradient>
  </defs>

  <!-- Background -->
  <rect width="512" height="512" rx="110" ry="110" fill="url(#kl-bg-{uid})"/>

  <!-- K — vertical bar + upper arm + lower arm -->
  <rect x="116" y="120" width="64" height="272" rx="6" ry="6" fill="url(#kl-k-{uid})"/>
  <polygon points="180,200 396,120 396,178 180,256"  fill="url(#kl-k-{uid})"/>
  <polygon points="180,256 396,334 396,392 180,312"  fill="url(#kl-k-{uid})"/>

  <!-- Hairline border -->
  <rect x="1.5" y="1.5" width="509" height="509" rx="109" ry="109"
        fill="none" stroke={c.border} stroke-width="1.5"/>
</svg>
