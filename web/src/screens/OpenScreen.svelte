<script lang="ts">
  import { onMount } from 'svelte'
  import ExitLink from '../components/ExitLink.svelte'
  import Logo from '../components/Logo.svelte'
  import LanguagePicker from '../components/LanguagePicker.svelte'
  import { Book } from '../lib/core'
  import { droppedFile, openInvestFile, savesInPlace, type OpenedFile } from '../lib/files'
  import { countries } from '../lib/configs'
  import { country, dateTime } from '../lib/format'
  import { i18n, t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'
  import { clearDraft, loadDraft, type Draft } from '../lib/storage'

  type Mode = 'start' | 'restore' | 'open' | 'create'

  let mode = $state<Mode>('start')
  let draft = $state.raw<Draft | undefined>()
  let picked = $state.raw<OpenedFile | null>(null)
  let password = $state('')
  let repeat = $state('')
  let fileName = $state('portfolio.invest')
  let baseCurrency = $state('EUR')
  let residency = $state('BG')
  let busy = $state(false)
  let error = $state('')

  onMount(async () => {
    draft = await loadDraft()
    if (draft) mode = 'restore'
  })

  function describe(e: unknown): string {
    const message = e instanceof Error ? e.message : String(e)
    if (message.includes('wrong password')) return t('open.wrongPassword')
    if (message.includes('not an .invest file')) return t('open.notInvest')
    return message
  }

  async function run(action: () => Promise<void>): Promise<void> {
    error = ''
    busy = true
    // Let the "Unlocking…" state paint before the deliberately slow key derivation.
    await new Promise((resolve) => setTimeout(resolve, 30))
    try {
      await action()
      password = ''
      repeat = ''
    } catch (e) {
      error = describe(e)
    } finally {
      busy = false
    }
  }

  function go(next: Mode): void {
    mode = next
    error = ''
    password = ''
    repeat = ''
  }

  const restore = () =>
    run(async () => {
      const book = await Book.unlock(draft!.bytes, password)
      app.open(book, draft!.fileName, { changes: draft!.changes, handle: draft!.handle })
    })

  async function choose(): Promise<void> {
    error = ''
    try {
      const file = await openInvestFile()
      if (file) {
        picked = file
        go('open')
      }
    } catch (e) {
      error = describe(e)
    }
  }

  async function dropped(e: DragEvent): Promise<void> {
    e.preventDefault()
    const item = e.dataTransfer?.items[0]
    const file = item ? await droppedFile(item) : null
    if (!file) return
    picked = file
    go('open')
  }

  const unlock = () =>
    run(async () => {
      if (draft && draft.changes > 0 && !confirm(t('open.replaceDraft', { name: draft.fileName }))) return
      const book = await Book.unlock(picked!.bytes, password)
      app.open(book, picked!.name, { handle: picked!.handle })
      await app.keepDraft()
    })

  const create = () =>
    run(async () => {
      if (password.length < 10) throw new Error(t('create.tooShort'))
      if (password !== repeat) throw new Error(t('create.mismatch'))
      const book = await Book.create({ base_currency: baseCurrency, language: i18n.lang, tax_residency: residency }, password)
      const name = fileName.trim().replace(/(\.invest)?$/, '.invest')
      app.open(book, name, { changes: 1 })
      await app.keepDraft()
    })

  async function discard(): Promise<void> {
    if (!confirm(t('open.discardConfirm'))) return
    await clearDraft()
    draft = undefined
    go('start')
  }
</script>

<div class="page" role="presentation" ondragover={(e) => e.preventDefault()} ondrop={dropped}>
  <header class="site">
    <a class="brand" href="../"><Logo size={30} /> Investor</a>
    <ExitLink long />
  </header>
  <div class="wrap">
    <section class="pitch">
      <h1>{t('open.headline')}</h1>
      <ul class="facts">
        <li><strong>{t('open.noAccount')}</strong><span>{t('open.noAccountText')}</span></li>
        <li><strong>{t('open.noDatabase')}</strong><span>{t('open.noDatabaseText')}</span></li>
        <li><strong>{t('open.noTracking')}</strong><span>{t('open.noTrackingText')}</span></li>
      </ul>
      <p>{t('open.yourFile')}</p>
      <p class="small">{t('open.encryption')}</p>
    </section>

    <section class="card panel" aria-labelledby="panel-title">
      <div class="panel-head">
        <h2 id="panel-title">
          {#if mode === 'create'}{t('create.title')}{:else}{t('open.title')}{/if}
        </h2>
        <LanguagePicker light />
      </div>

      {#if mode === 'restore' && draft}
        <div class="notice">
          <strong>{t('open.draftFound')}</strong><br />
          {draft.fileName} · {t('open.draftChanges', { count: draft.changes })} · {dateTime(draft.updatedAt)}
        </div>
        <form onsubmit={(e) => (e.preventDefault(), restore())}>
          <label class="field">{t('common.password')}
            <!-- svelte-ignore a11y_autofocus -->
            <input type="password" bind:value={password} autocomplete="current-password" autofocus required />
          </label>
          <button class="btn" disabled={busy}>{busy ? t('open.unlocking') : t('open.restore')}</button>
        </form>
        <div class="links">
          <button class="link" onclick={choose}>{t('open.openOther')}</button>
          <button class="link" onclick={() => go('create')}>{t('open.createNew')}</button>
          <button class="link" onclick={discard}>{t('open.discard')}</button>
        </div>
      {:else if mode === 'open' && picked}
        <p class="file"><span class="num">{picked.name}</span></p>
        <form onsubmit={(e) => (e.preventDefault(), unlock())}>
          <label class="field">{t('common.password')}
            <!-- svelte-ignore a11y_autofocus -->
            <input type="password" bind:value={password} autocomplete="current-password" autofocus required />
          </label>
          <button class="btn" disabled={busy}>{busy ? t('open.unlocking') : t('open.unlock')}</button>
        </form>
        <div class="links">
          <button class="link" onclick={choose}>{t('open.chooseAnother')}</button>
          <button class="link" onclick={() => go(draft ? 'restore' : 'start')}>{t('common.back')}</button>
        </div>
      {:else if mode === 'create'}
        <form onsubmit={(e) => (e.preventDefault(), create())}>
          <label class="field">{t('create.fileName')}
            <input bind:value={fileName} required />
          </label>
          <div class="two fields">
            <label class="field">{t('create.baseCurrency')}
              <select bind:value={baseCurrency}>
                <option value="EUR">EUR</option>
                <option value="USD">USD</option>
                <option value="GBP">GBP</option>
                <option value="CAD">CAD</option>
              </select>
            </label>
            <label class="field">{t('create.residency')}
              <select bind:value={residency}>
                {#each countries() as c (c)}<option value={c}>{country(c)}</option>{/each}
              </select>
            </label>
          </div>
          <label class="field">{t('common.password')}
            <input type="password" bind:value={password} autocomplete="new-password" required />
          </label>
          <label class="field">{t('create.repeat')}
            <input type="password" bind:value={repeat} autocomplete="new-password" required />
          </label>
          <p class="small muted">{t('create.passwordHint')}</p>
          <button class="btn" disabled={busy}>{busy ? t('create.creating') : t('create.submit')}</button>
        </form>
        <div class="links">
          <button class="link" onclick={() => go(draft ? 'restore' : 'start')}>{t('common.back')}</button>
        </div>
      {:else}
        <div class="choices">
          <button class="btn" onclick={choose}>{t('open.openFile')}</button>
          <button class="btn ghost" onclick={() => go('create')}>{t('open.createNew')}</button>
          <button class="link example" onclick={() => app.openExample(i18n.lang)}>{t('example.open')}</button>
        </div>
        <p class="small muted">{t('open.dropHint')}</p>
      {/if}

      {#if error}<p class="error" role="alert">{error}</p>{/if}

      <p class="small muted where">{t('open.whereToKeep')}</p>
      <p class="small muted">{savesInPlace ? t('open.savesInPlace') : t('open.downloads')}</p>
    </section>
  </div>
</div>

<style>
  .page {
    min-height: 100vh;
    background: var(--side);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 32px;
    padding: 20px 16px 48px;
  }
  .site {
    width: 100%;
    max-width: 1000px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .wrap {
    margin-block: auto;
  }
  .wrap {
    width: 100%;
    max-width: 1000px;
    display: flex;
    flex-wrap: wrap;
    gap: 40px;
  }
  .pitch {
    flex: 1 1 360px;
    color: var(--side-text);
    display: flex;
    flex-direction: column;
    gap: 20px;
    justify-content: center;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    text-decoration: none;
    color: #fff;
    font-weight: 600;
    font-size: 22px;
  }
  h1 {
    color: #fff;
    font-size: clamp(28px, 4vw, 40px);
    line-height: 1.15;
  }
  .facts {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
  }
  .facts li {
    background: var(--side-2);
    border-radius: 12px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
    line-height: 1.45;
  }
  .facts strong {
    color: #fff;
    font-size: 16px;
  }
  .pitch p {
    margin: 0;
    line-height: 1.6;
  }
  .panel {
    flex: 1 1 380px;
    padding: 28px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    align-self: center;
  }
  .panel-head {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .two {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .choices {
    display: grid;
    gap: 10px;
  }
  .links {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
  }
  .link {
    background: none;
    border: 0;
    padding: 8px 0;
    color: var(--accent);
    font: 500 14px var(--sans);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .file {
    margin: 0;
    padding: 12px;
    background: var(--surface-2);
    border-radius: 8px;
  }
  .small {
    font-size: 13px;
    line-height: 1.5;
    margin: 0;
  }
  .where {
    border-top: 1px solid var(--line-soft);
    padding-top: 14px;
  }
  /* On a phone the actions come right after the headline; the promises follow. */
  @media (max-width: 720px) {
    .page {
      align-items: flex-start;
      padding: 24px 16px 32px;
    }
    .wrap {
      flex-direction: column;
      flex-wrap: nowrap;
      gap: 20px;
    }
    .pitch {
      display: contents;
    }
    .brand {
      order: 0;
    }
    h1 {
      order: 1;
    }
    .panel {
      order: 2;
      flex: none;
      align-self: stretch;
      padding: 20px;
    }
    .facts {
      order: 3;
    }
    .pitch p {
      order: 4;
      color: var(--side-text);
    }
  }
  .example {
    min-height: 44px;
    align-self: center;
  }
</style>
