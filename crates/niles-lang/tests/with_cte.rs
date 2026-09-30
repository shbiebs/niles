//! **`with` and `with recursive` on the SQL surface** (cycle 15, C15-05b; the author's
//! decision 4, built by decision R2-c), and the confidentiality rule through them.
//!
//! The golden corpus fixes what the forms denote (cases 54, 55, 71-75). This file holds what
//! the corpus cannot: that a CTE's name does not launder a sealed column past either checker,
//! that a recursion evaluates on the ledger's own shape, and what the lowering produces.

use niles_ir::eval;
use niles_ir::operator::Op;
use niles_lang::{lower, parser, resolve, typecheck};
use std::collections::BTreeMap;

const SCHEMA: &str = "schema s {
    base t { k: Int, v: Int, n: Int, retain forever; }
    base p { who: Int, owner: Int @confidential(e2ee, subject = who), n: Int, retain forever; }
    base parent { child: Int, parent: Int, retain forever; }
    index ix_t on t (k) anchor;
    index ix_p on p (who) anchor;
    index ix_parent on parent (child) anchor;
}
";

/// Front-end diagnostics (errors only), and the IR verifier's codes.
fn check(query: &str) -> (Vec<&'static str>, Vec<&'static str>, lower::Lowered) {
    let src = format!("{SCHEMA}\nview v = sql {{ {query} }};\n");
    let (prog, mut d) = parser::parse_program(&src);
    let (cat, r) = resolve::resolve_program(&prog, 0);
    d.extend(r);
    let (_, t) = typecheck::check_program(&prog, &cat);
    d.extend(t);
    let (l, ld) = lower::lower_program(&prog, &cat);
    d.extend(ld);
    let front = d
        .items
        .iter()
        .filter(|x| x.severity == niles_lang::diagnostics::Severity::Error)
        .map(|x| x.code)
        .collect();
    let ir = niles_ir::verify::verify(&l.circuit)
        .violations
        .iter()
        .map(|v| v.code)
        .collect();
    (front, ir, l)
}

#[test]
fn a_cte_name_stands_for_the_relations_it_reads() {
    // The confidentiality rule looks relations up by name; `x` is not one. Before this
    // cycle the `with` list was discarded and this was NL0500; built without the name
    // mapping, it would have been a filter on a sealed column with no NL0260.
    let (front, _, _) = check("with x as (select * from p) select * from x where owner = 1");
    assert!(front.contains(&"NL0260"), "{front:?}");
    let (front, _, _) = check(
        "with x as (select * from p), y as (select * from x) select * from y where owner = 1",
    );
    assert!(
        front.contains(&"NL0260"),
        "through a second entry: {front:?}"
    );
}

#[test]
fn a_renamed_sealed_column_is_refused_by_the_verifier_across_a_join() {
    // The column list renames `owner` to `b`, so the type checker's rule, which matches
    // names, does not see it — and this is where the verifier is the second checker. It
    // was not: every source had arity 0, so across a join it checked the wrong column.
    let (_, ir, _) = check(
        "with x(a, b, c) as (select * from p) select * from t join x on t.k = x.a where x.b = 1",
    );
    assert!(ir.contains(&"IR022"), "{ir:?}");
    // And the column it checked instead is no longer refused.
    let (_, ir, _) = check("select * from t join p on t.k = p.who where t.v = 1");
    assert!(!ir.contains(&"IR022"), "{ir:?}");
}

#[test]
fn a_recursive_cte_is_a_fixpoint_over_a_delay() {
    let (front, _, l) = check(
        "with recursive anc(party, ancestor) as (
             select child, parent from parent
             union
             select anc.party, p.parent from anc join parent p on anc.ancestor = p.child
         ) select * from anc",
    );
    assert!(front.is_empty(), "{front:?}");
    let ops: Vec<&str> = l.circuit.nodes.iter().map(|n| n.op.name()).collect();
    assert!(
        ops.contains(&"fixpoint") && ops.contains(&"delay"),
        "{ops:?}"
    );
    // A chain 1 -> 2 -> 3 -> 4 and a cycle 7 <-> 8: six pairs on the chain, four on the cycle.
    let mut s = BTreeMap::new();
    s.insert(
        "parent".to_string(),
        eval::zset(&[
            (&[1, 2], 1),
            (&[2, 3], 1),
            (&[3, 4], 1),
            (&[7, 8], 1),
            (&[8, 7], 1),
        ]),
    );
    let (z, _) = eval::try_run(&l.circuit, "v", &s).expect("converges");
    assert_eq!(z.len(), 10);
    assert!(z.values().all(|w| *w == 1), "a set, not a bag");
}

#[test]
fn a_recursion_that_does_not_converge_is_refused_at_run_time() {
    // Counting without a bound: the rows never stop being new. The round bound is the only
    // guard SQL gives the fixpoint, and it stops the evaluation rather than the epoch.
    let (front, _, l) =
        check("with recursive c(k) as (select k from t union select k + 1 from c) select * from c");
    assert!(front.is_empty(), "{front:?}");
    let mut s = BTreeMap::new();
    s.insert("t".to_string(), eval::zset(&[(&[1, 0, 0], 1)]));
    match eval::try_run(&l.circuit, "v", &s) {
        Err(eval::EvalError::NonTerminating { .. }) => {}
        other => panic!(
            "expected NonTerminating, got {:?}",
            other.map(|(z, _)| z.len())
        ),
    }
}

#[test]
fn the_steps_sql_does_not_admit_are_refused() {
    for (q, why) in [
        (
            "with recursive r(a) as (select k from t union select a from r where a in (select a from r)) select * from r",
            "the CTE inside a subquery",
        ),
        (
            "with recursive r(a, b) as (select k, v from t union select r.a, r2.b from r join r r2 on r.b = r2.a) select * from r",
            "the CTE twice",
        ),
        (
            "with recursive r(a, b) as (select k, v from t union select t.k, r.b from t left join r on t.k = r.a) select * from r",
            "the null-padded side of an outer join",
        ),
        (
            "with recursive r(a) as (select k from t union select a from r limit 3) select * from r",
            "a limit",
        ),
        (
            "with recursive r(a) as (select k from t union select a from r except select k from t) select * from r",
            "a second set operation",
        ),
        (
            "with recursive r(a) as (select a from r union select k from t) select * from r",
            "a base that reads the CTE",
        ),
    ] {
        let (front, _, _) = check(q);
        assert!(front.contains(&"NL0525"), "{why}: {front:?}");
    }
}

#[test]
fn a_recursive_list_entry_that_does_not_recurse_is_a_plain_one() {
    let (front, _, l) = check("with recursive x as (select * from t where k = 2) select * from x");
    assert!(front.is_empty(), "{front:?}");
    assert!(!l
        .circuit
        .nodes
        .iter()
        .any(|n| matches!(n.op, Op::Fixpoint { .. })));
}

#[test]
fn a_cte_shadows_a_relation_only_inside_its_statement() {
    // `t` is rebound inside the first view and is the base again in the second.
    let src = format!(
        "{SCHEMA}\nview a = sql {{ with t as (select * from p) select who from t }};\nview b = sql {{ select k from t }};\n"
    );
    let (prog, _) = parser::parse_program(&src);
    let (cat, _) = resolve::resolve_program(&prog, 0);
    let (l, d) = lower::lower_program(&prog, &cat);
    assert!(!d.has_errors(), "{:?}", d.items);
    let mut s = BTreeMap::new();
    s.insert("t".to_string(), eval::zset(&[(&[1, 10, 100], 1)]));
    s.insert(
        "p".to_string(),
        eval::zset(&[(&[5, 6, 7], 1), (&[8, 9, 10], 1)]),
    );
    let (za, _) = eval::try_run(&l.circuit, "a", &s).expect("a");
    let (zb, _) = eval::try_run(&l.circuit, "b", &s).expect("b");
    assert_eq!((za.len(), zb.len()), (2, 1));
}

/// **The round bound is a serve knob** (the author's decision R2-e). Counting to 1,100 needs
/// about 1,100 rounds: refused under the default bound of 1,000, answered when the view's
/// contract raises it. The bound is still a bound: zero and a word are NL0528.
#[test]
fn a_view_may_raise_its_recursions_round_bound() {
    let count = "with recursive c(k) as (select k from t union select k + 1 from c where k < 1100) select * from c";
    let mut s = BTreeMap::new();
    s.insert("t".to_string(), eval::zset(&[(&[1, 0, 0], 1)]));
    let lower = |contract: &str| {
        let src = format!("{SCHEMA}\nview v = sql {{ {count} }} {contract};\n");
        let (prog, _) = parser::parse_program(&src);
        let (cat, _) = resolve::resolve_program(&prog, 0);
        lower::lower_program(&prog, &cat)
    };
    let (l, d) = lower("");
    assert!(!d.has_errors(), "{:?}", d.items);
    assert!(matches!(
        eval::try_run(&l.circuit, "v", &s),
        Err(eval::EvalError::NonTerminating { rounds: 1000, .. })
    ));
    let (l, d) = lower("serve { consistency: snapshot, materialize: auto, max_rounds: 1200 }");
    assert!(!d.has_errors(), "{:?}", d.items);
    let (z, _) = eval::try_run(&l.circuit, "v", &s).expect("converges under the raised bound");
    assert_eq!(z.len(), 1100);
    for bad in ["max_rounds: 0", "max_rounds: many"] {
        let (_, d) = lower(&format!("serve {{ consistency: snapshot, {bad} }}"));
        assert!(
            d.items.iter().any(|x| x.code == "NL0528"),
            "{bad}: {:?}",
            d.items
        );
    }
}
