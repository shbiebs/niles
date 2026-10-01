//! **Over the wire: an ungrouped aggregate over no rows answers one row** (C16-01, F-02-3).
//!
//! The served fold answered `(0 rows)` to `select count(amt), sum(amt) from postings where
//! acct = 1 and value_date >= 1000`; SQL answers `(0, NULL)`. That stopped E27b's sweep at
//! single 10⁵ (q3 on N, "unparseable reply: no row 0"). Every shape here goes through the
//! server's own planner, so whichever path it picks — the account-restricted fold, the
//! full fold, the reference evaluator — is held to the same answer, and the grouped forms
//! are held to answering no row.

use nilestream_server::daemon;
use nilestream_server::pg_wire::{self, Backend, Frontend};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};

fn ask(sql: &str) -> Vec<Vec<Option<String>>> {
    let e = RevEngine::seeded(20, 2, 10, ViewMode::Demand, EvictionPolicy::Lru);
    let mut s = Session::new("bench".into(), "bank".into(), daemon::DEFAULT_SCHEMA.into());
    let out = s.handle(Frontend::Query(sql.into()), &e);
    assert!(
        !out.iter()
            .any(|m| matches!(m, Backend::ErrorResponse { .. })),
        "refused: {sql}: {out:?}"
    );
    // Encoded and decoded as a client would read them: an evaluated answer is a `Rows`
    // block, which only the encoder turns into `DataRow`s.
    let mut bytes = Vec::new();
    for m in &out {
        pg_wire::encode_into(&mut bytes, m);
    }
    pg_wire::decoded_rows(&[Backend::Raw(bytes)])
}

/// The value cells, without the `anchor` stamp the served engine appends to every row.
fn values(rows: &[Vec<Option<String>>], width: usize) -> Vec<Vec<Option<String>>> {
    rows.iter().map(|r| r[..width].to_vec()).collect()
}

#[test]
fn an_empty_value_date_window_answers_zero_and_null() {
    let sql = "select count(amt), sum(amt) from postings where acct = 1 and value_date >= 1000";
    let rows = ask(sql);
    assert_eq!(
        values(&rows, 2),
        vec![vec![Some("0".to_string()), None]],
        "{sql}: SQL's scalar aggregate is one row, (0, NULL)"
    );
}

#[test]
fn an_account_with_no_postings_answers_one_row_when_ungrouped() {
    for (sql, want) in [
        (
            "select count(amt) from postings where acct = 99999",
            vec![Some("0".to_string())],
        ),
        (
            "select sum(amt) from postings where acct = 99999",
            vec![None],
        ),
        (
            "select count(amt), sum(amt) from postings where amt > 1000000000",
            vec![Some("0".to_string()), None],
        ),
    ] {
        let rows = ask(sql);
        assert_eq!(values(&rows, want.len()), vec![want], "{sql}");
    }
}

#[test]
fn the_grouped_forms_still_answer_no_row() {
    for sql in [
        "select acct, sum(amt) from postings where acct = 99999 group by acct",
        "select cur, count(amt) from postings where amt > 1000000000 group by cur",
    ] {
        assert!(ask(sql).is_empty(), "{sql}: no row passes, so no group");
    }
}

#[test]
fn an_ungrouped_aggregate_over_rows_still_counts_them() {
    let rows = ask("select count(amt) from postings where acct = 1");
    assert_eq!(rows.len(), 1);
    assert_ne!(rows[0][0], Some("0".to_string()), "account 1 has postings");
}
