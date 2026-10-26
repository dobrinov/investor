<script lang="ts">
  // Advice on a new password: how strong it is, roughly how long it would
  // take to guess, and one tip. Any password is accepted.
  import { i18n, t } from '../lib/i18n.svelte'
  import { duration, strength } from '../lib/strength'

  let { password }: { password: string } = $props()
  const s = $derived(password ? strength(password) : null)
  const time = $derived(s ? duration(s.seconds, i18n.lang) : null)
</script>

{#if s}
  <div class="strength" role="status" aria-live="polite">
    <div class="meter" aria-hidden="true">
      {#each [0, 1, 2, 3, 4] as i (i)}<span class:on={i <= s.score} data-score={s.score}></span>{/each}
    </div>
    <p>
      <strong class="level" data-score={s.score}>{t(`strength.${s.level}`)}</strong>
      · {time ? t('strength.time', { time }) : t('strength.never')}
    </p>
    <p class="tip">{t(`strength.tip.${s.tip}`)}</p>
  </div>
{/if}

<style>
  .strength {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
  }
  .strength p {
    margin: 0;
  }
  .meter {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 4px;
  }
  .meter span {
    height: 6px;
    border-radius: 3px;
    background: var(--line-soft);
  }
  .meter span.on[data-score='0'],
  .meter span.on[data-score='1'] {
    background: var(--warn);
  }
  .meter span.on[data-score='2'] {
    background: #c08a1e;
  }
  .meter span.on[data-score='3'],
  .meter span.on[data-score='4'] {
    background: var(--gain);
  }
  .level[data-score='0'],
  .level[data-score='1'] {
    color: var(--warn);
  }
  .level[data-score='3'],
  .level[data-score='4'] {
    color: var(--gain);
  }
  .tip {
    color: var(--muted);
  }
</style>
