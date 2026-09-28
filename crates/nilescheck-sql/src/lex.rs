//! **PostgreSQL 16's lexical structure, by hand** (PostgreSQL documentation §4.1).
//!
//! The author chose (2026-09-28) a full, hand-written parser with no external dependency. The
//! lexer is the part of that choice with the least room for interpretation: PostgreSQL's
//! `scan.l` defines the tokens, and every rule below cites the paragraph of §4.1 it
//! implements. Where this lexer is stricter or looser than PostgreSQL, the difference is
//! named at the rule, and the test suite holds the texts PostgreSQL itself accepts.
//!
//! What it produces: tokens with byte spans, identifiers already case-folded (an unquoted
//! identifier is folded to lower case; a quoted one is kept exactly, §4.1.1), string constants
//! already decoded (escape strings, Unicode escapes, dollar quoting, §4.1.2), and comments kept
//! as tokens of their own — the checker's annotations (`-- @linear hold`, R2-05) live in
//! comments, so a lexer that dropped them would drop the program's contract.

/// A half-open byte range into the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// An identifier or keyword, case-folded when unquoted. Keywords are not distinguished
    /// here: PostgreSQL's keywords are context-dependent (most are unreserved), so the parser
    /// decides, as `gram.y` does.
    Ident(String),
    /// A double-quoted identifier, exactly as written after `""` → `"`. Never a keyword.
    QuotedIdent(String),
    /// A string constant, decoded: `'…'`, `E'…'`, `U&'…'`, `$tag$…$tag$`, and the
    /// concatenation of adjacent constants separated by a newline (§4.1.2.1).
    Str(String),
    /// `B'…'` — a bit-string constant, the digits as written.
    BitStr(String),
    /// `X'…'` — a hexadecimal bit-string constant, the digits as written.
    HexStr(String),
    /// An integer constant, as written (underscores removed, radix prefix kept), with its
    /// value when it fits in an `i128`.
    Int(String),
    /// A numeric constant with a point or an exponent, as written.
    Num(String),
    /// `$n` — a positional parameter.
    Param(u32),
    /// An operator (§4.1.3), after the trailing `+`/`-` rule.
    Op(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    /// `::`
    Cast,
    Dot,
    /// `:=` (PL/pgSQL assignment; also accepted by SQL in named-argument calls).
    Assign,
    /// `=>` named-argument notation.
    Arrow,
    /// `..` (PL/pgSQL integer `FOR` range).
    DotDot,
    /// A comment, text included: `--` to end of line, or `/* … */`, which nests.
    Comment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub span: Span,
    pub msg: String,
}

/// §4.1.3: the characters an operator is made of.
fn is_op_char(c: char) -> bool {
    matches!(
        c,
        '+' | '-'
            | '*'
            | '/'
            | '<'
            | '>'
            | '='
            | '~'
            | '!'
            | '@'
            | '#'
            | '%'
            | '^'
            | '&'
            | '|'
            | '`'
            | '?'
    )
}

/// §4.1.1: an identifier starts with a letter (including non-ASCII letters) or underscore
/// and continues with letters, underscores, digits or dollar signs.
fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || (c as u32) >= 0x80
}
fn is_ident_cont(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit() || c == '$'
}

pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    /// `standard_conforming_strings`: on by default since PostgreSQL 9.1, so `\` in a plain
    /// `'…'` is an ordinary character. Kept as a field so a dump with it off can be lexed.
    pub standard_conforming_strings: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Lexer<'a> {
        Lexer {
            src,
            pos: 0,
            standard_conforming_strings: true,
        }
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
    fn starts(&self, s: &str) -> bool {
        self.src[self.pos..].starts_with(s)
    }
    fn err(&self, start: usize, msg: impl Into<String>) -> LexError {
        LexError {
            span: Span {
                start,
                end: self.pos.max(start + 1).min(self.src.len().max(start)),
            },
            msg: msg.into(),
        }
    }

    /// Every token, comments included; whitespace dropped.
    pub fn tokens(mut self) -> Result<Vec<Token>, LexError> {
        let mut out = Vec::new();
        while let Some(t) = self.next_token()? {
            out.push(t);
        }
        Ok(out)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_ws();
        let start = self.pos;
        let Some(c) = self.peek() else {
            return Ok(None);
        };
        let tok = match c {
            '-' if self.peek_at(1) == Some('-') => {
                let rest = &self.src[self.pos..];
                let len = rest.find('\n').unwrap_or(rest.len());
                self.pos += len;
                Tok::Comment(self.src[start..self.pos].to_string())
            }
            '/' if self.peek_at(1) == Some('*') => {
                // §4.1.5: block comments nest, as the SQL standard specifies and C does not.
                self.pos += 2;
                let mut depth = 1;
                while depth > 0 {
                    if self.starts("/*") {
                        depth += 1;
                        self.pos += 2;
                    } else if self.starts("*/") {
                        depth -= 1;
                        self.pos += 2;
                    } else if self.bump().is_none() {
                        return Err(self.err(start, "unterminated /* comment"));
                    }
                }
                Tok::Comment(self.src[start..self.pos].to_string())
            }
            '\'' => Tok::Str(self.quoted_string(start, !self.standard_conforming_strings)?),
            'e' | 'E' if self.peek_at(1) == Some('\'') => {
                self.bump();
                Tok::Str(self.quoted_string(start, true)?)
            }
            'b' | 'B' if self.peek_at(1) == Some('\'') => {
                self.bump();
                Tok::BitStr(self.quoted_string(start, false)?)
            }
            'x' | 'X' if self.peek_at(1) == Some('\'') => {
                self.bump();
                Tok::HexStr(self.quoted_string(start, false)?)
            }
            'n' | 'N' if self.peek_at(1) == Some('\'') => {
                // National character: PostgreSQL treats N'…' as an ordinary string.
                self.bump();
                Tok::Str(self.quoted_string(start, false)?)
            }
            'u' | 'U' if self.peek_at(1) == Some('&') && self.peek_at(2) == Some('\'') => {
                self.pos += 2;
                let raw = self.quoted_string(start, false)?;
                let esc = self.uescape()?;
                Tok::Str(unicode_unescape(&raw, esc).map_err(|m| self.err(start, m))?)
            }
            'u' | 'U' if self.peek_at(1) == Some('&') && self.peek_at(2) == Some('"') => {
                self.pos += 2;
                let raw = self.quoted_ident(start)?;
                let esc = self.uescape()?;
                Tok::QuotedIdent(unicode_unescape(&raw, esc).map_err(|m| self.err(start, m))?)
            }
            '"' => Tok::QuotedIdent(self.quoted_ident(start)?),
            '$' => {
                if self.peek_at(1).is_some_and(|d| d.is_ascii_digit()) {
                    self.bump();
                    let d0 = self.pos;
                    while self.peek().is_some_and(|d| d.is_ascii_digit()) {
                        self.bump();
                    }
                    let n: u32 = self.src[d0..self.pos]
                        .parse()
                        .map_err(|_| self.err(start, "parameter number out of range"))?;
                    Tok::Param(n)
                } else if let Some(s) = self.dollar_quoted(start)? {
                    Tok::Str(s)
                } else {
                    return Err(self.err(
                        start,
                        "a `$` that begins neither a parameter nor a dollar-quoted string",
                    ));
                }
            }
            c if c.is_ascii_digit()
                || (c == '.' && self.peek_at(1).is_some_and(|d| d.is_ascii_digit())) =>
            {
                self.number(start)?
            }
            c if is_ident_start(c) => {
                while self.peek().is_some_and(is_ident_cont) {
                    self.bump();
                }
                // §4.1.1: unquoted names are folded to lower case. PostgreSQL folds ASCII only
                // in a multibyte encoding; `to_ascii_lowercase` matches that.
                Tok::Ident(self.src[start..self.pos].to_ascii_lowercase())
            }
            '(' => self.one(Tok::LParen),
            ')' => self.one(Tok::RParen),
            '[' => self.one(Tok::LBracket),
            ']' => self.one(Tok::RBracket),
            ',' => self.one(Tok::Comma),
            ';' => self.one(Tok::Semi),
            ':' if self.peek_at(1) == Some(':') => {
                self.pos += 2;
                Tok::Cast
            }
            ':' if self.peek_at(1) == Some('=') => {
                self.pos += 2;
                Tok::Assign
            }
            ':' => self.one(Tok::Colon),
            '.' if self.peek_at(1) == Some('.') => {
                self.pos += 2;
                Tok::DotDot
            }
            '.' => self.one(Tok::Dot),
            '=' if self.peek_at(1) == Some('>') => {
                self.pos += 2;
                Tok::Arrow
            }
            c if is_op_char(c) => self.operator(),
            other => return Err(self.err(start, format!("unexpected character {other:?}"))),
        };
        Ok(Some(Token {
            tok,
            span: Span {
                start,
                end: self.pos,
            },
        }))
    }

    fn one(&mut self, t: Tok) -> Tok {
        self.bump();
        t
    }

    /// §4.1.3's two rules for where an operator ends: `--` and `/*` cannot appear inside one
    /// (they begin a comment), and a multi-character operator cannot end in `+` or `-` unless
    /// it also contains at least one of `~ ! @ # % ^ & | \` ?` — so `X*-Y` is `X * -Y`.
    fn operator(&mut self) -> Tok {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if !is_op_char(c) || self.starts("--") || self.starts("/*") {
                break;
            }
            self.bump();
        }
        let mut text = &self.src[start..self.pos];
        if text.len() > 1 {
            let special = text.chars().any(|c| "~!@#%^&|`?".contains(c));
            if !special {
                while text.len() > 1 && (text.ends_with('+') || text.ends_with('-')) {
                    text = &text[..text.len() - 1];
                }
                self.pos = start + text.len();
            }
        }
        Tok::Op(text.to_string())
    }

    /// §4.1.2.1–2: `'…'` with `''` for a quote; with `escapes`, the C-like backslash escapes
    /// of an `E'…'` string. A constant followed by whitespace containing a newline and then
    /// another `'…'` continues it — the standard's rule, which PostgreSQL implements.
    fn quoted_string(&mut self, start: usize, escapes: bool) -> Result<String, LexError> {
        let mut out = String::new();
        loop {
            if self.bump() != Some('\'') {
                return Err(self.err(start, "expected '"));
            }
            loop {
                match self.bump() {
                    None => return Err(self.err(start, "unterminated quoted string")),
                    Some('\'') if self.peek() == Some('\'') => {
                        self.bump();
                        out.push('\'');
                    }
                    Some('\'') => break,
                    Some('\\') if escapes => out.push(self.backslash_escape(start)?),
                    Some(c) => out.push(c),
                }
            }
            // Continuation: whitespace including at least one newline, then a quote.
            let save = self.pos;
            let mut saw_newline = false;
            while let Some(c) = self.peek() {
                if c == '\n' {
                    saw_newline = true;
                }
                if c.is_whitespace() {
                    self.bump();
                } else {
                    break;
                }
            }
            if saw_newline && self.peek() == Some('\'') {
                continue;
            }
            self.pos = save;
            return Ok(out);
        }
    }

    fn backslash_escape(&mut self, start: usize) -> Result<char, LexError> {
        let c = self
            .bump()
            .ok_or_else(|| self.err(start, "unterminated escape"))?;
        Ok(match c {
            'b' => '\u{8}',
            'f' => '\u{c}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0'..='7' => {
                let mut v = c.to_digit(8).unwrap();
                for _ in 0..2 {
                    match self.peek().and_then(|d| d.to_digit(8)) {
                        Some(d) => {
                            v = v * 8 + d;
                            self.bump();
                        }
                        None => break,
                    }
                }
                char::from_u32(v).ok_or_else(|| self.err(start, "invalid octal escape"))?
            }
            'x' => {
                let mut v = 0;
                let mut n = 0;
                while n < 2 {
                    match self.peek().and_then(|d| d.to_digit(16)) {
                        Some(d) => {
                            v = v * 16 + d;
                            self.bump();
                            n += 1;
                        }
                        None => break,
                    }
                }
                if n == 0 {
                    return Err(self.err(start, "\\x with no hex digits"));
                }
                char::from_u32(v).ok_or_else(|| self.err(start, "invalid hex escape"))?
            }
            'u' | 'U' => {
                let want = if c == 'u' { 4 } else { 8 };
                let mut v = 0u32;
                for _ in 0..want {
                    let d = self
                        .bump()
                        .and_then(|d| d.to_digit(16))
                        .ok_or_else(|| self.err(start, "invalid Unicode escape"))?;
                    v = v * 16 + d;
                }
                char::from_u32(v).ok_or_else(|| self.err(start, "invalid Unicode code point"))?
            }
            other => other,
        })
    }

    fn quoted_ident(&mut self, start: usize) -> Result<String, LexError> {
        self.bump();
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Err(self.err(start, "unterminated quoted identifier")),
                Some('"') if self.peek() == Some('"') => {
                    self.bump();
                    out.push('"');
                }
                Some('"') => break,
                Some(c) => out.push(c),
            }
        }
        if out.is_empty() {
            return Err(self.err(start, "zero-length delimited identifier"));
        }
        Ok(out)
    }

    /// An optional `UESCAPE 'c'` after a `U&` constant; the default escape is `\`.
    fn uescape(&mut self) -> Result<char, LexError> {
        let save = self.pos;
        self.skip_ws();
        let rest = &self.src[self.pos..];
        if rest.len() >= 7 && rest[..7].eq_ignore_ascii_case("uescape") {
            self.pos += 7;
            self.skip_ws();
            let s0 = self.pos;
            let s = self.quoted_string(s0, false)?;
            let mut it = s.chars();
            let (Some(c), None) = (it.next(), it.next()) else {
                return Err(self.err(s0, "UESCAPE needs a single character"));
            };
            if c.is_ascii_hexdigit() || "+'\" ".contains(c) || c.is_whitespace() {
                return Err(self.err(s0, "invalid UESCAPE character"));
            }
            return Ok(c);
        }
        self.pos = save;
        Ok('\\')
    }

    /// §4.1.2.4: `$tag$ … $tag$`, the tag empty or an identifier without `$`.
    fn dollar_quoted(&mut self, start: usize) -> Result<Option<String>, LexError> {
        let rest = &self.src[self.pos + 1..];
        let tag_len = rest
            .char_indices()
            .take_while(|&(i, c)| {
                if i == 0 {
                    c == '_' || c.is_alphabetic() || (c as u32) >= 0x80
                } else {
                    c == '_' || c.is_alphanumeric() || (c as u32) >= 0x80
                }
            })
            .map(|(i, c)| i + c.len_utf8())
            .last()
            .unwrap_or(0);
        if !rest[tag_len..].starts_with('$') {
            return Ok(None);
        }
        let delim = format!("${}$", &rest[..tag_len]);
        self.pos += delim.len();
        let body_start = self.pos;
        match self.src[body_start..].find(&delim) {
            Some(i) => {
                self.pos = body_start + i + delim.len();
                Ok(Some(self.src[body_start..body_start + i].to_string()))
            }
            None => {
                self.pos = self.src.len();
                Err(self.err(start, format!("unterminated dollar-quoted string {delim}")))
            }
        }
    }

    /// §4.1.2.6: integers (decimal, `0x`, `0o`, `0b`), decimals and exponents, `_` between
    /// digits (PostgreSQL 16). A letter immediately after a number is an error in 16 (it used
    /// to lex as a separate identifier: `123abc` was `123 AS abc`).
    fn number(&mut self, start: usize) -> Result<Tok, LexError> {
        let radix = if self.peek() == Some('0') {
            match self.peek_at(1) {
                Some('x' | 'X') => 16,
                Some('o' | 'O') => 8,
                Some('b' | 'B') => 2,
                _ => 10,
            }
        } else {
            10
        };
        if radix != 10 {
            self.pos += 2;
            let d0 = self.pos;
            while self.peek().is_some_and(|c| {
                c.is_digit(radix)
                    || (c == '_' && self.peek_at(1).is_some_and(|d| d.is_digit(radix)))
            }) {
                self.bump();
            }
            if self.pos == d0 {
                return Err(self.err(start, "invalid radix literal"));
            }
            self.junk_after_number(start)?;
            return Ok(Tok::Int(self.src[start..self.pos].replace('_', "")));
        }
        let digits = |l: &mut Lexer| {
            while l.peek().is_some_and(|c| {
                c.is_ascii_digit() || (c == '_' && l.peek_at(1).is_some_and(|d| d.is_ascii_digit()))
            }) {
                l.bump();
            }
        };
        digits(self);
        let mut is_num = false;
        // A `.` followed by another `.` is the `..` of a PL/pgSQL range, not a decimal point.
        if self.peek() == Some('.') && self.peek_at(1) != Some('.') {
            is_num = true;
            self.bump();
            digits(self);
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            let sign = matches!(self.peek_at(1), Some('+' | '-'));
            let d = self.peek_at(if sign { 2 } else { 1 });
            if d.is_some_and(|c| c.is_ascii_digit()) {
                is_num = true;
                self.pos += if sign { 2 } else { 1 };
                digits(self);
            }
        }
        self.junk_after_number(start)?;
        let text = self.src[start..self.pos].replace('_', "");
        Ok(if is_num {
            Tok::Num(text)
        } else {
            Tok::Int(text)
        })
    }

    fn junk_after_number(&self, start: usize) -> Result<(), LexError> {
        if self.peek().is_some_and(is_ident_start) {
            return Err(self.err(start, "trailing junk after numeric literal"));
        }
        Ok(())
    }
}

/// §4.1.2.3: `U&'d\0061t\+000061'` — `\XXXX`, `\+XXXXXX`, and the escape doubled for itself.
fn unicode_unescape(raw: &str, esc: char) -> Result<String, String> {
    let mut out = String::new();
    let mut it = raw.chars().peekable();
    while let Some(c) = it.next() {
        if c != esc {
            out.push(c);
            continue;
        }
        if it.peek() == Some(&esc) {
            it.next();
            out.push(esc);
            continue;
        }
        let want = if it.peek() == Some(&'+') {
            it.next();
            6
        } else {
            4
        };
        let mut v = 0u32;
        for _ in 0..want {
            let d = it
                .next()
                .and_then(|d| d.to_digit(16))
                .ok_or("invalid Unicode escape")?;
            v = v * 16 + d;
        }
        out.push(char::from_u32(v).ok_or("invalid Unicode code point")?);
    }
    Ok(out)
}

pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(src).tokens()
}

/// The tokens without comments, which is what the grammar reads; comments are kept beside
/// them for the annotation reader.
pub fn split_comments(toks: Vec<Token>) -> (Vec<Token>, Vec<Token>) {
    toks.into_iter()
        .partition(|t| !matches!(t.tok, Tok::Comment(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(s: &str) -> Vec<Tok> {
        lex(s).unwrap().into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn identifiers_fold_unless_quoted() {
        assert_eq!(
            kinds(r#"SELECT Foo, "Foo", "a""b", _x$1"#),
            vec![
                Tok::Ident("select".into()),
                Tok::Ident("foo".into()),
                Tok::Comma,
                Tok::QuotedIdent("Foo".into()),
                Tok::Comma,
                Tok::QuotedIdent("a\"b".into()),
                Tok::Comma,
                Tok::Ident("_x$1".into()),
            ]
        );
        assert!(lex(r#""""#).is_err(), "zero-length identifier");
    }

    #[test]
    fn strings_in_every_spelling() {
        assert_eq!(kinds("'it''s'"), vec![Tok::Str("it's".into())]);
        assert_eq!(
            kinds(r"'a\nb'"),
            vec![Tok::Str(r"a\nb".into())],
            "standard-conforming"
        );
        assert_eq!(kinds(r"E'a\nb\x41\101B'"), vec![Tok::Str("a\nbAAB".into())]);
        assert_eq!(kinds("'foo'\n  'bar'"), vec![Tok::Str("foobar".into())]);
        assert_eq!(
            kinds("'foo' 'bar'").len(),
            2,
            "no newline, no continuation (PostgreSQL then refuses it in the grammar)"
        );
        assert_eq!(kinds("$$a'b$$"), vec![Tok::Str("a'b".into())]);
        assert_eq!(
            kinds("$fn$ body $$ inner $$ $fn$"),
            vec![Tok::Str(" body $$ inner $$ ".into())]
        );
        assert_eq!(kinds(r"U&'d\0061t\+000061'"), vec![Tok::Str("data".into())]);
        assert_eq!(
            kinds(r"U&'d!0061t' UESCAPE '!'"),
            vec![Tok::Str("dat".into())]
        );
        assert_eq!(
            kinds(r#"U&"d\0061t""#),
            vec![Tok::QuotedIdent("dat".into())]
        );
        assert_eq!(
            kinds("B'1010' X'1F'"),
            vec![Tok::BitStr("1010".into()), Tok::HexStr("1F".into())]
        );
        assert!(lex("'open").is_err());
        assert!(lex("$x$ open").is_err());
    }

    #[test]
    fn numbers_including_postgresql_16_forms() {
        assert_eq!(
            kinds("42 3.5 4. .001 5e2 1.925e-3 0x1F 0o17 0b101 1_000_000"),
            vec![
                Tok::Int("42".into()),
                Tok::Num("3.5".into()),
                Tok::Num("4.".into()),
                Tok::Num(".001".into()),
                Tok::Num("5e2".into()),
                Tok::Num("1.925e-3".into()),
                Tok::Int("0x1F".into()),
                Tok::Int("0o17".into()),
                Tok::Int("0b101".into()),
                Tok::Int("1000000".into()),
            ]
        );
        assert!(
            lex("123abc").is_err(),
            "PostgreSQL 16 refuses trailing junk"
        );
        assert_eq!(
            kinds("1..10"),
            vec![Tok::Int("1".into()), Tok::DotDot, Tok::Int("10".into())]
        );
    }

    #[test]
    fn operators_end_by_the_documented_rules() {
        assert_eq!(
            kinds("x*-y"),
            vec![
                Tok::Ident("x".into()),
                Tok::Op("*".into()),
                Tok::Op("-".into()),
                Tok::Ident("y".into())
            ]
        );
        assert_eq!(
            kinds("a @- b")[1],
            Tok::Op("@-".into()),
            "a special character keeps the trailing -"
        );
        assert_eq!(kinds("a <> b")[1], Tok::Op("<>".into()));
        assert_eq!(
            kinds("a--c"),
            vec![Tok::Ident("a".into()), Tok::Comment("--c".into())]
        );
        assert_eq!(kinds("x::int")[1], Tok::Cast);
        assert_eq!(kinds("f(a => 1)")[3], Tok::Arrow);
        assert_eq!(kinds("v := 1")[1], Tok::Assign);
        assert_eq!(
            kinds("$1 + $12"),
            vec![Tok::Param(1), Tok::Op("+".into()), Tok::Param(12)]
        );
    }

    #[test]
    fn comments_nest_and_are_kept() {
        let t = kinds("/* a /* b */ c */ x -- @linear hold\ny");
        assert_eq!(t[0], Tok::Comment("/* a /* b */ c */".into()));
        assert_eq!(t[2], Tok::Comment("-- @linear hold".into()));
        assert!(lex("/* open /* nested */").is_err());
    }
}
