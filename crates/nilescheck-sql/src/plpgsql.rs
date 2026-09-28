//! **PL/pgSQL** (PostgreSQL documentation, chapter 43), parsed from a function's body.
//!
//! The checks R2-05 adds — linearity and conservation over a function's postings — need the
//! control flow: which statements run on which path, and where a value is consumed. So the
//! body is parsed into blocks, branches and loops, with every embedded SQL statement parsed
//! by the SQL parser (`Parser::statement`) and every expression by the SQL expression parser,
//! as PL/pgSQL itself does (it hands expressions to the SQL engine as `SELECT expr`).
//!
//! Spans: when the body is a dollar-quoted string, its text is the source verbatim and spans
//! point into the original file. A single-quoted body has had its `''` pairs decoded, so its
//! spans are relative to the decoded body — stated rather than approximated.

use crate::ast::*;
use crate::lex::{self, Span, Tok};
use crate::parser::{PResult, ParseError, Parser};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub label: Option<String>,
    pub decls: Vec<Decl>,
    pub body: Vec<PlStmt>,
    /// `exception when c1 or c2 then …`.
    pub handlers: Vec<(Vec<String>, Vec<PlStmt>)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decl {
    pub name: String,
    pub constant: bool,
    pub ty: Option<TypeName>,
    pub not_null: bool,
    pub default: Option<Expr>,
    /// `x alias for $1`.
    pub alias_for: Option<String>,
    /// `c cursor [(args)] for query`.
    pub cursor: Option<Box<Query>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopKind {
    Plain,
    While(Expr),
    ForInt {
        var: String,
        reverse: bool,
        lo: Expr,
        hi: Expr,
        by: Option<Expr>,
    },
    ForQuery {
        vars: Vec<Name>,
        query: Box<Query>,
    },
    ForExecute {
        vars: Vec<Name>,
        command: Expr,
        using: Vec<Expr>,
    },
    ForCursor {
        var: String,
        cursor: String,
        args: Vec<Expr>,
    },
    Foreach {
        vars: Vec<Name>,
        slice: Option<Expr>,
        array: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlStmt {
    Assign {
        target: Expr,
        value: Expr,
        span: Span,
    },
    /// An embedded SQL statement, with its `into` targets when it has any.
    Sql {
        stmt: Box<Stmt>,
        into: Vec<Name>,
        strict: bool,
        span: Span,
    },
    Perform(Box<Query>, Span),
    Execute {
        command: Expr,
        into: Vec<Name>,
        strict: bool,
        using: Vec<Expr>,
        span: Span,
    },
    If {
        branches: Vec<(Expr, Vec<PlStmt>)>,
        otherwise: Option<Vec<PlStmt>>,
        span: Span,
    },
    Case {
        operand: Option<Expr>,
        whens: Vec<(Vec<Expr>, Vec<PlStmt>)>,
        otherwise: Option<Vec<PlStmt>>,
        span: Span,
    },
    Loop {
        label: Option<String>,
        kind: Box<LoopKind>,
        body: Vec<PlStmt>,
        span: Span,
    },
    Exit {
        is_continue: bool,
        label: Option<String>,
        when: Option<Expr>,
        span: Span,
    },
    Return(Option<Expr>, Span),
    ReturnNext(Option<Expr>, Span),
    ReturnQuery {
        query: Option<Box<Query>>,
        execute: Option<Expr>,
        span: Span,
    },
    Raise {
        level: Option<String>,
        message: Option<String>,
        args: Vec<Expr>,
        options: Vec<(String, Expr)>,
        span: Span,
    },
    Assert {
        cond: Expr,
        message: Option<Expr>,
        span: Span,
    },
    GetDiagnostics {
        stacked: bool,
        items: Vec<(Name, String)>,
        span: Span,
    },
    Block(Box<Block>),
    Null(Span),
    /// `open`, `fetch`, `move`, `close`, `call`, `commit`, `rollback` — kept with their text.
    Other {
        keyword: String,
        text: String,
        span: Span,
    },
}

/// Parse a function body. `src` is the whole file and `offset` where the body's text starts
/// in it; when the text is the source verbatim (dollar quoting), spans point into `src`.
pub fn parse_body_in(src: &str, text: &str, offset: usize) -> PResult<Block> {
    let verbatim = src.get(offset..offset + text.len()) == Some(text);
    let (base, shift) = if verbatim { (src, offset) } else { (text, 0) };
    let toks = lex::lex(text).map_err(|e| ParseError {
        span: Span {
            start: e.span.start + shift,
            end: e.span.end + shift,
        },
        msg: e.msg,
    })?;
    let (mut toks, _comments) = lex::split_comments(toks);
    for t in &mut toks {
        t.span.start += shift;
        t.span.end += shift;
    }
    let mut p = Parser::new(base, toks);
    p.plpgsql = true;
    let b = p.pl_block()?;
    p.eat(&Tok::Semi);
    if !p.at_end() {
        return p.err("text after the function body's final `end`");
    }
    Ok(b)
}

/// A body parsed on its own (spans relative to the body text).
pub fn parse_body(text: &str) -> PResult<Block> {
    parse_body_in(text, text, 0)
}

impl<'a> Parser<'a> {
    fn label(&mut self) -> PResult<Option<String>> {
        if self.is_op("<<") {
            self.pos += 1;
            let l = self.any_word()?;
            if !self.eat_op(">>") {
                return self.err("expected `>>` after the label");
            }
            return Ok(Some(l));
        }
        Ok(None)
    }

    pub fn pl_block(&mut self) -> PResult<Block> {
        let start = self.here().start;
        let label = self.label()?;
        let mut decls = Vec::new();
        if self.eat_kw("declare") {
            while !self.is_kw("begin") {
                if self.at_end() {
                    return self.err("expected `begin`");
                }
                if self.eat_kw("declare") {
                    continue;
                }
                decls.push(self.pl_decl()?);
            }
        }
        self.expect_kw("begin")?;
        let body = self.pl_stmts(&["end", "exception"])?;
        let mut handlers = Vec::new();
        if self.eat_kw("exception") {
            while self.eat_kw("when") {
                let mut conds = Vec::new();
                loop {
                    if self.eat_kw("sqlstate") {
                        match self.bump().map(|t| t.tok) {
                            Some(Tok::Str(s)) => conds.push(format!("sqlstate {s}")),
                            _ => return self.err("expected a SQLSTATE string"),
                        }
                    } else {
                        conds.push(self.any_word()?);
                    }
                    if !self.eat_kw("or") {
                        break;
                    }
                }
                self.expect_kw("then")?;
                let stmts = self.pl_stmts(&["when", "end"])?;
                handlers.push((conds, stmts));
            }
        }
        self.expect_kw("end")?;
        if self.word().is_some() && !self.is_kw("if") && !self.is_kw("loop") && !self.is_kw("case")
        {
            // `end label`
            if let Some(l) = &label {
                if self.is_kw(l) {
                    self.pos += 1;
                }
            }
        }
        Ok(Block {
            label,
            decls,
            body,
            handlers,
            span: self.span_from(start),
        })
    }

    fn pl_decl(&mut self) -> PResult<Decl> {
        let start = self.here().start;
        let name = self.any_word()?;
        let mut d = Decl {
            name,
            constant: false,
            ty: None,
            not_null: false,
            default: None,
            alias_for: None,
            cursor: None,
            span: Span::default(),
        };
        if self.eat_kws(&["alias", "for"]) {
            d.alias_for = Some(match self.bump().map(|t| t.tok) {
                Some(Tok::Param(n)) => format!("${n}"),
                Some(Tok::Ident(s)) | Some(Tok::QuotedIdent(s)) => s,
                _ => return self.err("expected $n or a name after `alias for`"),
            });
        } else if self.is_kw("cursor")
            || (self.is_kw("no") && self.is_kw_n(1, "scroll"))
            || self.is_kw("scroll")
        {
            self.eat_kws(&["no", "scroll"]);
            self.eat_kw("scroll");
            self.expect_kw("cursor")?;
            if self.is(&Tok::LParen) {
                self.pos += 1;
                self.skip_balanced(&[]);
                self.expect(&Tok::RParen)?;
            }
            if !self.eat_kw("for") {
                self.expect_kw("is")?;
            }
            d.cursor = Some(Box::new(self.query()?));
        } else {
            d.constant = self.eat_kw("constant");
            d.ty = Some(self.type_name()?);
            if self.eat_kw("collate") {
                self.name()?;
            }
            if self.eat_kws(&["not", "null"]) {
                d.not_null = true;
            }
            if self.eat_kw("default") || self.eat(&Tok::Assign) || self.eat_op("=") {
                d.default = Some(self.expr()?);
            }
        }
        self.expect(&Tok::Semi)?;
        d.span = self.span_from(start);
        Ok(d)
    }

    fn pl_stmts(&mut self, stop: &[&str]) -> PResult<Vec<PlStmt>> {
        let mut v = Vec::new();
        loop {
            while self.eat(&Tok::Semi) {}
            if self.at_end() {
                return self.err("unterminated PL/pgSQL block");
            }
            if self.word().is_some_and(|w| stop.contains(&w)) {
                return Ok(v);
            }
            v.push(self.pl_stmt()?);
        }
    }

    fn end_semi(&mut self) -> PResult<()> {
        if self.eat(&Tok::Semi) || self.at_end() {
            Ok(())
        } else {
            self.err("expected `;`")
        }
    }

    fn read_into_targets(&mut self) -> PResult<(Vec<Name>, bool)> {
        if !self.eat_kw("into") {
            return Ok((vec![], false));
        }
        let strict = self.eat_kw("strict");
        let mut v = vec![self.name()?];
        while self.eat(&Tok::Comma) {
            v.push(self.name()?);
        }
        Ok((v, strict))
    }

    fn pl_stmt(&mut self) -> PResult<PlStmt> {
        let start = self.here().start;
        if self.is_op("<<") {
            // A labelled block or loop.
            let save = self.pos;
            let label = self.label()?;
            if self.is_kw("declare") || self.is_kw("begin") {
                self.pos = save;
                let b = self.pl_block()?;
                self.end_semi()?;
                return Ok(PlStmt::Block(Box::new(b)));
            }
            return self.pl_loop(label, start);
        }
        let Some(w) = self.word().map(str::to_string) else {
            return self.err("expected a PL/pgSQL statement");
        };
        match w.as_str() {
            "declare" | "begin" => {
                let b = self.pl_block()?;
                self.end_semi()?;
                Ok(PlStmt::Block(Box::new(b)))
            }
            "null" if self.peek_n(1) == Some(&Tok::Semi) => {
                self.pos += 2;
                Ok(PlStmt::Null(self.span_from(start)))
            }
            "if" => self.pl_if(start),
            "case" => self.pl_case(start),
            "loop" | "while" | "for" | "foreach" => self.pl_loop(None, start),
            "exit" | "continue" => {
                self.pos += 1;
                let label = if self.word().is_some_and(|x| x != "when") {
                    Some(self.any_word()?)
                } else {
                    None
                };
                let when = if self.eat_kw("when") {
                    Some(self.expr()?)
                } else {
                    None
                };
                self.end_semi()?;
                Ok(PlStmt::Exit {
                    is_continue: w == "continue",
                    label,
                    when,
                    span: self.span_from(start),
                })
            }
            "return" => {
                self.pos += 1;
                if self.eat_kw("next") {
                    let e = if self.is(&Tok::Semi) {
                        None
                    } else {
                        Some(self.expr()?)
                    };
                    self.end_semi()?;
                    return Ok(PlStmt::ReturnNext(e, self.span_from(start)));
                }
                if self.eat_kw("query") {
                    let (query, execute) = if self.eat_kw("execute") {
                        let e = self.expr()?;
                        if self.eat_kw("using") {
                            self.expr()?;
                            while self.eat(&Tok::Comma) {
                                self.expr()?;
                            }
                        }
                        (None, Some(e))
                    } else {
                        (Some(Box::new(self.query()?)), None)
                    };
                    self.end_semi()?;
                    return Ok(PlStmt::ReturnQuery {
                        query,
                        execute,
                        span: self.span_from(start),
                    });
                }
                let e = if self.is(&Tok::Semi) {
                    None
                } else {
                    Some(self.expr()?)
                };
                self.end_semi()?;
                Ok(PlStmt::Return(e, self.span_from(start)))
            }
            "perform" => {
                self.pos += 1;
                // `perform q` is `select q` with the result discarded.
                let q = self.perform_query()?;
                self.end_semi()?;
                Ok(PlStmt::Perform(Box::new(q), self.span_from(start)))
            }
            "execute" => {
                self.pos += 1;
                let command = self.expr()?;
                let (mut into, mut strict) = self.read_into_targets()?;
                let mut using = Vec::new();
                if self.eat_kw("using") {
                    using.push(self.expr()?);
                    while self.eat(&Tok::Comma) {
                        using.push(self.expr()?);
                    }
                }
                if into.is_empty() {
                    (into, strict) = self.read_into_targets()?;
                }
                self.end_semi()?;
                Ok(PlStmt::Execute {
                    command,
                    into,
                    strict,
                    using,
                    span: self.span_from(start),
                })
            }
            "raise" => self.pl_raise(start),
            "assert" => {
                self.pos += 1;
                let cond = self.expr()?;
                let message = if self.eat(&Tok::Comma) {
                    Some(self.expr()?)
                } else {
                    None
                };
                self.end_semi()?;
                Ok(PlStmt::Assert {
                    cond,
                    message,
                    span: self.span_from(start),
                })
            }
            "get" => {
                self.pos += 1;
                let stacked = self.eat_kw("stacked");
                self.eat_kw("current");
                self.expect_kw("diagnostics")?;
                let mut items = Vec::new();
                loop {
                    let target = self.name()?;
                    if !(self.eat(&Tok::Assign) || self.eat_op("=")) {
                        return self.err("expected `=` or `:=` in get diagnostics");
                    }
                    let item = self.any_word()?;
                    items.push((target, item));
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.end_semi()?;
                Ok(PlStmt::GetDiagnostics {
                    stacked,
                    items,
                    span: self.span_from(start),
                })
            }
            "open" | "fetch" | "move" | "close" | "call" | "commit" | "rollback" => {
                self.pos += 1;
                let mut depth = 0i32;
                while let Some(t) = self.peek() {
                    match t {
                        Tok::LParen => depth += 1,
                        Tok::RParen => depth -= 1,
                        Tok::Semi if depth <= 0 => break,
                        _ => {}
                    }
                    self.pos += 1;
                }
                let text = self.src[start..self.prev_end().max(start)].to_string();
                self.end_semi()?;
                Ok(PlStmt::Other {
                    keyword: w,
                    text,
                    span: self.span_from(start),
                })
            }
            "select" | "with" | "insert" | "update" | "delete" | "values" => {
                let stmt = self.statement()?;
                let (into, strict) = match &stmt {
                    Stmt::Query(q) => match &q.body {
                        QueryBody::Select(s) if !s.into.is_empty() => {
                            (s.into.clone(), s.into_strict)
                        }
                        _ => self.read_into_targets()?,
                    },
                    _ => self.read_into_targets()?,
                };
                self.end_semi()?;
                Ok(PlStmt::Sql {
                    stmt: Box::new(stmt),
                    into,
                    strict,
                    span: self.span_from(start),
                })
            }
            _ if self.assignment_ahead() => self.pl_assignment(start),
            // Any other SQL command is a statement of its own in PL/pgSQL (`drop trigger …`,
            // `create table …`, `truncate …`, `lock …`).
            _ => {
                let stmt = self.statement()?;
                self.end_semi()?;
                Ok(PlStmt::Sql {
                    stmt: Box::new(stmt),
                    into: vec![],
                    strict: false,
                    span: self.span_from(start),
                })
            }
        }
    }

    /// A target (`x`, `r.f`, `a[i]`) followed by `:=` or `=`.
    fn assignment_ahead(&self) -> bool {
        let mut i = 0;
        let mut depth = 0i32;
        loop {
            match self.peek_n(i) {
                Some(Tok::Assign) if depth == 0 => return true,
                Some(Tok::Op(o)) if o == "=" && depth == 0 => return true,
                Some(Tok::Ident(_)) | Some(Tok::QuotedIdent(_)) | Some(Tok::Dot) => {}
                Some(Tok::LBracket) => depth += 1,
                Some(Tok::RBracket) => depth -= 1,
                Some(_) if depth > 0 => {}
                _ => return false,
            }
            i += 1;
            if i > 64 {
                return false;
            }
        }
    }

    /// `perform` takes a select list without the `select` keyword.
    fn perform_query(&mut self) -> PResult<Query> {
        // Re-read as `select …` by parsing the items and the rest of a select here.
        let start = self.here().start;
        let items = self.select_items()?;
        let mut q = Query {
            with: vec![],
            recursive: false,
            body: QueryBody::Select(Box::new(Select {
                distinct: false,
                distinct_on: vec![],
                items,
                into: vec![],
                into_strict: false,
                from: vec![],
                where_: None,
                group_by: vec![],
                group_distinct: false,
                having: None,
                windows: vec![],
                span: Span::default(),
            })),
            order_by: vec![],
            limit: None,
            with_ties: false,
            offset: None,
            locking: vec![],
            span: Span::default(),
        };
        if let QueryBody::Select(s) = &mut q.body {
            if self.eat_kw("from") {
                s.from = self.from_list()?;
            }
            if self.eat_kw("where") {
                s.where_ = Some(self.expr()?);
            }
            if self.eat_kws(&["group", "by"]) {
                s.group_by.push(self.expr()?);
                while self.eat(&Tok::Comma) {
                    s.group_by.push(self.expr()?);
                }
            }
            if self.eat_kw("having") {
                s.having = Some(self.expr()?);
            }
            s.span = self.span_from(start);
        }
        if self.eat_kws(&["order", "by"]) {
            q.order_by = self.order_list()?;
        }
        if self.eat_kw("limit") {
            q.limit = Some(self.expr()?);
        }
        q.span = self.span_from(start);
        Ok(q)
    }

    fn pl_assignment(&mut self, start: usize) -> PResult<PlStmt> {
        let target = self.assignment_target()?;
        if !(self.eat(&Tok::Assign) || self.eat_op("=")) {
            return self.err("expected a PL/pgSQL statement (an assignment needs `:=` or `=`)");
        }
        let value = self.expr()?;
        self.end_semi()?;
        Ok(PlStmt::Assign {
            target,
            value,
            span: self.span_from(start),
        })
    }

    /// `x`, `r.f`, `a[i]`, `a[i].f`.
    fn assignment_target(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        let mut e = Expr::Col(self.name()?, self.span_from(start));
        while self.is(&Tok::LBracket) {
            self.pos += 1;
            let i = self.expr()?;
            self.expect(&Tok::RBracket)?;
            e = Expr::Subscript(
                Box::new(e),
                Box::new(Some(i)),
                Box::new(None),
                false,
                self.span_from(start),
            );
            if self.eat(&Tok::Dot) {
                let f = self.any_word()?;
                e = Expr::Field(Box::new(e), f, self.span_from(start));
            }
        }
        Ok(e)
    }

    fn pl_if(&mut self, start: usize) -> PResult<PlStmt> {
        self.expect_kw("if")?;
        let mut branches = Vec::new();
        let c = self.expr()?;
        self.expect_kw("then")?;
        let b = self.pl_stmts(&["elsif", "elseif", "else", "end"])?;
        branches.push((c, b));
        let mut otherwise = None;
        loop {
            if self.eat_kw("elsif") || self.eat_kw("elseif") {
                let c = self.expr()?;
                self.expect_kw("then")?;
                let b = self.pl_stmts(&["elsif", "elseif", "else", "end"])?;
                branches.push((c, b));
            } else if self.eat_kw("else") {
                otherwise = Some(self.pl_stmts(&["end"])?);
            } else {
                break;
            }
        }
        self.expect_kw("end")?;
        self.expect_kw("if")?;
        self.end_semi()?;
        Ok(PlStmt::If {
            branches,
            otherwise,
            span: self.span_from(start),
        })
    }

    fn pl_case(&mut self, start: usize) -> PResult<PlStmt> {
        self.expect_kw("case")?;
        let operand = if self.is_kw("when") {
            None
        } else {
            Some(self.expr()?)
        };
        let mut whens = Vec::new();
        while self.eat_kw("when") {
            let mut vals = vec![self.expr()?];
            while self.eat(&Tok::Comma) {
                vals.push(self.expr()?);
            }
            self.expect_kw("then")?;
            let b = self.pl_stmts(&["when", "else", "end"])?;
            whens.push((vals, b));
        }
        let otherwise = if self.eat_kw("else") {
            Some(self.pl_stmts(&["end"])?)
        } else {
            None
        };
        self.expect_kw("end")?;
        self.expect_kw("case")?;
        self.end_semi()?;
        Ok(PlStmt::Case {
            operand,
            whens,
            otherwise,
            span: self.span_from(start),
        })
    }

    fn loop_targets(&mut self) -> PResult<Vec<Name>> {
        let mut v = vec![self.name()?];
        while self.eat(&Tok::Comma) {
            v.push(self.name()?);
        }
        Ok(v)
    }

    fn pl_loop(&mut self, label: Option<String>, start: usize) -> PResult<PlStmt> {
        let kind = if self.eat_kw("loop") {
            return self.loop_body(label, LoopKind::Plain, start, false);
        } else if self.eat_kw("while") {
            let c = self.expr()?;
            LoopKind::While(c)
        } else if self.eat_kw("foreach") {
            let vars = self.loop_targets()?;
            let slice = if self.eat_kw("slice") {
                Some(self.expr()?)
            } else {
                None
            };
            self.expect_kw("in")?;
            self.expect_kw("array")?;
            let array = self.expr()?;
            LoopKind::Foreach { vars, slice, array }
        } else if self.eat_kw("for") {
            let vars = self.loop_targets()?;
            self.expect_kw("in")?;
            if self.eat_kw("execute") {
                let command = self.expr()?;
                let mut using = Vec::new();
                if self.eat_kw("using") {
                    using.push(self.expr()?);
                    while self.eat(&Tok::Comma) {
                        using.push(self.expr()?);
                    }
                }
                LoopKind::ForExecute {
                    vars,
                    command,
                    using,
                }
            } else if self.starts_query() {
                LoopKind::ForQuery {
                    vars,
                    query: Box::new(self.query()?),
                }
            } else {
                let reverse = self.eat_kw("reverse");
                // A cursor loop: `for r in cur [(args)] loop`, told apart from an integer
                // range by the absence of `..` before `loop`.
                let save = self.pos;
                let mut depth = 0i32;
                let mut has_range = false;
                while let Some(t) = self.peek() {
                    match t {
                        Tok::LParen => depth += 1,
                        Tok::RParen => depth -= 1,
                        Tok::DotDot if depth == 0 => {
                            has_range = true;
                            break;
                        }
                        Tok::Ident(w) if w == "loop" && depth == 0 => break,
                        _ => {}
                    }
                    self.pos += 1;
                }
                self.pos = save;
                if has_range {
                    let lo = self.expr()?;
                    self.expect(&Tok::DotDot)?;
                    let hi = self.expr()?;
                    let by = if self.eat_kw("by") {
                        Some(self.expr()?)
                    } else {
                        None
                    };
                    LoopKind::ForInt {
                        var: vars[0].dotted(),
                        reverse,
                        lo,
                        hi,
                        by,
                    }
                } else {
                    let cursor = self.any_word()?;
                    let mut args = Vec::new();
                    if self.eat(&Tok::LParen) {
                        if !self.is(&Tok::RParen) {
                            args.push(self.expr()?);
                            while self.eat(&Tok::Comma) {
                                args.push(self.expr()?);
                            }
                        }
                        self.expect(&Tok::RParen)?;
                    }
                    LoopKind::ForCursor {
                        var: vars[0].dotted(),
                        cursor,
                        args,
                    }
                }
            }
        } else {
            return self.err("expected loop, while, for or foreach");
        };
        self.expect_kw("loop")?;
        self.loop_body(label, kind, start, true)
    }

    fn loop_body(
        &mut self,
        label: Option<String>,
        kind: LoopKind,
        start: usize,
        _pre: bool,
    ) -> PResult<PlStmt> {
        let body = self.pl_stmts(&["end"])?;
        self.expect_kw("end")?;
        self.expect_kw("loop")?;
        if let Some(l) = &label {
            if self.is_kw(l) {
                self.pos += 1;
            }
        }
        self.end_semi()?;
        Ok(PlStmt::Loop {
            label,
            kind: Box::new(kind),
            body,
            span: self.span_from(start),
        })
    }

    fn pl_raise(&mut self, start: usize) -> PResult<PlStmt> {
        self.expect_kw("raise")?;
        let level = ["debug", "log", "info", "notice", "warning", "exception"]
            .iter()
            .find(|l| self.is_kw(l))
            .map(|l| l.to_string());
        if level.is_some() {
            self.pos += 1;
        }
        let mut message = None;
        let mut args = Vec::new();
        if let Some(Tok::Str(s)) = self.peek().cloned() {
            self.pos += 1;
            message = Some(s);
            while self.eat(&Tok::Comma) {
                args.push(self.expr()?);
            }
        } else if self.eat_kw("sqlstate") {
            if let Some(Tok::Str(s)) = self.peek().cloned() {
                self.pos += 1;
                message = Some(format!("sqlstate {s}"));
            }
        } else if self.word().is_some_and(|w| w != "using") {
            message = Some(self.any_word()?);
        }
        let mut options = Vec::new();
        if self.eat_kw("using") {
            loop {
                let k = self.any_word()?;
                if !(self.eat_op("=") || self.eat(&Tok::Assign)) {
                    return self.err("expected `=` in raise … using");
                }
                options.push((k, self.expr()?));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }
        self.end_semi()?;
        Ok(PlStmt::Raise {
            level,
            message,
            args,
            options,
            span: self.span_from(start),
        })
    }
}
