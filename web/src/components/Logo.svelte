<script lang="ts">
  // Three rounded squares that grow as they climb, each cut out of the next
  // by a thin gap. `gap` must match the background the mark sits on.
  let { size = 22, gap = 'var(--side)' }: { size?: number; gap?: string } = $props()
</script>

<svg width={size} height={size} viewBox="10 10 44 44" aria-hidden="true">
  <g stroke={gap} stroke-width="2.6">
    <rect x="29.01" y="13.82" width="22.99" height="22.99" rx="6.9" fill="#45c75b" />
    <rect x="19.09" y="28.19" width="16.42" height="16.42" rx="4.93" fill="#2a9ba3" />
    <rect x="12" y="38.45" width="11.73" height="11.73" rx="3.52" fill="#4a6cff" />
  </g>
</svg>
