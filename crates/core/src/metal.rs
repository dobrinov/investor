//! Physical precious metals: coins and bars kept at home or in a safe. Each
//! metal is a holding at the `PHYSICAL` location (`Assets:Securities:PHYSICAL:XAU`)
//! counted in grams of fine metal, so FIFO cost, net worth, projections and
//! realised gains work as for shares. Purchases are paid with money from
//! outside the books and sales take the money out again; the price per gram
//! is typed in now and then, like a bank balance.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ledger::{self, Instrument, JournalEntry, Ledger, Posting, Source};
use crate::lots::Lots;
use crate::prices::{FxQuote, Quote, QuoteSource};
use crate::trade::MANUAL;
use crate::vault::Vault;
use crate::{Currency, Error, Money, Result};

/// The location segment of every physical holding.
pub const PHYSICAL: &str = "PHYSICAL";

/// Grams in one troy ounce.
pub fn grams_per_ounce() -> Decimal {
    Decimal::new(311_034_768, 7)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metal {
    Gold,
    Silver,
    Platinum,
    Palladium,
}

impl Metal {
    pub const ALL: [Metal; 4] = [Metal::Gold, Metal::Silver, Metal::Platinum, Metal::Palladium];

    pub fn symbol(self) -> &'static str {
        match self {
            Metal::Gold => "XAU",
            Metal::Silver => "XAG",
            Metal::Platinum => "XPT",
            Metal::Palladium => "XPD",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Metal::Gold => "Gold (physical)",
            Metal::Silver => "Silver (physical)",
            Metal::Platinum => "Platinum (physical)",
            Metal::Palladium => "Palladium (physical)",
        }
    }

    pub fn of_symbol(symbol: &str) -> Option<Metal> {
        Metal::ALL.into_iter().find(|m| m.symbol() == symbol)
    }
}

pub fn account(metal: Metal) -> String {
    ledger::securities_account(PHYSICAL, metal.symbol())
}

/// Whether an account holds physical metal rather than a security.
pub fn is_physical(account: &str) -> bool {
    account.strip_prefix("Assets:Securities:").is_some_and(|rest| rest.starts_with(&format!("{PHYSICAL}:")))
}

/// A purchase or a sale; `amount` is what was paid or received in all,
/// dealer premium and fees included.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetalInput {
    /// Chosen when the form opens, so saving the same purchase twice adds it once.
    pub entry_id: String,
    pub date: NaiveDate,
    pub metal: Metal,
    /// Fine weight, e.g. 31.1035 for a one-ounce coin.
    pub grams: Decimal,
    pub amount: Decimal,
    pub currency: Currency,
    /// Base currency for one unit of `currency`; the latest known rate when missing.
    #[serde(default)]
    pub fx_rate: Option<Decimal>,
    /// What it is, e.g. "2 × Krugerrand 1 oz".
    #[serde(default)]
    pub description: String,
}

/// A price per gram of fine metal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetalPrice {
    pub date: NaiveDate,
    pub metal: Metal,
    pub per_gram: Decimal,
    pub currency: Currency,
    #[serde(default)]
    pub fx_rate: Option<Decimal>,
}

/// One metal held, at its latest price.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetalHolding {
    pub metal: Metal,
    pub grams: Decimal,
    /// What was paid, in the base currency.
    pub cost: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Quote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gain: Option<Decimal>,
    /// Purchases and sales, newest first.
    pub entries: Vec<String>,
}

fn fingerprint(entry_id: &str) -> String {
    format!("{MANUAL}:metal:{entry_id}")
}

fn bad<T>(code: &'static str) -> Result<T> {
    Err(Error::user(code))
}

fn rate(vault: &Vault, currency: &Currency, given: Option<Decimal>) -> Result<Decimal> {
    let base = &vault.ledger.base_currency;
    if currency == base {
        return Ok(Decimal::ONE);
    }
    let rate = given
        .or_else(|| vault.prices.latest_fx(currency).map(|q| q.rate))
        .ok_or_else(|| Error::user("fx.missing").arg("base", base).arg("currency", currency))?;
    if rate <= Decimal::ZERO {
        return bad("fx.notPositive");
    }
    Ok(rate)
}

impl MetalInput {
    fn check(&self, vault: &Vault) -> Result<Decimal> {
        if self.grams <= Decimal::ZERO {
            return bad("metal.weightNotPositive");
        }
        if self.amount <= Decimal::ZERO {
            return bad("metal.amountNotPositive");
        }
        if vault.ledger.contains_fingerprint(&fingerprint(&self.entry_id)) {
            return bad("metal.recorded");
        }
        rate(vault, &self.currency, self.fx_rate)
    }

    fn entry(&self, description: String, postings: Vec<Posting>) -> JournalEntry {
        JournalEntry {
            id: String::new(),
            date: self.date,
            description,
            postings,
            venue: None,
            source: Some(Source { broker: MANUAL.into(), file: MANUAL.into(), line: 0, fingerprint: fingerprint(&self.entry_id) }),
            notes: vec![],
            provisional: None,
        }
    }

    fn what(&self) -> String {
        let what = self.description.trim();
        let grams = self.grams.normalize();
        if what.is_empty() { format!("{grams} g") } else { format!("{grams} g, {what}") }
    }

    /// Keeps the rate used, and on a first purchase the price paid per
    /// gram as a first guess until one is typed in.
    fn remember(&self, vault: &mut Vault, rate: Decimal, first_price: bool) {
        let symbol = self.metal.symbol();
        if first_price && vault.prices.latest(symbol).is_none() {
            let per_gram = Money::new((self.amount / self.grams).round_dp(4), self.currency.clone());
            vault.prices.add_quote(symbol, Quote { date: self.date, price: per_gram, source: QuoteSource::Trade });
        }
        if self.currency != vault.ledger.base_currency && self.fx_rate.is_some() {
            vault.prices.add_fx(self.currency.clone(), FxQuote { date: self.date, rate, source: QuoteSource::Trade });
        }
    }
}

/// Records a purchase paid with money from outside the books.
pub fn buy(vault: &mut Vault, input: &MetalInput) -> Result<JournalEntry> {
    let rate = input.check(vault)?;
    let base = vault.ledger.base_currency.clone();
    let cost = (input.amount * rate).round_dp(2);
    let entry = input.entry(
        format!("Bought {}: {}", input.metal.name(), input.what()),
        vec![
            Posting::new(account(input.metal), Money::new(cost, base.clone()), cost).with_units(input.grams),
            Posting::new(ledger::CONTRIBUTIONS, Money::new(-cost, base), -cost),
        ],
    );
    let entry = vault.ledger.post(entry)?.clone();
    let symbol = input.metal.symbol();
    vault.ledger.instruments.entry(symbol.into()).or_insert_with(|| Instrument {
        symbol: symbol.into(),
        name: Some(input.metal.name().into()),
        isin: None,
        currency: None,
    });
    input.remember(vault, rate, true);
    Ok(entry)
}

/// Records a sale; the money leaves the books. The cost comes off the
/// oldest purchases first.
pub fn sell(vault: &mut Vault, input: &MetalInput) -> Result<JournalEntry> {
    let rate = input.check(vault)?;
    let account = account(input.metal);
    let mut lots = Lots::default();
    for e in vault.ledger.entries_by_date().into_iter().filter(|e| e.date <= input.date) {
        lots.apply(e);
    }
    let held = lots.held(&account);
    if input.grams > held {
        return Err(Error::user("metal.heldOnly").arg("grams", held.normalize()));
    }
    let disposal = lots.dispose(&account, input.grams)?;
    let base = vault.ledger.base_currency.clone();
    let proceeds = (input.amount * rate).round_dp(2);
    let cost = disposal.cost.round_dp(2);
    let gain = proceeds - cost;
    let entry = input.entry(
        format!("Sold {}: {}", input.metal.name(), input.what()),
        vec![
            Posting::new(&account, Money::new(-cost, base.clone()), -cost).with_units(-input.grams),
            Posting::new(ledger::CONTRIBUTIONS, Money::new(proceeds, base.clone()), proceeds),
            Posting::new(ledger::CAPITAL_GAINS, Money::new(-gain, base), -gain),
        ],
    );
    let entry = vault.ledger.post(entry)?.clone();
    input.remember(vault, rate, false);
    Ok(entry)
}

/// Sets the price per gram on a date.
pub fn set_price(vault: &mut Vault, input: &MetalPrice) -> Result<()> {
    if input.per_gram <= Decimal::ZERO {
        return bad("metal.priceNotPositive");
    }
    let rate = rate(vault, &input.currency, input.fx_rate)?;
    let price = Money::new(input.per_gram, input.currency.clone());
    vault.prices.add_quote(input.metal.symbol(), Quote { date: input.date, price, source: QuoteSource::Manual });
    if input.currency != vault.ledger.base_currency && input.fx_rate.is_some() {
        vault.prices.add_fx(input.currency.clone(), FxQuote { date: input.date, rate, source: QuoteSource::Manual });
    }
    Ok(())
}

/// Removes a purchase or sale, unless that would leave a later sale
/// selling metal that was never held.
pub fn remove(vault: &mut Vault, entry_id: &str) -> Result<()> {
    let ours = |e: &JournalEntry| e.id == entry_id && e.postings.iter().any(|p| is_physical(&p.account));
    if !vault.ledger.entries.iter().any(ours) {
        return bad("metal.noEntry");
    }
    let mut ledger = vault.ledger.clone();
    ledger.entries.retain(|e| !ours(e));
    if !holds_what_it_sells(&ledger) {
        return bad("metal.neededBySale");
    }
    vault.ledger = ledger;
    Ok(())
}

fn holds_what_it_sells(ledger: &Ledger) -> bool {
    let mut lots = Lots::default();
    for e in ledger.entries_by_date() {
        for p in e.postings.iter().filter(|p| is_physical(&p.account)) {
            match p.units {
                Some(u) if u > Decimal::ZERO => lots.open(&p.account, e.date, &e.id, u, p.base),
                Some(u) if u < Decimal::ZERO && lots.dispose(&p.account, -u).is_err() => return false,
                _ => {}
            }
        }
    }
    true
}

pub fn holdings(vault: &Vault) -> Vec<MetalHolding> {
    let ledger = &vault.ledger;
    let positions = crate::prices::valuations(ledger, &vault.prices);
    Metal::ALL
        .into_iter()
        .filter_map(|metal| {
            let account = account(metal);
            let mut entries: Vec<&JournalEntry> = ledger.entries.iter().filter(|e| e.postings.iter().any(|p| p.account == account)).collect();
            if entries.is_empty() {
                return None;
            }
            entries.sort_by(|a, b| (b.date, &b.id).cmp(&(a.date, &a.id)));
            let held = positions.iter().find(|v| v.broker == PHYSICAL && v.symbol == metal.symbol());
            Some(MetalHolding {
                metal,
                grams: held.map_or(Decimal::ZERO, |v| v.units),
                cost: held.map_or(Decimal::ZERO, |v| v.cost),
                price: vault.prices.latest(metal.symbol()).cloned(),
                value: held.and_then(|v| v.value),
                gain: held.and_then(|v| v.gain),
                entries: entries.into_iter().map(|e| e.id.clone()).collect(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports;
    use crate::vault::Settings;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn vault() -> Vault {
        Vault::new(Settings { base_currency: Currency::new("EUR").unwrap(), language: "en".into(), tax_residency: "BG".into() })
    }

    fn input(id: &str, date: NaiveDate, grams: &str, amount: &str) -> MetalInput {
        MetalInput {
            entry_id: id.into(),
            date,
            metal: Metal::Gold,
            grams: d(grams),
            amount: d(amount),
            currency: Currency::new("EUR").unwrap(),
            fx_rate: None,
            description: "coin".into(),
        }
    }

    #[test]
    fn gold_is_bought_valued_and_sold_oldest_first() {
        let mut v = vault();
        buy(&mut v, &input("a", day(2024, 1, 10), "31.1035", "1900")).unwrap();
        buy(&mut v, &input("b", day(2025, 6, 1), "31.1035", "2600")).unwrap();
        set_price(&mut v, &MetalPrice { date: day(2026, 10, 1), metal: Metal::Gold, per_gram: d("110"), currency: Currency::new("EUR").unwrap(), fx_rate: None }).unwrap();
        let h = &holdings(&v)[0];
        assert_eq!((h.grams, h.cost, h.value), (d("62.2070"), d("4500"), Some(d("6842.77"))));

        let sold = sell(&mut v, &input("c", day(2026, 10, 2), "31.1035", "3400")).unwrap();
        assert!(sold.postings.iter().any(|p| p.account == ledger::CAPITAL_GAINS && p.base == d("-1500")));
        let gains = reports::realized_gains(&v.ledger, 2026);
        assert_eq!(gains.len(), 1);
        assert!(gains[0].physical);
        assert_eq!(gains[0].held_from, Some(day(2024, 1, 10)));
        assert!(sell(&mut v, &input("d", day(2026, 10, 3), "40", "4000")).is_err(), "cannot sell more than held");
    }

    #[test]
    fn a_sale_across_purchases_is_split_by_purchase() {
        let mut v = vault();
        buy(&mut v, &input("a", day(2024, 1, 1), "10", "600")).unwrap();
        buy(&mut v, &input("b", day(2026, 1, 1), "20", "2000")).unwrap();
        sell(&mut v, &input("c", day(2026, 6, 1), "15", "1650")).unwrap();
        let parts = reports::realized_gains(&v.ledger, 2026);
        let got: Vec<_> = parts.iter().map(|r| (r.held_from.unwrap(), r.units, r.proceeds, r.cost, r.gain)).collect();
        assert_eq!(got, vec![(day(2024, 1, 1), d("10"), d("1100"), d("600"), d("500")), (day(2026, 1, 1), d("5"), d("550"), d("500"), d("50"))]);
    }

    #[test]
    fn a_purchase_a_sale_needs_cannot_be_removed() {
        let mut v = vault();
        let bought = buy(&mut v, &input("a", day(2025, 1, 1), "10", "800")).unwrap();
        let sold = sell(&mut v, &input("b", day(2025, 2, 1), "10", "850")).unwrap();
        assert!(remove(&mut v, &bought.id).is_err());
        remove(&mut v, &sold.id).unwrap();
        remove(&mut v, &bought.id).unwrap();
        assert!(holdings(&v).is_empty());
    }
}
