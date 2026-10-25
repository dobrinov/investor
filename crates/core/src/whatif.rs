//! Trying out moves on top of the plan: a lump sum, investing gradually,
//! a new monthly amount, or taking money out.
//!
//! The simulation starts from what you hold today — investments and cash
//! separately — adds what the plan says you invest each month, and takes
//! out the money your savings goals need in their year. The plan and the
//! plan with moves run on the same simulated markets, so any difference
//! between them comes from the moves alone. Steps are monthly, so investing
//! gradually over a few months is visible. Figures are in today's money.

use std::fmt;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::plan::{GoalKind, Market, Plan, Risk, Rng, log_params, percentile, to_f64};

/// Cash earns nothing and loses about this much a year to inflation.
pub const CASH_REAL_RETURN: f64 = -0.02;

/// A calendar month, written `YYYY-MM`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Month {
    pub year: i32,
    pub month: u32,
}

impl Month {
    /// Months from `today` to this month; zero for anything earlier.
    pub fn offset_from(self, today: Month) -> u32 {
        ((self.year - today.year) * 12 + self.month as i32 - today.month as i32).max(0) as u32
    }
}

impl TryFrom<String> for Month {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let invalid = || format!("expected a month like 2027-03, got {text:?}");
        let (year, month) = text.split_once('-').ok_or_else(invalid)?;
        let year = year.parse().map_err(|_| invalid())?;
        let month = month.parse().map_err(|_| invalid())?;
        if !(1..=12).contains(&month) {
            return Err(invalid());
        }
        Ok(Self { year, month })
    }
}

impl From<Month> for String {
    fn from(m: Month) -> Self {
        m.to_string()
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveKind {
    /// Invest `amount` at once.
    Lump,
    /// Invest `amount` in equal parts over `months` (cost averaging).
    Spread,
    /// Invest `amount` every month instead of the plan's monthly amount.
    Monthly,
    /// Take `amount` out.
    Withdraw,
}

/// One move being tried out, stored in the `.invest` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    pub kind: MoveKind,
    pub amount: Decimal,
    /// First month of the move; none means now.
    #[serde(default)]
    pub start: Option<Month>,
    /// Months to spread over; only for `Spread`.
    #[serde(default = "twelve")]
    pub months: u32,
    /// Use cash already held instead of new money; for `Lump` and `Spread`.
    #[serde(default)]
    pub from_cash: bool,
    /// Something other than the portfolio; for `Lump` and `Spread`.
    #[serde(default)]
    pub asset: Option<Asset>,
}

fn yes() -> bool {
    true
}

fn twelve() -> u32 {
    12
}

/// An investment with its own expectations, in percent a year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    /// Average return above inflation.
    pub real_return: Decimal,
    /// How much the value swings from year to year.
    pub volatility: Decimal,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WhatIfInput {
    /// Cash held today.
    #[serde(default)]
    pub cash: f64,
    /// Investments held today, at market value where known.
    #[serde(default)]
    pub invested: f64,
    pub today: Month,
    pub years: u32,
    pub market: Market,
    /// Take the money for savings goals out in their year.
    #[serde(default = "yes")]
    pub spend_goals: bool,
    #[serde(default = "default_paths")]
    pub paths: u32,
}

fn default_paths() -> u32 {
    1000
}

#[derive(Debug, Clone, Serialize)]
pub struct Band {
    pub year: u32,
    /// With the average return every month.
    pub steady: f64,
    pub p10: f64,
    pub p50: f64,
    pub p90: f64,
    /// New money added since today.
    pub put_in: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub bands: Vec<Band>,
    /// Share of futures where every withdrawal is fully paid; none without withdrawals.
    pub chance_covered: Option<f64>,
}

/// Money a savings goal takes out of the portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct Spending {
    pub goal: String,
    pub name: String,
    pub year: i32,
    pub amount: f64,
}

/// A move that wanted more cash than there was, with average returns.
#[derive(Debug, Clone, Serialize)]
pub struct CashShort {
    pub id: String,
    pub wanted: f64,
    pub available: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WhatIf {
    /// Invested every month by the plan.
    pub monthly: f64,
    pub spending: Vec<Spending>,
    pub plan: Outcome,
    /// None when no move is switched on.
    pub with_moves: Option<Outcome>,
    pub cash_short: Vec<CashShort>,
}

/// Money moving into one pot in one month.
struct Buy {
    pot: usize,
    amount: f64,
    from_cash: bool,
    id: usize,
}

/// What happens in each month, the same in every simulated future.
struct Schedule {
    new_money: Vec<f64>,
    buys: Vec<Vec<Buy>>,
    out: Vec<f64>,
}

impl Schedule {
    fn new(months: usize, monthly: f64) -> Self {
        Self { new_money: vec![monthly; months], buys: (0..months).map(|_| Vec::new()).collect(), out: vec![0.0; months] }
    }

    fn take_out(&mut self, at: u32, amount: f64) {
        if let Some(slot) = self.out.get_mut(at as usize) {
            *slot += amount;
        }
    }

    fn has_out(&self) -> bool {
        self.out.iter().any(|&o| o > 0.0)
    }
}

/// Monthly growth factors for one pot.
struct Draws {
    rng: Rng,
    mu: f64,
    sigma: f64,
    steady: f64,
}

impl Draws {
    fn new(seed: u64, mean: f64, volatility: f64) -> Self {
        let (mu, sigma) = log_params(mean, volatility);
        Self { rng: Rng(seed), mu: mu / 12.0, sigma: sigma / 12f64.sqrt(), steady: (1.0 + mean).powf(1.0 / 12.0) }
    }

    fn fill(&mut self, out: &mut [f64]) {
        for f in out {
            *f = (self.mu + self.sigma * self.rng.normal()).exp();
        }
    }
}

/// A stable seed per move, so switching one move on or off leaves the others' markets alone.
fn seed(id: &str) -> u64 {
    id.bytes().fold(0xCBF2_9CE4_8422_2325, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3))
}

/// Year-end totals of one future, and whether every withdrawal was paid.
fn run(s: &Schedule, cash: f64, invested: f64, growth: &[Vec<f64>], short: Option<&mut [(f64, f64)]>) -> (Vec<f64>, bool) {
    let cash_factor = (1.0 + CASH_REAL_RETURN).powf(1.0 / 12.0);
    let mut pots = vec![0.0; growth.len()];
    pots[0] = invested;
    let mut cash = cash;
    let mut totals = vec![cash + invested];
    let mut covered = true;
    let mut short = short;
    for t in 0..s.out.len() {
        pots[0] += s.new_money[t];
        for buy in &s.buys[t] {
            let amount = if buy.from_cash { buy.amount.min(cash.max(0.0)) } else { buy.amount };
            if buy.from_cash {
                cash -= amount;
            }
            if let Some(short) = short.as_deref_mut() {
                short[buy.id].0 += buy.amount;
                short[buy.id].1 += amount;
            }
            pots[buy.pot] += amount;
        }
        // Withdrawals come from investments first, then cash.
        let mut need = s.out[t];
        for pot in pots.iter_mut() {
            let take = need.min(*pot);
            *pot -= take;
            need -= take;
        }
        let take = need.min(cash.max(0.0));
        cash -= take;
        need -= take;
        if need > 0.005 {
            covered = false;
        }
        for (pot, g) in pots.iter_mut().zip(growth) {
            *pot *= g[t];
        }
        cash *= cash_factor;
        if (t + 1) % 12 == 0 {
            totals.push(cash + pots.iter().sum::<f64>());
        }
    }
    (totals, covered)
}

pub fn what_if(input: &WhatIfInput, plan: &Plan) -> WhatIf {
    let months = input.years as usize * 12;
    let profile = plan.profile.as_ref();
    let monthly = profile.and_then(|p| p.monthly_investing).map_or(0.0, to_f64);
    let risk = profile.map_or(Risk::Balanced, |p| p.risk);
    let at = |start: Option<Month>| start.map_or(0, |m| m.offset_from(input.today));

    let mut spending = Vec::new();
    if input.spend_goals {
        for goal in plan.goals.iter().filter(|g| g.kind == GoalKind::Savings) {
            let Some(year) = goal.year.filter(|&y| y >= input.today.year) else { continue };
            let offset = Month { year, month: 1 }.offset_from(input.today);
            if (offset as usize) < months {
                spending.push(Spending { goal: goal.id.clone(), name: goal.name.clone(), year, amount: to_f64(goal.target) });
            }
        }
    }

    let mut base = Schedule::new(months, monthly);
    for s in &spending {
        base.take_out(Month { year: s.year, month: 1 }.offset_from(input.today), s.amount);
    }

    // The plan plus every move that is switched on. Pot 0 is the portfolio;
    // each move into another investment gets a pot of its own.
    let moves: Vec<&Move> = plan.what_if.iter().filter(|m| m.enabled && m.amount > Decimal::ZERO).collect();
    let mut with = Schedule::new(months, monthly);
    with.out.clone_from(&base.out);
    let mut assets: Vec<(u64, f64, f64)> = Vec::new();
    // A later monthly change replaces an earlier one.
    let mut monthly_changes: Vec<(u32, f64)> = moves.iter().filter(|m| m.kind == MoveKind::Monthly).map(|m| (at(m.start), to_f64(m.amount))).collect();
    monthly_changes.sort_by_key(|&(from, _)| from);
    for (from, amount) in monthly_changes {
        for slot in with.new_money.iter_mut().skip(from as usize) {
            *slot = amount;
        }
    }
    for (id, m) in moves.iter().enumerate() {
        let amount = to_f64(m.amount);
        let pot = match &m.asset {
            Some(a) if matches!(m.kind, MoveKind::Lump | MoveKind::Spread) => {
                assets.push((seed(&m.id), to_f64(a.real_return) / 100.0, to_f64(a.volatility) / 100.0));
                assets.len()
            }
            _ => 0,
        };
        let start = at(m.start) as usize;
        let parts = match m.kind {
            MoveKind::Lump => 1,
            MoveKind::Spread => m.months.max(1) as usize,
            MoveKind::Withdraw => {
                with.take_out(start as u32, amount);
                continue;
            }
            MoveKind::Monthly => continue,
        };
        for t in (start..start + parts).filter(|&t| t < months) {
            with.buys[t].push(Buy { pot, amount: amount / parts as f64, from_cash: m.from_cash, id });
        }
    }

    let mut market = Draws::new(0x1DC0_FFEE, input.market.real_return(), risk.volatility());
    let mut others: Vec<Draws> = assets.iter().map(|&(seed, mean, vol)| Draws::new(seed, mean, vol)).collect();
    let mut growth: Vec<Vec<f64>> = vec![vec![0.0; months]; 1 + others.len()];

    let paths = input.paths.max(1) as usize;
    let years = input.years as usize;
    let mut plan_years = vec![Vec::with_capacity(paths); years + 1];
    let mut with_years = vec![Vec::with_capacity(paths); years + 1];
    let (mut plan_covered, mut with_covered) = (0usize, 0usize);
    for _ in 0..paths {
        market.fill(&mut growth[0]);
        for (draws, g) in others.iter_mut().zip(growth.iter_mut().skip(1)) {
            draws.fill(g);
        }
        let (totals, covered) = run(&base, input.cash, input.invested, &growth[..1], None);
        plan_covered += usize::from(covered);
        for (slot, v) in plan_years.iter_mut().zip(totals) {
            slot.push(v);
        }
        if !moves.is_empty() {
            let (totals, covered) = run(&with, input.cash, input.invested, &growth, None);
            with_covered += usize::from(covered);
            for (slot, v) in with_years.iter_mut().zip(totals) {
                slot.push(v);
            }
        }
    }

    let steady_growth = |draws: &[&Draws]| draws.iter().map(|d| vec![d.steady; months]).collect::<Vec<_>>();
    let outcome = |s: &Schedule, by_year: Vec<Vec<f64>>, steady: Vec<f64>, covered: usize| {
        let mut put_in = 0.0;
        let bands = by_year
            .into_iter()
            .zip(steady)
            .enumerate()
            .map(|(year, (mut values, steady))| {
                if year > 0 {
                    let months = (year - 1) * 12..year * 12;
                    put_in += s.new_money[months.clone()].iter().sum::<f64>();
                    put_in += s.buys[months].iter().flatten().filter(|b| !b.from_cash).map(|b| b.amount).sum::<f64>();
                }
                values.sort_by(f64::total_cmp);
                Band { year: year as u32, steady, p10: percentile(&values, 0.1), p50: percentile(&values, 0.5), p90: percentile(&values, 0.9), put_in }
            })
            .collect();
        Outcome { bands, chance_covered: s.has_out().then(|| covered as f64 / paths as f64) }
    };

    let (plan_steady, _) = run(&base, input.cash, input.invested, &steady_growth(&[&market]), None);
    let plan_outcome = outcome(&base, plan_years, plan_steady, plan_covered);

    let mut cash_short = Vec::new();
    let with_moves = (!moves.is_empty()).then(|| {
        let all: Vec<&Draws> = std::iter::once(&market).chain(others.iter()).collect();
        let mut short = vec![(0.0, 0.0); moves.len()];
        let (steady, _) = run(&with, input.cash, input.invested, &steady_growth(&all), Some(&mut short));
        for (m, (wanted, got)) in moves.iter().zip(short) {
            if m.from_cash && got + 1.0 < wanted {
                cash_short.push(CashShort { id: m.id.clone(), wanted, available: got });
            }
        }
        outcome(&with, with_years, steady, with_covered)
    });

    WhatIf { monthly, spending, plan: plan_outcome, with_moves, cash_short }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{Goal, Profile};

    fn plan(moves: Vec<Move>) -> Plan {
        Plan {
            profile: Some(Profile {
                birth_year: None,
                monthly_spending: None,
                monthly_investing: Some(Decimal::from(500)),
                risk: Risk::Balanced,
                retirement_age: None,
                pension_age: None,
                monthly_pension: None,
                retirement_spending: None,
            }),
            goals: vec![Goal {
                id: "house".into(),
                name: "House".into(),
                kind: GoalKind::Savings,
                target: Decimal::from(20_000),
                year: Some(2030),
                monthly: Decimal::ZERO,
            }],
            what_if: moves,
        }
    }

    fn input(spend_goals: bool) -> WhatIfInput {
        WhatIfInput {
            cash: 20_000.0,
            invested: 30_000.0,
            today: Month { year: 2026, month: 10 },
            years: 20,
            market: Market::Base,
            spend_goals,
            paths: 400,
        }
    }

    fn lump(id: &str, kind: MoveKind, from_cash: bool) -> Move {
        Move { id: id.into(), name: String::new(), enabled: true, kind, amount: Decimal::from(15_000), start: None, months: 12, from_cash, asset: None }
    }

    #[test]
    fn months_parse_and_count() {
        let m: Month = serde_json::from_str("\"2027-03\"").unwrap();
        assert_eq!(m, Month { year: 2027, month: 3 });
        assert_eq!(serde_json::to_string(&m).unwrap(), "\"2027-03\"");
        assert_eq!(m.offset_from(Month { year: 2026, month: 10 }), 5);
        assert_eq!(Month { year: 2026, month: 1 }.offset_from(Month { year: 2026, month: 10 }), 0);
        assert!(serde_json::from_str::<Month>("\"2027-13\"").is_err());
    }

    #[test]
    fn plan_counts_cash_and_goal_spending() {
        let r = what_if(&input(false), &plan(vec![]));
        assert!(r.with_moves.is_none() && r.spending.is_empty());
        let first = &r.plan.bands[1];
        // A year of 500 a month on 30,000 invested at 3.5%, and 20,000 cash losing 2%.
        let expected = 30_000.0 * 1.035 + 6_000.0 + 20_000.0 * 0.98;
        assert!((first.steady - expected).abs() < 150.0, "{} vs {expected}", first.steady);
        assert_eq!(first.put_in, 6_000.0);

        let spent = what_if(&input(true), &plan(vec![]));
        assert_eq!(spent.spending.len(), 1);
        assert!(spent.plan.chance_covered.unwrap() > 0.9);
        let last = |w: &WhatIf| w.plan.bands.last().unwrap().p50;
        assert!(last(&spent) < last(&r) - 20_000.0);
    }

    #[test]
    fn investing_idle_cash_helps_on_average() {
        let r = what_if(&input(false), &plan(vec![lump("a", MoveKind::Lump, true)]));
        let with = r.with_moves.unwrap();
        let (p, w) = (r.plan.bands.last().unwrap(), with.bands.last().unwrap());
        assert!(w.p50 > p.p50 && w.steady > p.steady);
        // Moving cash is not new money.
        assert_eq!(w.put_in, p.put_in);
        assert!(r.cash_short.is_empty());

        // New money adds to what was put in.
        let r = what_if(&input(false), &plan(vec![lump("a", MoveKind::Lump, false)]));
        assert_eq!(r.with_moves.unwrap().bands[1].put_in, 6_000.0 + 15_000.0);
    }

    #[test]
    fn lump_sum_beats_spreading_on_average_but_spreading_softens_the_bad_case_early() {
        let lumped = what_if(&WhatIfInput { years: 1, ..input(false) }, &plan(vec![lump("a", MoveKind::Lump, false)])).with_moves.unwrap();
        let spread = what_if(&WhatIfInput { years: 1, ..input(false) }, &plan(vec![lump("a", MoveKind::Spread, false)])).with_moves.unwrap();
        let (l, s) = (&lumped.bands[1], &spread.bands[1]);
        assert!(l.steady > s.steady);
        assert!(l.p90 - l.p10 > s.p90 - s.p10);
    }

    #[test]
    fn short_cash_is_reported_and_disabled_moves_ignored() {
        let mut big = lump("big", MoveKind::Lump, true);
        big.amount = Decimal::from(50_000);
        let r = what_if(&input(false), &plan(vec![big.clone()]));
        assert_eq!(r.cash_short.len(), 1);
        assert!((r.cash_short[0].available - 20_000.0).abs() < 0.01);

        big.enabled = false;
        assert!(what_if(&input(false), &plan(vec![big])).with_moves.is_none());
    }

    #[test]
    fn withdrawals_beyond_the_money_are_not_covered() {
        let mut out = lump("out", MoveKind::Withdraw, false);
        out.amount = Decimal::from(500_000);
        let r = what_if(&input(false), &plan(vec![out]));
        assert_eq!(r.with_moves.unwrap().chance_covered, Some(0.0));
        assert_eq!(r.plan.chance_covered, None);
    }

    #[test]
    fn a_monthly_change_replaces_the_plan_from_its_start() {
        let mut more = lump("more", MoveKind::Monthly, false);
        more.amount = Decimal::from(1_000);
        more.start = Some(Month { year: 2027, month: 4 });
        let r = what_if(&input(false), &plan(vec![more]));
        // Six months at 500, then six at 1,000.
        assert_eq!(r.with_moves.unwrap().bands[1].put_in, 9_000.0);
    }
}
