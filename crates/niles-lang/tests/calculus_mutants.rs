//! **The calculus, tested by what it refuses.**
//!
//! Thesis §4.5 states the consistency-effect calculus as typing rules, and Theorem 4.4
//! (Niles soundness) says a well-typed program conserves money, cannot mismatch currencies,
//! and cannot overdraw without authorization. A theorem of that shape is only as strong as
//! the set of programs the checker actually rejects — and until this file existed, the
//! repository had no test that any particular ill-typed program *is* rejected. It had tests
//! that well-typed programs are accepted, which is the half a checker that accepts
//! everything also passes.
//!
//! Each file in `tests/mutants/` violates exactly one rule and carries, in its own header:
//!
//! ```text
//! // RULE:   the rule of §4.5 it violates
//! // EXPECT: the diagnostic code that must be emitted
//! // STAGE:  check | lower
//! ```
//!
//! The header is the specification and this file is the harness. A mutant whose expected
//! code stops being emitted fails here, by name, with the rule it was protecting printed in
//! the failure message.
//!
//! # Why the expected code is named rather than "some error"
//!
//! Because "the compiler rejected it" is not the claim. The claim is that a *particular*
//! rule rejects it, and a mutant that starts failing for an unrelated reason — a typo in the
//! schema, a parse error, a stricter rule elsewhere — would keep a green tick over a rule
//! that had quietly stopped working. Every case below asserts its own code, and
//! `every_mutant_is_refused_for_its_own_reason` additionally asserts that no mutant is
//! rejected *only* by a parse error.

use niles_lang::{lower, parser, resolve, typecheck};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mutants");

struct Mutant {
    name: String,
    rule: String,
    expect: String,
    stage: String,
    source: String,
}

fn header(src: &str, key: &str) -> Option<String> {
    src.lines()
        .take_while(|l| l.starts_with("//"))
        .find_map(|l| l.trim_start_matches("//").trim().strip_prefix(key))
        .map(|v| v.trim().to_string())
}

fn mutants() -> Vec<Mutant> {
    let schema = std::fs::read_to_string(format!("{DIR}/schema.niles")).expect("schema.niles");
    let mut out = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(DIR)
        .expect("mutants dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("niles"))
        .filter(|p| p.file_name().and_then(|x| x.to_str()) != Some("schema.niles"))
        .collect();
    // Sorted, so a failure list reads the same way on every machine.
    entries.sort();
    for p in entries {
        let body = std::fs::read_to_string(&p).expect("read mutant");
        let name = p.file_stem().unwrap().to_string_lossy().into_owned();
        let expect = header(&body, "EXPECT:")
            .unwrap_or_else(|| panic!("{name}: no `// EXPECT: NLxxxx` header"));
        let stage = header(&body, "STAGE:")
            .unwrap_or_else(|| panic!("{name}: no `// STAGE: check|lower` header"));
        let rule = header(&body, "RULE:").unwrap_or_else(|| panic!("{name}: no `// RULE:` header"));
        out.push(Mutant {
            name,
            rule,
            expect,
            stage,
            source: format!("{schema}\n{body}"),
        });
    }
    out
}

/// Every diagnostic code a full compile of `src` emits, with the rendered output for a
/// failure message.
fn compile(src: &str, stage: &str) -> (Vec<String>, String) {
    let (prog, mut diags) = parser::parse_program(src);
    let (cat, rdiags) = resolve::resolve_program(&prog, 0);
    diags.extend(rdiags);
    let (_report, tdiags) = typecheck::check_program(&prog, &cat);
    diags.extend(tdiags);
    if stage == "lower" {
        let (_lowered, ldiags) = lower::lower_program(&prog, &cat);
        diags.extend(ldiags);
    }
    let codes = diags.items.iter().map(|d| d.code.to_string()).collect();
    (codes, diags.render(src, "mutant.niles"))
}

#[test]
fn every_mutant_is_refused_with_its_own_code() {
    let ms = mutants();
    assert!(
        ms.len() >= 20,
        "the work order asks for at least 20 mutants; there are {}",
        ms.len()
    );
    let mut failures = Vec::new();
    for m in &ms {
        let (codes, rendered) = compile(&m.source, &m.stage);
        if !codes.contains(&m.expect) {
            failures.push(format!(
                "\n{}\n  rule:     {}\n  expected: {}\n  got:      {:?}\n{}",
                m.name, m.rule, m.expect, codes, rendered
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} mutants were not refused for their own reason:{}",
        failures.len(),
        ms.len(),
        failures.join("\n")
    );
}

#[test]
fn no_mutant_is_refused_only_by_a_parse_error() {
    // The failure mode this whole file is exposed to: a mutant with a typo is rejected, the
    // suite is green, and the rule it was written to protect is unguarded. A parse error
    // (NL00xx) means the mutant is not exercising the rule at all.
    for m in mutants() {
        let (codes, rendered) = compile(&m.source, &m.stage);
        let parse_errors: Vec<&String> = codes.iter().filter(|c| c.starts_with("NL00")).collect();
        assert!(
            parse_errors.is_empty(),
            "{} does not parse, so it tests nothing. Codes {:?}\n{}",
            m.name,
            parse_errors,
            rendered
        );
    }
}

#[test]
fn the_schema_the_mutants_share_is_itself_clean() {
    // The control for every case above. If the shared schema emitted an error of its own,
    // every mutant would be "refused" and the suite would be measuring nothing.
    let schema = std::fs::read_to_string(format!("{DIR}/schema.niles")).expect("schema.niles");
    let (codes, rendered) = compile(&schema, "lower");
    let errors: Vec<&String> = codes
        .iter()
        .filter(|c| c.starts_with("NL02") || c.starts_with("NL03") || c.starts_with("NL05"))
        .collect();
    assert!(
        errors.is_empty(),
        "the mutants' shared schema must compile clean: {errors:?}\n{rendered}"
    );
}

#[test]
fn the_well_typed_neighbour_of_each_mutant_is_accepted() {
    // **The negative control, and the one that matters most.** A checker that rejected every
    // program would pass every test above. These four are the mutants with one edit undone,
    // and each must compile with no error at all.
    let schema = std::fs::read_to_string(format!("{DIR}/schema.niles")).expect("schema.niles");
    let cases: &[(&str, &str)] = &[
        (
            "the currency matches at the call",
            r#"fn pay(from: Id<Account>, to: Id<Account>, m: Money<usd>) -> Result<TxnId, TxnError>
                   ! { append, debit<usd>, credit<usd> }
               { txn idem("pay") { post(debit(from, m)?, credit(to, m)) } }
               fn settle(a: Id<Account>, b: Id<Account>) -> Result<TxnId, TxnError>
                   ! { append, debit<usd>, credit<usd> }
               { pay(a, b, 10.00 usd) }"#,
        ),
        (
            "the capability is received rather than built",
            r#"fn f(a: Id<Account>, m: Money<usd>, auth: Auth<authorize<usd>>)
                   -> Result<(), TxnError>
                   ! { append, authorize<usd>, debit<usd>, credit<usd> }
               { authorize(a, m); txn idem("f") { post(debit(a, m)?, credit(a, m)) } }"#,
        ),
        (
            "the declared rung is the rung that is read",
            "fn honest() -> Money<usd> ! { read@bounded } { stale_balance.get(1) }",
        ),
        (
            "both halves are posted across the call",
            r#"fn both_halves(a: Id<Account>, b: Id<Account>) -> Result<(), TxnError>
                   ! { append, debit<usd>, credit<usd> }
               { post(debit(a, 10.00 usd)?, credit(b, 10.00 usd)) }
               fn caller(a: Id<Account>, b: Id<Account>) -> Result<TxnId, TxnError>
                   ! { append, debit<usd>, credit<usd> }
               { txn idem("ok") { both_halves(a, b) } }"#,
        ),
        (
            // The neighbour of `currency_laundered_by_an_annotation`, one token apart:
            // `eur` becomes `usd`. An annotation that agrees with what the checker can see
            // is still an annotation, and must not be an error.
            "the annotation agrees with the initializer",
            r#"fn launder(from: Id<Account>, to: Id<Account>) -> Result<TxnId, TxnError>
                   ! { append, debit<usd>, credit<usd> }
               { let m: Money<usd> = 10.00 usd;
                 txn idem("launder") { post(debit(from, m)?, credit(to, m)) } }"#,
        ),
        (
            // The neighbour of `currency_laundered_by_a_return_type`.
            "the return type agrees with the body",
            "fn fee() -> Money<usd> ! { } { 1.50 usd }",
        ),
        (
            // The neighbour of `currency_laundered_by_an_annotation` in its *other*
            // direction: an annotation over a value the checker cannot see into is the
            // programmer telling it something true, and remains accepted. Without this
            // case, NL0332 could be "fixed" by rejecting every annotation.
            "an annotation over a value the checker cannot see stands",
            r#"fn opaque(from: Id<Account>, to: Id<Account>, k: Int) -> Result<TxnId, TxnError>
                   ! { append, debit<usd>, credit<usd> }
               { let m: Money<usd> = rate_lookup(k);
                 txn idem("opaque") { post(debit(from, m)?, credit(to, m)) } }
               fn rate_lookup(k: Int) -> Money<usd> ! { } { 1.00 usd }"#,
        ),
        (
            // The neighbour of `idem_window_without_a_key`.
            "the window has a key to bound",
            r#"fn sweep(a: Id<Account>, b: Id<Account>, m: Money<usd>) -> Result<TxnId, TxnError>
                   ! { append, debit<usd>, credit<usd> }
               { txn idem("sweep") { post(debit(a, m)?, credit(b, m)) } }"#,
        ),
    ];
    for (what, src) in cases {
        let full = format!("{schema}\n{src}\n");
        let (codes, rendered) = compile(&full, "check");
        let errors: Vec<&String> = codes.iter().filter(|c| c.starts_with("NL0")).collect();
        // Warnings are codes too; only the ones the mutants assert on are errors here.
        let real: Vec<&&String> = errors
            .iter()
            .filter(|c| {
                [
                    "NL0216", "NL0252", "NL0253", "NL0255", "NL0300", "NL0310", "NL0311", "NL0312",
                    "NL0322", "NL0330", "NL0331", "NL0332",
                ]
                .contains(&c.as_str())
            })
            .collect();
        assert!(
            real.is_empty(),
            "`{what}` is well typed and must be accepted: {real:?}\n{rendered}"
        );
    }
}

#[test]
fn a_bounded_contract_carries_its_parameters_into_the_circuit() {
    // Not a mutant: the positive obligation F-47 leaves behind. `bounded(epochs: 8,
    // millis: 500)` and `bounded(epochs: 1, millis: 1)` must produce *different* contracts,
    // because both used to produce `{epochs: 4, millis: 1000}` and nothing said so.
    let schema = std::fs::read_to_string(format!("{DIR}/schema.niles")).expect("schema.niles");
    let contract_of = |src: &str| -> niles_ir::Consistency {
        let (prog, _) = parser::parse_program(src);
        let (cat, _) = resolve::resolve_program(&prog, 0);
        let (lowered, _) = lower::lower_program(&prog, &cat);
        let id = lowered.circuit.outputs["parameterised"];
        lowered.circuit.node(id).contract.get().consistency
    };
    let a = contract_of(&format!(
        "{schema}\nview parameterised = postings.group_by(|p| p.acct).sum(|p| p.amt)\n\
         serve {{ consistency: bounded(epochs: 8, millis: 500), materialize: auto }};\n"
    ));
    let b = contract_of(&format!(
        "{schema}\nview parameterised = postings.group_by(|p| p.acct).sum(|p| p.amt)\n\
         serve {{ consistency: bounded(epochs: 1, millis: 1), materialize: auto }};\n"
    ));
    assert_eq!(
        a,
        niles_ir::Consistency::Bounded {
            epochs: 8,
            millis: 500
        }
    );
    assert_eq!(
        b,
        niles_ir::Consistency::Bounded {
            epochs: 1,
            millis: 1
        }
    );
    assert_ne!(a, b, "two different bounds must not lower to one contract");

    // And the unparameterised spelling keeps a *named* default rather than a typed-in one.
    let bare = contract_of(&format!(
        "{schema}\nview parameterised = postings.group_by(|p| p.acct).sum(|p| p.amt)\n\
         serve {{ consistency: bounded, materialize: auto }};\n"
    ));
    assert_eq!(
        bare,
        niles_ir::Consistency::Bounded {
            epochs: resolve::Staleness::UNPARAMETERISED.epochs,
            millis: resolve::Staleness::UNPARAMETERISED.millis,
        }
    );
}
