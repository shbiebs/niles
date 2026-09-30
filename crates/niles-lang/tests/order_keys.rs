//! **An `order_by` key the lowering cannot read is refused** (cycle 14, R2-06).
//!
//! E30's Q05 wrote `.order_by(|r| (-r.sum, r.acct))`. The key walker skipped the negation it did
//! not recognise and kept `r.acct`, so `.limit(5)` returned the five lowest account ids: the
//! wrong rows, accepted. The negation is now refused (NL0509), and the spelling the surface
//! has — `desc(r.sum)` — lowers to a descending key.

use niles_ir::operator::Op;
use niles_lang::{lower, parser, resolve, typecheck};

const SCHEMA: &str = "schema s {
    currency usd { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
    index ix_p on postings (acct) anchor;
}
";

fn lowered(view: &str) -> (Vec<&'static str>, Vec<Op>) {
    let src = format!("{SCHEMA}\n{view}\n");
    let (prog, mut d) = parser::parse_program(&src);
    let (cat, r) = resolve::resolve_program(&prog, 0);
    d.extend(r);
    let (_, t) = typecheck::check_program(&prog, &cat);
    d.extend(t);
    let (l, ld) = lower::lower_program(&prog, &cat);
    d.extend(ld);
    let errors = d
        .items
        .iter()
        .filter(|x| x.severity == niles_lang::diagnostics::Severity::Error)
        .map(|x| x.code)
        .collect();
    (
        errors,
        l.circuit.nodes.iter().map(|n| n.op.clone()).collect(),
    )
}

#[test]
fn a_negated_key_in_a_tuple_is_refused() {
    let (errors, _) = lowered(
        "view v = postings.group_by(|p| p.acct).sum(|p| p.amt).order_by(|r| (-r.sum, r.acct)).limit(5);",
    );
    assert!(errors.contains(&"NL0509"), "{errors:?}");
}

#[test]
fn desc_in_a_tuple_lowers_to_a_descending_key_then_an_ascending_one() {
    let (errors, ops) = lowered(
        "view v = postings.group_by(|p| p.acct).sum(|p| p.amt).order_by(|r| (desc(r.sum), r.acct)).limit(5);",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let keys = ops
        .iter()
        .find_map(|o| match o {
            Op::OrderBy { keys } => Some(keys.clone()),
            _ => None,
        })
        .expect("an order_by");
    // The aggregate emits (acct, sum): sum is column 1, descending; acct column 0, ascending.
    assert_eq!(keys, vec![(1, false), (0, true)]);
}
