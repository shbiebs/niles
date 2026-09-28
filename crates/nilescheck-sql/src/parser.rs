//! **The recursive-descent parser: the token stream, keywords, and expressions.**
//!
//! Statements are in `stmt.rs`, PL/pgSQL in `plpgsql.rs`. Operator precedence is §4.1.6's
//! table (PostgreSQL 16), highest first: `.` · `::` · `[ ]` · unary `+ -` · `collate` · `at` ·
//! `^` · `* / %` · binary `+ -` · every other operator · `between in like ilike similar` ·
//! comparison · `is isnull notnull` · `not` · `and` · `or`.

use crate::ast::*;
use crate::lex::{Span, Tok, Token};

/// PostgreSQL 16's fully reserved keywords (Appendix C, "reserved"): never a column name,
/// table name or bare alias.
pub const RESERVED: &[&str] = &[
    "all",
    "analyse",
    "analyze",
    "and",
    "any",
    "array",
    "as",
    "asc",
    "asymmetric",
    "both",
    "case",
    "cast",
    "check",
    "collate",
    "column",
    "constraint",
    "create",
    "current_catalog",
    "current_date",
    "current_role",
    "current_time",
    "current_timestamp",
    "current_user",
    "default",
    "deferrable",
    "desc",
    "distinct",
    "do",
    "else",
    "end",
    "except",
    "false",
    "fetch",
    "for",
    "foreign",
    "from",
    "grant",
    "group",
    "having",
    "in",
    "initially",
    "intersect",
    "into",
    "lateral",
    "leading",
    "limit",
    "localtime",
    "localtimestamp",
    "not",
    "null",
    "offset",
    "on",
    "only",
    "or",
    "order",
    "placing",
    "primary",
    "references",
    "returning",
    "select",
    "session_user",
    "some",
    "symmetric",
    "system_user",
    "table",
    "then",
    "to",
    "trailing",
    "true",
    "union",
    "unique",
    "user",
    "using",
    "variadic",
    "when",
    "where",
    "window",
    "with",
];

/// "Reserved (can be function or type name)": not a column or table name, but a function.
pub const TYPE_FUNC_RESERVED: &[&str] = &[
    "authorization",
    "binary",
    "collation",
    "concurrently",
    "cross",
    "current_schema",
    "freeze",
    "full",
    "ilike",
    "inner",
    "is",
    "isnull",
    "join",
    "left",
    "like",
    "natural",
    "notnull",
    "outer",
    "overlaps",
    "right",
    "similar",
    "tablesample",
    "verbose",
];

/// Words that end an expression where an alias could start without `as` — the non-reserved
/// words PostgreSQL's grammar does not accept as a bare label in that position.
const NOT_BARE_LABEL: &[&str] = &[
    "over",
    "filter",
    "within",
    "escape",
    "nulls",
    "rows",
    "range",
    "groups",
    "ordinality",
    "window",
    "limit",
    "offset",
    "fetch",
    "returning",
    "on",
    "using",
    "natural",
    "join",
    "inner",
    "left",
    "right",
    "full",
    "cross",
    "union",
    "intersect",
    "except",
    "where",
    "group",
    "having",
    "order",
    "into",
    "for",
    "from",
    "tablesample",
    "strict",
];

pub fn is_reserved(w: &str) -> bool {
    RESERVED.contains(&w)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub msg: String,
}

pub type PResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    pub src: &'a str,
    pub toks: Vec<Token>,
    pub pos: usize,
    /// Inside a PL/pgSQL body: `select … into x` and `returning … into x` are statements'
    /// targets, not SQL's `select into` (which creates a table).
    pub plpgsql: bool,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str, toks: Vec<Token>) -> Parser<'a> {
        Parser {
            src,
            toks,
            pos: 0,
            plpgsql: false,
        }
    }

    // ── the token stream ────────────────────────────────────────────────────────────────

    pub fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }
    pub fn peek_n(&self, n: usize) -> Option<&Tok> {
        self.toks.get(self.pos + n).map(|t| &t.tok)
    }
    pub fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }
    pub fn here(&self) -> Span {
        self.toks
            .get(self.pos)
            .map(|t| t.span)
            .or_else(|| {
                self.toks.last().map(|t| Span {
                    start: t.span.end,
                    end: t.span.end,
                })
            })
            .unwrap_or_default()
    }
    pub fn prev_end(&self) -> usize {
        if self.pos == 0 {
            0
        } else {
            self.toks[self.pos - 1].span.end
        }
    }
    pub fn span_from(&self, start: usize) -> Span {
        Span {
            start,
            end: self.prev_end().max(start),
        }
    }
    pub fn bump(&mut self) -> Option<Token> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    pub fn err<T>(&self, msg: impl Into<String>) -> PResult<T> {
        let found = match self.peek() {
            None => "end of input".to_string(),
            Some(t) => format!("{t:?}"),
        };
        Err(ParseError {
            span: self.here(),
            msg: format!("{} (found {found})", msg.into()),
        })
    }

    /// The unquoted word at the cursor, if any.
    pub fn word(&self) -> Option<&str> {
        match self.peek() {
            Some(Tok::Ident(w)) => Some(w.as_str()),
            _ => None,
        }
    }
    pub fn word_n(&self, n: usize) -> Option<&str> {
        match self.peek_n(n) {
            Some(Tok::Ident(w)) => Some(w.as_str()),
            _ => None,
        }
    }
    pub fn is_kw(&self, kw: &str) -> bool {
        self.word() == Some(kw)
    }
    pub fn is_kw_n(&self, n: usize, kw: &str) -> bool {
        self.word_n(n) == Some(kw)
    }
    pub fn eat_kw(&mut self, kw: &str) -> bool {
        if self.is_kw(kw) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    /// A sequence of keywords, all or nothing.
    pub fn eat_kws(&mut self, kws: &[&str]) -> bool {
        if kws.iter().enumerate().all(|(i, k)| self.is_kw_n(i, k)) {
            self.pos += kws.len();
            true
        } else {
            false
        }
    }
    pub fn expect_kw(&mut self, kw: &str) -> PResult<()> {
        if self.eat_kw(kw) {
            Ok(())
        } else {
            self.err(format!("expected `{kw}`"))
        }
    }
    pub fn is(&self, t: &Tok) -> bool {
        self.peek() == Some(t)
    }
    pub fn eat(&mut self, t: &Tok) -> bool {
        if self.is(t) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    pub fn expect(&mut self, t: &Tok) -> PResult<()> {
        if self.eat(t) {
            Ok(())
        } else {
            self.err(format!("expected {t:?}"))
        }
    }
    pub fn is_op(&self, op: &str) -> bool {
        matches!(self.peek(), Some(Tok::Op(o)) if o == op)
    }
    pub fn eat_op(&mut self, op: &str) -> bool {
        if self.is_op(op) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// An identifier: an unquoted word that is not fully reserved, or a quoted identifier.
    pub fn ident(&mut self) -> PResult<String> {
        match self.peek().cloned() {
            Some(Tok::QuotedIdent(s)) => {
                self.pos += 1;
                Ok(s)
            }
            Some(Tok::Ident(w)) if !is_reserved(&w) => {
                self.pos += 1;
                Ok(w)
            }
            _ => self.err("expected an identifier"),
        }
    }
    /// Any word, reserved or not — for positions where the grammar accepts every keyword
    /// (attribute names after `.`, option names).
    pub fn any_word(&mut self) -> PResult<String> {
        match self.peek().cloned() {
            Some(Tok::QuotedIdent(s)) | Some(Tok::Ident(s)) => {
                self.pos += 1;
                Ok(s)
            }
            _ => self.err("expected a name"),
        }
    }
    /// A qualified name: `a`, `a.b`, `a.b.c`.
    pub fn name(&mut self) -> PResult<Name> {
        let mut parts = vec![self.ident_or_func_reserved()?];
        while self.is(&Tok::Dot) && !matches!(self.peek_n(1), Some(Tok::Op(o)) if o == "*") {
            self.pos += 1;
            parts.push(self.any_word()?);
        }
        Ok(Name(parts))
    }
    /// An identifier that may also be a type/function-reserved word (`left`, `right`, …) —
    /// the position of a function or type name.
    pub fn ident_or_func_reserved(&mut self) -> PResult<String> {
        if let Some(Tok::Ident(w)) = self.peek() {
            if TYPE_FUNC_RESERVED.contains(&w.as_str()) {
                let w = w.clone();
                self.pos += 1;
                return Ok(w);
            }
        }
        self.ident()
    }
    pub fn ident_list(&mut self) -> PResult<Vec<String>> {
        let mut v = vec![self.ident()?];
        while self.eat(&Tok::Comma) {
            v.push(self.ident()?);
        }
        Ok(v)
    }
    pub fn paren_ident_list(&mut self) -> PResult<Vec<String>> {
        self.expect(&Tok::LParen)?;
        let v = self.ident_list()?;
        self.expect(&Tok::RParen)?;
        Ok(v)
    }

    /// Skip a balanced run of tokens up to (not including) a `;` or `)` at depth 0 or the end;
    /// returns the source text skipped. For clauses recognised but not modelled.
    pub fn skip_balanced(&mut self, stop_words: &[&str]) -> String {
        let start = self.here().start;
        let mut depth = 0i32;
        while let Some(t) = self.peek() {
            match t {
                Tok::LParen | Tok::LBracket => depth += 1,
                Tok::RParen | Tok::RBracket if depth == 0 => break,
                Tok::RParen | Tok::RBracket => depth -= 1,
                Tok::Semi if depth == 0 => break,
                Tok::Ident(w) if depth == 0 && stop_words.contains(&w.as_str()) => break,
                _ => {}
            }
            self.pos += 1;
        }
        self.src[start..self.prev_end().max(start)].to_string()
    }

    /// Skip to just past the `;` that ends the statement at the cursor (outside parentheses),
    /// or to the end.
    pub fn skip_statement(&mut self) {
        let mut depth = 0i32;
        while let Some(t) = self.peek() {
            match t {
                Tok::LParen | Tok::LBracket => depth += 1,
                Tok::RParen | Tok::RBracket => depth -= 1,
                Tok::Semi if depth <= 0 => {
                    self.pos += 1;
                    return;
                }
                _ => {}
            }
            self.pos += 1;
        }
    }

    // ── types ──────────────────────────────────────────────────────────────────────────────

    pub fn type_name(&mut self) -> PResult<TypeName> {
        let start = self.here().start;
        let mut name: Option<Name> = None;
        let multi: &[(&[&str], &str)] = &[
            (&["double", "precision"], "float8"),
            (&["character", "varying"], "varchar"),
            (&["char", "varying"], "varchar"),
            (&["national", "character", "varying"], "varchar"),
            (&["national", "char", "varying"], "varchar"),
            (&["national", "character"], "bpchar"),
            (&["national", "char"], "bpchar"),
            (&["bit", "varying"], "varbit"),
        ];
        for (words, canon) in multi {
            if self.eat_kws(words) {
                name = Some(Name(vec![(*canon).into()]));
                break;
            }
        }
        let name = match name {
            Some(n) => n,
            None => {
                if self.is_kw("setof") {
                    return self.err("`setof` is only allowed in `returns`");
                }
                let n = self.name()?;
                let canon = |s: &str| -> Option<&'static str> {
                    Some(match s {
                        "int" | "integer" | "int4" => "int4",
                        "smallint" | "int2" => "int2",
                        "bigint" | "int8" => "int8",
                        "real" | "float4" => "float4",
                        "float8" => "float8",
                        "decimal" | "numeric" => "numeric",
                        "boolean" | "bool" => "bool",
                        "varchar" => "varchar",
                        "character" | "char" | "bpchar" => "bpchar",
                        "dec" => "numeric",
                        _ => return None,
                    })
                };
                if n.0.len() == 1 {
                    match canon(&n.0[0]) {
                        Some(c) => Name(vec![c.into()]),
                        None => n,
                    }
                } else {
                    n
                }
            }
        };
        let mut ty = TypeName {
            name,
            mods: Vec::new(),
            array_dims: 0,
            percent: None,
            written: String::new(),
            span: Span::default(),
        };
        // `interval` takes field qualifiers: `interval day to second (3)`.
        if ty.name.0 == ["interval"] {
            let fields = ["year", "month", "day", "hour", "minute", "second"];
            while self
                .word()
                .is_some_and(|w| fields.contains(&w) || w == "to")
            {
                self.pos += 1;
            }
        }
        if self.is(&Tok::LParen) {
            self.pos += 1;
            if !self.is(&Tok::RParen) {
                ty.mods.push(self.expr()?);
                while self.eat(&Tok::Comma) {
                    ty.mods.push(self.expr()?);
                }
            }
            self.expect(&Tok::RParen)?;
        }
        // `timestamp(3) with time zone`, `time without time zone`.
        let base = ty.name.last().to_string();
        if base == "timestamp" || base == "time" {
            if self.eat_kws(&["with", "time", "zone"]) {
                ty.name = Name(vec![format!("{base}tz")]);
            } else {
                self.eat_kws(&["without", "time", "zone"]);
            }
        }
        if self.is(&Tok::Op("%".into())) {
            self.pos += 1;
            let w = self.any_word()?;
            ty.percent = Some(w);
        }
        loop {
            if self.is(&Tok::LBracket) {
                self.pos += 1;
                if let Some(Tok::Int(_)) = self.peek() {
                    self.pos += 1;
                }
                self.expect(&Tok::RBracket)?;
                ty.array_dims += 1;
            } else if self.is_kw("array") {
                self.pos += 1;
                if self.eat(&Tok::LBracket) {
                    if let Some(Tok::Int(_)) = self.peek() {
                        self.pos += 1;
                    }
                    self.expect(&Tok::RBracket)?;
                }
                ty.array_dims += 1;
            } else {
                break;
            }
        }
        ty.span = self.span_from(start);
        ty.written = self.src[ty.span.start..ty.span.end].to_string();
        Ok(ty)
    }

    // ── expressions ────────────────────────────────────────────────────────────────────────

    pub fn expr(&mut self) -> PResult<Expr> {
        self.or_expr()
    }

    fn or_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut l = self.and_expr()?;
        while self.eat_kw("or") {
            let r = self.and_expr()?;
            l = Expr::Bin(Box::new(l), "or".into(), Box::new(r), self.span_from(start));
        }
        Ok(l)
    }

    fn and_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut l = self.not_expr()?;
        while self.eat_kw("and") {
            let r = self.not_expr()?;
            l = Expr::Bin(
                Box::new(l),
                "and".into(),
                Box::new(r),
                self.span_from(start),
            );
        }
        Ok(l)
    }

    fn not_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        if self.eat_kw("not") {
            let e = self.not_expr()?;
            return Ok(Expr::Un("not".into(), Box::new(e), self.span_from(start)));
        }
        self.is_expr()
    }

    /// `is [not] null | true | false | unknown | distinct from e | document | normalized`,
    /// `isnull`, `notnull`.
    fn is_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut e = self.cmp_expr()?;
        loop {
            if self.eat_kw("isnull") {
                e = Expr::Is(Box::new(e), "null".into(), self.span_from(start));
            } else if self.eat_kw("notnull") {
                e = Expr::Is(Box::new(e), "not null".into(), self.span_from(start));
            } else if self.eat_kw("is") {
                let not = self.eat_kw("not");
                let pfx = if not { "not " } else { "" };
                if self.eat_kws(&["distinct", "from"]) {
                    let r = self.cmp_expr()?;
                    let op = if not {
                        "is not distinct from"
                    } else {
                        "is distinct from"
                    };
                    e = Expr::Bin(Box::new(e), op.into(), Box::new(r), self.span_from(start));
                } else if self.eat_kw("of") {
                    self.expect(&Tok::LParen)?;
                    let mut tys = vec![self.type_name()?.written];
                    while self.eat(&Tok::Comma) {
                        tys.push(self.type_name()?.written);
                    }
                    self.expect(&Tok::RParen)?;
                    e = Expr::Is(
                        Box::new(e),
                        format!("{pfx}of ({})", tys.join(", ")),
                        self.span_from(start),
                    );
                } else {
                    let w = self.any_word()?;
                    let w = if ["nfc", "nfd", "nfkc", "nfkd"].contains(&w.as_str()) {
                        self.expect_kw("normalized")?;
                        format!("{w} normalized")
                    } else {
                        w
                    };
                    if !["null", "true", "false", "unknown", "document", "normalized"]
                        .iter()
                        .any(|k| w.ends_with(k))
                    {
                        return self.err("expected null, true, false, unknown, distinct from, of, document or normalized after `is`");
                    }
                    e = Expr::Is(Box::new(e), format!("{pfx}{w}"), self.span_from(start));
                }
            } else {
                return Ok(e);
            }
        }
    }

    /// Comparison, non-associative in PostgreSQL ≥ 9.5: `a < b < c` is an error.
    fn cmp_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let l = self.pattern_expr()?;
        for op in ["<", ">", "=", "<=", ">=", "<>", "!="] {
            if self.is_op(op) {
                self.pos += 1;
                let op = if op == "!=" { "<>" } else { op };
                if self.is_quantifier() {
                    return self.quantified(l, op.into(), start);
                }
                let r = self.pattern_expr()?;
                return Ok(Expr::Bin(
                    Box::new(l),
                    op.into(),
                    Box::new(r),
                    self.span_from(start),
                ));
            }
        }
        Ok(l)
    }

    fn is_quantifier(&self) -> bool {
        self.word()
            .is_some_and(|w| w == "any" || w == "some" || w == "all")
            && self.peek_n(1) == Some(&Tok::LParen)
    }

    fn quantified(&mut self, l: Expr, op: String, start: usize) -> PResult<Expr> {
        let q = self.any_word()?;
        self.expect(&Tok::LParen)?;
        let inner = if self.starts_query() {
            Expr::Sub(Box::new(self.query()?), self.here())
        } else {
            self.expr()?
        };
        self.expect(&Tok::RParen)?;
        Ok(Expr::Quantified(
            Box::new(l),
            op,
            q,
            Box::new(inner),
            self.span_from(start),
        ))
    }

    /// `between`, `in`, `like`, `ilike`, `similar to`, each optionally negated.
    fn pattern_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let l = self.other_op_expr()?;
        let negated = self.is_kw("not")
            && self
                .word_n(1)
                .is_some_and(|w| ["between", "in", "like", "ilike", "similar"].contains(&w));
        if negated {
            self.pos += 1;
        }
        if self.eat_kw("between") {
            let symmetric = self.eat_kw("symmetric");
            self.eat_kw("asymmetric");
            let lo = self.other_op_expr()?;
            self.expect_kw("and")?;
            let hi = self.other_op_expr()?;
            return Ok(Expr::Between {
                e: Box::new(l),
                lo: Box::new(lo),
                hi: Box::new(hi),
                negated,
                symmetric,
                span: self.span_from(start),
            });
        }
        if self.eat_kw("in") {
            self.expect(&Tok::LParen)?;
            if self.starts_query() {
                let q = self.query()?;
                self.expect(&Tok::RParen)?;
                return Ok(Expr::InSub(
                    Box::new(l),
                    Box::new(q),
                    negated,
                    self.span_from(start),
                ));
            }
            let mut v = vec![self.expr()?];
            while self.eat(&Tok::Comma) {
                v.push(self.expr()?);
            }
            self.expect(&Tok::RParen)?;
            return Ok(Expr::InList(Box::new(l), v, negated, self.span_from(start)));
        }
        for (kw, op) in [("like", "like"), ("ilike", "ilike")] {
            if self.eat_kw(kw) {
                return self.like_rest(l, op, negated, start);
            }
        }
        if self.eat_kws(&["similar", "to"]) {
            return self.like_rest(l, "similar to", negated, start);
        }
        if negated {
            return self.err("expected between, in, like, ilike or similar after `not`");
        }
        // `(a, b) overlaps (c, d)`: two periods.
        if self.eat_kw("overlaps") {
            let r = self.other_op_expr()?;
            return Ok(Expr::Bin(
                Box::new(l),
                "overlaps".into(),
                Box::new(r),
                self.span_from(start),
            ));
        }
        Ok(l)
    }

    fn like_rest(&mut self, l: Expr, op: &str, negated: bool, start: usize) -> PResult<Expr> {
        if self.is_quantifier() {
            let q = self.quantified(l, op.into(), start)?;
            return Ok(if negated {
                Expr::Un("not".into(), Box::new(q), self.span_from(start))
            } else {
                q
            });
        }
        let mut r = self.other_op_expr()?;
        if self.eat_kw("escape") {
            let esc = self.other_op_expr()?;
            r = Expr::Bin(
                Box::new(r),
                "escape".into(),
                Box::new(esc),
                self.span_from(start),
            );
        }
        let op = if negated {
            format!("not {op}")
        } else {
            op.to_string()
        };
        Ok(Expr::Bin(
            Box::new(l),
            op,
            Box::new(r),
            self.span_from(start),
        ))
    }

    /// "Any other operator" — `||`, `@>`, `->>`, `&&`, `~`, … and `operator(schema.op)` —
    /// left-associative, one level. A prefix use of such an operator is handled in `unary`.
    fn other_op_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut l = self.add_expr()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Op(o))
                    if ![
                        "+", "-", "*", "/", "%", "^", "<", ">", "=", "<=", ">=", "<>", "!=",
                    ]
                    .contains(&o.as_str()) =>
                {
                    o.clone()
                }
                Some(Tok::Ident(w)) if w == "operator" && self.peek_n(1) == Some(&Tok::LParen) => {
                    self.explicit_operator_peek()
                }
                _ => return Ok(l),
            };
            if op.starts_with("operator(") {
                self.explicit_operator()?;
            } else {
                self.pos += 1;
            }
            let r = self.add_expr()?;
            l = Expr::Bin(Box::new(l), op, Box::new(r), self.span_from(start));
        }
    }

    fn explicit_operator_peek(&self) -> String {
        let mut s = String::from("operator(");
        let mut i = 2;
        while let Some(t) = self.peek_n(i) {
            match t {
                Tok::RParen => break,
                Tok::Ident(w) => s.push_str(w),
                Tok::Dot => s.push('.'),
                Tok::Op(o) => s.push_str(o),
                _ => {}
            }
            i += 1;
        }
        s.push(')');
        s
    }

    fn explicit_operator(&mut self) -> PResult<()> {
        self.expect_kw("operator")?;
        self.expect(&Tok::LParen)?;
        while !self.is(&Tok::RParen) {
            if self.bump().is_none() {
                return self.err("unterminated operator( … )");
            }
        }
        self.expect(&Tok::RParen)
    }

    fn add_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut l = self.mul_expr()?;
        loop {
            let op = if self.is_op("+") {
                "+"
            } else if self.is_op("-") {
                "-"
            } else {
                return Ok(l);
            };
            self.pos += 1;
            let r = self.mul_expr()?;
            l = Expr::Bin(Box::new(l), op.into(), Box::new(r), self.span_from(start));
        }
    }

    fn mul_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut l = self.exp_expr()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Op(o)) if o == "*" || o == "/" || o == "%" => o.clone(),
                _ => return Ok(l),
            };
            self.pos += 1;
            let r = self.exp_expr()?;
            l = Expr::Bin(Box::new(l), op, Box::new(r), self.span_from(start));
        }
    }

    /// `^` is left-associative in PostgreSQL (unlike mathematics).
    fn exp_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut l = self.at_expr()?;
        while self.eat_op("^") {
            let r = self.at_expr()?;
            l = Expr::Bin(Box::new(l), "^".into(), Box::new(r), self.span_from(start));
        }
        Ok(l)
    }

    fn at_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut l = self.collate_expr()?;
        loop {
            if self.eat_kws(&["at", "time", "zone"]) {
                let z = self.collate_expr()?;
                l = Expr::AtTimeZone(Box::new(l), Box::new(z), self.span_from(start));
            } else if self.eat_kws(&["at", "local"]) {
                l = Expr::Un("at local".into(), Box::new(l), self.span_from(start));
            } else {
                return Ok(l);
            }
        }
    }

    fn collate_expr(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut l = self.unary()?;
        while self.eat_kw("collate") {
            let n = self.name()?;
            l = Expr::Collate(Box::new(l), n, self.span_from(start));
        }
        Ok(l)
    }

    fn unary(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        if let Some(Tok::Op(o)) = self.peek() {
            let o = o.clone();
            if o == "+" || o == "-" {
                self.pos += 1;
                let e = self.unary()?;
                // A negative numeric literal stays a literal: the checker reads `-100` in a
                // posting as a constant, not as an expression.
                if o == "-" {
                    if let Expr::Lit(Literal::Int(n), _) = &e {
                        return Ok(Expr::Lit(
                            Literal::Int(format!("-{n}")),
                            self.span_from(start),
                        ));
                    }
                    if let Expr::Lit(Literal::Num(n), _) = &e {
                        return Ok(Expr::Lit(
                            Literal::Num(format!("-{n}")),
                            self.span_from(start),
                        ));
                    }
                }
                return Ok(Expr::Un(o, Box::new(e), self.span_from(start)));
            }
            // A prefix user operator: `~x`, `@x`, `|/x`, `!!x`.
            if !["*", "/", "%", "^", "<", ">", "=", "<=", ">=", "<>", "!="].contains(&o.as_str()) {
                self.pos += 1;
                let e = self.unary()?;
                return Ok(Expr::Un(o, Box::new(e), self.span_from(start)));
            }
        }
        self.postfix()
    }

    /// `::type`, `[subscript]`, `.field` after a primary.
    fn postfix(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut e = self.primary()?;
        loop {
            if self.eat(&Tok::Cast) {
                let t = self.type_name()?;
                e = Expr::Cast(Box::new(e), Box::new(t), self.span_from(start));
            } else if self.is(&Tok::LBracket) {
                self.pos += 1;
                let lo = if self.is(&Tok::Colon) || self.is(&Tok::RBracket) {
                    None
                } else {
                    Some(self.expr()?)
                };
                let (hi, slice) = if self.eat(&Tok::Colon) {
                    (
                        if self.is(&Tok::RBracket) {
                            None
                        } else {
                            Some(self.expr()?)
                        },
                        true,
                    )
                } else {
                    (None, false)
                };
                self.expect(&Tok::RBracket)?;
                e = Expr::Subscript(
                    Box::new(e),
                    Box::new(lo),
                    Box::new(hi),
                    slice,
                    self.span_from(start),
                );
            } else if self.is(&Tok::Dot)
                && matches!(
                    e,
                    Expr::Row(..)
                        | Expr::Sub(..)
                        | Expr::Func(..)
                        | Expr::Param(..)
                        | Expr::Subscript(..)
                        | Expr::Field(..)
                )
            {
                self.pos += 1;
                let f = if self.eat_op("*") {
                    "*".to_string()
                } else {
                    self.any_word()?
                };
                e = Expr::Field(Box::new(e), f, self.span_from(start));
            } else {
                return Ok(e);
            }
        }
    }

    pub fn starts_query(&self) -> bool {
        self.word()
            .is_some_and(|w| ["select", "with", "values", "table"].contains(&w))
            || (self.is(&Tok::LParen) && {
                // `((select …))`: look through parentheses.
                let mut i = 0;
                while self.peek_n(i) == Some(&Tok::LParen) {
                    i += 1;
                }
                self.word_n(i)
                    .is_some_and(|w| ["select", "with", "values"].contains(&w))
            })
    }

    fn primary(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let span = |p: &Parser| p.span_from(start);
        let Some(t) = self.peek().cloned() else {
            return self.err("expected an expression");
        };
        match t {
            Tok::Int(n) => {
                self.pos += 1;
                Ok(Expr::Lit(Literal::Int(n), span(self)))
            }
            Tok::Num(n) => {
                self.pos += 1;
                Ok(Expr::Lit(Literal::Num(n), span(self)))
            }
            Tok::Str(s) => {
                self.pos += 1;
                Ok(Expr::Lit(Literal::Str(s), span(self)))
            }
            Tok::BitStr(s) => {
                self.pos += 1;
                Ok(Expr::Lit(Literal::BitStr(s), span(self)))
            }
            Tok::HexStr(s) => {
                self.pos += 1;
                Ok(Expr::Lit(Literal::HexStr(s), span(self)))
            }
            Tok::Param(n) => {
                self.pos += 1;
                Ok(Expr::Param(n, span(self)))
            }
            Tok::Op(o) if o == "*" => {
                self.pos += 1;
                Ok(Expr::Star(span(self)))
            }
            Tok::LParen => {
                if self.starts_query() {
                    self.pos += 1;
                    let q = self.query()?;
                    self.expect(&Tok::RParen)?;
                    return Ok(Expr::Sub(Box::new(q), span(self)));
                }
                self.pos += 1;
                let first = self.expr()?;
                if self.eat(&Tok::Comma) {
                    let mut v = vec![first];
                    loop {
                        v.push(self.expr()?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(&Tok::RParen)?;
                    return Ok(Expr::Row(v, span(self)));
                }
                self.expect(&Tok::RParen)?;
                // `(e).field`: a parenthesised expression followed by `.` is a composite
                // value whose field is selected — kept as a one-element row, which `postfix`
                // accepts as a field base. Otherwise the parentheses are only grouping.
                if self.is(&Tok::Dot) {
                    return Ok(Expr::Row(vec![first], span(self)));
                }
                Ok(first)
            }
            Tok::QuotedIdent(_) => self.column_or_call(start),
            Tok::Ident(w) => self.keyword_primary(&w, start),
            _ => self.err("expected an expression"),
        }
    }

    fn keyword_primary(&mut self, w: &str, start: usize) -> PResult<Expr> {
        match w {
            "null" => {
                self.pos += 1;
                Ok(Expr::Lit(Literal::Null, self.span_from(start)))
            }
            "true" | "false" => {
                self.pos += 1;
                Ok(Expr::Lit(Literal::Bool(w == "true"), self.span_from(start)))
            }
            "default" => {
                self.pos += 1;
                Ok(Expr::Default(self.span_from(start)))
            }
            "case" => self.case_expr(start),
            "cast" if self.peek_n(1) == Some(&Tok::LParen) => {
                self.pos += 2;
                let e = self.expr()?;
                self.expect_kw("as")?;
                let t = self.type_name()?;
                self.expect(&Tok::RParen)?;
                Ok(Expr::Cast(Box::new(e), Box::new(t), self.span_from(start)))
            }
            "exists" if self.peek_n(1) == Some(&Tok::LParen) => {
                self.pos += 2;
                let q = self.query()?;
                self.expect(&Tok::RParen)?;
                Ok(Expr::Exists(Box::new(q), self.span_from(start)))
            }
            "array" if self.peek_n(1) == Some(&Tok::LBracket) => {
                self.pos += 1;
                self.array_literal(start)
            }
            "array" if self.peek_n(1) == Some(&Tok::LParen) => {
                self.pos += 2;
                let q = self.query()?;
                self.expect(&Tok::RParen)?;
                Ok(Expr::ArraySub(Box::new(q), self.span_from(start)))
            }
            "row" if self.peek_n(1) == Some(&Tok::LParen) => {
                self.pos += 2;
                let mut v = Vec::new();
                if !self.is(&Tok::RParen) {
                    v.push(self.expr()?);
                    while self.eat(&Tok::Comma) {
                        v.push(self.expr()?);
                    }
                }
                self.expect(&Tok::RParen)?;
                Ok(Expr::Row(v, self.span_from(start)))
            }
            // SQL value functions written without parentheses.
            "current_date" | "current_time" | "current_timestamp" | "localtime"
            | "localtimestamp" | "current_user" | "session_user" | "user" | "current_role"
            | "current_catalog" | "current_schema" | "system_user"
                if self.peek_n(1) != Some(&Tok::LParen)
                    || w.starts_with("current_t")
                    || w.starts_with("local") =>
            {
                self.pos += 1;
                // `current_timestamp(3)`: an optional precision.
                let mut args = Vec::new();
                if self.is(&Tok::LParen) {
                    self.pos += 1;
                    args.push((None, self.expr()?));
                    self.expect(&Tok::RParen)?;
                }
                Ok(Expr::Func(Box::new(FuncCall {
                    name: Name(vec![w.to_string()]),
                    args,
                    star: false,
                    distinct: false,
                    variadic: false,
                    order_by: vec![],
                    within_group: vec![],
                    filter: None,
                    over: None,
                    span: self.span_from(start),
                })))
            }
            // A typed literal: `date '2026-01-01'`, `interval '1 day'`, `numeric '1.5'`.
            _ if self.typed_literal_ahead() => {
                let ty = self.type_name()?;
                let Some(Tok::Str(s)) = self.peek().cloned() else {
                    return self.err("expected a string after the type name");
                };
                self.pos += 1;
                // `interval '1' day`: trailing field qualifiers.
                if ty.name.0 == ["interval"] {
                    let fields = ["year", "month", "day", "hour", "minute", "second", "to"];
                    while self.word().is_some_and(|w| fields.contains(&w)) {
                        self.pos += 1;
                    }
                }
                Ok(Expr::Lit(
                    Literal::Typed(Box::new(ty), s),
                    self.span_from(start),
                ))
            }
            "extract" | "position" | "substring" | "overlay" | "trim" | "normalize"
                if self.peek_n(1) == Some(&Tok::LParen) =>
            {
                self.special_call(w, start)
            }
            _ if is_reserved(w) => self.err(format!("`{w}` is a reserved word here")),
            _ => self.column_or_call(start),
        }
    }

    /// `date '…'`: a (non-reserved) type name immediately followed by a string constant.
    fn typed_literal_ahead(&self) -> bool {
        let simple = [
            "date",
            "time",
            "timestamp",
            "timestamptz",
            "interval",
            "numeric",
            "int",
            "integer",
            "bigint",
            "smallint",
            "boolean",
            "bool",
            "text",
            "varchar",
            "char",
            "real",
            "float",
            "decimal",
            "json",
            "jsonb",
            "uuid",
            "inet",
            "cidr",
            "bytea",
            "money",
            "point",
            "tsquery",
            "tsvector",
            "bit",
            "xml",
        ];
        match (self.word(), self.peek_n(1)) {
            (Some(w), Some(Tok::Str(_))) => simple.contains(&w) || !is_reserved(w),
            (Some("double"), _) => {
                self.word_n(1) == Some("precision") && matches!(self.peek_n(2), Some(Tok::Str(_)))
            }
            (Some("timestamp" | "time"), Some(Tok::Ident(x))) if x == "with" || x == "without" => {
                matches!(self.peek_n(4), Some(Tok::Str(_)))
            }
            _ => false,
        }
    }

    fn array_literal(&mut self, start: usize) -> PResult<Expr> {
        self.expect(&Tok::LBracket)?;
        let mut v = Vec::new();
        if !self.is(&Tok::RBracket) {
            loop {
                if self.is(&Tok::LBracket) {
                    let s = self.here().start;
                    v.push(self.array_literal(s)?);
                } else {
                    v.push(self.expr()?);
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }
        self.expect(&Tok::RBracket)?;
        Ok(Expr::Array(v, self.span_from(start)))
    }

    fn case_expr(&mut self, start: usize) -> PResult<Expr> {
        self.expect_kw("case")?;
        let operand = if self.is_kw("when") {
            None
        } else {
            Some(Box::new(self.expr()?))
        };
        let mut whens = Vec::new();
        while self.eat_kw("when") {
            let c = self.expr()?;
            self.expect_kw("then")?;
            let r = self.expr()?;
            whens.push((c, r));
        }
        if whens.is_empty() {
            return self.err("`case` needs at least one `when`");
        }
        let otherwise = if self.eat_kw("else") {
            Some(Box::new(self.expr()?))
        } else {
            None
        };
        self.expect_kw("end")?;
        Ok(Expr::Case {
            operand,
            whens,
            otherwise,
            span: self.span_from(start),
        })
    }

    /// The functions with keyword-separated arguments; kept as calls with positional args in
    /// source order (`extract(f from e)` → `extract('f', e)`).
    fn special_call(&mut self, w: &str, start: usize) -> PResult<Expr> {
        self.pos += 2;
        let mut args = Vec::new();
        match w {
            "extract" => {
                let f = self.any_word()?;
                args.push((None, Expr::Lit(Literal::Str(f), self.span_from(start))));
                self.expect_kw("from")?;
                args.push((None, self.expr()?));
            }
            "trim" => {
                if self
                    .word()
                    .is_some_and(|x| ["both", "leading", "trailing"].contains(&x))
                {
                    let how = self.any_word()?;
                    args.push((None, Expr::Lit(Literal::Str(how), self.span_from(start))));
                }
                if !self.eat_kw("from") {
                    args.push((None, self.expr()?));
                    if self.eat_kw("from") || self.eat(&Tok::Comma) {
                        args.push((None, self.expr()?));
                    }
                } else {
                    args.push((None, self.expr()?));
                }
            }
            "normalize" => {
                args.push((None, self.expr()?));
                if self.eat(&Tok::Comma) {
                    let f = self.any_word()?;
                    args.push((None, Expr::Lit(Literal::Str(f), self.span_from(start))));
                }
            }
            _ => {
                // position(a in b), substring(a from b for c | a similar b escape c | a, b, c),
                // overlay(a placing b from c for d).
                args.push((None, self.pattern_free_expr()?));
                loop {
                    if self.eat(&Tok::Comma)
                        || self.eat_kw("in")
                        || self.eat_kw("from")
                        || self.eat_kw("for")
                        || self.eat_kw("placing")
                        || self.eat_kw("similar")
                        || self.eat_kw("escape")
                    {
                        args.push((None, self.pattern_free_expr()?));
                    } else {
                        break;
                    }
                }
            }
        }
        self.expect(&Tok::RParen)?;
        Ok(Expr::Func(Box::new(FuncCall {
            name: Name(vec![w.to_string()]),
            args,
            star: false,
            distinct: false,
            variadic: false,
            order_by: vec![],
            within_group: vec![],
            filter: None,
            over: None,
            span: self.span_from(start),
        })))
    }

    /// An expression that stops before `in` (inside `position(a in b)`), which is what
    /// PostgreSQL's `b_expr` production is for.
    fn pattern_free_expr(&mut self) -> PResult<Expr> {
        self.other_op_expr()
    }

    fn column_or_call(&mut self, start: usize) -> PResult<Expr> {
        let mut parts = vec![self.ident_or_func_reserved()?];
        let mut star = false;
        while self.is(&Tok::Dot) {
            self.pos += 1;
            if self.eat_op("*") {
                star = true;
                break;
            }
            parts.push(self.any_word()?);
        }
        let name = Name(parts);
        if star {
            let mut n = name;
            n.0.push("*".into());
            return Ok(Expr::Col(n, self.span_from(start)));
        }
        if self.is(&Tok::LParen) {
            return self.call(name, start);
        }
        Ok(Expr::Col(name, self.span_from(start)))
    }

    fn call(&mut self, name: Name, start: usize) -> PResult<Expr> {
        self.expect(&Tok::LParen)?;
        let mut f = FuncCall {
            name,
            args: Vec::new(),
            star: false,
            distinct: false,
            variadic: false,
            order_by: vec![],
            within_group: vec![],
            filter: None,
            over: None,
            span: Span::default(),
        };
        if self.eat_op("*") {
            f.star = true;
        } else if !self.is(&Tok::RParen) {
            if self.eat_kw("distinct") {
                f.distinct = true;
            } else {
                self.eat_kw("all");
            }
            loop {
                if self.eat_kw("variadic") {
                    f.variadic = true;
                }
                // Named notation: `name => e` or `name := e`.
                let named = matches!(self.peek(), Some(Tok::Ident(_)) | Some(Tok::QuotedIdent(_)))
                    && matches!(self.peek_n(1), Some(Tok::Arrow) | Some(Tok::Assign));
                if named {
                    let n = self.any_word()?;
                    self.pos += 1;
                    f.args.push((Some(n), self.expr()?));
                } else {
                    f.args.push((None, self.expr()?));
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            if self.eat_kws(&["order", "by"]) {
                f.order_by = self.order_list()?;
            }
        }
        self.expect(&Tok::RParen)?;
        if self.eat_kws(&["within", "group"]) {
            self.expect(&Tok::LParen)?;
            self.expect_kws_order_by()?;
            f.within_group = self.order_list()?;
            self.expect(&Tok::RParen)?;
        }
        if self.eat_kw("filter") {
            self.expect(&Tok::LParen)?;
            self.expect_kw("where")?;
            f.filter = Some(self.expr()?);
            self.expect(&Tok::RParen)?;
        }
        if self.eat_kw("over") {
            f.over = Some(if self.is(&Tok::LParen) {
                self.window_spec()?
            } else {
                Window {
                    base: Some(self.ident()?),
                    partition_by: vec![],
                    order_by: vec![],
                    frame: None,
                }
            });
        }
        f.span = self.span_from(start);
        Ok(Expr::Func(Box::new(f)))
    }

    fn expect_kws_order_by(&mut self) -> PResult<()> {
        if self.eat_kws(&["order", "by"]) {
            Ok(())
        } else {
            self.err("expected `order by`")
        }
    }

    pub fn window_spec(&mut self) -> PResult<Window> {
        self.expect(&Tok::LParen)?;
        let mut w = Window {
            base: None,
            partition_by: vec![],
            order_by: vec![],
            frame: None,
        };
        if matches!(self.peek(), Some(Tok::Ident(x)) if !["partition", "order", "rows", "range", "groups"].contains(&x.as_str()))
            || matches!(self.peek(), Some(Tok::QuotedIdent(_)))
        {
            w.base = Some(self.ident()?);
        }
        if self.eat_kws(&["partition", "by"]) {
            w.partition_by.push(self.expr()?);
            while self.eat(&Tok::Comma) {
                w.partition_by.push(self.expr()?);
            }
        }
        if self.eat_kws(&["order", "by"]) {
            w.order_by = self.order_list()?;
        }
        if self
            .word()
            .is_some_and(|x| ["rows", "range", "groups"].contains(&x))
        {
            w.frame = Some(self.skip_balanced(&[]));
        }
        self.expect(&Tok::RParen)?;
        Ok(w)
    }

    pub fn order_list(&mut self) -> PResult<Vec<OrderItem>> {
        let mut v = Vec::new();
        loop {
            let expr = self.expr()?;
            let mut item = OrderItem {
                expr,
                desc: false,
                using: None,
                nulls_first: None,
            };
            if self.eat_kw("desc") {
                item.desc = true;
            } else if self.eat_kw("asc") {
            } else if self.eat_kw("using") {
                if let Some(Tok::Op(o)) = self.peek().cloned() {
                    self.pos += 1;
                    item.using = Some(o);
                } else {
                    return self.err("expected an operator after `using`");
                }
            }
            if self.eat_kws(&["nulls", "first"]) {
                item.nulls_first = Some(true);
            } else if self.eat_kws(&["nulls", "last"]) {
                item.nulls_first = Some(false);
            }
            v.push(item);
            if !self.eat(&Tok::Comma) {
                return Ok(v);
            }
        }
    }

    /// Whether the word at the cursor can start a bare (no `as`) alias.
    pub fn can_be_bare_alias(&self) -> bool {
        match self.peek() {
            Some(Tok::QuotedIdent(_)) => true,
            Some(Tok::Ident(w)) => {
                !is_reserved(w)
                    && !TYPE_FUNC_RESERVED.contains(&w.as_str())
                    && !NOT_BARE_LABEL.contains(&w.as_str())
            }
            _ => false,
        }
    }
}
