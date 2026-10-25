//! Market prices and exchange rates, kept with the book.
//!
//! Prices come from statements (a broker's position snapshot), from trades
//! (the price paid or received), or are typed in. When several land on the
//! same day the most direct one wins: manual over statement over trade.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ledger::Ledger;
use crate::lots::Lots;
use crate::{Currency, Money};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuoteSource {
    Trade,
    Statement,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quote {
    pub date: NaiveDate,
    pub price: Money,
    pub source: QuoteSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FxQuote {
    pub date: NaiveDate,
    /// Units of the base currency for one unit of this currency.
    pub rate: Decimal,
    pub source: QuoteSource,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceBook {
    /// By symbol, oldest first.
    #[serde(default)]
    pub quotes: BTreeMap<String, Vec<Quote>>,
    /// By currency, oldest first.
    #[serde(default)]
    pub fx: BTreeMap<Currency, Vec<FxQuote>>,
}

/// Inserts keeping date order; on the same date the stronger source wins.
fn insert<T>(list: &mut Vec<T>, item: T, key: impl Fn(&T) -> (NaiveDate, QuoteSource)) {
    let (date, source) = key(&item);
    match list.iter().position(|q| key(q).0 >= date) {
        Some(i) if key(&list[i]).0 == date => {
            if source >= key(&list[i]).1 {
                list[i] = item;
            }
        }
        Some(i) => list.insert(i, item),
        None => list.push(item),
    }
}

impl PriceBook {
    pub fn add_quote(&mut self, symbol: &str, quote: Quote) {
        insert(self.quotes.entry(symbol.to_owned()).or_default(), quote, |q| (q.date, q.source));
    }

    pub fn add_fx(&mut self, currency: Currency, quote: FxQuote) {
        insert(self.fx.entry(currency).or_default(), quote, |q| (q.date, q.source));
    }

    pub fn latest(&self, symbol: &str) -> Option<&Quote> {
        self.quotes.get(symbol)?.last()
    }

    pub fn latest_fx(&self, currency: &Currency) -> Option<&FxQuote> {
        self.fx.get(currency)?.last()
    }

    /// Converts to the base currency with the latest known rate.
    pub fn to_base(&self, money: &Money, base: &Currency) -> Option<Decimal> {
        if &money.currency == base {
            return Some(money.amount);
        }
        self.latest_fx(&money.currency).map(|fx| money.amount * fx.rate)
    }
}

/// An open position at its latest known price.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Valuation {
    pub broker: String,
    pub symbol: String,
    pub units: Decimal,
    /// What was paid, in the base currency.
    pub cost: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Quote>,
    /// Market value in the base currency, when a price and rate are known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gain: Option<Decimal>,
    /// A price exists but its currency has no exchange rate yet.
    pub missing_fx: bool,
}

pub fn valuations(ledger: &Ledger, prices: &PriceBook) -> Vec<Valuation> {
    Lots::from_ledger(ledger)
        .positions()
        .into_iter()
        .filter_map(|(account, (units, cost))| {
            let mut parts = account.strip_prefix("Assets:Securities:")?.splitn(2, ':');
            let broker = parts.next()?.to_owned();
            let symbol = parts.next()?.to_owned();
            let quote = prices.latest(&symbol).cloned();
            let value = quote
                .as_ref()
                .and_then(|q| prices.to_base(&q.price, &ledger.base_currency))
                .map(|price| (price * units).round_dp(2));
            let missing_fx = quote.is_some() && value.is_none();
            Some(Valuation { gain: value.map(|v| v - cost), broker, symbol, units, cost, quote, value, missing_fx })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, d).unwrap()
    }

    fn eur(v: i64) -> Money {
        Money::new(Decimal::from(v), Currency::new("EUR").unwrap())
    }

    #[test]
    fn keeps_order_and_prefers_stronger_sources() {
        let mut book = PriceBook::default();
        book.add_quote("VWCE", Quote { date: day(20), price: eur(130), source: QuoteSource::Statement });
        book.add_quote("VWCE", Quote { date: day(10), price: eur(120), source: QuoteSource::Trade });
        book.add_quote("VWCE", Quote { date: day(20), price: eur(131), source: QuoteSource::Trade });
        assert_eq!(book.latest("VWCE").unwrap().price, eur(130), "a trade does not replace a statement price");
        book.add_quote("VWCE", Quote { date: day(20), price: eur(132), source: QuoteSource::Manual });
        assert_eq!(book.latest("VWCE").unwrap().price, eur(132));
        let dates: Vec<_> = book.quotes["VWCE"].iter().map(|q| q.date).collect();
        assert_eq!(dates, [day(10), day(20)]);
    }
}
