<script lang="ts">
  import { describe } from '../lib/describe'
  import { date, money, units } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  let tab = $state<'journal' | 'trial'>('journal')
  let query = $state('')

  const entries = $derived.by(() => {
    const q = query.trim().toLowerCase()
    if (!q) return s.entries
    return s.entries.filter(
      (e) => e.description.toLowerCase().includes(q) || describe(e.description).toLowerCase().includes(q) || e.id.toLowerCase().includes(q) || e.postings.some((p) => p.account.toLowerCase().includes(q)),
    )
  })
  const totals = $derived(
    s.trial_balance.reduce((sum, r) => ({ debit: sum.debit + Number(r.debit), credit: sum.credit + Number(r.credit) }), { debit: 0, credit: 0 }),
  )
</script>

<header>
  <h1>{t('nav.ledger')}</h1>
  <p class="muted">{t('ledger.subtitle')}</p>
</header>

<div class="tabs" role="tablist" aria-label={t('ledger.views')}>
  <button role="tab" aria-selected={tab === 'journal'} class:on={tab === 'journal'} onclick={() => (tab = 'journal')}>{t('ledger.journal')}</button>
  <button role="tab" aria-selected={tab === 'trial'} class:on={tab === 'trial'} onclick={() => (tab = 'trial')}>{t('ledger.trialBalance')}</button>
</div>

{#if tab === 'journal'}
  <label class="filter">
    <span class="sr-only">{t('ledger.filter')}</span>
    <input class="input" type="search" placeholder={t('ledger.filterPlaceholder')} bind:value={query} />
  </label>
  {#each entries as e (e.id)}
    <article class="card entry">
      <div class="entry-head">
        <span><span class="muted num">{e.id} · {date(e.date)}</span> · <strong>{describe(e.description)}</strong></span>
        <span class="tags">
          {#if e.venue}<span class="pill" class:warn={e.venue.assumed}>{t('ledger.venue', { code: e.venue.code })}</span>{/if}
          {#if e.source}<span class="pill plain">{t('ledger.source', { file: e.source.file, line: e.source.line })}</span>{/if}
        </span>
      </div>
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th>{t('ledger.account')}</th><th class="right">{t('ledger.amount')}</th><th class="right">{t('ledger.debit')}</th><th class="right">{t('ledger.credit')}</th></tr>
          </thead>
          <tbody>
            {#each e.postings as p, i (i)}
              {@const value = Number(p.base)}
              <tr>
                <td class:credit={value < 0}>
                  <span class="num">{p.account}</span>
                  {#if p.units}<span class="muted"> · {t('ledger.units', { units: units(Math.abs(Number(p.units))) })}</span>{/if}
                </td>
                <td class="right num muted">{p.amount.currency !== s.settings.base_currency ? money(p.amount.amount, p.amount.currency) : ''}</td>
                <td class="right num">{value > 0 ? money(value) : ''}</td>
                <td class="right num">{value < 0 ? money(-value) : ''}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      {#each e.notes ?? [] as note, i (i)}
        <p class="note">{describe(note)}</p>
      {/each}
    </article>
  {:else}
    <p class="muted">{query ? t('ledger.noMatch') : t('ledger.empty')}</p>
  {/each}
{:else}
  <section class="card">
    <div class="table-wrap">
      <table>
        <thead>
          <tr><th>{t('ledger.account')}</th><th class="right">{t('ledger.debit')}</th><th class="right">{t('ledger.credit')}</th></tr>
        </thead>
        <tbody>
          {#each s.trial_balance as r (r.account)}
            <tr>
              <td class="num">{r.account}</td>
              <td class="right num">{Number(r.debit) ? money(r.debit) : ''}</td>
              <td class="right num">{Number(r.credit) ? money(r.credit) : ''}</td>
            </tr>
          {/each}
          <tr class="total">
            <td>{t('ledger.total')}</td>
            <td class="right num">{money(totals.debit)}</td>
            <td class="right num">{money(totals.credit)}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <p class="balanced">{Math.abs(totals.debit - totals.credit) < 0.005 ? t('ledger.inBalance') : t('ledger.outOfBalance')}</p>
  </section>
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
  header p {
    margin: 0;
  }
  .tabs {
    display: flex;
    gap: 4px;
    border-bottom: 1px solid var(--line);
  }
  .tabs button {
    min-height: 44px;
    padding: 0 14px;
    border: 0;
    border-bottom: 2px solid transparent;
    background: none;
    color: var(--muted);
    font: 500 14px var(--sans);
    cursor: pointer;
  }
  .tabs button.on {
    color: var(--ink);
    border-bottom-color: var(--accent);
    font-weight: 600;
  }
  .filter {
    max-width: 360px;
  }
  .entry {
    padding: 0;
    overflow: hidden;
  }
  .entry-head {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 8px;
    padding: 12px 16px;
    background: var(--surface-2);
    border-bottom: 1px solid var(--line-soft);
    font-size: 14px;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  td.credit {
    padding-left: 32px;
  }
  .note {
    margin: 0;
    padding: 10px 16px;
    font-size: 13px;
    color: var(--warn-ink);
    background: var(--warn-soft);
    border-top: 1px solid var(--warn-line);
  }
  .total td {
    font-weight: 600;
    border-bottom: 0;
  }
  .balanced {
    margin: 12px 0 0;
    font-weight: 600;
    color: var(--accent);
  }
</style>
