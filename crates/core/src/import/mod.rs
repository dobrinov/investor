//! Config-driven statement import.
//!
//! A broker is described by a TOML file (see `brokers/` in the repository):
//! how its CSV is laid out and which rows mean what. [`parse`] turns a
//! statement into broker-neutral [`Event`]s and [`post_events`] books them
//! into the ledger as balanced journal entries, skipping rows already imported.

mod config;
mod event;
mod parse;
mod post;

pub use config::{AmountStyle, BrokerConfig, BrokerInfo, Condition, CsvOptions, FeeColumn, Fields, FxRateMode, Layout, RecordKind, RecordRule};
pub use event::{Event, EventKind, FeeCharge, FeeKind, Fx};
pub use parse::{fit, parse, ParsedFile};
pub use post::{post_events, PostReport};

use crate::ledger::Ledger;
use crate::Result;

/// Parses a statement and books it, returning what happened.
pub fn import_statement(ledger: &mut Ledger, config: &BrokerConfig, file_name: &str, text: &str) -> Result<PostReport> {
    let parsed = parse(config, file_name, text)?;
    let mut report = post_events(ledger, config, parsed.events)?;
    report.warnings.splice(0..0, parsed.warnings);
    Ok(report)
}
