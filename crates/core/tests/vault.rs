use chrono::NaiveDate;
use investor_core::ledger::{JournalEntry, Posting};
use investor_core::vault::{self, KdfParams, Settings, Vault};
use investor_core::{Currency, Error, Money};
use rust_decimal::Decimal;

const FAST: KdfParams = KdfParams { memory_kib: 1024, iterations: 1, lanes: 1 };

fn sample() -> Vault {
    let eur = Currency::new("EUR").unwrap();
    let mut vault = Vault::new(Settings { base_currency: eur.clone(), language: "bg".into(), tax_residency: "BG".into() });
    let amount = Decimal::new(25000, 2);
    vault
        .ledger
        .post(JournalEntry {
            id: String::new(),
            date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            description: "Deposit".into(),
            postings: vec![
                Posting::new("Assets:Cash:IBKR:EUR", Money::new(amount, eur.clone()), amount),
                Posting::new("Equity:Contributions", Money::new(-amount, eur), -amount),
            ],
            venue: None,
            source: None,
            notes: vec![],
            provisional: None,
        })
        .unwrap();
    vault
}

#[test]
fn round_trips() {
    let vault = sample();
    let bytes = vault::seal(&vault, "correct horse", FAST).unwrap();
    assert!(bytes.starts_with(b"INVEST\0"));
    assert!(!bytes.windows(7).any(|w| w == b"Deposit"), "ledger text must not appear in the file");
    assert_eq!(vault::open(&bytes, "correct horse").unwrap(), vault);
}

#[test]
fn resaves_with_the_unlocked_key() {
    let vault = sample();
    let bytes = vault::seal(&vault, "correct horse", FAST).unwrap();
    let (opened, key) = vault::unlock(&bytes, "correct horse").unwrap();
    let again = vault::seal_with(&opened, &key).unwrap();
    assert_ne!(again, bytes, "a fresh nonce is used for every save");
    assert_eq!(&again[..37], &bytes[..37], "same salt and parameters");
    assert_eq!(vault::open(&again, "correct horse").unwrap(), vault);
}

#[test]
fn rejects_wrong_password_and_tampering() {
    let bytes = vault::seal(&sample(), "correct horse", FAST).unwrap();
    assert!(matches!(vault::open(&bytes, "wrong"), Err(Error::Decrypt)));

    let mut tampered = bytes.clone();
    tampered[25] ^= 1; // inside the salt, which is authenticated
    assert!(matches!(vault::open(&tampered, "correct horse"), Err(Error::Decrypt)));

    assert!(matches!(vault::open(b"not a vault at all, just text......................", "x"), Err(Error::NotInvestFile)));
}

#[test]
fn refuses_unbalanced_entries() {
    let mut vault = sample();
    let eur = vault.settings.base_currency.clone();
    let err = vault
        .ledger
        .post(JournalEntry {
            id: String::new(),
            date: NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
            description: "Broken".into(),
            postings: vec![
                Posting::new("Assets:Cash:IBKR:EUR", Money::new(Decimal::ONE, eur.clone()), Decimal::ONE),
                Posting::new("Equity:Contributions", Money::new(Decimal::NEGATIVE_ONE, eur), Decimal::new(-99, 2)),
            ],
            venue: None,
            source: None,
            notes: vec![],
            provisional: None,
        })
        .unwrap_err();
    assert!(matches!(err, Error::Unbalanced { .. }), "{err}");
}
