use std::collections::BTreeMap;

use rust_decimal::Decimal;

use super::config::BrokerConfig;
use super::event::{Event, EventKind, FeeCharge, Fx};
use crate::ledger::{self, Instrument, JournalEntry, Ledger, Posting};
use crate::lots::Lots;
use crate::{Currency, Error, Money, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PostReport {
    pub added: usize,
    pub duplicates: usize,
    pub warnings: Vec<String>,
}

/// Books events into the ledger in date order. Rows imported before are
/// skipped; position snapshots go last and only add what the history misses.
pub fn post_events(ledger: &mut Ledger, config: &BrokerConfig, mut events: Vec<Event>) -> Result<PostReport> {
    events.sort_by_key(|e| (matches!(e.kind, EventKind::PositionSnapshot { .. }), e.date, e.source.line));
    let mut lots = Lots::from_ledger(ledger);
    let mut report = PostReport::default();

    for event in events {
        if ledger.contains_fingerprint(&event.source.fingerprint) {
            report.duplicates += 1;
            continue;
        }
        let mut builder = EntryBuilder { base: &ledger.base_currency, config, event: &event, lots: &mut lots };
        let Some(entry) = builder.build().map_err(|message| Error::Import {
            file: event.source.file.clone(),
            line: event.source.line,
            message,
        })?
        else {
            continue;
        };
        if let Some(instrument) = instrument_of(&event.kind) {
            ledger.register_instrument(instrument);
        }
        for note in &entry.notes {
            report.warnings.push(format!("{}, line {}: {note}", event.source.file, event.source.line));
        }
        let entry = ledger.post(entry)?;
        // Disposals were taken out while building; new lots are opened now that the id is known.
        for p in &entry.postings {
            if let Some(units) = p.units.filter(|u| u.is_sign_positive()) {
                lots.open(&p.account, entry.date, &entry.id, units, p.base);
            }
        }
        report.added += 1;
    }
    Ok(report)
}

fn instrument_of(kind: &EventKind) -> Option<&Instrument> {
    match kind {
        EventKind::Buy { instrument, .. }
        | EventKind::Sell { instrument, .. }
        | EventKind::Dividend { instrument, .. }
        | EventKind::PositionSnapshot { instrument, .. } => Some(instrument),
        EventKind::WithholdingTax { instrument, .. } => instrument.as_ref(),
        _ => None,
    }
}

struct EntryBuilder<'a> {
    base: &'a Currency,
    config: &'a BrokerConfig,
    event: &'a Event,
    lots: &'a mut Lots,
}

type Built = std::result::Result<Option<JournalEntry>, String>;

impl EntryBuilder<'_> {
    fn build(&mut self) -> Built {
        let broker = &self.config.broker.account;
        match &self.event.kind {
            EventKind::Deposit { amount } => {
                let base = self.to_base(amount)?;
                self.entry(
                    format!("Deposit {amount}"),
                    vec![self.cash(amount.clone(), base), self.base_posting(ledger::CONTRIBUTIONS, -base)],
                )
            }
            EventKind::Withdrawal { amount } => {
                let base = self.to_base(amount)?;
                self.entry(
                    format!("Withdrawal {amount}"),
                    vec![self.cash(amount.negated(), -base), self.base_posting(ledger::CONTRIBUTIONS, base)],
                )
            }
            EventKind::Buy { instrument, units, amount, fees } => {
                let account = ledger::securities_account(broker, &instrument.symbol);
                let cost = self.to_base(amount)?;
                let mut postings = vec![self.base_posting(&account, cost).with_units(*units)];
                self.add_fees(&mut postings, fees)?;
                self.settle_cash(&mut postings, amount.clone(), fees)?;
                self.entry(format!("Buy {} {} for {amount}", units.normalize(), instrument.symbol), postings)
            }
            EventKind::Sell { instrument, units, amount, fees } => self.sell(instrument, *units, amount, fees),
            EventKind::Dividend { instrument, amount } => {
                let base = self.to_base(amount)?;
                let income = Posting::new(ledger::dividends_account(&instrument.symbol), amount.negated(), -base);
                self.entry(format!("Dividend {} {amount}", instrument.symbol), vec![self.cash(amount.clone(), base), income])
            }
            EventKind::WithholdingTax { instrument, amount } => {
                let base = self.to_base(amount)?;
                let tax = Posting::new(ledger::WITHHOLDING_TAX, amount.negated(), -base);
                let what = instrument.as_ref().map_or(String::new(), |i| format!(" {}", i.symbol));
                self.entry(format!("Withholding tax{what} {}", amount.abs()), vec![self.cash(amount.clone(), base), tax])
            }
            EventKind::Fee { kind, amount } => {
                let base = self.to_base(amount)?;
                let fee = Posting::new(kind.account(), amount.negated(), -base);
                self.entry(format!("Fee {}", amount.abs()), vec![self.cash(amount.clone(), base), fee])
            }
            EventKind::Interest { amount } => {
                let base = self.to_base(amount)?;
                let account = if amount.amount.is_sign_negative() { ledger::INTEREST_EXPENSE } else { ledger::INTEREST_INCOME };
                let other = Posting::new(account, amount.negated(), -base);
                self.entry(format!("Interest {amount}"), vec![self.cash(amount.clone(), base), other])
            }
            EventKind::FxConversion { sold, bought, fees } => {
                let mut postings = Vec::new();
                self.add_fees(&mut postings, fees)?;
                let mut cash: BTreeMap<Currency, Decimal> = BTreeMap::new();
                *cash.entry(sold.currency.clone()).or_default() -= sold.amount;
                *cash.entry(bought.currency.clone()).or_default() += bought.amount;
                for fee in fees {
                    *cash.entry(fee.amount.currency.clone()).or_default() -= fee.amount.amount;
                }
                // Legs are converted at the statement's rate except one, which
                // takes whatever balances, so no exchange gain or loss is invented.
                let balancing = if &bought.currency == self.base { &sold.currency } else { &bought.currency };
                let mut legs: Vec<(Currency, Decimal)> = cash.into_iter().collect();
                legs.sort_by_key(|(currency, _)| currency == balancing);
                let last = legs.len() - 1;
                for (i, (currency, amount)) in legs.into_iter().enumerate() {
                    let money = Money::new(amount, currency);
                    let base = if i == last { -postings.iter().map(|p| p.base).sum::<Decimal>() } else { self.to_base(&money)? };
                    postings.push(self.cash(money, base));
                }
                self.entry(format!("Convert {sold} to {bought}"), postings)
            }
            EventKind::PositionSnapshot { instrument, units, cost } => self.reconcile(instrument, *units, cost),
        }
    }

    fn sell(&mut self, instrument: &Instrument, units: Decimal, amount: &Money, fees: &[FeeCharge]) -> Built {
        let account = ledger::securities_account(&self.config.broker.account, &instrument.symbol);
        let held = self.lots.held(&account);
        let dust = self.config.broker.dust;
        let mut notes = Vec::new();
        // Brokers that trade fractions can leave a sliver behind; close it with the sale.
        let units = if (held - units).abs() <= dust && held > Decimal::ZERO && held != units {
            notes.push(format!(
                "sold {} {} but {} were held; the leftover {} is treated as sold",
                units.normalize(),
                instrument.symbol,
                held.normalize(),
                (held - units).normalize()
            ));
            held
        } else {
            units
        };
        let disposal = self.lots.dispose(&account, units).map_err(|e| e.to_string())?;
        let proceeds = self.to_base(amount)?;
        let gain = proceeds - disposal.cost;

        let mut postings = vec![
            self.base_posting(&account, -disposal.cost).with_units(-units),
            self.base_posting(ledger::CAPITAL_GAINS, -gain),
        ];
        self.add_fees(&mut postings, fees)?;
        self.settle_cash(&mut postings, amount.negated(), fees)?;
        let mut entry = self.entry(format!("Sell {} {} for {amount}", units.normalize(), instrument.symbol), postings)?;
        if let Some(entry) = entry.as_mut() {
            entry.notes = notes;
        }
        Ok(entry)
    }

    /// Adds an opening balance for units the broker reports but the imported history does not explain.
    fn reconcile(&mut self, instrument: &Instrument, units: Decimal, cost: &Money) -> Built {
        let account = ledger::securities_account(&self.config.broker.account, &instrument.symbol);
        let held = self.lots.held(&account);
        let missing = units - held;
        if missing.abs() <= self.config.broker.dust {
            return Ok(None);
        }
        if missing.is_sign_negative() {
            return Err(format!(
                "the statement shows {} {} but the imported history adds up to {}; check for a missing sale or transfer",
                units.normalize(),
                instrument.symbol,
                held.normalize()
            ));
        }
        let missing_cost = self.to_base(cost)? * missing / units;
        let postings = vec![
            self.base_posting(&account, missing_cost).with_units(missing),
            self.base_posting(ledger::OPENING_BALANCES, -missing_cost),
        ];
        let mut entry =
            self.entry(format!("Opening balance {} {} (from position snapshot)", missing.normalize(), instrument.symbol), postings)?;
        if let Some(entry) = entry.as_mut() {
            entry.notes.push(format!(
                "{} {} added as an opening balance because the purchase is not in the imported history; purchase date and fees are unknown",
                missing.normalize(),
                instrument.symbol
            ));
        }
        Ok(entry)
    }

    fn entry(&self, description: String, postings: Vec<Posting>) -> Built {
        let has_units = postings.iter().any(|p| p.units.is_some());
        Ok(Some(JournalEntry {
            id: String::new(),
            date: self.event.date,
            description,
            postings,
            venue: if has_units { self.event.venue.clone() } else { None },
            source: Some(self.event.source.clone()),
            notes: Vec::new(),
        }))
    }

    fn to_base(&self, money: &Money) -> std::result::Result<Decimal, String> {
        if &money.currency == self.base {
            return Ok(money.amount);
        }
        match &self.event.fx {
            Some(fx @ Fx { currency, .. }) if currency == &money.currency => {
                fx.to_base(money.amount).ok_or_else(|| format!("exchange rate for {currency} is zero"))
            }
            _ => Err(format!("no exchange rate from {} to {} in the statement", money.currency, self.base)),
        }
    }

    fn cash(&self, amount: Money, base: Decimal) -> Posting {
        Posting::new(ledger::cash_account(&self.config.broker.account, &amount.currency), amount, base)
    }

    fn base_posting(&self, account: &str, base: Decimal) -> Posting {
        Posting::new(account, Money::new(base, self.base.clone()), base)
    }

    fn add_fees(&self, postings: &mut Vec<Posting>, fees: &[FeeCharge]) -> std::result::Result<(), String> {
        for fee in fees {
            postings.push(Posting::new(fee.kind.account(), fee.amount.clone(), self.to_base(&fee.amount)?));
        }
        Ok(())
    }

    /// Adds the cash side: `trade` leaves (or arrives, if negative) plus all
    /// fees leave, one posting per currency. The last posting absorbs rounding
    /// so the entry balances exactly.
    fn settle_cash(&self, postings: &mut Vec<Posting>, trade: Money, fees: &[FeeCharge]) -> std::result::Result<(), String> {
        let mut by_currency: BTreeMap<Currency, Decimal> = BTreeMap::new();
        *by_currency.entry(trade.currency.clone()).or_default() -= trade.amount;
        for fee in fees {
            *by_currency.entry(fee.amount.currency.clone()).or_default() -= fee.amount.amount;
        }
        // Put the trade currency last so it absorbs any rounding.
        let mut legs: Vec<(Currency, Decimal)> = by_currency.into_iter().collect();
        legs.sort_by_key(|(currency, _)| currency == &trade.currency);
        let last = legs.len() - 1;
        for (i, (currency, amount)) in legs.into_iter().enumerate() {
            let money = Money::new(amount, currency);
            let base = if i == last { -postings.iter().map(|p| p.base).sum::<Decimal>() } else { self.to_base(&money)? };
            postings.push(self.cash(money, base));
        }
        Ok(())
    }
}
