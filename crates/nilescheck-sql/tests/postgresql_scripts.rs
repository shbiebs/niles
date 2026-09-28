//! **Coverage over PostgreSQL's own SQL**: the scripts PostgreSQL 16 ships and runs at
//! `initdb` and `create extension` time (`/usr/share/postgresql/16/*.sql` and
//! `extension/*.sql`) — real DDL and queries this parser did not see while it was written.
//! Every statement either parses or is reported with its error; the share that parses is
//! printed and held to a floor. Skipped with a message when the directory is absent (a host
//! with no PostgreSQL 16 server package).

use std::path::Path;

fn corpus() -> Vec<std::path::PathBuf> {
    let base = Path::new("/usr/share/postgresql/16");
    let mut v = Vec::new();
    for dir in [base.to_path_buf(), base.join("extension")] {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("sql") {
                    v.push(p);
                }
            }
        }
    }
    v.sort();
    v
}

/// Lines an extension script starts with that are psql meta-commands, not SQL.
fn strip_meta(src: &str) -> String {
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
    // Extension scripts are templates: `CREATE EXTENSION` replaces `@extschema@` and
    // `@extschema:name@` with a schema name before PostgreSQL parses them, so the parser is
    // given what PostgreSQL is given.
    let mut out = String::with_capacity(src.len());
    let mut rest = src.as_str();
    while let Some(i) = rest.find("@extschema") {
        out.push_str(&rest[..i]);
        let tail = &rest[i + 1..];
        match tail.find('@') {
            Some(j) if !tail[..j].contains(char::is_whitespace) => {
                out.push_str("ext");
                rest = &tail[j + 1..];
            }
            _ => {
                out.push('@');
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}

#[test]
fn postgresqls_own_scripts_parse() {
    let files = corpus();
    if files.is_empty() {
        eprintln!("no /usr/share/postgresql/16: skipped (nothing to measure on this host)");
        return;
    }
    let (mut ok, mut bad) = (0usize, Vec::new());
    let mut other: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for f in &files {
        let src = strip_meta(&std::fs::read_to_string(f).unwrap());
        let (stmts, errs) = nilescheck_sql::parse_recovering(&src);
        ok += stmts.len();
        for st in &stmts {
            if let nilescheck_sql::ast::Stmt::Other { keywords, .. } = st {
                *other
                    .entry(
                        keywords
                            .iter()
                            .take(2)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" "),
                    )
                    .or_insert(0usize) += 1;
            }
        }
        for e in errs {
            let line = src[..e.span.start.min(src.len())].lines().count();
            let text: String = src[e.span.start.min(src.len())..]
                .chars()
                .take(70)
                .collect();
            bad.push(format!(
                "{}:{line}: {} — {:?}",
                f.file_name().unwrap().to_string_lossy(),
                e.msg,
                text
            ));
        }
    }
    let total = ok + bad.len();
    let recognised: usize = other.values().sum();
    eprintln!(
        "{} files, {total} statements, {ok} parsed ({} into a modelled tree, {recognised} recognised by keyword only), {} refused",
        files.len(),
        ok - recognised,
        bad.len()
    );
    let mut kinds: Vec<_> = other.iter().collect();
    kinds.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!(
        "  recognised only: {}",
        kinds
            .iter()
            .map(|(k, n)| format!("{k} {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let show = if std::env::var("NSQ_ALL").is_ok() {
        usize::MAX
    } else {
        60
    };
    for b in bad.iter().take(show) {
        eprintln!("  {b}");
    }
    let share = ok as f64 / total as f64;
    assert!(
        share >= FLOOR,
        "parsed {:.2}% of PostgreSQL's own statements, below the floor {:.2}%",
        share * 100.0,
        FLOOR * 100.0
    );
}

/// Raised as coverage grows; never lowered to make a run pass.
const FLOOR: f64 = 1.0;
