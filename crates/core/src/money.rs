use std::fmt;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// An ISO 4217 currency code such as `EUR`, always upper case.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Currency(String);

impl Currency {
    pub fn new(code: &str) -> Result<Self> {
        let code = code.trim().to_ascii_uppercase();
        if code.len() == 3 && code.chars().all(|c| c.is_ascii_alphabetic()) {
            Ok(Self(code))
        } else {
            Err(Error::InvalidCurrency(code))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Currency {
    type Error = Error;

    fn try_from(code: String) -> Result<Self> {
        Self::new(&code)
    }
}

impl From<Currency> for String {
    fn from(currency: Currency) -> Self {
        currency.0
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An exact amount in one currency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn negated(&self) -> Self {
        Self::new(-self.amount, self.currency.clone())
    }

    pub fn abs(&self) -> Self {
        Self::new(self.amount.abs(), self.currency.clone())
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount.normalize(), self.currency)
    }
}
