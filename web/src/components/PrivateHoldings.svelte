<script lang="ts">
  // Private holdings: money in something without a market price, such as an
  // employer co-investment. Added here, valued at cost or at a value set by
  // hand, and left out of projections.
  import type { Payout, PrivateHolding } from '../lib/core'
  import { date, decimal, money } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const today = new Date().toISOString().slice(0, 10)
  const PAYOUTS: Payout[] = ['employment_income', 'capital_gains', 'unknown']

  const blank = () => ({ entry_id: crypto.randomUUID(), name: '', date: today, amount: '', currency: base, fx_rate: '', paid_from: '', payout: 'unknown' as Payout, condition: '' })
  let adding = $state(false)
  let form = $state(blank())
  let error = $state('')

  const currency = $derived(form.currency.trim().toUpperCase())
  const knownRate = $derived(s.fx.find(([c]) => c === currency)?.[1].rate)
  $effect(() => {
    if (currency !== base && !form.fx_rate && knownRate) form.fx_rate = String(Number(knownRate))
  })

  function open(): void {
    form = blank()
    error = ''
    adding = true
  }

  async function add(): Promise<void> {
    error = ''
    try {
      await app.addPrivate({
        ...form,
        currency,
        amount: decimal(form.amount),
        fx_rate: currency !== base && decimal(form.fx_rate) ? decimal(form.fx_rate) : null,
      })
      adding = false
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  // Value editor
  let editing = $state<string | null>(null)
  let value = $state('')
  let valueDate = $state(today)

  function editValue(h: PrivateHolding): void {
    editing = h.id
    value = h.estimate?.amount ?? ''
    valueDate = today
    error = ''
  }

  async function saveValue(id: string, amount: string): Promise<void> {
    try {
      await app.setPrivateValue(id, valueDate, decimal(amount))
      editing = null
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  async function remove(h: PrivateHolding): Promise<void> {
    if (!confirm(t('private.removeConfirm', { name: h.name }))) return
    await app.removePrivate(h.id)
  }
</script>

<section class="card" aria-labelledby="private-title">
  <div class="head">
    <h2 id="private-title">{t('private.title')}</h2>
    {#if !adding}<button class="btn ghost" onclick={open}>{t('private.add')}</button>{/if}
  </div>
  <p class="muted small">{t('private.intro')}</p>

  {#if adding}
    <form class="add" onsubmit={(e) => (e.preventDefault(), add())}>
      <label class="field wide">{t('private.name')}<input bind:value={form.name} placeholder={t('private.namePlaceholder')} required /></label>
      <label class="field">{t('private.paidOn')}<input type="date" bind:value={form.date} max={today} required /></label>
      <label class="field">{t('private.amount')}<input inputmode="decimal" bind:value={form.amount} required /></label>
      <label class="field">{t('overview.currency')}<input bind:value={form.currency} maxlength="3" required /></label>
      {#if currency !== base && currency.length === 3}
        <label class="field">{t('private.rate', { currency, base })}<input inputmode="decimal" bind:value={form.fx_rate} required /><span class="hint">{t('private.rateHint')}</span></label>
      {/if}
      <label class="field">{t('private.paidFrom')}<input bind:value={form.paid_from} placeholder={t('private.paidFromPlaceholder')} /><span class="hint">{t('private.paidFromHint')}</span></label>
      <label class="field">{t('private.payout')}
        <select bind:value={form.payout}>
          {#each PAYOUTS as p (p)}<option value={p}>{t(`private.payout.${p}`)}</option>{/each}
        </select>
      </label>
      <label class="field wide">{t('private.condition')}<input bind:value={form.condition} placeholder={t('private.conditionPlaceholder')} /></label>
      <div class="actions wide">
        <button class="btn">{t('private.save')}</button>
        <button type="button" class="btn ghost" onclick={() => (adding = false)}>{t('common.cancel')}</button>
      </div>
      {#if error && !editing}<p class="error wide" role="alert">{error}</p>{/if}
    </form>
  {/if}

  {#if s.private.length}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>{t('private.holding')}</th><th>{t('private.paidOn')}</th><th class="right">{t('private.paid')}</th>
            <th class="right">{t('overview.value', { currency: base })}</th><th>{t('private.payoutShort')}</th><th></th>
          </tr>
        </thead>
        <tbody>
          {#each s.private as h (h.id)}
            <tr>
              <td>
                <strong>{h.name}</strong>
                {#if h.condition}<span class="muted block">{h.condition}</span>{/if}
              </td>
              <td class="num">{h.since ? date(h.since) : '—'}{#if h.paid_from}<span class="muted block">{t('private.from', { from: h.paid_from })}</span>{/if}</td>
              <td class="right num">{money(h.paid, h.currency)}{#if h.currency !== base}<span class="muted block">{money(h.cost, base)}</span>{/if}</td>
              <td class="right num">
                {#if h.value !== undefined && h.estimate}
                  {money(h.value, base)}<span class="muted block">{t('private.setOn', { amount: money(h.estimate.amount, h.currency), date: date(h.estimate.date) })}</span>
                {:else if h.estimate}
                  {t('overview.needsRate')}
                {:else}
                  {money(h.cost, base)}<span class="muted block">{t('private.atCost')}</span>
                {/if}
              </td>
              <td><span class="pill" class:warn={h.payout === 'unknown'}>{t(`private.payout.${h.payout}`)}</span></td>
              <td class="right nowrap">
                <button class="btn ghost small-btn" onclick={() => editValue(h)}>{t('private.setValue')}</button>
                <button class="btn ghost small-btn" onclick={() => remove(h)}>{t('common.remove')}</button>
              </td>
            </tr>
            {#if editing === h.id}
              <tr class="editor">
                <td colspan="6">
                  <form onsubmit={(e) => (e.preventDefault(), saveValue(h.id, value))}>
                    <label class="field">{t('private.valueIn', { currency: h.currency })}<input inputmode="decimal" bind:value required /></label>
                    <label class="field">{t('common.date')}<input type="date" bind:value={valueDate} max={today} required /></label>
                    <button class="btn">{t('common.save')}</button>
                    {#if h.estimate}<button type="button" class="btn ghost" onclick={() => saveValue(h.id, '')}>{t('private.backToCost')}</button>{/if}
                    <button type="button" class="btn ghost" onclick={() => (editing = null)}>{t('common.cancel')}</button>
                  </form>
                  <p class="muted small">{t('private.valueHint')}</p>
                  {#if error}<p class="error" role="alert">{error}</p>{/if}
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>
    <p class="muted small">{t('private.outsideProjections')}</p>
  {/if}
</section>

<style>
  h2 {
    font-size: 18px;
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
    .nowrap {
      white-space: normal;
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
  .nowrap {
    white-space: nowrap;
  }
  .add {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 12px 16px;
    padding: 16px;
    margin-bottom: 16px;
    border-radius: 8px;
    background: var(--surface-2);
  }
  .wide {
    grid-column: 1 / -1;
  }
  .actions {
    display: flex;
    gap: 12px;
  }
  .hint {
    font-weight: 400;
    color: var(--muted);
  }
  .small-btn {
    min-height: 36px;
    padding: 0 12px;
    font-size: 13px;
  }
  .editor td {
    background: var(--surface-2);
  }
  .editor form {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: flex-end;
  }
  .editor .field {
    width: 180px;
  }
</style>
