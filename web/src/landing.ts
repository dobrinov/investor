// The marketing page is static HTML; this brings in its fonts and styles and
// fills the changelog from CHANGELOG.md at the repository root.
import '@fontsource/ibm-plex-sans/400.css'
import '@fontsource/ibm-plex-sans/600.css'
import '@fontsource/ibm-plex-mono/400.css'
import './landing.css'
import changelog from '../../CHANGELOG.md?raw'

type Release = { version: string; date: string; items: string[] }

// Reads "## 1.2.3 — 2026-01-31" headings and the "- " bullets under them.
function parseChangelog(text: string): Release[] {
  const releases: Release[] = []
  for (const line of text.split('\n')) {
    const heading = /^##\s+(.+?)(?:\s+[—–-]\s+(\d{4}-\d{2}-\d{2}))?\s*$/.exec(line)
    if (heading) {
      releases.push({ version: heading[1], date: heading[2] ?? '', items: [] })
    } else if (releases.length && /^[-*]\s+/.test(line)) {
      releases[releases.length - 1].items.push(line.replace(/^[-*]\s+/, ''))
    }
  }
  return releases.filter((r) => r.items.length)
}

const escape = (s: string) =>
  s.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!)

// Escaped text with `code` spans.
const inline = (s: string) => escape(s).replace(/`([^`]+)`/g, '<code>$1</code>')

function formatDate(iso: string): string {
  const date = new Date(`${iso}T00:00:00Z`)
  if (Number.isNaN(date.getTime())) return iso
  return date.toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric', timeZone: 'UTC' })
}

const target = document.querySelector<HTMLElement>('[data-changelog]')
if (target) {
  target.innerHTML = parseChangelog(changelog)
    .slice(0, 4)
    .map(
      (r) => `<article class="card release">
  <h3><span class="version">${escape(r.version)}</span>${
    r.date ? `<time datetime="${escape(r.date)}">${escape(formatDate(r.date))}</time>` : ''
  }</h3>
  <ul>${r.items.map((item) => `<li>${inline(item)}</li>`).join('')}</ul>
</article>`,
    )
    .join('')
}
