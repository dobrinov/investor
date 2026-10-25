//! The `.invest` file: the whole book, encrypted with a key derived from the
//! user's password.
//!
//! Layout (all integers little-endian):
//!
//! | bytes | field                                   |
//! |-------|-----------------------------------------|
//! | 7     | magic `INVEST\0`                        |
//! | 2     | format version (1)                      |
//! | 12    | Argon2id memory KiB, iterations, lanes  |
//! | 16    | salt                                    |
//! | 12    | AES-256-GCM nonce                       |
//! | rest  | ciphertext of the JSON document         |
//!
//! The header is authenticated as associated data, so changing the KDF
//! parameters or salt makes decryption fail instead of silently weakening it.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::ledger::Ledger;
use crate::{Currency, Error, Result};

const MAGIC: &[u8; 7] = b"INVEST\0";
const FORMAT_VERSION: u16 = 1;
const HEADER_LEN: usize = 7 + 2 + 12 + 16 + 12;
/// Refuse files asking for absurd work so a crafted file cannot hang the app.
const MAX_MEMORY_KIB: u32 = 1 << 21;
const MAX_ITERATIONS: u32 = 16;

/// Everything stored in a `.invest` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vault {
    pub settings: Settings,
    pub ledger: Ledger,
}

/// Preferences that travel with the file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub base_currency: Currency,
    /// BCP 47 language tag of the interface, e.g. `en` or `bg`.
    pub language: String,
    /// ISO 3166 country of tax residency, e.g. `BG`.
    pub tax_residency: String,
}

impl Vault {
    pub fn new(settings: Settings) -> Self {
        let ledger = Ledger::new(settings.base_currency.clone());
        Self { settings, ledger }
    }
}

/// Argon2id cost. The default suits a laptop; tests use cheaper settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub lanes: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self { memory_kib: 64 * 1024, iterations: 3, lanes: 1 }
    }
}

pub fn seal(vault: &Vault, password: &str, params: KdfParams) -> Result<Vec<u8>> {
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let mut header = Vec::with_capacity(HEADER_LEN);
    header.extend_from_slice(MAGIC);
    header.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    for value in [params.memory_kib, params.iterations, params.lanes] {
        header.extend_from_slice(&value.to_le_bytes());
    }
    header.extend_from_slice(&salt);
    header.extend_from_slice(&nonce);

    let key = derive_key(password, &salt, params)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| Error::Crypto(e.to_string()))?;
    let plain = serde_json::to_vec(vault)?;
    let sealed = cipher
        .encrypt(Nonce::from_slice(&nonce), Payload { msg: &plain, aad: &header })
        .map_err(|e| Error::Crypto(e.to_string()))?;

    header.extend_from_slice(&sealed);
    Ok(header)
}

pub fn open(bytes: &[u8], password: &str) -> Result<Vault> {
    if bytes.len() < HEADER_LEN || &bytes[..7] != MAGIC {
        return Err(Error::NotInvestFile);
    }
    let version = u16::from_le_bytes([bytes[7], bytes[8]]);
    if version != FORMAT_VERSION {
        return Err(Error::UnsupportedVersion(version));
    }
    let u32_at = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().expect("4 bytes"));
    let params = KdfParams { memory_kib: u32_at(9), iterations: u32_at(13), lanes: u32_at(17) };
    if params.memory_kib > MAX_MEMORY_KIB || params.iterations > MAX_ITERATIONS || params.lanes > 16 {
        return Err(Error::Decrypt);
    }
    let (header, sealed) = bytes.split_at(HEADER_LEN);
    let salt = &header[21..37];
    let nonce = &header[37..49];

    let key = derive_key(password, salt, params)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| Error::Crypto(e.to_string()))?;
    let plain = cipher
        .decrypt(Nonce::from_slice(nonce), Payload { msg: sealed, aad: header })
        .map_err(|_| Error::Decrypt)?;
    Ok(serde_json::from_slice(&plain)?)
}

fn derive_key(password: &str, salt: &[u8], params: KdfParams) -> Result<[u8; 32]> {
    let params = Params::new(params.memory_kib, params.iterations, params.lanes, Some(32))
        .map_err(|_| Error::Decrypt)?;
    let mut key = [0u8; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(key)
}
