//! **A client can ask for the answer at an epoch, and gets the answer at that epoch.**
//!
//! Item 2 of the thesis's fourteen is a read model that reconstructs evicted state "on demand
//! through upqueries anchored to a requested epoch". Until cycle 14 no client of the served
//! engine could request one: the SQL surface had no spelling for it (the sql-surface table's
//! `AS OF SYSTEM TIME` row meant that the *Niles* `.as_of(#e)` lowered), and the session
//! answered every circuit at the frontier whatever `Op::AsOf` it carried. The comparator
//! (R2-02) needs a historical-anchor read on every arm, so the path is made reachable here and
//! held to the obvious property: the answer at a past epoch is the answer that was given at
//! that epoch, whatever has been appended and whatever has been evicted since.

use nilestream_server::daemon;
use nilestream_server::pg_wire::{self, Backend, Frontend};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::{Serving, Session};
use proto_engine::{EvictionPolicy, ViewMode};

struct Reply {
    rows: Vec<String>,
    anchor: Option<u64>,
    error: Option<(String, String)>,
}

fn ask(s: &mut Session, e: &RevEngine, sql: &str) -> Reply {
    let out = s.handle(Frontend::Query(sql.into()), e);
    let error = out.iter().find_map(|m| match m {
        Backend::ErrorResponse { code, detail, .. } => {
            Some((code.clone(), detail.clone().unwrap_or_default()))
        }
        _ => None,
    });
    let mut rows: Vec<String> = pg_wire::decoded_rows(&out)
        .into_iter()
        .map(|r| format!("{r:?}"))
        .collect();
    let mut anchor = None;
    for m in &out {
        if let Backend::Rows(block) = m {
            anchor = Some(block.anchor);
            for (r, w) in &block.z {
                rows.push(format!("{r:?} x{w}"));
            }
        }
    }
    rows.sort();
    Reply {
        rows,
        anchor,
        error,
    }
}

fn session() -> Session {
    Session::new(
        "bench".into(),
        "bank".into(),
        daemon::DEFAULT_SCHEMA.to_string(),
    )
}

/// Accounts 1..=20, two postings each; a budget of `budget` resident keys.
fn engine(budget: usize) -> RevEngine {
    RevEngine::seeded(20, 2, budget, ViewMode::Demand, EvictionPolicy::Lru)
}

fn check_history_is_kept(budget: usize) {
    let e = engine(budget);
    let mut s = session();
    let f0 = e.frontier();

    let point = "select acct, sum(amt) from postings where acct = 7 group by acct";
    let report = "select acct, sum(amt) from postings group by acct";
    let before_point = ask(&mut s, &e, point);
    let before_report = ask(&mut s, &e, report);
    assert!(before_point.error.is_none(), "{:?}", before_point.error);
    assert_eq!(before_point.anchor, Some(f0));

    // Move money on account 7 three times, and read the other accounts in between so a small
    // budget evicts account 7's resident balance.
    for i in 0..3 {
        let w = ask(
            &mut s,
            &e,
            &format!(
                "insert into postings values ({}, 7, 0, -50), ({}, 8, 0, 50)",
                9001 + i,
                9001 + i
            ),
        );
        assert!(w.error.is_none(), "{:?}", w.error);
        for other in 10..=20 {
            let _ = ask(
                &mut s,
                &e,
                &format!("select acct, sum(amt) from postings where acct = {other} group by acct"),
            );
        }
    }
    assert!(e.frontier() > f0);
    let head = ask(&mut s, &e, point);
    assert_ne!(
        head.rows, before_point.rows,
        "the head moved, so the test can tell the two apart"
    );

    for spelling in [
        format!("select acct, sum(amt) from postings as of system time {f0} where acct = 7 group by acct"),
        format!("select acct, sum(amt) from postings as of system time #{f0} where acct = 7 group by acct"),
        format!("SELECT acct, sum(amt) FROM postings AS OF SYSTEM TIME {f0} WHERE acct = 7 GROUP BY acct"),
    ] {
        let at = ask(&mut s, &e, &spelling);
        assert!(at.error.is_none(), "{spelling}: {:?}", at.error);
        assert_eq!(at.rows, before_point.rows, "{spelling}: the answer at #{f0} is the answer given at #{f0}");
        assert_eq!(at.anchor, Some(f0), "{spelling}: stamped with the anchor asked for, not the frontier");
    }
    let report_at = ask(
        &mut s,
        &e,
        &format!("select acct, sum(amt) from postings as of system time {f0} group by acct"),
    );
    assert!(report_at.error.is_none(), "{:?}", report_at.error);
    assert_eq!(
        report_at.rows, before_report.rows,
        "the whole report at #{f0}"
    );
    assert_eq!(report_at.anchor, Some(f0));

    // A historical read does not move the session backwards: the next frontier read is at the
    // head again.
    let again = ask(&mut s, &e, point);
    assert_eq!(again.rows, head.rows);
    assert_eq!(again.anchor, Some(e.frontier()));
}

#[test]
fn an_answer_as_of_an_epoch_is_the_answer_given_at_that_epoch() {
    check_history_is_kept(100);
}

#[test]
fn and_it_still_is_after_the_key_has_been_evicted_and_must_be_reconstructed() {
    check_history_is_kept(2);
}

#[test]
fn an_epoch_beyond_the_frontier_is_refused_and_answers_no_row() {
    let e = engine(100);
    let mut s = session();
    let beyond = e.frontier() + 5;
    let r = ask(
        &mut s,
        &e,
        &format!("select acct, sum(amt) from postings as of system time {beyond} where acct = 7 group by acct"),
    );
    assert!(
        r.rows.is_empty(),
        "no row for an epoch that has not been sealed: {:?}",
        r.rows
    );
    let (code, detail) = r.error.expect("refused");
    assert_eq!(code, "22023");
    assert!(detail.contains("frontier"), "{detail}");
}

#[test]
fn an_as_of_that_is_not_an_epoch_is_refused_rather_than_read_at_the_frontier() {
    let e = engine(100);
    let mut s = session();
    let r = ask(
        &mut s,
        &e,
        "select acct, sum(amt) from postings as of system time 'yesterday' where acct = 7 group by acct",
    );
    assert!(r.rows.is_empty(), "{:?}", r.rows);
    assert!(
        r.error.is_some(),
        "a non-epoch pin must not fall through to the frontier"
    );
}

#[test]
fn an_alias_and_an_as_of_can_both_be_written() {
    let e = engine(100);
    let mut s = session();
    let f = e.frontier();
    let plain = ask(
        &mut s,
        &e,
        "select acct, sum(amt) from postings where acct = 3 group by acct",
    );
    let aliased = ask(
        &mut s,
        &e,
        &format!("select p.acct, sum(p.amt) from postings p as of system time {f} where p.acct = 3 group by p.acct"),
    );
    assert!(aliased.error.is_none(), "{:?}", aliased.error);
    assert_eq!(aliased.rows, plain.rows);
}
