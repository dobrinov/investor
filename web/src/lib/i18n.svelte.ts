import { en, type Key } from './locales/en'
import { recall, remember } from './storage'

export const languages = [
  { code: 'en', name: 'English' },
  { code: 'bg', name: 'Български' },
  { code: 'de', name: 'Deutsch' },
  { code: 'el', name: 'Ελληνικά' },
  { code: 'es', name: 'Español' },
  { code: 'fr', name: 'Français' },
  { code: 'it', name: 'Italiano' },
  { code: 'nl', name: 'Nederlands' },
  { code: 'pl', name: 'Polski' },
  { code: 'pt', name: 'Português' },
  { code: 'ro', name: 'Română' },
  { code: 'uk', name: 'Українська' },
]
const codes = languages.map((l) => l.code)

// Other languages load when chosen; English shows until they arrive.
const loaders = import.meta.glob<Record<Key, string>>(['./locales/*.ts', '!./locales/en.ts'], { import: '*' })
const dictionaries = $state<Record<string, Record<Key, string>>>({ en })

async function load(lang: string): Promise<void> {
  if (dictionaries[lang]) return
  const loader = loaders[`./locales/${lang}.ts`]
  if (!loader) return
  const module = (await loader()) as unknown as Record<string, Record<Key, string>>
  dictionaries[lang] = module[lang]
}

/** Before a file is unlocked, the last language used here or the browser's. */
function initial(): string {
  const saved = recall('language')
  if (saved && codes.includes(saved)) return saved
  const browser = navigator.language.slice(0, 2)
  return codes.includes(browser) ? browser : 'en'
}

export const i18n = $state({ lang: initial() })
void load(i18n.lang)

export function useLanguage(lang: string): void {
  if (!codes.includes(lang)) return
  i18n.lang = lang
  remember('language', lang)
  void load(lang)
}

/** Whether `key` has text, for keys built at run time. */
export function translates(key: string): key is Key {
  return key in en
}

export function t(key: Key, params?: Record<string, string | number>): string {
  let text = dictionaries[i18n.lang]?.[key] ?? en[key]
  for (const [name, value] of Object.entries(params ?? {})) {
    text = text.replaceAll(`{${name}}`, String(value))
  }
  return text
}

/** Text a tax pack gives in one language or several, by language code. */
export type Local = string | Record<string, string>

/** Pack text in the chosen language, else English, else any. */
export function local(text: Local | null | undefined): string {
  if (!text) return ''
  if (typeof text === 'string') return text
  return text[i18n.lang] ?? text.en ?? Object.values(text)[0] ?? ''
}
