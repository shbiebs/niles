//! nilescheck-sql — a hand-written PostgreSQL 16 SQL and PL/pgSQL parser, and the catalog
//! checker over it (cycle 14, R2-04). No external dependency (the author's decision of
//! 2026-09-28: a full hand-written parser).

pub mod ast;
pub mod capability;
pub mod check;
pub mod check2;
pub mod conserve;
pub mod conventions;
pub mod effects;
pub mod lex;
pub mod linear;
pub mod parser;
pub mod plpgsql;
pub mod stmt;
pub mod stmt2;
pub mod typing;

/// Parse a script: every statement, comments set aside.
pub fn parse(src: &str) -> Result<(Vec<ast::Stmt>, Vec<lex::Token>), parser::ParseError> {
    let toks = lex::lex(src).map_err(|e| parser::ParseError {
        span: e.span,
        msg: e.msg,
    })?;
    let (toks, comments) = lex::split_comments(toks);
    let mut p = parser::Parser::new(src, toks);
    let stmts = p.statements()?;
    Ok((stmts, comments))
}

/// Parse a script statement by statement, recovering after a statement that does not parse:
/// the failing statement is skipped to its `;` and reported, and parsing continues. For
/// measuring coverage over large real scripts; the checker itself uses [`parse`], which stops
/// at the first error.
pub fn parse_recovering(src: &str) -> (Vec<ast::Stmt>, Vec<parser::ParseError>) {
    let toks = match lex::lex(src) {
        Ok(t) => t,
        Err(e) => {
            return (
                vec![],
                vec![parser::ParseError {
                    span: e.span,
                    msg: format!("lexical: {}", e.msg),
                }],
            )
        }
    };
    let (toks, _) = lex::split_comments(toks);
    let mut p = parser::Parser::new(src, toks);
    let (mut stmts, mut errs) = (Vec::new(), Vec::new());
    loop {
        while p.eat(&lex::Tok::Semi) {}
        if p.at_end() {
            return (stmts, errs);
        }
        let start = p.pos;
        match p.statement() {
            Ok(s) if p.at_end() || p.is(&lex::Tok::Semi) => stmts.push(s),
            Ok(_) => {
                errs.push(parser::ParseError {
                    span: p.here(),
                    msg: "expected `;` after the statement".into(),
                });
                p.pos = start;
                p.skip_statement();
            }
            Err(e) => {
                errs.push(e);
                p.pos = start;
                p.skip_statement();
            }
        }
    }
}

/// **The catalog checker** (R2-04, E14's "PostgreSQL + checker" column): the serve-contract
/// family (`check.rs`) and the typed and whole-script rules (`check2.rs`).
pub fn check_catalog(stmts: &[ast::Stmt]) -> Vec<check::Diag> {
    let mut d = check::check(stmts);
    d.extend(check2::check(stmts));
    d
}

/// **SQL+C+L** (R2-05, E14's column of that name): the catalog checker, plus linearity
/// (`linear.rs`), conservation through `niles-lang`'s solver (`conserve.rs`) and capabilities
/// (`capability.rs`).
pub fn check_all(stmts: &[ast::Stmt]) -> Vec<check::Diag> {
    let mut d = check_catalog(stmts);
    d.extend(linear::check(stmts));
    d.extend(conserve::check(stmts));
    d.extend(capability::check(stmts));
    d.extend(effects::check(stmts));
    // The typing rules re-derive NL0250 where `check2` already found it on columns: a
    // typing report at the same place as an existing one of its code is not added twice.
    let mut seen: std::collections::BTreeSet<(&str, usize, usize)> =
        d.iter().map(|x| (x.code, x.span.start, x.span.end)).collect();
    for x in typing::check(stmts) {
        if seen.insert((x.code, x.span.start, x.span.end)) {
            d.push(x);
        }
    }
    d
}
