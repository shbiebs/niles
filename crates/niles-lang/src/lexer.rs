//! The Niles lexer.
//!
//! Two layers, following rustc's split of a dependency-free `rustc_lexer` producing raw
//! tokens from a `StringReader` that cooks them. The raw layer here ([`raw`]) knows
//! nothing about keywords, editions or the registry; it turns text into a lossless stream
//! in which every byte of the input belongs to exactly one token, trivia included. The
//! cooked layer ([`Lexer`]) consults [`crate::keywords`] to promote identifiers to keywords
//! and drops trivia for the parser, while keeping it addressable for a formatter.
//!
//! Losslessness is not decoration. A resilient parser needs to re-emit the exact input for
//! a formatter and an IDE, and a thesis artifact needs a pretty-print round-trip test to be
//! meaningful, which requires that no byte be discarded at the lexical layer.
//!
//! # The literals that are not in SQL or Rust
//!
//! * **Money** — `10.00 usd`, `1_250.75 eur`, `100 jpy`. A decimal immediately followed by
//!   a currency identifier. The scale is *counted from the literal* and checked against the
//!   currency's declared scale at type-check time, so `10.001 usd` is a compile error
//!   rather than a silent rounding.
//! * **Epoch** — `#4200`. Disambiguated from the attribute prefix `#[` by one character of
//!   lookahead.
//! * **Instant / date** — `@2026-03-01`, `@2026-03-01T12:00:00Z` on the system axis;
//!   `v@2026-03-01` on the valid-time axis. Two prefixes because bitemporality has two
//!   axes and a single spelling would make the axis invisible at the point of use.
//! * **Duration** — `7.days`, `24.hours`, `200.millis`. A number, a dot, and a time unit.
//!   This collides with field access on a numeric literal, which Niles simply does not
//!   have, so the collision is free.

use crate::keywords::{self, Kw};
use std::fmt;

/// A half-open byte range into the source. Every token, node and diagnostic carries one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span {
            start: start as u32,
            end: end as u32,
        }
    }
    pub fn to(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
    pub fn len(self) -> usize {
        (self.end - self.start) as usize
    }
    pub fn is_empty(self) -> bool {
        self.end == self.start
    }
    pub fn text(self, src: &str) -> &str {
        &src[self.start as usize..self.end as usize]
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// The cooked token kinds the parser sees.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// A keyword, already resolved through the registry.
    Kw(Kw),
    /// An identifier. Raw identifiers (`r#ledger`) arrive here with the escape stripped.
    Ident,
    /// Integer literal, value already parsed (underscores removed).
    Int(i128),
    /// Floating literal. Never used for money.
    Float(f64),
    /// `10.00 usd` — value in minor units at the literal's own scale, plus that scale and
    /// the currency word. The type checker reconciles the scale with the declaration.
    Money {
        minor: i128,
        scale: u32,
        currency: String,
    },
    /// `#4200`
    EpochLit(u64),
    /// `@2026-03-01`, system axis.
    Instant(String),
    /// `v@2026-03-01`, valid-time axis.
    ValidInstant(String),
    /// `7.days`
    Duration {
        value: i128,
        unit: TimeUnit,
    },
    /// A string literal with escapes already processed.
    Str(String),
    /// A byte-string literal.
    Bytes(Vec<u8>),
    Bool(bool),
    // --- punctuation ---
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    ColonColon,
    Dot,
    DotDot,
    Arrow,
    FatArrow,
    Pipe,
    PipePipe,
    PipeGt,
    Amp,
    AmpAmp,
    Bang,
    Question,
    At,
    Hash,
    HashBracket,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Underscore,
    /// End of input. Always present, so the parser never indexes past the end.
    Eof,
    /// A byte the lexer could not classify. Never silently dropped: it becomes a token so
    /// the parser can report it in position and keep going.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Millis,
    Seconds,
    Minutes,
    Hours,
    Days,
    Epochs,
}

impl TimeUnit {
    fn parse(s: &str) -> Option<TimeUnit> {
        Some(match s {
            "millis" | "ms" => TimeUnit::Millis,
            "seconds" | "secs" | "s" => TimeUnit::Seconds,
            "minutes" | "mins" => TimeUnit::Minutes,
            "hours" | "hrs" => TimeUnit::Hours,
            "days" => TimeUnit::Days,
            "epochs" => TimeUnit::Epochs,
            _ => return None,
        })
    }
    /// In milliseconds, except `Epochs`, which is not a wall-clock unit at all and is
    /// therefore kept separate rather than converted with an assumed epoch quantum.
    pub fn millis(self) -> Option<i128> {
        Some(match self {
            TimeUnit::Millis => 1,
            TimeUnit::Seconds => 1_000,
            TimeUnit::Minutes => 60_000,
            TimeUnit::Hours => 3_600_000,
            TimeUnit::Days => 86_400_000,
            TimeUnit::Epochs => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

impl Token {
    pub fn is(&self, t: &Tok) -> bool {
        &self.tok == t
    }
    pub fn is_kw(&self, k: Kw) -> bool {
        self.tok == Tok::Kw(k)
    }
}

/// Trivia: whitespace and comments. Retained so that the token stream is lossless and a
/// formatter can be written against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trivia {
    Whitespace,
    LineComment,
    BlockComment,
    /// `///` — attaches to the following item and reaches `docs/`.
    DocComment,
}

/// A lexical error. Errors are values, not panics: the lexer always produces a full token
/// stream, because a parser that stops at the first bad byte cannot serve an editor.
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub span: Span,
    pub msg: String,
    pub code: &'static str,
}

pub mod raw {
    //! The dependency-free layer. Knows characters, not keywords.
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub enum RawKind {
        Trivia(Trivia),
        Ident,
        RawIdent,
        Number,
        Str,
        ByteStr,
        Punct,
        Unknown,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct RawToken {
        pub kind: RawKind,
        pub span: Span,
    }

    pub fn is_ident_start(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }
    pub fn is_ident_continue(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}

/// The cooked lexer.
pub struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    /// Trivia spans, in source order, kept for the formatter and for doc extraction.
    pub trivia: Vec<(Trivia, Span)>,
    pub errors: Vec<LexError>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            trivia: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Lex the whole input. Always returns a stream ending in [`Tok::Eof`]; errors are
    /// accumulated in [`Lexer::errors`] rather than short-circuiting.
    pub fn tokenize(mut self) -> (Vec<Token>, Vec<(Trivia, Span)>, Vec<LexError>) {
        let mut out = Vec::new();
        loop {
            self.skip_trivia();
            let start = self.pos;
            if self.pos >= self.bytes.len() {
                out.push(Token {
                    tok: Tok::Eof,
                    span: Span::new(start, start),
                });
                break;
            }
            let tok = self.next_token();
            out.push(Token {
                tok,
                span: Span::new(start, self.pos),
            });
        }
        (out, self.trivia, self.errors)
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }
    fn peek_at(&self, n: usize) -> Option<char> {
        self.src[self.pos..].chars().nth(n)
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += c.len_utf8();
            true
        } else {
            false
        }
    }
    fn err(&mut self, start: usize, code: &'static str, msg: impl Into<String>) {
        self.errors.push(LexError {
            span: Span::new(start, self.pos),
            msg: msg.into(),
            code,
        });
    }

    fn skip_trivia(&mut self) {
        loop {
            let start = self.pos;
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    while self.peek().map_or(false, |c| c.is_whitespace()) {
                        self.bump();
                    }
                    self.trivia
                        .push((Trivia::Whitespace, Span::new(start, self.pos)));
                }
                Some('-') if self.peek_at(1) == Some('-') => {
                    // SQL line comment. Kept because Niles must accept pasted SQL.
                    while self.peek().map_or(false, |c| c != '\n') {
                        self.bump();
                    }
                    self.trivia
                        .push((Trivia::LineComment, Span::new(start, self.pos)));
                }
                Some('/') if self.peek_at(1) == Some('/') => {
                    let doc = self.peek_at(2) == Some('/');
                    while self.peek().map_or(false, |c| c != '\n') {
                        self.bump();
                    }
                    let kind = if doc {
                        Trivia::DocComment
                    } else {
                        Trivia::LineComment
                    };
                    self.trivia.push((kind, Span::new(start, self.pos)));
                }
                Some('/') if self.peek_at(1) == Some('*') => {
                    self.bump();
                    self.bump();
                    // Nested, as in Rust and unlike C: a commented-out block containing a
                    // block comment must stay commented out.
                    let mut depth = 1usize;
                    while depth > 0 {
                        match self.bump() {
                            None => {
                                self.err(start, "NL0101", "unterminated block comment");
                                break;
                            }
                            Some('*') if self.peek() == Some('/') => {
                                self.bump();
                                depth -= 1;
                            }
                            Some('/') if self.peek() == Some('*') => {
                                self.bump();
                                depth += 1;
                            }
                            _ => {}
                        }
                    }
                    self.trivia
                        .push((Trivia::BlockComment, Span::new(start, self.pos)));
                }
                _ => return,
            }
        }
    }

    fn next_token(&mut self) -> Tok {
        let start = self.pos;
        let c = self.peek().unwrap();

        // --- valid-time instant: v@... , before the identifier rule claims the `v` ---
        if c == 'v'
            && self.peek_at(1) == Some('@')
            && self.peek_at(2).map_or(false, |c| c.is_ascii_digit())
        {
            self.bump();
            self.bump();
            let s = self.lex_datetime_body();
            return Tok::ValidInstant(s);
        }
        // --- raw identifier: r#ledger ---
        if c == 'r'
            && self.peek_at(1) == Some('#')
            && self.peek_at(2).map_or(false, raw::is_ident_start)
        {
            self.bump();
            self.bump();
            let b = self.pos;
            while self.peek().map_or(false, raw::is_ident_continue) {
                self.bump();
            }
            let _ = &self.src[b..self.pos];
            return Tok::Ident;
        }
        // --- byte string: b"..." ---
        if c == 'b' && self.peek_at(1) == Some('"') {
            self.bump();
            let s = self.lex_string_body();
            return Tok::Bytes(s.into_bytes());
        }
        if raw::is_ident_start(c) {
            return self.lex_ident_or_keyword();
        }
        if c.is_ascii_digit() {
            return self.lex_number();
        }
        match c {
            '"' => {
                let s = self.lex_string_body();
                Tok::Str(s)
            }
            '@' => {
                self.bump();
                // An ISO-8601 timestamp always begins with a digit, so `@2026-03-01` is a
                // literal and `@confidential` / `read@snapshot` are the attribute and the
                // rung-qualifier forms. One character of lookahead separates them, which
                // is why the two spellings can share the sigil.
                if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    Tok::Instant(self.lex_datetime_body())
                } else {
                    Tok::At
                }
            }
            '#' => {
                self.bump();
                if self.peek() == Some('[') {
                    self.bump();
                    return Tok::HashBracket;
                }
                if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    let b = self.pos;
                    while self
                        .peek()
                        .map_or(false, |c| c.is_ascii_digit() || c == '_')
                    {
                        self.bump();
                    }
                    let raw_digits: String = self.src[b..self.pos]
                        .chars()
                        .filter(|c| *c != '_')
                        .collect();
                    return match raw_digits.parse::<u64>() {
                        Ok(v) => Tok::EpochLit(v),
                        Err(_) => {
                            self.err(start, "NL0102", "epoch literal does not fit in u64");
                            Tok::EpochLit(0)
                        }
                    };
                }
                Tok::Hash
            }
            _ => self.lex_punct(),
        }
    }

    fn lex_ident_or_keyword(&mut self) -> Tok {
        let b = self.pos;
        while self.peek().map_or(false, raw::is_ident_continue) {
            self.bump();
        }
        let word = &self.src[b..self.pos];
        if word == "_" {
            return Tok::Underscore;
        }
        match keywords::lookup(word) {
            Some(k) => match k.token {
                Kw::True => Tok::Bool(true),
                Kw::False => Tok::Bool(false),
                t => Tok::Kw(t),
            },
            None => Tok::Ident,
        }
    }

    /// Numbers, and the three literal forms that begin as numbers: money, duration, and
    /// plain integers/floats. The disambiguation is entirely local, which is why no
    /// backtracking is needed here.
    fn lex_number(&mut self) -> Tok {
        let start = self.pos;
        let int_b = self.pos;
        while self
            .peek()
            .map_or(false, |c| c.is_ascii_digit() || c == '_')
        {
            self.bump();
        }
        let int_part: String = self.src[int_b..self.pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();

        // `7.days` — a number, a dot, a time unit. Checked before the fraction rule,
        // because `7.days` must not lex as `7.` followed by `days`.
        if self.peek() == Some('.') && self.peek_at(1).map_or(false, |c| c.is_alphabetic()) {
            let save = self.pos;
            self.bump();
            let ub = self.pos;
            while self.peek().map_or(false, raw::is_ident_continue) {
                self.bump();
            }
            let unit_word = &self.src[ub..self.pos];
            if let Some(unit) = TimeUnit::parse(unit_word) {
                let value = int_part.parse::<i128>().unwrap_or(0);
                return Tok::Duration { value, unit };
            }
            self.pos = save;
        }

        let mut frac = String::new();
        if self.peek() == Some('.') && self.peek_at(1).map_or(false, |c| c.is_ascii_digit()) {
            self.bump();
            let fb = self.pos;
            while self
                .peek()
                .map_or(false, |c| c.is_ascii_digit() || c == '_')
            {
                self.bump();
            }
            frac = self.src[fb..self.pos]
                .chars()
                .filter(|c| *c != '_')
                .collect();
        }

        // Money: a decimal immediately followed (after at most one space) by a currency
        // word. One space is allowed because `10.00 usd` reads better than `10.00usd`, and
        // the currency namespace is closed, so the ambiguity with an ordinary identifier is
        // resolved at name resolution rather than here: the lexer emits Money only when the
        // following word is not a keyword and the number carried a fraction or the word is
        // a known-shaped currency code.
        let save = self.pos;
        let mut spaced = false;
        if self.peek() == Some(' ') {
            self.bump();
            spaced = true;
        }
        if self.peek().map_or(false, raw::is_ident_start) {
            let cb = self.pos;
            while self.peek().map_or(false, raw::is_ident_continue) {
                self.bump();
            }
            let word = &self.src[cb..self.pos];
            let looks_like_currency = word.len() == 3
                && word.bytes().all(|b| b.is_ascii_lowercase())
                && keywords::lookup(word).is_none();
            if looks_like_currency {
                let scale = frac.len() as u32;
                let combined = format!("{int_part}{frac}");
                return match combined.parse::<i128>() {
                    Ok(minor) => Tok::Money {
                        minor,
                        scale,
                        currency: word.to_string(),
                    },
                    Err(_) => {
                        self.err(
                            start,
                            "NL0103",
                            "money literal does not fit in i128 minor units",
                        );
                        Tok::Money {
                            minor: 0,
                            scale,
                            currency: word.to_string(),
                        }
                    }
                };
            }
            self.pos = save;
        } else if spaced {
            self.pos = save;
        }

        if frac.is_empty() {
            match int_part.parse::<i128>() {
                Ok(v) => Tok::Int(v),
                Err(_) => {
                    self.err(start, "NL0104", "integer literal out of range for i128");
                    Tok::Int(0)
                }
            }
        } else {
            let f: f64 = format!("{int_part}.{frac}").parse().unwrap_or(0.0);
            Tok::Float(f)
        }
    }

    fn lex_string_body(&mut self) -> String {
        let start = self.pos;
        self.bump(); // opening quote
        let mut out = String::new();
        loop {
            match self.bump() {
                None => {
                    self.err(start, "NL0105", "unterminated string literal");
                    break;
                }
                Some('"') => break,
                Some('\\') => match self.bump() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('0') => out.push('\0'),
                    Some('\\') => out.push('\\'),
                    Some('"') => out.push('"'),
                    Some(c) => {
                        self.err(start, "NL0106", format!("unknown escape `\\{c}`"));
                        out.push(c);
                    }
                    None => {
                        self.err(start, "NL0105", "unterminated string literal");
                        break;
                    }
                },
                Some(c) => out.push(c),
            }
        }
        out
    }

    /// The body of an `@` or `v@` literal: ISO-8601-shaped text, validated at type-check
    /// time rather than here. The lexer's job is to delimit it, not to know the calendar.
    fn lex_datetime_body(&mut self) -> String {
        let b = self.pos;
        while self.peek().map_or(false, |c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | ':' | 'T' | 'Z' | '+' | '.')
        }) {
            self.bump();
        }
        if self.pos == b {
            self.err(b, "NL0107", "expected an ISO-8601 timestamp after `@`");
        }
        self.src[b..self.pos].to_string()
    }

    fn lex_punct(&mut self) -> Tok {
        let start = self.pos;
        let c = self.bump().unwrap();
        match c {
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            '{' => Tok::LBrace,
            '}' => Tok::RBrace,
            '[' => Tok::LBracket,
            ']' => Tok::RBracket,
            ',' => Tok::Comma,
            ';' => Tok::Semi,
            ':' => {
                if self.eat(':') {
                    Tok::ColonColon
                } else {
                    Tok::Colon
                }
            }
            '.' => {
                if self.eat('.') {
                    Tok::DotDot
                } else {
                    Tok::Dot
                }
            }
            '-' => {
                if self.eat('>') {
                    Tok::Arrow
                } else {
                    Tok::Minus
                }
            }
            '=' => {
                if self.eat('=') {
                    Tok::EqEq
                } else if self.eat('>') {
                    Tok::FatArrow
                } else {
                    Tok::Eq
                }
            }
            '|' => {
                if self.eat('|') {
                    Tok::PipePipe
                } else if self.eat('>') {
                    Tok::PipeGt
                } else {
                    Tok::Pipe
                }
            }
            '&' => {
                if self.eat('&') {
                    Tok::AmpAmp
                } else {
                    Tok::Amp
                }
            }
            '!' => {
                if self.eat('=') {
                    Tok::Ne
                } else {
                    Tok::Bang
                }
            }
            '<' => {
                if self.eat('=') {
                    Tok::Le
                } else {
                    Tok::Lt
                }
            }
            '>' => {
                if self.eat('=') {
                    Tok::Ge
                } else {
                    Tok::Gt
                }
            }
            '?' => Tok::Question,
            '+' => Tok::Plus,
            '*' => Tok::Star,
            '/' => Tok::Slash,
            '%' => Tok::Percent,
            '^' => Tok::Caret,
            _ => {
                self.err(start, "NL0100", format!("unexpected character `{c}`"));
                Tok::Unknown
            }
        }
    }
}

/// Convenience: lex and discard trivia.
pub fn lex(src: &str) -> (Vec<Token>, Vec<LexError>) {
    let (toks, _triv, errs) = Lexer::new(src).tokenize();
    (toks, errs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<Tok> {
        lex(src).0.into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn stream_is_lossless() {
        // Every byte belongs to exactly one token or one trivium. This is the property a
        // formatter and a pretty-print round-trip test depend on.
        let src = "schema s {\n  // note\n  table t { id: i64 }\n}\n";
        let (toks, trivia, errs) = Lexer::new(src).tokenize();
        assert!(errs.is_empty());
        let mut covered = vec![false; src.len()];
        for t in &toks {
            for i in t.span.start as usize..t.span.end as usize {
                assert!(!covered[i], "byte {i} covered twice");
                covered[i] = true;
            }
        }
        for (_, s) in &trivia {
            for i in s.start as usize..s.end as usize {
                assert!(!covered[i], "byte {i} covered twice");
                covered[i] = true;
            }
        }
        assert!(
            covered.iter().all(|b| *b),
            "some byte was dropped by the lexer"
        );
    }

    #[test]
    fn money_literal_carries_its_own_scale() {
        assert_eq!(
            kinds("10.00 usd")[0],
            Tok::Money {
                minor: 1000,
                scale: 2,
                currency: "usd".into()
            }
        );
        assert_eq!(
            kinds("100 jpy")[0],
            Tok::Money {
                minor: 100,
                scale: 0,
                currency: "jpy".into()
            }
        );
        assert_eq!(
            kinds("1_250.750 bhd")[0],
            Tok::Money {
                minor: 1250750,
                scale: 3,
                currency: "bhd".into()
            }
        );
    }

    #[test]
    fn a_number_before_an_ordinary_word_is_not_money() {
        // `3 accounts` must not become money; only a three-letter lowercase non-keyword
        // word can close a money literal, and `accounts` is not one.
        assert_eq!(kinds("3 accounts")[0], Tok::Int(3));
        assert_eq!(kinds("3 accounts")[1], Tok::Ident);
    }

    #[test]
    fn epoch_and_attribute_do_not_collide() {
        assert_eq!(kinds("#4200")[0], Tok::EpochLit(4200));
        assert_eq!(kinds("#[udf]")[0], Tok::HashBracket);
    }

    #[test]
    fn both_time_axes_are_lexically_distinct() {
        assert_eq!(kinds("@2026-03-01")[0], Tok::Instant("2026-03-01".into()));
        assert_eq!(
            kinds("v@2026-03-01")[0],
            Tok::ValidInstant("2026-03-01".into())
        );
    }

    #[test]
    fn durations_beat_field_access() {
        assert_eq!(
            kinds("7.days")[0],
            Tok::Duration {
                value: 7,
                unit: TimeUnit::Days
            }
        );
        assert_eq!(
            kinds("200.millis")[0],
            Tok::Duration {
                value: 200,
                unit: TimeUnit::Millis
            }
        );
        // ... but a real fraction still lexes as a float.
        assert_eq!(kinds("7.5")[0], Tok::Float(7.5));
    }

    #[test]
    fn keywords_come_from_the_registry() {
        assert_eq!(kinds("ledger")[0], Tok::Kw(Kw::Ledger));
        assert_eq!(kinds("LEDGER")[0], Tok::Kw(Kw::Ledger));
        assert_eq!(kinds("balance")[0], Tok::Ident);
    }

    #[test]
    fn raw_identifiers_escape_reserved_words() {
        assert_eq!(kinds("r#select")[0], Tok::Ident);
        assert_eq!(kinds("select")[0], Tok::Kw(Kw::Select));
    }

    #[test]
    fn sql_and_rust_comments_are_both_trivia() {
        let (toks, trivia, errs) = Lexer::new("-- sql\n// rust\n/* /* nested */ */ x").tokenize();
        assert!(errs.is_empty());
        assert_eq!(toks[0].tok, Tok::Ident);
        assert_eq!(
            trivia
                .iter()
                .filter(|(t, _)| *t != Trivia::Whitespace)
                .count(),
            3
        );
    }

    #[test]
    fn errors_do_not_stop_the_stream() {
        let (toks, errs) = lex("let x = `;");
        assert!(!errs.is_empty());
        assert_eq!(
            toks.last().unwrap().tok,
            Tok::Eof,
            "lexing must always reach EOF"
        );
    }

    #[test]
    fn pipeline_and_closure_pipes_are_distinguished() {
        assert_eq!(kinds("|> ||")[0], Tok::PipeGt);
        assert_eq!(kinds("|> ||")[1], Tok::PipePipe);
        assert_eq!(kinds("|r|")[0], Tok::Pipe);
    }
}
