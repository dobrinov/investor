// Typed wrapper around the Rust core compiled to WebAssembly.
// Amounts arrive as exact decimal strings; convert only for display.
import init, { Session, brokerFit as fit, project as runProjection } from './core-wasm/core.js'

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

export interface Snapshot {
  settings: Settings
  plan: Plan
  valuations: Valuation[]
  /** Outside the projections: they cannot be sold at will. */
  private: PrivateHolding[]
  /** Their cash is also in `cash`. */
  banks: BankAccount[]
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

export type Treatment = 'exempt' | 'taxable' | 'venue_unknown'

export interface Realization {
  date: string
  entry_id: string
  symbol: string
  units: Decimal
  proceeds: Decimal
  cost: Decimal
  gain: Decimal
  venue?: Venue
}

export interface GainRow extends Realization {
  treatment: Treatment
}

export interface Gains {
  pack: { id: string; version: string; country: string; authority: string; website?: string; tax_year: number; currency: string; verified: boolean; notes?: string }
  rate: Decimal
  form: string
  rows: GainRow[]
  summary: { taxable_gains: Decimal; taxable_losses: Decimal; tax_due: Decimal; exempt: Decimal; unknown: Decimal }
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

/** An unlocked book. Its key exists only inside this object. */
export class Book {
  private constructor(private readonly session: Session) {}

  static async create(settings: Settings, password: string): Promise<Book> {
    await loadCore()
    return new Book(Session.create(JSON.stringify(settings), password))
  }

  static async unlock(bytes: Uint8Array, password: string): Promise<Book> {
    await loadCore()
    return new Book(Session.unlock(bytes, password))
  }

  /** Reopens a draft with a key kept from `exportKey`; throws if it does not fit. */
  static async resume(bytes: Uint8Array, key: Uint8Array): Promise<Book> {
    await loadCore()
    return new Book(Session.resume(bytes, key))
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

  removeBank(id: string): void {
    this.session.removeBank(id)
  }

  addPrivate(input: PrivateInput): Entry {
    return JSON.parse(this.session.addPrivate(JSON.stringify(input)))
  }

  /** An empty amount clears the value. */
  setPrivateValue(id: string, date: string, amount: string): void {
    this.session.setPrivateValue(id, date, amount)
  }

  removePrivate(id: string): void {
    this.session.removePrivate(id)
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
