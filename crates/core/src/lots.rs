//! First-in, first-out tax lots, rebuilt by replaying the ledger.

use std::collections::{BTreeMap, VecDeque};

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::ledger::Ledger;
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lot {
    pub opened: NaiveDate,
    pub entry_id: String,
    pub units: Decimal,
    /// Cost in the base currency.
    pub cost: Decimal,
}

/// What a sale took out of the open lots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disposal {
    pub units: Decimal,
    pub cost: Decimal,
    pub lots: Vec<Lot>,
}

#[derive(Debug, Clone, Default)]
pub struct Lots {
    open: BTreeMap<String, VecDeque<Lot>>,
}

impl Lots {
    pub fn from_ledger(ledger: &Ledger) -> Self {
        let mut lots = Self::default();
        for entry in ledger.entries_by_date() {
            lots.apply(entry);
        }
        lots
    }

    /// Opens or consumes lots for every posting that moves units.
    pub fn apply(&mut self, entry: &crate::ledger::JournalEntry) {
        for p in &entry.postings {
            match p.units {
                Some(units) if units > Decimal::ZERO => {
                    self.open(&p.account, entry.date, &entry.id, units, p.base)
                }
                // A valid ledger never sells more than it holds.
                Some(units) if units < Decimal::ZERO => {
                    let _ = self.dispose(&p.account, -units);
                }
                _ => {}
            }
        }
    }

    pub fn open(&mut self, account: &str, opened: NaiveDate, entry_id: &str, units: Decimal, cost: Decimal) {
        self.open.entry(account.to_owned()).or_default().push_back(Lot {
            opened,
            entry_id: entry_id.to_owned(),
            units,
            cost,
        });
    }

    pub fn held(&self, account: &str) -> Decimal {
        self.open.get(account).map_or(Decimal::ZERO, |q| q.iter().map(|l| l.units).sum())
    }

    pub fn dispose(&mut self, account: &str, units: Decimal) -> Result<Disposal> {
        let held = self.held(account);
        if units > held || units <= Decimal::ZERO {
            return Err(Error::InsufficientUnits { account: account.to_owned(), requested: units, held });
        }
        let queue = self.open.get_mut(account).expect("held > 0 implies lots exist");
        let mut remaining = units;
        let mut cost = Decimal::ZERO;
        let mut used = Vec::new();
        while remaining > Decimal::ZERO {
            let lot = queue.front_mut().expect("units <= held");
            if lot.units <= remaining {
                remaining -= lot.units;
                cost += lot.cost;
                used.push(queue.pop_front().expect("front exists"));
            } else {
                let part_cost = lot.cost * remaining / lot.units;
                lot.units -= remaining;
                lot.cost -= part_cost;
                used.push(Lot { units: remaining, cost: part_cost, ..lot.clone() });
                cost += part_cost;
                remaining = Decimal::ZERO;
            }
        }
        if queue.is_empty() {
            self.open.remove(account);
        }
        Ok(Disposal { units, cost, lots: used })
    }

    /// Open positions: account → (units, cost in base currency).
    pub fn positions(&self) -> BTreeMap<&str, (Decimal, Decimal)> {
        self.open
            .iter()
            .map(|(account, lots)| {
                let units = lots.iter().map(|l| l.units).sum();
                let cost = lots.iter().map(|l| l.cost).sum();
                (account.as_str(), (units, cost))
            })
            .collect()
    }
}
