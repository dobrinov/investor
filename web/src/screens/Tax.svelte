<script lang="ts">
  import { packs } from '../lib/configs'
  import type { Gains, Realization, Treatment } from '../lib/core'
  import { date, money } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const available = $derived(packs.filter((p) => p.country === s.settings.tax_residency))
  const saleYears = $derived(
    s.entries.filter((e) => e.postings.some((p) => p.account === 'Income:CapitalGains')).map((e) => Number(e.date.slice(0, 4))),
  )
  const years = $derived([...new Set([...available.map((p) => p.year), ...saleYears])].sort((a, b) => b - a))
  let year = $state<number | undefined>()
  const selectedYear = $derived(year ?? years[0])
  const pack = $derived(available.find((p) => p.year === selectedYear))

  const result = $derived.by((): Gains | { error: string } | undefined => {
    if (!pack || !app.book) return undefined
    void app.snapshot // recompute after imports
    try {
      return app.book.gains(pack.year, pack.toml)
    } catch (e) {
      return { error: e instanceof Error ? e.message : String(e) }
    }
  })

  const unclassified = $derived.by((): Realization[] => {
    void app.snapshot
    return !pack && app.book && selectedYear ? app.book.sales(selectedYear) : []
  })

  const labels: Record<Treatment, () => string> = {
    exempt: () => t('tax.exempt'),
    taxable: () => t('tax.taxable'),
    venue_unknown: () => t('tax.venueUnknown'),
  }
</script>

<header>
  <h1>{t('nav.tax')}</h1>
  <p class="muted">{t('tax.subtitle')}</p>
</header>

{#if years.length === 0}
  <p class="notice">{t('tax.noPack', { country: s.settings.tax_residency })}</p>
{:else}
  <div class="controls">
    <label class="field">{t('tax.year')}
      <select bind:value={() => selectedYear, (v) => (year = Number(v))}>
        {#each years as y (y)}<option value={y}>{y}</option>{/each}
      </select>
    </label>
    {#if pack}<span class="pill plain">{pack.authority}</span>{/if}
  </div>

  {#if !pack}
    <p class="notice">{t('tax.noPackForYear', { year: selectedYear, country: s.settings.tax_residency })}</p>
    <section class="card">
      <h2>{t('tax.sales', { year: selectedYear })}</h2>
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th>{t('common.date')}</th><th>{t('common.instrument')}</th><th class="right">{t('tax.proceeds')}</th><th class="right">{t('tax.cost')}</th><th class="right">{t('tax.gain')}</th><th>{t('tax.venue')}</th></tr>
          </thead>
          <tbody>
            {#each unclassified as r (r.entry_id)}
              <tr>
                <td>{date(r.date)}</td>
                <td class="num"><strong>{r.symbol}</strong></td>
                <td class="right num">{money(r.proceeds)}</td>
                <td class="right num">{money(r.cost)}</td>
                <td class="right num">{money(r.gain)}</td>
                <td class="num">{r.venue?.code ?? '—'}</td>
              </tr>
            {:else}
              <tr><td colspan="6" class="muted">{t('tax.noSales')}</td></tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {/if}

  {#if result && 'error' in result}
    <p class="error" role="alert">{result.error}</p>
  {:else if result}
    {#if !result.pack.verified}
      <p class="notice">{t('tax.draftPack', { id: result.pack.id, version: result.pack.version })}</p>
    {/if}

    <section class="summary" aria-label={t('tax.summary')}>
      <div class="card kpi">
        <span class="label">{t('tax.taxableNet')}</span>
        <span class="value num">{money(Number(result.summary.taxable_gains) + Number(result.summary.taxable_losses), result.pack.currency)}</span>
      </div>
      <div class="card kpi">
        <span class="label">{t('tax.estimated', { rate: Number(result.rate) * 100 })}</span>
        <span class="value num">{money(result.summary.tax_due, result.pack.currency)}</span>
        <span class="hint">{result.form}</span>
      </div>
      <div class="card kpi">
        <span class="label">{t('tax.exemptTotal')}</span>
        <span class="value num">{money(result.summary.exempt, result.pack.currency)}</span>
      </div>
      {#if Number(result.summary.unknown) !== 0 || result.rows.some((r) => r.treatment === 'venue_unknown')}
        <div class="card kpi warn">
          <span class="label">{t('tax.undecided')}</span>
          <span class="value num">{money(result.summary.unknown, result.pack.currency)}</span>
          <span class="hint">{t('tax.undecidedHint')}</span>
        </div>
      {/if}
    </section>

    <section class="card">
      <h2>{t('tax.sales', { year: selectedYear })}</h2>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>{t('common.date')}</th><th>{t('common.instrument')}</th><th class="right">{t('tax.proceeds')}</th><th class="right">{t('tax.cost')}</th>
              <th class="right">{t('tax.gain')}</th><th>{t('tax.venue')}</th><th>{t('tax.treatment')}</th>
            </tr>
          </thead>
          <tbody>
            {#each result.rows as r (r.entry_id)}
              <tr>
                <td>{date(r.date)}</td>
                <td class="num"><strong>{r.symbol}</strong></td>
                <td class="right num">{money(r.proceeds)}</td>
                <td class="right num">{money(r.cost)}</td>
                <td class="right num">{money(r.gain)}</td>
                <td class="num">{r.venue ? r.venue.code + (r.venue.assumed ? '*' : '') : '—'}</td>
                <td><span class="pill" class:warn={r.treatment !== 'exempt'} class:plain={r.treatment === 'taxable'}>{labels[r.treatment]()}</span></td>
              </tr>
            {:else}
              <tr><td colspan="7" class="muted">{t('tax.noSales')}</td></tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {/if}
{/if}

<style>
  header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  h1 {
    font-size: 28px;
  }
  h2 {
    font-size: 16px;
    margin-bottom: 8px;
  }
  header p {
    margin: 0;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    align-items: flex-end;
  }
  .controls .field {
    width: 140px;
  }
  .summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
  }
  .kpi {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .kpi.warn {
    border-color: var(--warn-line);
    background: var(--warn-soft);
  }
  .label,
  .hint {
    font-size: 13px;
    color: var(--muted);
  }
  .value {
    font-size: 24px;
    font-weight: 500;
  }
</style>
