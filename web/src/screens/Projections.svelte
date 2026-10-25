<script lang="ts">
  import Chart from '../components/Chart.svelte'
  import { project, retirement, type Market } from '../lib/core'
  import { money } from '../lib/format'
  import { i18n, t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const profile = $derived(s.plan.profile)
  const thisYear = new Date().getFullYear()
  const age = $derived(profile?.birth_year ? thisYear - profile.birth_year : null)

  const cash = $derived(
    s.trial_balance.filter((r) => r.account.startsWith('Assets:Cash:')).reduce((sum, r) => sum + Number(r.debit) - Number(r.credit), 0),
  )
  // Market value where a price is known, otherwise what was paid.
  const invested = $derived(s.valuations.reduce((sum, v) => sum + Number(v.value ?? v.cost), 0))
  const start = $derived(cash + invested)

  let market = $state<Market>('base')
  let years = $state(20)
  let monthlyOverride = $state<number | undefined>()
  const monthly = $derived(monthlyOverride ?? Number(profile?.monthly_investing ?? 0))
  const risk = $derived(profile?.risk ?? 'balanced')

  const projection = $derived(project({ start, monthly, years, market, risk }))
  const end = $derived(projection.bands[projection.bands.length - 1])
  const milestones = $derived(projection.bands.filter((b) => b.year > 0 && (b.year % 5 === 0 || b.year === years)))

  // Retirement
  let retireOverride = $state<number | undefined>()
  let spendingOverride = $state<number | undefined>()
  let pensionOverride = $state<number | undefined>()
  let planUntil = $state(95)
  const retireAt = $derived(retireOverride ?? profile?.retirement_age ?? null)
  // Without a retirement budget, today's spending is the starting point.
  const spending = $derived(spendingOverride ?? Number(profile?.retirement_spending ?? profile?.monthly_spending ?? 0))
  const pension = $derived(pensionOverride ?? Number(profile?.monthly_pension ?? 0))
  const pensionAge = $derived(profile?.pension_age ?? 65)

  const plan = $derived(
    age !== null && retireAt !== null
      ? retirement({
          start,
          age_now: age,
          retire_at: Math.max(age, retireAt),
          plan_until: planUntil,
          monthly_investing: monthly,
          monthly_spending: spending,
          pension_from: pensionAge,
          monthly_pension: pension,
          market,
          risk,
        })
      : undefined,
  )
  const rows = $derived(
    plan?.years.filter((y, i) => i === 0 || (y.age - (retireAt ?? 0)) % 5 === 0 || y.age === pensionAge || y.age === plan.runs_out_at) ?? [],
  )

  const compact = (v: number) => new Intl.NumberFormat(i18n.lang, { notation: 'compact', maximumFractionDigits: 1 }).format(v)
  const percent = (v: number) => `${Math.round(v * 100)}%`
  const markets: Market[] = ['bear', 'base', 'bull']
</script>

<header>
  <h1>{t('nav.projections')}</h1>
  <p class="muted">{t('proj.subtitle')}</p>
</header>

<section class="card controls" aria-label={t('proj.assumptions')}>
  <div class="field">
    <span id="market-label">{t('proj.market')}</span>
    <div role="radiogroup" aria-labelledby="market-label" class="segmented">
      {#each markets as m (m)}
        <button role="radio" aria-checked={market === m} class:on={market === m} onclick={() => (market = m)}>
          {t(`market.${m}`)}
        </button>
      {/each}
    </div>
  </div>
  <label class="field">{t('proj.years')}<input type="number" min="1" max="60" bind:value={years} /></label>
  <label class="field">{t('proj.monthly', { currency: base })}
    <input type="number" min="0" step="50" value={monthly} oninput={(e) => (monthlyOverride = Number(e.currentTarget.value))} />
  </label>
  <p class="muted note">{t(`market.${market}Hint`)} {t('proj.today', { amount: money(start, base) })}</p>
</section>

<section class="card">
  <h2>{t('proj.growthTitle', { years })}</h2>
  <div class="legend muted">
    <span><i class="sw band"></i>{t('proj.range')}</span>
    <span><i class="sw accent"></i>{t('proj.median')}</span>
    <span><i class="sw dashed"></i>{t('proj.steady')}</span>
  </div>
  <Chart
    label={t('proj.chartLabel', { years, median: money(end.p50, base) })}
    band={projection.bands.map((b) => ({ x: thisYear + b.year, low: b.p10, high: b.p90 }))}
    lines={[
      { points: projection.bands.map((b) => [thisYear + b.year, b.p50]), tone: 'accent' },
      { points: projection.bands.map((b) => [thisYear + b.year, b.steady]), tone: 'muted', dashed: true },
    ]}
    xLabel={(x) => String(x)}
    yLabel={compact}
  />
  <div class="table-wrap">
    <table>
      <thead><tr><th>{t('proj.year')}</th><th class="right">{t('proj.bad')}</th><th class="right">{t('proj.median')}</th><th class="right">{t('proj.good')}</th></tr></thead>
      <tbody>
        {#each milestones as b (b.year)}
          <tr><td>{thisYear + b.year}</td><td class="right num">{money(b.p10, base)}</td><td class="right num">{money(b.p50, base)}</td><td class="right num">{money(b.p90, base)}</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>

<section class="card retirement" aria-labelledby="ret-title">
  <h2 id="ret-title">{t('ret.title')}</h2>
  {#if age === null || retireAt === null}
    <p class="notice">{t('ret.needProfile')}</p>
    <button class="btn" onclick={() => (app.view = 'goals')}>{t('ret.setProfile')}</button>
  {:else if plan}
    <div class="ret-inputs">
      <label class="field">{t('ret.retireAt')}<input type="number" min={age} max="80" value={retireAt} oninput={(e) => (retireOverride = Number(e.currentTarget.value))} /></label>
      <label class="field">{t('ret.spending', { currency: base })}<input type="number" min="0" step="50" value={spending} oninput={(e) => (spendingOverride = Number(e.currentTarget.value))} /></label>
      <label class="field">{t('ret.pension', { age: pensionAge })}<input type="number" min="0" step="50" value={pension} oninput={(e) => (pensionOverride = Number(e.currentTarget.value))} /></label>
      <label class="field">{t('ret.planUntil')}<input type="number" min="70" max="110" bind:value={planUntil} /></label>
    </div>

    <div class="kpis">
      <div class="kpi"><span class="label">{t('ret.atRetirement')}</span><span class="value num">{money(plan.at_retirement, base)}</span><span class="hint">{thisYear + retireAt - age}</span></div>
      <div class="kpi"><span class="label">{t('ret.lasts')}</span><span class="value num">{plan.runs_out_at === null ? t('ret.past', { age: planUntil }) : t('ret.age', { age: plan.runs_out_at })}</span><span class="hint">{t('ret.lastsHint')}</span></div>
      <div class="kpi"><span class="label">{t('ret.chance', { age: planUntil })}</span><span class="value num" class:bad={plan.chance_lasts < 0.7}>{percent(plan.chance_lasts)}</span><span class="hint">{t('ret.chanceHint')}</span></div>
      <div class="kpi"><span class="label">{t('ret.safe')}</span><span class="value num">{money(plan.safe_monthly_spending, base)}</span><span class="hint">{t('ret.safeHint')}</span></div>
    </div>

    <Chart
      label={t('ret.chartLabel', { age: retireAt })}
      lines={[{ points: plan.path, tone: 'warn' }]}
      markers={[{ x: retireAt, tone: 'accent' }, { x: pensionAge, tone: 'muted' }]}
      xLabel={(x) => String(x)}
      yLabel={compact}
    />
    <p class="muted small">{t('ret.chartNote')}</p>

    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>{t('ret.ageCol')}</th><th>{t('proj.year')}</th><th class="right">{t('ret.start')}</th><th class="right">{t('ret.growth')}</th>
            <th class="right">{t('ret.withdrawn')}</th><th class="right">{t('ret.end')}</th><th class="right">{t('ret.fromPortfolio')}</th>
            <th class="right">{t('ret.pensionCol')}</th><th class="right">{t('ret.income')}</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as y (y.age)}
            <tr class:short={!y.covered}>
              <td class="num">{y.age}</td><td class="num">{thisYear + y.age - age}</td>
              <td class="right num">{money(y.start, base)}</td><td class="right num">+{money(y.growth, base)}</td>
              <td class="right num">−{money(y.withdrawn, base)}</td><td class="right num"><strong>{money(y.end, base)}</strong></td>
              <td class="right num">{money(y.from_portfolio_monthly, base)}</td><td class="right num">{money(y.pension_monthly, base)}</td>
              <td class="right num"><strong>{money(y.from_portfolio_monthly + y.pension_monthly, base)}</strong></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <p class="muted small">{t('ret.tableNote')}</p>
  {/if}
</section>

<p class="muted small">{t('proj.disclaimer')}</p>

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
    font-size: 18px;
  }
  p {
    margin: 0;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    align-items: flex-end;
  }
  .controls .field {
    width: auto;
    min-width: 150px;
  }
  .note {
    flex-basis: 100%;
    font-size: 13px;
  }
  .segmented {
    display: flex;
    gap: 4px;
    background: var(--line-soft);
    border-radius: 10px;
    padding: 4px;
  }
  .segmented button {
    min-height: 40px;
    padding: 0 14px;
    border: 0;
    border-radius: 8px;
    background: none;
    color: var(--ink-2);
    font: 500 14px var(--sans);
    cursor: pointer;
  }
  .segmented button.on {
    background: var(--surface);
    color: var(--ink);
    font-weight: 600;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    font-size: 13px;
    margin: 8px 0;
  }
  .sw {
    display: inline-block;
    width: 18px;
    height: 10px;
    margin-right: 6px;
    vertical-align: middle;
  }
  .sw.band {
    background: var(--accent-tint);
  }
  .sw.accent {
    height: 3px;
    background: var(--accent);
  }
  .sw.dashed {
    height: 0;
    border-top: 2px dashed var(--muted);
  }
  .retirement {
    display: flex;
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }
  .retirement > .btn {
    align-self: flex-start;
  }
  .ret-inputs {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
    gap: 12px;
  }
  .kpis {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
    gap: 12px;
  }
  .kpi {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 14px;
    border: 1px solid var(--line-soft);
    border-radius: 10px;
  }
  .label,
  .hint {
    font-size: 12px;
    color: var(--muted);
  }
  .value {
    font-size: 22px;
    font-weight: 500;
  }
  .value.bad {
    color: var(--warn);
  }
  tr.short td {
    background: var(--warn-soft);
    color: var(--warn-ink);
  }
  .small {
    font-size: 13px;
    line-height: 1.5;
  }
</style>
