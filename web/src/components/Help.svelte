<script lang="ts">
  // A "?" button with a tooltip: why a value is asked for, and what is
  // missing without it. Opens on hover, keyboard focus or tap.
  import { t } from '../lib/i18n.svelte'

  let { about, why, without }: { about: string; why: string; without: string } = $props()
  const id = `help-${Math.random().toString(36).slice(2, 8)}`
  let open = $state(false)
</script>

<span class="help" role="presentation" onmouseenter={() => (open = true)} onmouseleave={() => (open = false)}>
  <button
    type="button"
    aria-label={t('help.about', { about })}
    aria-describedby={id}
    aria-expanded={open}
    onclick={() => (open = true)}
    onfocus={() => (open = true)}
    onblur={() => (open = false)}
    onkeydown={(e) => e.key === 'Escape' && (open = false)}
  >?</button>
  <span role="tooltip" {id} class:open>
    <strong>{t('help.why')}</strong>
    {why}
    <strong class="gap">{t('help.without')}</strong>
    {without}
  </span>
</span>

<style>
  .help {
    position: relative;
    display: inline-flex;
    vertical-align: middle;
  }
  button {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 1px solid var(--field);
    background: var(--surface);
    color: var(--muted);
    font: 600 12px var(--sans);
    line-height: 1;
    cursor: help;
    padding: 0;
  }
  button:hover,
  button:focus-visible,
  button[aria-expanded='true'] {
    border-color: var(--accent);
    color: var(--accent);
  }
  [role='tooltip'] {
    position: absolute;
    top: calc(100% + 8px);
    left: -12px;
    z-index: 10;
    width: max-content;
    max-width: min(300px, 80vw);
    padding: 12px 14px;
    border-radius: 10px;
    background: var(--side);
    color: #e7ecf2;
    font: 400 13px/1.5 var(--sans);
    text-transform: none;
    letter-spacing: normal;
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.18);
    display: none;
  }
  [role='tooltip'].open {
    display: block;
  }
  strong {
    display: block;
    color: #fff;
    font-weight: 600;
  }
  .gap {
    margin-top: 8px;
  }
</style>
