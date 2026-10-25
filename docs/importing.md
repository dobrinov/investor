# Importing statements

Investor reads broker and exchange statements through config files in
[`brokers/`](../brokers). Adding a broker means adding one TOML file — no code.

## Exporting your statements

### Interactive Brokers (Flex Query) → `brokers/ibkr-flex.toml`

1. IBKR Portal → **Performance & Reports → Flex Queries**.
2. **Activity Flex Query → +**. Name it `investor-export`.
3. Sections (click **Select All** fields in each): Account Information, Trades
   (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary),
   Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.
4. Delivery: **CSV**; header and trailer records **No**; column headers **Yes**;
   section code and line descriptor **Yes**; date format `yyyy-MM-dd`; time
   format `HH:mm:ss`; date/time separator `;`.
5. **Run** with a **Custom Date Range** from your first deposit to today
   (one run per year if IBKR limits the range).

### Revolut → `brokers/revolut-trading.toml`

Invest → ⋯ / Documents → Brokerage account → **Account statement** → Excel/CSV,
all dates.

## Writing a broker config

```toml
[broker]
id = "my-broker"            # stable id, used to recognise rows already imported
name = "My Broker"
account = "MyBroker"        # appears in ledger accounts: Assets:Cash:MyBroker:EUR
default_venue = "XETR"      # optional: venue to assume when rows do not say
dust = "0.000001"           # leftover fractions treated as sold

[csv]
layout = "table"            # or "sectioned" (IBKR-style HEADER/DATA rows)
date_format = "%Y-%m-%d"    # chrono format, or "rfc3339"
amounts = "plain"           # or "currency_prefixed" for values like "EUR 250"
fx_rate = "multiply"        # base = amount × rate; "divide" for base = amount ÷ rate

[[record]]                  # first matching rule wins
when = [{ column = "Type", equals = "BUY" }]
kind = "buy"
fields = { date = "Date", symbol = "Ticker", quantity = "Qty", amount = "Total", currency = "Ccy" }
fees = [{ column = "Commission", kind = "commission" }]
```

**Kinds:** `buy`, `sell`, `deposit`, `withdrawal`, `cash_transfer` (sign
decides), `dividend`, `withholding_tax`, `fee`, `interest`, `fx_pair_trade`,
`position_snapshot`, `ignore`.

**Conditions** (`when`, all must hold): `equals`, `one_of`, `starts_with`, `not_empty`.

**Fields:** `date`, `symbol`, `name`, `isin`, `venue`, `quantity`, `amount`,
`currency`, `fx_rate`, `external_id` (a unique row id; without one a hash of
the row is used to detect re-imports).

**Fee kinds:** `commission`, `fx_conversion`, `exchange`, `transaction_tax`,
`custody`, `interest`, `other`. Each is booked to its own expense account.

Rows no rule matches are skipped with a warning naming their line numbers, so
an incomplete config never loses data silently.

### Test it

Add an anonymised sample to `fixtures/` and a test in
`crates/core/tests/import.rs`; `every_broker_config_and_pack_loads` already
checks that every file in `brokers/` parses.
