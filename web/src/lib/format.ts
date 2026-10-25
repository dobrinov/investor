import type { Decimal } from './core'
import { i18n } from './i18n.svelte'

export function money(value: Decimal | number, currency?: string): string {
  return new Intl.NumberFormat(i18n.lang, {
    style: currency ? 'currency' : 'decimal',
    currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Number(value))
}

export function units(value: Decimal | number): string {
  return new Intl.NumberFormat(i18n.lang, { maximumFractionDigits: 8 }).format(Number(value))
}

export function date(iso: string): string {
  return new Intl.DateTimeFormat(i18n.lang, { dateStyle: 'medium' }).format(new Date(`${iso}T00:00:00`))
}

export function dateTime(iso: string): string {
  return new Intl.DateTimeFormat(i18n.lang, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(iso))
}

/** Rounded to whole units, for estimates where cents mean nothing. */
export function whole(value: number, currency: string): string {
  return new Intl.NumberFormat(i18n.lang, { style: 'currency', currency, maximumFractionDigits: 0 }).format(value)
}

/** `YYYY-MM` as a short month and year. */
export function month(ym: string): string {
  return new Intl.DateTimeFormat(i18n.lang, { month: 'short', year: 'numeric' }).format(new Date(`${ym}-01T00:00:00`))
}

/** A country's name in the interface language, e.g. `DE` → Germany. */
export function country(code: string): string {
  try {
    return new Intl.DisplayNames([i18n.lang], { type: 'region' }).of(code) ?? code
  } catch {
    return code
  }
}

/**
 * A typed amount as a plain decimal string. Accepts "168,81", "168.81",
 * "23,882.00", "23.882,00" and "23 882,00": the last separator is the decimal
 * one when both kinds appear, and repeated ones are grouping.
 */
export function decimal(v: string): string {
  const s = v.trim().replace(/[\s ']/g, '')
  const comma = s.lastIndexOf(','), dot = s.lastIndexOf('.')
  if (comma >= 0 && dot >= 0) return comma > dot ? s.replace(/\./g, '').replace(',', '.') : s.replace(/,/g, '')
  const sep = comma >= 0 ? ',' : '.'
  return s.split(sep).length > 2 ? s.split(sep).join('') : s.replace(',', '.')
}
