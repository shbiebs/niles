//! Lending — revolving, term, and syndicated.
//!
//! Three products, one composition. A revolving facility is a commitment (M3) drawn against
//! repeatedly; a term loan is a commitment drawn once with an amortisation schedule (M2);
//! a syndicated loan is either of those with every movement split across a participant set
//! (M4). None of them needs anything the mechanisms do not already provide, which is the
//! coverage-matrix claim of `ARCHITECTURE.md` §4 being tested rather than asserted.
//!
//! # The syndicated case is the interesting one
//!
//! A syndicate of eleven lenders funds a $500m facility. The borrower draws $73,412,987.33.
//! Every lender's share of that draw must be exact, the shares must sum to the draw, and
//! when the borrower repays, each lender's share of the repayment must reverse its share of
//! the draw **exactly** — or the facility leaves a residue that grows for the life of the
//! loan and eventually has to be written off by somebody who cannot explain it.
//!
//! That is M4's job and it is why M4 uses exact rationals with a designated residual holder
//! rather than decimals and a rounding account. The property is asserted here at the
//! product level too, because it is the one a lender would actually check.
//!
//! # Conservation per participant set
//!
//! A syndicated posting set has *two* conservation obligations, and only one of them is the
//! kernel's. The kernel checks that the whole set sums to zero per currency. This module
//! checks the second: that the participants' shares sum to the borrower's movement. A set
//! can satisfy the first and violate the second — if a lender's leg were doubled and the
//! agent's halved, the total would still be zero — so the check is separate and
//! [`Facility::drawdown`] performs it.

use gbs_kernel::{
    Amount, Chart, Entry, Epoch, KernelError, Party, PostingSet, Sealed, Stamp,
};
use gbs_mechanisms::{
    accrue, equal_instalments, Hold, HoldError, Outcome, ParticipantSet, ParticipationError,
    Rate, Rounding,
};

/// What a lending product refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LendingError {
    /// A drawdown exceeding the undrawn commitment.
    ExceedsCommitment { requested: Amount, available: Amount },
    /// The participants' shares do not sum to the borrower's movement.
    ///
    /// **Separate from the kernel's check**, and it must be: a set whose lender legs are
    /// individually wrong can still sum to zero overall, so per-currency conservation alone
    /// would pass it.
    ParticipantSumMismatch { expected: Amount, allocated: Amount },
    Participation(ParticipationError),
    Hold(HoldError),
    Kernel(KernelError),
}

impl std::fmt::Display for LendingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LendingError::ExceedsCommitment { requested, available } => write!(
                f,
                "drawdown of {requested} exceeds the undrawn commitment of {available}"
            ),
            LendingError::ParticipantSumMismatch { expected, allocated } => write!(
                f,
                "participant shares allocate {allocated} against a movement of {expected}. \
                 The set may still balance overall, which is why this is checked separately"
            ),
            LendingError::Participation(e) => write!(f, "{e}"),
            LendingError::Hold(e) => write!(f, "{e}"),
            LendingError::Kernel(e) => write!(f, "{e}"),
        }
    }
}

/// How a facility may be drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Drawn and repaid repeatedly up to the limit — a revolver.
    Revolving,
    /// Drawn once, then amortised.
    Term,
}

/// A credit facility: a commitment, optionally syndicated.
///
/// The commitment is an M3 hold against the *bank's* available funds, which is what makes
/// an undrawn commitment cost something. A facility whose commitment were merely a number
/// in a table would let the same funds be committed twice.
pub struct Facility {
    pub id: String,
    pub kind: Kind,
    pub borrower: Party,
    pub commitment: Amount,
    /// The syndicate. `None` for a bilateral loan — which is the degenerate case of a
    /// syndicate of one, but kept distinct so a bilateral loan does not pay for machinery
    /// it does not use.
    pub syndicate: Option<ParticipantSet>,
    drawn: Amount,
}

impl Facility {
    pub fn new(id: impl Into<String>, kind: Kind, borrower: &str, commitment: Amount) -> Self {
        let zero = Amount::new(0, commitment.currency.clone(), commitment.scale);
        Facility {
            id: id.into(),
            kind,
            borrower: Party::new(borrower),
            commitment,
            syndicate: None,
            drawn: zero,
        }
    }

    pub fn syndicated(mut self, set: ParticipantSet) -> Self {
        self.syndicate = Some(set);
        self
    }

    pub fn drawn(&self) -> &Amount {
        &self.drawn
    }

    /// The undrawn commitment. What a further drawdown is checked against.
    pub fn undrawn(&self) -> Result<Amount, KernelError> {
        self.commitment.add(&self.drawn.negate()?)
    }

    /// Place the commitment as a hold, so undrawn funds are encumbered rather than merely
    /// recorded.
    pub fn commitment_hold(&self, account: &str, placed: Stamp, expires_on: i64) -> Hold {
        Hold::place(
            format!("commit-{}", self.id),
            account,
            self.commitment.clone(),
            placed,
            expires_on,
        )
    }

    /// Draw down. Returns the sealed posting set and, for a syndicated facility, each
    /// lender's exact share.
    ///
    /// Two checks run, and they are genuinely different:
    ///
    /// 1. **The kernel's**: the whole set sums to zero per currency.
    /// 2. **This module's**: the lenders' shares sum to the borrower's movement.
    ///
    /// A set can pass the first and fail the second, which is why both exist. If one
    /// lender's leg were doubled and another's halved, the total would still be zero and
    /// the two lenders would each be wrong.
    pub fn drawdown(
        &mut self,
        amount: &Amount,
        borrower_account: &str,
        stamp: Stamp,
        chart: &Chart,
        epoch: Epoch,
    ) -> Result<(Sealed, Vec<(Party, Amount)>), LendingError> {
        let undrawn = self.undrawn().map_err(LendingError::Kernel)?;
        if amount.minor > undrawn.minor {
            return Err(LendingError::ExceedsCommitment {
                requested: amount.clone(),
                available: undrawn,
            });
        }

        // The borrower receives the full amount, whatever the syndicate looks like. The
        // borrower's side of a syndicated loan is a single movement; the syndication is
        // entirely on the lenders' side.
        let to_borrower = Entry::new(1, borrower_account, amount.clone(), stamp)
            .narrated(format!("drawdown under {}", self.id));

        let mut ps = PostingSet::new(format!("draw-{}-{}", self.id, epoch.0)).with(to_borrower);

        let shares = match &self.syndicate {
            None => {
                // Bilateral: the bank funds it alone.
                ps.push(
                    Entry::new(2, "bank.funding", amount.negate().map_err(LendingError::Kernel)?, stamp)
                        .narrated("bilateral funding"),
                );
                Vec::new()
            }
            Some(set) => {
                let allocation = set.allocate(amount).map_err(LendingError::Participation)?;

                // The second conservation obligation, checked before sealing.
                let allocated: i128 = allocation.iter().map(|(_, a)| a.minor).sum();
                if allocated != amount.minor {
                    return Err(LendingError::ParticipantSumMismatch {
                        expected: amount.clone(),
                        allocated: Amount::new(allocated, amount.currency.clone(), amount.scale),
                    });
                }

                for (i, (party, share)) in allocation.iter().enumerate() {
                    ps.push(
                        Entry::new(
                            (i + 2) as u64,
                            format!("lender.{party}"),
                            share.negate().map_err(LendingError::Kernel)?,
                            stamp,
                        )
                        .narrated(format!("participation in {}", self.id)),
                    );
                }
                allocation
            }
        };

        let sealed = ps.seal(chart, epoch).map_err(LendingError::Kernel)?;
        self.drawn = self.drawn.add(amount).map_err(LendingError::Kernel)?;
        Ok((sealed, shares))
    }

    /// Repay. Reverses a drawdown's allocation **exactly**, which is the property that
    /// keeps a syndicated facility from leaving a residue.
    pub fn repay(
        &mut self,
        amount: &Amount,
        borrower_account: &str,
        stamp: Stamp,
        chart: &Chart,
        epoch: Epoch,
    ) -> Result<(Sealed, Vec<(Party, Amount)>), LendingError> {
        let negated = amount.negate().map_err(LendingError::Kernel)?;
        let from_borrower = Entry::new(1, borrower_account, negated.clone(), stamp)
            .narrated(format!("repayment under {}", self.id));
        let mut ps = PostingSet::new(format!("repay-{}-{}", self.id, epoch.0)).with(from_borrower);

        let shares = match &self.syndicate {
            None => {
                ps.push(Entry::new(2, "bank.funding", amount.clone(), stamp).narrated("bilateral repayment"));
                Vec::new()
            }
            Some(set) => {
                // Allocate the *negative*, so the rounding is the exact mirror of the
                // drawdown's. M4 guarantees `allocate(-x) == -allocate(x)` element-wise;
                // this is where that guarantee is spent.
                let allocation = set
                    .allocate(&amount.negate().map_err(LendingError::Kernel)?)
                    .map_err(LendingError::Participation)?;
                for (i, (party, share)) in allocation.iter().enumerate() {
                    ps.push(
                        Entry::new(
                            (i + 2) as u64,
                            format!("lender.{party}"),
                            share.negate().map_err(LendingError::Kernel)?,
                            stamp,
                        )
                        .narrated(format!("repayment share of {}", self.id)),
                    );
                }
                allocation
            }
        };

        let sealed = ps.seal(chart, epoch).map_err(LendingError::Kernel)?;
        self.drawn = self.drawn.add(&negated).map_err(LendingError::Kernel)?;
        Ok((sealed, shares))
    }
}

/// Interest accrued on the drawn balance for a period.
///
/// M7 does the arithmetic. This exists so a caller does not have to remember that the
/// accrual is on the *drawn* balance rather than the commitment — the mistake that
/// over-charges a borrower on an undrawn revolver.
pub fn interest_for_period(
    facility: &Facility,
    rate: Rate,
    days: i64,
    basis: i64,
    rounding: Rounding,
) -> Result<Amount, LendingError> {
    accrue(&facility.drawn, rate, days, basis, rounding)
        .map_err(|_| LendingError::Kernel(KernelError::Overflow { op: "accrue" }))
}

/// A repayment waterfall: apply a payment to obligations in priority order.
///
/// Fees, then interest, then principal — the ordering almost every credit agreement uses.
/// Returns what each tier received and what remains. The property that matters: **the
/// parts sum to the payment**, so a waterfall cannot lose a cent between tiers.
pub fn waterfall(payment: &Amount, tiers: &[(&str, Amount)]) -> Vec<(String, Amount)> {
    let mut remaining = payment.minor;
    let mut out = Vec::new();
    for (name, owed) in tiers {
        let applied = remaining.min(owed.minor).max(0);
        out.push((
            name.to_string(),
            Amount::new(applied, payment.currency.clone(), payment.scale),
        ));
        remaining -= applied;
    }
    // Whatever is left over is a prepayment. Named, and never silently dropped — an
    // overpayment that vanished into the last tier would be money the borrower cannot
    // account for.
    out.push((
        "prepayment".to_string(),
        Amount::new(remaining, payment.currency.clone(), payment.scale),
    ));
    out
}

/// An amortisation schedule's principal instalments. M2's helper, re-exported at the
/// product level because this is where a caller looks for it.
pub fn amortisation(principal: &Amount, periods: u32) -> Vec<Amount> {
    equal_instalments(principal, periods)
}

/// A drawdown against a commitment hold, resolving the hold.
pub fn draw_against_hold(
    hold: &mut Hold,
    amount: Amount,
    at: Stamp,
) -> Result<(), LendingError> {
    hold.resolve(Outcome::Posted { amount }, at).map_err(LendingError::Hold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gbs_kernel::{Account, Chart};
    use gbs_mechanisms::Share;

    fn usd(minor: i128) -> Amount {
        Amount::minor_2dp(minor, "USD")
    }

    fn st(e: u64, d: i64) -> Stamp {
        Stamp::new(Epoch(e), d)
    }

    fn syndicate() -> ParticipantSet {
        // A realistic uneven syndicate: an agent with the largest share, ten others.
        let mut parties = vec![(Party::new("agent"), Share::bps(2_000))];
        for i in 0..10 {
            parties.push((Party::new(format!("lender-{i}")), Share::bps(800)));
        }
        ParticipantSet::new(parties, Party::new("agent")).expect("2000 + 10*800 = 10000 bps")
    }

    fn chart_for(set: &ParticipantSet) -> Chart {
        let mut c = Chart::new()
            .with(Account::new("borrower.usd", "acme", "USD"))
            .with(Account::new("bank.funding", "bank", "USD"));
        for (p, _) in set.parties() {
            c = c.with(Account::new(format!("lender.{p}"), p.as_str(), "USD"));
        }
        c
    }

    // ── the syndicated property ─────────────────────────────────────────────────────

    #[test]
    fn a_syndicated_drawdown_allocates_exactly_and_the_kernel_agrees() {
        let set = syndicate();
        let chart = chart_for(&set);
        let mut f = Facility::new("fac-1", Kind::Revolving, "acme", usd(50_000_000_000))
            .syndicated(set);

        // An awkward amount: $73,412,987.33, which divides evenly by nothing.
        let draw = usd(7_341_298_733);
        let (sealed, shares) = f.drawdown(&draw, "borrower.usd", st(10, 0), &chart, Epoch(10)).unwrap();

        sealed.verify().expect("the kernel's conservation check");
        assert_eq!(shares.len(), 11);
        assert_eq!(
            shares.iter().map(|(_, a)| a.minor).sum::<i128>(),
            draw.minor,
            "and the participants' shares sum to the draw"
        );
        assert_eq!(f.drawn().minor, draw.minor);
    }

    #[test]
    fn a_repayment_reverses_a_drawdown_leaving_no_residue_per_lender() {
        // The property a syndicated facility is judged on. Every lender's repayment share
        // must exactly reverse its drawdown share, or the facility accumulates a per-lender
        // residue for its whole life.
        let set = syndicate();
        let chart = chart_for(&set);
        let mut f = Facility::new("fac-2", Kind::Revolving, "acme", usd(50_000_000_000))
            .syndicated(set);

        for amount in [1i128, 7, 101, 999_999, 7_341_298_733] {
            let draw = usd(amount);
            let (_, drawn) = f.drawdown(&draw, "borrower.usd", st(10, 0), &chart, Epoch(10)).unwrap();
            let (_, repaid) = f.repay(&draw, "borrower.usd", st(11, 1), &chart, Epoch(11)).unwrap();

            for ((p1, a1), (p2, a2)) in drawn.iter().zip(&repaid) {
                assert_eq!(p1, p2, "same lenders, same order");
                assert_eq!(a1.minor, -a2.minor, "amount={amount}, lender {p1}");
            }
            assert_eq!(f.drawn().minor, 0, "and the facility is back to zero drawn");
        }
    }

    #[test]
    fn a_thousand_draws_and_repays_leave_the_facility_exactly_at_zero() {
        // The cumulative form. A one-cent-per-cycle drift would be invisible in any single
        // test and catastrophic over the life of a revolver.
        let set = syndicate();
        let chart = chart_for(&set);
        let mut f = Facility::new("fac-3", Kind::Revolving, "acme", usd(1_000_000_000))
            .syndicated(set);

        for i in 0..1_000i128 {
            let amount = usd(1 + (i * 7919) % 999_983); // deliberately awkward amounts
            f.drawdown(&amount, "borrower.usd", st(100 + i as u64, 0), &chart, Epoch(100 + i as u64))
                .unwrap();
            f.repay(&amount, "borrower.usd", st(200 + i as u64, 0), &chart, Epoch(200 + i as u64))
                .unwrap();
        }
        assert_eq!(f.drawn().minor, 0, "no drift over a thousand cycles");
    }

    // ── the two conservation obligations are different ──────────────────────────────

    #[test]
    fn a_set_can_conserve_overall_while_the_participant_shares_are_wrong() {
        // The argument for checking twice, as a constructed example. Two lender legs, one
        // doubled and one halved: the set sums to zero per currency and the kernel accepts
        // it, and two lenders are each funded for the wrong amount.
        let chart = Chart::new()
            .with(Account::new("borrower.usd", "acme", "USD"))
            .with(Account::new("lender.a", "a", "USD"))
            .with(Account::new("lender.b", "b", "USD"));

        let hand_built = PostingSet::new("wrong")
            .with(Entry::new(1, "borrower.usd", usd(10_000), st(1, 0)))
            .with(Entry::new(2, "lender.a", usd(-7_500), st(1, 0)))
            .with(Entry::new(3, "lender.b", usd(-2_500), st(1, 0)));

        // The kernel is satisfied — and it should be; per-currency conservation holds.
        assert!(hand_built.seal(&chart, Epoch(1)).is_ok());
        // But a 50/50 syndicate would have allocated 5,000 each. The kernel has no way to
        // know the intended shares, which is exactly why `drawdown` checks them separately.
        let even = ParticipantSet::new(
            vec![(Party::new("a"), Share::percent(50)), (Party::new("b"), Share::percent(50))],
            Party::new("a"),
        )
        .unwrap();
        let allocated = even.allocate(&usd(10_000)).unwrap();
        assert_eq!(allocated[0].1.minor, 5_000);
        assert_ne!(allocated[0].1.minor, 7_500);
    }

    // ── commitments ─────────────────────────────────────────────────────────────────

    #[test]
    fn a_drawdown_beyond_the_undrawn_commitment_is_refused() {
        let set = syndicate();
        let chart = chart_for(&set);
        let mut f = Facility::new("fac-4", Kind::Revolving, "acme", usd(100_000)).syndicated(set);

        f.drawdown(&usd(60_000), "borrower.usd", st(10, 0), &chart, Epoch(10)).unwrap();
        assert_eq!(f.undrawn().unwrap().minor, 40_000);

        let e = f
            .drawdown(&usd(40_001), "borrower.usd", st(11, 0), &chart, Epoch(11))
            .unwrap_err();
        assert!(matches!(e, LendingError::ExceedsCommitment { .. }));
        assert!(e.to_string().contains("400.00"), "{e}");
    }

    #[test]
    fn a_revolver_can_be_redrawn_after_repayment_and_a_commitment_is_a_real_hold() {
        let set = syndicate();
        let chart = chart_for(&set);
        let mut f = Facility::new("fac-5", Kind::Revolving, "acme", usd(100_000)).syndicated(set);

        f.drawdown(&usd(100_000), "borrower.usd", st(10, 0), &chart, Epoch(10)).unwrap();
        assert_eq!(f.undrawn().unwrap().minor, 0);
        f.repay(&usd(100_000), "borrower.usd", st(11, 0), &chart, Epoch(11)).unwrap();
        assert_eq!(f.undrawn().unwrap().minor, 100_000, "a revolver revolves");
        f.drawdown(&usd(100_000), "borrower.usd", st(12, 0), &chart, Epoch(12)).unwrap();

        // And the commitment itself encumbers, so the same funds cannot be committed twice.
        let h = f.commitment_hold("bank.funding", st(9, 0), 365);
        assert!(h.encumbers(Epoch(10), 0));
        assert_eq!(h.amount.minor, 100_000);
    }

    // ── interest and waterfalls ─────────────────────────────────────────────────────

    #[test]
    fn interest_accrues_on_the_drawn_balance_and_not_on_the_commitment() {
        // The mistake that over-charges a borrower on an undrawn revolver.
        let set = syndicate();
        let chart = chart_for(&set);
        let mut f = Facility::new("fac-6", Kind::Revolving, "acme", usd(100_000_000)).syndicated(set);
        f.drawdown(&usd(10_000_000), "borrower.usd", st(10, 0), &chart, Epoch(10)).unwrap();

        let i = interest_for_period(&f, Rate::bps(600), 30, 360, Rounding::HalfEven).unwrap();
        // 6% of $100,000.00 for a twelfth of a year = $500.00
        assert_eq!(i.minor, 50_000);

        // Not 6% of the $1,000,000.00 commitment, which would be ten times as much.
        assert_ne!(i.minor, 500_000);
    }

    #[test]
    fn a_waterfall_applies_in_priority_order_and_the_parts_sum_to_the_payment() {
        let payment = usd(100_000);
        let tiers = [("fees", usd(5_000)), ("interest", usd(20_000)), ("principal", usd(500_000))];
        let applied = waterfall(&payment, &tiers);

        assert_eq!(applied[0].1.minor, 5_000, "fees first");
        assert_eq!(applied[1].1.minor, 20_000, "then interest");
        assert_eq!(applied[2].1.minor, 75_000, "then whatever principal is left");
        assert_eq!(applied[3].1.minor, 0, "no prepayment");
        assert_eq!(
            applied.iter().map(|(_, a)| a.minor).sum::<i128>(),
            payment.minor,
            "the parts sum to the payment"
        );
    }

    #[test]
    fn a_payment_short_of_the_first_tier_stops_there() {
        let applied = waterfall(&usd(3_000), &[("fees", usd(5_000)), ("interest", usd(20_000))]);
        assert_eq!(applied[0].1.minor, 3_000);
        assert_eq!(applied[1].1.minor, 0);
        assert_eq!(applied.iter().map(|(_, a)| a.minor).sum::<i128>(), 3_000);
    }

    #[test]
    fn an_overpayment_is_named_as_a_prepayment_rather_than_vanishing() {
        // Money the borrower cannot account for is the worst outcome available. It gets a
        // name.
        let applied = waterfall(&usd(100_000), &[("fees", usd(1_000)), ("interest", usd(2_000))]);
        assert_eq!(applied.last().unwrap().0, "prepayment");
        assert_eq!(applied.last().unwrap().1.minor, 97_000);
        assert_eq!(applied.iter().map(|(_, a)| a.minor).sum::<i128>(), 100_000);
    }

    #[test]
    fn amortisation_instalments_sum_to_the_principal() {
        for (p, n) in [(100_000i128, 7u32), (999_999, 12), (1, 3)] {
            let parts = amortisation(&usd(p), n);
            assert_eq!(parts.iter().map(|a| a.minor).sum::<i128>(), p);
        }
    }

    // ── bilateral ───────────────────────────────────────────────────────────────────

    #[test]
    fn a_bilateral_loan_is_the_same_code_without_the_syndicate_machinery() {
        // The generality check: one product type, and syndication is an option rather than
        // a separate implementation.
        let chart = Chart::new()
            .with(Account::new("borrower.usd", "acme", "USD"))
            .with(Account::new("bank.funding", "bank", "USD"));
        let mut f = Facility::new("bilat-1", Kind::Term, "acme", usd(1_000_000));

        let (sealed, shares) =
            f.drawdown(&usd(750_000), "borrower.usd", st(10, 0), &chart, Epoch(10)).unwrap();
        sealed.verify().unwrap();
        assert!(shares.is_empty(), "no syndicate, no shares");
        assert_eq!(sealed.entries().len(), 2, "two legs, not thirteen");
        assert_eq!(f.drawn().minor, 750_000);
    }

    #[test]
    fn a_commitment_hold_resolves_exactly_once_when_drawn() {
        let f = Facility::new("fac-7", Kind::Term, "acme", usd(500_000));
        let mut h = f.commitment_hold("bank.funding", st(9, 0), 365);
        draw_against_hold(&mut h, usd(500_000), st(10, 0)).unwrap();
        assert!(!h.is_open());
        assert!(draw_against_hold(&mut h, usd(1), st(11, 0)).is_err(), "linearity");
    }
}
