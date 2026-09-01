//! **M6 — position signals.** A position as a function of time.
//!
//! A balance, an exposure, a holding, a limit utilisation — each is a fold over entries up
//! to an anchor on the system axis and an as-of date on the world axis. This module is the
//! shape of that fold; the *runtime* that maintains it incrementally and partially is
//! `nilestream-core`, and the whole point of the layering is that a product never talks to
//! it directly.
//!
//! # Honest absence, and why the return type is not a number
//!
//! [`Signal::at`] returns [`Reading`], which distinguishes **a position of zero** from **no
//! position** from **a position that could not be computed**. That distinction is the
//! absence lattice of thesis §3.3 arriving in the product layer, and it is not a nicety.
//!
//! The GBS/Noria postmortem (thesis §1.1.1) records the exact line a working developer wrote
//! when the engine gave them no way to say "I don't know":
//!
//! ```text
//! Err(_) => 0, // Account not found or zero balance
//! ```
//!
//! A database error rendered as a zero balance. Downstream, that is an account with no
//! money in it — and every limit check passes, every sweep fires, and every overdraft
//! report is clean. This module makes that line unwritable: `Reading` has no arm that
//! collapses to zero, and a caller who wants a number has to say what an absent one means.
//!
//! # Signals are read at an anchor, never "now"
//!
//! Every accessor takes the anchor and the as-of date. There is no `current()`, because
//! "current" means "whenever this happened to run", and two consumers computing a limit
//! against two different instants is how an authorisation is approved against a balance
//! that a settlement had already consumed. This is the mechanical form of the
//! coordination-free read of thesis §7.5: a frozen prefix cannot change, so two readers at
//! one anchor get one answer without agreeing on anything.

use gbs_kernel::{AccountId, Amount, Currency, CurrencySums, Entry, KernelError, Party, Epoch};
use std::collections::BTreeMap;
use std::fmt;

/// What a signal reads at a point.
///
/// Three states, and the middle one is why this is an enum rather than an `Option<Amount>`:
/// "no entries have ever touched this key" and "the entries exist but the fold could not be
/// completed" are different facts, and only one of them is safe to treat as zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    /// A computed position. May legitimately be zero.
    Present(Amount),
    /// Nothing has ever touched this key at this anchor. **Not zero.**
    ///
    /// For a balance this is usually equivalent in effect — an account with no entries has
    /// no money — but the distinction survives into places where it is not: an *exposure*
    /// with no entries is unknown rather than nil, and a limit check against an unknown
    /// exposure must refuse rather than approve.
    Absent,
    /// The fold could not be completed. Carries the reason.
    ///
    /// Never silently zero. A caller must handle it or propagate it.
    Unavailable(String),
}

impl Reading {
    /// The amount, if present. `None` for both absent and unavailable — which is a
    /// deliberate flattening for the callers that genuinely do not care, and it is *not*
    /// the same as returning zero.
    pub fn amount(&self) -> Option<&Amount> {
        match self {
            Reading::Present(a) => Some(a),
            _ => None,
        }
    }

    /// Treat an absent position as zero, **explicitly**, in a named currency.
    ///
    /// The escape hatch, and it is named to be conspicuous at the call site. For an account
    /// balance this is correct and ordinary; for an exposure or a limit utilisation it is
    /// the defect of §1.1.1. Note what it does *not* do: an `Unavailable` reading stays
    /// unavailable, because "the computation failed" is never zero under any reading.
    pub fn or_zero_because_absent_means_nil(&self, c: &Currency, scale: u32) -> Option<Amount> {
        match self {
            Reading::Present(a) => Some(a.clone()),
            Reading::Absent => Some(Amount::new(0, c.clone(), scale)),
            Reading::Unavailable(_) => None,
        }
    }

    pub fn is_present(&self) -> bool {
        matches!(self, Reading::Present(_))
    }

    pub fn is_absent(&self) -> bool {
        matches!(self, Reading::Absent)
    }
}

impl fmt::Display for Reading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reading::Present(a) => write!(f, "{a}"),
            // Not "0.00", and not blank. A reader must be able to tell.
            Reading::Absent => f.write_str("(absent)"),
            Reading::Unavailable(why) => write!(f, "(unavailable: {why})"),
        }
    }
}

/// What a signal folds over, and how.
///
/// Selecting entries by predicate rather than by a stored classification is what keeps a
/// new signal from needing a schema change: a desk-level exposure, a legal-entity roll-up
/// and a single account balance are the same fold with different predicates.
pub struct Signal {
    pub name: String,
    /// Which entries count. Takes the entry and the resolved owning party, because most
    /// interesting scopes are party-based rather than account-based.
    selector: Box<dyn Fn(&Entry, Option<&Party>) -> bool>,
}

impl fmt::Debug for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signal").field("name", &self.name).finish_non_exhaustive()
    }
}

impl Signal {
    pub fn new(
        name: impl Into<String>,
        selector: Box<dyn Fn(&Entry, Option<&Party>) -> bool>,
    ) -> Self {
        Signal { name: name.into(), selector }
    }

    /// The balance of one account.
    pub fn balance_of(account: &str) -> Self {
        let id = AccountId::new(account);
        Signal::new(format!("balance:{account}"), Box::new(move |e, _| e.account == id))
    }

    /// Everything one party holds, across all their accounts in one currency.
    pub fn party_position(party: &str) -> Self {
        let p = Party::new(party);
        Signal::new(
            format!("position:{party}"),
            Box::new(move |_, owner| owner == Some(&p)),
        )
    }

    /// Read at an anchor and an as-of date, in one currency.
    ///
    /// Both coordinates are required. There is no `current()` — see the module docs.
    pub fn at(
        &self,
        entries: &[(Entry, Option<Party>)],
        currency: &Currency,
        anchor: Epoch,
        as_of: i64,
    ) -> Reading {
        let mut sums = CurrencySums::new();
        let mut matched = 0usize;

        for (e, owner) in entries {
            if !e.stamp.visible_at(anchor) || !e.stamp.effective_on(as_of) {
                continue;
            }
            if e.amount.currency != *currency {
                continue;
            }
            if !(self.selector)(e, owner.as_ref()) {
                continue;
            }
            if let Err(err) = sums.add(&e.amount) {
                // A scale conflict or an overflow. Reported, never swallowed: a fold that
                // silently dropped a conflicting entry would return a plausible number that
                // is missing money.
                return Reading::Unavailable(err.to_string());
            }
            matched += 1;
        }

        if matched == 0 {
            return Reading::Absent;
        }
        match sums.get(currency) {
            Some(a) => Reading::Present(a),
            None => Reading::Absent,
        }
    }

    /// Read across every currency present, as a map. The FX position.
    ///
    /// Returns a `Reading` per currency, so a currency with no movement is absent rather
    /// than zero — which matters for a position report, where a zero row and a missing row
    /// mean different things to a trader.
    pub fn by_currency(
        &self,
        entries: &[(Entry, Option<Party>)],
        anchor: Epoch,
        as_of: i64,
    ) -> BTreeMap<Currency, Reading> {
        let mut currencies: Vec<Currency> = entries
            .iter()
            .filter(|(e, o)| {
                e.stamp.visible_at(anchor)
                    && e.stamp.effective_on(as_of)
                    && (self.selector)(e, o.as_ref())
            })
            .map(|(e, _)| e.amount.currency.clone())
            .collect();
        currencies.sort();
        currencies.dedup();

        currencies
            .into_iter()
            .map(|c| {
                let r = self.at(entries, &c, anchor, as_of);
                (c, r)
            })
            .collect()
    }
}

/// An available balance: the ledger balance, less unresolved encumbrances.
///
/// The two numbers of thesis §1.1 — the pair whose disagreement the FDIC and CFPB named as
/// a consumer-harm pattern. They are computed here from **one anchor**, by one function, so
/// there is no configuration under which they can be sourced from different instants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Available {
    pub ledger: Reading,
    /// The total of holds live at this anchor.
    pub encumbered: Amount,
    pub available: Reading,
}

/// Compute both balances at one anchor.
///
/// `holds` is the total already netted by the caller from live holds — see
/// [`Hold::encumbers`](crate::hold::Hold::encumbers), which is the predicate that decides
/// liveness from the same anchor.
///
/// The signature is the design: one anchor in, both numbers out. There is no way to call
/// this such that the ledger balance and the available balance are computed at different
/// points, because there is only one point.
pub fn available_balance(
    ledger: Reading,
    holds_total: Amount,
) -> Result<Available, KernelError> {
    let available = match &ledger {
        Reading::Present(l) => {
            if l.currency != holds_total.currency {
                return Err(KernelError::CurrencyMismatch {
                    left: l.currency.clone(),
                    right: holds_total.currency.clone(),
                });
            }
            Reading::Present(l.add(&holds_total.negate()?)?)
        }
        // An absent ledger balance with live holds is genuinely strange — encumbrances
        // against an account with no entries — so it is reported rather than resolved.
        Reading::Absent if holds_total.minor != 0 => Reading::Unavailable(
            "holds exist against an account with no visible entries".into(),
        ),
        Reading::Absent => Reading::Absent,
        Reading::Unavailable(w) => Reading::Unavailable(w.clone()),
    };
    Ok(Available { ledger, encumbered: holds_total, available })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gbs_kernel::Stamp;

    fn e(id: u64, account: &str, minor: i128, cur: &str, epoch: u64, day: i64) -> (Entry, Option<Party>) {
        (
            Entry::new(id, account, Amount::minor_2dp(minor, cur), Stamp::new(Epoch(epoch), day)),
            Some(Party::new("cust-1")),
        )
    }

    fn history() -> Vec<(Entry, Option<Party>)> {
        vec![
            e(1, "cust.usd", 100_000, "USD", 10, 0),
            e(2, "cust.usd", -25_000, "USD", 20, 1),
            e(3, "cust.usd", -10_000, "USD", 30, 2),
            e(4, "cust.eur", 50_000, "EUR", 25, 1),
        ]
    }

    fn usd() -> Currency {
        Currency::new("USD")
    }

    // ── honest absence ──────────────────────────────────────────────────────────────

    #[test]
    fn an_account_with_no_entries_is_absent_and_not_zero() {
        // The defect of §1.1.1, prevented at the type level. There is no arm of `Reading`
        // that is a bare number, so `Err(_) => 0` has nowhere to go.
        let r = Signal::balance_of("nobody.usd").at(&history(), &usd(), Epoch(100), 10);
        assert_eq!(r, Reading::Absent);
        assert!(!r.is_present());
        assert_eq!(r.amount(), None, "absent yields no amount");
        assert_eq!(r.to_string(), "(absent)", "and renders distinguishably");
    }

    #[test]
    fn a_genuine_zero_balance_is_present_and_distinguishable_from_absent() {
        // The pair that matters: an account that received and returned the same amount has
        // a *zero* balance, which is a fact. An account nobody has touched has no balance.
        let entries = vec![
            e(1, "z.usd", 5_000, "USD", 10, 0),
            e(2, "z.usd", -5_000, "USD", 11, 0),
        ];
        let r = Signal::balance_of("z.usd").at(&entries, &usd(), Epoch(100), 10);
        assert_eq!(r, Reading::Present(Amount::minor_2dp(0, "USD")));
        assert!(r.is_present());
        assert_ne!(r, Reading::Absent, "zero is not absent");
    }

    #[test]
    fn treating_absent_as_zero_requires_saying_so_at_the_call_site() {
        // The escape hatch is named to be conspicuous. Correct for a balance, wrong for an
        // exposure — and the name forces the reader to decide which they have.
        let absent = Reading::Absent;
        assert_eq!(
            absent.or_zero_because_absent_means_nil(&usd(), 2),
            Some(Amount::minor_2dp(0, "USD"))
        );
    }

    #[test]
    fn an_unavailable_reading_never_becomes_zero_even_through_the_escape_hatch() {
        // "The computation failed" is not zero under any reading, so the hatch refuses it.
        let u = Reading::Unavailable("scale conflict".into());
        assert_eq!(u.or_zero_because_absent_means_nil(&usd(), 2), None);
        assert!(u.to_string().contains("unavailable"));
    }

    #[test]
    fn a_fold_that_cannot_complete_reports_rather_than_dropping_the_conflicting_entry() {
        // A scale conflict within one currency. Skipping the bad entry would return a
        // plausible number that is missing money — the worst available outcome.
        let entries = vec![
            (
                Entry::new(1, "x.usd", Amount::new(1_000, usd(), 2), Stamp::new(Epoch(1), 0)),
                None,
            ),
            (
                Entry::new(2, "x.usd", Amount::new(1_000, usd(), 4), Stamp::new(Epoch(2), 0)),
                None,
            ),
        ];
        let r = Signal::balance_of("x.usd").at(&entries, &usd(), Epoch(10), 10);
        assert!(matches!(r, Reading::Unavailable(_)), "{r}");
    }

    // ── reading at an anchor ────────────────────────────────────────────────────────

    #[test]
    fn a_position_is_a_function_of_the_anchor() {
        let s = Signal::balance_of("cust.usd");
        let h = history();
        assert_eq!(s.at(&h, &usd(), Epoch(10), 100).amount().unwrap().minor, 100_000);
        assert_eq!(s.at(&h, &usd(), Epoch(20), 100).amount().unwrap().minor, 75_000);
        assert_eq!(s.at(&h, &usd(), Epoch(30), 100).amount().unwrap().minor, 65_000);
        assert_eq!(s.at(&h, &usd(), Epoch(9), 100), Reading::Absent, "before anything");
    }

    #[test]
    fn a_position_is_also_a_function_of_the_world_axis() {
        // Two axes, independently. Entry 3 is effective on day 2, so as of day 1 it has not
        // happened yet even though it is visible at epoch 30.
        let s = Signal::balance_of("cust.usd");
        let h = history();
        assert_eq!(s.at(&h, &usd(), Epoch(30), 0).amount().unwrap().minor, 100_000);
        assert_eq!(s.at(&h, &usd(), Epoch(30), 1).amount().unwrap().minor, 75_000);
        assert_eq!(s.at(&h, &usd(), Epoch(30), 2).amount().unwrap().minor, 65_000);
    }

    #[test]
    fn the_past_is_stable_however_much_arrives_later() {
        // The coordination-free read, as a property: a frozen prefix cannot change, so the
        // answer at an anchor is fixed forever and needs no invalidation.
        let s = Signal::balance_of("cust.usd");
        let mut h = history();
        let at_20 = s.at(&h, &usd(), Epoch(20), 100);
        for i in 0..20 {
            h.push(e(100 + i, "cust.usd", 7_777, "USD", 50 + i, 10));
            assert_eq!(s.at(&h, &usd(), Epoch(20), 100), at_20);
        }
    }

    #[test]
    fn there_is_no_current_and_the_anchor_must_be_supplied() {
        // Documented as a test because it is an API-shape decision: two consumers computing
        // a limit against two different instants is how an authorisation is approved
        // against a balance a settlement had already consumed.
        let s = Signal::balance_of("cust.usd");
        let h = history();
        let a = s.at(&h, &usd(), Epoch(25), 5);
        for _ in 0..50 {
            assert_eq!(s.at(&h, &usd(), Epoch(25), 5), a);
        }
    }

    // ── scope ───────────────────────────────────────────────────────────────────────

    #[test]
    fn a_party_position_spans_accounts_and_a_balance_does_not() {
        // The same fold with a different predicate — which is what keeps a new signal from
        // needing a schema change.
        let h = history();
        let by_account = Signal::balance_of("cust.usd").at(&h, &usd(), Epoch(100), 100);
        let by_party = Signal::party_position("cust-1").at(&h, &usd(), Epoch(100), 100);
        assert_eq!(by_account.amount().unwrap().minor, 65_000);
        assert_eq!(by_party.amount().unwrap().minor, 65_000, "USD only, across the party");

        let eur = Signal::party_position("cust-1").at(&h, &Currency::new("EUR"), Epoch(100), 100);
        assert_eq!(eur.amount().unwrap().minor, 50_000);
    }

    #[test]
    fn the_fx_position_reports_every_currency_in_stable_order() {
        let by = Signal::party_position("cust-1").by_currency(&history(), Epoch(100), 100);
        let codes: Vec<String> = by.keys().map(|c| c.to_string()).collect();
        assert_eq!(codes, vec!["EUR", "USD"]);
        assert_eq!(by[&Currency::new("USD")].amount().unwrap().minor, 65_000);
        assert_eq!(by[&Currency::new("EUR")].amount().unwrap().minor, 50_000);
    }

    #[test]
    fn a_currency_with_no_movement_does_not_appear_rather_than_appearing_as_zero() {
        // For a trader's position report, a zero row and a missing row mean different
        // things, and the difference is whether there is a position that has netted out.
        let by = Signal::party_position("cust-1").by_currency(&history(), Epoch(100), 100);
        assert!(!by.contains_key(&Currency::new("JPY")));
    }

    // ── available balance: the APSN pair ────────────────────────────────────────────

    #[test]
    fn both_balances_come_from_one_anchor_by_one_call() {
        // The thesis's opening problem, closed by an API shape: there is one anchor and one
        // function, so there is no configuration in which the two numbers are sourced from
        // different instants.
        let ledger = Reading::Present(Amount::minor_2dp(65_000, "USD"));
        let a = available_balance(ledger, Amount::minor_2dp(20_000, "USD")).unwrap();
        assert_eq!(a.ledger.amount().unwrap().minor, 65_000);
        assert_eq!(a.encumbered.minor, 20_000);
        assert_eq!(a.available.amount().unwrap().minor, 45_000);
    }

    #[test]
    fn an_available_balance_may_go_negative_and_is_reported_as_such() {
        // Holds exceeding the balance is a real state — it is what an authorisation
        // decision needs to see. Clamping it to zero would make every overdraft look fine.
        let a = available_balance(
            Reading::Present(Amount::minor_2dp(1_000, "USD")),
            Amount::minor_2dp(5_000, "USD"),
        )
        .unwrap();
        assert_eq!(a.available.amount().unwrap().minor, -4_000);
    }

    #[test]
    fn an_unavailable_ledger_balance_propagates_and_does_not_become_a_number() {
        let a = available_balance(
            Reading::Unavailable("upstream fold failed".into()),
            Amount::minor_2dp(0, "USD"),
        )
        .unwrap();
        assert!(matches!(a.available, Reading::Unavailable(_)));
    }

    #[test]
    fn holds_against_an_account_with_no_entries_are_reported_rather_than_resolved() {
        // Genuinely strange, so it is surfaced. Silently treating the ledger as zero would
        // produce a negative available balance on an account that does not exist.
        let a = available_balance(Reading::Absent, Amount::minor_2dp(5_000, "USD")).unwrap();
        assert!(matches!(a.available, Reading::Unavailable(_)), "{}", a.available);
    }

    #[test]
    fn an_untouched_account_with_no_holds_is_absent_on_both_numbers() {
        let a = available_balance(Reading::Absent, Amount::minor_2dp(0, "USD")).unwrap();
        assert_eq!(a.available, Reading::Absent);
        assert_eq!(a.ledger, Reading::Absent);
    }

    #[test]
    fn a_currency_mismatch_between_balance_and_holds_is_refused() {
        let e = available_balance(
            Reading::Present(Amount::minor_2dp(100, "USD")),
            Amount::minor_2dp(100, "EUR"),
        )
        .unwrap_err();
        assert!(matches!(e, KernelError::CurrencyMismatch { .. }));
    }
}
