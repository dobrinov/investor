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

Milestone 1 in progress: core ledger, importers (IBKR Flex, Revolut), `.invest`
file, CLI. Next: WebAssembly bindings and the browser UI.

Tax packs are drafts and have not been reviewed by a tax professional.
