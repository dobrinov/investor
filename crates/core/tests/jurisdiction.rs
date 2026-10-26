use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::NaiveDate;
use investor_core::jurisdiction::{JurisdictionPack, Treatment};
use investor_core::reports::Realization;
use rust_decimal::Decimal;

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../jurisdictions")
}

fn pack(country: &str) -> JurisdictionPack {
    JurisdictionPack::from_toml(&fs::read_to_string(root().join(country).join("2026.toml")).unwrap()).unwrap()
}

fn sale(gain: &str) -> Realization {
    Realization {
        date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        entry_id: "JE-1".into(),
        symbol: "X".into(),
        units: dec("1"),
        proceeds: dec("0"),
        cost: dec("0"),
        gain: dec(gain),
        venue: None,
        physical: false,
        held_from: None,
        private: false,
        crypto: false,
    }
}

#[test]
fn a_crypto_sale_is_taxed_whatever_the_venue() {
    let pack = pack("bg");
    let coin = Realization { crypto: true, ..sale("100") };
    assert_eq!(pack.classify(&sale("100")), Treatment::VenueUnknown);
    assert_eq!(pack.classify(&coin), Treatment::Taxable);
}

#[test]
fn every_pack_parses_and_names_its_own_country() {
    let mut count = 0;
    for dir in fs::read_dir(root()).unwrap() {
        let dir = dir.unwrap().path();
        for file in fs::read_dir(&dir).unwrap() {
            let file = file.unwrap().path();
            let pack = JurisdictionPack::from_toml(&fs::read_to_string(&file).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
            let folder = dir.file_name().unwrap().to_string_lossy().to_uppercase();
            assert_eq!(pack.pack.country, folder, "{}", file.display());
            assert!(!pack.pack.verified, "{} claims to be verified", file.display());
            count += 1;
        }
    }
    assert!(count >= 11);
}

#[test]
fn the_annual_allowance_comes_off_the_net_gain() {
    let de = pack("de");
    let s = de.summarize(&[sale("1500"), sale("-200")]);
    assert_eq!((s.allowance_used, s.tax_due), (dec("1000"), dec("79.13")));
    let small = de.summarize(&[sale("600")]);
    assert_eq!((small.allowance_used, small.tax_due), (dec("600"), dec("0")));
}

#[test]
fn bands_tax_each_slice_at_its_own_rate() {
    let es = pack("es");
    // 6,000 × 19% + 44,000 × 21% + 10,000 × 23%
    assert_eq!(es.summarize(&[sale("60000")]).tax_due, dec("12680.00"));
    assert_eq!(es.tax_on(dec("4000")), dec("760"));
    // Above the last limit the top rate applies.
    assert_eq!(es.tax_on(dec("400000")), dec("1140") + dec("9240") + dec("34500") + dec("27000") + dec("30000"));
}

#[test]
fn physical_metal_follows_the_metals_rule_and_holding_period() {
    use investor_core::jurisdiction::Treatment;
    let bar = |from: (i32, u32, u32), gain: &str| Realization {
        physical: true,
        held_from: NaiveDate::from_ymd_opt(from.0, from.1, from.2),
        ..sale(gain)
    };
    let de = pack("de");
    assert_eq!(de.classify(&bar((2025, 5, 31), "500")), Treatment::Exempt, "more than a year");
    assert_eq!(de.classify(&bar((2025, 6, 1), "500")), Treatment::NotEstimated, "exactly a year");
    
    assert_eq!(pack("bg").classify(&bar((2026, 5, 1), "500")), Treatment::Exempt);
    assert_eq!(pack("it").classify(&bar((2026, 5, 1), "500")), Treatment::Taxable);
    let s = de.summarize(&[bar((2026, 1, 1), "300"), bar((2020, 1, 1), "900"), sale("100")]);
    assert_eq!((s.not_estimated, s.exempt, s.taxable_gains), (dec("300"), dec("900"), dec("100")));
}

#[test]
fn us_short_term_gains_are_left_out_and_canada_taxes_half() {
    use investor_core::jurisdiction::Treatment;
    let held = |from: (i32, u32, u32), gain: &str| Realization { held_from: NaiveDate::from_ymd_opt(from.0, from.1, from.2), ..sale(gain) };
    let us = pack("us");
    assert_eq!(us.classify(&held((2025, 5, 31), "1000")), Treatment::Taxable);
    assert_eq!(us.classify(&held((2025, 6, 1), "1000")), Treatment::NotEstimated, "a year or less is short-term");
    let s = us.summarize(&[held((2024, 1, 1), "1000"), held((2026, 1, 1), "400")]);
    assert_eq!((s.tax_due, s.not_estimated), (dec("150.00"), dec("400")));

    let ca = pack("ca");
    assert_eq!(ca.summarize(&[sale("1000")]).tax_due, dec("150.00"));
    assert_eq!(ca.sale_estimate(dec("1000")), dec("150.00"));
}

#[test]
fn a_private_stake_paid_out_as_a_gain_is_taxable_without_a_venue() {
    use investor_core::jurisdiction::Treatment;
    let stake = Realization { private: true, ..sale("1000") };
    assert_eq!(pack("bg").classify(&stake), Treatment::Taxable);
    assert_eq!(pack("bg").classify(&sale("1000")), Treatment::VenueUnknown);
}
