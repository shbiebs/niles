//! **A keyed join whose keys differ in length is refused, on both surfaces** (cycle 14, R2-06).
//!
//! E30's Q03 joined `postings`, anchored on `(acct, cur)`, to `accounts`, anchored on `(id)`.
//! Both spellings were accepted and lowered to `Join { left_key: [1, 2], right_key: [0] }`,
//! which can never match, so the view answered no rows. Refused now as NL0519; the same join
//! with `postings` anchored on `(acct)` is accepted and keyed `[1]` against `[0]`.
//!
//! **Since cycle 15 (C15-05b) this holds for the pipeline surface only.** The SQL spelling's
//! keys used to be the anchors too, with `on` a residual, so `on acct = id` over a `(acct,
//! cur)` anchor was the same unmatched join. Its keys now come from `on`, which names one
//! column a side, so it is keyed `[1]` against `[0]` whatever the anchors are, and answers.

use niles_lang::{lower, parser, resolve, typecheck};

fn schema(anchor: &str) -> String {
    format!(
        "schema s {{
    currency usd {{ scale: 2 }}
    table accounts {{ id: Id<Account> primary key, desk: Int }}
    ledger postings {{
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }}
    index ix_p on postings ({anchor}) anchor;
    index ix_a on accounts (id) anchor;
}}
"
    )
}

fn codes(src: &str) -> Vec<&'static str> {
    let (prog, mut d) = parser::parse_program(src);
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

const PIPELINE: &str =
    "view v = postings.join(accounts).group_by(|r| (r.desk, r.cur)).sum(|r| r.amt);";
const SQL: &str =
    "view v = sql { select desk, cur, sum(amt) from postings join accounts on acct = id group by desk, cur };";

#[test]
fn keys_of_different_length_are_refused_on_the_pipeline_surface() {
    let c = codes(&format!("{}\n{PIPELINE}\n", schema("acct, cur")));
    assert!(c.contains(&"NL0519"), "{PIPELINE}: {c:?}");
}

/// The SQL spelling over the same `(acct, cur)` anchor: keyed by its `on`, and answering.
#[test]
fn the_sql_join_is_keyed_by_its_on_clause_not_by_the_anchors() {
    use niles_ir::eval;
    use niles_ir::operator::Op;
    let src = format!("{}\n{SQL}\n", schema("acct, cur"));
    assert!(codes(&src).is_empty(), "{:?}", codes(&src));
    let (prog, _) = parser::parse_program(&src);
    let (cat, _) = resolve::resolve_program(&prog, 0);
    let (l, _) = lower::lower_program(&prog, &cat);
    let keys = l
        .circuit
        .nodes
        .iter()
        .find_map(|n| match &n.op {
            Op::Join {
                left_key,
                right_key,
                residual,
                ..
            } => Some((left_key.clone(), right_key.clone(), residual.is_none())),
            _ => None,
        })
        .expect("a join");
    assert_eq!(keys, (vec![1], vec![0], true));
    // postings(txn, acct, cur, amt, idem); accounts(id, desk).
    let mut sources = std::collections::BTreeMap::new();
    sources.insert(
        "accounts".to_string(),
        eval::zset(&[(&[1, 10], 1), (&[2, 20], 1)]),
    );
    sources.insert(
        "postings".to_string(),
        eval::zset(&[(&[1, 1, 0, -5, 1], 1), (&[1, 2, 0, 5, 1], 1)]),
    );
    let (z, _) = eval::try_run(&l.circuit, "v", &sources).expect("evaluates");
    assert_eq!(
        z.len(),
        2,
        "one row per desk, where the anchor-keyed join answered none"
    );
}

#[test]
fn keys_of_the_same_length_are_accepted() {
    for view in [PIPELINE, SQL] {
        let c = codes(&format!("{}\n{view}\n", schema("acct")));
        assert!(c.is_empty(), "{view}: {c:?}");
    }
}

/// **A join to an aggregate matches on the aggregate's output key** (cycle 14, R2-06).
///
/// E30's Q04 wrote `accounts.left_join(postings.group_by(|p| p.acct).count(|p| p.txn))`. The
/// aggregate's derived key is its *input* grouping column (postings' `acct`, column 1), and the
/// join used it as if it indexed the aggregate's output — where column 1 is the count — so
/// account ids were matched against counts. Evaluated here on three accounts, two of which post.
#[test]
fn a_join_to_a_grouped_count_matches_on_the_grouping_column() {
    use niles_ir::eval;
    use niles_ir::operator::Op;
    let src = format!(
        "{}\nview v = accounts\n    .left_join(postings.group_by(|p| p.acct).count(|p| p.txn))\n    .where(|r| r.count is null)\n    .map(|r| r.id);\n",
        schema("acct")
    );
    let (prog, _) = parser::parse_program(&src);
    let (cat, _) = resolve::resolve_program(&prog, 0);
    let (l, d) = lower::lower_program(&prog, &cat);
    assert!(!d.has_errors());
    let join = l
        .circuit
        .nodes
        .iter()
        .find_map(|n| match &n.op {
            Op::Join {
                left_key,
                right_key,
                ..
            } => Some((left_key.clone(), right_key.clone())),
            _ => None,
        })
        .expect("a join");
    assert_eq!(
        join,
        (vec![0], vec![0]),
        "the count's key is its first output column"
    );
    // accounts(id, desk); postings(txn, acct, cur, amt, idem): accounts 1 and 3 post.
    let mut sources = std::collections::BTreeMap::new();
    sources.insert(
        "accounts".to_string(),
        eval::zset(&[(&[1, 10], 1), (&[2, 20], 1), (&[3, 30], 1)]),
    );
    sources.insert(
        "postings".to_string(),
        eval::zset(&[(&[1, 1, 0, -5, 1], 1), (&[1, 3, 0, 5, 1], 1)]),
    );
    let (z, _) = eval::try_run(&l.circuit, "v", &sources).expect("evaluates");
    let rows: Vec<String> = z.iter().map(|(r, w)| format!("{:?} x{w}", r)).collect();
    assert_eq!(
        rows,
        vec!["[Int(2)] x1".to_string()],
        "only account 2 has no posting"
    );
}
