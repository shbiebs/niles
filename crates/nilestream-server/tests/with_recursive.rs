//! **`with recursive` through the served engine** (cycle 15, C15-05b; the author's decision
//! 4). The lowering and the reference semantics are held in niles-lang; this holds that a
//! client on the wire gets the same answer, and that the engine says how it answers it.
//!
//! The query is reachability over the ledger: the accounts that share a transaction with
//! account 0, and the accounts that share one with those, until nothing new is reached. It
//! is checked against a breadth-first search over the rows the same session returns.

use nilestream_server::daemon;
use nilestream_server::pg_wire::{self, Backend, Frontend};
use nilestream_server::rev_engine::{serve_path, RevEngine, ServePath};
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};
use std::collections::{BTreeMap, BTreeSet};

const REACH: &str = "with recursive reach(acct) as (\
    select acct from postings where acct = 0 \
    union \
    select q.acct from reach r join postings p on r.acct = p.acct join postings q on p.txn = q.txn\
) select acct from reach";

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
fn reachability_over_the_ledger_answers_what_a_search_finds() {
    let e = RevEngine::seeded(6, 2, 1000, ViewMode::Demand, EvictionPolicy::Lru);
    let mut s = Session::new("bench".into(), "bank".into(), daemon::DEFAULT_SCHEMA.into());
    // The oracle: accounts by transaction, searched from account 0.
    let mut by_txn: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for r in rows(&mut s, &e, "select p.txn, p.acct from postings p") {
        by_txn.entry(r[0].clone()).or_default().insert(r[1].clone());
    }
    let mut reached: BTreeSet<String> = BTreeSet::new();
    let mut frontier = vec!["0".to_string()];
    while let Some(a) = frontier.pop() {
        if !reached.insert(a.clone()) {
            continue;
        }
        for accts in by_txn.values().filter(|x| x.contains(&a)) {
            frontier.extend(accts.iter().cloned());
        }
    }
    let got: BTreeSet<String> = rows(&mut s, &e, REACH)
        .into_iter()
        .map(|r| r[0].clone())
        .collect();
    assert!(!reached.is_empty());
    assert_eq!(got, reached);
}

#[test]
fn a_recursion_is_materialised_and_explain_says_so() {
    let program = format!(
        "{}\nview __wire_result = sql {{ {REACH} }} serve {{ consistency: snapshot, materialize: auto }};\n",
        daemon::DEFAULT_SCHEMA
    );
    let (prog, d) = niles_lang::parser::parse_program(&program);
    assert!(!d.has_errors());
    let (cat, _) = niles_lang::resolve::resolve_program(&prog, 0);
    let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
    assert!(!ld.has_errors(), "{:?}", ld.items);
    assert_eq!(
        serve_path(&lowered.circuit, "__wire_result"),
        ServePath::Materialise
    );
}
