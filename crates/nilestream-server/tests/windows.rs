//! **A running balance in recording order, through the served engine** (cycle 15, C15-05b;
//! the author's decision 4: window functions, with the epoch as a readable column).
//!
//! E30's Q07 asked for each posting's running balance per account and currency in the order
//! the postings were recorded, and Niles could not say it: no window function, and no column
//! holding a posting's epoch. The oracle here is computed from the rows the same session
//! returns for `select p.txn, acct, cur, amt, recorded_at`, so the check is that the window
//! the engine evaluates is the running sum those rows define.

use nilestream_server::daemon;
use nilestream_server::pg_wire::{self, Backend, Frontend};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};
use std::collections::BTreeMap;

/// A posting as the oracle orders it: `(recorded_at, txn, amt)`.
type Posting = (i64, i64, i64);

fn rows(s: &mut Session, e: &RevEngine, sql: &str) -> Vec<Vec<String>> {
    let out = s.handle(Frontend::Query(sql.into()), e);
    assert!(
        !out.iter()
            .any(|m| matches!(m, Backend::ErrorResponse { .. })),
        "{sql}: {out:?}"
    );
    let mut bytes = Vec::new();
    for m in &out {
        pg_wire::encode_into(&mut bytes, m);
    }
    pg_wire::decoded_rows(&[Backend::Raw(bytes)])
        .into_iter()
        .map(|r| r.into_iter().map(|c| c.unwrap_or_default()).collect())
        .collect()
}

#[test]
fn a_running_balance_in_recording_order_is_what_the_rows_define() {
    let e = RevEngine::seeded(6, 3, 1000, ViewMode::Demand, EvictionPolicy::Lru);
    let mut s = Session::new("bench".into(), "bank".into(), daemon::DEFAULT_SCHEMA.into());
    // The oracle: (acct, cur) -> postings in (recorded_at, txn) order, summed as they come.
    let mut parts: BTreeMap<(i64, i64), Vec<Posting>> = BTreeMap::new();
    let base = rows(
        &mut s,
        &e,
        "select p.txn, acct, cur, amt, recorded_at from postings p",
    );
    assert!(!base.is_empty());
    for r in &base {
        let n = |i: usize| r[i].parse::<i64>().expect("an integer");
        parts
            .entry((n(1), n(2)))
            .or_default()
            .push((n(4), n(0), n(3)));
    }
    let mut want: Vec<String> = Vec::new();
    for ((acct, cur), mut ps) in parts {
        ps.sort();
        let mut sum = 0;
        for (_, txn, amt) in ps {
            sum += amt;
            want.push(format!("{acct} {cur} {txn} {sum}"));
        }
    }
    want.sort();
    let mut got: Vec<String> = rows(
        &mut s,
        &e,
        "select acct, cur, p.txn, sum(amt) over (partition by acct, cur order by recorded_at, p.txn) as running from postings p",
    )
    .into_iter()
    .map(|r| r[..4].join(" "))
    .collect();
    got.sort();
    assert_eq!(got, want);
}

#[test]
fn select_star_over_a_system_time_read_is_the_declared_columns() {
    let e = RevEngine::seeded(4, 2, 1000, ViewMode::Demand, EvictionPolicy::Lru);
    let mut s = Session::new("bench".into(), "bank".into(), daemon::DEFAULT_SCHEMA.into());
    // `select * from postings` with no clause is the base node itself, which the served
    // contract refuses (IR000); a clause that keeps every row is the comparison.
    let plain = rows(&mut s, &e, "select * from postings where acct >= 0");
    let timed = rows(&mut s, &e, "select * from postings where recorded_at >= 0");
    assert_eq!(plain.len(), timed.len());
    assert_eq!(
        plain.first().map(Vec::len),
        timed.first().map(Vec::len),
        "`recorded_at` is not one of the star's columns"
    );
}
