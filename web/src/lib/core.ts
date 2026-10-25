// Typed wrapper around the Rust core compiled to WebAssembly.
// Amounts arrive as exact decimal strings; convert only for display.
import init, { Session, assistantInstructions, assistantWrites, brokerFit as fit, project as runProjection } from './core-wasm/core.js'
import { i18n, t, translates } from './i18n.svelte'

let ready: Promise<unknown> | undefined

export function loadCore(): Promise<unknown> {
  ready ??= init()
  return ready
}

export type Decimal = string

export interface Settings {
  base_currency: string
  language: string
  tax_residency: string
}

export interface Money {
  amount: Decimal
  currency: string
}

export interface Posting {
  account: string
  amount: Money
  base: Decimal
  units?: Decimal
}

export interface Venue {
  code: string
  assumed: boolean
}

export interface Entry {
  id: string
  date: string
  description: string
  postings: Posting[]
  venue?: Venue
  source?: { broker: string; file: string; line: number; fingerprint: string }
  notes?: string[]
}

export interface Holding {
  broker: string
  symbol: string
  units: Decimal
  cost: Decimal
}

export interface CashBalance {
  broker: string
  currency: string
  amount: Decimal
}

export interface TrialBalanceRow {
  account: string
  debit: Decimal
  credit: Decimal
}

export interface Instrument {
  symbol: string
  name?: string
  isin?: string
  currency?: string
}

export type Risk = 'cautious' | 'balanced' | 'growth'
export type Market = 'bear' | 'base' | 'bull'

/** Every value is optional; a missing one only switches off what needs it. */
export interface Profile {
  birth_year: number | null
  monthly_spending: Decimal | null
  monthly_investing: Decimal | null
  risk: Risk
  retirement_age: number | null
  pension_age: number | null
  monthly_pension: Decimal | null
  retirement_spending: Decimal | null
}

export interface Goal {
  id: string
  name: string
  kind: 'emergency_fund' | 'savings'
  /** Amount, or months of spending for an emergency fund. */
  target: Decimal
  year?: number | null
  monthly: Decimal
}

export type MoveKind = 'lump' | 'spread' | 'monthly' | 'withdraw'

/** A move tried out on the Projections screen. */
export interface Move {
  id: string
  name: string
  enabled: boolean
  kind: MoveKind
  amount: Decimal
  /** YYYY-MM; null means now. */
  start: string | null
  /** Months to spread over (spread only). */
  months: number
  /** Use cash already held instead of new money (lump and spread). */
  from_cash: boolean
  /** Something other than the portfolio, in percent a year (lump and spread). */
  asset: { real_return: Decimal; volatility: Decimal } | null
}

export interface Plan {
  profile?: Profile | null
  goals: Goal[]
  what_if?: Move[]
}

export interface Band {
  year: number
  steady: number
  p10: number
  p50: number
  p90: number
  /** New money added since today. */
  put_in: number
}

export interface Outcome {
  bands: Band[]
  /** Share of futures where every withdrawal is paid; null without withdrawals. */
  chance_covered: number | null
}

export interface WhatIf {
  monthly: number
  spending: { goal: string; name: string; year: number; amount: number }[]
  plan: Outcome
  /** Null when no move is switched on. */
  with_moves: Outcome | null
  cash_short: { id: string; wanted: number; available: number }[]
  retirement: { plan: Retirement; with_moves: Retirement | null } | null
}

export interface RetireInput {
  age_now: number
  retire_at: number
  plan_until: number
  monthly_spending: number
  pension_from: number
  monthly_pension: number
}

export interface GoalStatus {
  id: string
  have: number
  target: number
  chance: number | null
  needed_monthly: number | null
  on_track: boolean
  /** Profile value needed to measure the goal. */
  missing?: 'monthly_spending'
}

export interface MonthFlow {
  /** YYYY-MM */
  month: string
  deposited: Decimal
  withdrawn: Decimal
  bought: Decimal
  sold: Decimal
}

export interface Projection {
  bands: { year: number; steady: number; p10: number; p50: number; p90: number }[]
  chance_of_target: number | null
}

export interface RetirementYear {
  age: number
  start: number
  growth: number
  withdrawn: number
  end: number
  from_portfolio_monthly: number
  pension_monthly: number
  covered: boolean
}

export interface Retirement {
  at_retirement: number
  runs_out_at: number | null
  chance_lasts: number
  safe_monthly_spending: number
  path: [number, number][]
  years: RetirementYear[]
}

export type QuoteSource = 'trade' | 'statement' | 'manual'

export interface Quote {
  date: string
  price: Money
  source: QuoteSource
}

export interface FxQuote {
  date: string
  /** Base currency units for one unit of the currency. */
  rate: Decimal
  source: QuoteSource
}

export interface Valuation {
  broker: string
  symbol: string
  units: Decimal
  cost: Decimal
  quote?: Quote
  value?: Decimal
  gain?: Decimal
  missing_fx: boolean
}

export type Payout = 'employment_income' | 'capital_gains' | 'unknown'

/** Money in something without a market price, e.g. an employer co-investment. */
export interface PrivateHolding {
  id: string
  name: string
  currency: string
  payout: Payout
  condition: string
  paid_from: string
  /** A value set by hand, in `currency`. */
  estimate?: { date: string; amount: Decimal }
  /** What was paid, in `currency`. */
  paid: Decimal
  /** The same in the base currency. */
  cost: Decimal
  /** The estimate in the base currency, when there is one. */
  value?: Decimal
  since: string | null
  /** Still waiting for the money; false once paid out. */
  open: boolean
  paid_out?: {
    date: string
    entry_id: string
    taxed_as: Payout
    /** Before tax, in `currency`. */
    amount: Decimal
    currency: string
    withheld: Decimal
    /** Gross payout less what was paid in, in the base currency. */
    gain: Decimal
  }
}

/** The money coming back from a private holding, all at once. */
export interface PayoutInput {
  entry_id: string
  date: string
  /** Before any tax. */
  amount: Decimal
  currency: string
  fx_rate?: Decimal | null
  /** Tax kept back before payment, in `currency`. */
  withheld: Decimal
  taxed_as: Payout
  paid_into: string
}

export interface PrivateInput {
  /** Chosen when the form opens, so adding twice adds once. */
  entry_id: string
  name: string
  date: string
  amount: Decimal
  currency: string
  fx_rate?: Decimal | null
  paid_from: string
  payout: Payout
  condition: string
}

/** A bank account whose balance is typed in by hand. */
export interface BankAccount {
  id: string
  name: string
  currency: string
  /** In `currency`. */
  balance: Decimal
  /** In the base currency at the latest rate, when there is one. */
  value?: Decimal
  updated: string | null
}

export interface BalanceInput {
  /** Chosen when the form opens, so saving twice books once. */
  entry_id: string
  date: string
  balance: Decimal
  fx_rate?: Decimal | null
}

export type Metal = 'gold' | 'silver' | 'platinum' | 'palladium'

export interface MetalHolding {
  metal: Metal
  /** Fine weight. */
  grams: Decimal
  /** In the base currency. */
  cost: Decimal
  /** Per gram. */
  price?: Quote
  value?: Decimal
  gain?: Decimal
  /** Purchases and sales, newest first. */
  entries: string[]
}

/** A purchase or sale; `amount` is everything paid or received. */
export interface MetalInput {
  entry_id: string
  date: string
  metal: Metal
  grams: Decimal
  amount: Decimal
  currency: string
  fx_rate?: Decimal | null
  description: string
}

export interface MetalPrice {
  date: string
  metal: Metal
  per_gram: Decimal
  currency: string
  fx_rate?: Decimal | null
}

export interface Snapshot {
  settings: Settings
  plan: Plan
  valuations: Valuation[]
  /** Outside the projections: they cannot be sold at will. */
  private: PrivateHolding[]
  /** Their cash is also in `cash`. */
  banks: BankAccount[]
  /** Coins and bars; also in `valuations` at the PHYSICAL location. */
  metals: MetalHolding[]
  fx: [string, FxQuote][]
  holdings: Holding[]
  cash: CashBalance[]
  trial_balance: TrialBalanceRow[]
  instruments: Instrument[]
  /** Newest first. */
  entries: Entry[]
}

export interface ImportReport {
  added: number
  duplicates: number
  warnings: string[]
}

export type Treatment = 'exempt' | 'taxable' | 'venue_unknown' | 'not_estimated'

export interface Realization {
  date: string
  entry_id: string
  symbol: string
  units: Decimal
  proceeds: Decimal
  cost: Decimal
  gain: Decimal
  venue?: Venue
  /** Coins or bars rather than a security. */
  physical: boolean
  /** When the oldest of the units sold was bought. */
  held_from?: string | null
  /** The payout of a private holding. */
  private: boolean
}

export interface GainRow extends Realization {
  treatment: Treatment
}

export interface Gains {
  pack: { id: string; version: string; country: string; authority: string; website?: string; tax_year: number; currency: string; verified: boolean; notes?: string }
  /** On the whole gain: the pack's rate times the share that is taxed. */
  rate: Decimal
  /** The share of the gain that is taxed, when not all of it. */
  inclusion: Decimal | null
  /** The pack's own rate, on the taxed share. */
  marginal: Decimal
  /** The year's tax is worked out by band, so `rate` is only the first band. */
  banded: boolean
  /** Net gains up to this much a year are tax-free, in the pack's currency. */
  allowance: Decimal | null
  form: string
  /** How the pack taxes coins and bars, when it says. */
  metals: { treatment: 'exempt' | 'capital_gains' | 'other'; exempt_after_months?: number; note?: string } | null
  rows: GainRow[]
  /** Pay from work that is not salary, such as a co-investment paid out through payroll. */
  employment: { date: string; entry_id: string; description: string; income: Decimal; withheld: Decimal }[]
  employment_rule: { rate?: Decimal; form: string; note?: string } | null
  summary: { taxable_gains: Decimal; taxable_losses: Decimal; tax_due: Decimal; exempt: Decimal; unknown: Decimal; not_estimated: Decimal; allowance_used: Decimal }
}

export type FeeKind = 'commission' | 'fx_conversion' | 'exchange' | 'transaction_tax' | 'other'

export interface TradeInput {
  /** Chosen when the form opens, so recording twice adds the trade once. */
  id: string
  date: string
  /** Account segment, e.g. IBKR. */
  broker: string
  side: 'buy' | 'sell'
  symbol: string
  name?: string | null
  isin?: string | null
  units: Decimal
  /** Per unit, in `currency`. */
  price: Decimal
  currency: string
  fees: { kind: FeeKind; amount: Decimal }[]
  venue?: string | null
  /** Base currency for one unit of `currency`. */
  fx_rate?: Decimal | null
}

export interface TradeOutlook {
  entry: Entry
  value: Decimal
  fees: Decimal
  fx_rate: Decimal
  cost: Decimal | null
  gain: Decimal | null
  units_before: Decimal
  units_after: Decimal
  cash: { currency: string; before: Decimal; after: Decimal }[]
  holdings: { symbol: string; before: Decimal; after: Decimal }[]
  invested_before: Decimal
  invested_after: Decimal
  cash_before: Decimal
  cash_after: Decimal
  /** Money to deposit for the trade, in the base currency; the projection counts it. */
  deposit: Decimal
  tax: { treatment: Treatment; rate: Decimal; form: string; estimate: Decimal; verified: boolean } | null
  /** Each year of the projection, without and with the trade. */
  before: Band[]
  after: Band[]
}

// The core's errors read "English text\0{"code","args"}". Thrown again in
// the reader's language, with numbers written their way.
function readable(e: unknown): unknown {
  if (!(e instanceof Error) || !e.message.includes('\u0000')) return e
  const [english, detail] = e.message.split('\u0000')
  try {
    const { code, args } = JSON.parse(detail) as { code: string; args: Record<string, string> }
    const key = `error.${code}`
    if (!translates(key)) return new Error(english)
    const values = Object.fromEntries(Object.entries(args).map(([k, v]) => [k, /^-?\d+(\.\d+)?$/.test(v) && k !== 'line' && k !== 'version' ? new Intl.NumberFormat(i18n.lang, { maximumFractionDigits: 8 }).format(Number(v)) : v]))
    return new Error(t(key, values))
  } catch {
    return new Error(english)
  }
}

function guarded<T>(run: () => T): T {
  try {
    return run()
  } catch (e) {
    throw readable(e)
  }
}

/** Every method of the session, rethrowing the core's errors readably. */
function guard(session: Session): Session {
  return new Proxy(session, {
    get(target, name, receiver) {
      const value = Reflect.get(target, name, receiver)
      return typeof value === 'function' ? (...args: unknown[]) => guarded(() => value.apply(target, args)) : value
    },
  })
}

/** One of the assistant's tools, as the core describes it. */
export interface AssistantTool {
  name: string
  description: string
  input_schema: Record<string, unknown>
}

/** Guidance for the assistant's model; the core must be loaded. */
export const assistantGuidance = (): string => assistantInstructions()

/** Whether an assistant tool changes the book; the core must be loaded. */
export const changesBook = (name: string): boolean => assistantWrites(name)

/** An unlocked book. Its key exists only inside this object. */
export class Book {
  private readonly session: Session

  private constructor(session: Session) {
    this.session = guard(session)
  }

  static async create(settings: Settings, password: string): Promise<Book> {
    await loadCore()
    return new Book(guarded(() => Session.create(JSON.stringify(settings), password)))
  }

  /** The made-up example portfolio, dated so its prices are today's. */
  static async example(today: string, language: string): Promise<Book> {
    await loadCore()
    return new Book(guarded(() => Session.example(today, language)))
  }

  static async unlock(bytes: Uint8Array, password: string): Promise<Book> {
    await loadCore()
    return new Book(guarded(() => Session.unlock(bytes, password)))
  }

  /** Reopens a draft with a key kept from `exportKey`; throws if it does not fit. */
  static async resume(bytes: Uint8Array, key: Uint8Array): Promise<Book> {
    await loadCore()
    return new Book(guarded(() => Session.resume(bytes, key)))
  }

  /** Opens the file without the password: handle like the password itself. */
  exportKey(): Uint8Array {
    return this.session.exportKey()
  }

  /** Encrypted file contents. */
  seal(): Uint8Array {
    return this.session.seal()
  }

  snapshot(): Snapshot {
    return JSON.parse(this.session.snapshot())
  }

  import(brokerToml: string, fileName: string, text: string): ImportReport {
    return JSON.parse(this.session.import(brokerToml, fileName, text))
  }

  gains(year: number, packToml: string): Gains {
    return JSON.parse(this.session.gains(year, packToml))
  }

  sales(year: number): Realization[] {
    return JSON.parse(this.session.sales(year))
  }

  setPlan(plan: Plan): void {
    this.session.setPlan(JSON.stringify(plan))
  }

  /** `pace` scales planned monthly amounts: 1 = as planned. */
  goals(thisYear: number, market: Market, pace = 1): GoalStatus[] {
    return JSON.parse(this.session.goals(thisYear, market, pace))
  }

  /** `today` is YYYY-MM. */
  whatIf(input: { today: string; years: number; market: Market; spend_goals: boolean; retirement?: RetireInput }): WhatIf {
    return JSON.parse(this.session.whatIf(JSON.stringify(input)))
  }

  /** Throws with a readable message when the trade cannot be made. */
  previewTrade(input: TradeInput, packToml: string | null, whatIf: { today: string; years: number; market: Market; spend_goals: boolean }): TradeOutlook {
    return JSON.parse(this.session.previewTrade(JSON.stringify(input), packToml ?? undefined, JSON.stringify(whatIf)))
  }

  recordTrade(input: TradeInput): Entry {
    return JSON.parse(this.session.recordTrade(JSON.stringify(input)))
  }

  undoTrade(id: string): void {
    this.session.undoTrade(id)
  }

  flows(): MonthFlow[] {
    return JSON.parse(this.session.flows())
  }

  addBank(input: BalanceInput & { name: string; currency: string }): Entry | null {
    return JSON.parse(this.session.addBank(JSON.stringify(input)))
  }

  setBankBalance(id: string, input: BalanceInput): Entry | null {
    return JSON.parse(this.session.setBankBalance(id, JSON.stringify(input)))
  }

  renameBank(id: string, name: string): void {
    this.session.renameBank(id, name)
  }

  removeBank(id: string): void {
    this.session.removeBank(id)
  }

  buyMetal(input: MetalInput): Entry {
    return JSON.parse(this.session.buyMetal(JSON.stringify(input)))
  }

  sellMetal(input: MetalInput): Entry {
    return JSON.parse(this.session.sellMetal(JSON.stringify(input)))
  }

  setMetalPrice(input: MetalPrice): void {
    this.session.setMetalPrice(JSON.stringify(input))
  }

  removeMetalEntry(id: string): void {
    this.session.removeMetalEntry(id)
  }

  addPrivate(input: PrivateInput): Entry {
    return JSON.parse(this.session.addPrivate(JSON.stringify(input)))
  }

  /** An empty amount clears the value. */
  setPrivateValue(id: string, date: string, amount: string): void {
    this.session.setPrivateValue(id, date, amount)
  }

  payOutPrivate(id: string, input: PayoutInput): Entry {
    return JSON.parse(this.session.payOutPrivate(id, JSON.stringify(input)))
  }

  undoPrivatePayout(id: string): void {
    this.session.undoPrivatePayout(id)
  }

  removePrivate(id: string): void {
    this.session.removePrivate(id)
  }

  /** Tax packs for the assistant, keyed `CC/YEAR`. */
  setPacks(packs: Record<string, string>): void {
    this.session.setPacks(JSON.stringify(packs))
  }

  assistantTools(writes: boolean): AssistantTool[] {
    return JSON.parse(this.session.assistantTools(writes))
  }

  /** Runs a tool; throws with a readable message when it fails. */
  assistantCall(name: string, args: unknown, today: string): unknown {
    return JSON.parse(this.session.assistantCall(name, JSON.stringify(args ?? {}), crypto.randomUUID(), today))
  }

  setTaxResidency(country: string): void {
    this.session.setTaxResidency(country)
  }

  setPrice(symbol: string, date: string, amount: string, currency: string): void {
    this.session.setPrice(symbol, date, amount, currency)
  }

  setFxRate(currency: string, date: string, rate: string): void {
    this.session.setFxRate(currency, date, rate)
  }

  /** Throws "wrong password" when `current` does not match. */
  changePassword(current: string, next: string): void {
    this.session.changePassword(current, next)
  }

  setLanguage(language: string): void {
    this.session.setLanguage(language)
  }

  close(): void {
    this.session.free()
  }
}

export function brokerFit(brokerToml: string, text: string): number {
  return fit(brokerToml, text)
}

export function project(input: { start: number; monthly: number; years: number; market: Market; risk: Risk; target?: number }): Projection {
  return JSON.parse(runProjection(JSON.stringify(input)))
}
