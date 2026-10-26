<script lang="ts">
  // In the desktop app: looks for a newer version now and then, and offers
  // to restart into it. Nothing is installed until the person asks.
  import { onMount } from 'svelte'
  import { desktop } from '../lib/files'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  type Invoke = (command: string, args?: unknown) => Promise<unknown>
  const invoke = (window as unknown as { __TAURI_INTERNALS__?: { invoke: Invoke } }).__TAURI_INTERNALS__?.invoke

  let version = $state('')
  let dismissed = $state(false)
  let installing = $state(false)
  let error = $state('')

  let last = 0

  async function check(): Promise<void> {
    last = Date.now()
    try {
      const found = (await invoke!('update_check')) as { version: string } | null
      if (found && found.version !== version) {
        version = found.version
        dismissed = false
      }
    } catch {
      // Offline, or no release to compare with: try again later.
    }
  }

  async function install(): Promise<void> {
    if (app.changes > 0 && !confirm(t('update.unsaved'))) return
    installing = true
    error = ''
    try {
      await invoke!('update_install')
    } catch (e) {
      error = t('update.failed', { reason: e instanceof Error ? e.message : String(e) })
      installing = false
    }
  }

  onMount(() => {
    if (!desktop || import.meta.env.DEV) return
    const first = setTimeout(check, 5000)
    const later = setInterval(check, 6 * 60 * 60 * 1000)
    // Coming back to an app left open for long checks again, at most hourly.
    const back = () => Date.now() - last > 60 * 60 * 1000 && void check()
    window.addEventListener('focus', back)
    return () => (clearTimeout(first), clearInterval(later), window.removeEventListener('focus', back))
  })
</script>

{#if version && !dismissed}
  <div class="update" role="status">
    <span>{error || t('update.available', { version })}</span>
    <button class="btn" onclick={install} disabled={installing}>{installing ? t('update.installing') : t('update.install')}</button>
    {#if !installing}<button class="btn ghost" onclick={() => (dismissed = true)}>{t('update.later')}</button>{/if}
  </div>
{/if}

<style>
  .update {
    position: fixed;
    top: 12px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 60;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: 8px 12px;
    width: max-content;
    max-width: calc(100vw - 32px);
    padding: 10px 12px 10px 16px;
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    border-radius: 12px;
    box-shadow: 0 12px 32px rgb(17 24 33 / 0.18);
    font-size: 14px;
  }
</style>
