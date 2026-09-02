//! **Values, and the third truth value.**
//!
//! Until now the IR had no null. `Scalar` had integers, booleans, text, money and the
//! anchor, and the reference evaluator's rows were `Vec<i128>`. That was adequate for
//! every rewrite in the schedule catalogue, because none of them turns on the difference
//! between "absent" and "zero" — and it was exactly the gap that made `not in` unstatable.
//!
//! # Three absences, kept apart
//!
//! The keyword registry already says the language has three distinct notions of absence,
//! and the thesis's lattice of absence (§3.3) turns on keeping them apart:
//!
//! | Absence | Means | Lives in |
//! |---|---|---|
//! | **`null`** | The value is unknown *in the data* | this file |
//! | `Option::None` | The value is absent *in a program* | the type system |
//! | `Hole` / `Slot::Absent` | The value was **evicted** and is reconstructible | `nilestream-core::absence` |
//!
//! Collapsing any pair of these is a defect with a name. A `null` read as an evicted hole
//! would trigger an upquery that reconstructs the same null; an evicted hole read as a
//! `null` would report unknown data where the ledger has an answer. The `Err(_) => 0`
//! defect of §1.1.1 is the third case: an absence read as a *number*.
//!
//! # Kleene's three-valued logic, and why the filter rule is what it is
//!
//! SQL's comparisons are three-valued: `x = NULL` is neither true nor false but unknown,
//! and unknown propagates through `and`, `or` and `not` by Kleene's tables. A `where`
//! clause keeps a row only when the predicate is **true** — not "not false" — and that one
//! asymmetry is the whole reason `not in` behaves the way it does:
//!
//! ```text
//! x IN (1, 2, NULL)       -- true if x ∈ {1,2}, else UNKNOWN, never false
//! x NOT IN (1, 2, NULL)   -- false if x ∈ {1,2}, else UNKNOWN, never true
//! ```
//!
//! So `not in` against a subquery containing a single null returns **no rows at all**,
//! whatever the left side holds. That is not a corner case invented for a test: it is the
//! single most frequently mis-implemented rule in query rewriting, and the roadmap makes
//! one wrong case a kill criterion. [`Tri`] exists so the rule is written once.

use std::fmt;

/// A value in the reference semantics.
///
/// `Int` carries integers, booleans (0/1), money in minor units and interned text alike:
/// the reference semantics is about *structure*, and giving each surface type its own
/// variant here would multiply the cases in every operator without changing any rewrite's
/// correctness. `Null` is the one distinction that changes answers, so it is the one
/// distinction this type makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    /// Unknown *in the data*. Not an evicted hole, and not `Option::None`.
    Null,
    Int(i128),
}

impl Value {
    pub fn is_null(self) -> bool {
        self == Value::Null
    }
    /// The integer, or `None` if null. Callers must decide what null means for them —
    /// there is deliberately no `unwrap_or(0)`, because that is the §1.1.1 defect written
    /// as a convenience method.
    pub fn int(self) -> Option<i128> {
        match self {
            Value::Int(i) => Some(i),
            Value::Null => None,
        }
    }
}

impl From<i128> for Value {
    fn from(i: i128) -> Self {
        Value::Int(i)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Int(b as i128)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Int(i) => write!(f, "{i}"),
        }
    }
}

/// A three-valued truth value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tri {
    False,
    Unknown,
    True,
}

impl Tri {
    pub fn of(b: bool) -> Tri {
        if b {
            Tri::True
        } else {
            Tri::False
        }
    }

    /// **The filter rule.** A row survives a predicate only when it is definitely true.
    ///
    /// Unknown is not "maybe keep": it is discard. This method is the single place that
    /// decision is written, and every operator that filters calls it, so the two cannot
    /// drift apart.
    pub fn keeps(self) -> bool {
        self == Tri::True
    }

    /// Kleene's `not`: unknown negates to unknown.
    pub fn not(self) -> Tri {
        match self {
            Tri::True => Tri::False,
            Tri::False => Tri::True,
            Tri::Unknown => Tri::Unknown,
        }
    }

    /// Kleene's `and`: false wins over unknown, because false ∧ anything is false.
    pub fn and(self, other: Tri) -> Tri {
        self.min(other)
    }

    /// Kleene's `or`: true wins over unknown, symmetrically.
    pub fn or(self, other: Tri) -> Tri {
        self.max(other)
    }

    /// The definite boolean an `is null` test produces. Never unknown: asking whether a
    /// value *is* null is a question about the value, not about the world.
    pub fn definite(self) -> Value {
        Value::Int((self == Tri::True) as i128)
    }
}

/// Comparison under three-valued logic: any null operand yields unknown.
pub fn compare(a: Value, b: Value, f: impl Fn(i128, i128) -> bool) -> Tri {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Tri::of(f(x, y)),
        _ => Tri::Unknown,
    }
}

/// Arithmetic under nulls: null propagates. `null + 1` is null, not 1.
pub fn arith(a: Value, b: Value, f: impl Fn(i128, i128) -> i128) -> Value {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Value::Int(f(x, y)),
        _ => Value::Null,
    }
}

/// Read a value as a truth value: null is unknown, zero is false, anything else true.
pub fn truth(v: Value) -> Tri {
    match v {
        Value::Null => Tri::Unknown,
        Value::Int(0) => Tri::False,
        Value::Int(_) => Tri::True,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filter_keeps_only_definite_truth() {
        // The asymmetry the whole `not in` story rests on. Unknown is discard, not "maybe".
        assert!(Tri::True.keeps());
        assert!(!Tri::False.keeps());
        assert!(
            !Tri::Unknown.keeps(),
            "unknown must not survive a predicate"
        );
    }

    #[test]
    fn kleene_and_or_match_the_published_tables() {
        use Tri::*;
        // and: false absorbs, true is the identity, unknown sits between.
        assert_eq!(False.and(Unknown), False);
        assert_eq!(True.and(Unknown), Unknown);
        assert_eq!(Unknown.and(Unknown), Unknown);
        // or: true absorbs.
        assert_eq!(True.or(Unknown), True);
        assert_eq!(False.or(Unknown), Unknown);
        // not is symmetric about unknown.
        assert_eq!(Unknown.not(), Unknown);
        assert_eq!(True.not(), False);
    }

    #[test]
    fn a_null_comparison_is_unknown_not_false() {
        // The mistake that makes `not in` wrong: treating `x = null` as false would make
        // `x != null` true, and the whole three-valued story collapses into two.
        assert_eq!(
            compare(Value::Int(1), Value::Null, |a, b| a == b),
            Tri::Unknown
        );
        assert_eq!(
            compare(Value::Null, Value::Null, |a, b| a == b),
            Tri::Unknown
        );
        assert_eq!(
            compare(Value::Int(1), Value::Int(1), |a, b| a == b),
            Tri::True
        );
        assert_ne!(
            compare(Value::Int(1), Value::Null, |a, b| a == b),
            Tri::False
        );
    }

    #[test]
    fn null_propagates_through_arithmetic() {
        assert_eq!(arith(Value::Null, Value::Int(1), |a, b| a + b), Value::Null);
        assert_eq!(
            arith(Value::Int(2), Value::Int(3), |a, b| a + b),
            Value::Int(5)
        );
    }

    #[test]
    fn there_is_no_unwrap_or_zero() {
        // Not a behavioural test — a design one. `Value::int` returns an `Option` so that
        // every caller must decide what a null means for it. A helper that returned 0
        // would be the §1.1.1 defect offered as an ergonomic.
        assert_eq!(Value::Null.int(), None);
        assert_eq!(Value::Int(7).int(), Some(7));
    }

    #[test]
    fn is_null_is_definite_even_about_a_null() {
        assert_eq!(truth(Value::Null), Tri::Unknown);
        assert_eq!(Tri::of(Value::Null.is_null()).definite(), Value::Int(1));
        assert_eq!(Tri::of(Value::Int(0).is_null()).definite(), Value::Int(0));
    }
}
