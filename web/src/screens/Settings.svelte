<script lang="ts">
  import { keepSessionKey, setStaysUnlocked, staysUnlocked } from '../lib/storage'
  import { countries } from '../lib/configs'
  import { country } from '../lib/format'
  import LanguagePicker from '../components/LanguagePicker.svelte'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)

  let current = $state('')
  let next = $state('')
  let repeat = $state('')
  let busy = $state(false)
  let error = $state('')
  let done = $state(false)

  async function change(): Promise<void> {
    error = ''
    done = false
    if (next.length < 10) return void (error = t('create.tooShort'))
    if (next !== repeat) return void (error = t('create.mismatch'))
    if (next === current) return void (error = t('settings.samePassword'))
    busy = true
    // Let the busy state paint before the deliberately slow key derivation.
    await new Promise((resolve) => setTimeout(resolve, 30))
    try {
      await app.changePassword(current, next)
      current = next = repeat = ''
      done = true
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      error = message.includes('wrong password') ? t('settings.wrongCurrent') : message
    } finally {
      busy = false
    }
  }

  let stay = $state(staysUnlocked())
  function toggleStay(on: boolean): void {
    stay = on
    setStaysUnlocked(on)
    if (on && app.book) keepSessionKey(app.book.exportKey())
  }
</script>

<header>
  <h1>{t('nav.settings')}</h1>
  <p class="muted">{t('settings.subtitle')}</p>
</header>

<section class="card" aria-labelledby="lang-title">
  <h2 id="lang-title">{t('common.language')}</h2>
  <LanguagePicker light />
</section>

<section class="card" aria-labelledby="pw-title">
  <h2 id="pw-title">{t('settings.passwordTitle')}</h2>
  <p class="muted">{t('settings.passwordText')}</p>
  <form onsubmit={(e) => (e.preventDefault(), change())}>
    <label class="field">{t('settings.current')}<input type="password" bind:value={current} autocomplete="current-password" required /></label>
    <label class="field">{t('settings.new')}<input type="password" bind:value={next} autocomplete="new-password" required /></label>
    <label class="field">{t('create.repeat')}<input type="password" bind:value={repeat} autocomplete="new-password" required /></label>
    <div><button class="btn" disabled={busy}>{busy ? t('settings.changing') : t('settings.change')}</button></div>
  </form>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if done}
    <div class="notice" role="status">
      <strong>{t('settings.changed')}</strong>
      {t('settings.changedText')}
    </div>
    <button class="btn" onclick={() => app.save()}>{t('shell.save')}</button>
  {/if}
</section>

<section class="card" aria-labelledby="stay-title">
  <h2 id="stay-title">{t('settings.stayTitle')}</h2>
  <label class="choice">
    <input type="checkbox" checked={stay} onchange={(e) => toggleStay(e.currentTarget.checked)} />
    {t('settings.stay')}
  </label>
  <p class="muted small">{t('settings.stayText')}</p>
</section>

<section class="card" aria-labelledby="ai-title">
  <h2 id="ai-title">{t('settings.aiTitle')}</h2>
  <p class="muted">{t('settings.aiText')}</p>
  <div><button class="btn ghost" onclick={() => app.openGuide('claude')}>{t('settings.aiHow')}</button></div>
</section>

<section class="card" aria-labelledby="file-title">
  <h2 id="file-title">{t('settings.fileTitle')}</h2>
  <dl>
    <div><dt>{t('create.fileName')}</dt><dd class="num">{app.fileName}</dd></div>
    <div><dt>{t('create.baseCurrency')}</dt><dd>{s.settings.base_currency}</dd></div>
    <div>
      <dt><label for="residency">{t('create.residency')}</label></dt>
      <dd>
        <select id="residency" class="input" value={s.settings.tax_residency} onchange={(e) => app.setTaxResidency(e.currentTarget.value)}>
          {#each countries().includes(s.settings.tax_residency) ? countries() : [s.settings.tax_residency, ...countries()] as c (c)}<option value={c}>{country(c)}</option>{/each}
        </select>
        <span class="muted small">{t('settings.residencyHint')}</span>
      </dd>
    </div>
    <div><dt>{t('settings.entries')}</dt><dd class="num">{s.entries.length}</dd></div>
  </dl>
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
  .choice {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 15px;
    margin: 8px 0;
  }
  .choice input {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }
  .small {
    font-size: 13px;
    line-height: 1.5;
    margin: 0;
  }
  h2 {
    font-size: 18px;
  }
  p {
    margin: 0;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 14px;
    align-items: flex-start;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 14px;
    width: 100%;
    max-width: 420px;
  }
  dl {
    display: flex;
    flex-wrap: wrap;
    gap: 12px 32px;
    margin: 0;
  }
  dt {
    font-size: 12px;
    color: var(--muted);
  }
  dd {
    margin: 2px 0 0;
    font-weight: 500;
  }
</style>
