<script lang="ts">
  import { tick } from 'svelte'
  import { bg } from '../lib/guide/bg'
  import { en } from '../lib/guide/en'
  import type { Section } from '../lib/guide/types'
  import { i18n, t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const guides: Record<string, Section[]> = { en, bg }
  const sections = $derived(guides[i18n.lang] ?? en)

  // Jump to the section another screen asked for.
  $effect(() => {
    const target = app.guideSection
    if (!target) return
    void tick().then(() => {
      document.getElementById(`guide-${target}`)?.scrollIntoView({ block: 'start' })
      app.guideSection = null
    })
  })
</script>

<header>
  <h1>{t('nav.guide')}</h1>
  <p class="muted">{t('guide.subtitle')}</p>
</header>

<nav class="card toc" aria-label={t('guide.contents')}>
  <h2>{t('guide.contents')}</h2>
  <ol>
    {#each sections as s (s.id)}
      <li><a href="#guide-{s.id}">{s.title}</a></li>
    {/each}
  </ol>
</nav>

{#each sections as s (s.id)}
  <section class="card" id="guide-{s.id}" aria-labelledby="guide-{s.id}-title">
    <h2 id="guide-{s.id}-title">{s.title}</h2>
    {#each s.blocks as b, i (i)}
      {#if 'p' in b}
        <p>{b.p}</p>
      {:else if 'steps' in b}
        <ol class="steps">{#each b.steps as step, j (j)}<li>{step}</li>{/each}</ol>
      {:else if 'list' in b}
        <ul>{#each b.list as item, j (j)}<li>{item}</li>{/each}</ul>
      {:else if 'code' in b}
        <pre class="num"><code>{b.code}</code></pre>
      {:else if 'terms' in b}
        <dl>
          {#each b.terms as [term, meaning] (term)}
            <div><dt>{term}</dt><dd>{meaning}</dd></div>
          {/each}
        </dl>
      {:else if 'link' in b}
        <p><a href={b.href} target="_blank" rel="noreferrer">{b.link} ↗</a></p>
      {/if}
    {/each}
  </section>
{/each}

<style>
  header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  h1 {
    font-size: 28px;
  }
  h2 {
    font-size: 18px;
    margin-bottom: 4px;
  }
  header p {
    margin: 0;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    scroll-margin-top: 16px;
    max-width: 860px;
  }
  .toc {
    max-width: 860px;
  }
  .toc ol {
    margin: 8px 0 0;
    padding-left: 20px;
    display: grid;
    gap: 6px;
  }
  p,
  li,
  dd {
    line-height: 1.6;
  }
  p {
    margin: 0;
  }
  ol,
  ul {
    margin: 0;
    padding-left: 22px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  pre {
    margin: 0;
    padding: 14px 16px;
    border-radius: 8px;
    background: var(--surface-2);
    border: 1px solid var(--line-soft);
    font-size: 13px;
    line-height: 1.55;
    overflow-x: auto;
  }
  dl {
    margin: 0;
    display: grid;
    gap: 12px;
  }
  dt {
    font-weight: 600;
  }
  dd {
    margin: 2px 0 0;
    color: var(--ink-2);
  }
</style>
