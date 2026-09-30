//! **A qualifier names a side of a join, on both surfaces** (cycle 14, R2-06; E30 finding F7).
//!
//! Column references resolved by name alone, first match, so every duplicated name in a
//! join's output was read off the left side. E30's Q07 and Q08 self-join `postings`:
//! `from postings p join postings q on q.cur = p.cur and q.txn < p.txn` lowered to
//! `Column(2) = Column(2) and Column(0) < Column(0)` — always false — and the pipeline's
//! `.join(postings, |l, r| r.cur == l.cur && r.txn < l.txn)` did the same, because its
//! residual was lowered against the left input only and dropped when it did not lower.
//!
//! Now a qualifier that names a side (a from-list alias, a join closure's parameter)
//! resolves inside that side's columns; a name that is on more than one side and is not
//! made one value by the join key is refused as NL0520; and a join residual that does not
//! lower refuses the join.

use niles_ir::eval;
use niles_ir::operator::{Op, Scalar};
use niles_lang::{lower, parser, resolve, typecheck};

const SCHEMA: &str = "schema s {
    currency usd { scale: 2 }
    table accounts { id: Id<Account> primary key, desk: Int }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
    index ix_p on postings (acct) anchor;
    index ix_a on accounts (id) anchor;
}
";

fn codes(view: &str) -> Vec<&'static str> {
    let src = format!("{SCHEMA}\n{view}\n");
    let (prog, mut d) = parser::parse_program(&src);
    let (cat, r) = resolve::resolve_program(&prog, 0);
    d.extend(r);
    let (_, t) = typecheck::check_program(&prog, &cat);
    d.extend(t);
    let (_, l) = lower::lower_program(&prog, &cat);
    d.extend(l);
    d.items
        .iter()
        .filter(|x| x.severity == niles_lang::diagnostics::Severity::Error)
        .map(|x| x.code)
        .collect()
}

/// The view's rows over three postings to account 1: txns 1 and 2 in currency 0, txn 3 in
/// currency 1. The self-join "an earlier posting in the same currency" has one row: 2 after 1.
fn rows(view: &str) -> Vec<String> {
    let src = format!("{SCHEMA}\n{view}\n");
    let (prog, _) = parser::parse_program(&src);
    let (cat, _) = resolve::resolve_program(&prog, 0);
    let (l, d) = lower::lower_program(&prog, &cat);
    assert!(!d.has_errors(), "{view}: {:?}", d.items);
    let mut sources = std::collections::BTreeMap::new();
    sources.insert(
        "postings".to_string(),
        eval::zset(&[
            (&[1, 1, 0, 5, 1], 1),
            (&[2, 1, 0, 7, 2], 1),
            (&[3, 1, 1, 9, 3], 1),
        ]),
    );
    sources.insert("accounts".to_string(), eval::zset(&[(&[1, 10], 1)]));
    let (z, _) = eval::try_run(&l.circuit, "v", &sources).expect("evaluates");
    z.iter().map(|(r, w)| format!("{r:?} x{w}")).collect()
}

const ONE_PAIR: &str =
    "[Int(2), Int(1), Int(0), Int(7), Int(2), Int(1), Int(1), Int(0), Int(5), Int(1)] x1";

#[test]
fn a_sql_self_join_reads_each_alias_off_its_own_side() {
    let got = rows(
        "view v = sql { select * from postings p join postings q on q.cur = p.cur and q.txn < p.txn };",
    );
    assert_eq!(got, vec![ONE_PAIR.to_string()]);
}

#[test]
fn a_pipeline_self_join_reads_each_parameter_off_its_own_side() {
    let got = rows("view v = postings.join(postings, |l, r| r.cur == l.cur && r.txn < l.txn);");
    assert_eq!(got, vec![ONE_PAIR.to_string()]);
}

/// A residual naming a right-only column used to be dropped (it did not lower against the
/// left input), so the join ran without it.
#[test]
fn a_pipeline_residual_on_the_right_side_is_kept() {
    let src = format!("{SCHEMA}\nview v = postings.join(accounts, |p, a| a.desk == 20);\n");
    let (prog, _) = parser::parse_program(&src);
    let (cat, _) = resolve::resolve_program(&prog, 0);
    let (l, d) = lower::lower_program(&prog, &cat);
    assert!(!d.has_errors(), "{:?}", d.items);
    let residual = l
        .circuit
        .nodes
        .iter()
        .find_map(|n| match &n.op {
            Op::Join { residual, .. } => Some(residual.clone()),
            _ => None,
        })
        .expect("a join");
    // postings has five columns, so `a.desk` is column 6.
    assert!(
        matches!(&residual, Some(Scalar::Binary { lhs, .. }) if **lhs == Scalar::Column(6)),
        "{residual:?}"
    );
    assert!(rows("view v = postings.join(accounts, |p, a| a.desk == 20);").is_empty());
}

#[test]
fn a_name_on_both_sides_is_refused_unless_the_key_makes_it_one_value() {
    for view in [
        // `amt` and `txn` are on both sides and the key is `acct`: which one?
        "view v = sql { select amt from postings p join postings q on q.cur = p.cur };",
        "view v = postings.join(postings).map(|r| r.txn);",
        "view v = postings.join(postings, |r| r.cur == r.cur);",
        // A qualifier that names no column of its side.
        "view v = sql { select p.txn from postings p join accounts a on a.txn = p.txn };",
        "view v = postings.join(accounts, |p, a| a.cur == p.cur);",
    ] {
        let c = codes(view);
        assert!(c.contains(&"NL0520"), "{view}: {c:?}");
    }
    // `acct` is the join key on both sides: one value, so unqualified is unambiguous.
    for view in [
        "view v = postings.join(postings).group_by(|r| r.acct).count(|r| r.acct);",
        "view v = sql { select p.txn, q.amt from postings p join postings q on q.txn < p.txn };",
    ] {
        let c = codes(view);
        assert!(c.is_empty(), "{view}: {c:?}");
    }
}
