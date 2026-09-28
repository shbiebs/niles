//! **Statements**: queries, DML, and the DDL the checker reads (tables, views, functions,
//! triggers, types, domains, indexes, comments, drops, alters). Every other statement is
//! recognised by its leading keywords and kept whole as [`Stmt::Other`], so a script
//! containing `grant`, `set` or `begin` parses; the checker reports what it did not analyse.

use crate::ast::*;
use crate::lex::{Span, Tok};
use crate::parser::{is_reserved, PResult, Parser};

impl<'a> Parser<'a> {
    /// Every statement in the input, separated by `;`.
    pub fn statements(&mut self) -> PResult<Vec<Stmt>> {
        let mut out = Vec::new();
        loop {
            while self.eat(&Tok::Semi) {}
            if self.at_end() {
                return Ok(out);
            }
            out.push(self.statement()?);
            if !self.at_end() && !self.is(&Tok::Semi) {
                return self.err("expected `;` after the statement");
            }
        }
    }

    pub fn statement(&mut self) -> PResult<Stmt> {
        let start = self.here().start;
        let Some(w) = self.word().map(str::to_string) else {
            if self.is(&Tok::LParen) {
                return Ok(Stmt::Query(Box::new(self.query()?)));
            }
            return self.err("expected a statement");
        };
        match w.as_str() {
            "select" | "values" | "table" => Ok(Stmt::Query(Box::new(self.query()?))),
            "with" => self.with_statement(),
            "insert" => Ok(Stmt::Insert(Box::new(self.insert(Vec::new())?))),
            "update" => Ok(Stmt::Update(Box::new(self.update(Vec::new())?))),
            "delete" => Ok(Stmt::Delete(Box::new(self.delete(Vec::new())?))),
            "create" => self.create(start),
            "comment" => self.comment(start),
            "drop" => self.drop(start),
            "alter" => self.alter(start),
            "do" => self.do_block(start),
            "truncate" => self.truncate(start),
            "merge" => self.merge(Vec::new(), start),
            "call" => self.call_stmt(start),
            "explain" => self.explain(start),
            "grant" | "revoke" => self.grant(start),
            _ => self.other(start),
        }
    }

    fn do_block(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("do")?;
        let mut language = "plpgsql".to_string();
        if self.eat_kw("language") {
            language = self.any_word()?;
        }
        let Some(Tok::Str(text)) = self.peek().cloned() else {
            return self.err("expected the `do` block's body as a string");
        };
        let span = self.here();
        self.pos += 1;
        if self.eat_kw("language") {
            language = self.any_word()?;
        }
        if language != "plpgsql" {
            return Ok(Stmt::Other {
                keywords: vec!["do".into(), language],
                text: self.src[start..self.prev_end()].to_string(),
                span: self.span_from(start),
            });
        }
        let offset = body_offset(self.src, span, &text);
        let block = crate::plpgsql::parse_body_in(self.src, &text, offset)?;
        Ok(Stmt::Do {
            block: Box::new(block),
            span: self.span_from(start),
        })
    }

    fn other(&mut self, start: usize) -> PResult<Stmt> {
        let mut keywords = Vec::new();
        while let Some(w) = self.word() {
            if keywords.len() >= 3 {
                break;
            }
            keywords.push(w.to_string());
            self.pos += 1;
        }
        self.rest_of_statement();
        Ok(Stmt::Other {
            keywords,
            text: self.src[start..self.prev_end().max(start)].to_string(),
            span: self.span_from(start),
        })
    }

    /// Skip to the `;` that ends this statement (outside parentheses), or the end.
    pub(crate) fn rest_of_statement(&mut self) -> String {
        let start = self.here().start;
        let mut depth = 0i32;
        while let Some(t) = self.peek() {
            match t {
                Tok::LParen | Tok::LBracket => depth += 1,
                Tok::RParen | Tok::RBracket => depth -= 1,
                Tok::Semi if depth <= 0 => break,
                _ => {}
            }
            self.pos += 1;
        }
        self.src[start..self.prev_end().max(start)].to_string()
    }

    // ── queries ────────────────────────────────────────────────────────────────────────────

    fn with_clause(&mut self) -> PResult<(bool, Vec<Cte>)> {
        self.expect_kw("with")?;
        let recursive = self.eat_kw("recursive");
        let mut ctes = Vec::new();
        loop {
            let name = self.ident()?;
            let columns = if self.is(&Tok::LParen) {
                self.paren_ident_list()?
            } else {
                vec![]
            };
            self.expect_kw("as")?;
            let materialized = if self.eat_kw("materialized") {
                Some(true)
            } else if self.eat_kws(&["not", "materialized"]) {
                Some(false)
            } else {
                None
            };
            self.expect(&Tok::LParen)?;
            let body = match self.word() {
                Some("insert") => Stmt::Insert(Box::new(self.insert(vec![])?)),
                Some("update") => Stmt::Update(Box::new(self.update(vec![])?)),
                Some("delete") => Stmt::Delete(Box::new(self.delete(vec![])?)),
                _ => Stmt::Query(Box::new(self.query()?)),
            };
            self.expect(&Tok::RParen)?;
            // `search depth first by … set …` / `cycle … set … using …` (PostgreSQL 14+).
            while self.word().is_some_and(|w| w == "search" || w == "cycle") {
                self.skip_balanced(&[]);
            }
            ctes.push(Cte {
                name,
                columns,
                materialized,
                body: Box::new(body),
            });
            if !self.eat(&Tok::Comma) {
                return Ok((recursive, ctes));
            }
        }
    }

    fn with_statement(&mut self) -> PResult<Stmt> {
        let save = self.pos;
        let (recursive, ctes) = self.with_clause()?;
        match self.word() {
            Some("insert") => Ok(Stmt::Insert(Box::new(self.insert(ctes)?))),
            Some("update") => Ok(Stmt::Update(Box::new(self.update(ctes)?))),
            Some("delete") => Ok(Stmt::Delete(Box::new(self.delete(ctes)?))),
            Some("merge") => {
                let start = self.here().start;
                self.merge(ctes, start)
            }
            _ => {
                let _ = recursive;
                self.pos = save;
                Ok(Stmt::Query(Box::new(self.query()?)))
            }
        }
    }

    /// A full query: `[with …] body [order by …] [limit|offset|fetch …] [for update …]`.
    pub fn query(&mut self) -> PResult<Query> {
        let start = self.here().start;
        let (recursive, with) = if self.is_kw("with") {
            self.with_clause()?
        } else {
            (false, vec![])
        };
        let body = self.set_expr(0)?;
        let mut q = Query {
            with,
            recursive,
            body,
            order_by: vec![],
            limit: None,
            with_ties: false,
            offset: None,
            locking: vec![],
            span: Span::default(),
        };
        if self.eat_kws(&["order", "by"]) {
            q.order_by = self.order_list()?;
        }
        loop {
            if self.eat_kw("limit") {
                q.limit = if self.eat_kw("all") {
                    None
                } else {
                    Some(self.expr()?)
                };
            } else if self.eat_kw("offset") {
                q.offset = Some(self.expr()?);
                let _ = self.eat_kw("row") || self.eat_kw("rows");
            } else if self.eat_kw("fetch") {
                let _ = self.eat_kw("first") || self.eat_kw("next");
                q.limit = if self.word().is_some_and(|w| w == "row" || w == "rows") {
                    Some(Expr::Lit(Literal::Int("1".into()), self.here()))
                } else {
                    Some(self.expr()?)
                };
                let _ = self.eat_kw("row") || self.eat_kw("rows");
                if self.eat_kws(&["with", "ties"]) {
                    // PostgreSQL's grammar refuses `with ties` without an `order by`
                    // ("WITH TIES cannot be specified without ORDER BY clause", 42601).
                    if q.order_by.is_empty() {
                        return self.err("`with ties` needs an `order by`");
                    }
                    q.with_ties = true;
                } else {
                    self.expect_kw("only")?;
                }
            } else if self.is_kw("for")
                && self
                    .word_n(1)
                    .is_some_and(|w| ["update", "share", "no", "key"].contains(&w))
            {
                q.locking
                    .push(self.skip_balanced(&["limit", "offset", "fetch", "into"]));
            } else {
                break;
            }
        }
        q.span = self.span_from(start);
        Ok(q)
    }

    /// Set operations: `intersect` binds tighter than `union`/`except` (both left-assoc).
    fn set_expr(&mut self, min: u8) -> PResult<QueryBody> {
        let mut l = self.set_primary()?;
        loop {
            let (op, prec) = match self.word() {
                Some("union") => (SetOp::Union, 1),
                Some("except") => (SetOp::Except, 1),
                Some("intersect") => (SetOp::Intersect, 2),
                _ => return Ok(l),
            };
            if prec < min {
                return Ok(l);
            }
            self.pos += 1;
            let all = self.eat_kw("all");
            if !all {
                self.eat_kw("distinct");
            }
            let r = self.set_expr(prec + 1)?;
            l = QueryBody::SetOp {
                op,
                all,
                left: Box::new(l),
                right: Box::new(r),
            };
        }
    }

    fn set_primary(&mut self) -> PResult<QueryBody> {
        if self.is(&Tok::LParen) {
            self.pos += 1;
            let q = self.query()?;
            self.expect(&Tok::RParen)?;
            return Ok(QueryBody::Nested(Box::new(q)));
        }
        match self.word() {
            Some("select") => Ok(QueryBody::Select(Box::new(self.select()?))),
            Some("values") => {
                self.pos += 1;
                let mut rows = Vec::new();
                loop {
                    self.expect(&Tok::LParen)?;
                    let mut row = vec![self.expr()?];
                    while self.eat(&Tok::Comma) {
                        row.push(self.expr()?);
                    }
                    self.expect(&Tok::RParen)?;
                    rows.push(row);
                    if !self.eat(&Tok::Comma) {
                        return Ok(QueryBody::Values(rows));
                    }
                }
            }
            Some("table") => {
                self.pos += 1;
                self.eat_kw("only");
                let n = self.name()?;
                self.eat_op("*");
                Ok(QueryBody::Table(n))
            }
            _ => self.err("expected select, values, table or ("),
        }
    }

    fn select(&mut self) -> PResult<Select> {
        let start = self.here().start;
        self.expect_kw("select")?;
        let mut s = Select {
            distinct: false,
            distinct_on: vec![],
            items: vec![],
            into: vec![],
            into_strict: false,
            from: vec![],
            where_: None,
            group_by: vec![],
            group_distinct: false,
            having: None,
            windows: vec![],
            span: Span::default(),
        };
        if self.eat_kw("distinct") {
            s.distinct = true;
            if self.eat_kw("on") {
                self.expect(&Tok::LParen)?;
                s.distinct_on.push(self.expr()?);
                while self.eat(&Tok::Comma) {
                    s.distinct_on.push(self.expr()?);
                }
                self.expect(&Tok::RParen)?;
            }
        } else {
            self.eat_kw("all");
        }
        // An empty target list is legal in PostgreSQL (`select from t`).
        if !self.select_list_ends() {
            s.items = self.select_items()?;
        }
        if self.eat_kw("into") {
            s.into_strict = self.eat_kw("strict");
            if !self.plpgsql {
                let _ = self.eat_kw("temporary") || self.eat_kw("temp") || self.eat_kw("unlogged");
                self.eat_kw("table");
            }
            s.into.push(self.name()?);
            while self.eat(&Tok::Comma) {
                s.into.push(self.name()?);
            }
        }
        if self.eat_kw("from") {
            s.from = self.from_list()?;
        }
        if self.eat_kw("where") {
            s.where_ = Some(self.expr()?);
        }
        if self.eat_kws(&["group", "by"]) {
            if self.eat_kw("distinct") {
                s.group_distinct = true;
            } else {
                self.eat_kw("all");
            }
            loop {
                s.group_by.push(self.grouping_element()?);
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }
        if self.eat_kw("having") {
            s.having = Some(self.expr()?);
        }
        if self.eat_kw("window") {
            loop {
                let n = self.ident()?;
                self.expect_kw("as")?;
                let w = self.window_spec()?;
                s.windows.push((n, w));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }
        s.span = self.span_from(start);
        Ok(s)
    }

    fn select_list_ends(&self) -> bool {
        self.at_end()
            || self.is(&Tok::Semi)
            || self.is(&Tok::RParen)
            || self.word().is_some_and(|w| {
                [
                    "from",
                    "where",
                    "group",
                    "having",
                    "window",
                    "order",
                    "limit",
                    "offset",
                    "union",
                    "intersect",
                    "except",
                    "into",
                    "for",
                    "fetch",
                ]
                .contains(&w)
            })
    }

    /// `rollup(…)`, `cube(…)`, `grouping sets (…)`, `()` or an expression.
    fn grouping_element(&mut self) -> PResult<Expr> {
        let start = self.here().start;
        if self.is(&Tok::LParen) && self.peek_n(1) == Some(&Tok::RParen) {
            self.pos += 2;
            return Ok(Expr::Row(vec![], self.span_from(start)));
        }
        if self.eat_kws(&["grouping", "sets"]) {
            self.expect(&Tok::LParen)?;
            let mut v = vec![self.grouping_element()?];
            while self.eat(&Tok::Comma) {
                v.push(self.grouping_element()?);
            }
            self.expect(&Tok::RParen)?;
            return Ok(Expr::Func(Box::new(FuncCall {
                name: Name(vec!["grouping sets".into()]),
                args: v.into_iter().map(|e| (None, e)).collect(),
                star: false,
                distinct: false,
                variadic: false,
                order_by: vec![],
                within_group: vec![],
                filter: None,
                over: None,
                span: self.span_from(start),
            })));
        }
        self.expr()
    }

    pub fn select_items(&mut self) -> PResult<Vec<SelectItem>> {
        let mut v = Vec::new();
        loop {
            let expr = self.expr()?;
            let alias = if self.eat_kw("as") || self.can_be_bare_alias() {
                Some(self.any_word()?)
            } else {
                None
            };
            v.push(SelectItem { expr, alias });
            if !self.eat(&Tok::Comma) {
                return Ok(v);
            }
        }
    }

    pub fn from_list(&mut self) -> PResult<Vec<FromItem>> {
        let mut v = vec![self.join_tree()?];
        while self.eat(&Tok::Comma) {
            v.push(self.join_tree()?);
        }
        Ok(v)
    }

    fn join_tree(&mut self) -> PResult<FromItem> {
        let start = self.here().start;
        let mut l = self.table_ref()?;
        loop {
            let natural = self.eat_kw("natural");
            let kind = if self.eat_kw("cross") {
                self.expect_kw("join")?;
                JoinKind::Cross
            } else if self.eat_kw("join") {
                JoinKind::Inner
            } else if self.eat_kw("inner") {
                self.expect_kw("join")?;
                JoinKind::Inner
            } else if let Some(k) = ["left", "right", "full"].iter().find(|k| self.is_kw(k)) {
                let k = *k;
                self.pos += 1;
                self.eat_kw("outer");
                self.expect_kw("join")?;
                match k {
                    "left" => JoinKind::Left,
                    "right" => JoinKind::Right,
                    _ => JoinKind::Full,
                }
            } else {
                if natural {
                    return self.err("expected `join` after `natural`");
                }
                return Ok(l);
            };
            let r = self.table_ref()?;
            let (mut on, mut using) = (None, vec![]);
            if kind != JoinKind::Cross && !natural {
                if self.eat_kw("on") {
                    on = Some(self.expr()?);
                } else if self.eat_kw("using") {
                    using = self.paren_ident_list()?;
                    if self.eat_kw("as") {
                        self.ident()?;
                    }
                } else {
                    return self.err("expected `on` or `using` after the joined table");
                }
            }
            l = FromItem::Join {
                kind,
                natural,
                left: Box::new(l),
                right: Box::new(r),
                on,
                using,
                span: self.span_from(start),
            };
        }
    }

    fn alias(&mut self) -> PResult<Option<Alias>> {
        let has = self.eat_kw("as") || self.can_be_bare_alias();
        if !has {
            return Ok(None);
        }
        let name = self.ident()?;
        let columns = if self.is(&Tok::LParen) {
            // Column aliases; for a function returning `record`, `(a int, b text)`.
            self.expect(&Tok::LParen)?;
            let mut cols = vec![self.ident()?];
            if !self.is(&Tok::Comma) && !self.is(&Tok::RParen) {
                self.type_name()?;
            }
            while self.eat(&Tok::Comma) {
                cols.push(self.ident()?);
                if !self.is(&Tok::Comma) && !self.is(&Tok::RParen) {
                    self.type_name()?;
                }
            }
            self.expect(&Tok::RParen)?;
            cols
        } else {
            vec![]
        };
        Ok(Some(Alias { name, columns }))
    }

    fn table_ref(&mut self) -> PResult<FromItem> {
        let start = self.here().start;
        let lateral = self.eat_kw("lateral");
        if self.is(&Tok::LParen) {
            if self.starts_query() {
                self.pos += 1;
                let q = self.query()?;
                self.expect(&Tok::RParen)?;
                let alias = self.alias()?;
                return Ok(FromItem::Sub {
                    lateral,
                    query: Box::new(q),
                    alias,
                    span: self.span_from(start),
                });
            }
            // A parenthesised join tree.
            self.pos += 1;
            let j = self.join_tree()?;
            self.expect(&Tok::RParen)?;
            let _ = self.alias()?;
            return Ok(j);
        }
        if self.eat_kws(&["rows", "from"]) {
            self.expect(&Tok::LParen)?;
            let text = self.skip_balanced(&[]);
            self.expect(&Tok::RParen)?;
            let ord = self.eat_kws(&["with", "ordinality"]);
            let alias = self.alias()?;
            return Ok(FromItem::Func {
                lateral,
                call: Box::new(Expr::Lit(Literal::Str(text), self.span_from(start))),
                with_ordinality: ord,
                alias,
                span: self.span_from(start),
            });
        }
        let only = self.eat_kw("only");
        let name_at = self.pos;
        let name = self.name()?;
        if self.is(&Tok::LParen) {
            // A function in `from`: re-read it as an expression, which parses the call.
            self.pos = name_at;
            let call = self.expr()?;
            let ord = self.eat_kws(&["with", "ordinality"]);
            let alias = self.alias()?;
            return Ok(FromItem::Func {
                lateral,
                call: Box::new(call),
                with_ordinality: ord,
                alias,
                span: self.span_from(start),
            });
        }
        self.eat_op("*");
        let alias = self.alias()?;
        let tablesample = if self.eat_kw("tablesample") {
            Some(self.skip_balanced(&["where", "join", "on", "group", "order", "limit"]))
        } else {
            None
        };
        Ok(FromItem::Table {
            name,
            only,
            alias,
            tablesample,
            span: self.span_from(start),
        })
    }

    // ── DML ────────────────────────────────────────────────────────────────────────────────

    fn returning(&mut self) -> PResult<Vec<SelectItem>> {
        if self.eat_kw("returning") {
            let items = self.select_items()?;
            Ok(items)
        } else {
            Ok(vec![])
        }
    }

    pub fn insert(&mut self, with: Vec<Cte>) -> PResult<Insert> {
        let start = self.here().start;
        self.expect_kw("insert")?;
        self.expect_kw("into")?;
        let table = self.name()?;
        let alias = if self.eat_kw("as") {
            Some(self.ident()?)
        } else {
            None
        };
        let columns = if self.is(&Tok::LParen) && !self.starts_query() {
            self.paren_column_list()?
        } else {
            vec![]
        };
        if self.eat_kw("overriding") {
            let _ = self.eat_kw("system") || self.eat_kw("user");
            self.expect_kw("value")?;
        }
        let source = if self.eat_kws(&["default", "values"]) {
            None
        } else {
            Some(Box::new(self.query()?))
        };
        let on_conflict = if self.eat_kws(&["on", "conflict"]) {
            let mut oc = OnConflict {
                target: vec![],
                constraint: None,
                update: None,
            };
            if self.eat_kws(&["on", "constraint"]) {
                oc.constraint = Some(self.ident()?);
            } else if self.is(&Tok::LParen) {
                self.pos += 1;
                oc.target.push(self.expr()?);
                while self.eat(&Tok::Comma) {
                    oc.target.push(self.expr()?);
                }
                self.expect(&Tok::RParen)?;
                if self.eat_kw("where") {
                    self.expr()?;
                }
            }
            self.expect_kw("do")?;
            if !self.eat_kw("nothing") {
                self.expect_kw("update")?;
                self.expect_kw("set")?;
                let set = self.set_list()?;
                let w = if self.eat_kw("where") {
                    Some(self.expr()?)
                } else {
                    None
                };
                oc.update = Some((set, w));
            }
            Some(oc)
        } else {
            None
        };
        let returning = self.returning()?;
        Ok(Insert {
            with,
            table,
            alias,
            columns,
            source,
            on_conflict,
            returning,
            span: self.span_from(start),
        })
    }

    /// Column names in an insert list may be qualified with subfields/subscripts (`a.b`,
    /// `a[1]`); the checker needs the base name.
    fn paren_column_list(&mut self) -> PResult<Vec<String>> {
        self.expect(&Tok::LParen)?;
        let mut v = Vec::new();
        loop {
            v.push(self.ident()?);
            while self.is(&Tok::Dot) || self.is(&Tok::LBracket) {
                if self.eat(&Tok::Dot) {
                    self.any_word()?;
                } else {
                    self.pos += 1;
                    self.expr()?;
                    self.expect(&Tok::RBracket)?;
                }
            }
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RParen)?;
        Ok(v)
    }

    pub(crate) fn set_list(&mut self) -> PResult<Vec<(Vec<String>, Expr)>> {
        let mut v = Vec::new();
        loop {
            let cols = if self.is(&Tok::LParen) {
                self.paren_column_list()?
            } else {
                let c = self.ident()?;
                while self.is(&Tok::Dot) || self.is(&Tok::LBracket) {
                    if self.eat(&Tok::Dot) {
                        self.any_word()?;
                    } else {
                        self.pos += 1;
                        self.expr()?;
                        self.expect(&Tok::RBracket)?;
                    }
                }
                vec![c]
            };
            if !self.eat_op("=") {
                return self.err("expected `=` in `set`");
            }
            self.eat_kw("row");
            let e = self.expr()?;
            v.push((cols, e));
            if !self.eat(&Tok::Comma) {
                return Ok(v);
            }
        }
    }

    fn target_alias(&mut self) -> PResult<Option<String>> {
        if self.eat_kw("as") {
            return Ok(Some(self.ident()?));
        }
        if self.can_be_bare_alias() && !self.is_kw("set") {
            return Ok(Some(self.ident()?));
        }
        Ok(None)
    }

    fn where_or_current(&mut self) -> PResult<(Option<Expr>, Option<String>)> {
        if self.eat_kw("where") {
            if self.eat_kws(&["current", "of"]) {
                return Ok((None, Some(self.ident()?)));
            }
            return Ok((Some(self.expr()?), None));
        }
        Ok((None, None))
    }

    pub fn update(&mut self, with: Vec<Cte>) -> PResult<Update> {
        let start = self.here().start;
        self.expect_kw("update")?;
        let only = self.eat_kw("only");
        let table = self.name()?;
        self.eat_op("*");
        let alias = self.target_alias()?;
        self.expect_kw("set")?;
        let set = self.set_list()?;
        let from = if self.eat_kw("from") {
            self.from_list()?
        } else {
            vec![]
        };
        let (where_, current_of) = self.where_or_current()?;
        let returning = self.returning()?;
        Ok(Update {
            with,
            table,
            only,
            alias,
            set,
            from,
            where_,
            current_of,
            returning,
            span: self.span_from(start),
        })
    }

    pub fn delete(&mut self, with: Vec<Cte>) -> PResult<Delete> {
        let start = self.here().start;
        self.expect_kw("delete")?;
        self.expect_kw("from")?;
        let only = self.eat_kw("only");
        let table = self.name()?;
        self.eat_op("*");
        let alias = self.target_alias()?;
        let using = if self.eat_kw("using") {
            self.from_list()?
        } else {
            vec![]
        };
        let (where_, current_of) = self.where_or_current()?;
        let returning = self.returning()?;
        Ok(Delete {
            with,
            table,
            only,
            alias,
            using,
            where_,
            current_of,
            returning,
            span: self.span_from(start),
        })
    }

    // ── DDL ────────────────────────────────────────────────────────────────────────────────

    fn create(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("create")?;
        let or_replace = self.eat_kws(&["or", "replace"]);
        let temporary = self.eat_kw("temporary") || self.eat_kw("temp") || {
            let g = self.eat_kw("global") || self.eat_kw("local");
            g && (self.eat_kw("temporary") || self.eat_kw("temp"))
        };
        let unlogged = self.eat_kw("unlogged");
        let _ = self.eat_kw("trusted");
        let _ = self.eat_kw("procedural");
        let _ = self.eat_kw("default");
        let materialized = self.eat_kw("materialized");
        let recursive = self.eat_kw("recursive");
        let constraint = self.eat_kw("constraint");
        let unique = self.eat_kw("unique");
        match self.word() {
            Some("table") => self.create_table(start, temporary, unlogged),
            Some("view") => self.create_view(start, or_replace, materialized, recursive),
            Some("function") | Some("procedure") => self.create_function(start, or_replace),
            Some("trigger") => self.create_trigger(start, or_replace, constraint),
            Some("index") => self.create_index(start, unique),
            Some("type") => self.create_type(start),
            Some("domain") => self.create_domain(start),
            Some("sequence") => self.create_sequence(start),
            Some("rule") => self.create_rule(start),
            Some(_) if self.define_kind().is_some() => {
                let kind = self.define_kind().unwrap();
                let words = kind.split(' ').count();
                self.pos += words;
                self.define(&kind, start)
            }
            Some("schema") => {
                self.pos += 1;
                let if_not_exists = self.eat_kws(&["if", "not", "exists"]);
                let name = self.ident()?;
                self.rest_of_statement();
                Ok(Stmt::CreateSchema {
                    name,
                    if_not_exists,
                    span: self.span_from(start),
                })
            }
            _ => self.other(start),
        }
    }

    fn create_table(&mut self, start: usize, temporary: bool, unlogged: bool) -> PResult<Stmt> {
        self.expect_kw("table")?;
        let if_not_exists = self.eat_kws(&["if", "not", "exists"]);
        let name = self.name()?;
        let mut t = CreateTable {
            name,
            temporary,
            unlogged,
            if_not_exists,
            columns: vec![],
            constraints: vec![],
            as_query: None,
            inherits: vec![],
            partition_by: None,
            span: Span::default(),
        };
        if self.is(&Tok::LParen) {
            self.pos += 1;
            if !self.is(&Tok::RParen) {
                loop {
                    self.table_element(&mut t)?;
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
            }
            self.expect(&Tok::RParen)?;
        } else if self.eat_kw("of") || self.eat_kws(&["partition", "of"]) {
            self.rest_of_statement();
            t.span = self.span_from(start);
            return Ok(Stmt::CreateTable(Box::new(t)));
        }
        loop {
            if self.eat_kw("inherits") {
                self.expect(&Tok::LParen)?;
                t.inherits.push(self.name()?);
                while self.eat(&Tok::Comma) {
                    t.inherits.push(self.name()?);
                }
                self.expect(&Tok::RParen)?;
            } else if self.eat_kws(&["partition", "by"]) {
                t.partition_by = Some(self.skip_balanced(&["with", "using", "tablespace", "as"]));
            } else if self.eat_kw("as") {
                t.as_query = Some(Box::new(self.query()?));
                if self.eat_kw("with") {
                    self.eat_kw("no");
                    self.expect_kw("data")?;
                }
            } else if self.is_kw("with")
                || self.is_kw("using")
                || self.is_kw("tablespace")
                || self.is_kw("on")
                || self.is_kw("without")
            {
                self.pos += 1;
                self.skip_balanced(&["as"]);
                if self.is(&Tok::LParen) {
                    self.pos += 1;
                    self.skip_balanced(&[]);
                    self.expect(&Tok::RParen)?;
                }
            } else {
                break;
            }
        }
        t.span = self.span_from(start);
        Ok(Stmt::CreateTable(Box::new(t)))
    }

    fn table_element(&mut self, t: &mut CreateTable) -> PResult<()> {
        let cname = if self.eat_kw("constraint") {
            Some(self.ident()?)
        } else {
            None
        };
        if cname.is_some()
            || self
                .word()
                .is_some_and(|w| ["primary", "unique", "check", "foreign", "exclude"].contains(&w))
        {
            let c = if self.eat_kws(&["primary", "key"]) {
                TableConstraint::PrimaryKey(self.paren_ident_list()?)
            } else if self.eat_kw("unique") {
                self.eat_kws(&["nulls", "not", "distinct"]);
                self.eat_kws(&["nulls", "distinct"]);
                TableConstraint::Unique(self.paren_ident_list()?)
            } else if self.eat_kw("check") {
                self.expect(&Tok::LParen)?;
                let e = self.expr()?;
                self.expect(&Tok::RParen)?;
                self.eat_kws(&["no", "inherit"]);
                TableConstraint::Check(e)
            } else if self.eat_kws(&["foreign", "key"]) {
                let cols = self.paren_ident_list()?;
                self.expect_kw("references")?;
                let rt = self.name()?;
                let rc = if self.is(&Tok::LParen) {
                    self.paren_ident_list()?
                } else {
                    vec![]
                };
                TableConstraint::ForeignKey(cols, rt, rc)
            } else if self.eat_kw("exclude") {
                TableConstraint::Exclude(self.skip_balanced(&["deferrable", "not", "initially"]))
            } else {
                return self.err("expected a table constraint");
            };
            self.constraint_attributes();
            t.constraints.push((cname, c));
            return Ok(());
        }
        if self.eat_kw("like") {
            self.name()?;
            while self.eat_kw("including") || self.eat_kw("excluding") {
                self.any_word()?;
            }
            return Ok(());
        }
        let name = self.ident()?;
        let ty = self.type_name()?;
        let mut c = ColumnDef {
            name,
            ty,
            not_null: false,
            default: None,
            primary_key: false,
            unique: false,
            check: None,
            references: None,
            generated: None,
            collate: None,
        };
        loop {
            if self.eat_kw("constraint") {
                self.ident()?;
                continue;
            }
            if self.eat_kws(&["not", "null"]) {
                c.not_null = true;
            } else if self.eat_kw("null") {
            } else if self.eat_kw("default") {
                c.default = Some(self.column_default_expr()?);
            } else if self.eat_kws(&["primary", "key"]) {
                c.primary_key = true;
                c.not_null = true;
            } else if self.eat_kw("unique") {
                c.unique = true;
            } else if self.eat_kw("check") {
                self.expect(&Tok::LParen)?;
                c.check = Some(self.expr()?);
                self.expect(&Tok::RParen)?;
                self.eat_kws(&["no", "inherit"]);
            } else if self.eat_kw("references") {
                let rt = self.name()?;
                let rc = if self.is(&Tok::LParen) {
                    self.paren_ident_list()?
                } else {
                    vec![]
                };
                c.references = Some((rt, rc));
            } else if self.eat_kw("generated") {
                // `generated always as (e) stored` or `generated {always | by default} as
                // identity [( sequence options )]`.
                let start = self.here().start;
                if !self.eat_kw("always") {
                    self.expect_kw("by")?;
                    self.expect_kw("default")?;
                }
                self.expect_kw("as")?;
                if self.eat_kw("identity") {
                    if self.is(&Tok::LParen) {
                        self.pos += 1;
                        self.skip_balanced(&[]);
                        self.expect(&Tok::RParen)?;
                    }
                } else {
                    self.expect(&Tok::LParen)?;
                    self.expr()?;
                    self.expect(&Tok::RParen)?;
                    self.expect_kw("stored")?;
                }
                c.generated = Some(self.src[start..self.prev_end()].to_string());
            } else if self.eat_kw("collate") {
                c.collate = Some(self.name()?);
            } else if self.eat_kw("compression") || self.eat_kw("storage") {
                self.any_word()?;
            } else if self.constraint_attributes() {
            } else {
                break;
            }
        }
        t.columns.push(c);
        Ok(())
    }

    /// A column default is a `b_expr` in PostgreSQL's grammar — an expression that cannot
    /// contain `not`/comparison at the top without parentheses — which this approximates by
    /// stopping before the keywords that begin the next column constraint.
    fn column_default_expr(&mut self) -> PResult<Expr> {
        self.expr()
    }

    /// `deferrable`, `initially deferred`, `on delete cascade`, `match full`, …; returns
    /// whether anything was consumed.
    fn constraint_attributes(&mut self) -> bool {
        let mut any = false;
        loop {
            if self.eat_kw("deferrable")
                || self.eat_kws(&["not", "deferrable"])
                || self.eat_kws(&["initially", "deferred"])
                || self.eat_kws(&["initially", "immediate"])
                || self.eat_kws(&["not", "valid"])
                || self.eat_kws(&["no", "inherit"])
            {
                any = true;
            } else if self.eat_kw("match") {
                let _ = self.any_word();
                any = true;
            } else if self.is_kw("on")
                && self
                    .word_n(1)
                    .is_some_and(|w| w == "delete" || w == "update")
            {
                self.pos += 2;
                if self.eat_kws(&["set", "null"])
                    || self.eat_kws(&["set", "default"])
                    || self.eat_kws(&["no", "action"])
                {
                    if self.is(&Tok::LParen) {
                        let _ = self.paren_ident_list();
                    }
                } else {
                    let _ = self.any_word();
                }
                any = true;
            } else if self.eat_kws(&["using", "index"]) {
                if self.eat_kw("tablespace") {
                    let _ = self.any_word();
                }
                any = true;
            } else if self.is_kw("with") && self.peek_n(1) == Some(&Tok::LParen) {
                self.pos += 2;
                self.skip_balanced(&[]);
                let _ = self.eat(&Tok::RParen);
                any = true;
            } else {
                return any;
            }
        }
    }

    fn create_view(
        &mut self,
        start: usize,
        or_replace: bool,
        materialized: bool,
        recursive: bool,
    ) -> PResult<Stmt> {
        self.expect_kw("view")?;
        self.eat_kws(&["if", "not", "exists"]);
        let name = self.name()?;
        let columns = if self.is(&Tok::LParen) {
            self.paren_ident_list()?
        } else {
            vec![]
        };
        if self.eat_kw("using") {
            self.any_word()?;
        }
        let mut options = Vec::new();
        if self.is_kw("with") && self.peek_n(1) == Some(&Tok::LParen) {
            self.pos += 2;
            loop {
                let mut k = self.any_word()?;
                while self.eat(&Tok::Dot) {
                    k.push('.');
                    k.push_str(&self.any_word()?);
                }
                let v = if self.eat_op("=") {
                    Some(match self.bump().map(|t| t.tok) {
                        Some(Tok::Ident(s))
                        | Some(Tok::Str(s))
                        | Some(Tok::Int(s))
                        | Some(Tok::Num(s))
                        | Some(Tok::QuotedIdent(s)) => s,
                        _ => return self.err("expected an option value"),
                    })
                } else {
                    None
                };
                options.push((k, v));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RParen)?;
        }
        if self.eat_kw("tablespace") {
            self.any_word()?;
        }
        self.expect_kw("as")?;
        let query = self.query()?;
        let mut check_option = None;
        let mut with_data = true;
        if self.eat_kw("with") {
            if self.eat_kw("cascaded") {
                check_option = Some("cascaded".into());
            } else if self.eat_kw("local") {
                check_option = Some("local".into());
            }
            if self.eat_kws(&["check", "option"]) {
                check_option.get_or_insert_with(|| "cascaded".into());
            } else if self.eat_kws(&["no", "data"]) {
                with_data = false;
            } else {
                self.eat_kw("data");
            }
        }
        Ok(Stmt::CreateView(Box::new(CreateView {
            name,
            or_replace,
            materialized,
            recursive,
            columns,
            options,
            query: Box::new(query),
            check_option,
            with_data,
            span: self.span_from(start),
        })))
    }

    fn func_args(&mut self) -> PResult<Vec<FuncArg>> {
        self.expect(&Tok::LParen)?;
        let mut v = Vec::new();
        if self.eat(&Tok::RParen) {
            return Ok(v);
        }
        loop {
            let mode = ["inout", "in", "out", "variadic"]
                .iter()
                .find(|m| self.is_kw(m))
                .map(|m| m.to_string());
            if mode.is_some() {
                self.pos += 1;
            }
            // `name type` or just `type`: a name is present when a type follows it.
            let save = self.pos;
            let name = match self.peek() {
                Some(Tok::Ident(_)) | Some(Tok::QuotedIdent(_)) => {
                    let n = self.any_word()?;
                    let ends = self.is(&Tok::Comma)
                        || self.is(&Tok::RParen)
                        || self.is_kw("default")
                        || self.is_op("=")
                        || self.is(&Tok::Dot)
                        || self.is(&Tok::LParen)
                        || self.is(&Tok::LBracket)
                        || self.is_op("%");
                    if ends
                        || matches!(
                            n.as_str(),
                            "double" | "character" | "timestamp" | "time" | "bit" | "national"
                        )
                    {
                        self.pos = save;
                        None
                    } else {
                        Some(n)
                    }
                }
                _ => None,
            };
            // `name mode type` is also PostgreSQL's (`blkno OUT bigint`).
            let mode = match (&mode, &name) {
                (None, Some(_)) => {
                    let m = ["inout", "in", "out", "variadic"]
                        .iter()
                        .find(|m| self.is_kw(m))
                        .map(|m| m.to_string());
                    if m.is_some() {
                        self.pos += 1;
                    }
                    m
                }
                _ => mode,
            };
            let ty = self.type_name()?;
            let default = if self.eat_kw("default") || self.eat_op("=") {
                Some(self.expr()?)
            } else {
                None
            };
            v.push(FuncArg {
                mode,
                name,
                ty,
                default,
            });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RParen)?;
        Ok(v)
    }

    fn create_function(&mut self, start: usize, or_replace: bool) -> PResult<Stmt> {
        let procedure = self.is_kw("procedure");
        self.pos += 1;
        let name = self.name()?;
        let args = self.func_args()?;
        let mut f = CreateFunction {
            name,
            or_replace,
            procedure,
            args,
            returns: None,
            language: None,
            volatility: None,
            strict: false,
            security_definer: false,
            leakproof: false,
            parallel: None,
            other: vec![],
            body: None,
            plpgsql: None,
            span: Span::default(),
        };
        let mut body_text: Option<(String, Span)> = None;
        loop {
            if self.eat_kws(&["returns", "null", "on", "null", "input"]) {
                f.strict = true;
            } else if self.eat_kw("returns") {
                f.returns = Some(if self.eat_kw("table") {
                    self.expect(&Tok::LParen)?;
                    let mut cols = Vec::new();
                    loop {
                        let n = self.any_word()?;
                        let t = self.type_name()?;
                        cols.push((n, t));
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(&Tok::RParen)?;
                    Returns::Table(cols)
                } else {
                    let setof = self.eat_kw("setof");
                    Returns::Type {
                        setof,
                        ty: self.type_name()?,
                    }
                });
            } else if self.eat_kw("language") {
                let l = match self.bump().map(|t| t.tok) {
                    Some(Tok::Ident(s)) | Some(Tok::Str(s)) | Some(Tok::QuotedIdent(s)) => {
                        s.to_ascii_lowercase()
                    }
                    _ => return self.err("expected a language name"),
                };
                f.language = Some(l);
            } else if self.eat_kw("as") {
                let Some(Tok::Str(s)) = self.peek().cloned() else {
                    return self.err("expected the function body as a string");
                };
                let span = self.here();
                self.pos += 1;
                // `as 'obj_file', 'link_symbol'` for C functions.
                if self.eat(&Tok::Comma) {
                    self.bump();
                }
                body_text = Some((s, span));
            } else if let Some(v) = ["immutable", "stable", "volatile"]
                .iter()
                .find(|v| self.is_kw(v))
            {
                f.volatility = Some(v.to_string());
                self.pos += 1;
            } else if self.eat_kw("strict") {
                f.strict = true;
            } else if self.eat_kws(&["called", "on", "null", "input"]) {
            } else if self.eat_kws(&["security", "definer"])
                || self.eat_kws(&["external", "security", "definer"])
            {
                f.security_definer = true;
            } else if self.eat_kws(&["security", "invoker"])
                || self.eat_kws(&["external", "security", "invoker"])
            {
            } else if self.eat_kw("leakproof") {
                f.leakproof = true;
            } else if self.eat_kws(&["not", "leakproof"]) {
            } else if self.eat_kw("parallel") {
                f.parallel = Some(self.any_word()?);
            } else if self.eat_kw("window") {
                f.other.push("window".into());
            } else if self.is_kw("cost") || self.is_kw("rows") || self.is_kw("support") {
                let k = self.any_word()?;
                let v = self.skip_balanced(&[
                    "language",
                    "as",
                    "immutable",
                    "stable",
                    "volatile",
                    "strict",
                    "security",
                    "set",
                    "cost",
                    "rows",
                    "parallel",
                    "leakproof",
                    "return",
                    "begin",
                    "returns",
                ]);
                f.other.push(format!("{k} {v}"));
            } else if self.is_kw("set") {
                self.pos += 1;
                let v = self.skip_balanced(&[
                    "language",
                    "as",
                    "immutable",
                    "stable",
                    "volatile",
                    "strict",
                    "security",
                    "cost",
                    "rows",
                    "parallel",
                    "leakproof",
                    "return",
                    "begin",
                    "returns",
                ]);
                f.other.push(format!("set {v}"));
            } else if self.is_kw("transform") {
                self.pos += 1;
                let v = self.skip_balanced(&["language", "as", "immutable", "stable", "volatile"]);
                f.other.push(format!("transform {v}"));
            } else if self.is_kw("return") {
                self.pos += 1;
                f.body = Some(FuncBody::Return(self.expr()?));
            } else if self.eat_kws(&["begin", "atomic"]) {
                let mut stmts = Vec::new();
                loop {
                    while self.eat(&Tok::Semi) {}
                    if self.eat_kw("end") {
                        break;
                    }
                    if self.at_end() {
                        return self.err("unterminated `begin atomic`");
                    }
                    stmts.push(self.statement()?);
                }
                f.body = Some(FuncBody::Atomic(stmts));
            } else {
                break;
            }
        }
        if let Some((text, span)) = body_text {
            // The body span is the string token's; the text inside begins after its opening
            // delimiter, which the PL/pgSQL parser needs to point into the original source.
            let offset = body_offset(self.src, span, &text);
            if f.language.as_deref() == Some("plpgsql") {
                f.plpgsql = Some(crate::plpgsql::parse_body_in(self.src, &text, offset)?);
            }
            f.body = Some(FuncBody::Text(text, span));
        }
        f.span = self.span_from(start);
        Ok(Stmt::CreateFunction(Box::new(f)))
    }

    fn create_trigger(
        &mut self,
        start: usize,
        or_replace: bool,
        constraint: bool,
    ) -> PResult<Stmt> {
        self.expect_kw("trigger")?;
        let name = self.ident()?;
        let timing = if self.eat_kws(&["instead", "of"]) {
            "instead of".to_string()
        } else {
            let w = self.any_word()?;
            if w != "before" && w != "after" {
                return self.err("expected before, after or instead of");
            }
            w
        };
        let mut events = Vec::new();
        loop {
            let mut ev = self.any_word()?;
            if ev == "update" && self.eat_kw("of") {
                ev = format!("update of {}", self.ident_list()?.join(", "));
            }
            events.push(ev);
            if !self.eat_kw("or") {
                break;
            }
        }
        self.expect_kw("on")?;
        let table = self.name()?;
        let mut t = CreateTrigger {
            name,
            or_replace,
            constraint,
            timing,
            events,
            table,
            deferrable: false,
            initially_deferred: false,
            for_each_row: false,
            when: None,
            function: Name(vec![]),
            span: Span::default(),
        };
        loop {
            if self.eat_kw("from") {
                self.name()?;
            } else if self.eat_kw("deferrable") {
                t.deferrable = true;
            } else if self.eat_kws(&["not", "deferrable"]) {
            } else if self.eat_kws(&["initially", "deferred"]) {
                t.initially_deferred = true;
            } else if self.eat_kws(&["initially", "immediate"]) {
            } else if self.eat_kw("referencing") {
                while self.eat_kw("old") || self.eat_kw("new") {
                    let _ = self.eat_kw("table");
                    self.eat_kw("as");
                    self.ident()?;
                }
            } else if self.eat_kw("for") {
                self.eat_kw("each");
                t.for_each_row = self.eat_kw("row");
                if !t.for_each_row {
                    self.expect_kw("statement")?;
                }
            } else if self.eat_kw("when") {
                self.expect(&Tok::LParen)?;
                t.when = Some(self.expr()?);
                self.expect(&Tok::RParen)?;
            } else {
                break;
            }
        }
        self.expect_kw("execute")?;
        let _ = self.eat_kw("function") || self.eat_kw("procedure");
        t.function = self.name()?;
        self.expect(&Tok::LParen)?;
        self.skip_balanced(&[]);
        self.expect(&Tok::RParen)?;
        t.span = self.span_from(start);
        Ok(Stmt::CreateTrigger(Box::new(t)))
    }

    fn create_index(&mut self, start: usize, unique: bool) -> PResult<Stmt> {
        self.expect_kw("index")?;
        self.eat_kw("concurrently");
        self.eat_kws(&["if", "not", "exists"]);
        let name = if !self.is_kw("on") {
            Some(self.ident()?)
        } else {
            None
        };
        self.expect_kw("on")?;
        self.eat_kw("only");
        let table = self.name()?;
        if self.eat_kw("using") {
            self.any_word()?;
        }
        self.expect(&Tok::LParen)?;
        let mut columns = Vec::new();
        loop {
            let expr = self.expr()?;
            // opclass
            if matches!(self.peek(), Some(Tok::Ident(w)) if !["asc", "desc", "nulls", "collate"].contains(&w.as_str()))
            {
                self.name()?;
            }
            let desc = self.eat_kw("desc");
            self.eat_kw("asc");
            let mut nulls_first = None;
            if self.eat_kws(&["nulls", "first"]) {
                nulls_first = Some(true);
            } else if self.eat_kws(&["nulls", "last"]) {
                nulls_first = Some(false);
            }
            columns.push(OrderItem {
                expr,
                desc,
                using: None,
                nulls_first,
            });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RParen)?;
        if self.eat_kw("include") {
            self.paren_ident_list()?;
        }
        self.eat_kws(&["nulls", "not", "distinct"]);
        self.eat_kws(&["nulls", "distinct"]);
        if self.is_kw("with") && self.peek_n(1) == Some(&Tok::LParen) {
            self.pos += 2;
            self.skip_balanced(&[]);
            self.expect(&Tok::RParen)?;
        }
        if self.eat_kw("tablespace") {
            self.any_word()?;
        }
        let where_ = if self.eat_kw("where") {
            Some(self.expr()?)
        } else {
            None
        };
        Ok(Stmt::CreateIndex {
            name,
            unique,
            table,
            columns,
            where_,
            span: self.span_from(start),
        })
    }

    fn create_type(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("type")?;
        let name = self.name()?;
        let mut attributes = Vec::new();
        let mut labels = Vec::new();
        let kind = if self.eat_kw("as") {
            if self.eat_kw("enum") {
                self.expect(&Tok::LParen)?;
                if !self.is(&Tok::RParen) {
                    loop {
                        match self.bump().map(|t| t.tok) {
                            Some(Tok::Str(s)) => labels.push(s),
                            _ => return self.err("expected an enum label"),
                        }
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&Tok::RParen)?;
                "enum".to_string()
            } else if self.eat_kw("range") {
                self.expect(&Tok::LParen)?;
                self.skip_balanced(&[]);
                self.expect(&Tok::RParen)?;
                "range".to_string()
            } else {
                self.expect(&Tok::LParen)?;
                if !self.is(&Tok::RParen) {
                    loop {
                        let n = self.ident()?;
                        let t = self.type_name()?;
                        if self.eat_kw("collate") {
                            self.name()?;
                        }
                        attributes.push((n, t));
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&Tok::RParen)?;
                "composite".to_string()
            }
        } else {
            self.rest_of_statement();
            "base".to_string()
        };
        Ok(Stmt::CreateType {
            name,
            kind,
            attributes,
            labels,
            span: self.span_from(start),
        })
    }

    fn create_domain(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("domain")?;
        let name = self.name()?;
        self.eat_kw("as");
        let ty = self.type_name()?;
        let (mut checks, mut not_null, mut default) = (vec![], false, None);
        loop {
            if self.eat_kw("constraint") {
                self.ident()?;
            } else if self.eat_kw("check") {
                self.expect(&Tok::LParen)?;
                checks.push(self.expr()?);
                self.expect(&Tok::RParen)?;
            } else if self.eat_kws(&["not", "null"]) {
                not_null = true;
            } else if self.eat_kw("null") {
            } else if self.eat_kw("default") {
                default = Some(self.expr()?);
            } else if self.eat_kw("collate") {
                self.name()?;
            } else {
                break;
            }
        }
        Ok(Stmt::CreateDomain {
            name,
            ty,
            checks,
            not_null,
            default,
            span: self.span_from(start),
        })
    }

    /// The object kinds `define` reads, when the cursor is at one (after `create [or
    /// replace] [trusted] [procedural]`).
    fn define_kind(&self) -> Option<String> {
        const KINDS: &[&[&str]] = &[
            &["text", "search", "configuration"],
            &["text", "search", "dictionary"],
            &["text", "search", "parser"],
            &["text", "search", "template"],
            &["foreign", "data", "wrapper"],
            &["operator", "class"],
            &["operator", "family"],
            &["access", "method"],
            &["event", "trigger"],
            &["foreign", "table"],
            &["user", "mapping"],
            &["operator"],
            &["aggregate"],
            &["collation"],
            &["conversion"],
            &["cast"],
            &["extension"],
            &["policy"],
            &["role"],
            &["user"],
            &["group"],
            &["server"],
            &["publication"],
            &["subscription"],
            &["statistics"],
            &["transform"],
            &["language"],
            &["database"],
            &["tablespace"],
        ];
        KINDS
            .iter()
            .find(|k| k.iter().enumerate().all(|(i, w)| self.is_kw_n(i, w)))
            .map(|k| k.join(" "))
    }

    /// The object kind after `comment on`, `drop` or `alter`: PostgreSQL's object types,
    /// longest spelling first (`text search dictionary`, `foreign data wrapper`, …).
    fn object_kind(&mut self) -> PResult<String> {
        const KINDS: &[&[&str]] = &[
            &["text", "search", "configuration"],
            &["text", "search", "dictionary"],
            &["text", "search", "parser"],
            &["text", "search", "template"],
            &["foreign", "data", "wrapper"],
            &["default", "privileges"],
            &["procedural", "language"],
            &["access", "method"],
            &["event", "trigger"],
            &["foreign", "table"],
            &["large", "object"],
            &["materialized", "view"],
            &["operator", "class"],
            &["operator", "family"],
            &["user", "mapping"],
        ];
        for k in KINDS {
            if self.eat_kws(k) {
                return Ok(k.join(" "));
            }
        }
        self.any_word()
    }

    /// The object an administrative statement names. An operator is named by its symbol
    /// (`drop operator @ (cube, cube)`), optionally schema-qualified; everything else by a
    /// qualified name. A following argument list, `using method` or `on relation` is read.
    fn object_target(&mut self, kind: &str) -> PResult<Name> {
        let name = if kind == "operator" {
            let mut parts = Vec::new();
            while matches!(self.peek(), Some(Tok::Ident(_)) | Some(Tok::QuotedIdent(_)))
                && self.peek_n(1) == Some(&Tok::Dot)
            {
                parts.push(self.any_word()?);
                self.pos += 1;
            }
            match self.bump().map(|t| t.tok) {
                Some(Tok::Op(o)) => parts.push(o),
                _ => return self.err("expected an operator symbol"),
            }
            Name(parts)
        } else if kind == "cast" {
            Name(vec!["cast".into()])
        } else if kind == "large object" {
            match self.bump().map(|t| t.tok) {
                Some(Tok::Int(n)) => Name(vec![n]),
                _ => return self.err("expected a large object's oid"),
            }
        } else if kind == "default privileges" || kind == "system" {
            Name(vec![])
        } else if kind == "transform" {
            self.expect_kw("for")?;
            let t = self.type_name()?;
            self.expect_kw("language")?;
            let l = self.any_word()?;
            Name(vec![t.written, l])
        } else {
            let mut n = self.name()?;
            // `comment on column t.c` and `alter table t` read a qualified name; a type name
            // with modifiers (`drop type numeric(3)`) does not occur in these positions.
            if kind == "user mapping" {
                n = Name(vec![]);
            }
            n
        };
        if self.is(&Tok::LParen) {
            self.pos += 1;
            self.skip_balanced(&[]);
            self.expect(&Tok::RParen)?;
        }
        if kind.starts_with("operator ") && self.eat_kw("using") {
            self.any_word()?;
        }
        Ok(name)
    }

    fn comment(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("comment")?;
        self.expect_kw("on")?;
        let kind = self.object_kind()?;
        let target = self.object_target(&kind)?;
        if self.eat_kw("on") {
            // `comment on trigger t on tbl`, `… constraint c on tbl`, `… rule r on tbl`,
            // `… policy p on tbl`, and `… constraint c on domain d`.
            self.eat_kw("domain");
            self.name()?;
        }
        self.expect_kw("is")?;
        let text = match self.bump().map(|t| t.tok) {
            Some(Tok::Str(s)) => Some(s),
            Some(Tok::Ident(n)) if n == "null" => None,
            _ => return self.err("expected a string or null"),
        };
        Ok(Stmt::Comment {
            kind,
            target,
            text,
            span: self.span_from(start),
        })
    }

    fn drop(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("drop")?;
        let kind = self.object_kind()?;
        if kind == "owned" {
            self.rest_of_statement();
            return Ok(Stmt::Drop {
                kind,
                names: vec![],
                if_exists: false,
                cascade: false,
                span: self.span_from(start),
            });
        }
        self.eat_kw("concurrently");
        let if_exists = self.eat_kws(&["if", "exists"]);
        let mut names = Vec::new();
        loop {
            names.push(self.object_target(&kind)?);
            if (kind == "trigger" || kind == "policy" || kind == "rule") && self.eat_kw("on") {
                names.push(self.name()?);
            }
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        let cascade = self.eat_kw("cascade");
        self.eat_kw("restrict");
        Ok(Stmt::Drop {
            kind,
            names,
            if_exists,
            cascade,
            span: self.span_from(start),
        })
    }

    fn alter(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("alter")?;
        let kind = self.object_kind()?;
        self.eat_kws(&["if", "exists"]);
        self.eat_kw("only");
        let name = if self.word().is_some_and(|w| w == "all") {
            Name(vec!["all".into()])
        } else {
            self.object_target(&kind)?
        };
        let action = self.rest_of_statement();
        Ok(Stmt::Alter {
            kind,
            name,
            action: action.to_ascii_lowercase(),
            span: self.span_from(start),
        })
    }
}

/// Where the text of a string constant begins in the source: after `'`, `E'`, or `$tag$`.
fn body_offset(src: &str, span: Span, _text: &str) -> usize {
    let s = &src[span.start..span.end];
    if let Some(rest) = s.strip_prefix('$') {
        let close = rest.find('$').map(|i| i + 2).unwrap_or(1);
        span.start + close
    } else if s.starts_with('\'') {
        span.start + 1
    } else {
        span.start + 2
    }
}

/// Whether a word can be used as a plain identifier.
pub fn usable_as_ident(w: &str) -> bool {
    !is_reserved(w)
}
