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
