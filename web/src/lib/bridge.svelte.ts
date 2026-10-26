// Lets AI tools on this computer use the file open in the desktop app. The
// app's native side listens on 127.0.0.1 and hands each MCP request here;
// reads are answered from the unlocked file, and changes wait for the person
// to approve them, then count as unsaved work like any other.
import { assistantGuidance, assistantTools, changesBook, loadCore } from './core'
import { desktop } from './files'
import { app } from './state.svelte'
import { recall, remember } from './storage'

type Invoke = (command: string, args?: unknown) => Promise<unknown>
const invoke = (window as unknown as { __TAURI_INTERNALS__?: { invoke: Invoke } }).__TAURI_INTERNALS__?.invoke

const STORE = 'assistant-bridge'
const PORT = 47821
const VERSIONS = ['2025-11-25', '2025-06-18', '2025-03-26']

interface Job {
  id: number
  message: { id: unknown; method: string; params?: Record<string, unknown> }
}

export interface Approval {
  client: string
  name: string
  args: Record<string, unknown>
  decide: (approved: boolean) => void
}

function newSecret(): string {
  return Array.from(crypto.getRandomValues(new Uint8Array(18)), (b) => b.toString(16).padStart(2, '0')).join('')
}

class Bridge {
  on = $state(false)
  secret = $state('')
  error = $state('')
  /** The AI tool that connected last, by the name it gave. */
  client = $state('')
  approvals = $state<Approval[]>([])
  private polling = false

  get address(): string {
    return `http://127.0.0.1:${PORT}/mcp/${this.secret}`
  }

  get command(): string {
    return `claude mcp add --transport http investor ${this.address}`
  }

  /** Starts again if the person left it on. */
  resume(): void {
    if (!desktop) return
    try {
      const saved = JSON.parse(recall(STORE) ?? 'null') as { on: boolean; secret: string } | null
      if (saved?.secret) this.secret = saved.secret
      if (saved?.on) void this.start()
    } catch {
      // Nothing saved.
    }
  }

  async start(): Promise<void> {
    if (!invoke) return
    if (!this.secret) this.secret = newSecret()
    this.error = ''
    try {
      await invoke('bridge_start', { port: PORT, secret: this.secret })
      this.on = true
      this.keep()
      void this.poll()
    } catch (e) {
      this.on = false
      this.error = e instanceof Error ? e.message : String(e)
    }
  }

  async stop(): Promise<void> {
    this.on = false
    this.client = ''
    this.keep()
    for (const a of this.approvals) a.decide(false)
    await invoke?.('bridge_stop')
  }

  /** A new secret: tools set up with the old address stop working. */
  async renew(): Promise<void> {
    this.secret = newSecret()
    this.client = ''
    if (this.on) await this.start()
    else this.keep()
  }

  private keep(): void {
    remember(STORE, JSON.stringify({ on: this.on, secret: this.secret }))
  }

  private async poll(): Promise<void> {
    if (this.polling) return
    this.polling = true
    await loadCore()
    while (this.on) {
      try {
        const job = (await invoke!('bridge_next')) as Job | null
        if (job) void this.handle(job)
      } catch {
        await new Promise((r) => setTimeout(r, 1000))
      }
    }
    this.polling = false
  }

  private async handle(job: Job): Promise<void> {
    const { id, method, params = {} } = job.message
    let reply: Record<string, unknown>
    try {
      reply = { result: await this.answer(method, params) }
    } catch (e) {
      reply = { error: { code: -32601, message: e instanceof Error ? e.message : String(e) } }
    }
    await invoke!('bridge_reply', { id: job.id, reply: { jsonrpc: '2.0', id, ...reply } })
  }

  private async answer(method: string, params: Record<string, unknown>): Promise<unknown> {
    switch (method) {
      case 'initialize': {
        const asked = String(params.protocolVersion ?? '')
        const info = params.clientInfo as { name?: string } | undefined
        this.client = info?.name ?? ''
        return {
          protocolVersion: VERSIONS.includes(asked) ? asked : VERSIONS[1],
          capabilities: { tools: {} },
          serverInfo: { name: 'investor', version: '1' },
          instructions: `${assistantGuidance()} You are connected to the Investor app, which has the person's file open. Changes wait for the person to approve them in the app, then count as unsaved changes until they save the file.`,
        }
      }
      case 'ping':
        return {}
      case 'tools/list':
        return { tools: assistantTools(true).map((t) => ({ name: t.name, description: t.description, inputSchema: t.input_schema })) }
      case 'tools/call':
        return this.call(String(params.name ?? ''), (params.arguments as Record<string, unknown>) ?? {})
      default:
        throw new Error(`unknown method ${method}`)
    }
  }

  private async call(name: string, args: Record<string, unknown>): Promise<unknown> {
    const text = (value: unknown, isError = false) => ({ content: [{ type: 'text', text: typeof value === 'string' ? value : JSON.stringify(value, null, 2) }], isError })
    if (!app.book) return text('No file is open in Investor. Ask the person to open and unlock their file in the app, then try again.', true)
    if (changesBook(name)) {
      const approved = await new Promise<boolean>((decide) => this.approvals.push({ client: this.client, name, args, decide }))
      if (!approved) return text('The person declined this change in the Investor app. Nothing was recorded.', true)
    }
    try {
      const result = await app.assistantCall(name, args)
      return changesBook(name) ? text({ recorded: true, note: 'The change is in the app as unsaved work; the person saves the file.', result }) : text(result)
    } catch (e) {
      return text(e instanceof Error ? e.message : String(e), true)
    }
  }

  /** Answers the oldest change waiting for approval. */
  decide(approved: boolean): void {
    const first = this.approvals.shift()
    first?.decide(approved)
  }
}

export const bridge = new Bridge()
