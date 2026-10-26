// Journal entries keep the description they were written with, in English,
// so a file reads the same everywhere. These show them in the language
// chosen here, by matching the patterns the app writes them in.
import { en, type Key } from './locales/en'
import { t } from './i18n.svelte'

// Longer patterns first, so "paid out … into …" wins over "paid …".
const KEYS = [
  'desc.openingSnapshot',
  'desc.privatePaidOutInto',
  'desc.privatePaidOut',
  'desc.privatePaidFrom',
  'desc.privatePaid',
  'desc.bankOpening',
  'desc.bankNow',
  'desc.metalBought',
  'desc.metalSold',
  'desc.buy',
  'desc.sell',
  'desc.dividend',
  'desc.withholdingOf',
  'desc.withholding',
  'desc.convert',
  'desc.deposit',
  'desc.withdrawal',
  'desc.fee',
  'desc.interest',
  'note.dust',
  'note.provisional',
] as const satisfies readonly Key[]

const METALS: Record<string, Key> = {
  'Gold (physical)': 'metal.gold',
  'Silver (physical)': 'metal.silver',
  'Platinum (physical)': 'metal.platinum',
  'Palladium (physical)': 'metal.palladium',
}

const patterns = KEYS.map((key) => {
  const names: string[] = []
  const source = en[key]
    .split(/(\{\w+\})/)
    .map((part) => {
      const name = /^\{(\w+)\}$/.exec(part)?.[1]
      if (!name) return part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      names.push(name)
      // Tight shapes, so only the app's own wording matches, not text
      // someone typed: a ticker, a number, or an amount with its currency.
      if (name === 'symbol') return '([A-Z0-9][A-Z0-9.\\-]*)'
      if (['units', 'held', 'left'].includes(name)) return '(-?[\\d.]+)'
      if (['amount', 'sold', 'bought'].includes(name)) return '(-?[\\d.]+ [A-Z]{3})'
      return '(.+?)'
    })
    .join('')
  return { key, names, re: new RegExp(`^${source}$`) }
})

/** The description or note in the chosen language, or as written when it is not one of the app's own. */
export function describe(text: string): string {
  for (const { key, names, re } of patterns) {
    const m = re.exec(text)
    if (!m) continue
    const params: Record<string, string> = {}
    names.forEach((name, i) => (params[name] = name === 'metal' && METALS[m[i + 1]] ? t(METALS[m[i + 1]]) : m[i + 1]))
    return t(key, params)
  }
  return text
}
