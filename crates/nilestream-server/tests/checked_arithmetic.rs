//! **A query whose arithmetic has no answer is refused over the wire, and never answers a row.**
//!
//! Before C11-02 the reference evaluator returned `Value::Int(0)` for `x / 0` and `x % 0`, and
//! the served daemon runs that evaluator for every operator above the keyed fold. So a query
//! whose projection or `having` clause divided by zero came back to the client as **a row
//! containing a zero** — indistinguishable, at the protocol, from a balance of zero. Its `+`,
//! `-` and `*` wrapped in a release build and panicked in a debug one, so the same expression
//! meant one thing under `cargo test` and another in a benchmark.
//!
//! These tests are at the session boundary rather than inside the evaluator, because the
//! evaluator's own unit tests would have passed throughout: `niles_ir::eval` was doing exactly
//! what it was written to do. What was wrong was what a client received.
//!
//! # Why the assertions are on the absence of rows
//!
//! An error message is easy to add and easy to add *beside* a row. The property that matters
//! is that the refusal replaces the answer: `no DataRow` is the assertion, and the diagnostic
//! is checked second.

use nilestream_server::daemon;
use nilestream_server::pg_wire::{self, Backend, Frontend};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};

fn session() -> Session {
    Session::new(
        "bench".into(),
        "bank".into(),
        daemon::DEFAULT_SCHEMA.to_string(),
    )
}

fn engine() -> RevEngine {
    RevEngine::seeded(20, 2, 50, ViewMode::Demand, EvictionPolicy::Lru)
}

struct Reply {
    rows: Vec<Vec<Option<String>>>,
    /// `(SQLSTATE, headline, detail)`. The detail is captured because that is where this
    /// server puts the sentence that says *what* failed; the headline is one of five fixed
    /// strings, and a test that only read it could not tell a division by zero from a
    /// fixpoint that did not converge.
    error: Option<(String, String, String)>,
}

fn ask(sql: &str) -> Reply {
    let (mut s, e) = (session(), engine());
    let out = s.handle(Frontend::Query(sql.into()), &e);
    let error = out.iter().find_map(|m| match m {
        Backend::ErrorResponse {
            code,
            message,
            detail,
            ..
        } => Some((
            code.clone(),
            message.clone(),
            detail.clone().unwrap_or_default(),
        )),
        _ => None,
    });
    // **Rows arrive two ways, and a test that looked at one would have been silent about
    // the other.** A served answer is framed once into a `Backend::Rows` block; a diagnostic
    // statement's rows are individual `DataRow`s that `decoded_rows` handles. Counting only
    // the second made every control in this file report zero rows and pass its refusal
    // assertions for the wrong reason.
    let mut rows = pg_wire::decoded_rows(&out);
    for m in &out {
        if let Backend::Rows(block) = m {
            for (r, _w) in block.z.iter() {
                rows.push(r.iter().map(|v| Some(format!("{v:?}"))).collect());
            }
        }
    }
    Reply { rows, error }
}

#[test]
fn a_projection_that_divides_by_zero_is_refused_and_answers_no_row() {
    let r = ask("select acct, amt / 0 from postings where acct = 3");
    assert!(
        r.rows.is_empty(),
        "the query answered {} row(s). Before C11-02 the first of them held a zero, which a \
         client cannot tell from a balance of zero: {:?}",
        r.rows.len(),
        r.rows
    );
    let (code, _headline, detail) = r.error.expect("a refusal, not silence");
    assert_eq!(code, "22000", "a data exception");
    assert!(
        detail.contains("is not a number"),
        "the diagnostic must say what failed, and its detail said {detail:?}"
    );
}

#[test]
fn a_remainder_by_zero_is_refused_the_same_way() {
    let r = ask("select acct, amt % 0 from postings where acct = 3");
    assert!(r.rows.is_empty(), "{:?}", r.rows);
    let (code, _, detail) = r.error.expect("a refusal");
    assert_eq!(code, "22000");
    assert!(detail.contains("is not a number"), "{detail:?}");
}

#[test]
fn a_filter_that_divides_by_zero_refuses_rather_than_filtering_everything_out() {
    // **The quiet one.** A predicate whose arithmetic fails could answer `false` for every
    // row and return an empty result — a plausible answer, delivered with no error, to a
    // question that was never evaluated. The empty answer and the refusal look identical in
    // the row count, which is why the assertion is on the error.
    let r = ask("select acct, amt from postings where acct = 3 and amt / 0 > 1");
    assert!(r.rows.is_empty(), "{:?}", r.rows);
    assert!(
        r.error.is_some(),
        "an empty answer with no error is the failure this test is for: the client cannot \
         tell `no rows matched` from `the predicate could not be evaluated`"
    );
}

#[test]
fn arithmetic_that_has_an_answer_still_has_it() {
    // The control, and it is not decoration: a repair that refused too much would make every
    // assertion above pass. Division by a non-zero constant is ordinary arithmetic and must
    // come back with rows.
    let r = ask("select acct, amt / 2 from postings where acct = 3");
    assert!(r.error.is_none(), "{:?}", r.error);
    assert!(
        !r.rows.is_empty(),
        "the control answered nothing, so the refusals above prove nothing"
    );
}

#[test]
fn an_overflow_is_a_refusal_in_this_profile_whichever_profile_this_is() {
    // `sum(amt) * <i128::MAX>` overflows for any non-zero sum. Written as an assertion on the
    // reply rather than as `#[should_panic]`, because the defect being guarded is exactly
    // that release did not panic: a `should_panic` test would have been green in debug and
    // silent about the build anyone measures.
    let big = i128::MAX.to_string();
    let r = ask(&format!(
        "select acct, amt * {big} from postings where acct = 3"
    ));
    assert!(r.rows.is_empty(), "{:?}", r.rows);
    let (code, _, detail) = r.error.expect("a refusal");
    assert_eq!(code, "22000");
    assert!(
        detail.contains("does not fit in a 128-bit integer"),
        "the diagnostic should name the overflow: {detail:?}"
    );
}

#[test]
fn the_boundary_cases_around_the_refusals_are_ordinary_arithmetic() {
    // A repair that refused `MIN / -1` by refusing every division by a negative number, or
    // every division involving `MIN`, would pass every test above. These are its controls.
    for sql in [
        "select acct, amt / -1 from postings where acct = 3",
        "select acct, amt / 1 from postings where acct = 3",
        "select acct, -amt from postings where acct = 3",
        "select acct, amt % -1 from postings where acct = 3",
    ] {
        let r = ask(sql);
        assert!(r.error.is_none(), "{sql} was refused: {:?}", r.error);
        assert!(!r.rows.is_empty(), "{sql} answered nothing");
    }
}
