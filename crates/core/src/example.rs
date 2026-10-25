//! The example portfolio: a made-up investor with ETFs, shares, a bond fund,
//! gold on an exchange and as a coin and a bar, crypto, a savings account and a private stake, booked through the
//! same code as real trades. Dates count back from `today`, so prices are
//! always fresh. Every figure is invented.

use std::str::FromStr;

use chrono::{Datelike, Months, NaiveDate};
use rust_decimal::Decimal;

use crate::bank::{self, BalanceInput, NewAccount};
use crate::import::FeeKind;
use crate::ledger::{self, JournalEntry, Posting};
use crate::metal::{self, Metal, MetalInput, MetalPrice};
use crate::plan::{Goal, GoalKind, Plan, Profile, Risk};
use crate::prices::{FxQuote, Quote, QuoteSource};
use crate::private::{self, Estimate, Payout, PrivateInput};
use crate::trade::{self, Side, TradeFee, TradeInput};
use crate::vault::{Settings, Vault};
use crate::{Currency, Money, Result};

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).expect("a literal decimal")
}

fn cur(code: &str) -> Currency {
    Currency::new(code).expect("a literal currency")
}

struct Book {
    vault: Vault,
    today: NaiveDate,
    next: u32,
}

impl Book {
    fn ago(&self, months: u32) -> NaiveDate {
        self.today.checked_sub_months(Months::new(months)).expect("a date in range")
    }

    fn id(&mut self) -> String {
        self.next += 1;
        format!("example-{}", self.next)
    }

    fn post(&mut self, date: NaiveDate, description: &str, postings: Vec<Posting>) -> Result<()> {
        self.vault.ledger.post(JournalEntry {
            id: String::new(),
            date,
            description: description.into(),
            postings,
            venue: None,
            source: None,
            notes: vec![],
            provisional: None,
        })?;
        Ok(())
    }

    fn deposit(&mut self, months: u32, broker: &str, euros: &str) -> Result<()> {
        let amount = d(euros);
        let eur = cur("EUR");
        self.post(self.ago(months), &format!("Deposit {amount} EUR"), vec![
            Posting::new(ledger::cash_account(broker, &eur), Money::new(amount, eur.clone()), amount),
            Posting::new(ledger::CONTRIBUTIONS, Money::new(-amount, eur), -amount),
        ])
    }

    /// Converts euros to dollars at `rate` euros per dollar.
    fn buy_dollars(&mut self, months: u32, broker: &str, dollars: &str, rate: &str) -> Result<()> {
        let (usd, eur) = (cur("USD"), cur("EUR"));
        let amount = d(dollars);
        let euros = (amount * d(rate)).round_dp(2);
        self.post(self.ago(months), &format!("Convert {euros} EUR to {amount} USD"), vec![
            Posting::new(ledger::cash_account(broker, &usd), Money::new(amount, usd), euros),
            Posting::new(ledger::cash_account(broker, &eur), Money::new(-euros, eur), -euros),
        ])
    }

    #[allow(clippy::too_many_arguments)]
    fn trade(&mut self, months: u32, side: Side, broker: &str, symbol: &str, name: &str, units: &str, price: &str, currency: &str, venue: Option<&str>, fx: Option<&str>) -> Result<()> {
        let input = TradeInput {
            id: self.id(),
            date: self.ago(months),
            broker: broker.into(),
            side,
            symbol: symbol.into(),
            name: Some(name.into()),
            isin: None,
            units: d(units),
            price: d(price),
            currency: cur(currency),
            fees: vec![TradeFee { kind: FeeKind::Commission, amount: if broker == "Revolut" { d("0.99") } else { d("1.25") } }],
            venue: venue.map(str::to_owned),
            fx_rate: fx.map(d),
        };
        trade::record(&mut self.vault.ledger, &mut self.vault.prices, &input)?;
        Ok(())
    }

    fn dividend(&mut self, months: u32, broker: &str, symbol: &str, dollars: &str, withheld: &str, rate: &str) -> Result<()> {
        let usd = cur("USD");
        let cash = ledger::cash_account(broker, &usd);
        let (gross, tax) = (d(dollars), d(withheld));
        let (gross_eur, tax_eur) = ((gross * d(rate)).round_dp(2), (tax * d(rate)).round_dp(2));
        let date = self.ago(months);
        self.post(date, &format!("Dividend {symbol} {gross} USD"), vec![
            Posting::new(cash.clone(), Money::new(gross, usd.clone()), gross_eur),
            Posting::new(ledger::dividends_account(symbol), Money::new(-gross, usd.clone()), -gross_eur),
        ])?;
        self.post(date, &format!("Withholding tax {symbol} {tax} USD"), vec![
            Posting::new(ledger::WITHHOLDING_TAX, Money::new(tax, usd.clone()), tax_eur),
            Posting::new(cash, Money::new(-tax, usd), -tax_eur),
        ])
    }

    fn price(&mut self, symbol: &str, amount: &str, currency: &str) {
        let date = self.today - chrono::Days::new(1);
        self.vault.prices.add_quote(symbol, Quote { date, price: Money::new(d(amount), cur(currency)), source: QuoteSource::Statement });
    }
}

/// A complete example book as of `today`, in `language`.
pub fn vault(today: NaiveDate, language: &str) -> Result<Vault> {
    let settings = Settings { base_currency: cur("EUR"), language: language.into(), tax_residency: "BG".into() };
    let mut b = Book { vault: Vault::new(settings), today, next: 0 };
    use Side::{Buy, Sell};

    // Interactive Brokers: index funds, shares, bonds and gold.
    b.deposit(30, "IBKR", "12000")?;
    b.buy_dollars(29, "IBKR", "5500", "0.92")?;
    b.trade(29, Buy, "IBKR", "VWCE", "Vanguard FTSE All-World UCITS ETF (Acc)", "40", "98.00", "EUR", Some("XETR"), None)?;
    b.trade(29, Buy, "IBKR", "AAPL", "Apple Inc.", "15", "175.00", "USD", Some("NASDAQ"), Some("0.92"))?;
    b.trade(28, Buy, "IBKR", "EUNL", "iShares Core MSCI World UCITS ETF", "30", "80.00", "EUR", Some("XETR"), None)?;
    b.trade(27, Buy, "IBKR", "MSFT", "Microsoft Corp.", "6", "410.00", "USD", Some("NASDAQ"), Some("0.93"))?;
    b.trade(24, Buy, "IBKR", "4GLD", "Xetra-Gold (physical gold)", "25", "68.00", "EUR", Some("XETR"), None)?;
    b.deposit(18, "IBKR", "6000")?;
    b.trade(17, Buy, "IBKR", "VWCE", "Vanguard FTSE All-World UCITS ETF (Acc)", "15", "112.00", "EUR", Some("XETR"), None)?;
    b.trade(16, Buy, "IBKR", "AGGH", "iShares Core Global Aggregate Bond UCITS ETF (EUR hedged)", "300", "4.85", "EUR", Some("XETR"), None)?;
    b.trade(14, Buy, "IBKR", "ASML", "ASML Holding N.V.", "3", "640.00", "EUR", Some("XAMS"), None)?;
    b.dividend(8, "IBKR", "AAPL", "3.75", "0.56", "0.90")?;
    b.deposit(6, "IBKR", "4000")?;
    // A taxable sale in the US and a sale on an EU regulated market.
    b.trade(4, Sell, "IBKR", "AAPL", "Apple Inc.", "5", "240.00", "USD", Some("NASDAQ"), Some("0.87"))?;
    b.trade(3, Sell, "IBKR", "VWCE", "Vanguard FTSE All-World UCITS ETF (Acc)", "10", "128.00", "EUR", Some("XETR"), None)?;

    // Revolut: a little crypto.
    b.deposit(20, "Revolut", "4000")?;
    b.trade(20, Buy, "Revolut", "BTC", "Bitcoin", "0.05", "38000", "EUR", None, None)?;
    b.trade(19, Buy, "Revolut", "ETH", "Ether", "0.8", "2300", "EUR", None, None)?;

    // A savings account and a private stake.
    let savings = NewAccount {
        name: "Savings account".into(),
        currency: cur("EUR"),
        balance: BalanceInput { entry_id: "example-bank".into(), date: b.ago(1), balance: d("14000"), fx_rate: None },
    };
    bank::add(&mut b.vault, &savings)?;
    let stake = PrivateInput {
        entry_id: "example-private".into(),
        name: "Startup stake (angel round)".into(),
        date: b.ago(15),
        amount: d("5000"),
        currency: cur("EUR"),
        fx_rate: None,
        paid_from: "Bank".into(),
        payout: Payout::CapitalGains,
        condition: "When the company is sold".into(),
    };
    private::add(&mut b.vault, &stake)?;
    let id = b.vault.private[0].id.clone();
    let estimate = Estimate { date: b.ago(2), amount: d("8000") };
    private::set_estimate(&mut b.vault, &id, Some(estimate))?;

    // A gold coin and a small bar, kept at home.
    for (id, months, grams, paid, what) in [("example-coin", 18, "31.1035", "2410", "Krugerrand 1 oz"), ("example-bar", 7, "20", "2140", "Bar")] {
        let input = MetalInput { entry_id: id.into(), date: b.ago(months), metal: Metal::Gold, grams: d(grams), amount: d(paid), currency: cur("EUR"), fx_rate: None, description: what.into() };
        metal::buy(&mut b.vault, &input)?;
    }
    let price = MetalPrice { date: today - chrono::Days::new(1), metal: Metal::Gold, per_gram: d("118.20"), currency: cur("EUR"), fx_rate: None };
    metal::set_price(&mut b.vault, &price)?;

    // Today's prices.
    for (symbol, amount, currency) in [
        ("VWCE", "131.40", "EUR"),
        ("EUNL", "104.60", "EUR"),
        ("AGGH", "4.95", "EUR"),
        ("4GLD", "118.20", "EUR"),
        ("ASML", "718.00", "EUR"),
        ("AAPL", "246.00", "USD"),
        ("MSFT", "512.00", "USD"),
        ("BTC", "88400", "EUR"),
        ("ETH", "3120", "EUR"),
    ] {
        b.price(symbol, amount, currency);
    }
    b.vault.prices.add_fx(cur("USD"), FxQuote { date: today - chrono::Days::new(1), rate: d("0.86"), source: QuoteSource::Statement });

    b.vault.plan = Plan {
        profile: Some(Profile {
            birth_year: Some(today.year() - 36),
            monthly_spending: Some(d("2500")),
            monthly_investing: Some(d("800")),
            risk: Risk::Balanced,
            retirement_age: Some(60),
            pension_age: Some(65),
            monthly_pension: Some(d("700")),
            retirement_spending: Some(d("2200")),
        }),
        goals: vec![
            Goal { id: "example-emergency".into(), name: "Emergency fund".into(), kind: GoalKind::EmergencyFund, target: d("6"), year: None, monthly: Decimal::ZERO },
            Goal { id: "example-house".into(), name: "House deposit".into(), kind: GoalKind::Savings, target: d("40000"), year: Some(today.year() + 4), monthly: d("500") },
        ],
        what_if: vec![],
    };
    Ok(b.vault)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{prices, reports};

    #[test]
    fn the_example_holds_a_bit_of_everything_and_balances() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        let v = vault(today, "en").unwrap();
        let symbols: Vec<String> = prices::valuations(&v.ledger, &v.prices).into_iter().map(|x| x.symbol).collect();
        for s in ["VWCE", "EUNL", "AGGH", "4GLD", "ASML", "AAPL", "MSFT", "BTC", "ETH"] {
            assert!(symbols.contains(&s.to_string()), "{s} missing");
        }
        assert!(reports::cash(&v.ledger).iter().all(|c| c.amount >= Decimal::ZERO), "no cash goes negative");
        assert_eq!(reports::realized_gains(&v.ledger, 2026).len(), 2);
        assert_eq!((v.banks.len(), v.private.len(), v.plan.goals.len()), (1, 1, 2));
        assert_eq!(metal::holdings(&v)[0].value, Some(d("6040.43")));
    }
}
