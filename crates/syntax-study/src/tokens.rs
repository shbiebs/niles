//! **The surface-neutral tokenizer** (design §6, expressiveness): identifiers and keywords,
//! numbers, strings, and each operator or punctuation mark are one token each; comments and
//! whitespace are none.
//!
//! One set of rules for every surface. What differs by surface is only what is a *lexical
//! fact* of that surface and not a choice: which characters open a comment (`--` in SQL, `#`
//! in PRQL, `//` in Niles and Soufflé), and which quote opens a string (`'` in SQL, where `"`
//! quotes an identifier; `"` elsewhere, and both in PRQL). Everything else — what an operator
//! is, what a number is — is decided here once:
//!
//! * a number is a run of digits with `_` separators and an optional fraction, and an ISO
//!   date `YYYY-MM-DD` is one number (a literal is a literal, however the surface spells it);
//! * an operator is the longest match from [`OPERATORS`], else one character;
//! * PL/pgSQL's `$$` body delimiter is one punctuation token, and the body is tokenised as
//!   code, because it is the program.

/// What a token is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Word,
    Number,
    Str,
    Op,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub text: String,
    /// Byte range in the source: mutants are made by splicing the source, not by printing
    /// tokens, so a mutant differs from its program in exactly the edited span.
    pub start: usize,
    pub end: usize,
    pub line: u32,
}

/// Multi-character operators, longest first.
pub const OPERATORS: &[&str] = &[
    "$$", "::", ":=", ":-", "->", "=>", "==", "!=", "<>", "<=", ">=", "&&", "||", "..",
];

/// Brackets and separators: tokens, but not operators when constructs are counted.
pub const PUNCTUATION: &[&str] = &["(", ")", "[", "]", "{", "}", ",", ";"];

/// The lexical facts of a surface the tokenizer needs.
#[derive(Debug, Clone, Copy)]
pub struct Lexis {
    pub line_comment: &'static str,
    pub block_comment: bool,
    pub string_quotes: &'static [char],
    pub ident_quote: Option<char>,
}

pub fn lexis(surface: &str) -> Lexis {
    match surface {
        "SQL" => Lexis {
            line_comment: "--",
            block_comment: true,
            string_quotes: &['\''],
            ident_quote: Some('"'),
        },
        "PRQL" => Lexis {
            line_comment: "#",
            block_comment: false,
            string_quotes: &['"', '\''],
            ident_quote: Some('`'),
        },
        // NL, RS and DL.
        _ => Lexis {
            line_comment: "//",
            block_comment: true,
            string_quotes: &['"'],
            ident_quote: None,
        },
    }
}

fn is_date_at(b: &[u8], i: usize) -> bool {
    let pat = b"dddd-dd-dd";
    b.len() >= i + pat.len()
        && pat.iter().enumerate().all(|(k, p)| {
            let c = b[i + k];
            if *p == b'd' {
                c.is_ascii_digit()
            } else {
                c == *p
            }
        })
        && !b.get(i + pat.len()).is_some_and(|c| c.is_ascii_digit())
}

pub fn tokenize(src: &str, surface: &str) -> Vec<Token> {
    let lx = lexis(surface);
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 1u32;
    while i < b.len() {
        let c = b[i] as char;
        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if src[i..].starts_with(lx.line_comment) {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if lx.block_comment && src[i..].starts_with("/*") {
            let end = src[i + 2..]
                .find("*/")
                .map(|e| i + 2 + e + 2)
                .unwrap_or(b.len());
            line += src[i..end].matches('\n').count() as u32;
            i = end;
            continue;
        }
        let start = i;
        let (kind, end) = if lx.string_quotes.contains(&c) || lx.ident_quote == Some(c) {
            let q = b[i];
            let mut j = i + 1;
            loop {
                if j >= b.len() {
                    break;
                }
                if b[j] == q {
                    // SQL doubles a quote to escape it.
                    if b.get(j + 1) == Some(&q) {
                        j += 2;
                        continue;
                    }
                    j += 1;
                    break;
                }
                if b[j] == b'\\' && surface != "SQL" {
                    j += 1;
                }
                j += 1;
            }
            let kind = if lx.ident_quote == Some(c) {
                Kind::Word
            } else {
                Kind::Str
            };
            (kind, j.min(b.len()))
        } else if c.is_ascii_digit() {
            if is_date_at(b, i) {
                (Kind::Number, i + 10)
            } else {
                let mut j = i;
                while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'_') {
                    j += 1;
                }
                if j + 1 < b.len() && b[j] == b'.' && b[j + 1].is_ascii_digit() {
                    j += 1;
                    while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'_') {
                        j += 1;
                    }
                }
                (Kind::Number, j)
            }
        } else if c.is_alphabetic() || c == '_' {
            let mut j = i;
            while j < b.len() && ((b[j] as char).is_alphanumeric() || b[j] == b'_') {
                j += 1;
            }
            (Kind::Word, j)
        } else {
            let op = OPERATORS.iter().find(|o| src[i..].starts_with(**o));
            match op {
                Some(o) => (Kind::Op, i + o.len()),
                None => (Kind::Op, i + c.len_utf8()),
            }
        };
        line += src[start..end].matches('\n').count() as u32;
        out.push(Token {
            kind,
            text: src[start..end].to_string(),
            start,
            end,
            line,
        });
        i = end;
    }
    out
}

/// Whether keyword matching ignores case on this surface: SQL's keywords do, and Niles's
/// (its registry: "case-insensitive on input"); PRQL's and Soufflé's do not.
pub fn case_insensitive(surface: &str) -> bool {
    matches!(surface, "SQL" | "NL" | "RS")
}

/// Distinct constructs (design §6): the distinct tokens that are in the surface's keyword
/// list, plus the distinct operator tokens (punctuation is not an operator).
pub fn constructs(toks: &[Token], surface: &str, keywords: &[String]) -> (usize, usize) {
    use std::collections::BTreeSet;
    let ci = case_insensitive(surface);
    let norm = |s: &str| if ci { s.to_lowercase() } else { s.to_string() };
    let kw: BTreeSet<String> = keywords.iter().map(|k| norm(k)).collect();
    let words: BTreeSet<String> = toks
        .iter()
        .filter(|t| t.kind == Kind::Word)
        .map(|t| norm(&t.text))
        .filter(|w| kw.contains(w))
        .collect();
    let ops: BTreeSet<&str> = toks
        .iter()
        .filter(|t| t.kind == Kind::Op && !PUNCTUATION.contains(&t.text.as_str()))
        .map(|t| t.text.as_str())
        .collect();
    (words.len(), ops.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(src: &str, s: &str) -> Vec<String> {
        tokenize(src, s).into_iter().map(|t| t.text).collect()
    }

    #[test]
    fn one_rule_per_token_class() {
        assert_eq!(
            texts(
                "select acct, (sum(amt_usd)).minor -- c\nfrom t where x <> 'a''b'",
                "SQL"
            ),
            [
                "select", "acct", ",", "(", "sum", "(", "amt_usd", ")", ")", ".", "minor", "from",
                "t", "where", "x", "<>", "'a''b'"
            ]
        );
        assert_eq!(
            texts("p.as_of(#5).valid_at(v@2026-01-21) // c", "NL"),
            [
                "p",
                ".",
                "as_of",
                "(",
                "#",
                "5",
                ")",
                ".",
                "valid_at",
                "(",
                "v",
                "@",
                "2026-01-21",
                ")"
            ]
        );
        assert_eq!(
            texts("filter cur == \"usd\" # c", "PRQL"),
            ["filter", "cur", "==", "\"usd\""]
        );
        assert_eq!(
            texts("a(x) :- b(x, 1_000), x != 2.5.", "DL"),
            [
                "a", "(", "x", ")", ":-", "b", "(", "x", ",", "1_000", ")", ",", "x", "!=", "2.5",
                "."
            ]
        );
    }

    #[test]
    fn a_plpgsql_body_is_code() {
        let t = texts("as $$ begin perform f(); end $$;", "SQL");
        assert_eq!(
            t,
            ["as", "$$", "begin", "perform", "f", "(", ")", ";", "end", "$$", ";"]
        );
    }

    #[test]
    fn constructs_count_keywords_and_operators_not_punctuation() {
        let t = tokenize("select a, b from t where a = 1 and b <> 2;", "SQL");
        let kw: Vec<String> = ["select", "from", "where", "and"]
            .map(String::from)
            .to_vec();
        assert_eq!(constructs(&t, "SQL", &kw), (4, 2));
    }
}
