use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::NaiveDate;
use investor_core::jurisdiction::JurisdictionPack;
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
    }
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
