<script lang="ts">
  import { describe } from '../lib/describe'
  import BankAccounts from '../components/BankAccounts.svelte'
  import Debts from '../components/Debts.svelte'
  import PhysicalMetals from '../components/PhysicalMetals.svelte'
  import PrivateHoldings from '../components/PrivateHoldings.svelte'
  import RowMenu from '../components/RowMenu.svelte'
  import { tick } from 'svelte'
  import type { Valuation } from '../lib/core'
  import { date, money, rate as formatRate, units } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const STALE_DAYS = 7
  // A loan dearer than this costs more than investing is likely to earn.
  const HIGH_RATE = 0.06

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

  // Net worth and what it is made of.
  const metals = $derived(s.valuations.filter((v) => v.broker === 'PHYSICAL').reduce((sum, v) => sum + Number(v.value ?? v.cost), 0))
  // What is owed, in the base currency; it comes off net worth.
  const owed = $derived(s.debts.reduce((sum, d) => sum + Number(d.value ?? 0), 0))
  const worth = $derived(value + cash + privately - owed)
  const putIn = $derived(contributed + opening)
  const growth = $derived(worth - putIn)
  const parts = $derived(
    [
      { key: 'securities', amount: value - metals, color: 'var(--part-securities)' },
      { key: 'cash', amount: cash, color: 'var(--part-cash)' },
      { key: 'private', amount: privately, color: 'var(--part-private)' },
      { key: 'metals', amount: metals, color: 'var(--part-metals)' },
    ].filter((p) => p.amount > 0),
  )
  const percent = (part: number, whole: number) => (whole ? `${((part / whole) * 100).toFixed(1)}%` : '')

  // Things that need the person, gathered in one place.
  const attention = $derived([
    ...securities.flatMap((v) => {
      if (!v.quote) return [{ id: `p:${v.broker}:${v.symbol}`, text: t('overview.attn.noPrice', { symbol: v.symbol }), action: t('overview.updatePrice'), run: () => fixPrice(v) }]
      const days = ageDays(v.quote.date)
      return days > STALE_DAYS ? [{ id: `p:${v.broker}:${v.symbol}`, text: t('overview.attn.stale', { symbol: v.symbol, days }), action: t('overview.updatePrice'), run: () => fixPrice(v) }] : []
    }),
    ...s.debts
      .filter((d) => Number(d.owed) > 0 && d.monthly && d.months_left === undefined)
      .map((d) => ({ id: `debt-never:${d.id}`, text: t('overview.attn.debtNever', { name: d.name }), action: t('overview.attn.showDebt'), run: () => showDebt() })),
    ...s.debts
      .filter((d) => Number(d.owed) > 0 && Number(d.rate ?? 0) >= HIGH_RATE)
      .map((d) => ({ id: `debt-rate:${d.id}`, text: t('overview.attn.debtRate', { name: d.name, rate: `${+(Number(d.rate) * 100).toFixed(2)}%` }), action: t('overview.attn.showDebt'), run: () => showDebt() })),
    ...fxNeeded.filter((c) => !fxRates.has(c)).map((c) => ({ id: `fx:${c}`, text: t('overview.attn.noRate', { currency: c }), action: t('overview.attn.addRate'), run: () => fixRate(c) })),
  ])

  function fixPrice(v: Valuation): void {
    tab = 'securities'
    editPrice(v)
    void tick().then(() => document.getElementById('holdings-title')?.scrollIntoView({ behavior: 'smooth', block: 'start' }))
  }

  function showDebt(): void {
    tab = 'debts'
    void tick().then(() => document.getElementById('debt-title')?.scrollIntoView({ behavior: 'smooth', block: 'start' }))
  }

  function fixRate(currency: string): void {
    editFx(currency)
    void tick().then(() => document.getElementById('cash-title')?.scrollIntoView({ behavior: 'smooth', block: 'start' }))
  }

  // What you own, one kind at a time.
  type Kind = 'securities' | 'banks' | 'metals' | 'private' | 'debts'
  let tab = $state<Kind>('securities')
  const kinds = $derived<{ id: Kind; count: number }[]>([
    { id: 'securities', count: securities.length },
    { id: 'banks', count: s.banks.length },
    { id: 'metals', count: new Set(s.valuations.filter((v) => v.broker === 'PHYSICAL').map((v) => v.symbol)).size },
    { id: 'private', count: s.private.length },
    { id: 'debts', count: s.debts.length },
  ])

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
  <Debts />
{:else}
  <section class="card hero" aria-labelledby="worth-title">
    <div class="worth">
      <h2 id="worth-title" class="label">{t('overview.netWorth')}</h2>
      <span class="value num">{money(worth, base)}</span>
      {#if putIn > 0}
        <span class="change" class:loss={growth < 0}>{t('overview.vsPutIn', { amount: `${growth >= 0 ? '+' : ''}${money(growth, base)}`, percent: `${growth >= 0 ? '+' : ''}${percent(growth, putIn)}` })}</span>
      {/if}
    </div>
    <div class="split">
      {#if parts.length}
        <div class="bar" role="img" aria-label={parts.map((p) => `${t(`overview.part.${p.key}` as 'overview.part.cash')} ${percent(p.amount, worth)}`).join(', ')}>
          {#each parts as p (p.key)}<span style="flex: {p.amount}; background: {p.color}"></span>{/each}
        </div>
        <div class="legend">
          {#each parts as p (p.key)}
            <div><span class="key"><i style="background: {p.color}"></i>{t(`overview.part.${p.key}` as 'overview.part.cash')}</span><span class="num">{money(p.amount, base)}</span></div>
          {/each}
        </div>
      {/if}
      <dl class="facts">
        <div><dt>{t('overview.unrealised')}</dt><dd class="num" class:loss={unrealised < 0}>{unrealised >= 0 ? '+' : ''}{money(unrealised, base)}</dd></div>
        <div><dt>{t('overview.putIn')}</dt><dd class="num">{money(putIn, base)}</dd></div>
        {#if owed > 0}<div><dt>{t('overview.owed')}</dt><dd class="num">−{money(owed, base)}</dd></div>{/if}
        <div><dt>{t('overview.realised')}</dt><dd class="num">{money(realised, base)}</dd></div>
        <div><dt>{t('overview.dividendsLabel')}</dt><dd class="num">{money(dividends, base)}</dd></div>
      </dl>
    </div>
  </section>

  {#if attention.length}
    <section class="attention" aria-labelledby="attention-title">
      <h2 id="attention-title">{t('overview.attention')} <span class="count num">{attention.length}</span></h2>
      <ul>
        {#each attention as item (item.id)}
          <li><span>{item.text}</span><button class="btn ghost warn-btn" onclick={item.run}>{item.action}</button></li>
        {/each}
      </ul>
    </section>
  {/if}

  <div class="owned">
    <h2>{t('overview.owned')}</h2>
    <div class="tabs" role="tablist" aria-label={t('overview.tabs')}>
      {#each kinds as kind (kind.id)}
        <button role="tab" id="tab-{kind.id}" aria-selected={tab === kind.id} aria-controls="panel-{kind.id}" onclick={() => (tab = kind.id)}>
          {t(`overview.tab.${kind.id}` as 'overview.tab.banks')}<span class="num tab-count">{kind.count}</span>
        </button>
      {/each}
    </div>
  </div>

  <div role="tabpanel" id="panel-{tab}" aria-labelledby="tab-{tab}" class="panel">
  {#if tab === 'securities'}
  <section class="card" aria-labelledby="holdings-title">
    <h2 id="holdings-title">{t('overview.holdings')}</h2>
    <p class="muted small">{t('overview.pricesNote')}</p>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>{t('common.instrument')}</th><th class="right">{t('common.units')}</th>
            <th class="right">{t('overview.cost', { currency: base })}</th><th>{t('overview.price')}</th>
            <th class="right">{t('overview.value', { currency: base })}</th><th class="right">{t('overview.gain')}</th><th></th>
          </tr>
        </thead>
        <tbody>
          {#each securities as v (v.broker + v.symbol)}
            {@const key = `${v.broker}:${v.symbol}`}
            <tr>
              <td><strong class="num">{v.symbol}</strong><span class="muted sub-line">{[names.get(v.symbol)?.name, v.broker].filter(Boolean).join(' · ')}</span></td>
              <td class="right num">{units(v.units)}</td>
              <td class="right num">{money(v.cost)}</td>
              <td>
                {#if v.quote}
                  <span class="num">{money(v.quote.price.amount, v.quote.price.currency)}</span>
                  {#if ageDays(v.quote.date) > STALE_DAYS}<span class="pill warn sub-pill">{t('overview.stale')}</span>{:else}<span class="muted sub-line">{date(v.quote.date)} · {t(`source.${v.quote.source}`)}</span>{/if}
                {:else}
                  <span class="pill warn">{t('overview.noPrice')}</span>
                {/if}
              </td>
              <td class="right num">{v.value !== undefined ? money(v.value) : v.missing_fx ? t('overview.needsRate') : '—'}</td>
              <td class="right num" class:loss={Number(v.gain ?? 0) < 0}>{v.gain !== undefined ? money(v.gain) : ''}</td>
              <td class="right">
                <RowMenu
                  label={t('common.moreFor', { name: v.symbol })}
                  items={[
                    { label: t('overview.updatePrice'), run: () => editPrice(v) },
                    { label: t('overview.trade'), run: () => (app.view = 'trade') },
                    { label: t('overview.inLedger'), run: () => (app.view = 'ledger') },
                  ]}
                />
              </td>
            </tr>
            {#if editing === key}
              <tr class="editor">
                <td colspan="7">{@render priceForm(v)}</td>
              </tr>
            {/if}
          {:else}
            <tr><td colspan="7" class="muted">{t('overview.noHoldings')}</td></tr>
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

  {:else if tab === 'banks'}
    <BankAccounts />
  {:else if tab === 'metals'}
    <PhysicalMetals />
  {:else if tab === 'private'}
    <PrivateHoldings />
  {:else}
    <Debts />
  {/if}
  </div>

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
          <li><span class="muted num">{date(e.date)}</span><span>{describe(e.description)}</span></li>
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
  .hero {
    display: flex;
    flex-wrap: wrap;
    gap: 24px 40px;
    padding: 28px;
  }
  .hero .worth {
    flex: 1 1 260px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    text-align: left;
  }
  .label {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--muted);
  }
  .value {
    font-size: 44px;
    font-weight: 500;
    letter-spacing: -0.01em;
    line-height: 1.1;
  }
  .change {
    font-size: 15px;
    font-weight: 500;
    color: var(--gain);
  }
  .split {
    flex: 2 1 420px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
    justify-content: center;
  }
  .bar {
    display: flex;
    gap: 2px;
    height: 14px;
    border-radius: 999px;
    overflow: hidden;
  }
  .bar span {
    min-width: 4px;
  }
  .legend {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 12px;
    font-size: 14px;
  }
  .legend div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .key {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--ink-2);
  }
  .key i {
    width: 10px;
    height: 10px;
    border-radius: 3px;
  }
  .legend .num {
    font-size: 16px;
  }
  .facts {
    margin: 0;
    padding-top: 14px;
    border-top: 1px solid var(--line-soft);
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 12px;
    font-size: 13px;
  }
  .facts dt {
    color: var(--muted);
  }
  .facts dd {
    margin: 2px 0 0;
    font-size: 15px;
    color: var(--ink);
  }
  .attention {
    background: var(--warn-soft);
    border: 1px solid var(--warn-line);
    border-radius: 12px;
    overflow: hidden;
  }
  .attention h2 {
    margin: 0;
    padding: 14px 20px 8px;
    font-size: 15px;
    color: var(--warn-ink);
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .count {
    padding: 0 8px;
    border-radius: 999px;
    background: var(--warn);
    color: var(--surface);
    font-size: 12px;
  }
  .attention ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .attention li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 12px;
    padding: 10px 20px;
    border-top: 1px solid var(--warn-line);
    font-size: 15px;
  }
  .attention li span {
    flex: 1 1 280px;
  }
  .warn-btn {
    min-height: 40px;
    border-color: var(--warn-line);
    color: var(--warn-ink);
  }
  .warn-btn:hover {
    background: var(--warn-tint);
  }
  .owned {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 20px;
    margin-top: 8px;
  }
  .owned h2 {
    margin: 0;
    font-size: 20px;
  }
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 4px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 10px;
  }
  .tabs button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 38px;
    padding: 0 14px;
    border: 0;
    border-radius: 7px;
    background: none;
    color: var(--ink-2);
    font: 500 14px var(--sans);
    cursor: pointer;
  }
  .tabs button:hover {
    background: var(--surface-2);
  }
  .tabs button[aria-selected='true'] {
    background: var(--accent-tint);
    color: var(--accent-strong);
    font-weight: 600;
  }
  .tab-count {
    font-size: 12px;
    color: var(--muted);
  }
  .panel {
    display: flex;
    flex-direction: column;
    gap: 20px;
    margin-top: -8px;
  }
  .sub-line {
    display: block;
    font-size: 13px;
  }
  .sub-pill {
    display: table;
    margin-top: 2px;
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
    .hero {
      padding: 18px;
      gap: 16px;
    }
    .value {
      font-size: 34px;
    }
    .facts {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .owned {
      flex-direction: column;
      align-items: stretch;
    }
    .tabs {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .tabs button {
      justify-content: center;
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
