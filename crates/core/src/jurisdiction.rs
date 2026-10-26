//! Tax rules for one country and tax year, loaded from
//! `jurisdictions/<country>/<year>.toml`.

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ledger::Venue;
use crate::reports::Realization;
use crate::{Currency, Result};

/// Text a pack gives in one language, or in several by language code
/// (`{ en = "…", bg = "…" }`), English being the fallback.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Text {
    One(String),
    Many(BTreeMap<String, String>),
}

impl Text {
    /// The text in `language`, else English, else any.
    pub fn get(&self, language: &str) -> &str {
        match self {
            Text::One(text) => text,
            Text::Many(texts) => texts.get(language).or_else(|| texts.get("en")).or_else(|| texts.values().next()).map_or("", String::as_str),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JurisdictionPack {
    pub pack: PackInfo,
    pub capital_gains: CapitalGains,
    pub dividends: Dividends,
    /// Coins and bars. Without this section their sales are not estimated.
    #[serde(default)]
    pub physical_metals: Option<PhysicalMetals>,
    /// Pay from work that is not salary, such as a co-investment paid out
    /// through payroll. Without this section it is not estimated.
    #[serde(default)]
    pub employment_income: Option<EmploymentIncome>,
    #[serde(default)]
    pub deadlines: BTreeMap<String, String>,
    /// What goes where on the return, and how to file it.
    #[serde(default)]
    pub declaration: Option<crate::declaration::Filing>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackInfo {
    pub id: String,
    pub version: String,
    pub country: String,
    pub authority: String,
    #[serde(default)]
    pub website: Option<String>,
    pub tax_year: i32,
    pub currency: Currency,
    /// False until a tax professional has reviewed the rules.
    pub verified: bool,
    #[serde(default)]
    pub notes: Option<Text>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapitalGains {
    pub rate: Decimal,
    pub form: String,
    #[serde(default)]
    pub exempt_on_eu_regulated_market: bool,
    /// Venue codes as brokers write them (`IBIS`, `BVME.ETF`, MICs…).
    #[serde(default)]
    pub eu_regulated_venues: Vec<String>,
    /// Net gains up to this much a year are tax-free, in the pack's currency.
    #[serde(default)]
    pub annual_exempt_amount: Option<Decimal>,
    /// Progressive rates on the net taxable gain, lowest first. When given,
    /// they replace `rate` for the year's total; `rate` stays the estimate
    /// for a single sale.
    #[serde(default)]
    pub bands: Vec<RateBand>,
    /// Units held less than this many months give a short-term gain, taxed
    /// with income at a rate the pack cannot know, so not estimated.
    #[serde(default)]
    pub short_term_months: Option<u32>,
    /// The share of the net gain that is taxed, e.g. 0.5 in Canada.
    #[serde(default)]
    pub inclusion_rate: Option<Decimal>,
    /// How the cost of units sold is worked out for tax.
    #[serde(default)]
    pub cost_basis: CostBasis,
}

/// First in, first out (most countries), or the average cost of all the
/// identical units held (Canada's adjusted cost base).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CostBasis {
    #[default]
    Fifo,
    Average,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RateBand {
    /// Upper end of the band; none for the last one.
    #[serde(default)]
    pub up_to: Option<Decimal>,
    pub rate: Decimal,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dividends {
    pub rate: Decimal,
    pub form: String,
    #[serde(default)]
    pub foreign_tax_credit: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmploymentIncome {
    /// A flat rate; none when it depends on the person's income.
    #[serde(default)]
    pub rate: Option<Decimal>,
    pub form: String,
    #[serde(default)]
    pub note: Option<Text>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalMetals {
    pub treatment: MetalTreatment,
    /// Sales of metal held at least this long are tax-free.
    #[serde(default)]
    pub exempt_after_months: Option<u32>,
    #[serde(default)]
    pub note: Option<Text>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetalTreatment {
    Exempt,
    /// Taxed with shares, at the capital gains rate.
    CapitalGains,
    /// Taxed some other way, e.g. as income; the estimate leaves it out.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Treatment {
    Exempt,
    Taxable,
    /// The venue is missing, so the rules cannot decide.
    VenueUnknown,
    /// Taxed outside this estimate; the pack's notes say how.
    NotEstimated,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GainsSummary {
    pub taxable_gains: Decimal,
    pub taxable_losses: Decimal,
    pub tax_due: Decimal,
    pub exempt: Decimal,
    pub unknown: Decimal,
    /// Gains taxed outside this estimate, e.g. physical metal taxed as income.
    pub not_estimated: Decimal,
    /// The part of the net gain the annual allowance made tax-free.
    pub allowance_used: Decimal,
}

impl JurisdictionPack {
    pub fn from_toml(text: &str) -> Result<Self> {
        Ok(toml::from_str(text)?)
    }

    pub fn treatment(&self, venue: Option<&Venue>) -> Treatment {
        if !self.capital_gains.exempt_on_eu_regulated_market {
            return Treatment::Taxable;
        }
        match venue {
            None => Treatment::VenueUnknown,
            Some(v) if self.capital_gains.eu_regulated_venues.iter().any(|e| e.eq_ignore_ascii_case(&v.code)) => {
                Treatment::Exempt
            }
            Some(_) => Treatment::Taxable,
        }
    }

    /// The share of a gain that is taxed: all of it unless the pack says.
    pub fn inclusion(&self) -> Decimal {
        self.capital_gains.inclusion_rate.unwrap_or(Decimal::ONE)
    }

    /// Tax on one sale's gain at the flat rate, before the year's netting.
    pub fn sale_estimate(&self, gain: Decimal) -> Decimal {
        (gain.max(Decimal::ZERO) * self.inclusion() * self.capital_gains.rate).round_dp(2)
    }

    /// How one sale is taxed: by venue for securities, by the metals rule
    /// and holding period for coins and bars.
    pub fn classify(&self, sale: &Realization) -> Treatment {
        let held_for = |months: u32| match sale.held_from {
            Some(from) => from.checked_add_months(chrono::Months::new(months)).is_some_and(|d| d < sale.date),
            None => true,
        };
        if !sale.physical {
            // A private stake is not sold on a market, so no venue exemption applies.
            let by_venue = if sale.private { Treatment::Taxable } else { self.treatment(sale.venue.as_ref()) };
            return match (by_venue, self.capital_gains.short_term_months) {
                (Treatment::Taxable, Some(months)) if !held_for(months) => Treatment::NotEstimated,
                (treatment, _) => treatment,
            };
        }
        let Some(rule) = &self.physical_metals else { return Treatment::NotEstimated };
        let held_long = match (rule.exempt_after_months, sale.held_from) {
            (Some(months), Some(from)) => from.checked_add_months(chrono::Months::new(months)).is_some_and(|d| d < sale.date),
            _ => false,
        };
        match rule.treatment {
            _ if held_long => Treatment::Exempt,
            MetalTreatment::Exempt => Treatment::Exempt,
            MetalTreatment::CapitalGains => Treatment::Taxable,
            MetalTreatment::Other => Treatment::NotEstimated,
        }
    }

    /// Losses on taxable sales offset taxable gains within the year.
    pub fn summarize(&self, realizations: &[Realization]) -> GainsSummary {
        let mut summary = GainsSummary::default();
        for r in realizations {
            match self.classify(r) {
                Treatment::Exempt => summary.exempt += r.gain,
                Treatment::VenueUnknown => summary.unknown += r.gain,
                Treatment::NotEstimated => summary.not_estimated += r.gain,
                Treatment::Taxable if r.gain.is_sign_negative() => summary.taxable_losses += r.gain,
                Treatment::Taxable => summary.taxable_gains += r.gain,
            }
        }
        let net = (summary.taxable_gains + summary.taxable_losses).max(Decimal::ZERO);
        summary.allowance_used = net.min(self.capital_gains.annual_exempt_amount.unwrap_or_default().max(Decimal::ZERO));
        summary.tax_due = self.tax_on(net - summary.allowance_used).round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
        summary
    }

    /// Tax on a year's net taxable gain: by band when the pack has bands,
    /// otherwise at the flat rate.
    pub fn tax_on(&self, gain: Decimal) -> Decimal {
        let gain = gain * self.inclusion();
        if self.capital_gains.bands.is_empty() {
            return gain * self.capital_gains.rate;
        }
        let (mut tax, mut floor) = (Decimal::ZERO, Decimal::ZERO);
        for band in &self.capital_gains.bands {
            let top = band.up_to.unwrap_or(Decimal::MAX).min(gain);
            if top > floor {
                tax += (top - floor) * band.rate;
            }
            match band.up_to {
                Some(up_to) if up_to < gain => floor = up_to,
                _ => break,
            }
        }
        tax
    }
}
