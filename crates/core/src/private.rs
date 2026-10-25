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
    /// Set once the money has come back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paid_out: Option<PaidOut>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaidOut {
    pub date: NaiveDate,
    /// The entry that booked it.
    pub entry_id: String,
    /// How it was taxed in the end.
    pub taxed_as: Payout,
    /// Before tax, in the payout's currency.
    pub amount: Decimal,
    pub currency: Currency,
    /// Tax kept back before payment, in the payout's currency.
    pub withheld: Decimal,
    /// Gross payout less what was paid in, in the base currency.
    pub gain: Decimal,
}

/// The money coming back, all at once.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutInput {
    /// Chosen when the form opens, so saving twice books once.
    pub entry_id: String,
    pub date: NaiveDate,
    /// Before any tax, in `currency`.
    pub amount: Decimal,
    pub currency: Currency,
    /// Base currency for one unit of `currency`; the latest known rate when missing.
    #[serde(default)]
    pub fx_rate: Option<Decimal>,
    /// Tax kept back before payment, in `currency`.
    #[serde(default)]
    pub withheld: Decimal,
    /// Employment income or a capital gain; not `unknown`.
    pub taxed_as: Payout,
    /// Where the money went, e.g. a bank account.
    #[serde(default)]
    pub paid_into: String,
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
    /// Still waiting for the money; false once paid out.
    pub open: bool,
}


fn fingerprint(entry_id: &str) -> String {
    format!("{MANUAL}:private:{entry_id}")
}

/// Records a new holding and the payment into it. Returns the entry.
pub fn add(vault: &mut Vault, input: &PrivateInput) -> Result<JournalEntry> {
    let bad = |code: &'static str| Err(Error::user(code));
    let mut id = ledger::slug(&input.name);
    if id.is_empty() {
        return bad("name.empty");
    }
    if input.amount <= Decimal::ZERO {
        return bad("private.amountNotPositive");
    }
    if vault.ledger.contains_fingerprint(&fingerprint(&input.entry_id)) {
        return bad("private.recorded");
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
            .ok_or_else(|| Error::user("fx.missing").arg("base", &base).arg("currency", &input.currency))?
    };
    if rate <= Decimal::ZERO {
        return bad("fx.notPositive");
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
        paid_out: None,
    });
    Ok(entry)
}

/// Sets or clears the value set by hand.
pub fn set_estimate(vault: &mut Vault, id: &str, estimate: Option<Estimate>) -> Result<()> {
    if estimate.as_ref().is_some_and(|e| e.amount < Decimal::ZERO) {
        return Err(Error::user("private.valueNegative"));
    }
    let holding = vault.private.iter_mut().find(|h| h.id == id).ok_or_else(|| Error::user("private.unknown").arg("id", id))?;
    holding.estimate = estimate;
    Ok(())
}

fn payout_fingerprint(entry_id: &str) -> String {
    format!("{MANUAL}:private-payout:{entry_id}")
}

/// Books the money coming back: the holding's account closes at what was
/// paid in, the money leaves the books, tax kept back is an expense, and the
/// difference is employment income or a capital gain.
pub fn pay_out(vault: &mut Vault, id: &str, input: &PayoutInput) -> Result<JournalEntry> {
    let bad = |code: &'static str| Err(Error::user(code));
    let holding = vault.private.iter().find(|h| h.id == id).cloned().ok_or_else(|| Error::user("private.unknown").arg("id", id))?;
    if holding.paid_out.is_some() {
        return bad("private.paidOut");
    }
    if input.amount < Decimal::ZERO || input.withheld < Decimal::ZERO || input.withheld > input.amount {
        return bad("private.payoutNegative");
    }
    if input.taxed_as == Payout::Unknown {
        return bad("private.payoutKind");
    }
    if vault.ledger.contains_fingerprint(&payout_fingerprint(&input.entry_id)) {
        return bad("private.payoutRecorded");
    }
    let account = private_account(id);
    let paid_in = vault.ledger.entries.iter().filter(|e| e.postings.iter().any(|p| p.account == account)).map(|e| e.date).min();
    if paid_in.is_some_and(|d| input.date < d) {
        return bad("private.payoutBeforePaid");
    }

    let base = vault.ledger.base_currency.clone();
    let rate = if input.currency == base {
        Decimal::ONE
    } else {
        input
            .fx_rate
            .or_else(|| vault.prices.latest_fx(&input.currency).map(|q| q.rate))
            .ok_or_else(|| Error::user("fx.missing").arg("base", &base).arg("currency", &input.currency))?
    };
    if rate <= Decimal::ZERO {
        return bad("fx.notPositive");
    }
    let balance = vault.ledger.balances().remove(&account).unwrap_or_default();
    let cost = balance.base;
    let gross = (input.amount * rate).round_dp(2);
    let withheld = (input.withheld * rate).round_dp(2);
    let gain = gross - cost;
    let income = match input.taxed_as {
        Payout::EmploymentIncome => ledger::EMPLOYMENT_INCOME,
        _ => ledger::CAPITAL_GAINS,
    };
    let money = |amount: Decimal| Money::new(amount, base.clone());
    let mut postings = vec![
        Posting::new(&account, Money::new(-balance.amounts.get(&holding.currency).copied().unwrap_or_default(), holding.currency.clone()), -cost),
        Posting::new(ledger::CONTRIBUTIONS, money(gross - withheld), gross - withheld),
        Posting::new(income, money(-gain), -gain),
    ];
    if !withheld.is_zero() {
        postings.push(Posting::new(ledger::INCOME_TAX, money(withheld), withheld));
    }
    let received = Money::new(input.amount, input.currency.clone());
    let into = input.paid_into.trim();
    let entry = JournalEntry {
        id: String::new(),
        date: input.date,
        description: if into.is_empty() { format!("{}: paid out {received}", holding.name) } else { format!("{}: paid out {received} into {into}", holding.name) },
        postings,
        venue: None,
        source: Some(Source { broker: MANUAL.into(), file: MANUAL.into(), line: 0, fingerprint: payout_fingerprint(&input.entry_id) }),
        notes: vec![],
        provisional: None,
    };
    let entry = vault.ledger.post(entry)?.clone();
    if input.currency != base && input.fx_rate.is_some() {
        vault.prices.add_fx(input.currency.clone(), FxQuote { date: input.date, rate, source: QuoteSource::Trade });
    }
    let holding = vault.private.iter_mut().find(|h| h.id == id).expect("found above");
    holding.paid_out = Some(PaidOut {
        date: input.date,
        entry_id: entry.id.clone(),
        taxed_as: input.taxed_as,
        amount: input.amount,
        currency: input.currency.clone(),
        withheld: input.withheld,
        gain,
    });
    Ok(entry)
}

/// Takes a payout back out of the books, so the holding is open again.
pub fn undo_payout(vault: &mut Vault, id: &str) -> Result<()> {
    let holding = vault.private.iter_mut().find(|h| h.id == id).ok_or_else(|| Error::user("private.unknown").arg("id", id))?;
    let Some(paid) = holding.paid_out.take() else { return Ok(()) };
    vault.ledger.entries.retain(|e| e.id != paid.entry_id);
    Ok(())
}

/// Removes a holding with every entry that touches its account.
pub fn remove(vault: &mut Vault, id: &str) -> Result<()> {
    if !vault.private.iter().any(|h| h.id == id) {
        return Err(Error::user("private.unknown").arg("id", id));
    }
    let account = private_account(id);
    vault.ledger.entries.retain(|e| !e.postings.iter().any(|p| p.account == account));
    vault.private.retain(|h| h.id != id);
    Ok(())
}

pub fn valuations(vault: &Vault) -> Vec<PrivateValuation> {
    vault
        .private
        .iter()
        .map(|h| {
            let account = private_account(&h.id);
            let since = vault.ledger.entries.iter().filter(|e| e.postings.iter().any(|p| p.account == account)).map(|e| e.date).min();
            let value = h
                .estimate
                .as_ref()
                .filter(|_| h.paid_out.is_none())
                .and_then(|e| vault.prices.to_base(&Money::new(e.amount, h.currency.clone()), &vault.ledger.base_currency))
                .map(|v| v.round_dp(2));
            // Once paid out the account is empty; show what had been paid in.
            let paid_in = |e: &&JournalEntry| h.paid_out.as_ref().is_none_or(|p| p.entry_id != e.id);
            let postings = || vault.ledger.entries.iter().filter(paid_in).flat_map(|e| &e.postings).filter(|p| p.account == account);
            PrivateValuation {
                paid: postings().filter(|p| p.amount.currency == h.currency).map(|p| p.amount.amount).sum(),
                cost: postings().map(|p| p.base).sum(),
                open: h.paid_out.is_none(),
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
    valuations(vault).iter().filter(|v| v.open).map(|v| v.value.unwrap_or(v.cost)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::slug;
    use crate::reports;
    use crate::vault::Settings;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// GBP 50,000 paid in at 1.15 EUR per pound.
    fn stake() -> Vault {
        let mut v = Vault::new(Settings { base_currency: Currency::new("EUR").unwrap(), language: "bg".into(), tax_residency: "BG".into() });
        let input = PrivateInput {
            entry_id: "in".into(),
            name: "IRIS co-invest".into(),
            date: day(2025, 10, 24),
            amount: d("50000"),
            currency: Currency::new("GBP").unwrap(),
            fx_rate: Some(d("1.15")),
            paid_from: "UBB".into(),
            payout: Payout::EmploymentIncome,
            condition: String::new(),
        };
        add(&mut v, &input).unwrap();
        v
    }

    fn payout(taxed_as: Payout) -> PayoutInput {
        PayoutInput {
            entry_id: "out".into(),
            date: day(2028, 3, 1),
            amount: d("80000"),
            currency: Currency::new("GBP").unwrap(),
            fx_rate: Some(d("1.20")),
            withheld: d("3000"),
            taxed_as,
            paid_into: "UBB".into(),
        }
    }

    #[test]
    fn a_payout_as_employment_income_closes_the_stake() {
        let mut v = stake();
        pay_out(&mut v, "IRIS-CO-INVEST", &payout(Payout::EmploymentIncome)).unwrap();
        // Paid in EUR 57,500; GBP 80,000 at 1.20 is EUR 96,000.
        let income = reports::employment_income(&v.ledger, 2028);
        assert_eq!((income[0].income, income[0].withheld), (d("38500"), d("3600")));
        assert!(reports::realized_gains(&v.ledger, 2028).is_empty());
        let val = &valuations(&v)[0];
        assert_eq!((val.open, val.cost, val.paid), (false, d("57500"), d("50000")));
        assert_eq!(total(&v), Decimal::ZERO);
        assert!(v.ledger.balances().get(&private_account("IRIS-CO-INVEST")).is_none_or(|b| b.base.is_zero()));
        assert!(pay_out(&mut v, "IRIS-CO-INVEST", &payout(Payout::EmploymentIncome)).is_err(), "only once");

        undo_payout(&mut v, "IRIS-CO-INVEST").unwrap();
        assert!(valuations(&v)[0].open);
        assert!(reports::employment_income(&v.ledger, 2028).is_empty());
    }

    #[test]
    fn a_payout_as_a_capital_gain_is_a_taxable_sale() {
        let mut v = stake();
        pay_out(&mut v, "IRIS-CO-INVEST", &payout(Payout::CapitalGains)).unwrap();
        let sales = reports::realized_gains(&v.ledger, 2028);
        assert_eq!((sales[0].gain, sales[0].private, sales[0].held_from), (d("38500"), true, Some(day(2025, 10, 24))));
        assert!(pay_out(&mut stake(), "IRIS-CO-INVEST", &payout(Payout::Unknown)).is_err());
    }

    #[test]
    fn ids_come_from_the_name() {
        assert_eq!(slug("IRIS co-invest 2025"), "IRIS-CO-INVEST-2025");
        assert_eq!(slug("  Ферма / Farm (50%) "), "FERMA-FARM-50");
        assert_eq!(slug("ОББ спестовна"), "OBB-SPESTOVNA");
        assert_eq!(slug("Щастие"), "SHTASTIE");
        assert_eq!(slug("!!!"), "");
    }
}
