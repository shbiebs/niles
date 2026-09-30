//! **Window functions and the system-time column on the SQL surface** (cycle 15, C15-05b;
//! the author's decision 4, built by decision R2-c). The golden corpus fixes what the forms
//! denote (cases 76-83); this file holds the rules around them: confidentiality, money, the
//! refusals, and a system-time read the caller did not supply.

use niles_ir::eval::{self, EvalError};
use niles_ir::value::{Mismatch, Value};
use niles_lang::{lower, parser, resolve, typecheck};
use std::collections::BTreeMap;

const SCHEMA: &str = "schema s {
    currency usd { scale: 2 }
    currency eur { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
    base p { who: Int, owner: Int @confidential(e2ee, subject = who), n: Int, retain forever; }
    table notes { id: Int primary key, body: Int }
    index ix_postings on postings (acct) anchor;
    index ix_p on p (who) anchor;
}
";

fn lowered(query: &str) -> (Vec<&'static str>, lower::Lowered) {
    let src = format!("{SCHEMA}\nview v = sql {{ {query} }};\n");
    let (prog, mut d) = parser::parse_program(&src);
    let (cat, r) = resolve::resolve_program(&prog, 0);
    d.extend(r);
    let (_, t) = typecheck::check_program(&prog, &cat);
    d.extend(t);
    let (l, ld) = lower::lower_program(&prog, &cat);
    d.extend(ld);
    let codes = d
        .items
        .iter()
        .filter(|x| x.severity == niles_lang::diagnostics::Severity::Error)
        .map(|x| x.code)
        .collect();
    (codes, l)
}

/// postings(txn, acct, cur, amt, idem) and its system time: account 7 posts -5.00 usd at
/// epoch 1, +3.00 usd at epoch 2 and +1.00 eur at epoch 3.
fn sources(with_system_time: bool) -> BTreeMap<String, eval::ZSet> {
    let rows = [
        (1, 7, 0, -500, 1),
        (1, 8, 0, 500, 1),
        (2, 7, 0, 300, 2),
        (2, 8, 0, -300, 2),
        (3, 7, 1, 100, 3),
        (3, 8, 1, -100, 3),
    ];
    let (mut base, mut system) = (eval::ZSet::new(), eval::ZSet::new());
    for (txn, acct, cur, amt, epoch) in rows {
        let r = vec![
            Value::Int(txn),
            Value::Int(acct),
            Value::Int(cur),
            Value::Int(amt),
            Value::Int(txn),
        ];
        eval::add(&mut base, r.clone(), 1);
        let mut s = r;
        s.push(Value::Int(epoch));
        eval::add(&mut system, s, 1);
    }
    let mut m = BTreeMap::new();
    m.insert("postings".to_string(), base);
    if with_system_time {
        m.insert(niles_ir::operator::system_time_relation("postings"), system);
    }
    m
}

fn rows(z: &eval::ZSet) -> Vec<String> {
    let mut v: Vec<String> = z
        .iter()
        .map(|(r, w)| {
            let c: Vec<String> = r.iter().map(|x| x.to_string()).collect();
            format!("{} x{w}", c.join(" "))
        })
        .collect();
    v.sort();
    v
}

#[test]
fn a_running_balance_in_recording_order_is_money_per_currency() {
    // E30's Q07: a running balance per account and currency, in the order the postings were
    // recorded.
    let (codes, l) = lowered(
        "select acct, cur, sum(amt) over (partition by acct, cur order by recorded_at) as running from postings",
    );
    assert!(codes.is_empty(), "{codes:?}");
    let (z, _) = eval::try_run(&l.circuit, "v", &sources(true)).expect("answers");
    assert_eq!(
        rows(&z),
        [
            "7 0 -200 x1",
            "7 0 -500 x1",
            "7 1 100 x1",
            "8 0 200 x1",
            "8 0 500 x1",
            "8 1 -100 x1"
        ]
    );
    // And the running sums are money, in their partition's currency.
    let currencies: std::collections::BTreeSet<u32> = z
        .keys()
        .filter_map(|r| match r.last() {
            Some(Value::Money { currency, .. }) => Some(*currency),
            _ => None,
        })
        .collect();
    assert_eq!(
        currencies.len(),
        2,
        "usd and eur, each in its own partition"
    );
}

#[test]
fn a_running_sum_across_two_currencies_is_refused_at_run_time() {
    let (codes, l) = lowered(
        "select acct, sum(amt) over (partition by acct order by recorded_at) as running from postings",
    );
    assert!(codes.is_empty(), "{codes:?}");
    match eval::try_run(&l.circuit, "v", &sources(true)) {
        Err(EvalError::Mismatch {
            why: Mismatch::Currencies(_, _),
            ..
        }) => {}
        other => panic!(
            "expected a currency mismatch, got {:?}",
            other.map(|(z, _)| rows(&z))
        ),
    }
}

#[test]
fn a_system_time_read_the_caller_did_not_supply_is_refused() {
    let (codes, l) = lowered("select acct, recorded_at from postings");
    assert!(codes.is_empty(), "{codes:?}");
    match eval::try_run(&l.circuit, "v", &sources(false)) {
        Err(EvalError::MissingSystemTime { .. }) => {}
        other => panic!("expected a refusal, got {:?}", other.map(|(z, _)| rows(&z))),
    }
}

#[test]
fn a_table_has_no_system_time() {
    // A `table` is updated in place, so it has no epoch per row to read.
    let (codes, _) = lowered("select id, recorded_at from notes");
    assert!(
        !codes.is_empty(),
        "a table's `recorded_at` must not resolve"
    );
}

#[test]
fn a_window_over_a_sealed_column_is_refused_by_both_checkers() {
    let (codes, l) = lowered("select who, rank() over (order by owner) as r from p");
    assert!(codes.contains(&"NL0260"), "{codes:?}");
    let ir: Vec<&str> = niles_ir::verify::verify(&l.circuit)
        .violations
        .iter()
        .map(|v| v.code)
        .collect();
    assert!(ir.contains(&"IR022"), "{ir:?}");
}

#[test]
fn what_is_not_a_window_function_is_refused() {
    for q in [
        "select acct, lag(amt) over (order by acct) from postings",
        "select acct, rank(amt) over (order by acct) from postings",
    ] {
        let (codes, _) = lowered(q);
        assert!(codes.contains(&"NL0527"), "{q}: {codes:?}");
    }
    // A frame clause is not in the fragment, and is refused where it is written.
    let (codes, _) = lowered(
        "select acct, sum(amt) over (order by acct rows between unbounded preceding and current row) from postings",
    );
    assert!(codes.contains(&"NL0001"), "{codes:?}");
}
