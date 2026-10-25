//! The tools Claude Code can call. Each reads the file and returns JSON;
//! none of them writes.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::{Datelike, Local};
use investor_core::jurisdiction::JurisdictionPack;
use investor_core::plan::{self, Market, Plan, to_f64};
use investor_core::prices;
use investor_core::private;
use investor_core::reports;
use investor_core::trade::{self, TradeInput};
use investor_core::vault::Vault;
use investor_core::whatif::{self, Band, Month, Move, RetireInput, WhatIfInput};
use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::book::Book;

pub fn list() -> Value {
    let market = json!({ "type": "string", "enum": ["bear", "base", "bull"], "description": "Market assumption: bear 1%, base 3.5%, bull 6% a year above inflation. Default base." });
    let move_schema = json!({
        "type": "object",
        "properties": {
            "kind": { "type": "string", "enum": ["lump", "spread", "monthly", "withdraw"], "description": "lump: invest at once; spread: invest in equal parts over `months` (cost averaging); monthly: invest this every month instead of the plan's amount; withdraw: take out" },
            "amount": { "type": "string", "description": "In the base currency, e.g. \"10000\"" },
            "start": { "type": "string", "description": "YYYY-MM; omit for now" },
            "months": { "type": "integer", "description": "For spread; default 12" },
            "from_cash": { "type": "boolean", "description": "For lump/spread: use cash already held instead of new money. Default false" },
            "asset": { "type": "object", "description": "For lump/spread into something other than the portfolio, in percent a year", "properties": { "real_return": { "type": "string" }, "volatility": { "type": "string" } }, "required": ["real_return", "volatility"] },
            "name": { "type": "string" }
        },
        "required": ["kind", "amount"]
    });
    json!([
        {
            "name": "portfolio",
            "description": "Holdings at their latest price (or cost), private holdings without a market price, cash per broker and currency, net worth and unrealised gain. Start here.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "plan",
            "description": "The person's profile (age, monthly investing, spending, risk level, retirement), goals and saved 'what if' moves.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "goals",
            "description": "Progress on each goal: money assigned today, chance of reaching it, monthly saving needed.",
            "inputSchema": { "type": "object", "properties": { "market": market } }
        },
        {
            "name": "project",
            "description": "Simulates the future from today's holdings and the plan (monthly investing, goal spending, retirement), compared with the same plan plus the given moves. Use it to test a lump sum against investing gradually, a higher monthly amount, or a withdrawal. Nothing is saved.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "years": { "type": "integer", "description": "Default 20" },
                    "market": market,
                    "moves": { "type": "array", "items": move_schema, "description": "Moves to try on top of the plan" },
                    "include_saved_moves": { "type": "boolean", "description": "Also apply the moves saved in the file (switched on). Default false" },
                    "spend_goals": { "type": "boolean", "description": "Take savings goals out in their year. Default true" }
                }
            }
        },
        {
            "name": "preview_trade",
            "description": "What a purchase or sale would change, without recording it: cash at the broker, fees, FIFO gain and tax treatment for a sale, holdings before and after, and all money in 20 years with and without it. Give either units or amount.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "side": { "type": "string", "enum": ["buy", "sell"] },
                    "broker": { "type": "string", "description": "Account name used in the books, e.g. IBKR or Revolut (see portfolio)" },
                    "symbol": { "type": "string" },
                    "units": { "type": "string", "description": "Number of units" },
                    "amount": { "type": "string", "description": "Instead of units: money to invest or raise, in the trade currency" },
                    "whole_units": { "type": "boolean", "description": "With amount: round down to whole units. Default false (fractions)" },
                    "price": { "type": "string", "description": "Per unit, in the trade currency; defaults to the latest known price" },
                    "currency": { "type": "string", "description": "Defaults to the instrument's currency or the base currency" },
                    "commission": { "type": "string" },
                    "fx_fee": { "type": "string", "description": "Currency conversion fee" },
                    "venue": { "type": "string", "description": "Exchange code, e.g. XETR; decides tax treatment in some countries" },
                    "date": { "type": "string", "description": "YYYY-MM-DD; default today" },
                    "fx_rate": { "type": "string", "description": "Base currency for one unit of the trade currency; default latest known" }
                },
                "required": ["side", "broker", "symbol"]
            }
        },
        {
            "name": "realized_gains",
            "description": "Sales in a year with their gain, and the tax pack's treatment and summary for the person's tax residency.",
            "inputSchema": { "type": "object", "properties": { "year": { "type": "integer" } }, "required": ["year"] }
        },
        {
            "name": "journal",
            "description": "Recent journal entries (newest first): trades, deposits, dividends, fees and taxes, with their postings.",
            "inputSchema": { "type": "object", "properties": { "limit": { "type": "integer", "description": "Default 30" }, "symbol": { "type": "string" } } }
        }
    ])
}

pub fn call(book: &mut Book, name: &str, args: &Value) -> Result<Value> {
    let v = book.vault()?;
    match name {
        "portfolio" => portfolio(v),
        "plan" => Ok(serde_json::to_value(&v.plan)?),
        "goals" => {
            let (cash, invested) = whatif::starting_point(&v.ledger, &v.prices);
            Ok(serde_json::to_value(plan::evaluate_goals(&v.plan, cash, invested, Local::now().year(), market(args)?, 1.0))?)
        }
        "project" => project(v, args),
        "preview_trade" => preview_trade(v, args),
        "realized_gains" => realized_gains(v, args),
        "journal" => journal(v, args),
        other => bail!("unknown tool {other}"),
    }
}

fn market(args: &Value) -> Result<Market> {
    Ok(serde_json::from_value(args.get("market").cloned().unwrap_or(json!("base")))?)
}

fn today() -> Month {
    let now = Local::now();
    Month { year: now.year(), month: now.month() }
}

fn portfolio(v: &Vault) -> Result<Value> {
    let values = prices::valuations(&v.ledger, &v.prices);
    let (cash, invested) = whatif::starting_point(&v.ledger, &v.prices);
    let unrealised: Decimal = values.iter().filter_map(|x| x.gain).sum();
    let private_total = to_f64(private::total(v));
    Ok(json!({
        "base_currency": v.settings.base_currency,
        "tax_residency": v.settings.tax_residency,
        "holdings": values,
        "private": private::valuations(v),
        "cash": reports::cash(&v.ledger),
        "totals": { "invested": round(invested), "cash": round(cash), "private": round(private_total), "net_worth": round(invested + cash + private_total), "unrealised_gain": unrealised },
        "note": "Holdings without a price are counted at cost. Cash in other currencies is at the rate when it arrived. Private holdings have no market price: they count at the value set by hand, else at cost, and are left out of projections because they cannot be sold at will.",
    }))
}

fn project(v: &Vault, args: &Value) -> Result<Value> {
    let (cash, invested) = whatif::starting_point(&v.ledger, &v.prices);
    let mut plan: Plan = v.plan.clone();
    if !args["include_saved_moves"].as_bool().unwrap_or(false) {
        plan.what_if.clear();
    }
    for (i, m) in args["moves"].as_array().into_iter().flatten().enumerate() {
        let mut m = m.clone();
        m["id"] = json!(format!("tried-{i}"));
        m["enabled"] = json!(true);
        plan.what_if.push(serde_json::from_value::<Move>(m).context("reading a move")?);
    }
    let now = today();
    let retirement = plan.profile.as_ref().and_then(|p| {
        let age = (now.year - p.birth_year?).max(0) as u32;
        Some(RetireInput {
            age_now: age,
            retire_at: p.retirement_age?.max(age),
            plan_until: 95.max(age + 1),
            monthly_spending: p.retirement_spending.or(p.monthly_spending).map_or(0.0, plan::to_f64),
            pension_from: p.pension_age.unwrap_or(65),
            monthly_pension: p.monthly_pension.map_or(0.0, plan::to_f64),
        })
    });
    let input = WhatIfInput {
        cash,
        invested,
        today: now,
        years: args["years"].as_u64().unwrap_or(20).clamp(1, 60) as u32,
        market: market(args)?,
        spend_goals: args["spend_goals"].as_bool().unwrap_or(true),
        retirement,
        paths: 1000,
    };
    let r = whatif::what_if(&input, &plan);
    let every_five = |bands: &[Band]| -> Vec<Value> {
        let last = bands.len() - 1;
        bands
            .iter()
            .filter(|b| b.year % 5 == 0 || b.year as usize == last)
            .map(|b| json!({ "year": now.year + b.year as i32, "bad_case_p10": round(b.p10), "middle_p50": round(b.p50), "good_case_p90": round(b.p90), "new_money_put_in": round(b.put_in) }))
            .collect()
    };
    let retire = |x: &whatif::Retirement| {
        json!({ "portfolio_at_retirement": round(x.at_retirement), "money_runs_out_at_age": x.runs_out_at, "chance_money_lasts_to_95": x.chance_lasts, "safe_monthly_spending": round(x.safe_monthly_spending) })
    };
    Ok(json!({
        "assumptions": { "market": input.market, "today": { "cash": round(cash), "invested": round(invested) }, "monthly_investing": r.monthly, "goal_spending": r.spending, "cash_real_return": whatif::CASH_REAL_RETURN },
        "plan": { "by_year": every_five(&r.plan.bands), "chance_withdrawals_paid": r.plan.chance_covered },
        "with_moves": r.with_moves.as_ref().map(|w| json!({ "by_year": every_five(&w.bands), "chance_withdrawals_paid": w.chance_covered })),
        "moves_short_of_cash": r.cash_short,
        "retirement": r.retirement.as_ref().map(|x| json!({ "plan": retire(&x.plan), "with_moves": x.with_moves.as_ref().map(retire) })),
    }))
}

fn decimal(args: &Value, key: &str) -> Result<Option<Decimal>> {
    args.get(key)
        .and_then(|v| v.as_str().map(str::to_owned).or_else(|| v.as_f64().map(|f| f.to_string())))
        .map(|s| s.trim().replace(',', ".").parse::<Decimal>().with_context(|| format!("{key} is not a number")))
        .transpose()
}

fn preview_trade(v: &Vault, args: &Value) -> Result<Value> {
    let symbol = args["symbol"].as_str().context("symbol is required")?.trim().to_uppercase();
    let quote = v.prices.quotes.get(&symbol).and_then(|q| q.last());
    let instrument = v.ledger.instruments.get(&symbol);
    let currency = match args["currency"].as_str() {
        Some(c) => investor_core::Currency::new(c)?,
        None => quote.map(|q| q.price.currency.clone()).or_else(|| instrument.and_then(|i| i.currency.clone())).unwrap_or_else(|| v.settings.base_currency.clone()),
    };
    let price = match decimal(args, "price")? {
        Some(p) => p,
        None => quote.filter(|q| q.price.currency == currency).map(|q| q.price.amount).context("no known price: give price")?,
    };
    let units = match (decimal(args, "units")?, decimal(args, "amount")?) {
        (Some(u), _) => u,
        (None, Some(amount)) if !price.is_zero() => trade::units_for(amount, price, args["whole_units"].as_bool().unwrap_or(false)),
        _ => bail!("give units, or an amount and a price"),
    };
    let mut fees = Vec::new();
    for (key, kind) in [("commission", "commission"), ("fx_fee", "fx_conversion")] {
        if let Some(amount) = decimal(args, key)? {
            fees.push(json!({ "kind": kind, "amount": amount }));
        }
    }
    let input: TradeInput = serde_json::from_value(json!({
        "id": "preview",
        "date": args["date"].as_str().map_or_else(|| Local::now().date_naive().to_string(), str::to_owned),
        "broker": args["broker"],
        "side": args["side"],
        "symbol": symbol,
        "units": units,
        "price": price,
        "currency": currency,
        "fees": fees,
        "venue": args["venue"],
        "fx_rate": decimal(args, "fx_rate")?,
    }))?;
    let pack = pack(&v.settings.tax_residency, input.date.year());
    let settings = WhatIfInput { cash: 0.0, invested: 0.0, today: today(), years: 20, market: Market::Base, spend_goals: true, retirement: None, paths: 1000 };
    let outlook = trade::outlook(&v.ledger, &v.prices, &v.plan, &input, pack.as_ref(), &settings)?;
    let mut value = serde_json::to_value(&outlook)?;
    value["units"] = json!(units);
    value["price"] = json!(price);
    value["note"] = json!("Nothing was recorded. The person records trades in the Investor app (Plan a trade).");
    Ok(value)
}

fn packs_dir() -> PathBuf {
    std::env::var_os("INVESTOR_JURISDICTIONS").map_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../jurisdictions")), PathBuf::from)
}

fn pack(country: &str, year: i32) -> Option<JurisdictionPack> {
    let text = std::fs::read_to_string(packs_dir().join(country.to_lowercase()).join(format!("{year}.toml"))).ok()?;
    JurisdictionPack::from_toml(&text).ok()
}

fn realized_gains(v: &Vault, args: &Value) -> Result<Value> {
    let year = args["year"].as_i64().context("year is required")? as i32;
    let gains = reports::realized_gains(&v.ledger, year);
    let Some(pack) = pack(&v.settings.tax_residency, year) else {
        return Ok(json!({ "sales": gains, "tax": format!("no tax pack for {} {year}", v.settings.tax_residency) }));
    };
    let sales: Vec<Value> = gains.iter().map(|g| json!({ "sale": g, "treatment": pack.classify(g) })).collect();
    Ok(json!({
        "sales": sales,
        "summary": pack.summarize(&gains),
        "pack": { "authority": pack.pack.authority, "verified": pack.pack.verified, "rate": pack.capital_gains.rate, "form": pack.capital_gains.form, "notes": pack.pack.notes },
    }))
}

fn journal(v: &Vault, args: &Value) -> Result<Value> {
    let limit = args["limit"].as_u64().unwrap_or(30) as usize;
    let symbol = args["symbol"].as_str().map(str::to_uppercase);
    let mut entries = v.ledger.entries_by_date();
    entries.reverse();
    let picked: Vec<_> = entries
        .into_iter()
        .filter(|e| symbol.as_ref().is_none_or(|s| e.postings.iter().any(|p| p.account.ends_with(&format!(":{s}")))))
        .take(limit)
        .collect();
    Ok(serde_json::to_value(picked)?)
}

fn round(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
