// The virtual CFO: a model reads the whole financial picture the core works
// out and says what matters most and what to do about it. The figures come
// from the app; the model explains them and puts them in order. Briefings
// stay in memory only, never in browser storage.
import { complete, Conversation, loadSettings, type ModelSettings } from './assistant.svelte'
import { i18n } from './i18n.svelte'
import { app } from './state.svelte'

export interface Action {
  title: string
  why: string
  impact: string
  priority: 'now' | 'soon' | 'later'
}

export interface Briefing {
  headline: string
  summary: string
  actions: Action[]
  watch: string[]
  questions: string[]
  /** The model's answer as written, when it was not the JSON asked for. */
  text?: string
}

function language(): string {
  return new Intl.DisplayNames([i18n.lang], { type: 'language' }).of(i18n.lang) ?? i18n.lang
}

/** The picture, as JSON text for the model. */
export function picture(): string {
  return JSON.stringify(app.book?.assistantCall('financial_picture', {}, new Date().toISOString().slice(0, 10)))
}

const ROLE = `You are the person's virtual CFO: a calm, practical finance chief for one household.
The Investor app gives you their whole financial picture. Every figure in it is computed by the app: trust it and never invent numbers.
Weigh things in this order: safety first (an emergency fund, expensive debt), then costs and taxes, then diversification and growth, then long-term goals.
\`signals\` are facts the app found worth acting on, most serious first; use them, but judge for yourself what matters most.
Be specific with their own figures, explain trade-offs, and say when something is a judgement call.
You are not a licensed adviser: do not name specific securities to buy; talk in principles (for example a low-cost, broadly diversified fund) and what to check.`

/** The CFO's role and the picture, for the system prompt of a conversation. */
export function persona(): string {
  return `${ROLE}\n\nTheir financial picture now (the financial_picture tool's result):\n${picture()}`
}

const FORMAT = `Write a short briefing. Reply with JSON only, no other text, in this shape:
{"headline": "one sentence on where they stand",
 "summary": "two or three sentences",
 "actions": [{"title": "what to do, as an instruction", "why": "the reason, with their figures", "impact": "what it is worth or changes", "priority": "now" | "soon" | "later"}],
 "watch": ["a risk to keep an eye on"],
 "questions": ["a question worth asking themselves"]}
Three to five actions, most important first; at most three items in watch and in questions.`

/** Reads the briefing out of the model's answer, which may be wrapped in a code fence or text. */
export function parse(text: string): Briefing {
  const start = text.indexOf('{')
  const end = text.lastIndexOf('}')
  try {
    const data = JSON.parse(text.slice(start, end + 1)) as Partial<Briefing>
    if (typeof data.headline !== 'string' || !Array.isArray(data.actions)) throw new Error('not a briefing')
    const priorities = ['now', 'soon', 'later']
    return {
      headline: data.headline,
      summary: String(data.summary ?? ''),
      actions: data.actions
        .filter((a) => a && typeof a.title === 'string')
        .map((a) => ({ title: a.title, why: String(a.why ?? ''), impact: String(a.impact ?? ''), priority: priorities.includes(a.priority) ? a.priority : 'soon' })),
      watch: (data.watch ?? []).map(String),
      questions: (data.questions ?? []).map(String),
    }
  } catch {
    return { headline: '', summary: '', actions: [], watch: [], questions: [], text: text.trim() }
  }
}

class Cfo {
  briefing = $state<Briefing | null>(null)
  at = $state<Date | null>(null)
  busy = $state(false)
  error = $state('')
  conversation = $state<Conversation | null>(null)

  get settings(): ModelSettings | null {
    return loadSettings()
  }

  async brief(): Promise<void> {
    const settings = this.settings
    if (!settings || this.busy) return
    this.busy = true
    this.error = ''
    try {
      const system = `${ROLE}\n\nAnswer in ${language()}.`
      const text = await complete(settings, system, `${FORMAT}\n\nTheir financial picture:\n${picture()}`)
      this.briefing = parse(text)
      this.at = new Date()
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e)
    } finally {
      this.busy = false
    }
  }

  /** Asks the CFO a question, starting a conversation if there is none. */
  async ask(text: string): Promise<void> {
    const settings = this.settings
    if (!settings) return
    this.conversation ??= new Conversation(settings, persona)
    await this.conversation.send(text)
  }

  /** Forgets the briefing and conversation, e.g. when the file is locked. */
  reset(): void {
    this.briefing = null
    this.at = null
    this.conversation = null
    this.error = ''
  }
}

export const cfo = new Cfo()
