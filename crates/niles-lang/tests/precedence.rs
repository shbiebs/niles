//! **Appendix B.10.1 against the parser.**
//!
//! The syntax-lineage rule of §6.25 makes the appendix normative: where the appendix and
//! the implementation disagree, the appendix wins and the implementation gets a failing
//! test. That is a rule about documents unless something reads both, so this file does.
//!
//! It parses the operator table out of `thesis/appendix-b.md` and checks it three ways:
//! that every level in the table matches `BinOp::precedence`, that the parser actually
//! associates the way the table says, and that the two spellings of `and` and `or` really
//! do produce one tree.
//!
//! # Why this file exists
//!
//! The table did not exist until a second implementation of the grammar was written.
//! `bootstrap/parser.niles` was built against it, the two parsers disagreed about
//! `a = b = c`, and the disagreement turned out to be a defect in the *reference* parser:
//! it recursed for an assignment's right-hand side at binding power 1, which put
//! assignment outside its own `min_bp == 0` guard and made it left-associative — the
//! opposite of Rust's rule and of the comment sitting directly above the code. No test
//! had asked, because a token-stream comparison cannot see associativity and nothing else
//! was looking.
//!
//! So this file is the arbiter that was missing, and the assignment case below is a
//! regression test for a real defect rather than a hypothetical one.

use niles_lang::ast::BinOp;
use niles_lang::{parser, sexpr};

const APPENDIX: &str = include_str!("../../../thesis/appendix-b.md");

/// The B.10.1 table, as `(power, operator spellings)`.
fn table() -> Vec<(u8, Vec<String>)> {
    let start = APPENDIX
        .find("### B.10.1 Operator Precedence and Associativity")
        .expect("Appendix B.10.1 must exist — it is normative for this test");
    let body = &APPENDIX[start..];
    let end = body.find("\n## ").unwrap_or(body.len());
    let mut out = Vec::new();
    for line in body[..end].lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        // A cell may *contain* a pipe — `|` is an operator — escaped as `\|` in the
        // markdown. Hiding the escaped ones before splitting on the real column
        // separators is the whole trick; without it the bitwise-or row splits in half and
        // the table silently loses an operator.
        const PIPE: &str = "\u{1}";
        let hidden = line.replace("\\|", PIPE);
        let cells: Vec<String> = hidden
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().to_string())
            .collect();
        if cells.len() < 3 {
            continue;
        }
        let Ok(power) = cells[0].parse::<u8>() else {
            continue;
        };
        // Split on backticks rather than whitespace: `is not` and `not in` are single
        // operators spelled with a space, and whitespace splitting would turn each into
        // two words that name nothing.
        let ops: Vec<String> = cells[1]
            .split('`')
            .skip(1)
            .step_by(2)
            .map(|w| w.trim().replace(PIPE, "|"))
            .filter(|w| !w.is_empty())
            .collect();
        out.push((power, ops));
    }
    out
}

/// The appendix spelling of an operator, mapped to the `BinOp` it denotes.
///
/// Deliberately explicit rather than derived: a mapping computed from the parser would
/// make the test check the parser against itself.
fn binop_of(spelling: &str) -> Option<BinOp> {
    Some(match spelling {
        "or" | "||" => BinOp::Or,
        "and" | "&&" => BinOp::And,
        "==" => BinOp::Eq,
        "!=" => BinOp::Ne,
        "<" => BinOp::Lt,
        "<=" => BinOp::Le,
        ">" => BinOp::Gt,
        ">=" => BinOp::Ge,
        "is" => BinOp::Is,
        "is not" => BinOp::IsNot,
        "in" => BinOp::In,
        "not in" => BinOp::NotIn,
        "like" => BinOp::Like,
        "between" => BinOp::Between,
        "|" => BinOp::BitOr,
        "^" => BinOp::BitXor,
        "&" => BinOp::BitAnd,
        "+" => BinOp::Add,
        "-" => BinOp::Sub,
        "*" => BinOp::Mul,
        "/" => BinOp::Div,
        "%" => BinOp::Rem,
        _ => return None,
    })
}

fn render(src: &str) -> String {
    let (p, _) = parser::parse_program(src);
    sexpr::program(&p)
}

#[test]
fn the_appendix_table_is_present_and_complete() {
    let t = table();
    assert_eq!(
        t.len(),
        8,
        "B.10.1 must have eight numbered binding powers, got {}",
        t.len()
    );
    let powers: Vec<u8> = t.iter().map(|(p, _)| *p).collect();
    assert_eq!(
        powers,
        vec![1, 2, 3, 4, 5, 6, 7, 8],
        "the levels must be 1..=8 in order"
    );

    // Every binary operator the compiler has must appear somewhere in the table, or the
    // table is documenting a subset and calling itself normative.
    let spelled: Vec<BinOp> = t
        .iter()
        .flat_map(|(_, ops)| ops.iter())
        .filter_map(|s| binop_of(s))
        .collect();
    for op in [
        BinOp::Add,
        BinOp::Sub,
        BinOp::Mul,
        BinOp::Div,
        BinOp::Rem,
        BinOp::Eq,
        BinOp::Ne,
        BinOp::Lt,
        BinOp::Le,
        BinOp::Gt,
        BinOp::Ge,
        BinOp::And,
        BinOp::Or,
        BinOp::BitAnd,
        BinOp::BitOr,
        BinOp::BitXor,
        BinOp::In,
        BinOp::Like,
        BinOp::Between,
    ] {
        assert!(
            spelled.contains(&op),
            "{op:?} is in the compiler but not in B.10.1"
        );
    }
}

#[test]
fn every_level_in_the_appendix_matches_the_compiler() {
    // The drift guard. If someone changes `BinOp::precedence` without changing the
    // appendix, or the other way round, this is what says so.
    for (power, ops) in table() {
        for spelling in &ops {
            let Some(op) = binop_of(spelling) else {
                panic!("B.10.1 lists `{spelling}`, which maps to no operator in the compiler");
            };
            assert_eq!(
                op.precedence(),
                power,
                "B.10.1 puts `{spelling}` at power {power}; the compiler puts it at {}",
                op.precedence()
            );
        }
    }
}

#[test]
fn the_parser_associates_the_way_the_table_says() {
    // Left-associative everywhere the table says left. Rendered trees, because
    // associativity is invisible in a token stream — which is exactly why the reference
    // parser could be wrong about it for as long as it was.
    assert!(render("fn f() { a - b - c; }")
        .contains("(binary sub (binary sub (path a) (path b)) (path c))"));
    assert!(render("fn f() { a / b / c; }")
        .contains("(binary div (binary div (path a) (path b)) (path c))"));
    assert!(render("fn f() { a || b || c; }")
        .contains("(binary or (binary or (path a) (path b)) (path c))"));

    // Right-associative where the table says right. This is the regression test for the
    // defect the second implementation found.
    assert!(
        render("fn f() { a = b = c; }").contains("(assign (path a) (assign (path b) (path c)))"),
        "assignment must associate right, following Rust: {}",
        render("fn f() { a = b = c; }")
    );
    // And an assignment's right-hand side still takes an ordinary expression, so the fix
    // did not simply move the bracket.
    assert!(render("fn f() { a = b + c * d; }")
        .contains("(assign (path a) (binary add (path b) (binary mul (path c) (path d))))"));
}

#[test]
fn the_levels_actually_nest_in_the_order_the_table_gives() {
    // Adjacent levels, checked pairwise, so a table that were merely internally
    // consistent could not pass. Each case is written so the *tighter* operator ends up
    // as the inner node.
    let cases = [
        // 8 over 7
        (
            "fn f() { a + b * c; }",
            "(binary add (path a) (binary mul (path b) (path c)))",
        ),
        // 7 over 6
        (
            "fn f() { a & b + c; }",
            "(binary bitand (path a) (binary add (path b) (path c)))",
        ),
        // 6 over 5
        (
            "fn f() { a ^ b & c; }",
            "(binary bitxor (path a) (binary bitand (path b) (path c)))",
        ),
        // 5 over 4
        (
            "fn f() { a | b ^ c; }",
            "(binary bitor (path a) (binary bitxor (path b) (path c)))",
        ),
        // 4 over 3
        (
            "fn f() { a == b | c; }",
            "(binary eq (path a) (binary bitor (path b) (path c)))",
        ),
        // 3 over 2
        (
            "fn f() { a && b == c; }",
            "(binary and (path a) (binary eq (path b) (path c)))",
        ),
        // 2 over 1
        (
            "fn f() { a || b && c; }",
            "(binary or (path a) (binary and (path b) (path c)))",
        ),
    ];
    for (src, want) in cases {
        let got = render(src);
        assert!(got.contains(want), "{src}\n  want: {want}\n  got:  {got}");
    }
}

#[test]
fn a_word_and_its_symbol_produce_one_tree() {
    // Niles has two ancestries and one meaning. `and`/`&&` and `or`/`||` must be the same
    // operator at the same level, or a program's meaning would depend on which language
    // the author was thinking in.
    assert_eq!(render("fn f() { a and b; }"), render("fn f() { a && b; }"));
    assert_eq!(render("fn f() { a or b; }"), render("fn f() { a || b; }"));
    assert_eq!(BinOp::And.precedence(), 2);
    assert_eq!(BinOp::Or.precedence(), 1);
}

#[test]
fn unary_binds_tighter_than_every_binary_level() {
    // The table's last row. A prefix operator that bound looser than `*` would make
    // `-a * b` mean `-(a * b)`, which is the same number for negation and a different one
    // for `!`.
    assert!(render("fn f() { -a * b; }").contains("(binary mul (unary neg (path a)) (path b))"));
    assert!(render("fn f() { !a && b; }").contains("(binary and (unary not (path a)) (path b))"));
    // Postfix binds tighter still, and applies to the operand rather than the result.
    assert!(render("fn f() { -a.b; }").contains("(unary neg (field (path a) b))"));
}
