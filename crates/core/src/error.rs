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

    /// Something the person can fix, with a code the interface translates.
    #[error("{}", .0.english())]
    User(Problem),

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

/// A message for the person using the app: a stable code, such as
/// `trade.recorded`, and the values to fill in. The English text lives here
/// for the command line and the MCP server; the web app translates the code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub code: &'static str,
    pub args: Vec<(&'static str, String)>,
}

impl Problem {
    pub fn english(&self) -> String {
        self.args.iter().fold(english(self.code).to_owned(), |text, (k, v)| text.replace(&format!("{{{k}}}"), v))
    }
}

impl Error {
    pub fn user(code: &'static str) -> Self {
        Error::User(Problem { code, args: Vec::new() })
    }

    /// Adds a value to fill in; does nothing for other errors.
    pub fn arg(mut self, key: &'static str, value: impl std::fmt::Display) -> Self {
        if let Error::User(p) = &mut self {
            p.args.push((key, value.to_string()));
        }
        self
    }

    /// The code and values of an error the person can act on, if it is one.
    pub fn problem(&self) -> Option<Problem> {
        let p = |code, args: Vec<(&'static str, String)>| Some(Problem { code, args });
        match self {
            Error::User(problem) => Some(problem.clone()),
            Error::Decrypt => p("file.decrypt", vec![]),
            Error::NotInvestFile => p("file.notInvest", vec![]),
            Error::UnsupportedVersion(v) => p("file.newer", vec![("version", v.to_string())]),
            Error::InvalidCurrency(c) => p("currency.invalid", vec![("currency", c.clone())]),
            Error::Import { file, line, message } => p("import.line", vec![("file", file.clone()), ("line", line.to_string()), ("message", message.clone())]),
            _ => None,
        }
    }
}

fn english(code: &str) -> &'static str {
    match code {
        "name.empty" => "give it a name with at least one letter or digit",
        "fx.missing" => "enter the exchange rate: how many {base} for one {currency}",
        "fx.notPositive" => "the exchange rate must be more than zero",
        "bank.unknown" => "there is no bank account {id}",
        "bank.exists" => "there is already a bank account called {id}",
        "bank.brokerClash" => "{id} is already the name of a broker in your books; choose another name",
        "bank.negative" => "the balance cannot be negative",
        "bank.recorded" => "this balance is already recorded",
        "private.unknown" => "there is no private holding {id}",
        "private.amountNotPositive" => "the amount paid must be more than zero",
        "private.recorded" => "this holding is already recorded",
        "private.valueNegative" => "the value cannot be negative",
        "private.paidOut" => "this holding has already paid out",
        "private.payoutNegative" => "the amount received and the tax kept back cannot be negative, and the tax cannot be more than the amount",
        "private.payoutKind" => "choose how the payout was taxed",
        "private.payoutRecorded" => "this payout is already recorded",
        "private.payoutBeforePaid" => "the payout cannot be before the money was paid in",
        "trade.brokerChars" => "the broker name may use only letters, digits, - _ and .",
        "trade.symbolChars" => "the symbol may use only letters, digits, - _ and .",
        "trade.unitsNotPositive" => "the number of units must be more than zero",
        "trade.negative" => "prices and fees cannot be negative",
        "trade.recorded" => "this trade is already recorded",
        "trade.notEnough" => "only {held} {symbol} are held at {broker} on that date",
        "trade.refused" => "the trade cannot be booked: {reason}",
        "trade.undoLatest" => "only the latest trade entered by hand can be undone",
        "metal.weightNotPositive" => "the weight must be more than zero",
        "metal.amountNotPositive" => "the amount must be more than zero",
        "metal.recorded" => "this is already recorded",
        "metal.heldOnly" => "only {grams} g are held on that date",
        "metal.priceNotPositive" => "the price must be more than zero",
        "metal.noEntry" => "there is no such purchase or sale",
        "metal.neededBySale" => "a later sale needs this purchase; remove the sale first",
        "file.decrypt" => "wrong password, or the file is damaged",
        "file.notInvest" => "not an .invest file",
        "file.newer" => "this .invest file was made by a newer version of Investor (format {version})",
        "file.wrongPassword" => "wrong password",
        "currency.invalid" => "invalid currency code {currency}",
        "country.invalid" => "a country is two letters, e.g. DE",
        "input.invalid" => "check the numbers and dates you entered ({detail})",
        "import.line" => "{file}, line {line}: {message}",
        _ => "something went wrong",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problems_read_in_english_with_their_values() {
        let e = Error::user("fx.missing").arg("base", "EUR").arg("currency", "USD");
        assert_eq!(e.to_string(), "enter the exchange rate: how many EUR for one USD");
        assert_eq!(e.problem().unwrap().code, "fx.missing");
    }
}
