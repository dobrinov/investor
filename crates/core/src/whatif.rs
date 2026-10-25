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

use crate::plan::{GoalKind, Market, Plan, RetirementYear, Risk, Rng, log_params, percentile, to_f64};

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
    /// Plan retirement too, when the profile allows.
    #[serde(default)]
    pub retirement: Option<RetireInput>,
    #[serde(default = "default_paths")]
    pub paths: u32,
}

/// Money in today's terms; ages in years.
#[derive(Debug, Clone, Deserialize)]
pub struct RetireInput {
    pub age_now: u32,
    pub retire_at: u32,
    pub plan_until: u32,
    pub monthly_spending: f64,
    pub pension_from: u32,
    pub monthly_pension: f64,
}

impl RetireInput {
    /// What the portfolio must pay each month at `age`.
    fn need(&self, age: u32, spending: f64) -> f64 {
        let pension = if age >= self.pension_from { self.monthly_pension } else { 0.0 };
        (spending - pension).max(0.0)
    }
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
    pub retirement: Option<Retirements>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Retirements {
    pub plan: Retirement,
    /// None when no move is switched on.
    pub with_moves: Option<Retirement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Retirement {
    /// Portfolio on the day you retire, with average returns.
    pub at_retirement: f64,
    /// Age the money runs out with average returns; none if it lasts.
    pub runs_out_at: Option<u32>,
    /// Share of simulated futures where the money lasts to the end of the plan.
    pub chance_lasts: f64,
    /// Highest monthly spending that lasts in 85% of simulated futures.
    pub safe_monthly_spending: f64,
    /// Balance at each age, with average returns.
    pub path: Vec<(u32, f64)>,
    pub years: Vec<RetirementYear>,
}

/// Money moving into one pot in one month.
#[derive(Clone)]
struct Buy {
    pot: usize,
    amount: f64,
    from_cash: bool,
    id: usize,
}

/// What happens in each month, the same in every simulated future.
#[derive(Clone)]
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

    /// New money added in each year.
    fn added(&self) -> Vec<f64> {
        (0..self.out.len() / 12)
            .map(|y| {
                let months = y * 12..(y + 1) * 12;
                self.new_money[months.clone()].iter().sum::<f64>()
                    + self.buys[months].iter().flatten().filter(|b| !b.from_cash).map(|b| b.amount).sum::<f64>()
            })
            .collect()
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

/// One simulated future.
struct Run {
    /// Total at the start and at the end of every year.
    totals: Vec<f64>,
    /// Withdrawals actually paid in each year.
    paid: Vec<f64>,
    /// First month a withdrawal could not be paid in full.
    first_short: Option<usize>,
}

fn run(s: &Schedule, cash: f64, invested: f64, growth: &[Vec<f64>], short: Option<&mut [(f64, f64)]>) -> Run {
    let cash_factor = (1.0 + CASH_REAL_RETURN).powf(1.0 / 12.0);
    let mut pots = vec![0.0; growth.len()];
    pots[0] = invested;
    let mut cash = cash;
    let mut totals = vec![cash + invested];
    let mut paid = vec![0.0; s.out.len() / 12];
    let mut first_short = None;
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
        if let Some(p) = paid.get_mut(t / 12) {
            *p += s.out[t] - need;
        }
        if need > 0.005 && first_short.is_none() {
            first_short = Some(t);
        }
        for (pot, g) in pots.iter_mut().zip(growth) {
            *pot *= g[t];
        }
        cash *= cash_factor;
        if (t + 1) % 12 == 0 {
            totals.push(cash + pots.iter().sum::<f64>());
        }
    }
    Run { totals, paid, first_short }
}

/// The plan, and the plan with every move switched on, over `months`.
struct Scenario {
    base: Schedule,
    /// None when no move is switched on.
    with: Option<Schedule>,
    /// Expected return and swings of each pot after the portfolio.
    assets: Vec<(u64, f64, f64)>,
    moves: Vec<(String, bool)>,
    spending: Vec<Spending>,
    monthly: f64,
    risk: Risk,
}

impl Scenario {
    fn new(input: &WhatIfInput, plan: &Plan, months: usize) -> Self {
        let profile = plan.profile.as_ref();
        let monthly = profile.and_then(|p| p.monthly_investing).map_or(0.0, to_f64);
        let risk = profile.map_or(Risk::Balanced, |p| p.risk);
        let at = |start: Option<Month>| start.map_or(0, |m| m.offset_from(input.today));

        let mut spending = Vec::new();
        if input.spend_goals {
            for goal in plan.goals.iter().filter(|g| g.kind == GoalKind::Savings) {
                let Some(year) = goal.year.filter(|&y| y >= input.today.year) else { continue };
                if (Month { year, month: 1 }.offset_from(input.today) as usize) < months {
                    spending.push(Spending { goal: goal.id.clone(), name: goal.name.clone(), year, amount: to_f64(goal.target) });
                }
            }
        }
        let mut base = Schedule::new(months, monthly);
        for s in &spending {
            base.take_out(Month { year: s.year, month: 1 }.offset_from(input.today), s.amount);
        }

        // Pot 0 is the portfolio; each move into another investment gets a pot of its own.
        let moves: Vec<&Move> = plan.what_if.iter().filter(|m| m.enabled && m.amount > Decimal::ZERO).collect();
        let mut with = base.clone();
        let mut assets = Vec::new();
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
        Self {
            with: (!moves.is_empty()).then_some(with),
            base,
            assets,
            moves: moves.iter().map(|m| (m.id.clone(), m.from_cash)).collect(),
            spending,
            monthly,
            risk,
        }
    }

    fn draws(&self, market: Market) -> (Draws, Vec<Draws>) {
        let market = Draws::new(0x1DC0_FFEE, market.real_return(), self.risk.volatility());
        (market, self.assets.iter().map(|&(seed, mean, vol)| Draws::new(seed, mean, vol)).collect())
    }

    /// Monthly growth factors for every pot in `paths` futures: `[path][pot][month]`.
    fn futures(&self, market: Market, paths: usize, months: usize) -> Vec<Vec<Vec<f64>>> {
        let (mut market, mut others) = self.draws(market);
        (0..paths)
            .map(|_| {
                let mut growth = vec![vec![0.0; months]; 1 + others.len()];
                market.fill(&mut growth[0]);
                for (draws, g) in others.iter_mut().zip(growth.iter_mut().skip(1)) {
                    draws.fill(g);
                }
                growth
            })
            .collect()
    }

    /// Growth with the average return every month.
    fn steady(&self, market: Market, months: usize) -> Vec<Vec<f64>> {
        let (market, others) = self.draws(market);
        std::iter::once(&market).chain(others.iter()).map(|d| vec![d.steady; months]).collect()
    }
}

pub fn what_if(input: &WhatIfInput, plan: &Plan) -> WhatIf {
    let months = input.years as usize * 12;
    let paths = input.paths.max(1) as usize;
    let scenario = Scenario::new(input, plan, months);
    let futures = scenario.futures(input.market, paths, months);
    let steady = scenario.steady(input.market, months);

    let outcome = |s: &Schedule, pots: usize| {
        let runs: Vec<Run> = futures.iter().map(|g| run(s, input.cash, input.invested, &g[..pots], None)).collect();
        let steady = run(s, input.cash, input.invested, &steady[..pots], None);
        let mut put_in = 0.0;
        let added = s.added();
        let bands = (0..=input.years as usize)
            .map(|year| {
                if year > 0 {
                    put_in += added[year - 1];
                }
                let mut values: Vec<f64> = runs.iter().map(|r| r.totals[year]).collect();
                values.sort_by(f64::total_cmp);
                Band { year: year as u32, steady: steady.totals[year], p10: percentile(&values, 0.1), p50: percentile(&values, 0.5), p90: percentile(&values, 0.9), put_in }
            })
            .collect();
        let covered = runs.iter().filter(|r| r.first_short.is_none()).count();
        Outcome { bands, chance_covered: s.has_out().then(|| covered as f64 / paths as f64) }
    };

    let mut cash_short = Vec::new();
    let with_moves = scenario.with.as_ref().map(|with| {
        let mut short = vec![(0.0, 0.0); scenario.moves.len()];
        run(with, input.cash, input.invested, &steady, Some(&mut short));
        for ((id, from_cash), (wanted, got)) in scenario.moves.iter().zip(short) {
            if *from_cash && got + 1.0 < wanted {
                cash_short.push(CashShort { id: id.clone(), wanted, available: got });
            }
        }
        outcome(with, steady.len())
    });

    let retirement = input.retirement.as_ref().map(|r| {
        let months = r.plan_until.saturating_sub(r.age_now) as usize * 12;
        let scenario = Scenario::new(input, plan, months);
        let futures = scenario.futures(input.market, paths.min(500), months);
        let steady = scenario.steady(input.market, months);
        let outcome = |s: &Schedule, pots: usize| retire(s, r, input, &futures, &steady, pots);
        Retirements { plan: outcome(&scenario.base, 1), with_moves: scenario.with.as_ref().map(|w| outcome(w, steady.len())) }
    });

    WhatIf { monthly: scenario.monthly, spending: scenario.spending.clone(), plan: outcome(&scenario.base, 1), with_moves, cash_short, retirement }
}

/// Saving stops at retirement; from then on spending, less the pension, is taken out.
fn retirement_schedule(s: &Schedule, r: &RetireInput, spending: f64) -> Schedule {
    let mut s = s.clone();
    let from = r.retire_at.saturating_sub(r.age_now) as usize * 12;
    for t in from..s.out.len() {
        let age = r.age_now + (t / 12) as u32;
        s.new_money[t] = 0.0;
        s.out[t] += r.need(age, spending);
    }
    s
}

fn retire(s: &Schedule, r: &RetireInput, input: &WhatIfInput, futures: &[Vec<Vec<f64>>], steady: &[Vec<f64>], pots: usize) -> Retirement {
    let (cash, invested) = (input.cash, input.invested);
    let chance = |spending: f64, futures: &[Vec<Vec<f64>>]| {
        let s = retirement_schedule(s, r, spending);
        let lasted = futures.iter().filter(|g| run(&s, cash, invested, &g[..pots], None).first_short.is_none()).count();
        lasted as f64 / futures.len().max(1) as f64
    };

    let schedule = retirement_schedule(s, r, r.monthly_spending);
    let added = schedule.added();
    let Run { totals, paid, first_short } = run(&schedule, cash, invested, &steady[..pots], None);
    let retire_year = r.retire_at.saturating_sub(r.age_now) as usize;
    let years = (retire_year..paid.len())
        .map(|k| {
            let age = r.age_now + k as u32;
            let (start, end, withdrawn) = (totals[k], totals[k + 1], paid[k]);
            RetirementYear {
                age,
                start,
                growth: end - start + withdrawn - added[k],
                withdrawn,
                end,
                from_portfolio_monthly: withdrawn.min(r.need(age, r.monthly_spending) * 12.0) / 12.0,
                pension_monthly: if age >= r.pension_from { r.monthly_pension } else { 0.0 },
                covered: first_short.is_none_or(|m| m >= (k + 1) * 12),
            }
        })
        .collect();

    // Bisect for the spending that lasts in 85% of futures.
    let sample = &futures[..futures.len().min(300)];
    let (mut low, mut high) = (0.0, (r.monthly_spending * 3.0).max(r.monthly_pension + 10_000.0));
    for _ in 0..18 {
        let mid = (low + high) / 2.0;
        if chance(mid, sample) >= 0.85 { low = mid } else { high = mid }
    }

    Retirement {
        at_retirement: totals[retire_year.min(totals.len() - 1)],
        runs_out_at: first_short.map(|m| r.age_now + (m / 12) as u32),
        chance_lasts: chance(r.monthly_spending, futures),
        safe_monthly_spending: low,
        path: totals.iter().enumerate().map(|(k, &v)| (r.age_now + k as u32, v)).collect(),
        years,
    }
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
            retirement: None,
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

    fn retiring(spending: f64) -> WhatIfInput {
        WhatIfInput {
            retirement: Some(RetireInput { age_now: 40, retire_at: 60, plan_until: 95, monthly_spending: spending, pension_from: 65, monthly_pension: 500.0 }),
            ..input(false)
        }
    }

    #[test]
    fn money_runs_out_when_spending_outpaces_it() {
        let mut bare = plan(vec![]);
        bare.profile.as_mut().unwrap().monthly_investing = Some(Decimal::ZERO);
        let input = WhatIfInput {
            cash: 0.0,
            invested: 120_000.0,
            market: Market::Bear,
            retirement: Some(RetireInput { age_now: 60, retire_at: 60, plan_until: 95, monthly_spending: 1_000.0, pension_from: 100, monthly_pension: 0.0 }),
            ..input(false)
        };
        // 120,000 at 12,000 a year lasts a little over ten years at 1%.
        let r = what_if(&input, &bare).retirement.unwrap().plan;
        assert!(matches!(r.runs_out_at, Some(70..=72)), "{:?}", r.runs_out_at);
        assert!(r.years.iter().take(10).all(|y| y.covered));
        assert!(r.chance_lasts < 0.5);
        assert!(r.safe_monthly_spending < 1_000.0);
    }

    #[test]
    fn saving_stops_at_retirement() {
        let r = what_if(&retiring(1_500.0), &plan(vec![])).retirement.unwrap().plan;
        assert_eq!(r.path.first().unwrap().0, 40);
        assert_eq!(r.path.last().unwrap().0, 95);
        assert_eq!(r.years.first().unwrap().age, 60);
        // From 65 the pension covers 500 of the 1,500.
        let y65 = r.years.iter().find(|y| y.age == 65).unwrap();
        assert!((y65.from_portfolio_monthly - 1_000.0).abs() < 0.01 || !y65.covered);
        assert_eq!(y65.pension_monthly, 500.0);
    }

    #[test]
    fn a_lump_sum_lifts_retirement() {
        let r = what_if(&retiring(1_800.0), &plan(vec![lump("a", MoveKind::Lump, false)])).retirement.unwrap();
        let with = r.with_moves.unwrap();
        assert!(with.at_retirement > r.plan.at_retirement + 15_000.0);
        assert!(with.chance_lasts >= r.plan.chance_lasts);
        assert!(with.safe_monthly_spending > r.plan.safe_monthly_spending);
    }
}
