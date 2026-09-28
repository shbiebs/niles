//! **A reply is sent in the order its query asked for.**
//!
//! Ordering is not part of a Z-set's denotation, so the engine's answer to
//! `order by sum(amt) desc limit 10` is the right ten rows with no order; the wire is where the
//! order has to be applied, and until cycle 14 it was not: the ten rows went out in ascending
//! account order. E27's correctness pass (R2-02) found it on its first run — every top-10 read
//! against Nilestream diverged from the oracle in order and in nothing else.

use nilestream_server::daemon;
use nilestream_server::pg_wire::{self, Backend, Frontend};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};

/// The decoded rows, in the order the client receives them (the wire encoding, then decoded).
fn reply(s: &mut Session, e: &RevEngine, sql: &str) -> Vec<Vec<i128>> {
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
        .map(|r| r.into_iter().map(|c| c.unwrap().parse().unwrap()).collect())
        .collect()
}

fn setup() -> (Session, RevEngine) {
    let e = RevEngine::seeded(40, 3, 1000, ViewMode::Demand, EvictionPolicy::Lru);
    let mut s = Session::new("bench".into(), "bank".into(), daemon::DEFAULT_SCHEMA.into());
    // Unequal balances in an order unrelated to the account number (17a mod 41 permutes
    // 1..=40), so a reply in account order and a reply in balance order are different.
    for a in 1..=40 {
        let w = s.handle(
            Frontend::Query(format!(
                "insert into postings values ({}, {a}, 0, {}), ({}, 0, 0, -{})",
                10_000 + a,
                (a * 17) % 41 * 1000,
                10_000 + a,
                (a * 17) % 41 * 1000
            )),
            &e,
        );
        assert!(!w.iter().any(|m| matches!(m, Backend::ErrorResponse { .. })));
    }
    (s, e)
}

#[test]
fn order_by_desc_with_limit_is_sent_largest_first() {
    let (mut s, e) = setup();
    let top = reply(
        &mut s,
        &e,
        "select acct, sum(amt) from postings where cur = 0 group by acct order by sum(amt) desc limit 10",
    );
    assert_eq!(top.len(), 10);
    let sums: Vec<i128> = top.iter().map(|r| r[1]).collect();
    let mut sorted = sums.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(sums, sorted, "sent in descending order of sum");
    // And they are the ten largest: compare against the full, unordered report.
    let mut all: Vec<i128> = reply(
        &mut s,
        &e,
        "select acct, sum(amt) from postings group by acct",
    )
    .iter()
    .map(|r| r[1])
    .collect();
    all.sort_by(|a, b| b.cmp(a));
    assert_eq!(sums, all[..10].to_vec());
}

#[test]
fn order_by_asc_and_by_a_key_column_are_honoured_too() {
    let (mut s, e) = setup();
    let asc = reply(
        &mut s,
        &e,
        "select acct, sum(amt) from postings group by acct order by sum(amt) asc limit 5",
    );
    let v: Vec<i128> = asc.iter().map(|r| r[1]).collect();
    assert!(v.windows(2).all(|w| w[0] <= w[1]), "{v:?}");
    let by_acct = reply(
        &mut s,
        &e,
        "select acct, sum(amt) from postings group by acct order by acct desc",
    );
    let accts: Vec<i128> = by_acct.iter().map(|r| r[0]).collect();
    assert!(accts.windows(2).all(|w| w[0] > w[1]), "{accts:?}");
}

#[test]
fn without_order_by_the_reply_order_is_unchanged() {
    let (mut s, e) = setup();
    let plain = reply(
        &mut s,
        &e,
        "select acct, sum(amt) from postings group by acct",
    );
    let accts: Vec<i128> = plain.iter().map(|r| r[0]).collect();
    assert!(
        accts.windows(2).all(|w| w[0] < w[1]),
        "rows' own order, as before"
    );
}
