//! Read-only views over the ledger.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::jurisdiction::CostBasis;
use crate::ledger::{self, AccountKind, Ledger, Venue};
use crate::lots::{Lot, Lots};
use crate::metal;
use crate::private::PRIVATE;
use crate::Currency;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Holding {
    pub broker: String,
    pub symbol: String,
    pub units: Decimal,
    /// In the base currency.
    pub cost: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CashBalance {
    pub broker: String,
    pub currency: Currency,
    pub amount: Decimal,
}

/// Open positions with their FIFO cost.
pub fn holdings(ledger: &Ledger) -> Vec<Holding> {
    Lots::from_ledger(ledger)
        .positions()
        .into_iter()
        .filter_map(|(account, (units, cost))| {
            let mut parts = account.strip_prefix("Assets:Securities:")?.splitn(2, ':');
            Some(Holding { broker: parts.next()?.to_owned(), symbol: parts.next()?.to_owned(), units, cost })
        })
        .collect()
}

pub fn cash(ledger: &Ledger) -> Vec<CashBalance> {
    let mut out = Vec::new();
    for (account, balance) in ledger.balances() {
        let Some(rest) = account.strip_prefix("Assets:Cash:") else { continue };
        let broker = rest.rsplit_once(':').map_or(rest, |(b, _)| b).to_owned();
        for (currency, amount) in balance.amounts {
            if !amount.is_zero() {
                out.push(CashBalance { broker: broker.clone(), currency, amount });
            }
        }
    }
    out
}

/// One sale and what it realised, in the base currency. A sale that takes
/// units from several purchases is split by purchase, since some countries
/// tax a gain by how long it was held.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Realization {
    pub date: NaiveDate,
    pub entry_id: String,
    pub symbol: String,
    pub units: Decimal,
    pub proceeds: Decimal,
    pub cost: Decimal,
    pub gain: Decimal,
    pub venue: Option<Venue>,
    /// Coins or bars rather than a security.
    #[serde(default)]
    pub physical: bool,
    /// When the oldest of the units sold was bought.
    #[serde(default)]
    pub held_from: Option<NaiveDate>,
    /// The payout of a private holding, such as a stake in a company.
    #[serde(default)]
    pub private: bool,
    /// A coin such as BTC rather than a security.
    #[serde(default)]
    pub crypto: bool,
}

/// Coins such as BTC: no ISIN and a well-known crypto ticker. The books do
/// not record what kind of asset something is, so this is a best guess.
pub fn is_crypto(ledger: &Ledger, symbol: &str) -> bool {
    const COINS: &[&str] = &[
        "BTC", "ETH", "SOL", "XRP", "ADA", "DOGE", "DOT", "LTC", "AVAX", "MATIC", "POL", "LINK", "TRX", "XLM", "BNB", "SHIB", "USDT", "USDC", "BCH", "ATOM", "UNI", "ALGO", "XTZ", "NEAR", "ETC",
    ];
    ledger.instruments.get(symbol).and_then(|i| i.isin.as_ref()).is_none() && COINS.contains(&symbol.to_ascii_uppercase().as_str())
}

pub fn realized_gains(ledger: &Ledger, year: i32) -> Vec<Realization> {
    realized_gains_by(ledger, year, CostBasis::Fifo)
}

/// Realised gains with the cost worked out the way a country taxes them.
/// With an average cost, each sale is one row whose cost is its share of
/// everything held at the time; the books themselves stay first in, first out.
pub fn realized_gains_by(ledger: &Ledger, year: i32, basis: CostBasis) -> Vec<Realization> {
    let mut out = Vec::new();
    // Units and cost held per account, for an average cost.
    let mut pools: BTreeMap<String, (Decimal, Decimal)> = BTreeMap::new();
    let mut lots = Lots::default();
    let mut first_paid: BTreeMap<&str, NaiveDate> = BTreeMap::new();
    for entry in ledger.entries_by_date() {
        for p in entry.postings.iter().filter(|p| p.account.starts_with(PRIVATE)) {
            first_paid.entry(p.account.as_str()).or_insert(entry.date);
        }
        let mut disposals = Vec::new();
        let mut average_cost = None;
        for p in &entry.postings {
            match p.units {
                Some(u) if u > Decimal::ZERO => {
                    lots.open(&p.account, entry.date, &entry.id, u, p.base);
                    let pool = pools.entry(p.account.clone()).or_default();
                    *pool = (pool.0 + u, pool.1 + p.base);
                }
                Some(u) if u < Decimal::ZERO => {
                    disposals.push(lots.dispose(&p.account, -u).ok());
                    let pool = pools.entry(p.account.clone()).or_default();
                    let cost = if pool.0.is_zero() { Decimal::ZERO } else { (pool.1 * -u / pool.0).round_dp(2) };
                    *pool = (pool.0 + u, pool.1 - cost);
                    average_cost.get_or_insert(cost);
                }
                _ => {}
            }
        }
        if entry.date.year() != year {
            continue;
        }
        let Some(gain) = entry.postings.iter().find(|p| p.account == ledger::CAPITAL_GAINS) else { continue };
        if let Some(stake) = entry.postings.iter().find(|p| p.account.starts_with(PRIVATE) && p.base.is_sign_negative()) {
            let (cost, gain) = (-stake.base, -gain.base);
            out.push(Realization {
                date: entry.date,
                entry_id: entry.id.clone(),
                symbol: stake.account.rsplit(':').next().unwrap_or_default().to_owned(),
                units: Decimal::ONE,
                proceeds: cost + gain,
                cost,
                gain,
                venue: None,
                physical: false,
                held_from: first_paid.get(stake.account.as_str()).copied(),
                private: true,
                crypto: false,
            });
            continue;
        }
        let Some(sold) = entry.postings.iter().find(|p| p.units.is_some_and(|u| u.is_sign_negative())) else { continue };
        let disposal = disposals.into_iter().next().flatten();
        let gain = -gain.base;
        let cost = -sold.base;
        let sale = Realization {
            date: entry.date,
            entry_id: entry.id.clone(),
            symbol: sold.account.rsplit(':').next().unwrap_or_default().to_owned(),
            units: -sold.units.unwrap_or_default(),
            proceeds: cost + gain,
            cost,
            gain,
            venue: entry.venue.clone(),
            physical: metal::is_physical(&sold.account),
            held_from: disposal.as_ref().and_then(|d| d.lots.first()).map(|l| l.opened),
            private: false,
            crypto: false,
        };
        let sale = Realization { crypto: is_crypto(ledger, &sale.symbol), ..sale };
        match (basis, disposal) {
            (CostBasis::Average, _) => {
                let cost = average_cost.unwrap_or(sale.cost);
                out.push(Realization { cost, gain: sale.proceeds - cost, ..sale });
            }
            (CostBasis::Fifo, Some(d)) if d.lots.len() > 1 => out.extend(by_lot(&sale, &d.lots)),
            _ => out.push(sale),
        }
    }
    out
}

/// Splits a sale across the purchases it came from, sharing the proceeds by
/// weight; the last part takes the rounding so the parts add up.
fn by_lot(sale: &Realization, lots: &[Lot]) -> Vec<Realization> {
    let (mut proceeds_left, mut cost_left) = (sale.proceeds, sale.cost);
    lots.iter()
        .enumerate()
        .map(|(i, lot)| {
            let last = i + 1 == lots.len();
            let proceeds = if last { proceeds_left } else { (sale.proceeds * lot.units / sale.units).round_dp(2) };
            let cost = if last { cost_left } else { lot.cost.round_dp(2) };
            (proceeds_left, cost_left) = (proceeds_left - proceeds, cost_left - cost);
            Realization { units: lot.units, proceeds, cost, gain: proceeds - cost, held_from: Some(lot.opened), ..sale.clone() }
        })
        .collect()
}

/// Pay from work booked in the year that is not salary, such as a
/// co-investment paid out through payroll, in the base currency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncomeRow {
    pub date: NaiveDate,
    pub entry_id: String,
    pub description: String,
    pub income: Decimal,
    /// Tax kept back before payment.
    pub withheld: Decimal,
}

pub fn employment_income(ledger: &Ledger, year: i32) -> Vec<IncomeRow> {
    ledger
        .entries_by_date()
        .into_iter()
        .filter(|e| e.date.year() == year)
        .filter_map(|e| {
            let income = e.postings.iter().find(|p| p.account == ledger::EMPLOYMENT_INCOME)?;
            let withheld = e.postings.iter().filter(|p| p.account == ledger::INCOME_TAX).map(|p| p.base).sum();
            Some(IncomeRow { date: e.date, entry_id: e.id.clone(), description: e.description.clone(), income: -income.base, withheld })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrialBalanceRow {
    pub account: String,
    pub debit: Decimal,
    pub credit: Decimal,
}

/// Every account's balance in the base currency, debit or credit side.
pub fn trial_balance(ledger: &Ledger) -> Vec<TrialBalanceRow> {
    let mut rows: Vec<TrialBalanceRow> = ledger
        .balances()
        .into_iter()
        .filter(|(_, b)| !b.base.is_zero())
        .map(|(account, b)| TrialBalanceRow {
            account,
            debit: b.base.max(Decimal::ZERO),
            credit: (-b.base).max(Decimal::ZERO),
        })
        .collect();
    rows.sort_by_key(|r| (AccountKind::of(&r.account), r.account.clone()));
    rows
}

/// Totals per account kind, handy for a quick sanity check.
pub fn totals_by_kind(ledger: &Ledger) -> BTreeMap<AccountKind, Decimal> {
    let mut totals = BTreeMap::new();
    for (account, balance) in ledger.balances() {
        if let Some(kind) = AccountKind::of(&account) {
            *totals.entry(kind).or_default() += balance.base;
        }
    }
    totals
}

/// Money moved in one calendar month, in the base currency.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct MonthFlow {
    /// `YYYY-MM`.
    pub month: String,
    pub deposited: Decimal,
    pub withdrawn: Decimal,
    /// Cost of securities bought.
    pub bought: Decimal,
    /// Cost basis of securities sold.
    pub sold: Decimal,
}

/// What actually happened each month: money put in or taken out, and how
/// much was bought or sold. Opening balances are not counted as deposits.
pub fn monthly_flows(ledger: &Ledger) -> Vec<MonthFlow> {
    let mut months: BTreeMap<String, MonthFlow> = BTreeMap::new();
    for entry in &ledger.entries {
        let key = entry.date.format("%Y-%m").to_string();
        let flow = months.entry(key.clone()).or_insert_with(|| MonthFlow { month: key, ..Default::default() });
        // Paying down a debt is money put in, but not towards investing.
        let debt = entry.postings.iter().any(|p| p.account.starts_with("Liabilities:"));
        for p in &entry.postings {
            if p.account == ledger::CONTRIBUTIONS && !debt {
                // Contributions are credited when money comes in.
                if p.base.is_sign_negative() {
                    flow.deposited -= p.base;
                } else {
                    flow.withdrawn += p.base;
                }
            } else if entry.provisional.is_none() {
                match p.units {
                    Some(u) if u.is_sign_positive() => flow.bought += p.base,
                    Some(u) if u.is_sign_negative() => flow.sold -= p.base,
                    _ => {}
                }
            }
        }
    }
    months
        .into_values()
        .filter(|m| !(m.deposited.is_zero() && m.withdrawn.is_zero() && m.bought.is_zero() && m.sold.is_zero()))
        .collect()
}
