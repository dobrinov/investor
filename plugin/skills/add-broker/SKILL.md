---
name: add-broker
description: Write an Investor broker configuration (a TOML file in brokers/) from a sample CSV statement, so the app can import a broker or exchange it does not know yet, and test it. Use when the person's broker is missing, an import leaves rows unrecognised, or they ask to support a new statement format.
---

# Add a broker

Investor reads statements through one configuration file per format in
`brokers/`. This skill writes one from a sample and checks it against the
real import code. It needs the Investor repository checked out.

## Steps

1. Get a sample CSV from the person. Treat it as private: copy it to a new
   folder under `samples/` (git-ignored) and never commit it. If it must
   become a test fixture, anonymise it first (amounts, IDs, names).
2. Read the existing configurations, `brokers/ibkr-flex.toml` and
   `brokers/revolut-trading.toml`, and the guide section "Your broker or exchange is
   missing?" in `web/src/lib/guide/en.ts` for the keys and record kinds.
3. Work out from the sample: the date format, how amounts are written, and
   which rows are buys, sells, deposits, withdrawals, dividends, withholding
   tax, fees, interest, currency conversions and position snapshots. Rows
   that carry no money movement get `kind = "ignore"`.
4. Write `brokers/<id>.toml`. Choose a stable `id` (imports use it to skip
   rows already imported) and a short `account` name.
5. Import the sample into a scratch file with the command-line tool and fix
   the configuration until no rows are reported as unrecognised and the
   trial balance holds:

   ```sh
   export INVESTOR_PASSWORD=scratch-password
   cargo run -q -p investor-cli -- new /tmp/scratch.invest
   cargo run -q -p investor-cli -- import --broker brokers/<id>.toml /tmp/scratch.invest samples/<folder>/<file>.csv
   cargo run -q -p investor-cli -- holdings /tmp/scratch.invest
   cargo run -q -p investor-cli -- trial-balance /tmp/scratch.invest
   ```
6. Add an anonymised fixture and a test in `crates/core/tests/` like the
   tests in `crates/core/tests/import.rs`, then run `cargo test -p investor-core`.

Tell the person what each rule matches, which rows are ignored and why, and
anything in the sample you could not map.
