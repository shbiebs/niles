//! **E30b′'s report**, rendered from the rows in `results/E30b-syntax/` (design
//! `docs/study/E30b-design.md` §9). Nothing in the report is typed in by hand except the
//! prose that explains the numbers; every number is read from a row.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

type Row = BTreeMap<String, String>;

fn tsv(dir: &Path, name: &str) -> (Vec<String>, Vec<Row>) {
    let text = std::fs::read_to_string(dir.join(name)).unwrap_or_default();
    let mut header: Vec<String> = Vec::new();
    let mut comments = Vec::new();
    let mut rows = Vec::new();
    for line in text.lines() {
        if let Some(c) = line.strip_prefix("# ") {
            comments.push(c.to_string());
            continue;
        }
        if header.is_empty() {
            header = line.split('\t').map(str::to_string).collect();
            continue;
        }
        let r: Row = header
            .iter()
            .cloned()
            .zip(line.split('\t').map(str::to_string))
            .collect();
        rows.push(r);
    }
    (comments, rows)
}

fn g<'a>(r: &'a Row, k: &str) -> &'a str {
    r.get(k).map(String::as_str).unwrap_or("")
}

const SURF: [&str; 4] = ["SQL", "SQL2", "NL", "RS"];

fn label(s: &str) -> &'static str {
    crate::e30b::label(s)
}

fn class_name(c: &str) -> &'static str {
    match c {
        "Q" => "Q — queries",
        "T" => "T — transactions",
        "V" => "V — view contracts",
        "B" => "B — temporal reads",
        _ => "all 30",
    }
}

#[derive(Default, Clone, Copy)]
struct Counts {
    stat: usize,
    runtime: usize,
    silent: usize,
    equivalent: usize,
    unexecuted: usize,
}

impl Counts {
    fn add(&mut self, class: &str) {
        match class {
            "static" => self.stat += 1,
            "runtime" => self.runtime += 1,
            "silent" => self.silent += 1,
            "equivalent" => self.equivalent += 1,
            _ => self.unexecuted += 1,
        }
    }
    fn den(&self) -> usize {
        self.stat + self.runtime + self.silent
    }
    fn rate(&self) -> Option<f64> {
        (self.den() > 0).then(|| self.stat as f64 * 100.0 / self.den() as f64)
    }
    fn pct(&self) -> String {
        self.rate()
            .map(|r| format!("{r:.1}%"))
            .unwrap_or_else(|| "—".into())
    }
}

/// The rule's verdict over the adversarial rows (design §1): per case, whether SQL+C+L
/// accepts its defective program in some representation the case has; whether Niles refuses
/// it, cannot express it, or accepts it; whether Niles can express the correct twin.
struct CaseVerdict {
    id: String,
    class: String,
    sql: String,
    sql_accepts_defective: bool,
    sql_refuses_correct: bool,
    niles: String,
    niles_expresses_correct: bool,
    niles_refuses_or_cannot: bool,
    counts: &'static str,
}

fn verdicts(adv: &[Row]) -> Vec<CaseVerdict> {
    let mut ids: Vec<String> = Vec::new();
    for r in adv {
        if !ids.contains(&r["case"]) {
            ids.push(r["case"].clone());
        }
    }
    ids.into_iter()
        .map(|id| {
            let of = |surface_prefix: &str, twin: &str| -> Vec<&Row> {
                adv.iter()
                    .filter(|r| {
                        r["case"] == id
                            && r["surface"].starts_with(surface_prefix)
                            && r["twin"] == twin
                    })
                    .collect()
            };
            let sd = of("SQL", "defective");
            let sc = of("SQL", "correct");
            let nd = of("NL", "defective");
            let nc = of("NL", "correct");
            let class = sd.first().map(|r| r["class"].clone()).unwrap_or_default();
            let sql = sd
                .iter()
                .map(|r| {
                    let v = match g(r, "outcome") {
                        "refused" => format!("refused ({})", g(r, "codes")),
                        o => o.to_string(),
                    };
                    format!("{}: {v}", g(r, "surface"))
                })
                .collect::<Vec<_>>()
                .join("; ");
            let sql_accepts_defective = sd.iter().any(|r| g(r, "outcome") == "accepted");
            let sql_refuses_correct = sc.iter().any(|r| g(r, "outcome") == "refused");
            let n = nd.first();
            let niles = match n.map(|r| g(r, "outcome")) {
                Some("refused") => format!("refused ({})", g(n.unwrap(), "codes")),
                Some("not expressible") => "not expressible".to_string(),
                Some(o) => o.to_string(),
                None => "—".into(),
            };
            let niles_expresses_correct = nc.iter().any(|r| g(r, "outcome") != "not expressible");
            let niles_refuses_or_cannot = matches!(
                n.map(|r| g(r, "outcome")),
                Some("refused") | Some("not expressible")
            );
            let counts = if !sql_accepts_defective {
                "no — SQL+C+L refuses it"
            } else if !niles_expresses_correct {
                "no — outside Niles (neither twin expressible)"
            } else if niles_refuses_or_cannot {
                "**yes**"
            } else {
                "no — Niles accepts it too"
            };
            CaseVerdict {
                id,
                class,
                sql,
                sql_accepts_defective,
                sql_refuses_correct,
                niles,
                niles_expresses_correct,
                niles_refuses_or_cannot,
                counts,
            }
        })
        .collect()
}

pub fn render(dir: &Path) -> String {
    let (_, programs) = tsv(dir, "programs.tsv");
    let (_, mutants) = tsv(dir, "mutants.tsv");
    let (_, adv) = tsv(dir, "adversarial.tsv");
    let (cost_head, cost) = tsv(dir, "cost-inprocess.tsv");
    let commit = cost_head
        .iter()
        .find_map(|c| c.strip_prefix("commit: "))
        .unwrap_or("?")
        .to_string();
    let mut o = String::new();

    o += "# E30b′ — does the language need its own grammar? (cycle 15, C15-05; decision DA-1)\n\n";
    let _ = writeln!(
        o,
        "*Rendered by `cargo run -p syntax-study --bin e30b -- report` from the rows in `results/E30b-syntax/`, which `e30b measure`, `e30b mutate`, `e30b adversarial` and the memory probe's `e30bcost` wrote at commit `{commit}`. The design, pre-registered before any program or checker change, is `docs/study/E30b-design.md` (commit `163456d`); the corpus was committed before any addition was built (`7b5425c`). The SQL+C+L additions are `634f289` (effect annotations), `cfcfbff` (body typing) and `93c529a` (NSQ002); the niles-interp additions are `25e6df7`. E30's results (`results/E30-syntax.md`) are not overwritten.*\n"
    );

    // ------------------------------------------------------------------ the rule
    o += "## The rule, evaluated\n\n";
    o += "> The Niles grammar is kept for exactly the defect classes that the upgraded SQL+C+L cannot refuse at check time on the adversarial corpus. If it refuses all of them, annotated SQL is the language, and Niles is its specification and reference checker. Check time per KLOC above 2× is reported as a cost, not a veto.\n\n";
    o += "A class **counts for the Niles grammar** when SQL+C+L accepts a defective program of it that Niles refuses, or cannot express while it can express the correct twin (design §1). A case whose correct twin Niles cannot write is *outside Niles*, and counts for neither language.\n\n";
    let vs = verdicts(&adv);
    o += "| case | class | SQL+C+L on the defective program | Niles on it | Niles writes the correct twin | counts for the Niles grammar |\n|---|---|---|---|---|---|\n";
    for v in &vs {
        let _ = writeln!(
            o,
            "| {} | {} | {} | {} | {} | {} |",
            v.id,
            v.class,
            v.sql,
            v.niles,
            if v.niles_expresses_correct {
                "yes"
            } else {
                "no"
            },
            v.counts
        );
    }
    let mut counting: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut all_classes: BTreeSet<String> = BTreeSet::new();
    for v in &vs {
        all_classes.insert(v.class.clone());
        if v.sql_accepts_defective && v.niles_expresses_correct && v.niles_refuses_or_cannot {
            counting
                .entry(v.class.clone())
                .or_default()
                .push(v.id.clone());
        }
    }
    o += "\n";
    if counting.is_empty() {
        o += "**Verdict: the upgraded SQL+C+L refuses every defect class that Niles refuses on the adversarial corpus. By the rule, annotated SQL is the language, and Niles is its specification and reference checker.**\n\n";
    } else {
        let list: Vec<String> = counting
            .iter()
            .map(|(c, ids)| format!("*{c}* ({})", ids.join(", ")))
            .collect();
        let _ = writeln!(
            o,
            "**Verdict: {} of the {} classes in `cases.tsv` count{} for the Niles grammar:** {}. In every other class the upgraded SQL+C+L refuses what Niles refuses, or neither refuses it, or the case lies outside Niles. By the rule, the Niles grammar is kept for exactly the class{} named, and for no other.\n",
            counting.len(),
            all_classes.len(),
            if counting.len() == 1 { "s" } else { "" },
            list.join("; "),
            if counting.len() == 1 { "" } else { "es" }
        );
    }
    let accepted_by_both: Vec<&str> = vs
        .iter()
        .filter(|v| {
            v.sql_accepts_defective && v.niles_expresses_correct && !v.niles_refuses_or_cannot
        })
        .map(|v| v.id.as_str())
        .collect();
    let outside: Vec<&str> = vs
        .iter()
        .filter(|v| v.sql_accepts_defective && !v.niles_expresses_correct)
        .map(|v| v.id.as_str())
        .collect();
    let false_ref: Vec<&str> = vs
        .iter()
        .filter(|v| v.sql_refuses_correct)
        .map(|v| v.id.as_str())
        .collect();
    let _ = writeln!(
        o,
        "* **Accepted by both checkers:** {}.\n* **Accepted by SQL+C+L and outside Niles** (Niles writes neither twin): {}.\n* **False refusals** (a correct twin refused): SQL+C+L {}; Niles {}.\n",
        if accepted_by_both.is_empty() { "none".into() } else { accepted_by_both.join(", ") },
        if outside.is_empty() { "none".into() } else { outside.join(", ") },
        if false_ref.is_empty() { "none".into() } else { false_ref.join(", ") },
        {
            let n: Vec<&str> = adv
                .iter()
                .filter(|r| g(r, "surface") == "NL" && g(r, "twin") == "correct" && g(r, "outcome") == "refused")
                .map(|r| g(r, "case"))
                .collect();
            if n.is_empty() { "none".to_string() } else { n.join(", ") }
        }
    );

    // ------------------------------------------------------------------ adversarial detail
    o += "## The adversarial corpus, every program\n\n`crates/syntax-study/corpus/e30b/adversarial/`, with `cases.tsv`. Every correct SQL twin was also run on PostgreSQL 16 against the dataset, in `begin … rollback`, with the deferred conservation check forced; every correct Niles twin was run by `niles-interp` where it can be.\n\n| case | surface | twin | verdict | codes | run |\n|---|---|---|---|---|---|\n";
    for r in &adv {
        let _ = writeln!(
            o,
            "| {} | {} | {} | {} | {} | {} |",
            g(r, "case"),
            g(r, "surface"),
            g(r, "twin"),
            g(r, "outcome"),
            g(r, "codes"),
            g(r, "execution")
        );
    }
    o += "\n";

    // ------------------------------------------------------------------ E30 corpus: expressiveness
    o += "## The E30 corpus: expressiveness\n\nEach program checked by its surface's checker and run against the oracle. *correct* is the checker's acceptance plus an answer equal to the oracle's; *unexecuted* is accepted with no executor that can run it.\n\n| class | surface | correct | unexecuted | not expressible | tokens (sum) |\n|---|---|--:|--:|--:|--:|\n";
    for c in ["Q", "T", "V", "B", "all"] {
        for s in SURF {
            let rs: Vec<&Row> = programs
                .iter()
                .filter(|r| g(r, "surface") == s && (c == "all" || g(r, "task").starts_with(c)))
                .collect();
            let n = |v: &str| rs.iter().filter(|r| g(r, "verdict") == v).count();
            let toks: usize = rs
                .iter()
                .filter_map(|r| g(r, "tokens").parse::<usize>().ok())
                .sum();
            let _ = writeln!(
                o,
                "| {} | {} | {} | {} | {} | {toks} |",
                class_name(c),
                label(s),
                n("correct"),
                n("unexecuted"),
                n("not expressible")
            );
        }
    }
    let wrong: Vec<String> = programs
        .iter()
        .filter(|r| matches!(g(r, "verdict"), "wrong" | "refused" | "raised"))
        .map(|r| {
            format!(
                "{} {} ({})",
                g(r, "task"),
                label(g(r, "surface")),
                g(r, "verdict")
            )
        })
        .collect();
    let _ = writeln!(
        o,
        "\nPrograms wrong, refused or raising: {}. Not expressible, with the reason each program's row gives: {}.\n",
        if wrong.is_empty() { "none".into() } else { wrong.join(", ") },
        programs
            .iter()
            .filter(|r| g(r, "verdict") == "not expressible")
            .map(|r| format!("{} {}", g(r, "task"), label(g(r, "surface"))))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // ------------------------------------------------------------------ E30 corpus: mutation
    o += "## The E30 corpus: mutation\n\nE30's operators M1–M6 and site patterns, unchanged, with R2's one M2 pattern (design §7). Static-rejection rate = static ÷ (static + runtime + silent); equivalent and unexecuted mutants are reported and excluded. *Without annotation sites* removes the mutants whose edit falls inside an effect annotation (SQL's `'effects: …'` string, Niles's `! { … }` row), since those sites exist only because the annotation does.\n\n| class | surface | mutants | static | runtime | silent | equivalent | unexecuted | static-rejection rate | without annotation sites |\n|---|---|--:|--:|--:|--:|--:|--:|--:|--:|\n";
    for c in ["Q", "T", "V", "B", "all"] {
        for s in SURF {
            let mut all = Counts::default();
            let mut noann = Counts::default();
            let mut n = 0;
            for r in mutants
                .iter()
                .filter(|r| g(r, "surface") == s && (c == "all" || g(r, "task").starts_with(c)))
            {
                n += 1;
                all.add(g(r, "class"));
                if g(r, "in_annotation") == "no" {
                    noann.add(g(r, "class"));
                }
            }
            let _ = writeln!(
                o,
                "| {} | {} | {n} | {} | {} | {} | {} | {} | {} | {} |",
                class_name(c),
                label(s),
                all.stat,
                all.runtime,
                all.silent,
                all.equivalent,
                all.unexecuted,
                all.pct(),
                noann.pct()
            );
        }
    }

    // Candidates: (operator, class) cells where a Niles surface's rate is strictly higher than
    // an SQL representation's, both defined (design §7).
    o += "\n### Candidates from the E30 corpus (design §7)\n\nCells where NL's or RS's static-rejection rate is strictly higher than SQL-R1's or SQL-R2's, both rates defined. They are candidates only: the adversarial corpus decides the rule.\n\n| operator | class | SQL-R1 | SQL-R2 | NL | RS | Niles's static refusals, NL and RS together |\n|---|---|--:|--:|--:|--:|---|\n";
    let mut any = false;
    for op in ["M1", "M2", "M3", "M4", "M5", "M6"] {
        for c in ["Q", "T", "V", "B"] {
            let cell = |s: &str| {
                let mut k = Counts::default();
                for r in mutants.iter().filter(|r| {
                    g(r, "op") == op && g(r, "surface") == s && g(r, "task").starts_with(c)
                }) {
                    k.add(g(r, "class"));
                }
                k
            };
            let (a, b, n, rs) = (cell("SQL"), cell("SQL2"), cell("NL"), cell("RS"));
            let higher = |x: &Counts| {
                [&a, &b].iter().any(|y| match (x.rate(), y.rate()) {
                    (Some(p), Some(q)) => p > q,
                    _ => false,
                })
            };
            if higher(&n) || higher(&rs) {
                any = true;
                let mut codes: BTreeMap<String, usize> = BTreeMap::new();
                for r in mutants.iter().filter(|r| {
                    g(r, "op") == op
                        && (g(r, "surface") == "NL" || g(r, "surface") == "RS")
                        && g(r, "task").starts_with(c)
                        && g(r, "class") == "static"
                }) {
                    *codes.entry(g(r, "detail").to_string()).or_default() += 1;
                }
                let what: Vec<String> = codes.iter().map(|(k, v)| format!("{k} ×{v}")).collect();
                let _ = writeln!(
                    o,
                    "| {op} | {} | {} ({}) | {} ({}) | {} ({}) | {} ({}) | {} |",
                    class_name(c),
                    a.pct(),
                    a.den(),
                    b.pct(),
                    b.den(),
                    n.pct(),
                    n.den(),
                    rs.pct(),
                    rs.den(),
                    what.join(", ")
                );
            }
        }
    }
    if !any {
        o += "| — | — | | | | | none |\n";
    }
    o += "\n";

    // ------------------------------------------------------------------ cost
    o += "## Check cost\n\nIn-process, release build, by E30's protocol (design §8).\n\n";
    for c in &cost_head {
        let _ = writeln!(o, "* {c}");
    }
    o += "\n| corpus | surface | files | lines | µs per KLOC | MAD of a pass, µs | peak heap per KLOC, max bytes |\n|---|---|--:|--:|--:|--:|--:|\n";
    let mut per: BTreeMap<(String, String), f64> = BTreeMap::new();
    for r in &cost {
        let _ = writeln!(
            o,
            "| {} | {} | {} | {} | {} | {} | {} |",
            g(r, "corpus"),
            g(r, "surface"),
            g(r, "files"),
            g(r, "lines"),
            g(r, "us per KLOC"),
            g(r, "MAD us"),
            g(r, "peak heap per KLOC max bytes")
        );
        if let Ok(v) = g(r, "us per KLOC").parse::<f64>() {
            per.insert((g(r, "corpus").to_string(), g(r, "surface").to_string()), v);
        }
    }
    o += "\n";
    for corpus in ["E30 corpus", "adversarial"] {
        let nl = per.get(&(corpus.to_string(), "NL".to_string())).copied();
        for s in ["SQL-R1", "SQL-R2"] {
            if let (Some(n), Some(x)) = (nl, per.get(&(corpus.to_string(), s.to_string()))) {
                let ratio = x / n;
                let _ = writeln!(
                    o,
                    "* {corpus}: {s} takes **{ratio:.2}×** NL's time per KLOC{}.",
                    if ratio > 2.0 {
                        " — above 2×, so reported as a cost (the rule: not a veto)"
                    } else {
                        ""
                    }
                );
            }
        }
    }
    o += "\n";

    o += NOTES;
    o
}

const NOTES: &str = "## Deviations from the design\n\n\
* **D1 — a defect in the existing conservation adapter was fixed** (`3ee2e76`). Building the typing rules found that `(m).minor` parses as the field of a one-element row, which `conserve.rs` read as a fresh amount, contrary to its own documentation (\"`row(-(m).minor)::usd` and `m` cancel\"). The adapter now reads the parenthesised form as `m`. It is not one of the three additions; it was made before any adversarial verdict was read, and it has a guard test that fails against the old reading.\n\
* **D2 — `hold`'s named arguments are not evaluated** by niles-interp (`25e6df7`). The design said they are evaluated and not otherwise interpreted. A duration literal (`expires: 7.days`) is outside the interpreter's subset, so evaluating one would have refused every hold, T05 included; the arguments have no effect under design §6.2 either way.\n\
* **D3 — no program was edited after its mutants were classified,** and no builder fix was needed: every SQL-R1 and SQL-R2 program, and T02, T05 and T06 in NL and RS, equalled the oracle on their first run (`e30b run`, before `e30b mutate`).\n\n\
## What the numbers contain\n\n\
* **Niles's static refusals of a flipped leg (M2 on T) are all NL0257.** Turning `debit(a, m)?` into `credit(a, m)?` leaves a `?` on a value that is not a `Result`, which C15-01 made a check-time error; E30 classed the same mutants *runtime* (its finding F12). The refusal comes from the shape of the two functions' return types, not from the conservation solver: the flip in the other direction (`credit` to `debit`, which leaves no `?` behind) is refused at run time by both languages' ledgers. SQL+C+L refuses no M2 mutant statically, because its solver reads a parameter as a symbol that may be zero, so `2m ≠ 0` is undecided and discharged to the conservation trigger, exactly as Niles's solver does.\n\
* **Every run-time currency swap in SQL's transactions is on the annotation's signature line.** Each M1 mutant of a T program classed *runtime*, in both representations, is an edit to the parameter types written in `comment on function task(bigint, bigint, usd)`: the comment then names a function that does not exist, and PostgreSQL raises when the script defines it. The checker keys annotations by function name, so it does not see this; neither would a reviewer reading the effect list, which is unchanged.\n\
* **M6 finds no site inside SQL's annotation strings.** E30's M6 pattern matches a rung that is a whole word or a whole string; `reads@ledger_consistent` inside `'effects: …'` is neither. So M6 mutates Niles's `read@…` rows (three T sites, all refused as NL0310) and nothing in SQL's T programs. This is E30's site pattern, kept unchanged by the design, and it is stated here because it makes the M6 row incomparable.\n\
* **SQL+C+L fails open on a missing annotation.** A01 (a hold domain never declared linear) and A02 (a ledger never declared a ledger) are accepted because the checker's rules are keyed on the annotations the schema omitted; Niles's holds are linear and its conservation applies to every `post`, whatever the schema says. This is the class the verdict names.\n\
* **Niles accepts A05.** A helper posting one half of a transfer, called inside the caller's transaction, leaves the caller's row unbalanced by the symbolic amount `m`, which may be zero, so the solver discharges it to the run-time seal rather than refusing it. Neither checker refuses A05; it counts for neither language.\n\
* **NSQ002 refuses A03's correct twin as well as its defect,** by design: dynamic SQL in a ledger writer cannot be checked, so the rule refuses it whatever it builds. A04's ledger name, assembled by concatenation, escapes the string-literal clause of §5.3, as the design said it would; A04 is outside Niles either way.\n\n";

#[cfg(test)]
mod tests {
    use super::*;

    fn row(pairs: &[(&str, &str)]) -> Row {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn a_class_counts_only_when_sql_accepts_and_niles_refuses_or_cannot_write_it() {
        let mk = |case: &str, surface: &str, twin: &str, outcome: &str| {
            row(&[
                ("case", case),
                ("class", "c"),
                ("surface", surface),
                ("twin", twin),
                ("outcome", outcome),
                ("codes", ""),
                ("execution", ""),
            ])
        };
        let adv = vec![
            // counts: SQL accepts, Niles refuses, Niles writes the correct twin
            mk("X1", "SQL-R1", "defective", "accepted"),
            mk("X1", "SQL-R1", "correct", "accepted"),
            mk("X1", "NL", "defective", "refused"),
            mk("X1", "NL", "correct", "accepted"),
            // does not: SQL refuses
            mk("X2", "SQL-R1", "defective", "refused"),
            mk("X2", "NL", "defective", "refused"),
            mk("X2", "NL", "correct", "accepted"),
            // does not: outside Niles
            mk("X3", "SQL-R1", "defective", "accepted"),
            mk("X3", "NL", "defective", "not expressible"),
            mk("X3", "NL", "correct", "not expressible"),
            // does not: Niles accepts it too
            mk("X4", "SQL-R1", "defective", "accepted"),
            mk("X4", "NL", "defective", "accepted"),
            mk("X4", "NL", "correct", "accepted"),
        ];
        let v = verdicts(&adv);
        let counts: Vec<&str> = v.iter().map(|x| x.counts).collect();
        assert_eq!(counts[0], "**yes**");
        assert!(counts[1].starts_with("no — SQL+C+L refuses"));
        assert!(counts[2].starts_with("no — outside Niles"));
        assert!(counts[3].starts_with("no — Niles accepts"));
    }
}
