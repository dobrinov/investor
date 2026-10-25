use std::str::FromStr;

use chrono::NaiveDate;
use investor_core::private::{self, Estimate, Payout, PrivateInput};
use investor_core::vault::{Settings, Vault};
use investor_core::{prices, whatif, Currency};
use rust_decimal::Decimal;

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn vault() -> Vault {
    Vault::new(Settings { base_currency: Currency::new("EUR").unwrap(), language: "en".into(), tax_residency: "BG".into() })
}

fn co_invest() -> PrivateInput {
    PrivateInput {
        entry_id: "form-1".into(),
        name: "Employer co-invest 2025".into(),
        date: day(2025, 10, 24),
        amount: dec("50000"),
        currency: Currency::new("GBP").unwrap(),
        fx_rate: Some(dec("1.15")),
        paid_from: "Bank".into(),
        payout: Payout::EmploymentIncome,
        condition: "Paid at an exit while employed".into(),
    }
}

#[test]
fn a_payment_from_outside_is_a_contribution_at_cost() {
    let mut v = vault();
    let entry = private::add(&mut v, &co_invest()).unwrap();
    assert_eq!(entry.postings[0].account, "Assets:Private:EMPLOYER-CO-INVEST-2025");
    assert_eq!((entry.postings[0].amount.amount, entry.postings[0].base), (dec("50000"), dec("57500.00")));
    assert_eq!(entry.postings[1].account, "Equity:Contributions");
    assert_eq!(entry.description, "Employer co-invest 2025: paid 50000 GBP from Bank");

    let [h] = private::valuations(&v).try_into().unwrap();
    assert_eq!((h.paid, h.cost, h.value, h.since), (dec("50000"), dec("57500.00"), None, Some(day(2025, 10, 24))));
    assert_eq!(private::total(&v), dec("57500.00"));

    // Not cash and not a security: projections and FIFO lots leave it out.
    assert_eq!(whatif::starting_point(&v.ledger, &v.prices), (0.0, 0.0));
    assert!(prices::valuations(&v.ledger, &v.prices).is_empty());

    assert!(private::add(&mut v, &co_invest()).unwrap_err().to_string().contains("already recorded"));
}

#[test]
fn a_value_set_by_hand_uses_the_latest_rate() {
    let mut v = vault();
    private::add(&mut v, &co_invest()).unwrap();
    let id = v.private[0].id.clone();
    private::set_estimate(&mut v, &id, Some(Estimate { date: day(2026, 10, 1), amount: dec("80000") })).unwrap();
    assert_eq!(private::valuations(&v)[0].value, Some(dec("92000.00")));
    assert!(private::set_estimate(&mut v, &id, Some(Estimate { date: day(2026, 10, 1), amount: dec("-1") })).is_err());
}

#[test]
fn same_names_get_their_own_accounts_and_removal_clears_both() {
    let mut v = vault();
    private::add(&mut v, &co_invest()).unwrap();
    private::add(&mut v, &PrivateInput { entry_id: "form-2".into(), ..co_invest() }).unwrap();
    let ids: Vec<_> = v.private.iter().map(|h| h.id.as_str()).collect();
    assert_eq!(ids, ["EMPLOYER-CO-INVEST-2025", "EMPLOYER-CO-INVEST-2025-2"]);

    private::remove(&mut v, "EMPLOYER-CO-INVEST-2025").unwrap();
    assert_eq!(v.private.len(), 1);
    assert_eq!(v.ledger.entries.len(), 1);
}

#[test]
fn a_foreign_payment_needs_a_rate() {
    let mut v = vault();
    let err = private::add(&mut v, &PrivateInput { fx_rate: None, ..co_invest() }).unwrap_err();
    assert!(err.to_string().contains("exchange rate"));
    assert!(v.ledger.entries.is_empty() && v.private.is_empty());
}
