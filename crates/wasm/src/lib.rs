//! The browser's view of the core. Data crosses the boundary as JSON strings
//! (amounts stay exact decimal strings) and file contents as byte arrays.

use std::fmt::Display;

use investor_core::import::{self, BrokerConfig};
use investor_core::jurisdiction::JurisdictionPack;
use investor_core::ledger::{Instrument, JournalEntry};
use investor_core::reports::{self, CashBalance, Holding, TrialBalanceRow};
use investor_core::plan::{self, Market, Plan, ProjectionInput};
use investor_core::prices::{self, FxQuote, Quote, QuoteSource, Valuation};
use investor_core::{Currency, Money};
use investor_core::vault::{self, KdfParams, SealingKey, Settings, Vault};
use investor_core::trade::{self, TradeInput};
use investor_core::whatif::{self, WhatIfInput};
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
    plan: &'a Plan,
    holdings: Vec<Holding>,
    valuations: Vec<Valuation>,
    /// Latest rate per currency: units of the base currency for one unit.
    fx: Vec<(&'a Currency, &'a FxQuote)>,
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

    /// Reopens bytes sealed by this session's key, kept from an earlier unlock.
    pub fn resume(bytes: &[u8], key: &[u8]) -> Result<Session, JsError> {
        let key = SealingKey::from_bytes(key).map_err(js_error)?;
        let vault = vault::unlock_with_key(bytes, &key).map_err(js_error)?;
        Ok(Session { vault, key })
    }

    /// The key, so a refresh can reopen the file without the password.
    #[wasm_bindgen(js_name = exportKey)]
    pub fn export_key(&self) -> Vec<u8> {
        self.key.to_bytes()
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
            plan: &self.vault.plan,
            holdings: reports::holdings(ledger),
            valuations: prices::valuations(ledger, &self.vault.prices),
            fx: self.vault.prices.fx.iter().filter_map(|(c, q)| Some((c, q.last()?))).collect(),
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
        let mut prices = self.vault.prices.clone();
        let report = import::import_statement(&mut ledger, &mut prices, &config, file_name, text).map_err(js_error)?;
        self.vault.ledger = ledger;
        self.vault.prices = prices;
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

    /// Deposits, withdrawals, buys and sells per month, as JSON.
    pub fn flows(&self) -> Result<String, JsError> {
        json(&reports::monthly_flows(&self.vault.ledger))
    }

    /// Realised gains for a year without tax classification, as JSON.
    pub fn sales(&self, year: i32) -> Result<String, JsError> {
        json(&reports::realized_gains(&self.vault.ledger, year))
    }

    /// Replaces the profile and goals. `plan` is JSON.
    #[wasm_bindgen(js_name = setPlan)]
    pub fn set_plan(&mut self, plan: &str) -> Result<(), JsError> {
        self.vault.plan = serde_json::from_str(plan).map_err(js_error)?;
        Ok(())
    }

    fn holdings_value(&self) -> (f64, f64) {
        whatif::starting_point(&self.vault.ledger, &self.vault.prices)
    }

    /// Progress on every goal, with today's money assigned in priority order.
    /// `pace` scales the planned monthly amounts (1 = as planned).
    pub fn goals(&self, this_year: i32, market: &str, pace: f64) -> Result<String, JsError> {
        let market: Market = serde_json::from_value(serde_json::Value::String(market.to_owned())).map_err(js_error)?;
        let (cash, invested) = self.holdings_value();
        json(&plan::evaluate_goals(&self.vault.plan, cash, invested, this_year, market, pace))
    }

    /// Today's holdings under the plan, with and without the moves being
    /// tried. `input` is JSON:
    /// `{"today","years","market","spend_goals","retirement"?}`.
    #[wasm_bindgen(js_name = whatIf)]
    pub fn what_if(&self, input: &str) -> Result<String, JsError> {
        let mut input: WhatIfInput = serde_json::from_str(input).map_err(js_error)?;
        (input.cash, input.invested) = self.holdings_value();
        json(&whatif::what_if(&input, &self.vault.plan))
    }

    /// What a trade would change, without changing anything. `input` is a
    /// trade as JSON; `pack_toml` the tax pack for the trade's year, if any;
    /// `what_if` the projection settings as for `whatIf`.
    #[wasm_bindgen(js_name = previewTrade)]
    pub fn preview_trade(&self, input: &str, pack_toml: Option<String>, what_if: &str) -> Result<String, JsError> {
        let input: TradeInput = serde_json::from_str(input).map_err(js_error)?;
        let pack = pack_toml.map(|toml| JurisdictionPack::from_toml(&toml)).transpose().map_err(js_error)?;
        let settings: WhatIfInput = serde_json::from_str(what_if).map_err(js_error)?;
        let v = &self.vault;
        json(&trade::outlook(&v.ledger, &v.prices, &v.plan, &input, pack.as_ref(), &settings).map_err(js_error)?)
    }

    /// Books a trade. Returns the new entry as JSON.
    #[wasm_bindgen(js_name = recordTrade)]
    pub fn record_trade(&mut self, input: &str) -> Result<String, JsError> {
        let input: TradeInput = serde_json::from_str(input).map_err(js_error)?;
        json(&trade::record(&mut self.vault.ledger, &mut self.vault.prices, &input).map_err(js_error)?)
    }

    /// Removes the latest trade entered by hand.
    #[wasm_bindgen(js_name = undoTrade)]
    pub fn undo_trade(&mut self, id: &str) -> Result<(), JsError> {
        trade::undo(&mut self.vault.ledger, id).map_err(js_error)
    }

    /// Records a price typed in by the user. `date` is `YYYY-MM-DD`.
    #[wasm_bindgen(js_name = setPrice)]
    pub fn set_price(&mut self, symbol: &str, date: &str, amount: &str, currency: &str) -> Result<(), JsError> {
        let date = date.parse().map_err(js_error)?;
        let amount = amount.trim().replace(',', ".").parse().map_err(js_error)?;
        let price = Money::new(amount, Currency::new(currency).map_err(js_error)?);
        self.vault.prices.add_quote(symbol, Quote { date, price, source: QuoteSource::Manual });
        Ok(())
    }

    /// Records an exchange rate typed in by the user: base units for one `currency`.
    #[wasm_bindgen(js_name = setFxRate)]
    pub fn set_fx_rate(&mut self, currency: &str, date: &str, rate: &str) -> Result<(), JsError> {
        let date = date.parse().map_err(js_error)?;
        let rate = rate.trim().replace(',', ".").parse().map_err(js_error)?;
        self.vault.prices.add_fx(Currency::new(currency).map_err(js_error)?, FxQuote { date, rate, source: QuoteSource::Manual });
        Ok(())
    }

    /// Replaces the key after checking the current password. Saves made
    /// afterwards need the new password; older copies keep the old one.
    #[wasm_bindgen(js_name = changePassword)]
    pub fn change_password(&mut self, current: &str, new: &str) -> Result<(), JsError> {
        if !self.key.matches(current).map_err(js_error)? {
            return Err(JsError::new("wrong password"));
        }
        self.key = SealingKey::derive(new, KdfParams::default()).map_err(js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setLanguage)]
    pub fn set_language(&mut self, language: &str) {
        self.vault.settings.language = language.to_owned();
    }
}

/// Simulated growth of a portfolio. `input` and the result are JSON.
#[wasm_bindgen]
pub fn project(input: &str) -> Result<String, JsError> {
    let input: ProjectionInput = serde_json::from_str(input).map_err(js_error)?;
    json(&plan::project(&input))
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
