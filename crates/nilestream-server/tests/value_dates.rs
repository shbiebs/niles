//! **Value dates on the served relation, and a back-valued correction** (cycle 15, C15-02;
//! E27b's pre-registered change E3, `docs/study/E27b-design.md` §2).
//!
//! The ledger has always stored and chained each posting's valid time, and the served relation
//! could not read it: the insert path wrote 0 and the rows had no column for it. A
//! back-valued correction is a posting with an earlier value date and a later epoch. A read of
//! the balance *valid at* a day includes it once it is recorded; a read *as of* an epoch
//! before it was recorded does not, whatever its value date says.

use nilestream_server::daemon;
use nilestream_server::pg_wire::{self, Backend, Frontend};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::{Serving, Session};
use proto_engine::{EvictionPolicy, ViewMode};

fn ask(s: &mut Session, e: &RevEngine, sql: &str) -> Vec<Vec<String>> {
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

fn one(rows: &[Vec<String>]) -> String {
    assert_eq!(rows.len(), 1, "{rows:?}");
    rows[0][1].clone()
}

#[test]
fn a_back_valued_correction_changes_a_valid_time_read_and_not_an_earlier_as_of_read() {
    let e = RevEngine::seeded(12, 2, 1000, ViewMode::Demand, EvictionPolicy::Lru);
    let mut s = Session::new("bench".into(), "bank".into(), daemon::DEFAULT_SCHEMA.into());
    let before = e.frontier();
    let valid_at = |e_: Option<u64>| {
        let asof = e_
            .map(|x| format!(" as of system time {x}"))
            .unwrap_or_default();
        format!("select acct, sum(amt) from postings{asof} where acct = 7 and value_date <= 0 group by acct")
    };
    let valid_before = one(&ask(&mut s, &e, &valid_at(None)));
    // The correction: recorded now, valid on day 0, moving 3.00 out of account 7. Beside
    // it, a posting recorded now and valid on day 5, which a read valid at day 0 must not
    // see: an insert path that dropped the value date (writing 0, as it did) would count it.
    ask(
        &mut s,
        &e,
        "insert into postings values (9101, 7, 0, -300, 0), (9101, 8, 0, 300, 0)",
    );
    ask(
        &mut s,
        &e,
        "insert into postings values (9102, 7, 0, -100, 5), (9102, 8, 0, 100, 5)",
    );
    assert!(e.frontier() > before);
    let valid_after = one(&ask(&mut s, &e, &valid_at(None)));
    assert_eq!(
        valid_after.parse::<i128>().unwrap(),
        valid_before.parse::<i128>().unwrap() - 300,
        "the balance valid at day 0 includes the correction once it is recorded"
    );
    assert_eq!(
        one(&ask(&mut s, &e, &valid_at(Some(before)))),
        valid_before,
        "as of an epoch before the correction was recorded, it is not there"
    );
    // And the value date is readable as the column it is.
    let rows = ask(
        &mut s,
        &e,
        "select p.txn, value_date from postings p where acct = 7 and p.txn = 9101",
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][..2], ["9101".to_string(), "0".to_string()]);
}

/// E27's q3 and q4 on N, which the served relation could not express (E27b's E3 and the
/// desk deviation): each answers, and answers what the rows define.
#[test]
fn a_month_statement_and_a_desk_exposure_answer_on_n() {
    let e = RevEngine::seeded(40, 3, 1000, ViewMode::Demand, EvictionPolicy::Lru);
    let mut s = Session::new("bench".into(), "bank".into(), daemon::DEFAULT_SCHEMA.into());
    let rows = ask(
        &mut s,
        &e,
        "select acct, cur, amt, value_date from postings",
    );
    let n = |r: &Vec<String>, i: usize| r[i].parse::<i128>().unwrap();
    // q3: account 7's postings with a value date in [1, 2]. `between` does not lower on the
    // SQL surface, so the two comparisons are the spelling (E27b deviation, before any run).
    let mine: Vec<i128> = rows
        .iter()
        .filter(|r| n(r, 0) == 7 && n(r, 1) == 0 && (1..=2).contains(&n(r, 3)))
        .map(|r| n(r, 2))
        .collect();
    let q3 = ask(
        &mut s,
        &e,
        "select count(amt), sum(amt) from postings where acct = 7 and cur = 0 and value_date >= 1 and value_date <= 2",
    );
    assert!(!mine.is_empty());
    assert_eq!(
        q3,
        vec![vec![
            mine.len().to_string(),
            mine.iter().sum::<i128>().to_string(),
            q3[0][2].clone()
        ]]
    );
    // q4: desk 3 is every account with acct % 16 = 3.
    let desk: i128 = rows
        .iter()
        .filter(|r| n(r, 0) % 16 == 3)
        .map(|r| n(r, 2))
        .sum();
    let q4 = ask(
        &mut s,
        &e,
        "select cur, sum(amt) from postings where acct % 16 = 3 group by cur",
    );
    assert_eq!(q4.len(), 1);
    assert_eq!(q4[0][1], desk.to_string());
}
