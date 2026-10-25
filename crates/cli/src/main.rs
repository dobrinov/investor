//! `investor`: create, import into and inspect `.invest` files from a terminal.
//!
//! The password is read from `INVESTOR_PASSWORD` when set, otherwise prompted.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use investor_core::import::{import_statement, BrokerConfig};
use investor_core::jurisdiction::{JurisdictionPack, Treatment};
use investor_core::{prices, reports};
use investor_core::vault::{self, KdfParams, Settings, Vault};
use investor_core::Currency;
use rust_decimal::Decimal;

#[derive(Parser)]
#[command(name = "investor", version, about = "Work with encrypted .invest files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new, empty .invest file.
    New {
        file: PathBuf,
        #[arg(long, default_value = "EUR")]
        base_currency: String,
        #[arg(long, default_value = "en")]
        language: String,
        #[arg(long, default_value = "BG")]
        residency: String,
    },
    /// Import broker statements using a broker config from brokers/.
    Import {
        file: PathBuf,
        #[arg(long)]
        broker: PathBuf,
        #[arg(required = true)]
        statements: Vec<PathBuf>,
    },
    /// Open positions and cash.
    Holdings { file: PathBuf },
    /// All journal entries with their postings.
    Journal { file: PathBuf },
    /// Every account's balance in the base currency.
    TrialBalance { file: PathBuf },
    /// Realised gains for a year, classified by a jurisdiction pack.
    Gains {
        file: PathBuf,
        #[arg(long)]
        year: i32,
        #[arg(long)]
        pack: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::New { file, base_currency, language, residency } => {
            if file.exists() {
                bail!("{} already exists", file.display());
            }
            let password = new_password()?;
            let settings = Settings { base_currency: Currency::new(&base_currency)?, language, tax_residency: residency };
            save(&file, &Vault::new(settings), &password)?;
            println!("Created {}. Keep it, and a backup — the password cannot be recovered.", file.display());
        }
        Command::Import { file, broker, statements } => {
            let password = password()?;
            let mut vault = load(&file, &password)?;
            let config = BrokerConfig::from_toml(&read(&broker)?)?;
            for statement in &statements {
                let name = statement.file_name().map_or_else(|| statement.display().to_string(), |n| n.to_string_lossy().into_owned());
                let report = import_statement(&mut vault.ledger, &mut vault.prices, &config, &name, &read(statement)?)?;
                println!("{name}: {} entries added, {} already imported", report.added, report.duplicates);
                for warning in &report.warnings {
                    println!("  ! {warning}");
                }
            }
            save(&file, &vault, &password)?;
        }
        Command::Holdings { file } => {
            let vault = load(&file, &password()?)?;
            let base = &vault.settings.base_currency;
            println!("{:<12} {:<10} {:>16} {:>14} {:>14} {:>12}", "Broker", "Symbol", "Units", format!("Cost ({base})"), "Value", "Price date");
            for v in prices::valuations(&vault.ledger, &vault.prices) {
                let value = v.value.map_or("?".to_owned(), money);
                let date = v.quote.as_ref().map_or("—".to_owned(), |q| q.date.to_string());
                println!("{:<12} {:<10} {:>16} {:>14} {:>14} {:>12}", v.broker, v.symbol, v.units.normalize(), money(v.cost), value, date);
            }
            println!("\n{:<12} {:<10} {:>16}", "Broker", "Cash", "Amount");
            for c in reports::cash(&vault.ledger) {
                println!("{:<12} {:<10} {:>16}", c.broker, c.currency, money(c.amount));
            }
        }
        Command::Journal { file } => {
            let vault = load(&file, &password()?)?;
            for e in vault.ledger.entries_by_date() {
                let venue = e.venue.as_ref().map_or(String::new(), |v| format!("  [venue {}{}]", v.code, if v.assumed { ", assumed" } else { "" }));
                println!("{} {}  {}{venue}", e.id, e.date, e.description);
                for p in &e.postings {
                    let units = p.units.map_or(String::new(), |u| format!("  {} units", u.normalize()));
                    println!("    {:<40} {:>14} {:>4} {:>14}{units}", p.account, money(p.amount.amount), p.amount.currency, money(p.base));
                }
                for note in &e.notes {
                    println!("    note: {note}");
                }
            }
        }
        Command::TrialBalance { file } => {
            let vault = load(&file, &password()?)?;
            let rows = reports::trial_balance(&vault.ledger);
            println!("{:<40} {:>14} {:>14}", "Account", "Debit", "Credit");
            for r in &rows {
                println!("{:<40} {:>14} {:>14}", r.account, blank_zero(r.debit), blank_zero(r.credit));
            }
            let (debit, credit): (Decimal, Decimal) = rows.iter().fold(Default::default(), |(d, c), r| (d + r.debit, c + r.credit));
            println!("{:<40} {:>14} {:>14}", "Total", money(debit), money(credit));
        }
        Command::Gains { file, year, pack } => {
            let vault = load(&file, &password()?)?;
            let pack = JurisdictionPack::from_toml(&read(&pack)?)?;
            if !pack.pack.verified {
                println!("! {} {} is a draft and has not been checked by a tax professional.\n", pack.pack.id, pack.pack.version);
            }
            let gains = reports::realized_gains(&vault.ledger, year);
            println!("{:<10} {:<8} {:>12} {:>12} {:>12} {:>10}  Treatment", "Date", "Symbol", "Proceeds", "Cost", "Gain", "Venue");
            for g in &gains {
                let treatment = match pack.classify(g) {
                    Treatment::Exempt if g.physical => "exempt",
                    Treatment::Exempt => "exempt (EU regulated market)",
                    Treatment::Taxable => "taxable",
                    Treatment::VenueUnknown => "venue unknown: check where it was sold",
                    Treatment::NotEstimated => "not estimated: see the pack's notes",
                };
                let venue = g.venue.as_ref().map_or("?".to_owned(), |v| v.code.clone());
                println!("{:<10} {:<8} {:>12} {:>12} {:>12} {:>10}  {treatment}", g.date, g.symbol, money(g.proceeds), money(g.cost), money(g.gain), venue);
            }
            let s = pack.summarize(&gains);
            println!("\nTaxable gains {}  losses {}  exempt {}  undecided {}", money(s.taxable_gains), money(s.taxable_losses), money(s.exempt), money(s.unknown));
            println!("Estimated tax at {}%: {} {}  ({})", pack.capital_gains.rate * Decimal::ONE_HUNDRED, money(s.tax_due), pack.pack.currency, pack.capital_gains.form);
        }
    }
    Ok(())
}

fn money(d: Decimal) -> String {
    format!("{:.2}", d.round_dp(2))
}

fn blank_zero(d: Decimal) -> String {
    if d.is_zero() { String::new() } else { money(d) }
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}

fn password() -> Result<String> {
    if let Ok(p) = std::env::var("INVESTOR_PASSWORD") {
        return Ok(p);
    }
    Ok(rpassword::prompt_password("Password: ")?)
}

fn new_password() -> Result<String> {
    if let Ok(p) = std::env::var("INVESTOR_PASSWORD") {
        return Ok(p);
    }
    let first = rpassword::prompt_password("New password: ")?;
    if first.chars().count() < 10 {
        bail!("use at least 10 characters; a short phrase of random words works well");
    }
    if rpassword::prompt_password("Repeat password: ")? != first {
        bail!("passwords do not match");
    }
    Ok(first)
}

fn load(path: &Path, password: &str) -> Result<Vault> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(vault::open(&bytes, password)?)
}

/// Writes next to the target first so a crash never leaves a half-written file.
fn save(path: &Path, vault: &Vault, password: &str) -> Result<()> {
    let bytes = vault::seal(vault, password, KdfParams::default())?;
    let tmp = path.with_extension("invest.tmp");
    fs::write(&tmp, bytes).with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
    Ok(())
}
