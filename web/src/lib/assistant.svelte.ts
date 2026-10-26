// The assistant in the desktop app: a conversation with the AI model the
// person set up, using the same tools as the MCP server. Reads run at once;
// anything that would change the file waits for the person to approve it,
// and an approved change counts as unsaved work like any other.
//
// Requests to the model go through the desktop app's native side (the web
// view may only talk to the app itself), so the assistant needs the app.
import { assistantGuidance, changesBook, type AssistantTool } from './core'
import { desktop } from './files'
import { i18n } from './i18n.svelte'
import { app } from './state.svelte'
import { recall, remember } from './storage'

export type ModelKind = 'local' | 'claude_code' | 'anthropic' | 'openai'

export interface ModelSettings {
  kind: ModelKind
  /** Base address: the local server's /v1, or the provider's API. */
  url: string
  model: string
  /** Kept on this computer only; local models need none. */
  apiKey: string
}

export const DEFAULTS: Record<ModelKind, Omit<ModelSettings, 'kind'>> = {
  local: { url: 'http://localhost:11434/v1', model: '', apiKey: '' },
  claude_code: { url: '', model: 'Claude Code', apiKey: '' },
  anthropic: { url: 'https://api.anthropic.com', model: 'claude-opus-5-5', apiKey: '' },
  openai: { url: 'https://api.openai.com/v1', model: '', apiKey: '' },
}

const STORE = 'assistant-model'

export function loadSettings(): ModelSettings | null {
  try {
    const saved = JSON.parse(recall(STORE) ?? 'null') as ModelSettings | null
    return saved?.kind && saved.model ? saved : null
  } catch {
    return null
  }
}

export function saveSettings(settings: ModelSettings | null): void {
  remember(STORE, settings ? JSON.stringify(settings) : '')
}

// --- Talking to the model through the app ---------------------------------

interface Reply {
  status: number
  body: string
}

type Invoke = (command: string, args?: unknown) => Promise<unknown>
const invoke = (window as unknown as { __TAURI_INTERNALS__?: { invoke: Invoke } }).__TAURI_INTERNALS__?.invoke

async function request(method: 'GET' | 'POST', url: string, headers: Record<string, string>, body?: unknown): Promise<unknown> {
  if (!invoke) throw new Error('The assistant needs the desktop app.')
  const reply = (await invoke('model_request', { request: { method, url, headers, body } })) as Reply
  let parsed: unknown = reply.body
  try {
    parsed = JSON.parse(reply.body)
  } catch {
    // Not JSON; keep the text for the error message.
  }
  if (reply.status < 200 || reply.status >= 300) {
    const message = (parsed as { error?: { message?: string } | string })?.error
    throw new Error(`${reply.status}: ${typeof message === 'string' ? message : (message?.message ?? reply.body.slice(0, 300))}`)
  }
  return parsed
}

/** Models served on this computer by Ollama or LM Studio, when they run. */
export interface Found {
  kind: ModelKind
  server: string
  url: string
  model: string
}

export async function findLocalModels(): Promise<Found[]> {
  const found: Found[] = []
  if (invoke && (await invoke('claude_code_path').catch(() => null))) found.push({ kind: 'claude_code', ...DEFAULTS.claude_code, server: 'Claude Code' })
  const ollama = (await request('GET', 'http://localhost:11434/api/tags', {}).catch(() => null)) as { models?: { name: string }[] } | null
  for (const m of ollama?.models ?? []) found.push({ kind: 'local', server: 'Ollama', url: 'http://localhost:11434/v1', model: m.name })
  const studio = (await request('GET', 'http://localhost:1234/v1/models', {}).catch(() => null)) as { data?: { id: string }[] } | null
  for (const m of studio?.data ?? []) found.push({ kind: 'local', server: 'LM Studio', url: 'http://localhost:1234/v1', model: m.id })
  return found
}

// --- One conversation, provider by provider --------------------------------

interface Call {
  id: string
  name: string
  args: Record<string, unknown>
}

type Item =
  | { role: 'user'; text: string }
  /** `raw` is the provider's own message, sent back unchanged. */
  | { role: 'assistant'; text: string; calls: Call[]; raw: unknown }
  | { role: 'tool'; id: string; name: string; content: string; error: boolean }

interface Turn {
  text: string
  calls: Call[]
  raw: unknown
}

// Server-side fallback on a declined request, for the models that take it.
const FALLBACK_MODELS = ['claude-fable-5-1', 'claude-opus-5-5', 'claude-opus-5', 'claude-sonnet-5-5']

async function anthropic(s: ModelSettings, system: string, items: Item[], tools: AssistantTool[]): Promise<Turn> {
  const messages: unknown[] = []
  for (const item of items) {
    if (item.role === 'user') messages.push({ role: 'user', content: [{ type: 'text', text: item.text }] })
    else if (item.role === 'assistant') messages.push({ role: 'assistant', content: item.raw })
    else {
      const result = { type: 'tool_result', tool_use_id: item.id, content: item.content, is_error: item.error }
      // All results of one turn go back in a single user message.
      const last = messages[messages.length - 1] as { role: string; content: unknown[] } | undefined
      if (last?.role === 'user' && (last.content[0] as { type: string }).type === 'tool_result') last.content.push(result)
      else messages.push({ role: 'user', content: [result] })
    }
  }
  const fallback = FALLBACK_MODELS.includes(s.model)
  const headers: Record<string, string> = { 'x-api-key': s.apiKey, 'anthropic-version': '2026-10-21', 'content-type': 'application/json' }
  if (fallback) headers['anthropic-beta'] = 'server-side-fallback-2026-07-01'
  const body: Record<string, unknown> = {
    model: s.model,
    max_tokens: 16000,
    system,
    messages,
    output_config: { effort: 'medium' },
  }
  if (tools.length) body.tools = tools.map((t) => ({ name: t.name, description: t.description, input_schema: t.input_schema }))
  if (fallback) body.fallbacks = 'default'
  const reply = (await request('POST', `${s.url.replace(/\/$/, '')}/v1/messages`, headers, body)) as {
    content: { type: string; text?: string; id?: string; name?: string; input?: Record<string, unknown> }[]
    stop_reason: string
  }
  if (reply.stop_reason === 'refusal') throw new Error('The model declined to answer this.')
  return {
    text: reply.content.filter((b) => b.type === 'text').map((b) => b.text ?? '').join('\n\n'),
    calls: reply.content.filter((b) => b.type === 'tool_use').map((b) => ({ id: b.id!, name: b.name!, args: b.input ?? {} })),
    raw: reply.content,
  }
}

async function openaiCompatible(s: ModelSettings, system: string, items: Item[], tools: AssistantTool[]): Promise<Turn> {
  const messages: unknown[] = [{ role: 'system', content: system }]
  for (const item of items) {
    if (item.role === 'user') messages.push({ role: 'user', content: item.text })
    else if (item.role === 'assistant') messages.push(item.raw)
    else messages.push({ role: 'tool', tool_call_id: item.id, content: item.content })
  }
  const headers: Record<string, string> = { 'content-type': 'application/json' }
  if (s.apiKey) headers.authorization = `Bearer ${s.apiKey}`
  const body: Record<string, unknown> = { model: s.model, messages }
  // Some local servers reject an empty tool list.
  if (tools.length) body.tools = tools.map((t) => ({ type: 'function', function: { name: t.name, description: t.description, parameters: t.input_schema } }))
  const reply = (await request('POST', `${s.url.replace(/\/$/, '')}/chat/completions`, headers, body)) as { choices: { message: { content?: string | null; tool_calls?: { id?: string; function: { name: string; arguments: string | object } }[] } }[] }
  const message = reply.choices?.[0]?.message ?? {}
  const calls = (message.tool_calls ?? []).map((c, i) => {
    let args: Record<string, unknown> = {}
    try {
      args = typeof c.function.arguments === 'string' ? JSON.parse(c.function.arguments || '{}') : (c.function.arguments as Record<string, unknown>)
    } catch {
      args = {}
    }
    return { id: c.id || `call-${Date.now()}-${i}`, name: c.function.name, args }
  })
  const raw = { role: 'assistant', content: message.content ?? '', ...(calls.length ? { tool_calls: calls.map((c) => ({ id: c.id, type: 'function', function: { name: c.name, arguments: JSON.stringify(c.args) } })) } : {}) }
  return { text: message.content ?? '', calls, raw }
}

// Claude Code with the person's own sign-in, run by the desktop app with no
// tools: it answers from what it is sent, so it can advise but not record.
async function claudeCode(_s: ModelSettings, system: string, items: Item[]): Promise<Turn> {
  const transcript = items
    .map((i) => (i.role === 'user' ? `Person: ${i.text}` : i.role === 'assistant' ? `You: ${i.text}` : ''))
    .filter(Boolean)
    .join('\n\n')
  const prompt = items.length === 1 && items[0].role === 'user' ? items[0].text : `The conversation so far:\n\n${transcript}\n\nAnswer the person's last message.`
  const text = (await invoke!('claude_code_ask', { system, prompt })) as string
  return { text, calls: [], raw: { role: 'assistant', content: text } }
}

function provider(kind: ModelKind) {
  return kind === 'anthropic' ? anthropic : kind === 'claude_code' ? claudeCode : openaiCompatible
}

/** One answer to one message, without tools, e.g. for a briefing. */
export async function complete(settings: ModelSettings, system: string, text: string): Promise<string> {
  const turn = await provider(settings.kind)(settings, system, [{ role: 'user', text }], [])
  return turn.text
}

// --- Skills -----------------------------------------------------------------

// The same SKILL.md files the Claude Code plugin ships, so the app and the
// plugin follow the same routines. Adding a broker needs the source code, so
// it stays out of the app.
const skillFiles = import.meta.glob('../../../plugin/skills/*/SKILL.md', { query: '?raw', import: 'default', eager: true }) as Record<string, string>

export interface Skill {
  name: string
  body: string
}

export const skills: Skill[] = Object.entries(skillFiles)
  .map(([path, text]) => ({ name: path.split('/').slice(-2)[0], body: text.replace(/^---[\s\S]*?---\s*/, '') }))
  .filter((s) => s.name !== 'add-broker')

// --- The conversation -------------------------------------------------------

export type Bubble =
  | { kind: 'user'; text: string }
  | { kind: 'assistant'; text: string }
  | { kind: 'read'; name: string }
  | { kind: 'write'; name: string; args: Record<string, unknown>; state: 'waiting' | 'approved' | 'declined' | 'failed'; note?: string }
  | { kind: 'error'; text: string }

const MAX_STEPS = 16

export class Conversation {
  bubbles = $state<Bubble[]>([])
  busy = $state(false)
  /** The change waiting for the person's answer. */
  private answer: ((approved: boolean) => void) | null = null
  private items: Item[] = []
  private skill: Skill | null = null

  /** `persona` adds a role and context to the system prompt, e.g. the CFO's. */
  constructor(
    private readonly settings: ModelSettings,
    private readonly persona?: () => string,
  ) {}

  /** Approves or declines the change that is waiting. */
  decide(approved: boolean): void {
    this.answer?.(approved)
    this.answer = null
  }

  async send(text: string, skill?: Skill): Promise<void> {
    if (this.busy || !text.trim()) return
    if (skill) this.skill = skill
    this.items.push({ role: 'user', text })
    this.bubbles.push({ kind: 'user', text })
    this.busy = true
    try {
      await this.run()
    } catch (e) {
      this.bubbles.push({ kind: 'error', text: e instanceof Error ? e.message : String(e) })
    } finally {
      this.busy = false
    }
  }

  private system(): string {
    const language = new Intl.DisplayNames([i18n.lang], { type: 'language' }).of(i18n.lang) ?? i18n.lang
    const parts = [
      assistantGuidance(),
      `Today is ${new Date().toISOString().slice(0, 10)}. Answer in ${language}. Keep answers short and use the person's own figures.`,
      'Tools that change the file ask the person to approve first; if they decline, accept it and do not try again unless they ask.',
    ]
    if (this.persona) parts.push(this.persona())
    // Local models often have a small context window: let them call portfolio themselves.
    else if (this.settings.kind !== 'local') {
      try {
        parts.push(`The person's portfolio now (the portfolio tool's result):\n${JSON.stringify(app.book?.assistantCall('portfolio', {}, new Date().toISOString().slice(0, 10)))}`)
      } catch {
        // The model can call the tool itself.
      }
    }
    if (this.skill) parts.push(`Follow this routine for the person's request:\n\n${this.skill.body}`)
    return parts.join('\n\n')
  }

  private async run(): Promise<void> {
    const tools = app.book!.assistantTools(true)
    for (let step = 0; step < MAX_STEPS; step++) {
      const turn = await provider(this.settings.kind)(this.settings, this.system(), this.items, tools)
      this.items.push({ role: 'assistant', text: turn.text, calls: turn.calls, raw: turn.raw })
      if (turn.text.trim()) this.bubbles.push({ kind: 'assistant', text: turn.text.trim() })
      if (!turn.calls.length) return
      for (const call of turn.calls) this.items.push(await this.use(call))
    }
    this.bubbles.push({ kind: 'error', text: 'Stopped after many steps; ask again to continue.' })
  }

  private async use(call: Call): Promise<Item> {
    const result = (content: unknown, error = false): Item => ({ role: 'tool', id: call.id, name: call.name, content: typeof content === 'string' ? content : JSON.stringify(content), error })
    if (!changesBook(call.name)) {
      this.bubbles.push({ kind: 'read', name: call.name })
      try {
        return result(await app.assistantCall(call.name, call.args))
      } catch (e) {
        return result(e instanceof Error ? e.message : String(e), true)
      }
    }
    const bubble: Bubble = { kind: 'write', name: call.name, args: call.args, state: 'waiting' }
    this.bubbles.push(bubble)
    const index = this.bubbles.length - 1
    const approved = await new Promise<boolean>((resolve) => (this.answer = resolve))
    if (!approved) {
      this.bubbles[index] = { ...bubble, state: 'declined' }
      return result('The person declined this change. Nothing was written.', true)
    }
    try {
      const out = await app.assistantCall(call.name, call.args)
      this.bubbles[index] = { ...bubble, state: 'approved' }
      return result(out)
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      this.bubbles[index] = { ...bubble, state: 'failed', note: message }
      return result(message, true)
    }
  }
}

/** Why the assistant cannot run here, if it cannot. */
export const available = desktop
