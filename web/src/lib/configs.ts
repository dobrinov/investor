// Broker configs and tax packs are the same TOML files the repository holds,
// bundled at build time. Adding a file to brokers/ or jurisdictions/ adds it here.

const brokerFiles = import.meta.glob('../../../brokers/*.toml', { query: '?raw', import: 'default', eager: true }) as Record<string, string>
const packFiles = import.meta.glob('../../../jurisdictions/*/*.toml', { query: '?raw', import: 'default', eager: true }) as Record<string, string>

function value(toml: string, key: string): string | undefined {
  return new RegExp(`^${key}\\s*=\\s*"([^"]*)"`, 'm').exec(toml)?.[1]
}

export interface BrokerDef {
  id: string
  name: string
  toml: string
}

export interface PackDef {
  country: string
  year: number
  authority: string
  toml: string
}

export const brokers: BrokerDef[] = Object.entries(brokerFiles)
  .map(([path, toml]) => ({ id: value(toml, 'id') ?? path, name: value(toml, 'name') ?? path, toml }))
  .sort((a, b) => a.name.localeCompare(b.name))

export const packs: PackDef[] = Object.entries(packFiles)
  .map(([path, toml]) => {
    const [country, file] = path.split('/').slice(-2)
    return { country: country.toUpperCase(), year: Number(file.replace('.toml', '')), authority: value(toml, 'authority') ?? '', toml }
  })
  .sort((a, b) => a.country.localeCompare(b.country) || b.year - a.year)
