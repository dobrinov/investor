import { bg } from './locales/bg'
import { en, type Key } from './locales/en'
import { recall, remember } from './storage'

const dictionaries: Record<string, Record<Key, string>> = { en, bg }

export const languages = [
  { code: 'en', name: 'English' },
  { code: 'bg', name: 'Български' },
]

/** Before a file is unlocked, the last language used here or the browser's. */
function initial(): string {
  const saved = recall('language')
  if (saved && saved in dictionaries) return saved
  const browser = navigator.language.slice(0, 2)
  return browser in dictionaries ? browser : 'en'
}

export const i18n = $state({ lang: initial() })

export function useLanguage(lang: string): void {
  if (!(lang in dictionaries)) return
  i18n.lang = lang
  remember('language', lang)
}

export function t(key: Key, params?: Record<string, string | number>): string {
  let text = dictionaries[i18n.lang]?.[key] ?? en[key]
  for (const [name, value] of Object.entries(params ?? {})) {
    text = text.replaceAll(`{${name}}`, String(value))
  }
  return text
}
