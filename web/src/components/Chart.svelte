<script lang="ts">
  // A small line-and-band chart drawn as SVG. The table next to each chart
  // carries the same numbers for screen readers.

  interface Line {
    points: [number, number][]
    tone: 'accent' | 'warn' | 'muted'
    dashed?: boolean
  }

  interface Props {
    label: string
    lines: Line[]
    band?: { x: number; low: number; high: number }[]
    markers?: { x: number; tone: 'accent' | 'muted' }[]
    xLabel: (x: number) => string
    yLabel: (y: number) => string
  }

  let { label, lines, band = [], markers = [], xLabel, yLabel }: Props = $props()

  const W = 760
  const H = 300
  const left = 64
  const right = 16
  const top = 16
  const bottom = 32

  const xs = $derived([...lines.flatMap((l) => l.points.map((p) => p[0])), ...band.map((b) => b.x)])
  const ys = $derived([...lines.flatMap((l) => l.points.map((p) => p[1])), ...band.map((b) => b.high), 0])
  const xMin = $derived(Math.min(...xs))
  const xMax = $derived(Math.max(...xs))
  const yMax = $derived(niceMax(Math.max(...ys)))

  function niceMax(v: number): number {
    if (v <= 0) return 1
    const step = 10 ** Math.floor(Math.log10(v))
    return Math.ceil(v / step) * step
  }

  const x = (v: number) => left + ((v - xMin) / Math.max(1, xMax - xMin)) * (W - left - right)
  const y = (v: number) => top + (1 - v / yMax) * (H - top - bottom)

  const path = (points: [number, number][]) => points.map(([px, py], i) => `${i ? 'L' : 'M'}${x(px).toFixed(1)},${y(py).toFixed(1)}`).join(' ')
  const area = $derived(
    band.length
      ? `${band.map((b, i) => `${i ? 'L' : 'M'}${x(b.x).toFixed(1)},${y(b.high).toFixed(1)}`).join(' ')} ${[...band]
          .reverse()
          .map((b) => `L${x(b.x).toFixed(1)},${y(b.low).toFixed(1)}`)
          .join(' ')} Z`
      : '',
  )
  const yTicks = $derived([0, 0.25, 0.5, 0.75, 1].map((f) => f * yMax))
  const xTicks = $derived.by(() => {
    const span = xMax - xMin
    const step = span <= 10 ? 1 : span <= 25 ? 5 : 10
    const ticks: number[] = []
    for (let v = Math.ceil(xMin / step) * step; v <= xMax; v += step) ticks.push(v)
    return ticks
  })
</script>

<svg viewBox="0 0 {W} {H}" role="img" aria-label={label}>
  {#each yTicks as t (t)}
    <line x1={left} x2={W - right} y1={y(t)} y2={y(t)} class="grid" />
    <text x={left - 8} y={y(t) + 4} text-anchor="end" class="tick">{yLabel(t)}</text>
  {/each}
  {#each xTicks as t (t)}
    <text x={x(t)} y={H - 10} text-anchor="middle" class="tick">{xLabel(t)}</text>
  {/each}
  {#if area}<path d={area} class="band" />{/if}
  {#each markers as m, i (i)}
    <line x1={x(m.x)} x2={x(m.x)} y1={top} y2={H - bottom} class="marker {m.tone}" />
  {/each}
  {#each lines as l, i (i)}
    <path d={path(l.points)} class="line {l.tone}" class:dashed={l.dashed} />
  {/each}
</svg>

<style>
  svg {
    width: 100%;
    height: auto;
    display: block;
  }
  .grid {
    stroke: var(--line-soft);
  }
  .tick {
    font: 11px var(--mono);
    fill: var(--muted);
  }
  .band {
    fill: var(--accent-tint);
  }
  .line {
    fill: none;
    stroke-width: 3;
    stroke-linejoin: round;
  }
  .accent {
    stroke: var(--accent);
  }
  .warn {
    stroke: var(--warn);
  }
  .muted {
    stroke: var(--muted);
  }
  .dashed {
    stroke-dasharray: 6 5;
    stroke-width: 2;
  }
  .marker {
    stroke-width: 2;
  }
  .marker.muted {
    stroke-dasharray: 4 4;
  }
</style>
