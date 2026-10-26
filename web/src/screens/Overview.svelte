<script lang="ts">
  import BankAccounts from '../components/BankAccounts.svelte'
  import PhysicalMetals from '../components/PhysicalMetals.svelte'
  import PrivateHoldings from '../components/PrivateHoldings.svelte'
  import type { Valuation } from '../lib/core'
  import { date, money, rate as formatRate, units } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const STALE_DAYS = 7

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const today = new Date().toISOString().slice(0, 10)

  function total(prefix: string): number {
    return s.trial_balance
      .filter((r) => r.account === prefix || r.account.startsWith(`${prefix}:`))
      .reduce((sum, r) => sum + Number(r.debit) - Number(r.credit), 0)
  }

  const cost = $derived(s.valuations.reduce((sum, v) => sum + Number(v.cost), 0))
  const value = $derived(s.valuations.reduce((sum, v) => sum + Number(v.value ?? v.cost), 0))
  const unpriced = $derived(s.valuations.filter((v) => v.value === undefined).length)
  const unrealised = $derived(s.valuations.reduce((sum, v) => sum + Number(v.gain ?? 0), 0))
  const cash = $derived(total('Assets:Cash'))
  // Private holdings at the value set by hand, else at cost.
  const open = $derived(s.private.filter((p) => p.open))
  const privately = $derived(open.reduce((sum, p) => sum + Number(p.value ?? p.cost), 0))
  const contributed = $derived(-total('Equity:Contributions'))
  const opening = $derived(-total('Equity:OpeningBalances'))
  const realised = $derived(-total('Income:CapitalGains'))
  const dividends = $derived(-total('Income:Dividends'))
  const fees = $derived(total('Expenses:Fees'))
  const withheld = $derived(total('Expenses:Taxes'))
  const names = $derived(new Map(s.instruments.map((i) => [i.symbol, i])))
  const recent = $derived(s.entries.slice(0, 6))
  // Coins and bars have their own card; they still count in the totals above.
  const securities = $derived(s.valuations.filter((v) => v.broker !== 'PHYSICAL'))

  // Currencies that need a rate to show values in the base currency.
  const fxNeeded = $derived(
    [...new Set([...s.valuations.map((v) => v.quote?.price.currency), ...s.cash.map((c) => c.currency), ...s.private.map((p) => p.currency)])].filter(
      (c): c is string => !!c && c !== base,
    ),
  )
  const fxRates = $derived(new Map(s.fx))

  const ageDays = (iso: string) => Math.floor((Date.parse(today) - Date.parse(iso)) / 86_400_000)

  // Inline editors
  let editing = $state<string | null>(null)
  let price = $state('')
  let priceCurrency = $state('')
  let priceDate = $state(today)
  let editingFx = $state<string | null>(null)
  let rate = $state('')
  let rateDate = $state(today)
  let error = $state('')

  function editPrice(v: Valuation): void {
    editing = `${v.broker}:${v.symbol}`
    price = v.quote?.price.amount ?? ''
    priceCurrency = v.quote?.price.currency ?? names.get(v.symbol)?.currency ?? base
    priceDate = today
    error = ''
  }

  async function savePrice(symbol: string): Promise<void> {
    try {
      await app.setPrice(symbol, priceDate, price, priceCurrency)
      editing = null
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  function editFx(currency: string): void {
    editingFx = currency
    rate = fxRates.get(currency)?.rate ?? ''
    rateDate = today
    error = ''
  }

  async function saveFx(currency: string): Promise<void> {
    try {
      await app.setFxRate(currency, rateDate, rate)
      editingFx = null
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }
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
  <BankAccounts />
  <PhysicalMetals />
  <PrivateHoldings />
{:else}
  <section class="kpis" aria-label={t('overview.keyFigures')}>
    <div class="card kpi">
      <span class="label">{t('overview.netWorth')}</span>
      <span class="value num">{money(value + cash + privately, base)}</span>
      <span class="hint">{t('overview.netWorthSplit', { holdings: money(value, base), cash: money(cash, base) })}{#if open.length}{' · '}{t('overview.netWorthPrivate', { amount: money(privately, base) })}{/if}</span>
    </div>
    <div class="card kpi">
      <span class="label">{t('overview.unrealised')}</span>
      <span class="value num" class:loss={unrealised < 0}>{unrealised >= 0 ? '+' : ''}{money(unrealised, base)}</span>
      <span class="hint">{unpriced ? t('overview.unpriced', { count: unpriced }) : t('overview.onCost', { cost: money(cost, base) })}</span>
    </div>
    <div class="card kpi">
      <span class="label">{t('overview.contributed')}</span>
      <span class="value num">{money(contributed, base)}</span>
      {#if opening}<span class="hint">{t('overview.plusOpening', { amount: money(opening, base) })}</span>{/if}
    </div>
    <div class="card kpi">
      <span class="label">{t('overview.realised')}</span>
      <span class="value num">{money(realised, base)}</span>
      <span class="hint">{t('overview.dividends', { amount: money(dividends, base) })} · {t('overview.costsSplit', { fees: money(fees, base), tax: money(withheld, base) })}</span>
    </div>
  </section>

  <section class="card" aria-labelledby="holdings-title">
    <h2 id="holdings-title">{t('overview.holdings')}</h2>
    <p class="muted small">{t('overview.pricesNote')}</p>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>{t('common.instrument')}</th><th>{t('common.broker')}</th><th class="right">{t('common.units')}</th>
            <th class="right">{t('overview.cost', { currency: base })}</th><th>{t('overview.price')}</th>
            <th class="right">{t('overview.value', { currency: base })}</th><th class="right">{t('overview.gain')}</th><th></th>
          </tr>
        </thead>
        <tbody>
          {#each securities as v (v.broker + v.symbol)}
            {@const key = `${v.broker}:${v.symbol}`}
            <tr>
              <td><strong class="num">{v.symbol}</strong> <span class="muted">{names.get(v.symbol)?.name ?? ''}</span></td>
              <td>{v.broker}</td>
              <td class="right num">{units(v.units)}</td>
              <td class="right num">{money(v.cost)}</td>
              <td>
                {#if v.quote}
                  <span class="num">{money(v.quote.price.amount, v.quote.price.currency)}</span>
                  <span class="muted small-inline">{date(v.quote.date)} · {t(`source.${v.quote.source}`)}</span>
                  {#if ageDays(v.quote.date) > STALE_DAYS}<span class="pill warn">{t('overview.stale')}</span>{/if}
                {:else}
                  <span class="pill warn">{t('overview.noPrice')}</span>
                {/if}
              </td>
              <td class="right num">{v.value !== undefined ? money(v.value) : v.missing_fx ? t('overview.needsRate') : '—'}</td>
              <td class="right num" class:loss={Number(v.gain ?? 0) < 0}>{v.gain !== undefined ? money(v.gain) : ''}</td>
              <td class="right"><button class="btn ghost small-btn" onclick={() => editPrice(v)}>{t('overview.updatePrice')}</button></td>
            </tr>
            {#if editing === key}
              <tr class="editor">
                <td colspan="8">{@render priceForm(v)}</td>
              </tr>
            {/if}
          {:else}
            <tr><td colspan="8" class="muted">{t('overview.noHoldings')}</td></tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- On a phone the table becomes rows: what you hold, its value and gain. -->
    <ul class="holding-list">
      {#each securities as v (v.broker + v.symbol)}
        <li>
          <button class="holding" aria-expanded={editing === `${v.broker}:${v.symbol}`} onclick={() => (editing === `${v.broker}:${v.symbol}` ? (editing = null) : editPrice(v))}>
            <span class="what">
              <strong class="num">{v.symbol}</strong>
              <span class="muted">{units(v.units)} · {v.broker} · {#if v.quote}{money(v.quote.price.amount, v.quote.price.currency)} {date(v.quote.date)}{:else}{t('overview.noPrice')}{/if}</span>
            </span>
            <span class="worth num">
              <span>{v.value !== undefined ? money(v.value, base) : v.missing_fx ? t('overview.needsRate') : money(v.cost, base)}</span>
              {#if v.gain !== undefined}<span class="gain" class:loss={Number(v.gain) < 0}>{Number(v.gain) >= 0 ? '+' : ''}{money(v.gain, base)}</span>{/if}
            </span>
          </button>
          {#if editing === `${v.broker}:${v.symbol}`}<div class="editor-block">{@render priceForm(v)}</div>{/if}
        </li>
      {:else}
        <li class="muted">{t('overview.noHoldings')}</li>
      {/each}
    </ul>
  </section>

  {#snippet priceForm(v: Valuation)}
    <form class="price-form" onsubmit={(e) => (e.preventDefault(), savePrice(v.symbol))}>
      <label class="field">{t('overview.pricePer', { symbol: v.symbol })}<input inputmode="decimal" bind:value={price} required /></label>
      <label class="field">{t('overview.currency')}<input bind:value={priceCurrency} maxlength="3" required /></label>
      <label class="field">{t('common.date')}<input type="date" bind:value={priceDate} max={today} required /></label>
      <button class="btn">{t('common.save')}</button>
      <button type="button" class="btn ghost" onclick={() => (editing = null)}>{t('common.cancel')}</button>
    </form>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  {/snippet}

  <BankAccounts />
  <PhysicalMetals />
  <PrivateHoldings />

  <div class="row">
    <section class="card" aria-labelledby="cash-title">
      <h2 id="cash-title">{t('overview.cash')}</h2>
      <table>
        <tbody>
          {#each s.cash.filter((c) => !s.banks.some((a) => a.id === c.broker)) as c (c.broker + c.currency)}
            <tr><td>{c.broker}</td><td class="right num">{money(c.amount, c.currency)}</td></tr>
          {:else}
            <tr><td class="muted">{t('overview.noCash')}</td></tr>
          {/each}
        </tbody>
      </table>

      {#if fxNeeded.length}
        <h3 class="sub">{t('overview.rates', { currency: base })}</h3>
        <table class="rates">
          <tbody>
            {#each fxNeeded as c (c)}
              {@const fx = fxRates.get(c)}
              <tr>
                <td class="num">1 {c}</td>
                <td><span class="num">{fx ? `${formatRate(fx.rate)} ${base}` : '—'}</span> <span class="muted small-inline rate-meta">{fx ? `${date(fx.date)} · ${t(`source.${fx.source}`)}` : ''}</span></td>
                <td class="right"><button class="btn ghost small-btn" onclick={() => editFx(c)}>{t('overview.updateRate')}</button></td>
              </tr>
              {#if editingFx === c}
                <tr class="editor">
                  <td colspan="3">
                    <form onsubmit={(e) => (e.preventDefault(), saveFx(c))}>
                      <label class="field">{t('overview.rateFor', { currency: c, base })}<input inputmode="decimal" bind:value={rate} required /></label>
                      <label class="field">{t('common.date')}<input type="date" bind:value={rateDate} max={today} required /></label>
                      <button class="btn">{t('common.save')}</button>
                      <button type="button" class="btn ghost" onclick={() => (editingFx = null)}>{t('common.cancel')}</button>
                    </form>
                    {#if error}<p class="error" role="alert">{error}</p>{/if}
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      {/if}
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
  .small-inline {
    font-size: 12px;
    white-space: nowrap;
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
  .loss {
    color: var(--warn);
  }
  .small-btn {
    white-space: nowrap;
    min-height: 36px;
    padding: 0 12px;
    font-size: 13px;
  }
  .editor td {
    background: var(--surface-2);
  }
  /* The rate's date and source go under it, so the row fits a narrow card. */
  .rate-meta {
    display: block;
  }
  .editor form,
  .price-form {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: flex-end;
  }
  .price-form .field,
  .editor .field {
    width: 160px;
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
  .sub {
    font-size: 14px;
    margin: 16px 0 4px;
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
  .holding-list {
    display: none;
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .holding {
    width: 100%;
    min-height: 64px;
    padding: 10px 0;
    border: 0;
    border-top: 1px solid var(--line-soft);
    background: none;
    display: flex;
    align-items: center;
    gap: 12px;
    text-align: left;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  .what {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .what .muted {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .worth {
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
    font-size: 15px;
  }
  .gain {
    font-size: 13px;
    color: var(--muted);
  }
  .editor-block {
    background: var(--surface-2);
    border-radius: 10px;
    padding: 12px;
    margin-bottom: 10px;
  }
  .editor-block .field {
    width: 100%;
  }

  @media (max-width: 720px) {
    .kpis {
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 10px;
    }
    .kpi {
      padding: 12px;
      gap: 2px;
    }
    .kpi:first-child {
      grid-column: 1 / -1;
      padding: 16px;
    }
    .kpi:not(:first-child) .hint {
      display: none;
    }
    .kpi:not(:first-child) .label {
      font-size: 12px;
    }
    .kpi:not(:first-child) {
      padding: 10px;
    }
    .kpi:not(:first-child) .value {
      font-size: 14px;
      white-space: nowrap;
      letter-spacing: -0.02em;
    }
    .table-wrap {
      display: none;
    }
    .holding-list {
      display: block;
    }
    .small {
      margin-bottom: 4px;
    }
    .rates tr {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 0 8px;
    }
    .rates tr {
      border-bottom: 1px solid var(--line-soft);
      padding: 6px 0;
    }
    .rates td {
      padding-left: 0;
      padding-right: 0;
      border: 0;
    }
    .rates td.right {
      margin-left: auto;
    }
    .editor td {
      display: block;
      width: 100%;
    }
  }
</style>
