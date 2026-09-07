//! End-to-end tests: Niles source text → typed IR.
//!
//! These are the tests that make the thesis's claims checkable rather than merely stated.
//! Three of them correspond directly to named results:
//!
//! * `the_two_surfaces_lower_to_the_same_circuit` is the operational form of the
//!   generality claim. If the SQL surface and the pipeline surface produced different
//!   circuits, "the SQL surface is the same language" would be marketing.
//! * `conservation_is_decided_statically` and its siblings demonstrate the currency-row
//!   solver on source text, which is the first half of Contribution 4.
//! * `rung_monotonicity_is_enforced_on_real_source` demonstrates the effect checker, which
//!   is the second half.

use niles_ir::{upquery_path, verify};
use niles_lang::{lower, parser, resolve, typecheck};

/// Compile a source string all the way to a verified circuit.
struct Compiled {
    diags: String,
    error_codes: Vec<&'static str>,
    warn_codes: Vec<&'static str>,
    circuit: niles_ir::circuit::Circuit,
    proved: usize,
    runtime_obligations: usize,
}

fn compile(src: &str) -> Compiled {
    let (prog, mut d) = parser::parse_program(src);
    let (cat, rd) = resolve::resolve_program(&prog, 0);
    d.extend(rd);
    let (report, td) = typecheck::check_program(&prog, &cat);
    d.extend(td);
    let (lowered, ld) = lower::lower_program(&prog, &cat);
    d.extend(ld);
    let error_codes = d
        .items
        .iter()
        .filter(|x| x.severity == niles_lang::diagnostics::Severity::Error)
        .map(|x| x.code)
        .collect();
    let warn_codes = d
        .items
        .iter()
        .filter(|x| x.severity == niles_lang::diagnostics::Severity::Warning)
        .map(|x| x.code)
        .collect();
    Compiled {
        diags: d.render(src, "t.niles"),
        error_codes,
        warn_codes,
        circuit: lowered.circuit,
        proved: report.conservation_proved,
        runtime_obligations: report.runtime_obligations,
    }
}

const SCHEMA: &str = "\
schema bank {
    currency usd { scale: 2 }
    currency eur { scale: 2 }
    currency jpy { scale: 0 }
    table accounts { id: Id<Account> primary key, owner: Text @confidential(e2ee, subject = id) }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
    index by_acct on postings (acct, cur) anchor;
";

/// The schema above, plus `body`. The body closes the schema's brace itself: a helper
/// that guessed at brace balance would make a test failure look like a compiler bug.
fn with_body(body: &str) -> String {
    format!("{SCHEMA}{body}\n")
}

// ============ the worked example ============

#[test]
fn the_worked_example_compiles_clean_and_verifies() {
    let src = include_str!("../../../examples/demo_bank.niles");
    let c = compile(src);
    assert!(
        c.error_codes.is_empty(),
        "the worked example must compile clean:\n{}",
        c.diags
    );
    let r = verify::verify(&c.circuit);
    assert!(r.is_ok(), "and its circuit must verify:\n{}", r.render());
    assert!(
        c.proved >= 3,
        "it should prove its conservation obligations, proved {}",
        c.proved
    );
    assert_eq!(
        c.runtime_obligations, 0,
        "and discharge none to the runtime"
    );
}

#[test]
fn every_view_in_the_worked_example_has_an_anchored_upquery_path() {
    // The property Contribution 1 rests on, checked on real source rather than a
    // hand-built circuit: every demand-materialized view can be reconstructed from a
    // frozen prefix of an immutable base.
    let src = include_str!("../../../examples/demo_bank.niles");
    let c = compile(src);
    for (name, id) in &c.circuit.outputs {
        let p = upquery_path::derive(&c.circuit, *id, 4200)
            .unwrap_or_else(|e| panic!("view `{name}` has no upquery path: {}", e.explain()));
        assert!(p.is_anchored(), "view `{name}`: {}", p.render());
        assert_eq!(p.anchor, 4200, "one epoch anchors the whole path");
    }
}

// ============ the generality claim ============

#[test]
fn the_two_surfaces_lower_to_the_same_circuit() {
    // The operational form of the generality claim. Both spellings of one query must
    // produce the same operators over the same keys.
    let pipeline = with_body(
        "    view v = postings.group_by(|p| (p.acct, p.cur)).sum(|p| p.amt)
        serve { consistency: snapshot, materialize: auto };
}",
    );
    let sql = with_body(
        "    view v = sql { select acct, cur, sum(amt) as bal from postings group by acct, cur }
        serve { consistency: snapshot, materialize: auto };
}",
    );
    let (a, b) = (compile(&pipeline), compile(&sql));
    assert!(a.error_codes.is_empty(), "{}", a.diags);
    assert!(b.error_codes.is_empty(), "{}", b.diags);

    let shape = |c: &Compiled| -> Vec<String> {
        let id = c.circuit.outputs["v"];
        let live = c.circuit.live_nodes();
        let mut v: Vec<String> = live
            .iter()
            .map(|i| {
                let n = c.circuit.node(*i);
                format!("{} key={:?}", n.op, n.key)
            })
            .collect();
        v.sort();
        let _ = id;
        v
    };
    assert_eq!(
        shape(&a),
        shape(&b),
        "the SQL surface and the pipeline surface must lower to the same circuit:\n{:#?}\nvs\n{:#?}",
        shape(&a),
        shape(&b)
    );
}

// ============ the currency-row solver, on source text ============

#[test]
fn a_balanced_transfer_is_proved_conserving() {
    let c = compile(&with_body(
        "}
fn t(a: Id<Account>, b: Id<Account>) -> Result<(), E> ! { append, debit<usd>, credit<usd> } {
    txn idem(\"k\") { let d = debit(a, 100.00 usd)?; let x = credit(b, 100.00 usd); post(d, x) }
}",
    ));
    assert!(c.error_codes.is_empty(), "{}", c.diags);
    assert_eq!(c.proved, 1, "one conservation obligation, proved");
}

#[test]
fn an_unbalanced_transfer_is_rejected_with_the_residue() {
    let c = compile(&with_body(
        "}
fn t(a: Id<Account>, b: Id<Account>) -> Result<(), E> ! { append, debit<usd>, credit<usd> } {
    txn idem(\"k\") { let d = debit(a, 100.00 usd)?; let x = credit(b, 60.00 usd); post(d, x) }
}",
    ));
    assert!(c.error_codes.contains(&"NL0300"), "{}", c.diags);
    assert!(
        c.diags.contains("-40.00"),
        "the residue must be named in minor units at the currency's scale:\n{}",
        c.diags
    );
    assert!(
        c.diags.contains("conserve per"),
        "and the rule that forbids it must be shown:\n{}",
        c.diags
    );
}

#[test]
fn conservation_is_per_currency_and_never_netted_across_them() {
    // The failure the whole design exists to prevent: amounts that sum to zero *across*
    // currencies, which is conservation in no ledger.
    let c = compile(&with_body(
        "}
fn t(a: Id<Account>, b: Id<Account>) -> Result<(), E> ! { append, debit<usd>, credit<eur> } {
    txn idem(\"k\") { let d = debit(a, 100.00 usd)?; let x = credit(b, 100.00 eur); post(d, x) }
}",
    ));
    let violations = c.error_codes.iter().filter(|x| **x == "NL0300").count();
    assert_eq!(
        violations, 2,
        "both currencies must be reported, not netted:\n{}",
        c.diags
    );
}

#[test]
fn adding_two_currencies_has_no_well_typed_spelling() {
    let c = compile(&with_body(
        "}
fn t() -> Money<usd> { 10.00 usd + 5.00 eur }
",
    ));
    assert!(c.error_codes.contains(&"NL0250"), "{}", c.diags);
    assert!(
        c.diags.contains("use an `fx`"),
        "the error must point at the construct that does work:\n{}",
        c.diags
    );
}

#[test]
fn an_fx_form_conserves_each_leg_independently() {
    let c = compile(&with_body(
        "}
fn t(a: Id<Account>, b: Id<Account>, r: Rate) -> Result<(), E>
    ! { append, debit<usd>, credit<usd>, debit<eur>, credit<eur> } {
    txn idem(\"k\") {
        fx {
            leg u: post(debit(a, 100.00 usd)?, credit(b, 100.00 usd)),
            leg e: post(debit(b, 92.00 eur)?, credit(a, 92.00 eur)),
            rate: r,
        }
    }
}
",
    ));
    assert!(c.error_codes.is_empty(), "{}", c.diags);
    assert!(
        c.proved >= 2,
        "each leg is its own obligation, proved {} ",
        c.proved
    );
}

#[test]
fn a_money_literal_at_the_wrong_scale_is_an_error_not_a_rounding() {
    let c = compile(&with_body("}\nfn t() -> Money<jpy> { 100.50 jpy }\n"));
    assert!(c.error_codes.contains(&"NL0240"), "{}", c.diags);
    assert!(
        c.diags.contains("scale 0"),
        "the declared scale must be shown:\n{}",
        c.diags
    );
}

#[test]
fn an_amount_the_checker_cannot_see_is_undecided_not_violating() {
    // The honest boundary. A checker that accused every program it could not follow would
    // be useless; one that assumed the good case would make the theorem vacuous.
    let c = compile(&with_body(
        "}
fn t(a: Id<Account>, m: Money<usd>) -> Result<(), E> ! { append, debit<usd> } {
    txn idem(\"k\") { let d = debit(a, m)?; post(d) }
}",
    ));
    assert!(
        !c.error_codes.contains(&"NL0300"),
        "an opaque amount must not be accused:\n{}",
        c.diags
    );
    assert_eq!(
        c.runtime_obligations, 1,
        "it must be counted as discharged to the runtime instead"
    );
}

// ============ the effect checker, on source text ============

#[test]
fn rung_monotonicity_is_enforced_on_real_source() {
    let c = compile(&with_body(
        "    view cheap = postings.group_by(|p| p.acct).sum(|p| p.amt)
        serve { consistency: bounded(epochs: 8), materialize: auto };
    view strict = cheap.group_by(|r| r.acct).sum(|r| r.sum)
        serve { consistency: ledger_consistent, materialize: demand };
}",
    ));
    assert!(
        c.error_codes.contains(&"NL0311"),
        "a strict view over a stale one must be rejected:\n{}",
        c.diags
    );
    assert!(
        c.diags.contains("no fresher than its stalest input"),
        "{}",
        c.diags
    );
}

#[test]
fn the_safe_direction_of_rung_monotonicity_is_permitted() {
    let c = compile(&with_body(
        "    view strict = postings.group_by(|p| p.acct).sum(|p| p.amt)
        serve { consistency: ledger_consistent, materialize: demand };
    view cheap = strict.group_by(|r| r.acct).sum(|r| r.sum)
        serve { consistency: bounded(epochs: 8), materialize: auto };
}",
    ));
    assert!(
        !c.error_codes.contains(&"NL0311"),
        "a weak view over a strict one is fine:\n{}",
        c.diags
    );
}

#[test]
fn an_undeclared_effect_is_reported_at_the_site_that_incurs_it() {
    let c = compile(&with_body(
        "}
fn t(a: Id<Account>) -> Result<(), E> ! { append } {
    txn idem(\"k\") { let d = debit(a, 10.00 usd)?; let x = credit(a, 10.00 usd); post(d, x) }
}",
    ));
    assert!(c.error_codes.contains(&"NL0310"), "{}", c.diags);
    assert!(c.diags.contains("debit<usd>"), "{}", c.diags);
}

#[test]
fn authorization_without_a_capability_is_rejected() {
    let c = compile(&with_body(
        "}
fn t(a: Id<Account>) -> Result<(), E> ! { authorize<usd> } { authorize(a, 50.00 usd) }
",
    ));
    assert!(c.error_codes.contains(&"NL0312"), "{}", c.diags);
    assert!(
        c.diags.contains("Proposition 3.2"),
        "the diagnostic must say what this check does *not* buy:\n{}",
        c.diags
    );
}

#[test]
fn holding_the_capability_discharges_the_requirement() {
    let c = compile(&with_body(
        "}
fn t(a: Id<Account>, auth: Auth<authorize<usd>>) -> Result<(), E> ! { authorize<usd> } {
    authorize(a, 50.00 usd)
}
",
    ));
    assert!(!c.error_codes.contains(&"NL0312"), "{}", c.diags);
}

#[test]
fn a_linear_hold_must_be_resolved_exactly_once() {
    let dropped = compile(&with_body(
        "}
fn t(a: Id<Account>) -> Result<(), E> ! { append, hold<usd> } {
    let h = hold(a, 20.00 usd, expires: 7.days)?;
    Ok(())
}
",
    ));
    assert!(
        dropped.error_codes.contains(&"NL0320"),
        "a dropped hold must be caught:\n{}",
        dropped.diags
    );

    let twice = compile(&with_body(
        "}
fn t(a: Id<Account>) -> Result<(), E> ! { append, hold<usd> } {
    let h = hold(a, 20.00 usd, expires: 7.days)?;
    resolve h post 10.00 usd;
    resolve h void
}
",
    ));
    assert!(
        twice.error_codes.contains(&"NL0321"),
        "a doubly-resolved hold must be caught:\n{}",
        twice.diags
    );
    assert!(
        twice.diags.contains("release the same reservation twice"),
        "{}",
        twice.diags
    );
}

// ============ base immutability ============

#[test]
fn a_ledger_cannot_be_updated_or_deleted_from() {
    for verb in [
        "update postings set amt = 0.00 usd;",
        "delete from postings;",
    ] {
        let c = compile(&with_body(&format!("}}\nfn t() {{ {verb} }}\n")));
        assert!(
            c.error_codes.contains(&"NL0230"),
            "`{verb}` must be rejected:\n{}",
            c.diags
        );
        assert!(c.diags.contains("history is the authority"), "{}", c.diags);
    }
}

#[test]
fn a_table_may_be_updated() {
    let c = compile(&with_body(
        "}\nfn t() { update accounts set owner = \"x\"; }\n",
    ));
    assert!(!c.error_codes.contains(&"NL0230"), "{}", c.diags);
}

#[test]
fn a_confidential_column_cannot_be_filtered_on() {
    let c = compile(&with_body(
        "    view v = accounts.where(|a| a.owner == \"ada\") serve { consistency: snapshot };
}",
    ));
    assert!(c.error_codes.contains(&"NL0260"), "{}", c.diags);
    assert!(c.diags.contains("leak through timing"), "{}", c.diags);
}

// ============ contract feasibility ============

#[test]
fn an_infeasible_contract_is_rejected_before_lowering() {
    let c = compile(&with_body(
        "    view v = postings.group_by(|p| p.acct).sum(|p| p.amt)
        serve { consistency: ledger_consistent, materialize: spilled };
}",
    ));
    assert!(c.error_codes.contains(&"NL0220"), "{}", c.diags);
}

#[test]
fn a_missing_anchor_index_warns_with_the_measured_reason() {
    let src = "\
schema b {
    currency usd { scale: 2 }
    ledger postings { txn: TxnId, acct: Id<A>, cur: Currency, amt: Money,
        idem: IdemKey window 50_000.epochs, conserve per (txn, cur); retain forever; }
    view v = postings.group_by(|p| (p.acct, p.cur)).sum(|p| p.amt)
        serve { consistency: snapshot, materialize: demand };
}
";
    let c = compile(src);
    assert!(c.warn_codes.contains(&"NL0223"), "{}", c.diags);
    assert!(
        c.diags.contains("80-112x"),
        "the warning cites the measured constant factor:\n{}",
        c.diags
    );
}
