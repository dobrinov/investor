import type { Book, Snapshot } from './core'
import { saveFile } from './files'
import { useLanguage } from './i18n.svelte'
import { saveDraft } from './storage'

export type View = 'overview' | 'ledger' | 'import' | 'tax'

class AppState {
  book = $state.raw<Book | null>(null)
  snapshot = $state.raw<Snapshot | null>(null)
  fileName = $state('portfolio.invest')
  /** Changes made since the file was last saved. */
  changes = $state(0)
  view = $state<View>('overview')
  /** False when the browser refuses to keep a draft (private mode, blocked storage). */
  draftKept = $state(true)
  private handle: FileSystemFileHandle | undefined

  open(book: Book, fileName: string, options: { handle?: FileSystemFileHandle; changes?: number } = {}): void {
    this.book = book
    this.fileName = fileName
    this.handle = options.handle
    this.changes = options.changes ?? 0
    this.view = 'overview'
    this.refresh()
    useLanguage(this.snapshot!.settings.language)
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
      await saveDraft({ fileName: this.fileName, bytes: this.book.seal(), changes: this.changes, updatedAt: new Date().toISOString() })
      this.draftKept = true
    } catch {
      this.draftKept = false
    }
  }

  async save(): Promise<void> {
    if (!this.book) return
    const result = await saveFile(this.book.seal(), this.fileName, this.handle)
    if (!result.saved) return
    this.handle = result.handle ?? this.handle
    this.fileName = result.name ?? this.fileName
    this.changes = 0
    await this.keepDraft()
  }

  setLanguage(lang: string): void {
    useLanguage(lang)
    if (this.book && this.snapshot?.settings.language !== lang) {
      this.book.setLanguage(lang)
      void this.changed()
    }
  }

  lock(): void {
    this.book?.close()
    this.book = null
    this.snapshot = null
    this.handle = undefined
  }
}

export const app = new AppState()
