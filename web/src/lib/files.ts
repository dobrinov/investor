// Opening and saving files on the user's computer. The desktop app and
// Chrome and Edge save back to the same file; other browsers download a new
// copy.

interface PickerType {
  description: string
  accept: Record<string, string[]>
}

interface FilePickers {
  showOpenFilePicker?: (options: { types: PickerType[]; multiple: boolean }) => Promise<FileSystemFileHandle[]>
  showSaveFilePicker?: (options: { suggestedName: string; types: PickerType[] }) => Promise<FileSystemFileHandle>
}

const pickers = window as unknown as FilePickers
const types: PickerType[] = [{ description: 'Investor file', accept: { 'application/octet-stream': ['.invest'] } }]

// The desktop app (Tauri) opens and saves through its own commands, by path.
type Invoke = (command: string, args?: unknown, options?: { headers: Record<string, string> }) => Promise<unknown>
const tauri = (window as unknown as { __TAURI_INTERNALS__?: { invoke: Invoke } }).__TAURI_INTERNALS__
export const desktop = tauri !== undefined

export const savesInPlace = desktop || typeof pickers.showSaveFilePicker === 'function'

/** Where a file lives: a browser handle, or a path in the desktop app. */
export type FileRef = FileSystemFileHandle | { path: string }

export interface OpenedFile {
  name: string
  bytes: Uint8Array
  handle?: FileRef
}

interface Picked {
  path: string
  name: string
}

async function readPath(picked: Picked): Promise<OpenedFile> {
  const bytes = new Uint8Array((await tauri!.invoke('read_invest', { path: picked.path })) as ArrayBuffer)
  return { name: picked.name, bytes, handle: { path: picked.path } }
}

async function writePath(bytes: Uint8Array, path: string): Promise<void> {
  await tauri!.invoke('write_invest', bytes, { headers: { path: encodeURIComponent(path) } })
}

/** When the file was last changed on disk, in milliseconds; null if unknown. */
export async function modifiedAt(handle: FileRef): Promise<number | null> {
  try {
    if ('path' in handle) return tauri ? ((await tauri.invoke('modified_invest', { path: handle.path })) as number) : null
    return (await handle.getFile()).lastModified
  } catch {
    return null
  }
}

/** The .invest file the desktop app was started with, if any. */
export async function launchedFile(): Promise<OpenedFile | null> {
  if (!tauri) return null
  const picked = (await tauri.invoke('launch_file')) as Picked | null
  return picked ? readPath(picked) : null
}

function isAbort(e: unknown): boolean {
  return e instanceof DOMException && e.name === 'AbortError'
}

/** Asks for files with a plain file input; resolves to [] when cancelled. */
export function chooseFiles(accept: string, multiple: boolean): Promise<File[]> {
  return new Promise((resolve) => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = accept
    input.multiple = multiple
    input.addEventListener('change', () => resolve([...(input.files ?? [])]))
    input.addEventListener('cancel', () => resolve([]))
    input.click()
  })
}

export async function openInvestFile(): Promise<OpenedFile | null> {
  if (tauri) {
    const picked = (await tauri.invoke('pick_invest')) as Picked | null
    return picked ? readPath(picked) : null
  }
  if (pickers.showOpenFilePicker) {
    try {
      const [handle] = await pickers.showOpenFilePicker({ types, multiple: false })
      const file = await handle.getFile()
      return { name: file.name, bytes: new Uint8Array(await file.arrayBuffer()), handle }
    } catch (e) {
      if (isAbort(e)) return null
      throw e
    }
  }
  const [file] = await chooseFiles('.invest', false)
  return file ? { name: file.name, bytes: new Uint8Array(await file.arrayBuffer()) } : null
}

interface PermissionHandle {
  queryPermission?: (options: { mode: 'readwrite' }) => Promise<PermissionState>
  requestPermission?: (options: { mode: 'readwrite' }) => Promise<PermissionState>
}

/** A handle restored from the browser copy needs the user's permission again. */
async function canWrite(handle: FileSystemFileHandle): Promise<boolean> {
  const h = handle as FileSystemFileHandle & PermissionHandle
  if (!h.queryPermission || !h.requestPermission) return true
  if ((await h.queryPermission({ mode: 'readwrite' })) === 'granted') return true
  return (await h.requestPermission({ mode: 'readwrite' })) === 'granted'
}

/** The file behind a dropped item, where the browser can save back to it. */
export async function droppedFile(item: DataTransferItem): Promise<OpenedFile | null> {
  const withHandle = item as DataTransferItem & { getAsFileSystemHandle?: () => Promise<FileSystemHandle | null> }
  const handle = await withHandle.getAsFileSystemHandle?.().catch(() => null)
  if (handle?.kind === 'file') {
    const file = await (handle as FileSystemFileHandle).getFile()
    return { name: file.name, bytes: new Uint8Array(await file.arrayBuffer()), handle: handle as FileSystemFileHandle }
  }
  const file = item.getAsFile()
  return file ? { name: file.name, bytes: new Uint8Array(await file.arrayBuffer()) } : null
}

export interface SaveResult {
  saved: boolean
  handle?: FileRef
  name?: string
}

async function write(bytes: Uint8Array, handle: FileSystemFileHandle): Promise<void> {
  const writable = await handle.createWritable()
  await writable.write(bytes as Uint8Array<ArrayBuffer>)
  await writable.close()
}

/** Saves into the file that was opened; asks where only when there is none yet. */
export async function saveFile(bytes: Uint8Array, name: string, handle?: FileRef): Promise<SaveResult> {
  if (handle && 'path' in handle) {
    if (!tauri) return saveFileAs(bytes, name)
    await writePath(bytes, handle.path)
    return { saved: true, handle }
  }
  if (handle && (await canWrite(handle))) {
    await write(bytes, handle)
    return { saved: true, handle }
  }
  return saveFileAs(bytes, name)
}

/** Always asks where to save; later saves go to the new file. */
export async function saveFileAs(bytes: Uint8Array, name: string): Promise<SaveResult> {
  const data = bytes as Uint8Array<ArrayBuffer>
  if (tauri) {
    const picked = (await tauri.invoke('save_invest_as', { name })) as Picked | null
    if (!picked) return { saved: false }
    await writePath(bytes, picked.path)
    return { saved: true, handle: { path: picked.path }, name: picked.name }
  }
  if (pickers.showSaveFilePicker) {
    try {
      const picked = await pickers.showSaveFilePicker({ suggestedName: name, types })
      await write(bytes, picked)
      return { saved: true, handle: picked, name: picked.name }
    } catch (e) {
      if (isAbort(e)) return { saved: false }
      throw e
    }
  }
  const url = URL.createObjectURL(new Blob([data], { type: 'application/octet-stream' }))
  const link = document.createElement('a')
  link.href = url
  link.download = name
  link.click()
  setTimeout(() => URL.revokeObjectURL(url), 10_000)
  return { saved: true }
}
