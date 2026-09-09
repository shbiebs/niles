//! **One integer arithmetic, for every evaluator in this workspace.**
//!
//! Before this module there were three semantics for the same five operators, and the
//! difference reached the wire.
//!
//! | | `7 / 0` | `MAX + 1` (release) | `MAX + 1` (debug) | `MIN / -1` |
//! |---|---|---|---|---|
//! | the reference evaluator (`niles_ir::eval`) | `0` | wraps | panics | panics |
//! | the interpreter (`niles-interp`) | refuses | refuses | refuses | panics |
//! | what a ledger may do | — | — | — | — |
//!
//! The first column is the serious one. `eval::eval_scalar` answered `Value::Int(0)` for a
//! division by zero, and the served daemon runs that evaluator for every operator above the
//! keyed fold — so a query whose `having` clause or projection divided by zero was answered
//! **with a row containing a zero**, over the wire, to a client that had no way to know the
//! division had not happened. `0` is a plausible balance. The two rewrite corpora
//! (`sql_golden`, the unnesting suite) were certified against that behaviour, so the defaulting
//! was load-bearing in the tests as well.
//!
//! The second and third columns are the profile split: the same expression answers one thing in
//! `cargo test` and another in a release binary. A benchmark and a test suite that disagree
//! about arithmetic are two systems.
//!
//! # The rule
//!
//! Every integer operation in this workspace goes through this module, and every one of them
//! either returns a value that is arithmetically correct or returns an error. There is no third
//! outcome: no wrap, no saturate, no default. A caller that wants a total operation has to say
//! what it wants done with the error, at its own call site, where a reader can see it.
//!
//! # Why not wrapping, and why not saturating
//!
//! Both were considered and both are refused for the same reason. This system's central claim
//! is that money is conserved — that a sum over a set of postings equals the sum over any
//! partition of it. Wrapping breaks that at `i128::MAX` and saturating breaks it earlier and
//! more quietly, and in both cases the *conservation checker* would go on passing, because the
//! wrong total is a total. An error stops the query; a wrapped value becomes a balance.
//!
//! `i128::MIN / -1` is the case that catches an implementation using `checked_div` for the
//! zero divisor only: the quotient `170141183460469231731687303715884105728` is one past
//! `i128::MAX`, so the division overflows on a divisor that is neither zero nor unusual.
//! `i128::MIN % -1` is the same shape and is `0` mathematically, but Rust's `%` panics on it,
//! so it is refused here rather than special-cased — a remainder that is right for one input
//! and a panic for its neighbour is worse than a refusal for both.

/// What went wrong, and with which operator.
///
/// The operands travel with the error because the diagnostic a client receives is the only
/// place they exist: by the time a refusal reaches the wire the row that produced it is gone,
/// and "the query overflowed somewhere" is not a message anyone can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithError {
    /// `x / 0` or `x % 0`.
    DivideByZero { op: &'static str, lhs: i128 },
    /// The true result is outside `i128`.
    Overflow {
        op: &'static str,
        lhs: i128,
        rhs: i128,
    },
}

impl ArithError {
    /// The operator that failed, as it is written.
    pub fn op(&self) -> &'static str {
        match self {
            ArithError::DivideByZero { op, .. } | ArithError::Overflow { op, .. } => op,
        }
    }
}

impl std::fmt::Display for ArithError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithError::DivideByZero { op, lhs } => {
                write!(f, "{lhs} {op} 0 is not a number")
            }
            ArithError::Overflow { op, lhs, rhs } => {
                write!(f, "{lhs} {op} {rhs} does not fit in a 128-bit integer")
            }
        }
    }
}

impl std::error::Error for ArithError {}

pub fn add(x: i128, y: i128) -> Result<i128, ArithError> {
    x.checked_add(y).ok_or(ArithError::Overflow {
        op: "+",
        lhs: x,
        rhs: y,
    })
}

pub fn sub(x: i128, y: i128) -> Result<i128, ArithError> {
    x.checked_sub(y).ok_or(ArithError::Overflow {
        op: "-",
        lhs: x,
        rhs: y,
    })
}

pub fn mul(x: i128, y: i128) -> Result<i128, ArithError> {
    x.checked_mul(y).ok_or(ArithError::Overflow {
        op: "*",
        lhs: x,
        rhs: y,
    })
}

/// Integer division. **Two distinct failures**, kept apart: a zero divisor is a question with
/// no answer, and `MIN / -1` is a question whose answer does not fit. A caller that reported
/// them as one would tell a client to check its divisor when the divisor was `-1`.
pub fn div(x: i128, y: i128) -> Result<i128, ArithError> {
    if y == 0 {
        return Err(ArithError::DivideByZero { op: "/", lhs: x });
    }
    x.checked_div(y).ok_or(ArithError::Overflow {
        op: "/",
        lhs: x,
        rhs: y,
    })
}

pub fn rem(x: i128, y: i128) -> Result<i128, ArithError> {
    if y == 0 {
        return Err(ArithError::DivideByZero { op: "%", lhs: x });
    }
    x.checked_rem(y).ok_or(ArithError::Overflow {
        op: "%",
        lhs: x,
        rhs: y,
    })
}

/// Unary negation. `-i128::MIN` has no representation, which is the only way this fails.
pub fn neg(x: i128) -> Result<i128, ArithError> {
    x.checked_neg().ok_or(ArithError::Overflow {
        op: "unary -",
        lhs: x,
        rhs: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn division_by_zero_is_refused_and_not_defaulted() {
        assert_eq!(div(7, 0), Err(ArithError::DivideByZero { op: "/", lhs: 7 }));
        assert_eq!(rem(7, 0), Err(ArithError::DivideByZero { op: "%", lhs: 7 }));
        // The value that used to come back. It is a plausible balance, which is the point.
        assert_ne!(div(7, 0), Ok(0));
    }

    #[test]
    fn min_divided_by_minus_one_is_an_overflow_and_not_a_divide_by_zero() {
        // The case a `checked_div` for the zero divisor alone would miss: the divisor is
        // -1, the quotient is one past i128::MAX, and Rust's `/` panics rather than wrapping.
        assert_eq!(
            div(i128::MIN, -1),
            Err(ArithError::Overflow {
                op: "/",
                lhs: i128::MIN,
                rhs: -1
            })
        );
        assert_eq!(
            rem(i128::MIN, -1),
            Err(ArithError::Overflow {
                op: "%",
                lhs: i128::MIN,
                rhs: -1
            })
        );
        assert_eq!(
            neg(i128::MIN),
            Err(ArithError::Overflow {
                op: "unary -",
                lhs: i128::MIN,
                rhs: 0
            })
        );
    }

    #[test]
    fn the_neighbours_of_the_overflowing_cases_are_ordinary() {
        assert_eq!(div(i128::MIN, 1), Ok(i128::MIN));
        assert_eq!(div(i128::MAX, -1), Ok(-i128::MAX));
        assert_eq!(div(-1, i128::MIN), Ok(0));
        assert_eq!(rem(i128::MIN, 1), Ok(0));
        assert_eq!(rem(i128::MAX, -1), Ok(0));
        assert_eq!(neg(i128::MIN + 1), Ok(i128::MAX));
    }

    #[test]
    fn every_overflow_is_an_error_in_both_profiles() {
        // Written as assertions rather than as `#[should_panic]`, because the defect being
        // guarded is precisely that the release profile did not panic: a test that asserted a
        // panic would have passed in debug and told nobody what release did.
        assert!(add(i128::MAX, 1).is_err());
        assert!(sub(i128::MIN, 1).is_err());
        assert!(mul(i128::MAX, 2).is_err());
        assert!(add(i128::MIN, -1).is_err());
    }

    #[test]
    fn ordinary_arithmetic_is_unchanged() {
        assert_eq!(add(2, 3), Ok(5));
        assert_eq!(sub(2, 3), Ok(-1));
        assert_eq!(mul(-4, 5), Ok(-20));
        assert_eq!(div(7, 2), Ok(3));
        assert_eq!(div(-7, 2), Ok(-3));
        assert_eq!(rem(7, 2), Ok(1));
        assert_eq!(rem(-7, 2), Ok(-1));
        assert_eq!(neg(5), Ok(-5));
    }
}
