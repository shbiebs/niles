//! The statements beyond queries, DML and the core DDL: `truncate`, `merge`, `call`,
//! `explain`, `grant`/`revoke`, `create sequence`, `create rule`, and the objects defined by a
//! `(key = value, …)` list or a fixed clause. Two of them matter to the checker directly —
//! `truncate` and `merge` can destroy or rewrite ledger history, and a `rule` can turn an
//! insert into an update — so they are parsed into trees rather than recognised by keyword.

use crate::ast::*;
use crate::lex::Tok;
use crate::parser::{PResult, Parser};

impl<'a> Parser<'a> {
    pub fn truncate(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("truncate")?;
        self.eat_kw("table");
        let mut tables = Vec::new();
        loop {
            self.eat_kw("only");
            tables.push(self.name()?);
            self.eat_op("*");
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        let _ = self.eat_kws(&["restart", "identity"]) || self.eat_kws(&["continue", "identity"]);
        let cascade = self.eat_kw("cascade");
        self.eat_kw("restrict");
        Ok(Stmt::Truncate {
            tables,
            cascade,
            span: self.span_from(start),
        })
    }

    pub fn merge(&mut self, with: Vec<Cte>, start: usize) -> PResult<Stmt> {
        self.expect_kw("merge")?;
        self.expect_kw("into")?;
        self.eat_kw("only");
        let table = self.name()?;
        self.eat_op("*");
        let alias = if self.eat_kw("as") || (self.can_be_bare_alias() && !self.is_kw("using")) {
            Some(self.ident()?)
        } else {
            None
        };
        self.expect_kw("using")?;
        let mut source = self.from_list()?;
        if source.len() != 1 {
            return self.err("`merge … using` takes one source");
        }
        let source = source.remove(0);
        self.expect_kw("on")?;
        let on = self.expr()?;
        let mut whens = Vec::new();
        while self.eat_kw("when") {
            let matched = if self.eat_kw("matched") {
                true
            } else {
                self.expect_kw("not")?;
                self.expect_kw("matched")?;
                // PostgreSQL 17 adds `by source` / `by target`; 16 does not.
                false
            };
            let condition = if self.eat_kw("and") {
                Some(self.expr()?)
            } else {
                None
            };
            self.expect_kw("then")?;
            let action = if self.eat_kws(&["do", "nothing"]) {
                MergeAction::DoNothing
            } else if matched && self.eat_kw("update") {
                self.expect_kw("set")?;
                MergeAction::Update(self.set_list()?)
            } else if matched && self.eat_kw("delete") {
                MergeAction::Delete
            } else if !matched && self.eat_kw("insert") {
                let columns = if self.is(&Tok::LParen) {
                    self.paren_ident_list()?
                } else {
                    vec![]
                };
                if self.eat_kw("overriding") {
                    let _ = self.eat_kw("system") || self.eat_kw("user");
                    self.expect_kw("value")?;
                }
                let values = if self.eat_kws(&["default", "values"]) {
                    None
                } else {
                    self.expect_kw("values")?;
                    self.expect(&Tok::LParen)?;
                    let mut v = vec![self.expr()?];
                    while self.eat(&Tok::Comma) {
                        v.push(self.expr()?);
                    }
                    self.expect(&Tok::RParen)?;
                    Some(v)
                };
                MergeAction::Insert { columns, values }
            } else {
                return self.err(if matched {
                    "`when matched` takes update, delete or do nothing"
                } else {
                    "`when not matched` takes insert or do nothing"
                });
            };
            whens.push(MergeWhen {
                matched,
                condition,
                action,
            });
        }
        if whens.is_empty() {
            return self.err("`merge` needs at least one `when` clause");
        }
        Ok(Stmt::Merge(Box::new(Merge {
            with,
            table,
            alias,
            source,
            on,
            whens,
            span: self.span_from(start),
        })))
    }

    pub fn call_stmt(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("call")?;
        let e = self.expr()?;
        if !matches!(e, Expr::Func(_)) {
            return self.err("`call` takes a procedure call");
        }
        Ok(Stmt::Call(Box::new(e), self.span_from(start)))
    }

    pub fn explain(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("explain")?;
        let opt_start = self.here().start;
        if self.is(&Tok::LParen) {
            self.pos += 1;
            self.skip_balanced(&[]);
            self.expect(&Tok::RParen)?;
        } else {
            let _ = self.eat_kw("analyze") || self.eat_kw("analyse");
            self.eat_kw("verbose");
        }
        let options = self.src[opt_start..self.prev_end().max(opt_start)].to_string();
        let stmt = self.statement()?;
        Ok(Stmt::Explain {
            options,
            stmt: Box::new(stmt),
            span: self.span_from(start),
        })
    }

    /// `grant privileges on [kind] objects to grantees [with grant option]`, or the role form
    /// `grant role, … to role, … [with admin option]`; `revoke` mirrors both.
    pub fn grant(&mut self, start: usize) -> PResult<Stmt> {
        let revoke = self.is_kw("revoke");
        self.pos += 1;
        if revoke {
            let _ = self.eat_kws(&["grant", "option", "for"])
                || self.eat_kws(&["admin", "option", "for"]);
        }
        let mut privileges = Vec::new();
        loop {
            let mut p = String::new();
            while let Some(w) = self.word() {
                if ["on", "to", "from"].contains(&w) {
                    break;
                }
                if !p.is_empty() {
                    p.push(' ');
                }
                p.push_str(w);
                self.pos += 1;
            }
            if let Some(Tok::QuotedIdent(q)) = self.peek().cloned() {
                self.pos += 1;
                p.push_str(&q);
            }
            if self.is(&Tok::LParen) {
                self.pos += 1;
                let cols = self.ident_list()?;
                self.expect(&Tok::RParen)?;
                p.push_str(&format!(" ({})", cols.join(", ")));
            }
            if p.is_empty() {
                return self.err("expected a privilege or a role");
            }
            privileges.push(p);
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        let (kind, objects) = if self.eat_kw("on") {
            const KINDS: &[&[&str]] = &[
                &["all", "tables", "in", "schema"],
                &["all", "sequences", "in", "schema"],
                &["all", "functions", "in", "schema"],
                &["all", "procedures", "in", "schema"],
                &["all", "routines", "in", "schema"],
                &["foreign", "data", "wrapper"],
                &["foreign", "server"],
                &["large", "object"],
                &["table"],
                &["sequence"],
                &["database"],
                &["domain"],
                &["function"],
                &["procedure"],
                &["routine"],
                &["language"],
                &["schema"],
                &["tablespace"],
                &["type"],
                &["parameter"],
            ];
            let kind = KINDS
                .iter()
                .find(|k| self.eat_kws(k))
                .map(|k| k.join(" "))
                .unwrap_or_else(|| "table".into());
            let mut objects = Vec::new();
            loop {
                objects.push(self.name()?);
                if self.is(&Tok::LParen) {
                    self.pos += 1;
                    self.skip_balanced(&[]);
                    self.expect(&Tok::RParen)?;
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            (kind, objects)
        } else {
            ("role".to_string(), vec![])
        };
        if !(self.eat_kw("to") || self.eat_kw("from")) {
            return self.err(if revoke {
                "expected `from`"
            } else {
                "expected `to`"
            });
        }
        let mut grantees = Vec::new();
        loop {
            self.eat_kw("group");
            grantees.push(self.any_word()?);
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        let _ = self.eat_kws(&["with", "grant", "option"])
            || self.eat_kws(&["with", "admin", "option"])
            || self.eat_kws(&["with", "inherit", "true"])
            || self.eat_kws(&["with", "inherit", "false"]);
        if self.eat_kws(&["granted", "by"]) {
            self.any_word()?;
        }
        let _ = self.eat_kw("cascade") || self.eat_kw("restrict");
        Ok(Stmt::Grant {
            revoke,
            privileges,
            kind,
            objects,
            grantees,
            span: self.span_from(start),
        })
    }

    pub fn create_sequence(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("sequence")?;
        self.eat_kws(&["if", "not", "exists"]);
        let name = self.name()?;
        let options = self.rest_of_statement();
        Ok(Stmt::CreateSequence {
            name,
            options,
            span: self.span_from(start),
        })
    }

    pub fn create_rule(&mut self, start: usize) -> PResult<Stmt> {
        self.expect_kw("rule")?;
        let name = self.ident()?;
        self.expect_kw("as")?;
        self.expect_kw("on")?;
        let event = self.any_word()?;
        if !["select", "insert", "update", "delete"].contains(&event.as_str()) {
            return self.err("a rule's event is select, insert, update or delete");
        }
        self.expect_kw("to")?;
        let table = self.name()?;
        let where_ = if self.eat_kw("where") {
            Some(self.expr()?)
        } else {
            None
        };
        self.expect_kw("do")?;
        let instead = self.eat_kw("instead");
        if !instead {
            self.eat_kw("also");
        }
        let mut actions = Vec::new();
        if self.eat_kw("nothing") {
        } else if self.eat(&Tok::LParen) {
            loop {
                while self.eat(&Tok::Semi) {}
                if self.eat(&Tok::RParen) {
                    break;
                }
                actions.push(self.statement()?);
            }
        } else {
            actions.push(self.statement()?);
        }
        Ok(Stmt::CreateRule {
            name,
            event,
            table,
            where_,
            instead,
            actions,
            span: self.span_from(start),
        })
    }

    /// `create <kind> name [(args)] ( key = value, … )` and the clause-shaped definitions
    /// (`create cast`, `create operator class … as …`, `create extension …`, `create policy …`,
    /// `create role …`). Items are kept as written: the checker has no rule that reads them,
    /// and a definition whose keys PostgreSQL does not know is PostgreSQL's to refuse.
    pub fn define(&mut self, kind: &str, start: usize) -> PResult<Stmt> {
        let name = match kind {
            "cast" => Name(vec!["cast".into()]),
            // `create user mapping for role server s …`, `create transform for type language l`:
            // no name of their own.
            "user mapping" | "transform" => Name(vec![]),
            "operator" => {
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
            }
            _ => {
                self.eat_kws(&["if", "not", "exists"]);
                self.name()?
            }
        };
        let mut items = Vec::new();
        // An argument list before the definition (`create aggregate f(int) (…)`), or the whole
        // of a cast's source and target (`create cast (a as b)`).
        let has_args = matches!(kind, "aggregate" | "cast");
        if has_args && self.is(&Tok::LParen) {
            self.pos += 1;
            items.push(("args".to_string(), self.skip_balanced(&[])));
            self.expect(&Tok::RParen)?;
        }
        if self.is(&Tok::LParen) {
            self.pos += 1;
            loop {
                if self.is(&Tok::RParen) {
                    break;
                }
                let key = self.any_word()?;
                let value = if self.eat_op("=") {
                    self.skip_balanced_comma()
                } else {
                    String::new()
                };
                items.push((key, value));
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RParen)?;
        }
        let rest = self.rest_of_statement();
        if !rest.trim().is_empty() {
            items.push(("clause".into(), rest));
        }
        Ok(Stmt::Define {
            kind: kind.to_string(),
            name,
            items,
            span: self.span_from(start),
        })
    }

    /// A value in a definition list: tokens up to a `,` or `)` at depth 0.
    fn skip_balanced_comma(&mut self) -> String {
        let start = self.here().start;
        let mut depth = 0i32;
        while let Some(t) = self.peek() {
            match t {
                Tok::LParen | Tok::LBracket => depth += 1,
                Tok::RParen | Tok::RBracket if depth == 0 => break,
                Tok::RParen | Tok::RBracket => depth -= 1,
                Tok::Comma if depth == 0 => break,
                Tok::Semi => break,
                _ => {}
            }
            self.pos += 1;
        }
        self.src[start..self.prev_end().max(start)].to_string()
    }
}
