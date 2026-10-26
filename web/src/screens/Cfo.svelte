<script lang="ts">
  // Your CFO: a briefing on the whole financial picture from an AI model on
  // this computer, then a conversation about any decision. Changes it
  // suggests wait for approval, like the assistant's.
  import { tick } from 'svelte'
  import { available, findLocalModels, saveSettings, type Bubble } from '../lib/assistant.svelte'
  import { cfo, picture } from '../lib/cfo.svelte'
  import { t, translates } from '../lib/i18n.svelte'

  let settings = $state(cfo.settings)
  let found = $state<{ server: string; url: string; model: string }[] | null>(null)
  let looking = $state(false)
  let draft = $state('')
  let chat = $state<HTMLElement>()

  async function look(): Promise<void> {
    looking = true
    found = await findLocalModels().catch(() => [])
    looking = false
  }

  function use(m: { url: string; model: string }): void {
    saveSettings({ kind: 'local', url: m.url, model: m.model, apiKey: '' })
    settings = cfo.settings
  }

  async function ask(text = draft): Promise<void> {
    if (!text.trim()) return
    draft = ''
    const running = cfo.ask(text)
    await tick()
    chat?.scrollIntoView({ behavior: 'smooth', block: 'start' })
    await running
  }

  const provider = $derived(settings?.kind === 'anthropic' ? 'Anthropic' : settings?.kind === 'openai' ? new URL(settings.url).host : '')
  const toolName = (name: string) => (translates(`assistant.tool.${name}`) ? t(`assistant.tool.${name}` as 'assistant.tool.portfolio') : name.replaceAll('_', ' '))

  /** Plain text with **bold** and "- " bullets; everything is escaped first. */
  function render(text: string): string {
    const escaped = text.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!)
    return escaped.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>').replace(/^[-*] /gm, '• ')
  }

  let seen = $state('')
</script>

<header>
  <h1>{t('nav.cfo')}</h1>
  <p class="muted">{t('cfo.subtitle')}</p>
</header>

{#if !available}
  <section class="card intro">
    <p class="lead">{t('cfo.webLead')}</p>
    <p class="muted">{t('cfo.webWhy')}</p>
    <a class="btn" href="../#install" target="_top">{t('cfo.download')}</a>
  </section>
{:else if !settings}
  <section class="card intro">
    <p class="lead">{t('cfo.setupLead')}</p>
    <div><button class="btn" onclick={look} disabled={looking}>{looking ? t('cfo.looking') : t('cfo.find')}</button></div>
    {#if found}
      {#if found.length}
        <div class="found">
          {#each found as m (m.server + m.model)}
            <button class="btn ghost" onclick={() => use(m)}>{t('cfo.use', { model: m.model })} <span class="muted">· {m.server}</span></button>
          {/each}
        </div>
      {:else}
        <p class="notice">{t('cfo.noneFound')}</p>
      {/if}
    {/if}
    <p class="muted small">{t('cfo.orCloud')}</p>
  </section>
{:else}
  <section class="card bar">
    <div class="who">
      <strong>{t('cfo.model', { model: settings.model })}</strong>
      <span class="muted small">{settings.kind === 'local' ? t('cfo.local') : t('cfo.cloud', { provider })}</span>
    </div>
    <button class="btn" onclick={() => cfo.brief()} disabled={cfo.busy}>{cfo.busy ? t('cfo.reading') : cfo.briefing ? t('cfo.again') : t('cfo.brief')}</button>
  </section>

  {#if cfo.error}<p class="error" role="alert">{cfo.error}</p>{/if}

  {#if cfo.busy && !cfo.briefing}
    <section class="card"><p class="muted thinking">{t('cfo.reading')}</p></section>
  {/if}

  {#if cfo.briefing}
    {@const b = cfo.briefing}
    <section class="card briefing" aria-labelledby="brief-title">
      {#if b.text}
        <h2 id="brief-title" class="sr-only">{t('cfo.actions')}</h2>
        <div class="text">{@html render(b.text)}</div>
      {:else}
        <h2 id="brief-title">{b.headline}</h2>
        {#if b.summary}<p>{b.summary}</p>{/if}
      {/if}
      {#if cfo.at}<p class="muted small">{t('cfo.asOf', { time: cfo.at.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) })}</p>{/if}
    </section>

    {#if b.actions.length}
      <section aria-labelledby="actions-title">
        <h2 id="actions-title" class="section-title">{t('cfo.actions')}</h2>
        <ol class="actions">
          {#each b.actions as a, i (i)}
            <li class="card action">
              <div class="action-head">
                <span class="pill priority {a.priority}">{t(`cfo.priority.${a.priority}`)}</span>
                <h3>{a.title}</h3>
              </div>
              {#if a.why}<p>{a.why}</p>{/if}
              {#if a.impact}<p class="impact"><strong>{t('cfo.impact')}:</strong> {a.impact}</p>{/if}
              <div><button class="btn ghost" onclick={() => ask(t('cfo.askAbout', { title: a.title }))}>{t('cfo.ask')}</button></div>
            </li>
          {/each}
        </ol>
      </section>
    {/if}

    {#if b.watch.length || b.questions.length}
      <div class="two">
        {#if b.watch.length}
          <section class="card" aria-labelledby="watch-title">
            <h2 id="watch-title">{t('cfo.watch')}</h2>
            <ul>{#each b.watch as w, i (i)}<li>{w}</li>{/each}</ul>
          </section>
        {/if}
        {#if b.questions.length}
          <section class="card" aria-labelledby="questions-title">
            <h2 id="questions-title">{t('cfo.questions')}</h2>
            <ul class="questions">
              {#each b.questions as q, i (i)}<li><button class="link" onclick={() => ask(q)}>{q}</button></li>{/each}
            </ul>
          </section>
        {/if}
      </div>
    {/if}
  {/if}

  <section class="card chat" aria-labelledby="chat-title" bind:this={chat}>
    <h2 id="chat-title">{t('cfo.chat')}</h2>
    {#if cfo.conversation}
      <div class="bubbles" aria-live="polite">
        {#each cfo.conversation.bubbles as b, i (i)}
          {@const bubble = b as Bubble}
          {#if bubble.kind === 'user'}
            <p class="bubble user">{bubble.text}</p>
          {:else if bubble.kind === 'assistant'}
            <p class="bubble assistant">{@html render(bubble.text)}</p>
          {:else if bubble.kind === 'read'}
            <p class="read muted">{t('assistant.looked', { what: toolName(bubble.name) })}</p>
          {:else if bubble.kind === 'write'}
            <div class="approve" class:waiting={bubble.state === 'waiting'}>
              <strong>{toolName(bubble.name)}</strong>
              <dl>
                {#each Object.entries(bubble.args).filter(([, v]) => v !== null && v !== '' && v !== undefined) as [k, v] (k)}
                  <div><dt>{k.replaceAll('_', ' ')}</dt><dd>{typeof v === 'object' ? JSON.stringify(v) : String(v)}</dd></div>
                {/each}
              </dl>
              {#if bubble.state === 'waiting'}
                <p class="small">{t('assistant.approveHint')}</p>
                <div class="row">
                  <button class="btn" onclick={() => cfo.conversation!.decide(true)}>{t('assistant.approve')}</button>
                  <button class="btn ghost" onclick={() => cfo.conversation!.decide(false)}>{t('assistant.decline')}</button>
                </div>
              {:else}
                <p class="small">{t(`assistant.state.${bubble.state}`)}{bubble.note ? `: ${bubble.note}` : ''}</p>
              {/if}
            </div>
          {:else}
            <p class="error">{bubble.text}</p>
          {/if}
        {/each}
        {#if cfo.conversation.busy && !cfo.conversation.bubbles.some((x) => x.kind === 'write' && x.state === 'waiting')}<p class="read muted thinking">{t('assistant.thinking')}</p>{/if}
      </div>
    {/if}
    <form class="compose" onsubmit={(e) => (e.preventDefault(), ask())}>
      <label class="sr-only" for="cfo-ask">{t('cfo.placeholder')}</label>
      <textarea
        id="cfo-ask"
        rows="2"
        bind:value={draft}
        placeholder={t('cfo.placeholder')}
        onkeydown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault()
            void ask()
          }
        }}
      ></textarea>
      <button class="btn" disabled={cfo.conversation?.busy || !draft.trim()}>{t('cfo.send')}</button>
    </form>
  </section>

  <details class="card seen" ontoggle={(e) => (e.currentTarget.open ? (seen = JSON.stringify(JSON.parse(picture()), null, 2)) : null)}>
    <summary>{t('cfo.seen')}</summary>
    <p class="muted small">{t('cfo.seenHint')}</p>
    <pre>{seen}</pre>
  </details>
{/if}

<p class="muted small">{t('cfo.disclaimer')}</p>

<style>
  header p,
  .intro p,
  .briefing p,
  .action p,
  .chat h2 {
    margin: 0;
  }
  .intro,
  .briefing,
  .chat {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .lead {
    font-size: 17px;
    font-weight: 500;
  }
  .intro .btn {
    align-self: flex-start;
  }
  .found {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .small {
    font-size: 13px;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .who {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .briefing h2 {
    margin: 0;
    font-size: 22px;
    line-height: 1.3;
  }
  .briefing p {
    font-size: 16px;
    line-height: 1.55;
  }
  .text {
    white-space: pre-wrap;
    line-height: 1.55;
  }
  .section-title {
    font-size: 18px;
    margin: 4px 0 12px;
  }
  .actions {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
  }
  .action {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .action-head {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  .action h3 {
    margin: 0;
    font-size: 17px;
  }
  .action p {
    line-height: 1.5;
  }
  .impact {
    font-size: 14px;
    color: var(--ink-2);
  }
  .priority.now {
    background: var(--warn-tint);
    color: var(--warn-ink);
  }
  .priority.soon {
    background: var(--accent-tint);
    color: var(--accent-strong);
  }
  .priority.later {
    background: var(--line-soft);
    color: var(--ink-2);
  }
  .two {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
  }
  .two h2 {
    margin: 0 0 8px;
    font-size: 16px;
  }
  .two ul {
    margin: 0;
    padding-left: 20px;
    display: grid;
    gap: 6px;
    line-height: 1.5;
  }
  .link {
    border: 0;
    padding: 0;
    background: none;
    color: var(--accent);
    font: inherit;
    text-align: left;
    text-decoration: underline;
    cursor: pointer;
  }
  .bubbles {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .bubble {
    margin: 0;
    padding: 10px 12px;
    border-radius: 12px;
    font-size: 15px;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .bubble.user {
    align-self: flex-end;
    max-width: 80%;
    background: var(--accent);
    color: var(--on-accent);
  }
  .bubble.assistant {
    align-self: flex-start;
    max-width: 100%;
    background: var(--surface-2);
  }
  .read {
    margin: 0;
    font-size: 12px;
  }
  .thinking {
    animation: pulse 1.2s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.4;
    }
  }
  .approve {
    border: 1px solid var(--line);
    border-radius: 12px;
    padding: 12px;
    display: grid;
    gap: 8px;
  }
  .approve.waiting {
    border-color: var(--warn-line);
    background: var(--warn-soft);
  }
  .approve dl {
    margin: 0;
    display: grid;
    gap: 2px;
    font-size: 13px;
  }
  .approve dl div {
    display: flex;
    gap: 8px;
  }
  .approve dt {
    color: var(--muted);
    min-width: 96px;
  }
  .approve dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .approve p {
    margin: 0;
  }
  .row {
    display: flex;
    gap: 8px;
  }
  .compose {
    display: flex;
    gap: 8px;
  }
  .compose textarea {
    flex: 1;
    resize: vertical;
    min-height: 48px;
    padding: 10px 12px;
    border: 1px solid var(--field);
    border-radius: 10px;
    font: 15px var(--sans);
    color: var(--ink);
    background: var(--surface);
  }
  .compose .btn {
    align-self: flex-end;
  }
  .seen summary {
    cursor: pointer;
    font-weight: 600;
  }
  .seen pre {
    max-height: 360px;
    overflow: auto;
    font: 12px var(--mono);
    background: var(--surface-2);
    padding: 12px;
    border-radius: 8px;
  }
</style>
