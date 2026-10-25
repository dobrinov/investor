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
        let net = summary.taxable_gains + summary.taxable_losses;
        summary.tax_due = (net.max(Decimal::ZERO) * self.capital_gains.rate).round_dp(2);
        summary
    }
}
