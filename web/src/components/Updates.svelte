<script lang="ts">
  // Settings → Updates, in the desktop app: this version, and a check for a
  // newer one on demand that says what happened, including errors.
  import { onMount } from 'svelte'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  type Invoke = (command: string, args?: unknown) => Promise<unknown>
  const invoke = (window as unknown as { __TAURI_INTERNALS__?: { invoke: Invoke } }).__TAURI_INTERNALS__!.invoke

  let current = $state('')
  let phase = $state<'idle' | 'checking' | 'latest' | 'available' | 'installing' | 'failed'>('idle')
  let found = $state('')
  let error = $state('')

  onMount(() => void invoke('app_version').then((v) => (current = String(v))))

  async function check(): Promise<void> {
    phase = 'checking'
    try {
      const update = (await invoke('update_check')) as { version: string } | null
      found = update?.version ?? ''
      phase = update ? 'available' : 'latest'
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
      phase = 'failed'
    }
  }

  async function install(): Promise<void> {
    if (app.changes > 0 && !app.example && !confirm(t('update.unsaved'))) return
    phase = 'installing'
    try {
      await invoke('update_install')
    } catch (e) {
      error = e instanceof Error ? e.message : String(e)
      phase = 'failed'
    }
  }
</script>

<section class="card" aria-labelledby="updates-title">
  <h2 id="updates-title">{t('settings.updatesTitle')}</h2>
  {#if current}<p class="muted">{t('settings.version', { version: current })}</p>{/if}
  <div class="row">
    {#if phase === 'available' || phase === 'installing'}
      <span role="status">{t('update.available', { version: found })}</span>
      <button class="btn" onclick={install} disabled={phase === 'installing'}>{phase === 'installing' ? t('update.installing') : t('update.install')}</button>
    {:else}
      <button class="btn ghost" onclick={check} disabled={phase === 'checking'}>{phase === 'checking' ? t('settings.checking') : t('settings.check')}</button>
      {#if phase === 'latest'}<span role="status">{t('settings.latest')}</span>{/if}
    {/if}
  </div>
  {#if phase === 'failed'}<p class="error" role="alert">{t('settings.checkFailed', { reason: error })}</p>{/if}
</section>

<style>
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin-top: 12px;
  }
  p {
    margin: 0;
  }
</style>
