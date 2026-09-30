//! **A read that names its currency is answered from the view over a base of two currencies**
//! (cycle 15, C15-02; E27b's pre-registered change E1, `docs/study/E27b-design.md` §2).
//!
//! `answer_from_view` refused whenever the base held more than one currency, including for a
//! read grouped by `(acct, cur)` or restricted to `cur = k`, which the `(account,
//! currency)`-keyed balance view answers exactly. So on E27's `multi` series every keyed read
//! of N folded the base and nothing was resident. The refusal that stays is the one that is
//! right: a read that sums one account over several currencies.

use nilestream_server::pg_wire::{self, Backend, Frontend};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};

const SCHEMA: &str = "\
schema bank {
    currency usd { scale: 2 }
    currency eur { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
    index ix_postings on postings (acct) anchor;
}
";

fn ask(s: &mut Session, e: &RevEngine, sql: &str) -> Result<Vec<Vec<String>>, String> {
    let out = s.handle(Frontend::Query(sql.into()), e);
    if let Some(m) = out.iter().find_map(|m| match m {
        Backend::ErrorResponse { code, message, .. } => Some(format!("{code}: {message}")),
        _ => None,
    }) {
        return Err(m);
    }
    let mut bytes = Vec::new();
    for m in &out {
        pg_wire::encode_into(&mut bytes, m);
    }
    Ok(pg_wire::decoded_rows(&[Backend::Raw(bytes)])
        .into_iter()
        .map(|r| r.into_iter().map(|c| c.unwrap_or_default()).collect())
        .collect())
}

/// `view_answers` from `select nilestream_stats` (its sixth column).
fn view_answers(s: &mut Session, e: &RevEngine) -> u64 {
    ask(s, e, "select nilestream_stats").unwrap()[0][5]
        .parse()
        .unwrap()
}

/// The balance of `(acct, cur)` summed from the account's rows, and whether it has any.
fn oracle(s: &mut Session, e: &RevEngine, acct: u64, cur: u64) -> Option<i128> {
    let rows = ask(
        s,
        e,
        &format!("select acct, cur, amt from postings where acct = {acct}"),
    )
    .unwrap();
    let mine: Vec<i128> = rows
        .iter()
        .filter(|r| r[1] == cur.to_string())
        .map(|r| r[2].parse().unwrap())
        .collect();
    (!mine.is_empty()).then(|| mine.iter().sum())
}

#[test]
fn a_currency_named_read_is_answered_from_the_view_over_two_currencies() {
    let e = RevEngine::seeded(12, 2, 1000, ViewMode::Demand, EvictionPolicy::Lru);
    let mut s = Session::new("bench".into(), "bank".into(), SCHEMA.into());
    // A second currency: two eur transfers between accounts 7 and 8.
    for (txn, amt) in [(9001, 125), (9002, 40)] {
        ask(
            &mut s,
            &e,
            &format!("insert into postings values ({txn}, 7, 1, -{amt}), ({txn}, 8, 1, {amt})"),
        )
        .unwrap();
    }
    for (acct, cur) in [(7u64, 1u64), (8, 1), (7, 0), (8, 0)] {
        let want = oracle(&mut s, &e, acct, cur).expect("the key exists");
        for sql in [
            format!("select acct, cur, sum(amt) from postings where acct = {acct} and cur = {cur} group by acct, cur"),
            format!("select acct, sum(amt) from postings where acct = {acct} and cur = {cur} group by acct"),
        ] {
            let before = view_answers(&mut s, &e);
            let got = ask(&mut s, &e, &sql).unwrap();
            assert_eq!(view_answers(&mut s, &e), before + 1, "{sql}: not answered from the view");
            assert_eq!(got.len(), 1, "{sql}: {got:?}");
            let value = &got[0][got[0].len() - 2];
            assert_eq!(value, &want.to_string(), "{sql}");
        }
    }
    // A key the base holds no posting on is no row, not a zero: account 3 posts only in usd.
    assert_eq!(oracle(&mut s, &e, 3, 1), None);
    let got = ask(
        &mut s,
        &e,
        "select acct, cur, sum(amt) from postings where acct = 3 and cur = 1 group by acct, cur",
    )
    .unwrap();
    assert!(got.is_empty(), "{got:?}");
    // The refusal that stays: one account summed across two currencies.
    let refused = ask(
        &mut s,
        &e,
        "select acct, sum(amt) from postings where acct = 7 group by acct",
    );
    assert!(refused.is_err(), "{refused:?}");
}
