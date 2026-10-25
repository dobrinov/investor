<script lang="ts">
  import type { GoalStatus, Plan } from '../lib/core'
  import { money } from '../lib/format'
  import { i18n, t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  let { plan, base, cash, planned }: { plan: Plan; base: string; cash: number; planned: Map<string, GoalStatus> } = $props()

  const WINDOW = 12
  const thisYear = new Date().getFullYear()

  function monthKey(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
  }

  const flows = $derived.by(() => {
    void app.snapshot
    return app.book?.flows() ?? []
  })

  // The last twelve months, or fewer if the book is younger.
  const months = $derived.by(() => {
    const now = new Date()
    const keys: string[] = []
    for (let i = WINDOW - 1; i >= 0; i--) keys.push(monthKey(new Date(now.getFullYear(), now.getMonth() - i, 1)))
    const first = flows[0]?.month
    return first ? keys.filter((k) => k >= first) : []
  })
  const byMonth = $derived(new Map(flows.map((f) => [f.month, f])))
  const monthlyPlan = $derived(Number(plan.profile?.monthly_investing ?? 0))
  const hasPlan = $derived(plan.profile?.monthly_investing != null)
  const rows = $derived(
    months.map((m) => {
      const f = byMonth.get(m)
      const put = Number(f?.deposited ?? 0) - Number(f?.withdrawn ?? 0)
      const invested = Number(f?.bought ?? 0) - Number(f?.sold ?? 0)
      return { month: m, put, invested }
    }),
  )
  const plannedTotal = $derived(monthlyPlan * rows.length)
  const putTotal = $derived(rows.reduce((s, r) => s + r.put, 0))
  const investedTotal = $derived(rows.reduce((s, r) => s + r.invested, 0))
  const pace = $derived(plannedTotal > 0 ? Math.max(0, Math.min(2, putTotal / plannedTotal)) : 1)

  const atPace = $derived.by(() => {
    void app.snapshot
    return new Map((app.book?.goals(thisYear, 'base', pace) ?? []).map((g) => [g.id, g]))
  })
  const savings = $derived(plan.goals.filter((g) => g.kind === 'savings'))

  const scale = $derived(Math.max(1, monthlyPlan, ...rows.map((r) => Math.max(r.put, r.invested))))
  const label = (m: string) => new Intl.DateTimeFormat(i18n.lang, { month: 'short' }).format(new Date(`${m}-01T00:00:00`))
  const percent = (v: number | null | undefined) => (v === null || v === undefined ? '—' : `${Math.round(v * 100)}%`)
</script>

<section class="card" aria-labelledby="pva-title">
  <h2 id="pva-title">{t('pva.title')}</h2>
  {#if !hasPlan}
    <p class="muted">{t('pva.needProfile')}</p>
  {:else if rows.length === 0}
    <p class="muted">{t('pva.noHistory')}</p>
  {:else}
    <p class="muted small">{t('pva.subtitle', { months: rows.length })}</p>

    <div class="kpis">
      <div class="kpi"><span class="label">{t('pva.planned')}</span><span class="value num">{money(plannedTotal, base)}</span><span class="hint">{t('pva.perMonth', { amount: money(monthlyPlan, base) })}</span></div>
      <div class="kpi"><span class="label">{t('pva.putIn')}</span><span class="value num" class:bad={pace < 0.9}>{money(putTotal, base)}</span><span class="hint">{t('pva.pace', { pct: percent(pace) })}</span></div>
      <div class="kpi"><span class="label">{t('pva.invested')}</span><span class="value num">{money(investedTotal, base)}</span><span class="hint">{t('pva.investedHint')}</span></div>
      <div class="kpi"><span class="label">{t('pva.idle')}</span><span class="value num">{money(cash, base)}</span><span class="hint">{t('pva.idleHint')}</span></div>
    </div>

    <div class="bars" role="img" aria-label={t('pva.chartLabel')}>
      {#each rows as r (r.month)}
        <div class="month">
          <div class="cols">
            <span class="bar plan" style:height="{(monthlyPlan / scale) * 100}%"></span>
            <span class="bar put" style:height="{(Math.max(0, r.put) / scale) * 100}%"></span>
            <span class="bar inv" style:height="{(Math.max(0, r.invested) / scale) * 100}%"></span>
          </div>
          <span class="m">{label(r.month)}</span>
        </div>
      {/each}
    </div>
    <div class="legend muted">
      <span><i class="sw plan"></i>{t('pva.legendPlan')}</span>
      <span><i class="sw put"></i>{t('pva.legendPut')}</span>
      <span><i class="sw inv"></i>{t('pva.legendInvested')}</span>
    </div>

    {#if pace < 0.9}
      <p class="notice">{t('pva.behindPlan', { pct: percent(pace) })}</p>
    {:else if pace > 1.1}
      <p class="notice good">{t('pva.aheadOfPlan', { pct: percent(pace) })}</p>
    {/if}
    {#if putTotal > 0 && investedTotal < putTotal * 0.8}
      <p class="notice">{t('pva.notInvested', { amount: money(putTotal - investedTotal, base) })}</p>
    {/if}

    {#if savings.length && Math.abs(pace - 1) > 0.01}
      <div class="table-wrap">
        <table>
          <thead><tr><th>{t('pva.goal')}</th><th class="right">{t('pva.chancePlanned')}</th><th class="right">{t('pva.chanceActual')}</th></tr></thead>
          <tbody>
            {#each savings as g (g.id)}
              <tr>
                <td>{g.name}</td>
                <td class="right num">{percent(planned.get(g.id)?.chance)}</td>
                <td class="right num">{percent(atPace.get(g.id)?.chance)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</section>

<style>
  h2 {
    font-size: 18px;
    margin-bottom: 4px;
  }
  p {
    margin: 0;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .small {
    font-size: 13px;
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
  .bars {
    display: flex;
    gap: 6px;
    align-items: stretch;
    height: 160px;
    overflow-x: auto;
  }
  .month {
    flex: 1 0 40px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .cols {
    flex: 1;
    display: flex;
    gap: 2px;
    align-items: flex-end;
    border-bottom: 1px solid var(--line);
  }
  .bar {
    flex: 1;
    min-height: 1px;
    border-radius: 3px 3px 0 0;
  }
  .plan,
  .sw.plan {
    background: transparent;
    border: 2px dashed var(--muted);
    border-bottom: 0;
  }
  .put,
  .sw.put {
    background: var(--accent);
  }
  .inv,
  .sw.inv {
    background: #c77a2e;
  }
  .m {
    font-size: 11px;
    color: var(--muted);
    text-align: center;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    font-size: 13px;
  }
  .sw {
    display: inline-block;
    width: 14px;
    height: 10px;
    margin-right: 6px;
    vertical-align: middle;
  }
  .sw.plan {
    border-bottom: 2px dashed var(--muted);
  }
  .notice.good {
    border-color: var(--line);
    background: var(--accent-soft);
    color: var(--ink);
  }
</style>
