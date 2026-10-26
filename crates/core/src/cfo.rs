//! The whole financial picture in one place, for a virtual CFO: what the
//! person owns and owes, cash against the emergency fund, how saving went
//! against the plan, where the money is concentrated, this year's income,
//! costs and tax, and goals. The app works out every figure; `signals` are
//! plain facts worth acting on, for a model to explain and put in order.

use std::collections::BTreeMap;

use chrono::{Datelike, Months, NaiveDate};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::{Value, json};

use crate::jurisdiction::JurisdictionPack;
use crate::ledger;
use crate::plan::{self, GoalKind, Market, to_f64};
use crate::vault::Vault;
use crate::{debt, metal, prices, private, reports, whatif};

/// A loan dearer than this costs more than investing is likely to earn.
const HIGH_RATE: f64 = 0.06;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    High,
    Medium,
    Low,
}

/// A fact worth acting on, with the figures behind it.
#[derive(Debug, Clone, Serialize)]
pub struct Signal {
    pub kind: &'static str,
    pub severity: Severity,
    pub facts: Value,
}

fn round(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn share(part: f64, whole: f64) -> f64 {
    if whole > 0.0 { (part / whole * 1000.0).round() / 10.0 } else { 0.0 }
}

/// The picture on `today`, with this year's tax when a pack is given.
pub fn picture(v: &Vault, today: NaiveDate, pack: Option<&JurisdictionPack>) -> Value {
    let base = &v.settings.base_currency;
    let values = prices::valuations(&v.ledger, &v.prices);
    let (cash, invested) = whatif::starting_point(&v.ledger, &v.prices);
    let metals: f64 = values.iter().filter(|x| x.broker == metal::PHYSICAL).map(|x| to_f64(x.value.unwrap_or(x.cost))).sum();
    let private_total = to_f64(private::total(v));
    let debts = debt::balances(v);
    let owed: f64 = debts.iter().filter_map(|d| d.value).map(to_f64).sum();
    let net_worth = invested + cash + private_total - owed;
    let profile = v.plan.profile.clone();
    let spending = profile.as_ref().and_then(|p| p.monthly_spending).map(to_f64);
    let planned_monthly = profile.as_ref().and_then(|p| p.monthly_investing).map(to_f64);
    let emergency_months = v.plan.goals.iter().find(|g| g.kind == GoalKind::EmergencyFund).map(|g| to_f64(g.target));
    let target = match (spending, emergency_months) {
        (Some(s), Some(m)) => Some(s * m),
        _ => None,
    };

    // Where the invested money sits.
    let mut by_holding: BTreeMap<String, (String, f64)> = BTreeMap::new();
    let mut by_currency: BTreeMap<String, f64> = BTreeMap::new();
    let mut crypto = 0.0;
    for x in &values {
        let value = to_f64(x.value.unwrap_or(x.cost));
        let name = v.ledger.instruments.get(&x.symbol).and_then(|i| i.name.clone()).unwrap_or_else(|| x.symbol.clone());
        by_holding.entry(x.symbol.clone()).or_insert((name, 0.0)).1 += value;
        let currency = x.quote.as_ref().map_or_else(|| base.to_string(), |q| q.price.currency.to_string());
        *by_currency.entry(currency).or_default() += value;
        if reports::is_crypto(&v.ledger, &x.symbol) {
            crypto += value;
        }
    }
    let mut holdings: Vec<_> = by_holding.into_iter().map(|(symbol, (name, value))| (symbol, name, value)).collect();
    holdings.sort_by(|a, b| b.2.total_cmp(&a.2));
    let largest = holdings.first().map(|h| (h.0.clone(), h.1.clone(), share(h.2, invested)));
    let foreign: f64 = by_currency.iter().filter(|(c, _)| c.as_str() != base.as_str()).map(|(_, x)| x).sum();

    // The last twelve months against the plan.
    let since = today.checked_sub_months(Months::new(12)).unwrap_or(today).format("%Y-%m").to_string();
    let flows: Vec<_> = reports::monthly_flows(&v.ledger).into_iter().filter(|m| m.month > since).collect();
    let put_in: f64 = flows.iter().map(|m| to_f64(m.deposited - m.withdrawn)).sum();
    let bought: f64 = flows.iter().map(|m| to_f64(m.bought)).sum();

    // This year's income, costs and tax.
    let year = today.year();
    let total = |prefix: &str| -> f64 {
        v.ledger.entries.iter().filter(|e| e.date.year() == year).flat_map(|e| &e.postings).filter(|p| p.account.starts_with(prefix)).map(|p| to_f64(p.base)).sum()
    };
    let dividends = -total("Income:Dividends");
    let interest = -total(ledger::INTEREST_INCOME);
    let fees = total("Expenses:Fees");
    let withheld = total(ledger::WITHHOLDING_TAX);
    let tax = pack.map(|pack| {
        let sales = reports::realized_gains_by(&v.ledger, year, pack.capital_gains.cost_basis);
        let summary = pack.summarize(&sales);
        json!({ "authority": pack.pack.authority, "realised_taxable_net": summary.taxable_gains + summary.taxable_losses, "exempt": summary.exempt, "estimated_tax": summary.tax_due, "deadlines": pack.deadlines, "verified": pack.pack.verified })
    });
    let goals = plan::evaluate_goals(&v.plan, cash, invested, year, Market::Base, 1.0);
    let stale = values.iter().filter(|x| x.broker != metal::PHYSICAL && x.quote.as_ref().is_none_or(|q| (today - q.date).num_days() > 30)).count();

    let mut signals = Vec::new();
    let mut add = |kind: &'static str, severity: Severity, facts: Value| signals.push(Signal { kind, severity, facts });
    for d in debts.iter().filter(|d| d.owed > Decimal::ZERO) {
        let rate = d.debt.rate.map(to_f64).unwrap_or(0.0);
        if rate >= HIGH_RATE {
            add("high_interest_debt", Severity::High, json!({ "debt": d.debt.name, "rate_percent": round(rate * 100.0), "owed": d.value, "monthly": d.debt.monthly }));
        }
        if d.debt.monthly.is_some() && d.months_left.is_none() {
            add("debt_never_paid_off", Severity::High, json!({ "debt": d.debt.name, "rate_percent": round(rate * 100.0), "monthly": d.debt.monthly }));
        }
    }
    match (spending, target) {
        (Some(s), Some(t)) if cash < t => add("emergency_fund_short", Severity::High, json!({ "cash": round(cash), "target": round(t), "short_by": round(t - cash), "months_covered": round(cash / s) })),
        (Some(s), None) if cash < 3.0 * s => add("emergency_fund_short", Severity::High, json!({ "cash": round(cash), "months_covered": round(cash / s), "note": "no emergency fund goal; three to six months of spending is usual" })),
        (Some(s), t) => {
            let keep = t.unwrap_or(6.0 * s) + 3.0 * s;
            if cash > keep {
                add("idle_cash", Severity::Medium, json!({ "cash": round(cash), "emergency_target": t.map(round), "above_need": round(cash - keep), "note": "cash loses value to inflation" }));
            }
        }
        _ => add("profile_missing", Severity::Low, json!({ "missing": ["monthly_spending"], "why": "needed to size the emergency fund and judge idle cash" })),
    }
    if let Some((symbol, name, percent)) = &largest
        && *percent > 20.0
        && holdings.len() > 1
    {
        add("concentration", Severity::Medium, json!({ "symbol": symbol, "name": name, "share_of_invested_percent": percent }));
    }
    if share(crypto, invested) > 10.0 {
        add("crypto_share", Severity::Medium, json!({ "share_of_invested_percent": share(crypto, invested), "value": round(crypto) }));
    }
    if share(foreign, invested) > 50.0 {
        add("currency_exposure", Severity::Low, json!({ "outside_base_percent": share(foreign, invested), "base_currency": base }));
    }
    if let Some(planned) = planned_monthly.filter(|p| *p > 0.0) {
        if put_in < 0.75 * planned * 12.0 {
            add("behind_plan", Severity::Medium, json!({ "planned_12_months": round(planned * 12.0), "put_in_12_months": round(put_in), "percent_of_plan": share(put_in, planned * 12.0) }));
        }
    } else {
        add("profile_missing", Severity::Low, json!({ "missing": ["monthly_investing"], "why": "needed to compare saving with a plan" }));
    }
    if put_in > 0.0 && bought < 0.5 * put_in && cash > target.unwrap_or(0.0) {
        add("money_not_invested", Severity::Medium, json!({ "put_in_12_months": round(put_in), "bought_12_months": round(bought) }));
    }
    for (goal, status) in v.plan.goals.iter().zip(&goals) {
        if let Some(chance) = status.chance.filter(|c| *c < 0.5) {
            add("goal_at_risk", Severity::High, json!({ "goal": goal.name, "chance_percent": round(chance * 100.0), "year": goal.year, "needed_monthly": status.needed_monthly.map(round), "planned_monthly": goal.monthly }));
        }
    }
    if let Some(tax) = &tax
        && tax["estimated_tax"].as_str().and_then(|t| t.parse::<f64>().ok()).is_some_and(|t| t > 0.0)
    {
        add("tax_to_set_aside", Severity::Low, json!({ "estimated_tax": tax["estimated_tax"], "deadlines": tax["deadlines"] }));
    }
    if bought > 0.0 && fees > 0.01 * bought {
        add("high_fees", Severity::Low, json!({ "fees_this_year": round(fees), "bought_12_months": round(bought) }));
    }
    if stale > 0 {
        add("stale_prices", Severity::Low, json!({ "holdings": stale, "note": "values use old prices or cost" }));
    }
    signals.sort_by_key(|s| s.severity);

    json!({
        "as_of": today,
        "base_currency": base,
        "tax_residency": v.settings.tax_residency,
        "profile": profile,
        "balance_sheet": {
            "invested": round(invested),
            "of_which_coins_and_bars": round(metals),
            "cash": round(cash),
            "private_holdings": round(private_total),
            "debts": round(owed),
            "net_worth": round(net_worth),
        },
        "cash": { "total": round(cash), "monthly_spending": spending, "months_covered": spending.filter(|s| *s > 0.0).map(|s| round(cash / s)), "emergency_fund_target": target.map(round) },
        "debts": debts.iter().map(|d| json!({ "name": d.debt.name, "kind": d.debt.kind, "rate_percent": d.debt.rate.map(|r| round(to_f64(r) * 100.0)), "monthly": d.debt.monthly, "owed": d.value, "months_left": d.months_left })).collect::<Vec<_>>(),
        "allocation": {
            "largest_holdings": holdings.iter().take(8).map(|(symbol, name, value)| json!({ "symbol": symbol, "name": name, "value": round(*value), "share_percent": share(*value, invested) })).collect::<Vec<_>>(),
            "holdings_count": holdings.len(),
            "by_currency_percent": by_currency.iter().map(|(c, x)| (c.clone(), share(*x, invested))).collect::<BTreeMap<_, _>>(),
            "crypto_percent": share(crypto, invested),
            "unrealised_gain": values.iter().filter_map(|x| x.gain).sum::<Decimal>(),
        },
        "last_12_months": { "put_in": round(put_in), "bought": round(bought), "planned": planned_monthly.map(|p| round(p * 12.0)) },
        "this_year": { "dividends": round(dividends), "interest": round(interest), "fees": round(fees), "tax_withheld_abroad": round(withheld), "tax": tax },
        "goals": v.plan.goals.iter().zip(&goals).map(|(g, s)| json!({ "name": g.name, "kind": g.kind, "target": g.target, "year": g.year, "monthly": g.monthly, "have": round(s.have), "chance_percent": s.chance.map(|c| round(c * 100.0)), "on_track": s.on_track })).collect::<Vec<_>>(),
        "signals": signals,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_example_flags_the_car_loan_first() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap();
        let v = crate::example::vault(today, "en").unwrap();
        let p = picture(&v, today, None);
        assert!(p["balance_sheet"]["debts"].as_f64().unwrap() > 8000.0);
        let signals = p["signals"].as_array().unwrap();
        assert_eq!(signals[0]["severity"], "high");
        assert!(signals.iter().any(|s| s["kind"] == "high_interest_debt" && s["facts"]["debt"] == "Car loan"), "{signals:?}");
        let net = p["balance_sheet"]["net_worth"].as_f64().unwrap();
        let parts = p["balance_sheet"]["invested"].as_f64().unwrap() + p["balance_sheet"]["cash"].as_f64().unwrap() + p["balance_sheet"]["private_holdings"].as_f64().unwrap() - p["balance_sheet"]["debts"].as_f64().unwrap();
        assert!((net - parts).abs() < 0.02);
    }
}
