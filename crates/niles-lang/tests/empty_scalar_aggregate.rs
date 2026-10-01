//! **SQL's scalar aggregate answers one row, even over no input** (cycle 16, C16-01, F-02-3).
//!
//! `select count(amt), sum(amt) from postings where …` over no rows is `(0, NULL)` in SQL:
//! an aggregate with no `group by` forms exactly one group, empty or not. The reference
//! evaluator answered **no row**, and so did the served fold. E27b's q3 stopped on it at 10⁵,
//! where a key's 22-day value-date window first came up empty. A grouped aggregate over no
//! rows still forms no group and answers no row; this file holds both halves.

use niles_ir::eval;
use niles_ir::value::Value;
use niles_lang::{lower, parser, resolve};

const SCHEMA: &str = "schema s {
    currency usd { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        conserve per (txn, cur);
        retain forever;
    }
    index ix_p on postings (acct) anchor;
}
";

fn run(view: &str) -> eval::ZSet {
    let src = format!("{SCHEMA}\nview v = sql {{ {view} }};\n");
    let (prog, d) = parser::parse_program(&src);
    assert!(!d.has_errors(), "{d:?}");
    let (cat, r) = resolve::resolve_program(&prog, 0);
    assert!(!r.has_errors(), "{r:?}");
    let (l, ld) = lower::lower_program(&prog, &cat);
    assert!(!ld.has_errors(), "{ld:?}");
    let mut sources = std::collections::BTreeMap::new();
    // postings(txn, acct, cur, amt): two balanced legs on accounts 1 and 2.
    sources.insert(
        "postings".to_string(),
        eval::zset(&[(&[1, 1, 0, -5], 1), (&[1, 2, 0, 5], 1)]),
    );
    eval::try_run(&l.circuit, "v", &sources)
        .expect("evaluates")
        .0
}

#[test]
fn an_ungrouped_aggregate_over_no_rows_answers_one_row_of_zero_and_null() {
    let z = run("select count(amt), sum(amt) from postings where acct = 99");
    let rows: Vec<_> = z.iter().filter(|(_, w)| **w > 0).collect();
    assert_eq!(rows.len(), 1, "SQL's scalar aggregate is one row: {z:?}");
    assert_eq!(rows[0].0, &vec![Value::Int(0), Value::Null], "{z:?}");
    assert_eq!(*rows[0].1, 1);
}

#[test]
fn a_grouped_aggregate_over_no_rows_still_answers_no_row() {
    let z = run("select acct, sum(amt) from postings where acct = 99 group by acct");
    assert!(z.is_empty(), "no row passes, so there is no group: {z:?}");
}

#[test]
fn an_ungrouped_aggregate_over_rows_is_unchanged() {
    let z = run("select count(amt) from postings where acct = 1");
    let rows: Vec<_> = z.iter().collect();
    assert_eq!(rows.len(), 1, "{z:?}");
    assert_eq!(rows[0].0, &vec![Value::Int(1)]);
}
