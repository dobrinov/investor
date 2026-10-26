//! What to type into a tax return: each section of the country's forms the
//! pack describes, filled from the books for one year. The pack names the
//! columns as the form does, so the figures can be copied across as they are.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::jurisdiction::{JurisdictionPack, Treatment};
use crate::ledger::{self, Ledger};
use crate::lots::Lots;
use crate::{metal, reports};

/// How to file, as a pack describes it.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Filing {
    /// Where the return is filed online.
    #[serde(default)]
    pub portal: Option<String>,
    /// Steps to file, by language code; `en` is the fallback.
    #[serde(default)]
    pub steps: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub sections: Vec<Section>,
}

/// One part of the forms.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub what: What,
    /// As the form puts it, in the form's language.
    pub title: String,
    pub form: String,
    #[serde(default)]
    pub code: Option<String>,
    /// `[label on the form, value]` pairs; the values each `what` offers are
    /// listed on [`What`].
    pub columns: Vec<(String, String)>,
    /// Tax rate where the section works one out, e.g. on interest.
    #[serde(default)]
    pub rate: Option<Decimal>,
    /// Declaring it is allowed but not required.
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub note: Option<String>,
}

/// What a section is filled from, and the values its columns can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum What {
    /// One row: `proceeds`, `cost`, `gains`, `losses`, `net`, `count`.
    TaxableSales,
    /// One row of sales the pack exempts: `proceeds`, `cost`, `gain`, `count`.
    ExemptSales,
    /// One row of crypto sales, which some countries declare apart:
    /// `proceeds`, `cost`, `gains`, `losses`, `net`, `count`.
    CryptoSales,
    /// A row per company: `name`, `symbol`, `country`, `gross`, `withheld`,
    /// `credit`, `tax_due`.
    Dividends,
    /// One row: `amount`, `tax_due`.
    Interest,
    /// A row per position held at 31 December: `name`, `symbol`, `isin`,
    /// `country`, `broker`, `units`, `cost`, `since`.
    Holdings,
    /// A row per payout: `date`, `description`, `income`, `withheld`.
    EmploymentIncome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cell {
    pub text: String,
    /// A sum of money in the pack's currency, rather than text.
    pub amount: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FilledSection {
    #[serde(flatten)]
    pub section: Section,
    pub rows: Vec<Vec<Cell>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Declaration {
    pub year: i32,
    pub portal: Option<String>,
    pub steps: Vec<String>,
    pub sections: Vec<FilledSection>,
    /// Sales whose venue is unknown, which the return needs decided first.
    pub undecided: usize,
}

type Values = BTreeMap<&'static str, Value>;

enum Value {
    Text(String),
    Money(Decimal),
    Count(Decimal),
}

fn money(amount: Decimal) -> Value {
    Value::Money(amount.round_dp(2))
}

/// Fills the pack's sections for `year`, with the steps in `language`.
pub fn declaration(ledger: &Ledger, pack: &JurisdictionPack, year: i32, language: &str) -> Declaration {
    let filing = pack.declaration.clone().unwrap_or_default();
    let sales = reports::realized_gains_by(ledger, year, pack.capital_gains.cost_basis);
    // Coins are not traded on a stock exchange, so their venue never makes them exempt.
    let treated: Vec<_> = sales
        .iter()
        .map(|r| match pack.classify(r) {
            Treatment::VenueUnknown if is_crypto(ledger, &r.symbol) => (r, Treatment::Taxable),
            t => (r, t),
        })
        .collect();
    let undecided = treated.iter().filter(|(_, t)| *t == Treatment::VenueUnknown).count();
    let sections = filing
        .sections
        .iter()
        .map(|section| {
            let rows: Vec<Values> = match section.what {
                What::TaxableSales | What::CryptoSales => {
                    // Crypto goes in its own section only when the pack has one.
                    let apart = filing.sections.iter().any(|s| s.what == What::CryptoSales);
                    let wanted = |symbol: &str| match section.what {
                        What::CryptoSales => is_crypto(ledger, symbol),
                        _ => !(apart && is_crypto(ledger, symbol)),
                    };
                    let taxable: Vec<_> = treated.iter().filter(|(r, t)| *t == Treatment::Taxable && wanted(&r.symbol)).map(|(r, _)| *r).collect();
                    if taxable.is_empty() {
                        vec![]
                    } else {
                        let gains: Decimal = taxable.iter().map(|r| r.gain).filter(|g| g.is_sign_positive()).sum();
                        let losses: Decimal = taxable.iter().map(|r| r.gain).filter(|g| g.is_sign_negative()).map(|g| g.abs()).sum();
                        vec![Values::from([
                            ("proceeds", money(taxable.iter().map(|r| r.proceeds).sum())),
                            ("cost", money(taxable.iter().map(|r| r.cost).sum())),
                            ("gains", money(gains)),
                            ("losses", money(losses)),
                            ("net", money(gains - losses)),
                            ("count", Value::Count(Decimal::from(taxable.len()))),
                        ])]
                    }
                }
                What::ExemptSales => {
                    let exempt: Vec<_> = treated.iter().filter(|(r, t)| *t == Treatment::Exempt && !r.physical && !r.private).map(|(r, _)| *r).collect();
                    if exempt.is_empty() {
                        vec![]
                    } else {
                        vec![Values::from([
                            ("proceeds", money(exempt.iter().map(|r| r.proceeds).sum())),
                            ("cost", money(exempt.iter().map(|r| r.cost).sum())),
                            ("gain", money(exempt.iter().map(|r| r.gain).sum())),
                            ("count", Value::Count(Decimal::from(exempt.len()))),
                        ])]
                    }
                }
                What::Dividends => dividends(ledger, year, pack.dividends.rate),
                What::Interest => {
                    let amount: Decimal = in_year(ledger, year).flat_map(|e| &e.postings).filter(|p| p.account == ledger::INTEREST_INCOME).map(|p| -p.base).sum();
                    if amount.is_zero() {
                        vec![]
                    } else {
                        vec![Values::from([("amount", money(amount)), ("tax_due", money(amount * section.rate.unwrap_or_default()))])]
                    }
                }
                What::Holdings => holdings(ledger, year),
                What::EmploymentIncome => reports::employment_income(ledger, year)
                    .into_iter()
                    .map(|r| {
                        Values::from([
                            ("date", Value::Text(r.date.format("%d.%m.%Y").to_string())),
                            ("description", Value::Text(r.description)),
                            ("income", money(r.income)),
                            ("withheld", money(r.withheld)),
                        ])
                    })
                    .collect(),
            };
            let rows = rows
                .into_iter()
                .map(|values| {
                    section
                        .columns
                        .iter()
                        .map(|(_, name)| match values.get(name.as_str()) {
                            // A zero never shows as -0.00.
                            Some(Value::Money(a)) => Cell { text: format!("{:.2}", if a.is_zero() { Decimal::ZERO } else { *a }), amount: true },
                            Some(Value::Count(n)) => Cell { text: n.normalize().to_string(), amount: false },
                            Some(Value::Text(t)) => Cell { text: t.clone(), amount: false },
                            None => Cell { text: String::new(), amount: false },
                        })
                        .collect()
                })
                .collect();
            FilledSection { section: section.clone(), rows }
        })
        .collect();
    let steps = filing.steps.get(language).or_else(|| filing.steps.get("en")).cloned().unwrap_or_default();
    Declaration { year, portal: filing.portal, steps, sections, undecided }
}

/// Coins such as BTC: no ISIN and a well-known crypto ticker. The books do
/// not record what kind of asset something is, so this is a best guess.
fn is_crypto(ledger: &Ledger, symbol: &str) -> bool {
    const COINS: &[&str] = &[
        "BTC", "ETH", "SOL", "XRP", "ADA", "DOGE", "DOT", "LTC", "AVAX", "MATIC", "POL", "LINK", "TRX", "XLM", "BNB", "SHIB", "USDT", "USDC", "BCH", "ATOM", "UNI", "ALGO", "XTZ", "NEAR", "ETC",
    ];
    isin(ledger, symbol).is_none() && COINS.contains(&symbol.to_ascii_uppercase().as_str())
}

fn in_year(ledger: &Ledger, year: i32) -> impl Iterator<Item = &ledger::JournalEntry> {
    ledger.entries.iter().filter(move |e| e.date.year() == year)
}

/// The issuer's country, from the first two letters of the ISIN.
fn country(ledger: &Ledger, symbol: &str) -> String {
    isin(ledger, symbol).filter(|i| i.len() >= 2).map_or_else(String::new, |i| i[..2].to_owned())
}

fn isin<'a>(ledger: &'a Ledger, symbol: &str) -> Option<&'a str> {
    ledger.instruments.get(symbol).and_then(|i| i.isin.as_deref())
}

fn name(ledger: &Ledger, symbol: &str) -> String {
    ledger.instruments.get(symbol).and_then(|i| i.name.clone()).unwrap_or_else(|| symbol.to_owned())
}

/// Dividends and the tax kept back on them, per company, with the credit for
/// foreign tax capped at the country's own tax.
fn dividends(ledger: &Ledger, year: i32, rate: Decimal) -> Vec<Values> {
    let mut gross: BTreeMap<String, Decimal> = BTreeMap::new();
    let mut withheld: BTreeMap<String, Decimal> = BTreeMap::new();
    for entry in in_year(ledger, year) {
        for p in &entry.postings {
            if let Some(symbol) = p.account.strip_prefix("Income:Dividends:") {
                *gross.entry(symbol.to_owned()).or_default() -= p.base;
            } else if p.account == ledger::WITHHOLDING_TAX {
                // Imports describe it as "Withholding tax <symbol> <amount>".
                let symbol = entry.description.strip_prefix("Withholding tax ").and_then(|rest| rest.split_whitespace().next()).filter(|s| ledger.instruments.contains_key(*s) || gross.contains_key(*s));
                *withheld.entry(symbol.unwrap_or("").to_owned()).or_default() += p.base;
            }
        }
    }
    gross
        .into_iter()
        .map(|(symbol, gross)| {
            let withheld = withheld.get(&symbol).copied().unwrap_or_default().max(Decimal::ZERO);
            let own = gross * rate;
            let credit = withheld.min(own);
            Values::from([
                ("name", Value::Text(name(ledger, &symbol))),
                ("country", Value::Text(country(ledger, &symbol))),
                ("gross", money(gross)),
                ("withheld", money(withheld)),
                ("credit", money(credit)),
                ("tax_due", money(own - credit)),
                ("symbol", Value::Text(symbol)),
            ])
        })
        .collect()
}

/// Securities held at the end of the year, without coins and bars.
fn holdings(ledger: &Ledger, year: i32) -> Vec<Values> {
    let end = NaiveDate::from_ymd_opt(year, 12, 31).expect("a valid date");
    let mut lots = Lots::default();
    for entry in ledger.entries_by_date().into_iter().filter(|e| e.date <= end) {
        lots.apply(entry);
    }
    lots.positions()
        .into_iter()
        .filter(|(account, _)| !metal::is_physical(account))
        .filter_map(|(account, (units, cost))| {
            let mut parts = account.strip_prefix("Assets:Securities:")?.splitn(2, ':');
            let (broker, symbol) = (parts.next()?.to_owned(), parts.next()?.to_owned());
            if is_crypto(ledger, &symbol) {
                return None;
            }
            let since = lots.first_opened(account).map(|d| d.format("%d.%m.%Y").to_string()).unwrap_or_default();
            Some(Values::from([
                ("name", Value::Text(name(ledger, &symbol))),
                ("isin", Value::Text(isin(ledger, &symbol).unwrap_or_default().to_owned())),
                ("country", Value::Text(country(ledger, &symbol))),
                ("broker", Value::Text(broker)),
                ("units", Value::Count(units)),
                ("cost", money(cost)),
                ("since", Value::Text(since)),
                ("symbol", Value::Text(symbol)),
            ]))
        })
        .collect()
}
