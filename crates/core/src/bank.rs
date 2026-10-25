//! Bank accounts kept by hand: there is no statement to import, so the user
//! types the balance now and then. Each account is ordinary cash in the books
//! (`Assets:Cash:<id>:<currency>`), so goals, net worth and projections count
//! it like cash at a broker. The first balance is an opening balance; every
//! later one books only the difference, as money added or taken out.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ledger::{self, JournalEntry, Posting, Source};
use crate::prices::{FxQuote, QuoteSource};
use crate::trade::MANUAL;
use crate::vault::Vault;
use crate::{Currency, Error, Money, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BankAccount {
    /// Account segment, made from the name, e.g. `UBB`.
    pub id: String,
    pub name: String,
    pub currency: Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInput {
    /// Chosen when the form opens, so saving the same balance twice books it once.
    pub entry_id: String,
    pub date: NaiveDate,
    /// The whole balance on `date`, in the account's currency.
    pub balance: Decimal,
    /// Base currency for one unit of the account's currency; the latest known rate when missing.
    #[serde(default)]
    pub fx_rate: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccount {
    pub name: String,
    pub currency: Currency,
    #[serde(flatten)]
    pub balance: BalanceInput,
}

/// An account with its balance, for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BankBalance {
    #[serde(flatten)]
    pub account: BankAccount,
    /// In the account's currency.
    pub balance: Decimal,
    /// The balance in the base currency at the latest rate, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Decimal>,
    /// The date of the latest balance.
    pub updated: Option<NaiveDate>,
}

fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_uppercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_end_matches('-').chars().take(32).collect::<String>().trim_end_matches('-').to_owned()
}

fn fingerprint(entry_id: &str) -> String {
    format!("{MANUAL}:bank:{entry_id}")
}

fn account(vault: &Vault, id: &str) -> Result<BankAccount> {
    vault.banks.iter().find(|a| a.id == id).cloned().ok_or_else(|| Error::Trade(format!("no bank account {id}")))
}

/// Adds an account with its balance today. Returns the opening entry, if the balance is not zero.
pub fn add(vault: &mut Vault, input: &NewAccount) -> Result<Option<JournalEntry>> {
    let id = slug(&input.name);
    if id.is_empty() {
        return Err(Error::Trade("give the account a name with at least one letter or digit".into()));
    }
    if vault.banks.iter().any(|a| a.id == id) {
        return Err(Error::Trade(format!("there is already a bank account called {id}")));
    }
    let prefix = format!("Assets:Cash:{id}:");
    if vault.ledger.entries.iter().flat_map(|e| &e.postings).any(|p| p.account.starts_with(&prefix)) {
        return Err(Error::Trade(format!("{id} is already the name of a broker in your books; choose another name")));
    }
    // Book on a copy so a refused balance leaves no account behind.
    let mut copy = vault.clone();
    copy.banks.push(BankAccount { id: id.clone(), name: input.name.trim().to_owned(), currency: input.currency.clone() });
    let entry = set_balance(&mut copy, &id, &input.balance)?;
    *vault = copy;
    Ok(entry)
}

/// Records the balance on a date. Books the difference from the balance the
/// books already hold; returns `None` when there is none.
pub fn set_balance(vault: &mut Vault, id: &str, input: &BalanceInput) -> Result<Option<JournalEntry>> {
    let account = account(vault, id)?;
    if input.balance < Decimal::ZERO {
        return Err(Error::Trade("the balance cannot be negative".into()));
    }
    if vault.ledger.contains_fingerprint(&fingerprint(&input.entry_id)) {
        return Err(Error::Trade("this balance is already recorded".into()));
    }
    let cash = ledger::cash_account(id, &account.currency);
    let held: Decimal = vault.ledger.entries.iter().flat_map(|e| &e.postings).filter(|p| p.account == cash).map(|p| p.amount.amount).sum();
    let opening = !vault.ledger.entries.iter().any(|e| e.postings.iter().any(|p| p.account == cash));
    let diff = input.balance - held;
    if diff.is_zero() {
        return Ok(None);
    }

    let base = vault.ledger.base_currency.clone();
    let rate = if account.currency == base {
        Decimal::ONE
    } else {
        input
            .fx_rate
            .or_else(|| vault.prices.latest_fx(&account.currency).map(|q| q.rate))
            .ok_or_else(|| Error::Trade(format!("enter the exchange rate: how many {base} for one {}", account.currency)))?
    };
    if rate <= Decimal::ZERO {
        return Err(Error::Trade("the exchange rate must be more than zero".into()));
    }
    let value = (diff * rate).round_dp(2);
    let balance = Money::new(input.balance, account.currency.clone());
    let (other, description) = if opening {
        (ledger::OPENING_BALANCES, format!("{}: opening balance {balance}", account.name))
    } else {
        (ledger::CONTRIBUTIONS, format!("{}: balance now {balance}", account.name))
    };
    let entry = JournalEntry {
        id: String::new(),
        date: input.date,
        description,
        postings: vec![
            Posting::new(cash, Money::new(diff, account.currency.clone()), value),
            Posting::new(other, Money::new(-value, base.clone()), -value),
        ],
        venue: None,
        source: Some(Source { broker: MANUAL.into(), file: MANUAL.into(), line: 0, fingerprint: fingerprint(&input.entry_id) }),
        notes: vec![],
        provisional: None,
    };
    let entry = vault.ledger.post(entry)?.clone();
    if account.currency != base && input.fx_rate.is_some() {
        vault.prices.add_fx(account.currency.clone(), FxQuote { date: input.date, rate, source: QuoteSource::Manual });
    }
    Ok(Some(entry))
}

/// Removes an account and every entry that touches it.
pub fn remove(vault: &mut Vault, id: &str) -> Result<()> {
    let account = account(vault, id)?;
    let cash = ledger::cash_account(id, &account.currency);
    vault.ledger.entries.retain(|e| !e.postings.iter().any(|p| p.account == cash));
    vault.banks.retain(|a| a.id != id);
    Ok(())
}

pub fn balances(vault: &Vault) -> Vec<BankBalance> {
    vault
        .banks
        .iter()
        .map(|a| {
            let cash = ledger::cash_account(&a.id, &a.currency);
            let postings = || vault.ledger.entries.iter().filter(|e| e.postings.iter().any(|p| p.account == cash));
            let balance: Decimal = postings().flat_map(|e| &e.postings).filter(|p| p.account == cash).map(|p| p.amount.amount).sum();
            let value = vault.prices.to_base(&Money::new(balance, a.currency.clone()), &vault.ledger.base_currency).map(|v| v.round_dp(2));
            BankBalance { balance, value, updated: postings().map(|e| e.date).max(), account: a.clone() }
        })
        .collect()
}
