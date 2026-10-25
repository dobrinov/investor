// The in-app guide, one file per language. Plain data so the same layout
// renders every language and nothing has to be fetched.

export type Block =
  | { p: string }
  | { steps: string[] }
  | { list: string[] }
  | { code: string }
  | { terms: [string, string][] }
  | { link: string; href: string }

export interface Section {
  id: SectionId
  title: string
  blocks: Block[]
}

export type SectionId = 'start' | 'file' | 'example' | 'bank' | 'metals' | 'private' | 'claude' | 'export-ibkr' | 'export-revolut' | 'add-broker' | 'add-tax' | 'glossary'
