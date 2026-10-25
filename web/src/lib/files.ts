// Opening and saving files on the user's computer. Chrome and Edge can save
// back to the same file; other browsers download a new copy.

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

export const savesInPlace = typeof pickers.showSaveFilePicker === 'function'

export interface OpenedFile {
  name: string
  bytes: Uint8Array
  handle?: FileSystemFileHandle
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

export interface SaveResult {
  saved: boolean
  handle?: FileSystemFileHandle
  name?: string
}

export async function saveFile(bytes: Uint8Array, name: string, handle?: FileSystemFileHandle): Promise<SaveResult> {
  const data = bytes as Uint8Array<ArrayBuffer>
  if (handle) {
    const writable = await handle.createWritable()
    await writable.write(data)
    await writable.close()
    return { saved: true, handle }
  }
  if (pickers.showSaveFilePicker) {
    try {
      const picked = await pickers.showSaveFilePicker({ suggestedName: name, types })
      return saveFile(bytes, picked.name, picked).then((r) => ({ ...r, name: picked.name }))
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
