<script lang="ts">
  let { size = 22 }: { size?: number } = $props()
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="var(--side-accent)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <path d="M4 18l6-6 4 4 6-8" />
  <path d="M15 8h5v5" />
</svg>
