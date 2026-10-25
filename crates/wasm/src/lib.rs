//! The browser's view of the core. Data crosses the boundary as JSON strings
//! (amounts stay exact decimal strings) and file contents as byte arrays.

use std::fmt::Display;

use investor_core::bank::{self, BalanceInput, BankBalance, NewAccount};
use investor_core::import::{self, BrokerConfig};
use investor_core::jurisdiction::JurisdictionPack;
use investor_core::ledger::{Instrument, JournalEntry};
use investor_core::metal::{self, MetalHolding, MetalInput, MetalPrice};
use investor_core::reports::{self, CashBalance, Holding, TrialBalanceRow};
use investor_core::plan::{self, Market, Plan, ProjectionInput};
use investor_core::prices::{self, FxQuote, Quote, QuoteSource, Valuation};
use investor_core::private::{self, Estimate, PayoutInput, PrivateInput, PrivateValuation};
use investor_core::{Currency, Money};
use investor_core::vault::{self, KdfParams, SealingKey, Settings, Vault};
use investor_core::trade::{self, TradeInput};
use investor_core::whatif::{self, WhatIfInput};
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn js_error(e: impl Display) -> JsError {
    JsError::new(&e.to_string())
}

/// The English text, then after a NUL the code and values as JSON, so the
/// web app can show the message in the reader's language.
fn core_error(e: investor_core::Error) -> JsError {
    let Some(problem) = e.problem() else { return js_error(e) };
    let args: serde_json::Map<String, serde_json::Value> = problem.args.iter().map(|(k, v)| ((*k).to_owned(), v.clone().into())).collect();
    JsError::new(&format!("{}\u{0}{}", problem.english(), serde_json::json!({ "code": problem.code, "args": args })))
}

/// A typed-in value that does not parse, such as an empty amount.
fn bad_input(e: impl Display) -> JsError {
    core_error(investor_core::Error::user("input.invalid").arg("detail", e))
}

fn problem(code: &'static str) -> JsError {
    core_error(investor_core::Error::user(code))
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
    /// Holdings without a market price, outside the projections.
    private: Vec<PrivateValuation>,
    /// Bank accounts whose balance is typed in; their cash is also in `cash`.
    banks: Vec<BankBalance>,
    /// Coins and bars; they are also in `valuations`, at the `PHYSICAL` location.
    metals: Vec<MetalHolding>,
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
    /// On the whole gain: the pack's rate times the share that is taxed.
    rate: String,
    /// The share of the gain that is taxed, when not all of it.
    inclusion: Option<String>,
    /// The pack's own rate, on the taxed share.
    marginal: String,
    /// The year's tax is worked out by band, so `rate` is only the first band.
    banded: bool,
    /// Net gains up to this much a year are tax-free, in the pack's currency.
    allowance: Option<String>,
    form: String,
    /// How the pack taxes coins and bars, when it says.
    metals: Option<investor_core::jurisdiction::PhysicalMetals>,
    rows: Vec<GainRow>,
    /// Pay from work that is not salary, with the pack's rule when it has one.
    employment: Vec<reports::IncomeRow>,
    employment_rule: Option<investor_core::jurisdiction::EmploymentIncome>,
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
        let key = SealingKey::derive(password, KdfParams::default()).map_err(core_error)?;
        Ok(Session { vault: Vault::new(settings), key })
    }

    /// The example portfolio as of `today` (`YYYY-MM-DD`). It is never meant
    /// to be saved; its key is a throwaway.
    pub fn example(today: &str, language: &str) -> Result<Session, JsError> {
        let today = today.parse().map_err(js_error)?;
        let vault = investor_core::example::vault(today, language).map_err(core_error)?;
        let key = SealingKey::derive("example", KdfParams { memory_kib: 8 * 1024, iterations: 1, lanes: 1 }).map_err(core_error)?;
        Ok(Session { vault, key })
    }

    /// Opens the bytes of a `.invest` file.
    pub fn unlock(bytes: &[u8], password: &str) -> Result<Session, JsError> {
        let (vault, key) = vault::unlock(bytes, password).map_err(core_error)?;
        Ok(Session { vault, key })
    }

    /// Reopens bytes sealed by this session's key, kept from an earlier unlock.
    pub fn resume(bytes: &[u8], key: &[u8]) -> Result<Session, JsError> {
        let key = SealingKey::from_bytes(key).map_err(core_error)?;
        let vault = vault::unlock_with_key(bytes, &key).map_err(core_error)?;
        Ok(Session { vault, key })
    }

    /// The key, so a refresh can reopen the file without the password.
    #[wasm_bindgen(js_name = exportKey)]
    pub fn export_key(&self) -> Vec<u8> {
        self.key.to_bytes()
    }

    /// The encrypted file contents, ready to save.
    pub fn seal(&self) -> Result<Vec<u8>, JsError> {
        vault::seal_with(&self.vault, &self.key).map_err(core_error)
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
            private: private::valuations(&self.vault),
            banks: bank::balances(&self.vault),
            metals: metal::holdings(&self.vault),
            fx: self.vault.prices.fx.iter().filter_map(|(c, q)| Some((c, q.last()?))).collect(),
            cash: reports::cash(ledger),
            trial_balance: reports::trial_balance(ledger),
            instruments: ledger.instruments.values().collect(),
            entries,
        })
    }

    /// Imports a statement. Returns the import report as JSON.
    pub fn import(&mut self, broker_toml: &str, file_name: &str, text: &str) -> Result<String, JsError> {
        let config = BrokerConfig::from_toml(broker_toml).map_err(core_error)?;
        // Import into a copy so a failure halfway leaves the book untouched.
        let mut ledger = self.vault.ledger.clone();
        let mut prices = self.vault.prices.clone();
        let report = import::import_statement(&mut ledger, &mut prices, &config, file_name, text).map_err(core_error)?;
        self.vault.ledger = ledger;
        self.vault.prices = prices;
        json(&report)
    }

    /// Realised gains for a year under a jurisdiction pack, as JSON.
    pub fn gains(&self, year: i32, pack_toml: &str) -> Result<String, JsError> {
        let pack = JurisdictionPack::from_toml(pack_toml).map_err(core_error)?;
        let rows: Vec<GainRow> = reports::realized_gains(&self.vault.ledger, year)
            .into_iter()
            .map(|r| GainRow { treatment: pack.classify(&r), realization: r })
            .collect();
        let summary = pack.summarize(&rows.iter().map(|r| r.realization.clone()).collect::<Vec<_>>());
        json(&Gains {
            rate: (pack.capital_gains.rate * pack.inclusion()).normalize().to_string(),
            inclusion: pack.capital_gains.inclusion_rate.map(|r| r.to_string()),
            marginal: pack.capital_gains.rate.to_string(),
            banded: !pack.capital_gains.bands.is_empty(),
            allowance: pack.capital_gains.annual_exempt_amount.map(|a| a.to_string()),
            form: pack.capital_gains.form.clone(),
            metals: pack.physical_metals.clone(),
            employment: reports::employment_income(&self.vault.ledger, year),
            employment_rule: pack.employment_income.clone(),
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
        self.vault.plan = serde_json::from_str(plan).map_err(bad_input)?;
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
        let mut input: WhatIfInput = serde_json::from_str(input).map_err(bad_input)?;
        (input.cash, input.invested) = self.holdings_value();
        json(&whatif::what_if(&input, &self.vault.plan))
    }

    /// What a trade would change, without changing anything. `input` is a
    /// trade as JSON; `pack_toml` the tax pack for the trade's year, if any;
    /// `what_if` the projection settings as for `whatIf`.
    #[wasm_bindgen(js_name = previewTrade)]
    pub fn preview_trade(&self, input: &str, pack_toml: Option<String>, what_if: &str) -> Result<String, JsError> {
        let input: TradeInput = serde_json::from_str(input).map_err(bad_input)?;
        let pack = pack_toml.map(|toml| JurisdictionPack::from_toml(&toml)).transpose().map_err(core_error)?;
        let settings: WhatIfInput = serde_json::from_str(what_if).map_err(bad_input)?;
        let v = &self.vault;
        json(&trade::outlook(&v.ledger, &v.prices, &v.plan, &input, pack.as_ref(), &settings).map_err(core_error)?)
    }

    /// Books a trade. Returns the new entry as JSON.
    #[wasm_bindgen(js_name = recordTrade)]
    pub fn record_trade(&mut self, input: &str) -> Result<String, JsError> {
        let input: TradeInput = serde_json::from_str(input).map_err(bad_input)?;
        json(&trade::record(&mut self.vault.ledger, &mut self.vault.prices, &input).map_err(core_error)?)
    }

    /// Removes the latest trade entered by hand.
    #[wasm_bindgen(js_name = undoTrade)]
    pub fn undo_trade(&mut self, id: &str) -> Result<(), JsError> {
        trade::undo(&mut self.vault.ledger, id).map_err(core_error)
    }

    /// Adds a bank account with its balance. `input` is JSON; returns the
    /// opening entry as JSON, or `null` for a zero balance.
    #[wasm_bindgen(js_name = addBank)]
    pub fn add_bank(&mut self, input: &str) -> Result<String, JsError> {
        let input: NewAccount = serde_json::from_str(input).map_err(bad_input)?;
        json(&bank::add(&mut self.vault, &input).map_err(core_error)?)
    }

    /// Records a bank account's balance. `input` is JSON; returns the entry
    /// for the difference as JSON, or `null` when nothing changed.
    #[wasm_bindgen(js_name = setBankBalance)]
    pub fn set_bank_balance(&mut self, id: &str, input: &str) -> Result<String, JsError> {
        let input: BalanceInput = serde_json::from_str(input).map_err(bad_input)?;
        json(&bank::set_balance(&mut self.vault, id, &input).map_err(core_error)?)
    }

    /// Removes a bank account and its entries.
    #[wasm_bindgen(js_name = removeBank)]
    pub fn remove_bank(&mut self, id: &str) -> Result<(), JsError> {
        bank::remove(&mut self.vault, id).map_err(core_error)
    }

    /// Records a purchase of coins or bars. `input` is JSON; returns the entry.
    #[wasm_bindgen(js_name = buyMetal)]
    pub fn buy_metal(&mut self, input: &str) -> Result<String, JsError> {
        let input: MetalInput = serde_json::from_str(input).map_err(bad_input)?;
        json(&metal::buy(&mut self.vault, &input).map_err(core_error)?)
    }

    /// Records a sale of coins or bars. `input` is JSON; returns the entry.
    #[wasm_bindgen(js_name = sellMetal)]
    pub fn sell_metal(&mut self, input: &str) -> Result<String, JsError> {
        let input: MetalInput = serde_json::from_str(input).map_err(bad_input)?;
        json(&metal::sell(&mut self.vault, &input).map_err(core_error)?)
    }

    /// Sets a metal's price per gram. `input` is JSON.
    #[wasm_bindgen(js_name = setMetalPrice)]
    pub fn set_metal_price(&mut self, input: &str) -> Result<(), JsError> {
        let input: MetalPrice = serde_json::from_str(input).map_err(bad_input)?;
        metal::set_price(&mut self.vault, &input).map_err(core_error)
    }

    /// Removes a purchase or sale of coins or bars.
    #[wasm_bindgen(js_name = removeMetalEntry)]
    pub fn remove_metal_entry(&mut self, entry_id: &str) -> Result<(), JsError> {
        metal::remove(&mut self.vault, entry_id).map_err(core_error)
    }

    /// Records a private holding and the payment into it. `input` is JSON;
    /// returns the new entry as JSON.
    #[wasm_bindgen(js_name = addPrivate)]
    pub fn add_private(&mut self, input: &str) -> Result<String, JsError> {
        let input: PrivateInput = serde_json::from_str(input).map_err(bad_input)?;
        json(&private::add(&mut self.vault, &input).map_err(core_error)?)
    }

    /// Sets a private holding's value in its own currency, or clears it when
    /// `amount` is empty. `date` is `YYYY-MM-DD`.
    #[wasm_bindgen(js_name = setPrivateValue)]
    pub fn set_private_value(&mut self, id: &str, date: &str, amount: &str) -> Result<(), JsError> {
        let estimate = if amount.trim().is_empty() {
            None
        } else {
            Some(Estimate { date: date.parse().map_err(bad_input)?, amount: amount.trim().parse().map_err(bad_input)? })
        };
        private::set_estimate(&mut self.vault, id, estimate).map_err(core_error)
    }

    /// Books a private holding's payout. `input` is JSON; returns the entry.
    #[wasm_bindgen(js_name = payOutPrivate)]
    pub fn pay_out_private(&mut self, id: &str, input: &str) -> Result<String, JsError> {
        let input: PayoutInput = serde_json::from_str(input).map_err(bad_input)?;
        json(&private::pay_out(&mut self.vault, id, &input).map_err(core_error)?)
    }

    /// Takes a private holding's payout back out of the books.
    #[wasm_bindgen(js_name = undoPrivatePayout)]
    pub fn undo_private_payout(&mut self, id: &str) -> Result<(), JsError> {
        private::undo_payout(&mut self.vault, id).map_err(core_error)
    }

    /// Removes a private holding and its entries.
    #[wasm_bindgen(js_name = removePrivate)]
    pub fn remove_private(&mut self, id: &str) -> Result<(), JsError> {
        private::remove(&mut self.vault, id).map_err(core_error)
    }

    /// Records a price typed in by the user. `date` is `YYYY-MM-DD`.
    #[wasm_bindgen(js_name = setPrice)]
    pub fn set_price(&mut self, symbol: &str, date: &str, amount: &str, currency: &str) -> Result<(), JsError> {
        let date = date.parse().map_err(bad_input)?;
        let amount = amount.trim().replace(',', ".").parse().map_err(bad_input)?;
        let price = Money::new(amount, Currency::new(currency).map_err(core_error)?);
        self.vault.prices.add_quote(symbol, Quote { date, price, source: QuoteSource::Manual });
        Ok(())
    }

    /// Records an exchange rate typed in by the user: base units for one `currency`.
    #[wasm_bindgen(js_name = setFxRate)]
    pub fn set_fx_rate(&mut self, currency: &str, date: &str, rate: &str) -> Result<(), JsError> {
        let date = date.parse().map_err(bad_input)?;
        let rate = rate.trim().replace(',', ".").parse().map_err(bad_input)?;
        self.vault.prices.add_fx(Currency::new(currency).map_err(core_error)?, FxQuote { date, rate, source: QuoteSource::Manual });
        Ok(())
    }

    /// Replaces the key after checking the current password. Saves made
    /// afterwards need the new password; older copies keep the old one.
    #[wasm_bindgen(js_name = changePassword)]
    pub fn change_password(&mut self, current: &str, new: &str) -> Result<(), JsError> {
        if !self.key.matches(current).map_err(core_error)? {
            return Err(problem("file.wrongPassword"));
        }
        self.key = SealingKey::derive(new, KdfParams::default()).map_err(core_error)?;
        Ok(())
    }

    /// Changes the country whose tax rules apply, e.g. `DE`.
    #[wasm_bindgen(js_name = setTaxResidency)]
    pub fn set_tax_residency(&mut self, country: &str) -> Result<(), JsError> {
        let country = country.trim().to_ascii_uppercase();
        if country.len() != 2 || !country.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(problem("country.invalid"));
        }
        self.vault.settings.tax_residency = country;
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
    let input: ProjectionInput = serde_json::from_str(input).map_err(bad_input)?;
    json(&plan::project(&input))
}

/// How well a broker config fits a statement, from 0 to 1.
#[wasm_bindgen(js_name = brokerFit)]
pub fn broker_fit(broker_toml: &str, text: &str) -> Result<f64, JsError> {
    let config = BrokerConfig::from_toml(broker_toml).map_err(core_error)?;
    Ok(import::fit(&config, text))
}

/// Checks a password against a file without keeping it open.
#[wasm_bindgen(js_name = isInvestFile)]
pub fn is_invest_file(bytes: &[u8]) -> bool {
    bytes.starts_with(b"INVEST\0")
}
