//! **A string reaches the circuit only as a currency's name** (cycle 14, R2-06; E30 finding
//! F11, found by the first trial of the mutation stage).
//!
//! The IR's values are integers, and the one scalar evaluator — shared by the reference
//! evaluator and the server's scan fold — read every `LitText` as `0`. `0` is the first
//! declared currency's code, so `where cur = "eur"` answered the `usd` rows, and E30's mutant
//! swapping `"usd"` for `"eur"` in Q02 came back *equivalent*. A declared currency's name now
//! lowers to its code; any other string is refused (NL0521).

use niles_ir::eval;
use niles_lang::{lower, parser, resolve};

const SCHEMA: &str = "schema s {
    currency usd { scale: 2 }
    currency eur { scale: 2 }
    base t { k: Int, cur: Currency, name: Text, retain forever; }
    index ix_t on t (k) anchor;
}
";

fn lowered(view: &str) -> (lower::Lowered, niles_lang::diagnostics::Diagnostics) {
    let src = format!("{SCHEMA}\n{view}\n");
    let (prog, _) = parser::parse_program(&src);
    let (cat, _) = resolve::resolve_program(&prog, 0);
    lower::lower_program(&prog, &cat)
}

fn rows(view: &str) -> Vec<String> {
    let (l, d) = lowered(view);
    assert!(!d.has_errors(), "{view}: {:?}", d.items);
    let mut sources = std::collections::BTreeMap::new();
    // t(k, cur, name): row 1 in usd (code 0), row 2 in eur (code 1).
    sources.insert(
        "t".to_string(),
        eval::zset(&[(&[1, 0, 0], 1), (&[2, 1, 0], 1)]),
    );
    let (z, _) = eval::try_run(&l.circuit, "v", &sources).expect("evaluates");
    z.iter().map(|(r, w)| format!("{r:?} x{w}")).collect()
}

#[test]
fn a_currency_name_selects_that_currency_on_both_surfaces() {
    for view in [
        "view v = t.where(|r| r.cur == \"eur\");",
        "view v = sql { select * from t where cur = \"eur\" };",
    ] {
        assert_eq!(rows(view), ["[Int(2), Int(1), Int(0)] x1"], "{view}");
    }
}

#[test]
fn any_other_string_is_refused() {
    for view in [
        "view v = t.where(|r| r.name == \"alice\");",
        "view v = t.where(|r| r.cur == \"xyz\");",
        "view v = sql { select * from t where name like \"a%\" };",
    ] {
        let (_, d) = lowered(view);
        assert!(
            d.items.iter().any(|x| x.code == "NL0521"),
            "{view}: {:?}",
            d.items.iter().map(|x| x.code).collect::<Vec<_>>()
        );
    }
}
