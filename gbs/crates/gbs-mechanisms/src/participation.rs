//! **M4 — fractional participation.** Splitting one amount across a participant set.
//!
//! This is the mechanism syndicated lending, fund and ETF units, supply-chain-finance
//! assignment, waterfall tranches and fee sharing are all made of, and it is the one where
//! money is most often lost — not dramatically, but a cent at a time, invisibly, for years.
//!
//! # The rounding problem, stated exactly
//!
//! Split $100.00 three ways evenly. Each share is 33.333…, which is not representable in
//! cents. Every implementation must decide what to do with the third of a cent, and the
//! decisions that seem reasonable are mostly wrong:
//!
//! * **Round each share independently.** 33.33 × 3 = 99.99. One cent has been destroyed.
//!   With banker's rounding it is sometimes 100.01 and a cent has been *created*, which is
//!   worse, because a system that can create money has no conservation property at all.
//! * **Round each share and post the difference to a rounding account.** Conservation now
//!   holds, but the rounding account accumulates a balance nobody owns and nobody
//!   reconciles, and at year end somebody writes it off. The error was not eliminated; it
//!   was collected in one place and then discarded.
//! * **Use floating point and hope.** 0.1 + 0.2 ≠ 0.3, and the discrepancy is
//!   irreproducible across targets, which defeats Appendix C.4's determinism obligation as
//!   well as conservation.
//!
//! # What this module does instead: largest-remainder with a designated residual holder
//!
//! Shares are **exact rationals**, not decimals — `1/3`, not `0.3333`. The split is computed
//! by the largest-remainder method: give every participant the floor of its exact share,
//! then distribute the remaining units one at a time to the participants with the largest
//! fractional remainders, breaking ties by a **declared order** rather than by iteration
//! order.
//!
//! The result has three properties, and the third is the one that makes this a mechanism
//! rather than a utility:
//!
//! 1. **The parts sum exactly to the whole.** Not approximately — the loop distributes
//!    exactly the units that floor division left over, so the identity is arithmetic rather
//!    than empirical.
//! 2. **Every unit belongs to a named participant.** There is no rounding account. The
//!    extra cent went to a party who can be told they received it.
//! 3. **The allocation is deterministic.** Same shares, same amount, same order, same
//!    answer, on every target and every run. Tie-breaking by declared order rather than by
//!    map iteration is what buys this, and it is the difference between an allocation you
//!    can reproduce at an epoch and one you cannot audit.
//!
//! Largest-remainder is the Hamilton method from apportionment, and it is chosen over
//! the alternatives (D'Hondt, Sainte-Laguë) for a specific reason: it satisfies **quota** —
//! every participant receives either the floor or the ceiling of its exact entitlement, and
//! never anything further away. A participant whose exact share is 33.33 cents receives 33
//! or 34, never 32. The divisor methods can violate that, and a lender receiving less than
//! the floor of its contractual share is a conversation nobody wants to have.

use gbs_kernel::{Amount, KernelError, Party};
use std::collections::BTreeMap;
use std::fmt;

/// An exact rational share. Never a decimal, never a float.
///
/// **Always stored in lowest terms.** An earlier version kept shares unreduced, on the
/// reasoning that contract denominators are small (100, 10 000) so the products would stay
/// bounded. That reasoning was wrong, and the lending product found it: summing eleven
/// shares each with denominator 10 000 by the schoolbook rule `a/b + c/d = (ad+cb)/bd`
/// reaches a denominator of 10^44, which overflows `i128` at around 1.7×10^38.
///
/// An eleven-lender syndicate is not exotic. Reducing on construction costs one `gcd` and
/// removes the failure entirely, and the arithmetic below reduces again after each addition
/// so a long sum cannot grow its denominator either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Share {
    pub num: i128,
    pub den: i128,
}

/// Greatest common divisor, by Euclid. On `i128` so a share of any representable size
/// reduces.
fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.max(1)
}

impl Share {
    /// A share of `num/den`, reduced to lowest terms. Panics on a zero denominator, which is
    /// a programming error rather than a runtime condition — there is no sensible `Result`
    /// for "one over zero" that a caller could act on.
    pub fn new(num: i128, den: i128) -> Self {
        assert!(den != 0, "a share's denominator cannot be zero");
        let (num, den) = if den < 0 { (-num, -den) } else { (num, den) };
        let g = gcd(num, den);
        Share { num: num / g, den: den / g }
    }

    /// `bps` basis points — the unit syndicated loan documents are actually written in.
    pub fn bps(bps: i128) -> Self {
        Share::new(bps, 10_000)
    }

    /// A percentage, exact: `Share::percent(33)` is `33/100`, not `0.33`.
    pub fn percent(p: i128) -> Self {
        Share::new(p, 100)
    }

    pub fn is_zero(&self) -> bool {
        self.num == 0
    }

    fn add(&self, other: &Share) -> Option<Share> {
        // a/b + c/d over the *least* common denominator rather than the product. Using
        // `bd` is what made an eleven-lender sum overflow; using `lcm(b,d)` keeps the
        // denominator at 10 000 however many shares are added.
        let g = gcd(self.den, other.den);
        let lcm = self.den.checked_div(g)?.checked_mul(other.den)?;
        let n = self
            .num
            .checked_mul(lcm.checked_div(self.den)?)?
            .checked_add(other.num.checked_mul(lcm.checked_div(other.den)?)?)?;
        Some(Share::new(n, lcm))
    }

    fn is_one(&self) -> bool {
        self.num == self.den
    }
}

impl fmt::Display for Share {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.num, self.den)
    }
}

/// What a participation can refuse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticipationError {
    /// The shares do not sum to exactly one.
    ///
    /// Carries the sum as an exact rational, so the diagnostic can say `9999/10000` rather
    /// than `0.9999` — which is the difference between "one basis point is missing" and "a
    /// rounding thing, probably fine".
    SharesDoNotSumToOne { sum: Share },
    /// A participant appears twice. Refused rather than merged: two entries for one party
    /// usually means two *different* intended shares, and silently adding them is a guess.
    DuplicateParticipant { party: Party },
    /// No participants. An empty set cannot receive an amount, and splitting into nothing
    /// would destroy the whole of it.
    Empty,
    /// The residual holder is not one of the participants.
    ResidualHolderNotAParticipant { party: Party },
    Arithmetic(KernelError),
}

impl fmt::Display for ParticipationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParticipationError::SharesDoNotSumToOne { sum } => write!(
                f,
                "participation shares sum to {sum}, not 1. A split whose shares do not sum \
                 to one creates or destroys value by construction"
            ),
            ParticipationError::DuplicateParticipant { party } => {
                write!(f, "`{party}` appears twice in the participant set")
            }
            ParticipationError::Empty => write!(f, "a participant set must not be empty"),
            ParticipationError::ResidualHolderNotAParticipant { party } => write!(
                f,
                "`{party}` is named as the residual holder but is not a participant; the \
                 remainder must go to someone who is entitled to it"
            ),
            ParticipationError::Arithmetic(e) => write!(f, "{e}"),
        }
    }
}

/// A participant set: an ordered list of parties with exact shares, plus a residual holder.
///
/// **Order is semantic.** It breaks ties in the largest-remainder distribution, so two sets
/// with the same members in a different order are different sets and may allocate the odd
/// cent differently. That is deliberate: a syndicate agreement names an order of precedence,
/// and reproducing an allocation years later requires that the order be recorded rather than
/// inferred from whatever a hash map did that afternoon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantSet {
    parties: Vec<(Party, Share)>,
    /// Who receives the remainder when the largest-remainder pass runs out of ties to
    /// break. Not a rounding account — a participant, who is entitled to it.
    residual_holder: Party,
}

impl ParticipantSet {
    /// Build a set. Validates that shares sum to exactly one, that no party repeats, and
    /// that the residual holder participates.
    pub fn new(
        parties: Vec<(Party, Share)>,
        residual_holder: Party,
    ) -> Result<Self, ParticipationError> {
        if parties.is_empty() {
            return Err(ParticipationError::Empty);
        }

        let mut seen: BTreeMap<&Party, ()> = BTreeMap::new();
        for (p, _) in &parties {
            if seen.insert(p, ()).is_some() {
                return Err(ParticipationError::DuplicateParticipant { party: p.clone() });
            }
        }

        let mut sum = Share::new(0, 1);
        for (_, s) in &parties {
            sum = sum
                .add(s)
                .ok_or(ParticipationError::Arithmetic(KernelError::Overflow { op: "share sum" }))?;
        }
        if !sum.is_one() {
            return Err(ParticipationError::SharesDoNotSumToOne { sum });
        }

        if !parties.iter().any(|(p, _)| p == &residual_holder) {
            return Err(ParticipationError::ResidualHolderNotAParticipant {
                party: residual_holder,
            });
        }

        Ok(ParticipantSet { parties, residual_holder })
    }

    /// An even split among *n* parties, with the first named as residual holder.
    ///
    /// The convenience that most often hides a rounding bug elsewhere, provided here so it
    /// goes through the same exact-rational path as everything else.
    pub fn even(parties: Vec<Party>) -> Result<Self, ParticipationError> {
        if parties.is_empty() {
            return Err(ParticipationError::Empty);
        }
        let n = parties.len() as i128;
        let residual = parties[0].clone();
        let with_shares = parties.into_iter().map(|p| (p, Share::new(1, n))).collect();
        ParticipantSet::new(with_shares, residual)
    }

    pub fn len(&self) -> usize {
        self.parties.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parties.is_empty()
    }

    pub fn parties(&self) -> impl Iterator<Item = &(Party, Share)> {
        self.parties.iter()
    }

    pub fn residual_holder(&self) -> &Party {
        &self.residual_holder
    }

    /// Allocate `amount` across the set. **The parts sum exactly to the whole.**
    ///
    /// Largest-remainder: floor each exact share, then hand the leftover units out one at a
    /// time in descending remainder order, ties broken by position in the set.
    ///
    /// # Negative amounts
    ///
    /// A repayment, a reversal and a redemption are all negative allocations, and they must
    /// round the *same way* as the positive allocation they reverse, or a drawdown followed
    /// by its exact repayment leaves a residue. This is handled by allocating the magnitude
    /// and negating, so `allocate(-x) == -allocate(x)` element-wise — asserted in the tests,
    /// because it is the property most likely to be broken by a later "simplification".
    pub fn allocate(&self, amount: &Amount) -> Result<Vec<(Party, Amount)>, ParticipationError> {
        let negative = amount.minor < 0;
        let magnitude = amount
            .minor
            .checked_abs()
            .ok_or(ParticipationError::Arithmetic(KernelError::Overflow { op: "abs" }))?;

        // floor(magnitude * num / den) for each participant, plus the remainder numerator
        // that decides who gets the extra units.
        let mut floors: Vec<i128> = Vec::with_capacity(self.parties.len());
        let mut remainders: Vec<(i128, usize)> = Vec::with_capacity(self.parties.len());
        let mut allocated: i128 = 0;

        for (i, (_, share)) in self.parties.iter().enumerate() {
            let scaled = magnitude.checked_mul(share.num).ok_or(ParticipationError::Arithmetic(
                KernelError::Overflow { op: "allocate" },
            ))?;
            let q = scaled.div_euclid(share.den);
            let r = scaled.rem_euclid(share.den);
            floors.push(q);
            // Remainders are compared across participants who may have *different*
            // denominators, so the raw remainder is not comparable. Scale to a common
            // basis: r/den, compared as r * (LCM/den) — here approximated safely by
            // comparing r * (other dens) is expensive, so we compare the rational r/den
            // via cross-multiplication in the sort below.
            remainders.push((r, i));
            allocated = allocated.checked_add(q).ok_or(ParticipationError::Arithmetic(
                KernelError::Overflow { op: "allocate" },
            ))?;
        }

        let mut leftover = magnitude - allocated;

        // Sort by remainder *fraction* descending — r_a/den_a vs r_b/den_b compared by
        // cross-multiplication, which is exact — ties broken by declared position ascending.
        // Position is the tie-break rather than party name, so the syndicate's stated order
        // of precedence decides, which is what the contract says and what an auditor will
        // check.
        remainders.sort_by(|&(ra, ia), &(rb, ib)| {
            let da = self.parties[ia].1.den;
            let db = self.parties[ib].1.den;
            match (rb.saturating_mul(da)).cmp(&(ra.saturating_mul(db))) {
                std::cmp::Ordering::Equal => ia.cmp(&ib),
                other => other,
            }
        });

        let mut idx = 0;
        while leftover > 0 {
            // If every remainder is exhausted and units are still left — only reachable
            // when a share is zero for everyone remaining — the residual holder takes them.
            let target = if idx < remainders.len() {
                remainders[idx].1
            } else {
                self.parties
                    .iter()
                    .position(|(p, _)| p == &self.residual_holder)
                    .expect("validated at construction")
            };
            floors[target] += 1;
            leftover -= 1;
            idx += 1;
        }

        let out = self
            .parties
            .iter()
            .zip(floors)
            .map(|((p, _), m)| {
                let minor = if negative { -m } else { m };
                (p.clone(), Amount::new(minor, amount.currency.clone(), amount.scale))
            })
            .collect();

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Party {
        Party::new(s)
    }

    fn sum_of(alloc: &[(Party, Amount)]) -> i128 {
        alloc.iter().map(|(_, a)| a.minor).sum()
    }

    // ── the property this module exists for ─────────────────────────────────────────

    #[test]
    fn the_parts_sum_exactly_to_the_whole_for_the_classic_bad_case() {
        // $100.00 three ways. Naive rounding gives 33.33 × 3 = 99.99 and destroys a cent.
        let set = ParticipantSet::even(vec![p("a"), p("b"), p("c")]).unwrap();
        let alloc = set.allocate(&Amount::minor_2dp(10_000, "USD")).unwrap();
        assert_eq!(sum_of(&alloc), 10_000, "not one cent may be lost");
        let minors: Vec<i128> = alloc.iter().map(|(_, a)| a.minor).collect();
        assert_eq!(minors, vec![3_334, 3_333, 3_333], "the extra cent goes to the first, by order");
    }

    #[test]
    fn conservation_holds_across_a_wide_sweep_of_amounts_and_set_sizes() {
        // The empirical form of the same claim. Every combination must be exact; a single
        // failure here is a cent created or destroyed.
        for n in 1..=13usize {
            let parties: Vec<Party> = (0..n).map(|i| p(&format!("lender-{i}"))).collect();
            let set = ParticipantSet::even(parties).unwrap();
            for amount in [0i128, 1, 2, 7, 99, 100, 101, 9_999, 10_000, 123_456_789] {
                let a = Amount::minor_2dp(amount, "USD");
                let alloc = set.allocate(&a).unwrap();
                assert_eq!(sum_of(&alloc), amount, "n={n} amount={amount}");
                assert_eq!(alloc.len(), n);
            }
        }
    }

    #[test]
    fn a_repayment_reverses_a_drawdown_exactly() {
        // The property that keeps a loan from leaving a residue: allocate(-x) must be the
        // element-wise negation of allocate(x). If rounding drifted between the two, a
        // drawdown followed by its exact repayment would leave each lender a cent out.
        let set = ParticipantSet::new(
            vec![(p("agent"), Share::bps(4_500)), (p("b"), Share::bps(3_300)), (p("c"), Share::bps(2_200))],
            p("agent"),
        )
        .unwrap();

        for amount in [1i128, 3, 17, 101, 999, 1_000_003] {
            let out = set.allocate(&Amount::minor_2dp(amount, "USD")).unwrap();
            let back = set.allocate(&Amount::minor_2dp(-amount, "USD")).unwrap();
            for (i, ((pa, aa), (pb, ab))) in out.iter().zip(&back).enumerate() {
                assert_eq!(pa, pb, "same order, position {i}");
                assert_eq!(aa.minor, -ab.minor, "amount={amount}, participant {pa}");
            }
            assert_eq!(sum_of(&out) + sum_of(&back), 0);
        }
    }

    #[test]
    fn every_share_lands_on_the_floor_or_the_ceiling_of_its_exact_entitlement() {
        // The **quota** property, which is why largest-remainder was chosen over the
        // divisor methods. A lender contractually entitled to 33.33 cents gets 33 or 34 —
        // never 32. A method that violates quota is a conversation with a syndicate member
        // that nobody wants.
        let set = ParticipantSet::new(
            vec![(p("a"), Share::bps(3_333)), (p("b"), Share::bps(3_333)), (p("c"), Share::bps(3_334))],
            p("c"),
        )
        .unwrap();
        for amount in [7i128, 101, 1_000, 99_999, 1_234_567] {
            let alloc = set.allocate(&Amount::minor_2dp(amount, "USD")).unwrap();
            for ((_, got), (_, share)) in alloc.iter().zip(set.parties()) {
                let exact_num = amount * share.num;
                let floor = exact_num.div_euclid(share.den);
                let ceil = if exact_num.rem_euclid(share.den) == 0 { floor } else { floor + 1 };
                assert!(
                    got.minor == floor || got.minor == ceil,
                    "amount={amount}: got {} for share {share}, expected {floor} or {ceil}",
                    got.minor
                );
            }
        }
    }

    #[test]
    fn allocation_is_deterministic_across_repeated_runs() {
        // Reproducibility: an allocation that cannot be recomputed years later cannot be
        // audited, and `reproduce e at #4200` is the test the thesis sets itself.
        let set = ParticipantSet::new(
            vec![(p("z"), Share::percent(50)), (p("a"), Share::percent(25)), (p("m"), Share::percent(25))],
            p("z"),
        )
        .unwrap();
        let a = set.allocate(&Amount::minor_2dp(10_001, "USD")).unwrap();
        for _ in 0..50 {
            assert_eq!(set.allocate(&Amount::minor_2dp(10_001, "USD")).unwrap(), a);
        }
    }

    #[test]
    fn declared_order_breaks_ties_not_party_name() {
        // Two sets, same members and shares, different declared order. The odd cent follows
        // the *declared* order — which is the contract's order of precedence, recorded
        // rather than inferred.
        let s1 = ParticipantSet::new(
            vec![(p("bravo"), Share::new(1, 3)), (p("alpha"), Share::new(1, 3)), (p("delta"), Share::new(1, 3))],
            p("bravo"),
        )
        .unwrap();
        let s2 = ParticipantSet::new(
            vec![(p("alpha"), Share::new(1, 3)), (p("bravo"), Share::new(1, 3)), (p("delta"), Share::new(1, 3))],
            p("alpha"),
        )
        .unwrap();
        let a1 = s1.allocate(&Amount::minor_2dp(100, "USD")).unwrap();
        let a2 = s2.allocate(&Amount::minor_2dp(100, "USD")).unwrap();
        assert_eq!(a1[0].0, p("bravo"));
        assert_eq!(a1[0].1.minor, 34, "first in declared order takes the extra unit");
        assert_eq!(a2[0].0, p("alpha"));
        assert_eq!(a2[0].1.minor, 34);
    }

    // ── the refusals ────────────────────────────────────────────────────────────────

    #[test]
    fn a_large_syndicate_sums_without_overflowing() {
        // **The regression test for a real defect.** An earlier version kept shares
        // unreduced and summed by `a/b + c/d = (ad+cb)/bd`, on the reasoning that contract
        // denominators are small. Eleven lenders at 10 000ths reaches a denominator of
        // 10^44 and overflows i128 at ~1.7×10^38, so an ordinary syndicated loan failed to
        // construct at all.
        //
        // It was found by the lending product rather than by this module's own tests, which
        // is the layering working: a mechanism's tests use the sizes its author imagined,
        // and a product uses the sizes the business has.
        let mut parties = vec![(p("agent"), Share::bps(2_000))];
        for i in 0..10 {
            parties.push((p(&format!("lender-{i}")), Share::bps(800)));
        }
        let set = ParticipantSet::new(parties, p("agent")).expect("2000 + 10*800 = 10000 bps");
        assert_eq!(set.len(), 11);

        // And much larger sets, with awkward denominators that do not share factors.
        for n in [1usize, 2, 11, 50, 200] {
            let parties: Vec<_> = (0..n).map(|i| (p(&format!("l{i}")), Share::new(1, n as i128))).collect();
            let set = ParticipantSet::new(parties, p("l0")).unwrap_or_else(|e| panic!("n={n}: {e}"));
            let alloc = set.allocate(&Amount::minor_2dp(1_000_003, "USD")).unwrap();
            assert_eq!(sum_of(&alloc), 1_000_003, "n={n}");
        }
    }

    #[test]
    fn shares_are_stored_in_lowest_terms() {
        // The fix, pinned at the smallest scope. If reduction were removed, this fails
        // before the overflow test does, which is a better place to find out.
        assert_eq!(Share::new(50, 100), Share::new(1, 2));
        assert_eq!(Share::bps(2_500), Share::new(1, 4));
        assert_eq!(Share::percent(20), Share::new(1, 5));
        assert_eq!(Share::new(0, 7), Share::new(0, 1), "zero reduces to 0/1");
        assert_eq!(Share::new(3, -6), Share::new(-1, 2), "a negative denominator normalises");
    }

    #[test]
    fn shares_that_do_not_sum_to_one_are_refused_with_the_exact_shortfall() {
        // 99.99% — one basis point missing. The diagnostic must show `9999/10000`, not
        // `0.9999`, because the second reads like a rounding artefact and the first reads
        // like a missing basis point.
        let e = ParticipantSet::new(
            vec![(p("a"), Share::bps(5_000)), (p("b"), Share::bps(4_999))],
            p("a"),
        )
        .unwrap_err();
        assert!(matches!(e, ParticipationError::SharesDoNotSumToOne { .. }));
        let msg = e.to_string();
        assert!(msg.contains("9999") || msg.contains("/"), "{msg}");
        assert!(msg.contains("creates or destroys value"), "{msg}");
    }

    #[test]
    fn shares_summing_to_more_than_one_are_refused_too() {
        let e = ParticipantSet::new(
            vec![(p("a"), Share::percent(60)), (p("b"), Share::percent(60))],
            p("a"),
        );
        assert!(matches!(e, Err(ParticipationError::SharesDoNotSumToOne { .. })));
    }

    #[test]
    fn a_duplicate_participant_is_refused_rather_than_merged() {
        // Merging would be a guess. Two rows for one lender almost always means two
        // different intended shares and a transcription error.
        let e = ParticipantSet::new(
            vec![(p("a"), Share::percent(50)), (p("a"), Share::percent(50))],
            p("a"),
        )
        .unwrap_err();
        assert!(matches!(e, ParticipationError::DuplicateParticipant { .. }));
    }

    #[test]
    fn an_empty_participant_set_is_refused() {
        assert!(matches!(ParticipantSet::even(vec![]), Err(ParticipationError::Empty)));
        assert!(matches!(
            ParticipantSet::new(vec![], p("a")),
            Err(ParticipationError::Empty)
        ));
    }

    #[test]
    fn the_residual_holder_must_be_entitled_to_the_remainder() {
        // No rounding account. The extra cent goes to someone who can be told they got it.
        let e = ParticipantSet::new(
            vec![(p("a"), Share::percent(50)), (p("b"), Share::percent(50))],
            p("rounding-suspense"),
        )
        .unwrap_err();
        assert!(matches!(e, ParticipationError::ResidualHolderNotAParticipant { .. }));
        assert!(e.to_string().contains("entitled to it"));
    }

    // ── edges ───────────────────────────────────────────────────────────────────────

    #[test]
    fn a_zero_share_participant_receives_nothing_and_the_set_still_conserves() {
        // Real: a lender whose commitment has been fully repaid but who remains a party to
        // the agreement. They must stay in the set — dropping them would change the
        // tie-break order and therefore the allocation.
        let set = ParticipantSet::new(
            vec![(p("a"), Share::percent(100)), (p("retired"), Share::new(0, 1))],
            p("a"),
        )
        .unwrap();
        let alloc = set.allocate(&Amount::minor_2dp(777, "USD")).unwrap();
        assert_eq!(sum_of(&alloc), 777);
        assert_eq!(alloc[1].1.minor, 0);
        assert!(set.parties().any(|(q, s)| q == &p("retired") && s.is_zero()));
    }

    #[test]
    fn a_single_participant_receives_everything() {
        let set = ParticipantSet::even(vec![p("only")]).unwrap();
        let alloc = set.allocate(&Amount::minor_2dp(1, "USD")).unwrap();
        assert_eq!(alloc[0].1.minor, 1);
    }

    #[test]
    fn one_unit_split_many_ways_goes_to_exactly_one_participant() {
        // A cent split seven ways. Six get nothing, one gets the cent, and the total is one.
        // The failure mode this catches: seven participants each receiving a rounded-up
        // cent, creating six cents from nothing.
        let set = ParticipantSet::even((0..7).map(|i| p(&format!("l{i}"))).collect()).unwrap();
        let alloc = set.allocate(&Amount::minor_2dp(1, "USD")).unwrap();
        assert_eq!(sum_of(&alloc), 1);
        assert_eq!(alloc.iter().filter(|(_, a)| a.minor == 1).count(), 1);
        assert_eq!(alloc.iter().filter(|(_, a)| a.minor == 0).count(), 6);
    }

    #[test]
    fn allocating_zero_gives_everyone_zero_and_conserves() {
        let set = ParticipantSet::even(vec![p("a"), p("b"), p("c")]).unwrap();
        let alloc = set.allocate(&Amount::minor_2dp(0, "USD")).unwrap();
        assert_eq!(sum_of(&alloc), 0);
        assert!(alloc.iter().all(|(_, a)| a.minor == 0));
        assert_eq!(alloc.len(), 3, "everyone is still listed, at zero");
    }

    #[test]
    fn the_allocated_currency_and_scale_follow_the_input() {
        let set = ParticipantSet::even(vec![p("a"), p("b")]).unwrap();
        let btc = Amount::new(3, gbs_kernel::Currency::new("BTC"), 8);
        let alloc = set.allocate(&btc).unwrap();
        assert_eq!(sum_of(&alloc), 3);
        assert!(alloc.iter().all(|(_, a)| a.scale == 8 && a.currency.code() == "BTC"));
    }

    #[test]
    fn uneven_shares_with_different_denominators_still_conserve() {
        // 1/2, 1/3, 1/6 — three different denominators, which is where a remainder
        // comparison that ignored the denominator would go wrong.
        let set = ParticipantSet::new(
            vec![(p("a"), Share::new(1, 2)), (p("b"), Share::new(1, 3)), (p("c"), Share::new(1, 6))],
            p("a"),
        )
        .unwrap();
        for amount in [1i128, 5, 7, 100, 101, 999, 100_003] {
            let alloc = set.allocate(&Amount::minor_2dp(amount, "USD")).unwrap();
            assert_eq!(sum_of(&alloc), amount, "amount={amount}");
        }
    }
}
