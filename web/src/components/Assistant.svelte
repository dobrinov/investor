<script lang="ts">
  // The magic button in the bottom-right corner and the assistant it opens.
  // On the website it explains that the assistant needs the desktop app; in
  // the app it sets up a model, then chats with the person's books as
  // context, asking before anything is written. It also lets the person's
  // own AI tool use the open file, and shows that tool's changes to approve.
  import { tick } from 'svelte'
  import { bridge } from '../lib/bridge.svelte'
  import { available, Conversation, DEFAULTS, findLocalModels, type Found, loadSettings, saveSettings, skills, type Bubble, type ModelKind, type ModelSettings, type Skill } from '../lib/assistant.svelte'
  import { t, translates } from '../lib/i18n.svelte'

  let open = $state(false)
  const saved = available ? loadSettings() : null
  let settings = $state<ModelSettings | null>(saved)
  let setup = $state(false)
  let conversation = $state<Conversation | null>(null)
  let draft = $state('')
  let list = $state<HTMLElement>()
  let mode = $state<'chat' | 'connect'>('chat')
  let copied = $state('')

  async function copy(text: string): Promise<void> {
    await navigator.clipboard.writeText(text)
    copied = text
    setTimeout(() => (copied = ''), 1500)
  }

  // A change from the person's own AI tool opens the panel to approve it.
  $effect(() => {
    if (bridge.approvals.length) open = true
  })

  // Setup form
  let form = $state<ModelSettings>(saved ?? { kind: 'local', ...DEFAULTS.local })
  let found = $state<Found[] | null>(null)
  let looking = $state(false)

  function choose(kind: ModelKind): void {
    form = { kind, ...DEFAULTS[kind], apiKey: form.kind === kind ? form.apiKey : '' }
    found = null
  }

  async function look(): Promise<void> {
    looking = true
    found = await findLocalModels().catch(() => [])
    looking = false
    if (found.length && !form.model) form = { ...form, kind: found[0].kind, url: found[0].url, model: found[0].model }
  }

  function saveSetup(): void {
    settings = { ...form, url: form.url.trim(), model: form.model.trim(), apiKey: form.apiKey.trim() }
    saveSettings(settings)
    setup = false
    conversation = new Conversation(settings)
  }

  function forget(): void {
    saveSettings(null)
    settings = null
    conversation = null
    form = { kind: 'local', ...DEFAULTS.local }
    setup = true
  }

  function toggle(): void {
    open = !open
    if (open && available && settings && !conversation) conversation = new Conversation(settings)
    if (open && available && !settings) setup = true
  }

  async function send(text = draft, skill?: Skill): Promise<void> {
    if (!conversation || !text.trim()) return
    draft = ''
    const running = conversation.send(text, skill)
    await scrollDown()
    await running
    await scrollDown()
  }

  async function scrollDown(): Promise<void> {
    await tick()
    list?.scrollTo({ top: list.scrollHeight, behavior: 'smooth' })
  }

  $effect(() => {
    void conversation?.bubbles.length
    void scrollDown()
  })

  const toolName = (name: string) => (translates(`assistant.tool.${name}`) ? t(`assistant.tool.${name}` as 'assistant.tool.portfolio') : name.replaceAll('_', ' '))
  const skillLabel = (s: Skill) => (translates(`assistant.skill.${s.name}`) ? t(`assistant.skill.${s.name}` as 'assistant.skill.portfolio-checkup') : s.name)
  const skillPrompt = (s: Skill) => (translates(`assistant.prompt.${s.name}`) ? t(`assistant.prompt.${s.name}` as 'assistant.prompt.portfolio-checkup') : s.name)

  /** Plain text with **bold** and "- " bullets; everything is escaped first. */
  function render(text: string): string {
    const escaped = text.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!)
    return escaped.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>').replace(/^[-*] /gm, '• ')
  }

</script>

<button class="magic" class:on={open} aria-expanded={open} aria-controls="assistant" onclick={toggle} title={t('assistant.open')}>
  <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M12 3l1.6 4.4L18 9l-4.4 1.6L12 15l-1.6-4.4L6 9l4.4-1.6zM19 14l.8 2.2L22 17l-2.2.8L19 20l-.8-2.2L16 17l2.2-.8zM5 15l.6 1.4L7 17l-1.4.6L5 19l-.6-1.4L3 17l1.4-.6z" />
  </svg>
  <span class="sr-only">{t('assistant.open')}</span>
  {#if bridge.approvals.length}<span class="badge num" aria-label={t('bridge.waitingApproval', { count: bridge.approvals.length })}>{bridge.approvals.length}</span>{/if}
</button>

{#if open}
  <aside id="assistant" class="panel" aria-label={t('assistant.title')}>
    <header>
      <h2>{t('assistant.title')}</h2>
      {#if available && settings && !setup && mode === 'chat'}
        <span class="model muted" title={settings.url}>{settings.model}</span>
        <button class="icon" onclick={() => ((setup = true), (form = { ...settings! }))} title={t('assistant.settings')} aria-label={t('assistant.settings')}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM4 12h2M18 12h2M12 4v2M12 18v2M6.3 6.3l1.4 1.4M16.3 16.3l1.4 1.4M6.3 17.7l1.4-1.4M16.3 7.7l1.4-1.4" /></svg>
        </button>
        <button class="icon" onclick={() => (conversation = new Conversation(settings!))} title={t('assistant.newChat')} aria-label={t('assistant.newChat')}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
        </button>
      {/if}
      <button class="icon" onclick={() => (open = false)} aria-label={t('common.close')}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
      </button>
    </header>

    {#if available}
      <div class="modes" role="tablist" aria-label={t('assistant.title')}>
        <button role="tab" aria-selected={mode === 'chat'} onclick={() => (mode = 'chat')}>{t('assistant.tab.chat')}</button>
        <button role="tab" aria-selected={mode === 'connect'} onclick={() => (mode = 'connect')}>
          {t('assistant.tab.tool')}{#if bridge.on}<span class="live-dot" class:live={bridge.client}></span>{/if}
        </button>
      </div>
    {/if}
    {#if bridge.approvals.length}
      {@const a = bridge.approvals[0]}
      <div class="approve waiting outside">
        <span class="small">{t('bridge.wants', { name: a.client || t('bridge.aTool') })}</span>
        <strong>{toolName(a.name)}</strong>
        <dl>
          {#each Object.entries(a.args).filter(([, v]) => v !== null && v !== '' && v !== undefined) as [k, v] (k)}
            <div><dt>{k.replaceAll('_', ' ')}</dt><dd>{typeof v === 'object' ? JSON.stringify(v) : String(v)}</dd></div>
          {/each}
        </dl>
        <p class="small">{t('assistant.approveHint')}</p>
        <div class="actions">
          <button class="btn" onclick={() => bridge.decide(true)}>{t('assistant.approve')}</button>
          <button class="btn ghost" onclick={() => bridge.decide(false)}>{t('assistant.decline')}</button>
        </div>
      </div>
    {/if}
    {#if !available}
      <div class="body intro">
        <p class="lead">{t('assistant.webLead')}</p>
        <ul>
          <li>{t('assistant.can1')}</li>
          <li>{t('assistant.can2')}</li>
          <li>{t('assistant.can3')}</li>
        </ul>
        <p class="muted">{t('assistant.webWhy')}</p>
        <a class="btn" href="../#install" target="_top">{t('assistant.download')}</a>
        <p class="muted small">{t('assistant.webModels')}</p>
      </div>
    {:else if mode === 'connect'}
      <div class="body connect">
        <p>{t('bridge.lead')}</p>
        {#if bridge.on}
          <p class="status"><span class="dot" class:live={bridge.client}></span>{bridge.client ? t('bridge.connected', { name: bridge.client }) : t('bridge.waiting')}</p>
          <p class="small">{t('bridge.claudeCode')}</p>
          <div class="code"><code>{bridge.command}</code><button class="btn ghost" onclick={() => copy(bridge.command)}>{copied === bridge.command ? t('bridge.copied') : t('bridge.copy')}</button></div>
          <p class="small">{t('bridge.other')}</p>
          <div class="code"><code>{bridge.address}</code><button class="btn ghost" onclick={() => copy(bridge.address)}>{copied === bridge.address ? t('bridge.copied') : t('bridge.copy')}</button></div>
          <div class="actions">
            <button class="btn ghost" onclick={() => bridge.stop()}>{t('bridge.turnOff')}</button>
            <button class="link" onclick={() => bridge.renew()}>{t('bridge.newAddress')}</button>
          </div>
        {:else}
          <div class="actions"><button class="btn" onclick={() => bridge.start()}>{t('bridge.turnOn')}</button></div>
        {/if}
        {#if bridge.error}<p class="error">{t('bridge.failed', { reason: bridge.error })}</p>{/if}
        <p class="muted small">{t('bridge.privacy')}</p>
      </div>
    {:else if setup || !settings}
      <form class="body setup" onsubmit={(e) => (e.preventDefault(), saveSetup())}>
        <p class="muted">{t('assistant.setupLead')}</p>
        <div class="kinds" role="radiogroup" aria-label={t('assistant.kind')}>
          {#each ['local', 'claude_code', 'anthropic', 'openai'] as const as kind (kind)}
            <button type="button" role="radio" aria-checked={form.kind === kind} class:on={form.kind === kind} onclick={() => choose(kind)}>
              <strong>{t(`assistant.kind.${kind}`)}</strong>
              <span>{t(`assistant.kindHint.${kind}`)}</span>
            </button>
          {/each}
        </div>
        {#if form.kind === 'local'}
          <button type="button" class="btn ghost" onclick={look} disabled={looking}>{looking ? t('assistant.looking') : t('assistant.find')}</button>
          {#if found}
            {#if found.length}
              <div class="found">
                {#each found as m (m.server + m.model)}
                  <button type="button" class:on={form.model === m.model && form.url === m.url} onclick={() => (form = { ...form, kind: m.kind, url: m.url, model: m.model })}>{m.model} <span class="muted">· {m.server}</span></button>
                {/each}
              </div>
            {:else}
              <p class="notice small">{t('assistant.noneFound')}</p>
            {/if}
          {/if}
        {/if}
        {#if form.kind !== 'claude_code'}
          <label class="field">{t('assistant.address')}<input bind:value={form.url} required /></label>
          <label class="field">{t('assistant.modelName')}<input bind:value={form.model} required placeholder={form.kind === 'local' ? 'qwen3:8b' : ''} /></label>
        {/if}
        {#if form.kind === 'anthropic' || form.kind === 'openai'}
          <label class="field">{t('assistant.apiKey')}<input type="password" bind:value={form.apiKey} required autocomplete="off" /><span class="hint">{t('assistant.apiKeyHint')}</span></label>
        {/if}
        <p class="muted small">{t(`assistant.privacy.${form.kind}`)}</p>
        <div class="actions">
          <button class="btn">{t('assistant.start')}</button>
          {#if settings}<button type="button" class="btn ghost" onclick={() => (setup = false)}>{t('common.cancel')}</button>{/if}
          {#if settings}<button type="button" class="link" onclick={forget}>{t('assistant.forget')}</button>{/if}
        </div>
      </form>
    {:else if conversation}
      <div class="body chat" bind:this={list} aria-live="polite">
        {#if conversation.bubbles.length === 0}
          <p class="muted">{t('assistant.hello')}</p>
          <div class="starters">
            {#each skills as s (s.name)}
              <button onclick={() => send(skillPrompt(s), s)}>{skillLabel(s)}</button>
            {/each}
          </div>
        {/if}
        {#each conversation.bubbles as b, i (i)}
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
                <div class="actions">
                  <button class="btn" onclick={() => conversation!.decide(true)}>{t('assistant.approve')}</button>
                  <button class="btn ghost" onclick={() => conversation!.decide(false)}>{t('assistant.decline')}</button>
                </div>
              {:else}
                <p class="small state {bubble.state}">{t(`assistant.state.${bubble.state}`)}{bubble.note ? `: ${bubble.note}` : ''}</p>
              {/if}
            </div>
          {:else}
            <p class="error">{bubble.text}</p>
          {/if}
        {/each}
        {#if conversation.busy && !conversation.bubbles.some((x) => x.kind === 'write' && x.state === 'waiting')}<p class="read muted thinking">{t('assistant.thinking')}</p>{/if}
      </div>
      <form class="compose" onsubmit={(e) => (e.preventDefault(), send())}>
        <textarea
          bind:value={draft}
          rows="2"
          placeholder={t('assistant.placeholder')}
          aria-label={t('assistant.placeholder')}
          onkeydown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault()
              void send()
            }
          }}
        ></textarea>
        <button class="btn" disabled={conversation.busy || !draft.trim()}>{t('assistant.send')}</button>
      </form>
    {/if}
  </aside>
{/if}

<style>
  .magic {
    position: fixed;
    right: 24px;
    bottom: 24px;
    z-index: 40;
    width: 56px;
    height: 56px;
    border: 0;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--on-accent);
    background: var(--accent);
    box-shadow: 0 10px 28px rgb(17 24 33 / 0.28);
    cursor: pointer;
    transition: transform 0.15s;
  }
  .badge {
    position: absolute;
    top: -2px;
    right: -2px;
    min-width: 22px;
    height: 22px;
    padding: 0 6px;
    border-radius: 11px;
    border: 2px solid var(--surface);
    background: var(--warn);
    color: #fff;
    font-size: 12px;
    font-weight: 600;
    display: grid;
    place-items: center;
  }
  .magic:hover {
    transform: scale(1.06);
    background: var(--accent-strong);
  }
  .modes {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 4px;
    margin: 12px 16px 0;
    padding: 4px;
    background: var(--surface-2);
    border: 1px solid var(--line-soft);
    border-radius: 10px;
  }
  .modes button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 36px;
    border: 0;
    border-radius: 7px;
    background: none;
    color: var(--ink-2);
    font: 500 14px var(--sans);
    cursor: pointer;
  }
  .modes button[aria-selected='true'] {
    background: var(--surface);
    color: var(--ink);
    font-weight: 600;
    box-shadow: 0 1px 2px rgb(17 24 33 / 0.12);
  }
  .live-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--muted);
  }
  .live-dot.live {
    background: var(--gain);
  }
  .magic.on {
    transform: rotate(20deg);
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }
  .panel {
    position: fixed;
    right: 24px;
    bottom: 92px;
    z-index: 39;
    width: min(440px, calc(100vw - 48px));
    height: min(680px, calc(100dvh - 120px));
    display: flex;
    flex-direction: column;
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    border-radius: 16px;
    box-shadow: 0 24px 60px rgb(17 24 33 / 0.25);
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 12px 12px 12px 18px;
    border-bottom: 1px solid var(--line);
  }
  h2 {
    margin: 0;
    font-size: 16px;
    flex: 1;
  }
  .model {
    font-size: 12px;
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .icon {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 8px;
    background: none;
    color: var(--ink-2);
    cursor: pointer;
  }
  .icon:hover {
    background: var(--surface-2);
  }
  .outside {
    margin: 12px 12px 0;
  }
  .connect p {
    margin: 0;
  }
  .status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--muted);
  }
  .dot.live {
    background: var(--accent);
  }
  .code {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 8px 8px 8px 12px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--surface-2);
  }
  .code code {
    flex: 1;
    min-width: 0;
    font: 12px var(--mono, monospace);
    overflow-wrap: anywhere;
  }
  .link.left {
    align-self: flex-start;
    padding: 0;
  }
  .body {
    flex: 1;
    overflow-y: auto;
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .lead {
    margin: 0;
    font-size: 16px;
    font-weight: 500;
  }
  .intro ul {
    margin: 0;
    padding-left: 20px;
    display: grid;
    gap: 6px;
  }
  .intro p,
  .setup p {
    margin: 0;
  }
  .small {
    font-size: 13px;
  }
  .kinds {
    display: grid;
    gap: 8px;
  }
  .kinds button,
  .found button,
  .starters button {
    text-align: left;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--surface);
    color: var(--ink);
    padding: 10px 12px;
    font: 14px var(--sans);
    cursor: pointer;
  }
  .kinds button {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .kinds button span {
    font-size: 12px;
    color: var(--muted);
  }
  .kinds button.on,
  .found button.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .found {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .found button {
    padding: 6px 10px;
    font-size: 13px;
  }
  .hint {
    font-size: 12px;
    font-weight: 400;
    color: var(--muted);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }
  .link {
    border: 0;
    background: none;
    color: var(--accent);
    font: 14px var(--sans);
    cursor: pointer;
    text-decoration: underline;
  }
  .starters {
    display: grid;
    gap: 8px;
  }
  .starters button:hover,
  .kinds button:hover {
    border-color: var(--accent);
  }
  .bubble {
    margin: 0;
    padding: 10px 12px;
    border-radius: 12px;
    font-size: 14px;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .bubble.user {
    align-self: flex-end;
    max-width: 85%;
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
    font-size: 14px;
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
  .state.approved {
    color: var(--accent);
  }
  .state.declined,
  .state.failed {
    color: var(--warn);
  }
  .compose {
    display: flex;
    gap: 8px;
    padding: 12px;
    border-top: 1px solid var(--line);
  }
  .compose textarea {
    flex: 1;
    resize: none;
    min-height: 44px;
    padding: 10px 12px;
    border: 1px solid var(--field);
    border-radius: 10px;
    font: 14px var(--sans);
    color: var(--ink);
    background: var(--surface);
  }
  .compose .btn {
    align-self: flex-end;
  }
  @media (max-width: 720px) {
    .magic {
      right: 16px;
      bottom: calc(76px + env(safe-area-inset-bottom));
      width: 50px;
      height: 50px;
    }
    .panel {
      inset: 0;
      width: 100%;
      height: 100dvh;
      border-radius: 0;
      border: 0;
      z-index: 50;
    }
  }
</style>
