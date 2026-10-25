use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use chrono::{DateTime, NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};

use super::config::{AmountStyle, BrokerConfig, Layout, RecordKind, RecordRule};
use super::event::{Event, EventKind, FeeCharge, FeeKind, Fx};
use crate::ledger::{Instrument, Source, Venue};
use crate::{Currency, Error, Money, Result};

#[derive(Debug, Clone, Default)]
pub struct ParsedFile {
    pub events: Vec<Event>,
    pub warnings: Vec<String>,
}

struct Row {
    line: usize,
    section: Option<String>,
    cells: HashMap<String, String>,
    raw: String,
}

impl Row {
    fn get(&self, column: Option<&str>) -> Option<&str> {
        let value = self.cells.get(column?)?.trim();
        (!value.is_empty()).then_some(value)
    }
}

pub fn parse(config: &BrokerConfig, file_name: &str, text: &str) -> Result<ParsedFile> {
    let rows = read_rows(config, text)?;
    let mut parsed = ParsedFile::default();
    let mut unmatched: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for row in &rows {
        let Some(rule) = config.records.iter().find(|r| applies(r, row)) else {
            let label = match &row.section {
                Some(section) => format!("section {section}"),
                None => "rows".into(),
            };
            unmatched.entry(label).or_default().push(row.line);
            continue;
        };
        if rule.kind == RecordKind::Ignore {
            continue;
        }
        let event = build_event(config, rule, row, file_name).map_err(|message| Error::Import {
            file: file_name.to_owned(),
            line: row.line,
            message,
        })?;
        parsed.events.push(event);
    }

    for (label, lines) in unmatched {
        let shown: Vec<String> = lines.iter().take(5).map(usize::to_string).collect();
        let more = if lines.len() > 5 { ", …" } else { "" };
        parsed.warnings.push(format!(
            "{file_name}: {} {label} not recognised by the {} config and skipped (lines {}{more})",
            lines.len(),
            config.broker.name,
            shown.join(", "),
        ));
    }
    Ok(parsed)
}

fn applies(rule: &RecordRule, row: &Row) -> bool {
    rule.section.as_deref().is_none_or(|s| row.section.as_deref() == Some(s))
        && rule.when.iter().all(|c| c.matches(row.cells.get(&c.column).map(String::as_str)))
}

fn read_rows(config: &BrokerConfig, text: &str) -> Result<Vec<Row>> {
    let text = text.trim_start_matches('\u{feff}');
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(config.csv.delimiter as u8)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut rows = Vec::new();
    let mut headers: HashMap<String, Vec<String>> = HashMap::new();
    for record in reader.records() {
        let record = record?;
        let line = record.position().map_or(0, |p| p.line() as usize);
        let cells: Vec<&str> = record.iter().collect();
        if cells.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let (section, values) = match config.csv.layout {
            Layout::Table => {
                if !headers.contains_key("") {
                    headers.insert(String::new(), cells.iter().map(|c| c.trim().to_owned()).collect());
                    continue;
                }
                (String::new(), cells.as_slice())
            }
            Layout::Sectioned => {
                let section = cells.get(1).copied().unwrap_or_default().to_owned();
                match cells[0] {
                    "HEADER" => {
                        headers.insert(section, cells[2..].iter().map(|c| c.trim().to_owned()).collect());
                        continue;
                    }
                    "DATA" => (section, &cells[2..]),
                    // BOF/EOF and other trailer records carry no data.
                    _ => continue,
                }
            }
        };
        let Some(names) = headers.get(&section) else {
            return Err(Error::Import {
                file: String::new(),
                line,
                message: format!("data for section {section} comes before its HEADER row"),
            });
        };
        rows.push(Row {
            line,
            section: (config.csv.layout == Layout::Sectioned).then_some(section.clone()),
            cells: names.iter().cloned().zip(values.iter().map(|v| (*v).to_owned())).collect(),
            raw: format!("{section}\u{1f}{}", values.join("\u{1f}")),
        });
    }
    Ok(rows)
}

fn build_event(config: &BrokerConfig, rule: &RecordRule, row: &Row, file: &str) -> std::result::Result<Event, String> {
    let f = &rule.fields;
    let date_text = row.get(f.date.as_deref()).ok_or("missing date")?;
    let date = parse_date(date_text, &config.csv.date_format)?;

    let amount_text = row.get(f.amount.as_deref()).ok_or("missing amount")?;
    let amount = parse_money(amount_text, row.get(f.currency.as_deref()), config.csv.amounts)?;

    let fx = match row.get(f.fx_rate.as_deref()) {
        Some(rate) => Some(Fx { currency: amount.currency.clone(), rate: parse_decimal(rate)?, mode: config.csv.fx_rate }),
        None => None,
    };

    let instrument = row.get(f.symbol.as_deref()).map(|symbol| Instrument {
        symbol: symbol.to_owned(),
        name: row.get(f.name.as_deref()).map(str::to_owned),
        isin: row.get(f.isin.as_deref()).map(str::to_owned),
        currency: Some(amount.currency.clone()),
    });
    let require_instrument = || instrument.clone().ok_or_else(|| "missing symbol".to_owned());
    let units = || -> std::result::Result<Decimal, String> {
        parse_decimal(row.get(f.quantity.as_deref()).ok_or("missing quantity")?)
    };

    let venue = match row.get(f.venue.as_deref()) {
        Some(code) => Some(Venue { code: code.to_owned(), assumed: false }),
        None => config.broker.default_venue.clone().map(|code| Venue { code, assumed: true }),
    };

    let mut fees = Vec::new();
    for column in &rule.fees {
        let Some(value) = row.get(Some(&column.column)) else { continue };
        let value = parse_decimal(value)?;
        if value.is_zero() {
            continue;
        }
        let currency = match row.get(column.currency_column.as_deref()) {
            Some(code) => Currency::new(code).map_err(|e| e.to_string())?,
            None => amount.currency.clone(),
        };
        fees.push(FeeCharge { kind: column.kind, amount: Money::new(value.abs(), currency) });
    }

    let kind = match rule.kind {
        RecordKind::Buy => EventKind::Buy { instrument: require_instrument()?, units: units()?.abs(), amount: amount.abs(), fees },
        RecordKind::Sell => EventKind::Sell { instrument: require_instrument()?, units: units()?.abs(), amount: amount.abs(), fees },
        RecordKind::Deposit => EventKind::Deposit { amount: amount.abs() },
        RecordKind::Withdrawal => EventKind::Withdrawal { amount: amount.abs() },
        RecordKind::CashTransfer if amount.amount.is_sign_negative() => EventKind::Withdrawal { amount: amount.abs() },
        RecordKind::CashTransfer => EventKind::Deposit { amount },
        RecordKind::Dividend => EventKind::Dividend { instrument: require_instrument()?, amount },
        RecordKind::WithholdingTax => EventKind::WithholdingTax { instrument, amount },
        RecordKind::Fee => EventKind::Fee { kind: rule.fee_kind.unwrap_or(FeeKind::Other), amount },
        RecordKind::Interest => EventKind::Interest { amount },
        RecordKind::FxPairTrade => {
            let pair = row.get(f.symbol.as_deref()).ok_or("missing currency pair")?;
            let (first, _) = pair.split_once('.').ok_or_else(|| format!("{pair:?} is not a currency pair like EUR.USD"))?;
            let first = Currency::new(first).map_err(|e| e.to_string())?;
            let units = units()?;
            let first_leg = Money::new(units.abs(), first);
            if units.is_sign_positive() {
                EventKind::FxConversion { sold: amount.abs(), bought: first_leg, fees }
            } else {
                EventKind::FxConversion { sold: first_leg, bought: amount.abs(), fees }
            }
        }
        RecordKind::PositionSnapshot => EventKind::PositionSnapshot { instrument: require_instrument()?, units: units()?, cost: amount },
        RecordKind::Ignore => unreachable!("ignored rows are skipped before building events"),
    };

    let fingerprint = match row.get(f.external_id.as_deref()) {
        Some(id) => format!("{}:{}:{id}", config.broker.id, row.section.as_deref().unwrap_or("")),
        None => {
            let digest = Sha256::digest(format!("{}\u{1f}{}", config.broker.id, row.raw));
            let hex: String = digest.iter().take(12).map(|b| format!("{b:02x}")).collect();
            format!("{}:row:{hex}", config.broker.id)
        }
    };

    Ok(Event {
        date,
        kind,
        fx,
        venue,
        source: Source { broker: config.broker.id.clone(), file: file.to_owned(), line: row.line, fingerprint },
    })
}

fn parse_date(text: &str, format: &str) -> std::result::Result<NaiveDate, String> {
    if format == "rfc3339" {
        return DateTime::parse_from_rfc3339(text)
            .map(|dt| dt.date_naive())
            .map_err(|e| format!("date {text:?} is not RFC 3339: {e}"));
    }
    NaiveDate::parse_from_str(text, format)
        .or_else(|_| NaiveDateTime::parse_from_str(text, format).map(|dt| dt.date()))
        // Same date with a time appended, e.g. `2026-10-23;09:30:00`.
        .or_else(|_| NaiveDate::parse_from_str(text.get(..10).unwrap_or(text), "%Y-%m-%d"))
        .map_err(|_| format!("date {text:?} does not match {format:?}"))
}

fn parse_decimal(text: &str) -> std::result::Result<Decimal, String> {
    let cleaned: String = text.trim().chars().filter(|c| *c != ',' && *c != ' ').collect();
    Decimal::from_str(&cleaned)
        .or_else(|_| Decimal::from_scientific(&cleaned))
        .map_err(|_| format!("{text:?} is not a number"))
}

fn parse_money(text: &str, currency: Option<&str>, style: AmountStyle) -> std::result::Result<Money, String> {
    let (code, number) = match style {
        AmountStyle::CurrencyPrefixed => match text.split_once(char::is_whitespace) {
            Some((code, number)) if code.len() == 3 && code.chars().all(|c| c.is_ascii_alphabetic()) => (Some(code), number),
            _ => (currency, text),
        },
        AmountStyle::Plain => (currency, text),
    };
    let code = code.ok_or_else(|| format!("no currency for amount {text:?}"))?;
    let currency = Currency::new(code).map_err(|e| e.to_string())?;
    Ok(Money::new(parse_decimal(number)?, currency))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_currency_prefixed_amounts() {
        let m = parse_money("USD -64.17", None, AmountStyle::CurrencyPrefixed).unwrap();
        assert_eq!(m.to_string(), "-64.17 USD");
        let m = parse_money("1,250.5", Some("eur"), AmountStyle::Plain).unwrap();
        assert_eq!(m.to_string(), "1250.5 EUR");
    }

    #[test]
    fn parses_dates_with_or_without_time() {
        let d = NaiveDate::from_ymd_opt(2026, 10, 23).unwrap();
        assert_eq!(parse_date("2026-10-23", "%Y-%m-%d").unwrap(), d);
        assert_eq!(parse_date("2026-10-23;09:30:00", "%Y-%m-%d").unwrap(), d);
        assert_eq!(parse_date("2026-10-23T21:27:46.583307Z", "rfc3339").unwrap(), d);
    }
}
