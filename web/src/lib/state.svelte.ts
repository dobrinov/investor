import type { BalanceInput, Entry, Plan, PrivateInput, Snapshot, TradeInput } from './core'
import type { SectionId } from './guide/types'
import { saveFile, saveFileAs, type SaveResult } from './files'
import { useLanguage } from './i18n.svelte'
import { Book } from './core'
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
  private handle = $state.raw<FileSystemFileHandle | undefined>()

  open(book: Book, fileName: string, options: { handle?: FileSystemFileHandle; changes?: number } = {}): void {
    this.book = book
    this.fileName = fileName
    this.handle = options.handle
    this.changes = options.changes ?? 0
    this.view = 'overview'
    this.refresh()
    useLanguage(this.snapshot!.settings.language)
    keepSessionKey(book.exportKey())
  }

  /** Reopens the browser copy after a refresh, when this tab kept the key. */
  async resume(): Promise<boolean> {
    const key = sessionKey()
    if (!key) return false
    const draft = await loadDraft()
    try {
      if (!draft) throw new Error('no draft')
      this.open(await Book.resume(draft.bytes, key), draft.fileName, { changes: draft.changes, handle: draft.handle })
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
    if (!this.book) return
    try {
      await saveDraft({ fileName: this.fileName, bytes: this.book.seal(), changes: this.changes, updatedAt: new Date().toISOString(), handle: this.handle })
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
    if (!this.book) return
    await this.saved(await saveFile(this.book.seal(), this.fileName, this.handle))
  }

  /** Saves to a new file; later saves go there. */
  async saveAs(): Promise<void> {
    if (!this.book) return
    await this.saved(await saveFileAs(this.book.seal(), this.fileName))
  }

  private async saved(result: SaveResult): Promise<void> {
    if (!result.saved) return
    this.handle = result.handle ?? this.handle
    this.fileName = result.name ?? this.fileName
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

  async removeBank(id: string): Promise<void> {
    this.book?.removeBank(id)
    await this.changed()
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

  async removePrivate(id: string): Promise<void> {
    this.book?.removePrivate(id)
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
    forgetSessionKey()
    this.book?.close()
    this.book = null
    this.snapshot = null
    this.handle = undefined
  }
}

export const app = new AppState()
