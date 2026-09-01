//! **M5 — capability-gated state machines.** Lifecycles whose transitions *are* ledger
//! events.
//!
//! A letter of credit is issued, amended, presented against, and either honoured or
//! dishonoured. An M&A deal moves through mandate, diligence, signing and closing. A
//! dispute is raised, represented, and arbitrated. Every one of these is a lifecycle, and
//! every banking system has dozens.
//!
//! # There is no status field, and that is the whole design
//!
//! The state is a **fold over the transition events**, not a column. This is the same
//! argument the kernel makes about balances, applied to lifecycles, and it has the same
//! justification: a stored status is a second source of truth that can disagree with the
//! events that produced it, and when they disagree there is no principled way to decide
//! which is right.
//!
//! It also gives something a status column cannot: **the state at any past epoch**. "Was
//! this LC live when the presentation arrived?" is a fold over a prefix, and it returns the
//! same answer today and in seven years when the litigation starts. A status column has
//! already been overwritten and can only report the present.
//!
//! # Capabilities
//!
//! Each transition names a capability the actor must hold. The thesis checks this
//! statically (Contribution 4's effect rows, thesis §6.16); this module is the runtime half.
//! The static check is the valuable one — an amendment written without the issuing-bank
//! capability does not compile — and this exists because a transition can also arrive over
//! a wire from a party whose capabilities are not known until run time.
//!
//! # Why transitions carry a posting set
//!
//! Some transitions move value and some do not. Issuing an LC creates a contingent
//! liability — which is a posting, to memorandum accounts. Amending an expiry date is not.
//! Modelling the posting as `Option` on the transition keeps the two facts in one place, so
//! there is no way to record a transition and forget its accounting, which is the single
//! most common defect in a lifecycle implemented as a status column plus a separate posting
//! routine.

use gbs_kernel::{PostingSet, Stamp, Epoch};
use std::collections::BTreeSet;
use std::fmt;

/// A capability an actor may hold.
///
/// A string newtype rather than an enum, because the set is deployment-specific — an
/// institution's capability vocabulary is a policy artefact, not a language one — and a
/// closed enum would force a kernel change to add a role, which is exactly the falsification
/// condition of `ARCHITECTURE.md` §1.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Capability(String);

impl Capability {
    pub fn new(c: impl Into<String>) -> Self {
        Capability(c.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Who is attempting a transition, and what they may do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub id: String,
    caps: BTreeSet<Capability>,
}

impl Actor {
    pub fn new(id: impl Into<String>) -> Self {
        Actor { id: id.into(), caps: BTreeSet::new() }
    }

    pub fn holding(mut self, cap: &str) -> Self {
        self.caps.insert(Capability::new(cap));
        self
    }

    pub fn holds(&self, cap: &Capability) -> bool {
        self.caps.contains(cap)
    }

    /// Capabilities in stable order, for the diagnostic. `BTreeSet`, so an authorisation
    /// failure message is byte-identical across runs.
    pub fn capabilities(&self) -> impl Iterator<Item = &Capability> {
        self.caps.iter()
    }
}

/// A named state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct State(String);

impl State {
    pub fn new(s: impl Into<String>) -> Self {
        State(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A legal transition: from a state, to a state, requiring a capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub from: State,
    pub to: State,
    pub requires: Capability,
    /// Whether this transition is terminal. A terminal state has no outgoing rules, and
    /// declaring it explicitly lets [`Lifecycle::is_terminal`] answer without searching —
    /// and lets a rule set be checked for states that can be entered and never left.
    pub terminal: bool,
}

impl Rule {
    pub fn new(name: &str, from: &str, to: &str, requires: &str) -> Self {
        Rule {
            name: name.into(),
            from: State::new(from),
            to: State::new(to),
            requires: Capability::new(requires),
            terminal: false,
        }
    }

    pub fn terminal(mut self) -> Self {
        self.terminal = true;
        self
    }
}

/// One transition that actually happened.
#[derive(Debug, Clone)]
pub struct Event {
    pub rule: String,
    pub actor: String,
    pub from: State,
    pub to: State,
    pub stamp: Stamp,
    /// The accounting this transition produced, if any.
    ///
    /// `Option`, in the same structure as the transition, so that recording a transition
    /// and forgetting its posting is not expressible. In a status-column design these are
    /// two writes in two places and the second one is the one that gets missed.
    pub postings: Option<PostingSet>,
}

/// What a lifecycle refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    /// The actor lacks the capability. Names the capability required and the ones held,
    /// because "unauthorised" alone sends an operator to guess.
    Unauthorised { rule: String, required: Capability, actor: String, held: Vec<Capability> },
    /// No rule permits this transition from the current state.
    NoSuchTransition { rule: String, from: State },
    /// The lifecycle has reached a terminal state.
    ///
    /// A distinct variant from `NoSuchTransition` even though both mean "you cannot do
    /// that", because "the LC has already been honoured" and "that is not a valid step from
    /// here" send an operator to two different places.
    Terminal { state: State, attempted: String },
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecycleError::Unauthorised { rule, required, actor, held } => {
                write!(f, "`{actor}` cannot perform `{rule}`: it requires the `{required}` capability. Held: ")?;
                if held.is_empty() {
                    write!(f, "(none)")
                } else {
                    let names: Vec<&str> = held.iter().map(|c| c.as_str()).collect();
                    write!(f, "{}", names.join(", "))
                }
            }
            LifecycleError::NoSuchTransition { rule, from } => {
                write!(f, "`{rule}` is not a legal transition from `{from}`")
            }
            LifecycleError::Terminal { state, attempted } => write!(
                f,
                "the lifecycle is terminal at `{state}`; `{attempted}` cannot follow it"
            ),
        }
    }
}

/// A lifecycle: a rule set, an initial state, and the events that have happened.
#[derive(Debug, Clone)]
pub struct Lifecycle {
    pub id: String,
    pub initial: State,
    rules: Vec<Rule>,
    events: Vec<Event>,
}

impl Lifecycle {
    pub fn new(id: impl Into<String>, initial: &str, rules: Vec<Rule>) -> Self {
        Lifecycle {
            id: id.into(),
            initial: State::new(initial),
            rules,
            events: Vec::new(),
        }
    }

    /// The current state: a **fold over the events**, computed every time.
    ///
    /// Not cached. Caching it would reintroduce the status field this design exists to
    /// remove, and the fold is over a handful of events — a lifecycle with enough
    /// transitions for this to be slow is a lifecycle with a design problem.
    pub fn state(&self) -> State {
        self.events.last().map(|e| e.to.clone()).unwrap_or_else(|| self.initial.clone())
    }

    /// **The state as of a past epoch.** What a status column cannot answer.
    ///
    /// "Was this letter of credit live when the presentation arrived?" is a question asked
    /// years later, in a dispute, and it is a fold over a prefix. The answer is stable
    /// forever because the prefix is immutable.
    pub fn state_at(&self, anchor: Epoch) -> State {
        self.events
            .iter()
            .filter(|e| e.stamp.visible_at(anchor))
            .next_back()
            .map(|e| e.to.clone())
            .unwrap_or_else(|| self.initial.clone())
    }

    pub fn is_terminal(&self) -> bool {
        let s = self.state();
        self.events.last().map(|e| {
            self.rules.iter().any(|r| r.name == e.rule && r.terminal)
        }).unwrap_or(false)
            || (!self.rules.iter().any(|r| r.from == s) && !self.events.is_empty())
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Attempt a transition.
    ///
    /// Checks, in order: terminality, the existence of a rule from the current state, then
    /// the capability. The order is chosen for the diagnostic — telling someone they lack a
    /// capability for a transition that was never legal from here would send them to request
    /// a permission that would not have helped.
    pub fn transition(
        &mut self,
        rule_name: &str,
        actor: &Actor,
        stamp: Stamp,
        postings: Option<PostingSet>,
    ) -> Result<&Event, LifecycleError> {
        let current = self.state();

        if self.is_terminal() {
            return Err(LifecycleError::Terminal {
                state: current,
                attempted: rule_name.to_string(),
            });
        }

        let rule = self
            .rules
            .iter()
            .find(|r| r.name == rule_name && r.from == current)
            .cloned()
            .ok_or_else(|| LifecycleError::NoSuchTransition {
                rule: rule_name.to_string(),
                from: current.clone(),
            })?;

        if !actor.holds(&rule.requires) {
            return Err(LifecycleError::Unauthorised {
                rule: rule.name.clone(),
                required: rule.requires.clone(),
                actor: actor.id.clone(),
                held: actor.capabilities().cloned().collect(),
            });
        }

        self.events.push(Event {
            rule: rule.name,
            actor: actor.id.clone(),
            from: current,
            to: rule.to,
            stamp,
            postings,
        });
        Ok(self.events.last().expect("just pushed"))
    }

    /// States that can be entered but never left, other than declared terminals.
    ///
    /// A design-time check. A lifecycle with an undeclared sink is one where something gets
    /// stuck and somebody eventually updates the database by hand — which is the thing this
    /// whole architecture exists to make unnecessary.
    pub fn undeclared_sinks(&self) -> Vec<State> {
        let mut reachable: BTreeSet<State> = BTreeSet::new();
        reachable.insert(self.initial.clone());
        for r in &self.rules {
            reachable.insert(r.to.clone());
        }
        let declared_terminal: BTreeSet<State> =
            self.rules.iter().filter(|r| r.terminal).map(|r| r.to.clone()).collect();
        reachable
            .into_iter()
            .filter(|s| {
                !declared_terminal.contains(s) && !self.rules.iter().any(|r| &r.from == s)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gbs_kernel::{Account, Amount, Chart, Entry, PostingSet};

    /// A letter of credit, as the rule set it actually is.
    fn lc_rules() -> Vec<Rule> {
        vec![
            Rule::new("issue", "draft", "issued", "lc.issue"),
            Rule::new("amend", "issued", "issued", "lc.amend"),
            Rule::new("present", "issued", "presented", "lc.present"),
            Rule::new("honour", "presented", "honoured", "lc.honour").terminal(),
            Rule::new("dishonour", "presented", "dishonoured", "lc.honour").terminal(),
            Rule::new("expire", "issued", "expired", "lc.expire").terminal(),
        ]
    }

    fn lc() -> Lifecycle {
        Lifecycle::new("lc-4471", "draft", lc_rules())
    }

    fn issuing_bank() -> Actor {
        Actor::new("bank-of-x")
            .holding("lc.issue")
            .holding("lc.amend")
            .holding("lc.honour")
            .holding("lc.expire")
    }

    fn beneficiary() -> Actor {
        Actor::new("exporter-y").holding("lc.present")
    }

    fn st(e: u64, d: i64) -> Stamp {
        Stamp::new(Epoch(e), d)
    }

    fn contingent_posting() -> PostingSet {
        // Issuing an LC creates a contingent liability, recorded in memorandum accounts —
        // real double entry, on accounts that net to zero on the balance sheet.
        let d = Entry::new(1, "memo.contra.usd", Amount::minor_2dp(-1_000_000, "USD"), st(10, 0));
        let c = d.mirrored_to(2, "memo.liability.usd").unwrap();
        PostingSet::new("lc-4471-issue").with(d).with(c)
    }

    fn memo_chart() -> Chart {
        Chart::new()
            .with(Account::new("memo.contra.usd", "bank", "USD"))
            .with(Account::new("memo.liability.usd", "bank", "USD"))
    }

    // ── the state is a fold ─────────────────────────────────────────────────────────

    #[test]
    fn the_state_is_a_fold_and_there_is_no_status_field() {
        let mut l = lc();
        assert_eq!(l.state(), State::new("draft"));
        l.transition("issue", &issuing_bank(), st(10, 0), None).unwrap();
        assert_eq!(l.state(), State::new("issued"));
        l.transition("present", &beneficiary(), st(20, 5), None).unwrap();
        assert_eq!(l.state(), State::new("presented"));

        let d = format!("{l:?}");
        assert!(!d.contains("status"), "status is a fold, never a field");
    }

    #[test]
    fn the_state_at_a_past_epoch_is_answerable_which_a_status_column_cannot_do() {
        // The question a dispute asks years later: was the LC live when the presentation
        // arrived? A status column has been overwritten and can only report the present.
        let mut l = lc();
        l.transition("issue", &issuing_bank(), st(10, 0), None).unwrap();
        l.transition("present", &beneficiary(), st(20, 5), None).unwrap();
        l.transition("honour", &issuing_bank(), st(30, 9), None).unwrap();

        assert_eq!(l.state(), State::new("honoured"));
        assert_eq!(l.state_at(Epoch(30)), State::new("honoured"));
        assert_eq!(l.state_at(Epoch(29)), State::new("presented"));
        assert_eq!(l.state_at(Epoch(19)), State::new("issued"));
        assert_eq!(l.state_at(Epoch(9)), State::new("draft"), "before anything happened");
    }

    #[test]
    fn the_answer_at_a_past_epoch_never_changes_however_much_happens_later() {
        // Immutability, as the property that makes an audit answer stable.
        let mut l = lc();
        l.transition("issue", &issuing_bank(), st(10, 0), None).unwrap();
        let at_15 = l.state_at(Epoch(15));
        for (rule, actor, e) in [
            ("present", beneficiary(), 20u64),
            ("honour", issuing_bank(), 30),
        ] {
            l.transition(rule, &actor, st(e, 0), None).unwrap();
            assert_eq!(l.state_at(Epoch(15)), at_15, "the past does not move");
        }
    }

    // ── capabilities ────────────────────────────────────────────────────────────────

    #[test]
    fn a_transition_without_its_capability_is_refused_and_says_which_one() {
        // "Unauthorised" alone sends an operator to guess. The message names the capability
        // required and the ones held.
        let mut l = lc();
        let e = l.transition("issue", &beneficiary(), st(10, 0), None).unwrap_err();
        assert!(matches!(e, LifecycleError::Unauthorised { .. }));
        let msg = e.to_string();
        assert!(msg.contains("lc.issue"), "names the requirement: {msg}");
        assert!(msg.contains("lc.present"), "and what is held: {msg}");
    }

    #[test]
    fn an_actor_with_no_capabilities_gets_a_readable_message_rather_than_an_empty_list() {
        let mut l = lc();
        let e = l.transition("issue", &Actor::new("nobody"), st(10, 0), None).unwrap_err();
        assert!(e.to_string().contains("(none)"), "{e}");
    }

    #[test]
    fn held_capabilities_are_listed_in_stable_order() {
        // Determinism reaching an error message, so a golden test on one is possible.
        let a = Actor::new("x").holding("z.cap").holding("a.cap").holding("m.cap");
        let names: Vec<&str> = a.capabilities().map(|c| c.as_str()).collect();
        assert_eq!(names, vec!["a.cap", "m.cap", "z.cap"]);
    }

    // ── the rule set ────────────────────────────────────────────────────────────────

    #[test]
    fn a_transition_illegal_from_the_current_state_is_refused_distinctly() {
        // Distinct from an authorisation failure: telling someone they lack a capability
        // for a transition that was never legal from here sends them to request a
        // permission that would not have helped.
        let mut l = lc();
        let e = l.transition("honour", &issuing_bank(), st(10, 0), None).unwrap_err();
        assert!(matches!(e, LifecycleError::NoSuchTransition { .. }));
        assert!(e.to_string().contains("not a legal transition from `draft`"));
    }

    #[test]
    fn a_terminal_state_refuses_everything_and_says_it_is_terminal() {
        let mut l = lc();
        l.transition("issue", &issuing_bank(), st(10, 0), None).unwrap();
        l.transition("present", &beneficiary(), st(20, 0), None).unwrap();
        l.transition("honour", &issuing_bank(), st(30, 0), None).unwrap();
        assert!(l.is_terminal());

        let e = l.transition("amend", &issuing_bank(), st(40, 0), None).unwrap_err();
        assert!(matches!(e, LifecycleError::Terminal { .. }));
        assert!(e.to_string().contains("already") || e.to_string().contains("terminal"));
    }

    #[test]
    fn a_self_loop_transition_works_because_amendments_are_repeatable() {
        // An LC may be amended any number of times and stays issued. A rule set that could
        // not express `issued -> issued` would force an artificial state per amendment.
        let mut l = lc();
        l.transition("issue", &issuing_bank(), st(10, 0), None).unwrap();
        for e in 11..=15 {
            l.transition("amend", &issuing_bank(), st(e, 0), None).unwrap();
        }
        assert_eq!(l.state(), State::new("issued"));
        assert_eq!(l.events().len(), 6, "every amendment is an event, none overwrites another");
    }

    #[test]
    fn undeclared_sinks_are_reported_at_design_time() {
        // A state that can be entered and never left, other than a declared terminal, is
        // where something gets stuck and somebody eventually edits the database by hand.
        let leaky = Lifecycle::new(
            "x",
            "start",
            vec![
                Rule::new("go", "start", "middle", "c"),
                Rule::new("stall", "middle", "limbo", "c"),
            ],
        );
        assert_eq!(leaky.undeclared_sinks(), vec![State::new("limbo")]);
        // The LC rule set has none: every sink is a declared terminal.
        assert!(lc().undeclared_sinks().is_empty(), "{:?}", lc().undeclared_sinks());
    }

    // ── postings ────────────────────────────────────────────────────────────────────

    #[test]
    fn a_transition_carries_its_accounting_so_it_cannot_be_forgotten() {
        // The defect this structure removes: in a status-column design, the transition and
        // its posting are two writes in two places, and the second is the one that is
        // missed. Here they are one value.
        let mut l = lc();
        let ev = l
            .transition("issue", &issuing_bank(), st(10, 0), Some(contingent_posting()))
            .unwrap();
        let ps = ev.postings.clone().expect("issuing an LC creates a contingent liability");
        ps.seal(&memo_chart(), Epoch(10)).expect("and it conserves");
    }

    #[test]
    fn a_transition_that_moves_no_value_carries_no_postings() {
        // Amending an expiry date is not an accounting event. `None` is the honest answer,
        // not an empty posting set — which the kernel would refuse as `Empty` anyway.
        let mut l = lc();
        l.transition("issue", &issuing_bank(), st(10, 0), Some(contingent_posting())).unwrap();
        let ev = l.transition("amend", &issuing_bank(), st(11, 0), None).unwrap();
        assert!(ev.postings.is_none());
    }

    #[test]
    fn a_refused_transition_records_nothing_at_all() {
        // Atomicity at the lifecycle level: a failed transition must not leave an event
        // behind, or the fold would report a state that was never reached.
        let mut l = lc();
        assert!(l.transition("issue", &beneficiary(), st(10, 0), Some(contingent_posting())).is_err());
        assert_eq!(l.events().len(), 0);
        assert_eq!(l.state(), State::new("draft"));
    }

    #[test]
    fn the_capability_vocabulary_is_open_so_a_new_role_needs_no_kernel_change() {
        // The falsification condition of ARCHITECTURE.md §1, at this mechanism's scope: a
        // deployment adding a role must not require a change below the product layer.
        let custom = Lifecycle::new(
            "deal-9",
            "mandate",
            vec![
                Rule::new("diligence", "mandate", "diligence", "ib.mandate.sign"),
                Rule::new("sign", "diligence", "signed", "ib.deal.sign"),
                Rule::new("close", "signed", "closed", "ib.deal.close").terminal(),
            ],
        );
        assert!(custom.undeclared_sinks().is_empty());
        let banker = Actor::new("md-1").holding("ib.mandate.sign");
        assert!(banker.holds(&Capability::new("ib.mandate.sign")));
    }
}
