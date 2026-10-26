<script lang="ts">
  import { describe } from '../lib/describe'
  import { packs } from '../lib/configs'
  import type { Declaration, Gains, Metal, Realization, Treatment } from '../lib/core'
  import { country, date, money } from '../lib/format'
  import { i18n, local, t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
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

  const filing = $derived.by((): Declaration | undefined => {
    if (!pack || !app.book) return undefined
    void app.snapshot
    try {
      return app.book.declaration(pack.year, pack.toml, i18n.lang)
    } catch {
      return undefined
    }
  })

  let copied = $state('')
  async function copy(text: string): Promise<void> {
    await navigator.clipboard.writeText(text)
    copied = text
    setTimeout(() => (copied = ''), 1500)
  }

  const unclassified = $derived.by((): Realization[] => {
    void app.snapshot
    return !pack && app.book && selectedYear ? app.book.sales(selectedYear) : []
  })

  const labels: Record<Treatment, () => string> = {
    exempt: () => t('tax.exempt'),
    taxable: () => t('tax.taxable'),
    venue_unknown: () => t('tax.venueUnknown'),
    not_estimated: () => t('tax.notEstimated'),
  }
  const metalNames: Record<string, Metal> = { XAU: 'gold', XAG: 'silver', XPT: 'platinum', XPD: 'palladium' }
  const privateNames = $derived(new Map(s.private.map((p) => [p.id, p.name])))
  const what = (r: Realization) => (r.physical && metalNames[r.symbol] ? t(`metal.${metalNames[r.symbol]}` as const) : r.private ? (privateNames.get(r.symbol) ?? r.symbol) : r.symbol)
</script>

<header>
  <h1>{t('nav.tax')}</h1>
  <p class="muted">{t('tax.subtitle')}</p>
</header>

{#if years.length === 0}
  <p class="notice">{t('tax.noPack', { country: country(s.settings.tax_residency) })}</p>
  <button class="btn ghost" onclick={() => app.openGuide('add-tax')}>{t('tax.howToAdd')}</button>
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
    <p class="notice">{t('tax.noPackForYear', { year: selectedYear, country: country(s.settings.tax_residency) })}</p>
    <button class="btn ghost" onclick={() => app.openGuide('add-tax')}>{t('tax.howToAdd')}</button>
    <section class="card">
      <h2>{t('tax.sales', { year: selectedYear })}</h2>
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th>{t('common.date')}</th><th>{t('common.instrument')}</th><th class="right">{t('tax.proceeds')}</th><th class="right">{t('tax.cost')}</th><th class="right">{t('tax.gain')}</th><th>{t('tax.venue')}</th></tr>
          </thead>
          <tbody>
            {#each unclassified as r, i (r.entry_id + ':' + i)}
              <tr>
                <td>{date(r.date)}</td>
                <td class="num"><strong>{what(r)}</strong>{#if r.physical && r.held_from}<span class="muted block">{t('tax.boughtOn', { date: date(r.held_from) })}</span>{/if}</td>
                <td class="right num">{money(r.proceeds)}</td>
                <td class="right num">{money(r.cost)}</td>
                <td class="right num">{money(r.gain)}</td>
                <td class="num">{r.private ? t('tax.privateStake') : r.physical ? t('tax.physical') : r.venue?.code ?? '—'}</td>
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
    {#if result.pack.notes}<p class="muted pack-notes">{local(result.pack.notes)}</p>{/if}
    {#if result.pack.currency !== base}<p class="notice">{t('tax.currencyNote', { base, currency: result.pack.currency })}</p>{/if}
    {#if result.metals?.note && result.rows.some((r) => r.physical)}<p class="muted pack-notes">{t('tax.metalsNote', { note: local(result.metals.note) })}</p>{/if}

    <section class="summary" aria-label={t('tax.summary')}>
      <div class="card kpi">
        <span class="label">{t('tax.taxableNet')}</span>
        <span class="value num">{money(Number(result.summary.taxable_gains) + Number(result.summary.taxable_losses), base)}</span>
      </div>
      <div class="card kpi">
        <span class="label">{result.banded ? t('tax.estimatedBands') : t('tax.estimated', { rate: +(Number(result.rate) * 100).toFixed(3) })}</span>
        <span class="value num">{money(result.summary.tax_due, base)}</span>
        <span class="hint">{#if result.inclusion}{t('tax.inclusionHint', { share: +(Number(result.inclusion) * 100).toFixed(1), rate: +(Number(result.marginal) * 100).toFixed(3) })}{' · '}{/if}{result.form}</span>
      </div>
      {#if result.allowance}
        <div class="card kpi">
          <span class="label">{t('tax.allowance')}</span>
          <span class="value num">{money(result.summary.allowance_used, base)}</span>
          <span class="hint">{t('tax.allowanceOf', { amount: money(result.allowance, result.pack.currency) })}</span>
        </div>
      {/if}
      <div class="card kpi">
        <span class="label">{t('tax.exemptTotal')}</span>
        <span class="value num">{money(result.summary.exempt, base)}</span>
      </div>
      {#if Number(result.summary.unknown) !== 0 || result.rows.some((r) => r.treatment === 'venue_unknown')}
        <div class="card kpi warn">
          <span class="label">{t('tax.undecided')}</span>
          <span class="value num">{money(result.summary.unknown, base)}</span>
          <span class="hint">{t('tax.undecidedHint')}</span>
        </div>
      {/if}
      {#if result.rows.some((r) => r.treatment === 'not_estimated')}
        <div class="card kpi warn">
          <span class="label">{t('tax.notEstimatedTotal')}</span>
          <span class="value num">{money(result.summary.not_estimated, base)}</span>
          <span class="hint">{t('tax.notEstimatedHint')}</span>
        </div>
      {/if}
    </section>

    {#if filing?.sections.length}
      <section class="card declaration" aria-labelledby="decl-title">
        <h2 id="decl-title">{t('decl.title', { year: filing.year })}</h2>
        <p class="muted">{t('decl.lead', { currency: result.pack.currency })}</p>
        {#if filing.undecided}<p class="notice">{t('decl.undecided', { count: filing.undecided })}</p>{/if}
        {#if filing.steps.length}
          <h3>{t('decl.steps')}</h3>
          <ol class="steps">
            {#each filing.steps as step, i (i)}<li>{step}</li>{/each}
          </ol>
          {#if filing.portal}<a class="btn ghost" href={filing.portal} target="_blank" rel="noopener noreferrer">{t('decl.portal')}</a>{/if}
        {/if}
        {#each filing.sections as section, i (i)}
          <article class="form-part">
            <div class="part-head">
              <h3>{section.form}{#if section.code}<span class="pill plain">{t('decl.code', { code: section.code })}</span>{/if}{#if section.optional}<span class="pill">{t('decl.optional')}</span>{/if}</h3>
              <p class="muted small">{section.title}</p>
            </div>
            {#if section.rows.length}
              <div class="table-wrap">
                <table>
                  <thead><tr>{#each section.columns as [label] (label)}<th class:right={section.rows[0][section.columns.findIndex(([l]) => l === label)]?.amount}>{label}</th>{/each}</tr></thead>
                  <tbody>
                    {#each section.rows as row, r (r)}
                      <tr>
                        {#each row as cell, c (c)}
                          <td class:right={cell.amount} class:num={cell.amount}>
                            {#if cell.amount}
                              <button class="copy" onclick={() => copy(cell.text)} title={t('decl.copy')} aria-label="{t('decl.copy')} {cell.text}">
                                {copied === cell.text ? t('decl.copied') : cell.text}
                              </button>
                            {:else if section.columns[c]?.[1] === 'description'}{describe(cell.text)}{:else}{cell.text}{/if}
                          </td>
                        {/each}
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {:else}
              <p class="muted small">{t('decl.nothing', { year: filing.year })}</p>
            {/if}
            {#if section.note}<p class="muted small">{local(section.note)}</p>{/if}
          </article>
        {/each}
      </section>
    {/if}

    {#if result.employment.length}
      {@const rule = result.employment_rule}
      {@const income = result.employment.reduce((sum, r) => sum + Number(r.income), 0)}
      {@const withheld = result.employment.reduce((sum, r) => sum + Number(r.withheld), 0)}
      <section class="card">
        <h2>{t('tax.employmentTitle')}</h2>
        <div class="table-wrap">
          <table>
            <thead>
              <tr><th>{t('common.date')}</th><th></th><th class="right">{t('tax.employmentIncome')}</th><th class="right">{t('tax.employmentWithheld')}</th></tr>
            </thead>
            <tbody>
              {#each result.employment as r (r.entry_id)}
                <tr><td>{date(r.date)}</td><td>{describe(r.description)}</td><td class="right num">{money(r.income)}</td><td class="right num">{money(r.withheld)}</td></tr>
              {/each}
            </tbody>
          </table>
        </div>
        <p class="small">
          {#if rule?.rate}{t('tax.employmentEstimate', { estimate: money(Math.max(0, income) * Number(rule.rate), base), rate: +(Number(rule.rate) * 100).toFixed(3), form: rule.form, withheld: money(withheld, base) })}
          {:else}{t('tax.employmentNoRule')}{/if}
        </p>
        {#if rule?.note}<p class="muted small">{local(rule.note)}</p>{/if}
      </section>
    {/if}

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
            {#each result.rows as r, i (r.entry_id + ':' + i)}
              <tr>
                <td>{date(r.date)}</td>
                <td class="num"><strong>{what(r)}</strong>{#if r.physical && r.held_from}<span class="muted block">{t('tax.boughtOn', { date: date(r.held_from) })}</span>{/if}</td>
                <td class="right num">{money(r.proceeds)}</td>
                <td class="right num">{money(r.cost)}</td>
                <td class="right num">{money(r.gain)}</td>
                <td class="num">{r.private ? t('tax.privateStake') : r.physical ? t('tax.physical') : r.venue ? r.venue.code + (r.venue.assumed ? '*' : '') : '—'}</td>
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
  .block {
    display: block;
    font-size: 12px;
  }
  .pack-notes {
    margin: 0;
    font-size: 14px;
    line-height: 1.55;
    max-width: 820px;
  }
  .declaration {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .declaration h2,
  .declaration p,
  .declaration h3 {
    margin: 0;
  }
  .steps {
    margin: 0;
    padding-left: 22px;
    display: grid;
    gap: 6px;
    line-height: 1.5;
  }
  .declaration > .btn {
    align-self: flex-start;
  }
  .form-part {
    border-top: 1px solid var(--line-soft);
    padding-top: 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .part-head h3 {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    font-size: 16px;
  }
  .copy {
    border: 1px dashed transparent;
    border-radius: 6px;
    padding: 4px 6px;
    background: none;
    color: inherit;
    font: inherit;
    cursor: copy;
  }
  .copy:hover {
    border-color: var(--line);
    background: var(--accent-soft);
  }
</style>
