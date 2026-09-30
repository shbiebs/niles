//! **Text values and run-time money currency in the IR** (cycle 15, C15-05b; the author's
//! decisions 6 and 8 of 2026-09-30, built by decision R2-c).
//!
//! * Decision 6: a string is a text value, with equality, ordering and `like`. Until cycle
//!   14 every string evaluated as `0` (E30's F11), and after that every string that was not a
//!   currency's name was refused (NL0521). NL0521 now refuses only a string that is still
//!   wrong: an undeclared currency against a currency column, or a string against a column
//!   that is neither text nor currency.
//! * Decision 8: money carries its currency. A money literal met an amount as a number, so
//!   `having sum(amt) < 0.00 eur` over usd rows answered (E30's F13). The amount is now
//!   lowered with its currency, and a comparison across two currencies is refused at run
//!   time. Where the lowering cannot find the currency, NL0522 refuses the comparison.

use niles_ir::eval::{self, EvalError, ZSet};
use niles_ir::value::{Mismatch, Value};
use niles_lang::{lower, parser, resolve};
use std::collections::BTreeMap;

const SCHEMA: &str = "schema s {
    currency usd { scale: 2 }
    currency eur { scale: 2 }
    base t { k: Int, cur: Currency, name: Text, retain forever; }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
    base fees { k: Int, fee: Money<usd>, retain forever; }
    base loose { k: Int, amt: Money, retain forever; }
    index ix_t on t (k) anchor;
    index ix_postings on postings (acct) anchor;
    index ix_fees on fees (k) anchor;
    index ix_loose on loose (k) anchor;
}
";

fn lowered(view: &str) -> (lower::Lowered, niles_lang::diagnostics::Diagnostics) {
    let src = format!("{SCHEMA}\n{view}\n");
    let (prog, _) = parser::parse_program(&src);
    let (cat, _) = resolve::resolve_program(&prog, 0);
    lower::lower_program(&prog, &cat)
}

fn codes(view: &str) -> Vec<String> {
    lowered(view)
        .1
        .items
        .iter()
        .map(|d| d.code.to_string())
        .collect()
}

fn sources() -> BTreeMap<String, ZSet> {
    let mut s = BTreeMap::new();
    // t(k, cur, name): usd is code 0, eur code 1.
    let mut t = ZSet::new();
    for (k, c, n) in [(1, 0, "alice"), (2, 1, "bob"), (3, 0, "alfred")] {
        eval::add(
            &mut t,
            vec![Value::Int(k), Value::Int(c), Value::text(n)],
            1,
        );
    }
    s.insert("t".to_string(), t);
    // postings(txn, acct, cur, amt, idem): account 7 is -5.00 usd and +3.00 eur.
    let mut p = ZSet::new();
    for (txn, acct, cur, amt) in [
        (1, 7, 0, -500),
        (1, 8, 0, 500),
        (2, 7, 1, 300),
        (2, 8, 1, -300),
    ] {
        eval::add(
            &mut p,
            vec![
                Value::Int(txn),
                Value::Int(acct),
                Value::Int(cur),
                Value::Int(amt),
                Value::Int(txn),
            ],
            1,
        );
    }
    s.insert("postings".to_string(), p);
    s
}

fn run(view: &str) -> Result<Vec<String>, EvalError> {
    let (l, d) = lowered(view);
    assert!(!d.has_errors(), "{view}: {:?}", d.items);
    let (z, _) = eval::try_run(&l.circuit, "v", &sources())?;
    let mut rows: Vec<String> = z
        .iter()
        .map(|(r, w)| {
            let cells: Vec<String> = r.iter().map(|v| v.to_string()).collect();
            format!("{} x{w}", cells.join(" "))
        })
        .collect();
    rows.sort();
    Ok(rows)
}

// ---------------------------------------------------------------- decision 6: text

#[test]
fn a_string_against_a_text_column_is_text() {
    for view in [
        "view v = t.where(|r| r.name == \"alice\");",
        "view v = sql { select * from t where name = \"alice\" };",
    ] {
        assert_eq!(run(view).unwrap(), ["1 0 alice x1"], "{view}");
    }
}

#[test]
fn like_matches_text_by_pattern() {
    assert_eq!(
        run("view v = sql { select * from t where name like \"al%\" };").unwrap(),
        ["1 0 alice x1", "3 0 alfred x1"]
    );
    assert_eq!(
        run("view v = sql { select * from t where name like \"_ob\" };").unwrap(),
        ["2 1 bob x1"]
    );
}

#[test]
fn text_orders_by_content() {
    assert_eq!(
        run("view v = sql { select * from t where name < \"b\" };").unwrap(),
        ["1 0 alice x1", "3 0 alfred x1"]
    );
}

#[test]
fn a_currency_name_is_still_a_currency_code_against_a_currency_column() {
    assert_eq!(
        run("view v = sql { select * from t where cur = \"eur\" };").unwrap(),
        ["2 1 bob x1"]
    );
}

#[test]
fn nl0521_now_refuses_only_what_is_still_wrong() {
    // An undeclared currency against a currency column, and a string against an integer.
    for view in [
        "view v = t.where(|r| r.cur == \"xyz\");",
        "view v = sql { select * from t where k = \"one\" };",
    ] {
        assert!(codes(view).contains(&"NL0521".to_string()), "{view}");
    }
}

// ---------------------------------------------------------------- decision 8: money

#[test]
fn a_money_literal_in_the_amounts_currency_compares() {
    let view = "view v = sql { select acct, sum(amt) from postings where cur = \"usd\" group by acct having sum(amt) < 0.00 usd };";
    assert_eq!(run(view).unwrap(), ["7 -500 x1"]);
}

#[test]
fn a_money_literal_in_another_currency_is_refused_at_run_time() {
    // E30's F13: this answered `7 -500` before, comparing the numbers.
    let view = "view v = sql { select acct, sum(amt) from postings where cur = \"usd\" group by acct having sum(amt) < 0.00 eur };";
    match run(view) {
        Err(EvalError::Mismatch {
            why: Mismatch::Currencies(_, _),
            ..
        }) => {}
        other => panic!("expected a currency mismatch, got {other:?}"),
    }
}

#[test]
fn a_row_carries_its_own_currency_where_nothing_pins_one() {
    // `cur` is a column of each row, so each amount is tagged with its own currency: the usd
    // rows meet an eur literal and the comparison is refused.
    match run("view v = sql { select * from postings where amt > 1.00 eur };") {
        Err(EvalError::Mismatch {
            why: Mismatch::Currencies(_, _),
            ..
        }) => {}
        other => panic!("expected a currency mismatch, got {other:?}"),
    }
    assert_eq!(
        run("view v = sql { select * from postings where cur = \"eur\" and amt > 1.00 eur };")
            .unwrap(),
        ["2 7 1 300 2 x1"]
    );
}

#[test]
fn a_declared_money_currency_is_its_currency() {
    let codes_ok = codes("view v = fees.where(|r| r.fee > 1.00 usd);");
    assert!(codes_ok.is_empty(), "{codes_ok:?}");
}

#[test]
fn an_amount_whose_currency_cannot_be_found_is_nl0522() {
    assert!(codes("view v = loose.where(|r| r.amt > 1.00 usd);").contains(&"NL0522".to_string()));
}

#[test]
fn in_a_self_join_an_amount_takes_its_own_sides_currency() {
    // A pin on `q`'s own currency governs `q.amt`: transaction 1's legs are usd, and the
    // positive one is account 8's, met by both of `p`'s legs.
    let view = "view v = sql { select q.acct from postings p join postings q on p.txn = q.txn where q.cur = \"usd\" and q.amt > 1.00 usd };";
    assert_eq!(run(view).unwrap(), ["8 x2"]);
    // A pin on `p`'s currency does not. Account 7 has a usd leg (txn 1) and an eur leg
    // (txn 2); `q` is its eur leg, so `q.amt > 1.00 usd` compares eur 3.00 with a usd
    // literal and is refused. The lookup this replaces took the most recent pin on *any*
    // `postings.cur` — here `p.cur = "usd"` — and answered `7`.
    let view = "view v = sql { select q.acct from postings p join postings q on p.acct = q.acct where q.cur = \"eur\" and p.cur = \"usd\" and q.amt > 1.00 usd };";
    match run(view) {
        Err(EvalError::Mismatch {
            why: Mismatch::Currencies(_, _),
            ..
        }) => {}
        other => panic!("expected a currency mismatch, got {other:?}"),
    }
}
