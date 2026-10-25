use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use investor_core::import::{fit, import_statement, BrokerConfig};
use investor_core::jurisdiction::{JurisdictionPack, Treatment};
use investor_core::ledger::{Ledger, Venue};
use investor_core::prices::{self, PriceBook};
use investor_core::{reports, Currency};
use rust_decimal::Decimal;

fn repo(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn eur_ledger() -> Ledger {
    Ledger::new(Currency::new("EUR").unwrap())
}

fn cash(ledger: &Ledger, currency: &str) -> Decimal {
    reports::cash(ledger)
        .into_iter()
        .find(|c| c.currency.as_str() == currency)
        .map_or(Decimal::ZERO, |c| c.amount)
}

#[test]
fn every_broker_config_and_pack_loads() {
    for entry in fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../brokers")).unwrap() {
        let path = entry.unwrap().path();
        BrokerConfig::from_toml(&fs::read_to_string(&path).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
    JurisdictionPack::from_toml(&repo("jurisdictions/bg/2026.toml")).unwrap();
}

#[test]
fn revolut_statement() {
    let config = BrokerConfig::from_toml(&repo("brokers/revolut-trading.toml")).unwrap();
    let mut ledger = eur_ledger();
    let report = import_statement(&mut ledger, &mut PriceBook::default(), &config, "revolut.csv", &repo("fixtures/revolut-trading.csv")).unwrap();

    assert_eq!(report.added, 6);
    assert!(report.warnings.iter().any(|w| w.contains("not recognised") && w.contains("lines 8")), "{:?}", report.warnings);
    assert!(report.warnings.iter().any(|w| w.contains("treated as sold")), "{:?}", report.warnings);

    // The 0.00000008 VWCE left after the sale is dust, so nothing is held.
    assert!(reports::holdings(&ledger).is_empty());
    assert_eq!(cash(&ledger, "EUR"), dec("551.12"));
    assert_eq!(cash(&ledger, "USD"), dec("108.00"));
    // 108 USD at 1.08 USD per EUR.
    let contributions = ledger.balances()["Equity:Contributions"].base;
    assert_eq!(contributions, dec("-600.00"));

    let gains = reports::realized_gains(&ledger, 2025);
    assert_eq!(gains.len(), 1);
    assert_eq!(gains[0].gain, dec("51.24"));
    assert_eq!(gains[0].venue, None, "Revolut does not report venues");

    // Importing the same file again adds nothing.
    let again = import_statement(&mut ledger, &mut PriceBook::default(), &config, "revolut.csv", &repo("fixtures/revolut-trading.csv")).unwrap();
    assert_eq!((again.added, again.duplicates), (0, 6));
}

#[test]
fn ibkr_flex_statement() {
    let config = BrokerConfig::from_toml(&repo("brokers/ibkr-flex.toml")).unwrap();
    let mut ledger = eur_ledger();
    let mut book = PriceBook::default();
    let report = import_statement(&mut ledger, &mut book, &config, "ibkr.csv", &repo("fixtures/ibkr-flex.csv")).unwrap();

    // The snapshot's market price beats the earlier trade price; EUNL has none.
    let values = prices::valuations(&ledger, &book);
    let vuaa = values.iter().find(|v| v.symbol == "VUAA").unwrap();
    assert_eq!((vuaa.value, vuaa.gain), (Some(dec("529.10")), Some(dec("89.10"))));
    assert_eq!(values.iter().find(|v| v.symbol == "EUNL").unwrap().value, None);
    // AAPL traded in USD, so the statement's USD rate is kept.
    assert_eq!(book.latest_fx(&Currency::new("USD").unwrap()).unwrap().rate, dec("0.93"));

    // deposit, VUAA buy, EUR→USD, AAPL buy, dividend, withholding, AAPL sell, EUNL opening balance
    assert_eq!(report.added, 8, "{:#?}", ledger.entries);
    assert!(report.warnings.iter().any(|w| w.contains("section CORP")), "{:?}", report.warnings);
    assert!(report.warnings.iter().any(|w| w.contains("opening balance")), "{:?}", report.warnings);

    let holdings = reports::holdings(&ledger);
    let units: Vec<_> = holdings.iter().map(|h| (h.symbol.as_str(), h.units, h.cost)).collect();
    assert_eq!(units, [("EUNL", dec("3"), dec("300")), ("VUAA", dec("4"), dec("440"))]);

    assert_eq!(cash(&ledger, "EUR"), dec("98.75"));
    assert_eq!(cash(&ledger, "USD"), dec("516.46"));

    let balances = ledger.balances();
    assert_eq!(balances["Expenses:Fees:Commission"].base, dec("1.25") + dec("0.92") + dec("0.93"));
    assert_eq!(balances["Expenses:Fees:FX"].base, dec("1.84"));
    assert_eq!(balances["Expenses:Taxes:Withholding"].base, dec("0.0744"));

    let gains = reports::realized_gains(&ledger, 2026);
    assert_eq!(gains.len(), 1);
    assert_eq!((gains[0].proceeds, gains[0].cost, gains[0].gain), (dec("446.40"), dec("423.20"), dec("23.20")));
    assert_eq!(gains[0].venue, Some(Venue { code: "NASDAQ".into(), assumed: false }));

    let pack = JurisdictionPack::from_toml(&repo("jurisdictions/bg/2026.toml")).unwrap();
    assert_eq!(pack.treatment(gains[0].venue.as_ref()), Treatment::Taxable);
    assert_eq!(pack.treatment(Some(&Venue { code: "BVME".into(), assumed: false })), Treatment::Exempt);
    assert_eq!(pack.summarize(&gains).tax_due, dec("2.32"));

    // Every entry balances and the books sum to zero.
    let total: Decimal = reports::trial_balance(&ledger).iter().map(|r| r.debit - r.credit).sum();
    assert!(total.is_zero());

    let again = import_statement(&mut ledger, &mut PriceBook::default(), &config, "ibkr.csv", &repo("fixtures/ibkr-flex.csv")).unwrap();
    assert_eq!(again.added, 0);
}

#[test]
fn picks_the_right_config_for_each_statement() {
    let revolut = BrokerConfig::from_toml(&repo("brokers/revolut-trading.toml")).unwrap();
    let ibkr = BrokerConfig::from_toml(&repo("brokers/ibkr-flex.toml")).unwrap();
    let revolut_csv = repo("fixtures/revolut-trading.csv");
    let ibkr_csv = repo("fixtures/ibkr-flex.csv");

    assert_eq!(fit(&revolut, &revolut_csv), 1.0);
    assert_eq!(fit(&ibkr, &revolut_csv), 0.0);
    assert_eq!(fit(&revolut, &ibkr_csv), 0.0);
    assert!(fit(&ibkr, &ibkr_csv) > 0.9, "{}", fit(&ibkr, &ibkr_csv));
}

#[test]
fn history_replaces_a_provisional_opening_balance() {
    let config = BrokerConfig::from_toml(&repo("brokers/ibkr-flex.toml")).unwrap();
    let mut ledger = eur_ledger();
    let snapshot = "HEADER,POST,Symbol,Quantity,CostBasisMoney,CurrencyPrimary,FXRateToBase,ReportDate,LevelOfDetail\n\
                    DATA,POST,EUNL,3,300,EUR,1,2026-10-23,SUMMARY\n";
    let history = "HEADER,TRNT,Buy/Sell,LevelOfDetail,AssetClass,TradeDate,Symbol,Exchange,Quantity,TradeMoney,CurrencyPrimary,FXRateToBase,IBCommission,IBCommissionCurrency,TradeID\n\
                   DATA,TRNT,BUY,EXECUTION,STK,2026-03-01,EUNL,IBIS,2,190,EUR,1,-1,EUR,9001\n";

    let first = import_statement(&mut ledger, &mut PriceBook::default(), &config, "snapshot.csv", snapshot).unwrap();
    assert_eq!(first.added, 1);
    assert_eq!(ledger.balances()["Equity:OpeningBalances"].base, dec("-300"));

    // The purchase of 2 arrives later: only the third unit stays an opening balance.
    let second = import_statement(&mut ledger, &mut PriceBook::default(), &config, "history.csv", history).unwrap();
    assert_eq!(second.added, 1);
    assert!(second.warnings.iter().any(|w| w.contains("replaced")), "{:?}", second.warnings);
    let holdings = reports::holdings(&ledger);
    assert_eq!((holdings[0].units, holdings[0].cost), (dec("3"), dec("290")));
    assert_eq!(ledger.balances()["Equity:OpeningBalances"].base, dec("-100"));

    let again = import_statement(&mut ledger, &mut PriceBook::default(), &config, "snapshot.csv", snapshot).unwrap();
    assert_eq!((again.added, again.duplicates), (0, 1));

    let mut ids: Vec<_> = ledger.entries.iter().map(|e| e.id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), ledger.entries.len(), "entry ids stay unique");
}
