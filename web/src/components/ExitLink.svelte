<script lang="ts">
  // Back to the marketing page one level up. Unsaved work is still caught by
  // the leave-page warning and kept in this browser as a draft.
  import { t } from '../lib/i18n.svelte'

  let { label = true, long = false }: { label?: boolean; long?: boolean } = $props()
</script>

<a class="exit" href="../" title={t('shell.exitHint')} aria-label={label ? undefined : t('shell.exitHint')}>
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M10 4H5v16h5M14 8l-4 4 4 4M10 12h10" /></svg>
  {#if label}<span>{long ? t('shell.exitLong') : t('shell.exit')}</span>{/if}
</a>

<style>
  .exit {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: 40px;
    min-width: 40px;
    padding: 0 12px;
    border: 1px solid rgb(255 255 255 / 0.2);
    border-radius: 10px;
    color: var(--side-text);
    font: 500 14px var(--sans);
    text-decoration: none;
    white-space: nowrap;
  }
  .exit:hover {
    color: #fff;
    border-color: rgb(255 255 255 / 0.4);
    background: var(--side-2);
  }
  .exit:not(:has(span)) {
    padding: 0;
    width: 44px;
    min-height: 44px;
  }
</style>
