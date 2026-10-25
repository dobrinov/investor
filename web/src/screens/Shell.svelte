<script lang="ts">
  import LanguagePicker from '../components/LanguagePicker.svelte'
  import Logo from '../components/Logo.svelte'
  import { t } from '../lib/i18n.svelte'
  import { app, type View } from '../lib/state.svelte'
  import Import from './Import.svelte'
  import Ledger from './Ledger.svelte'
  import Overview from './Overview.svelte'
  import Tax from './Tax.svelte'

  const IDLE_LOCK_MS = 15 * 60 * 1000

  const nav: { view: View; label: () => string; icon: string }[] = [
    { view: 'overview', label: () => t('nav.overview'), icon: 'M3 3h7v9H3zM14 3h7v5h-7zM14 12h7v9h-7zM3 16h7v5H3z' },
    { view: 'ledger', label: () => t('nav.ledger'), icon: 'M4 4h12a4 4 0 0 1 4 4v12H8a4 4 0 0 1-4-4zM8 9h8M8 13h8' },
    { view: 'import', label: () => t('nav.import'), icon: 'M12 3v12M7 10l5 5 5-5M4 19h16' },
    { view: 'tax', label: () => t('nav.tax'), icon: 'M6 3h9l4 4v14H6zM9 12h7M9 16h7M14 3v5h5' },
  ]

  let saving = $state(false)
  let saveError = $state('')

  async function save(): Promise<void> {
    saving = true
    saveError = ''
    try {
      await app.save()
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e)
    } finally {
      saving = false
    }
  }

  function lock(): void {
    if (app.changes > 0 && !confirm(t('shell.lockUnsaved'))) return
    app.lock()
  }

  // Lock after a quiet spell; unsaved work stays encrypted in this browser.
  $effect(() => {
    let timer = setTimeout(() => app.lock(), IDLE_LOCK_MS)
    const reset = () => {
      clearTimeout(timer)
      timer = setTimeout(() => app.lock(), IDLE_LOCK_MS)
    }
    const events = ['pointerdown', 'keydown', 'wheel'] as const
    events.forEach((e) => window.addEventListener(e, reset, { passive: true }))
    return () => {
      clearTimeout(timer)
      events.forEach((e) => window.removeEventListener(e, reset))
    }
  })

  $effect(() => {
    const warn = (e: BeforeUnloadEvent) => {
      if (app.changes > 0) e.preventDefault()
    }
    window.addEventListener('beforeunload', warn)
    return () => window.removeEventListener('beforeunload', warn)
  })
</script>

<div class="shell">
  <aside>
    <div class="brand"><Logo /> Investor</div>

    <div class="file">
      <span class="name num">{app.fileName}</span>
      {#if app.changes > 0}
        <span class="unsaved">{t('shell.unsaved', { count: app.changes })}</span>
        <span>{app.draftKept ? t('shell.keptInBrowser') : t('shell.notKept')}</span>
      {:else}
        <span>{t('shell.allSaved')}</span>
      {/if}
    </div>
    <button class="save" onclick={save} disabled={saving}>{saving ? t('shell.saving') : t('shell.save')}</button>
    {#if saveError}<p class="save-error" role="alert">{saveError}</p>{/if}

    <nav aria-label={t('nav.label')}>
      {#each nav as item (item.view)}
        <button class="nav" class:on={app.view === item.view} aria-current={app.view === item.view ? 'page' : undefined} onclick={() => (app.view = item.view)}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d={item.icon} /></svg>
          {item.label()}
        </button>
      {/each}
    </nav>

    <div class="bottom">
      <LanguagePicker />
      <p class="privacy">{t('shell.privacy')}</p>
      <button class="nav" onclick={lock}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><rect x="5" y="11" width="14" height="10" rx="2" /><path d="M8 11V7a4 4 0 0 1 8 0v4" /></svg>
        {t('shell.lock')}
      </button>
    </div>
  </aside>

  <main>
    {#if app.view === 'overview'}
      <Overview />
    {:else if app.view === 'ledger'}
      <Ledger />
    {:else if app.view === 'import'}
      <Import />
    {:else}
      <Tax />
    {/if}
  </main>
</div>

<style>
  .shell {
    display: flex;
    flex-wrap: wrap;
    min-height: 100vh;
  }
  aside {
    flex: 1 1 230px;
    max-width: 100%;
    background: var(--side);
    color: var(--side-text);
    padding: 24px 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  main {
    flex: 999 1 560px;
    min-width: 0;
    padding: 32px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    color: #fff;
    font-weight: 600;
    font-size: 18px;
    padding: 0 8px;
  }
  .file {
    background: var(--side-2);
    border-radius: 8px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
  }
  .name {
    color: #fff;
    overflow-wrap: anywhere;
  }
  .unsaved {
    color: #f0b98a;
  }
  .save {
    min-height: 44px;
    border: 0;
    border-radius: 8px;
    background: var(--side-accent);
    color: #111821;
    font: 600 14px var(--sans);
    cursor: pointer;
  }
  .save-error {
    margin: 0;
    color: #f0b98a;
    font-size: 12px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .nav {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    min-height: 44px;
    border: 0;
    border-radius: 8px;
    background: none;
    color: var(--side-text);
    font: 14px var(--sans);
    text-align: left;
    cursor: pointer;
  }
  .nav:hover {
    background: var(--side-2);
    color: #fff;
  }
  .nav.on {
    background: var(--side-active);
    color: #fff;
  }
  .bottom {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-top: 16px;
  }
  .privacy {
    margin: 0;
    padding: 0 4px;
    font-size: 12px;
    line-height: 1.45;
    color: #9aa5b1;
  }
  @media (max-width: 640px) {
    main {
      padding: 20px 16px;
    }
  }
</style>
