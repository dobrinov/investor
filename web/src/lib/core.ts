// Typed wrapper around the Rust core compiled to WebAssembly.
// Amounts arrive as exact decimal strings; convert only for display.
import init, { Session, brokerFit as fit } from './core-wasm/core.js'

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

export interface Snapshot {
  settings: Settings
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
