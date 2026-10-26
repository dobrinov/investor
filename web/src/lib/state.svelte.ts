import type { BalanceInput, DebtTerms, Entry, MetalInput, MetalPrice, PayoutInput, Plan, PrivateInput, Snapshot, TradeInput } from './core'
import type { SectionId } from './guide/types'
import { modifiedAt, saveFile, saveFileAs, type FileRef, type SaveResult } from './files'
import { t, useLanguage } from './i18n.svelte'
import { Book, changesBook } from './core'
import { packs } from './configs'
import { forgetSessionKey, keepSessionKey, loadDraft, saveDraft, sessionKey } from './storage'

const VIEWS = ['overview', 'trade', 'goals', 'projections', 'ledger', 'import', 'tax', 'settings', 'guide'] as const
export type View = (typeof VIEWS)[number]

class AppState {
  book = $state.raw<Book | null>(null)
  snapshot = $state.raw<Snapshot | null>(null)
  fileName = $state('portfolio.invest')
  /** Changes made since the file was last saved. */
  changes = $state(0)
  view = $state<View>('overview')
  /** Section the guide should scroll to when it opens. */
  guideSection = $state<SectionId | null>(null)
  /** False when the browser refuses to keep a draft (private mode, blocked storage). */
  draftKept = $state(true)
  private handle = $state.raw<FileRef | undefined>()
  /** The file's modification time when it was opened or last saved here, to
   * notice changes made outside the app, e.g. by an assistant. */
  private seen: number | null = null
  /** The example portfolio is open: nothing is kept or saved. */
  example = $state(false)

  open(book: Book, fileName: string, options: { handle?: FileRef; changes?: number; modified?: number | null } = {}): void {
    this.example = false
    this.book = book
    book.setPacks(Object.fromEntries(packs.map((p) => [`${p.country}/${p.year}`, p.toml])))
    this.fileName = fileName
    this.handle = options.handle
    this.seen = options.modified ?? null
    if (options.modified === undefined && options.handle) void modifiedAt(options.handle).then((m) => (this.seen = m))
    this.changes = options.changes ?? 0
    this.view = 'overview'
    this.refresh()
    useLanguage(this.snapshot!.settings.language)
    keepSessionKey(book.exportKey())
  }

  /** Opens the made-up example portfolio. It never touches the browser copy of your own file. */
  async openExample(language: string, view?: string): Promise<void> {
    const today = new Date().toISOString().slice(0, 10)
    const book = await Book.example(today, language)
    this.example = true
    this.book = book
    book.setPacks(Object.fromEntries(packs.map((p) => [`${p.country}/${p.year}`, p.toml])))
    this.fileName = 'example.invest'
    this.handle = undefined
    this.changes = 0
    this.view = VIEWS.includes(view as View) ? (view as View) : 'overview'
    this.refresh()
  }

  /** Reopens the browser copy after a refresh, when this tab kept the key. */
  async resume(): Promise<boolean> {
    const key = sessionKey()
    if (!key) return false
    const draft = await loadDraft()
    try {
      if (!draft) throw new Error('no draft')
      this.open(await Book.resume(draft.bytes, key), draft.fileName, { changes: draft.changes, handle: draft.handle, modified: draft.modified })
      // Stay on the screen the address names.
      const view = location.hash.slice(1) as View
      if (VIEWS.includes(view)) this.view = view
      return true
    } catch {
      forgetSessionKey()
      return false
    }
  }

  openGuide(section: SectionId): void {
    this.guideSection = section
    this.view = 'guide'
  }

  refresh(): void {
    this.snapshot = this.book?.snapshot() ?? null
  }

  /** Call after anything that alters the book. */
  async changed(count = 1): Promise<void> {
    this.changes += count
    this.refresh()
    await this.keepDraft()
  }

  async keepDraft(): Promise<void> {
    if (!this.book || this.example) return
    try {
      await saveDraft({ fileName: this.fileName, bytes: this.book.seal(), changes: this.changes, updatedAt: new Date().toISOString(), handle: this.handle, modified: this.seen })
      this.draftKept = true
    } catch {
      this.draftKept = false
    }
  }

  /** True when saving goes straight back into the opened file. */
  get hasFile(): boolean {
    return this.handle !== undefined
  }

  /** Saves into the opened file, or asks where when there is none. */
  async save(): Promise<void> {
    if (!this.book || this.example) return
    if (this.handle && this.seen !== null) {
      const now = await modifiedAt(this.handle)
      if (now !== null && now > this.seen + 1000 && !confirm(t('shell.changedOutside', { name: this.fileName }))) return
    }
    await this.saved(await saveFile(this.book.seal(), this.fileName, this.handle))
  }

  /** Saves to a new file; later saves go there. */
  async saveAs(): Promise<void> {
    if (!this.book || this.example) return
    await this.saved(await saveFileAs(this.book.seal(), this.fileName))
  }

  private async saved(result: SaveResult): Promise<void> {
    if (!result.saved) return
    this.handle = result.handle ?? this.handle
    this.fileName = result.name ?? this.fileName
    this.seen = this.handle ? await modifiedAt(this.handle) : null
    this.changes = 0
    await this.keepDraft()
  }

  async setPlan(plan: Plan): Promise<void> {
    this.book?.setPlan(plan)
    await this.changed()
  }

  async recordTrade(input: TradeInput): Promise<Entry | undefined> {
    const entry = this.book?.recordTrade(input)
    await this.changed()
    return entry
  }

  async undoTrade(id: string): Promise<void> {
    this.book?.undoTrade(id)
    await this.changed()
  }

  async addBank(input: BalanceInput & { name: string; currency: string }): Promise<void> {
    this.book?.addBank(input)
    await this.changed()
  }

  async setBankBalance(id: string, input: BalanceInput): Promise<void> {
    this.book?.setBankBalance(id, input)
    await this.changed()
  }

  async renameBank(id: string, name: string): Promise<void> {
    this.book?.renameBank(id, name)
    await this.changed()
  }

  async removeBank(id: string): Promise<void> {
    this.book?.removeBank(id)
    await this.changed()
  }

  async addDebt(input: BalanceInput & DebtTerms & { currency: string }): Promise<void> {
    this.book?.addDebt(input)
    await this.changed()
  }

  async setDebtOwed(id: string, input: BalanceInput): Promise<void> {
    this.book?.setDebtOwed(id, input)
    await this.changed()
  }

  async editDebt(id: string, terms: DebtTerms): Promise<void> {
    this.book?.editDebt(id, terms)
    await this.changed()
  }

  async removeDebt(id: string): Promise<void> {
    this.book?.removeDebt(id)
    await this.changed()
  }

  async buyMetal(input: MetalInput): Promise<void> {
    this.book?.buyMetal(input)
    await this.changed()
  }

  async sellMetal(input: MetalInput): Promise<void> {
    this.book?.sellMetal(input)
    await this.changed()
  }

  async setMetalPrice(input: MetalPrice): Promise<void> {
    this.book?.setMetalPrice(input)
    await this.changed()
  }

  async removeMetalEntry(id: string): Promise<void> {
    this.book?.removeMetalEntry(id)
    await this.changed()
  }

  /** Runs one of the assistant's tools; a tool that writes counts as a change to save. */
  async assistantCall(name: string, args: unknown): Promise<unknown> {
    if (!this.book) throw new Error('no file is open')
    const result = this.book.assistantCall(name, args, new Date().toISOString().slice(0, 10))
    if (changesBook(name)) await this.changed()
    return result
  }

  async addPrivate(input: PrivateInput): Promise<Entry | undefined> {
    const entry = this.book?.addPrivate(input)
    await this.changed()
    return entry
  }

  async setPrivateValue(id: string, date: string, amount: string): Promise<void> {
    this.book?.setPrivateValue(id, date, amount)
    await this.changed()
  }

  async payOutPrivate(id: string, input: PayoutInput): Promise<void> {
    this.book?.payOutPrivate(id, input)
    await this.changed()
  }

  async undoPrivatePayout(id: string): Promise<void> {
    this.book?.undoPrivatePayout(id)
    await this.changed()
  }

  async removePrivate(id: string): Promise<void> {
    this.book?.removePrivate(id)
    await this.changed()
  }

  async setTaxResidency(country: string): Promise<void> {
    this.book?.setTaxResidency(country)
    await this.changed()
  }

  async setPrice(symbol: string, date: string, amount: string, currency: string): Promise<void> {
    this.book?.setPrice(symbol, date, amount, currency)
    await this.changed()
  }

  async setFxRate(currency: string, date: string, rate: string): Promise<void> {
    this.book?.setFxRate(currency, date, rate)
    await this.changed()
  }

  /** The browser copy is re-encrypted with the new password straight away. */
  async changePassword(current: string, next: string): Promise<void> {
    this.book?.changePassword(current, next)
    if (this.book) keepSessionKey(this.book.exportKey())
    await this.changed()
  }

  setLanguage(lang: string): void {
    useLanguage(lang)
    if (this.book && this.snapshot?.settings.language !== lang) {
      this.book.setLanguage(lang)
      void this.changed()
    }
  }

  lock(): void {
    if (this.example) {
      this.example = false
      history.replaceState(null, '', location.pathname)
    } else forgetSessionKey()
    this.book?.close()
    this.book = null
    this.snapshot = null
    this.handle = undefined
  }
}

export const app = new AppState()
