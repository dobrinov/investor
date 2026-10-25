use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::NaiveDate;
use investor_core::import::{import_statement, BrokerConfig, FeeKind};
use investor_core::ledger::Ledger;
use investor_core::prices::PriceBook;
use investor_core::trade::{self, Side, TradeFee, TradeInput};
use investor_core::Currency;
use rust_decimal::Decimal;

fn repo(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)).unwrap()
}

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// The IBKR fixture: 4 VUAA bought at 110 EUR, 3 EUNL as an opening balance, 98.75 EUR and 516.46 USD in cash.
fn books() -> (Ledger, PriceBook) {
    let mut ledger = Ledger::new(Currency::new("EUR").unwrap());
    let mut prices = PriceBook::default();
    let config = BrokerConfig::from_toml(&repo("brokers/ibkr-flex.toml")).unwrap();
    import_statement(&mut ledger, &mut prices, &config, "ibkr-flex.csv", &repo("fixtures/ibkr-flex.csv")).unwrap();
    (ledger, prices)
}

fn trade(side: Side, symbol: &str, units: &str, price: &str, currency: &str) -> TradeInput {
    TradeInput {
        id: format!("{symbol}-{units}"),
        date: NaiveDate::from_ymd_opt(2026, 12, 1).unwrap(),
        broker: "IBKR".into(),
        side,
        symbol: symbol.into(),
        name: None,
        isin: None,
        units: dec(units),
        price: dec(price),
        currency: Currency::new(currency).unwrap(),
        fees: vec![TradeFee { kind: FeeKind::Commission, amount: dec("1.25") }],
        venue: Some("xetr".into()),
        fx_rate: None,
    }
}

#[test]
fn previewing_a_sale_changes_nothing() {
    let (ledger, prices) = books();
    let p = trade::preview(&ledger, &prices, &trade(Side::Sell, "VUAA", "2", "132.275", "EUR")).unwrap();
    assert_eq!((p.cost, p.gain), (Some(dec("220")), Some(dec("44.55"))));
    assert_eq!((p.fees, p.value), (dec("1.25"), dec("264.55")));
    assert_eq!((p.units_before, p.units_after), (dec("4"), dec("2")));
    assert_eq!((p.cash[0].before, p.cash[0].after), (dec("98.75"), dec("362.05")));
    assert_eq!(p.entry.venue.as_ref().unwrap().code, "XETR");
    assert_eq!(ledger.entries.len(), books().0.entries.len());
}

#[test]
fn a_purchase_beyond_the_cash_needs_a_deposit() {
    let (ledger, prices) = books();
    // 98.75 EUR at the broker; 10 × 132.275 + 1.25 = 1324.00.
    let p = trade::preview(&ledger, &prices, &trade(Side::Buy, "VUAA", "10", "132.275", "EUR")).unwrap();
    assert_eq!(p.cash[0].after, dec("98.75") - dec("1324.00"));
    assert_eq!(p.deposit, dec("1225.25"));
    let small = trade::preview(&ledger, &prices, &trade(Side::Buy, "VUAA", "0.5", "132.275", "EUR")).unwrap();
    assert_eq!(small.deposit, dec("0"));
}

#[test]
fn a_foreign_purchase_uses_the_latest_rate() {
    let (ledger, prices) = books();
    let p = trade::preview(&ledger, &prices, &trade(Side::Buy, "AAPL", "1", "200", "USD")).unwrap();
    assert_eq!((p.fx_rate, p.value), (dec("0.93"), dec("186.00")));
    assert_eq!(p.cash[0].after, dec("516.46") - dec("201.25"));
    assert!(p.invested_after > p.invested_before && p.cash_after < p.cash_before);
    assert_eq!(p.deposit, dec("0"));

    let mut no_rate = trade(Side::Buy, "AAPL", "1", "200", "GBP");
    no_rate.fx_rate = None;
    assert!(trade::preview(&ledger, &prices, &no_rate).unwrap_err().to_string().contains("exchange rate"));
}

#[test]
fn selling_more_than_held_is_refused() {
    let (ledger, prices) = books();
    let err = trade::preview(&ledger, &prices, &trade(Side::Sell, "VUAA", "10", "130", "EUR")).unwrap_err();
    assert!(err.to_string().contains("only 4 held"), "{err}");
}

#[test]
fn recording_once_then_undoing() {
    let (mut ledger, mut prices) = books();
    let before = ledger.entries.len();
    // A purchase after the EUNL snapshot leaves its opening balance alone.
    let input = trade(Side::Buy, "EUNL", "1", "105.5", "EUR");
    let entry = trade::record(&mut ledger, &mut prices, &input).unwrap();
    assert_eq!(ledger.entries.len(), before + 1);
    assert_eq!(ledger.units_at("Assets:Securities:IBKR:EUNL", NaiveDate::MAX), dec("4"));
    assert!(trade::record(&mut ledger, &mut prices, &input).is_err());

    let older = ledger.entries[0].id.clone();
    assert!(trade::undo(&mut ledger, &older).is_err());
    trade::undo(&mut ledger, &entry.id).unwrap();
    assert_eq!(ledger.entries.len(), before);
}

#[test]
fn an_amount_buys_units_rounded_down() {
    assert_eq!(trade::units_for(dec("1000"), dec("132.275"), true), dec("7"));
    assert_eq!(trade::units_for(dec("1000"), dec("132.275"), false), dec("7.560007"));
    assert!(trade::units_for(dec("1000"), dec("132.275"), false) * dec("132.275") <= dec("1000"));
    assert_eq!(trade::units_for(dec("1000"), Decimal::ZERO, false), Decimal::ZERO);
}
