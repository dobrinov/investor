use rust_decimal::Decimal;
use serde::Deserialize;

use super::event::FeeKind;
use crate::{Error, Result};

/// A broker or exchange statement format, loaded from `brokers/<id>.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrokerConfig {
    pub broker: BrokerInfo,
    pub csv: CsvOptions,
    #[serde(rename = "record", default)]
    pub records: Vec<RecordRule>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrokerInfo {
    pub id: String,
    pub name: String,
    /// Segment used in ledger account names, e.g. `IBKR` in `Assets:Cash:IBKR:EUR`.
    pub account: String,
    /// Venue to assume when the statement does not say where a trade happened.
    #[serde(default)]
    pub default_venue: Option<String>,
    /// Leftover fractions of a unit at or below this are treated as sold.
    #[serde(default = "default_dust")]
    pub dust: Decimal,
}

fn default_dust() -> Decimal {
    Decimal::new(1, 6)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CsvOptions {
    #[serde(default = "default_delimiter")]
    pub delimiter: char,
    #[serde(default)]
    pub layout: Layout,
    /// `rfc3339`, or a chrono format such as `%Y-%m-%d`.
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default)]
    pub amounts: AmountStyle,
    #[serde(default)]
    pub fx_rate: FxRateMode,
}

fn default_delimiter() -> char {
    ','
}

fn default_date_format() -> String {
    "%Y-%m-%d".into()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    /// One header row followed by data rows.
    #[default]
    Table,
    /// IBKR Flex style: `HEADER`/`DATA` in column 1, a section code in column 2.
    Sectioned,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmountStyle {
    /// `250.00`, with the currency in its own column.
    #[default]
    Plain,
    /// `EUR 250.00`.
    CurrencyPrefixed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FxRateMode {
    /// base amount = amount × rate
    #[default]
    Multiply,
    /// base amount = amount ÷ rate
    Divide,
}

/// Maps matching rows to one kind of event. The first matching rule wins.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordRule {
    /// Section code, for `sectioned` layouts.
    #[serde(default)]
    pub section: Option<String>,
    /// All conditions must hold.
    #[serde(default)]
    pub when: Vec<Condition>,
    pub kind: RecordKind,
    /// For `kind = "fee"`.
    #[serde(default)]
    pub fee_kind: Option<FeeKind>,
    #[serde(default)]
    pub fields: Fields,
    #[serde(default)]
    pub fees: Vec<FeeColumn>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub column: String,
    #[serde(default)]
    pub equals: Option<String>,
    #[serde(default)]
    pub one_of: Vec<String>,
    #[serde(default)]
    pub starts_with: Option<String>,
    #[serde(default)]
    pub not_empty: bool,
}

impl Condition {
    pub fn matches(&self, value: Option<&str>) -> bool {
        let value = value.unwrap_or("").trim();
        self.equals.as_deref().is_none_or(|e| value == e)
            && (self.one_of.is_empty() || self.one_of.iter().any(|o| o == value))
            && self.starts_with.as_deref().is_none_or(|p| value.starts_with(p))
            && (!self.not_empty || !value.is_empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    Buy,
    Sell,
    Deposit,
    Withdrawal,
    /// Deposit or withdrawal depending on the amount's sign.
    CashTransfer,
    Dividend,
    WithholdingTax,
    Fee,
    Interest,
    /// A currency conversion written as a trade in a pair like `EUR.USD`.
    FxPairTrade,
    /// Holdings as of the statement date, used to reconcile.
    PositionSnapshot,
    /// Known rows that carry nothing to book.
    Ignore,
}

/// Column names for each piece of information.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fields {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub isin: Option<String>,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub quantity: Option<String>,
    /// Trade value, cash amount, or cost for snapshots.
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub fx_rate: Option<String>,
    #[serde(default)]
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeeColumn {
    pub column: String,
    pub kind: FeeKind,
    /// Column holding the fee's currency when it differs from the trade's.
    #[serde(default)]
    pub currency_column: Option<String>,
}

impl BrokerConfig {
    pub fn from_toml(text: &str) -> Result<Self> {
        let config: Self = toml::from_str(text)?;
        config.check()?;
        Ok(config)
    }

    fn check(&self) -> Result<()> {
        let fail = |msg: String| Err(Error::Config(format!("{}: {msg}", self.broker.id)));
        if self.records.is_empty() {
            return fail("no [[record]] rules".into());
        }
        for (i, rule) in self.records.iter().enumerate() {
            let n = i + 1;
            if self.csv.layout == Layout::Sectioned && rule.section.is_none() {
                return fail(format!("record {n} needs a section for the sectioned layout"));
            }
            if rule.kind == RecordKind::Ignore {
                continue;
            }
            let f = &rule.fields;
            let mut needed = vec![("date", &f.date), ("amount", &f.amount)];
            match rule.kind {
                RecordKind::Buy | RecordKind::Sell | RecordKind::PositionSnapshot => {
                    needed.extend([("symbol", &f.symbol), ("quantity", &f.quantity)])
                }
                RecordKind::Dividend => needed.push(("symbol", &f.symbol)),
                RecordKind::FxPairTrade => needed.extend([("symbol", &f.symbol), ("quantity", &f.quantity)]),
                _ => {}
            }
            if let Some((name, _)) = needed.iter().find(|(_, col)| col.is_none()) {
                return fail(format!("record {n} ({:?}) needs fields.{name}", rule.kind));
            }
            if self.csv.amounts == AmountStyle::Plain && f.currency.is_none() {
                return fail(format!("record {n} needs fields.currency when amounts are plain numbers"));
            }
        }
        Ok(())
    }
}
