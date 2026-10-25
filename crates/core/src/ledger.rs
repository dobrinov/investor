//! Double-entry general ledger.
//!
//! Accounts are colon-separated paths whose first segment is the account kind,
//! e.g. `Assets:Cash:IBKR:EUR` or `Expenses:Fees:Commission`. Every posting
//! carries its amount in its own currency plus the same value in the book's
//! base currency; an entry balances when the base values sum to zero.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{Currency, Error, Money, Result};

pub const CONTRIBUTIONS: &str = "Equity:Contributions";
pub const OPENING_BALANCES: &str = "Equity:OpeningBalances";
pub const CAPITAL_GAINS: &str = "Income:CapitalGains";
pub const INTEREST_INCOME: &str = "Income:Interest";
pub const INTEREST_EXPENSE: &str = "Expenses:Interest";
pub const WITHHOLDING_TAX: &str = "Expenses:Taxes:Withholding";
/// Pay from work that is not salary, such as a co-investment paid out through payroll.
pub const EMPLOYMENT_INCOME: &str = "Income:Employment";
/// Income tax kept back before money is paid, e.g. through payroll.
pub const INCOME_TAX: &str = "Expenses:Taxes:Income";

/// An account segment made from a name typed by the user: Latin letters and
/// digits in upper case, with Bulgarian (and Russian) Cyrillic transliterated
/// by the official Bulgarian system, so "ОББ спестовна" becomes
/// `OBB-SPESTOVNA`. Anything else turns into a single dash.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().chars() {
        let latin = translit(c);
        if let Some(l) = latin {
            out.push_str(l);
        } else if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_uppercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out: String = out.trim_end_matches('-').chars().take(32).collect();
    out.trim_end_matches('-').to_owned()
}

fn translit(c: char) -> Option<&'static str> {
    let upper = c.to_uppercase().next()?;
    Some(match upper {
        'А' => "A", 'Б' => "B", 'В' => "V", 'Г' => "G", 'Д' => "D", 'Е' => "E", 'Ж' => "ZH", 'З' => "Z",
        'И' => "I", 'Й' => "Y", 'К' => "K", 'Л' => "L", 'М' => "M", 'Н' => "N", 'О' => "O", 'П' => "P",
        'Р' => "R", 'С' => "S", 'Т' => "T", 'У' => "U", 'Ф' => "F", 'Х' => "H", 'Ц' => "TS", 'Ч' => "CH",
        'Ш' => "SH", 'Щ' => "SHT", 'Ъ' => "A", 'Ь' => "Y", 'Ю' => "YU", 'Я' => "YA",
        'Ё' => "YO", 'Ы' => "Y", 'Э' => "E",
        _ => return None,
    })
}

pub fn cash_account(broker: &str, currency: &Currency) -> String {
    format!("Assets:Cash:{broker}:{currency}")
}

pub fn securities_account(broker: &str, symbol: &str) -> String {
    format!("Assets:Securities:{broker}:{symbol}")
}

pub fn dividends_account(symbol: &str) -> String {
    format!("Income:Dividends:{symbol}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AccountKind {
    Assets,
    Liabilities,
    Equity,
    Income,
    Expenses,
}

impl AccountKind {
    pub fn of(account: &str) -> Option<Self> {
        match account.split(':').next()? {
            "Assets" => Some(Self::Assets),
            "Liabilities" => Some(Self::Liabilities),
            "Equity" => Some(Self::Equity),
            "Income" => Some(Self::Income),
            "Expenses" => Some(Self::Expenses),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Posting {
    pub account: String,
    /// Debit positive, credit negative, in the posting's own currency.
    pub amount: Money,
    /// The same value in the book's base currency.
    pub base: Decimal,
    /// Units of an instrument moved in or out of a securities account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<Decimal>,
}

impl Posting {
    pub fn new(account: impl Into<String>, amount: Money, base: Decimal) -> Self {
        Self { account: account.into(), amount, base, units: None }
    }

    pub fn with_units(mut self, units: Decimal) -> Self {
        self.units = Some(units);
        self
    }
}

/// Where a trade was executed. Tax rules can depend on it, e.g. Bulgaria
/// exempts gains from sales on EU regulated markets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Venue {
    pub code: String,
    /// True when the broker's statement did not say and a default was used.
    #[serde(default)]
    pub assumed: bool,
}

/// The statement row an entry was imported from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub broker: String,
    pub file: String,
    pub line: usize,
    /// Stable identity of the row, used to skip it when imported again.
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: String,
    pub date: NaiveDate,
    pub description: String,
    pub postings: Vec<Posting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venue: Option<Venue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Set on opening balances made from a broker's position snapshot. They
    /// are replaced when real history for the position is imported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisional: Option<Provisional>,
}

/// The broker snapshot an opening balance was derived from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provisional {
    pub account: String,
    pub instrument: Instrument,
    /// Units the broker reported as of `as_of`.
    pub units: Decimal,
    /// Their cost in the base currency.
    pub cost: Decimal,
    pub as_of: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<Currency>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Balance {
    pub base: Decimal,
    pub amounts: BTreeMap<Currency, Decimal>,
    pub units: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    pub base_currency: Currency,
    #[serde(default)]
    pub instruments: BTreeMap<String, Instrument>,
    #[serde(default)]
    pub entries: Vec<JournalEntry>,
}

impl Ledger {
    pub fn new(base_currency: Currency) -> Self {
        Self { base_currency, instruments: BTreeMap::new(), entries: Vec::new() }
    }

    /// Validates and appends an entry, assigning the next id when it has none.
    pub fn post(&mut self, mut entry: JournalEntry) -> Result<&JournalEntry> {
        if entry.id.is_empty() {
            let last = self.entries.iter().filter_map(|e| e.id.strip_prefix("JE-")?.parse::<u32>().ok()).max();
            entry.id = format!("JE-{:05}", last.unwrap_or(0) + 1);
        }
        Self::validate(&entry)?;
        self.entries.push(entry);
        Ok(self.entries.last().expect("just pushed"))
    }

    pub fn validate(entry: &JournalEntry) -> Result<()> {
        let invalid = |reason: String| Error::InvalidEntry { id: entry.id.clone(), reason };
        if entry.postings.len() < 2 {
            return Err(invalid("needs at least two postings".into()));
        }
        for p in &entry.postings {
            if AccountKind::of(&p.account).is_none() {
                return Err(invalid(format!("unknown account type in {:?}", p.account)));
            }
        }
        let diff: Decimal = entry.postings.iter().map(|p| p.base).sum();
        if !diff.is_zero() {
            return Err(Error::Unbalanced { id: entry.id.clone(), diff });
        }
        Ok(())
    }

    /// Units in an account from entries dated on or before `date`.
    pub fn units_at(&self, account: &str, date: NaiveDate) -> Decimal {
        self.entries
            .iter()
            .filter(|e| e.date <= date)
            .flat_map(|e| &e.postings)
            .filter(|p| p.account == account)
            .filter_map(|p| p.units)
            .sum()
    }

    pub fn contains_fingerprint(&self, fingerprint: &str) -> bool {
        self.entries
            .iter()
            .any(|e| e.source.as_ref().is_some_and(|s| s.fingerprint == fingerprint))
    }

    /// Entries in date order; entries on the same day keep their posting order.
    pub fn entries_by_date(&self) -> Vec<&JournalEntry> {
        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by_key(|e| e.date);
        entries
    }

    pub fn balances(&self) -> BTreeMap<String, Balance> {
        let mut balances: BTreeMap<String, Balance> = BTreeMap::new();
        for p in self.entries.iter().flat_map(|e| &e.postings) {
            let b = balances.entry(p.account.clone()).or_default();
            b.base += p.base;
            *b.amounts.entry(p.amount.currency.clone()).or_default() += p.amount.amount;
            b.units += p.units.unwrap_or_default();
        }
        balances
    }

    /// Records an instrument, filling in details that were missing before.
    pub fn register_instrument(&mut self, instrument: &Instrument) {
        let known = self
            .instruments
            .entry(instrument.symbol.clone())
            .or_insert_with(|| instrument.clone());
        known.name = known.name.take().or_else(|| instrument.name.clone());
        known.isin = known.isin.take().or_else(|| instrument.isin.clone());
        known.currency = known.currency.take().or_else(|| instrument.currency.clone());
    }
}
