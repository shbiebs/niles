//! The currency-row solver.
//!
//! This is the machinery behind the first two clauses of the Niles soundness theorem
//! (thesis Contribution 4, §4.6): a well-typed program **conserves value** and **cannot
//! mismatch currencies**. Both are properties of a single object — the *currency row* of a
//! transaction — and both are decided here, statically, without running anything.
//!
//! # The idea
//!
//! A transaction's net effect on the world is not a number. It is a finite map from
//! currency to signed amount: a *row*. `debit(a, 10.00 usd)` contributes `usd ↦ −1000`;
//! `credit(b, 10.00 usd)` contributes `usd ↦ +1000`. The double-entry rule
//! `conserve per (txn, cur)` says exactly that **every entry of the row is zero at the
//! transaction boundary**, and nothing weaker: not that the total is zero (which would let
//! 10 USD cancel 10 EUR), but that each currency's entry is zero independently.
//!
//! That framing makes conservation a *typing* judgement rather than a runtime check, and
//! it makes the cross-currency rule fall out rather than needing separate machinery. There
//! is no single currency in which an FX transaction sums to zero, which is why `fx` is a
//! form of its own: it opens **two** rows, requires each to be zero, and records the rate
//! that relates them. A conversion that "just adds" two currencies has no well-typed
//! spelling, because addition of `Money<a>` and `Money<b>` requires `a ~ b`, and `usd` and
//! `eur` do not unify.
//!
//! # Why rows rather than a flat check
//!
//! Because currencies are not always known at the point of the arithmetic. A generic
//! `fn move<C>(from: Acct, to: Acct, m: Money<C>)` should type-check once, for all `C`,
//! and be conserving for all `C`. That requires *variables* in currency position and a
//! unifier over them — a row variable and a union-find — which is what this module is.
//! The alternative, monomorphising every currency, gives up the generality claim and
//! multiplies the code by the number of currencies a bank supports.
//!
//! # Amounts are symbolic
//!
//! Entries are linear combinations of *symbols* (opaque runtime amounts) plus a literal
//! constant, so `debit(a, x); credit(b, x)` conserves for an unknown `x`. Conservation is
//! therefore decidable statically in exactly the cases where it should be — when the same
//! amount reaches both sides — and reported honestly as *not proven* when the program
//! computes an amount the checker cannot see through. That is the correct boundary: a
//! checker that claimed to prove conservation for arbitrary computed amounts would be
//! claiming to solve arithmetic.

use crate::diagnostics::{Applicability, Diagnostic};
use crate::lexer::Span;
use std::collections::BTreeMap;
use std::fmt;

/// A currency, as far as the type system is concerned: either a declared one, or a
/// variable awaiting unification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cur {
    /// A declared currency, e.g. `usd`. Carries its scale so that a scale mismatch is a
    /// typing error rather than a rounding.
    Known(String),
    /// A unification variable, introduced by a generic `Money<C>`.
    Var(u32),
}

impl fmt::Display for Cur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cur::Known(c) => write!(f, "{c}"),
            Cur::Var(v) => write!(f, "?c{v}"),
        }
    }
}

/// An opaque runtime amount the checker cannot see the value of — a parameter, a column, a
/// call result. Two occurrences of the same symbol are the same amount, which is what lets
/// `debit(a, x); credit(b, x)` be proved conserving without knowing `x`.
pub type Symbol = u32;

/// A linear amount: a literal constant in minor units, plus a signed multiset of symbols.
///
/// This is deliberately the *weakest* arithmetic that suffices. It is closed under
/// addition, negation and multiplication by an integer constant, which covers every
/// operation the language permits on money — and it excludes multiplication of two money
/// values and division, which the language also excludes, because neither has a meaning in
/// minor units at a fixed scale.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Amount {
    pub constant: i128,
    pub symbols: BTreeMap<Symbol, i128>,
}

impl Amount {
    pub fn zero() -> Self {
        Amount::default()
    }
    pub fn constant(v: i128) -> Self {
        Amount {
            constant: v,
            symbols: BTreeMap::new(),
        }
    }
    pub fn symbol(s: Symbol) -> Self {
        let mut m = BTreeMap::new();
        m.insert(s, 1);
        Amount {
            constant: 0,
            symbols: m,
        }
    }
    pub fn is_zero(&self) -> bool {
        self.constant == 0 && self.symbols.values().all(|c| *c == 0)
    }
    /// Whether the checker can *decide* this amount. An amount mentioning a symbol whose
    /// coefficient does not cancel is undecided, not non-zero: the distinction matters,
    /// because reporting "does not conserve" for a program the checker merely cannot see
    /// through would be a false accusation.
    pub fn is_decided(&self) -> bool {
        self.symbols.values().all(|c| *c == 0)
    }
    pub fn add(&self, other: &Amount) -> Amount {
        let mut out = self.clone();
        out.constant += other.constant;
        for (s, c) in &other.symbols {
            *out.symbols.entry(*s).or_insert(0) += c;
        }
        out.symbols.retain(|_, c| *c != 0);
        out
    }
    pub fn neg(&self) -> Amount {
        Amount {
            constant: -self.constant,
            symbols: self.symbols.iter().map(|(s, c)| (*s, -c)).collect(),
        }
    }
    pub fn scale(&self, k: i128) -> Amount {
        if k == 0 {
            return Amount::zero();
        }
        Amount {
            constant: self.constant * k,
            symbols: self.symbols.iter().map(|(s, c)| (*s, c * k)).collect(),
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.symbols.is_empty() {
            return write!(f, "{}", self.constant);
        }
        let mut parts: Vec<String> = Vec::new();
        if self.constant != 0 {
            parts.push(self.constant.to_string());
        }
        for (s, c) in &self.symbols {
            parts.push(match c {
                1 => format!("x{s}"),
                -1 => format!("-x{s}"),
                _ => format!("{c}*x{s}"),
            });
        }
        write!(f, "{}", parts.join(" + "))
    }
}

/// How a row was accumulated. This is what decides whether a non-zero row is a **sound
/// accusation** or merely an unproven one, and getting it wrong is how a checker reports
/// correct programs as broken.
///
/// The distinction was found by an adversarial reading of this module against the
/// abstract-interpretation literature, and then reproduced: before this type existed, the
/// checker summed *both arms of an `if`* into one row and reported a conserving program as
/// creating money. See `docs/research/currency-row-analysis.md` §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Straight-line, abort-free. Every movement in the row definitely happens, so a
    /// non-zero decided entry is a **must**-violation and may be reported as one.
    StraightLine,
    /// A control-flow merge occurred. Entries that survived the join are agreed by every
    /// path, but the row as a whole is not a statement about a single execution, so a
    /// non-zero entry is a *may*-violation.
    Merged,
    /// A path can leave the transaction early. The accumulated row may describe a prefix
    /// that the runtime discards, so a non-zero entry says nothing at all.
    MayAbort,
}

impl Provenance {
    /// The join: any weakening on either side weakens the result.
    pub fn join(self, other: Provenance) -> Provenance {
        match (self, other) {
            (Provenance::MayAbort, _) | (_, Provenance::MayAbort) => Provenance::MayAbort,
            (Provenance::Merged, _) | (_, Provenance::Merged) => Provenance::Merged,
            _ => Provenance::StraightLine,
        }
    }
    // **Why there is no `supports_must_violation` here.**
    //
    // There was one, and it returned `true` unconditionally while `check_conservation`
    // branched on it — a soundness decision routed through a constant. Both the predicate
    // and the branch are gone; what remains is the reasoning that makes the unconditional
    // answer right, because the first version of this module got it wrong in both
    // directions in turn.
    //
    // The worry provenance was introduced to answer is: *can a row that came through a
    // control-flow merge be trusted as a statement about every execution?* With a
    // top-preserving join the answer is yes, and by construction. An entry only survives
    // `Row::join` if every arm agreed on it; where the arms disagree it is poisoned with a
    // fresh symbol and the verdict becomes `Undecided`. So a decided, non-zero entry after
    // a merge is one that *every* arm produced — which is precisely a must-violation.
    //
    // Gating `Violates` on `StraightLine` threw away real precision: a transaction losing
    // ten dollars down every path was downgraded to a warning. The abort case, the other
    // motivation, is handled at the source instead — a `txn` is sealed atomically, so a
    // path that leaves early commits nothing and is dropped rather than merged.
    //
    // `Provenance` is retained because it records *how* a row was built, which the
    // diagnostic uses to say "every path through this transaction" rather than "this
    // transaction", and because a future precision-preserving join — an affine hull in the
    // manner of Karr, rather than top-on-disagreement — would reintroduce exactly the
    // question this answered.

    /// How to describe the paths this row covers, in a diagnostic.
    pub fn describe(self) -> &'static str {
        match self {
            Provenance::StraightLine => "this transaction",
            Provenance::Merged => "every path through this transaction",
            Provenance::MayAbort => "every committing path through this transaction",
        }
    }
}

/// A currency row: the net effect of a fragment of program on each currency.
#[derive(Debug, Clone)]
pub struct Row {
    entries: BTreeMap<Cur, Amount>,
    /// Where each currency first entered the row, for the two-span diagnostic.
    origins: BTreeMap<Cur, Span>,
    /// What kind of statement this row is. See [`Provenance`].
    pub provenance: Provenance,
    /// Fresh-symbol counter for poisoning a disagreeing merge.
    poison: u32,
}

impl Default for Row {
    fn default() -> Self {
        Row {
            entries: BTreeMap::new(),
            origins: BTreeMap::new(),
            provenance: Provenance::StraightLine,
            poison: 0,
        }
    }
}

impl Row {
    pub fn new() -> Self {
        Row::default()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn currencies(&self) -> impl Iterator<Item = &Cur> {
        self.entries.keys()
    }
    pub fn get(&self, c: &Cur) -> Amount {
        self.entries.get(c).cloned().unwrap_or_default()
    }
    pub fn origin(&self, c: &Cur) -> Option<Span> {
        self.origins.get(c).copied()
    }
    /// Add a signed movement.
    pub fn movement(&mut self, cur: Cur, amount: Amount, span: Span) {
        self.origins.entry(cur.clone()).or_insert(span);
        let e = self.entries.entry(cur).or_default();
        *e = e.add(&amount);
    }
    /// Row addition: the net effect of two fragments **in sequence**. Not to be confused
    /// with [`Row::join`], which is the merge of two *alternative* paths.
    pub fn merge(&mut self, other: &Row) {
        self.provenance = self.provenance.join(other.provenance);
        self.poison = self.poison.max(other.poison);
        for (c, a) in &other.entries {
            self.origins
                .entry(c.clone())
                .or_insert_with(|| other.origins[c]);
            let e = self.entries.entry(c.clone()).or_default();
            *e = e.add(a);
        }
    }
    /// **The control-flow join.** Merge two rows that came from alternative paths.
    ///
    /// This is the operation whose absence made the checker unsound. The domain is a
    /// single affine form per currency, and on such a representation the only sound join
    /// is: *where the two sides agree, keep the value; where they disagree, go to top.*
    /// Going to top is expressed by poisoning the entry with a fresh symbol, so that
    /// [`Amount::is_decided`] reports false and the verdict becomes `Undecided` rather
    /// than an accusation.
    ///
    /// This is strictly weaker than Karr's affine-hull join, which on a *relational*
    /// domain could retain `net = x − y` even where the branches disagree on `x` and `y`
    /// separately. The weakening is the price of a one-expression-per-currency
    /// representation, and it is a weakening toward *silence*, never toward a false
    /// accusation.
    pub fn join(mut self, other: Row, at: Span) -> Row {
        let mut out = Row {
            entries: BTreeMap::new(),
            origins: BTreeMap::new(),
            provenance: self
                .provenance
                .join(other.provenance)
                .join(Provenance::Merged),
            poison: self.poison.max(other.poison) + 1,
        };
        let mut currencies: Vec<Cur> = self.entries.keys().cloned().collect();
        for c in other.entries.keys() {
            if !currencies.contains(c) {
                currencies.push(c.clone());
            }
        }
        for c in currencies {
            let (a, b) = (self.get(&c), other.get(&c));
            let span = self.origin(&c).or_else(|| other.origin(&c)).unwrap_or(at);
            out.origins.insert(c.clone(), span);
            if a == b {
                out.entries.insert(c, a);
            } else {
                // The branches disagree. Poison with a fresh symbol: the row is now
                // undecided in this currency, which is the honest statement.
                let mut poisoned = a;
                poisoned.symbols.insert(u32::MAX - out.poison, 1);
                out.entries.insert(c, poisoned);
            }
        }
        self.entries.clear();
        out
    }

    /// Mark that a path can leave the transaction early, so the accumulated row may
    /// describe a prefix the runtime discards.
    ///
    /// Without this, `txn { debit(a, 5); if !ok { abort } credit(b, 5) }` is reported as
    /// losing five dollars, when the ledger's own atomicity means the abort path commits
    /// nothing at all.
    pub fn mark_may_abort(&mut self) {
        self.provenance = self.provenance.join(Provenance::MayAbort);
    }

    /// **The loop rule.** A loop body whose net is zero contributes zero for any trip
    /// count; a body with a non-zero net contributes `n · body` for a symbolic `n`, which
    /// is a product of two symbolic values and therefore outside this domain's fragment.
    ///
    /// This asymmetry is not a limitation to apologise for. Because a row is a
    /// homomorphism from statement sequences into a free abelian group, "balanced per
    /// iteration implies balanced overall" needs no widening, no trip-count reasoning and
    /// no fixpoint — it is immediate from the algebra. It is also exactly the discipline a
    /// batch posting loop should follow anyway.
    pub fn iterate(mut self, at: Span) -> Row {
        let all_zero = self.entries.values().all(|a| a.is_zero());
        if all_zero {
            self.provenance = self.provenance.join(Provenance::Merged);
            return self;
        }
        let mut out = Row {
            entries: BTreeMap::new(),
            origins: self.origins.clone(),
            provenance: self.provenance.join(Provenance::Merged),
            poison: self.poison + 1,
        };
        for (c, a) in std::mem::take(&mut self.entries) {
            let mut scaled = a;
            // Multiply by an unknown trip count: a fresh symbol, hence undecided.
            scaled.symbols.insert(u32::MAX - out.poison, 1);
            out.origins.entry(c.clone()).or_insert(at);
            out.entries.insert(c, scaled);
        }
        out
    }

    /// Rewrite currency variables that have since been unified with something.
    pub fn substitute(&mut self, u: &Unifier) {
        let mut new_entries: BTreeMap<Cur, Amount> = BTreeMap::new();
        let mut new_origins: BTreeMap<Cur, Span> = BTreeMap::new();
        for (c, a) in std::mem::take(&mut self.entries) {
            let r = u.resolve(&c);
            let sp = self.origins.get(&c).copied().unwrap_or_default();
            new_origins.entry(r.clone()).or_insert(sp);
            let e = new_entries.entry(r).or_default();
            *e = e.add(&a);
        }
        self.entries = new_entries;
        self.origins = new_origins;
    }
}

/// Union-find over currency variables. Small, because currency positions in one function
/// are few; the algorithm is the standard one and its only subtlety is that a known
/// currency always wins over a variable, so a resolved root is never a variable when a
/// concrete currency is in the class.
#[derive(Debug, Default)]
pub struct Unifier {
    parent: Vec<Cur>,
    next: u32,
}

impl Unifier {
    pub fn new() -> Self {
        Unifier::default()
    }
    /// A fresh currency variable, for a generic `Money<C>`.
    pub fn fresh(&mut self) -> Cur {
        let v = self.next;
        self.next += 1;
        self.parent.push(Cur::Var(v));
        Cur::Var(v)
    }
    pub fn resolve(&self, c: &Cur) -> Cur {
        match c {
            Cur::Known(_) => c.clone(),
            Cur::Var(v) => {
                let p = self
                    .parent
                    .get(*v as usize)
                    .cloned()
                    .unwrap_or_else(|| c.clone());
                if &p == c {
                    c.clone()
                } else {
                    self.resolve(&p)
                }
            }
        }
    }
    /// Unify two currencies. Returns the conflict when they are distinct known
    /// currencies — which is the "cannot mismatch currencies" clause of the soundness
    /// theorem, decided here and nowhere else.
    pub fn unify(&mut self, a: &Cur, b: &Cur) -> Result<Cur, (String, String)> {
        let (ra, rb) = (self.resolve(a), self.resolve(b));
        match (&ra, &rb) {
            _ if ra == rb => Ok(ra),
            (Cur::Known(x), Cur::Known(y)) => Err((x.clone(), y.clone())),
            (Cur::Var(v), other) | (other, Cur::Var(v)) => {
                self.parent[*v as usize] = other.clone();
                Ok(other.clone())
            }
        }
    }
}

/// What a conservation check concluded.
///
/// Three verdicts, and the boundary between the second and third is the whole honesty of
/// the analysis. `Violates` is an accusation and must be a **must**-statement: every
/// execution of this transaction moves money that does not balance. `MayViolate` is the
/// same arithmetic reached along a path the checker cannot guarantee is taken.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// Every currency's entry is provably zero, on every path.
    Conserves,
    /// **A must-violation.** The body is straight-line and abort-free, every symbol
    /// cancelled, and the residue is a non-zero constant. This transaction cannot balance.
    Violates {
        currency: String,
        residue: Amount,
        span: Span,
    },
    /// The arithmetic does not balance, but the row was accumulated across a control-flow
    /// merge or a path that can abort, so this is an alarm rather than a proof. Reported
    /// as a warning with the reason, never as an error: a checker that accuses a program
    /// it cannot follow teaches its users to disable it.
    MayViolate {
        currency: String,
        residue: Amount,
        span: Span,
        why: Provenance,
    },
    /// The row mentions an amount the checker cannot see through. Not a violation —
    /// an honest "not proven", which the caller turns into a runtime obligation.
    Undecided {
        currency: String,
        residue: Amount,
        span: Span,
    },
}

/// The conservation judgement: is this row zero in every currency?
///
/// This is the static half of Contribution 1's corollary. The dynamic half — that eviction
/// and reconstruction cannot create or destroy money — is a property of the engine, proved
/// separately; together they say money is conserved both by what a program can express and
/// by what the runtime can do to the state a program produced.
///
/// # Soundness, stated exactly
///
/// * `Conserves` is **sound**: it is returned only when every currency's entry is
///   canonically zero, which given the transfer functions means every path's net is zero.
/// * `Violates` is **sound** only under `Provenance::StraightLine`, which is why the
///   provenance is consulted here rather than assumed. Under a merge or a possible abort
///   the same arithmetic yields `MayViolate`.
/// * `Undecided` is the honest remainder. With affine equality *guards* in the language it
///   is provably unavoidable — Müller-Olm and Seidl reduce Post's Correspondence Problem
///   to deciding whether an affine relation holds at a program point — so this third
///   verdict is forced by the problem, not conceded by the implementation.
pub fn check_conservation(row: &Row) -> Vec<Verdict> {
    let mut out = Vec::new();
    for (c, a) in &row.entries {
        let span = row.origins.get(c).copied().unwrap_or_default();
        let name = c.to_string();
        if a.is_zero() {
            continue;
        } else if a.is_decided() {
            // A decided, non-zero entry is a must-violation whatever the provenance, and
            // the `if` that used to stand here consulted a predicate that returned `true`
            // unconditionally. A soundness decision routed through a constant is not a
            // decision; it is the `_ => true` of GC-04 with a name on it. The reasoning
            // that makes the unconditional answer correct is on `Provenance` itself: the
            // join is top-preserving, so an entry that survives a merge is one *every* arm
            // produced, and an abort path is dropped at the source rather than merged.
            out.push(Verdict::Violates {
                currency: name,
                residue: a.clone(),
                span,
            });
        } else {
            out.push(Verdict::Undecided {
                currency: name,
                residue: a.clone(),
                span,
            });
        }
    }
    if out.is_empty() {
        out.push(Verdict::Conserves);
    }
    out
}

/// Turn a verdict into the diagnostic the user sees. The message names the residue and
/// the currency, and the caller attaches the `conserve per (..)` clause as a secondary
/// label — the two-span form the whole diagnostics design exists for.
pub fn diagnose(v: &Verdict, scale: u32) -> Option<Diagnostic> {
    diagnose_with(v, scale, Provenance::StraightLine)
}

/// As [`diagnose`], but able to say which paths the row covers.
pub fn diagnose_with(v: &Verdict, scale: u32, prov: Provenance) -> Option<Diagnostic> {
    match v {
        Verdict::Conserves => None,
        Verdict::Violates { currency, residue, span } => Some(
            Diagnostic::error("NL0300", format!("this transaction does not conserve `{currency}`"))
                .primary(
                    *span,
                    format!("net movement on {} is {}, which must be zero", prov.describe(), fmt_money(residue, scale)),
                )
                .note("`conserve per (txn, cur)` requires each currency's amounts to sum to zero independently")
                .note("a total of zero across currencies is not conservation: 10.00 usd and -10.00 eur cancel in no ledger")
                .suggest(
                    *span,
                    "// add the balancing posting",
                    format!("add a posting of {} in `{currency}`", fmt_money(&residue.neg(), scale)),
                    Applicability::HasPlaceholders,
                ),
        ),
        Verdict::MayViolate { currency, residue, span, why } => Some(
            Diagnostic::warning("NL0301", format!("this transaction may not conserve `{currency}`"))
                .primary(*span, format!("net movement on some path is {}", fmt_money(residue, scale)))
                .note(match why {
                    Provenance::Merged => "the paths through this transaction do not agree, so this is what one of them does; \
                                           the checker cannot tell which path runs",
                    Provenance::MayAbort => "a path can leave this transaction early, so the accumulated movement may be \
                                             a prefix the runtime discards rather than a transaction that commits",
                    Provenance::StraightLine => "unreachable",
                })
                .note("reported as a warning rather than an error because it is not a proof: a checker that accuses a \
                       program it cannot follow teaches its users to switch it off"),
        ),
        // An undecided row is deliberately *not* a diagnostic. The checker cannot see
        // through the amount, so it discharges the obligation to the runtime instead of
        // accusing a program that may well be correct.
        Verdict::Undecided { .. } => None,
    }
}

fn fmt_money(a: &Amount, scale: u32) -> String {
    if !a.is_decided() {
        return a.to_string();
    }
    let v = a.constant;
    if scale == 0 {
        return v.to_string();
    }
    let d = 10i128.pow(scale);
    let (sign, v) = if v < 0 { ("-", -v) } else { ("", v) };
    format!("{sign}{}.{:0width$}", v / d, v % d, width = scale as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usd() -> Cur {
        Cur::Known("usd".into())
    }
    fn eur() -> Cur {
        Cur::Known("eur".into())
    }
    fn sp(a: u32) -> Span {
        Span::new(a as usize, a as usize + 1)
    }

    #[test]
    fn a_balanced_transfer_conserves() {
        let mut r = Row::new();
        r.movement(usd(), Amount::constant(-1000), sp(1));
        r.movement(usd(), Amount::constant(1000), sp(2));
        assert_eq!(check_conservation(&r), vec![Verdict::Conserves]);
    }

    #[test]
    fn an_unbalanced_transfer_names_the_residue() {
        let mut r = Row::new();
        r.movement(usd(), Amount::constant(-1000), sp(1));
        r.movement(usd(), Amount::constant(500), sp(2));
        let v = check_conservation(&r);
        let Verdict::Violates {
            currency, residue, ..
        } = &v[0]
        else {
            panic!("{v:?}")
        };
        assert_eq!(currency, "usd");
        assert_eq!(residue.constant, -500);
        let d = diagnose(&v[0], 2).unwrap();
        assert!(d.labels[0].msg.contains("-5.00"), "{}", d.labels[0].msg);
    }

    #[test]
    fn conservation_is_per_currency_not_in_total() {
        // The failure this whole design exists to prevent: a transaction whose amounts sum
        // to zero *across* currencies, which is not conservation in any ledger.
        let mut r = Row::new();
        r.movement(usd(), Amount::constant(1000), sp(1));
        r.movement(eur(), Amount::constant(-1000), sp(2));
        let v = check_conservation(&r);
        assert_eq!(
            v.len(),
            2,
            "both currencies must be reported, not netted: {v:?}"
        );
        assert!(v.iter().all(|x| matches!(x, Verdict::Violates { .. })));
    }

    #[test]
    fn conservation_holds_for_an_unknown_amount() {
        // `debit(a, x); credit(b, x)` conserves for every x, and the checker sees it,
        // because the two occurrences are the same symbol.
        let x = 7;
        let mut r = Row::new();
        r.movement(usd(), Amount::symbol(x).neg(), sp(1));
        r.movement(usd(), Amount::symbol(x), sp(2));
        assert_eq!(check_conservation(&r), vec![Verdict::Conserves]);
    }

    #[test]
    fn an_opaque_amount_is_undecided_not_violating() {
        // The honest boundary: the checker must not accuse a program it merely cannot see
        // through. `debit(a, x)` alone is not proven non-conserving; it is unproven.
        let mut r = Row::new();
        r.movement(usd(), Amount::symbol(3), sp(1));
        let v = check_conservation(&r);
        assert!(matches!(v[0], Verdict::Undecided { .. }), "{v:?}");
        assert!(
            diagnose(&v[0], 2).is_none(),
            "an undecided row must not produce an error"
        );
    }

    #[test]
    fn distinct_known_currencies_do_not_unify() {
        let mut u = Unifier::new();
        assert!(
            u.unify(&usd(), &eur()).is_err(),
            "usd and eur must not unify"
        );
        assert!(u.unify(&usd(), &usd()).is_ok());
    }

    #[test]
    fn a_variable_takes_the_known_currency_of_its_class() {
        let mut u = Unifier::new();
        let c = u.fresh();
        assert_eq!(u.resolve(&c), c);
        u.unify(&c, &usd()).unwrap();
        assert_eq!(u.resolve(&c), usd());
        // ... and once fixed, it conflicts with a different currency.
        assert!(u.unify(&c, &eur()).is_err());
    }

    #[test]
    fn generic_money_conserves_for_every_currency() {
        // `fn move<C>(a, b, m: Money<C>)` type-checks once and conserves for all C. This
        // is what row variables buy over monomorphising by currency.
        let mut u = Unifier::new();
        let c = u.fresh();
        let x = 1;
        let mut r = Row::new();
        r.movement(c.clone(), Amount::symbol(x).neg(), sp(1));
        r.movement(c, Amount::symbol(x), sp(2));
        assert_eq!(check_conservation(&r), vec![Verdict::Conserves]);
    }

    #[test]
    fn unification_collapses_a_row_that_looked_split() {
        // Two variables that later unify must merge into one row entry, or a conserving
        // program would be reported as violating in two half-currencies.
        let mut u = Unifier::new();
        let (a, b) = (u.fresh(), u.fresh());
        let mut r = Row::new();
        r.movement(a.clone(), Amount::constant(-1000), sp(1));
        r.movement(b.clone(), Amount::constant(1000), sp(2));
        assert_eq!(
            check_conservation(&r).len(),
            2,
            "before unification, two entries"
        );
        u.unify(&a, &b).unwrap();
        r.substitute(&u);
        assert_eq!(check_conservation(&r), vec![Verdict::Conserves]);
    }

    #[test]
    fn an_fx_form_is_two_rows_each_zero() {
        // There is no single currency in which a cross-currency transaction sums to zero.
        // The `fx` form opens two rows and requires each to be zero independently, which
        // is the only construction under which cross-currency movement is conserving.
        let mut leg_usd = Row::new();
        leg_usd.movement(usd(), Amount::constant(-100_000), sp(1));
        leg_usd.movement(usd(), Amount::constant(100_000), sp(2));
        let mut leg_eur = Row::new();
        leg_eur.movement(eur(), Amount::constant(-92_000), sp(3));
        leg_eur.movement(eur(), Amount::constant(92_000), sp(4));
        assert_eq!(check_conservation(&leg_usd), vec![Verdict::Conserves]);
        assert_eq!(check_conservation(&leg_eur), vec![Verdict::Conserves]);

        // Merged into one row they still conserve — but only because each leg already did.
        // A "conversion" that debits usd and credits eur without balancing legs does not:
        let mut bad = Row::new();
        bad.movement(usd(), Amount::constant(-100_000), sp(5));
        bad.movement(eur(), Amount::constant(92_000), sp(6));
        assert_eq!(check_conservation(&bad).len(), 2);
    }

    #[test]
    fn scaling_by_an_integer_preserves_decidability() {
        let mut r = Row::new();
        let x = Amount::symbol(4);
        r.movement(usd(), x.scale(3).neg(), sp(1));
        r.movement(usd(), x.scale(1), sp(2));
        r.movement(usd(), x.scale(2), sp(3));
        assert_eq!(check_conservation(&r), vec![Verdict::Conserves]);
    }
}
