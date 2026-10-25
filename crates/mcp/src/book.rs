//! The `.invest` file, read again whenever it changes on disk, and the
//! sources its password may come from.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use anyhow::{Context, Result, bail};
use investor_core::vault::{self, SealingKey, Vault};

const KEYCHAIN_SERVICE: &str = "investor";

pub struct Book {
    path: PathBuf,
    /// The open vault and the file's modification time when it was read.
    open: Option<(Vault, SystemTime)>,
    /// Kept so a re-read after a save skips the slow key derivation.
    key: Option<SealingKey>,
}

impl Book {
    pub fn new(path: PathBuf) -> Self {
        Self { path, open: None, key: None }
    }

    /// The vault as it is on disk now. Unsaved work in the browser is not seen.
    pub fn vault(&mut self) -> Result<&Vault> {
        let modified = fs::metadata(&self.path)
            .and_then(|m| m.modified())
            .with_context(|| format!("cannot read {}: check the path given with --file", self.path.display()))?;
        if self.open.as_ref().is_none_or(|(_, at)| *at != modified) {
            let bytes = fs::read(&self.path)?;
            let kept = self.key.as_ref().and_then(|key| vault::unlock_with_key(&bytes, key).ok());
            let vault = match kept {
                Some(vault) => vault,
                None => {
                    let (vault, key) = vault::unlock(&bytes, &password(&self.path)?).map_err(|e| {
                        anyhow::anyhow!("{e}. If the password changed, store the new one: investor-mcp --file {} remember-password", self.path.display())
                    })?;
                    self.key = Some(key);
                    vault
                }
            };
            self.open = Some((vault, modified));
        }
        Ok(&self.open.as_ref().expect("just opened").0)
    }
}

fn full(path: &Path) -> String {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_owned()).display().to_string()
}

fn password(path: &Path) -> Result<String> {
    if let Ok(command) = std::env::var("INVESTOR_PASSWORD_COMMAND") {
        let out = Command::new("sh").arg("-c").arg(&command).output().context("running INVESTOR_PASSWORD_COMMAND")?;
        if !out.status.success() {
            bail!("INVESTOR_PASSWORD_COMMAND failed: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
        return Ok(String::from_utf8(out.stdout)?.trim_end_matches(['\n', '\r']).to_owned());
    }
    if cfg!(target_os = "macos")
        && let Ok(out) = Command::new("security").args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-a", &full(path), "-w"]).output()
        && out.status.success()
    {
        return Ok(String::from_utf8(out.stdout)?.trim_end_matches('\n').to_owned());
    }
    if let Ok(password) = std::env::var("INVESTOR_PASSWORD") {
        return Ok(password);
    }
    bail!(
        "no password for {file}. Fix it in a terminal (not in this chat): \
         on a Mac run `investor-mcp --file {file} remember-password`; elsewhere set INVESTOR_PASSWORD_COMMAND \
         to a command that prints the password, e.g. from your password manager. Then restart Claude Code.",
        file = path.display()
    )
}

/// Stores the password in the macOS Keychain; `security` asks for it, so it
/// never appears on a command line.
pub fn remember_password(path: &Path) -> Result<()> {
    if !cfg!(target_os = "macos") {
        bail!("the Keychain is macOS only; set INVESTOR_PASSWORD_COMMAND to a command that prints the password instead");
    }
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let account = full(path);
    println!("Enter the password of {account} (it is stored in your login Keychain).");
    let status = Command::new("security").args(["add-generic-password", "-U", "-s", KEYCHAIN_SERVICE, "-a", &account, "-w"]).status()?;
    if !status.success() {
        bail!("the Keychain did not store the password");
    }
    match vault::unlock(&bytes, &password(path)?) {
        Ok(_) => println!("Stored. Claude Code can now read this file; restart it if it is running."),
        Err(e) => {
            Command::new("security").args(["delete-generic-password", "-s", KEYCHAIN_SERVICE, "-a", &account]).output()?;
            bail!("{e}: nothing was stored, try again");
        }
    }
    Ok(())
}
