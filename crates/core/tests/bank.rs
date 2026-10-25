use std::str::FromStr;

use chrono::NaiveDate;
use investor_core::bank::{self, BalanceInput, NewAccount};
use investor_core::ledger::{CONTRIBUTIONS, OPENING_BALANCES};
use investor_core::vault::{Settings, Vault};
use investor_core::{reports, whatif, Currency};
use rust_decimal::Decimal;

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn day(m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, m, d).unwrap()
}

fn vault() -> Vault {
    Vault::new(Settings { base_currency: Currency::new("EUR").unwrap(), language: "en".into(), tax_residency: "BG".into() })
}

fn balance(entry_id: &str, date: NaiveDate, amount: &str) -> BalanceInput {
    BalanceInput { entry_id: entry_id.into(), date, balance: dec(amount), fx_rate: None }
}

fn bank(name: &str, currency: &str, amount: &str) -> NewAccount {
    NewAccount { name: name.into(), currency: Currency::new(currency).unwrap(), balance: balance("open", day(10, 1), amount) }
}

#[test]
fn a_balance_is_cash_that_goals_and_projections_count() {
    let mut v = vault();
    let entry = bank::add(&mut v, &bank("UBB", "EUR", "30000")).unwrap().unwrap();
    assert_eq!(entry.postings[0].account, "Assets:Cash:UBB:EUR");
    assert_eq!(entry.postings[1].account, OPENING_BALANCES);
    assert_eq!(whatif::starting_point(&v.ledger, &v.prices), (30000.0, 0.0));
    assert_eq!(reports::cash(&v.ledger)[0].broker, "UBB");

    let [b] = bank::balances(&v).try_into().unwrap();
    assert_eq!((b.balance, b.value, b.updated), (dec("30000"), Some(dec("30000")), Some(day(10, 1))));
}

#[test]
fn later_balances_book_only_the_difference() {
    let mut v = vault();
    bank::add(&mut v, &bank("UBB", "EUR", "30000")).unwrap();
    let up = bank::set_balance(&mut v, "UBB", &balance("b", day(10, 20), "32500")).unwrap().unwrap();
    assert_eq!((up.postings[0].amount.amount, up.postings[1].account.as_str()), (dec("2500"), CONTRIBUTIONS));
    let down = bank::set_balance(&mut v, "UBB", &balance("c", day(10, 25), "31000")).unwrap().unwrap();
    assert_eq!(down.postings[0].amount.amount, dec("-1500"));
    assert!(bank::set_balance(&mut v, "UBB", &balance("d", day(10, 26), "31000")).unwrap().is_none(), "no change, no entry");
    assert!(bank::set_balance(&mut v, "UBB", &balance("c", day(10, 27), "1")).unwrap_err().to_string().contains("already recorded"));
    assert_eq!(bank::balances(&v)[0].balance, dec("31000"));
}

#[test]
fn a_foreign_account_needs_a_rate_and_names_must_be_free() {
    let mut v = vault();
    let err = bank::add(&mut v, &bank("Savings GBP", "GBP", "1000")).unwrap_err();
    assert!(err.to_string().contains("exchange rate"));
    assert!(v.banks.is_empty(), "a refused account is not kept");

    let mut gbp = bank("Savings GBP", "GBP", "1000");
    gbp.balance.fx_rate = Some(dec("1.15"));
    let entry = bank::add(&mut v, &gbp).unwrap().unwrap();
    assert_eq!(entry.postings[0].base, dec("1150.00"));

    assert!(bank::add(&mut v, &bank("savings gbp", "EUR", "1")).unwrap_err().to_string().contains("already"));
    bank::remove(&mut v, "SAVINGS-GBP").unwrap();
    assert!(v.banks.is_empty() && v.ledger.entries.is_empty());
}
