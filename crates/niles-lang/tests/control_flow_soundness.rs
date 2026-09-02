//! Soundness of the currency-row solver and the linearity checker **across control flow**.
//!
//! This file exists because the analysis was unsound when it was written, in the direction
//! that matters least for money and most for adoption: it accused correct programs. A
//! transaction whose two branches each conserve was reported as creating money, and a hold
//! resolved once in each arm of an `if` was reported as resolved twice. Four false errors
//! on an eleven-line program.
//!
//! The cause was that the checker had no **join**. It walked both arms of a branch into one
//! accumulator, which is not an abstract interpretation of anything — it is the analysis of
//! a program in which both branches run. Every test below pins one case of the fix.
//!
//! The organising distinction, from `docs/research/currency-row-analysis.md`:
//!
//! * `Conserves` is a **proof** — sound, and it must never be issued for a program that
//!   does not conserve on some path.
//! * `Violates` is an **accusation** — it must be sound too, in the other direction: only
//!   issued when *every* committing path fails to balance.
//! * `MayViolate` and `Undecided` are the honest remainder. A checker that accuses a
//!   program it cannot follow teaches its users to switch it off, and then it protects
//!   nobody.

use niles_lang::diagnostics::Severity;
use niles_lang::{lower, parser, resolve, typecheck};

struct Out {
    errors: Vec<&'static str>,
    warnings: Vec<&'static str>,
    rendered: String,
    proved: usize,
    obligations: usize,
}

fn check(body: &str) -> Out {
    let src = format!(
        "schema b {{
    currency usd {{ scale: 2 }}
    currency eur {{ scale: 2 }}
    ledger postings {{ txn: TxnId, acct: Id<A>, cur: Currency, amt: Money,
        idem: IdemKey window 1.days, conserve per (txn, cur); retain forever; }}
    index ix on postings (acct) anchor;
}}
{body}
"
    );
    let (prog, mut d) = parser::parse_program(&src);
    let (cat, rd) = resolve::resolve_program(&prog, 0);
    d.extend(rd);
    let (report, td) = typecheck::check_program(&prog, &cat);
    d.extend(td);
    let (_l, ld) = lower::lower_program(&prog, &cat);
    d.extend(ld);
    Out {
        errors: d
            .items
            .iter()
            .filter(|x| x.severity == Severity::Error)
            .map(|x| x.code)
            .collect(),
        warnings: d
            .items
            .iter()
            .filter(|x| x.severity == Severity::Warning)
            .map(|x| x.code)
            .collect(),
        rendered: d.render(&src, "t.niles"),
        proved: report.conservation_proved,
        obligations: report.runtime_obligations,
    }
}

const SIG: &str = "-> Result<(), E> ! { append, debit<usd>, credit<usd> }";

// ===================== the bug that started this file =====================

#[test]
fn two_arms_that_each_conserve_do_not_sum_into_a_violation() {
    // The original failure. Before the join existed this produced FOUR false errors:
    // a phantom ten-dollar imbalance, a "never consumed" on the shadowed binding, and two
    // "consumed twice" on values each used exactly once.
    let o = check(&format!(
        "fn pick(a: Id<A>, b: Id<A>, p: bool) {SIG} {{
    txn idem(\"k\") {{
        let d = debit(a, 10.00 usd)?;
        if p {{ let c = credit(a, 10.00 usd); post(d, c) }}
        else {{ let c = credit(b, 10.00 usd); post(d, c) }}
    }}
}}"
    ));
    assert!(
        o.errors.is_empty(),
        "a program in which every path conserves must compile:\n{}",
        o.rendered
    );
    assert_eq!(
        o.proved, 1,
        "and the agreement between the arms must be *proved*, not merely tolerated"
    );
}

#[test]
fn a_hold_resolved_once_in_each_arm_is_resolved_once() {
    // Linear uses take the maximum across arms, not the sum. Summing them is the same
    // mistake as summing the rows, in the other discipline.
    let o = check(
        "fn f(a: Id<A>, p: bool) -> Result<(), E> ! { append, hold<usd> } {
    let h = hold(a, 20.00 usd, expires: 7.days)?;
    if p { resolve h post 20.00 usd } else { resolve h void }
}",
    );
    assert!(
        !o.errors.contains(&"NL0321"),
        "a hold resolved once per arm is not resolved twice:\n{}",
        o.rendered
    );
    assert!(
        !o.errors.contains(&"NL0320"),
        "nor is it dropped:\n{}",
        o.rendered
    );
}

// ===================== the accusation must stay sound =====================

#[test]
fn a_straight_line_imbalance_is_still_a_hard_error() {
    // The join must not have bought precision by giving up the ability to say no.
    let o = check(&format!(
        "fn bad(a: Id<A>, b: Id<A>) {SIG} {{
    txn idem(\"k\") {{ let d = debit(a, 100.00 usd)?; let c = credit(b, 60.00 usd); post(d, c) }}
}}"
    ));
    assert!(
        o.errors.contains(&"NL0300"),
        "a straight-line hole is a must-violation:\n{}",
        o.rendered
    );
    assert!(
        o.rendered.contains("-40.00"),
        "and the residue must be named:\n{}",
        o.rendered
    );
}

#[test]
fn an_early_exit_does_not_weaken_a_real_violation() {
    // `?` is an abort, and an aborted transaction commits nothing — so the abort path is
    // dropped, not treated as a path with a different net. Treating it as weakening
    // downgraded the forty-dollar hole above to a warning, which is exactly wrong: the
    // atomicity of the epoch seal is what licenses the stronger reading.
    let o = check(&format!(
        "fn bad(a: Id<A>, b: Id<A>) {SIG} {{
    txn idem(\"k\") {{ let d = debit(a, 100.00 usd)?; let c = credit(b, 60.00 usd); post(d, c) }}
}}"
    ));
    assert!(
        o.errors.contains(&"NL0300"),
        "atomicity means the abort path is not a committing path:\n{}",
        o.rendered
    );
    assert!(
        !o.warnings.contains(&"NL0301"),
        "and it must not be downgraded to a may-violation"
    );
}

#[test]
fn arms_that_disagree_are_undecided_rather_than_accused() {
    // One arm balances, the other does not. The join cannot represent "either 0 or −10" in
    // a domain with one affine form per currency, so it goes to top and the verdict is
    // `Undecided` — discharged to the runtime and counted, not reported as a hole.
    //
    // This is where the domain's weakness relative to Karr's shows: a *relational* domain
    // could keep `net ∈ {0, −10}` and reason further. The weakening is always toward
    // silence, never toward a false accusation, which is the property that matters.
    let o = check(&format!(
        "fn maybe(a: Id<A>, b: Id<A>, p: bool) {SIG} {{
    txn idem(\"k\") {{
        if p {{ let d = debit(a, 10.00 usd)?; let c = credit(b, 10.00 usd); post(d, c) }}
        else {{ let d = debit(a, 10.00 usd)?; post(d) }}
    }}
}}"
    ));
    assert!(
        !o.errors.contains(&"NL0300"),
        "a disagreement between arms is not a proof:\n{}",
        o.rendered
    );
    assert!(
        o.obligations >= 1,
        "it must be counted as discharged to the runtime instead"
    );
}

#[test]
fn arms_that_agree_on_a_hole_is_a_must_violation() {
    // The case the provenance gate got wrong. Both arms lose ten dollars, so *every* path
    // loses ten dollars — which is a proof, not an alarm. A top-preserving join guarantees
    // that a decided entry surviving the merge was agreed by every arm, so gating the
    // accusation on "was there a merge" threw away real precision for nothing.
    let o = check(&format!(
        "fn f(a: Id<A>, b: Id<A>, p: bool) {SIG} {{
    txn idem(\"k\") {{
        if p {{ let d = debit(a, 10.00 usd)?; post(d) }}
        else {{ let d = debit(b, 10.00 usd)?; post(d) }}
    }}
}}"
    ));
    assert!(
        o.errors.contains(&"NL0300"),
        "agreement on a hole is a proof of a hole:\n{}",
        o.rendered
    );
    assert!(
        o.rendered.contains("every path through this transaction"),
        "and the message must say the accusation covers every path:\n{}",
        o.rendered
    );
}

// ===================== the loop rule =====================

#[test]
fn a_loop_whose_body_balances_balances_for_any_trip_count() {
    // Immediate from the algebra: a row is a homomorphism into a free abelian group, so
    // zero per iteration is zero overall. No widening, no fixpoint, no trip count.
    let o = check(&format!(
        "fn batch(a: Id<A>, b: Id<A>, xs: Vec<i64>) {SIG} {{
    txn idem(\"k\") {{
        for x in xs {{ let d = debit(a, 10.00 usd)?; let c = credit(b, 10.00 usd); post(d, c) }}
    }}
}}"
    ));
    assert!(
        o.errors.is_empty(),
        "a balanced loop body must be accepted:\n{}",
        o.rendered
    );
}

#[test]
fn a_loop_whose_body_does_not_balance_is_undecided_not_accused() {
    // The body nets `+m`, so the loop nets `n · m` for an unknown `n` — a product of two
    // symbolic values, outside this domain's fragment. The honest answer is "unknown".
    let o = check(&format!(
        "fn leaky(a: Id<A>, xs: Vec<i64>) {SIG} {{
    txn idem(\"k\") {{
        for x in xs {{ let d = debit(a, 10.00 usd)?; post(d) }}
    }}
}}"
    ));
    assert!(
        !o.errors.contains(&"NL0300"),
        "an unknown trip count is not a proof of imbalance:\n{}",
        o.rendered
    );
    assert!(
        o.obligations >= 1,
        "it must be counted as discharged to the runtime instead"
    );
}

#[test]
fn a_hold_resolved_inside_a_loop_is_resolved_an_unknown_number_of_times() {
    // The linear analogue of `n · m`: a hold consumed inside a loop is consumed a number
    // of times the checker cannot count, and once is the only legal number.
    let o = check(
        "fn f(a: Id<A>, xs: Vec<i64>) -> Result<(), E> ! { append, hold<usd> } {
    let h = hold(a, 20.00 usd, expires: 7.days)?;
    for x in xs { resolve h void }
}",
    );
    assert!(
        o.errors.contains(&"NL0321"),
        "resolving a hold in a loop must be rejected:\n{}",
        o.rendered
    );
}

// ===================== scoping =====================

#[test]
fn a_hold_created_in_one_arm_must_be_resolved_in_that_arm() {
    // A linear value declared inside a branch cannot escape it: the other arm has no such
    // value, so there is nowhere for a later resolution to be well defined.
    let o = check(
        "fn f(a: Id<A>, p: bool) -> Result<(), E> ! { append, hold<usd> } {
    if p { let h = hold(a, 20.00 usd, expires: 7.days)?; } else { }
    Ok(())
}",
    );
    assert!(
        o.errors.contains(&"NL0320"),
        "a hold dropped inside an arm must be caught there:\n{}",
        o.rendered
    );
}

#[test]
fn shadowed_bindings_in_sibling_arms_do_not_collide() {
    // The false "never consumed" in the original bug: two arms each binding `c` produced
    // two entries in one flat list, and the checker blamed whichever it saw last.
    let o = check(&format!(
        "fn f(a: Id<A>, b: Id<A>, p: bool) {SIG} {{
    txn idem(\"k\") {{
        if p {{ let d = debit(a, 5.00 usd)?; let c = credit(b, 5.00 usd); post(d, c) }}
        else {{ let d = debit(b, 5.00 usd)?; let c = credit(a, 5.00 usd); post(d, c) }}
    }}
}}"
    ));
    assert!(
        o.errors.is_empty(),
        "sibling arms have separate scopes:\n{}",
        o.rendered
    );
}

#[test]
fn a_one_armed_if_is_still_a_branch() {
    // An `if` without an `else` has an implicit arm that does nothing. Omitting it would
    // let a conditional posting look unconditional, which is the unsound direction.
    let o = check(&format!(
        "fn f(a: Id<A>, b: Id<A>, p: bool) {SIG} {{
    txn idem(\"k\") {{
        let d = debit(a, 5.00 usd)?;
        let c = credit(b, 5.00 usd);
        if p {{ post(d, c) }}
    }}
}}"
    ));
    // The row is the same on both paths here (the postings happen before the branch), so
    // this must not warn — the point is that the empty arm participates in the join.
    assert!(!o.errors.contains(&"NL0300"), "{}", o.rendered);
}

#[test]
fn match_arms_join_like_if_arms() {
    let o = check(&format!(
        "fn f(a: Id<A>, b: Id<A>, o: Outcome) {SIG} {{
    txn idem(\"k\") {{
        let d = debit(a, 7.00 usd)?;
        match o {{
            Post(m) => {{ let c = credit(b, 7.00 usd); post(d, c) }},
            Void => {{ let c = credit(a, 7.00 usd); post(d, c) }},
        }}
    }}
}}"
    ));
    assert!(
        o.errors.is_empty(),
        "match arms are alternatives, not a sequence:\n{}",
        o.rendered
    );
}

#[test]
fn nested_branches_join_at_every_level() {
    let o = check(&format!(
        "fn f(a: Id<A>, b: Id<A>, p: bool, q: bool) {SIG} {{
    txn idem(\"k\") {{
        let d = debit(a, 3.00 usd)?;
        if p {{
            if q {{ let c = credit(b, 3.00 usd); post(d, c) }} else {{ let c = credit(a, 3.00 usd); post(d, c) }}
        }} else {{
            let c = credit(b, 3.00 usd); post(d, c)
        }}
    }}
}}"
    ));
    assert!(o.errors.is_empty(), "{}", o.rendered);
}

// ===================== per-currency, still =====================

#[test]
fn a_cross_currency_hole_survives_the_join() {
    // The join must not accidentally net USD against EUR while merging.
    let o = check(
        "fn f(a: Id<A>, b: Id<A>) -> Result<(), E> ! { append, debit<usd>, credit<eur> } {
    txn idem(\"k\") { let d = debit(a, 10.00 usd)?; let c = credit(b, 10.00 eur); post(d, c) }
}",
    );
    let holes = o.errors.iter().filter(|c| **c == "NL0300").count();
    assert_eq!(
        holes, 2,
        "both currencies must still be reported separately:\n{}",
        o.rendered
    );
}
