//! **The GBS schema, checked by the real compiler.**
//!
//! `gbs/niles/gbs.niles` declares the ledger and the read models. This file runs `nilesc`
//! over it and asserts what the compiler proves, so three things cannot drift apart: the
//! schema, the language that validates it, and the claims made about both.
//!
//! It also runs the compiler over deliberately *broken* schemas, because a static check
//! that has never rejected anything is a static check nobody has tested. The two errors
//! exercised here — an understated effect row and a rung-monotonicity violation — are the
//! two the architecture actually leans on:
//!
//! * **NL0310** is Contribution 4's guarantee in its narrowest form: a function cannot move
//!   a currency its signature does not declare.
//! * **NL0311** is what makes the FDIC/CFPB authorize-positive-settle-negative pattern a
//!   compile error. It is also the check that rejected the thesis's own worked example
//!   (§9.13.4), which is the best evidence available that it is not decorative.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    // gbs/crates/gbs-products -> gbs/crates -> gbs -> repo root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the crate sits three levels below the repo root")
        .to_path_buf()
}

/// Run a `nilesc` subcommand over a file, returning (exit ok, stdout, stderr).
///
/// Invokes the built binary rather than calling the library, so what is tested is the
/// compiler a person would run. A library call could pass while `nilesc` itself was broken.
fn nilesc(cmd: &str, file: &Path) -> (bool, String, String) {
    let out = Command::new(env!("CARGO"))
        .args(["run", "-q", "-p", "nilesc", "--", cmd])
        .arg(file)
        .current_dir(repo_root())
        .output()
        .expect("nilesc should run");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn schema_path() -> PathBuf {
    repo_root().join("gbs/niles/gbs.niles")
}

/// Write a temporary schema and check it. Returns the compiler's stderr.
fn check_source(name: &str, src: &str) -> (bool, String) {
    let path = std::env::temp_dir().join(format!("gbs-niles-{name}.niles"));
    std::fs::write(&path, src).expect("write temp schema");
    let (ok, _out, err) = nilesc("check", &path);
    let _ = std::fs::remove_file(&path);
    (ok, err)
}

// ── the schema itself ────────────────────────────────────────────────────────────────

#[test]
fn the_gbs_schema_compiles() {
    let (ok, out, err) = nilesc("check", &schema_path());
    assert!(ok, "gbs.niles must compile:\n{err}");
    assert!(out.starts_with("ok:"), "{out}");
}

#[test]
fn the_schema_declares_the_relations_views_and_functions_it_claims_to() {
    // Counted by the compiler, not by this test, so the numbers cannot be wrong in the
    // reassuring direction.
    let (_, out, _) = nilesc("check", &schema_path());
    assert!(out.contains("6 relation(s)"), "{out}");
    assert!(out.contains("7 view(s)"), "{out}");
    assert!(out.contains("6 function(s)"), "{out}");
}

#[test]
fn every_conservation_obligation_is_proved_statically_and_none_is_deferred() {
    // **The claim that matters.** Five obligations, all discharged before the program runs
    // — nothing handed to the runtime to find out about later. If this ever reports an
    // obligation "discharged to the runtime", a posting set in this schema is one the
    // compiler could not prove balanced, and that is worth knowing loudly.
    let (_, out, _) = nilesc("check", &schema_path());
    assert!(out.contains("5 conservation obligation(s) proved statically"), "{out}");
    assert!(out.contains("0 discharged to the runtime"), "{out}");
}

#[test]
fn the_lowered_circuit_passes_the_ir_verifier() {
    // The verifier is in the trusted base (§7.4), so this is the check that does not depend
    // on the compiler being right about its own output.
    let (ok, out, err) = nilesc("verify", &schema_path());
    assert!(ok, "the IR verifier rejected the schema:\n{err}");
    assert!(out.contains("no violations"), "{out}");
}

#[test]
fn every_function_carries_the_effect_row_the_products_rely_on() {
    let (ok, out, err) = nilesc("effects", &schema_path());
    assert!(ok, "{err}");

    // FX debits *and* credits in both currencies, because an `fx` form is two conserved
    // legs. A row reading `debit<usd>, credit<eur>` would describe a transaction conserving
    // neither — the error the thesis's printed example contained.
    let fx = out.lines().find(|l| l.contains("fn fx_settle")).expect("fx_settle listed");
    for effect in ["debit<usd>", "credit<usd>", "debit<eur>", "credit<eur>"] {
        assert!(fx.contains(effect), "fx_settle should carry {effect}: {fx}");
    }

    // A hold is its own effect, distinct from a debit: it encumbers without moving.
    let auth = out.lines().find(|l| l.contains("card_authorization")).expect("listed");
    assert!(auth.contains("hold<usd>"), "{auth}");
    assert!(!auth.contains("debit<usd>"), "a hold does not debit: {auth}");
}

#[test]
fn the_availability_view_is_served_at_the_strictest_rung() {
    // The decision path. A view that authorises a movement must see every committed write
    // that precedes it in real time.
    let (ok, out, _) = nilesc("effects", &schema_path());
    assert!(ok);
    let line = out
        .lines()
        .find(|l| l.contains("available_balance"))
        .expect("available_balance listed");
    assert!(line.contains("ledger_consistent"), "{line}");
}

// ── the static checks are exercised, not merely trusted ──────────────────────────────

#[test]
fn an_understated_effect_row_is_rejected_with_both_spans() {
    // NL0310, Contribution 4 at its narrowest: a function cannot move a currency its
    // signature does not declare. The diagnostic must name *both* places — where the effect
    // was incurred and where the row was declared — because an error naming only one sends
    // the reader to the wrong file.
    let (ok, err) = check_source(
        "understated",
        r#"
schema t {
    currency usd { scale: 2 }
    currency eur { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money, value_date: Date,
        idem: IdemKey window 30.days,
        conserve per (txn, cur);
        retain forever;
    }
    index by_acct on postings (acct, cur) anchor;
}
fn understated(a: Id<Account>, b: Id<Account>) -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
{
    txn idem("x") {
        let d = debit(a, 100.00 eur)?;
        let c = credit(b, 100.00 eur);
        post(d, c)
    }
}
"#,
    );
    assert!(!ok, "moving EUR under a USD-only row must not compile");
    assert!(err.contains("NL0310"), "{err}");
    assert!(err.contains("debit<eur>"), "names the offending effect: {err}");
    assert!(err.contains("incurred here"), "and where it happened: {err}");
    assert!(err.contains("declared effects are"), "and what was promised: {err}");
    // A machine-applicable suggestion, so the fix is one keystroke rather than a search.
    assert!(err.contains("add it to the declared row"), "{err}");
}

#[test]
fn a_rung_monotonicity_violation_is_rejected() {
    // **NL0311.** A view promising `ledger_consistent` while reading a `bounded` one. This
    // is the check that turns the authorize-positive-settle-negative pattern into a compile
    // error, and the one that rejected the thesis's own worked example.
    let (ok, err) = check_source(
        "rung",
        r#"
schema t {
    currency usd { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money, value_date: Date,
        idem: IdemKey window 30.days,
        conserve per (txn, cur);
        retain forever;
    }
    index by_acct on postings (acct, cur) anchor;
    view stale = postings.group_by(|p| (p.acct, p.cur)).sum(|p| p.amt)
        serve { consistency: bounded(epochs: 10, millis: 30000), materialize: auto };
    view decision = stale.group_by(|p| p.acct).sum(|p| p.amt)
        serve { consistency: ledger_consistent, materialize: demand };
}
"#,
    );
    assert!(!ok, "promising a rung above your input must not compile");
    assert!(err.contains("NL0311"), "{err}");
    assert!(err.contains("promises `ledger_consistent`"), "{err}");
    assert!(err.contains("only `bounded`"), "{err}");
    // The explanation, which is the part that teaches rather than merely refuses.
    assert!(
        err.contains("no fresher than its stalest input"),
        "the diagnostic should say why: {err}"
    );
    assert!(
        err.contains("two derived views of one ledger disagreeing"),
        "and connect it to the failure it prevents: {err}"
    );
}

#[test]
fn the_gbs_schema_would_fail_that_check_if_availability_derived_from_the_cheap_view() {
    // The counterfactual, run rather than asserted. `available_balance` derives from
    // `postings` directly; this proves that deriving it from `ledger_balance` — which is
    // the obvious, tempting, cheaper thing — would not compile.
    let (ok, err) = check_source(
        "counterfactual",
        r#"
schema t {
    currency usd { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money, value_date: Date,
        idem: IdemKey window 30.days,
        conserve per (txn, cur);
        retain forever;
    }
    index by_acct on postings (acct, cur) anchor;
    view ledger_balance = postings.group_by(|p| (p.acct, p.cur)).sum(|p| p.amt)
        serve { consistency: read_your_writes, materialize: auto };
    view available_balance = ledger_balance.group_by(|p| p.acct).sum(|p| p.amt)
        serve { consistency: ledger_consistent, materialize: demand };
}
"#,
    );
    assert!(
        !ok,
        "deriving the availability decision from a read_your_writes view must not compile"
    );
    assert!(err.contains("NL0311"), "{err}");
}

#[test]
fn a_reserved_word_as_a_field_name_is_refused_with_the_escape_named() {
    // Recorded because it is what the GBS schema actually hit: `actor` is reserved for
    // future use, and the field became `acted_by`. The diagnostic offers the raw-identifier
    // escape, which is the right thing to offer and the wrong thing to take — a raw
    // identifier in a schema is a sign the schema is fighting the language.
    let (ok, err) = check_source(
        "reserved",
        r#"
schema t {
    currency usd { scale: 2 }
    base events { actor: Text, at: Instant, retain forever; }
}
"#,
    );
    assert!(!ok);
    assert!(err.contains("NL0002"), "{err}");
    assert!(err.contains("reserved"), "{err}");
    assert!(err.contains("r#actor"), "the escape is offered: {err}");
}

#[test]
fn the_checks_pass_on_the_real_schema_which_makes_the_negative_controls_meaningful() {
    // The pairing that matters: the broken schemas above fail, and the real one does not.
    // Either half alone proves nothing — a checker that rejected everything would pass the
    // negative controls, and one that accepted everything would pass this.
    let (ok, _, err) = nilesc("check", &schema_path());
    assert!(ok, "{err}");
    assert!(!err.contains("NL0310"));
    assert!(!err.contains("NL0311"));
}
