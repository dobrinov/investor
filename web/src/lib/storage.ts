// The browser copy of unsaved work, kept in IndexedDB. It holds the same
// encrypted bytes as a .invest file, never anything readable.

import type { FileRef } from './files'

export interface Draft {
  fileName: string
  bytes: Uint8Array
  /** Changes not yet saved to the file. */
  changes: number
  updatedAt: string
  /** The opened file, so saving after a refresh still goes back into it. */
  handle?: FileRef
  /** When that file was last changed on disk as far as the app knows. */
  modified?: number | null
}

const DB_NAME = 'investor'
const STORE = 'drafts'
const KEY = 'current'

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, 1)
    request.onupgradeneeded = () => request.result.createObjectStore(STORE)
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
}

async function run<T>(mode: IDBTransactionMode, action: (store: IDBObjectStore) => IDBRequest<T>): Promise<T> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, mode)
    const request = action(tx.objectStore(STORE))
    tx.oncomplete = () => {
      db.close()
      resolve(request.result)
    }
    tx.onerror = tx.onabort = () => {
      db.close()
      reject(tx.error)
    }
  })
}

export async function saveDraft(draft: Draft): Promise<void> {
  await run('readwrite', (store) => store.put(draft, KEY))
}

export async function loadDraft(): Promise<Draft | undefined> {
  try {
    return await run<Draft | undefined>('readonly', (store) => store.get(KEY))
  } catch {
    return undefined
  }
}

export async function clearDraft(): Promise<void> {
  await run('readwrite', (store) => store.delete(KEY))
}

export function remember(key: string, value: string): void {
  try {
    localStorage.setItem(`investor.${key}`, value)
  } catch {
    // Private mode or blocked storage: nothing to remember then.
  }
}

export function recall(key: string): string | null {
  try {
    return localStorage.getItem(`investor.${key}`)
  } catch {
    return null
  }
}

// The unlocked key, kept for this tab only so a refresh does not ask for the
// password again. Closing the tab, locking or the idle lock removes it.
const SESSION_KEY = 'investor.sessionKey'

/** Whether a refresh may reopen the file without the password. On unless turned off. */
export function staysUnlocked(): boolean {
  return recall('stayUnlocked') !== '0'
}

export function setStaysUnlocked(on: boolean): void {
  remember('stayUnlocked', on ? '1' : '0')
  if (!on) forgetSessionKey()
}

export function keepSessionKey(key: Uint8Array): void {
  if (!staysUnlocked()) return
  try {
    sessionStorage.setItem(SESSION_KEY, btoa(String.fromCharCode(...key)))
  } catch {
    // Storage blocked: the password is asked for after a refresh.
  }
}

export function sessionKey(): Uint8Array | null {
  try {
    const text = sessionStorage.getItem(SESSION_KEY)
    return text ? Uint8Array.from(atob(text), (c) => c.charCodeAt(0)) : null
  } catch {
    return null
  }
}

export function forgetSessionKey(): void {
  try {
    sessionStorage.removeItem(SESSION_KEY)
  } catch {
    // Nothing kept.
  }
}
