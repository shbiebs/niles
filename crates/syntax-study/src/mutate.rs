//! **The mutation operators** (design §6, safety), applied mechanically at every site where
//! each applies. A site is found on the program's tokens ([`crate::tokens`]) by the per-surface
//! patterns below, and a mutant is the program's source with one span replaced — so a mutant
//! differs from its program in exactly one edit.
//!
//! The site patterns, per operator (the table in `results/E30-syntax.md` repeats them):
//!
//! * **M1 currency swap** — every occurrence of a currency name in a word or a string, as a
//!   whole word or as an `_`-separated part (`amt_usd`, `Money<usd>`, `"usd"`, `'usd'`):
//!   usd → eur, eur → usd, jpy → usd.
//! * **M2 debit ↔ credit** — Niles: every `debit(` becomes `credit(` and every `credit(`
//!   becomes `debit(`. SQL, inside an `insert into postings`: every `row(x)::c` has its
//!   sign toggled (`row(-(m).minor)::usd` → `row((m).minor)::usd`, `row(e)::usd` →
//!   `row(-(e))::usd`), and every bare money variable in a `values` tuple's `amt_c` position
//!   `m` becomes `row(-(m).minor)::c`.
//! * **M3 drop a conjunct** — in each condition, each top-level conjunct is deleted with its
//!   connective; a condition with one conjunct loses the whole clause. Conditions: SQL
//!   `where`, `having`, `on` (to the clause's end); PL/pgSQL `if`/`elsif` (to `then`); Niles
//!   `.where`/`.filter`/`.having` closure bodies and a join closure's body; Niles `if` (to
//!   `{`); PRQL `filter` (to the line's end) and `join … (cond)`; Soufflé rule bodies (`:-` to
//!   `.`, conjunct `,`, only with two or more). An `if` with one conjunct is a site only when
//!   it guards a failure (its body raises or returns `Err`), where the mutant makes the guard
//!   `false`; an `if` that chooses between two computations has no clause to drop.
//! * **M4 epoch + 1** — Niles `#e`; SQL and PRQL a number compared with `epoch`.
//! * **M5 remove a consumption or the conservation statement** — SQL `perform resolve_…(…);`
//!   and Niles `resolve …` (to its statement's end); Niles `conserve …;` and SQL `comment on
//!   … is '…'` whose annotation names `consumes`, `linear` or `conserve`.
//! * **M6 weaken a contract** — every consistency rung, as a word or a string's text, one
//!   step down the ladder: bounded < monotonic < read_your_writes < snapshot < serializable <
//!   ledger_consistent (`bounded` has no step down).

// Token scanning moves between indices (a bracket's partner, a clause's end), so index loops
// are the natural form here.
#![allow(clippy::needless_range_loop)]

use crate::tokens::{tokenize, Kind, Token};

#[derive(Debug, Clone)]
pub struct Mutant {
    pub op: &'static str,
    /// The site's ordinal within its operator, in source order.
    pub site: usize,
    pub line: u32,
    pub what: String,
    pub text: String,
}

struct Edit {
    op: &'static str,
    start: usize,
    end: usize,
    with: String,
    line: u32,
    what: String,
}

pub const LADDER: &[&str] = &[
    "bounded",
    "monotonic",
    "read_your_writes",
    "snapshot",
    "serializable",
    "ledger_consistent",
];

fn lw(t: &Token) -> String {
    t.text.to_lowercase()
}

fn is(t: Option<&Token>, s: &str) -> bool {
    t.is_some_and(|t| t.text.eq_ignore_ascii_case(s))
}

/// The index of the bracket closing the one at `open`, if any.
fn closing(toks: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (i, t) in toks.iter().enumerate().skip(open) {
        match t.text.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------- M1

fn m1(src: &str, toks: &[Token], out: &mut Vec<Edit>) {
    for t in toks
        .iter()
        .filter(|t| matches!(t.kind, Kind::Word | Kind::Str))
    {
        let low = t.text.to_lowercase();
        let b = low.as_bytes();
        for (from, to) in [("usd", "eur"), ("eur", "usd"), ("jpy", "usd")] {
            let mut at = 0;
            while let Some(p) = low[at..].find(from) {
                let s = at + p;
                let e = s + 3;
                let before_ok = s == 0 || !b[s - 1].is_ascii_alphanumeric();
                let after_ok = e == b.len() || !b[e].is_ascii_alphanumeric();
                if before_ok && after_ok {
                    let orig = &src[t.start + s..t.start + e];
                    let rep = if orig.chars().all(|c| c.is_uppercase()) {
                        to.to_uppercase()
                    } else {
                        to.to_string()
                    };
                    out.push(Edit {
                        op: "M1",
                        start: t.start + s,
                        end: t.start + e,
                        with: rep,
                        line: t.line,
                        what: format!("{from} → {to} in `{}`", t.text),
                    });
                }
                at = e;
            }
        }
    }
}

// ---------------------------------------------------------------- M2

fn m2(src: &str, toks: &[Token], surface: &str, out: &mut Vec<Edit>) {
    if surface != "SQL" {
        for (i, t) in toks.iter().enumerate() {
            if t.kind == Kind::Word && is(toks.get(i + 1), "(") {
                let to = match t.text.as_str() {
                    "debit" => "credit",
                    "credit" => "debit",
                    _ => continue,
                };
                out.push(Edit {
                    op: "M2",
                    start: t.start,
                    end: t.end,
                    with: to.into(),
                    line: t.line,
                    what: format!("{} → {to}", t.text),
                });
            }
        }
        return;
    }
    // SQL: inside each `insert into postings … ;`.
    let mut i = 0;
    while i + 2 < toks.len() {
        if !(is(toks.get(i), "insert")
            && is(toks.get(i + 1), "into")
            && is(toks.get(i + 2), "postings"))
        {
            i += 1;
            continue;
        }
        let end = (i..toks.len())
            .find(|&k| toks[k].text == ";")
            .unwrap_or(toks.len());
        // The column list, when there is one.
        let mut cols: Vec<String> = Vec::new();
        let mut k = i + 3;
        if is(toks.get(k), "(") {
            let c = closing(toks, k).unwrap_or(k);
            cols = toks[k + 1..c]
                .iter()
                .filter(|t| t.kind == Kind::Word)
                .map(lw)
                .collect();
            k = c + 1;
        }
        // (a) every `row( … )::c`.
        for r in k..end {
            if is(toks.get(r), "row") && is(toks.get(r + 1), "(") {
                let Some(c) = closing(toks, r + 1) else {
                    continue;
                };
                if !is(toks.get(c + 1), "::") {
                    continue;
                }
                let inner = &toks[r + 2..c];
                if let Some(first) = inner.first() {
                    if first.text == "-" {
                        out.push(Edit {
                            op: "M2",
                            start: first.start,
                            end: first.end,
                            with: String::new(),
                            line: first.line,
                            what: format!(
                                "the sign of `{}` removed",
                                &src[toks[r].start..toks[c + 2].end]
                            ),
                        });
                    } else {
                        let (s, e) = (first.start, inner.last().expect("nonempty").end);
                        out.push(Edit {
                            op: "M2",
                            start: s,
                            end: e,
                            with: format!("-({})", &src[s..e]),
                            line: first.line,
                            what: format!("`{}` negated", &src[toks[r].start..toks[c + 2].end]),
                        });
                    }
                }
            }
        }
        // (b) bare money variables in `values` tuples.
        if is(toks.get(k), "values") {
            let mut t = k + 1;
            while t < end && toks[t].text == "(" {
                let c = closing(toks, t).unwrap_or(end);
                // Split the tuple at depth-0 commas.
                let mut elems: Vec<(usize, usize)> = Vec::new();
                let mut depth = 0;
                let mut s = t + 1;
                for x in t + 1..c {
                    match toks[x].text.as_str() {
                        "(" | "[" => depth += 1,
                        ")" | "]" => depth -= 1,
                        "," if depth == 0 => {
                            elems.push((s, x));
                            s = x + 1;
                        }
                        _ => {}
                    }
                }
                elems.push((s, c));
                for (n, (a, b)) in elems.iter().enumerate() {
                    let Some(col) = cols.get(n) else { continue };
                    let Some(cur) = col.strip_prefix("amt_") else {
                        continue;
                    };
                    if b - a == 1 && toks[*a].kind == Kind::Word && !is(toks.get(*a), "null") {
                        let v = &toks[*a];
                        out.push(Edit {
                            op: "M2",
                            start: v.start,
                            end: v.end,
                            with: format!("row(-({}).minor)::{cur}", v.text),
                            line: v.line,
                            what: format!("`{}` negated", v.text),
                        });
                    }
                }
                t = c + 1;
                if is(toks.get(t), ",") {
                    t += 1;
                }
            }
        }
        i = end;
    }
}

// ---------------------------------------------------------------- M3

/// A condition: its tokens `[from, to)`, the connective, and the span the whole clause
/// occupies (deleted when the condition has one conjunct), or `None` when a one-conjunct
/// condition is not a site.
struct Cond {
    from: usize,
    to: usize,
    conj: &'static [&'static str],
    clause: Option<(usize, usize)>,
    guard_false: bool,
    label: String,
}

const SQL_END: &[&str] = &[
    "group",
    "order",
    "limit",
    "having",
    "union",
    "except",
    "intersect",
    "returning",
    "window",
    "join",
    "left",
    "right",
    "inner",
    "full",
    "cross",
    "where",
    "then",
    "loop",
    ";",
    "$$",
    "on",
    "offset",
];

/// From `from`, the first index at depth 0 that is in `ends`, or where the depth goes negative.
fn scan(toks: &[Token], from: usize, ends: &[&str], line_end: bool) -> usize {
    let mut depth = 0i32;
    let line = toks.get(from).map(|t| t.line).unwrap_or(0);
    for (i, t) in toks.iter().enumerate().skip(from) {
        if depth == 0 && i > from && line_end && t.line != line {
            return i;
        }
        if depth == 0 && i > from && ends.iter().any(|e| t.text.eq_ignore_ascii_case(e)) {
            return i;
        }
        match t.text.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => {
                depth -= 1;
                if depth < 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    toks.len()
}

fn body_guards_failure(toks: &[Token], from: usize, to: usize) -> bool {
    toks[from..to.min(toks.len())]
        .iter()
        .any(|t| matches!(lw(t).as_str(), "raise" | "err"))
}

fn conditions(toks: &[Token], surface: &str) -> Vec<Cond> {
    let mut out = Vec::new();
    let niles = matches!(surface, "NL" | "RS");
    for (i, t) in toks.iter().enumerate() {
        let w = lw(t);
        let prev_dot = i > 0 && toks[i - 1].text == ".";
        match surface {
            "DL" => {
                if t.text == ":-" {
                    let to = scan(toks, i + 1, &["."], false);
                    out.push(Cond {
                        from: i + 1,
                        to,
                        conj: &[","],
                        clause: None,
                        guard_false: false,
                        label: "rule body".into(),
                    });
                }
            }
            "PRQL" => {
                if w == "filter" {
                    let to = scan(toks, i + 1, &[], true);
                    out.push(Cond {
                        from: i + 1,
                        to,
                        conj: &["&&"],
                        clause: Some((i, to)),
                        guard_false: false,
                        label: "filter".into(),
                    });
                }
                if w == "join" {
                    // `join [side:x] rel (cond)`: the first parenthesised group on the line.
                    if let Some(open) = (i + 1..toks.len())
                        .take_while(|&k| toks[k].line == t.line)
                        .find(|&k| toks[k].text == "(")
                    {
                        if let Some(c) = closing(toks, open) {
                            out.push(Cond {
                                from: open + 1,
                                to: c,
                                conj: &["&&"],
                                clause: None,
                                guard_false: false,
                                label: "join condition".into(),
                            });
                        }
                    }
                }
            }
            _ => {
                // SQL clauses — on the SQL surface, and inside a Niles `sql { … }` block.
                // `on` is a join condition only after a `join`: not `create index … on t`,
                // `comment on …`, or a Niles `index … on postings (…)`.
                let join_on = w == "on"
                    && toks[..i]
                        .iter()
                        .rev()
                        .map(lw)
                        .find(|x| {
                            matches!(
                                x.as_str(),
                                "join"
                                    | "index"
                                    | "comment"
                                    | "trigger"
                                    | "policy"
                                    | "grant"
                                    | "revoke"
                                    | ";"
                            )
                        })
                        .is_some_and(|x| x == "join");
                if !prev_dot
                    && (matches!(w.as_str(), "where" | "having") || join_on)
                    && t.kind == Kind::Word
                {
                    let to = scan(toks, i + 1, SQL_END, false);
                    out.push(Cond {
                        from: i + 1,
                        to,
                        conj: &["and"],
                        clause: Some((i, to)),
                        guard_false: false,
                        label: w.clone(),
                    });
                }
                if !niles && matches!(w.as_str(), "if" | "elsif") {
                    let to = scan(toks, i + 1, &["then"], false);
                    let end_if = scan(toks, to, &["end", "elsif", "else"], false);
                    let guard = body_guards_failure(toks, to, end_if);
                    out.push(Cond {
                        from: i + 1,
                        to,
                        conj: &["and"],
                        clause: None,
                        guard_false: guard,
                        label: w.clone(),
                    });
                }
                if niles && w == "if" && !prev_dot {
                    let to = scan(toks, i + 1, &["{"], false);
                    let body_end = if toks.get(to).is_some_and(|x| x.text == "{") {
                        closing(toks, to).unwrap_or(to)
                    } else {
                        to
                    };
                    let guard = body_guards_failure(toks, to, body_end);
                    out.push(Cond {
                        from: i + 1,
                        to,
                        conj: &["&&", "and"],
                        clause: None,
                        guard_false: guard,
                        label: "if".into(),
                    });
                }
                // Niles stage closures.
                if niles && prev_dot && is(toks.get(i + 1), "(") {
                    let join = matches!(
                        w.as_str(),
                        "join" | "left_join" | "right_join" | "full_outer_join"
                    );
                    let filter = matches!(w.as_str(), "where" | "filter" | "having");
                    if !(join || filter) {
                        continue;
                    }
                    let open = i + 1;
                    let Some(close) = closing(toks, open) else {
                        continue;
                    };
                    // The closure: after the last depth-0 comma for a join, else the whole arg.
                    let mut start = open + 1;
                    if join {
                        let mut depth = 0;
                        let mut comma = None;
                        for k in open + 1..close {
                            match toks[k].text.as_str() {
                                "(" | "[" | "{" => depth += 1,
                                ")" | "]" | "}" => depth -= 1,
                                "," if depth == 0 => comma = Some(k),
                                _ => {}
                            }
                        }
                        let Some(c) = comma else { continue };
                        start = c + 1;
                    }
                    // Skip `|params|`.
                    if toks.get(start).is_some_and(|x| x.text == "|") {
                        if let Some(p) = (start + 1..close).find(|&k| toks[k].text == "|") {
                            start = p + 1;
                        }
                    } else if toks.get(start).is_some_and(|x| x.text == "||") {
                        start += 1;
                    }
                    out.push(Cond {
                        from: start,
                        to: close,
                        conj: &["&&", "and"],
                        // A one-conjunct filter stage is dropped whole: `.where( … )`.
                        clause: filter.then_some((i - 1, close + 1)),
                        guard_false: false,
                        label: format!(".{w}"),
                    });
                }
            }
        }
    }
    out
}

fn m3(src: &str, toks: &[Token], surface: &str, out: &mut Vec<Edit>) {
    for c in conditions(toks, surface) {
        if c.from >= c.to {
            continue;
        }
        // Split at depth-0 connectives, skipping the `and` of a `between`.
        let mut parts: Vec<(usize, usize)> = Vec::new();
        let mut conn: Vec<usize> = Vec::new();
        let mut depth = 0;
        let mut s = c.from;
        let mut between = false;
        for k in c.from..c.to {
            let t = &toks[k];
            match t.text.as_str() {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth -= 1,
                _ => {}
            }
            if depth != 0 {
                continue;
            }
            if lw(t) == "between" {
                between = true;
                continue;
            }
            if c.conj.iter().any(|x| t.text.eq_ignore_ascii_case(x)) {
                if between && lw(t) == "and" {
                    between = false;
                    continue;
                }
                parts.push((s, k));
                conn.push(k);
                s = k + 1;
            }
        }
        parts.push((s, c.to));
        let text = |a: usize, b: usize| src[toks[a].start..toks[b - 1].end].to_string();
        if parts.len() == 1 {
            if let Some((a, b)) = c.clause {
                let end = if b >= toks.len() {
                    src.len()
                } else {
                    toks[b - 1].end
                };
                out.push(Edit {
                    op: "M3",
                    start: toks[a].start,
                    end,
                    with: String::new(),
                    line: toks[a].line,
                    what: format!("`{}` clause dropped", c.label),
                });
            } else if c.guard_false {
                out.push(Edit {
                    op: "M3",
                    start: toks[c.from].start,
                    end: toks[c.to - 1].end,
                    with: "false".into(),
                    line: toks[c.from].line,
                    what: format!("`{}` guard `{}` made false", c.label, text(c.from, c.to)),
                });
            }
            continue;
        }
        for (n, (a, b)) in parts.iter().enumerate() {
            if a >= b {
                continue;
            }
            // The conjunct and its connective, with the whitespace between them.
            let (start, end) = if n == 0 {
                (toks[*a].start, toks[conn[0] + 1].start)
            } else {
                (toks[conn[n - 1] - 1].end, toks[b - 1].end)
            };
            out.push(Edit {
                op: "M3",
                start,
                end,
                with: String::new(),
                line: toks[*a].line,
                what: format!("conjunct `{}` of `{}` dropped", text(*a, *b), c.label),
            });
        }
    }
}

// ---------------------------------------------------------------- M4

fn m4(toks: &[Token], surface: &str, out: &mut Vec<Edit>) {
    const CMP: &[&str] = &["<", "<=", ">", ">=", "=", "=="];
    for (i, t) in toks.iter().enumerate() {
        if t.kind != Kind::Number {
            continue;
        }
        let Ok(n) = t.text.replace('_', "").parse::<i64>() else {
            continue;
        };
        let site = match surface {
            "NL" | "RS" => i > 0 && toks[i - 1].text == "#",
            "SQL" | "PRQL" => {
                let left = i >= 2
                    && CMP.contains(&toks[i - 1].text.as_str())
                    && lw(&toks[i - 2]) == "epoch";
                let right = CMP.contains(&toks.get(i + 1).map(|x| x.text.as_str()).unwrap_or(""))
                    && toks.get(i + 2).is_some_and(|x| lw(x) == "epoch");
                left || right
            }
            _ => false,
        };
        if site {
            out.push(Edit {
                op: "M4",
                start: t.start,
                end: t.end,
                with: (n + 1).to_string(),
                line: t.line,
                what: format!("epoch {n} → {}", n + 1),
            });
        }
    }
}

// ---------------------------------------------------------------- M5

fn m5(src: &str, toks: &[Token], surface: &str, out: &mut Vec<Edit>) {
    for (i, t) in toks.iter().enumerate() {
        let w = lw(t);
        let stmt_to_semicolon = |from: usize| -> usize {
            let e = scan(toks, from, &[";"], false);
            if toks.get(e).is_some_and(|x| x.text == ";") {
                e + 1
            } else {
                e
            }
        };
        if surface == "SQL" {
            if w == "perform"
                && toks
                    .get(i + 1)
                    .is_some_and(|x| lw(x).starts_with("resolve_"))
            {
                let e = stmt_to_semicolon(i);
                out.push(Edit {
                    op: "M5",
                    start: t.start,
                    end: toks[e - 1].end,
                    with: String::new(),
                    line: t.line,
                    what: format!("`{}` removed", &src[t.start..toks[e - 1].end]),
                });
            }
            if w == "comment" && is(toks.get(i + 1), "on") {
                let e = stmt_to_semicolon(i);
                let body = src[t.start..toks[e - 1].end].to_lowercase();
                if ["consumes", "linear", "conserve"]
                    .iter()
                    .any(|k| body.contains(k))
                {
                    out.push(Edit {
                        op: "M5",
                        start: t.start,
                        end: toks[e - 1].end,
                        with: String::new(),
                        line: t.line,
                        what: "an annotation removed".into(),
                    });
                }
            }
        } else if matches!(surface, "NL" | "RS") {
            if w == "resolve" && toks.get(i + 1).is_some_and(|x| x.kind == Kind::Word) {
                let e = scan(toks, i, &[";"], false);
                let end = if toks.get(e).is_some_and(|x| x.text == ";") {
                    toks[e].end
                } else {
                    toks[e - 1].end
                };
                out.push(Edit {
                    op: "M5",
                    start: t.start,
                    end,
                    with: String::new(),
                    line: t.line,
                    what: format!("`{}` removed", src[t.start..end].trim()),
                });
            }
            if w == "conserve" {
                let e = stmt_to_semicolon(i);
                out.push(Edit {
                    op: "M5",
                    start: t.start,
                    end: toks[e - 1].end,
                    with: String::new(),
                    line: t.line,
                    what: "the `conserve` clause removed".into(),
                });
            }
        }
    }
}

// ---------------------------------------------------------------- M6

fn m6(toks: &[Token], out: &mut Vec<Edit>) {
    for t in toks
        .iter()
        .filter(|t| matches!(t.kind, Kind::Word | Kind::Str))
    {
        let (inner_start, inner) = if t.kind == Kind::Str && t.text.len() >= 2 {
            (t.start + 1, &t.text[1..t.text.len() - 1])
        } else {
            (t.start, t.text.as_str())
        };
        let Some(r) = LADDER.iter().position(|x| x.eq_ignore_ascii_case(inner)) else {
            continue;
        };
        if r == 0 {
            continue;
        }
        out.push(Edit {
            op: "M6",
            start: inner_start,
            end: inner_start + inner.len(),
            with: LADDER[r - 1].into(),
            line: t.line,
            what: format!("{} → {}", LADDER[r], LADDER[r - 1]),
        });
    }
}

/// Every mutant of a program, operator by operator, in source order.
pub fn mutants(src: &str, surface: &str) -> Vec<Mutant> {
    let toks = tokenize(src, surface);
    let mut edits = Vec::new();
    m1(src, &toks, &mut edits);
    m2(src, &toks, surface, &mut edits);
    m3(src, &toks, surface, &mut edits);
    m4(&toks, surface, &mut edits);
    m5(src, &toks, surface, &mut edits);
    m6(&toks, &mut edits);
    let mut out = Vec::new();
    for op in ["M1", "M2", "M3", "M4", "M5", "M6"] {
        let mut mine: Vec<&Edit> = edits.iter().filter(|e| e.op == op).collect();
        mine.sort_by_key(|e| (e.start, e.end));
        mine.dedup_by_key(|e| (e.start, e.end, e.with.clone()));
        for (site, e) in mine.into_iter().enumerate() {
            let text = format!("{}{}{}", &src[..e.start], e.with, &src[e.end..]);
            out.push(Mutant {
                op,
                site,
                line: e.line,
                what: e.what.clone(),
                text,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn whats(src: &str, s: &str, op: &str) -> Vec<String> {
        mutants(src, s)
            .into_iter()
            .filter(|m| m.op == op)
            .map(|m| m.text)
            .collect()
    }

    #[test]
    fn m1_swaps_each_currency_occurrence() {
        let m = whats(
            "select (sum(amt_usd)).minor from postings where cur = 'usd'",
            "SQL",
            "M1",
        );
        assert_eq!(m.len(), 2);
        assert!(m[0].contains("amt_eur") && m[0].contains("'usd'"));
        assert!(m[1].contains("amt_usd") && m[1].contains("'eur'"));
    }

    #[test]
    fn m2_toggles_legs() {
        let src = "insert into postings (txn, acct, cur, amt_usd) values (t, a, 'usd', row(-(m).minor)::usd), (t, b, 'usd', m);";
        let m = whats(src, "SQL", "M2");
        assert_eq!(m.len(), 2, "{m:?}");
        assert!(m[0].contains("row((m).minor)::usd), (t, b, 'usd', m)"));
        assert!(m[1].contains("(t, b, 'usd', row(-(m).minor)::usd)"));
        let n = whats("post(debit(a, m)?, credit(b, m))", "NL", "M2");
        assert_eq!(
            n,
            [
                "post(credit(a, m)?, credit(b, m))",
                "post(debit(a, m)?, debit(b, m))"
            ]
        );
    }

    #[test]
    fn m3_drops_each_conjunct_or_the_clause() {
        let m = whats(
            "select a from t where x = 1 and y between 2 and 3 group by a",
            "SQL",
            "M3",
        );
        assert_eq!(
            m,
            [
                "select a from t where y between 2 and 3 group by a",
                "select a from t where x = 1 group by a"
            ]
        );
        let one = whats("view v = t.where(|r| r.x == 1).map(|r| r.a);", "RS", "M3");
        assert_eq!(one, ["view v = t.map(|r| r.a);"]);
        let j = whats(
            "view v = t.join(u, |l, r| r.c == l.c && r.n < l.n);",
            "RS",
            "M3",
        );
        assert_eq!(j.len(), 2);
        assert!(whats(
            "create index i on t (a); comment on table t is 'x';",
            "SQL",
            "M3"
        )
        .is_empty());
        let dl = whats("a(x) :- b(x), c(x), x > 1.", "DL", "M3");
        assert_eq!(dl.len(), 3);
        let prql = whats("from t\nfilter x == 1 && y == 2\nselect {x}", "PRQL", "M3");
        assert_eq!(
            prql,
            [
                "from t\nfilter y == 2\nselect {x}",
                "from t\nfilter x == 1\nselect {x}"
            ]
        );
    }

    #[test]
    fn m3_leaves_a_choice_alone() {
        assert!(whats("fn f() { if a > b { x } else { y } }", "NL", "M3").is_empty());
        let g = whats("fn f() { if a > b { return Err(e) } x }", "NL", "M3");
        assert_eq!(g, ["fn f() { if false { return Err(e) } x }"]);
    }

    #[test]
    fn m4_m5_m6() {
        assert_eq!(whats("t.as_of(#5)", "RS", "M4"), ["t.as_of(#6)"]);
        assert_eq!(whats("where epoch <= 5", "SQL", "M4"), ["where epoch <= 6"]);
        assert_eq!(
            whats(
                "begin h := hold(a, m); perform resolve_void(h); end",
                "SQL",
                "M5"
            ),
            ["begin h := hold(a, m);  end"]
        );
        assert_eq!(
            whats("serve { consistency: snapshot }", "NL", "M6"),
            ["serve { consistency: read_your_writes }"]
        );
        assert_eq!(
            whats("values ('v', 'ledger_consistent')", "SQL", "M6"),
            ["values ('v', 'serializable')"]
        );
        assert!(whats("serve { consistency: bounded(1.epochs) }", "NL", "M6").is_empty());
    }
}
