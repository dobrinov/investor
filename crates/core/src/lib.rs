//! Core of Investor: the double-entry ledger, config-driven broker importers,
//! jurisdiction tax packs and the encrypted `.invest` file format.
//!
//! Everything here is pure computation so the same code runs natively in the
//! desktop app and as WebAssembly in the browser edition.

pub mod assistant;
pub mod bank;
pub mod example;
mod error;
pub mod import;
pub mod jurisdiction;
pub mod ledger;
pub mod lots;
pub mod metal;
mod money;
pub mod plan;
pub mod prices;
pub mod private;
pub mod reports;
pub mod trade;
pub mod vault;
pub mod whatif;

pub use error::{Error, Result};
pub use money::{Currency, Money};
