<script lang="ts">
  import { brokers } from '../lib/configs'
  import { brokerFit, type ImportReport } from '../lib/core'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  interface Pending {
    name: string
    text: string
    brokerId: string
    fit: number
    report?: ImportReport
    error?: string
  }

  let items = $state<Pending[]>([])
  let dragging = $state(false)

  async function add(files: File[]): Promise<void> {
    for (const file of files) {
      const text = await file.text()
      const scored = brokers.map((b) => ({ id: b.id, fit: brokerFit(b.toml, text) })).sort((a, b) => b.fit - a.fit)
      const best = scored[0]
      items.push({ name: file.name, text, brokerId: best && best.fit >= 0.5 ? best.id : '', fit: best?.fit ?? 0 })
    }
  }

  async function run(item: Pending): Promise<void> {
    const broker = brokers.find((b) => b.id === item.brokerId)
    if (!broker || !app.book) return
    item.error = undefined
    try {
      item.report = app.book.import(broker.toml, item.name, item.text)
      if (item.report.added > 0) await app.changed(item.report.added)
    } catch (e) {
      item.error = e instanceof Error ? e.message : String(e)
    }
  }

  async function runAll(): Promise<void> {
    for (const item of items.filter((i) => !i.report && i.brokerId)) await run(item)
  }

  function drop(e: DragEvent): void {
    e.preventDefault()
    dragging = false
    void add([...(e.dataTransfer?.files ?? [])])
  }
</script>

<header>
  <h1>{t('nav.import')}</h1>
  <p class="muted">{t('import.subtitle')}</p>
</header>

<label
  class="drop"
  class:dragging
  ondragover={(e) => (e.preventDefault(), (dragging = true))}
  ondragleave={() => (dragging = false)}
  ondrop={drop}
>
  <input
    class="sr-only"
    type="file"
    accept=".csv,.txt"
    multiple
    onchange={(e) => {
      void add([...(e.currentTarget.files ?? [])])
      e.currentTarget.value = ''
    }}
  />
  <strong>{t('import.drop')}</strong>
  <span class="muted">{t('import.supported', { list: brokers.map((b) => b.name).join(', ') })}</span>
</label>

{#if items.length}
  <section class="card list" aria-label={t('import.files')}>
    {#each items as item, i (i)}
      <div class="item">
        <div class="item-head">
          <strong class="num">{item.name}</strong>
          <label class="broker">
            <span>{t('import.format')}</span>
            <select class="input" bind:value={item.brokerId} disabled={!!item.report}>
              <option value="">{t('import.chooseFormat')}</option>
              {#each brokers as b (b.id)}<option value={b.id}>{b.name}</option>{/each}
            </select>
          </label>
          {#if !item.report}
            <button class="btn" disabled={!item.brokerId} onclick={() => run(item)}>{t('import.import')}</button>
          {/if}
        </div>
        {#if !item.brokerId && !item.report}
          <p class="notice">
            {t('import.unknownFormat')}
            <button class="inline-link" onclick={() => app.openGuide('add-broker')}>{t('import.guideMissing')}</button>
          </p>
        {/if}
        {#if item.report}
          <p class="result">{t('import.result', { added: item.report.added, duplicates: item.report.duplicates })}</p>
          {#each item.report.warnings as warning, w (w)}
            <p class="notice">{warning}</p>
          {/each}
        {/if}
        {#if item.error}<p class="error" role="alert">{item.error}</p>{/if}
      </div>
    {/each}
    {#if items.some((i) => !i.report && i.brokerId)}
      <button class="btn" onclick={runAll}>{t('import.importAll')}</button>
    {/if}
  </section>
{/if}

<section class="card help">
  <h2>{t('import.howTitle')}</h2>
  <p>{t('import.howText')}</p>
  <div class="links">
    <button class="btn ghost" onclick={() => app.openGuide('export-ibkr')}>{t('import.guideIbkr')}</button>
    <button class="btn ghost" onclick={() => app.openGuide('export-revolut')}>{t('import.guideRevolut')}</button>
    <button class="btn ghost" onclick={() => app.openGuide('add-broker')}>{t('import.guideMissing')}</button>
  </div>
</section>

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
    font-size: 16px;
  }
  header p {
    margin: 0;
  }
  .drop {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
    justify-content: center;
    min-height: 140px;
    padding: 24px;
    border: 2px dashed var(--field);
    border-radius: 12px;
    background: var(--surface);
    color: var(--ink);
    font: 15px var(--sans);
    cursor: pointer;
    text-align: center;
  }
  .drop.dragging,
  .drop:hover,
  .drop:focus-within {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 16px;
    align-items: flex-start;
  }
  .item {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--line-soft);
  }
  .item-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
  }
  .item-head strong {
    flex: 1 1 200px;
    overflow-wrap: anywhere;
  }
  .broker {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--muted);
  }
  .broker select {
    width: auto;
  }
  .item p {
    margin: 0;
  }
  .result {
    font-weight: 600;
    color: var(--accent);
  }
  .links {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .inline-link {
    background: none;
    border: 0;
    padding: 0;
    color: var(--accent);
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
  }
  .help p {
    margin: 8px 0;
    line-height: 1.55;
    color: var(--ink-2);
  }
</style>
