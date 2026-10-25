//! Read-only views over the ledger.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::ledger::{self, AccountKind, Ledger, Venue};
use crate::lots::{Lot, Lots};
use crate::metal;
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
}

pub fn realized_gains(ledger: &Ledger, year: i32) -> Vec<Realization> {
    let mut out = Vec::new();
    let mut lots = Lots::default();
    for entry in ledger.entries_by_date() {
        let mut disposals = Vec::new();
        for p in &entry.postings {
            match p.units {
                Some(u) if u > Decimal::ZERO => lots.open(&p.account, entry.date, &entry.id, u, p.base),
                Some(u) if u < Decimal::ZERO => disposals.push(lots.dispose(&p.account, -u).ok()),
                _ => {}
            }
        }
        if entry.date.year() != year {
            continue;
        }
        let Some(gain) = entry.postings.iter().find(|p| p.account == ledger::CAPITAL_GAINS) else { continue };
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
        };
        match disposal {
            Some(d) if d.lots.len() > 1 => out.extend(by_lot(&sale, &d.lots)),
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
        for p in &entry.postings {
            if p.account == ledger::CONTRIBUTIONS {
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
