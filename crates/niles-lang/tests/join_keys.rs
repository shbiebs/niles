//! **A keyed join whose keys differ in length is refused, on both surfaces** (cycle 14, R2-06).
//!
//! E30's Q03 joined `postings`, anchored on `(acct, cur)`, to `accounts`, anchored on `(id)`.
//! Both spellings were accepted and lowered to `Join { left_key: [1, 2], right_key: [0] }`,
//! which can never match, so the view answered no rows. Refused now as NL0519; the same join
//! with `postings` anchored on `(acct)` is accepted and keyed `[1]` against `[0]`.

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
fn keys_of_different_length_are_refused_on_both_surfaces() {
    for view in [PIPELINE, SQL] {
        let c = codes(&format!("{}\n{view}\n", schema("acct, cur")));
        assert!(c.contains(&"NL0519"), "{view}: {c:?}");
    }
}

#[test]
fn keys_of_the_same_length_are_accepted() {
    for view in [PIPELINE, SQL] {
        let c = codes(&format!("{}\n{view}\n", schema("acct")));
        assert!(c.is_empty(), "{view}: {c:?}");
    }
}
