//! A purchase or sale entered by hand: previewed on a copy of the books
//! first, then recorded through the same path as imported statements, so
//! FIFO lots, fees and exchange rates are handled the same way.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::import::{BrokerConfig, Event, EventKind, FeeCharge, FeeKind, Fx, FxRateMode, post_events};
use crate::ledger::{self, Instrument, JournalEntry, Ledger, Source, Venue};
use crate::prices::{self, PriceBook};
use crate::jurisdiction::{JurisdictionPack, Treatment};
use crate::plan::{Plan, to_f64};
use crate::reports;
use crate::whatif::{self, Band, WhatIfInput};
use crate::{Currency, Error, Money, Result};

/// Recorded trades carry this as their source broker.
pub const MANUAL: &str = "manual";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeFee {
    pub kind: FeeKind,
    /// In the trade's currency.
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInput {
    /// Chosen when the form opens, so recording the same trade twice adds it once.
    pub id: String,
    pub date: NaiveDate,
    /// Account segment, e.g. `IBKR` in `Assets:Cash:IBKR:EUR`.
    pub broker: String,
    pub side: Side,
    pub symbol: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub isin: Option<String>,
    pub units: Decimal,
    /// Per unit, in `currency`.
    pub price: Decimal,
    pub currency: Currency,
    #[serde(default)]
    pub fees: Vec<TradeFee>,
    /// Where the trade happens, e.g. `XETR`.
    #[serde(default)]
    pub venue: Option<String>,
    /// Base currency for one unit of `currency`; the latest known rate when missing.
    #[serde(default)]
    pub fx_rate: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CashChange {
    pub currency: Currency,
    pub before: Decimal,
    pub after: Decimal,
}

/// Value of one instrument across all brokers, in the base currency.
#[derive(Debug, Clone, Serialize)]
pub struct Weight {
    pub symbol: String,
    pub before: Decimal,
    pub after: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradePreview {
    /// The entry as it would be recorded.
    pub entry: JournalEntry,
    /// Trade amount in the base currency, before fees.
    pub value: Decimal,
    /// All fees in the base currency.
    pub fees: Decimal,
    /// The rate used, base currency for one unit of the trade currency.
    pub fx_rate: Decimal,
    /// FIFO cost of the units sold.
    pub cost: Option<Decimal>,
    /// Realised gain (negative for a loss) of a sale.
    pub gain: Option<Decimal>,
    pub units_before: Decimal,
    pub units_after: Decimal,
    /// Cash at this broker, per currency the trade touches.
    pub cash: Vec<CashChange>,
    pub holdings: Vec<Weight>,
    pub invested_before: Decimal,
    pub invested_after: Decimal,
    pub cash_before: Decimal,
    pub cash_after: Decimal,
    /// Money to deposit at the broker for this trade, in the base currency:
    /// what it spends beyond the cash already there. A plan does not need the
    /// money today, so the projection counts it as deposited.
    pub deposit: Decimal,
}

impl TradeInput {
    fn check(&self) -> Result<()> {
        let bad = |code: &'static str| Err(Error::user(code));
        let valid = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c));
        if !valid(&self.broker) {
            return bad("trade.brokerChars");
        }
        if !valid(&self.symbol) {
            return bad("trade.symbolChars");
        }
        if self.units <= Decimal::ZERO {
            return bad("trade.unitsNotPositive");
        }
        if self.price < Decimal::ZERO || self.fees.iter().any(|f| f.amount < Decimal::ZERO) {
            return bad("trade.negative");
        }
        Ok(())
    }

    /// Only the account name and dust matter when posting; no CSV rules are needed.
    fn config(&self) -> Result<BrokerConfig> {
        let text = format!("[broker]\nid = \"{MANUAL}\"\nname = \"Entered by hand\"\naccount = \"{}\"\n\n[csv]\n", self.broker);
        Ok(toml::from_str(&text)?)
    }

    fn event(&self, base: &Currency, prices: &PriceBook) -> Result<(Event, Decimal)> {
        let rate = if &self.currency == base {
            Decimal::ONE
        } else {
            self.fx_rate
                .or_else(|| prices.fx.get(&self.currency).and_then(|q| q.last()).map(|q| q.rate))
                .ok_or_else(|| Error::user("fx.missing").arg("base", base).arg("currency", &self.currency))?
        };
        let instrument = Instrument {
            symbol: self.symbol.clone(),
            name: self.name.clone().filter(|n| !n.trim().is_empty()),
            isin: self.isin.clone().filter(|n| !n.trim().is_empty()),
            currency: Some(self.currency.clone()),
        };
        let amount = Money::new((self.units * self.price).round_dp(2), self.currency.clone());
        let fees = self
            .fees
            .iter()
            .filter(|f| !f.amount.is_zero())
            .map(|f| FeeCharge { kind: f.kind, amount: Money::new(f.amount, self.currency.clone()) })
            .collect();
        let kind = match self.side {
            Side::Buy => EventKind::Buy { instrument, units: self.units, amount, fees },
            Side::Sell => EventKind::Sell { instrument, units: self.units, amount, fees },
        };
        let event = Event {
            date: self.date,
            kind,
            fx: Some(Fx { currency: self.currency.clone(), rate, mode: FxRateMode::Multiply }),
            venue: self.venue.as_ref().filter(|v| !v.trim().is_empty()).map(|v| Venue { code: v.trim().to_uppercase(), assumed: false }),
            source: Source { broker: MANUAL.into(), file: MANUAL.into(), line: 0, fingerprint: format!("{MANUAL}:{}", self.id) },
        };
        Ok((event, rate))
    }
}

/// Units an amount buys at `price`: whole units, or fractions to six
/// decimals, always rounded down so the amount is never exceeded.
pub fn units_for(amount: Decimal, price: Decimal, whole: bool) -> Decimal {
    if price <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    let dp = if whole { 0 } else { 6 };
    (amount / price).round_dp_with_strategy(dp, rust_decimal::RoundingStrategy::ToZero)
}

/// Books the trade. Returns the new entry.
pub fn record(ledger: &mut Ledger, prices: &mut PriceBook, input: &TradeInput) -> Result<JournalEntry> {
    input.check()?;
    let (event, _) = input.event(&ledger.base_currency, prices)?;
    let fingerprint = event.source.fingerprint.clone();
    if ledger.contains_fingerprint(&fingerprint) {
        return Err(Error::user("trade.recorded"));
    }
    if input.side == Side::Sell {
        let held = ledger.units_at(&ledger::securities_account(&input.broker, &input.symbol), input.date);
        if input.units > held {
            return Err(Error::user("trade.notEnough").arg("held", held.normalize()).arg("symbol", &input.symbol).arg("broker", &input.broker));
        }
    }
    // Work on copies so a refused trade leaves the books untouched.
    let (mut l, mut p) = (ledger.clone(), prices.clone());
    post_events(&mut l, &mut p, &input.config()?, vec![event]).map_err(|e| match e {
        Error::Import { message, .. } => Error::user("trade.refused").arg("reason", message),
        e => e,
    })?;
    let entry = l.entries.iter().find(|e| e.source.as_ref().is_some_and(|s| s.fingerprint == fingerprint)).cloned().expect("just posted");
    (*ledger, *prices) = (l, p);
    Ok(entry)
}

/// What recording the trade would change, without changing anything.
pub fn preview(ledger: &Ledger, prices: &PriceBook, input: &TradeInput) -> Result<TradePreview> {
    let (mut after, mut after_prices) = (ledger.clone(), prices.clone());
    let entry = record(&mut after, &mut after_prices, input)?;
    let (_, rate) = input.event(&ledger.base_currency, prices)?;

    let account = ledger::securities_account(&input.broker, &input.symbol);
    let base_of = |kinds: &[&str]| -> Decimal { entry.postings.iter().filter(|p| kinds.iter().any(|k| p.account.starts_with(k))).map(|p| p.base).sum() };
    let fees = base_of(&["Expenses:"]);
    let gain = (input.side == Side::Sell).then(|| -base_of(&[ledger::CAPITAL_GAINS]));
    let cost = (input.side == Side::Sell).then(|| -base_of(&[account.as_str()]));

    let cash_of = |l: &Ledger, currency: &Currency| {
        l.balances().get(&ledger::cash_account(&input.broker, currency)).and_then(|b| b.amounts.get(currency).copied()).unwrap_or_default()
    };
    let mut currencies: Vec<Currency> = entry.postings.iter().filter(|p| p.account.starts_with("Assets:Cash:")).map(|p| p.amount.currency.clone()).collect();
    currencies.dedup();
    let cash: Vec<CashChange> = currencies.into_iter().map(|c| CashChange { before: cash_of(ledger, &c), after: cash_of(&after, &c), currency: c }).collect();
    let deposit = cash
        .iter()
        .map(|c| {
            let short = (-c.after).max(Decimal::ZERO) - (-c.before).max(Decimal::ZERO);
            let rate = if c.currency == ledger.base_currency { Decimal::ONE } else { rate };
            (short.max(Decimal::ZERO) * rate).round_dp(2)
        })
        .sum();

    let by_symbol = |l: &Ledger, p: &PriceBook| {
        let mut out = std::collections::BTreeMap::<String, Decimal>::new();
        for v in prices::valuations(l, p) {
            *out.entry(v.symbol).or_default() += v.value.unwrap_or(v.cost);
        }
        out
    };
    let (before_w, after_w) = (by_symbol(ledger, prices), by_symbol(&after, &after_prices));
    let mut symbols: Vec<&String> = before_w.keys().chain(after_w.keys()).collect();
    symbols.sort();
    symbols.dedup();
    let holdings = symbols
        .into_iter()
        .map(|s| Weight { symbol: s.clone(), before: before_w.get(s).copied().unwrap_or_default(), after: after_w.get(s).copied().unwrap_or_default() })
        .collect();
    let total_cash = |l: &Ledger| l.balances().iter().filter(|(a, _)| a.starts_with("Assets:Cash:")).map(|(_, b)| b.base).sum();

    Ok(TradePreview {
        value: (input.units * input.price * rate).round_dp(2),
        fees,
        fx_rate: rate,
        cost,
        gain,
        units_before: ledger.units_at(&account, NaiveDate::MAX),
        units_after: after.units_at(&account, NaiveDate::MAX),
        cash,
        invested_before: before_w.values().sum(),
        invested_after: after_w.values().sum(),
        cash_before: total_cash(ledger),
        cash_after: total_cash(&after),
        deposit,
        holdings,
        entry,
    })
}

/// A preview with the tax pack's view of a sale and the long-run effect.
#[derive(Debug, Clone, Serialize)]
pub struct TradeOutlook {
    #[serde(flatten)]
    pub preview: TradePreview,
    /// For a sale, when there is a tax pack for its year.
    pub tax: Option<TradeTax>,
    /// Where all your money could be each year of the projection, without and
    /// with the trade. Both count the deposit, so they differ by the trade alone.
    pub before: Vec<Band>,
    pub after: Vec<Band>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradeTax {
    pub treatment: Treatment,
    pub rate: Decimal,
    pub form: String,
    /// Tax on this gain alone, before netting the year's other gains and losses.
    pub estimate: Decimal,
    pub verified: bool,
}

/// Previews a trade and adds tax and the projection under the plan.
/// `settings` sets the projection; its holdings and retirement are ignored.
pub fn outlook(ledger: &Ledger, prices: &PriceBook, plan: &Plan, input: &TradeInput, pack: Option<&JurisdictionPack>, settings: &WhatIfInput) -> Result<TradeOutlook> {
    let preview = preview(ledger, prices, input)?;
    let tax = match (preview.gain, pack) {
        (Some(gain), Some(pack)) => {
            // Classify each purchase the sale draws on, so a holding period counts.
            let (mut after, mut after_prices) = (ledger.clone(), prices.clone());
            let entry = record(&mut after, &mut after_prices, input)?;
            let parts: Vec<_> = reports::realized_gains_by(&after, entry.date.year(), pack.capital_gains.cost_basis).into_iter().filter(|r| r.entry_id == entry.id).collect();
            let treatments: Vec<Treatment> = parts.iter().map(|r| pack.classify(r)).collect();
            let treatment = if treatments.contains(&Treatment::Taxable) {
                Treatment::Taxable
            } else {
                treatments.first().copied().unwrap_or_else(|| pack.treatment(preview.entry.venue.as_ref()))
            };
            let taxable: Decimal = parts.iter().zip(&treatments).filter(|(_, t)| **t == Treatment::Taxable).map(|(r, _)| r.gain).sum();
            let estimate = if parts.is_empty() { pack.sale_estimate(gain) } else { pack.sale_estimate(taxable) };
            // The rate on the whole gain, e.g. 15% when half is taxed at 30%.
            let rate = pack.capital_gains.rate * pack.inclusion();
            Some(TradeTax { treatment, rate, form: pack.capital_gains.form.clone(), estimate, verified: pack.pack.verified })
        }
        _ => None,
    };
    let path = |cash: Decimal, invested: Decimal| {
        let input = WhatIfInput { cash: to_f64(cash), invested: to_f64(invested), retirement: None, ..settings.clone() };
        whatif::what_if(&input, plan).plan.bands
    };
    let deposit = preview.deposit;
    let (before, after) = (path(preview.cash_before + deposit, preview.invested_before), path(preview.cash_after + deposit, preview.invested_after));
    Ok(TradeOutlook { preview, tax, before, after })
}

/// Removes a trade entered by hand, if it is the latest entry in the books.
pub fn undo(ledger: &mut Ledger, id: &str) -> Result<()> {
    let latest = ledger.entries.iter().map(|e| e.id.as_str()).max();
    let manual = ledger.entries.iter().any(|e| e.id == id && e.source.as_ref().is_some_and(|s| s.broker == MANUAL));
    if latest != Some(id) || !manual {
        return Err(Error::user("trade.undoLatest"));
    }
    ledger.entries.retain(|e| e.id != id);
    Ok(())
}
