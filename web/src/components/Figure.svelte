<script lang="ts">
  // A chart the CFO asked for: bars, parts of a whole, or lines over time.
  // Every chart has its numbers in a table too, for screen readers and for
  // anyone who wants the exact figures.
  import { i18n, t } from '../lib/i18n.svelte'
  import type { Figure, Item } from '../lib/figure'

  let { figure }: { figure: Figure } = $props()

  const currency = $derived(/^[A-Z]{3}$/.test(figure.unit) ? figure.unit : '')

  function amount(v: number, compact = false): string {
    const options: Intl.NumberFormatOptions = { maximumFractionDigits: Math.abs(v) < 10 && !Number.isInteger(v) ? 2 : 0 }
    if (compact) options.notation = 'compact'
    if (currency) return new Intl.NumberFormat(i18n.lang, { ...options, style: 'currency', currency }).format(v)
    const n = new Intl.NumberFormat(i18n.lang, options).format(v)
    return figure.unit === '%' ? `${n}%` : figure.unit ? `${n} ${figure.unit}` : n
  }

  const label = (i: Item) => (i.other ? t('cfo.other') : i.label)
  const percent = (v: number) => new Intl.NumberFormat(i18n.lang, { style: 'percent', maximumFractionDigits: 1 }).format(v)

  // Bars: one scale from zero to the largest amount or target.
  const top = $derived(figure.type === 'bar' ? Math.max(...figure.items.flatMap((i) => [Math.abs(i.value), Math.abs(i.target ?? 0)]), 1) : 1)
  const width = (v: number) => `${(Math.abs(v) / top) * 100}%`

  const total = $derived(figure.type === 'share' ? figure.items.reduce((s, i) => s + i.value, 0) : 0)

  // Lines: a plain SVG with a crosshair that reads out every series.
  // Drawn at the width it is shown at, so the labels keep their size.
  let measured = $state(640)
  const W = $derived(Math.max(280, measured))
  const H = 220
  const pad = { left: 64, right: 16, top: 12, bottom: 28 }
  const values = $derived(figure.type === 'line' ? figure.series.flatMap((s) => s.values) : [])
  // Lines show change, so the scale fits the values rather than starting at zero.
  const step = $derived.by(() => {
    const span = Math.max(...values) - Math.min(...values) || Math.abs(values[0] ?? 1) || 1
    const raw = span / 2
    const p = 10 ** Math.floor(Math.log10(raw))
    return [1, 2, 2.5, 5, 10].map((m) => m * p).find((s) => s >= raw)!
  })
  const yMin = $derived(Math.floor(Math.min(...values) / step) * step)
  const yMax = $derived(Math.max(yMin + step, Math.ceil(Math.max(...values) / step) * step))
  const count = $derived(figure.type === 'line' ? figure.labels.length : 0)
  const x = (i: number) => pad.left + (i / Math.max(1, count - 1)) * (W - pad.left - pad.right)
  const y = (v: number) => pad.top + (1 - (v - yMin) / (yMax - yMin || 1)) * (H - pad.top - pad.bottom)
  const yTicks = $derived(Array.from({ length: Math.round((yMax - yMin) / step) + 1 }, (_, i) => yMin + i * step))
  const every = $derived(Math.max(1, Math.ceil(count / Math.max(2, Math.floor(W / 70)))))
  let hover = $state<number | null>(null)

  function move(e: PointerEvent): void {
    const box = (e.currentTarget as SVGElement).getBoundingClientRect()
    const at = ((e.clientX - box.left) / box.width) * W
    hover = Math.max(0, Math.min(count - 1, Math.round(((at - pad.left) / (W - pad.left - pad.right)) * (count - 1))))
  }
</script>

<figure class="figure">
  {#if figure.title}<figcaption>{figure.title}</figcaption>{/if}

  {#if figure.type === 'bar'}
    <ul class="bars">
      {#each figure.items as item, i (i)}
        <li title="{label(item)}: {amount(item.value)}{item.target !== undefined ? ` / ${amount(item.target)}` : ''}">
          <span class="name">{label(item)}</span>
          <span class="track">
            <span class="bar" class:negative={item.value < 0} style:width={width(item.value)}></span>
            {#if item.target !== undefined}<span class="target" style:left={width(item.target)}></span>{/if}
          </span>
          <span class="num value">{amount(item.value)}{#if item.target !== undefined}<span class="muted">&nbsp;/ {amount(item.target)}</span>{/if}</span>
        </li>
      {/each}
    </ul>
  {:else if figure.type === 'share'}
    <div class="stack" role="img" aria-label={figure.title}>
      {#each figure.items as item, i (i)}
        <span class="segment" style:flex-grow={item.value} style:background="var(--series-{i + 1})" title="{label(item)}: {amount(item.value)} ({percent(item.value / total)})"></span>
      {/each}
    </div>
    <ul class="legend">
      {#each figure.items as item, i (i)}
        <li>
          <span class="swatch" style:background="var(--series-{i + 1})"></span>
          <span class="name">{label(item)}</span>
          <span class="num">{amount(item.value)}</span>
          <span class="num muted">{percent(item.value / total)}</span>
        </li>
      {/each}
    </ul>
  {:else}
    {@const f = figure}
    {#if f.series.length > 1}
      <ul class="legend inline">
        {#each f.series as s, i (i)}<li><span class="swatch line" style:background="var(--series-{i + 1})"></span>{s.name}</li>{/each}
      </ul>
    {/if}
    <div class="plot" bind:clientWidth={measured}>
      <svg viewBox="0 0 {W} {H}" role="img" aria-label={f.title} onpointermove={move} onpointerleave={() => (hover = null)}>
        {#each yTicks as v (v)}
          <line x1={pad.left} x2={W - pad.right} y1={y(v)} y2={y(v)} class="grid" />
          <text x={pad.left - 8} y={y(v) + 4} text-anchor="end" class="tick">{amount(v, true)}</text>
        {/each}
        {#each f.labels as l, i (i)}
          {#if i % every === 0 || i === count - 1}<text x={x(i)} y={H - 8} text-anchor="middle" class="tick">{l}</text>{/if}
        {/each}
        {#if hover !== null}<line x1={x(hover)} x2={x(hover)} y1={pad.top} y2={H - pad.bottom} class="cross" />{/if}
        {#each f.series as s, i (i)}
          <path d={s.values.map((v, j) => `${j ? 'L' : 'M'}${x(j).toFixed(1)},${y(v).toFixed(1)}`).join(' ')} class="line" style:stroke="var(--series-{i + 1})" />
          {#if hover !== null}<circle cx={x(hover)} cy={y(s.values[hover])} r="4.5" class="dot" style:fill="var(--series-{i + 1})" />{/if}
        {/each}
      </svg>
      {#if hover !== null}
        <div class="tip" style:left="{(x(hover) / W) * 100}%">
          <strong>{f.labels[hover]}</strong>
          {#each f.series as s, i (i)}
            <span><span class="swatch" style:background="var(--series-{i + 1})"></span>{f.series.length > 1 ? `${s.name}: ` : ''}<span class="num">{amount(s.values[hover])}</span></span>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <details class="table">
    <summary>{t('cfo.table')}</summary>
    <table>
      {#if figure.type === 'line'}
        <thead><tr><th></th>{#each figure.series as s, i (i)}<th>{s.name}</th>{/each}</tr></thead>
        <tbody>
          {#each figure.labels as l, j (j)}
            <tr><th>{l}</th>{#each figure.series as s, i (i)}<td class="num">{amount(s.values[j])}</td>{/each}</tr>
          {/each}
        </tbody>
      {:else}
        <tbody>
          {#each figure.items as item, i (i)}
            <tr>
              <th>{label(item)}</th>
              <td class="num">{amount(item.value)}</td>
              {#if figure.type === 'share'}<td class="num">{percent(item.value / total)}</td>{:else if item.target !== undefined}<td class="num">{amount(item.target)}</td>{/if}
            </tr>
          {/each}
        </tbody>
      {/if}
    </table>
  </details>
</figure>

<style>
  .figure {
    margin: 0;
    padding: 14px 16px;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--surface);
    display: grid;
    gap: 12px;
    min-width: 0;
    white-space: normal;
  }
  figcaption {
    font-weight: 600;
    font-size: 15px;
    color: var(--ink);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .bars {
    display: grid;
    gap: 10px;
  }
  .bars li {
    display: grid;
    grid-template-columns: minmax(80px, 30%) 1fr auto;
    align-items: center;
    gap: 10px;
    font-size: 14px;
  }
  .name {
    color: var(--ink-2);
    overflow-wrap: anywhere;
  }
  .track {
    position: relative;
    height: 14px;
  }
  .bar {
    position: absolute;
    inset: 0 auto 0 0;
    min-width: 2px;
    border-radius: 0 4px 4px 0;
    background: var(--series-1);
  }
  .bar.negative {
    background: var(--warn);
  }
  .target {
    position: absolute;
    top: -4px;
    bottom: -4px;
    width: 2px;
    margin-left: -1px;
    background: var(--ink);
  }
  .value {
    font-size: 13px;
    text-align: right;
    white-space: nowrap;
  }
  .stack {
    display: flex;
    gap: 2px;
    height: 18px;
  }
  .segment {
    flex-basis: 0;
    min-width: 3px;
  }
  .segment:first-child {
    border-radius: 4px 0 0 4px;
  }
  .segment:last-child {
    border-radius: 0 4px 4px 0;
  }
  .legend {
    display: grid;
    gap: 6px;
    font-size: 14px;
  }
  .legend li {
    display: grid;
    grid-template-columns: 12px 1fr auto 56px;
    align-items: center;
    gap: 8px;
  }
  .legend li .num {
    text-align: right;
    font-size: 13px;
  }
  .legend.inline {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    font-size: 13px;
    color: var(--ink-2);
  }
  .legend.inline li {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .swatch {
    display: inline-block;
    width: 12px;
    height: 12px;
    border-radius: 3px;
    margin-right: 4px;
    vertical-align: -1px;
  }
  .legend li .swatch {
    margin-right: 0;
  }
  .swatch.line {
    height: 3px;
    border-radius: 2px;
  }
  .plot {
    position: relative;
  }
  svg {
    width: 100%;
    height: auto;
    display: block;
    touch-action: pan-y;
  }
  .grid {
    stroke: var(--line-soft);
  }
  .tick {
    font: 11px var(--mono);
    fill: var(--muted);
  }
  .cross {
    stroke: var(--field);
  }
  .line {
    fill: none;
    stroke-width: 2;
    stroke-linejoin: round;
    stroke-linecap: round;
  }
  .dot {
    stroke: var(--surface);
    stroke-width: 2;
  }
  .tip {
    position: absolute;
    top: 0;
    transform: translateX(-50%);
    display: grid;
    gap: 2px;
    padding: 6px 10px;
    font-size: 12px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 8px;
    box-shadow: 0 6px 16px rgb(17 24 33 / 0.14);
    pointer-events: none;
    white-space: nowrap;
  }
  .table summary {
    cursor: pointer;
    font-size: 13px;
    color: var(--muted);
  }
  table {
    margin-top: 8px;
    border-collapse: collapse;
    font-size: 13px;
  }
  th,
  td {
    padding: 3px 12px 3px 0;
    text-align: left;
    font-weight: 400;
  }
  td.num {
    text-align: right;
  }
  thead th {
    color: var(--muted);
  }
</style>
