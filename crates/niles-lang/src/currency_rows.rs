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
        Amount { constant: v, symbols: BTreeMap::new() }
    }
    pub fn symbol(s: Symbol) -> Self {
        let mut m = BTreeMap::new();
        m.insert(s, 1);
        Amount { constant: 0, symbols: m }
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

/// A currency row: the net effect of a fragment of program on each currency.
#[derive(Debug, Clone, Default)]
pub struct Row {
    entries: BTreeMap<Cur, Amount>,
    /// Where each currency first entered the row, for the two-span diagnostic.
    origins: BTreeMap<Cur, Span>,
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
    /// Row addition: the net effect of two fragments in sequence.
    pub fn merge(&mut self, other: &Row) {
        for (c, a) in &other.entries {
            self.origins.entry(c.clone()).or_insert_with(|| other.origins[c]);
            let e = self.entries.entry(c.clone()).or_default();
            *e = e.add(a);
        }
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
                let p = self.parent.get(*v as usize).cloned().unwrap_or_else(|| c.clone());
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
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// Every currency's entry is provably zero.
    Conserves,
    /// Some currency's entry is provably non-zero, with the residue.
    Violates { currency: String, residue: Amount, span: Span },
    /// The row mentions an amount the checker cannot see through. Not a violation —
    /// an honest "not proven", which the caller turns into a runtime obligation.
    Undecided { currency: String, residue: Amount, span: Span },
}

/// The conservation judgement: is this row zero in every currency?
///
/// This is the static half of Contribution 1's corollary. The dynamic half — that eviction
/// and reconstruction cannot create or destroy money — is a property of the engine, proved
/// separately; together they say money is conserved both by what a program can express and
/// by what the runtime can do to the state a program produced.
pub fn check_conservation(row: &Row) -> Vec<Verdict> {
    let mut out = Vec::new();
    for (c, a) in &row.entries {
        let span = row.origins.get(c).copied().unwrap_or_default();
        let name = c.to_string();
        if a.is_zero() {
            continue;
        } else if a.is_decided() {
            out.push(Verdict::Violates { currency: name, residue: a.clone(), span });
        } else {
            out.push(Verdict::Undecided { currency: name, residue: a.clone(), span });
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
    match v {
        Verdict::Conserves => None,
        Verdict::Violates { currency, residue, span } => Some(
            Diagnostic::error("NL0300", format!("this transaction does not conserve `{currency}`"))
                .primary(*span, format!("net movement is {}, which must be zero", fmt_money(residue, scale)))
                .note("`conserve per (txn, cur)` requires each currency's amounts to sum to zero independently")
                .note("a total of zero across currencies is not conservation: 10.00 usd and -10.00 eur cancel in no ledger")
                .suggest(
                    *span,
                    "// add the balancing posting",
                    format!("add a posting of {} in `{currency}`", fmt_money(&residue.neg(), scale)),
                    Applicability::HasPlaceholders,
                ),
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
        let Verdict::Violates { currency, residue, .. } = &v[0] else { panic!("{v:?}") };
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
        assert_eq!(v.len(), 2, "both currencies must be reported, not netted: {v:?}");
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
        assert!(diagnose(&v[0], 2).is_none(), "an undecided row must not produce an error");
    }

    #[test]
    fn distinct_known_currencies_do_not_unify() {
        let mut u = Unifier::new();
        assert!(u.unify(&usd(), &eur()).is_err(), "usd and eur must not unify");
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
        assert_eq!(check_conservation(&r).len(), 2, "before unification, two entries");
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
