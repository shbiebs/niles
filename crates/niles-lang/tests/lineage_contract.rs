//! **`lineage` is a real serve-contract key** (cycle 16, C16-09; cycle 14's design decision 8).
//!
//! `serve { lineage: full }` was listed in `keyword_samples.rs` as a MISMATCH: the registry
//! documented a contract key that no frame of that test could compile. The key was in fact read
//! by the resolver, but as *any word*: the lowering mapped every value it did not recognise to
//! `Lineage::Off`, so `lineage: ful` compiled to a view with no lineage and said nothing.
//!
//! These tests hold the three modes end to end, from source text to the IR's serve contract,
//! and hold the refusal of a fourth. The keyword sample now compiles in the serve-contract frame,
//! and its MISMATCH row is gone because the change made it compile, not because the row was
//! edited.

use niles_lang::{lower, parser, resolve};

const SCHEMA: &str = r#"
schema lin {
    currency usd { scale: 2 }
    base t { k: Int, v: Int, retain forever; }
    index ix_t on t (k) anchor;
}
"#;

/// Compile one view with the given contract; return the diagnostic codes and the `Debug` of
/// the view's IR serve contract.
fn compile(contract: &str) -> (Vec<String>, String) {
    let src = format!(
        "{SCHEMA}\nview v = t.where(|r| r.v > 0) serve {{ consistency: snapshot, {contract} }};\n"
    );
    let (prog, mut d) = parser::parse_program(&src);
    let (cat, rd) = resolve::resolve_program(&prog, 0);
    d.extend(rd);
    let (lowered, ld) = lower::lower_program(&prog, &cat);
    d.extend(ld);
    let codes = d.items.iter().map(|x| x.code.to_string()).collect();
    let contract = lowered
        .circuit
        .outputs
        .get("v")
        .and_then(|id| lowered.circuit.nodes.iter().find(|n| n.id == *id))
        .map(|n| format!("{:?}", n.contract.peek()))
        .unwrap_or_default();
    (codes, contract)
}

#[test]
fn each_lineage_mode_reaches_the_ir() {
    for (word, ir) in [("off", "Off"), ("key", "Key"), ("full", "Full")] {
        let (codes, contract) = compile(&format!("lineage: {word}"));
        assert!(codes.is_empty(), "`lineage: {word}` raised {codes:?}");
        assert!(
            contract.contains(&format!("lineage: {ir}")),
            "`lineage: {word}` did not reach the IR as Lineage::{ir}: {contract}"
        );
    }
}

#[test]
fn an_absent_lineage_key_is_off() {
    let (codes, contract) = compile("materialize: auto");
    assert!(codes.is_empty(), "{codes:?}");
    assert!(contract.contains("lineage: Off"), "{contract}");
}

/// The defect: an unknown mode was served as `off`. It is now NL0206, with a suggestion.
#[test]
fn an_unknown_lineage_mode_is_refused_not_turned_off() {
    let (codes, _) = compile("lineage: ful");
    assert!(
        codes.iter().any(|c| c == "NL0206"),
        "`lineage: ful` was accepted ({codes:?}); before C16-09 it lowered to Lineage::Off"
    );
    let (codes, _) = compile("lineage: provenance");
    assert!(codes.iter().any(|c| c == "NL0206"), "{codes:?}");
}
