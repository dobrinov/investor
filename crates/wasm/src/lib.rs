//! The browser's view of the core. Data crosses the boundary as JSON strings
//! (amounts stay exact decimal strings) and file contents as byte arrays.

use std::fmt::Display;

use investor_core::import::{self, BrokerConfig};
use investor_core::jurisdiction::JurisdictionPack;
use investor_core::ledger::{Instrument, JournalEntry};
use investor_core::reports::{self, CashBalance, Holding, TrialBalanceRow};
use investor_core::vault::{self, KdfParams, SealingKey, Settings, Vault};
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn js_error(e: impl Display) -> JsError {
    JsError::new(&e.to_string())
}

fn json(value: &impl Serialize) -> Result<String, JsError> {
    serde_json::to_string(value).map_err(js_error)
}

/// An unlocked book. The key lives only in this object's memory.
#[wasm_bindgen]
pub struct Session {
    vault: Vault,
    key: SealingKey,
}

#[derive(Serialize)]
struct Snapshot<'a> {
    settings: &'a Settings,
    holdings: Vec<Holding>,
    cash: Vec<CashBalance>,
    trial_balance: Vec<TrialBalanceRow>,
    instruments: Vec<&'a Instrument>,
    /// Newest first.
    entries: Vec<&'a JournalEntry>,
}

#[derive(Serialize)]
struct Gains {
    pack: investor_core::jurisdiction::PackInfo,
    rate: String,
    form: String,
    rows: Vec<GainRow>,
    summary: investor_core::jurisdiction::GainsSummary,
}

#[derive(Serialize)]
struct GainRow {
    #[serde(flatten)]
    realization: reports::Realization,
    treatment: investor_core::jurisdiction::Treatment,
}

#[wasm_bindgen]
impl Session {
    /// Starts a new book. `settings` is JSON: `{"base_currency","language","tax_residency"}`.
    pub fn create(settings: &str, password: &str) -> Result<Session, JsError> {
        let settings: Settings = serde_json::from_str(settings).map_err(js_error)?;
        let key = SealingKey::derive(password, KdfParams::default()).map_err(js_error)?;
        Ok(Session { vault: Vault::new(settings), key })
    }

    /// Opens the bytes of a `.invest` file.
    pub fn unlock(bytes: &[u8], password: &str) -> Result<Session, JsError> {
        let (vault, key) = vault::unlock(bytes, password).map_err(js_error)?;
        Ok(Session { vault, key })
    }

    /// The encrypted file contents, ready to save.
    pub fn seal(&self) -> Result<Vec<u8>, JsError> {
        vault::seal_with(&self.vault, &self.key).map_err(js_error)
    }

    pub fn snapshot(&self) -> Result<String, JsError> {
        let ledger = &self.vault.ledger;
        let mut entries: Vec<&JournalEntry> = ledger.entries_by_date();
        entries.reverse();
        json(&Snapshot {
            settings: &self.vault.settings,
            holdings: reports::holdings(ledger),
            cash: reports::cash(ledger),
            trial_balance: reports::trial_balance(ledger),
            instruments: ledger.instruments.values().collect(),
            entries,
        })
    }

    /// Imports a statement. Returns the import report as JSON.
    pub fn import(&mut self, broker_toml: &str, file_name: &str, text: &str) -> Result<String, JsError> {
        let config = BrokerConfig::from_toml(broker_toml).map_err(js_error)?;
        // Import into a copy so a failure halfway leaves the book untouched.
        let mut ledger = self.vault.ledger.clone();
        let report = import::import_statement(&mut ledger, &config, file_name, text).map_err(js_error)?;
        self.vault.ledger = ledger;
        json(&report)
    }

    /// Realised gains for a year under a jurisdiction pack, as JSON.
    pub fn gains(&self, year: i32, pack_toml: &str) -> Result<String, JsError> {
        let pack = JurisdictionPack::from_toml(pack_toml).map_err(js_error)?;
        let rows: Vec<GainRow> = reports::realized_gains(&self.vault.ledger, year)
            .into_iter()
            .map(|r| GainRow { treatment: pack.treatment(r.venue.as_ref()), realization: r })
            .collect();
        let summary = pack.summarize(&rows.iter().map(|r| r.realization.clone()).collect::<Vec<_>>());
        json(&Gains {
            rate: pack.capital_gains.rate.to_string(),
            form: pack.capital_gains.form.clone(),
            pack: pack.pack,
            rows,
            summary,
        })
    }

    /// Realised gains for a year without tax classification, as JSON.
    pub fn sales(&self, year: i32) -> Result<String, JsError> {
        json(&reports::realized_gains(&self.vault.ledger, year))
    }

    #[wasm_bindgen(js_name = setLanguage)]
    pub fn set_language(&mut self, language: &str) {
        self.vault.settings.language = language.to_owned();
    }
}

/// How well a broker config fits a statement, from 0 to 1.
#[wasm_bindgen(js_name = brokerFit)]
pub fn broker_fit(broker_toml: &str, text: &str) -> Result<f64, JsError> {
    let config = BrokerConfig::from_toml(broker_toml).map_err(js_error)?;
    Ok(import::fit(&config, text))
}

/// Checks a password against a file without keeping it open.
#[wasm_bindgen(js_name = isInvestFile)]
pub fn is_invest_file(bytes: &[u8]) -> bool {
    bytes.starts_with(b"INVEST\0")
}
