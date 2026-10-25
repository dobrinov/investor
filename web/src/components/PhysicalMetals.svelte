<script lang="ts">
  // Coins and bars kept at home or in a safe. Weights are fine metal in
  // grams; troy ounces are converted on the way in. The price is typed in
  // now and then, so the value is only as fresh as the last price.
  import type { Metal, MetalHolding } from '../lib/core'
  import { date, decimal, money, units } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const GRAMS_PER_OUNCE = 31.1034768
  const METALS: Metal[] = ['gold', 'silver', 'platinum', 'palladium']

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const today = new Date().toISOString().slice(0, 10)
  const rateOf = (currency: string) => s.fx.find(([c]) => c === currency)?.[1].rate
  const entries = $derived(new Map(s.entries.map((e) => [e.id, e])))

  /** Grams from a weight in grams or troy ounces, to four decimals. */
  const grams = (weight: string, unit: string) => {
    const g = Number(decimal(weight)) * (unit === 'oz' ? GRAMS_PER_OUNCE : 1)
    return String(Math.round(g * 10_000) / 10_000)
  }
  const ounces = (g: string | number) => units(Math.round((Number(g) / GRAMS_PER_OUNCE) * 1000) / 1000)

  let error = $state('')

  // Purchase or sale
  const blank = (side: 'buy' | 'sell', metal: Metal = 'gold') => ({ side, entry_id: crypto.randomUUID(), metal, description: '', weight: '', unit: 'oz', amount: '', currency: base, date: today, fx_rate: '' })
  let form = $state<ReturnType<typeof blank> | null>(null)
  const currency = $derived((form?.currency ?? base).trim().toUpperCase())
  $effect(() => {
    const known = rateOf(currency)
    if (form && currency !== base && !form.fx_rate && known) form.fx_rate = String(Number(known))
  })

  function open(side: 'buy' | 'sell', metal?: Metal): void {
    form = blank(side, metal)
    pricing = null
    error = ''
  }

  async function save(): Promise<void> {
    if (!form) return
    error = ''
    const input = {
      entry_id: form.entry_id,
      date: form.date,
      metal: form.metal,
      grams: grams(form.weight, form.unit),
      amount: decimal(form.amount),
      currency,
      fx_rate: currency !== base && decimal(form.fx_rate) ? decimal(form.fx_rate) : null,
      description: form.description,
    }
    try {
      await (form.side === 'buy' ? app.buyMetal(input) : app.sellMetal(input))
      form = null
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  // Price editor
  let pricing = $state<Metal | null>(null)
  let price = $state({ amount: '', unit: 'oz', currency: '', date: today, fx_rate: '' })

  function editPrice(h: MetalHolding): void {
    pricing = h.metal
    form = null
    error = ''
    const c = h.price?.price.currency ?? base
    const perOunce = h.price ? Math.round(Number(h.price.price.amount) * GRAMS_PER_OUNCE * 100) / 100 : ''
    price = { amount: String(perOunce), unit: 'oz', currency: c, date: today, fx_rate: c !== base ? String(Number(rateOf(c) ?? '')) || '' : '' }
  }

  async function savePrice(metal: Metal): Promise<void> {
    error = ''
    const c = price.currency.trim().toUpperCase()
    const perGram = Number(decimal(price.amount)) / (price.unit === 'oz' ? GRAMS_PER_OUNCE : 1)
    try {
      await app.setMetalPrice({ date: price.date, metal, per_gram: String(Math.round(perGram * 10_000) / 10_000), currency: c, fx_rate: c !== base && decimal(price.fx_rate) ? decimal(price.fx_rate) : null })
      pricing = null
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  async function remove(id: string): Promise<void> {
    if (!confirm(t('metal.removeConfirm'))) return
    error = ''
    try {
      await app.removeMetalEntry(id)
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }
</script>

<section class="card" aria-labelledby="metal-title">
  <div class="head">
    <h2 id="metal-title">{t('metal.title')}</h2>
    {#if !form}<button class="btn ghost" onclick={() => open('buy')}>{t('metal.add')}</button>{/if}
  </div>
  <p class="muted small">{t('metal.intro')}</p>

  {#if form}
    <form class="add" onsubmit={(e) => (e.preventDefault(), save())}>
      <h3 class="wide">{form.side === 'buy' ? t('metal.buyTitle') : t('metal.sellTitle')}</h3>
      <label class="field">{t('metal.metal')}
        <select bind:value={form.metal}>
          {#each METALS as m (m)}<option value={m}>{t(`metal.${m}`)}</option>{/each}
        </select>
      </label>
      <label class="field">{t('metal.what')}<input bind:value={form.description} placeholder={t('metal.whatPlaceholder')} /></label>
      <div class="field weight">
        <label for="metal-weight">{t('metal.weight')}</label>
        <div class="pair">
          <input id="metal-weight" inputmode="decimal" bind:value={form.weight} required />
          <select bind:value={form.unit} aria-label={t('metal.unit')}>
            <option value="oz">{t('metal.oz')}</option>
            <option value="g">{t('metal.g')}</option>
          </select>
        </div>
        <span class="hint">{t('metal.weightHint')}</span>
      </div>
      <label class="field">{form.side === 'buy' ? t('metal.paid') : t('metal.received')}<input inputmode="decimal" bind:value={form.amount} required /><span class="hint">{form.side === 'buy' ? t('metal.amountHint') : t('metal.receivedHint')}</span></label>
      <label class="field">{t('overview.currency')}<input bind:value={form.currency} maxlength="3" required /></label>
      <label class="field">{t('common.date')}<input type="date" bind:value={form.date} max={today} required /></label>
      {#if currency !== base && currency.length === 3}
        <label class="field">{t('private.rate', { currency, base })}<input inputmode="decimal" bind:value={form.fx_rate} required /></label>
      {/if}
      <p class="muted small wide">{form.side === 'buy' ? t('metal.buyHint') : t('metal.sellHint')}</p>
      <div class="actions wide">
        <button class="btn">{form.side === 'buy' ? t('metal.saveBuy') : t('metal.saveSell')}</button>
        <button type="button" class="btn ghost" onclick={() => (form = null)}>{t('common.cancel')}</button>
      </div>
      {#if error}<p class="error wide" role="alert">{error}</p>{/if}
    </form>
  {/if}

  {#each s.metals as h (h.metal)}
    <article class="metal">
      <div class="summary">
        <div>
          <strong>{t(`metal.${h.metal}`)}</strong>
          <span class="muted block num">{units(h.grams)} {t('metal.g')} · {ounces(h.grams)} oz</span>
        </div>
        <div class="right num">
          <strong>{money(h.value ?? h.cost, base)}</strong>
          {#if h.gain !== undefined}<span class="block gain" class:loss={Number(h.gain) < 0}>{Number(h.gain) >= 0 ? '+' : ''}{money(h.gain, base)}</span>{/if}
        </div>
      </div>
      <p class="muted small line num">
        {t('metal.cost', { amount: money(h.cost, base) })}
        {#if h.price}
          · {t('metal.priceOn', { oz: money(Number(h.price.price.amount) * GRAMS_PER_OUNCE, h.price.price.currency), g: money(h.price.price.amount, h.price.price.currency), date: date(h.price.date) })}
          {#if h.price.source === 'trade'}<span class="pill warn">{t('metal.priceFromTrade')}</span>{/if}
        {/if}
      </p>
      <div class="actions">
        <button class="btn ghost small-btn" onclick={() => editPrice(h)}>{t('metal.updatePrice')}</button>
        <button class="btn ghost small-btn" onclick={() => open('buy', h.metal)}>{t('metal.buyMore')}</button>
        {#if Number(h.grams) > 0}<button class="btn ghost small-btn" onclick={() => open('sell', h.metal)}>{t('metal.sell')}</button>{/if}
      </div>

      {#if pricing === h.metal}
        <form class="price" onsubmit={(e) => (e.preventDefault(), savePrice(h.metal))}>
          <div class="field">
            <label for="metal-price">{t('metal.price')}</label>
            <div class="pair">
              <input id="metal-price" inputmode="decimal" bind:value={price.amount} required />
              <select bind:value={price.unit} aria-label={t('metal.unit')}>
                <option value="oz">{t('metal.perOz')}</option>
                <option value="g">{t('metal.perG')}</option>
              </select>
            </div>
          </div>
          <label class="field">{t('overview.currency')}<input bind:value={price.currency} maxlength="3" required /></label>
          <label class="field">{t('common.date')}<input type="date" bind:value={price.date} max={today} required /></label>
          {#if price.currency.trim().toUpperCase() !== base}
            <label class="field">{t('private.rate', { currency: price.currency.trim().toUpperCase(), base })}<input inputmode="decimal" bind:value={price.fx_rate} required /></label>
          {/if}
          <button class="btn">{t('common.save')}</button>
          <button type="button" class="btn ghost" onclick={() => (pricing = null)}>{t('common.cancel')}</button>
          <p class="muted small wide">{t('metal.priceHint')}</p>
          {#if error}<p class="error wide" role="alert">{error}</p>{/if}
        </form>
      {/if}

      <details>
        <summary>{t('metal.history', { count: h.entries.length })}</summary>
        <ul class="history">
          {#each h.entries as id (id)}
            {@const e = entries.get(id)}
            {#if e}
              <li>
                <span class="num">{date(e.date)}</span>
                <span class="grow">{e.description}</span>
                <button class="btn ghost small-btn" onclick={() => remove(id)}>{t('common.remove')}</button>
              </li>
            {/if}
          {/each}
        </ul>
      </details>
    </article>
  {/each}
  {#if error && !form && !pricing}<p class="error" role="alert">{error}</p>{/if}
</section>

<style>
  h2 {
    font-size: 18px;
  }
  h3 {
    margin: 0;
    font-size: 16px;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px 12px;
  }
  @media (max-width: 720px) {
    .head .btn {
      min-height: 40px;
      padding: 0 12px;
      font-size: 14px;
    }
  }
  .small {
    font-size: 13px;
    margin: 6px 0 12px;
  }
  .block {
    display: block;
    font-size: 12px;
  }
  .add,
  .price {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 12px 16px;
    align-items: end;
    padding: 16px;
    margin-bottom: 16px;
    border-radius: 8px;
    background: var(--surface-2);
  }
  .price {
    margin: 12px 0 0;
  }
  .wide {
    grid-column: 1 / -1;
  }
  .pair {
    display: flex;
    gap: 8px;
  }
  .pair input {
    flex: 1;
    min-width: 0;
  }
  .pair select {
    width: auto;
  }
  .hint {
    font-size: 12px;
    color: var(--muted);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 12px;
  }
  .small-btn {
    min-height: 36px;
    padding: 0 12px;
    font-size: 13px;
  }
  .metal {
    padding: 14px 0;
    border-top: 1px solid var(--line);
  }
  .summary {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }
  .line {
    margin: 6px 0 10px;
  }
  .gain {
    color: var(--muted);
  }
  .loss {
    color: var(--warn);
  }
  details {
    margin-top: 10px;
    font-size: 13px;
  }
  summary {
    cursor: pointer;
    color: var(--muted);
  }
  .history {
    list-style: none;
    margin: 8px 0 0;
    padding: 0;
    display: grid;
    gap: 6px;
  }
  .history li {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
</style>
