//! Tax rules for one country and tax year, loaded from
//! `jurisdictions/<country>/<year>.toml`.

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ledger::Venue;
use crate::reports::Realization;
use crate::{Currency, Result};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JurisdictionPack {
    pub pack: PackInfo,
    pub capital_gains: CapitalGains,
    pub dividends: Dividends,
    #[serde(default)]
    pub deadlines: BTreeMap<String, String>,
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
    pub notes: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Treatment {
    Exempt,
    Taxable,
    /// The venue is missing, so the rules cannot decide.
    VenueUnknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GainsSummary {
    pub taxable_gains: Decimal,
    pub taxable_losses: Decimal,
    pub tax_due: Decimal,
    pub exempt: Decimal,
    pub unknown: Decimal,
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

    /// Losses on taxable sales offset taxable gains within the year.
    pub fn summarize(&self, realizations: &[Realization]) -> GainsSummary {
        let mut summary = GainsSummary::default();
        for r in realizations {
            match self.treatment(r.venue.as_ref()) {
                Treatment::Exempt => summary.exempt += r.gain,
                Treatment::VenueUnknown => summary.unknown += r.gain,
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
