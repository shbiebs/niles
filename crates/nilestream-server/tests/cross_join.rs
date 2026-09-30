//! **The engine's cost for a product, checked before `cross join` was admitted** (cycle 15,
//! C15-05b; the author's decision 5: "admit the product; the engine's cost for a product is
//! checked first").
//!
//! A product has no fold plan and no view path, so the engine answers it by materialising
//! through the reference evaluator: `|L| × |R|` rows. That is its cost, stated rather than
//! hidden. This test holds the answer (the row count of the product is the product of the row
//! counts, for the comma spelling and the `cross join` spelling alike) and the serve path
//! (`explain` says it materialises), so a later change that made a product silently cheaper by
//! answering a different query would fail here.

use nilestream_server::daemon;
use nilestream_server::pg_wire::{self, Backend, Frontend};
use nilestream_server::rev_engine::{serve_path, RevEngine, ServePath};
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};

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
fn a_product_answers_every_pair_on_both_spellings() {
    // 6 accounts, 2 rounds: 24 postings.
    let e = RevEngine::seeded(6, 2, 1000, ViewMode::Demand, EvictionPolicy::Lru);
    let mut s = Session::new("bench".into(), "bank".into(), daemon::DEFAULT_SCHEMA.into());
    let n = rows(&mut s, &e, "select acct, amt from postings").len();
    assert_eq!(n, 24, "the seeded base");
    for sql in [
        "select p.acct, q.acct from postings p cross join postings q",
        "select p.acct, q.acct from postings p, postings q",
    ] {
        assert_eq!(rows(&mut s, &e, sql).len(), n * n, "{sql}");
    }
}

#[test]
fn a_product_is_materialised_and_explain_says_so() {
    let program = format!(
        "{}\nview __wire_result = sql {{ select p.acct, q.acct from postings p cross join postings q }} serve {{ consistency: snapshot, materialize: auto }};\n",
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
