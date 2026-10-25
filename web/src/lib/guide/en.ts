import type { Section } from './types'

export const brokerExample = `[broker]
id = "my-broker"           # stable id; used to skip rows imported before
name = "My Broker"
account = "MyBroker"       # appears in accounts: Assets:Cash:MyBroker:EUR

[csv]
date_format = "%Y-%m-%d"   # or "rfc3339"
amounts = "plain"          # "currency_prefixed" for values like "EUR 250"

[[record]]                 # first matching rule wins
when = [{ column = "Type", equals = "BUY" }]
kind = "buy"
fields = { date = "Date", symbol = "Ticker", quantity = "Qty", amount = "Total", currency = "Currency" }
fees = [{ column = "Commission", kind = "commission" }]`

export const packExample = `[pack]
id = "xx-tax"              # country code + authority
version = "2026.1"
country = "XX"
authority = "Example Tax Office"
tax_year = 2026
currency = "EUR"
verified = false           # true only after a tax professional checked it

[capital_gains]
rate = "0.00"              # put the real rate here
form = "Annual return, schedule X"
exempt_on_eu_regulated_market = false

[dividends]
rate = "0.00"
form = "Annual return, schedule Y"
foreign_tax_credit = true`

export const CLAUDE_SETUP = `git clone https://github.com/dobrinov/investor
cd investor
cargo build --release -p investor-mcp

# macOS: store the password in your Keychain (asked for, never shown)
./target/release/investor-mcp --file ~/Documents/portfolio.invest remember-password

# connect it to Claude Code
claude mcp add investor -- "\$PWD/target/release/investor-mcp" --file ~/Documents/portfolio.invest`

export const en: Section[] = [
  {
    id: 'start',
    title: 'Getting started',
    blocks: [
      { p: 'Investor keeps your books, plans and tax figures in one encrypted file on your computer. A sensible first session:' },
      {
        steps: [
          'Export your statements from each broker (see the next sections).',
          'Import them on the Import statements screen. Every trade, dividend, fee and tax becomes a balanced journal entry.',
          'Check the Overview. Holdings without a price are counted at what you paid; add a price with “Update price”.',
          'Fill in your profile and add goals on the Goals screen.',
          'Explore Projections, and Tax & reporting before filing your return.',
          'Save your .invest file, and keep a backup copy.',
        ],
      },
    ],
  },
  {
    id: 'file',
    title: 'Your file, your password',
    blocks: [
      { p: 'There is no account and no server. Everything lives in your .invest file, encrypted with a key made from your password. Without the password the file is unreadable — to anyone, including us.' },
      {
        list: [
          'Keep the file in your Documents folder and a copy on a USB drive or in a cloud folder you already use. The copy stays encrypted.',
          'There is no password reset. Write the password down somewhere safe, or use a password manager.',
          'Unsaved changes are kept encrypted in this browser so a refresh does not lose them. Clearing your browser data deletes that copy — save the file to keep your work.',
          'Changing the password (Settings) only affects copies saved afterwards. Older copies still open with the old password, so replace them.',
        ],
      },
    ],
  },
  {
    id: 'claude',
    title: 'Ask Claude Code about your investments',
    blocks: [
      { p: 'If you use Claude Code on your computer, it can read your saved .invest file and run the same calculations as this app: your holdings, goals, projections, what-if moves, trade previews and tax. It only reads. It never changes the file, and you record trades yourself here.' },
      { p: 'What you need: Claude Code, and Rust to build the connector once (rustup.rs). Then run, in a terminal:' },
      { code: CLAUDE_SETUP },
      { p: 'Not on a Mac? Instead of remember-password, set INVESTOR_PASSWORD_COMMAND to a command that prints the password, for example from your password manager.' },
      {
        list: [
          'Claude reads the saved file. Save here first; changes that are only kept in this browser are not seen.',
          'The password never goes into the chat. If Claude says it cannot open the file, it tells you which command to run in your terminal.',
          'What Claude reads is sent to Anthropic to answer you, like anything else in a Claude Code conversation.',
        ],
      },
      { p: 'Things to ask:' },
      {
        list: [
          '“What do I hold, and how much is in cash?”',
          '“Compare investing €10,000 at once with spreading it over 12 months, over 20 years.”',
          '“What would selling 2 VUAA on Xetra change, including tax?”',
          '“Am I on track for my goals? What would it take to retire at 60?”',
        ],
      },
    ],
  },
  {
    id: 'export-ibkr',
    title: 'Exporting from Interactive Brokers',
    blocks: [
      { p: 'Investor reads an Activity Flex Query in CSV. You set it up once and run it whenever you want fresh data.' },
      {
        steps: [
          'In the IBKR Portal open Performance & Reports → Flex Queries.',
          'Next to Activity Flex Query press +, and name it “investor-export”.',
          'Tick these sections and press Select All in each: Account Information, Trades (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary), Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.',
          'Delivery configuration: Format CSV; Include header and trailer records — No; Include column headers — Yes; Include section code and line descriptor — Yes; Date format yyyy-MM-dd; Time format HH:mm:ss; Date/time separator “;”.',
          'Save, then press Run. Choose Custom Date Range from your first deposit to today. If IBKR limits the range to 365 days, run it once per year.',
          'Import the downloaded file here. Running it again later is safe: rows already imported are skipped.',
        ],
      },
      { p: 'If you only export open positions, holdings arrive as opening balances without purchase dates or fees. Importing the full history later replaces them automatically.' },
    ],
  },
  {
    id: 'export-revolut',
    title: 'Exporting from Revolut',
    blocks: [
      {
        steps: [
          'In the Revolut app open Invest (or Stocks).',
          'Open ⋯ or Documents → Brokerage account → Account statement.',
          'Choose Excel/CSV and the full date range, then import the file here.',
        ],
      },
      { p: 'Revolut statements do not say on which exchange a trade happened. In Bulgaria that decides whether a gain is tax-free, so such sales are flagged “check venue” on the Tax screen.' },
    ],
  },
  {
    id: 'add-broker',
    title: 'Your broker or exchange is missing?',
    blocks: [
      { p: 'Each broker is described by a small configuration file, so adding one needs no programming. There are two ways.' },
      { p: 'Ask for it. Open an issue on GitHub with the broker’s name and an anonymised export:' },
      {
        steps: [
          'Export a statement that has at least a deposit, a buy, a sell, a dividend and a fee if you have them.',
          'Replace your account number, name and amounts with made-up values. Keep the column names, the date format and the type labels exactly as exported — they are what the configuration reads.',
          'Open a new issue on GitHub and attach the file.',
        ],
      },
      { link: 'Open an issue on GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
      { p: 'Add it yourself. Write a configuration file like this one, put it in the brokers/ folder of the project, and open a pull request. Rows your configuration does not recognise are listed by line number during import, so you can see what is still missing.' },
      { code: brokerExample },
      { p: 'Kinds of rows: buy, sell, deposit, withdrawal, cash_transfer, dividend, withholding_tax, fee, interest, fx_pair_trade, position_snapshot, ignore. Fee kinds: commission, fx_conversion, exchange, transaction_tax, custody, interest, other.' },
    ],
  },
  {
    id: 'add-tax',
    title: 'Your tax authority is missing?',
    blocks: [
      { p: 'Tax rules come in packs: one file per country and tax year, in the jurisdictions/ folder of the project (for example jurisdictions/bg/2026.toml). A pack sets the rates, the exemptions, the forms and the deadlines.' },
      {
        steps: [
          'Open an issue on GitHub naming the country and tax year, and link the official guidance from the tax authority.',
          'Or copy the Bulgarian pack, change the values, and open a pull request.',
          'Leave verified = false until a tax professional has checked the pack. The app shows a warning for unverified packs.',
        ],
      },
      { code: packExample },
      { p: 'The values above are placeholders. Every rate, exemption and form name must come from the official sources.' },
      { link: 'Open an issue on GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
    ],
  },
  {
    id: 'glossary',
    title: 'Words used in the app',
    blocks: [
      {
        terms: [
          ['Double-entry bookkeeping', 'Every movement of money is recorded twice — where it came from and where it went — so the books always balance.'],
          ['Journal entry', 'One event, such as a purchase or a dividend, recorded as postings that add up to zero.'],
          ['Trial balance', 'Every account’s balance. Debits equal credits when the books are correct.'],
          ['Cost basis', 'What you paid for an investment, including fees where the rules allow.'],
          ['FIFO', 'First in, first out: when you sell, the oldest units you bought are treated as sold first.'],
          ['Realised gain', 'Profit or loss locked in by selling. This is what is taxed.'],
          ['Unrealised gain', 'Profit or loss on investments you still hold, at their latest price.'],
          ['TER', 'Total expense ratio: a fund’s yearly running cost, taken from its price.'],
          ['Withholding tax', 'Tax taken by another country before a dividend reaches you. Often partly credited at home.'],
          ['Trading venue', 'The exchange where a trade happened, such as Xetra or NASDAQ.'],
          ['ETF', 'A fund traded on an exchange like a share, usually holding many companies at once.'],
          ['Emergency fund', 'Money kept in cash for surprises, so you never have to sell investments at a bad moment.'],
        ],
      },
    ],
  },
]
