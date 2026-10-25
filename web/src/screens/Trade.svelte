<script lang="ts">
  // Plan a trade: fill in a purchase or sale, see everything it changes —
  // cash, fees, gain and tax, holdings, the emergency fund, the long run and
  // the journal entry — and only then record it.
  import Help from '../components/Help.svelte'
  import type { Entry, FeeKind, TradeInput, TradeOutlook } from '../lib/core'
  import { packs, brokers } from '../lib/configs'
  import { date as formatDate, money, units as formatUnits, whole } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const now = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  const today = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`
  const OTHER = '__other__'

  // Brokers you already use first, then the ones Investor can import.
  const knownBrokers = $derived([...new Set([...s.valuations.map((v) => v.broker), ...s.cash.map((c) => c.broker), ...brokers.map((b) => b.account)])])

  const feeKinds: FeeKind[] = ['commission', 'fx_conversion', 'other']

  function blank(broker = ''): Record<string, string> {
    return { id: crypto.randomUUID(), side: 'buy', broker, otherBroker: '', symbol: '', name: '', isin: '', units: '1', price: '', currency: '', date: today, venue: '', fx_rate: '', commission: '', fx_conversion: '', other: '' }
  }

  let form = $state(blank())
  $effect(() => {
    if (!form.broker && knownBrokers.length) form.broker = knownBrokers[0]
  })
  let recorded = $state<Entry | null>(null)
  let recordError = $state('')
  let busy = $state(false)

  const broker = $derived(form.broker === OTHER ? form.otherBroker.trim() : form.broker)
  const held = $derived(s.valuations.filter((v) => v.broker === broker))
  const heldUnits = $derived(held.find((v) => v.symbol === form.symbol)?.units)
  const currency = $derived(form.currency || base)
  const quote = $derived(s.valuations.find((v) => v.symbol === form.symbol && v.quote)?.quote)

  /** Fills currency, price and exchange from what the books already know about a symbol. */
  function chooseSymbol(symbol: string): void {
    form.symbol = symbol.trim().toUpperCase()
    const instrument = s.instruments.find((i) => i.symbol === form.symbol)
    const q = s.valuations.find((v) => v.symbol === form.symbol && v.quote)?.quote
    form.currency = q?.price.currency ?? instrument?.currency ?? form.currency
    if (q) form.price = String(Number(q.price.amount))
    const venue = s.entries.find((e) => e.venue && e.postings.some((p) => p.account.endsWith(`:${form.symbol}`)))?.venue
    if (venue && !form.venue) form.venue = venue.code
    if (form.side === 'sell' && !instrument) form.name = ''
  }

  const fxKnown = $derived(s.fx.find(([c]) => c === currency)?.[1].rate)
  $effect(() => {
    if (currency !== base && !form.fx_rate && fxKnown) form.fx_rate = String(Number(fxKnown))
  })

  const decimal = (v: string) => v.trim().replace(',', '.')
  const input = $derived.by((): TradeInput | null => {
    if (!broker || !form.symbol || !(Number(decimal(form.units)) > 0) || decimal(form.price) === '') return null
    return {
      id: form.id,
      date: form.date || today,
      broker,
      side: form.side as 'buy' | 'sell',
      symbol: form.symbol,
      name: form.name || null,
      isin: form.isin || null,
      units: decimal(form.units),
      price: decimal(form.price),
      currency,
      fees: feeKinds.filter((k) => decimal(form[k]) !== '').map((kind) => ({ kind, amount: decimal(form[kind]) })),
      venue: form.venue || null,
      fx_rate: currency !== base && decimal(form.fx_rate) ? decimal(form.fx_rate) : null,
    }
  })

  const year = $derived(Number((form.date || today).slice(0, 4)))
  const pack = $derived(packs.find((p) => p.country === s.settings.tax_residency && p.year === year))
  const projectionYears = 20

  const outlook = $derived.by((): { ok: TradeOutlook } | { error: string } | null => {
    void app.snapshot
    if (!input || !app.book) return null
    try {
      return { ok: app.book.previewTrade(input, pack?.toml ?? null, { today: today.slice(0, 7), years: projectionYears, market: 'base', spend_goals: true }) }
    } catch (e) {
      return { error: e instanceof Error ? e.message : String(e) }
    }
  })
  const o = $derived(outlook && 'ok' in outlook ? outlook.ok : null)

  const percent = (v: number) => `${(v * 100).toFixed(v < 0.1 ? 1 : 0)}%`
  const feeShare = $derived(o && Number(o.value) > 0 ? Number(o.fees) / Number(o.value) : 0)
  const short = $derived(o?.cash.some((c) => Number(c.after) < 0) ?? false)

  // The largest share of the investments, before and after the trade.
  function largest(side: 'before' | 'after'): { symbol: string; share: number } | undefined {
    const total = o ? Number(side === 'before' ? o.invested_before : o.invested_after) : 0
    if (!o || total <= 0) return undefined
    return o.holdings.map((h) => ({ symbol: h.symbol, share: Number(h[side]) / total })).sort((a, b) => b.share - a.share)[0]
  }
  const biggest = $derived(largest('after'))
  // Warn only when the trade makes the portfolio more concentrated.
  const concentrates = $derived(!!biggest && biggest.share > 0.5 && biggest.share > (largest('before')?.share ?? 0) + 0.005)
  const scale = $derived(o ? Math.max(1, ...o.holdings.map((h) => Math.max(Number(h.before), Number(h.after)))) : 1)

  const spending = $derived(Number(s.plan.profile?.monthly_spending ?? 0))
  const bufferGoal = $derived(s.plan.goals.find((g) => g.kind === 'emergency_fund'))
  const bufferTarget = $derived(bufferGoal && spending > 0 ? Number(bufferGoal.target) * spending : 0)

  async function record(): Promise<void> {
    if (!input) return
    busy = true
    recordError = ''
    try {
      recorded = (await app.recordTrade(input)) ?? null
      form = blank(form.broker)
    } catch (e) {
      recordError = e instanceof Error ? e.message : String(e)
    } finally {
      busy = false
    }
  }

  async function undo(): Promise<void> {
    if (!recorded) return
    await app.undoTrade(recorded.id)
    recorded = null
  }
</script>

<header>
  <h1>{t('nav.trade')}</h1>
  <p class="muted">{t('trade.subtitle')}</p>
</header>

{#if recorded}
  <div class="notice ok" role="status">
    <span>{t('trade.recorded', { id: recorded.id })}</span>
    <button class="link" onclick={undo}>{t('trade.undo')}</button>
    <button class="link" onclick={() => (app.view = 'ledger')}>{t('trade.viewLedger')}</button>
  </div>
{/if}

<div class="layout">
  <form class="card form" onsubmit={(e) => (e.preventDefault(), record())}>
    <div class="field">
      <span id="side-label">{t('trade.side')}</span>
      <div role="radiogroup" aria-labelledby="side-label" class="segmented">
        {#each ['buy', 'sell'] as const as side (side)}
          <button type="button" role="radio" aria-checked={form.side === side} class:on={form.side === side} onclick={() => (form.side = side)}>{t(`trade.${side}`)}</button>
        {/each}
      </div>
    </div>

    <label class="field">{t('trade.broker')}
      <select bind:value={form.broker}>
        {#each knownBrokers as b (b)}<option value={b}>{b}</option>{/each}
        <option value={OTHER}>{t('trade.otherBroker')}</option>
      </select>
    </label>
    {#if form.broker === OTHER}
      <label class="field">{t('trade.brokerName')}<input bind:value={form.otherBroker} required /><span class="hint">{t('trade.brokerNameHint')}</span></label>
    {/if}

    {#if form.side === 'sell' && held.length}
      <label class="field">{t('trade.instrument')}
        <select value={form.symbol} onchange={(e) => chooseSymbol(e.currentTarget.value)}>
          <option value="">{t('trade.pickHolding')}</option>
          {#each held as h (h.symbol)}<option value={h.symbol}>{h.symbol} · {formatUnits(h.units)}</option>{/each}
        </select>
      </label>
    {:else}
      <label class="field">{t('trade.symbol')}
        <input list="known-symbols" value={form.symbol} onchange={(e) => chooseSymbol(e.currentTarget.value)} autocapitalize="characters" required />
        <span class="hint">{t('trade.symbolHint')}</span>
      </label>
      <datalist id="known-symbols">{#each s.instruments as i (i.symbol)}<option value={i.symbol}>{i.name ?? ''}</option>{/each}</datalist>
      {#if form.symbol && !s.instruments.some((i) => i.symbol === form.symbol)}
        <div class="row">
          <label class="field">{t('trade.name')}<input bind:value={form.name} /></label>
          <label class="field">{t('trade.isin')}<input bind:value={form.isin} /></label>
        </div>
      {/if}
    {/if}

    <div class="row">
      <label class="field">{t('trade.units')}
        <input inputmode="decimal" bind:value={form.units} required />
        {#if heldUnits}<span class="hint">{t('trade.held', { units: formatUnits(heldUnits) })}</span>{/if}
      </label>
      <label class="field">{t('trade.price')}
        <input inputmode="decimal" bind:value={form.price} required />
        {#if quote}<span class="hint">{t('trade.lastPrice', { price: money(quote.price.amount, quote.price.currency), date: formatDate(quote.date) })}</span>{/if}
      </label>
    </div>
    <div class="row">
      <label class="field">{t('trade.currency')}<input value={currency} oninput={(e) => (form.currency = e.currentTarget.value.trim().toUpperCase())} maxlength="3" /></label>
      <label class="field">{t('trade.date')}<input type="date" bind:value={form.date} /></label>
    </div>
    {#if currency !== base}
      <div class="field">
        <span class="with-help"><label for="fx">{t('trade.fxRate', { base, currency })}</label><Help about={t('trade.fxRate', { base, currency })} why={t('trade.fxWhy', { base })} without={t('trade.fxWithout')} /></span>
        <input id="fx" inputmode="decimal" bind:value={form.fx_rate} />
      </div>
    {/if}
    <div class="field">
      <span class="with-help"><label for="venue">{t('trade.venue')}</label><Help about={t('trade.venue')} why={t('trade.venueWhy')} without={t('trade.venueWithout')} /></span>
      <input id="venue" list="venues" bind:value={form.venue} autocapitalize="characters" />
      <datalist id="venues">{#each ['XETR', 'XAMS', 'XPAR', 'XMIL', 'XBRU', 'XLIS', 'BVME', 'XBUL', 'LSE', 'NASDAQ', 'NYSE'] as v (v)}<option value={v}></option>{/each}</datalist>
    </div>

    <fieldset>
      <legend>{t('trade.fees', { currency })}</legend>
      <div class="row three">
        {#each feeKinds as k (k)}
          <label class="field">{t(`trade.fee.${k}` as 'trade.fee.commission')}<input inputmode="decimal" bind:value={form[k]} placeholder="0" /></label>
        {/each}
      </div>
    </fieldset>

    <div class="actions">
      <button class="btn" disabled={!o || busy}>{busy ? t('trade.recording') : t('trade.record')}</button>
      <button type="button" class="btn ghost" onclick={() => (form = blank(form.broker))}>{t('trade.reset')}</button>
    </div>
    {#if recordError}<p class="error" role="alert">{recordError}</p>{/if}
  </form>

  <div class="preview" aria-live="polite">
    {#if !outlook}
      <p class="card muted">{t('trade.fill')}</p>
    {:else if 'error' in outlook}
      <p class="notice" role="alert">{outlook.error}</p>
    {:else if o && input}
      <section class="card">
        <h2>{t(`trade.summary.${input.side}`, { units: formatUnits(input.units), symbol: input.symbol, value: money(o.value, base) })}</h2>
        <p class="muted">{t('trade.plusFees', { fees: money(o.fees, base) })} · {formatUnits(o.units_before)} → {formatUnits(o.units_after)} {input.symbol}</p>
      </section>

      <section class="card">
        <h3>{t('trade.cashTitle', { broker })}</h3>
        {#each o.cash as c (c.currency)}
          <p class="change num"><span>{money(c.before, c.currency)}</span> → <strong class:bad={Number(c.after) < 0}>{money(c.after, c.currency)}</strong></p>
        {/each}
        {#if short}<p class="notice">{t('trade.noCash')}</p>{/if}
      </section>

      <section class="card">
        <h3>{t('trade.costsTitle')}</h3>
        <p>{t('trade.feeShare', { fees: money(o.fees, base), share: percent(feeShare) })}</p>
        {#if Number(o.fees) === 0}<p class="muted small">{t('trade.noFees')}</p>{:else if feeShare > 0.01}<p class="notice">{t('trade.highFees')}</p>{/if}
      </section>

      {#if o.gain !== null && o.cost !== null}
        <section class="card">
          <h3>{t('trade.gainTitle')}</h3>
          <p class="big num" class:up={Number(o.gain) >= 0} class:bad={Number(o.gain) < 0}>{Number(o.gain) >= 0 ? '+' : ''}{money(o.gain, base)}</p>
          <p class="muted small">{t('trade.gainText', { proceeds: money(o.value, base), cost: money(o.cost, base) })}</p>
          <h3 class="gap">{t('trade.taxTitle')}</h3>
          {#if o.tax}
            <span class="pill" class:warn={o.tax.treatment !== 'exempt'}>{t(`trade.treatment.${o.tax.treatment}`)}</span>
            <p class="small">
              {#if Number(o.gain) < 0}{t('trade.taxLoss')}
              {:else if o.tax.treatment === 'exempt'}{t('trade.taxExempt', { form: o.tax.form })}
              {:else if o.tax.treatment === 'taxable'}{t('trade.taxTaxable', { estimate: money(o.tax.estimate, base), rate: percent(Number(o.tax.rate)), form: o.tax.form })}
              {:else}{t('trade.taxUnknown')}{/if}
            </p>
            {#if !o.tax.verified}<p class="muted small">{t('trade.taxUnverified')}</p>{/if}
          {:else}
            <p class="muted small">{t('trade.noPack', { country: s.settings.tax_residency, year })}</p>
          {/if}
        </section>
      {/if}

      <section class="card">
        <h3>{t('trade.holdingsTitle')}</h3>
        <div class="bars">
          {#each o.holdings as h (h.symbol)}
            <div class="bar-row" class:this={h.symbol === input.symbol}>
              <span class="sym">{h.symbol}</span>
              <span class="track"><span class="b before" style:width="{(Number(h.before) / scale) * 100}%"></span><span class="b after" style:width="{(Number(h.after) / scale) * 100}%"></span></span>
              <span class="num small">{whole(Number(h.before), base)} → {whole(Number(h.after), base)}</span>
            </div>
          {/each}
        </div>
        <p class="legend muted small"><i class="sw before"></i>{t('trade.before')} <i class="sw after"></i>{t('trade.after')}</p>
        {#if biggest && concentrates}<p class="notice">{t('trade.concentrated', { symbol: biggest.symbol, share: percent(biggest.share) })}</p>{/if}
      </section>

      {#if bufferTarget > 0}
        <section class="card">
          <h3>{t('trade.bufferTitle')}</h3>
          {#if Number(o.cash_after) < bufferTarget}
            <p class="notice">{t('trade.bufferLow', { cash: whole(Number(o.cash_after), base), target: whole(bufferTarget, base) })}</p>
          {:else}
            <p>{t('trade.bufferOk', { cash: whole(Number(o.cash_after), base), target: whole(bufferTarget, base) })}</p>
          {/if}
        </section>
      {/if}

      <section class="card">
        <h3>{t('trade.futureTitle', { years: projectionYears })}</h3>
        <p>{t('trade.futureText', { before: whole(o.before.p50, base), after: whole(o.after.p50, base) })}
          <strong class:up={o.after.p50 >= o.before.p50} class:bad={o.after.p50 < o.before.p50}>({o.after.p50 >= o.before.p50 ? '+' : '−'}{whole(Math.abs(o.after.p50 - o.before.p50), base)})</strong></p>
        <p class="muted small">{t('trade.futureBad', { before: whole(o.before.p10, base), after: whole(o.after.p10, base) })}</p>
        <p class="muted small">{t('trade.futureNote')}</p>
      </section>

      <section class="card">
        <h3>{t('trade.entryTitle')}</h3>
        <p class="muted small">{t('trade.entryText')}</p>
        <div class="table-wrap">
          <table>
            <thead><tr><th>{t('trade.account')}</th><th class="right">{t('trade.amount')}</th><th class="right">{t('trade.inBase', { base })}</th></tr></thead>
            <tbody>
              {#each o.entry.postings as p, i (i)}
                <tr><td class="num acct">{p.account}</td><td class="right num">{money(p.amount.amount, p.amount.currency)}</td><td class="right num">{money(p.base, base)}</td></tr>
              {/each}
            </tbody>
          </table>
        </div>
      </section>
    {/if}
  </div>
</div>

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
    margin: 0;
  }
  h3 {
    font-size: 15px;
    margin: 0 0 8px;
  }
  h3.gap {
    margin-top: 16px;
  }
  p {
    margin: 0;
  }
  .layout {
    display: grid;
    grid-template-columns: minmax(0, 400px) minmax(0, 1fr);
    gap: 20px;
    align-items: start;
  }
  @media (max-width: 960px) {
    .layout {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .preview {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .preview .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .row.three {
    grid-template-columns: repeat(3, 1fr);
    align-items: end;
  }
  .pill {
    align-self: flex-start;
  }
  .hint {
    font-weight: 400;
    font-size: 12px;
    color: var(--muted);
  }
  .with-help {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  fieldset {
    border: 0;
    padding: 0;
    margin: 0;
  }
  legend {
    font-size: 13px;
    font-weight: 600;
    color: var(--ink-2);
    margin-bottom: 6px;
  }
  .segmented {
    display: flex;
    gap: 4px;
    background: var(--line-soft);
    border-radius: 10px;
    padding: 4px;
  }
  .segmented button {
    flex: 1;
    min-height: 40px;
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
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .notice.ok {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
    align-items: center;
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--ink);
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
  .change {
    font-size: 18px;
  }
  .big {
    font-size: 24px;
    font-weight: 500;
  }
  .up {
    color: var(--accent-strong);
  }
  .bad {
    color: var(--warn);
  }
  .small {
    font-size: 13px;
    line-height: 1.5;
  }
  .bars {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .bar-row {
    display: grid;
    grid-template-columns: 64px 1fr auto;
    gap: 10px;
    align-items: center;
  }
  .bar-row.this .sym {
    font-weight: 700;
  }
  .sym {
    font: 500 13px var(--mono);
  }
  .track {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .b {
    display: block;
    height: 7px;
    border-radius: 4px;
    min-width: 2px;
  }
  .b.before,
  .sw.before {
    background: var(--line);
  }
  .b.after,
  .sw.after {
    background: var(--accent);
  }
  .sw {
    display: inline-block;
    width: 14px;
    height: 8px;
    border-radius: 3px;
    margin: 0 4px 0 8px;
    vertical-align: middle;
  }
  .acct {
    font-size: 12px;
  }
</style>
