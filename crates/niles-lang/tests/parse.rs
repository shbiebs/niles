//! Parser tests, in the UI-test style: a source, and an expectation about the diagnostics.
//!
//! The two properties defended here are resilience (one bad declaration must not destroy
//! the tree for the rest of the file) and the reservation policy (a novel keyword must
//! still work as a column name, which is what makes Niles adoptable against an existing
//! bank schema).

use niles_lang::ast::*;
use niles_lang::parser::{parse_expr, parse_program};

fn ok(src: &str) -> Program {
    let (p, d) = parse_program(src);
    assert!(!d.has_errors(), "expected a clean parse, got:\n{}", d.render(src, "test.niles"));
    p
}

#[test]
fn the_worked_example_parses() {
    let src = include_str!("../../../examples/demo_bank.niles");
    let (prog, d) = parse_program(src);
    assert!(!d.has_errors(), "the worked program must parse:\n{}", d.render(src, "demo_bank.niles"));
    assert!(prog.items.len() >= 5, "{}", prog.items.len());
    let Item::Schema(s) = &prog.items[0] else { panic!("first item is the schema") };
    assert_eq!(s.name.text, "demo_bank");
    let views: Vec<_> = s.items.iter().filter_map(|i| match i {
        SchemaItem::View(v) => Some(v),
        _ => None,
    }).collect();
    assert_eq!(views.len(), 3);
    assert!(views.iter().all(|v| v.contract.is_some()));
}

#[test]
fn a_ledger_is_a_base_with_a_conservation_rule() {
    let p = ok("schema s { ledger l { txn: TxnId, amt: Money, conserve per (txn, cur); retain forever; } }");
    let Item::Schema(s) = &p.items[0] else { panic!() };
    let SchemaItem::Base(r) = &s.items[0] else { panic!() };
    assert_eq!(r.kind, RelKind::Ledger);
    assert!(r.rules.iter().any(|x| matches!(x, RelRule::Conserve { .. })));
    assert!(r.rules.iter().any(|x| matches!(x, RelRule::Retain { .. })));
}

#[test]
fn novel_keywords_remain_usable_as_column_names() {
    // The whole point of the reservation policy: a bank whose ledger table already has
    // columns called `epoch`, `ledger`, `serve` and `budget` can adopt Niles without
    // renaming them. If this test fails, a keyword was promoted to reserved and the
    // registry needs a written justification.
    ok("schema s { table t { epoch: i64, ledger: Text, serve: Text, anchor: i64, budget: i64, scale: i32 } }");
}

#[test]
fn reserved_words_need_the_raw_escape() {
    let src = "schema s { table t { select: i64 } }";
    let (_, d) = parse_program(src);
    assert!(d.has_errors());
    let rendered = d.render(src, "t.niles");
    assert!(rendered.contains("r#select"), "must suggest the escape:\n{rendered}");
    ok("schema s { table t { r#select: i64 } }");
}

#[test]
fn recovery_keeps_the_rest_of_the_file() {
    let src = "schema good_one { table t { id: i64 } }\nfn broken( { }\nschema good_two { table u { id: i64 } }\n";
    let (prog, d) = parse_program(src);
    assert!(d.has_errors());
    let schemas: Vec<&str> = prog.items.iter().filter_map(|i| match i {
        Item::Schema(s) => Some(s.name.text.as_str()),
        _ => None,
    }).collect();
    assert!(schemas.contains(&"good_one") && schemas.contains(&"good_two"),
        "recovery lost a good declaration: {schemas:?}");
}

#[test]
fn both_pipeline_spellings_produce_one_node() {
    let (a, _) = parse_expr("postings.where(|p| p.amt > 0.00 usd)");
    let (b, _) = parse_expr("postings |> where(|p| p.amt > 0.00 usd)");
    let (Expr::Stage { kind: ka, .. }, Expr::Stage { kind: kb, .. }) = (&a, &b) else {
        panic!("both spellings must be Stage nodes")
    };
    assert_eq!(ka, kb);
    assert_eq!(*ka, StageKind::Where);
}

#[test]
fn precedence_follows_the_pratt_table() {
    let (e, d) = parse_expr("a + b * c == d and e");
    assert!(!d.has_errors());
    let Expr::Binary { op, .. } = &e else { panic!("{e:?}") };
    assert_eq!(*op, BinOp::And);
}

#[test]
fn an_unguarded_fixpoint_has_no_spelling() {
    let src = "edges.fixpoint(step)";
    let (_, d) = parse_expr(src);
    assert!(d.has_errors(), "unguarded recursion must be rejected at parse time");
    assert!(d.render(src, "t.niles").contains("NL0400"));
    let (e, d2) = parse_expr("edges.fixpoint(step) guard measure(depth)");
    assert!(!d2.has_errors());
    assert!(matches!(e, Expr::Fixpoint { .. }));
}

#[test]
fn a_hold_must_be_resolved_by_an_outcome() {
    for src in ["resolve h post 18.50 usd", "resolve h void", "resolve h expire"] {
        let (e, d) = parse_expr(src);
        assert!(!d.has_errors(), "{src}");
        assert!(matches!(e, Expr::Resolve { .. }));
    }
    let (_, d) = parse_expr("resolve h maybe");
    assert!(d.has_errors());
}

#[test]
fn unknown_stages_are_a_suggestion_not_a_parse_error() {
    let src = "postings.wher(|p| p)";
    let (e, d) = parse_expr(src);
    assert!(!d.has_errors(), "an unknown stage must still parse");
    assert!(matches!(e, Expr::Stage { kind: StageKind::Unknown, .. }));
    assert!(d.render(src, "t.niles").contains("did you mean `where`"));
}

#[test]
fn the_sql_surface_parses_to_a_select_node() {
    let (e, d) = parse_expr("sql { select acct, sum(amt) as bal from postings group by acct }");
    assert!(!d.has_errors(), "{:?}", d.items);
    let Expr::Sql { inner, .. } = &e else { panic!("{e:?}") };
    let Expr::Select(s) = &**inner else { panic!() };
    assert_eq!(s.projections.len(), 2);
    assert_eq!(s.group_by.len(), 1);
    assert_eq!(s.from.len(), 1);
}

#[test]
fn money_and_epoch_literals_survive_to_the_tree() {
    let (e, _) = parse_expr("10.00 usd");
    assert!(matches!(e, Expr::Money { minor: 1000, scale: 2, .. }));
    let (e, _) = parse_expr("#4200");
    assert!(matches!(e, Expr::Epoch(4200, _)));
    let (e, _) = parse_expr("v@2026-03-01");
    assert!(matches!(e, Expr::Instant { valid_axis: true, .. }));
}

#[test]
fn a_view_without_a_contract_warns_but_parses() {
    let src = "view v = postings.count();";
    let (prog, d) = parse_program(src);
    assert!(!d.has_errors());
    assert!(d.items.iter().any(|x| x.code == "NL0100"));
    assert!(matches!(&prog.items[0], Item::View(_)));
}

#[test]
fn effect_rows_are_part_of_the_signature() {
    let p = ok("fn f(a: i64) -> Result<(), E> ! { append, read@snapshot, debit<usd> } { Ok(()) }");
    let Item::Fn(f) = &p.items[0] else { panic!() };
    let row = f.effects.as_ref().expect("declared effects");
    assert_eq!(row.effects.len(), 3);
    assert_eq!(row.effects[1].at.as_ref().unwrap().text, "snapshot");
    assert_eq!(row.effects[2].args[0].text, "usd");
}

#[test]
fn parsing_is_total_on_arbitrary_input() {
    // A parser that can hang or panic is not usable in an editor. Anything at all must
    // produce a tree and terminate.
    for src in [
        "", "{", "}}}", "schema", "schema s {", "view v =", "fn f(", "((((((((((",
        "select select select", "10.00", "#", "@", "|>", "ledger l { conserve per (",
        "\u{0}\u{1}\u{2}", "txn { fx { leg", "resolve", "a.b.c.d.e(", "1 + + + 2",
    ] {
        let _ = parse_program(src);
    }
}
