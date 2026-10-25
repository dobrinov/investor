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
| `crates/mcp` | `investor-mcp`: read-only MCP server so Claude Code can analyse your file. |
| `crates/wasm` and `web/` | The browser edition. |
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

## Ask Claude Code

`investor-mcp` lets Claude Code read a saved `.invest` file and run the
app's calculations: portfolio, plan and goals, projections with what-if
moves, trade previews, realised gains with tax treatment, and the journal.
It never writes to the file.

```sh
cargo build --release -p investor-mcp
# macOS: keep the password in the Keychain (prompted, never on the command line)
./target/release/investor-mcp --file ~/Documents/portfolio.invest remember-password
claude mcp add investor -- "$PWD/target/release/investor-mcp" --file ~/Documents/portfolio.invest
```

Elsewhere, set `INVESTOR_PASSWORD_COMMAND` to a command that prints the
password (e.g. `pass show investor`). What Claude reads is sent to Anthropic
as part of the conversation.

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
trades, private holdings, and Bulgarian capital-gains reporting. English and Bulgarian. Claude Code can read the file through
`investor-mcp`.

Next: Claude proposing changes for approval, market data, more languages.

Tax packs are drafts and have not been reviewed by a tax professional.
