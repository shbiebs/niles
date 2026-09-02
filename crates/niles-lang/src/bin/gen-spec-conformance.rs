//! Generates Part V of `docs/SPEC-LANGUAGE.md` — the conformance summary — from the
//! per-requirement sections above it.
//!
//! Part V was a hand-maintained table beside twenty sections that each carry a status line,
//! and it had drifted in three separate ways at once: four requirements (L-3, L-4, L-7, L-14)
//! were counted **Built** with no section anywhere in the document; the prose said "Ten of
//! twenty-four built" while its own row totals summed to eleven; and L-8/L-9's row said
//! `Specified` beside a section describing a checker that exists.
//!
//! A summary that counts sections must be computed from the sections. This binary reads them,
//! and `crates/niles-lang/tests/spec_conformance.rs` fails if the checked-in Part V differs
//! from what it would produce — the same discipline `gen-keyword-ref` applies to Appendix B
//! and `gen-sql-surface` to the SQL surface table.
//!
//! Run it with `cargo run -p niles-lang --bin gen-spec-conformance`.

use std::collections::BTreeMap;
use std::fmt::Write;

pub const BEGIN: &str = "<!-- BEGIN:conformance -->";
pub const END: &str = "<!-- END:conformance -->";

/// One requirement's heading and the status its section declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub ids: Vec<String>,
    pub title: String,
    pub status: String,
}

/// The part of the document a requirement's section sits in.
fn part_of(heading_line: usize, part_starts: &[(usize, String)]) -> String {
    let mut current = "Unassigned".to_string();
    for (at, name) in part_starts {
        if *at < heading_line {
            current = name.clone();
        }
    }
    current
}

/// Read every `### L-… ` section and the status it declares.
///
/// A section with no status line is an error rather than a default: "the status is whatever
/// the summary says" is the failure this generator exists to remove.
pub fn parse(doc: &str) -> Result<Vec<(String, Requirement)>, String> {
    let lines: Vec<&str> = doc.lines().collect();
    let mut part_starts = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if let Some(rest) = l.strip_prefix("## Part ") {
            let name = rest
                .split_once('—')
                .map(|(_, n)| n.trim().to_string())
                .unwrap_or_else(|| rest.trim().to_string());
            part_starts.push((i, name));
        }
    }

    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let Some(rest) = l.strip_prefix("### L-") else {
            continue;
        };
        // `L-16 / L-22 No interpretive overhead…` — one section, two requirements.
        let mut ids = Vec::new();
        let mut words = rest.split_whitespace().peekable();
        ids.push(format!("L-{}", words.next().unwrap_or_default()));
        while words.peek() == Some(&"/") {
            words.next();
            if let Some(w) = words.next() {
                ids.push(w.trim_start_matches("L-").to_string());
                let last = ids.len() - 1;
                if !ids[last].starts_with("L-") {
                    ids[last] = format!("L-{}", ids[last]);
                }
            }
        }
        let title = words.collect::<Vec<_>>().join(" ");

        // The status line: the first `Status: X` at or after the heading, before the next one.
        let end = lines[i + 1..]
            .iter()
            .position(|l| l.starts_with("### ") || l.starts_with("## "))
            .map(|p| i + 1 + p)
            .unwrap_or(lines.len());
        let mut status = None;
        for l in &lines[i..end] {
            if let Some(at) = l.find("Status:") {
                let tail = &l[at + "Status:".len()..];
                let word = tail
                    .trim_start_matches([' ', '*'])
                    .split([' ', '*', '.', ',', '—', ':'])
                    .find(|w| !w.is_empty())
                    .unwrap_or("");
                if !word.is_empty() {
                    status = Some(word.to_string());
                    break;
                }
            }
        }
        let Some(status) = status else {
            return Err(format!(
                "section `### L-{rest}` declares no `Status:`; the summary would have to \
                 invent one, which is how L-8/L-9 came to read `Specified` beside a checker \
                 that exists"
            ));
        };
        out.push((part_of(i, &part_starts), Requirement { ids, title, status }));
    }
    Ok(out)
}

/// Render Part V's table and its sentence.
pub fn render(doc: &str) -> Result<String, String> {
    let reqs = parse(doc)?;
    let mut by_part: BTreeMap<String, Vec<&Requirement>> = BTreeMap::new();
    let mut order = Vec::new();
    for (part, r) in &reqs {
        if !order.contains(part) {
            order.push(part.clone());
        }
        by_part.entry(part.clone()).or_default().push(r);
    }

    let mut totals: BTreeMap<String, usize> = BTreeMap::new();
    let mut s = String::new();
    s.push_str("| Part | Requirements | Built | Partial | Specified | Adopt |\n");
    s.push_str("|---|---|---|--:|--:|--:|\n");
    for part in &order {
        let rs = &by_part[part];
        let mut ids: Vec<String> = rs.iter().flat_map(|r| r.ids.clone()).collect();
        ids.sort_by_key(|i| i.trim_start_matches("L-").parse::<u32>().unwrap_or(0));
        let count = |want: &str| {
            rs.iter()
                .filter(|r| r.status.eq_ignore_ascii_case(want))
                .count()
        };
        for w in ["Built", "Partial", "Specified", "Adopt"] {
            *totals.entry(w.to_string()).or_default() += count(w);
        }
        let _ = writeln!(
            s,
            "| {part} | {} | {} | {} | {} | {} |",
            ids.join(", "),
            count("Built"),
            count("Partial"),
            count("Specified"),
            count("Adopt"),
        );
    }

    let sections = reqs.len();
    let requirements: usize = reqs.iter().map(|(_, r)| r.ids.len()).sum();
    let b = totals.get("Built").copied().unwrap_or(0);
    let p = totals.get("Partial").copied().unwrap_or(0);
    let sp = totals.get("Specified").copied().unwrap_or(0);
    let a = totals.get("Adopt").copied().unwrap_or(0);
    let _ = write!(
        s,
        "\n**{sections} sections covering {requirements} requirement identifiers: \
         {b} Built, {p} Partial, {sp} Specified, {a} Adopt.** Counted from the sections \
         themselves by `cargo run -p niles-lang --bin gen-spec-conformance`, because the \
         hand-maintained version of this table counted four requirements that had no section \
         and printed a total its own rows did not sum to.\n"
    );
    Ok(s)
}

#[allow(dead_code)]
fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/SPEC-LANGUAGE.md".to_string());
    let doc = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    let block = render(&doc).unwrap_or_else(|e| panic!("{e}"));

    let check = std::env::args().any(|a| a == "--check");
    let Some(start) = doc.find(BEGIN) else {
        panic!("{path} has no `{BEGIN}` marker; add it under `## Part V`");
    };
    let Some(end) = doc.find(END) else {
        panic!("{path} has no `{END}` marker");
    };
    let want = format!("{BEGIN}\n\n{block}\n{END}");
    let have = &doc[start..end + END.len()];
    if have == want {
        println!("{path}: Part V is current");
        return;
    }
    if check {
        eprintln!(
            "{path}: Part V is stale; run `cargo run -p niles-lang --bin gen-spec-conformance`"
        );
        std::process::exit(1);
    }
    let out = format!("{}{}{}", &doc[..start], want, &doc[end + END.len()..]);
    std::fs::write(&path, out).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
    println!("updated {path}");
}
