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
/// `Int` carries integers, booleans (0/1), dates, currency codes, and money whose currency
/// the circuit does not know. Until cycle 15 it carried text and every money value too, and
/// two defects came from that: every string literal evaluated as `0` (E30's F11), and a
/// money literal's currency was dropped, so `having sum(amt) < 0.00 eur` over usd rows
/// compared the numbers and answered (F13). The author's decisions 6 and 8 of 2026-09-30,
/// built in C15-05b, give each its own variant:
///
/// * **`Text`**, an interned string: equality by identity, ordering and `like` by content.
/// * **`Money`**, an amount in minor units *with its currency's catalog code*. Two money
///   values meet in an operator only in one currency; `Money` against `Int` is allowed,
///   because the integer carries no currency (`amt < 0`).
///
/// `Null` is still the one distinction every operator must respect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Value {
    /// Unknown *in the data*. Not an evicted hole, and not `Option::None`.
    Null,
    Int(i128),
    /// Minor units, and the currency's catalog code.
    Money {
        minor: i128,
        currency: u32,
    },
    Text(TextId),
}

/// An interned string. See [`Value::text`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextId(u32);

/// **The text interner.** Append-only and process-wide, so `Value` stays `Copy` and two
/// equal strings are one id. Ordering is by content (below), never by id, because ids depend
/// on the order strings were first seen, and an answer's order must not. It never shrinks:
/// every distinct string a process sees is held until it exits, a stated cost.
fn interner() -> &'static std::sync::RwLock<Interner> {
    static I: std::sync::OnceLock<std::sync::RwLock<Interner>> = std::sync::OnceLock::new();
    I.get_or_init(|| std::sync::RwLock::new(Interner::default()))
}

#[derive(Default)]
struct Interner {
    strings: Vec<std::sync::Arc<str>>,
    ids: std::collections::HashMap<std::sync::Arc<str>, u32>,
}

impl Value {
    pub fn is_null(self) -> bool {
        self == Value::Null
    }
    /// The number, or `None` if null or text. Money gives its minor units: the number is
    /// the same number it always was, and a caller that needs the currency asks
    /// [`Value::currency`]. Callers must decide what null means for them — there is
    /// deliberately no `unwrap_or(0)`, because that is the §1.1.1 defect written as a
    /// convenience method.
    pub fn int(self) -> Option<i128> {
        match self {
            Value::Int(i) | Value::Money { minor: i, .. } => Some(i),
            Value::Null | Value::Text(_) => None,
        }
    }
    /// The currency code of a money value.
    pub fn currency(self) -> Option<u32> {
        match self {
            Value::Money { currency, .. } => Some(currency),
            _ => None,
        }
    }
    /// The value of a string: its interned id.
    pub fn text(s: &str) -> Value {
        if let Some(id) = interner().read().expect("interner").ids.get(s) {
            return Value::Text(TextId(*id));
        }
        let mut w = interner().write().expect("interner");
        if let Some(id) = w.ids.get(s) {
            return Value::Text(TextId(*id));
        }
        let arc: std::sync::Arc<str> = std::sync::Arc::from(s);
        let id = w.strings.len() as u32;
        w.strings.push(arc.clone());
        w.ids.insert(arc, id);
        Value::Text(TextId(id))
    }
    /// The string a text value holds.
    pub fn as_text(self) -> Option<std::sync::Arc<str>> {
        match self {
            Value::Text(TextId(i)) => interner()
                .read()
                .expect("interner")
                .strings
                .get(i as usize)
                .cloned(),
            _ => None,
        }
    }
    fn rank(self) -> u8 {
        match self {
            Value::Null => 0,
            Value::Int(_) => 1,
            Value::Money { .. } => 2,
            Value::Text(_) => 3,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A total order for keys and sorted answers: by variant, then by value, and text **by
/// content**. (Comparing a usd with an eur is refused in an operator; ordering them as keys
/// is not a comparison of amounts, only a place in a map.)
impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (
                Value::Money {
                    minor: a,
                    currency: ca,
                },
                Value::Money {
                    minor: b,
                    currency: cb,
                },
            ) => ca.cmp(cb).then(a.cmp(b)),
            (Value::Text(a), Value::Text(b)) if a == b => Ordering::Equal,
            (Value::Text(_), Value::Text(_)) => self.as_text().cmp(&other.as_text()),
            (a, b) => a.rank().cmp(&b.rank()),
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
            // Money renders as its minor units, as it did when it was an `Int`: the wire and
            // every answer file keep the number they always had.
            Value::Int(i) | Value::Money { minor: i, .. } => write!(f, "{i}"),
            Value::Text(_) => write!(f, "{}", self.as_text().as_deref().unwrap_or("")),
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
///
/// Numbers only: `Int` with `Int`, and `Money` with `Int` (the integer carries no currency).
/// Two money values and two texts are compared by [`compare_values`], which can refuse.
pub fn compare(a: Value, b: Value, f: impl Fn(i128, i128) -> bool) -> Tri {
    match (a, b) {
        (Value::Int(x), Value::Int(y))
        | (Value::Money { minor: x, .. }, Value::Int(y))
        | (Value::Int(x), Value::Money { minor: y, .. }) => Tri::of(f(x, y)),
        (
            Value::Money {
                minor: x,
                currency: cx,
            },
            Value::Money {
                minor: y,
                currency: cy,
            },
        ) if cx == cy => Tri::of(f(x, y)),
        _ => Tri::Unknown,
    }
}

/// Why two values cannot meet in an operator (cycle 15, decisions 6 and 8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mismatch {
    /// Two money values in different currencies, by catalog code.
    Currencies(u32, u32),
    /// Two values of kinds no operator relates (text with a number).
    Kinds(&'static str, &'static str),
}

impl Value {
    pub fn kind(self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Int(_) => "integer",
            Value::Money { .. } => "money",
            Value::Text(_) => "text",
        }
    }
}

/// A comparison that **refuses** what [`compare`] cannot answer: money in two currencies,
/// and text against a number. Text compares by content. Null is still unknown.
pub fn compare_values(
    a: Value,
    b: Value,
    f: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Tri, Mismatch> {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => Ok(Tri::Unknown),
        (Value::Money { currency: x, .. }, Value::Money { currency: y, .. }) if x != y => {
            Err(Mismatch::Currencies(x, y))
        }
        (Value::Text(_), Value::Text(_)) => Ok(Tri::of(f(a.as_text().cmp(&b.as_text())))),
        (Value::Text(_), other) | (other, Value::Text(_)) => {
            Err(Mismatch::Kinds("text", other.kind()))
        }
        (x, y) => {
            let (p, q) = (x.int().expect("a number"), y.int().expect("a number"));
            Ok(Tri::of(f(p.cmp(&q))))
        }
    }
}

/// Arithmetic under nulls: null propagates. `null + 1` is null, not 1.
///
/// **`f` is fallible, and that is the whole change.** The previous signature took
/// `Fn(i128, i128) -> i128`, so every caller had to produce *some* `i128` for every pair of
/// operands — and the only ways to do that are to wrap, to saturate, or to invent a value.
/// `eval.rs` invented one: `x / 0` was `0`, and the served daemon answered a row with it.
/// A total signature cannot express a refusal, so the refusal has to be in the type.
///
/// Null still propagates before `f` is consulted: `null / 0` is null, not an arithmetic
/// error, because there is no division to fail.
pub fn arith(
    a: Value,
    b: Value,
    f: impl Fn(i128, i128) -> Result<i128, crate::arith::ArithError>,
) -> Result<Value, crate::arith::ArithError> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => f(x, y).map(Value::Int),
        _ => Ok(Value::Null),
    }
}

/// Arithmetic that knows money (cycle 15, decision 8). `+` and `-` of two money values need
/// one currency and keep it; money scaled by an integer (`*`, `/`, `%` with an `Int`) keeps
/// its currency; `+`/`-` of money and a bare integer is refused, because the integer is not
/// an amount of anything; text is never an operand. Null still propagates first.
pub fn arith_values(
    a: Value,
    b: Value,
    additive: bool,
    f: impl Fn(i128, i128) -> Result<i128, crate::arith::ArithError>,
) -> Result<Result<Value, crate::arith::ArithError>, Mismatch> {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => Ok(Ok(Value::Null)),
        (Value::Text(_), o) | (o, Value::Text(_)) => Err(Mismatch::Kinds("text", o.kind())),
        (Value::Int(x), Value::Int(y)) => Ok(f(x, y).map(Value::Int)),
        (
            Value::Money {
                minor: x,
                currency: cx,
            },
            Value::Money {
                minor: y,
                currency: cy,
            },
        ) => {
            if cx != cy {
                return Err(Mismatch::Currencies(cx, cy));
            }
            if additive {
                Ok(f(x, y).map(|m| Value::Money {
                    minor: m,
                    currency: cx,
                }))
            } else {
                // money × money is not money; the product of two amounts is a number.
                Ok(f(x, y).map(Value::Int))
            }
        }
        (Value::Money { minor: x, currency }, Value::Int(y))
        | (Value::Int(y), Value::Money { minor: x, currency }) => {
            if additive {
                Err(Mismatch::Kinds("money", "integer"))
            } else {
                let r = if matches!(a, Value::Money { .. }) {
                    f(x, y)
                } else {
                    f(y, x)
                };
                Ok(r.map(|m| Value::Money { minor: m, currency }))
            }
        }
    }
}

/// Read a value as a truth value: null is unknown, zero is false, anything else true.
pub fn truth(v: Value) -> Tri {
    match v {
        Value::Null => Tri::Unknown,
        Value::Int(0) => Tri::False,
        Value::Int(_) => Tri::True,
        // Not truth values. A predicate that is an amount or a string is a lowering defect;
        // reading it as unknown discards the row rather than inventing a verdict.
        Value::Money { .. } | Value::Text(_) => Tri::Unknown,
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
        assert_eq!(
            arith(Value::Null, Value::Int(1), crate::arith::add),
            Ok(Value::Null)
        );
        assert_eq!(
            arith(Value::Int(2), Value::Int(3), crate::arith::add),
            Ok(Value::Int(5))
        );
        // `null / 0` is null and not an arithmetic error: null propagates before the
        // operation is attempted, so there is no division to fail.
        assert_eq!(
            arith(Value::Null, Value::Int(0), crate::arith::div),
            Ok(Value::Null)
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

// ===================== instants =====================

/// The canonical reading of a date literal: **days since 1970-01-01**, proleptic Gregorian.
///
/// One function, because there were two places that needed it and a system in which
/// `where p.value_date >= v@2026-08-01` and `.valid_at(v@2026-08-01)` disagreed about what
/// that date *is* would be a system whose bitemporal answers depend on which surface asked.
///
/// `None` on anything that is not exactly `YYYY-MM-DD`, and the caller reports it. There is
/// no "best effort" reading of a date: a literal parsed leniently into the wrong day is a
/// worse outcome than a compile error, and an as-of read is where that would first be
/// noticed — months later, in a reconciliation.
///
/// No clock is consulted, which is the property the IR verifier's IR013 rule is about: a
/// view boundary must be a value, so that an evicted entry reconstructs to the number it
/// held rather than to the number today would give.
pub fn days_since_epoch(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let num = |s: &str| -> Option<i64> {
        if s.bytes().all(|c| c.is_ascii_digit()) {
            s.parse().ok()
        } else {
            None
        }
    };
    let (y, m, d) = (num(&text[0..4])?, num(&text[5..7])?, num(&text[8..10])?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    if d > days_in_month(y, m) {
        return None;
    }
    // Howard Hinnant's `days_from_civil`: exact, branch-free of any calendar table, and
    // correct for the whole proleptic Gregorian range rather than for a window around now.
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (m + 9) % 12; // March = 0
    let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    Some(era * 146_097 + doe - 719_468)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        _ => 0,
    }
}

#[cfg(test)]
mod instant_tests {
    use super::*;

    #[test]
    fn the_epoch_itself_is_day_zero_and_the_arithmetic_is_exact() {
        assert_eq!(days_since_epoch("1970-01-01"), Some(0));
        assert_eq!(days_since_epoch("1970-01-02"), Some(1));
        assert_eq!(days_since_epoch("1969-12-31"), Some(-1));
        // A leap day, and the day after it, on both sides of the century rule.
        assert_eq!(days_since_epoch("2000-02-29"), Some(11_016));
        assert_eq!(days_since_epoch("2000-03-01"), Some(11_017));
        assert_eq!(days_since_epoch("2026-08-01"), Some(20_666));
        // Consecutive days differ by one, across a month, a year and a leap year.
        for (a, b) in [
            ("2026-01-31", "2026-02-01"),
            ("2026-12-31", "2027-01-01"),
            ("2024-02-28", "2024-02-29"),
            ("2024-02-29", "2024-03-01"),
        ] {
            assert_eq!(
                days_since_epoch(b).unwrap() - days_since_epoch(a).unwrap(),
                1,
                "{a} -> {b}"
            );
        }
    }

    #[test]
    fn a_date_that_is_not_a_date_is_refused_rather_than_read_leniently() {
        // Every one of these has a "reasonable" reading, and every reasonable reading is a
        // different day from the one written. A literal parsed into the wrong day is worse
        // than a compile error, because nothing downstream can tell.
        for bad in [
            "2026-02-30", // no such day
            "2023-02-29", // not a leap year
            "1900-02-29", // the century rule
            "2026-13-01", // no such month
            "2026-00-10", // no such month
            "2026-08-00", // no such day
            "2026-8-01",  // not zero-padded
            "26-08-01",   // two-digit year
            "2026/08/01", // wrong separator
            "2026-08-01T00:00:00Z",
            "",
            "today",
        ] {
            assert_eq!(days_since_epoch(bad), None, "`{bad}` must be refused");
        }
    }

    /// **Money meets money only in one currency; text orders by content** (cycle 15,
    /// C15-05b; decisions 6 and 8).
    #[test]
    fn money_and_text_are_compared_by_their_own_rules() {
        use std::cmp::Ordering;
        let usd = |m| Value::Money {
            minor: m,
            currency: 0,
        };
        let eur = |m| Value::Money {
            minor: m,
            currency: 1,
        };
        assert_eq!(
            compare_values(usd(1), usd(2), Ordering::is_lt),
            Ok(Tri::True)
        );
        assert_eq!(
            compare_values(usd(1), eur(2), Ordering::is_lt),
            Err(Mismatch::Currencies(0, 1))
        );
        // An integer carries no currency: `amt < 0`.
        assert_eq!(
            compare_values(usd(-1), Value::Int(0), Ordering::is_lt),
            Ok(Tri::True)
        );
        // Text by content, whatever order the strings were interned in.
        let (b, a) = (Value::text("b-later"), Value::text("a-earlier"));
        assert_eq!(compare_values(a, b, Ordering::is_lt), Ok(Tri::True));
        assert!(a < b, "keys order by content too");
        assert_eq!(
            compare_values(a, Value::Int(1), Ordering::is_eq),
            Err(Mismatch::Kinds("text", "integer"))
        );
        assert_eq!(Value::text("a-earlier"), a, "one string is one value");
        assert_eq!(
            compare_values(Value::Null, eur(1), Ordering::is_eq),
            Ok(Tri::Unknown)
        );
    }

    #[test]
    fn money_arithmetic_keeps_its_currency_and_refuses_two() {
        let usd = |m| Value::Money {
            minor: m,
            currency: 0,
        };
        assert_eq!(
            arith_values(usd(5), usd(3), true, crate::arith::sub),
            Ok(Ok(usd(2)))
        );
        assert_eq!(
            arith_values(usd(5), Value::Int(3), false, crate::arith::mul),
            Ok(Ok(usd(15)))
        );
        assert_eq!(
            arith_values(
                usd(5),
                Value::Money {
                    minor: 1,
                    currency: 1
                },
                true,
                crate::arith::add
            ),
            Err(Mismatch::Currencies(0, 1))
        );
        assert_eq!(
            arith_values(usd(5), Value::Int(1), true, crate::arith::add),
            Err(Mismatch::Kinds("money", "integer"))
        );
    }
}
