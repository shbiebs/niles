//! **M3 — holds and commitments.** An affine reservation resolved exactly once.
//!
//! A hold reserves value against an account without moving it. It must be resolved exactly
//! once — **posted**, **voided**, or **expired** — and each of those is a resolution. An
//! expiry is not the absence of a decision; it is a decision the calendar made.
//!
//! This is the mechanism behind card authorisation, loan commitments and drawdown, letter
//! of credit issuance, margin and collateral, settlement obligations, and fund subscription
//! and redemption windows.
//!
//! # Why this is the mechanism the regulator noticed
//!
//! §1.1 of the thesis opens with authorize-positive-settle-negative — the FDIC and CFPB fee
//! practice that arises because the *available balance* (which nets authorised-but-unsettled
//! holds) and the *ledger balance* (which counts only settled movements) can disagree at the
//! moment a decision is made. That is not an obscure edge case; it is a supervisory finding
//! about two views of one ledger differing in exactly which pending facts they include.
//!
//! The design here is what removes it. A hold is a **fact on the ledger**, sealed in an
//! epoch, not a row in a side table with its own lifecycle. Available balance and ledger
//! balance are therefore two REVs over the same prefix, differing only in whether they fold
//! unresolved holds. Anchored at the same epoch, they cannot disagree about which holds
//! exist, because there is only one answer to that question and both are reading it.
//!
//! # Linearity, and why an expiry is a resolution
//!
//! The obligation is affine: at most one resolution, and — for a hold that has expired —
//! exactly one. The thesis checks this statically (Contribution 4's linearity judgement);
//! this module checks it dynamically as the runtime half of the same guarantee.
//!
//! Modelling expiry as a resolution rather than as "no resolution yet, past the date" is the
//! decision that makes the state machine total. The alternative leaves a hold in a state
//! that is neither open nor closed, and every consumer has to re-derive the answer from a
//! clock — which is how two consumers come to disagree about whether a hold is live.

use gbs_kernel::{AccountId, Amount, Entry, KernelError, PostingSet, Stamp, Epoch};
use std::fmt;

/// A hold's identity, and its idempotency key.
///
/// Same reasoning as [`TxnId`](gbs_kernel::TxnId): a re-delivered authorisation message must
/// reserve once, and the identity of the business event is what decides sameness.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HoldId(String);

impl HoldId {
    pub fn new(id: impl Into<String>) -> Self {
        HoldId(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HoldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// How a hold ends. Every variant is a resolution; there is no fourth state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Settled, for an amount that may differ from the reservation.
    ///
    /// The amount is carried because **a settlement almost never equals its authorisation**:
    /// a restaurant tip, a fuel pump's pre-authorisation, a partial shipment under a letter
    /// of credit. A design that assumed equality would be wrong on the common case and
    /// would force every product to work around it.
    Posted { amount: Amount },
    /// Released without settling — a cancelled order, a reversed authorisation.
    Void,
    /// The reservation window closed. A decision, made by the calendar.
    Expired,
}

impl Outcome {
    pub fn moved_value(&self) -> bool {
        matches!(self, Outcome::Posted { .. })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Outcome::Posted { .. } => "posted",
            Outcome::Void => "void",
            Outcome::Expired => "expired",
        }
    }
}

/// What a hold refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoldError {
    /// The linearity violation. Names both resolutions, because the interesting question is
    /// always *which two* — a void followed by a post is a different bug from two posts.
    AlreadyResolved { id: HoldId, first: String, second: String },
    /// A settlement in a currency other than the reservation's. Refused: a hold reserves a
    /// specific currency against a specific account, and settling in another is an FX
    /// transaction, which is two conserved legs rather than a resolution.
    CurrencyChanged { id: HoldId, reserved: String, settled: String },
    /// A settlement exceeding the reservation by more than the declared tolerance.
    OverSettled { id: HoldId, reserved: Amount, settled: Amount, tolerance_bps: u32 },
    /// Settling a negative amount. A hold reserves value; a negative settlement would
    /// return more than was held.
    NegativeSettlement { id: HoldId },
    Arithmetic(KernelError),
}

impl fmt::Display for HoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HoldError::AlreadyResolved { id, first, second } => write!(
                f,
                "hold `{id}` was already resolved as {first}; cannot now resolve it as \
                 {second}. A hold is resolved exactly once"
            ),
            HoldError::CurrencyChanged { id, reserved, settled } => write!(
                f,
                "hold `{id}` reserved {reserved} but settlement is in {settled}; \
                 cross-currency settlement is an FX transaction, not a resolution"
            ),
            HoldError::OverSettled { id, reserved, settled, tolerance_bps } => write!(
                f,
                "hold `{id}` reserved {reserved} but settlement is {settled}, over the \
                 {tolerance_bps} bp tolerance"
            ),
            HoldError::NegativeSettlement { id } => {
                write!(f, "hold `{id}` cannot settle a negative amount")
            }
            HoldError::Arithmetic(e) => write!(f, "{e}"),
        }
    }
}

/// A reservation against an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hold {
    pub id: HoldId,
    pub account: AccountId,
    pub amount: Amount,
    pub placed: Stamp,
    /// The valid-time day on which an unresolved hold expires.
    ///
    /// Mandatory, with no `Option`. A hold with no expiry is a reservation that reduces an
    /// available balance forever, and every institution that permits one accumulates them.
    /// Making it required forces the question to be answered at the point where somebody
    /// knows the answer.
    pub expires_on: i64,
    /// How far over the reservation a settlement may go, in basis points.
    ///
    /// Zero for a strict reservation. Non-zero for the cases where over-settlement is
    /// contractual rather than erroneous — a card tip allowance is the canonical one, and
    /// hard-coding "20%" into a card product is how that number becomes unfindable.
    pub tolerance_bps: u32,
    resolution: Option<(Outcome, Stamp)>,
}

impl Hold {
    pub fn place(
        id: impl Into<String>,
        account: impl Into<String>,
        amount: Amount,
        placed: Stamp,
        expires_on: i64,
    ) -> Self {
        Hold {
            id: HoldId::new(id),
            account: AccountId::new(account),
            amount,
            placed,
            expires_on,
            tolerance_bps: 0,
            resolution: None,
        }
    }

    pub fn with_tolerance_bps(mut self, bps: u32) -> Self {
        self.tolerance_bps = bps;
        self
    }

    pub fn is_open(&self) -> bool {
        self.resolution.is_none()
    }

    pub fn outcome(&self) -> Option<&Outcome> {
        self.resolution.as_ref().map(|(o, _)| o)
    }

    /// Whether this hold reduces an available balance as read at `anchor` on `as_of`.
    ///
    /// The predicate the available-balance REV folds over, and the reason available and
    /// ledger balance cannot disagree: **both** are computed from the same prefix at the
    /// same anchor, and this function is the only thing that differs between them.
    ///
    /// Note what it does *not* consult: a clock. Whether a hold is live is decided by the
    /// anchor and the as-of date the caller supplies, so two readers asking the same
    /// question at the same anchor get the same answer, even if one of them asks an hour
    /// later.
    pub fn encumbers(&self, anchor: Epoch, as_of: i64) -> bool {
        if !self.placed.visible_at(anchor) {
            return false;
        }
        match &self.resolution {
            // Resolved, and the resolution is visible: no longer an encumbrance.
            Some((_, at)) if at.visible_at(anchor) => false,
            // Either unresolved, or resolved after the anchor — in which case, as of the
            // anchor, it was still open. This is the case a system reading a mutable status
            // column gets wrong, because the column has already been overwritten.
            _ => as_of <= self.expires_on,
        }
    }

    /// Resolve. Refuses a second resolution, naming both.
    pub fn resolve(&mut self, outcome: Outcome, at: Stamp) -> Result<(), HoldError> {
        if let Some((first, _)) = &self.resolution {
            return Err(HoldError::AlreadyResolved {
                id: self.id.clone(),
                first: first.name().to_string(),
                second: outcome.name().to_string(),
            });
        }
        if let Outcome::Posted { amount } = &outcome {
            if amount.currency != self.amount.currency {
                return Err(HoldError::CurrencyChanged {
                    id: self.id.clone(),
                    reserved: self.amount.currency.to_string(),
                    settled: amount.currency.to_string(),
                });
            }
            if amount.minor < 0 {
                return Err(HoldError::NegativeSettlement { id: self.id.clone() });
            }
            // reserved * (10000 + tolerance) / 10000, computed as a single checked
            // multiplication before the division so no precision is lost on the way.
            let ceiling = self
                .amount
                .minor
                .checked_mul(10_000i128 + self.tolerance_bps as i128)
                .ok_or(HoldError::Arithmetic(KernelError::Overflow { op: "tolerance" }))?
                / 10_000;
            if amount.minor > ceiling {
                return Err(HoldError::OverSettled {
                    id: self.id.clone(),
                    reserved: self.amount.clone(),
                    settled: amount.clone(),
                    tolerance_bps: self.tolerance_bps,
                });
            }
        }
        self.resolution = Some((outcome, at));
        Ok(())
    }

    /// The posting set a settlement produces: the reserved account is debited and the
    /// destination credited, for the **settled** amount rather than the reserved one.
    ///
    /// Returns `None` for a void or an expiry, which is the honest answer — those move no
    /// value and therefore have no posting set. An implementation that returned an empty
    /// set here would be refused by the kernel anyway ([`KernelError::Empty`]), which is the
    /// two layers agreeing.
    pub fn settlement_postings(
        &self,
        to: impl Into<String>,
        first_entry_id: u64,
    ) -> Result<Option<PostingSet>, HoldError> {
        let (outcome, at) = match &self.resolution {
            Some(r) => r,
            None => return Ok(None),
        };
        let amount = match outcome {
            Outcome::Posted { amount } => amount,
            _ => return Ok(None),
        };
        let debit = Entry::new(
            first_entry_id,
            self.account.as_str(),
            amount.negate().map_err(HoldError::Arithmetic)?,
            *at,
        )
        .narrated(format!("settlement of hold {}", self.id));
        let credit = debit
            .mirrored_to(first_entry_id + 1, to)
            .map_err(HoldError::Arithmetic)?;
        Ok(Some(PostingSet::new(format!("settle-{}", self.id)).with(debit).with(credit)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gbs_kernel::{Account, Chart};

    fn st(e: u64, d: i64) -> Stamp {
        Stamp::new(Epoch(e), d)
    }

    fn hold() -> Hold {
        Hold::place("auth-1", "cust.usd", Amount::minor_2dp(5_000, "USD"), st(10, 0), 7)
    }

    fn chart() -> Chart {
        Chart::new()
            .with(Account::new("cust.usd", "cust-1", "USD"))
            .with(Account::new("merchant.usd", "merch-9", "USD"))
    }

    // ── linearity ───────────────────────────────────────────────────────────────────

    #[test]
    fn a_hold_resolves_exactly_once() {
        let mut h = hold();
        assert!(h.is_open());
        h.resolve(Outcome::Posted { amount: Amount::minor_2dp(5_000, "USD") }, st(11, 0)).unwrap();
        assert!(!h.is_open());

        let e = h.resolve(Outcome::Void, st(12, 0)).unwrap_err();
        assert!(matches!(e, HoldError::AlreadyResolved { .. }));
        // Both resolutions are named: "posted then void" is a different bug from "posted
        // twice", and the message has to distinguish them.
        let msg = e.to_string();
        assert!(msg.contains("posted"), "{msg}");
        assert!(msg.contains("void"), "{msg}");
    }

    #[test]
    fn every_pair_of_resolutions_is_refused_including_two_of_the_same_kind() {
        let outcomes = || {
            vec![
                Outcome::Posted { amount: Amount::minor_2dp(1, "USD") },
                Outcome::Void,
                Outcome::Expired,
            ]
        };
        for first in outcomes() {
            for second in outcomes() {
                let mut h = hold();
                h.resolve(first.clone(), st(11, 0)).unwrap();
                assert!(
                    h.resolve(second.clone(), st(12, 0)).is_err(),
                    "{} then {} must be refused",
                    first.name(),
                    second.name()
                );
            }
        }
    }

    #[test]
    fn an_expiry_is_a_resolution_and_not_an_absence_of_one() {
        // The decision that makes the state machine total. If expiry were "unresolved, past
        // the date", every consumer would re-derive liveness from a clock and two consumers
        // would eventually disagree.
        let mut h = hold();
        h.resolve(Outcome::Expired, st(20, 8)).unwrap();
        assert!(!h.is_open());
        assert_eq!(h.outcome().map(|o| o.name()), Some("expired"));
        assert!(!h.outcome().unwrap().moved_value());
    }

    // ── the APSN property ───────────────────────────────────────────────────────────

    #[test]
    fn available_and_ledger_balance_cannot_disagree_about_which_holds_exist() {
        // The thesis's opening problem, as a test. `encumbers` is the *only* difference
        // between the two views, and it is a pure function of the anchor and the as-of date
        // — so two readers at one anchor get one answer.
        let h = hold();
        assert!(h.encumbers(Epoch(10), 0), "live at the epoch that placed it");
        assert!(!h.encumbers(Epoch(9), 0), "not before it was placed");
        assert!(h.encumbers(Epoch(100), 7), "live on its expiry day");
        assert!(!h.encumbers(Epoch(100), 8), "and not after");
    }

    #[test]
    fn a_hold_resolved_after_the_anchor_was_still_open_at_the_anchor() {
        // The case a mutable status column gets wrong. Reading "as of epoch 10" must see
        // the hold as live, even though by epoch 12 it has settled — because at epoch 10 it
        // had. A status column has already been overwritten and cannot answer.
        let mut h = hold();
        h.resolve(Outcome::Posted { amount: Amount::minor_2dp(5_000, "USD") }, st(12, 1)).unwrap();
        assert!(h.encumbers(Epoch(10), 0), "as of epoch 10 the hold was open");
        assert!(h.encumbers(Epoch(11), 0));
        assert!(!h.encumbers(Epoch(12), 1), "and at 12 it is settled");
    }

    #[test]
    fn liveness_never_consults_a_clock() {
        // Determinism: the same question at the same anchor gives the same answer whenever
        // it is asked. Enforced by `encumbers` taking both coordinates as parameters.
        let h = hold();
        let first = h.encumbers(Epoch(50), 3);
        for _ in 0..100 {
            assert_eq!(h.encumbers(Epoch(50), 3), first);
        }
    }

    // ── settlement ──────────────────────────────────────────────────────────────────

    #[test]
    fn settlement_may_differ_from_authorisation_because_it_usually_does() {
        // A fuel pump pre-authorises $100 and settles $43.17. A design assuming equality
        // would be wrong on the ordinary case.
        let mut h = hold();
        h.resolve(Outcome::Posted { amount: Amount::minor_2dp(4_317, "USD") }, st(11, 1)).unwrap();
        let ps = h.settlement_postings("merchant.usd", 1).unwrap().unwrap();
        let sealed = ps.seal(&chart(), Epoch(11)).expect("a settlement conserves");
        assert_eq!(sealed.entries().len(), 2);
        assert_eq!(sealed.entries()[0].amount.minor, -4_317, "the settled amount, not the held one");
        assert_eq!(sealed.entries()[1].amount.minor, 4_317);
    }

    #[test]
    fn over_settlement_beyond_tolerance_is_refused_and_within_it_is_allowed() {
        // The tip allowance, as a declared number rather than a constant buried in a card
        // product.
        let mut strict = hold();
        assert!(matches!(
            strict.resolve(Outcome::Posted { amount: Amount::minor_2dp(5_001, "USD") }, st(11, 0)),
            Err(HoldError::OverSettled { .. })
        ));

        let mut lenient = hold().with_tolerance_bps(2_000); // 20%
        assert!(lenient
            .resolve(Outcome::Posted { amount: Amount::minor_2dp(6_000, "USD") }, st(11, 0))
            .is_ok());

        let mut also_lenient = hold().with_tolerance_bps(2_000);
        assert!(matches!(
            also_lenient.resolve(Outcome::Posted { amount: Amount::minor_2dp(6_001, "USD") }, st(11, 0)),
            Err(HoldError::OverSettled { .. })
        ));
    }

    #[test]
    fn settling_in_another_currency_is_refused_as_an_fx_transaction() {
        let mut h = hold();
        let e = h
            .resolve(Outcome::Posted { amount: Amount::minor_2dp(4_600, "EUR") }, st(11, 0))
            .unwrap_err();
        assert!(matches!(e, HoldError::CurrencyChanged { .. }));
        assert!(e.to_string().contains("FX transaction, not a resolution"));
    }

    #[test]
    fn a_negative_settlement_is_refused() {
        let mut h = hold();
        assert!(matches!(
            h.resolve(Outcome::Posted { amount: Amount::minor_2dp(-1, "USD") }, st(11, 0)),
            Err(HoldError::NegativeSettlement { .. })
        ));
    }

    #[test]
    fn a_void_and_an_expiry_produce_no_postings_at_all() {
        // Honest absence rather than an empty posting set. An empty set would be refused by
        // the kernel as `Empty`, so returning one would be two layers disagreeing about
        // whether a void is a transaction.
        for outcome in [Outcome::Void, Outcome::Expired] {
            let mut h = hold();
            h.resolve(outcome, st(11, 0)).unwrap();
            assert_eq!(h.settlement_postings("merchant.usd", 1).unwrap(), None);
        }
    }

    #[test]
    fn an_unresolved_hold_produces_no_postings_either() {
        assert_eq!(hold().settlement_postings("merchant.usd", 1).unwrap(), None);
    }

    #[test]
    fn a_zero_settlement_is_legal_and_still_conserves() {
        // A pre-authorisation settled at zero — an abandoned transaction that the scheme
        // requires be settled rather than voided. The posting set is two zero legs, which
        // the kernel accepts, because a leg present for audit symmetry is a real leg.
        let mut h = hold();
        h.resolve(Outcome::Posted { amount: Amount::minor_2dp(0, "USD") }, st(11, 0)).unwrap();
        let ps = h.settlement_postings("merchant.usd", 1).unwrap().unwrap();
        assert!(ps.seal(&chart(), Epoch(11)).is_ok());
    }

    #[test]
    fn an_expiry_date_is_mandatory_and_there_is_no_way_to_omit_it() {
        // Documented as a test because the field's *type* is what enforces it: `i64`, not
        // `Option<i64>`. A hold with no expiry reduces an available balance forever.
        let h = hold();
        assert_eq!(h.expires_on, 7);
        let d = format!("{h:?}");
        assert!(!d.contains("expires_on: None"));
    }
}
