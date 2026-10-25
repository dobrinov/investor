<script lang="ts">
  import { date, money, units } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)

  function total(prefix: string): number {
    return s.trial_balance
      .filter((r) => r.account === prefix || r.account.startsWith(`${prefix}:`))
      .reduce((sum, r) => sum + Number(r.debit) - Number(r.credit), 0)
  }

  const invested = $derived(s.holdings.reduce((sum, h) => sum + Number(h.cost), 0))
  const contributed = $derived(-total('Equity:Contributions'))
  const opening = $derived(-total('Equity:OpeningBalances'))
  const realised = $derived(-total('Income:CapitalGains'))
  const dividends = $derived(-total('Income:Dividends'))
  const fees = $derived(total('Expenses:Fees'))
  const withheld = $derived(total('Expenses:Taxes'))
  const names = $derived(new Map(s.instruments.map((i) => [i.symbol, i.name])))
  const recent = $derived(s.entries.slice(0, 6))
</script>

<header>
  <h1>{t('nav.overview')}</h1>
  <p class="muted">{t('overview.subtitle', { currency: base })}</p>
</header>

{#if s.entries.length === 0}
  <section class="card empty">
    <h2>{t('overview.emptyTitle')}</h2>
    <p class="muted">{t('overview.emptyText')}</p>
    <button class="btn" onclick={() => (app.view = 'import')}>{t('overview.importFirst')}</button>
  </section>
{:else}
  <section class="kpis" aria-label={t('overview.keyFigures')}>
    <div class="card kpi">
      <span class="label">{t('overview.invested')}</span>
      <span class="value num">{money(invested, base)}</span>
      <span class="hint">{t('overview.atCost')}</span>
    </div>
    <div class="card kpi">
      <span class="label">{t('overview.contributed')}</span>
      <span class="value num">{money(contributed, base)}</span>
      {#if opening}<span class="hint">{t('overview.plusOpening', { amount: money(opening, base) })}</span>{/if}
    </div>
    <div class="card kpi">
      <span class="label">{t('overview.realised')}</span>
      <span class="value num">{money(realised, base)}</span>
      <span class="hint">{t('overview.dividends', { amount: money(dividends, base) })}</span>
    </div>
    <div class="card kpi">
      <span class="label">{t('overview.costs')}</span>
      <span class="value num">{money(fees + withheld, base)}</span>
      <span class="hint">{t('overview.costsSplit', { fees: money(fees, base), tax: money(withheld, base) })}</span>
    </div>
  </section>

  <section class="card" aria-labelledby="holdings-title">
    <h2 id="holdings-title">{t('overview.holdings')}</h2>
    <p class="muted small">{t('overview.pricesLater')}</p>
    <div class="table-wrap">
      <table>
        <thead>
          <tr><th>{t('common.instrument')}</th><th>{t('common.broker')}</th><th class="right">{t('common.units')}</th><th class="right">{t('overview.cost', { currency: base })}</th></tr>
        </thead>
        <tbody>
          {#each s.holdings as h (h.broker + h.symbol)}
            <tr>
              <td><strong class="num">{h.symbol}</strong> <span class="muted">{names.get(h.symbol) ?? ''}</span></td>
              <td>{h.broker}</td>
              <td class="right num">{units(h.units)}</td>
              <td class="right num">{money(h.cost)}</td>
            </tr>
          {:else}
            <tr><td colspan="4" class="muted">{t('overview.noHoldings')}</td></tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>

  <div class="row">
    <section class="card" aria-labelledby="cash-title">
      <h2 id="cash-title">{t('overview.cash')}</h2>
      <table>
        <tbody>
          {#each s.cash as c (c.broker + c.currency)}
            <tr><td>{c.broker}</td><td class="right num">{money(c.amount, c.currency)}</td></tr>
          {:else}
            <tr><td class="muted">{t('overview.noCash')}</td></tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section class="card" aria-labelledby="recent-title">
      <div class="head">
        <h2 id="recent-title">{t('overview.recent')}</h2>
        <button class="btn ghost" onclick={() => (app.view = 'ledger')}>{t('overview.allEntries')}</button>
      </div>
      <ul class="recent">
        {#each recent as e (e.id)}
          <li><span class="muted num">{date(e.date)}</span><span>{e.description}</span></li>
        {/each}
      </ul>
    </section>
  </div>
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
    font-size: 18px;
  }
  header p {
    margin: 0;
  }
  .small {
    font-size: 13px;
    margin: 6px 0 12px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
    padding: 32px;
  }
  .empty p {
    margin: 0;
    max-width: 560px;
    line-height: 1.55;
  }
  .kpis {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
  }
  .kpi {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .label,
  .hint {
    font-size: 13px;
    color: var(--muted);
  }
  .value {
    font-size: 26px;
    font-weight: 500;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    align-items: flex-start;
  }
  .row > * {
    flex: 1 1 320px;
    min-width: 0;
  }
  .row h2 {
    margin-bottom: 8px;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
  }
  .recent {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .recent li {
    display: flex;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid var(--line-soft);
    font-size: 14px;
  }
  .recent li span:first-child {
    flex: none;
    width: 110px;
  }
</style>
