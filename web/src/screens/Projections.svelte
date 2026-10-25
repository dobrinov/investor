<script lang="ts">
  import Chart from '../components/Chart.svelte'
  import Help from '../components/Help.svelte'
  import WhatIfMoves from '../components/WhatIfMoves.svelte'
  import type { Market, Move } from '../lib/core'
  import { whole } from '../lib/format'
  import { i18n, t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const profile = $derived(s.plan.profile)
  const now = new Date()
  const thisYear = now.getFullYear()
  const today = `${thisYear}-${String(now.getMonth() + 1).padStart(2, '0')}`
  const age = $derived(profile?.birth_year ? thisYear - profile.birth_year : null)

  const cash = $derived(
    s.trial_balance.filter((r) => r.account.startsWith('Assets:Cash:')).reduce((sum, r) => sum + Number(r.debit) - Number(r.credit), 0),
  )
  // Market value where a price is known, otherwise what was paid.
  const invested = $derived(s.valuations.reduce((sum, v) => sum + Number(v.value ?? v.cost), 0))

  let market = $state<Market>('base')
  let years = $state(20)
  let spendGoals = $state(true)

  // Retirement: the same plan and moves, until the money runs out or the plan ends.
  let retireOverride = $state<number | undefined>()
  let spendingOverride = $state<number | undefined>()
  let pensionOverride = $state<number | undefined>()
  let planUntil = $state(95)
  const retireAt = $derived(retireOverride ?? profile?.retirement_age ?? null)
  // Without a retirement budget, today's spending is the starting point.
  const spending = $derived(spendingOverride ?? Number(profile?.retirement_spending ?? profile?.monthly_spending ?? 0))
  const pension = $derived(pensionOverride ?? Number(profile?.monthly_pension ?? 0))
  const pensionAge = $derived(profile?.pension_age ?? 65)
  const retireInput = $derived(
    age !== null && retireAt !== null
      ? {
          age_now: age,
          retire_at: Math.max(age, retireAt),
          plan_until: Math.max(age + 1, Math.min(110, planUntil || 95)),
          monthly_spending: spending || 0,
          pension_from: pensionAge,
          monthly_pension: pension || 0,
        }
      : undefined,
  )

  // Recomputed whenever the book changes (moves are saved in the plan).
  const result = $derived.by(() => {
    void app.snapshot
    return app.book!.whatIf({ today, years: Math.max(1, Math.min(60, years || 1)), market, spend_goals: spendGoals, retirement: retireInput })
  })
  const shown = $derived(result.with_moves ?? result.plan)
  const end = $derived(shown.bands[shown.bands.length - 1])
  const planEnd = $derived(result.plan.bands[result.plan.bands.length - 1])
  const endYear = $derived(thisYear + end.year)
  const milestones = $derived(
    shown.bands.map((b, i) => ({ b, plan: result.plan.bands[i] })).filter(({ b }) => b.year > 0 && (b.year % 5 === 0 || b.year === end.year)),
  )
  const moveName = (id: string) => {
    const m = s.plan.what_if?.find((x) => x.id === id)
    return m ? m.name || t(`proj.kind.${m.kind}`) : ''
  }

  async function saveMoves(moves: Move[]): Promise<void> {
    await app.setPlan({ ...s.plan, what_if: moves })
  }

  const planRet = $derived(result.retirement?.plan)
  const plan = $derived(result.retirement?.with_moves ?? planRet)
  const rows = $derived(
    plan?.years.filter((y, i) => i === 0 || (y.age - (retireAt ?? 0)) % 5 === 0 || y.age === pensionAge || y.age === plan.runs_out_at) ?? [],
  )

  const money = (v: number) => whole(v, base)
  const lasts = (r: { runs_out_at: number | null }) => (r.runs_out_at === null ? t('ret.past', { age: planUntil }) : t('ret.age', { age: r.runs_out_at }))
  const signed = (v: number) => (v >= 0 ? '+' : '−') + whole(Math.abs(v), base)
  const compact = (v: number) => new Intl.NumberFormat(i18n.lang, { notation: 'compact', maximumFractionDigits: 1 }).format(v)
  const percent = (v: number) => `${Math.round(v * 100)}%`
  const markets: Market[] = ['bear', 'base', 'bull']
</script>

<header>
  <h1>{t('nav.projections')}</h1>
  <p class="muted">{t('proj.subtitle')}</p>
</header>

<section class="card start" aria-labelledby="start-title">
  <h2 id="start-title">{t('proj.start')}</h2>
  <div class="kpis">
    <div class="kpi"><span class="label">{t('proj.invested')}</span><span class="value num">{money(invested)}</span><span class="hint">{t('proj.investedHint')}</span></div>
    <div class="kpi"><span class="label">{t('proj.cash')}</span><span class="value num">{money(cash)}</span><span class="hint">{t('proj.cashHint')}</span></div>
    <div class="kpi">
      <span class="label">{t('proj.planMonthly')}</span>
      <span class="value num">{t('proj.perMonth', { amount: money(result.monthly) })}</span>
      <span class="hint">{t('proj.planMonthlyHint')}</span>
    </div>
  </div>
  {#if profile?.monthly_investing == null}
    <p class="notice">{t('proj.noMonthly')}<button class="link" onclick={() => (app.view = 'goals')}>{t('proj.editProfile')}</button></p>
  {/if}
  <div class="goals-out">
    <span class="with-help">
      <label class="choice"><input type="checkbox" bind:checked={spendGoals} />{t('proj.spendGoals')}</label>
      <Help about={t('proj.spendGoals')} why={t('proj.spendGoalsWhy')} without={t('proj.spendGoalsWithout')} />
    </span>
    {#if result.spending.length}
      <ul>
        {#each result.spending as g (g.goal)}
          <li class="muted">{t('proj.goalOut', { name: g.name, amount: money(g.amount), year: g.year })}</li>
        {/each}
      </ul>
    {:else if spendGoals}
      <p class="muted small">{t('proj.noGoals')}</p>
    {/if}
  </div>
</section>

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
  <p class="muted note">{t(`market.${market}Hint`)}</p>
</section>

<section class="card whatif" aria-labelledby="whatif-title">
  <h2 id="whatif-title">{t('proj.whatIf')}</h2>
  <p class="muted small">{t('proj.whatIfText')}</p>
  <WhatIfMoves plan={s.plan} {base} onsave={saveMoves} />
</section>

<section class="card result">
  <h2>{t('proj.growthTitle', { years: end.year })}</h2>
  {#each result.cash_short as c (c.id)}
    <p class="notice">{t('proj.cashShort', { name: moveName(c.id), wanted: money(c.wanted), available: money(c.available) })}</p>
  {/each}

  <div class="kpis">
    {#each [['proj.middleIn', end.p50, planEnd.p50], ['proj.badIn', end.p10, planEnd.p10], ['proj.goodIn', end.p90, planEnd.p90]] as const as [key, value, planValue] (key)}
      <div class="kpi">
        <span class="label">{t(key, { year: endYear })}</span>
        <span class="value num">{money(value)}</span>
        {#if result.with_moves}
          <span class="hint">{t('proj.vsPlan', { amount: money(planValue) })} · <strong class:up={value >= planValue} class:down={value < planValue}>{signed(value - planValue)}</strong></span>
        {/if}
      </div>
    {/each}
    <div class="kpi">
      <span class="label">{t('proj.putIn')}</span>
      <span class="value num">{money(end.put_in)}</span>
      {#if result.with_moves}<span class="hint">{t('proj.vsPlan', { amount: money(planEnd.put_in) })}</span>{/if}
    </div>
    {#if shown.chance_covered !== null || result.plan.chance_covered !== null}
      <div class="kpi">
        <span class="label">{t('proj.covered')}</span>
        <span class="value num" class:bad={(shown.chance_covered ?? 1) < 0.8}>{percent(shown.chance_covered ?? 1)}</span>
        <span class="hint">
          {t('proj.coveredHint')}{#if result.with_moves && result.plan.chance_covered !== null}{' · '}{t('proj.vsPlan', { amount: percent(result.plan.chance_covered) })}{/if}
        </span>
      </div>
    {/if}
  </div>

  <div class="legend muted">
    <span><i class="sw band"></i>{t('proj.range')}</span>
    <span><i class="sw accent"></i>{result.with_moves ? t('proj.withMoves') : t('proj.median')}</span>
    {#if result.with_moves}<span><i class="sw dashed"></i>{t('proj.planMedian')}</span>{/if}
  </div>
  <Chart
    label={t('proj.chartLabel', { years: end.year, median: money(end.p50) })}
    band={shown.bands.map((b) => ({ x: thisYear + b.year, low: b.p10, high: b.p90 }))}
    lines={[
      { points: shown.bands.map((b) => [thisYear + b.year, b.p50]), tone: 'accent' },
      ...(result.with_moves ? [{ points: result.plan.bands.map((b) => [thisYear + b.year, b.p50] as [number, number]), tone: 'muted' as const, dashed: true }] : []),
    ]}
    xLabel={(x) => String(x)}
    yLabel={compact}
  />
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>{t('proj.year')}</th>
          {#if result.with_moves}
            <th class="right">{t('proj.planLine')}</th><th class="right">{t('proj.withMoves')}</th><th class="right">{t('proj.difference')}</th>
          {:else}
            <th class="right">{t('proj.median')}</th>
          {/if}
          <th class="right">{t('proj.bad')}</th><th class="right">{t('proj.good')}</th><th class="right">{t('proj.putIn')}</th>
        </tr>
      </thead>
      <tbody>
        {#each milestones as { b, plan: p } (b.year)}
          <tr>
            <td>{thisYear + b.year}</td>
            {#if result.with_moves}
              <td class="right num">{money(p.p50)}</td><td class="right num"><strong>{money(b.p50)}</strong></td>
              <td class="right num" class:up={b.p50 >= p.p50} class:down={b.p50 < p.p50}>{signed(b.p50 - p.p50)}</td>
            {:else}
              <td class="right num"><strong>{money(b.p50)}</strong></td>
            {/if}
            <td class="right num">{money(b.p10)}</td><td class="right num">{money(b.p90)}</td><td class="right num">{money(b.put_in)}</td>
          </tr>
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

    {#if result.retirement?.with_moves}<p class="muted small">{t('ret.withMovesNote')}</p>{/if}
    <div class="kpis">
      <div class="kpi">
        <span class="label">{t('ret.atRetirement')}</span><span class="value num">{money(plan.at_retirement)}</span>
        <span class="hint">{thisYear + retireAt - age}{#if planRet && plan !== planRet}{' · '}{t('proj.vsPlan', { amount: money(planRet.at_retirement) })}{/if}</span>
      </div>
      <div class="kpi">
        <span class="label">{t('ret.lasts')}</span><span class="value num">{lasts(plan)}</span>
        <span class="hint">{t('ret.lastsHint')}{#if planRet && plan !== planRet}{' · '}{t('proj.vsPlan', { amount: lasts(planRet) })}{/if}</span>
      </div>
      <div class="kpi">
        <span class="label">{t('ret.chance', { age: planUntil })}</span><span class="value num" class:bad={plan.chance_lasts < 0.7}>{percent(plan.chance_lasts)}</span>
        <span class="hint">{t('ret.chanceHint')}{#if planRet && plan !== planRet}{' · '}{t('proj.vsPlan', { amount: percent(planRet.chance_lasts) })}{/if}</span>
      </div>
      <div class="kpi">
        <span class="label">{t('ret.safe')}</span><span class="value num">{money(plan.safe_monthly_spending)}</span>
        <span class="hint">{t('ret.safeHint')}{#if planRet && plan !== planRet}{' · '}{t('proj.vsPlan', { amount: money(planRet.safe_monthly_spending) })}{/if}</span>
      </div>
    </div>

    <Chart
      label={t('ret.chartLabel', { age: retireAt })}
      lines={[
        { points: plan.path, tone: 'warn' },
        ...(planRet && plan !== planRet ? [{ points: planRet.path, tone: 'muted' as const, dashed: true }] : []),
      ]}
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
              <td class="right num">{money(y.start)}</td><td class="right num">+{money(y.growth)}</td>
              <td class="right num">−{money(y.withdrawn)}</td><td class="right num"><strong>{money(y.end)}</strong></td>
              <td class="right num">{money(y.from_portfolio_monthly)}</td><td class="right num">{money(y.pension_monthly)}</td>
              <td class="right num"><strong>{money(y.from_portfolio_monthly + y.pension_monthly)}</strong></td>
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
  .start,
  .whatif,
  .result {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .goals-out ul {
    margin: 6px 0 0;
    padding-left: 22px;
    font-size: 14px;
  }
  .with-help {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .choice {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    min-height: 32px;
  }
  .choice input {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }
  .link {
    border: 0;
    background: none;
    padding: 0;
    color: var(--accent);
    font: 600 14px var(--sans);
    cursor: pointer;
    text-decoration: underline;
  }
  .up {
    color: var(--accent-strong);
  }
  .down {
    color: var(--warn);
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
