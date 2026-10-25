//! Private holdings: money put into something without a market price, such
//! as an employer's co-investment or a stake in a private company. Each one
//! is an `Assets:Private:<id>` account, funded with money from outside the
//! books, and valued at what was paid until a value is set by hand. They
//! stay out of projections because they cannot be sold at will.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ledger::{self, JournalEntry, Posting, Source};
use crate::prices::{FxQuote, QuoteSource};
use crate::trade::MANUAL;
use crate::vault::Vault;
use crate::{Currency, Error, Money, Result};

pub const PRIVATE: &str = "Assets:Private";

pub fn private_account(id: &str) -> String {
    format!("{PRIVATE}:{id}")
}

/// How the money that comes back is taxed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Payout {
    /// Paid through payroll, e.g. a bonus that tracks shares.
    EmploymentIncome,
    CapitalGains,
    Unknown,
}

/// A value set by hand, in the holding's currency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Estimate {
    pub date: NaiveDate,
    pub amount: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateHolding {
    /// Account segment, made from the name.
    pub id: String,
    pub name: String,
    pub currency: Currency,
    pub payout: Payout,
    /// When and on what terms the money comes back.
    #[serde(default)]
    pub condition: String,
    /// Where the money came from, e.g. a bank account the books do not track.
    #[serde(default)]
    pub paid_from: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimate: Option<Estimate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateInput {
    /// Chosen when the form opens, so adding the same holding twice adds it once.
    pub entry_id: String,
    pub name: String,
    pub date: NaiveDate,
    /// What was paid, in `currency`.
    pub amount: Decimal,
    pub currency: Currency,
    /// Base currency for one unit of `currency`; the latest known rate when missing.
    #[serde(default)]
    pub fx_rate: Option<Decimal>,
    #[serde(default)]
    pub paid_from: String,
    pub payout: Payout,
    #[serde(default)]
    pub condition: String,
}

/// A holding with what was paid and what it is worth now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrivateValuation {
    #[serde(flatten)]
    pub holding: PrivateHolding,
    /// What was paid, in the holding's currency.
    pub paid: Decimal,
    /// The same in the base currency, at the rates of the payment days.
    pub cost: Decimal,
    /// The estimate in the base currency at the latest rate, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Decimal>,
    pub since: Option<NaiveDate>,
}


fn fingerprint(entry_id: &str) -> String {
    format!("{MANUAL}:private:{entry_id}")
}

/// Records a new holding and the payment into it. Returns the entry.
pub fn add(vault: &mut Vault, input: &PrivateInput) -> Result<JournalEntry> {
    let bad = |m: &str| Err(Error::Trade(m.into()));
    let mut id = ledger::slug(&input.name);
    if id.is_empty() {
        return bad("give the holding a name with at least one letter or digit");
    }
    if input.amount <= Decimal::ZERO {
        return bad("the amount paid must be more than zero");
    }
    if vault.ledger.contains_fingerprint(&fingerprint(&input.entry_id)) {
        return bad("this holding is already recorded");
    }
    let taken = |id: &str| vault.private.iter().any(|h| h.id == id);
    if taken(&id) {
        let first = (2..).map(|n| format!("{id}-{n}")).find(|c| !taken(c)).expect("a free id");
        id = first;
    }

    let base = vault.ledger.base_currency.clone();
    let rate = if input.currency == base {
        Decimal::ONE
    } else {
        input
            .fx_rate
            .or_else(|| vault.prices.latest_fx(&input.currency).map(|q| q.rate))
            .ok_or_else(|| Error::Trade(format!("enter the exchange rate: how many {base} for one {}", input.currency)))?
    };
    if rate <= Decimal::ZERO {
        return bad("the exchange rate must be more than zero");
    }
    let value = (input.amount * rate).round_dp(2);
    let paid = Money::new(input.amount, input.currency.clone());
    let from = input.paid_from.trim();
    let entry = JournalEntry {
        id: String::new(),
        date: input.date,
        description: if from.is_empty() { format!("{}: paid {paid}", input.name.trim()) } else { format!("{}: paid {paid} from {from}", input.name.trim()) },
        postings: vec![
            Posting::new(private_account(&id), paid, value),
            Posting::new(ledger::CONTRIBUTIONS, Money::new(-value, base.clone()), -value),
        ],
        venue: None,
        source: Some(Source { broker: MANUAL.into(), file: MANUAL.into(), line: 0, fingerprint: fingerprint(&input.entry_id) }),
        notes: vec![],
        provisional: None,
    };
    let entry = vault.ledger.post(entry)?.clone();
    if input.currency != base && input.fx_rate.is_some() {
        vault.prices.add_fx(input.currency.clone(), FxQuote { date: input.date, rate, source: QuoteSource::Trade });
    }
    vault.private.push(PrivateHolding {
        id,
        name: input.name.trim().to_owned(),
        currency: input.currency.clone(),
        payout: input.payout,
        condition: input.condition.trim().to_owned(),
        paid_from: from.to_owned(),
        estimate: None,
    });
    Ok(entry)
}

/// Sets or clears the value set by hand.
pub fn set_estimate(vault: &mut Vault, id: &str, estimate: Option<Estimate>) -> Result<()> {
    if estimate.as_ref().is_some_and(|e| e.amount < Decimal::ZERO) {
        return Err(Error::Trade("the value cannot be negative".into()));
    }
    let holding = vault.private.iter_mut().find(|h| h.id == id).ok_or_else(|| Error::Trade(format!("no private holding {id}")))?;
    holding.estimate = estimate;
    Ok(())
}

/// Removes a holding with every entry that touches its account.
pub fn remove(vault: &mut Vault, id: &str) -> Result<()> {
    if !vault.private.iter().any(|h| h.id == id) {
        return Err(Error::Trade(format!("no private holding {id}")));
    }
    let account = private_account(id);
    vault.ledger.entries.retain(|e| !e.postings.iter().any(|p| p.account == account));
    vault.private.retain(|h| h.id != id);
    Ok(())
}

pub fn valuations(vault: &Vault) -> Vec<PrivateValuation> {
    let balances = vault.ledger.balances();
    vault
        .private
        .iter()
        .map(|h| {
            let account = private_account(&h.id);
            let balance = balances.get(&account);
            let since = vault.ledger.entries.iter().filter(|e| e.postings.iter().any(|p| p.account == account)).map(|e| e.date).min();
            let value = h
                .estimate
                .as_ref()
                .and_then(|e| vault.prices.to_base(&Money::new(e.amount, h.currency.clone()), &vault.ledger.base_currency))
                .map(|v| v.round_dp(2));
            PrivateValuation {
                paid: balance.and_then(|b| b.amounts.get(&h.currency).copied()).unwrap_or_default(),
                cost: balance.map(|b| b.base).unwrap_or_default(),
                value,
                since,
                holding: h.clone(),
            }
        })
        .collect()
}

/// All private holdings in the base currency: the estimate where there is
/// one, otherwise what was paid.
pub fn total(vault: &Vault) -> Decimal {
    valuations(vault).iter().map(|v| v.value.unwrap_or(v.cost)).sum()
}

#[cfg(test)]
mod tests {
    use crate::ledger::slug;

    #[test]
    fn ids_come_from_the_name() {
        assert_eq!(slug("IRIS co-invest 2025"), "IRIS-CO-INVEST-2025");
        assert_eq!(slug("  Ферма / Farm (50%) "), "FERMA-FARM-50");
        assert_eq!(slug("ОББ спестовна"), "OBB-SPESTOVNA");
        assert_eq!(slug("Щастие"), "SHTASTIE");
        assert_eq!(slug("!!!"), "");
    }
}
