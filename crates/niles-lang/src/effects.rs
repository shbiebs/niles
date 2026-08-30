//! The consistency-effect calculus.
//!
//! This is the machinery behind the last two clauses of the Niles soundness theorem
//! (thesis Contribution 4): a well-typed program **cannot overdraw without authorization**,
//! and every read it performs is **at or above the rung the enclosing view promises**.
//! Conservation and currency discipline are decided next door in [`crate::currency_rows`];
//! what is decided here is *authority* and *consistency*.
//!
//! # Effects
//!
//! An effect is a capability the runtime must grant, recorded in the function's type:
//!
//! ```text
//! fn transfer(a: Acct, b: Acct, m: Money<usd>) -> Result<TxnId, E>
//!     ! { append, debit<usd>, credit<usd>, read@snapshot }
//! ```
//!
//! Rows are **open** by default — an implied row variable — so a function declaring
//! `{ append }` may be used where `{ append, emit }` is permitted. Closing a row is what a
//! `serve` contract does, and what a `capability` declaration does.
//!
//! # The rung is part of the effect, not a comment on it
//!
//! `read@snapshot` and `read@ledger_consistent` are *different effects*, and that is the
//! whole point of the calculus. It makes one property statically checkable that is
//! otherwise a source of quiet, expensive production bugs:
//!
//! > **Rung monotonicity.** A view served at rung ℓ may not read from a view served at a
//! > rung below ℓ.
//!
//! An availability decision reading a balance that is itself only bounded-stale is not
//! ledger-consistent, however carefully the decision path was written; the staleness
//! entered two hops upstream. Supervisory guidance calls the resulting failure by name —
//! two derived views of one ledger disagreeing at the moment a decision is made — and it is
//! the exact failure this rule makes unspellable. Under this calculus that program does
//! not compile, and the error names both views and both rungs.
//!
//! # What this cannot buy
//!
//! Stated here because the thesis states it: **non-negativity is provably not
//! coordination-free** (Proposition 3.2), and no type system repeals that. The calculus
//! guarantees that a path which can reduce a balance below zero *holds an `Auth<E>`*; it
//! does not and cannot guarantee that the floor is respected without coordination. The
//! authorization effect is therefore a marker that coordination is required, not a
//! substitute for it.

use crate::diagnostics::{Applicability, Diagnostic};
use crate::lexer::Span;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The consistency ladder (thesis §3.7). Ordered: `Bounded` is weakest, `LedgerConsistent`
/// strongest. The ordering is not cosmetic — it is what `subsumes` and rung monotonicity
/// are defined over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rung {
    Bounded = 0,
    Monotonic = 1,
    ReadYourWrites = 2,
    Snapshot = 3,
    Serializable = 4,
    LedgerConsistent = 5,
}

impl Rung {
    pub fn parse(s: &str) -> Option<Rung> {
        Some(match s {
            "bounded" => Rung::Bounded,
            "monotonic" => Rung::Monotonic,
            "read_your_writes" => Rung::ReadYourWrites,
            "snapshot" => Rung::Snapshot,
            "serializable" => Rung::Serializable,
            "ledger_consistent" => Rung::LedgerConsistent,
            _ => return None,
        })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Rung::Bounded => "bounded",
            Rung::Monotonic => "monotonic",
            Rung::ReadYourWrites => "read_your_writes",
            Rung::Snapshot => "snapshot",
            Rung::Serializable => "serializable",
            Rung::LedgerConsistent => "ledger_consistent",
        }
    }
    pub fn all() -> &'static [Rung] {
        &[
            Rung::Bounded, Rung::Monotonic, Rung::ReadYourWrites,
            Rung::Snapshot, Rung::Serializable, Rung::LedgerConsistent,
        ]
    }
    /// Whether the rung is in the highly-available class. The top rung is provably in the
    /// unavailable one, which is why declaring it is a statement about what a view gives
    /// up rather than a free choice from a menu.
    pub fn is_highly_available(self) -> bool {
        self <= Rung::ReadYourWrites
    }
}

impl fmt::Display for Rung {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One effect.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    /// Read derived state at a rung. Two reads at different rungs are different effects.
    Read(Rung),
    /// Append to a base or ledger. The only write effect a base admits.
    Append,
    /// Mutate a `table`. Never legal against a base or ledger (W4).
    Mutate,
    /// Move money out of an account in a currency.
    Debit(String),
    /// Move money into an account in a currency.
    Credit(String),
    /// Place a reservation.
    Hold(String),
    /// Approach or cross a balance floor. **Requires an `Auth<E>` in scope** (W15, W16).
    Authorize(String),
    /// Lower a confidentiality level. Audited; requires a capability (W18).
    Declassify,
    /// Publish downstream.
    Emit,
    /// Change a declaration: grant, revoke, backfill, alter.
    Admin,
}

impl Effect {
    /// Whether this effect may only be exercised through an unforgeable capability.
    ///
    /// The set is deliberately small. Every member is an operation whose misuse is not
    /// recoverable by compensation: an unauthorized overdraft has already left the
    /// institution exposed, and a declassification has already disclosed.
    pub fn requires_authority(&self) -> bool {
        matches!(self, Effect::Authorize(_) | Effect::Declassify | Effect::Admin)
    }
    pub fn parse(name: &str, at: Option<&str>, args: &[String]) -> Option<Effect> {
        let arg = || args.first().cloned().unwrap_or_else(|| "*".into());
        Some(match name {
            "read" => Effect::Read(at.and_then(Rung::parse)?),
            "append" => Effect::Append,
            "mutate" => Effect::Mutate,
            "debit" => Effect::Debit(arg()),
            "credit" => Effect::Credit(arg()),
            "hold" => Effect::Hold(arg()),
            "authorize" => Effect::Authorize(arg()),
            "declassify" => Effect::Declassify,
            "emit" => Effect::Emit,
            "admin" => Effect::Admin,
            _ => return None,
        })
    }
    pub fn all_names() -> &'static [&'static str] {
        &["read", "append", "mutate", "debit", "credit", "hold", "authorize", "declassify", "emit", "admin"]
    }
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effect::Read(r) => write!(f, "read@{r}"),
            Effect::Append => f.write_str("append"),
            Effect::Mutate => f.write_str("mutate"),
            Effect::Debit(c) => write!(f, "debit<{c}>"),
            Effect::Credit(c) => write!(f, "credit<{c}>"),
            Effect::Hold(c) => write!(f, "hold<{c}>"),
            Effect::Authorize(c) => write!(f, "authorize<{c}>"),
            Effect::Declassify => f.write_str("declassify"),
            Effect::Emit => f.write_str("emit"),
            Effect::Admin => f.write_str("admin"),
        }
    }
}

/// A set of effects, with the place each was first incurred — because the interesting
/// diagnostic is never "this function has effect X", it is "this function has effect X
/// *because of this line*, and its declaration on that line says it does not".
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Row {
    set: BTreeSet<Effect>,
    origins: BTreeMap<Effect, Span>,
}

impl Row {
    pub fn new() -> Self {
        Row::default()
    }
    pub fn of(effects: impl IntoIterator<Item = Effect>) -> Self {
        let mut r = Row::new();
        for e in effects {
            r.add(e, Span::default());
        }
        r
    }
    pub fn add(&mut self, e: Effect, span: Span) {
        self.origins.entry(e.clone()).or_insert(span);
        self.set.insert(e);
    }
    pub fn union(&mut self, other: &Row) {
        for e in &other.set {
            self.origins.entry(e.clone()).or_insert_with(|| other.origins[e]);
            self.set.insert(e.clone());
        }
    }
    pub fn contains(&self, e: &Effect) -> bool {
        self.set.contains(e)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Effect> {
        self.set.iter()
    }
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
    pub fn len(&self) -> usize {
        self.set.len()
    }
    pub fn origin(&self, e: &Effect) -> Option<Span> {
        self.origins.get(e).copied()
    }

    /// The strictest rung this row reads at, if it reads at all.
    ///
    /// "Strictest" and not "weakest" because this is the rung the *reader* is entitled to
    /// assume; the weakest read is what bounds what the row can be served at, and that is
    /// [`Row::weakest_read`].
    pub fn strictest_read(&self) -> Option<Rung> {
        self.set.iter().filter_map(|e| match e {
            Effect::Read(r) => Some(*r),
            _ => None,
        }).max()
    }

    /// The weakest rung this row reads at. This is the ceiling on what a view containing
    /// it may promise: a computation is no fresher than its stalest input.
    pub fn weakest_read(&self) -> Option<Rung> {
        self.set.iter().filter_map(|e| match e {
            Effect::Read(r) => Some(*r),
            _ => None,
        }).min()
    }

    /// Row subsumption: is every effect of `self` permitted by `declared`?
    ///
    /// A `Read(r)` is permitted by a declared `Read(r')` whenever `r <= r'` — reading at a
    /// *weaker* rung than declared is safe, since the declaration is a promise about the
    /// strongest guarantee the caller may rely on. Every other effect must match exactly,
    /// and a currency-parameterised effect is matched by the wildcard `*`.
    pub fn subsumed_by(&self, declared: &Row) -> Vec<Effect> {
        self.set.iter().filter(|e| !declared_permits(declared, e)).cloned().collect()
    }
}

fn declared_permits(declared: &Row, e: &Effect) -> bool {
    if declared.set.contains(e) {
        return true;
    }
    match e {
        Effect::Read(r) => declared.set.iter().any(|d| matches!(d, Effect::Read(dr) if r <= dr)),
        Effect::Debit(_) => declared.set.contains(&Effect::Debit("*".into())),
        Effect::Credit(_) => declared.set.contains(&Effect::Credit("*".into())),
        Effect::Hold(_) => declared.set.contains(&Effect::Hold("*".into())),
        Effect::Authorize(_) => declared.set.contains(&Effect::Authorize("*".into())),
        _ => false,
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v: Vec<String> = self.set.iter().map(|e| e.to_string()).collect();
        write!(f, "{{ {} }}", v.join(", "))
    }
}

// ===================== the four judgements =====================

/// **W14.** A function's inferred effects must be permitted by its declared row.
pub fn check_declaration(
    fn_name: &str,
    inferred: &Row,
    declared: &Row,
    decl_span: Span,
) -> Vec<Diagnostic> {
    inferred
        .subsumed_by(declared)
        .into_iter()
        .map(|e| {
            let site = inferred.origin(&e).unwrap_or(decl_span);
            let mut d = Diagnostic::error(
                "NL0310",
                format!("`{fn_name}` has the effect `{e}`, which its declaration does not permit"),
            )
            .primary(site, format!("`{e}` incurred here"))
            .secondary(decl_span, format!("declared effects are {declared}"));
            if let Effect::Read(r) = &e {
                d = d.note(format!(
                    "a read at `{r}` is not covered by a declaration of a weaker rung: \
                     declaring less than you do would let a caller rely on a guarantee you never made"
                ));
            }
            d.suggest(
                decl_span,
                format!("! {{ {}, {e} }}", declared.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
                "add it to the declared row",
                Applicability::MachineApplicable,
            )
        })
        .collect()
}

/// **Rung monotonicity.** A view served at rung ℓ may not read below ℓ.
///
/// This is the judgement that makes the two-derived-views-disagreeing failure unspellable,
/// and the one clause of the calculus that is genuinely novel rather than a
/// currency-flavoured restatement of an existing effect system.
pub fn check_rung_monotonicity(
    view_name: &str,
    served_at: Rung,
    body: &Row,
    view_span: Span,
    contract_span: Span,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for e in body.iter() {
        let Effect::Read(r) = e else { continue };
        if *r < served_at {
            let site = body.origin(e).unwrap_or(view_span);
            out.push(
                Diagnostic::error(
                    "NL0311",
                    format!("view `{view_name}` promises `{served_at}` but reads at `{r}`"),
                )
                .primary(site, format!("this read is only `{r}`"))
                .secondary(contract_span, format!("`{served_at}` promised here"))
                .note("a computation is no fresher than its stalest input: staleness entering upstream is not removed by a stricter contract downstream")
                .note("this is the failure supervisory guidance describes as two derived views of one ledger disagreeing at the moment a decision is made")
                .suggest(
                    contract_span,
                    format!("consistency: {r}"),
                    format!("either weaken this view to `{r}`, or raise the input's contract to `{served_at}`"),
                    Applicability::MaybeIncorrect,
                ),
            );
        }
    }
    out
}

/// **W15/W16.** An effect requiring authority must be reachable only from a capability in
/// scope.
pub fn check_authority(
    fn_name: &str,
    inferred: &Row,
    capabilities: &Row,
    decl_span: Span,
) -> Vec<Diagnostic> {
    inferred
        .iter()
        .filter(|e| e.requires_authority())
        .filter(|e| !declared_permits(capabilities, e))
        .map(|e| {
            let site = inferred.origin(e).unwrap_or(decl_span);
            Diagnostic::error(
                "NL0312",
                format!("`{fn_name}` exercises `{e}` without holding the capability for it"),
            )
            .primary(site, format!("`{e}` requires an `Auth<{e}>` in scope"))
            .note("capabilities are unforgeable: they are received as parameters or granted, never constructed")
            .note("this check guarantees that an overdrawing path *holds authority*; it does not make the floor coordination-free, which Proposition 3.2 shows is impossible")
            .suggest(
                decl_span,
                format!("auth: Auth<{e}>"),
                "take the capability as a parameter",
                Applicability::HasPlaceholders,
            )
        })
        .collect()
}

// ===================== linearity =====================

/// How many times a linear value was consumed. **W8**: exactly once.
///
/// `Debit`, `Credit` and `Hold` are linear because each denotes a *half* of a ledger fact
/// that has not yet been sealed. Dropping one silently loses money; consuming one twice
/// silently creates it. Neither is a runtime error that can be recovered from, which is
/// why both are typing errors here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearKind {
    Debit,
    Credit,
    Hold,
}

impl LinearKind {
    fn describe(self) -> &'static str {
        match self {
            LinearKind::Debit => "debit half",
            LinearKind::Credit => "credit half",
            LinearKind::Hold => "hold",
        }
    }
    fn consumer(self) -> &'static str {
        match self {
            LinearKind::Debit | LinearKind::Credit => "`post(..)`",
            LinearKind::Hold => "`resolve .. post/void/expire`",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinearValue {
    pub name: String,
    pub kind: LinearKind,
    pub bound_at: Span,
    pub uses: Vec<Span>,
}

/// Check every linear binding in one scope.
pub fn check_linearity(values: &[LinearValue]) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for v in values {
        match v.uses.len() {
            1 => {}
            0 => out.push(
                Diagnostic::error("NL0320", format!("`{}` is never consumed", v.name))
                    .primary(v.bound_at, format!("this {} is dropped", v.kind.describe()))
                    .note(format!("a {} must be consumed exactly once, by {}", v.kind.describe(), v.kind.consumer()))
                    .note("dropping it would lose the movement silently, which is the failure mode double-entry exists to prevent"),
            ),
            n => {
                let mut d = Diagnostic::error(
                    "NL0321",
                    format!("`{}` is consumed {n} times, but must be consumed exactly once", v.name),
                )
                .primary(v.uses[1], "consumed again here")
                .secondary(v.uses[0], "first consumed here")
                .secondary(v.bound_at, format!("this {} is linear", v.kind.describe()));
                if v.kind == LinearKind::Hold {
                    d = d.note("resolving a hold twice would release the same reservation twice, and capture against it twice");
                } else {
                    d = d.note("posting the same half twice would create money");
                }
                out.push(d);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sp(a: u32) -> Span {
        Span::new(a as usize, a as usize + 1)
    }

    #[test]
    fn the_ladder_is_ordered_and_the_top_is_unavailable() {
        assert!(Rung::Bounded < Rung::LedgerConsistent);
        assert!(Rung::Snapshot < Rung::Serializable);
        assert!(Rung::ReadYourWrites.is_highly_available());
        assert!(!Rung::LedgerConsistent.is_highly_available());
        for r in Rung::all() {
            assert_eq!(Rung::parse(r.as_str()), Some(*r), "round trip for {r}");
        }
    }

    #[test]
    fn a_read_at_a_weaker_rung_is_permitted_by_a_stricter_declaration() {
        // Declaring `read@snapshot` and doing `read@bounded` is safe: the declaration is a
        // promise about the strongest guarantee a caller may rely on.
        let declared = Row::of([Effect::Read(Rung::Snapshot), Effect::Append]);
        let inferred = Row::of([Effect::Read(Rung::Bounded), Effect::Append]);
        assert!(inferred.subsumed_by(&declared).is_empty());
    }

    #[test]
    fn a_read_at_a_stricter_rung_is_not() {
        let declared = Row::of([Effect::Read(Rung::Bounded)]);
        let inferred = Row::of([Effect::Read(Rung::LedgerConsistent)]);
        let missing = inferred.subsumed_by(&declared);
        assert_eq!(missing, vec![Effect::Read(Rung::LedgerConsistent)]);
        let ds = check_declaration("f", &inferred, &declared, sp(1));
        assert_eq!(ds.len(), 1);
        assert_eq!(ds[0].code, "NL0310");
        assert_eq!(ds[0].labels.len(), 2, "must point at the site and the declaration");
    }

    #[test]
    fn an_undeclared_money_effect_is_reported_with_its_currency() {
        let declared = Row::of([Effect::Append]);
        let mut inferred = Row::new();
        inferred.add(Effect::Append, sp(1));
        inferred.add(Effect::Debit("usd".into()), sp(9));
        let ds = check_declaration("pay", &inferred, &declared, sp(1));
        assert_eq!(ds.len(), 1);
        assert!(ds[0].msg.contains("debit<usd>"), "{}", ds[0].msg);
        assert_eq!(ds[0].labels[0].span, sp(9), "must point at the debit, not the signature");
    }

    #[test]
    fn a_currency_wildcard_covers_every_currency() {
        let declared = Row::of([Effect::Debit("*".into())]);
        let inferred = Row::of([Effect::Debit("usd".into()), Effect::Debit("jpy".into())]);
        assert!(inferred.subsumed_by(&declared).is_empty());
    }

    #[test]
    fn rung_monotonicity_rejects_a_strict_view_over_a_stale_input() {
        // The central judgement: an availability decision reading a bounded-stale balance
        // is not ledger-consistent, however the decision path was written.
        let mut body = Row::new();
        body.add(Effect::Read(Rung::Bounded), sp(20));
        let ds = check_rung_monotonicity("available_balance", Rung::LedgerConsistent, &body, sp(1), sp(5));
        assert_eq!(ds.len(), 1);
        assert_eq!(ds[0].code, "NL0311");
        assert!(ds[0].msg.contains("ledger_consistent") && ds[0].msg.contains("bounded"), "{}", ds[0].msg);
        // Both the read site and the contract must be shown.
        assert_eq!(ds[0].labels[0].span, sp(20));
        assert_eq!(ds[0].labels[1].span, sp(5));
    }

    #[test]
    fn rung_monotonicity_permits_a_weak_view_over_a_strict_input() {
        // The safe direction: a statement view may read a ledger-consistent balance.
        let mut body = Row::new();
        body.add(Effect::Read(Rung::LedgerConsistent), sp(20));
        assert!(check_rung_monotonicity("statement", Rung::Bounded, &body, sp(1), sp(5)).is_empty());
    }

    #[test]
    fn a_view_is_no_fresher_than_its_stalest_input() {
        let mut body = Row::new();
        body.add(Effect::Read(Rung::LedgerConsistent), sp(1));
        body.add(Effect::Read(Rung::Snapshot), sp(2));
        body.add(Effect::Read(Rung::Bounded), sp(3));
        assert_eq!(body.weakest_read(), Some(Rung::Bounded));
        assert_eq!(body.strictest_read(), Some(Rung::LedgerConsistent));
        // ... so the strictest contract this body can honestly carry is `bounded`.
        assert!(check_rung_monotonicity("v", Rung::Bounded, &body, sp(1), sp(5)).is_empty());
        assert_eq!(check_rung_monotonicity("v", Rung::Snapshot, &body, sp(1), sp(5)).len(), 1);
    }

    #[test]
    fn authority_is_required_for_authorization_and_declassification() {
        let mut inferred = Row::new();
        inferred.add(Effect::Authorize("usd".into()), sp(11));
        inferred.add(Effect::Declassify, sp(12));
        inferred.add(Effect::Append, sp(13));
        let ds = check_authority("overdraw", &inferred, &Row::new(), sp(1));
        assert_eq!(ds.len(), 2, "append needs no capability; the other two do");
        assert!(ds.iter().all(|d| d.code == "NL0312"));
        // Holding the capability discharges it.
        let caps = Row::of([Effect::Authorize("usd".into()), Effect::Declassify]);
        assert!(check_authority("overdraw", &inferred, &caps, sp(1)).is_empty());
    }

    #[test]
    fn the_capability_set_is_deliberately_small() {
        // Every effect requiring authority is one whose misuse cannot be compensated.
        // Growing this set is a change to the thesis's claims, not a feature.
        let requiring: Vec<&str> = ["read", "append", "mutate", "debit", "credit", "hold",
                                    "authorize", "declassify", "emit", "admin"]
            .into_iter()
            .filter(|n| Effect::parse(n, Some("snapshot"), &["usd".into()])
                .map_or(false, |e| e.requires_authority()))
            .collect();
        assert_eq!(requiring, vec!["authorize", "declassify", "admin"]);
    }

    #[test]
    fn a_linear_value_must_be_consumed_exactly_once() {
        let ok = LinearValue { name: "d".into(), kind: LinearKind::Debit, bound_at: sp(1), uses: vec![sp(2)] };
        assert!(check_linearity(&[ok]).is_empty());

        let dropped = LinearValue { name: "d".into(), kind: LinearKind::Debit, bound_at: sp(1), uses: vec![] };
        let ds = check_linearity(&[dropped]);
        assert_eq!(ds[0].code, "NL0320");
        assert!(ds[0].notes.iter().any(|n| n.contains("lose the movement")));

        let twice = LinearValue { name: "h".into(), kind: LinearKind::Hold, bound_at: sp(1), uses: vec![sp(2), sp(3)] };
        let ds = check_linearity(&[twice]);
        assert_eq!(ds[0].code, "NL0321");
        assert_eq!(ds[0].labels.len(), 3, "the second use, the first use, and the binding");
        assert!(ds[0].notes.iter().any(|n| n.contains("release the same reservation twice")));
    }

    #[test]
    fn effect_rows_display_stably() {
        let r = Row::of([Effect::Append, Effect::Read(Rung::Snapshot), Effect::Debit("usd".into())]);
        // Ordering comes from the enum's declaration order, so a diagnostic reads the same
        // way every time — which matters because these strings appear in UI test
        // expectations and in the thesis.
        assert_eq!(r.to_string(), "{ read@snapshot, append, debit<usd> }");
    }
}
