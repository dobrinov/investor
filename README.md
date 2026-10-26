# Investor

Plan, track and rehearse investing decisions — with double-entry books,
scenario projections and tax reporting help. Local-first: no account, no
server database, no tracking. Everything lives in one password-encrypted
`.invest` file that you keep.

## Layout

| Path | What |
|------|------|
| `crates/core` | Ledger, FIFO lots, config-driven importers, tax packs, `.invest` encryption. Pure Rust, compiles to WebAssembly. |
| `crates/cli` | `investor` command for development and scripting. |
| `crates/mcp` | `investor-mcp`: MCP server so any AI assistant can read your file and record changes you approve. |
| `crates/wasm` and `web/` | The browser edition. |
| `desktop/` | The desktop app (Tauri) for Windows, macOS and Linux, wrapping the web app. |
| `brokers/` | One TOML file per broker or exchange statement format. |
| `jurisdictions/` | One tax pack per country and tax year. |
| `fixtures/` | Anonymised sample statements used by tests. |
| `samples/` | Your real statements. Git-ignored. |

## Try it

```sh
cargo build
export INVESTOR_PASSWORD='use a long passphrase'   # or leave unset to be prompted
target/debug/investor new my.invest
target/debug/investor import my.invest --broker brokers/ibkr-flex.toml samples/ibkr/*.csv
target/debug/investor import my.invest --broker brokers/revolut-trading.toml samples/revolut/*.csv
target/debug/investor holdings my.invest
target/debug/investor gains my.invest --year 2026 --pack jurisdictions/bg/2026.toml
```

How to export statements and add brokers: [docs/importing.md](docs/importing.md).

## Install it

- **Desktop app** for Windows, macOS and Linux: download it from
  [the latest release](https://github.com/dobrinov/investor/releases/latest)
  (`.msi`/`-setup.exe`, `.dmg`, `.AppImage`/`.deb`/`.rpm`). It opens and saves
  `.invest` files in place and never connects to the internet. The macOS app is
  signed and notarized by Apple. The Windows app is not signed yet, so choose
  More info, then Run anyway, on the first start.
- **From the browser**: open the app in Chrome or Edge and choose Install, or
  in Safari on a Mac choose File, then Add to Dock. It works offline after the
  first visit.

To build the desktop app yourself: `cd desktop && npx @tauri-apps/cli@2 build`
(needs Rust, pnpm and wasm-pack; on Linux also the WebKitGTK 4.1 development
packages). Pushing a `v*` tag builds all three systems and publishes a release
(`.github/workflows/desktop.yml`).

## Ask an AI assistant

Investor works without AI. If you want one, an assistant can read your
portfolio, answer with the app's own calculations, help you plan, and record
what you decide once you approve it.

**In the desktop app**, the ✦ button in the bottom-right corner opens the
assistant. Choose a model on your computer (Ollama or LM Studio, found
automatically; nothing leaves your computer) or Anthropic or OpenAI with
your own key. Reads run at once; every change shows an approval card and
then counts as an unsaved change. On the website the button explains that
the assistant needs the desktop app.

**From your own AI tool** (Claude Code, Cursor, VS Code or any other MCP
client): in the desktop app, open the assistant, choose **Your AI tool** and
turn it on. Copy the command it shows into Claude Code, or the address into
another tool's MCP settings. The tool works with the file open in the app,
only while it is open and unlocked, and every change it wants to make waits
for your approval in the app. The address carries a secret; make a new one
if it leaks. With a cloud model, what the tool reads is sent to that provider.

Without the desktop app, `investor-mcp` (attached to every release) serves the
same tools from a saved file: `investor-mcp --file portfolio.invest`, with the
password from the macOS Keychain (`investor-mcp --file … remember-password`),
`INVESTOR_PASSWORD_COMMAND` or `INVESTOR_PASSWORD`. It keeps the last 20
versions next to the file for undo; `--read-only` turns writing off.

**Skills**: `plugin/skills/` holds routines for a portfolio check-up,
rehearsing and recording a trade, tax season, retirement and what-if, and
adding a broker. In Claude Code:
`/plugin install investor --marketplace dobrinov/investor` (the plugin starts
`investor-mcp` from your `PATH` with `INVESTOR_FILE` set to your file). The
desktop app offers the same skills as starters.

## `.invest` format

Argon2id (64 MiB, 3 iterations) derives a key from the password; the JSON
document is sealed with AES-256-GCM, with the header authenticated. See
`crates/core/src/vault.rs`.

## Development

```sh
cargo test
cargo clippy --all-targets
# WebAssembly check (needs rustup's toolchain; Homebrew's Rust has no wasm target):
RUSTC=$(rustup which rustc) CARGO_TARGET_DIR=target/wasm \
  $(rustup which cargo) check -p investor-core --target wasm32-unknown-unknown
```

## Status

Browser edition, live at <https://dobrinov.github.io/investor/app/> (the
marketing page is one level up). Locally: `cd web && pnpm dev`, then open
`/app/`. It can open/create/restore an encrypted `.invest` file, import IBKR
and Revolut statements, prices, overview, general ledger, goals with plan vs.
actual, projections with what-if moves and retirement, planning and recording
trades, bank accounts, debts (mortgages, loans, credit cards: they come off
net worth, with the payoff date and a warning when a loan costs more than
investing earns), physical gold and silver, private holdings and their
payouts (as employment income or a capital gain), and
capital-gains reporting (physical metals by holding period) with
draft tax packs for Austria, Bulgaria, Canada, France, Germany, Ireland,
Italy, Poland, Portugal, Romania, Spain, the UK and the US. Works on phones. Open
`/app/#example` for a made-up portfolio with ETFs, shares, bonds, gold (an ETC,
a coin and a bar), shares in USD, CHF and DKK, a GBP account,
crypto, a savings account and a private stake. In 12 languages: English,
Bulgarian, German, Greek, Spanish, French, Italian, Dutch, Polish, Portuguese,
Romanian and Ukrainian. The desktop app has an AI assistant, lets your own AI
tool use the open file, and updates itself.

Next: market data.

Tax packs are drafts and have not been reviewed by a tax professional.
