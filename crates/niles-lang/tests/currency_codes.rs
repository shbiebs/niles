//! **A currency's code is its declaration index, in the IR and on the wire — A9-F05.**
//!
//! The code is written into the ledger, where it is permanent, so it has to be a property of
//! the schema and not of the process that compiled it. It was derived in two places and two
//! ways: the wire sorted the catalog's currencies by source span; the compiler took
//! `cat.currencies.keys().position(..)` over a `HashMap` with the standard hasher, whose
//! iteration order is seeded per process. With one declared currency both answer zero, which
//! is why every schema this project serves hid it.
//!
//! The adversary is that seed. A test that resolves one program once cannot see the defect —
//! it would have to be unlucky. So this resolves the *same declarations* many times over,
//! in one process and across a shape the hasher treats differently, and requires the answer
//! to be identical every time and equal to declaration order.

use niles_lang::{lower, parser, resolve};

const SCHEMA: &str = r#"
schema treasury {
    currency zar { scale: 2 }
    currency jpy { scale: 0 }
    currency bhd { scale: 3 }
    currency usd { scale: 2 }

    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
}
"#;

/// Declaration order, which is what every consumer must agree on.
const DECLARED: [(&str, u32, u32); 4] =
    [("zar", 0, 2), ("jpy", 1, 0), ("bhd", 2, 3), ("usd", 3, 2)];

#[test]
fn the_catalog_numbers_currencies_by_declaration_and_not_by_hash_order() {
    // Many resolutions in one process. `HashMap`'s iteration order depends on a per-process
    // seed *and* on insertion history, so building the catalog repeatedly and reading it
    // back is what would expose an order-derived code.
    for round in 0..200 {
        let (prog, d) = parser::parse_program(SCHEMA);
        assert!(!d.has_errors(), "round {round}: {:?}", d.sorted());
        let (cat, rd) = resolve::resolve_program(&prog, 0);
        assert!(!rd.has_errors(), "round {round}: {:?}", rd.sorted());
        for (name, code, scale) in DECLARED {
            let c = cat
                .currencies
                .get(name)
                .unwrap_or_else(|| panic!("round {round}: `{name}` is declared"));
            assert_eq!(
                c.code, code,
                "round {round}: `{name}` is the currency declared at position {code}, and its \
                 code must be {code} in every process that ever compiles this schema"
            );
            assert_eq!(c.scale, scale, "round {round}: `{name}`'s scale");
        }
    }
}

/// The IR carries the same integer the catalog assigned.
#[test]
fn a_money_literal_lowers_to_its_declaration_index() {
    use niles_ir::operator::Scalar;

    for round in 0..50 {
        let program = format!(
            "{SCHEMA}\nview m = sql {{ select 1 as one from postings where amt > 5 bhd }} \
             serve {{ consistency: snapshot, materialize: auto }};\n"
        );
        let (prog, d) = parser::parse_program(&program);
        assert!(!d.has_errors(), "round {round}: {:?}", d.sorted());
        let (cat, rd) = resolve::resolve_program(&prog, 0);
        assert!(!rd.has_errors(), "round {round}: {:?}", rd.sorted());
        let (lowered, ld) = lower::lower_program(&prog, &cat);
        assert!(!ld.has_errors(), "round {round}: {:?}", ld.sorted());

        let mut found = Vec::new();
        for node in &lowered.circuit.nodes {
            collect_money(&node.op, &mut found);
        }
        assert!(
            found.contains(&2),
            "round {round}: `5 bhd` must lower to currency 2 — `bhd` is the third \
             declaration — and the circuit carries {found:?}"
        );
    }

    /// Every `LitMoney` currency code reachable from an operator.
    fn collect_money(op: &niles_ir::operator::Op, out: &mut Vec<u32>) {
        use niles_ir::operator::Op;
        fn scalar(s: &Scalar, out: &mut Vec<u32>) {
            match s {
                Scalar::LitMoney { currency, .. } => out.push(*currency),
                Scalar::Binary { lhs, rhs, .. } => {
                    scalar(lhs, out);
                    scalar(rhs, out);
                }
                Scalar::Not(a) | Scalar::Neg(a) | Scalar::IsNull(a) => scalar(a, out),
                _ => {}
            }
        }
        match op {
            Op::Filter { predicate } => scalar(predicate, out),
            Op::Map { exprs, .. } => exprs.iter().for_each(|e| scalar(e, out)),
            _ => {}
        }
    }
}
