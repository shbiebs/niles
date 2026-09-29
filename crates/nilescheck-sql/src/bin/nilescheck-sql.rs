//! `nilescheck-sql [--catalog-only] FILE…` — parse the files as one script, run every rule,
//! print one line per diagnostic (`file:line: error[CODE]: message`) and exit 1 if any is an
//! error. `--catalog-only` runs R2-04's catalog checker alone (E14's "PostgreSQL + checker"
//! column); without it, SQL+C+L (R2-05): linearity, conservation and capabilities as well.

fn main() {
    let mut files: Vec<String> = std::env::args().skip(1).collect();
    let catalog_only = files.iter().any(|f| f == "--catalog-only");
    files.retain(|f| f != "--catalog-only");
    if files.is_empty() {
        eprintln!("usage: nilescheck-sql FILE…  (the files are checked as one script, in order)");
        std::process::exit(2);
    }
    let mut src = String::new();
    let mut starts = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap_or_else(|e| {
            eprintln!("nilescheck-sql: cannot read {f}: {e}");
            std::process::exit(2);
        });
        starts.push((src.len(), f.clone()));
        src.push_str(&text);
        src.push('\n');
    }
    let locate = |at: usize| {
        let (base, f) = starts
            .iter()
            .rev()
            .find(|(s, _)| *s <= at)
            .cloned()
            .unwrap_or((0, files[0].clone()));
        let line = src[base..at.min(src.len())].lines().count().max(1);
        format!("{f}:{line}")
    };
    let (stmts, _) = match nilescheck_sql::parse(&src) {
        Ok(x) => x,
        Err(e) => {
            println!("{}: error[parse]: {}", locate(e.span.start), e.msg);
            std::process::exit(1);
        }
    };
    let diags = if catalog_only {
        nilescheck_sql::check_catalog(&stmts)
    } else {
        nilescheck_sql::check_all(&stmts)
    };
    for d in &diags {
        println!(
            "{}: {}[{}]: {}",
            locate(d.span.start),
            if d.error { "error" } else { "warning" },
            d.code,
            d.msg
        );
    }
    if diags.is_empty() {
        println!("ok: {} statements, no diagnostics", stmts.len());
    }
    std::process::exit(if diags.iter().any(|d| d.error) { 1 } else { 0 });
}
