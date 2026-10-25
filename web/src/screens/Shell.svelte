<script lang="ts">
  import { savesInPlace } from '../lib/files'
  import LanguagePicker from '../components/LanguagePicker.svelte'
  import Logo from '../components/Logo.svelte'
  import { t } from '../lib/i18n.svelte'
  import { app, type View } from '../lib/state.svelte'
  import Goals from './Goals.svelte'
  import Guide from './Guide.svelte'
  import Import from './Import.svelte'
  import Ledger from './Ledger.svelte'
  import Overview from './Overview.svelte'
  import Projections from './Projections.svelte'
  import Settings from './Settings.svelte'
  import Trade from './Trade.svelte'
  import Tax from './Tax.svelte'

  const IDLE_LOCK_MS = 15 * 60 * 1000

  const nav: { view: View; label: () => string; icon: string }[] = [
    { view: 'overview', label: () => t('nav.overview'), icon: 'M3 3h7v9H3zM14 3h7v5h-7zM14 12h7v9h-7zM3 16h7v5H3z' },
    { view: 'trade', label: () => t('nav.trade'), icon: 'M7 4v16M3 8l4-4 4 4M17 20V4M13 16l4 4 4-4' },
    { view: 'goals', label: () => t('nav.goals'), icon: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10zM12 11a1 1 0 1 0 0 2 1 1 0 0 0 0-2z' },
    { view: 'projections', label: () => t('nav.projections'), icon: 'M3 20h18M4 16l5-6 4 3 7-8' },
    { view: 'ledger', label: () => t('nav.ledger'), icon: 'M4 4h12a4 4 0 0 1 4 4v12H8a4 4 0 0 1-4-4zM8 9h8M8 13h8' },
    { view: 'import', label: () => t('nav.import'), icon: 'M12 3v12M7 10l5 5 5-5M4 19h16' },
    { view: 'tax', label: () => t('nav.tax'), icon: 'M6 3h9l4 4v14H6zM9 12h7M9 16h7M14 3v5h5' },
    { view: 'guide', label: () => t('nav.guide'), icon: 'M12 4c-2-1-5-1-8 0v15c3-1 6-1 8 0 2-1 5-1 8 0V4c-3-1-6-1-8 0zM12 4v15' },
    { view: 'settings', label: () => t('nav.settings'), icon: 'M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM4 12h2M18 12h2M12 4v2M12 18v2M6.3 6.3l1.4 1.4M16.3 16.3l1.4 1.4M6.3 17.7l1.4-1.4M16.3 7.7l1.4-1.4' },
  ]

  // On a phone the sidebar becomes a top bar, these tabs and a "More" sheet.
  const tabs: { view: View; label: () => string }[] = [
    { view: 'overview', label: () => t('tab.overview') },
    { view: 'trade', label: () => t('tab.trade') },
    { view: 'goals', label: () => t('tab.goals') },
    { view: 'projections', label: () => t('tab.projections') },
  ]
  const icon = (view: View) => nav.find((n) => n.view === view)!.icon
  const inMore = $derived(!tabs.some((tab) => tab.view === app.view))
  let more = $state<HTMLDialogElement>()

  function go(view: View): void {
    app.view = view
    more?.close()
    window.scrollTo(0, 0)
  }

  let saving = $state(false)
  let saveError = $state('')

  async function save(as = false): Promise<void> {
    saving = true
    saveError = ''
    try {
      await (as ? app.saveAs() : app.save())
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e)
    } finally {
      saving = false
    }
  }

  function lock(): void {
    if (app.changes > 0 && !app.example && !confirm(t('shell.lockUnsaved'))) return
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
      if (app.changes > 0 && !app.example) e.preventDefault()
    }
    window.addEventListener('beforeunload', warn)
    return () => window.removeEventListener('beforeunload', warn)
  })

  // The current screen goes in the address, so a refresh comes back to it.
  $effect(() => {
    history.replaceState(null, '', `#${app.view}`)
  })
</script>

<div class="shell">
  <header class="topbar">
    <span class="brand"><Logo size={26} /> Investor</span>
    {#if app.example}
      <button class="example-close" onclick={lock}>{t('example.close')}</button>
    {:else}
    {#if app.changes > 0}<span class="dirty num">{t('shell.unsavedShort', { count: app.changes })}</span>{/if}
    <button class="save top-save" onclick={() => save()} disabled={saving}>
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 3v12M7 10l5 5 5-5M5 21h14" /></svg>
      {saving ? t('shell.saving') : t('shell.saveShort')}
    </button>
    {/if}
  </header>

  <aside>
    <div class="brand"><Logo /> Investor</div>

    {#if app.example}
      <div class="file example">
        <span class="name">{t('example.title')}</span>
        <span>{t('example.text')}</span>
      </div>
      <button class="save" onclick={lock}>{t('example.close')}</button>
    {:else}
    <div class="file">
      <span class="name num">{app.fileName}</span>
      {#if app.changes > 0}
        <span class="unsaved">{t('shell.unsaved', { count: app.changes })}</span>
        <span>{app.draftKept ? t('shell.keptInBrowser') : t('shell.notKept')}</span>
      {:else}
        <span>{t('shell.allSaved')}</span>
      {/if}
    </div>
    <button class="save" onclick={() => save()} disabled={saving} title={app.hasFile ? t('shell.saveHint', { name: app.fileName }) : undefined}>
      {saving ? t('shell.saving') : t('shell.save')}
    </button>
    <button class="save-as" onclick={() => save(true)} disabled={saving}>{t('shell.saveAs')}</button>
    {#if !savesInPlace}<p class="save-note">{t('open.downloads')}</p>{/if}
    {#if saveError}<p class="save-error" role="alert">{saveError}</p>{/if}
    {/if}

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
    {#if app.example}<p class="example-banner phone-only" role="status"><strong>{t('example.title')}</strong> {t('example.text')}</p>{/if}
    {#if saveError}<p class="save-error phone-only" role="alert">{saveError}</p>{/if}
    {#if app.view === 'overview'}
      <Overview />
    {:else if app.view === 'trade'}
      <Trade />
    {:else if app.view === 'goals'}
      <Goals />
    {:else if app.view === 'projections'}
      <Projections />
    {:else if app.view === 'ledger'}
      <Ledger />
    {:else if app.view === 'import'}
      <Import />
    {:else if app.view === 'tax'}
      <Tax />
    {:else if app.view === 'guide'}
      <Guide />
    {:else}
      <Settings />
    {/if}
  </main>

  <nav class="tabbar" aria-label={t('nav.label')}>
    {#each tabs as tab (tab.view)}
      <button class:on={app.view === tab.view} aria-current={app.view === tab.view ? 'page' : undefined} onclick={() => go(tab.view)}>
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d={icon(tab.view)} /></svg>
        {tab.label()}
      </button>
    {/each}
    <button class:on={inMore} aria-haspopup="dialog" onclick={() => more?.showModal()}>
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><path d="M4 7h16M4 12h16M4 17h16" /></svg>
      {t('tab.more')}
    </button>
  </nav>

  <dialog class="sheet" bind:this={more} aria-label={t('tab.more')} onclick={(e) => e.target === more && more?.close()}>
    <div class="sheet-body">
      {#if app.example}
      <div class="file dark">
        <span class="name">{t('example.title')}</span>
        <span>{t('example.text')}</span>
        <button class="save" onclick={lock}>{t('example.close')}</button>
      </div>
      {:else}
      <div class="file dark">
        <span class="name num">{app.fileName}</span>
        {#if app.changes > 0}
          <span class="unsaved">{t('shell.unsaved', { count: app.changes })}</span>
          <span>{app.draftKept ? t('shell.keptInBrowser') : t('shell.notKept')}</span>
        {:else}
          <span>{t('shell.allSaved')}</span>
        {/if}
        <div class="file-actions">
          <button class="save" onclick={() => save()} disabled={saving}>{saving ? t('shell.saving') : t('shell.save')}</button>
          <button class="save-as" onclick={() => save(true)} disabled={saving}>{t('shell.saveAs')}</button>
        </div>
        {#if !savesInPlace}<p class="save-note">{t('open.downloads')}</p>{/if}
      </div>
      {/if}
      <div class="more-nav">
        {#each nav.filter((n) => !tabs.some((tab) => tab.view === n.view)) as item (item.view)}
          <button class:on={app.view === item.view} aria-current={app.view === item.view ? 'page' : undefined} onclick={() => go(item.view)}>
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d={item.icon} /></svg>
            {item.label()}
          </button>
        {/each}
      </div>
      <div class="more-bottom">
        <LanguagePicker light />
        <button class="lock" onclick={lock}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><rect x="5" y="11" width="14" height="10" rx="2" /><path d="M8 11V7a4 4 0 0 1 8 0v4" /></svg>
          {t('shell.lock')}
        </button>
      </div>
      <p class="sheet-privacy">{t('shell.privacy')}</p>
    </div>
  </dialog>
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
  .save-as {
    min-height: 36px;
    border: 1px solid rgb(255 255 255 / 0.18);
    border-radius: 8px;
    background: none;
    color: #e7ecf2;
    font: 500 13px var(--sans);
    cursor: pointer;
    margin-top: -4px;
  }
  .save-as:hover {
    background: rgb(255 255 255 / 0.06);
  }
  .save-note {
    margin: 0;
    color: #aab4c0;
    font-size: 12px;
    line-height: 1.4;
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
  .topbar,
  .tabbar,
  .phone-only {
    display: none;
  }
  .file.example {
    background: #1e3a2a;
    color: #cdeed6;
  }
  .example-close {
    margin-left: auto;
    min-height: 44px;
    padding: 0 14px;
    border: 1px solid rgb(255 255 255 / 0.25);
    border-radius: 10px;
    background: none;
    color: #fff;
    font: 600 15px var(--sans);
    cursor: pointer;
  }
  .example-banner {
    margin: 0;
    padding: 10px 12px;
    border-radius: 10px;
    background: #e6f4ea;
    color: #1b4d2b;
    font-size: 14px;
  }
  .sheet {
    margin: auto 0 0;
    width: 100%;
    max-width: 100%;
    max-height: 90vh;
    padding: 0;
    border: 0;
    border-radius: 20px 20px 0 0;
    background: var(--surface);
    color: var(--ink);
  }
  .sheet::backdrop {
    background: rgb(11 16 22 / 0.55);
  }
  .sheet-body {
    padding: 16px 16px calc(16px + env(safe-area-inset-bottom));
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .file.dark {
    background: var(--side);
    color: var(--side-text);
    border-radius: 14px;
    padding: 14px;
    font-size: 13px;
  }
  .file-actions {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    margin-top: 8px;
  }
  .file-actions .save-as {
    margin-top: 0;
    min-height: 44px;
  }
  .more-nav {
    display: flex;
    flex-direction: column;
  }
  .more-nav button {
    display: flex;
    align-items: center;
    gap: 14px;
    min-height: 52px;
    padding: 0 4px;
    border: 0;
    border-bottom: 1px solid var(--line-soft);
    background: none;
    color: var(--ink);
    font: 16px var(--sans);
    text-align: left;
    cursor: pointer;
  }
  .more-nav button.on {
    color: var(--accent);
    font-weight: 600;
  }
  .more-bottom {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 60px;
  }
  .lock {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 44px;
    padding: 0 14px;
    border: 1px solid var(--field);
    border-radius: 10px;
    background: none;
    color: var(--ink);
    font: 600 15px var(--sans);
    cursor: pointer;
  }
  .sheet-privacy {
    margin: 0;
    font-size: 13px;
    color: var(--muted);
  }

  @media (max-width: 720px) {
    aside {
      display: none;
    }
    .topbar {
      position: sticky;
      top: 0;
      z-index: 20;
      width: 100%;
      display: flex;
      align-items: center;
      gap: 10px;
      min-height: 56px;
      padding: 6px 8px 6px 16px;
      background: var(--side);
      color: #fff;
    }
    .topbar .brand {
      padding: 0;
      font-size: 17px;
    }
    .dirty {
      margin-left: auto;
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 12px;
      color: var(--side-text);
    }
    .dirty::before {
      content: '';
      width: 8px;
      height: 8px;
      border-radius: 4px;
      background: #f0b98a;
    }
    .top-save {
      margin-left: auto;
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 0 14px;
      font-size: 15px;
    }
    .dirty + .top-save {
      margin-left: 0;
    }
    main {
      padding: 20px 16px calc(96px + env(safe-area-inset-bottom));
      gap: 16px;
    }
    .phone-only {
      display: block;
    }
    .save-error.phone-only {
      color: var(--warn);
    }
    .tabbar {
      position: fixed;
      left: 0;
      right: 0;
      bottom: 0;
      z-index: 20;
      display: grid;
      grid-template-columns: repeat(5, minmax(0, 1fr));
      padding-bottom: env(safe-area-inset-bottom);
      background: var(--surface);
      border-top: 1px solid var(--line);
    }
    .tabbar button {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 3px;
      min-height: 64px;
      padding: 0 2px;
      border: 0;
      background: none;
      color: var(--muted);
      font: 11px var(--sans);
      cursor: pointer;
    }
    .tabbar button.on {
      color: var(--accent);
      font-weight: 600;
    }
  }
</style>
