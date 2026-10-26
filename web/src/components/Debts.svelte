<script lang="ts">
  import RowMenu from './RowMenu.svelte'
  // Debts kept by hand: a mortgage, a car loan, a credit card. What is still
  // owed is typed in now and then; it comes off net worth, and paying it
  // down counts as money put in.
  import type { Debt, DebtKind } from '../lib/core'
  import { date, decimal, money } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const KINDS: DebtKind[] = ['mortgage', 'car', 'personal', 'student', 'credit_card', 'family', 'other']

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const today = new Date().toISOString().slice(0, 10)
  const rateOf = (currency: string) => s.fx.find(([c]) => c === currency)?.[1].rate

  let error = $state('')

  // A rate typed as a percentage, kept as a fraction.
  const toFraction = (percent: string) => (decimal(percent) ? String(Number(decimal(percent)) / 100) : null)
  const toPercent = (fraction: string | undefined) => (fraction ? String(+(Number(fraction) * 100).toFixed(3)) : '')

  // New debt
  const blank = () => ({ entry_id: crypto.randomUUID(), name: '', kind: 'mortgage' as DebtKind, currency: base, date: today, owed: '', rate: '', monthly: '', fx_rate: '' })
  let adding = $state(false)
  let form = $state(blank())
  const currency = $derived(form.currency.trim().toUpperCase())
  $effect(() => {
    const known = rateOf(currency)
    if (currency !== base && !form.fx_rate && known) form.fx_rate = String(Number(known))
  })

  function open(): void {
    form = blank()
    error = ''
    editing = updating = null
    adding = true
  }

  async function add(): Promise<void> {
    error = ''
    try {
      await app.addDebt({
        entry_id: form.entry_id,
        name: form.name,
        kind: form.kind,
        currency,
        date: form.date,
        balance: decimal(form.owed),
        rate: toFraction(form.rate),
        monthly: decimal(form.monthly) || null,
        fx_rate: currency !== base && decimal(form.fx_rate) ? decimal(form.fx_rate) : null,
      })
      adding = false
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  // What is owed now
  let updating = $state<string | null>(null)
  let update = $state({ entry_id: '', date: today, owed: '', fx_rate: '' })

  function startUpdate(d: Debt): void {
    updating = d.id
    editing = null
    adding = false
    error = ''
    update = { entry_id: crypto.randomUUID(), date: today, owed: d.owed, fx_rate: d.currency !== base ? String(Number(rateOf(d.currency) ?? '')) || '' : '' }
  }

  async function saveOwed(d: Debt): Promise<void> {
    error = ''
    try {
      await app.setDebtOwed(d.id, {
        entry_id: update.entry_id,
        date: update.date,
        balance: decimal(update.owed),
        fx_rate: d.currency !== base && decimal(update.fx_rate) ? decimal(update.fx_rate) : null,
      })
      updating = null
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  // Name, kind, rate and payment
  let editing = $state<string | null>(null)
  let terms = $state({ name: '', kind: 'other' as DebtKind, rate: '', monthly: '' })

  function startEdit(d: Debt): void {
    editing = d.id
    updating = null
    adding = false
    error = ''
    terms = { name: d.name, kind: d.kind, rate: toPercent(d.rate), monthly: d.monthly ?? '' }
  }

  async function saveTerms(d: Debt): Promise<void> {
    error = ''
    try {
      await app.editDebt(d.id, { name: terms.name, kind: terms.kind, rate: toFraction(terms.rate), monthly: decimal(terms.monthly) || null })
      editing = null
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  async function remove(d: Debt): Promise<void> {
    if (!confirm(t('debt.removeConfirm', { name: d.name }))) return
    await app.removeDebt(d.id)
  }

  /** "2 years 5 months", or when the payment does not cover the interest. */
  function paidOff(d: Debt): string {
    if (Number(d.owed) === 0) return t('debt.paidOff')
    if (!d.monthly) return '—'
    if (d.months_left === undefined) return t('debt.never')
    const years = Math.floor(d.months_left / 12)
    const months = d.months_left % 12
    return t('debt.inTime', { years, months })
  }
</script>

<section class="card" aria-labelledby="debt-title">
  <div class="head">
    <h2 id="debt-title">{t('debt.title')}</h2>
    {#if !adding}<button class="btn ghost" onclick={open}>{t('debt.add')}</button>{/if}
  </div>
  <p class="muted small">{t('debt.intro')}</p>

  {#if adding}
    <form class="add fields" onsubmit={(e) => (e.preventDefault(), add())}>
      <label class="field">{t('debt.name')}<input bind:value={form.name} placeholder={t('debt.namePlaceholder')} required /></label>
      <label class="field">{t('debt.kind')}
        <select bind:value={form.kind}>
          {#each KINDS as k (k)}<option value={k}>{t(`debt.kind.${k}`)}</option>{/each}
        </select>
      </label>
      <label class="field">{t('overview.currency')}<input bind:value={form.currency} maxlength="3" required /></label>
      <label class="field">{t('debt.owed')}<input inputmode="decimal" bind:value={form.owed} required /></label>
      <label class="field">{t('debt.rate')}<input inputmode="decimal" bind:value={form.rate} placeholder="6.9" /><span class="hint">{t('debt.rateHint')}</span></label>
      <label class="field">{t('debt.monthly')}<input inputmode="decimal" bind:value={form.monthly} /></label>
      <label class="field">{t('bank.on')}<input type="date" bind:value={form.date} max={today} required /></label>
      {#if currency !== base && currency.length === 3}
        <label class="field">{t('private.rate', { currency, base })}<input inputmode="decimal" bind:value={form.fx_rate} required /></label>
      {/if}
      <div class="actions wide">
        <button class="btn">{t('debt.save')}</button>
        <button type="button" class="btn ghost" onclick={() => (adding = false)}>{t('common.cancel')}</button>
      </div>
      {#if error}<p class="error wide" role="alert">{error}</p>{/if}
    </form>
  {/if}

  {#if s.debts.length}
    <div class="table-wrap">
      <table>
        <thead>
          <tr><th>{t('debt.debt')}</th><th class="right">{t('debt.rate')}</th><th class="right">{t('debt.monthly')}</th><th>{t('debt.paidOffIn')}</th><th class="right">{t('debt.owed')}</th><th></th></tr>
        </thead>
        <tbody>
          {#each s.debts as d (d.id)}
            <tr>
              <td><strong>{d.name}</strong><span class="muted block">{t(`debt.kind.${d.kind}`)}{d.updated ? ` · ${date(d.updated)}` : ''}</span></td>
              <td class="right num">{d.rate ? `${toPercent(d.rate)}%` : '—'}</td>
              <td class="right num">{d.monthly ? money(d.monthly, d.currency) : '—'}</td>
              <td class:warn-text={d.monthly && d.months_left === undefined && Number(d.owed) > 0}>{paidOff(d)}</td>
              <td class="right num">{money(d.owed, d.currency)}{#if d.currency !== base && d.value !== undefined}<span class="muted block">{money(d.value, base)}</span>{/if}</td>
              <td class="right">
                <RowMenu
                  label={t('common.moreFor', { name: d.name })}
                  items={[
                    { label: t('debt.update'), run: () => startUpdate(d) },
                    { label: t('debt.edit'), run: () => startEdit(d) },
                    { label: t('common.remove'), run: () => remove(d), danger: true },
                  ]}
                />
              </td>
            </tr>
            {#if updating === d.id}
              <tr class="editor">
                <td colspan="6">
                  <form onsubmit={(e) => (e.preventDefault(), saveOwed(d))}>
                    <label class="field">{t('debt.owedIn', { currency: d.currency })}<input inputmode="decimal" bind:value={update.owed} required /></label>
                    <label class="field">{t('bank.on')}<input type="date" bind:value={update.date} max={today} required /></label>
                    {#if d.currency !== base}
                      <label class="field">{t('private.rate', { currency: d.currency, base })}<input inputmode="decimal" bind:value={update.fx_rate} required /></label>
                    {/if}
                    <button class="btn">{t('common.save')}</button>
                    <button type="button" class="btn ghost" onclick={() => (updating = null)}>{t('common.cancel')}</button>
                  </form>
                  <p class="muted small">{t('debt.updateHint')}</p>
                  {#if error}<p class="error" role="alert">{error}</p>{/if}
                </td>
              </tr>
            {/if}
            {#if editing === d.id}
              <tr class="editor">
                <td colspan="6">
                  <form onsubmit={(e) => (e.preventDefault(), saveTerms(d))}>
                    <label class="field wide-name">{t('debt.name')}<input bind:value={terms.name} required /></label>
                    <label class="field">{t('debt.kind')}
                      <select bind:value={terms.kind}>
                        {#each KINDS as k (k)}<option value={k}>{t(`debt.kind.${k}`)}</option>{/each}
                      </select>
                    </label>
                    <label class="field">{t('debt.rate')}<input inputmode="decimal" bind:value={terms.rate} /></label>
                    <label class="field">{t('debt.monthly')}<input inputmode="decimal" bind:value={terms.monthly} /></label>
                    <button class="btn">{t('common.save')}</button>
                    <button type="button" class="btn ghost" onclick={() => (editing = null)}>{t('common.cancel')}</button>
                  </form>
                  {#if error}<p class="error" role="alert">{error}</p>{/if}
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>
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
  }
  .small {
    font-size: 13px;
    margin: 6px 0 12px;
  }
  .block {
    display: block;
    font-size: 12px;
  }
  .hint {
    font-weight: 400;
    font-size: 12px;
    color: var(--muted);
  }
  .warn-text {
    color: var(--warn);
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
  .editor .wide-name {
    width: min(100%, 320px);
  }
</style>
