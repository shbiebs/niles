//! The parser against texts PostgreSQL 16 accepts: the repository's own SQL (every file under
//! `crates/counterproposal/sql` and `crates/comparator/sql`), and a statement corpus covering
//! each construct the parser claims. A text PostgreSQL refuses is in `refused`.

mod corpus;

use nilescheck_sql::ast::*;
use nilescheck_sql::parse;

fn ok(src: &str) -> Vec<Stmt> {
    match parse(src) {
        Ok((s, _)) => s,
        Err(e) => panic!(
            "refused: {}\n  at {:?}: {:?}\n  in: {src}",
            e.msg,
            e.span,
            src.get(e.span.start..(e.span.end.max(e.span.start + 20)).min(src.len()))
        ),
    }
}

#[test]
fn the_repositorys_own_sql_parses() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut n = 0;
    for dir in ["counterproposal/sql", "comparator/sql"] {
        let Ok(rd) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("sql") {
                let src = std::fs::read_to_string(&p).unwrap();
                // psql meta-commands (`\set`, `\i`) are not SQL; strip those lines.
                let src: String = src
                    .lines()
                    .map(|l| {
                        if l.trim_start().starts_with('\\') {
                            ""
                        } else {
                            l
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Err(e) = parse(&src) {
                    panic!(
                        "{}: {} at {:?}: {:?}",
                        p.display(),
                        e.msg,
                        e.span,
                        &src[e.span.start..(e.span.start + 60).min(src.len())]
                    );
                }
                n += 1;
            }
        }
    }
    assert!(n >= 4, "found {n} files");
}

#[test]
fn queries() {
    for q in corpus::QUERIES {
        ok(q);
    }
    assert!(
        parse(corpus::ADJACENT).is_err(),
        "adjacent constants without a newline are an error"
    );
}

#[test]
fn dml() {
    for q in corpus::DML {
        ok(q);
    }
}

#[test]
fn ddl() {
    for q in corpus::DDL {
        ok(q);
    }
}

#[test]
fn plpgsql() {
    let src = corpus::PLPGSQL;
    let stmts = ok(src);
    let Some(Stmt::CreateFunction(f)) = stmts.iter().find(|s| matches!(s, Stmt::CreateFunction(_)))
    else {
        panic!("no function")
    };
    let b = f.plpgsql.as_ref().expect("the plpgsql body is parsed");
    assert_eq!(b.label.as_deref(), Some("outer"));
    assert_eq!(b.decls.len(), 5);
    assert!(b.body.len() >= 15, "{}", b.body.len());
    // Spans point into the file: the first statement's text is the select.
    let first = match &b.body[0] {
        nilescheck_sql::plpgsql::PlStmt::Sql {
            span, into, strict, ..
        } => {
            assert_eq!(into[0].dotted(), "v_bal");
            assert!(*strict);
            *span
        }
        other => panic!("{other:?}"),
    };
    assert!(src[first.start..first.end].starts_with("select sum(amt)"));
}

#[test]
fn refused() {
    for q in corpus::REFUSED {
        assert!(parse(q).is_err(), "should be refused: {q}");
    }
}
