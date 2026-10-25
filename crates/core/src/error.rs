use rust_decimal::Decimal;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid currency code {0:?}")]
    InvalidCurrency(String),

    #[error("journal entry {id} is invalid: {reason}")]
    InvalidEntry { id: String, reason: String },

    #[error("journal entry {id} does not balance: off by {diff} in the base currency")]
    Unbalanced { id: String, diff: Decimal },

    #[error("cannot sell {requested} units from {account}: only {held} held")]
    InsufficientUnits { account: String, requested: Decimal, held: Decimal },

    #[error("not an .invest file")]
    NotInvestFile,

    #[error("this .invest file was made by a newer version of Investor (format {0})")]
    UnsupportedVersion(u16),

    #[error("wrong password, or the file is damaged")]
    Decrypt,

    #[error("encryption failed: {0}")]
    Crypto(String),

    #[error("invalid config: {0}")]
    Config(String),

    #[error("{0}")]
    Trade(String),

    #[error("{file}, line {line}: {message}")]
    Import { file: String, line: usize, message: String },

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Csv(#[from] csv::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
