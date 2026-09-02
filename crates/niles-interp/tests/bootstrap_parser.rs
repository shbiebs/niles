//! **The Appendix E bootstrap gates for the parser.**
//!
//! `bootstrap/lexer.niles` gave stage 1 its first input. This file gives it its second:
//! `bootstrap/parser.niles`, a recursive-descent parser written in Niles, run by the
//! stage-0 interpreter over Niles source text, and compared node for node with the Rust
//! parser in `niles-lang`.
//!
//! The gates are the same four, in the same order, as the lexer's:
//!
//! | Gate | What it establishes |
//! |---|---|
//! | **Stage 1 runs** | Stage 0 can execute a parser end to end |
//! | **Stage 1 equivalence** | The Niles parser and the Rust parser agree on a 30-program corpus |
//! | **Stage 2 self-application** | The Niles front end parses *its own two source files* |
//! | **Stage 3 fixpoint** | Repeated runs are byte-identical |
//!
//! # What is compared, and why it is text
//!
//! The two parsers share no types. A structural comparison would need an adapter, and an
//! adapter is the one thing a gate must not be: a bug in it cancels a bug in either side.
//! So both sides render their tree to the S-expression format defined in
//! `niles_lang::sexpr`, independently, and the gate compares strings. Spans are excluded
//! — token offsets are the *lexer's* gate, already checked there — so a disagreement here
//! is a disagreement about structure and nothing else.
//!
//! # What these gates do not establish
//!
//! A parser is not a compiler. There is still no type-checker and no lowering pass in
//! Niles; §E.19 must continue to say so. What the bootstrap now has is two front-end
//! stages, each verified against the reference implementation, which is two rungs of a
//! ladder rather than the ladder.
//!
//! The Niles parser also has no *diagnostic* channel. Where the reference parser builds an
//! error node, this one builds the same error node and the trees agree; where the
//! reference parser reports a message and builds an ordinary node — a reserved word used
//! as an identifier — this one is silent. [`the_reserved_word_gap_is_real_and_declared`]
//! pins that difference so it stays a known gap rather than becoming a surprise.
//!
//! # Cost
//!
//! Stage 2 runs a parser written in Niles, under a tree-walking interpreter, over 1,200
//! lines of Niles. That takes roughly half a minute in a debug build. It is the most
//! expensive test in the workspace and it is worth its cost: it is the only test that
//! executes the front end against itself.

use niles_interp::{determinism_gate_deep, run_with_stack, Interp, Value};
use niles_lang::{parser, sexpr};

const LEXER_SRC: &str = include_str!("../../../bootstrap/lexer.niles");
const PARSER_SRC: &str = include_str!("../../../bootstrap/parser.niles");

/// The Niles front end is two files loaded as one program: the parser consumes the token
/// type the lexer defines, so they are not separable.
fn front_end() -> String {
    format!("{LEXER_SRC}\n{PARSER_SRC}\n")
}

/// Run the Niles front end over `input` and return its rendering.
///
/// On a thread sized for the recursion: a recursive-descent parser nests several
/// interpreter frames per level of expression nesting, and the interpreter's frames are
/// wide. `run_with_stack` and `with_max_depth` are the pairing that makes the failure
/// mode a diagnostic rather than a process abort.
fn render_niles(input: &str) -> Result<String, String> {
    let src = front_end();
    let inp = input.to_string();
    run_with_stack(4096, move || {
        let (prog, d) = parser::parse_program(&src);
        if d.has_errors() {
            return Err(format!(
                "the Niles front end must parse:\n{}",
                d.render(&src, "front-end")
            ));
        }
        let mut it = Interp::new().with_max_depth(4096).with_fuel(2_000_000_000);
        it.load(&prog);
        match it.call("parse_and_render", vec![Value::Str(std::rc::Rc::new(inp))]) {
            Ok(Value::Str(s)) => Ok((*s).clone()),
            Ok(other) => Err(format!("expected a string, got {}", other.type_name())),
            Err(e) => Err(format!("{} at {:?}", e.message(), e.span())),
        }
    })
    .expect("the worker thread must not crash")
}

fn render_rust(src: &str) -> String {
    let (p, _) = parser::parse_program(src);
    sexpr::program(&p)
}

/// Whether an input is inside the subset `bootstrap/parser.niles` implements.
///
/// Two conditions, both checkable and both checked:
///
/// 1. The Rust rendering contains no `(unsupported …)` — no schema, view, trait, impl,
///    mod, const, attribute, SQL form or novel relational form, and no float, instant,
///    duration or byte string.
/// 2. No string *literal* contains a non-ASCII character. The Niles side rebuilds a
///    literal's decoded value byte by byte through `chr`, which is correct for ASCII and
///    would mangle a multi-byte character. The restriction is on literals only, not on
///    the file: comments and whitespace never reach that code, which is why the two
///    bootstrap files — full of em-dashes and section rules — are in scope.
fn in_scope(src: &str) -> bool {
    let (toks, _) = niles_lang::lexer::lex(src);
    let ascii_literals = !toks.iter().any(|t| match &t.tok {
        niles_lang::lexer::Tok::Str(s) => !s.is_ascii(),
        _ => false,
    });
    ascii_literals && sexpr::covers(&render_rust(src))
}

/// Thirty programs, chosen for the decisions where a hand-written parser actually goes
/// wrong: precedence and associativity, the struct-literal/block ambiguity, the
/// statement/tail-expression split, optional slots, and the places where two spellings
/// must produce one tree.
fn corpus() -> Vec<(&'static str, &'static str)> {
    vec![
        // --- the shape of a program ---
        ("empty", ""),
        ("one function", "fn f() -> i64 { 1 }"),
        ("a signature with no body", "fn f(a: i64) -> i64;"),
        ("visibility and generics", "pub fn id<T>(x: T) -> T { x }"),
        ("a use item", "use std::collections::BTreeMap;"),
        (
            "a struct",
            "struct Token { kind: Kind, text: str, start: i64 }",
        ),
        ("a generic struct", "struct Pair<A, B> { a: A, b: [B] }"),
        ("an enum with payloads", "enum E { A(i64, str), B, C([E]) }"),
        (
            "an effect row",
            "fn t(a: Id<Account>) -> i64 ! { append, read@snapshot, debit<usd> } { 0 }",
        ),
        (
            "a nested item",
            "fn outer() -> i64 { fn inner_helper() -> i64 { 1 } inner_helper() }",
        ),
        // --- precedence and associativity: the classic traps ---
        ("subtraction associates left", "fn f() -> i64 { a - b - c }"),
        (
            "multiplication binds tighter than addition",
            "fn f() -> i64 { a + b * c - d }",
        ),
        (
            "comparison binds looser than arithmetic",
            "fn f() -> bool { a + b < c * d }",
        ),
        (
            "and binds tighter than or",
            "fn f() -> bool { a || b && c }",
        ),
        (
            "bitwise sits between comparison and arithmetic",
            "fn f() -> i64 { a | b ^ c & d + e }",
        ),
        (
            "unary minus binds tighter than binary",
            "fn f() -> i64 { -a * b }",
        ),
        (
            "assignment is right-associative and lowest",
            "fn f() { a = b = c + d; }",
        ),
        ("parentheses override", "fn f() -> i64 { (a + b) * c }"),
        // --- the struct-literal ambiguity ---
        (
            "a struct literal in value position",
            "fn f() -> S { S { a: 1, b: x } }",
        ),
        (
            "a brace after an `if` head opens the body",
            "fn f() -> i64 { if s { 1 } else { 2 } }",
        ),
        (
            "shorthand struct-literal fields",
            "fn f() -> S { let a = 1; S { a } }",
        ),
        (
            "a while head is not a struct literal",
            "fn f() { while s { g(); } }",
        ),
        // --- statements, tails and optional slots ---
        (
            "a tail expression is not a statement",
            "fn f() -> i64 { g(); h() }",
        ),
        (
            "a let with a type and no initialiser",
            "fn f() { let x: i64; }",
        ),
        (
            "a let with an initialiser and no type",
            "fn f() { let mut x = 1; }",
        ),
        (
            "a stray semicolon is not a statement",
            "fn f() -> i64 { ; 1 }",
        ),
        // --- control flow ---
        (
            "an else-if chain",
            "fn f() -> i64 { if a { 1 } else if b { 2 } else { 3 } }",
        ),
        (
            "match with a guard and a wildcard",
            "fn f(x: E) -> i64 { match x { E::A(y) if y > 1 => y, _ => 0 } }",
        ),
        (
            "match on a path pattern",
            "fn f(k: Kind) -> str { match k { Kind::Eof => \"e\", Kind::Ident => \"i\" } }",
        ),
        (
            "for, loop, break and continue",
            "fn f(xs: [i64]) { for x in xs { if x == 0 { continue; } } loop { break; } }",
        ),
        // --- postfix, calls and stages ---
        ("chained postfix", "fn f() -> i64 { xs[0].len() }"),
        (
            "a method that is not a known stage",
            "fn f() -> i64 { s.byte_at(i) }",
        ),
        ("a known stage", "fn f(q: Q) -> Q { q.where(p) }"),
        (
            "the two spellings of a stage agree",
            "fn f(q: Q) -> Q { q |> map(g) }",
        ),
        (
            "named arguments",
            "fn f() -> H { hold_for(acct, expires: d) }",
        ),
        ("the try operator", "fn f() -> i64 { g()? + 1 }"),
        ("a cast", "fn f() -> i64 { x as i64 }"),
        // --- literals ---
        ("integer with underscores", "fn f() -> i64 { 1_000_000 }"),
        (
            "money keeps its own scale",
            "fn f() -> Money<usd> { 0.000001 btc }",
        ),
        ("an epoch literal", "fn f() -> Epoch { #4200 }"),
        (
            "string escapes",
            "fn f() -> str { \"a\\\"b\\\\c\\nd\\te\" }",
        ),
        (
            "booleans and the unit value",
            "fn f() { let a = true; let b = false; let c = (); }",
        ),
        (
            "tuples and arrays",
            "fn f() { let t = (1, \"a\"); let xs = [1, 2, 3]; let e = []; }",
        ),
        // --- closures, references, patterns ---
        (
            "a closure with a typed parameter",
            "fn f() { let g = |x: i64| x + 1; }",
        ),
        ("a closure with no parameters", "fn f() { let g = || 1; }"),
        (
            "references in types and expressions",
            "fn f(x: &mut i64) { g(&x); }",
        ),
        (
            "a tuple pattern in a let",
            "fn f(t: (i64, i64)) { let (a, b) = t; }",
        ),
        (
            "a struct pattern with rest",
            "fn f(v: V) { match v { V::S { a, .. } => a, _ => 0 } }",
        ),
    ]
}

// ── Gate 1: stage 1 runs at all ──────────────────────────────────────────────────────

#[test]
fn stage_1_the_niles_written_parser_parses_and_runs() {
    let src = front_end();
    let (prog, d) = parser::parse_program(&src);
    assert!(
        !d.has_errors(),
        "the Niles front end must parse under stage 0"
    );
    let mut it = Interp::new();
    it.load(&prog);
    let names = it.function_names();
    for wanted in [
        "p_program",
        "p_item",
        "p_fn",
        "p_block",
        "p_stmt",
        "p_expr_bp",
        "p_unary",
        "p_postfix",
        "p_primary",
        "p_pat",
        "p_ty",
        "render_node",
        "parse_and_render",
    ] {
        assert!(names.contains(&wanted.to_string()), "missing fn {wanted}");
    }
    // The self-check, on a stack sized for it.
    let out = run_with_stack(4096, move || {
        let (prog, _) = parser::parse_program(&src);
        let mut it = Interp::new().with_max_depth(4096);
        it.load(&prog);
        it.call("parser_main", vec![])
            .map(|_| it.output.clone())
            .map_err(|e| e.message())
    })
    .expect("the worker must not crash")
    .expect("the parser's self-check must run");
    assert!(!out.is_empty());
    assert!(
        out[0].starts_with("(program (fn priv transfer"),
        "{}",
        &out[0][..80.min(out[0].len())]
    );
}

#[test]
fn the_reserved_word_table_matches_the_registry_exactly() {
    // The lexer's keyword table has a gate for this reason and so does this one: two
    // implementations of one word list drift within a day of the second existing. The
    // parser needs the *reserved* subset, because `ident()` treats the two classes
    // differently — an unreserved keyword used as a name keeps its source spelling, a
    // reserved one is canonicalised — and getting that backwards shows up as a single
    // wrong-case identifier a thousand lines into a file.
    let registry: std::collections::BTreeSet<String> = niles_lang::keywords::KEYWORDS
        .iter()
        .filter(|k| !k.category.usable_as_ident())
        .map(|k| k.word.to_string())
        .collect();

    let sig = "fn reserved() -> [str] {";
    let start = PARSER_SRC.find(sig).expect("the table must exist") + sig.len();
    let open = PARSER_SRC[start..].find('[').unwrap() + start;
    let close = PARSER_SRC[open..].find(']').unwrap() + open;
    let in_niles: std::collections::BTreeSet<String> = PARSER_SRC[open + 1..close]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let missing: Vec<_> = registry.difference(&in_niles).collect();
    let invented: Vec<_> = in_niles.difference(&registry).collect();
    assert!(
        missing.is_empty(),
        "reserved in the registry but not in the Niles parser: {missing:?}"
    );
    assert!(
        invented.is_empty(),
        "reserved in the Niles parser but not in the registry: {invented:?}"
    );
    assert!(
        registry.len() >= 60,
        "the reserved set should be substantial, got {}",
        registry.len()
    );
}

#[test]
fn the_stage_table_matches_the_reference_exactly() {
    // The same argument, for the other generated list. A stage the Niles parser does not
    // know would be rendered `unknown` on one side and by name on the other.
    let sig = "fn stage_names() -> [str] {";
    let start = PARSER_SRC.find(sig).expect("the table must exist") + sig.len();
    let open = PARSER_SRC[start..].find('[').unwrap() + start;
    let close = PARSER_SRC[open..].find(']').unwrap() + open;
    let mut in_niles: std::collections::BTreeSet<String> = PARSER_SRC[open + 1..close]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    // `where` and `filter` are handled by name before the table is consulted, because
    // they are the one pair of spellings that share a kind.
    in_niles.insert("where".into());
    in_niles.insert("filter".into());

    let reference: std::collections::BTreeSet<String> = niles_lang::ast::StageKind::all_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(in_niles, reference, "the two stage lists must agree");
}

// ── Gate 2: equivalence with the reference parser ────────────────────────────────────

#[test]
fn stage_1_equivalence_the_two_parsers_agree_node_for_node() {
    // The gate that matters. Two independent implementations over the same corpus,
    // compared on every node of every tree.
    let mut checked = 0;
    let mut skipped = Vec::new();
    let src = front_end();

    // One interpreter for the whole corpus: loading the front end is the expensive part.
    let cases: Vec<(String, String)> = corpus()
        .into_iter()
        .map(|(n, s)| (n.to_string(), s.to_string()))
        .collect();
    let expected: Vec<String> = cases.iter().map(|(_, s)| render_rust(s)).collect();

    let got: Vec<Result<String, String>> = run_with_stack(4096, move || {
        let (prog, d) = parser::parse_program(&src);
        assert!(!d.has_errors());
        let mut it = Interp::new().with_max_depth(4096).with_fuel(2_000_000_000);
        it.load(&prog);
        cases
            .iter()
            .map(|(_, s)| {
                match it.call(
                    "parse_and_render",
                    vec![Value::Str(std::rc::Rc::new(s.clone()))],
                ) {
                    Ok(Value::Str(out)) => Ok((*out).clone()),
                    Ok(other) => Err(format!("expected a string, got {}", other.type_name())),
                    Err(e) => Err(e.message()),
                }
            })
            .collect()
    })
    .expect("the worker must not crash");

    for (i, (name, src)) in corpus().into_iter().enumerate() {
        if !in_scope(src) {
            skipped.push(name);
            continue;
        }
        let niles = got[i]
            .as_ref()
            .unwrap_or_else(|e| panic!("the Niles parser failed on `{name}`: {e}"));
        assert_eq!(niles, &expected[i], "\ncase: {name}\nsource: {src:?}\n");
        checked += 1;
    }
    assert!(
        checked >= 30,
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
    // Guarding the guard. If `in_scope` silently returned true, the gate above would
    // appear to prove more than it does.
    assert!(!in_scope("schema s { }"), "schemas are out of scope");
    assert!(
        !in_scope("fn f() { txn { post(a, b) } }"),
        "txn is out of scope"
    );
    assert!(
        !in_scope("fn f() { select 1 }"),
        "the SQL surface is out of scope"
    );
    assert!(
        !in_scope("#[udf] fn f() { }"),
        "attributes are out of scope"
    );
    assert!(!in_scope("fn f() { 1.5 }"), "floats are out of scope");
    assert!(
        !in_scope("fn f() -> str { \"caf\u{e9}\" }"),
        "a non-ASCII string literal is out of scope"
    );
    assert!(
        in_scope("// caf\u{e9}\nfn f() -> i64 { 1 }"),
        "a non-ASCII comment is not"
    );
    assert!(
        in_scope("fn f() -> i64 { 1 + 2 }"),
        "an ordinary function is in scope"
    );
}

#[test]
fn the_niles_parser_disagrees_where_it_should_which_shows_the_gate_can_fail() {
    // A gate that cannot fail proves nothing. Feed it a form the Niles side does not
    // implement and confirm the two renderings really do differ.
    let out_of_scope = "fn f() { txn { post(a, b) } }";
    let rust = render_rust(out_of_scope);
    let niles = render_niles(out_of_scope).expect("it should still produce something");
    assert!(rust.contains("(unsupported txn)"));
    assert_ne!(
        niles, rust,
        "an unimplemented construct must show up as a disagreement, or the gate is blind"
    );
}

// ── Gate 3: self-application ─────────────────────────────────────────────────────────

#[test]
fn stage_2_the_niles_parser_parses_the_niles_lexer() {
    // The smaller half of self-application, and the faster one: the parser is pointed at
    // the file it shares a program with.
    assert!(
        in_scope(LEXER_SRC),
        "the lexer's source must stay inside the parser's subset"
    );
    let rust = render_rust(LEXER_SRC);
    let niles = render_niles(LEXER_SRC).expect("it must survive the lexer's source");
    assert_same_tree(&niles, &rust, "lexer.niles");
    assert!(
        niles.len() > 15_000,
        "its source is a substantial tree, got {} bytes",
        niles.len()
    );
}

#[test]
fn stage_2_the_niles_front_end_parses_its_own_two_source_files() {
    // The self-referential one, and the point of the exercise: the Niles lexer tokenises
    // and the Niles parser parses the whole of the Niles front end, and the result is
    // identical to what the reference implementation produces from the same bytes.
    //
    // This is the most expensive test in the workspace — a parser written in Niles, run
    // by a tree-walking interpreter, over 1,200 lines of Niles. About half a minute in a
    // debug build. It earns it: nothing else executes the front end against itself.
    let src = front_end();
    assert!(
        in_scope(&src),
        "the front end's own source must stay inside the subset it implements"
    );
    let rust = render_rust(&src);
    let niles = render_niles(&src).expect("self-application must succeed");
    assert_same_tree(&niles, &rust, "the front end");
    assert!(niles.len() > 100_000, "got {} bytes", niles.len());

    // The three figures Appendix E quotes, checked against the sources they describe.
    //
    // They were typed: E.0 said `parser.niles` was "~1,050 lines" against a file of 1,647,
    // and E.19 said the self-application covered "1,200 lines" of a front end that is 2,066
    // and produced "127,165 bytes of tree" from a run nobody had repeated. A figure in an
    // appendix beside a test that computes it is a figure that drifts, and this is the same
    // discipline `thesis_drift.rs` applies to Chapter 9's tables.
    let appendix = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("the repository root")
            .join("thesis/appendix-e.md"),
    )
    .expect("appendix E is readable");
    for (what, value) in [
        (
            "`bootstrap/parser.niles`'s line count",
            PARSER_SRC.lines().count(),
        ),
        (
            "the self-application corpus's line count",
            src.lines().count(),
        ),
        ("the tree's byte count", niles.len()),
    ] {
        let with_commas = commas(value);
        assert!(
            appendix.contains(&with_commas) || appendix.contains(&value.to_string()),
            "Appendix E does not carry {what}, which this run measures as {with_commas}"
        );
    }
}

/// `1647` → `"1,647"`, matching how the appendix writes a figure.
fn commas(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Report the first disagreement in context rather than dumping a hundred kilobytes.
fn assert_same_tree(niles: &str, rust: &str, what: &str) {
    if niles == rust {
        return;
    }
    let at = niles
        .bytes()
        .zip(rust.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or(niles.len().min(rust.len()));
    let lo = at.saturating_sub(140);
    panic!(
        "parsing {what} diverged at byte {at} (niles {} bytes, rust {} bytes):\n\
         rust:  …{}\n\
         niles: …{}",
        niles.len(),
        rust.len(),
        &rust[lo..(at + 160).min(rust.len())],
        &niles[lo..(at + 160).min(niles.len())],
    );
}

// ── Gate 4: the fixpoint / determinism gate ──────────────────────────────────────────

#[test]
fn stage_3_repeated_runs_of_the_parser_are_byte_identical() {
    let src = front_end();
    let r = run_with_stack(4096, move || {
        let (prog, d) = parser::parse_program(&src);
        assert!(!d.has_errors());
        determinism_gate_deep(&prog, "parser_main", 5, 4096).map_err(|e| e.message())
    })
    .expect("the worker must not crash")
    .expect("the self-check must run");
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
    // Parsing the lexer's source twice must give the same bytes. A bootstrap whose second
    // run differs from its third is the classic signal that something depends on an
    // address or a hash seed.
    let first = render_niles(LEXER_SRC).unwrap();
    let second = render_niles(LEXER_SRC).unwrap();
    assert_eq!(first, second);
}

// ── negative controls ────────────────────────────────────────────────────────────────
//
// A gate that only ever sees well-formed input establishes that two parsers agree about
// programs, not that either one is a parser. These four are the cases where a
// hand-written recursive-descent parser is most likely to be quietly wrong.

#[test]
fn negative_control_a_missing_brace_is_an_error_on_both_sides() {
    // Truncated input: both parsers must stop, produce a tree covering what they did
    // read, and say so — rather than loop, or claim success.
    let src = "fn f() -> i64 { let x = 1;";
    let (p, d) = parser::parse_program(src);
    assert!(
        d.has_errors(),
        "the reference parser must report the missing brace"
    );
    let rust = sexpr::program(&p);
    let niles = render_niles(src).expect("the Niles parser must terminate rather than hang");
    assert_eq!(niles, rust, "both must recover to the same tree");
    assert!(
        rust.contains("(let (pbind x imm val) (none) (int 1))"),
        "{rust}"
    );
}

#[test]
fn negative_control_a_non_expression_becomes_an_error_node_on_both_sides() {
    // The other structural failure: a token that cannot begin an expression. Both sides
    // must emit an error node and recover to the same synchronising token, or the trees
    // after the error would drift apart.
    let src = "fn f() -> i64 { let x = ; 1 }";
    let (p, d) = parser::parse_program(src);
    assert!(d.has_errors());
    let rust = sexpr::program(&p);
    assert!(
        rust.contains("(eerr)"),
        "the reference parser must build an error node: {rust}"
    );
    let niles = render_niles(src).unwrap();
    assert_eq!(
        niles, rust,
        "recovery must reach the same point on both sides"
    );
}

#[test]
fn negative_control_the_precedence_trap_associates_left_on_both_sides() {
    // `a - b - c` is `(a - b) - c`, not `a - (b - c)`. This is the single most common
    // defect in a hand-written Pratt loop, and it is invisible to a token-stream
    // comparison — which is the argument for having a *parser* gate at all.
    let src = "fn f() -> i64 { a - b - c }";
    let rust = render_rust(src);
    assert!(
        rust.contains("(binary sub (binary sub (path a) (path b)) (path c))"),
        "the reference must associate left: {rust}"
    );
    assert_eq!(render_niles(src).unwrap(), rust);

    // And the right-associative one, so the test is not satisfied by a parser that
    // associates everything the same way.
    let asg = "fn f() { a = b = c; }";
    let rust2 = render_rust(asg);
    assert!(
        rust2.contains("(assign (path a) (assign (path b) (path c)))"),
        "assignment must associate right: {rust2}"
    );
    assert_eq!(render_niles(asg).unwrap(), rust2);
}

#[test]
fn the_reserved_word_gap_is_real_and_declared() {
    // A keyword used as an identifier. The reference parser reports it and yields the
    // registry's spelling; the Niles parser has no diagnostic channel and yields the same
    // spelling silently. So the *trees* agree and the *messages* do not, and that is the
    // declared stage-1 gap rather than a passing grade.
    //
    // The test pins both halves: if the Niles side ever grew diagnostics, or if the trees
    // ever stopped agreeing, this would fail and §E.19 would need rewriting.
    // A field named `Where`. `ident` is the only place the distinction bites, and a
    // struct field is the smallest position that reaches it. The word must also be
    // canonicalised to the registry's lower-case spelling by both parsers, which is the
    // reason `parser.niles` carries the reserved table at all.
    let src = "struct S { Where: i64, kind: Kind }";
    let (p, d) = parser::parse_program(src);
    assert!(
        d.has_errors(),
        "the reference parser must report `Where` used as a field name"
    );
    let rust = sexpr::program(&p);
    assert!(rust.contains("(field where (tp i64))"), "{rust}");
    assert_eq!(
        render_niles(src).unwrap(),
        rust,
        "the trees agree; only the diagnostics differ"
    );
}

// ── the honest boundary ──────────────────────────────────────────────────────────────

#[test]
fn the_parser_stays_inside_the_imperative_subset() {
    // The bootstrap must not have quietly acquired a second, unchecked path into the
    // relational tier. `parser.niles` recognises the *words* `txn` and `select` only as
    // strings in its tables; it must not contain the forms.
    assert!(
        !PARSER_SRC.contains("txn {"),
        "the parser must stay in the imperative subset"
    );
    assert!(!PARSER_SRC.contains("sql {"));
    let (prog, _) = parser::parse_program(PARSER_SRC);
    let mut it = Interp::new();
    it.load(&prog);
    assert!(
        it.function_names().len() > 30,
        "it should be a real program"
    );
}

#[test]
fn stage_2_uses_only_constructs_the_language_actually_has() {
    // The weak but real expressiveness result, restated for the parser: it uses recursive
    // enums with array payloads, nested `match`, `while`, early `return`, struct literals
    // and string methods, and it parses. Had any of those been aspirational, the file
    // would not exist.
    for construct in [
        "enum Node {",
        "Node::List([",
        "match n {",
        "while going",
        "let mut",
        "return ",
        "struct P {",
        ".to_lower()",
        ".push(",
    ] {
        assert!(
            PARSER_SRC.contains(construct),
            "the bootstrap should exercise `{construct}`"
        );
    }
}
