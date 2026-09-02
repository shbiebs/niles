//! The Niles parser: hand-written recursive descent with a Pratt loop for expressions.
//!
//! # Why hand-written
//!
//! Three reasons, in order of weight.
//!
//! 1. **Reserved-word count is a function of parser technology, not of vocabulary size.**
//!    SQL reserves heavily in part because an LALR(1) grammar cannot resolve
//!    context-dependent identifier/keyword ambiguity, so words get reserved to remove
//!    conflicts. Niles has a large novel vocabulary — `epoch`, `ledger`, `serve`, `posted`,
//!    `valid` — and must be adoptable against bank schemas whose columns already use those
//!    words. Unbounded lookahead and local backtracking are what buy a 37-word reserved set
//!    against a 173-word vocabulary.
//! 2. **Resilience.** A generated LR parser's error recovery is a bolt-on; a hand-written
//!    one can decide, per construct, what a plausible continuation is. Every `parse_*`
//!    here returns a node — an `Error` node if it must — so a file with one bad
//!    declaration still yields a tree for the rest of it.
//! 3. **Diagnostics.** The parser knows what it was in the middle of, and can say so.
//!
//! # Resilience discipline
//!
//! Two rules, and they are what keep recovery from looping. Every recovery point names a
//! *synchronising set* — tokens at which the parser can restart with confidence — and
//! [`Parser::recover_to`] always consumes at least one token. Delimiters are tracked so
//! that a missing `}` inside a schema does not swallow the rest of the file.

use crate::ast::*;
use crate::diagnostics::{Applicability, Diagnostic, Diagnostics};
use crate::keywords::{self, Kw};
use crate::lexer::{lex, Span, Tok, Token};

pub struct Parser<'a> {
    toks: Vec<Token>,
    pos: usize,
    src: &'a str,
    pub diags: Diagnostics,
    /// Guards against a recovery loop: if the parser makes no progress this many times in
    /// a row it emits one diagnostic and gives up on the construct rather than spinning.
    fuel: u32,
    /// Non-zero while parsing the head of `if`/`while`/`match`/`for`, where a `{`
    /// opens the body rather than a struct literal. The classic Rust ambiguity, solved
    /// the way Rust solves it: a flag, not a grammar change.
    no_struct_depth: u32,
    /// Non-zero while parsing a SQL projection or table reference, where a trailing `as`
    /// introduces an *alias*, not a cast. Both readings are legal Niles elsewhere, and the
    /// ambiguity is genuinely local: only SQL clause position disambiguates them.
    no_alias_depth: u32,
    /// Non-zero while parsing inside a SQL statement, where **`=` is equality**.
    ///
    /// Niles has two ancestries and they disagree about one character. In Rust `a = b` is
    /// an assignment; in SQL's `where` clause it is a comparison, and SQL has no
    /// assignment expression at all. Before this counter existed, `select k from t where
    /// t.z = 1` parsed as an `Assign`, lowering had no case for it, and the predicate
    /// became `LitBool(true)` — a `where` clause silently discarded, returning every row
    /// from a query that looked correct. That is the same shape as the `Err(_) => 0`
    /// defect of §1.1.1, one level up, and it is the reason both this counter and the
    /// diagnostic that replaced the `true` fallback exist.
    sql_depth: u32,
}

/// Parse a whole file. Always returns a program; the diagnostics say whether it is sound.
pub fn parse_program(src: &str) -> (Program, Diagnostics) {
    let (toks, lex_errors) = lex(src);
    let mut p = Parser {
        toks,
        pos: 0,
        src,
        diags: Diagnostics::new(),
        fuel: 100_000,
        no_struct_depth: 0,
        no_alias_depth: 0,
        sql_depth: 0,
    };
    for e in lex_errors {
        p.diags
            .push(Diagnostic::error(e.code, e.msg).primary(e.span, "here"));
    }
    let prog = p.program();
    (prog, p.diags)
}

/// Parse a single expression, for tests and for the REPL.
pub fn parse_expr(src: &str) -> (Expr, Diagnostics) {
    let (toks, _) = lex(src);
    let mut p = Parser {
        toks,
        pos: 0,
        src,
        diags: Diagnostics::new(),
        fuel: 100_000,
        no_struct_depth: 0,
        no_alias_depth: 0,
        sql_depth: 0,
    };
    let e = p.expr();
    (e, p.diags)
}

impl<'a> Parser<'a> {
    // ---------------- token plumbing ----------------

    fn cur(&self) -> &Tok {
        &self.toks[self.pos.min(self.toks.len() - 1)].tok
    }
    fn cur_span(&self) -> Span {
        self.toks[self.pos.min(self.toks.len() - 1)].span
    }
    fn nth(&self, n: usize) -> &Tok {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)].tok
    }
    fn at_eof(&self) -> bool {
        matches!(self.cur(), Tok::Eof)
    }
    fn bump(&mut self) -> Span {
        let s = self.cur_span();
        if !self.at_eof() {
            self.pos += 1;
        }
        s
    }
    fn at(&self, t: &Tok) -> bool {
        self.cur() == t
    }
    fn at_kw(&self, k: Kw) -> bool {
        matches!(self.cur(), Tok::Kw(x) if *x == k)
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.at(t) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_kw(&mut self, k: Kw) -> bool {
        if self.at_kw(k) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn text(&self, s: Span) -> &str {
        s.text(self.src)
    }

    fn err(&mut self, code: &'static str, msg: impl Into<String>, label: impl Into<String>) {
        let span = self.cur_span();
        self.diags
            .push(Diagnostic::error(code, msg).primary(span, label));
    }

    /// Expect a token. On failure, report and do **not** consume, so the caller's recovery
    /// set still sees the offending token.
    fn expect(&mut self, t: Tok, ctx: &str) -> Option<Span> {
        if self.at(&t) {
            Some(self.bump())
        } else {
            let found = self.describe_cur();
            self.err(
                "NL0001",
                format!("expected {} {ctx}, found {found}", describe(&t)),
                format!("expected {}", describe(&t)),
            );
            None
        }
    }

    fn expect_kw(&mut self, k: Kw, ctx: &str) -> Option<Span> {
        self.expect(Tok::Kw(k), ctx)
    }

    fn describe_cur(&self) -> String {
        match self.cur() {
            Tok::Ident => format!("identifier `{}`", self.text(self.cur_span())),
            Tok::Eof => "end of file".into(),
            t => describe(t),
        }
    }

    /// An identifier, accepting any keyword whose registry category permits it. This is
    /// where the reservation policy actually pays: `select epoch from t` works because
    /// `epoch` is `Unreserved`, and it works without a special case in the grammar.
    fn ident(&mut self, ctx: &str) -> Name {
        match self.cur().clone() {
            Tok::Ident => {
                let s = self.bump();
                Name::new(self.text(s).trim_start_matches("r#"), s)
            }
            Tok::Kw(k) => {
                let word = keywords::KEYWORDS.iter().find(|kw| kw.token == k);
                if word.is_some_and(|w| w.category.usable_as_ident()) {
                    let s = self.bump();
                    Name::new(self.text(s), s)
                } else {
                    let span = self.cur_span();
                    let w = word.map(|w| w.word).unwrap_or("?");
                    self.diags.push(
                        Diagnostic::error(
                            "NL0002",
                            format!("`{w}` is a reserved word and cannot be used as {ctx}"),
                        )
                        .primary(span, "reserved word")
                        .note(format!(
                            "`{w}` is {}",
                            word.map(|x| x.category.as_str()).unwrap_or("reserved")
                        ))
                        .suggest(
                            span,
                            format!("r#{w}"),
                            "escape it to use it as an identifier",
                            Applicability::MachineApplicable,
                        ),
                    );
                    self.bump();
                    Name::new(w, span)
                }
            }
            _ => {
                let found = self.describe_cur();
                let span = self.cur_span();
                self.err(
                    "NL0003",
                    format!("expected {ctx}, found {found}"),
                    "expected a name",
                );
                Name::new("<error>", span)
            }
        }
    }

    /// A name in a position where *any* word is unambiguous: after `.`, after `|>`, and
    /// as a struct-literal field. Reservation exists to resolve grammar ambiguity, and
    /// there is none here — `p.where(..)`, `p.select`, `p.order` and `p.check` all have
    /// exactly one reading. Refusing them would reserve words for no grammatical benefit,
    /// which is precisely the failure mode the registry's policy note warns against.
    fn member_name(&mut self, ctx: &str) -> Name {
        match self.cur().clone() {
            Tok::Ident => {
                let s = self.bump();
                Name::new(self.text(s).trim_start_matches("r#"), s)
            }
            Tok::Kw(k) => {
                let w = keywords::KEYWORDS
                    .iter()
                    .find(|kw| kw.token == k)
                    .map(|w| w.word)
                    .unwrap_or("?");
                let s = self.bump();
                Name::new(w, s)
            }
            Tok::Int(n) => {
                // `t.0` — tuple field access.
                let s = self.bump();
                Name::new(n.to_string(), s)
            }
            _ => self.ident(ctx),
        }
    }

    /// Skip tokens until one of `sync` (or EOF), consuming at least one so recovery
    /// terminates. Returns the span covering everything skipped.
    fn recover_to(&mut self, sync: &[Tok]) -> Span {
        let start = self.cur_span();
        let mut end = start;
        let mut depth = 0i32;
        let mut first = true;
        while !self.at_eof() {
            match self.cur() {
                Tok::LBrace | Tok::LParen | Tok::LBracket => depth += 1,
                Tok::RBrace | Tok::RParen | Tok::RBracket => {
                    if depth == 0 && sync.contains(self.cur()) && !first {
                        break;
                    }
                    depth -= 1;
                }
                t if depth == 0 && sync.contains(t) && !first => break,
                _ => {}
            }
            end = self.bump();
            first = false;
            if depth < 0 {
                break;
            }
        }
        start.to(end)
    }

    fn burn(&mut self) -> bool {
        self.fuel = self.fuel.saturating_sub(1);
        self.fuel == 0
    }

    // ---------------- program and items ----------------

    fn program(&mut self) -> Program {
        let start = self.cur_span();
        let mut items = Vec::new();
        let mut last = self.pos;
        while !self.at_eof() {
            if self.burn() {
                break;
            }
            items.push(self.item());
            if self.pos == last {
                // No progress: force one token so the loop terminates.
                self.bump();
            }
            last = self.pos;
        }
        let end = self.cur_span();
        Program {
            items,
            span: start.to(end),
        }
    }

    /// The item-level synchronising set: the keywords that can begin an item. Recovery
    /// stops here because these are the points at which the parser is certain again.
    fn item_sync() -> Vec<Tok> {
        [
            Kw::Schema,
            Kw::Fn,
            Kw::Struct,
            Kw::Enum,
            Kw::Trait,
            Kw::Impl,
            Kw::Mod,
            Kw::Use,
            Kw::Const,
            Kw::Static,
            Kw::Type,
            Kw::View,
            Kw::Pub,
            Kw::Capability,
        ]
        .into_iter()
        .map(Tok::Kw)
        .chain([Tok::RBrace, Tok::Semi])
        .collect()
    }

    fn item(&mut self) -> Item {
        let attrs = self.attrs();
        let start = self.cur_span();
        let public = self.eat_kw(Kw::Pub);
        // `create` is optional noise from SQL: `create view v = ..` and `view v = ..` are
        // the same declaration, which is how the SQL surface and the pipeline surface stay
        // one language rather than two.
        self.eat_kw(Kw::Create);
        match self.cur().clone() {
            Tok::Kw(Kw::Schema) => Item::Schema(self.schema_decl(attrs, start)),
            Tok::Kw(Kw::Fn) => Item::Fn(self.fn_decl(attrs, public, start)),
            Tok::Kw(Kw::Struct) => Item::Struct(self.struct_decl(attrs, public, start)),
            Tok::Kw(Kw::Enum) => Item::Enum(self.enum_decl(public, start)),
            Tok::Kw(Kw::Trait) => Item::Trait(self.trait_decl(start)),
            Tok::Kw(Kw::Impl) => Item::Impl(self.impl_decl(start)),
            Tok::Kw(Kw::View) => Item::View(self.view_decl(attrs, public, start)),
            Tok::Kw(Kw::Capability) => {
                self.bump();
                let name = self.ident("a capability name");
                self.expect(Tok::Colon, "in a capability declaration");
                let ty = self.ty();
                self.eat(&Tok::Semi);
                Item::Capability {
                    name,
                    ty,
                    span: start.to(self.cur_span()),
                }
            }
            Tok::Kw(Kw::Mod) => {
                self.bump();
                let name = self.ident("a module name");
                self.expect(Tok::LBrace, "to open a module");
                let mut items = Vec::new();
                while !self.at(&Tok::RBrace) && !self.at_eof() {
                    if self.burn() {
                        break;
                    }
                    let before = self.pos;
                    items.push(self.item());
                    if self.pos == before {
                        self.bump();
                    }
                }
                let end = self
                    .expect(Tok::RBrace, "to close a module")
                    .unwrap_or(self.cur_span());
                Item::Mod {
                    name,
                    items,
                    span: start.to(end),
                }
            }
            Tok::Kw(Kw::Use) => {
                self.bump();
                let path = self.path();
                let end = self.expect(Tok::Semi, "after a `use`").unwrap_or(path.span);
                Item::Use {
                    path,
                    span: start.to(end),
                }
            }
            Tok::Kw(k @ (Kw::Const | Kw::Static)) => {
                self.bump();
                let name = self.ident("a constant name");
                self.expect(Tok::Colon, "in a constant declaration");
                let ty = self.ty();
                self.expect(Tok::Eq, "in a constant declaration");
                let value = self.expr();
                let end = self
                    .expect(Tok::Semi, "after a constant")
                    .unwrap_or(value.span());
                Item::Const {
                    name,
                    ty,
                    value,
                    is_static: k == Kw::Static,
                    span: start.to(end),
                }
            }
            Tok::Kw(Kw::Type) => {
                self.bump();
                let name = self.ident("a type name");
                self.expect(Tok::Eq, "in a type alias");
                let ty = self.ty();
                let end = self
                    .expect(Tok::Semi, "after a type alias")
                    .unwrap_or(ty.span());
                Item::TypeAlias {
                    name,
                    ty,
                    span: start.to(end),
                }
            }
            _ => {
                let found = self.describe_cur();
                self.diags.push(
                    Diagnostic::error("NL0004", format!("expected an item, found {found}"))
                        .primary(start, "not the start of an item")
                        .note("items are: schema, fn, struct, enum, trait, impl, mod, use, const, static, type, view, capability"),
                );
                let span = self.recover_to(&Self::item_sync());
                Item::Error(start.to(span))
            }
        }
    }

    fn attrs(&mut self) -> Vec<Attr> {
        let mut out = Vec::new();
        loop {
            if self.at(&Tok::HashBracket) {
                let start = self.bump();
                let name = self.ident("an attribute name");
                let args = self.attr_args();
                let end = self
                    .expect(Tok::RBracket, "to close an attribute")
                    .unwrap_or(name.span);
                out.push(Attr {
                    name,
                    args,
                    at_style: false,
                    span: start.to(end),
                });
            } else if self.at(&Tok::At) {
                let start = self.bump();
                let name = self.ident("an attribute name");
                let args = self.attr_args();
                let end = args.last().map(|_| self.cur_span()).unwrap_or(name.span);
                out.push(Attr {
                    name,
                    args,
                    at_style: true,
                    span: start.to(end),
                });
            } else {
                return out;
            }
        }
    }

    fn attr_args(&mut self) -> Vec<AttrArg> {
        let mut args = Vec::new();
        if self.eat(&Tok::LParen) {
            while !self.at(&Tok::RParen) && !self.at_eof() {
                if self.burn() {
                    break;
                }
                let n = self.ident("an attribute argument");
                if self.eat(&Tok::Eq) || self.eat(&Tok::Colon) {
                    args.push(AttrArg::KeyValue(n, self.expr()));
                } else {
                    args.push(AttrArg::Word(n));
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::RParen, "to close attribute arguments");
        }
        args
    }

    // ---------------- schema ----------------

    fn schema_decl(&mut self, attrs: Vec<Attr>, start: Span) -> SchemaDecl {
        self.bump(); // `schema`
        let name = self.ident("a schema name");
        self.expect(Tok::LBrace, "to open a schema");
        let mut items = Vec::new();
        while !self.at(&Tok::RBrace) && !self.at_eof() {
            if self.burn() {
                break;
            }
            let before = self.pos;
            items.push(self.schema_item());
            if self.pos == before {
                self.bump();
            }
        }
        let end = self
            .expect(Tok::RBrace, "to close a schema")
            .unwrap_or(self.cur_span());
        SchemaDecl {
            name,
            items,
            attrs,
            span: start.to(end),
        }
    }

    fn schema_item(&mut self) -> SchemaItem {
        let attrs = self.attrs();
        let start = self.cur_span();
        let public = self.eat_kw(Kw::Pub);
        self.eat_kw(Kw::Create);
        match self.cur().clone() {
            Tok::Kw(Kw::Currency) => {
                self.bump();
                let name = self.ident("a currency code");
                let mut scale = 2u32;
                let mut scale_span = name.span;
                if self.eat(&Tok::LBrace) {
                    // `currency jpy { scale: 0 }`
                    while !self.at(&Tok::RBrace) && !self.at_eof() {
                        let key = self.ident("a currency property");
                        self.expect(Tok::Colon, "in a currency declaration");
                        let v = self.expr();
                        if key.text == "scale" {
                            scale_span = v.span();
                            match &v {
                                Expr::Int(n, _) if (0..=6).contains(n) => scale = *n as u32,
                                Expr::Int(n, s) => self.diags.push(
                                    Diagnostic::error("NL0010", format!("currency scale {n} is out of range"))
                                        .primary(*s, "scale must be 0..=6")
                                        .note("ISO 4217 minor-unit exponents are 0, 2 or 3; the range is widened to 6 for internal precision"),
                                ),
                                other => self.diags.push(
                                    Diagnostic::error("NL0011", "currency scale must be an integer literal")
                                        .primary(other.span(), "not an integer"),
                                ),
                            }
                        }
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(Tok::RBrace, "to close a currency declaration");
                }
                self.eat(&Tok::Semi);
                SchemaItem::Currency(CurrencyDecl {
                    name,
                    scale,
                    scale_span,
                    span: start.to(self.cur_span()),
                })
            }
            Tok::Kw(k @ (Kw::Table | Kw::Base | Kw::Ledger)) => {
                let kind = match k {
                    Kw::Table => RelKind::Table,
                    Kw::Base => RelKind::Base,
                    _ => RelKind::Ledger,
                };
                SchemaItem::Base(self.rel_decl(kind, attrs, start))
            }
            Tok::Kw(Kw::View) => SchemaItem::View(self.view_decl(attrs, public, start)),
            Tok::Kw(Kw::Index) => {
                self.bump();
                let name = self.ident("an index name");
                self.expect_kw(Kw::On, "in an index declaration");
                let on = self.ident("the indexed relation");
                self.expect(Tok::LParen, "to open the index columns");
                let mut cols = Vec::new();
                while !self.at(&Tok::RParen) && !self.at_eof() {
                    cols.push(self.ident("an index column"));
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(Tok::RParen, "to close the index columns");
                let anchor = self.eat_kw(Kw::Anchor);
                let end = self
                    .expect(Tok::Semi, "after an index")
                    .unwrap_or(self.cur_span());
                SchemaItem::Index(IndexDecl {
                    name,
                    on,
                    cols,
                    anchor,
                    span: start.to(end),
                })
            }
            _ => {
                let found = self.describe_cur();
                self.diags.push(
                    Diagnostic::error("NL0005", format!("expected a schema item, found {found}"))
                        .primary(start, "not a schema item")
                        .note("schema items are: currency, table, base, ledger, view, index"),
                );
                let span = self.recover_to(&[
                    Tok::Kw(Kw::Currency),
                    Tok::Kw(Kw::Table),
                    Tok::Kw(Kw::Base),
                    Tok::Kw(Kw::Ledger),
                    Tok::Kw(Kw::View),
                    Tok::Kw(Kw::Index),
                    Tok::RBrace,
                ]);
                SchemaItem::Error(start.to(span))
            }
        }
    }

    fn rel_decl(&mut self, kind: RelKind, attrs: Vec<Attr>, start: Span) -> RelDecl {
        self.bump(); // table / base / ledger
        let name = self.ident("a relation name");
        self.expect(Tok::LBrace, "to open a relation body");
        let mut fields = Vec::new();
        let mut rules = Vec::new();
        while !self.at(&Tok::RBrace) && !self.at_eof() {
            if self.burn() {
                break;
            }
            let before = self.pos;
            match self.cur().clone() {
                Tok::Kw(Kw::Conserve) => {
                    let s = self.bump();
                    self.expect_kw(Kw::Per, "in a conservation rule");
                    self.expect(Tok::LParen, "to open the conservation key");
                    let mut keys = Vec::new();
                    while !self.at(&Tok::RParen) && !self.at_eof() {
                        keys.push(self.ident("a conservation key column"));
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(Tok::RParen, "to close the conservation key");
                    let end = self
                        .expect(Tok::Semi, "after a conservation rule")
                        .unwrap_or(s);
                    rules.push(RelRule::Conserve {
                        keys,
                        span: s.to(end),
                    });
                }
                Tok::Kw(Kw::Retain) => {
                    let s = self.bump();
                    let mode = self.ident("a retention mode");
                    let end = self
                        .expect(Tok::Semi, "after a retention clause")
                        .unwrap_or(s);
                    rules.push(RelRule::Retain {
                        mode,
                        span: s.to(end),
                    });
                }
                Tok::Kw(Kw::Bitemporal) => {
                    let s = self.bump();
                    let end = self.expect(Tok::Semi, "after `bitemporal`").unwrap_or(s);
                    rules.push(RelRule::Bitemporal { span: s.to(end) });
                }
                Tok::Kw(Kw::Foreign) => {
                    let s = self.bump();
                    self.expect_kw(Kw::Key, "in a foreign key");
                    let cols = self.paren_names();
                    self.expect_kw(Kw::References, "in a foreign key");
                    let target = self.ident("the referenced relation");
                    let target_cols = self.paren_names();
                    let end = self.expect(Tok::Semi, "after a foreign key").unwrap_or(s);
                    rules.push(RelRule::ForeignKey {
                        cols,
                        target,
                        target_cols,
                        span: s.to(end),
                    });
                }
                Tok::Kw(Kw::Primary) if matches!(self.nth(2), Tok::LParen) => {
                    let s = self.bump();
                    self.expect_kw(Kw::Key, "in a primary key");
                    let cols = self.paren_names();
                    let end = self.expect(Tok::Semi, "after a primary key").unwrap_or(s);
                    rules.push(RelRule::PrimaryKey {
                        cols,
                        span: s.to(end),
                    });
                }
                _ => fields.push(self.field_decl()),
            }
            if self.pos == before {
                self.bump();
            }
            self.eat(&Tok::Comma);
        }
        let mut end = self
            .expect(Tok::RBrace, "to close a relation body")
            .unwrap_or(self.cur_span());
        // Trailing clauses after the body: `ledger p { .. } retain forever;`
        loop {
            match self.cur().clone() {
                Tok::Kw(Kw::Retain) => {
                    let s = self.bump();
                    let mode = self.ident("a retention mode");
                    end = self
                        .expect(Tok::Semi, "after a retention clause")
                        .unwrap_or(s);
                    rules.push(RelRule::Retain {
                        mode,
                        span: s.to(end),
                    });
                }
                Tok::Kw(Kw::Bitemporal) => {
                    let s = self.bump();
                    end = self.expect(Tok::Semi, "after `bitemporal`").unwrap_or(s);
                    rules.push(RelRule::Bitemporal { span: s.to(end) });
                }
                Tok::Semi => {
                    end = self.bump();
                }
                _ => break,
            }
        }
        RelDecl {
            kind,
            name,
            fields,
            rules,
            attrs,
            span: start.to(end),
        }
    }

    fn paren_names(&mut self) -> Vec<Name> {
        let mut out = Vec::new();
        if self.expect(Tok::LParen, "to open a column list").is_some() {
            while !self.at(&Tok::RParen) && !self.at_eof() {
                out.push(self.ident("a column name"));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::RParen, "to close a column list");
        }
        out
    }

    fn field_decl(&mut self) -> FieldDecl {
        let attrs = self.attrs();
        let start = self.cur_span();
        let name = self.ident("a field name");
        self.expect(Tok::Colon, "between a field name and its type");
        let ty = self.ty();
        let mut f = FieldDecl {
            name,
            ty,
            primary_key: false,
            unique: false,
            default: None,
            check: None,
            attrs: attrs.clone(),
            span: start,
        };
        // Trailing column constraints, in any order.
        loop {
            match self.cur().clone() {
                Tok::Kw(Kw::Primary) => {
                    self.bump();
                    self.expect_kw(Kw::Key, "after `primary`");
                    f.primary_key = true;
                }
                Tok::Kw(Kw::Unique) => {
                    self.bump();
                    f.unique = true;
                }
                Tok::Kw(Kw::Default) => {
                    self.bump();
                    f.default = Some(self.expr());
                }
                Tok::Kw(Kw::Check) => {
                    self.bump();
                    self.expect(Tok::LParen, "to open a check constraint");
                    f.check = Some(self.expr());
                    self.expect(Tok::RParen, "to close a check constraint");
                }
                // `idem: IdemKey window 30.days` — the window belongs to the column,
                // because an idempotency key without a window is not idempotent, it is
                // merely unique, and the difference is a business rule.
                Tok::Kw(Kw::Window) => {
                    self.bump();
                    f.default = Some(self.expr());
                }
                Tok::At | Tok::HashBracket => f.attrs.extend(self.attrs()),
                _ => break,
            }
        }
        f.span = start.to(self.cur_span());
        f
    }

    // ---------------- views and contracts ----------------

    fn view_decl(&mut self, attrs: Vec<Attr>, public: bool, start: Span) -> ViewDecl {
        self.bump(); // `view`
        let name = self.ident("a view name");
        self.expect(Tok::Eq, "in a view declaration");
        let body = self.expr();
        let contract = if self.at_kw(Kw::Serve) {
            Some(self.serve_contract())
        } else {
            None
        };
        let end = self
            .expect(Tok::Semi, "after a view declaration")
            .unwrap_or(body.span());
        if contract.is_none() {
            self.diags.push(
                Diagnostic::warning("NL0100", format!("view `{}` has no serve contract", name.text))
                    .primary(name.span, "no `serve { .. }`")
                    .note("without a contract the view is served at the schema default rung; declaring it makes the guarantee reviewable")
                    .suggest(end, " serve { consistency: snapshot, materialize: auto }", "add a contract", Applicability::MaybeIncorrect),
            );
        }
        ViewDecl {
            name,
            body,
            contract,
            public,
            attrs,
            span: start.to(end),
        }
    }

    fn serve_contract(&mut self) -> ServeContract {
        let start = self.bump(); // `serve`
        self.expect(Tok::LBrace, "to open a serve contract");
        let mut entries = Vec::new();
        while !self.at(&Tok::RBrace) && !self.at_eof() {
            if self.burn() {
                break;
            }
            // Contract keys and values are in a position where any word is unambiguous:
            // a `serve { .. }` block has exactly one reading, so reserving a word buys
            // nothing here. Without this, `lineage: full` did not parse, because `full` is
            // reserved for `full outer join` — a restriction with no grammatical
            // justification in this position.
            let key = self.member_name("a contract key");
            self.expect(Tok::Colon, "in a serve contract");
            let value = self.contract_value();
            entries.push((key, value));
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        let end = self
            .expect(Tok::RBrace, "to close a serve contract")
            .unwrap_or(start);
        ServeContract {
            entries,
            span: start.to(end),
        }
    }

    fn contract_value(&mut self) -> ContractValue {
        match self.cur().clone() {
            Tok::Int(n) => ContractValue::Int(n, self.bump()),
            Tok::Duration { value, unit } => {
                let s = self.bump();
                ContractValue::Duration {
                    value,
                    unit,
                    span: s,
                }
            }
            Tok::Ident | Tok::Kw(_) => {
                let name = self.member_name("a contract value");
                if self.at(&Tok::LParen) {
                    let start = self.bump();
                    let mut args = Vec::new();
                    while !self.at(&Tok::RParen) && !self.at_eof() {
                        let label = if matches!(self.nth(1), Tok::Colon) {
                            let n = self.member_name("a contract argument name");
                            self.bump();
                            Some(n)
                        } else {
                            None
                        };
                        args.push((label, self.contract_value()));
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    let end = self
                        .expect(Tok::RParen, "to close contract arguments")
                        .unwrap_or(start);
                    ContractValue::Call {
                        name,
                        args,
                        span: start.to(end),
                    }
                } else {
                    ContractValue::Word(name)
                }
            }
            _ => {
                let found = self.describe_cur();
                self.err(
                    "NL0006",
                    format!("expected a contract value, found {found}"),
                    "here",
                );
                ContractValue::Error(self.bump())
            }
        }
    }

    // ---------------- functions, structs, traits ----------------

    fn fn_decl(&mut self, attrs: Vec<Attr>, public: bool, start: Span) -> FnDecl {
        self.bump(); // `fn`
        let name = self.ident("a function name");
        let generics = self.generics();
        let mut params = Vec::new();
        if self
            .expect(Tok::LParen, "to open the parameter list")
            .is_some()
        {
            while !self.at(&Tok::RParen) && !self.at_eof() {
                if self.burn() {
                    break;
                }
                let p_start = self.cur_span();
                // `&self` / `self` receiver.
                if self.at(&Tok::Amp) && matches!(self.nth(1), Tok::Kw(Kw::SelfValue)) {
                    self.bump();
                    let s = self.bump();
                    params.push(Param {
                        pat: Pat::Bind {
                            name: Name::new("self", s),
                            mutable: false,
                            by_ref: true,
                            span: s,
                        },
                        ty: Ty::Infer(s),
                        span: s,
                    });
                } else if self.at_kw(Kw::SelfValue) {
                    let s = self.bump();
                    params.push(Param {
                        pat: Pat::Bind {
                            name: Name::new("self", s),
                            mutable: false,
                            by_ref: false,
                            span: s,
                        },
                        ty: Ty::Infer(s),
                        span: s,
                    });
                } else {
                    let pat = self.pat();
                    self.expect(Tok::Colon, "between a parameter and its type");
                    let ty = self.ty();
                    params.push(Param {
                        pat,
                        ty,
                        span: p_start.to(self.cur_span()),
                    });
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::RParen, "to close the parameter list");
        }
        let ret = if self.eat(&Tok::Arrow) {
            Some(self.ty())
        } else {
            None
        };
        let effects = if self.at(&Tok::Bang) {
            Some(self.effect_row())
        } else {
            None
        };
        let body = if self.at(&Tok::LBrace) {
            Some(self.block())
        } else {
            self.expect(Tok::Semi, "after a function signature");
            None
        };
        let end = body.as_ref().map(|b| b.span).unwrap_or(self.cur_span());
        FnDecl {
            name,
            generics,
            params,
            ret,
            effects,
            body,
            public,
            attrs,
            span: start.to(end),
        }
    }

    /// `! { read@snapshot, append, debit<usd> }`
    fn effect_row(&mut self) -> EffectRow {
        let start = self.bump(); // `!`
        let mut effects = Vec::new();
        if self.expect(Tok::LBrace, "to open an effect row").is_some() {
            while !self.at(&Tok::RBrace) && !self.at_eof() {
                if self.burn() {
                    break;
                }
                effects.push(self.effect());
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::RBrace, "to close an effect row");
        }
        EffectRow {
            effects,
            span: start.to(self.cur_span()),
        }
    }

    fn effect(&mut self) -> Effect {
        let start = self.cur_span();
        let name = self.ident("an effect name");
        let at = if self.eat(&Tok::At) {
            Some(self.ident("a consistency rung"))
        } else {
            None
        };
        let mut args = Vec::new();
        if self.at(&Tok::Lt) {
            self.bump();
            while !self.at(&Tok::Gt) && !self.at_eof() {
                args.push(self.ident("an effect parameter"));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::Gt, "to close effect parameters");
        }
        Effect {
            name,
            at,
            args,
            span: start.to(self.cur_span()),
        }
    }

    fn generics(&mut self) -> Vec<Name> {
        let mut out = Vec::new();
        if self.at(&Tok::Lt) {
            self.bump();
            while !self.at(&Tok::Gt) && !self.at_eof() {
                out.push(self.ident("a type parameter"));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::Gt, "to close type parameters");
        }
        out
    }

    fn struct_decl(&mut self, attrs: Vec<Attr>, public: bool, start: Span) -> StructDecl {
        self.bump();
        let name = self.ident("a struct name");
        let generics = self.generics();
        let mut fields = Vec::new();
        if self.expect(Tok::LBrace, "to open a struct body").is_some() {
            while !self.at(&Tok::RBrace) && !self.at_eof() {
                if self.burn() {
                    break;
                }
                let before = self.pos;
                fields.push(self.field_decl());
                if self.pos == before {
                    self.bump();
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::RBrace, "to close a struct body");
        }
        StructDecl {
            name,
            generics,
            fields,
            public,
            attrs,
            span: start.to(self.cur_span()),
        }
    }

    fn enum_decl(&mut self, public: bool, start: Span) -> EnumDecl {
        self.bump();
        let name = self.ident("an enum name");
        let generics = self.generics();
        let mut variants = Vec::new();
        if self.expect(Tok::LBrace, "to open an enum body").is_some() {
            while !self.at(&Tok::RBrace) && !self.at_eof() {
                if self.burn() {
                    break;
                }
                let vname = self.ident("a variant name");
                let mut tys = Vec::new();
                if self.eat(&Tok::LParen) {
                    while !self.at(&Tok::RParen) && !self.at_eof() {
                        tys.push(self.ty());
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(Tok::RParen, "to close variant fields");
                }
                variants.push((vname, tys));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::RBrace, "to close an enum body");
        }
        EnumDecl {
            name,
            generics,
            variants,
            public,
            span: start.to(self.cur_span()),
        }
    }

    fn trait_decl(&mut self, start: Span) -> TraitDecl {
        self.bump();
        let name = self.ident("a trait name");
        let generics = self.generics();
        let mut items = Vec::new();
        if self.expect(Tok::LBrace, "to open a trait body").is_some() {
            while !self.at(&Tok::RBrace) && !self.at_eof() {
                if self.burn() {
                    break;
                }
                let a = self.attrs();
                let s = self.cur_span();
                if self.at_kw(Kw::Fn) {
                    items.push(self.fn_decl(a, false, s));
                } else {
                    self.recover_to(&[Tok::Kw(Kw::Fn), Tok::RBrace]);
                }
            }
            self.expect(Tok::RBrace, "to close a trait body");
        }
        TraitDecl {
            name,
            generics,
            items,
            span: start.to(self.cur_span()),
        }
    }

    fn impl_decl(&mut self, start: Span) -> ImplDecl {
        self.bump();
        self.generics();
        let first = self.ty();
        let (trait_, self_ty) = if self.eat_kw(Kw::For) {
            let p = match &first {
                Ty::Path { path, .. } => Some(path.clone()),
                _ => None,
            };
            (p, self.ty())
        } else {
            (None, first)
        };
        let mut items = Vec::new();
        if self.expect(Tok::LBrace, "to open an impl body").is_some() {
            while !self.at(&Tok::RBrace) && !self.at_eof() {
                if self.burn() {
                    break;
                }
                let a = self.attrs();
                let s = self.cur_span();
                self.eat_kw(Kw::Pub);
                if self.at_kw(Kw::Fn) {
                    items.push(self.fn_decl(a, false, s));
                } else {
                    self.recover_to(&[Tok::Kw(Kw::Fn), Tok::RBrace]);
                }
            }
            self.expect(Tok::RBrace, "to close an impl body");
        }
        ImplDecl {
            trait_,
            self_ty,
            items,
            span: start.to(self.cur_span()),
        }
    }

    // ---------------- types ----------------

    fn ty(&mut self) -> Ty {
        let start = self.cur_span();
        match self.cur().clone() {
            Tok::Amp => {
                self.bump();
                let mutable = self.eat_kw(Kw::Mut);
                let inner = Box::new(self.ty());
                Ty::Ref {
                    inner,
                    mutable,
                    span: start.to(self.cur_span()),
                }
            }
            Tok::LParen => {
                self.bump();
                if self.at(&Tok::RParen) {
                    let end = self.bump();
                    return Ty::Unit(start.to(end));
                }
                let mut elems = vec![self.ty()];
                while self.eat(&Tok::Comma) {
                    if self.at(&Tok::RParen) {
                        break;
                    }
                    elems.push(self.ty());
                }
                let end = self
                    .expect(Tok::RParen, "to close a tuple type")
                    .unwrap_or(start);
                if elems.len() == 1 {
                    elems.pop().unwrap()
                } else {
                    Ty::Tuple {
                        elems,
                        span: start.to(end),
                    }
                }
            }
            Tok::LBracket => {
                self.bump();
                let elem = Box::new(self.ty());
                if self.eat(&Tok::Semi) {
                    let len = Box::new(self.expr());
                    let end = self
                        .expect(Tok::RBracket, "to close an array type")
                        .unwrap_or(start);
                    Ty::Array {
                        elem,
                        len,
                        span: start.to(end),
                    }
                } else {
                    let end = self
                        .expect(Tok::RBracket, "to close a slice type")
                        .unwrap_or(start);
                    Ty::Slice {
                        elem,
                        span: start.to(end),
                    }
                }
            }
            Tok::Underscore => Ty::Infer(self.bump()),
            Tok::Kw(Kw::Dyn) => {
                self.bump();
                let path = self.path();
                let sp = start.to(path.span);
                Ty::Dyn { path, span: sp }
            }
            Tok::Kw(Kw::Fn) => {
                self.bump();
                self.expect(Tok::LParen, "to open a function type");
                let mut params = Vec::new();
                while !self.at(&Tok::RParen) && !self.at_eof() {
                    params.push(self.ty());
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(Tok::RParen, "to close a function type");
                let ret = Box::new(if self.eat(&Tok::Arrow) {
                    self.ty()
                } else {
                    Ty::Unit(self.cur_span())
                });
                let effects = if self.at(&Tok::Bang) {
                    self.effect_row()
                } else {
                    EffectRow::default()
                };
                Ty::Fn {
                    params,
                    ret,
                    effects,
                    span: start.to(self.cur_span()),
                }
            }
            Tok::Ident | Tok::Kw(_) => {
                let path = self.path();
                let mut args = Vec::new();
                if self.at(&Tok::Lt) {
                    self.bump();
                    while !self.at(&Tok::Gt) && !self.at_eof() {
                        if self.burn() {
                            break;
                        }
                        args.push(self.ty());
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(Tok::Gt, "to close type arguments");
                }
                Ty::Path {
                    span: start.to(self.cur_span()),
                    path,
                    args,
                }
            }
            _ => {
                let found = self.describe_cur();
                self.err(
                    "NL0007",
                    format!("expected a type, found {found}"),
                    "expected a type",
                );
                Ty::Error(self.bump())
            }
        }
    }

    fn path(&mut self) -> Path {
        let start = self.cur_span();
        let mut segments = vec![self.ident("a path segment")];
        while self.at(&Tok::ColonColon) {
            self.bump();
            segments.push(self.ident("a path segment"));
        }
        let end = segments.last().unwrap().span;
        Path {
            segments,
            span: start.to(end),
        }
    }

    // ---------------- patterns ----------------

    fn pat(&mut self) -> Pat {
        let start = self.cur_span();
        match self.cur().clone() {
            Tok::Underscore => Pat::Wild(self.bump()),
            Tok::Kw(Kw::Mut) => {
                self.bump();
                let name = self.ident("a binding name");
                Pat::Bind {
                    span: start.to(name.span),
                    name,
                    mutable: true,
                    by_ref: false,
                }
            }
            Tok::Kw(Kw::Ref) => {
                self.bump();
                let name = self.ident("a binding name");
                Pat::Bind {
                    span: start.to(name.span),
                    name,
                    mutable: false,
                    by_ref: true,
                }
            }
            Tok::LParen => {
                self.bump();
                let mut elems = Vec::new();
                while !self.at(&Tok::RParen) && !self.at_eof() {
                    elems.push(self.pat());
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                let end = self
                    .expect(Tok::RParen, "to close a tuple pattern")
                    .unwrap_or(start);
                Pat::Tuple {
                    elems,
                    span: start.to(end),
                }
            }
            Tok::Int(_) | Tok::Str(_) | Tok::Bool(_) | Tok::Money { .. } | Tok::EpochLit(_) => {
                Pat::Lit(Box::new(self.primary()))
            }
            Tok::Ident | Tok::Kw(_) => {
                let path = self.path();
                if self.at(&Tok::LParen) {
                    self.bump();
                    let mut elems = Vec::new();
                    while !self.at(&Tok::RParen) && !self.at_eof() {
                        elems.push(self.pat());
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    let end = self
                        .expect(Tok::RParen, "to close a pattern")
                        .unwrap_or(start);
                    Pat::TupleStruct {
                        path,
                        elems,
                        span: start.to(end),
                    }
                } else if self.at(&Tok::LBrace) && path.segments.len() > 1 {
                    self.bump();
                    let mut fields = Vec::new();
                    let mut rest = false;
                    while !self.at(&Tok::RBrace) && !self.at_eof() {
                        if self.eat(&Tok::DotDot) {
                            rest = true;
                            break;
                        }
                        let n = self.ident("a field name");
                        let p = if self.eat(&Tok::Colon) {
                            self.pat()
                        } else {
                            Pat::Bind {
                                name: n.clone(),
                                mutable: false,
                                by_ref: false,
                                span: n.span,
                            }
                        };
                        fields.push((n, p));
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    let end = self
                        .expect(Tok::RBrace, "to close a struct pattern")
                        .unwrap_or(start);
                    Pat::Struct {
                        path,
                        fields,
                        rest,
                        span: start.to(end),
                    }
                } else if path.segments.len() == 1
                    && path.segments[0]
                        .text
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_lowercase() || c == '_')
                {
                    let n = path.segments.into_iter().next().unwrap();
                    Pat::Bind {
                        span: n.span,
                        name: n,
                        mutable: false,
                        by_ref: false,
                    }
                } else {
                    Pat::Path(path)
                }
            }
            _ => {
                let found = self.describe_cur();
                self.err(
                    "NL0008",
                    format!("expected a pattern, found {found}"),
                    "expected a pattern",
                );
                Pat::Error(self.bump())
            }
        }
    }

    // ---------------- statements ----------------

    fn block(&mut self) -> Block {
        let start = self
            .expect(Tok::LBrace, "to open a block")
            .unwrap_or(self.cur_span());
        let mut stmts = Vec::new();
        let mut tail = None;
        while !self.at(&Tok::RBrace) && !self.at_eof() {
            if self.burn() {
                break;
            }
            let before = self.pos;
            match self.stmt() {
                Some(Stmt::Expr(e)) if self.at(&Tok::RBrace) => tail = Some(Box::new(e)),
                Some(s) => stmts.push(s),
                None => {}
            }
            if self.pos == before {
                self.bump();
            }
        }
        let end = self
            .expect(Tok::RBrace, "to close a block")
            .unwrap_or(start);
        Block {
            stmts,
            tail,
            span: start.to(end),
        }
    }

    fn stmt(&mut self) -> Option<Stmt> {
        let start = self.cur_span();
        match self.cur().clone() {
            Tok::Semi => {
                self.bump();
                None
            }
            Tok::Kw(Kw::Let) => {
                self.bump();
                let pat = self.pat();
                let ty = if self.eat(&Tok::Colon) {
                    Some(self.ty())
                } else {
                    None
                };
                let init = if self.eat(&Tok::Eq) {
                    Some(self.expr())
                } else {
                    None
                };
                let end = self.expect(Tok::Semi, "after a `let`").unwrap_or(start);
                Some(Stmt::Let {
                    pat,
                    ty,
                    init,
                    span: start.to(end),
                })
            }
            Tok::Kw(
                Kw::Insert
                | Kw::Update
                | Kw::Delete
                | Kw::Begin
                | Kw::Commit
                | Kw::Rollback
                | Kw::Grant
                | Kw::Revoke
                | Kw::Backfill
                | Kw::Emit,
            ) => self.dml().map(Stmt::Dml),
            Tok::Kw(
                Kw::Fn | Kw::Struct | Kw::Enum | Kw::Use | Kw::Const | Kw::Static | Kw::Schema,
            ) => Some(Stmt::Item(Box::new(self.item()))),
            _ => {
                let e = self.expr();
                if self.eat(&Tok::Semi) {
                    Some(Stmt::Semi(e))
                } else {
                    Some(Stmt::Expr(e))
                }
            }
        }
    }

    fn dml(&mut self) -> Option<Dml> {
        let start = self.cur_span();
        match self.cur().clone() {
            Tok::Kw(Kw::Begin) => {
                let s = self.bump();
                self.eat(&Tok::Semi);
                Some(Dml::Begin(s))
            }
            Tok::Kw(Kw::Commit) => {
                let s = self.bump();
                self.eat(&Tok::Semi);
                Some(Dml::Commit(s))
            }
            Tok::Kw(Kw::Rollback) => {
                let s = self.bump();
                self.eat(&Tok::Semi);
                Some(Dml::Rollback(s))
            }
            Tok::Kw(Kw::Insert) => {
                self.bump();
                self.expect_kw(Kw::Into, "in an `insert`");
                let table = self.ident("a table name");
                let cols = if self.at(&Tok::LParen) {
                    self.paren_names()
                } else {
                    Vec::new()
                };
                self.expect_kw(Kw::Values, "in an `insert`");
                let mut rows = Vec::new();
                loop {
                    self.expect(Tok::LParen, "to open a value row");
                    let mut row = Vec::new();
                    while !self.at(&Tok::RParen) && !self.at_eof() {
                        row.push(self.expr());
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(Tok::RParen, "to close a value row");
                    rows.push(row);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                let end = self.expect(Tok::Semi, "after an `insert`").unwrap_or(start);
                Some(Dml::Insert {
                    table,
                    cols,
                    rows,
                    span: start.to(end),
                })
            }
            Tok::Kw(Kw::Update) => {
                self.bump();
                let table = self.ident("a table name");
                self.expect_kw(Kw::Set, "in an `update`");
                let mut sets = Vec::new();
                loop {
                    let col = self.ident("a column name");
                    self.expect(Tok::Eq, "in an assignment");
                    sets.push((col, self.expr()));
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                let filter = if self.eat_kw(Kw::Where) {
                    Some(self.expr())
                } else {
                    None
                };
                let end = self.expect(Tok::Semi, "after an `update`").unwrap_or(start);
                Some(Dml::Update {
                    table,
                    sets,
                    filter,
                    span: start.to(end),
                })
            }
            Tok::Kw(Kw::Delete) => {
                self.bump();
                self.expect_kw(Kw::From, "in a `delete`");
                let table = self.ident("a table name");
                let filter = if self.eat_kw(Kw::Where) {
                    Some(self.expr())
                } else {
                    None
                };
                let end = self.expect(Tok::Semi, "after a `delete`").unwrap_or(start);
                Some(Dml::Delete {
                    table,
                    filter,
                    span: start.to(end),
                })
            }
            Tok::Kw(k @ (Kw::Grant | Kw::Revoke)) => {
                self.bump();
                let effect = self.effect();
                self.expect_kw(Kw::On, "in a grant or revoke");
                let on = self.ident("the object");
                let subject = if k == Kw::Grant {
                    self.ident("the grantee")
                } else {
                    self.ident("the holder")
                };
                let end = self
                    .expect(Tok::Semi, "after a grant or revoke")
                    .unwrap_or(start);
                Some(if k == Kw::Grant {
                    Dml::Grant {
                        effect,
                        on,
                        to: subject,
                        span: start.to(end),
                    }
                } else {
                    Dml::Revoke {
                        effect,
                        on,
                        from: subject,
                        span: start.to(end),
                    }
                })
            }
            Tok::Kw(Kw::Backfill) => {
                self.bump();
                let view = self.ident("a view name");
                let upto = if self.eat_kw(Kw::Upto) {
                    Some(self.expr())
                } else {
                    None
                };
                let end = self
                    .expect(Tok::Semi, "after a `backfill`")
                    .unwrap_or(start);
                Some(Dml::Backfill {
                    view,
                    upto,
                    span: start.to(end),
                })
            }
            Tok::Kw(Kw::Emit) => {
                self.bump();
                let view = self.ident("a view name");
                let to = if matches!(self.cur(), Tok::Ident) {
                    self.ident("a sink")
                } else {
                    Name::new("<default>", start)
                };
                let end = self.expect(Tok::Semi, "after an `emit`").unwrap_or(start);
                Some(Dml::Emit {
                    view,
                    to,
                    span: start.to(end),
                })
            }
            _ => None,
        }
    }

    // ---------------- expressions: Pratt ----------------

    pub fn expr(&mut self) -> Expr {
        self.expr_bp(0)
    }

    fn expr_bp(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.unary();
        loop {
            if self.burn() {
                return lhs;
            }
            // Assignment is right-associative and lowest; handled outside the table — and
            // does not exist at all inside a SQL statement, where `=` is equality.
            //
            // The right-hand side is parsed at binding power **0**, not 1. At 1 the inner
            // call cannot take an assignment of its own — the guard above is `min_bp == 0`
            // — so `a = b = c` came out as `(a = b) = c`, which is left-associative and
            // the opposite of both this comment and Rust's rule. The defect survived
            // because no test asked, and it was found when a second implementation of the
            // same grammar (`bootstrap/parser.niles`) was written against Appendix B.10.1 and
            // the two disagreed. That is the whole argument for having two.
            if min_bp == 0 && self.sql_depth == 0 && self.at(&Tok::Eq) {
                self.bump();
                let value = self.expr_bp(0);
                let span = lhs.span().to(value.span());
                lhs = Expr::Assign {
                    target: Box::new(lhs),
                    value: Box::new(value),
                    span,
                };
                continue;
            }
            let Some(op) = self.peek_binop() else { break };
            let bp = op.precedence();
            if bp < min_bp {
                break;
            }
            self.consume_binop(op);
            // `between (a, b)` takes a tuple, so the parse is uniform.
            let rhs = self.expr_bp(bp + 1);
            let span = lhs.span().to(rhs.span());
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }
        lhs
    }

    fn peek_binop(&self) -> Option<BinOp> {
        Some(match self.cur() {
            // SQL's `=`. Outside a SQL statement this token never reaches here, because
            // `expr_bp` takes it as an assignment first.
            Tok::Eq if self.sql_depth > 0 => BinOp::Eq,
            Tok::Plus => BinOp::Add,
            Tok::Minus => BinOp::Sub,
            Tok::Star => BinOp::Mul,
            Tok::Slash => BinOp::Div,
            Tok::Percent => BinOp::Rem,
            Tok::EqEq => BinOp::Eq,
            Tok::Ne => BinOp::Ne,
            Tok::Lt => BinOp::Lt,
            Tok::Le => BinOp::Le,
            Tok::Gt => BinOp::Gt,
            Tok::Ge => BinOp::Ge,
            Tok::AmpAmp => BinOp::And,
            Tok::PipePipe => BinOp::Or,
            Tok::Amp => BinOp::BitAnd,
            Tok::Pipe => BinOp::BitOr,
            Tok::Caret => BinOp::BitXor,
            Tok::Kw(Kw::And) => BinOp::And,
            Tok::Kw(Kw::Or) => BinOp::Or,
            Tok::Kw(Kw::Is) => BinOp::Is,
            Tok::Kw(Kw::In) => BinOp::In,
            Tok::Kw(Kw::Like) => BinOp::Like,
            Tok::Kw(Kw::Between) => BinOp::Between,
            Tok::Kw(Kw::Not) if matches!(self.nth(1), Tok::Kw(Kw::In)) => BinOp::NotIn,
            _ => return None,
        })
    }

    fn consume_binop(&mut self, op: BinOp) {
        self.bump();
        match op {
            BinOp::NotIn => {
                self.bump();
            }
            BinOp::Is if self.at_kw(Kw::Not) => {
                self.bump();
            }
            _ => {}
        }
    }

    fn unary(&mut self) -> Expr {
        let start = self.cur_span();
        let e = match self.cur().clone() {
            Tok::Minus => {
                self.bump();
                let operand = Box::new(self.unary());
                Expr::Unary {
                    op: UnOp::Neg,
                    span: start.to(operand.span()),
                    operand,
                }
            }
            Tok::Bang | Tok::Kw(Kw::Not) => {
                self.bump();
                let operand = Box::new(self.unary());
                Expr::Unary {
                    op: UnOp::Not,
                    span: start.to(operand.span()),
                    operand,
                }
            }
            Tok::Star => {
                self.bump();
                let operand = Box::new(self.unary());
                Expr::Unary {
                    op: UnOp::Deref,
                    span: start.to(operand.span()),
                    operand,
                }
            }
            Tok::Amp => {
                self.bump();
                let mutable = self.eat_kw(Kw::Mut);
                let operand = Box::new(self.unary());
                let op = if mutable { UnOp::RefMut } else { UnOp::Ref };
                Expr::Unary {
                    op,
                    span: start.to(operand.span()),
                    operand,
                }
            }
            _ => self.primary(),
        };
        self.postfix(e)
    }

    /// Postfix: field access, indexing, calls, `?`, `as`, and the two pipeline spellings.
    fn postfix(&mut self, mut e: Expr) -> Expr {
        loop {
            if self.burn() {
                return e;
            }
            match self.cur().clone() {
                Tok::Dot => {
                    self.bump();
                    let name = self.member_name("a field or stage name");
                    if self.at(&Tok::LParen) {
                        let args = self.call_args();
                        let kind = StageKind::from_name(&name.text);
                        let span = e.span().to(self.cur_span());
                        // `.fixpoint(step) guard measure(m)` — the guard is part of the
                        // stage, not an afterthought, because an unguarded fixpoint must
                        // have no spelling at all.
                        if kind == StageKind::Fixpoint {
                            let step = args
                                .into_iter()
                                .next()
                                .map(|a| a.value)
                                .unwrap_or(Expr::Error(span));
                            let measure = if self.eat_kw(Kw::Guard) {
                                self.expect_kw(Kw::Measure, "after `guard`");
                                self.expect(Tok::LParen, "to open a measure");
                                let m = self.expr();
                                self.expect(Tok::RParen, "to close a measure");
                                m
                            } else {
                                self.diags.push(
                                    Diagnostic::error("NL0400", "`fixpoint` requires a termination guard")
                                        .primary(span, "unguarded recursion")
                                        .note("every recursive view must name a well-founded measure that strictly decreases")
                                        .suggest(self.cur_span(), " guard measure(depth)", "add a measure", Applicability::HasPlaceholders),
                                );
                                Expr::Error(span)
                            };
                            e = Expr::Fixpoint {
                                recv: Box::new(e),
                                step: Box::new(step),
                                measure: Box::new(measure),
                                span: span.to(self.cur_span()),
                            };
                        } else {
                            if kind == StageKind::Unknown {
                                if let Some(sugg) = crate::diagnostics::closest(
                                    &name.text,
                                    StageKind::all_names().iter().copied(),
                                ) {
                                    self.diags.push(
                                        Diagnostic::warning(
                                            "NL0401",
                                            format!(
                                                "`{}` is not a known pipeline stage",
                                                name.text
                                            ),
                                        )
                                        .primary(name.span, "unknown stage")
                                        .suggest(
                                            name.span,
                                            sugg,
                                            format!("did you mean `{sugg}`?"),
                                            Applicability::MaybeIncorrect,
                                        ),
                                    );
                                }
                            }
                            e = Expr::Stage {
                                recv: Box::new(e),
                                kind,
                                name,
                                args,
                                span,
                            };
                        }
                    } else {
                        let span = e.span().to(name.span);
                        e = Expr::Field {
                            base: Box::new(e),
                            name,
                            span,
                        };
                    }
                }
                Tok::PipeGt => {
                    // `q |> where(|r| p)` — the same node as `q.where(..)`.
                    self.bump();
                    let name = self.member_name("a stage name");
                    let args = self.call_args();
                    let kind = StageKind::from_name(&name.text);
                    let span = e.span().to(self.cur_span());
                    e = Expr::Stage {
                        recv: Box::new(e),
                        kind,
                        name,
                        args,
                        span,
                    };
                }
                Tok::LParen => {
                    let args = self.call_args();
                    let span = e.span().to(self.cur_span());
                    e = Expr::Call {
                        callee: Box::new(e),
                        args,
                        span,
                    };
                }
                Tok::LBracket => {
                    self.bump();
                    let index = Box::new(self.expr());
                    let end = self
                        .expect(Tok::RBracket, "to close an index")
                        .unwrap_or(index.span());
                    let span = e.span().to(end);
                    e = Expr::Index {
                        base: Box::new(e),
                        index,
                        span,
                    };
                }
                Tok::Question => {
                    let end = self.bump();
                    let span = e.span().to(end);
                    e = Expr::Try {
                        expr: Box::new(e),
                        span,
                    };
                }
                Tok::Kw(Kw::As) if self.no_alias_depth == 0 => {
                    self.bump();
                    let ty = self.ty();
                    let span = e.span().to(ty.span());
                    e = Expr::Cast {
                        expr: Box::new(e),
                        ty,
                        span,
                    };
                }
                _ => return e,
            }
        }
    }

    fn call_args(&mut self) -> Vec<Arg> {
        let mut args = Vec::new();
        if self
            .expect(Tok::LParen, "to open an argument list")
            .is_none()
        {
            return args;
        }
        while !self.at(&Tok::RParen) && !self.at_eof() {
            if self.burn() {
                break;
            }
            let start = self.cur_span();
            // A named argument is `ident:` — but `a::b` and a bare path must not be
            // mistaken for one, hence the two-token probe.
            let name = if matches!(self.cur(), Tok::Ident | Tok::Kw(_))
                && matches!(self.nth(1), Tok::Colon)
                && !matches!(self.nth(1), Tok::ColonColon)
            {
                let n = self.ident("an argument name");
                self.bump();
                Some(n)
            } else {
                None
            };
            let value = self.expr();
            args.push(Arg {
                name,
                span: start.to(value.span()),
                value,
            });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(Tok::RParen, "to close an argument list");
        args
    }

    fn primary(&mut self) -> Expr {
        let start = self.cur_span();
        match self.cur().clone() {
            Tok::Int(v) => Expr::Int(v, self.bump()),
            Tok::Float(v) => Expr::Float(v, self.bump()),
            Tok::Bool(v) => Expr::Bool(v, self.bump()),
            Tok::Str(s) => Expr::Str(s, self.bump()),
            Tok::Bytes(b) => Expr::Bytes(b, self.bump()),
            Tok::EpochLit(e) => Expr::Epoch(e, self.bump()),
            Tok::Money {
                minor,
                scale,
                currency,
            } => {
                let s = self.bump();
                Expr::Money {
                    minor,
                    scale,
                    currency: Name::new(currency, s),
                    span: s,
                }
            }
            Tok::Instant(t) => {
                let s = self.bump();
                Expr::Instant {
                    text: t,
                    valid_axis: false,
                    span: s,
                }
            }
            Tok::ValidInstant(t) => {
                let s = self.bump();
                Expr::Instant {
                    text: t,
                    valid_axis: true,
                    span: s,
                }
            }
            Tok::Duration { value, unit } => {
                let s = self.bump();
                Expr::Duration {
                    value,
                    unit,
                    span: s,
                }
            }
            Tok::Underscore => {
                let s = self.bump();
                Expr::Path(Path {
                    segments: vec![Name::new("_", s)],
                    span: s,
                })
            }
            Tok::LParen => {
                self.bump();
                if self.at(&Tok::RParen) {
                    let end = self.bump();
                    return Expr::Unit(start.to(end));
                }
                let mut elems = vec![self.expr()];
                let mut is_tuple = false;
                while self.eat(&Tok::Comma) {
                    is_tuple = true;
                    if self.at(&Tok::RParen) {
                        break;
                    }
                    elems.push(self.expr());
                }
                let end = self
                    .expect(Tok::RParen, "to close a parenthesised expression")
                    .unwrap_or(start);
                if is_tuple {
                    Expr::Tuple {
                        elems,
                        span: start.to(end),
                    }
                } else {
                    elems.pop().unwrap()
                }
            }
            Tok::LBracket => {
                self.bump();
                let mut elems = Vec::new();
                while !self.at(&Tok::RBracket) && !self.at_eof() {
                    elems.push(self.expr());
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                let end = self
                    .expect(Tok::RBracket, "to close an array")
                    .unwrap_or(start);
                Expr::Array {
                    elems,
                    span: start.to(end),
                }
            }
            Tok::LBrace => Expr::Block(Box::new(self.block())),
            Tok::Pipe | Tok::PipePipe => self.closure(false),
            Tok::Kw(Kw::Move) => {
                self.bump();
                self.closure(true)
            }
            Tok::Kw(Kw::If) => {
                self.bump();
                let cond = Box::new(self.expr_no_struct());
                let then = Box::new(self.block());
                let els = if self.eat_kw(Kw::Else) {
                    Some(Box::new(if self.at_kw(Kw::If) {
                        self.primary()
                    } else {
                        Expr::Block(Box::new(self.block()))
                    }))
                } else {
                    None
                };
                Expr::If {
                    cond,
                    then,
                    els,
                    span: start.to(self.cur_span()),
                }
            }
            Tok::Kw(Kw::Case) => self.case_expr(start),
            Tok::Kw(Kw::Match) => {
                self.bump();
                let scrutinee = Box::new(self.expr_no_struct());
                self.expect(Tok::LBrace, "to open match arms");
                let mut arms = Vec::new();
                while !self.at(&Tok::RBrace) && !self.at_eof() {
                    if self.burn() {
                        break;
                    }
                    let a_start = self.cur_span();
                    let pat = self.pat();
                    let guard = if self.eat_kw(Kw::When) || self.eat_kw(Kw::If) {
                        Some(self.expr())
                    } else {
                        None
                    };
                    self.expect(Tok::FatArrow, "in a match arm");
                    let body = self.expr();
                    arms.push(MatchArm {
                        pat,
                        guard,
                        span: a_start.to(body.span()),
                        body,
                    });
                    self.eat(&Tok::Comma);
                }
                let end = self
                    .expect(Tok::RBrace, "to close match arms")
                    .unwrap_or(start);
                Expr::Match {
                    scrutinee,
                    arms,
                    span: start.to(end),
                }
            }
            Tok::Kw(Kw::While) => {
                self.bump();
                let cond = Box::new(self.expr_no_struct());
                let body = Box::new(self.block());
                Expr::While {
                    cond,
                    span: start.to(body.span),
                    body,
                }
            }
            Tok::Kw(Kw::Loop) => {
                self.bump();
                let body = Box::new(self.block());
                Expr::Loop {
                    span: start.to(body.span),
                    body,
                }
            }
            Tok::Kw(Kw::For) => {
                self.bump();
                let pat = self.pat();
                self.expect_kw(Kw::In, "in a `for` loop");
                let iter = Box::new(self.expr_no_struct());
                let body = Box::new(self.block());
                Expr::For {
                    pat,
                    iter,
                    span: start.to(body.span),
                    body,
                }
            }
            Tok::Kw(Kw::Return) => {
                self.bump();
                let value = if self.at(&Tok::Semi) || self.at(&Tok::RBrace) {
                    None
                } else {
                    Some(Box::new(self.expr()))
                };
                Expr::Return {
                    span: start.to(self.cur_span()),
                    value,
                }
            }
            Tok::Kw(Kw::Break) => Expr::Break(self.bump()),
            Tok::Kw(Kw::Continue) => Expr::Continue(self.bump()),
            Tok::Kw(Kw::Txn) => self.txn_expr(start),
            Tok::Kw(Kw::Hold) => {
                self.bump();
                let args = self.call_args();
                Expr::Hold {
                    args,
                    span: start.to(self.cur_span()),
                }
            }
            Tok::Kw(Kw::Resolve) => {
                self.bump();
                let hold = Box::new(self.unary());
                let outcome = match self.cur().clone() {
                    Tok::Kw(Kw::Post) => {
                        self.bump();
                        ResolveOutcome::Post(Box::new(self.expr()))
                    }
                    Tok::Kw(Kw::Void) => ResolveOutcome::Void(self.bump()),
                    Tok::Kw(Kw::Expire) => ResolveOutcome::Expire(self.bump()),
                    _ => {
                        let sp = self.cur_span();
                        self.diags.push(
                            Diagnostic::error("NL0402", "a hold must be resolved by `post`, `void` or `expire`")
                                .primary(sp, "expected an outcome")
                                .note("a hold is linear: it is consumed exactly once, and the outcome is what consumes it"),
                        );
                        ResolveOutcome::Error(sp)
                    }
                };
                Expr::Resolve {
                    hold,
                    outcome,
                    span: start.to(self.cur_span()),
                }
            }
            Tok::Kw(Kw::Fx) => {
                self.bump();
                self.expect(Tok::LBrace, "to open an `fx` form");
                let mut legs = Vec::new();
                let mut rate = None;
                while !self.at(&Tok::RBrace) && !self.at_eof() {
                    if self.burn() {
                        break;
                    }
                    if self.eat_kw(Kw::Leg) {
                        let n = self.ident("a leg name");
                        self.expect(Tok::Colon, "after a leg name");
                        legs.push((n, self.expr()));
                    } else if self.eat_kw(Kw::Rate) {
                        self.expect(Tok::Colon, "after `rate`");
                        rate = Some(Box::new(self.expr()));
                    } else {
                        let sp = self.cur_span();
                        self.err("NL0403", "expected `leg` or `rate` in an `fx` form", "here");
                        self.recover_to(&[Tok::Comma, Tok::RBrace]);
                        let _ = sp;
                    }
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                let end = self
                    .expect(Tok::RBrace, "to close an `fx` form")
                    .unwrap_or(start);
                Expr::Fx {
                    legs,
                    rate,
                    span: start.to(end),
                }
            }
            Tok::Kw(Kw::Authorize) => {
                self.bump();
                let args = self.call_args();
                Expr::Authorize {
                    args,
                    span: start.to(self.cur_span()),
                }
            }
            Tok::Kw(Kw::Declassify) => {
                self.bump();
                let args = self.call_args();
                Expr::Declassify {
                    args,
                    span: start.to(self.cur_span()),
                }
            }
            Tok::Kw(Kw::Explain) => {
                self.bump();
                self.eat_kw(Kw::Of);
                let target = Box::new(self.expr());
                Expr::Explain {
                    span: start.to(target.span()),
                    target,
                }
            }
            Tok::Kw(Kw::Reproduce) => {
                self.bump();
                let target = Box::new(self.expr());
                let at = if self.eat_kw(Kw::AsOf) || self.eat(&Tok::At) {
                    Some(Box::new(self.expr()))
                } else {
                    None
                };
                Expr::Reproduce {
                    span: start.to(self.cur_span()),
                    target,
                    at,
                }
            }
            Tok::Kw(Kw::Impact) => {
                self.bump();
                self.eat_kw(Kw::Of);
                let target = Box::new(self.expr());
                Expr::Impact {
                    span: start.to(target.span()),
                    target,
                }
            }
            Tok::Kw(Kw::Sql) => {
                self.bump();
                self.expect(Tok::LBrace, "to open a `sql` block");
                let inner = Box::new(self.select_stmt());
                let end = self
                    .expect(Tok::RBrace, "to close a `sql` block")
                    .unwrap_or(start);
                Expr::Sql {
                    inner,
                    span: start.to(end),
                }
            }
            Tok::Kw(Kw::Select) | Tok::Kw(Kw::With) => self.select_stmt(),
            // `exists (select ..)`. The parentheses are required: without them the
            // subquery's `from` clause and the enclosing one would be ambiguous to a
            // reader, whatever the grammar could be made to accept.
            Tok::Kw(Kw::Exists) => {
                self.bump();
                self.expect(Tok::LParen, "to open an `exists` subquery");
                let inner = self.select_inner(start);
                let end = self
                    .expect(Tok::RParen, "to close an `exists` subquery")
                    .unwrap_or(start);
                Expr::Exists {
                    query: Box::new(inner),
                    span: start.to(end),
                }
            }
            Tok::Ident | Tok::Kw(_) => {
                let path = self.path();
                // A struct literal, but only where a `{` cannot be a block — which is why
                // `expr_no_struct` exists for `if`/`while`/`match` scrutinees.
                if self.at(&Tok::LBrace) && self.struct_lit_ok() {
                    self.bump();
                    let mut fields = Vec::new();
                    while !self.at(&Tok::RBrace) && !self.at_eof() {
                        let n = self.ident("a field name");
                        let v = if self.eat(&Tok::Colon) {
                            self.expr()
                        } else {
                            Expr::Path(Path {
                                segments: vec![n.clone()],
                                span: n.span,
                            })
                        };
                        fields.push((n, v));
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    let end = self
                        .expect(Tok::RBrace, "to close a struct literal")
                        .unwrap_or(start);
                    Expr::StructLit {
                        path,
                        fields,
                        span: start.to(end),
                    }
                } else {
                    Expr::Path(path)
                }
            }
            _ => {
                let found = self.describe_cur();
                self.err(
                    "NL0009",
                    format!("expected an expression, found {found}"),
                    "expected an expression",
                );
                let sp = self.recover_to(&[Tok::Semi, Tok::RBrace, Tok::RParen, Tok::Comma]);
                Expr::Error(start.to(sp))
            }
        }
    }

    /// Struct literals are permitted here. Set to `false` for the head of `if`, `while`,
    /// `match` and `for`, where a `{` opens the body.
    fn struct_lit_ok(&self) -> bool {
        self.no_struct_depth == 0
    }

    fn expr_no_struct(&mut self) -> Expr {
        self.no_struct_depth += 1;
        let e = self.expr();
        self.no_struct_depth -= 1;
        e
    }

    fn closure(&mut self, is_move: bool) -> Expr {
        let start = self.cur_span();
        let mut params = Vec::new();
        if self.eat(&Tok::PipePipe) {
            // `||` — no parameters.
        } else {
            self.expect(Tok::Pipe, "to open closure parameters");
            while !self.at(&Tok::Pipe) && !self.at_eof() {
                if self.burn() {
                    break;
                }
                let p = self.pat();
                let t = if self.eat(&Tok::Colon) {
                    Some(self.ty())
                } else {
                    None
                };
                params.push((p, t));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(Tok::Pipe, "to close closure parameters");
        }
        let body = Box::new(self.expr());
        Expr::Closure {
            params,
            span: start.to(body.span()),
            body,
            is_move,
        }
    }

    fn txn_expr(&mut self, start: Span) -> Expr {
        self.bump(); // `txn`
        let idem = if self.at_kw(Kw::Idem) {
            let i_start = self.bump();
            let args = self.call_args();
            let mut key = None;
            let mut window = None;
            for a in args {
                match a.name.as_ref().map(|n| n.text.as_str()) {
                    Some("window") => window = Some(Box::new(a.value)),
                    _ if key.is_none() => key = Some(Box::new(a.value)),
                    _ => {}
                }
            }
            Some(IdemSpec {
                key: key.unwrap_or_else(|| Box::new(Expr::Error(i_start))),
                window,
                span: i_start.to(self.cur_span()),
            })
        } else {
            None
        };
        let body = Box::new(self.block());
        Expr::Txn {
            idem,
            span: start.to(body.span),
            body,
        }
    }

    fn case_expr(&mut self, start: Span) -> Expr {
        self.bump(); // `case`
        let mut arms = Vec::new();
        while self.eat_kw(Kw::When) {
            if self.burn() {
                break;
            }
            let cond = self.expr();
            self.expect_kw(Kw::Then, "in a `case` arm");
            let value = self.expr();
            arms.push((cond, value));
        }
        let els = if self.eat_kw(Kw::Else) {
            Some(Box::new(self.expr()))
        } else {
            None
        };
        let end = self
            .expect_kw(Kw::End, "to close a `case`")
            .unwrap_or(start);
        Expr::Case {
            arms,
            els,
            span: start.to(end),
        }
    }

    // ---------------- the SQL surface ----------------

    fn select_stmt(&mut self) -> Expr {
        let start = self.cur_span();
        let s = self.select_inner(start);
        Expr::Select(Box::new(s))
    }

    fn select_inner(&mut self, start: Span) -> SelectStmt {
        self.sql_depth += 1;
        let s = self.select_inner_body(start);
        self.sql_depth -= 1;
        s
    }

    fn select_inner_body(&mut self, start: Span) -> SelectStmt {
        // `with t as (..) select ..` — the CTE is parsed and, at lowering, inlined.
        if self.eat_kw(Kw::With) {
            self.eat_kw(Kw::Recursive);
            loop {
                let _name = self.ident("a CTE name");
                self.expect_kw(Kw::As, "in a CTE");
                self.expect(Tok::LParen, "to open a CTE body");
                let inner_start = self.cur_span();
                let _ = self.select_inner(inner_start);
                self.expect(Tok::RParen, "to close a CTE body");
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }
        self.expect_kw(Kw::Select, "to begin a query");
        let distinct = self.eat_kw(Kw::Distinct);
        let mut projections = Vec::new();
        loop {
            if self.at(&Tok::Star) {
                let s = self.bump();
                projections.push((
                    Expr::Path(Path {
                        segments: vec![Name::new("*", s)],
                        span: s,
                    }),
                    None,
                ));
            } else {
                self.no_alias_depth += 1;
                let e = self.expr();
                self.no_alias_depth -= 1;
                let alias = if self.eat_kw(Kw::As) {
                    Some(self.ident("a column alias"))
                } else if matches!(self.cur(), Tok::Ident) {
                    // A bare label, permitted where the following word's registry entry
                    // says `Label::Bare`.
                    Some(self.ident("a column alias"))
                } else {
                    None
                };
                projections.push((e, alias));
            }
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        let mut from = Vec::new();
        if self.eat_kw(Kw::From) {
            loop {
                from.push(self.table_ref());
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }
        let filter = if self.eat_kw(Kw::Where) {
            Some(self.expr())
        } else {
            None
        };
        let mut group_by = Vec::new();
        if self.eat_kw(Kw::Group) {
            self.expect_kw(Kw::By, "after `group`");
            loop {
                group_by.push(self.expr());
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }
        let having = if self.eat_kw(Kw::Having) {
            Some(self.expr())
        } else {
            None
        };
        let mut order_by = Vec::new();
        if self.eat_kw(Kw::Order) {
            self.expect_kw(Kw::By, "after `order`");
            loop {
                let e = self.expr();
                let asc = if self.eat_kw(Kw::Desc) {
                    false
                } else {
                    // `asc` is the default, so the keyword is optional and consuming it
                    // is the whole point of this call; the direction is ascending either
                    // way. Written as `eat_kw(Asc) || true` this reads as a logic bug.
                    self.eat_kw(Kw::Asc);
                    true
                };
                order_by.push((e, asc));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }
        let limit = if self.eat_kw(Kw::Limit) {
            Some(self.expr())
        } else {
            None
        };
        let offset = if self.eat_kw(Kw::Offset) {
            Some(self.expr())
        } else {
            None
        };
        let set_op = if self.at_kw(Kw::Union) || self.at_kw(Kw::Except) || self.at_kw(Kw::Intersect)
        {
            let k = self.cur().clone();
            self.bump();
            let op = match k {
                Tok::Kw(Kw::Union) => {
                    if self.eat_kw(Kw::All) {
                        SetOp::UnionAll
                    } else {
                        SetOp::Union
                    }
                }
                Tok::Kw(Kw::Except) => SetOp::Except,
                _ => SetOp::Intersect,
            };
            let inner_start = self.cur_span();
            Some((op, Box::new(self.select_inner(inner_start))))
        } else {
            None
        };
        SelectStmt {
            distinct,
            projections,
            from,
            filter,
            group_by,
            having,
            order_by,
            limit,
            offset,
            set_op,
            span: start.to(self.cur_span()),
        }
    }

    fn table_ref(&mut self) -> TableRef {
        let start = self.cur_span();
        let mut left = if self.at(&Tok::LParen) {
            self.bump();
            let inner_start = self.cur_span();
            let q = Box::new(self.select_inner(inner_start));
            self.expect(Tok::RParen, "to close a subquery");
            let alias = if self.eat_kw(Kw::As) {
                Some(self.ident("an alias"))
            } else {
                None
            };
            TableRef::Sub {
                query: q,
                alias,
                span: start.to(self.cur_span()),
            }
        } else {
            let name = self.ident("a relation name");
            // `as x` and a bare `x` are the same alias; the `as` is optional sugar.
            let alias = if self.eat_kw(Kw::As) || matches!(self.cur(), Tok::Ident) {
                Some(self.ident("an alias"))
            } else {
                None
            };
            TableRef::Named {
                span: start.to(self.cur_span()),
                name,
                alias,
            }
        };
        loop {
            let kind = match self.cur().clone() {
                Tok::Kw(Kw::Join) => JoinKind::Inner,
                Tok::Kw(Kw::Inner) => JoinKind::Inner,
                Tok::Kw(Kw::Left) => JoinKind::Left,
                Tok::Kw(Kw::Right) => JoinKind::Right,
                Tok::Kw(Kw::Full) => JoinKind::Full,
                Tok::Kw(Kw::Cross) => JoinKind::Cross,
                _ => return left,
            };
            self.bump();
            self.eat_kw(Kw::Outer);
            self.eat_kw(Kw::Join);
            let right = Box::new(self.table_ref());
            let on = if self.eat_kw(Kw::On) {
                Some(self.expr())
            } else {
                None
            };
            left = TableRef::Join {
                left: Box::new(left),
                right,
                kind,
                on,
                span: start.to(self.cur_span()),
            };
        }
    }
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Kw(k) => {
            let w = keywords::KEYWORDS
                .iter()
                .find(|kw| kw.token == *k)
                .map(|w| w.word)
                .unwrap_or("?");
            format!("`{w}`")
        }
        Tok::Ident => "an identifier".into(),
        Tok::LParen => "`(`".into(),
        Tok::RParen => "`)`".into(),
        Tok::LBrace => "`{`".into(),
        Tok::RBrace => "`}`".into(),
        Tok::LBracket => "`[`".into(),
        Tok::RBracket => "`]`".into(),
        Tok::Comma => "`,`".into(),
        Tok::Semi => "`;`".into(),
        Tok::Colon => "`:`".into(),
        Tok::Eq => "`=`".into(),
        Tok::Arrow => "`->`".into(),
        Tok::FatArrow => "`=>`".into(),
        Tok::Pipe => "`|`".into(),
        Tok::Gt => "`>`".into(),
        Tok::Lt => "`<`".into(),
        Tok::Eof => "end of file".into(),
        other => format!("{other:?}"),
    }
}
