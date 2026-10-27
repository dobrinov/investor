// Charts the CFO can draw. The model describes a chart as a small JSON block
// (inside its briefing, or as a ```chart fence in an answer) and the app
// checks it and draws it. Any model can do this: no tools are needed.

export interface Item {
  label: string
  value: number
  /** A goal or comparison value drawn as a tick on the bar. */
  target?: number
  /** The parts folded together when there are too many to tell apart. */
  other?: boolean
}

export interface Series {
  name: string
  values: number[]
}

export type Figure =
  | { type: 'bar'; title: string; unit: string; items: Item[] }
  | { type: 'share'; title: string; unit: string; items: Item[] }
  | { type: 'line'; title: string; unit: string; labels: string[]; series: Series[] }

/** What the model is told about drawing charts. */
export const CHART_GUIDE = `You can draw charts. A chart is JSON in one of these shapes:
{"type": "bar", "title": "…", "unit": "EUR", "items": [{"label": "…", "value": 1200, "target": 1500}]}   compares amounts; "target" is optional (a goal or limit)
{"type": "share", "title": "…", "unit": "EUR", "items": [{"label": "…", "value": 60}]}   parts of a whole, e.g. where the money is
{"type": "line", "title": "…", "unit": "EUR", "labels": ["Jan", "Feb"], "series": [{"name": "…", "values": [1, 2]}]}   change over time, at most 3 series
"unit" is a currency code, "%", or a word such as "months". Use only figures from their picture or arithmetic on them; never invent data.
Draw a chart only when it makes a point clearer than words; at most 8 items, and say in words what it shows.`

const MAX_ITEMS = 8

const finite = (v: unknown): v is number => typeof v === 'number' && Number.isFinite(v)

function items(raw: unknown): Item[] | null {
  if (!Array.isArray(raw)) return null
  const list = raw
    .filter((i) => i && typeof i.label === 'string' && finite(i.value))
    .map((i) => ({ label: i.label as string, value: i.value as number, ...(finite(i.target) ? { target: i.target as number } : {}) }))
  return list.length ? list : null
}

/** A chart from what the model wrote, or null when it is not one we can draw. */
export function figure(raw: unknown): Figure | null {
  if (!raw || typeof raw !== 'object') return null
  const r = raw as Record<string, unknown>
  const title = typeof r.title === 'string' ? r.title : ''
  const unit = typeof r.unit === 'string' ? r.unit : ''
  if (r.type === 'bar') {
    const list = items(r.items)
    return list && { type: 'bar', title, unit, items: list.slice(0, 12) }
  }
  if (r.type === 'share' || r.type === 'pie' || r.type === 'donut') {
    let list = items(r.items)?.filter((i) => i.value > 0)
    if (!list?.length) return null
    list = [...list].sort((a, b) => b.value - a.value)
    // Past eight parts the colours stop being told apart: the rest are "Other".
    if (list.length > MAX_ITEMS) {
      const rest = list.slice(MAX_ITEMS - 1).reduce((s, i) => s + i.value, 0)
      list = [...list.slice(0, MAX_ITEMS - 1), { label: 'Other', value: rest, other: true }]
    }
    return { type: 'share', title, unit, items: list }
  }
  if (r.type === 'line' && Array.isArray(r.labels) && Array.isArray(r.series)) {
    const labels = r.labels.map(String)
    const series = r.series
      .filter((s) => s && Array.isArray(s.values) && s.values.length === labels.length && s.values.every(finite))
      .slice(0, 3)
      .map((s, i) => ({ name: typeof s.name === 'string' ? s.name : `${i + 1}`, values: s.values as number[] }))
    return labels.length >= 2 && series.length ? { type: 'line', title, unit, labels, series } : null
  }
  return null
}

export type Part = { kind: 'text'; text: string } | { kind: 'chart'; figure: Figure }

/** Splits an answer into text and the ```chart blocks in it. A block that is not a chart stays text. */
export function parts(text: string): Part[] {
  const out: Part[] = []
  const fence = /```(?:chart|json)?[ \t]*\n([\s\S]*?)```/g
  let at = 0
  for (const m of text.matchAll(fence)) {
    let f: Figure | null = null
    try {
      f = figure(JSON.parse(m[1]))
    } catch {
      // Not JSON: leave it as text.
    }
    if (!f) continue
    const before = text.slice(at, m.index).trim()
    if (before) out.push({ kind: 'text', text: before })
    out.push({ kind: 'chart', figure: f })
    at = m.index! + m[0].length
  }
  const rest = text.slice(at).trim()
  if (rest || !out.length) out.push({ kind: 'text', text: rest })
  return out
}
