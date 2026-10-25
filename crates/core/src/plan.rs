//! Goals, projections and retirement planning.
//!
//! Projections are estimates, so they use `f64` rather than exact decimals.
//! All figures are in today's money: returns are real (after inflation and
//! fund costs). Simulations use a fixed seed, so the same inputs always give
//! the same numbers.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

/// What the user is saving for, stored in the `.invest` file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    #[serde(default)]
    pub profile: Option<Profile>,
    #[serde(default)]
    pub goals: Vec<Goal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub birth_year: i32,
    pub monthly_spending: Decimal,
    pub monthly_investing: Decimal,
    pub risk: Risk,
    pub retirement_age: u32,
    pub pension_age: u32,
    /// Expected state pension per month, in today's money.
    pub monthly_pension: Decimal,
    /// Spending per month once retired, in today's money.
    pub retirement_spending: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Cautious,
    Balanced,
    Growth,
}

impl Risk {
    /// Yearly volatility of a typical portfolio at this risk level.
    pub fn volatility(self) -> f64 {
        match self {
            Self::Cautious => 0.07,
            Self::Balanced => 0.11,
            Self::Growth => 0.15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Market {
    Bear,
    Base,
    Bull,
}

impl Market {
    /// Average yearly return after inflation and fund costs.
    pub fn real_return(self) -> f64 {
        match self {
            Self::Bear => 0.01,
            Self::Base => 0.035,
            Self::Bull => 0.06,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub kind: GoalKind,
    /// In today's money. For an emergency fund, months of spending.
    pub target: Decimal,
    /// Year the money is needed. Not used for an emergency fund.
    #[serde(default)]
    pub year: Option<i32>,
    /// Put towards this goal each month.
    #[serde(default)]
    pub monthly: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalKind {
    /// Months of spending kept in cash.
    EmergencyFund,
    /// An amount needed by a year.
    Savings,
}

/// SplitMix64 with Box–Muller normals: small, fast and reproducible.
struct Rng(u64);

impl Rng {
    fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        // 53 random bits in (0, 1).
        ((z >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }

    fn normal(&mut self) -> f64 {
        let (u1, u2) = (self.next_f64(), self.next_f64());
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// Yearly returns drawn so that their average matches `mean`.
struct Returns {
    rng: Rng,
    mu: f64,
    sigma: f64,
}

impl Returns {
    fn new(mean: f64, volatility: f64) -> Self {
        // Log-normal: E[1 + r] = 1 + mean.
        let sigma = (1.0 + (volatility / (1.0 + mean)).powi(2)).ln().sqrt();
        let mu = (1.0 + mean).ln() - sigma * sigma / 2.0;
        Self { rng: Rng(0x1DC0_FFEE), mu, sigma }
    }

    fn next(&mut self) -> f64 {
        (self.mu + self.sigma * self.rng.normal()).exp() - 1.0
    }
}

/// One year of saving: growth on the balance, and on contributions spread over the year.
fn grow(balance: f64, yearly_contribution: f64, r: f64) -> f64 {
    balance * (1.0 + r) + yearly_contribution * (1.0 + r / 2.0)
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let i = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[i]
}

pub fn to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectionInput {
    pub start: f64,
    pub monthly: f64,
    pub years: u32,
    pub market: Market,
    pub risk: Risk,
    #[serde(default)]
    pub target: Option<f64>,
    #[serde(default = "default_paths")]
    pub paths: u32,
}

fn default_paths() -> u32 {
    2000
}

#[derive(Debug, Clone, Serialize)]
pub struct YearBand {
    pub year: u32,
    /// With the average return every year.
    pub steady: f64,
    pub p10: f64,
    pub p50: f64,
    pub p90: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Projection {
    pub bands: Vec<YearBand>,
    /// Share of simulated futures ending at or above the target.
    pub chance_of_target: Option<f64>,
}

pub fn project(input: &ProjectionInput) -> Projection {
    let years = input.years as usize;
    let paths = input.paths.max(1) as usize;
    let yearly = input.monthly * 12.0;
    let mean = input.market.real_return();
    let mut returns = Returns::new(mean, input.risk.volatility());

    let mut by_year = vec![Vec::with_capacity(paths); years + 1];
    let mut hits = 0usize;
    for _ in 0..paths {
        let mut balance = input.start;
        by_year[0].push(balance);
        for slot in by_year.iter_mut().skip(1) {
            balance = grow(balance, yearly, returns.next()).max(0.0);
            slot.push(balance);
        }
        if input.target.is_some_and(|t| balance >= t) {
            hits += 1;
        }
    }

    let mut steady = input.start;
    let bands = by_year
        .into_iter()
        .enumerate()
        .map(|(year, mut values)| {
            if year > 0 {
                steady = grow(steady, yearly, mean);
            }
            values.sort_by(f64::total_cmp);
            YearBand { year: year as u32, steady, p10: percentile(&values, 0.1), p50: percentile(&values, 0.5), p90: percentile(&values, 0.9) }
        })
        .collect();
    Projection { bands, chance_of_target: input.target.map(|_| hits as f64 / paths as f64) }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetirementInput {
    pub start: f64,
    pub age_now: u32,
    pub retire_at: u32,
    pub plan_until: u32,
    pub monthly_investing: f64,
    pub monthly_spending: f64,
    pub pension_from: u32,
    pub monthly_pension: f64,
    pub market: Market,
    pub risk: Risk,
    #[serde(default = "default_paths")]
    pub paths: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RetirementYear {
    pub age: u32,
    pub start: f64,
    pub growth: f64,
    pub withdrawn: f64,
    pub end: f64,
    pub from_portfolio_monthly: f64,
    pub pension_monthly: f64,
    /// False once the portfolio can no longer cover spending.
    pub covered: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Retirement {
    /// Portfolio on the day you retire, with average returns.
    pub at_retirement: f64,
    /// Age the money runs out with average returns; none if it lasts.
    pub runs_out_at: Option<u32>,
    /// Share of simulated futures where the money lasts to `plan_until`.
    pub chance_lasts: f64,
    /// Highest monthly spending that lasts in 85% of simulated futures.
    pub safe_monthly_spending: f64,
    /// Balance at each age, with average returns.
    pub path: Vec<(u32, f64)>,
    pub years: Vec<RetirementYear>,
}

impl RetirementInput {
    fn need(&self, age: u32, spending: f64) -> f64 {
        let pension = if age >= self.pension_from { self.monthly_pension } else { 0.0 };
        ((spending - pension) * 12.0).max(0.0)
    }

    /// Simulates every path; returns the share where the money lasts.
    fn chance(&self, spending: f64, paths: usize) -> f64 {
        let mut returns = Returns::new(self.market.real_return(), self.risk.volatility());
        let mut lasted = 0;
        for _ in 0..paths {
            let mut balance = self.start;
            let mut ok = true;
            for age in self.age_now..self.plan_until {
                let r = returns.next();
                if age < self.retire_at {
                    balance = grow(balance, self.monthly_investing * 12.0, r);
                } else {
                    let need = self.need(age, spending);
                    balance = balance * (1.0 + r) - need * (1.0 + r / 2.0);
                    if balance < 0.0 {
                        ok = false;
                        break;
                    }
                }
            }
            lasted += usize::from(ok);
        }
        lasted as f64 / paths as f64
    }
}

pub fn retirement(input: &RetirementInput) -> Retirement {
    let mean = input.market.real_return();
    let mut balance = input.start;
    let mut path = vec![(input.age_now, balance)];
    let mut years = Vec::new();
    let mut runs_out_at = None;
    let mut at_retirement = if input.retire_at <= input.age_now { balance } else { 0.0 };

    for age in input.age_now..input.plan_until {
        if age < input.retire_at {
            balance = grow(balance, input.monthly_investing * 12.0, mean);
            if age + 1 == input.retire_at {
                at_retirement = balance;
            }
        } else {
            let start = balance;
            let growth = start * mean;
            let need = input.need(age, input.monthly_spending);
            let available = (start + growth * 0.5).max(0.0);
            let withdrawn = need.min(available);
            balance = (start * (1.0 + mean) - withdrawn * (1.0 + mean / 2.0)).max(0.0);
            let covered = withdrawn + 0.005 >= need;
            if !covered && runs_out_at.is_none() {
                runs_out_at = Some(age);
            }
            let pension = if age >= input.pension_from { input.monthly_pension } else { 0.0 };
            years.push(RetirementYear {
                age,
                start,
                growth,
                withdrawn,
                end: balance,
                from_portfolio_monthly: withdrawn / 12.0,
                pension_monthly: pension,
                covered,
            });
        }
        path.push((age + 1, balance));
    }

    let paths = input.paths.max(1) as usize;
    let chance_lasts = input.chance(input.monthly_spending, paths);

    // Bisect for the spending that lasts in 85% of futures.
    let (mut low, mut high) = (0.0, (input.monthly_spending * 3.0).max(input.monthly_pension + 10_000.0));
    for _ in 0..24 {
        let mid = (low + high) / 2.0;
        if input.chance(mid, paths.min(1000)) >= 0.85 { low = mid } else { high = mid }
    }

    Retirement { at_retirement, runs_out_at, chance_lasts, safe_monthly_spending: low, path, years }
}

#[derive(Debug, Clone, Serialize)]
pub struct GoalStatus {
    pub id: String,
    /// Money assigned to the goal today.
    pub have: f64,
    pub target: f64,
    /// Chance of reaching the target by its year, if it has one.
    pub chance: Option<f64>,
    /// Monthly saving needed with average returns.
    pub needed_monthly: Option<f64>,
    pub on_track: bool,
}

/// Assigns today's money to goals in priority order — emergency funds from
/// cash first, then savings goals by year — and estimates each one.
pub fn evaluate_goals(plan: &Plan, cash: f64, invested: f64, this_year: i32, market: Market) -> Vec<GoalStatus> {
    let risk = plan.profile.as_ref().map_or(Risk::Balanced, |p| p.risk);
    let spending = plan.profile.as_ref().map_or(0.0, |p| to_f64(p.monthly_spending));
    let mut cash_left = cash;
    let mut pool = invested;
    let mut statuses = Vec::with_capacity(plan.goals.len());

    let mut order: Vec<usize> = (0..plan.goals.len()).collect();
    order.sort_by_key(|&i| {
        let g = &plan.goals[i];
        (g.kind != GoalKind::EmergencyFund, g.year.unwrap_or(i32::MAX))
    });
    for i in order {
        let goal = &plan.goals[i];
        let monthly = to_f64(goal.monthly);
        let status = match goal.kind {
            GoalKind::EmergencyFund => {
                let target = to_f64(goal.target) * spending;
                let have = cash_left.min(target);
                cash_left -= have;
                GoalStatus { id: goal.id.clone(), have, target, chance: None, needed_monthly: None, on_track: have + 0.005 >= target }
            }
            GoalKind::Savings => {
                pool += cash_left;
                cash_left = 0.0;
                let target = to_f64(goal.target);
                let have = pool.min(target);
                pool -= have;
                let years = goal.year.map_or(0, |y| (y - this_year).max(0) as u32);
                let projection = project(&ProjectionInput {
                    start: have,
                    monthly,
                    years,
                    market,
                    risk,
                    target: Some(target),
                    paths: 2000,
                });
                let needed = needed_monthly(have, target, years, market.real_return());
                GoalStatus {
                    id: goal.id.clone(),
                    have,
                    target,
                    chance: projection.chance_of_target,
                    needed_monthly: Some(needed),
                    on_track: monthly + 0.005 >= needed,
                }
            }
        };
        statuses.push(status);
    }
    statuses
}

/// Monthly saving that reaches `target` in `years` at a steady return.
pub fn needed_monthly(have: f64, target: f64, years: u32, r: f64) -> f64 {
    if years == 0 {
        return (target - have).max(0.0);
    }
    let growth = (1.0 + r).powi(years as i32);
    let shortfall = target - have * growth;
    if shortfall <= 0.0 {
        return 0.0;
    }
    // Sum of yearly contributions grown with mid-year timing, as in `grow`.
    let factor: f64 = (0..years).map(|k| (1.0 + r / 2.0) * (1.0 + r).powi(k as i32)).sum();
    shortfall / factor / 12.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_is_reproducible_and_ordered() {
        let input = ProjectionInput { start: 10_000.0, monthly: 500.0, years: 20, market: Market::Base, risk: Risk::Balanced, target: Some(250_000.0), paths: 2000 };
        let a = project(&input);
        let b = project(&input);
        assert_eq!(a.bands.last().unwrap().p50, b.bands.last().unwrap().p50);
        let last = a.bands.last().unwrap();
        assert!(last.p10 < last.p50 && last.p50 < last.p90);
        // The median of a log-normal sits a little below the steady path.
        assert!((last.p50 / last.steady - 1.0).abs() < 0.15, "{} vs {}", last.p50, last.steady);
        let chance = a.chance_of_target.unwrap();
        assert!(chance > 0.0 && chance < 1.0);
    }

    #[test]
    fn money_runs_out_when_spending_outpaces_it() {
        let input = RetirementInput {
            start: 120_000.0,
            age_now: 60,
            retire_at: 60,
            plan_until: 95,
            monthly_investing: 0.0,
            monthly_spending: 1_000.0,
            pension_from: 100,
            monthly_pension: 0.0,
            market: Market::Bear,
            risk: Risk::Balanced,
            paths: 200,
        };
        // 120,000 at 12,000 a year lasts a little over ten years at 1%.
        let r = retirement(&input);
        assert!(r.runs_out_at.is_some());
        assert!(r.years.iter().take(10).all(|y| y.covered));
        assert!(r.chance_lasts < 0.5);
        assert!(r.safe_monthly_spending < 1_000.0);
    }

    #[test]
    fn needed_monthly_reaches_the_target() {
        let m = needed_monthly(5_000.0, 40_000.0, 4, 0.035);
        let mut balance = 5_000.0;
        for _ in 0..4 {
            balance = grow(balance, m * 12.0, 0.035);
        }
        assert!((balance - 40_000.0).abs() < 0.01, "{balance}");
        assert_eq!(needed_monthly(50_000.0, 40_000.0, 4, 0.035), 0.0);
    }

    #[test]
    fn goals_take_money_in_priority_order() {
        let plan = Plan {
            profile: Some(Profile {
                birth_year: 1992,
                monthly_spending: Decimal::from(1_400),
                monthly_investing: Decimal::from(500),
                risk: Risk::Balanced,
                retirement_age: 60,
                pension_age: 65,
                monthly_pension: Decimal::from(650),
                retirement_spending: Decimal::from(1_600),
            }),
            goals: vec![
                Goal { id: "house".into(), name: "House".into(), kind: GoalKind::Savings, target: Decimal::from(40_000), year: Some(2030), monthly: Decimal::from(400) },
                Goal { id: "buffer".into(), name: "Buffer".into(), kind: GoalKind::EmergencyFund, target: Decimal::from(3), year: None, monthly: Decimal::ZERO },
            ],
        };
        let s = evaluate_goals(&plan, 5_000.0, 10_000.0, 2026, Market::Base);
        let buffer = s.iter().find(|g| g.id == "buffer").unwrap();
        let house = s.iter().find(|g| g.id == "house").unwrap();
        assert_eq!((buffer.have, buffer.target, buffer.on_track), (4_200.0, 4_200.0, true));
        assert_eq!(house.have, 10_800.0);
        assert!(house.needed_monthly.unwrap() > 400.0 && !house.on_track);
    }
}
