//! Trade finance — letters of credit, trade loans, and supply-chain finance.
//!
//! The lifecycle products. A letter of credit is M5 (a capability-gated state machine) over
//! M3 (the issuance is a reservation against the applicant's limit), with M2 for the expiry
//! and M4 where a receivable is assigned.
//!
//! # Why an LC is the best test of M5
//!
//! A letter of credit has the property that makes lifecycle modelling hard: **the parties
//! are adversarial, and the question asked years later is what the state was at a moment,
//! not what it is now.** Did the presentation arrive before expiry? Was the amendment
//! consented to before the goods shipped? A `status` column answers neither, because it has
//! been overwritten; [`Lifecycle::state_at`](gbs_mechanisms::Lifecycle::state_at) answers
//! both from an immutable prefix, and gives the same answer to both parties' lawyers.
//!
//! # Supply-chain finance: conservation across assignor and assignee
//!
//! When a supplier assigns a receivable to a financier, value moves between three parties —
//! and the obligation that must hold is not merely that the posting set balances, but that
//! **the assignor's reduction equals the assignee's increase plus the discount**. That is
//! the "conservation across assignor/assignee" the scope names, and it is checked here
//! separately from the kernel's per-currency sum, for the same reason lending checks
//! participant shares separately: a set can balance overall while the individual legs are
//! wrong.

use gbs_kernel::{Amount, Chart, Entry, Epoch, KernelError, Party, PostingSet, Sealed, Stamp};
use gbs_mechanisms::{Actor, Hold, Lifecycle, LifecycleError, Rule, State};

/// What a trade-finance product refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeError {
    Lifecycle(LifecycleError),
    Kernel(KernelError),
    /// A presentation for more than the credit's undrawn value.
    ExceedsCredit { presented: Amount, available: Amount },
    /// The assignment legs do not reconcile across the three parties.
    AssignmentImbalance { assignor: Amount, assignee: Amount, discount: Amount },
    /// A presentation after expiry. A distinct error because it is the one that ends up in
    /// court, and "the LC had expired on day 214" is the sentence that settles it.
    Expired { expired_on: i64, presented_on: i64 },
}

impl std::fmt::Display for TradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeError::Lifecycle(e) => write!(f, "{e}"),
            TradeError::Kernel(e) => write!(f, "{e}"),
            TradeError::ExceedsCredit { presented, available } => {
                write!(f, "presentation of {presented} exceeds the available credit of {available}")
            }
            TradeError::AssignmentImbalance { assignor, assignee, discount } => write!(
                f,
                "assignment does not reconcile: the assignor gives up {assignor}, the \
                 assignee receives {assignee}, and the discount is {discount}. The set may \
                 still balance overall, which is why this is checked separately"
            ),
            TradeError::Expired { expired_on, presented_on } => write!(
                f,
                "the credit expired on day {expired_on}; presentation was on day {presented_on}"
            ),
        }
    }
}

/// The letter-of-credit rule set, as UCP 600 actually shapes it.
///
/// A function rather than a constant, because a deployment may add states — a *documents
/// discrepant, awaiting waiver* step, say — and the whole point of M5's open vocabulary is
/// that doing so needs no change below this file.
pub fn lc_rules() -> Vec<Rule> {
    vec![
        Rule::new("issue", "draft", "issued", "lc.issue"),
        Rule::new("advise", "issued", "advised", "lc.advise"),
        Rule::new("confirm", "advised", "confirmed", "lc.confirm"),
        // Amendable from any live state, which is why there are three rules rather than one:
        // a transition is `(name, from)`, so an amendment from each live state is its own
        // rule. Listing them is better than a wildcard, because the set of live states is
        // then visible rather than implied.
        Rule::new("amend", "issued", "issued", "lc.amend"),
        Rule::new("amend", "advised", "advised", "lc.amend"),
        Rule::new("amend", "confirmed", "confirmed", "lc.amend"),
        Rule::new("present", "issued", "presented", "lc.present"),
        Rule::new("present", "advised", "presented", "lc.present"),
        Rule::new("present", "confirmed", "presented", "lc.present"),
        // **Partial drawings.** UCP 600 permits a credit to be drawn in instalments unless
        // it says otherwise, so `present` must be a self-loop — a second presentation
        // against a partly-drawn credit is ordinary, not an error.
        //
        // The first version of this rule set omitted it, and the partial-presentation test
        // found it immediately: the second drawing was refused as "not a legal transition
        // from `presented`". That is the layering working in the direction it is supposed
        // to — a product exercising a mechanism against real domain rules and finding what
        // the mechanism's own tests could not know to look for.
        Rule::new("present", "presented", "presented", "lc.present"),
        Rule::new("honour", "presented", "honoured", "lc.honour").terminal(),
        Rule::new("dishonour", "presented", "dishonoured", "lc.honour").terminal(),
        Rule::new("expire", "issued", "expired", "lc.expire").terminal(),
        Rule::new("expire", "advised", "expired", "lc.expire").terminal(),
        Rule::new("expire", "confirmed", "expired", "lc.expire").terminal(),
    ]
}

/// A documentary credit.
pub struct LetterOfCredit {
    pub lifecycle: Lifecycle,
    pub applicant: Party,
    pub beneficiary: Party,
    pub amount: Amount,
    /// The valid-time day the credit expires. Mandatory: a credit with no expiry is a
    /// contingent liability that never leaves the balance sheet.
    pub expires_on: i64,
    drawn: Amount,
}

impl LetterOfCredit {
    pub fn new(
        id: &str,
        applicant: &str,
        beneficiary: &str,
        amount: Amount,
        expires_on: i64,
    ) -> Self {
        let zero = Amount::new(0, amount.currency.clone(), amount.scale);
        LetterOfCredit {
            lifecycle: Lifecycle::new(id, "draft", lc_rules()),
            applicant: Party::new(applicant),
            beneficiary: Party::new(beneficiary),
            amount,
            expires_on,
            drawn: zero,
        }
    }

    pub fn state(&self) -> State {
        self.lifecycle.state()
    }

    /// **What the state was at a past epoch.** The question a dispute asks.
    pub fn state_at(&self, anchor: Epoch) -> State {
        self.lifecycle.state_at(anchor)
    }

    pub fn undrawn(&self) -> Result<Amount, KernelError> {
        self.amount.add(&self.drawn.negate()?)
    }

    /// Issue the credit, creating the contingent liability.
    ///
    /// The posting is to memorandum accounts — real double entry, on accounts that net to
    /// zero on the balance sheet. A contingent liability that is *not* posted is one that
    /// does not appear in a regulatory exposure calculation, which is how an institution
    /// discovers its true exposure during a crisis rather than before one.
    pub fn issue(
        &mut self,
        by: &Actor,
        stamp: Stamp,
        chart: &Chart,
        epoch: Epoch,
    ) -> Result<Sealed, TradeError> {
        let contra = Entry::new(1, "memo.lc.contra", self.amount.negate().map_err(TradeError::Kernel)?, stamp)
            .narrated(format!("contingent liability under {}", self.lifecycle.id));
        let liability = contra.mirrored_to(2, "memo.lc.liability").map_err(TradeError::Kernel)?;
        let ps = PostingSet::new(format!("issue-{}", self.lifecycle.id)).with(contra).with(liability);

        self.lifecycle
            .transition("issue", by, stamp, Some(ps.clone()))
            .map_err(TradeError::Lifecycle)?;
        ps.seal(chart, epoch).map_err(TradeError::Kernel)
    }

    /// The applicant's limit reservation, as an M3 hold.
    ///
    /// An issued credit encumbers the applicant's facility for its full value until it is
    /// honoured or expires. Modelling it as a hold rather than a number means the same
    /// limit cannot be committed twice.
    pub fn applicant_hold(&self, account: &str, placed: Stamp) -> Hold {
        Hold::place(
            format!("lc-limit-{}", self.lifecycle.id),
            account,
            self.amount.clone(),
            placed,
            self.expires_on,
        )
    }

    /// A complying presentation, drawn against the credit.
    ///
    /// Checks expiry on the **valid-time** axis, against the presentation's own date rather
    /// than a clock — so the answer is the same whenever the question is asked, which is
    /// the property a dispute needs.
    pub fn present(
        &mut self,
        by: &Actor,
        amount: &Amount,
        stamp: Stamp,
        chart: &Chart,
        epoch: Epoch,
    ) -> Result<Sealed, TradeError> {
        if stamp.value_date > self.expires_on {
            return Err(TradeError::Expired {
                expired_on: self.expires_on,
                presented_on: stamp.value_date,
            });
        }
        let available = self.undrawn().map_err(TradeError::Kernel)?;
        if amount.minor > available.minor {
            return Err(TradeError::ExceedsCredit { presented: amount.clone(), available });
        }

        let to_beneficiary = Entry::new(1, "beneficiary.usd", amount.clone(), stamp)
            .narrated(format!("presentation under {}", self.lifecycle.id));
        let from_applicant = to_beneficiary.mirrored_to(2, "applicant.usd").map_err(TradeError::Kernel)?;
        let ps = PostingSet::new(format!("present-{}-{}", self.lifecycle.id, epoch.0))
            .with(to_beneficiary)
            .with(from_applicant);

        self.lifecycle
            .transition("present", by, stamp, Some(ps.clone()))
            .map_err(TradeError::Lifecycle)?;
        let sealed = ps.seal(chart, epoch).map_err(TradeError::Kernel)?;
        self.drawn = self.drawn.add(amount).map_err(TradeError::Kernel)?;
        Ok(sealed)
    }
}

/// Assign a receivable from a supplier to a financier at a discount.
///
/// Supply-chain finance, and the conservation obligation the scope names: the assignor's
/// reduction must equal the assignee's payment plus the discount. Checked **separately**
/// from the kernel's per-currency sum, because a set can balance overall while the three
/// parties' individual legs are wrong.
pub fn assign_receivable(
    id: &str,
    face_value: &Amount,
    discount: &Amount,
    assignor_account: &str,
    assignee_account: &str,
    discount_account: &str,
    stamp: Stamp,
    chart: &Chart,
    epoch: Epoch,
) -> Result<Sealed, TradeError> {
    let proceeds = face_value.add(&discount.negate().map_err(TradeError::Kernel)?)
        .map_err(TradeError::Kernel)?;

    // The three-party check. The assignor gives up the face value; the assignee pays the
    // proceeds and books the discount as income. Face = proceeds + discount, exactly.
    let reconstructed = proceeds.add(discount).map_err(TradeError::Kernel)?;
    if reconstructed.minor != face_value.minor {
        return Err(TradeError::AssignmentImbalance {
            assignor: face_value.clone(),
            assignee: proceeds,
            discount: discount.clone(),
        });
    }

    let ps = PostingSet::new(format!("assign-{id}"))
        // The supplier receives the discounted proceeds...
        .with(Entry::new(1, assignor_account, proceeds.clone(), stamp)
            .narrated(format!("assignment proceeds under {id}")))
        // ...the financier pays out the face value's worth of claim...
        .with(Entry::new(2, assignee_account, face_value.negate().map_err(TradeError::Kernel)?, stamp)
            .narrated(format!("receivable acquired under {id}")))
        // ...and the difference is the financier's income.
        .with(Entry::new(3, discount_account, discount.clone(), stamp)
            .narrated("discount earned"));

    ps.seal(chart, epoch).map_err(TradeError::Kernel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gbs_kernel::Account;

    fn usd(m: i128) -> Amount {
        Amount::minor_2dp(m, "USD")
    }
    fn st(e: u64, d: i64) -> Stamp {
        Stamp::new(Epoch(e), d)
    }

    fn issuing_bank() -> Actor {
        Actor::new("issuing-bank")
            .holding("lc.issue")
            .holding("lc.amend")
            .holding("lc.honour")
            .holding("lc.expire")
    }
    fn exporter() -> Actor {
        Actor::new("exporter").holding("lc.present")
    }
    fn advising_bank() -> Actor {
        Actor::new("advising-bank").holding("lc.advise").holding("lc.confirm")
    }

    fn chart() -> Chart {
        Chart::new()
            .with(Account::new("memo.lc.contra", "bank", "USD"))
            .with(Account::new("memo.lc.liability", "bank", "USD"))
            .with(Account::new("beneficiary.usd", "exporter", "USD"))
            .with(Account::new("applicant.usd", "importer", "USD"))
            .with(Account::new("supplier.usd", "supplier", "USD"))
            .with(Account::new("financier.usd", "financier", "USD"))
            .with(Account::new("discount.income", "financier", "USD"))
            .with(Account::new("applicant.limit", "importer", "USD"))
    }

    fn lc() -> LetterOfCredit {
        LetterOfCredit::new("lc-4471", "importer", "exporter", usd(100_000_00), 214)
    }

    // ── the lifecycle ───────────────────────────────────────────────────────────────

    #[test]
    fn a_credit_runs_its_full_lifecycle() {
        let mut l = lc();
        assert_eq!(l.state(), State::new("draft"));
        l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
        assert_eq!(l.state(), State::new("issued"));

        l.lifecycle.transition("advise", &advising_bank(), st(11, 1), None).unwrap();
        l.lifecycle.transition("confirm", &advising_bank(), st(12, 2), None).unwrap();
        assert_eq!(l.state(), State::new("confirmed"));

        l.present(&exporter(), &usd(100_000_00), st(20, 30), &chart(), Epoch(20)).unwrap();
        l.lifecycle.transition("honour", &issuing_bank(), st(21, 35), None).unwrap();
        assert_eq!(l.state(), State::new("honoured"));
        assert!(l.lifecycle.is_terminal());
    }

    #[test]
    fn the_state_at_the_moment_of_presentation_is_answerable_years_later() {
        // The question a dispute asks, and the reason there is no status column.
        let mut l = lc();
        l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
        l.lifecycle.transition("advise", &advising_bank(), st(11, 1), None).unwrap();
        l.present(&exporter(), &usd(50_000_00), st(20, 30), &chart(), Epoch(20)).unwrap();
        l.lifecycle.transition("honour", &issuing_bank(), st(30, 40), None).unwrap();

        assert_eq!(l.state(), State::new("honoured"), "today");
        assert_eq!(l.state_at(Epoch(20)), State::new("presented"), "when it was presented");
        assert_eq!(l.state_at(Epoch(15)), State::new("advised"), "and before that");
        assert_eq!(l.state_at(Epoch(10)), State::new("issued"));
        assert_eq!(l.state_at(Epoch(1)), State::new("draft"));
    }

    #[test]
    fn a_credit_is_amendable_from_every_live_state_and_from_no_terminal_one() {
        for (advance_to, rule, actor) in [
            ("issued", "issue", issuing_bank()),
            ("advised", "advise", advising_bank()),
        ] {
            let mut l = lc();
            l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
            if advance_to != "issued" {
                l.lifecycle.transition(rule, &actor, st(11, 1), None).unwrap();
            }
            assert!(
                l.lifecycle.transition("amend", &issuing_bank(), st(12, 2), None).is_ok(),
                "amendable from {advance_to}"
            );
        }

        // And not once honoured.
        let mut l = lc();
        l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
        l.present(&exporter(), &usd(1), st(20, 30), &chart(), Epoch(20)).unwrap();
        l.lifecycle.transition("honour", &issuing_bank(), st(21, 31), None).unwrap();
        assert!(l.lifecycle.transition("amend", &issuing_bank(), st(22, 32), None).is_err());
    }

    #[test]
    fn the_lc_rule_set_has_no_undeclared_sinks() {
        // Every state can be left, or is a declared terminal. A credit that got stuck would
        // be a contingent liability nobody could remove from the balance sheet.
        assert!(lc().lifecycle.undeclared_sinks().is_empty(), "{:?}", lc().lifecycle.undeclared_sinks());
    }

    #[test]
    fn presenting_without_the_capability_is_refused() {
        let mut l = lc();
        l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
        let e = l.present(&advising_bank(), &usd(1), st(20, 30), &chart(), Epoch(20)).unwrap_err();
        assert!(matches!(e, TradeError::Lifecycle(LifecycleError::Unauthorised { .. })));
    }

    #[test]
    fn issuing_without_the_capability_records_neither_a_transition_nor_a_posting() {
        // Atomicity across the two mechanisms: a refused transition must leave no event and
        // no accounting.
        let mut l = lc();
        assert!(l.issue(&exporter(), st(10, 0), &chart(), Epoch(10)).is_err());
        assert_eq!(l.state(), State::new("draft"));
        assert_eq!(l.lifecycle.events().len(), 0);
    }

    // ── expiry, on the world axis ───────────────────────────────────────────────────

    #[test]
    fn a_presentation_after_expiry_is_refused_and_names_both_dates() {
        // The sentence that settles the argument.
        let mut l = lc();
        l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
        let e = l.present(&exporter(), &usd(1), st(50, 215), &chart(), Epoch(50)).unwrap_err();
        assert!(matches!(e, TradeError::Expired { expired_on: 214, presented_on: 215 }));
        assert!(e.to_string().contains("214"), "{e}");
        assert!(e.to_string().contains("215"), "{e}");
    }

    #[test]
    fn expiry_is_decided_on_the_valid_time_axis_and_not_by_a_clock() {
        // A presentation dated within the credit's life is good even if it is *recorded*
        // long afterwards — a late-arriving courier, a reconstructed record. The epoch is
        // 900 and the value date is 200, and the credit was live on day 200.
        let mut l = lc();
        l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
        assert!(l.present(&exporter(), &usd(1), st(900, 200), &chart(), Epoch(900)).is_ok());
    }

    #[test]
    fn a_presentation_exactly_on_the_expiry_day_is_good() {
        let mut l = lc();
        l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
        assert!(l.present(&exporter(), &usd(1), st(50, 214), &chart(), Epoch(50)).is_ok());
    }

    // ── amounts ─────────────────────────────────────────────────────────────────────

    #[test]
    fn issuing_posts_the_contingent_liability_and_it_conserves() {
        // A contingent liability that is not posted does not appear in an exposure
        // calculation, which is how an institution discovers its true exposure during a
        // crisis rather than before one.
        let mut l = lc();
        let sealed = l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
        sealed.verify().unwrap();
        assert_eq!(sealed.entries().len(), 2);
        assert_eq!(sealed.entries()[0].amount.minor, -100_000_00);
    }

    #[test]
    fn partial_presentations_draw_the_credit_down_and_the_last_one_cannot_overdraw() {
        let mut l = lc();
        l.issue(&issuing_bank(), st(10, 0), &chart(), Epoch(10)).unwrap();
        l.present(&exporter(), &usd(60_000_00), st(20, 30), &chart(), Epoch(20)).unwrap();
        assert_eq!(l.undrawn().unwrap().minor, 40_000_00);

        let e = l.present(&exporter(), &usd(40_000_01), st(21, 31), &chart(), Epoch(21)).unwrap_err();
        assert!(matches!(e, TradeError::ExceedsCredit { .. }));
        assert!(l.present(&exporter(), &usd(40_000_00), st(22, 32), &chart(), Epoch(22)).is_ok());
        assert_eq!(l.undrawn().unwrap().minor, 0);
    }

    #[test]
    fn an_issued_credit_encumbers_the_applicants_limit_for_its_full_value() {
        // Modelled as a hold rather than a number, so the same limit cannot be committed
        // twice.
        let l = lc();
        let h = l.applicant_hold("applicant.limit", st(10, 0));
        assert_eq!(h.amount.minor, 100_000_00);
        assert!(h.encumbers(Epoch(11), 100), "live during the credit's life");
        assert!(!h.encumbers(Epoch(11), 215), "and not after it expires");
    }

    // ── supply-chain finance ────────────────────────────────────────────────────────

    #[test]
    fn a_receivable_assignment_reconciles_across_all_three_parties() {
        // The conservation obligation the scope names: assignor's face value = assignee's
        // proceeds + discount, exactly.
        let sealed = assign_receivable(
            "scf-1",
            &usd(1_000_000),
            &usd(23_500),
            "supplier.usd",
            "financier.usd",
            "discount.income",
            st(10, 0),
            &chart(),
            Epoch(10),
        )
        .unwrap();
        sealed.verify().unwrap();
        assert_eq!(sealed.entries().len(), 3);
        assert_eq!(sealed.entries()[0].amount.minor, 976_500, "the supplier's proceeds");
        assert_eq!(sealed.entries()[1].amount.minor, -1_000_000, "the financier's outlay");
        assert_eq!(sealed.entries()[2].amount.minor, 23_500, "the discount earned");
    }

    #[test]
    fn a_zero_discount_assignment_is_legal_and_conserves() {
        // Assignment at par — an intragroup transfer, or a factoring arrangement whose fee
        // is charged separately.
        let sealed = assign_receivable(
            "scf-2",
            &usd(500_000),
            &usd(0),
            "supplier.usd",
            "financier.usd",
            "discount.income",
            st(10, 0),
            &chart(),
            Epoch(10),
        )
        .unwrap();
        sealed.verify().unwrap();
    }

    #[test]
    fn an_assignment_at_a_discount_exceeding_the_face_value_still_conserves_but_is_visible() {
        // A negative-proceeds assignment: economically strange, arithmetically fine, and
        // the kernel will accept it. Recorded as a test so the behaviour is a known
        // decision rather than a discovery: the product does not refuse it, because a
        // deeply distressed receivable really can be assigned at a discount above par, and
        // refusing it would push the transaction into a manual journal.
        let sealed = assign_receivable(
            "scf-3",
            &usd(100_000),
            &usd(150_000),
            "supplier.usd",
            "financier.usd",
            "discount.income",
            st(10, 0),
            &chart(),
            Epoch(10),
        )
        .unwrap();
        sealed.verify().unwrap();
        assert_eq!(sealed.entries()[0].amount.minor, -50_000, "the supplier pays to be rid of it");
    }
}
