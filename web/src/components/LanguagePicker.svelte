<script lang="ts">
  import { i18n, languages, t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  let { light = false }: { light?: boolean } = $props()
  const id = `lang-${Math.random().toString(36).slice(2, 8)}`
</script>

<div class="picker" class:light>
  <label for={id}>{t('common.language')}</label>
  <select {id} value={i18n.lang} onchange={(e) => app.setLanguage(e.currentTarget.value)}>
    {#each languages as language (language.code)}
      <option value={language.code}>{language.name}</option>
    {/each}
  </select>
</div>

<style>
  .picker {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--side-text);
  }
  select {
    min-height: 36px;
    padding: 0 8px;
    border-radius: 6px;
    border: 1px solid #3a4656;
    background: var(--side-2);
    color: #fff;
    font: 500 13px var(--sans);
  }
  .light {
    color: var(--ink-2);
  }
  .light select {
    border-color: var(--field);
    background: var(--surface);
    color: var(--ink);
  }
</style>
