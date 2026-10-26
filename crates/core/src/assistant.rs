//! The tools an AI assistant can use on a book: the same reads and writes in
//! the app's assistant panel and in the `investor-mcp` server. Reads return
//! JSON for the model; writes go through the same code as the app's screens
//! and say what they changed. Whoever calls a write is responsible for asking
//! the person first: the app shows an approval card, the MCP client asks.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::bank::{self, BalanceInput, NewAccount};
use crate::jurisdiction::JurisdictionPack;
use crate::metal::{self, Metal, MetalInput, MetalPrice};
use crate::plan::{self, Goal, Market, Plan, Profile, to_f64};
use crate::prices::{self, FxQuote, Quote, QuoteSource};
use crate::private::{self, Estimate, PayoutInput, PrivateInput};
use crate::reports;
use crate::trade::{self, TradeInput};
use crate::vault::Vault;
use crate::whatif::{self, Band, Month, Move, RetireInput, WhatIfInput};
use crate::{Currency, Error, Money, Result};

/// What a tool call needs from outside the book.
pub struct Env<'a> {
    pub today: NaiveDate,
    /// Unique for this call; becomes the id of anything it records, so a
    /// repeated call cannot record twice.
    pub stamp: String,
    /// The tax pack for a country and year, when there is one.
    pub pack: &'a dyn Fn(&str, i32) -> Option<JurisdictionPack>,
}

/// Guidance for the model, shared by the app and the MCP server.
pub const INSTRUCTIONS: &str = "Investor keeps a person's investments in an encrypted .invest file: a double-entry ledger, \
prices, a plan with goals, and 'what if' moves. Read tools look at it and run simulations. Write tools change it; use one \
only after the person has agreed to that exact change, then say what changed. Never invent numbers: get them from the tools. \
Amounts are in the file's base currency unless a tool says otherwise. Projections are in today's money (after inflation and \
fund costs) and come from simulated markets: give ranges, not promises. Tax figures come from draft rule packs that may be \
unverified: say so. Before suggesting a purchase or sale, run preview_trade and project to show its effects. You are not a \
licensed adviser: explain trade-offs instead of giving orders, and answer in the person's language.";

const WRITES: [&str; 15] = [
    "record_trade",
    "add_bank_account",
    "update_bank_balance",
    "set_price",
    "set_fx_rate",
    "record_metal",
    "set_metal_price",
    "add_private_holding",
    "set_private_value",
    "record_private_payout",
    "update_profile",
    "save_goal",
    "remove_goal",
    "save_move",
    "remove_move",
];

/// Whether a tool changes the book.
pub fn is_write(name: &str) -> bool {
    WRITES.contains(&name)
}

fn schema(properties: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": properties, "required": required })
}

fn trade_properties() -> Value {
    json!({
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
    })
}

/// Every tool with its JSON schema, as `{name, description, input_schema}`.
/// Without `writes`, only the tools that read.
pub fn tools(writes: bool) -> Value {
    let market = json!({ "type": "string", "enum": ["bear", "base", "bull"], "description": "Market assumption: bear 1%, base 3.5%, bull 6% a year above inflation. Default base." });
    let move_properties = json!({
        "kind": { "type": "string", "enum": ["lump", "spread", "monthly", "withdraw"], "description": "lump: invest at once; spread: invest in equal parts over `months` (cost averaging); monthly: invest this every month instead of the plan's amount; withdraw: take out" },
        "amount": { "type": "string", "description": "In the base currency, e.g. \"10000\"" },
        "start": { "type": "string", "description": "YYYY-MM; omit for now" },
        "months": { "type": "integer", "description": "For spread; default 12" },
        "from_cash": { "type": "boolean", "description": "For lump/spread: use cash already held instead of new money. Default false" },
        "asset": { "type": "object", "description": "For lump/spread into something other than the portfolio, in percent a year", "properties": { "real_return": { "type": "string" }, "volatility": { "type": "string" } }, "required": ["real_return", "volatility"] },
        "name": { "type": "string" }
    });
    let date = json!({ "type": "string", "description": "YYYY-MM-DD; default today" });
    let fx = json!({ "type": "string", "description": "Base currency for one unit of `currency` on that day; default latest known" });
    let mut list = vec![
        json!({ "name": "portfolio", "description": "Holdings at their latest price (or cost), coins and bars, bank accounts, private holdings, cash per broker and currency, net worth and unrealised gain. Start here.", "input_schema": schema(json!({}), &[]) }),
        json!({ "name": "plan", "description": "The person's profile (age, monthly investing, spending, risk level, retirement), goals and saved 'what if' moves.", "input_schema": schema(json!({}), &[]) }),
        json!({ "name": "goals", "description": "Progress on each goal: money assigned today, chance of reaching it, monthly saving needed.", "input_schema": schema(json!({ "market": market }), &[]) }),
        json!({
            "name": "project",
            "description": "Simulates the future from today's holdings and the plan (monthly investing, goal spending, retirement), compared with the same plan plus the given moves. Use it to test a lump sum against investing gradually, a higher monthly amount, or a withdrawal. Nothing is saved.",
            "input_schema": schema(json!({
                "years": { "type": "integer", "description": "Default 20" },
                "market": market,
                "moves": { "type": "array", "items": { "type": "object", "properties": move_properties, "required": ["kind", "amount"] }, "description": "Moves to try on top of the plan" },
                "include_saved_moves": { "type": "boolean", "description": "Also apply the moves saved in the file (switched on). Default false" },
                "spend_goals": { "type": "boolean", "description": "Take savings goals out in their year. Default true" }
            }), &[])
        }),
        json!({ "name": "preview_trade", "description": "What a purchase or sale would change, without recording it: cash at the broker, fees, FIFO gain and tax treatment for a sale, holdings before and after, and all money in 20 years with and without it. Give either units or amount.", "input_schema": schema(trade_properties(), &["side", "broker", "symbol"]) }),
        json!({ "name": "realized_gains", "description": "Sales in a year with their gain, the tax pack's treatment and summary for the person's tax residency, and employment income from payouts of private holdings.", "input_schema": schema(json!({ "year": { "type": "integer" } }), &["year"]) }),
        json!({ "name": "declaration", "description": "What to type into the person's tax return for a year: each section of the forms (appendix, code, the form's own column names) filled from the books, the steps to file, and how many sales still need their venue decided. Follow it when helping someone file.", "input_schema": schema(json!({ "year": { "type": "integer" } }), &["year"]) }),
        json!({ "name": "journal", "description": "Recent journal entries (newest first): trades, deposits, dividends, fees and taxes, with their postings.", "input_schema": schema(json!({ "limit": { "type": "integer", "description": "Default 30" }, "symbol": { "type": "string" } }), &[]) }),
    ];
    if writes {
        list.extend([
            json!({ "name": "record_trade", "description": "Records a purchase or sale in the books, exactly like the app's Plan a trade screen. Same fields as preview_trade; preview it first and record only what the person agreed to.", "input_schema": schema(trade_properties(), &["side", "broker", "symbol"]) }),
            json!({ "name": "add_bank_account", "description": "Adds a bank account kept by hand with its balance; the money counts as cash.", "input_schema": schema(json!({ "name": { "type": "string" }, "currency": { "type": "string" }, "balance": { "type": "string" }, "date": date, "fx_rate": fx }), &["name", "currency", "balance"]) }),
            json!({ "name": "update_bank_balance", "description": "Sets a bank account's whole balance on a date; the books record the difference as money added or taken out.", "input_schema": schema(json!({ "account": { "type": "string", "description": "The account's name or id (see portfolio)" }, "balance": { "type": "string" }, "date": date, "fx_rate": fx }), &["account", "balance"]) }),
            json!({ "name": "set_price", "description": "Sets the price of a holding on a date, e.g. from the person's broker.", "input_schema": schema(json!({ "symbol": { "type": "string" }, "price": { "type": "string" }, "currency": { "type": "string" }, "date": date }), &["symbol", "price", "currency"]) }),
            json!({ "name": "set_fx_rate", "description": "Sets an exchange rate: how much of the base currency one unit of `currency` buys.", "input_schema": schema(json!({ "currency": { "type": "string" }, "rate": { "type": "string" }, "date": date }), &["currency", "rate"]) }),
            json!({ "name": "record_metal", "description": "Records buying or selling physical gold, silver, platinum or palladium (coins and bars). `amount` is everything paid or received, premium and fees included.", "input_schema": schema(json!({ "side": { "type": "string", "enum": ["buy", "sell"] }, "metal": { "type": "string", "enum": ["gold", "silver", "platinum", "palladium"] }, "weight": { "type": "string", "description": "Fine (pure) metal" }, "unit": { "type": "string", "enum": ["oz", "g"], "description": "Troy ounces or grams; default oz" }, "amount": { "type": "string" }, "currency": { "type": "string" }, "date": date, "fx_rate": fx, "description": { "type": "string", "description": "e.g. 2 x Krugerrand 1 oz" } }), &["side", "metal", "weight", "amount", "currency"]) }),
            json!({ "name": "set_metal_price", "description": "Sets the price of a metal per troy ounce or gram.", "input_schema": schema(json!({ "metal": { "type": "string", "enum": ["gold", "silver", "platinum", "palladium"] }, "price": { "type": "string" }, "unit": { "type": "string", "enum": ["oz", "g"], "description": "Default oz" }, "currency": { "type": "string" }, "date": date, "fx_rate": fx }), &["metal", "price", "currency"]) }),
            json!({ "name": "add_private_holding", "description": "Adds money put into something without a market price, e.g. an employer co-investment, paid from outside the books.", "input_schema": schema(json!({ "name": { "type": "string" }, "amount": { "type": "string", "description": "What was paid" }, "currency": { "type": "string" }, "date": date, "fx_rate": fx, "payout": { "type": "string", "enum": ["employment_income", "capital_gains", "unknown"], "description": "How the money that comes back will be taxed" }, "condition": { "type": "string" }, "paid_from": { "type": "string" } }), &["name", "amount", "currency", "payout"]) }),
            json!({ "name": "set_private_value", "description": "Sets what a private holding is worth now, in its own currency; leave amount out to go back to valuing it at cost.", "input_schema": schema(json!({ "holding": { "type": "string", "description": "Name or id (see portfolio)" }, "amount": { "type": "string" }, "date": date }), &["holding"]) }),
            json!({ "name": "record_private_payout", "description": "Records the money coming back from a private holding, all at once: the holding closes and the difference to what was paid in is booked as employment income or a capital gain.", "input_schema": schema(json!({ "holding": { "type": "string" }, "amount": { "type": "string", "description": "Before tax" }, "currency": { "type": "string" }, "date": date, "fx_rate": fx, "withheld": { "type": "string", "description": "Tax kept back through payroll, in `currency`" }, "taxed_as": { "type": "string", "enum": ["employment_income", "capital_gains"] }, "paid_into": { "type": "string" } }), &["holding", "amount", "currency", "taxed_as"]) }),
            json!({ "name": "update_profile", "description": "Changes the given fields of the person's profile; the others stay.", "input_schema": schema(json!({ "birth_year": { "type": "integer" }, "monthly_spending": { "type": "string" }, "monthly_investing": { "type": "string" }, "risk": { "type": "string", "enum": ["cautious", "balanced", "growth"] }, "retirement_age": { "type": "integer" }, "pension_age": { "type": "integer" }, "monthly_pension": { "type": "string" }, "retirement_spending": { "type": "string" } }), &[]) }),
            json!({ "name": "save_goal", "description": "Adds a goal, or changes one when `id` is given. An emergency fund's target is months of spending; a savings goal's is an amount by a year.", "input_schema": schema(json!({ "id": { "type": "string" }, "name": { "type": "string" }, "kind": { "type": "string", "enum": ["emergency_fund", "savings"] }, "target": { "type": "string" }, "year": { "type": "integer" }, "monthly": { "type": "string", "description": "Put towards it each month" } }), &["name", "kind", "target"]) }),
            json!({ "name": "remove_goal", "description": "Removes a goal by id (see plan).", "input_schema": schema(json!({ "id": { "type": "string" } }), &["id"]) }),
            json!({ "name": "save_move", "description": "Saves a 'what if' move to the plan so it shows in the app's Projections.", "input_schema": schema(move_properties, &["kind", "amount"]) }),
            json!({ "name": "remove_move", "description": "Removes a saved 'what if' move by id (see plan).", "input_schema": schema(json!({ "id": { "type": "string" } }), &["id"]) }),
        ]);
    }
    Value::Array(list)
}

/// Runs a tool. Reads leave the book as it is; writes change it.
pub fn call(v: &mut Vault, name: &str, args: &Value, env: &Env) -> Result<Value> {
    match name {
        "portfolio" => Ok(portfolio(v)),
        "plan" => Ok(serde_json::to_value(&v.plan)?),
        "goals" => {
            let (cash, invested) = whatif::starting_point(&v.ledger, &v.prices);
            Ok(serde_json::to_value(plan::evaluate_goals(&v.plan, cash, invested, env.today.year(), market(args)?, 1.0))?)
        }
        "project" => project(v, args, env),
        "preview_trade" => preview_trade(v, args, env),
        "realized_gains" => realized_gains(v, args, env),
        "declaration" => {
            let year = args["year"].as_i64().ok_or_else(|| bad("year is required"))? as i32;
            let pack = (env.pack)(&v.settings.tax_residency, year).ok_or_else(|| bad(format!("no tax pack for {} {year}", v.settings.tax_residency)))?;
            Ok(serde_json::to_value(crate::declaration::declaration(&v.ledger, &pack, year, &v.settings.language))?)
        }
        "journal" => Ok(journal(v, args)),
        "record_trade" => record_trade(v, args, env),
        "add_bank_account" => add_bank_account(v, args, env),
        "update_bank_balance" => update_bank_balance(v, args, env),
        "set_price" => set_price(v, args, env),
        "set_fx_rate" => set_fx_rate(v, args, env),
        "record_metal" => record_metal(v, args, env),
        "set_metal_price" => set_metal_price(v, args, env),
        "add_private_holding" => add_private_holding(v, args, env),
        "set_private_value" => set_private_value(v, args, env),
        "record_private_payout" => record_private_payout(v, args, env),
        "update_profile" => update_profile(v, args),
        "save_goal" => save_goal(v, args, env),
        "remove_goal" => remove_goal(v, args),
        "save_move" => save_move(v, args, env),
        "remove_move" => remove_move(v, args),
        other => Err(Error::user("assistant.unknownTool").arg("name", other)),
    }
}

fn bad(message: impl Into<String>) -> Error {
    Error::user("assistant.badArgs").arg("detail", message.into())
}

fn text<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty())
}

fn required<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    text(args, key).ok_or_else(|| bad(format!("{key} is required")))
}

fn decimal(args: &Value, key: &str) -> Result<Option<Decimal>> {
    let Some(raw) = args.get(key).and_then(|v| v.as_str().map(str::to_owned).or_else(|| v.as_f64().map(|f| f.to_string()))) else { return Ok(None) };
    let cleaned = raw.trim().replace([' ', '\u{a0}'], "").replace(',', ".");
    if cleaned.is_empty() {
        return Ok(None);
    }
    cleaned.parse::<Decimal>().map(Some).map_err(|_| bad(format!("{key} is not a number: {raw}")))
}

fn amount(args: &Value, key: &str) -> Result<Decimal> {
    decimal(args, key)?.ok_or_else(|| bad(format!("{key} is required")))
}

fn date(args: &Value, env: &Env) -> Result<NaiveDate> {
    match text(args, "date") {
        Some(d) => d.parse().map_err(|_| bad(format!("date is not YYYY-MM-DD: {d}"))),
        None => Ok(env.today),
    }
}

fn currency(args: &Value, key: &str) -> Result<Currency> {
    Currency::new(&required(args, key)?.to_uppercase())
}

fn market(args: &Value) -> Result<Market> {
    Ok(serde_json::from_value(args.get("market").cloned().filter(|m| !m.is_null()).unwrap_or(json!("base")))?)
}

fn this_month(env: &Env) -> Month {
    Month { year: env.today.year(), month: env.today.month() }
}

fn round(v: f64) -> f64 {
    // Adding zero turns -0.0 into 0.0.
    (v * 100.0).round() / 100.0 + 0.0
}

fn portfolio(v: &Vault) -> Value {
    let values = prices::valuations(&v.ledger, &v.prices);
    let (cash, invested) = whatif::starting_point(&v.ledger, &v.prices);
    let unrealised: Decimal = values.iter().filter_map(|x| x.gain).sum();
    let private_total = to_f64(private::total(v));
    json!({
        "base_currency": v.settings.base_currency,
        "tax_residency": v.settings.tax_residency,
        "language": v.settings.language,
        "holdings": values.iter().filter(|x| x.broker != metal::PHYSICAL).collect::<Vec<_>>(),
        "coins_and_bars": metal::holdings(v),
        "bank_accounts": bank::balances(v),
        "private": private::valuations(v),
        "cash": reports::cash(&v.ledger),
        "totals": { "invested": round(invested), "cash": round(cash), "private": round(private_total), "net_worth": round(invested + cash + private_total), "unrealised_gain": unrealised },
        "note": "Coins and bars are counted in grams of fine metal and are part of `invested`. Bank accounts are part of `cash`. Holdings without a price are counted at cost. Private holdings have no market price: they count at the value set by hand, else at cost, and are left out of projections.",
    })
}

fn project(v: &Vault, args: &Value, env: &Env) -> Result<Value> {
    let (cash, invested) = whatif::starting_point(&v.ledger, &v.prices);
    let mut plan: Plan = v.plan.clone();
    if !args["include_saved_moves"].as_bool().unwrap_or(false) {
        plan.what_if.clear();
    }
    for (i, m) in args["moves"].as_array().into_iter().flatten().enumerate() {
        plan.what_if.push(read_move(m, format!("tried-{i}"))?);
    }
    let now = this_month(env);
    let retirement = plan.profile.as_ref().and_then(|p| {
        let age = (now.year - p.birth_year?).max(0) as u32;
        Some(RetireInput {
            age_now: age,
            retire_at: p.retirement_age?.max(age),
            plan_until: 95.max(age + 1),
            monthly_spending: p.retirement_spending.or(p.monthly_spending).map_or(0.0, to_f64),
            pension_from: p.pension_age.unwrap_or(65),
            monthly_pension: p.monthly_pension.map_or(0.0, to_f64),
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

fn read_move(m: &Value, id: String) -> Result<Move> {
    let mut m = m.clone();
    if let Some(o) = m.as_object_mut() {
        o.retain(|_, v| !v.is_null());
        o.insert("id".into(), json!(id));
        o.insert("enabled".into(), json!(true));
    }
    serde_json::from_value(m).map_err(|e| bad(format!("reading a move: {e}")))
}

/// The trade the arguments describe, with the price and units filled in.
fn trade_input(v: &Vault, args: &Value, env: &Env, id: String) -> Result<TradeInput> {
    let symbol = required(args, "symbol")?.to_uppercase();
    let quote = v.prices.latest(&symbol);
    let instrument = v.ledger.instruments.get(&symbol);
    let currency = match text(args, "currency") {
        Some(c) => Currency::new(&c.to_uppercase())?,
        None => quote.map(|q| q.price.currency.clone()).or_else(|| instrument.and_then(|i| i.currency.clone())).unwrap_or_else(|| v.settings.base_currency.clone()),
    };
    let price = match decimal(args, "price")? {
        Some(p) => p,
        None => quote.filter(|q| q.price.currency == currency).map(|q| q.price.amount).ok_or_else(|| bad("no known price: give price"))?,
    };
    let units = match (decimal(args, "units")?, decimal(args, "amount")?) {
        (Some(u), _) => u,
        (None, Some(amount)) if !price.is_zero() => trade::units_for(amount, price, args["whole_units"].as_bool().unwrap_or(false)),
        _ => return Err(bad("give units, or an amount and a price")),
    };
    let mut fees = Vec::new();
    for (key, kind) in [("commission", "commission"), ("fx_fee", "fx_conversion")] {
        if let Some(amount) = decimal(args, key)? {
            fees.push(json!({ "kind": kind, "amount": amount }));
        }
    }
    let venue = text(args, "venue").map(str::to_owned).or_else(|| {
        // The exchange used before for this symbol, so tax treatment carries over.
        v.ledger.entries.iter().rev().find(|e| e.postings.iter().any(|p| p.account.ends_with(&format!(":{symbol}")))).and_then(|e| e.venue.as_ref()).map(|v| v.code.clone())
    });
    serde_json::from_value(json!({
        "id": id,
        "date": date(args, env)?,
        "broker": required(args, "broker")?,
        "side": required(args, "side")?,
        "symbol": symbol,
        "name": instrument.and_then(|i| i.name.clone()),
        "units": units,
        "price": price,
        "currency": currency,
        "fees": fees,
        "venue": venue,
        "fx_rate": decimal(args, "fx_rate")?,
    }))
    .map_err(|e| bad(e.to_string()))
}

fn preview_trade(v: &Vault, args: &Value, env: &Env) -> Result<Value> {
    let input = trade_input(v, args, env, "preview".into())?;
    let pack = (env.pack)(&v.settings.tax_residency, input.date.year());
    let settings = WhatIfInput { cash: 0.0, invested: 0.0, today: this_month(env), years: 20, market: Market::Base, spend_goals: true, retirement: None, paths: 1000 };
    let outlook = trade::outlook(&v.ledger, &v.prices, &v.plan, &input, pack.as_ref(), &settings)?;
    let mut value = serde_json::to_value(&outlook)?;
    value["units"] = json!(input.units);
    value["price"] = json!(input.price);
    value["venue"] = json!(input.venue);
    value["note"] = json!("Nothing was recorded. record_trade with the same fields records it.");
    Ok(value)
}

fn record_trade(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let input = trade_input(v, args, env, format!("assistant-{}", env.stamp))?;
    let entry = trade::record(&mut v.ledger, &mut v.prices, &input)?;
    Ok(json!({ "recorded": entry.description, "entry": entry }))
}

fn realized_gains(v: &Vault, args: &Value, env: &Env) -> Result<Value> {
    let year = args["year"].as_i64().ok_or_else(|| bad("year is required"))? as i32;
    let employment = reports::employment_income(&v.ledger, year);
    let Some(pack) = (env.pack)(&v.settings.tax_residency, year) else {
        let gains = reports::realized_gains(&v.ledger, year);
        return Ok(json!({ "sales": gains, "employment_income": employment, "tax": format!("no tax pack for {} {year}", v.settings.tax_residency) }));
    };
    let gains = reports::realized_gains_by(&v.ledger, year, pack.capital_gains.cost_basis);
    let sales: Vec<Value> = gains.iter().map(|g| json!({ "sale": g, "treatment": pack.classify(g) })).collect();
    Ok(json!({
        "sales": sales,
        "summary": pack.summarize(&gains),
        "employment_income": employment,
        "employment_rule": pack.employment_income,
        "pack": { "authority": pack.pack.authority, "verified": pack.pack.verified, "rate": pack.capital_gains.rate, "inclusion_rate": pack.capital_gains.inclusion_rate, "form": pack.capital_gains.form, "notes": pack.pack.notes.as_ref().map(|n| n.get(&v.settings.language)), "deadlines": pack.deadlines, "currency": pack.pack.currency },
    }))
}

fn journal(v: &Vault, args: &Value) -> Value {
    let limit = args["limit"].as_u64().unwrap_or(30) as usize;
    let symbol = text(args, "symbol").map(str::to_uppercase);
    let mut entries = v.ledger.entries_by_date();
    entries.reverse();
    let picked: Vec<_> = entries
        .into_iter()
        .filter(|e| symbol.as_ref().is_none_or(|s| e.postings.iter().any(|p| p.account.ends_with(&format!(":{s}")))))
        .take(limit)
        .collect();
    json!(picked)
}

fn bank_id(v: &Vault, name: &str) -> Result<String> {
    let slug = crate::ledger::slug(name);
    v.banks
        .iter()
        .find(|a| a.id == name || a.name.eq_ignore_ascii_case(name) || a.id == slug)
        .map(|a| a.id.clone())
        .ok_or_else(|| Error::user("bank.unknown").arg("id", name))
}

fn add_bank_account(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let input = NewAccount {
        name: required(args, "name")?.to_owned(),
        currency: currency(args, "currency")?,
        balance: BalanceInput { entry_id: format!("assistant-{}", env.stamp), date: date(args, env)?, balance: amount(args, "balance")?, fx_rate: decimal(args, "fx_rate")? },
    };
    bank::add(v, &input)?;
    Ok(json!({ "added": input.name, "accounts": bank::balances(v) }))
}

fn update_bank_balance(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let id = bank_id(v, required(args, "account")?)?;
    let input = BalanceInput { entry_id: format!("assistant-{}", env.stamp), date: date(args, env)?, balance: amount(args, "balance")?, fx_rate: decimal(args, "fx_rate")? };
    let entry = bank::set_balance(v, &id, &input)?;
    Ok(json!({ "booked": entry.map(|e| e.description), "account": bank::balances(v).into_iter().find(|b| b.account.id == id) }))
}

fn set_price(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let symbol = required(args, "symbol")?.to_uppercase();
    let price = Money::new(amount(args, "price")?, currency(args, "currency")?);
    if price.amount <= Decimal::ZERO {
        return Err(Error::user("metal.priceNotPositive"));
    }
    v.prices.add_quote(&symbol, Quote { date: date(args, env)?, price: price.clone(), source: QuoteSource::Manual });
    Ok(json!({ "symbol": symbol, "price": price }))
}

fn set_fx_rate(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let currency = currency(args, "currency")?;
    let rate = amount(args, "rate")?;
    if rate <= Decimal::ZERO {
        return Err(Error::user("fx.notPositive"));
    }
    v.prices.add_fx(currency.clone(), FxQuote { date: date(args, env)?, rate, source: QuoteSource::Manual });
    Ok(json!({ "currency": currency, "rate": rate, "base": v.settings.base_currency }))
}

fn metal_of(args: &Value) -> Result<Metal> {
    serde_json::from_value(json!(required(args, "metal")?.to_lowercase())).map_err(|_| bad("metal is gold, silver, platinum or palladium"))
}

/// Grams in one of the given unit (troy ounces unless `g`).
fn grams_in(args: &Value) -> Decimal {
    if text(args, "unit").is_some_and(|u| u.eq_ignore_ascii_case("g")) { Decimal::ONE } else { metal::grams_per_ounce() }
}

fn record_metal(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let input = MetalInput {
        entry_id: format!("assistant-{}", env.stamp),
        date: date(args, env)?,
        metal: metal_of(args)?,
        grams: (amount(args, "weight")? * grams_in(args)).round_dp(4),
        amount: amount(args, "amount")?,
        currency: currency(args, "currency")?,
        fx_rate: decimal(args, "fx_rate")?,
        description: text(args, "description").unwrap_or_default().to_owned(),
    };
    let entry = match required(args, "side")? {
        "buy" => metal::buy(v, &input)?,
        "sell" => metal::sell(v, &input)?,
        _ => return Err(bad("side is buy or sell")),
    };
    Ok(json!({ "recorded": entry.description, "holdings": metal::holdings(v) }))
}

fn set_metal_price(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let input = MetalPrice {
        date: date(args, env)?,
        metal: metal_of(args)?,
        per_gram: (amount(args, "price")? / grams_in(args)).round_dp(4),
        currency: currency(args, "currency")?,
        fx_rate: decimal(args, "fx_rate")?,
    };
    metal::set_price(v, &input)?;
    Ok(json!({ "holdings": metal::holdings(v) }))
}

fn private_id(v: &Vault, name: &str) -> Result<String> {
    let slug = crate::ledger::slug(name);
    v.private
        .iter()
        .find(|h| h.id == name || h.name.eq_ignore_ascii_case(name) || h.id == slug)
        .map(|h| h.id.clone())
        .ok_or_else(|| Error::user("private.unknown").arg("id", name))
}

fn add_private_holding(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let input = PrivateInput {
        entry_id: format!("assistant-{}", env.stamp),
        name: required(args, "name")?.to_owned(),
        date: date(args, env)?,
        amount: amount(args, "amount")?,
        currency: currency(args, "currency")?,
        fx_rate: decimal(args, "fx_rate")?,
        paid_from: text(args, "paid_from").unwrap_or_default().to_owned(),
        payout: serde_json::from_value(json!(required(args, "payout")?)).map_err(|_| bad("payout is employment_income, capital_gains or unknown"))?,
        condition: text(args, "condition").unwrap_or_default().to_owned(),
    };
    private::add(v, &input)?;
    Ok(json!({ "added": input.name, "private": private::valuations(v) }))
}

fn set_private_value(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let id = private_id(v, required(args, "holding")?)?;
    let estimate = decimal(args, "amount")?.map(|amount| Ok::<_, Error>(Estimate { date: date(args, env)?, amount })).transpose()?;
    private::set_estimate(v, &id, estimate)?;
    Ok(json!({ "private": private::valuations(v).into_iter().find(|h| h.holding.id == id) }))
}

fn record_private_payout(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let id = private_id(v, required(args, "holding")?)?;
    let input = PayoutInput {
        entry_id: format!("assistant-{}", env.stamp),
        date: date(args, env)?,
        amount: amount(args, "amount")?,
        currency: currency(args, "currency")?,
        fx_rate: decimal(args, "fx_rate")?,
        withheld: decimal(args, "withheld")?.unwrap_or_default(),
        taxed_as: serde_json::from_value(json!(required(args, "taxed_as")?)).map_err(|_| bad("taxed_as is employment_income or capital_gains"))?,
        paid_into: text(args, "paid_into").unwrap_or_default().to_owned(),
    };
    let entry = private::pay_out(v, &id, &input)?;
    Ok(json!({ "recorded": entry.description, "private": private::valuations(v).into_iter().find(|h| h.holding.id == id) }))
}

fn update_profile(v: &mut Vault, args: &Value) -> Result<Value> {
    let mut profile = serde_json::to_value(&v.plan.profile)?;
    if !profile.is_object() {
        profile = json!({});
    }
    for key in ["birth_year", "monthly_spending", "monthly_investing", "risk", "retirement_age", "pension_age", "monthly_pension", "retirement_spending"] {
        if let Some(value) = args.get(key).filter(|v| !v.is_null()) {
            profile[key] = match value {
                Value::Number(n) if !matches!(key, "birth_year" | "retirement_age" | "pension_age") => json!(n.to_string()),
                other => other.clone(),
            };
        }
    }
    let profile: Profile = serde_json::from_value(profile).map_err(|e| bad(format!("profile: {e}")))?;
    v.plan.profile = Some(profile);
    Ok(json!({ "profile": v.plan.profile }))
}

fn save_goal(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let id = text(args, "id").map_or_else(|| format!("goal-{}", env.stamp), str::to_owned);
    let goal: Goal = serde_json::from_value(json!({
        "id": id,
        "name": required(args, "name")?,
        "kind": required(args, "kind")?,
        "target": amount(args, "target")?,
        "year": args.get("year").filter(|y| !y.is_null()),
        "monthly": decimal(args, "monthly")?.unwrap_or_default(),
    }))
    .map_err(|e| bad(format!("goal: {e}")))?;
    match v.plan.goals.iter_mut().find(|g| g.id == goal.id) {
        Some(existing) => *existing = goal,
        None => v.plan.goals.push(goal),
    }
    Ok(json!({ "goals": v.plan.goals }))
}

fn remove_goal(v: &mut Vault, args: &Value) -> Result<Value> {
    let id = required(args, "id")?;
    let before = v.plan.goals.len();
    v.plan.goals.retain(|g| g.id != id);
    if v.plan.goals.len() == before {
        return Err(bad(format!("no goal {id}")));
    }
    Ok(json!({ "goals": v.plan.goals }))
}

fn save_move(v: &mut Vault, args: &Value, env: &Env) -> Result<Value> {
    let m = read_move(args, format!("move-{}", env.stamp))?;
    v.plan.what_if.push(m);
    Ok(json!({ "what_if": v.plan.what_if }))
}

fn remove_move(v: &mut Vault, args: &Value) -> Result<Value> {
    let id = required(args, "id")?;
    let before = v.plan.what_if.len();
    v.plan.what_if.retain(|m| m.id != id);
    if v.plan.what_if.len() == before {
        return Err(bad(format!("no move {id}")));
    }
    Ok(json!({ "what_if": v.plan.what_if }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(stamp: &str) -> Env<'static> {
        Env { today: NaiveDate::from_ymd_opt(2026, 10, 25).unwrap(), stamp: stamp.into(), pack: &|_, _| None }
    }

    fn example() -> Vault {
        crate::example::vault(NaiveDate::from_ymd_opt(2026, 10, 25).unwrap(), "en").unwrap()
    }

    #[test]
    fn every_tool_is_listed_and_reads_are_separate() {
        let all = tools(true);
        let names: Vec<&str> = all.as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(names.len(), 23);
        assert!(WRITES.iter().all(|w| names.contains(w)));
        assert_eq!(tools(false).as_array().unwrap().len(), 8);
    }

    #[test]
    fn reads_leave_the_book_alone() {
        let mut v = example();
        let before = v.clone();
        for name in ["portfolio", "plan", "goals", "journal"] {
            call(&mut v, name, &json!({}), &env("r")).unwrap();
        }
        call(&mut v, "preview_trade", &json!({ "side": "buy", "broker": "IBKR", "symbol": "VWCE", "units": "2" }), &env("r")).unwrap();
        assert_eq!(v, before);
    }

    #[test]
    fn writes_record_through_the_same_code_as_the_app() {
        let mut v = example();
        let args = json!({ "side": "buy", "broker": "IBKR", "symbol": "VWCE", "units": "2", "commission": "1.25" });
        let out = call(&mut v, "record_trade", &args, &env("t1")).unwrap();
        assert!(out["recorded"].as_str().unwrap().contains("VWCE"));
        assert!(call(&mut v, "record_trade", &args, &env("t1")).is_err(), "the same call records once");

        call(&mut v, "update_bank_balance", &json!({ "account": "savings account", "balance": "15000" }), &env("b1")).unwrap();
        assert_eq!(bank::balances(&v)[0].balance, Decimal::from(15000));

        call(&mut v, "record_metal", &json!({ "side": "buy", "metal": "silver", "weight": "10", "amount": "350", "currency": "EUR" }), &env("m1")).unwrap();
        assert!(metal::holdings(&v).iter().any(|h| h.metal == Metal::Silver));

        call(&mut v, "update_profile", &json!({ "monthly_investing": 1000 }), &env("p")).unwrap();
        assert_eq!(v.plan.profile.as_ref().unwrap().monthly_investing, Some(Decimal::from(1000)));
        assert_eq!(v.plan.profile.as_ref().unwrap().retirement_age, Some(60), "other fields stay");

        call(&mut v, "save_goal", &json!({ "name": "Car", "kind": "savings", "target": "20000", "year": 2029 }), &env("g1")).unwrap();
        assert!(v.plan.goals.iter().any(|g| g.id == "goal-g1"));
        call(&mut v, "remove_goal", &json!({ "id": "goal-g1" }), &env("g2")).unwrap();
        assert!(call(&mut v, "remove_goal", &json!({ "id": "goal-g1" }), &env("g3")).is_err());
    }
}
