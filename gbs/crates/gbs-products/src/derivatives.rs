//! Derivatives — forwards, swaps, caps and floors.
//!
//! M2 (contingent schedules) and M7 (rate-indexed valuation), composed. A swap is two
//! schedules whose generators price legs from observations; a cap is one schedule whose
//! contingency is a strike comparison; a forward is a single-occurrence schedule.
//!
//! # The property these products are built to have
//!
//! **A derivative's cashflows are a schedule, and the schedule is balanced by
//! construction.** Not "we check the coupon before we post it" — the *generator* is checked
//! once, at declaration, and every occurrence it will ever produce is balanced thereafter.
//! For an instrument with a twenty-year schedule that is the difference between a proof and
//! a nightly reconciliation.
//!
//! The generators here all build their two legs by mirroring, so the balanced-by-
//! construction claim is not a hope about the arithmetic; there is no path by which the
//! credit can differ from the debit.
//!
//! # Netting is deliberately not automatic
//!
//! A swap's two legs are settled gross unless the parties have agreed to net, and whether
//! they have is a term of the ISDA master agreement rather than a property of the
//! instrument. [`Swap::net_settlement`] exists and must be asked for. A system that netted
//! by default would produce the right cash and the wrong gross exposure, and gross exposure
//! is what a capital calculation uses.

use gbs_kernel::{Amount, Chart, Currency, Entry, Epoch, KernelError, PostingSet, Stamp};
use gbs_mechanisms::{
    accrue, cap_payoff, floor_payoff, AllDays, Calendar, Contingency, Generator, Observations,
    Occurrence, Rate, Rounding, Schedule, ScheduleError, ValuationError,
};

/// What a derivative refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivativeError {
    Schedule(ScheduleError),
    Valuation(ValuationError),
    Kernel(KernelError),
    /// The two legs of a swap are in different currencies but no FX rate was supplied.
    ///
    /// Refused rather than assumed: a cross-currency swap netted at an invented rate is a
    /// position that does not exist.
    CrossCurrencyWithoutRate { pay: Currency, receive: Currency },
}

impl std::fmt::Display for DerivativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DerivativeError::Schedule(e) => write!(f, "{e}"),
            DerivativeError::Valuation(e) => write!(f, "{e}"),
            DerivativeError::Kernel(e) => write!(f, "{e}"),
            DerivativeError::CrossCurrencyWithoutRate { pay, receive } => write!(
                f,
                "cannot net a {pay} leg against a {receive} leg without an FX rate; a \
                 cross-currency swap netted at an invented rate is a position that does \
                 not exist"
            ),
        }
    }
}

/// A fixed-rate leg: pays the same rate every period.
///
/// Returns a generator, so the leg *is* a schedule rather than a list of amounts. This is
/// M2's central distinction: the rule is checked once, and every cashflow it produces is
/// balanced.
pub fn fixed_leg(
    notional: Amount,
    rate: Rate,
    basis: i64,
    effective: i64,
    pay_account: String,
    receive_account: String,
    rounding: Rounding,
) -> Generator {
    Box::new(move |occ: &Occurrence, _obs: &Observations| {
        let days = occ.accrual_days(effective);
        let amount = accrue(&notional, rate, days, basis, rounding).ok()?;
        let debit = Entry::new(
            occ.index as u64 * 2 + 1,
            pay_account.clone(),
            amount.negate().ok()?,
            Stamp::new(Epoch(0), occ.adjusted),
        )
        .narrated(format!("fixed leg, period {}, {days}d at {rate}", occ.index));
        let credit = debit.mirrored_to(occ.index as u64 * 2 + 2, receive_account.clone()).ok()?;
        Some(PostingSet::new(format!("fixed-{}", occ.index)).with(debit).with(credit))
    })
}

/// A floating-rate leg: prices from an observed index each period.
///
/// The index is read from the observations passed in, never fetched — so replaying the
/// schedule with the same fixings reproduces the same cashflows exactly, which is what
/// `reproduce e at #4200` requires.
///
/// If the fixing is absent, the generator produces **nothing**. It does not price at zero.
/// A floating leg whose index did not fix has an unknown cashflow, and posting zero would
/// assert that it was known to be nil.
pub fn floating_leg(
    notional: Amount,
    index: String,
    spread: Rate,
    basis: i64,
    effective: i64,
    pay_account: String,
    receive_account: String,
    rounding: Rounding,
) -> Generator {
    Box::new(move |occ: &Occurrence, obs: &Observations| {
        // Absent means absent. See the doc comment.
        let (value, scale) = obs.get(&index)?;
        let all_in = Rate::new(
            value.checked_add(rescale(spread, scale)?)?,
            scale,
        );
        let days = occ.accrual_days(effective);
        let amount = accrue(&notional, all_in, days, basis, rounding).ok()?;
        let debit = Entry::new(
            occ.index as u64 * 2 + 1,
            pay_account.clone(),
            amount.negate().ok()?,
            Stamp::new(Epoch(0), occ.adjusted),
        )
        .narrated(format!("floating leg, period {}, {index} + {spread}", occ.index));
        let credit = debit.mirrored_to(occ.index as u64 * 2 + 2, receive_account.clone()).ok()?;
        Some(PostingSet::new(format!("float-{}", occ.index)).with(debit).with(credit))
    })
}

/// Lift a rate to a finer scale. Returns `None` on overflow rather than saturating, because
/// a saturated spread is a silently wrong cashflow.
fn rescale(r: Rate, to_scale: u32) -> Option<i128> {
    if to_scale < r.scale {
        // Scaling *down* would discard precision from a contractual spread. Refused.
        return None;
    }
    r.value.checked_mul(10i128.pow(to_scale - r.scale))
}

/// An interest-rate swap: two legs on one schedule.
pub struct Swap {
    pub id: String,
    pub pay: Schedule,
    pub receive: Schedule,
    pub notional: Amount,
}

impl Swap {
    /// Build a vanilla fixed-for-floating swap.
    #[allow(clippy::too_many_arguments)]
    pub fn vanilla(
        id: &str,
        notional: Amount,
        fixed_rate: Rate,
        index: &str,
        spread: Rate,
        effective: i64,
        maturity: i64,
        every_days: i64,
        basis: i64,
    ) -> Result<Self, DerivativeError> {
        let pay = Schedule::new(
            format!("{id}-fixed"),
            effective,
            maturity,
            every_days,
            fixed_leg(
                notional.clone(),
                fixed_rate,
                basis,
                effective,
                "swap.pay.fixed".into(),
                "counterparty.usd".into(),
                Rounding::HalfEven,
            ),
        )
        .map_err(DerivativeError::Schedule)?;

        let receive = Schedule::new(
            format!("{id}-floating"),
            effective,
            maturity,
            every_days,
            floating_leg(
                notional.clone(),
                index.to_string(),
                spread,
                basis,
                effective,
                "counterparty.usd".into(),
                "swap.receive.floating".into(),
                Rounding::HalfEven,
            ),
        )
        .map_err(DerivativeError::Schedule)?;

        Ok(Swap { id: id.to_string(), pay, receive, notional })
    }

    /// Check both legs' generators before the swap is booked.
    ///
    /// The declaration-time gate. A twenty-year swap whose generator could produce an
    /// unbalanced cashflow is rejected now rather than on a payment date in 2044.
    pub fn check<C: Calendar>(
        &self,
        cal: &C,
        chart: &Chart,
        samples: &[Observations],
    ) -> Result<(), DerivativeError> {
        self.pay.check_generator(cal, chart, samples).map_err(DerivativeError::Schedule)?;
        self.receive.check_generator(cal, chart, samples).map_err(DerivativeError::Schedule)
    }

    /// The net cash on a payment date. **Must be asked for** — see the module docs.
    ///
    /// Returns `None` when neither leg has a cashflow on that date. Refuses a
    /// cross-currency net, because netting two currencies requires a rate and a rate that
    /// was not supplied cannot be invented.
    pub fn net_settlement<C: Calendar>(
        &self,
        cal: &C,
        as_of: i64,
        epoch: Epoch,
        obs: &Observations,
    ) -> Result<Option<Amount>, DerivativeError> {
        let pay = self.pay.generate(cal, as_of, epoch, obs).map_err(DerivativeError::Schedule)?;
        let receive =
            self.receive.generate(cal, as_of, epoch, obs).map_err(DerivativeError::Schedule)?;
        if pay.is_empty() && receive.is_empty() {
            return Ok(None);
        }

        // Take each leg's own currency from its first entry, and refuse to net across two.
        let leg_amount = |sets: &[(Occurrence, PostingSet)]| -> Option<Amount> {
            sets.first().and_then(|(_, ps)| ps.entries.first().map(|e| e.amount.clone()))
        };
        match (leg_amount(&pay), leg_amount(&receive)) {
            (Some(p), Some(r)) => {
                if p.currency != r.currency {
                    return Err(DerivativeError::CrossCurrencyWithoutRate {
                        pay: p.currency,
                        receive: r.currency,
                    });
                }
                // `p` is the payer's debit (negative) and `r` the receiver's debit; the net
                // is their difference in the payer's direction.
                Ok(Some(p.add(&r.negate().map_err(DerivativeError::Kernel)?)
                    .map_err(DerivativeError::Kernel)?))
            }
            (Some(p), None) => Ok(Some(p)),
            (None, Some(r)) => Ok(Some(r.negate().map_err(DerivativeError::Kernel)?)),
            (None, None) => Ok(None),
        }
    }
}

/// An interest-rate cap: pays when the index exceeds the strike.
///
/// One schedule whose contingency is the strike comparison. Below the strike it produces
/// **no posting set at all** — the distinction M2 and M7 both maintain, and the one an
/// auditor asks about.
pub fn cap(
    id: &str,
    notional: Amount,
    index: &str,
    strike: Rate,
    effective: i64,
    maturity: i64,
    every_days: i64,
    basis: i64,
) -> Result<Schedule, DerivativeError> {
    let idx = index.to_string();
    let idx_for_gen = idx.clone();
    let n = notional.clone();

    let generator: Generator = Box::new(move |occ, obs| {
        let observed = obs.get(&idx_for_gen).map(|(v, s)| Rate::new(v, s));
        let days = occ.accrual_days(effective);
        let payoff = cap_payoff(&n, observed, strike, days, basis, Rounding::HalfEven).ok()??;
        let debit = Entry::new(
            occ.index as u64 * 2 + 1,
            "cap.writer",
            payoff.negate().ok()?,
            Stamp::new(Epoch(0), occ.adjusted),
        )
        .narrated(format!("cap payoff, period {}, strike {strike}", occ.index));
        let credit = debit.mirrored_to(occ.index as u64 * 2 + 2, "cap.holder").ok()?;
        Some(PostingSet::new(format!("cap-{}", occ.index)).with(debit).with(credit))
    });

    let idx_for_contingency = idx;
    Ok(Schedule::new(id, effective, maturity, every_days, generator)
        .map_err(DerivativeError::Schedule)?
        .contingent_on(Contingency::When(Box::new(move |_occ, obs| {
            // Fires only when the index fixed *and* exceeded the strike. An absent fixing
            // does not fire, which is the honest answer: an unfixed index is not known to
            // be below the strike either.
            matches!(obs.get(&idx_for_contingency), Some((v, s)) if above(v, s, strike))
        }))))
}

/// An interest-rate floor: the mirror of a cap.
pub fn floor(
    id: &str,
    notional: Amount,
    index: &str,
    strike: Rate,
    effective: i64,
    maturity: i64,
    every_days: i64,
    basis: i64,
) -> Result<Schedule, DerivativeError> {
    let idx = index.to_string();
    let idx_for_gen = idx.clone();
    let n = notional.clone();

    let generator: Generator = Box::new(move |occ, obs| {
        let observed = obs.get(&idx_for_gen).map(|(v, s)| Rate::new(v, s));
        let days = occ.accrual_days(effective);
        let payoff = floor_payoff(&n, observed, strike, days, basis, Rounding::HalfEven).ok()??;
        let debit = Entry::new(
            occ.index as u64 * 2 + 1,
            "floor.writer",
            payoff.negate().ok()?,
            Stamp::new(Epoch(0), occ.adjusted),
        )
        .narrated(format!("floor payoff, period {}, strike {strike}", occ.index));
        let credit = debit.mirrored_to(occ.index as u64 * 2 + 2, "floor.holder").ok()?;
        Some(PostingSet::new(format!("floor-{}", occ.index)).with(debit).with(credit))
    });

    let idx_for_contingency = idx;
    Ok(Schedule::new(id, effective, maturity, every_days, generator)
        .map_err(DerivativeError::Schedule)?
        .contingent_on(Contingency::When(Box::new(move |_occ, obs| {
            matches!(obs.get(&idx_for_contingency), Some((v, s)) if below(v, s, strike))
        }))))
}

/// Compare an observed value at its own scale against a strike at its scale, exactly.
fn above(value: i128, scale: u32, strike: Rate) -> bool {
    let common = scale.max(strike.scale);
    let lift = |v: i128, s: u32| v.checked_mul(10i128.pow(common - s));
    matches!((lift(value, scale), lift(strike.value, strike.scale)), (Some(v), Some(k)) if v > k)
}

fn below(value: i128, scale: u32, strike: Rate) -> bool {
    let common = scale.max(strike.scale);
    let lift = |v: i128, s: u32| v.checked_mul(10i128.pow(common - s));
    matches!((lift(value, scale), lift(strike.value, strike.scale)), (Some(v), Some(k)) if v < k)
}

/// A forward: a single exchange at a future date. A schedule with one occurrence.
pub fn forward(
    id: &str,
    notional: Amount,
    strike_rate: Rate,
    settles_on: i64,
    basis: i64,
) -> Result<Schedule, DerivativeError> {
    // One period, from day zero to settlement. Expressed as a schedule rather than as a
    // special case, so a forward and a swap go through the same declaration-time check.
    Schedule::new(
        id,
        0,
        settles_on,
        settles_on,
        fixed_leg(
            notional,
            strike_rate,
            basis,
            0,
            "forward.pay".into(),
            "forward.receive".into(),
            Rounding::HalfEven,
        ),
    )
    .map_err(DerivativeError::Schedule)
}

/// A convenience for the common declaration-time check, with an all-days calendar.
pub fn check_with_all_days(
    s: &Schedule,
    chart: &Chart,
    samples: &[Observations],
) -> Result<(), DerivativeError> {
    s.check_generator(&AllDays, chart, samples).map_err(DerivativeError::Schedule)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gbs_kernel::Account;

    fn usd(m: i128) -> Amount {
        Amount::minor_2dp(m, "USD")
    }

    fn chart() -> Chart {
        Chart::new()
            .with(Account::new("swap.pay.fixed", "bank", "USD"))
            .with(Account::new("swap.receive.floating", "bank", "USD"))
            .with(Account::new("counterparty.usd", "cpty", "USD"))
            .with(Account::new("cap.writer", "bank", "USD"))
            .with(Account::new("cap.holder", "cust", "USD"))
            .with(Account::new("floor.writer", "bank", "USD"))
            .with(Account::new("floor.holder", "cust", "USD"))
            .with(Account::new("forward.pay", "bank", "USD"))
            .with(Account::new("forward.receive", "cpty", "USD"))
    }

    fn fixings() -> Vec<Observations> {
        // Sample the boundaries, which is what `check_generator`'s honesty note asks for:
        // an unfixed index, a low rate, the strike itself, and a high one.
        vec![
            Observations::new(),
            Observations::new().fix("SOFR", 100, 4),
            Observations::new().fix("SOFR", 500, 4),
            Observations::new().fix("SOFR", 900, 4),
        ]
    }

    // ── swaps ───────────────────────────────────────────────────────────────────────

    #[test]
    fn a_vanilla_swaps_generators_are_balanced_at_declaration_time() {
        // The property. A twenty-year swap is checked once, now, across sampled fixings —
        // not on a payment date in 2044.
        let s = Swap::vanilla(
            "irs-1",
            usd(1_000_000_00),
            Rate::bps(450),
            "SOFR",
            Rate::bps(25),
            0,
            7_200, // twenty years of 360-day periods
            180,
            360,
        )
        .unwrap();
        s.check(&AllDays, &chart(), &fixings()).expect("both legs conserve for every occurrence");
    }

    #[test]
    fn both_legs_produce_conserving_posting_sets_on_a_payment_date() {
        let s = Swap::vanilla("irs-2", usd(1_000_000_00), Rate::bps(450), "SOFR", Rate::bps(25), 0, 720, 180, 360)
            .unwrap();
        let obs = Observations::new().fix("SOFR", 500, 4);

        let fixed = s.pay.generate(&AllDays, 180, Epoch(5), &obs).unwrap();
        let floating = s.receive.generate(&AllDays, 180, Epoch(5), &obs).unwrap();
        assert_eq!(fixed.len(), 1);
        assert_eq!(floating.len(), 1);
        fixed[0].1.clone().seal(&chart(), Epoch(5)).unwrap();
        floating[0].1.clone().seal(&chart(), Epoch(5)).unwrap();
    }

    #[test]
    fn the_floating_leg_produces_nothing_when_the_index_did_not_fix() {
        // Absent is not zero. A floating cashflow with no fixing is unknown, and posting
        // zero would assert it was known to be nil.
        let s = Swap::vanilla("irs-3", usd(1_000_000_00), Rate::bps(450), "SOFR", Rate::bps(25), 0, 720, 180, 360)
            .unwrap();
        let no_fixing = Observations::new();
        assert!(s.receive.generate(&AllDays, 180, Epoch(5), &no_fixing).unwrap().is_empty());
        // And the fixed leg still pays, because it does not depend on an index.
        assert_eq!(s.pay.generate(&AllDays, 180, Epoch(5), &no_fixing).unwrap().len(), 1);
    }

    #[test]
    fn replaying_a_leg_with_the_same_fixings_reproduces_the_same_cashflows() {
        // `reproduce e at #4200`, at the product level.
        let s = Swap::vanilla("irs-4", usd(1_000_000_00), Rate::bps(450), "SOFR", Rate::bps(25), 0, 720, 180, 360)
            .unwrap();
        let obs = Observations::new().fix("SOFR", 487, 4);
        let first = s.receive.generate(&AllDays, 180, Epoch(9), &obs).unwrap();
        for _ in 0..20 {
            assert_eq!(s.receive.generate(&AllDays, 180, Epoch(9), &obs).unwrap(), first);
        }
    }

    #[test]
    fn netting_must_be_asked_for_and_gives_the_difference() {
        // Gross by default; netting is a term of the master agreement, not a property of
        // the instrument. A system that netted automatically would produce the right cash
        // and the wrong gross exposure — and gross exposure is what a capital calculation
        // uses.
        let s = Swap::vanilla("irs-5", usd(1_000_000_00), Rate::bps(450), "SOFR", Rate::bps(50), 0, 720, 180, 360)
            .unwrap();
        // Floating fixes at 4.00% + 0.50% spread = 4.50%, equal to the fixed leg, so the
        // net is zero while both legs are individually large.
        let obs = Observations::new().fix("SOFR", 400, 4);
        let net = s.net_settlement(&AllDays, 180, Epoch(5), &obs).unwrap().unwrap();
        assert_eq!(net.minor, 0, "the legs offset exactly");

        let gross = s.pay.generate(&AllDays, 180, Epoch(5), &obs).unwrap();
        assert_ne!(gross[0].1.entries[0].amount.minor, 0, "while the gross leg is not zero");
    }

    #[test]
    fn there_is_no_settlement_on_a_non_payment_date() {
        let s = Swap::vanilla("irs-6", usd(1_000_000_00), Rate::bps(450), "SOFR", Rate::bps(25), 0, 720, 180, 360)
            .unwrap();
        let obs = Observations::new().fix("SOFR", 500, 4);
        assert_eq!(s.net_settlement(&AllDays, 181, Epoch(5), &obs).unwrap(), None);
    }

    // ── caps and floors ─────────────────────────────────────────────────────────────

    #[test]
    fn a_cap_pays_above_its_strike_and_produces_nothing_below() {
        let c = cap("cap-1", usd(100_000_000), "SOFR", Rate::bps(500), 0, 360, 90, 360).unwrap();
        let ch = chart();

        let high = Observations::new().fix("SOFR", 700, 4);
        let out = c.generate(&AllDays, 90, Epoch(3), &high).unwrap();
        assert_eq!(out.len(), 1);
        out[0].1.clone().seal(&ch, Epoch(3)).unwrap();
        // 2% excess over a quarter of a year on $1,000,000.00 = $5,000.00
        assert_eq!(out[0].1.entries[0].amount.minor, -500_000);

        let low = Observations::new().fix("SOFR", 300, 4);
        assert!(c.generate(&AllDays, 90, Epoch(3), &low).unwrap().is_empty());
    }

    #[test]
    fn a_cap_at_exactly_its_strike_does_not_pay() {
        let c = cap("cap-2", usd(100_000_000), "SOFR", Rate::bps(500), 0, 360, 90, 360).unwrap();
        let at = Observations::new().fix("SOFR", 500, 4);
        assert!(c.generate(&AllDays, 90, Epoch(3), &at).unwrap().is_empty());
    }

    #[test]
    fn a_cap_with_no_fixing_produces_nothing_rather_than_paying_or_not_paying() {
        // The one that matters. Priced at zero, every cap pays; treated as below strike,
        // every cap silently does not. Neither is knowledge, so neither is asserted.
        let c = cap("cap-3", usd(100_000_000), "SOFR", Rate::bps(500), 0, 360, 90, 360).unwrap();
        assert!(c.generate(&AllDays, 90, Epoch(3), &Observations::new()).unwrap().is_empty());
    }

    #[test]
    fn a_floor_is_the_mirror_and_both_check_at_declaration_time() {
        let f = floor("floor-1", usd(100_000_000), "SOFR", Rate::bps(500), 0, 360, 90, 360).unwrap();
        check_with_all_days(&f, &chart(), &fixings()).unwrap();

        let low = Observations::new().fix("SOFR", 300, 4);
        let out = f.generate(&AllDays, 90, Epoch(3), &low).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].1.entries[0].amount.minor, -500_000, "2% shortfall over a quarter");

        let high = Observations::new().fix("SOFR", 700, 4);
        assert!(f.generate(&AllDays, 90, Epoch(3), &high).unwrap().is_empty());
    }

    #[test]
    fn a_cap_and_a_floor_at_one_strike_are_a_collar_that_never_both_fire() {
        // Put-call parity's structural form: at any fixing, exactly one of the two fires,
        // or neither at the strike. Both firing would mean the collar paid twice.
        let c = cap("c", usd(100_000_000), "SOFR", Rate::bps(500), 0, 360, 90, 360).unwrap();
        let f = floor("f", usd(100_000_000), "SOFR", Rate::bps(500), 0, 360, 90, 360).unwrap();
        for fixing in [0i128, 100, 499, 500, 501, 900, 2_000] {
            let obs = Observations::new().fix("SOFR", fixing, 4);
            let cap_fires = !c.generate(&AllDays, 90, Epoch(1), &obs).unwrap().is_empty();
            let floor_fires = !f.generate(&AllDays, 90, Epoch(1), &obs).unwrap().is_empty();
            assert!(!(cap_fires && floor_fires), "both fired at {fixing}");
            if fixing != 500 {
                assert!(cap_fires || floor_fires, "neither fired at {fixing}");
            }
        }
    }

    // ── forwards ────────────────────────────────────────────────────────────────────

    #[test]
    fn a_forward_is_a_one_occurrence_schedule_and_uses_the_same_check() {
        let fwd = forward("fwd-1", usd(500_000_00), Rate::bps(300), 90, 360).unwrap();
        check_with_all_days(&fwd, &chart(), &[Observations::new()]).unwrap();
        assert_eq!(fwd.occurrences(&AllDays).unwrap().len(), 1);
        let out = fwd.generate(&AllDays, 90, Epoch(2), &Observations::new()).unwrap();
        assert_eq!(out.len(), 1);
        out[0].1.clone().seal(&chart(), Epoch(2)).unwrap();
    }

    // ── the refusals ────────────────────────────────────────────────────────────────

    #[test]
    fn a_spread_at_a_coarser_scale_than_the_index_is_lifted_exactly() {
        // A 25bp spread (scale 4) against an index quoted at scale 6. Lifting must be
        // exact; scaling the index *down* to meet the spread would discard precision from
        // the fixing.
        assert_eq!(rescale(Rate::bps(25), 6), Some(2_500));
        assert_eq!(rescale(Rate::bps(25), 4), Some(25));
        assert_eq!(rescale(Rate::new(2_500, 6), 4), None, "never scale a rate down");
    }

    #[test]
    fn a_maturity_before_the_effective_date_is_refused_at_construction() {
        assert!(matches!(
            Swap::vanilla("bad", usd(100), Rate::bps(1), "SOFR", Rate::bps(0), 100, 50, 10, 360),
            Err(DerivativeError::Schedule(_))
        ));
    }
}
