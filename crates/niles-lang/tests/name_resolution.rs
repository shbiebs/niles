//! **What `names.rs` catches, what it deliberately does not, and the two lists it depends on.**
//!
//! `crates/niles-lang/src/names.rs` is the pass that answers *does this name denote
//! anything?* — a question the compiler could not answer until cycle 13. This file is its
//! specification. It has three jobs.
//!
//! 1. **The positives.** An unbound name and a call to an undeclared function are refused,
//!    with their own codes. The three Appendix C controls live in `calculus_mutants.rs`
//!    beside the other refusals of §4.5's rules; here they are exercised in the shapes the
//!    corpus actually contains.
//! 2. **The negatives, which matter more.** A pass that reported everything would satisfy
//!    job 1. Each of this pass's *stated limits* — query context, SQL subtrees, multi-segment
//!    paths, type positions — is a test here, so a limit cannot quietly become a defect, and
//!    a later cycle that narrows one has to come and delete the test that says it was wide.
//! 3. **The two lists.** `BUILTIN_FNS` names what the interpreter provides and `QUERY_FNS`
//!    names what lowering recognises. Both are second copies of information that lives
//!    elsewhere, and a second copy that nothing checks is the defect this repository has
//!    found more often than any other. They are checked against their sources below.

use niles_lang::{names, parser, resolve};

const SCHEMA: &str = "\
schema t {
    currency usd { scale: 2 }
    table accounts { id: Id<Account> primary key, owner: Text, tier: Int }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
    index ix_postings on postings (acct) anchor;
}
";

/// Every diagnostic code `resolve_program` emits for the schema plus `body`.
fn codes(body: &str) -> Vec<String> {
    let src = format!("{SCHEMA}\n{body}\n");
    let (prog, mut d) = parser::parse_program(&src);
    let (_cat, rd) = resolve::resolve_program(&prog, 0);
    d.extend(rd);
    assert!(
        !d.items.iter().any(|x| x.code.starts_with("NL00")),
        "the fixture must parse, or it tests nothing:\n{}",
        d.render(&src, "fixture.niles")
    );
    d.items.iter().map(|x| x.code.to_string()).collect()
}

fn has(body: &str, code: &str) -> bool {
    codes(body).iter().any(|c| c == code)
}

// ============ the positives ============

#[test]
fn an_unbound_name_is_refused() {
    assert!(has("fn f() -> Int { nowhere }", names::UNBOUND));
}

#[test]
fn a_call_to_an_undeclared_function_is_refused() {
    assert!(has("fn f() -> Int { nowhere(1) }", names::UNKNOWN_FN));
}

#[test]
fn a_name_bound_only_in_a_sibling_scope_is_refused() {
    // Scoping, not merely presence: a `let` inside one arm of an `if` does not escape it.
    // A pass built on a flat set of every name the file mentions would pass every other
    // test here and fail this one.
    assert!(has(
        "fn f(c: Bool) -> Int { if c { let only_here: Int = 1; only_here } else { only_here } }",
        names::UNBOUND
    ));
}

#[test]
fn a_loop_variable_does_not_outlive_its_loop() {
    assert!(has(
        "fn f() -> Int { for i in range(0, 3) { print(i); } i }",
        names::UNBOUND
    ));
}

// ============ the negatives ============

#[test]
fn a_parameter_a_let_and_a_closure_binding_are_all_in_scope() {
    assert!(!has(
        r#"fn f(p: Int) -> Int { let q: Int = p; q }"#,
        names::UNBOUND
    ));
    assert!(!has(
        "view v = postings.where(|r| r.amt == r.amt) serve { consistency: snapshot };",
        names::UNBOUND
    ));
}

#[test]
fn a_function_may_call_one_declared_below_it() {
    // Items are order-independent, as in Rust. A one-pass walk would report `later` here,
    // and the report would be an artefact of the compiler rather than a fact about the
    // language.
    assert!(!has(
        "fn early() -> Int { later() }\nfn later() -> Int { 1 }",
        names::UNKNOWN_FN
    ));
}

#[test]
fn a_relation_and_a_view_are_names() {
    assert!(!has(
        "view v = postings.where(|r| r.amt == 0.00 usd) serve { consistency: snapshot };\n\
         fn f() -> Int { let x: Int = 1; x }",
        names::UNBOUND
    ));
}

#[test]
fn a_shadowing_let_sees_the_outer_binding_on_its_right_hand_side() {
    // `let x = x;` refers to the outer `x`, which is what the interpreter does. A pass that
    // bound the pattern before walking the initializer would accept a program the
    // interpreter rejects, which is the wrong direction to be wrong in.
    assert!(has("fn f() -> Int { let x: Int = x; x }", names::UNBOUND));
    assert!(!has(
        "fn f(x: Int) -> Int { let x: Int = x; x }",
        names::UNBOUND
    ));
}

// ============ the stated limits ============
//
// Each of these asserts that something is *not* reported. They are the limits declared in
// `names.rs`'s header, written down where they can fail.

#[test]
fn limit_a_bare_name_in_query_context_is_a_column_not_a_binding() {
    // `lower::collect_order_keys` reads a bare path inside `order_by` as a column name, and
    // unknown columns are resolved against the relation by `resolve::check_view` — NL0400,
    // NL0403 and the `*_unknown_column` goldens. This pass must not answer that question a
    // second time and worse.
    let c = codes(
        "view v = postings.order_by(|r| asc(amt)).limit(3) \
         serve { consistency: snapshot };",
    );
    assert!(
        !c.iter().any(|x| x == names::UNBOUND),
        "a bare name in query context was reported as unbound: {c:?}"
    );
}

#[test]
fn limit_a_query_form_is_callable_only_inside_a_stage() {
    // `sum` is not a function and there is nowhere to declare one: `lower::aggregate_of`
    // matches the name and emits an operator. Inside a stage it is a form; in a function
    // body it denotes nothing, and saying so is the point of the pass.
    assert!(!has(
        "view v = postings.group_by(|r| r.acct).sum(|r| r.amt) \
         serve { consistency: snapshot };",
        names::UNKNOWN_FN
    ));
    assert!(has("fn f() -> Int { sum(1) }", names::UNKNOWN_FN));
}

#[test]
fn limit_a_multi_segment_path_is_not_judged() {
    // Resolving `a::b` needs the module and `use` graph, which this compiler does not
    // build. Reporting from the last segment alone would be a guess, and a guess in a
    // diagnostic teaches users to switch the diagnostic off.
    assert!(!has(
        "fn f() -> Int { let x: Int = 1; some_module::helper(x) }",
        names::UNKNOWN_FN
    ));
}

#[test]
fn limit_a_type_position_is_not_judged() {
    // Type names live in their own namespace with their own declarations and generics.
    assert!(!has(
        "fn f() -> Int { let x: NoSuchType = 1; 2 }",
        names::UNBOUND
    ));
}

// ============ the two lists, held to their sources ============

#[test]
fn every_builtin_this_pass_knows_is_one_the_interpreter_dispatches() {
    // `names::BUILTIN_FNS` is a second copy of `Interp::builtin_ledger` and
    // `Interp::builtin_free`. A name here that the interpreter does not have is a call that
    // resolves to nothing at run time; a name the interpreter has and this list does not is
    // a false NL0205 on correct code. Read from the interpreter's source rather than from a
    // list beside it, because a list beside it is the thing being checked.
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../niles-interp/src/lib.rs"
    ))
    .expect("the interpreter's source is readable");
    let mut missing = Vec::new();
    for b in names::BUILTIN_FNS {
        // The dispatch arms are written `("name", [..]) =>` / `("name", legs) =>`.
        if !src.contains(&format!("(\"{b}\", ")) {
            missing.push(*b);
        }
    }
    assert!(
        missing.is_empty(),
        "these are accepted as builtins and the interpreter does not dispatch them: {missing:?}"
    );
}

#[test]
fn every_query_form_this_pass_knows_is_one_lowering_recognises() {
    // `names::QUERY_FNS` is a second copy of the names `lower::collect_order_keys` and
    // `lower::aggregate_of` match on. A form here that lowering does not have would lower to
    // nothing with no diagnostic — which is F-09's shape exactly: `desc` was documented,
    // accepted, and silently sorted ascending.
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lower.rs"))
        .expect("lower.rs is readable");
    let mut missing = Vec::new();
    for q in names::QUERY_FNS {
        if !src.contains(&format!("\"{q}\" =>")) {
            missing.push(*q);
        }
    }
    assert!(
        missing.is_empty(),
        "these are accepted in query context and lowering does not recognise them: {missing:?}"
    );
}

#[test]
fn the_corpus_this_repository_ships_resolves() {
    // The acceptance criterion of cycle 13's L-1, as a test rather than as a sentence in a
    // report. `bootstrap/` is **one program in two files** — `bootstrap_parser.rs` says so
    // and concatenates them, because the parser consumes the token type the lexer defines —
    // so it is checked the way it is run. Checked separately, `parser.niles` reports `lex`
    // as undeclared, which is true of half a program and is not a finding about the file.
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let read = |p: &str| std::fs::read_to_string(format!("{root}/{p}")).expect(p);
    let cases: Vec<(String, String)> = vec![
        (
            "examples/available_balance.niles".into(),
            read("examples/available_balance.niles"),
        ),
        (
            "examples/demo_bank.niles".into(),
            read("examples/demo_bank.niles"),
        ),
        (
            "examples/inventory.niles".into(),
            read("examples/inventory.niles"),
        ),
        (
            "bootstrap/{lexer,parser}.niles".into(),
            format!(
                "{}\n{}\n",
                read("bootstrap/lexer.niles"),
                read("bootstrap/parser.niles")
            ),
        ),
    ];
    for (name, src) in cases {
        let (prog, _) = parser::parse_program(&src);
        let (_cat, d) = resolve::resolve_program(&prog, 0);
        let found: Vec<&str> = d
            .items
            .iter()
            .map(|x| x.code)
            .filter(|c| *c == names::UNBOUND || *c == names::UNKNOWN_FN)
            .collect();
        assert!(
            found.is_empty(),
            "{name} has names that denote nothing: {found:?}\n{}",
            d.render(&src, &name)
        );
    }
}
