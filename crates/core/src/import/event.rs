use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::config::FxRateMode;
use crate::ledger::{Instrument, Source, Venue};
use crate::{Currency, Money};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeeKind {
    Commission,
    FxConversion,
    Exchange,
    TransactionTax,
    Custody,
    Interest,
    Other,
}

impl FeeKind {
    pub fn account(self) -> &'static str {
        match self {
            Self::Commission => "Expenses:Fees:Commission",
            Self::FxConversion => "Expenses:Fees:FX",
            Self::Exchange => "Expenses:Fees:Exchange",
            Self::TransactionTax => "Expenses:Fees:TransactionTax",
            Self::Custody => "Expenses:Fees:Custody",
            Self::Interest => crate::ledger::INTEREST_EXPENSE,
            Self::Other => "Expenses:Fees:Other",
        }
    }
}

/// A cost charged on top of a trade. Always positive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeCharge {
    pub kind: FeeKind,
    pub amount: Money,
}

/// The statement's exchange rate between one currency and the base currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fx {
    pub currency: Currency,
    pub rate: Decimal,
    pub mode: FxRateMode,
}

impl Fx {
    pub fn to_base(&self, amount: Decimal) -> Option<Decimal> {
        match self.mode {
            FxRateMode::Multiply => Some(amount * self.rate),
            FxRateMode::Divide if !self.rate.is_zero() => Some(amount / self.rate),
            FxRateMode::Divide => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Deposit { amount: Money },
    Withdrawal { amount: Money },
    Buy { instrument: Instrument, units: Decimal, amount: Money, fees: Vec<FeeCharge> },
    Sell { instrument: Instrument, units: Decimal, amount: Money, fees: Vec<FeeCharge> },
    /// Signed: negative for a reversal.
    Dividend { instrument: Instrument, amount: Money },
    /// Signed as on the statement: negative when tax is taken.
    WithholdingTax { instrument: Option<Instrument>, amount: Money },
    /// Signed as on the statement: negative when charged.
    Fee { kind: FeeKind, amount: Money },
    /// Signed: positive when received.
    Interest { amount: Money },
    FxConversion { sold: Money, bought: Money, fees: Vec<FeeCharge> },
    /// `price` is the market price per unit in `cost`'s currency, when the statement gives one.
    PositionSnapshot { instrument: Instrument, units: Decimal, cost: Money, price: Option<Decimal> },
}

/// One broker-neutral thing that happened, read from a statement row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub date: NaiveDate,
    pub kind: EventKind,
    pub fx: Option<Fx>,
    pub venue: Option<Venue>,
    pub source: Source,
}
