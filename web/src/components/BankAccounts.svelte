<script lang="ts">
  // Bank accounts kept by hand: no statement to import, so the balance is
  // typed in now and then. Their money is cash like a broker's: it fills the
  // emergency fund and counts in net worth and projections.
  import type { BankAccount } from '../lib/core'
  import { date, decimal, money } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const today = new Date().toISOString().slice(0, 10)
  const rateOf = (currency: string) => s.fx.find(([c]) => c === currency)?.[1].rate

  let error = $state('')

  // New account
  const blank = () => ({ entry_id: crypto.randomUUID(), name: '', currency: base, date: today, balance: '', fx_rate: '' })
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
    editing = null
    adding = true
  }

  async function add(): Promise<void> {
    error = ''
    try {
      await app.addBank({
        entry_id: form.entry_id,
        name: form.name,
        currency,
        date: form.date,
        balance: decimal(form.balance),
        fx_rate: currency !== base && decimal(form.fx_rate) ? decimal(form.fx_rate) : null,
      })
      adding = false
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  // Balance editor
  let editing = $state<string | null>(null)
  let update = $state({ entry_id: '', date: today, balance: '', fx_rate: '' })

  function edit(a: BankAccount): void {
    editing = a.id
    renaming = null
    adding = false
    error = ''
    update = { entry_id: crypto.randomUUID(), date: today, balance: a.balance, fx_rate: a.currency !== base ? String(Number(rateOf(a.currency) ?? '')) || '' : '' }
  }

  async function save(a: BankAccount): Promise<void> {
    error = ''
    try {
      await app.setBankBalance(a.id, {
        entry_id: update.entry_id,
        date: update.date,
        balance: decimal(update.balance),
        fx_rate: a.currency !== base && decimal(update.fx_rate) ? decimal(update.fx_rate) : null,
      })
      editing = null
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  // Renaming changes only the name shown; the account and its history stay.
  let renaming = $state<string | null>(null)
  let newName = $state('')

  function startRename(a: BankAccount): void {
    renaming = a.id
    editing = null
    adding = false
    error = ''
    newName = a.name
  }

  async function rename(a: BankAccount): Promise<void> {
    error = ''
    try {
      await app.renameBank(a.id, newName)
      renaming = null
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
    }
  }

  async function remove(a: BankAccount): Promise<void> {
    if (!confirm(t('bank.removeConfirm', { name: a.name }))) return
    await app.removeBank(a.id)
  }
</script>

<section class="card" aria-labelledby="bank-title">
  <div class="head">
    <h2 id="bank-title">{t('bank.title')}</h2>
    {#if !adding}<button class="btn ghost" onclick={open}>{t('bank.add')}</button>{/if}
  </div>
  <p class="muted small">{t('bank.intro')}</p>

  {#if adding}
    <form class="add fields" onsubmit={(e) => (e.preventDefault(), add())}>
      <label class="field">{t('bank.name')}<input bind:value={form.name} placeholder={t('bank.namePlaceholder')} required /></label>
      <label class="field">{t('overview.currency')}<input bind:value={form.currency} maxlength="3" required /></label>
      <label class="field">{t('bank.balance')}<input inputmode="decimal" bind:value={form.balance} required /></label>
      <label class="field">{t('bank.on')}<input type="date" bind:value={form.date} max={today} required /></label>
      {#if currency !== base && currency.length === 3}
        <label class="field">{t('private.rate', { currency, base })}<input inputmode="decimal" bind:value={form.fx_rate} required /></label>
      {/if}
      <div class="actions wide">
        <button class="btn">{t('bank.save')}</button>
        <button type="button" class="btn ghost" onclick={() => (adding = false)}>{t('common.cancel')}</button>
      </div>
      {#if error}<p class="error wide" role="alert">{error}</p>{/if}
    </form>
  {/if}

  {#if s.banks.length}
    <div class="table-wrap">
      <table>
        <thead>
          <tr><th>{t('bank.account')}</th><th>{t('bank.updated')}</th><th class="right">{t('bank.balance')}</th><th></th></tr>
        </thead>
        <tbody>
          {#each s.banks as a (a.id)}
            <tr>
              <td><strong>{a.name}</strong></td>
              <td class="num">{a.updated ? date(a.updated) : '—'}</td>
              <td class="right num">{money(a.balance, a.currency)}{#if a.currency !== base && a.value !== undefined}<span class="muted block">{money(a.value, base)}</span>{/if}</td>
              <td class="right nowrap">
                <button class="btn ghost small-btn" onclick={() => edit(a)}>{t('bank.update')}</button>
                <button class="btn ghost small-btn" onclick={() => startRename(a)}>{t('bank.rename')}</button>
                <button class="btn ghost small-btn" onclick={() => remove(a)}>{t('common.remove')}</button>
              </td>
            </tr>
            {#if renaming === a.id}
              <tr class="editor">
                <td colspan="4">
                  <form onsubmit={(e) => (e.preventDefault(), rename(a))}>
                    <label class="field wide-name">{t('bank.name')}<input bind:value={newName} required /></label>
                    <button class="btn">{t('common.save')}</button>
                    <button type="button" class="btn ghost" onclick={() => (renaming = null)}>{t('common.cancel')}</button>
                  </form>
                  <p class="muted small">{t('bank.renameHint')}</p>
                  {#if error}<p class="error" role="alert">{error}</p>{/if}
                </td>
              </tr>
            {/if}
            {#if editing === a.id}
              <tr class="editor">
                <td colspan="4">
                  <form onsubmit={(e) => (e.preventDefault(), save(a))}>
                    <label class="field">{t('bank.balanceIn', { currency: a.currency })}<input inputmode="decimal" bind:value={update.balance} required /></label>
                    <label class="field">{t('bank.on')}<input type="date" bind:value={update.date} max={today} required /></label>
                    {#if a.currency !== base}
                      <label class="field">{t('private.rate', { currency: a.currency, base })}<input inputmode="decimal" bind:value={update.fx_rate} required /></label>
                    {/if}
                    <button class="btn">{t('common.save')}</button>
                    <button type="button" class="btn ghost" onclick={() => (editing = null)}>{t('common.cancel')}</button>
                  </form>
                  <p class="muted small">{t('bank.updateHint')}</p>
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
  .editor .wide-name {
    width: min(100%, 320px);
  }
</style>
