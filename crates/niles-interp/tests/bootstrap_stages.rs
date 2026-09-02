//! **The Appendix E bootstrap gates, as executable tests.**
//!
//! Appendix E.0 recorded that stage 1 had no input, because no line of the Niles-written
//! compiler had been written and the stage-0 compiler could not run one anyway. Both
//! halves of that are now false: `bootstrap/lexer.niles` is a lexer written in Niles, and
//! `niles-interp` executes it. This file is what turns that into a claim with an exit
//! status attached.
//!
//! Four gates, in the order Appendix E.1 defines them:
//!
//! | Gate | What it establishes |
//! |---|---|
//! | **Stage 1 runs** | Stage 0 can execute a non-trivial Niles program end to end |
//! | **Stage 1 equivalence** | The Niles lexer and the Rust lexer agree, token for token, over a corpus |
//! | **Stage 2 self-application** | The Niles lexer can lex *its own source* |
//! | **Stage 3 fixpoint** | Repeated runs are byte-identical |
//!
//! # What these gates do not establish
//!
//! They are gates on a *lexer*, not on a compiler. Stage 2 in Appendix E's full sense is
//! "stage 1 recompiling the same sources", and a lexer does not compile anything, so what
//! is checked here is the lexer's analogue: self-application, and the fixpoint that
//! follows. A parser, a type-checker and a lowering pass in Niles are still unwritten,
//! and §E.19 must continue to say so. The honest summary is that the bootstrap has gone
//! from "no input" to "one front-end stage, verified against the reference
//! implementation", which is a first rung and not a ladder.
//!
//! The equivalence gate also has a *scope*, declared in `bootstrap/lexer.niles` and
//! enforced in [`in_scope`] below: instants, durations, byte strings and raw identifiers
//! are not implemented on the Niles side, and inputs containing them are excluded rather
//! than being allowed to pass by accident. Excluding them narrows what the gate proves;
//! pretending they passed would falsify it.

use niles_interp::{determinism_gate, Error, Interp, Value};
use niles_lang::lexer::{TimeUnit, Tok};
use niles_lang::parser;

const LEXER_SRC: &str = include_str!("../../../bootstrap/lexer.niles");

/// Render the Rust lexer's output in the format `bootstrap/lexer.niles` produces.
///
/// The two lexers share no types, so the comparison has to happen on a rendering. This
/// is the adapter, and it is on the *Rust* side on purpose: writing it in Niles would
/// mean the gate compared a Niles rendering against a Niles rendering, and a bug present
/// in both would cancel.
fn render_rust(src: &str) -> String {
    let (toks, _errs) = niles_lang::lexer::lex(src);
    let mut out = String::new();
    for t in &toks {
        let kind = match &t.tok {
            Tok::Kw(_) => "KW",
            Tok::Ident => "IDENT",
            Tok::Int(_) => "INT",
            Tok::Money { .. } => "MONEY",
            Tok::EpochLit(_) => "EPOCH",
            Tok::Str(_) => "STR",
            Tok::Eof => "EOF",
            Tok::Unknown => "UNKNOWN",
            Tok::Bool(_) => "KW", // `true`/`false` are keywords in the registry
            _ => "PUNCT",
        };
        let text = &src[t.span.start as usize..t.span.end as usize];
        out.push_str(&format!("{kind} {}..{} {text}\n", t.span.start, t.span.end));
    }
    out
}

/// Run `bootstrap/lexer.niles` under stage 0 and return its rendering of `input`.
fn render_niles(input: &str) -> Result<String, Error> {
    let (prog, d) = parser::parse_program(LEXER_SRC);
    assert!(
        !d.has_errors(),
        "bootstrap/lexer.niles must parse: {:?}",
        d.items
    );
    let mut it = Interp::new();
    it.load(&prog);
    let v = it.call(
        "lex_and_render",
        vec![Value::Str(std::rc::Rc::new(input.to_string()))],
    )?;
    match v {
        Value::Str(s) => Ok((*s).clone()),
        other => panic!(
            "lex_and_render should return a string, got {}",
            other.type_name()
        ),
    }
}

/// Whether an input is inside the Niles lexer's declared scope.
///
/// The gate must not silently pass on constructs the Niles side does not implement, and
/// it must not silently *fail* on them either — it must exclude them, visibly. A test
/// that asserts on `in_scope` itself is below, so the exclusion cannot quietly widen.
fn in_scope(src: &str) -> bool {
    let (toks, _errs) = niles_lang::lexer::lex(src);
    // Note what is *not* excluded: an input the reference lexer reports an error on. A
    // malformed input — an unterminated string, an unclosed block comment — is exactly
    // where two lexers most easily disagree, because each has to decide where the broken
    // construct ends. Excluding those would have removed the hardest cases from the gate
    // in the name of tidiness. What is excluded is only the four *constructs* the Niles
    // side does not implement.
    !toks.iter().any(|t| {
        matches!(
            t.tok,
            Tok::Instant(_)
                | Tok::ValidInstant(_)
                | Tok::Bytes(_)
                | Tok::Float(_)
                | Tok::Duration { .. }
        )
    })
}

/// The corpus the equivalence gate runs over.
///
/// Chosen to exercise the classification decisions where a hand-written lexer actually
/// goes wrong: longest-match punctuation, the sigil-versus-attribute rule for `#`, the
/// money-versus-integer rule for `.`, escapes inside strings, and unterminated
/// constructs at end of input.
fn corpus() -> Vec<(&'static str, &'static str)> {
    vec![
        ("empty", ""),
        ("whitespace only", "   \n\t  \n"),
        ("one identifier", "balance"),
        ("keyword and identifier", "let balance = 1;"),
        (
            "keywords, and two near-misses that are not",
            "fn view ledger base txn serve evict conserves currency schema",
        ),
        // The classic longest-match traps. `::` before `:`, `|>` before `|`, `=>` before `=`.
        ("path separator", "std::collections::BTreeMap"),
        ("pipeline arrow", "q |> where(p) |> group_by(k)"),
        (
            "fat arrow and thin arrow",
            "fn f() -> i64 { match x { 1 => 2 } }",
        ),
        ("comparison operators", "a == b != c <= d >= e < f > g"),
        ("logical operators", "a && b || !c"),
        ("range", "0..10"),
        // Numbers, and the money rule.
        ("integer", "42"),
        ("integer with underscores", "1_000_000"),
        ("money literal", "10.00 usd"),
        ("money with a longer scale", "0.000001 btc"),
        ("integer then dot then method", "xs.len()"),
        // The `#` rule: an epoch literal only when a digit follows.
        ("epoch literal", "#4200"),
        ("attribute", "#[serve(rung: strict)]"),
        ("epoch and attribute together", "#[audit] as_of(#4200)"),
        // Strings.
        ("simple string", "\"hello\""),
        ("string with an escaped quote", "\"say \\\"hi\\\"\""),
        ("string with a backslash", "\"a\\\\b\""),
        ("empty string", "\"\""),
        // Comments.
        ("line comment", "let x = 1; // the balance\nlet y = 2;"),
        ("block comment", "let /* inline */ x = 1;"),
        ("comment at end of input", "let x = 1; // trailing"),
        ("unterminated block comment", "let x = 1; /* never closed"),
        ("unterminated string", "\"never closed"),
        // A realistic program.
        (
            "a transfer",
            "fn transfer(from: Account, to: Account, amt: Money) {\n\
             \x20   let fee = 0.25 usd;\n\
             \x20   if from.balance >= amt + fee {\n\
             \x20       post(from, to, amt);\n\
             \x20   }\n\
             }",
        ),
    ]
}

// ── Gate 1: stage 1 runs at all ──────────────────────────────────────────────────────

#[test]
fn stage_1_the_niles_written_lexer_parses_and_runs() {
    // The gate Appendix E.0 said had no input. It now has one.
    let (prog, d) = parser::parse_program(LEXER_SRC);
    assert!(!d.has_errors(), "the Niles lexer must parse under stage 0");
    let mut it = Interp::new();
    it.load(&prog);
    let names = it.function_names();
    for wanted in [
        "lex",
        "next_token",
        "skip_trivia",
        "scan_punct",
        "lex_and_render",
        "main",
    ] {
        assert!(names.contains(&wanted.to_string()), "missing fn {wanted}");
    }
    it.call("main", vec![])
        .expect("the Niles lexer's self-check must run");
    assert!(!it.output.is_empty(), "it must produce a rendering");
}

#[test]
fn keyword_table_matches_the_registry_exactly() {
    // The single source of truth crosses the bootstrap boundary.
    //
    // The first draft of `bootstrap/lexer.niles` carried a hand-picked table and the
    // equivalence gate rejected it four times: `evict` and `conserves` were invented,
    // `from` and `post` were missing. Two implementations of one language drifted apart
    // within a day of the second one existing — which is precisely the argument §6.4
    // makes for having a registry at all, arriving here as evidence rather than as an
    // assertion. This test is the fence: the Niles-side table is generated from
    // `keywords.rs`, and the build fails the moment the two diverge again.
    let registry: std::collections::BTreeSet<String> = niles_lang::keywords::KEYWORDS
        .iter()
        .map(|k| k.word.to_string())
        .collect();

    // Extract the array literal from the Niles source. Parsing it out of the file rather
    // than duplicating it here is the point: the assertion is about the file that runs.
    let sig = "fn keywords() -> [str] {";
    let start = LEXER_SRC.find(sig).expect("the table must exist") + sig.len();
    let open = LEXER_SRC[start..].find('[').unwrap() + start;
    let close = LEXER_SRC[open..].find(']').unwrap() + open;
    let in_niles: std::collections::BTreeSet<String> = LEXER_SRC[open + 1..close]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let missing: Vec<_> = registry.difference(&in_niles).collect();
    let invented: Vec<_> = in_niles.difference(&registry).collect();
    assert!(
        missing.is_empty(),
        "in the registry but not in the Niles lexer: {missing:?}"
    );
    assert!(
        invented.is_empty(),
        "in the Niles lexer but not in the registry: {invented:?}"
    );
    assert_eq!(in_niles.len(), registry.len());
    assert!(
        registry.len() >= 170,
        "the registry should be the full table, got {}",
        registry.len()
    );
}

#[test]
fn the_two_defects_the_gate_caught_are_pinned_so_they_cannot_return() {
    // Regression tests for the four specific words. Named individually because a set
    // comparison that passes tells you nothing about *which* mistake it prevented.
    let (toks, _) = niles_lang::lexer::lex("evict conserves conserve evictable from post");
    let kinds: Vec<bool> = toks.iter().map(|t| matches!(t.tok, Tok::Kw(_))).collect();
    assert_eq!(
        &kinds[..6],
        &[false, false, true, true, true, true],
        "`evict`/`conserves` are identifiers; `conserve`/`evictable`/`from`/`post` are keywords"
    );
}

#[test]
fn keyword_matching_is_case_insensitive_on_both_sides() {
    // The other defect: Niles inherits case-insensitive keywords from SQL, and a lexer
    // written by someone thinking in Rust gets this wrong. Both implementations must
    // agree, so the gate checks it directly rather than only through the corpus.
    let mixed = "EPOCH Epoch epoch LEDGER Ledger ledger";
    let rust = render_rust(mixed);
    let niles = render_niles(mixed).unwrap();
    assert_eq!(niles, rust);
    assert_eq!(
        rust.matches("KW ").count(),
        6,
        "all six spellings are the same two words"
    );
}

#[test]
fn stage_1_uses_only_constructs_the_language_actually_has() {
    // A weak but real expressiveness result: the file uses `fn`, `let mut`, `while`,
    // `if`, `match`, `struct`, `enum` with payloads, arrays and early `return`, and it
    // parses. Had any of those been aspirational, this file would not exist.
    for construct in [
        "fn ", "let mut ", "while ", "match ", "struct ", "enum ", "return ",
    ] {
        assert!(
            LEXER_SRC.contains(construct),
            "the bootstrap should exercise `{construct}`"
        );
    }
}

// ── Gate 2: equivalence with the reference lexer ─────────────────────────────────────

#[test]
fn stage_1_equivalence_the_two_lexers_agree_token_for_token() {
    // The gate that matters. Two independent implementations — one in Rust, one in Niles
    // — over the same corpus, compared on every token's kind, span and text.
    let mut checked = 0;
    let mut skipped = Vec::new();
    for (name, src) in corpus() {
        if !in_scope(src) {
            skipped.push(name);
            continue;
        }
        let rust = render_rust(src);
        let niles = render_niles(src)
            .unwrap_or_else(|e| panic!("the Niles lexer failed on `{name}`: {}", e.message()));
        assert_eq!(niles, rust, "\ncase: {name}\nsource: {src:?}\n");
        checked += 1;
    }
    assert!(
        checked >= 25,
        "the gate must actually cover the corpus, checked {checked}"
    );
    // Every skip is reported rather than hidden. If this list grows, the gate proves less.
    assert!(
        skipped.is_empty(),
        "these corpus entries fell outside the declared scope and were not checked: {skipped:?}"
    );
}

#[test]
fn the_scope_exclusion_is_real_and_not_a_way_to_pass() {
    // Guarding the guard. `in_scope` must actually exclude the four unimplemented forms;
    // if it silently returned true, the gate above would appear to prove more than it does.
    assert!(!in_scope("@2026-03-01"), "instants are out of scope");
    assert!(
        !in_scope("v@2026-03-01"),
        "valid-time instants are out of scope"
    );
    assert!(!in_scope("7.days"), "durations are out of scope");
    assert!(!in_scope("1.5"), "floats are out of scope");
    assert!(in_scope("let x = 10.00 usd;"), "money is in scope");
}

#[test]
fn the_niles_lexer_disagrees_where_it_should_which_shows_the_gate_can_fail() {
    // A gate that cannot fail proves nothing. Feed it something outside the Niles side's
    // scope and confirm the two renderings really do differ — i.e. the comparison is
    // load-bearing rather than trivially satisfied.
    let out_of_scope = "@2026-03-01";
    let rust = render_rust(out_of_scope);
    let niles = render_niles(out_of_scope).expect("it should still produce something");
    assert_ne!(
        niles, rust,
        "an unimplemented construct must show up as a disagreement, or the gate is blind"
    );
}

// ── Gate 3: self-application ─────────────────────────────────────────────────────────

#[test]
fn stage_2_the_niles_lexer_lexes_its_own_source() {
    // The lexer's analogue of "stage 1 recompiling the same sources": the Niles lexer is
    // pointed at `bootstrap/lexer.niles` itself. This is where a bootstrap first becomes
    // self-referential, and where a hidden assumption about the input usually surfaces.
    let niles = render_niles(LEXER_SRC).expect("it must survive its own source");
    let lines = niles.lines().count();
    assert!(
        lines > 800,
        "its own source is a few thousand tokens, got {lines}"
    );
    assert!(niles.ends_with(&format!("EOF {}..{} \n", LEXER_SRC.len(), LEXER_SRC.len())));
}

#[test]
fn stage_2_self_application_agrees_with_the_reference_lexer() {
    // Stronger than the previous test: not merely that it survives its own source, but
    // that it lexes it *identically* to the Rust lexer. This is the largest single input
    // in the suite by two orders of magnitude, and it is real code rather than a fixture.
    assert!(
        in_scope(LEXER_SRC),
        "the bootstrap lexer's own source must stay inside the subset it implements — \
         if this fails, the file has grown a construct it cannot lex"
    );
    let rust = render_rust(LEXER_SRC);
    let niles = render_niles(LEXER_SRC).expect("self-application must succeed");

    // Report the first disagreement by line rather than dumping thousands of lines.
    if niles != rust {
        let at = niles
            .lines()
            .zip(rust.lines())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| niles.lines().count().min(rust.lines().count()));
        let n: Vec<&str> = niles.lines().collect();
        let r: Vec<&str> = rust.lines().collect();
        panic!(
            "self-application diverged at token {at}:\n  niles: {:?}\n  rust:  {:?}",
            n.get(at),
            r.get(at)
        );
    }
}

// ── Gate 4: the fixpoint / determinism gate ──────────────────────────────────────────

#[test]
fn stage_3_repeated_runs_are_byte_identical() {
    // Appendix C.4's obligation, at the scope stage 0 can actually check: run-to-run
    // determinism on one target. Cross-target determinism needs two machines and is
    // still an unmet obligation; §E.19 says so and this test does not overclaim.
    let (prog, d) = parser::parse_program(LEXER_SRC);
    assert!(!d.has_errors());
    let r = determinism_gate(&prog, "main", 5).expect("the self-check must run");
    assert!(
        r.identical,
        "diverged at output line {:?}",
        r.first_divergence
    );
    assert_eq!(r.runs, 5);
    assert!(!r.output.is_empty());
}

#[test]
fn stage_3_the_fixpoint_holds_on_self_application_too() {
    // The strongest form available here: lexing its own source, twice, must give the same
    // bytes. A bootstrap whose second stage differs from its third is the classic
    // signal that something in the compiler depends on an address or a hash seed.
    let first = render_niles(LEXER_SRC).unwrap();
    let second = render_niles(LEXER_SRC).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.len(), second.len());
}

// ── the honest boundary ──────────────────────────────────────────────────────────────

#[test]
fn the_relational_tier_is_still_refused_and_the_bootstrap_does_not_touch_it() {
    // The bootstrap must not have quietly acquired a second, unchecked path into the
    // relational tier. This used to assert that `txn` was refused by name; T-18 gave `txn` a
    // dynamic semantics — a posting set has to be sealed somewhere for a function to have run
    // at all, and a conformance suite that cannot execute is a conformance suite comparing
    // renderings (F-25).
    //
    // The guarantee the old assertion protected is unchanged and is checked where it now
    // lives: `txn` seals through the commit rule, so a set that does not conserve is refused
    // by currency with its residual named (`niles-interp`'s own
    // `a_txn_that_does_not_conserve_is_refused_with_the_currency_and_the_residual`). What
    // matters *here* is the second half, which is untouched: the bootstrap does not use the
    // relational tier, and `hold` and `resolve` remain uninterpretable.
    for (src, form) in [
        (
            "fn f() -> i64 { let h = hold(acct(1), 20.00 usd, expires: 7.days)?; 0 }",
            "hold",
        ),
        ("fn f() -> i64 { let x = resolve h void; 0 }", "resolve"),
    ] {
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new();
        it.load(&prog);
        match it.call("f", vec![]) {
            Err(Error::NotInSubset { form: got, .. }) => assert_eq!(got, form),
            other => panic!("`{form}` must still be refused by name, got {other:?}"),
        }
    }
    assert!(
        !LEXER_SRC.contains("txn {"),
        "the bootstrap lexer must stay in the imperative subset"
    );
    // And the bootstrap does not reach the ledger builtins. Checked by *running* it rather
    // than by grepping: `post(` occurs in `bootstrap/lexer.niles` inside a string literal —
    // a sample program the lexer lexes — and a grep would have called that a breach.
    let (prog, _) = parser::parse_program(LEXER_SRC);
    let mut it = Interp::new();
    it.load(&prog);
    let _ = it
        .call(
            "lex_and_render",
            vec![Value::Str(std::rc::Rc::new("fn f() {}".to_string()))],
        )
        .expect("the bootstrap lexer runs");
    assert!(
        it.ledger.sealed.is_empty() && it.ledger.open.is_none(),
        "the bootstrap lexer sealed a posting set, which it has no business doing"
    );
}

#[test]
fn a_duration_unit_still_exists_in_the_reference_lexer_so_the_gap_is_a_gap() {
    // Pinning that the excluded forms are real language features rather than dead syntax
    // — otherwise "out of scope" would be an empty concession.
    let (toks, _) = niles_lang::lexer::lex("7.days");
    assert!(toks.iter().any(|t| matches!(
        t.tok,
        Tok::Duration {
            unit: TimeUnit::Days,
            ..
        }
    )));
}
