//! **M7 — rate-indexed valuation.** A deterministic price, settled as balanced postings.
//!
//! An external observation — a rate fixing, a market price, an FX rate — feeds a pricing
//! function whose output becomes a posting set. This is the mechanism behind caps and
//! floors, OTC derivatives, mark-to-market, NAV strikes, collateral revaluation, impairment
//! on distressed portfolios, and real-estate revaluation.
//!
//! # Determinism is the requirement, not a preference
//!
//! A valuation that cannot be recomputed is a valuation that cannot be audited, and
//! `reproduce e at #4200` (thesis §6.17) is the test it has to pass. Three consequences
//! follow, and each is enforced structurally rather than by convention:
//!
//! 1. **Observations are arguments, never fetched.** A pricer that read a live feed would
//!    return a different answer on every replay. [`Priced`] therefore records the exact
//!    observations used, so the input to a valuation is as immutable as its output.
//! 2. **No floating point.** Not defensive style: floating-point summation is not
//!    associative, so a portfolio revaluation would depend on the order in which positions
//!    happened to be visited, and Appendix C.4 requires byte-identical results across
//!    targets. Rates are integers at a declared scale, and the arithmetic below is exact.
//! 3. **Rounding is declared, once, at the point of settlement.** Every valuation
//!    eventually divides, and division is where money is created or destroyed. See
//!    [`Rounding`].
//!
//! # FX conversion lives here and not in the kernel
//!
//! `ARCHITECTURE.md` §6 records that the kernel has no currency conversion, because a rate
//! applied *inside* a posting is how a cross-currency imbalance becomes invisible: the
//! entry balances against itself and no per-currency check can see the error.
//!
//! Conversion is therefore a **valuation**, producing two conserved legs that are sealed in
//! one epoch. [`convert`] returns the converted amount and the exact rate used; building
//! the two legs is the product's job, and the kernel checks both currencies independently.
//! The rate never appears in an entry.

use gbs_kernel::{Amount, Currency, KernelError};
use std::collections::BTreeMap;
use std::fmt;

/// An index quote: a value at a declared scale.
///
/// `Rate::new(52_500, 6)` is 0.0525 — that is, 5.25%. The scale is carried rather than
/// assumed, because a system in which 5.25% is sometimes `525` and sometimes `52_500` has
/// a factor-of-a-hundred error waiting in it, and that error is entirely plausible-looking
/// in every log it appears in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rate {
    pub value: i128,
    pub scale: u32,
}

impl Rate {
    pub fn new(value: i128, scale: u32) -> Self {
        Rate { value, scale }
    }

    /// A rate in basis points: `Rate::bps(525)` is 5.25%.
    pub fn bps(bps: i128) -> Self {
        Rate { value: bps, scale: 4 }
    }

    /// A percentage at two decimals: `Rate::percent(5_25)` is 5.25%.
    pub fn percent(hundredths: i128) -> Self {
        Rate { value: hundredths, scale: 4 }
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    fn denominator(&self) -> i128 {
        10i128.pow(self.scale)
    }
}

impl fmt::Display for Rate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.denominator();
        let sign = if self.value < 0 { "-" } else { "" };
        let a = self.value.abs();
        write!(f, "{sign}{}.{:0width$}", a / d, a % d, width = self.scale as usize)
    }
}

/// How a division rounds.
///
/// Declared explicitly at every point where money is divided, because there is no correct
/// default and a hidden one is a hidden decision about who gets the fraction of a cent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    /// Toward negative infinity. Exact and symmetric across sign, which matters when a
    /// position is revalued up and then back down.
    Floor,
    /// Toward positive infinity.
    Ceiling,
    /// Toward zero — the direction that favours the paying side on a positive amount and
    /// the receiving side on a negative one, which is why it is *not* the default.
    Truncate,
    /// Half away from zero. What a person means by "round".
    HalfUp,
    /// Half to even — banker's rounding, which removes the upward bias of `HalfUp` across
    /// many roundings. The right choice for a large portfolio revaluation, and the wrong
    /// one for a single customer-facing amount, where it surprises people.
    HalfEven,
}

impl Rounding {
    /// `numerator / denominator`, rounded. Exact integer arithmetic throughout.
    pub fn divide(&self, numerator: i128, denominator: i128) -> Result<i128, KernelError> {
        if denominator == 0 {
            return Err(KernelError::Overflow { op: "divide by zero" });
        }
        let q = numerator.div_euclid(denominator);
        let r = numerator.rem_euclid(denominator);
        if r == 0 {
            return Ok(q);
        }
        // `div_euclid` floors toward negative infinity, so `q` is the floor and `r` is
        // non-negative. Everything else is expressed relative to that, which keeps the
        // sign handling in one place rather than in five.
        let neg = (numerator < 0) != (denominator < 0);
        Ok(match self {
            Rounding::Floor => q,
            Rounding::Ceiling => q + 1,
            Rounding::Truncate => {
                if neg {
                    q + 1
                } else {
                    q
                }
            }
            Rounding::HalfUp => {
                let twice = r.checked_mul(2).ok_or(KernelError::Overflow { op: "rounding" })?;
                let d = denominator.abs();
                if neg {
                    // `q` is the floor, so the true value is `q + r/d` and the two
                    // candidates are `q` (further from zero) and `q + 1` (nearer). Half
                    // away from zero therefore takes `q` on a tie, and only moves up when
                    // `q + 1` is strictly nearer — i.e. when `2r > d`.
                    //
                    // An earlier version had this inverted, and -7/2 came out as -3 instead
                    // of -4. It is exactly the asymmetry that makes a refund round
                    // differently from the payment it reverses.
                    if twice > d { q + 1 } else { q }
                } else if twice >= d {
                    q + 1
                } else {
                    q
                }
            }
            Rounding::HalfEven => {
                let twice = r.checked_mul(2).ok_or(KernelError::Overflow { op: "rounding" })?;
                let d = denominator.abs();
                match twice.cmp(&d) {
                    std::cmp::Ordering::Less => q,
                    std::cmp::Ordering::Greater => q + 1,
                    std::cmp::Ordering::Equal => {
                        if q.rem_euclid(2) == 0 {
                            q
                        } else {
                            q + 1
                        }
                    }
                }
            }
        })
    }
}

/// What a valuation refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValuationError {
    /// A required observation was absent.
    ///
    /// **Never defaulted to zero.** A cap whose index did not fix must not be priced as
    /// though the index were zero — that would make every cap pay and every floor not.
    /// This is the `Err(_) => 0` defect of thesis §1.1.1, in the place it would cost most.
    MissingObservation { index: String },
    /// A conversion between two currencies with no rate.
    NoRate { from: Currency, to: Currency },
    /// A rate of zero used as a divisor.
    ZeroRate { context: &'static str },
    Arithmetic(KernelError),
}

impl fmt::Display for ValuationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValuationError::MissingObservation { index } => write!(
                f,
                "observation `{index}` is absent and cannot be priced. A missing fixing is \
                 not zero"
            ),
            ValuationError::NoRate { from, to } => {
                write!(f, "no rate from {from} to {to}; conversion cannot be priced")
            }
            ValuationError::ZeroRate { context } => {
                write!(f, "a zero rate cannot be used as a divisor in {context}")
            }
            ValuationError::Arithmetic(e) => write!(f, "{e}"),
        }
    }
}

/// A priced result, with the evidence that produced it.
///
/// The `inputs` field is what makes `reproduce e at #4200` answerable: the exact
/// observations used are recorded beside the answer, so a valuation can be recomputed and
/// checked rather than taken on trust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Priced {
    pub amount: Amount,
    /// Every observation the pricer consulted, in stable order.
    pub inputs: BTreeMap<String, Rate>,
    pub rounding: Rounding,
}

impl Priced {
    /// Recompute-and-compare: does `f`, given the recorded inputs, still produce this
    /// amount?
    ///
    /// The audit operation. A valuation that fails this has either a non-deterministic
    /// pricer or a corrupted record, and both are worth knowing about.
    pub fn reproduces_under(
        &self,
        f: impl Fn(&BTreeMap<String, Rate>, Rounding) -> Result<Amount, ValuationError>,
    ) -> bool {
        matches!(f(&self.inputs, self.rounding), Ok(a) if a == self.amount)
    }
}

/// Apply a rate to an amount: `amount × rate`, rounded as declared.
///
/// The core operation of interest accrual, fee calculation and percentage allocation. The
/// multiplication happens *before* the division, so no precision is lost on the way — which
/// is the difference between an accrual that is exact and one that drifts a cent per period
/// for thirty years.
pub fn apply_rate(
    amount: &Amount,
    rate: Rate,
    rounding: Rounding,
) -> Result<Amount, ValuationError> {
    let numerator = amount
        .minor
        .checked_mul(rate.value)
        .ok_or(ValuationError::Arithmetic(KernelError::Overflow { op: "apply_rate" }))?;
    let minor = rounding
        .divide(numerator, rate.denominator())
        .map_err(ValuationError::Arithmetic)?;
    Ok(Amount::new(minor, amount.currency.clone(), amount.scale))
}

/// Accrue interest over a period: `principal × rate × days / basis`.
///
/// `basis` is the day-count denominator — 360 for actual/360, 365 for actual/365. Passed in
/// rather than defaulted, because the convention is a term of the contract and a hidden
/// default is a hidden term.
///
/// Both multiplications precede the division, for the same precision reason as
/// [`apply_rate`].
pub fn accrue(
    principal: &Amount,
    rate: Rate,
    days: i64,
    basis: i64,
    rounding: Rounding,
) -> Result<Amount, ValuationError> {
    if basis == 0 {
        return Err(ValuationError::ZeroRate { context: "day-count basis" });
    }
    let numerator = principal
        .minor
        .checked_mul(rate.value)
        .and_then(|n| n.checked_mul(days as i128))
        .ok_or(ValuationError::Arithmetic(KernelError::Overflow { op: "accrue" }))?;
    let denominator = rate
        .denominator()
        .checked_mul(basis as i128)
        .ok_or(ValuationError::Arithmetic(KernelError::Overflow { op: "accrue" }))?;
    let minor = rounding.divide(numerator, denominator).map_err(ValuationError::Arithmetic)?;
    Ok(Amount::new(minor, principal.currency.clone(), principal.scale))
}

/// Convert between currencies at a rate. **Produces an amount, never a posting.**
///
/// The result is one side of an FX transaction; the caller builds two conserved legs from
/// it and seals them in one epoch. The rate never enters an entry, so the kernel's
/// per-currency check sees two independent obligations and can catch an error in either.
pub fn convert(
    amount: &Amount,
    to: &Currency,
    rate: Rate,
    to_scale: u32,
    rounding: Rounding,
) -> Result<Priced, ValuationError> {
    if rate.is_zero() {
        return Err(ValuationError::ZeroRate { context: "fx conversion" });
    }
    // Rescale from the source scale to the target scale in the same division, so a
    // conversion between currencies with different scales — USD at 2 to JPY at 0, say —
    // loses nothing to an intermediate rounding.
    let numerator = amount
        .minor
        .checked_mul(rate.value)
        .and_then(|n| n.checked_mul(10i128.pow(to_scale)))
        .ok_or(ValuationError::Arithmetic(KernelError::Overflow { op: "convert" }))?;
    let denominator = rate
        .denominator()
        .checked_mul(10i128.pow(amount.scale))
        .ok_or(ValuationError::Arithmetic(KernelError::Overflow { op: "convert" }))?;
    let minor = rounding.divide(numerator, denominator).map_err(ValuationError::Arithmetic)?;

    let mut inputs = BTreeMap::new();
    inputs.insert(format!("{}/{}", amount.currency, to), rate);

    Ok(Priced { amount: Amount::new(minor, to.clone(), to_scale), inputs, rounding })
}

/// The payoff of a cap: `notional × max(index − strike, 0) × days / basis`.
///
/// Returns `Ok(None)` when the option is out of the money — **not** a zero amount. A cap
/// below its strike did not pay; it did not pay zero, and the auditor's question is which.
/// This mirrors [`Contingency`](crate::schedule::Contingency) producing no posting set
/// rather than an empty one.
pub fn cap_payoff(
    notional: &Amount,
    index: Option<Rate>,
    strike: Rate,
    days: i64,
    basis: i64,
    rounding: Rounding,
) -> Result<Option<Amount>, ValuationError> {
    let index = index.ok_or_else(|| ValuationError::MissingObservation { index: "index".into() })?;
    // Compare at a common scale rather than assuming the two are quoted the same way.
    let (i, s) = align(index, strike)?;
    if i <= s.value {
        return Ok(None);
    }
    let excess = Rate::new(i - s.value, s.scale);
    Ok(Some(accrue(notional, excess, days, basis, rounding)?))
}

/// The payoff of a floor: fires when the index is *below* the strike.
pub fn floor_payoff(
    notional: &Amount,
    index: Option<Rate>,
    strike: Rate,
    days: i64,
    basis: i64,
    rounding: Rounding,
) -> Result<Option<Amount>, ValuationError> {
    let index = index.ok_or_else(|| ValuationError::MissingObservation { index: "index".into() })?;
    let (i, s) = align(index, strike)?;
    if i >= s.value {
        return Ok(None);
    }
    let shortfall = Rate::new(s.value - i, s.scale);
    Ok(Some(accrue(notional, shortfall, days, basis, rounding)?))
}

/// Bring two rates to a common scale so they can be compared exactly.
///
/// Scales *up* to the finer of the two, never down, so no precision is discarded in a
/// comparison that decides whether an option is in the money.
fn align(a: Rate, b: Rate) -> Result<(i128, Rate), ValuationError> {
    let scale = a.scale.max(b.scale);
    let lift = |r: Rate| -> Result<i128, ValuationError> {
        r.value
            .checked_mul(10i128.pow(scale - r.scale))
            .ok_or(ValuationError::Arithmetic(KernelError::Overflow { op: "align" }))
    };
    Ok((lift(a)?, Rate::new(lift(b)?, scale)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usd(minor: i128) -> Amount {
        Amount::minor_2dp(minor, "USD")
    }

    // ── rounding ────────────────────────────────────────────────────────────────────

    #[test]
    fn every_rounding_mode_does_what_it_says_on_positives_and_negatives() {
        // 7/2 = 3.5 and -7/2 = -3.5 — the case that separates all five.
        assert_eq!(Rounding::Floor.divide(7, 2).unwrap(), 3);
        assert_eq!(Rounding::Floor.divide(-7, 2).unwrap(), -4);
        assert_eq!(Rounding::Ceiling.divide(7, 2).unwrap(), 4);
        assert_eq!(Rounding::Ceiling.divide(-7, 2).unwrap(), -3);
        assert_eq!(Rounding::Truncate.divide(7, 2).unwrap(), 3);
        assert_eq!(Rounding::Truncate.divide(-7, 2).unwrap(), -3, "toward zero");
        assert_eq!(Rounding::HalfUp.divide(7, 2).unwrap(), 4);
        assert_eq!(Rounding::HalfUp.divide(-7, 2).unwrap(), -4, "half away from zero");
    }

    #[test]
    fn half_even_removes_the_upward_bias_that_half_up_has() {
        // The reason banker's rounding exists. Across 1.5, 2.5, 3.5, 4.5, HalfUp adds 2 and
        // HalfEven adds 0 — which over a large portfolio revaluation is the difference
        // between a drift and none.
        let halves = [(3, 2), (5, 2), (7, 2), (9, 2)];
        let up: i128 = halves.iter().map(|&(n, d)| Rounding::HalfUp.divide(n, d).unwrap()).sum();
        let even: i128 = halves.iter().map(|&(n, d)| Rounding::HalfEven.divide(n, d).unwrap()).sum();
        let exact_doubled: i128 = halves.iter().map(|&(n, _)| n).sum();
        assert_eq!(up, 2 + 3 + 4 + 5);
        assert_eq!(even, 2 + 2 + 4 + 4);
        assert_eq!(even * 2, exact_doubled, "half-even is unbiased across this set");
        assert!(up * 2 > exact_doubled, "half-up is biased upward");
    }

    #[test]
    fn exact_division_is_unaffected_by_the_mode() {
        for m in [Rounding::Floor, Rounding::Ceiling, Rounding::Truncate, Rounding::HalfUp, Rounding::HalfEven] {
            assert_eq!(m.divide(100, 4).unwrap(), 25);
            assert_eq!(m.divide(-100, 4).unwrap(), -25);
        }
    }

    #[test]
    fn dividing_by_zero_is_an_error_rather_than_a_panic() {
        assert!(Rounding::HalfUp.divide(1, 0).is_err());
    }

    // ── rates and accrual ───────────────────────────────────────────────────────────

    #[test]
    fn a_rate_carries_its_scale_so_five_and_a_quarter_percent_is_unambiguous() {
        assert_eq!(Rate::bps(525).to_string(), "0.0525");
        assert_eq!(Rate::new(52_500, 6).to_string(), "0.052500");
        // Same rate, two scales, and both render as the same number to the reader.
        let a = apply_rate(&usd(1_000_000), Rate::bps(525), Rounding::HalfEven).unwrap();
        let b = apply_rate(&usd(1_000_000), Rate::new(52_500, 6), Rounding::HalfEven).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn accrual_multiplies_before_dividing_so_it_does_not_drift() {
        // $1,000,000.00 at 5.25% for 30 days on actual/360.
        // Exact: 100_000_000 × 525 × 30 / (10_000 × 360) = 437_500 minor = $4,375.00
        // — one twelfth of a year's 5.25%, which is $52,500.00 annually.
        let i = accrue(&usd(100_000_000), Rate::bps(525), 30, 360, Rounding::HalfEven).unwrap();
        assert_eq!(i.minor, 437_500);
        let annual = accrue(&usd(100_000_000), Rate::bps(525), 360, 360, Rounding::HalfEven).unwrap();
        assert_eq!(annual.minor, 5_250_000, "and the annual figure is twelve times it");

        // The drift this prevents: rounding the daily rate first would give a different and
        // slightly wrong answer every period, compounding over the life of the loan.
        let daily_rounded = Rounding::HalfEven.divide(525, 360).unwrap(); // 1, badly lossy
        assert_ne!(daily_rounded * 100_000_000 * 30 / 10_000, i.minor);
    }

    #[test]
    fn a_full_year_of_daily_accruals_sums_to_the_annual_figure_within_rounding() {
        // The property a loan system is judged on: 360 daily accruals must not drift away
        // from one annual accrual by more than the rounding permits.
        let principal = usd(100_000_000);
        let rate = Rate::bps(525);
        let annual = accrue(&principal, rate, 360, 360, Rounding::HalfEven).unwrap();
        let daily: i128 = (0..360)
            .map(|_| accrue(&principal, rate, 1, 360, Rounding::HalfEven).unwrap().minor)
            .sum();
        let drift = (annual.minor - daily).abs();
        assert!(drift <= 360, "drift of {drift} minor units over a year");
    }

    #[test]
    fn a_zero_day_count_basis_is_refused() {
        assert!(matches!(
            accrue(&usd(100), Rate::bps(500), 30, 0, Rounding::Floor),
            Err(ValuationError::ZeroRate { .. })
        ));
    }

    #[test]
    fn accrual_overflow_is_reported_rather_than_wrapping() {
        assert!(accrue(&usd(i128::MAX), Rate::bps(525), 360, 360, Rounding::Floor).is_err());
    }

    // ── fx conversion ───────────────────────────────────────────────────────────────

    #[test]
    fn conversion_produces_an_amount_and_the_rate_never_enters_an_entry() {
        // $100.00 at 0.92 EUR/USD = €92.00. What comes back is an amount plus the evidence;
        // building the two conserved legs is the product's job, and the kernel then checks
        // USD and EUR independently.
        let p = convert(&usd(10_000), &Currency::new("EUR"), Rate::new(9_200, 4), 2, Rounding::HalfEven)
            .unwrap();
        assert_eq!(p.amount.minor, 9_200);
        assert_eq!(p.amount.currency, Currency::new("EUR"));
        assert_eq!(p.inputs.len(), 1, "the rate is recorded as evidence, beside the answer");
        assert!(p.inputs.contains_key("USD/EUR"));
    }

    #[test]
    fn conversion_between_different_scales_loses_nothing_to_an_intermediate_rounding() {
        // USD at 2 decimals to JPY at 0. $100.00 at 150 JPY/USD = ¥15,000.
        let jpy = convert(&usd(10_000), &Currency::new("JPY"), Rate::new(150_0000, 4), 0, Rounding::HalfEven)
            .unwrap();
        assert_eq!(jpy.amount.minor, 15_000);
        assert_eq!(jpy.amount.scale, 0);
    }

    #[test]
    fn a_zero_rate_conversion_is_refused() {
        assert!(matches!(
            convert(&usd(100), &Currency::new("EUR"), Rate::new(0, 4), 2, Rounding::Floor),
            Err(ValuationError::ZeroRate { .. })
        ));
    }

    // ── option payoffs, and the absence rule ────────────────────────────────────────

    #[test]
    fn a_cap_above_its_strike_pays_and_below_it_pays_nothing_rather_than_zero() {
        // The distinction that matters to an auditor: the cap did not pay, as against the
        // cap paid zero. `Ok(None)` and `Ok(Some(0))` are different answers.
        let notional = usd(100_000_000);
        let strike = Rate::bps(500);

        let paid = cap_payoff(&notional, Some(Rate::bps(600)), strike, 90, 360, Rounding::HalfEven)
            .unwrap();
        // 1% excess over 90/360 of a year on $1M = $2,500.00
        assert_eq!(paid.map(|a| a.minor), Some(250_000));

        let unpaid = cap_payoff(&notional, Some(Rate::bps(400)), strike, 90, 360, Rounding::HalfEven)
            .unwrap();
        assert_eq!(unpaid, None, "out of the money produces nothing, not zero");
    }

    #[test]
    fn a_cap_exactly_at_its_strike_does_not_pay() {
        let r = cap_payoff(&usd(1_000_000), Some(Rate::bps(500)), Rate::bps(500), 90, 360, Rounding::Floor)
            .unwrap();
        assert_eq!(r, None, "at the money is not in the money");
    }

    #[test]
    fn a_floor_is_the_mirror_of_a_cap() {
        let notional = usd(100_000_000);
        let strike = Rate::bps(500);
        assert_eq!(
            floor_payoff(&notional, Some(Rate::bps(400)), strike, 90, 360, Rounding::HalfEven)
                .unwrap()
                .map(|a| a.minor),
            Some(250_000)
        );
        assert_eq!(
            floor_payoff(&notional, Some(Rate::bps(600)), strike, 90, 360, Rounding::HalfEven).unwrap(),
            None
        );
    }

    #[test]
    fn a_missing_fixing_is_an_error_and_never_priced_as_zero() {
        // The single most consequential absence in this module. Priced as zero, every cap
        // pays out and every floor does not — a systematic, one-directional error.
        let e = cap_payoff(&usd(1_000_000), None, Rate::bps(500), 90, 360, Rounding::Floor)
            .unwrap_err();
        assert!(matches!(e, ValuationError::MissingObservation { .. }));
        assert!(e.to_string().contains("not zero"));

        let e2 = floor_payoff(&usd(1_000_000), None, Rate::bps(500), 90, 360, Rounding::Floor)
            .unwrap_err();
        assert!(matches!(e2, ValuationError::MissingObservation { .. }));
    }

    #[test]
    fn rates_quoted_at_different_scales_compare_exactly() {
        // 5.25% quoted at scale 4 against 5.2500% at scale 6 — the same rate, and an
        // implementation comparing raw values would decide the option was deep in the money.
        let strike = Rate::bps(525);
        let same_rate_finer = Rate::new(52_500, 6);
        assert_eq!(
            cap_payoff(&usd(1_000_000), Some(same_rate_finer), strike, 90, 360, Rounding::Floor).unwrap(),
            None,
            "the same rate at a finer scale is not above the strike"
        );
        // And a genuinely higher one at the finer scale does pay.
        assert!(cap_payoff(&usd(1_000_000), Some(Rate::new(52_501, 6)), strike, 90, 360, Rounding::Ceiling)
            .unwrap()
            .is_some());
    }

    // ── reproducibility ─────────────────────────────────────────────────────────────

    #[test]
    fn a_priced_result_carries_its_inputs_so_it_can_be_reproduced() {
        // `reproduce e at #4200`, as a unit test. A valuation that cannot be recomputed
        // cannot be audited.
        let p = convert(&usd(12_345), &Currency::new("EUR"), Rate::new(9_237, 4), 2, Rounding::HalfEven)
            .unwrap();
        assert!(p.reproduces_under(|inputs, rounding| {
            let rate = inputs["USD/EUR"];
            Ok(convert(&usd(12_345), &Currency::new("EUR"), rate, 2, rounding)?.amount)
        }));
    }

    #[test]
    fn a_non_deterministic_pricer_fails_reproduction_which_is_the_point() {
        let p = convert(&usd(10_000), &Currency::new("EUR"), Rate::new(9_200, 4), 2, Rounding::HalfEven)
            .unwrap();
        assert!(
            !p.reproduces_under(|_, _| Ok(Amount::minor_2dp(9_201, "EUR"))),
            "a pricer that ignores its recorded inputs must not reproduce"
        );
    }

    #[test]
    fn pricing_is_bit_identical_across_repeated_runs() {
        // No floats anywhere, so this holds by construction — and the test is here so that
        // introducing one breaks something named.
        let first = accrue(&usd(987_654_321), Rate::new(4_873, 6), 173, 365, Rounding::HalfEven).unwrap();
        for _ in 0..100 {
            assert_eq!(
                accrue(&usd(987_654_321), Rate::new(4_873, 6), 173, 365, Rounding::HalfEven).unwrap(),
                first
            );
        }
    }

    #[test]
    fn recorded_inputs_iterate_in_stable_order() {
        let p = convert(&usd(1), &Currency::new("EUR"), Rate::new(9_200, 4), 2, Rounding::Floor).unwrap();
        let keys: Vec<&String> = p.inputs.keys().collect();
        let keys2: Vec<&String> = p.inputs.keys().collect();
        assert_eq!(keys, keys2);
    }
}
