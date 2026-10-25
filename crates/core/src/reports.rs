//! Read-only views over the ledger.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::ledger::{self, AccountKind, Ledger, Venue};
use crate::lots::Lots;
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

/// One sale and what it realised, in the base currency.
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
}

pub fn realized_gains(ledger: &Ledger, year: i32) -> Vec<Realization> {
    let mut out = Vec::new();
    for entry in ledger.entries_by_date().into_iter().filter(|e| e.date.year() == year) {
        let Some(gain) = entry.postings.iter().find(|p| p.account == ledger::CAPITAL_GAINS) else { continue };
        let Some(sold) = entry.postings.iter().find(|p| p.units.is_some_and(|u| u.is_sign_negative())) else { continue };
        let gain = -gain.base;
        let cost = -sold.base;
        out.push(Realization {
            date: entry.date,
            entry_id: entry.id.clone(),
            symbol: sold.account.rsplit(':').next().unwrap_or_default().to_owned(),
            units: -sold.units.unwrap_or_default(),
            proceeds: cost + gain,
            cost,
            gain,
            venue: entry.venue.clone(),
        });
    }
    out
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
