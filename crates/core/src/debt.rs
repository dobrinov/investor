//! Debts kept by hand: a mortgage, a car loan, a credit card. Each is a
//! liability in the books (`Liabilities:Loans:<id>:<currency>`), so it comes
//! off net worth. Like a bank account, the user types what is still owed now
//! and then; the first amount is an opening balance and every later one
//! books only the difference, as money put in to pay it down or borrowed.

use chrono::NaiveDate;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::bank::BalanceInput;
use crate::ledger::{self, JournalEntry, Posting, Source};
use crate::prices::{FxQuote, QuoteSource};
use crate::trade::MANUAL;
use crate::vault::Vault;
use crate::{Currency, Error, Money, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebtKind {
    Mortgage,
    Car,
    Personal,
    Student,
    CreditCard,
    Family,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Debt {
    /// Account segment, made from the name.
    pub id: String,
    pub name: String,
    pub kind: DebtKind,
    pub currency: Currency,
    /// Yearly interest rate, e.g. 0.069 for 6.9%.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<Decimal>,
    /// The regular monthly payment, in the debt's currency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monthly: Option<Decimal>,
}

/// The terms of a debt, without its balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terms {
    pub name: String,
    pub kind: DebtKind,
    #[serde(default)]
    pub rate: Option<Decimal>,
    #[serde(default)]
    pub monthly: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDebt {
    #[serde(flatten)]
    pub terms: Terms,
    pub currency: Currency,
    /// What is owed on `date`, as a positive amount.
    #[serde(flatten)]
    pub owed: BalanceInput,
}

/// A debt with what is owed, for display.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DebtBalance {
    #[serde(flatten)]
    pub debt: Debt,
    /// Still owed, in the debt's currency.
    pub owed: Decimal,
    /// Still owed in the base currency at the latest rate, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Decimal>,
    pub updated: Option<NaiveDate>,
    /// Months until it is paid off at the monthly payment and rate; none when
    /// the payment does not cover the interest or is not known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub months_left: Option<u32>,
}

fn fingerprint(entry_id: &str) -> String {
    format!("{MANUAL}:debt:{entry_id}")
}

fn account_of(debt: &Debt) -> String {
    format!("Liabilities:Loans:{}:{}", debt.id, debt.currency)
}

fn find(vault: &Vault, id: &str) -> Result<Debt> {
    vault.debts.iter().find(|d| d.id == id).cloned().ok_or_else(|| Error::user("debt.unknown").arg("id", id))
}

fn check(terms: &Terms) -> Result<()> {
    if ledger::slug(&terms.name).is_empty() {
        return Err(Error::user("name.empty"));
    }
    if terms.rate.is_some_and(|r| r < Decimal::ZERO || r > Decimal::ONE) {
        return Err(Error::user("debt.rate"));
    }
    if terms.monthly.is_some_and(|m| m < Decimal::ZERO) {
        return Err(Error::user("debt.negative"));
    }
    Ok(())
}

/// Adds a debt with what is owed today. Returns the opening entry, if anything is owed.
pub fn add(vault: &mut Vault, input: &NewDebt) -> Result<Option<JournalEntry>> {
    check(&input.terms)?;
    let id = ledger::slug(&input.terms.name);
    if vault.debts.iter().any(|d| d.id == id) {
        return Err(Error::user("debt.exists").arg("id", id));
    }
    // Book on a copy so a refused amount leaves no debt behind.
    let mut copy = vault.clone();
    copy.debts.push(Debt {
        id: id.clone(),
        name: input.terms.name.trim().to_owned(),
        kind: input.terms.kind,
        currency: input.currency.clone(),
        rate: input.terms.rate,
        monthly: input.terms.monthly,
    });
    let entry = set_owed(&mut copy, &id, &input.owed)?;
    *vault = copy;
    Ok(entry)
}

/// Records what is owed on a date. Books the difference from what the books
/// already hold; returns `None` when there is none.
pub fn set_owed(vault: &mut Vault, id: &str, input: &BalanceInput) -> Result<Option<JournalEntry>> {
    let debt = find(vault, id)?;
    if input.balance < Decimal::ZERO {
        return Err(Error::user("debt.negative"));
    }
    if vault.ledger.contains_fingerprint(&fingerprint(&input.entry_id)) {
        return Err(Error::user("bank.recorded"));
    }
    let account = account_of(&debt);
    let owed = owed_now(vault, &account);
    let opening = !vault.ledger.entries.iter().any(|e| e.postings.iter().any(|p| p.account == account));
    let diff = input.balance - owed;
    if diff.is_zero() {
        return Ok(None);
    }
    let base = vault.ledger.base_currency.clone();
    let rate = if debt.currency == base {
        Decimal::ONE
    } else {
        input
            .fx_rate
            .or_else(|| vault.prices.latest_fx(&debt.currency).map(|q| q.rate))
            .ok_or_else(|| Error::user("fx.missing").arg("base", &base).arg("currency", &debt.currency))?
    };
    if rate <= Decimal::ZERO {
        return Err(Error::user("fx.notPositive"));
    }
    let value = (diff * rate).round_dp(2);
    let owed_money = Money::new(input.balance, debt.currency.clone());
    let (other, description) = if opening {
        (ledger::OPENING_BALANCES, format!("{}: owed {owed_money}", debt.name))
    } else {
        (ledger::CONTRIBUTIONS, format!("{}: owed now {owed_money}", debt.name))
    };
    // More owed is a credit to the liability; paying down is a debit, as
    // money put in from outside the books.
    let entry = JournalEntry {
        id: String::new(),
        date: input.date,
        description,
        postings: vec![
            Posting::new(account, Money::new(-diff, debt.currency.clone()), -value),
            Posting::new(other, Money::new(value, base.clone()), value),
        ],
        venue: None,
        source: Some(Source { broker: MANUAL.into(), file: MANUAL.into(), line: 0, fingerprint: fingerprint(&input.entry_id) }),
        notes: vec![],
        provisional: None,
    };
    let entry = vault.ledger.post(entry)?.clone();
    if debt.currency != base && input.fx_rate.is_some() {
        vault.prices.add_fx(debt.currency.clone(), FxQuote { date: input.date, rate, source: QuoteSource::Manual });
    }
    Ok(Some(entry))
}

fn owed_now(vault: &Vault, account: &str) -> Decimal {
    -vault.ledger.entries.iter().flat_map(|e| &e.postings).filter(|p| p.account == account).map(|p| p.amount.amount).sum::<Decimal>()
}

/// Changes the name, kind, rate or payment. The account in the books keeps
/// the id it got when the debt was added.
pub fn edit(vault: &mut Vault, id: &str, terms: &Terms) -> Result<()> {
    check(terms)?;
    let name = terms.name.trim();
    if vault.debts.iter().any(|d| d.id != id && d.name.eq_ignore_ascii_case(name)) {
        return Err(Error::user("debt.exists").arg("id", name));
    }
    let debt = vault.debts.iter_mut().find(|d| d.id == id).ok_or_else(|| Error::user("debt.unknown").arg("id", id))?;
    debt.name = name.to_owned();
    debt.kind = terms.kind;
    debt.rate = terms.rate;
    debt.monthly = terms.monthly;
    Ok(())
}

/// Removes a debt and every entry that touches it.
pub fn remove(vault: &mut Vault, id: &str) -> Result<()> {
    let debt = find(vault, id)?;
    let account = account_of(&debt);
    vault.ledger.entries.retain(|e| !e.postings.iter().any(|p| p.account == account));
    vault.debts.retain(|d| d.id != id);
    Ok(())
}

/// Months to pay `owed` off at `monthly` with yearly `rate`, compounding monthly.
fn months_left(owed: Decimal, rate: Option<Decimal>, monthly: Option<Decimal>) -> Option<u32> {
    let (owed, monthly) = (owed.to_f64()?, monthly?.to_f64()?);
    if owed <= 0.0 {
        return Some(0);
    }
    if monthly <= 0.0 {
        return None;
    }
    let r = rate.and_then(|r| r.to_f64()).unwrap_or(0.0) / 12.0;
    if r <= 0.0 {
        return Some((owed / monthly).ceil() as u32);
    }
    // n = -ln(1 - r·P/M) / ln(1 + r); no answer when interest outgrows the payment.
    let left = 1.0 - r * owed / monthly;
    if left <= 0.0 {
        return None;
    }
    Some((-left.ln() / (1.0 + r).ln()).ceil() as u32)
}

pub fn balances(vault: &Vault) -> Vec<DebtBalance> {
    vault
        .debts
        .iter()
        .map(|d| {
            let account = account_of(d);
            let entries = || vault.ledger.entries.iter().filter(|e| e.postings.iter().any(|p| p.account == account));
            let owed = owed_now(vault, &account);
            let value = vault.prices.to_base(&Money::new(owed, d.currency.clone()), &vault.ledger.base_currency).map(|v| v.round_dp(2));
            DebtBalance { owed, value, updated: entries().map(|e| e.date).max(), months_left: months_left(owed, d.rate, d.monthly), debt: d.clone() }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::Settings;

    fn d(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    fn owed(id: &str, date: (i32, u32, u32), amount: &str) -> BalanceInput {
        BalanceInput { entry_id: id.into(), date: NaiveDate::from_ymd_opt(date.0, date.1, date.2).unwrap(), balance: d(amount), fx_rate: None }
    }

    #[test]
    fn a_debt_comes_off_net_worth_and_paying_it_down_counts_as_money_put_in() {
        let mut v = Vault::new(Settings { base_currency: Currency::new("EUR").unwrap(), language: "en".into(), tax_residency: "BG".into() });
        let terms = Terms { name: "Car loan".into(), kind: DebtKind::Car, rate: Some(d("0.069")), monthly: Some(d("310")) };
        add(&mut v, &NewDebt { terms, currency: Currency::new("EUR").unwrap(), owed: owed("a", (2026, 1, 1), "8400") }).unwrap();
        let balance = v.ledger.balances();
        assert_eq!(balance["Liabilities:Loans:CAR-LOAN:EUR"].base, d("-8400"));
        assert_eq!(balance[ledger::OPENING_BALANCES].base, d("8400"));

        set_owed(&mut v, "CAR-LOAN", &owed("b", (2026, 2, 1), "8140")).unwrap();
        let balance = v.ledger.balances();
        assert_eq!(balance["Liabilities:Loans:CAR-LOAN:EUR"].base, d("-8140"));
        assert_eq!(balance[ledger::CONTRIBUTIONS].base, d("-260"));

        let shown = &balances(&v)[0];
        assert_eq!((shown.owed, shown.value), (d("8140"), Some(d("8140"))));
        // 8,140 at 6.9% paid 310 a month: about two and a half years.
        assert_eq!(shown.months_left, Some(29));
        assert!(set_owed(&mut v, "CAR-LOAN", &owed("b", (2026, 3, 1), "7900")).is_err(), "the same entry twice");
    }

    #[test]
    fn a_payment_below_the_interest_never_pays_it_off() {
        assert_eq!(months_left(d("10000"), Some(d("0.24")), Some(d("150"))), None);
        assert_eq!(months_left(d("1200"), None, Some(d("100"))), Some(12));
        assert_eq!(months_left(d("0"), Some(d("0.1")), Some(d("100"))), Some(0));
    }
}
