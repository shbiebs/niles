//! **`results/E30-syntax.md`**, rendered from the per-program and per-mutant rows in
//! `results/E30-syntax/` (design §8). Every number in the report is computed here from those
//! files; the prose that does not depend on a number (the deviations, the findings) is fixed
//! text below, written before this renderer first ran.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

fn tsv(p: &Path) -> Vec<Vec<String>> {
    std::fs::read_to_string(p)
        .unwrap_or_else(|e| panic!("{}: {e}", p.display()))
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .skip(1)
        .map(|l| l.split('\t').map(String::from).collect())
        .collect()
}

fn header(p: &Path) -> BTreeMap<String, String> {
    std::fs::read_to_string(p)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.strip_prefix("# "))
        .filter_map(|l| l.split_once(": "))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub const CLASSES: [(&str, &str); 4] = [
    ("Q", "queries"),
    ("T", "transactions"),
    ("V", "view contracts"),
    ("B", "temporal reads"),
];
const SURF: [&str; 5] = ["SQL", "NL", "RS", "PRQL", "DL"];
const IN_PROCESS: [&str; 3] = ["SQL", "NL", "RS"];

#[derive(Default, Clone, Copy)]
struct Tally {
    stat: usize,
    run: usize,
    silent: usize,
    equiv: usize,
    unexec: usize,
    run_analysis: usize,
}

impl Tally {
    fn add(&mut self, class: &str, detail: &str) {
        match class {
            "static" => self.stat += 1,
            "runtime" => {
                self.run += 1;
                if detail.contains("[42") {
                    self.run_analysis += 1;
                }
            }
            "silent" => self.silent += 1,
            "equivalent" => self.equiv += 1,
            "unexecuted" => self.unexec += 1,
            other => panic!("no class {other}"),
        }
    }
    fn den(&self) -> usize {
        self.stat + self.run + self.silent
    }
    fn rate(&self) -> Option<f64> {
        (self.den() > 0).then(|| self.stat as f64 / self.den() as f64)
    }
    fn total(&self) -> usize {
        self.den() + self.equiv + self.unexec
    }
    /// The rate's range were every unexecuted mutant caught, or every one missed.
    fn bounds(&self) -> Option<(f64, f64)> {
        let d = self.den() + self.unexec;
        (self.unexec > 0 && d > 0).then(|| {
            (
                self.stat as f64 / d as f64,
                (self.stat + self.unexec) as f64 / d as f64,
            )
        })
    }
}

fn pct(x: Option<f64>) -> String {
    x.map(|v| format!("{:.1}%", v * 100.0))
        .unwrap_or("—".into())
}

/// A surface's standing in one class: name, expresses, verified, rate, cost.
type Ranked = (String, usize, usize, Option<f64>, Option<f64>);

struct Prog {
    verdict: String,
    detail: String,
    tokens: usize,
    kw: usize,
    ops: usize,
}

pub struct Inputs<'a> {
    pub dir: &'a Path,
    /// The design's commits, oldest first, as `git log` gives them.
    pub design: &'a str,
}

pub fn render(i: &Inputs) -> String {
    let dir = i.dir;
    let mut progs: BTreeMap<(String, String), Prog> = BTreeMap::new();
    for r in tsv(&dir.join("programs.tsv")) {
        progs.insert(
            (r[0].clone(), r[1].clone()),
            Prog {
                verdict: r[2].clone(),
                detail: r[3].clone(),
                tokens: r[5].parse().unwrap_or(0),
                kw: r[6].parse().unwrap_or(0),
                ops: r[7].parse().unwrap_or(0),
            },
        );
    }
    let tasks_of = |c: &str| -> Vec<String> {
        let set: BTreeSet<String> = progs
            .keys()
            .filter(|(t, _)| t.starts_with(c))
            .map(|(t, _)| t.clone())
            .collect();
        set.into_iter().collect()
    };
    let in_scope = |t: &str, s: &str| progs.contains_key(&(t.to_string(), s.to_string()));
    let expressible = |t: &str, s: &str| {
        progs
            .get(&(t.to_string(), s.to_string()))
            .is_some_and(|p| p.verdict == "correct" || p.verdict == "unexecuted")
    };
    let mut tally: BTreeMap<(String, String), Tally> = BTreeMap::new();
    let mut by_op: BTreeMap<(String, String), Tally> = BTreeMap::new();
    let mutants = tsv(&dir.join("mutants.tsv"));
    for r in &mutants {
        let class = r[0][..1].to_string();
        tally
            .entry((r[1].clone(), class.clone()))
            .or_default()
            .add(&r[6], &r[7]);
        tally
            .entry((r[1].clone(), "all".into()))
            .or_default()
            .add(&r[6], &r[7]);
        by_op
            .entry((r[1].clone(), r[2].clone()))
            .or_default()
            .add(&r[6], &r[7]);
    }
    let cost_in = tsv(&dir.join("cost-inprocess.tsv"));
    let cost_hdr = header(&dir.join("cost-inprocess.tsv"));
    let cost_ext = tsv(&dir.join("cost-external.tsv"));
    let us_per_kloc: BTreeMap<String, f64> = cost_in
        .iter()
        .map(|r| (r[0].clone(), r[5].parse().unwrap_or(f64::NAN)))
        .collect();
    let load = tsv(&dir.join("load.tsv"));
    let schemas = tsv(&dir.join("schemas.tsv"));

    let mut o = String::new();
    o += "# E30 — the syntax study (cycle 14, R2-06; decision D-3)\n\n";
    let _ = writeln!(
        o,
        "*Rendered by `cargo run -p syntax-study --bin e30 -- report` from the rows in `results/E30-syntax/`, every one of which `make e30` regenerated at commit `{}`. The design, pre-registered before any program was written, is `docs/study/E30-syntax-design.md` ({}); every departure from it is listed under **Deviations** with its reason. Reproducing needs PostgreSQL 16 as the gate's `bench` role and the approved `prqlc` and `souffle` under `/opt/arms`.*\n",
        cost_hdr.get("commit").cloned().unwrap_or_default(),
        i.design
    );

    // ------------------------------------------------------------ the rule
    o += "## The rule, evaluated\n\n";
    o += "> The principal surface is the one that (i) expresses the most corpus tasks, then (ii) has the highest static-rejection rate under the mutation set, then (iii) the lowest check cost; ties are reported as ties. A surface that cannot express a task class is not principal for that class; new syntax is justified only for classes no surface expresses.\n\n";
    o += "Applied per class, over the surfaces in scope for the class, and overall over the three surfaces in scope for all thirty tasks (PRQL and DL are in scope for the ten queries only, design §2). *Expresses* counts a program its checker accepts whose answer equals the oracle's, and — for NL and RS — a program its checker accepts that no Niles executor can run (A1's *unexecuted*), whose correctness is therefore not established; the verified count is shown beside it. Check cost ranks only the in-process checkers (SQL, NL, RS); PRQL's and DL's are external binaries, reported apart and never ranked against them (design §6). A checker's cost is its time per KLOC over its whole corpus (the class files alone are too few to time apart), and NL and RS share every non-query file.\n\n";
    o += "| class | surface | expresses (verified) | static-rejection rate | check cost, µs per KLOC | position |\n|---|---|--:|--:|--:|---|\n";
    let mut verdicts: Vec<(String, String)> = Vec::new();
    let mut rank = |label: &str, cls: &str, surfaces: &[&str], tasks: &[String], o: &mut String| {
        let mut rows: Vec<Ranked> = surfaces
            .iter()
            .map(|s| {
                let e = tasks.iter().filter(|t| expressible(t, s)).count();
                let v = tasks
                    .iter()
                    .filter(|t| {
                        progs
                            .get(&(t.to_string(), s.to_string()))
                            .is_some_and(|p| p.verdict == "correct")
                    })
                    .count();
                let rate = tally
                    .get(&(s.to_string(), cls.to_string()))
                    .and_then(Tally::rate);
                let cost = IN_PROCESS
                    .contains(s)
                    .then(|| us_per_kloc.get(*s).copied())
                    .flatten();
                (s.to_string(), e, v, rate, cost)
            })
            .collect();
        let n = tasks.len();
        let full: Vec<&str> = rows
            .iter()
            .filter(|r| r.1 == n)
            .map(|r| r.0.as_str())
            .collect();
        let best_e = rows.iter().map(|r| r.1).max().unwrap_or(0);
        let step1: Vec<_> = rows.iter().filter(|r| r.1 == best_e).cloned().collect();
        let best_r = step1
            .iter()
            .map(|r| r.3.unwrap_or(-1.0))
            .fold(f64::MIN, f64::max);
        let step2: Vec<_> = step1
            .iter()
            .filter(|r| r.3.unwrap_or(-1.0) == best_r)
            .cloned()
            .collect();
        let decided = if step1.len() == 1 {
            format!("**{}**, by (i)", step1[0].0)
        } else if step2.len() == 1 {
            format!(
                "**{}**, by (ii) — (i) tied {}",
                step2[0].0,
                step1
                    .iter()
                    .map(|r| r.0.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            // (iii): only the in-process checkers are ranked.
            let costs: Vec<_> = step2.iter().filter(|r| r.4.is_some()).cloned().collect();
            let external: Vec<&str> = step2
                .iter()
                .filter(|r| r.4.is_none())
                .map(|r| r.0.as_str())
                .collect();
            let identical: Vec<&str> = {
                // NL and RS share every non-query file (design §2): the same program cannot
                // be separated, and is reported as a tie at every step.
                if cls != "Q"
                    && costs.iter().any(|r| r.0 == "NL")
                    && costs.iter().any(|r| r.0 == "RS")
                {
                    vec!["NL", "RS"]
                } else {
                    Vec::new()
                }
            };
            let best = costs
                .iter()
                .min_by(|a, b| a.4.partial_cmp(&b.4).expect("finite"))
                .map(|r| r.0.clone());
            let mut s = match best {
                Some(b) if identical.contains(&b.as_str()) => format!(
                    "**NL and RS, tied** — (i) and (ii) tied {}; the two share this class's programs, so (iii) cannot separate them",
                    step2.iter().map(|r| r.0.as_str()).collect::<Vec<_>>().join(", ")
                ),
                Some(b) => format!(
                    "**{b}**, by (iii) — (i) and (ii) tied {}",
                    step2.iter().map(|r| r.0.as_str()).collect::<Vec<_>>().join(", ")
                ),
                None => "a tie that (iii) cannot rank".into(),
            };
            if !external.is_empty() {
                let _ = write!(
                    s,
                    "; {} tied too and has no in-process cost to rank",
                    external.join(", ")
                );
            }
            s
        };
        for r in &rows {
            let pos = if cls != "all" && r.1 < n && !full.is_empty() {
                "cannot express the whole class: not principal for it".to_string()
            } else {
                String::new()
            };
            let _ = writeln!(
                o,
                "| {label} | {} | {} of {n} ({}) | {} | {} | {pos} |",
                r.0,
                r.1,
                r.2,
                pct(r.3),
                r.4.map(|c| format!("{c:.0}"))
                    .unwrap_or("not ranked (external)".into())
            );
        }
        rows.clear();
        verdicts.push((label.to_string(), decided));
    };
    for (c, name) in CLASSES {
        let ts = tasks_of(c);
        let surfaces: Vec<&str> = SURF
            .iter()
            .copied()
            .filter(|s| ts.iter().all(|t| in_scope(t, s)))
            .collect();
        rank(&format!("{c} — {name}"), c, &surfaces, &ts, &mut o);
    }
    let all: Vec<String> = progs
        .keys()
        .map(|(t, _)| t.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    rank("all 30", "all", &IN_PROCESS, &all, &mut o);
    o += "\n";
    for (label, v) in &verdicts {
        let _ = writeln!(o, "* **{label}:** {v}.");
    }
    let no_surface: Vec<&str> = CLASSES
        .iter()
        .filter(|(c, _)| {
            let ts = tasks_of(c);
            !SURF.iter().any(|s| ts.iter().all(|t| expressible(t, s)))
        })
        .map(|(c, _)| *c)
        .collect();
    let _ = writeln!(
        o,
        "* **New syntax:** {}.\n",
        if no_surface.is_empty() {
            "not justified — every class has a surface that expresses all of its tasks".to_string()
        } else {
            format!("justified for class(es) {}", no_surface.join(", "))
        }
    );
    o += "**How much weight this bears.** Read the static-rejection rates with the mutant counts below them: a class with few mutants, or with many excluded as unexecuted, supports a ranking less than its percentage suggests, and the NL/RS transaction rate is measured on the three transactions a Niles executor could run (T01, T03, T07) plus every mutant the checker refused. Its range, were every unexecuted mutant caught or every one missed, is given in the safety table. The syntax rule in §6.25, B.1 and J.1 is restated from this result in R2-11 or round 3, never here (design §8).\n\n";

    // ------------------------------------------------------------ expressiveness
    o += "## Expressiveness\n\n";
    o += "| surface | Q | T | V | B | verified correct | unexecuted (A1) | not expressible |\n|---|--:|--:|--:|--:|--:|--:|---|\n";
    for s in SURF {
        let mut cells = Vec::new();
        let (mut ok, mut un) = (0, 0);
        let mut none = Vec::new();
        for (c, _) in CLASSES {
            let ts = tasks_of(c);
            if !ts.iter().any(|t| in_scope(t, s)) {
                cells.push("not in scope".to_string());
                continue;
            }
            let e = ts.iter().filter(|t| expressible(t, s)).count();
            cells.push(format!("{e}/{}", ts.len()));
            for t in &ts {
                match progs
                    .get(&(t.clone(), s.to_string()))
                    .map(|p| p.verdict.as_str())
                {
                    Some("correct") => ok += 1,
                    Some("unexecuted") => un += 1,
                    Some("not expressible") => none.push(t.clone()),
                    _ => {}
                }
            }
        }
        let _ = writeln!(
            o,
            "| {s} | {} | {ok} | {un} | {} |",
            cells.join(" | "),
            if none.is_empty() {
                "—".into()
            } else {
                none.join(", ")
            }
        );
    }
    o += "\n**Why each task is not expressible**, in the program's own `.none` file:\n\n";
    for ((t, s), p) in &progs {
        if p.verdict == "not expressible" {
            let _ = writeln!(o, "* **{t} {s}.** {}", p.detail);
        }
    }
    o += "\n**Why each unexecuted program is unexecuted:**\n\n";
    let mut un: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for ((t, s), p) in &progs {
        if p.verdict == "unexecuted" && s == "NL" {
            un.entry(p.detail.clone()).or_default().push(t.clone());
        }
    }
    for (why, ts) in &un {
        let _ = writeln!(o, "* {} (NL and RS, one file): {why}.", ts.join(", "));
    }
    o += "\n**Tokens and constructs**, over the tasks every compared surface expresses in the class (so each column counts the same tasks); the schema or preamble each surface's programs are checked with is counted once, separately. Tokens are the surface-neutral tokenizer's (`crates/syntax-study/src/tokens.rs`); constructs are a program's distinct keywords (the surface's list in `crates/syntax-study/corpus/keywords/`) plus its distinct operators, summed over programs.\n\n";
    o += "| class | common tasks | surface | tokens | keywords | operators |\n|---|---|---|--:|--:|--:|\n";
    for (c, name) in CLASSES {
        let ts = tasks_of(c);
        let surfaces: Vec<&str> = SURF
            .iter()
            .copied()
            .filter(|s| ts.iter().any(|t| in_scope(t, s)))
            .collect();
        let common: Vec<&String> = ts
            .iter()
            .filter(|t| {
                surfaces.iter().all(|s| {
                    progs
                        .get(&((*t).clone(), s.to_string()))
                        .is_some_and(|p| p.tokens > 0)
                })
            })
            .collect();
        for s in &surfaces {
            let (mut tk, mut kw, mut op) = (0, 0, 0);
            for t in &common {
                let p = &progs[&((*t).clone(), s.to_string())];
                tk += p.tokens;
                kw += p.kw;
                op += p.ops;
            }
            let _ = writeln!(
                o,
                "| {c} — {name} | {} | {s} | {tk} | {kw} | {op} |",
                common
                    .iter()
                    .map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
    o += "\n| surface | schema file | lines | tokens |\n|---|---|--:|--:|\n";
    for r in &schemas {
        let _ = writeln!(o, "| {} | `{}` | {} | {} |", r[0], r[1], r[2], r[3]);
    }
    o += "\nPRQL has no data-definition language; its programs run over the plain schema in SQL (`schema_plain.sql`), counted with the SQL tokenizer.\n\n";

    // ------------------------------------------------------------ safety
    o += "## Safety — the mutation set\n\n";
    let _ = writeln!(
        o,
        "{} mutants, every site of M1–M6 in every expressible program (the site patterns are in `crates/syntax-study/src/mutate.rs` and listed below). Static-rejection rate = static ÷ (static + runtime + silent); equivalent and unexecuted mutants are reported and excluded (design §6, A1).\n",
        mutants.len()
    );
    o += "| surface | class | mutants | static | runtime | silent | equivalent | unexecuted | rate | range, every unexecuted missed – caught |\n|---|---|--:|--:|--:|--:|--:|--:|--:|---|\n";
    for s in SURF {
        for c in ["Q", "T", "V", "B", "all"] {
            let Some(t) = tally.get(&(s.to_string(), c.to_string())) else {
                continue;
            };
            let _ = writeln!(
                o,
                "| {s} | {c} | {} | {} | {} | {} | {} | {} | {} | {} |",
                t.total(),
                t.stat,
                t.run,
                t.silent,
                t.equiv,
                t.unexec,
                pct(t.rate()),
                t.bounds()
                    .map(|(a, b)| format!("{} – {}", pct(Some(a)), pct(Some(b))))
                    .unwrap_or("—".into())
            );
        }
    }
    let sql = tally
        .get(&("SQL".to_string(), "all".to_string()))
        .copied()
        .unwrap_or_default();
    let _ = writeln!(
        o,
        "\n**SQL's run-time failures, split.** {} of SQL's {} runtime mutants were raised by PostgreSQL's analyser (SQLSTATE class 42: an undefined operator or function, a datatype mismatch) — when the view or function was created or the query was planned, before any row was read. The design's checker for SQL is SQL+C+L (`nilescheck_sql::check_all`), so these count as runtime; had PostgreSQL's analyser been counted as SQL's checker, SQL's overall rate would be {}. The rest ({}) were raised while running, most by the ledger's deferred conservation trigger (`P0001`).\n",
        sql.run_analysis,
        sql.run,
        pct(Some((sql.stat + sql.run_analysis) as f64 / sql.den().max(1) as f64)),
        sql.run - sql.run_analysis
    );
    o += "**By operator**, all classes:\n\n| surface | operator | mutants | static | runtime | silent | equivalent | unexecuted | rate |\n|---|---|--:|--:|--:|--:|--:|--:|--:|\n";
    for ((s, op), t) in &by_op {
        let _ = writeln!(
            o,
            "| {s} | {op} | {} | {} | {} | {} | {} | {} | {} |",
            t.total(),
            t.stat,
            t.run,
            t.silent,
            t.equiv,
            t.unexec,
            pct(t.rate())
        );
    }
    o += "\n";
    o += SITE_PATTERNS;
    o += SAFETY_NOTES;

    // ------------------------------------------------------------ speed
    o += "## Speed — check cost\n\n";
    let _ = writeln!(
        o,
        "In-process, by `tools/memprobe`'s `e30cost` (E14's protocol). Host: {}; toolchain `{}`; measured at commit `{}` ({} worktree). {}.\n",
        cost_hdr.get("host").cloned().unwrap_or_default(),
        cost_hdr.get("toolchain").cloned().unwrap_or_default(),
        cost_hdr.get("commit").cloned().unwrap_or_default(),
        cost_hdr.get("worktree").cloned().unwrap_or_default(),
        cost_hdr.get("protocol").cloned().unwrap_or_default()
    );
    o += "| checker | files | lines read | median pass µs (MAD) | µs per KLOC | peak heap per check, max, bytes | peak heap per KLOC, max, bytes |\n|---|--:|--:|--:|--:|--:|--:|\n";
    for r in &cost_in {
        let _ = writeln!(
            o,
            "| {} | {} | {} | {} ({}) | {} | {} | {} |",
            r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]
        );
    }
    o += "\nA line is a line the checker reads: each SQL file is checked as the 108-line schema followed by the program, and each Niles file as its 39-line schema followed by the program, so most of SQL's lines are its schema's, re-checked every time — the E14 convention.\n\n";
    o += "External, wall time per invocation (5 warm-up passes, 30 timed, the median per file), **not ranked against the in-process figures**:\n\n| surface | binary | files | program lines | median ms per invocation | empty program ms | net ms per KLOC of program |\n|---|---|--:|--:|--:|--:|--:|\n";
    for r in &cost_ext {
        let _ = writeln!(
            o,
            "| {} | {} | {} | {} | {} | {} | {} |",
            r[0], r[1], r[2], r[3], r[4], r[5], r[6]
        );
    }
    o += "\n**Run-time speed is not measured** (design §6): NL and RS lower to the same IR, so their run time is identical by construction, and PRQL runs as the SQL it emits.\n\n";

    // ------------------------------------------------------------ data
    o += "## The dataset, loaded\n\nOne dataset from seed 1 (`e30 gen`; fixtures in `crates/syntax-study/corpus/data/`). Each surface's rows were read back from its own executor and compared with the dataset's (design §4):\n\n| surface | relation | rows | SHA-256 of the canonical rows | equal |\n|---|---|--:|---|---|\n";
    for r in &load {
        let h = if r[3].is_empty() {
            "—".to_string()
        } else {
            format!("`{}`", r[3])
        };
        let _ = writeln!(o, "| {} | {} | {} | {h} | {} |", r[0], r[1], r[2], r[4]);
    }
    o += "\n";
    o += DEVIATIONS;
    o += FINDINGS;
    o += NOT_MEASURED;
    o
}

const SITE_PATTERNS: &str = "**The site patterns** (`mutate.rs`):\n\n| op | where it applies |\n|---|---|\n| M1 | every currency name in a word or a string, whole or `_`-separated (`amt_usd`, `Money<usd>`, `\"usd\"`, `'usd'`): usd → eur, eur → usd, jpy → usd |\n| M2 | Niles: each `debit(` ↔ `credit(`. SQL, inside `insert into postings`: each `row(x)::c` has its sign toggled, and each bare money variable in a `values` tuple's `amt_c` position becomes `row(-(m).minor)::c` |\n| M3 | each top-level conjunct of a condition, with its connective; a one-conjunct condition loses its clause. SQL `where`, `having`, a join's `on`; PL/pgSQL `if`/`elsif`; Niles `.where`/`.filter`/`.having` and a join closure's body, Niles `if`; PRQL `filter` and a join's condition; Soufflé rule bodies of two or more literals. An `if` with one conjunct is a site only when it guards a failure (the mutant makes it `false`) |\n| M4 | Niles `#e`; SQL and PRQL a number compared with `epoch`: e → e + 1 |\n| M5 | SQL `perform resolve_…(…);`, Niles `resolve …`; a `conserve` clause or a conservation or linearity annotation in a program |\n| M6 | each consistency rung, as a word or a string's text, one step down: bounded < monotonic < read_your_writes < snapshot < serializable < ledger_consistent |\n\n";

const SAFETY_NOTES: &str = "**What the classes contain**, read from the rows in `results/E30-syntax/mutants.tsv`:\n\n* **A leg's sign (M2).** Neither checker refuses a flipped leg in these programs: both conservation solvers (Niles's, and SQL+C+L's, which is the same solver) read a parameter as a symbol that may be zero, so `2m ≠ 0` is *undecided* and discharged to the run-time seal, where both ledgers refuse the posting set: runtime, wherever the flip unbalances the set. SQL's T10 is the exception — its reversal is an `insert … select`, and a flipped sign copies the reversed legs unchanged, which conserves and is silent. On Niles, turning a `debit(..)?` into a `credit(..)?` is runtime for a different reason: the typechecker accepts `?` on a value that is not a `Result`, and the interpreter raises (F12).\n* **A consumption removed (M5).** Refused statically on both SQL+C+L (NL0320) and Niles (NL0320), on T04 and T05 — the one operator every checker catches everywhere it applies.\n* **A weakened contract (M6).** No checker refuses a view served one rung lower when nothing else reads it, and no single-session executor can tell the difference, so M6 is *equivalent* on V01, V02, V04 and V05 on every surface. Niles refuses it only where a function declares the rung it reads at (T08, T10: NL0310).\n* **Datalog's grounding check (M3).** Soufflé refuses most dropped body literals because a variable is left ungrounded; that is a static check of Datalog's own, and it is why DL's query rate is high. PRQL has no checker beyond compilation, and every one of its mutants ran.\n* **Equivalent mutants that depend on the data.** B04's value-date conjunct and its epoch + 1, T10's eur and jpy branches, and several currency swaps change nothing on this dataset (no posting of epoch ≤ 5 is valued after 2026-01-21, and none of epoch 6 on or before it; the reversed transaction is usd only). They are equivalent here, not in general; the design fixed one dataset.\n\n";

const DEVIATIONS: &str = "## Deviations from the design\n\n* **D1 — a relational read inside a Niles function is unexecuted.** `niles-interp` has no relational tier, so a function reading a view or a relation (T08's `loan_balance.get`, T10's `postings.where`) reaches it as an unbound name; the harness classes this as unexecuted rather than as the program's failure. A1 named `hold`, `resolve`, `fx` and `authorize` only.\n* **D2 — money arithmetic outside the checked tier is unexecuted.** `niles-interp` also refuses money arithmetic (T02's `m - fee`, T06's `m1 + m2 + m3`) outside its checked tier. As found; A1 did not list it. T03 was predicted unexecuted by A1 (`fx`) and is not: its program states the settlement as four legs, and runs.\n* **D3 — B03 is not expressible in NL/RS, not unexecuted.** A2 expected B03's Niles program to be checked and unexecuted. Writing it found that no Niles form sets a posting's value date (F9), so there is no program to check.\n* **D4 — the mutation site patterns were written after the corpus.** The design fixes the operators and says their per-surface site patterns are data \"fixed with this design\"; they were written after every program existed and before any mutant was counted, and are listed above. One trial run on Q02 was made to test the classifier; it found F11, its output was discarded, and the full run was made from the start after the fix.\n* **D5 — the Niles checker was fixed before the mutation stage.** Writing the corpus found five silent wrong answers in Niles's lowering (F1, F4–F7), and the mutation trial a sixth (F11). The author decided (2026-09-30) to fix them before the mutation stage: six commits, \"Niles fix 1/5\" … \"6/6\", each with a guard test that fails against the old lowering. The corpus was re-run after the fixes; three programs were restored to the spelling first written and Q08 NL became expressible (`crates/syntax-study/corpus/builder-fixes.tsv`). The checkers were frozen from the full mutation run on.\n* **D6 — the Niles harness gained a typed call boundary after the first full mutation run.** The first full run classed four NL/RS mutants of T03 as equivalent: M1 changed a parameter's declared currency, and the interpreter's `call` did not compare the argument's currency with the declaration, so the program ran on the harness's `usd` value regardless. PostgreSQL refuses the same call at run time (\"function … does not exist\"). The harness now refuses a call whose argument currency differs from the parameter's, as a typed caller would, and the four mutants are runtime. This changed Niles's numbers against Niles (equivalent is excluded from the rate; runtime counts against it), and it is the only change made after a full mutation run.\n\n";

const FINDINGS: &str = "## Findings\n\nFound while writing and running the corpus, in order; each names the program that found it.\n\n| id | finding | status |\n|---|---|---|\n| F1 | Niles SQL surface: an aggregate call in `having` (`having sum(amt) < 0.00 usd`) lowered to an argument-less call and dropped every group (Q02 NL: 0 rows for 2) | fixed, Niles fix 1/5, NL0518 |\n| F2 | Niles has no currency literal for a predicate: a bare `usd` is read as a column (NL0501), a string must be used, and an undeclared currency string was accepted; the SQL surface lexes only double-quoted strings | the undeclared string is refused since fix 6/6 (NL0521); the rest stands |\n| F3 | `niles_ir::eval` passes `AsOf`/`ValidAt` through; `niles-interp` refuses `hold`, `resolve`, `fx`, `authorize` | A1; the harness applies the pins |\n| F4 | a keyed join whose two sides have keys of different length never matches, on either surface (Q03: 0 rows for 12) | fixed, Niles fix 2/5, NL0519 |\n| F5 | pipeline `.order_by(\\|r\\| (-r.sum, r.acct))` lowered to an ascending order on the first column (Q05 RS) | fixed, Niles fix 3/5, NL0509; descending is `desc(..)` |\n| F6 | a join to a grouped count matched on the aggregate's *input* key, so account ids were matched against counts (Q04 RS: 10 rows for 2) | fixed, Niles fix 4/5 |\n| F7 | a self-join's qualifiers were ignored, so both sides resolved to the left (Q07/Q08) | fixed, Niles fix 5/5, NL0520 |\n| F8 | no window functions on either Niles surface, and a posting's epoch is not a column | stands; Q07 and Q08 RS are not expressible |\n| F9 | no Niles form sets a posting's value date, so a back-valued correction cannot be written | stands; B03 is not expressible |\n| F10 | the SQL surface refuses `cross join` (NL0516, \"no product operator\") but lowers a comma from-list to a join with empty keys, which the evaluator runs as a product; the pipeline has no product spelling | recorded for the author, not fixed |\n| F11 | the one scalar evaluator (reference evaluator and server scan fold) read every string literal as `0`, so `where cur = \"eur\"` answered the usd rows and `like` compared with 0 (found by the Q02 mutation trial) | fixed, Niles fix 6/6, NL0521 |\n| F12 | the typechecker accepts `?` on a value that is not a `Result` (`credit(a, m)?`); the interpreter raises at run time | recorded, not fixed (the checker is frozen for the study) |\n| F13 | a money literal's currency is dropped when evaluated, and a `Money` column of undeclared currency compares with any currency's literal: Q02's `having sum(amt) < 0.00 eur` over usd rows is accepted and answers as `< 0` | recorded, not fixed |\n\n";

const NOT_MEASURED: &str = "## Not measured, and said so\n\nReadability and learnability (no user study). The programs were written by the builder of two of the checkers — mitigated by the pre-registration, the mechanical mutation and the independent oracle, not removed. Programs were written for correctness, not brevity, and none was shortened after its token count was known. The study has one dataset; equivalence is equivalence on it.\n";
