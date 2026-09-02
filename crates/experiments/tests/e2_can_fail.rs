//! **The two sides of E2 come from two implementations.**
//!
//! E2 reports "1,000 epochs, 0 mismatches" for H-F2, the stream–relation duality. Until this
//! cycle that number was produced by integrating a changelog, differentiating the resulting
//! state sequence, integrating again and comparing — a telescoping sum against its own
//! parts, in one function, on two arrays. No input could have made it report a mismatch.
//!
//! The repair is that the two sides are now computed by different code: a linear scan of the
//! base prefix (`Ledger::reconstruct_balance_scan`) against a `PartialView` advanced one
//! epoch at a time (`PartialView::apply_epoch`). The binary carries its own runtime control —
//! it withholds one epoch's deltas and asserts the comparison notices — and this file is the
//! structural half: a check that the experiment did not quietly go back to comparing a
//! computation with itself.
//!
//! A source-text test is a blunt instrument and is used here deliberately. What went wrong
//! was not a wrong number; it was a *shape* — expected and actual sharing a derivation — and
//! the shape is what this asserts.

fn e2_source() -> String {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("experiments/src/main.rs");
    let start = src
        .find("fn e2_one_seed")
        .expect("E2's per-seed body is named `e2_one_seed`");
    let rest = &src[start..];
    let end = rest.find("\n// ---").unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn e2_compares_a_scan_with_an_incremental_fold() {
    let body = e2_source();
    assert!(
        body.contains("reconstruct_balance_scan"),
        "one side of E2 must be the fold of the whole prefix, computed by a scan that \
         holds no view state"
    );
    assert!(
        body.contains("apply_epoch"),
        "the other side must be the engine's own per-epoch derivative fold; `apply_through` \
         would fold a whole range at once and is not the same claim"
    );
}

#[test]
fn e2_carries_a_control_that_must_fail() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("experiments/src/main.rs");
    assert!(
        src.contains("fn e2_control"),
        "E2 must run a sabotaged variant alongside the real one"
    );
    assert!(
        src.contains("control_mismatch > 0"),
        "and must assert that the sabotaged variant *does* mismatch, at run time, in the \
         binary — a control nobody checks is a comment"
    );
    let body = e2_source();
    assert!(
        body.contains("sabotage") && body.contains("apply_epoch"),
        "the sabotage must act on the incremental path, not on the comparison"
    );
}
