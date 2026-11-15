//! **E27c's rendering** (cycle 17, C17-12 run 3; `docs/study/E27c-design.md` §7.1 item 8).
//!
//! Written after the points existed, as §7.1 item 8 pre-registered, from §7.1 and §7.3
//! without changing them. It reuses E27b's per-seed judge, size verdicts and joint gate
//! (`render::size_cell`), so a verdict here means what it means in `results/E27b-*.md`.
//!
//! What it adds is E27c's: the rule per REV arm (N and N′-lazy) and per anchor distribution
//! (§7.1 item 6), the swap label's exclusion (item 7), the counted pass's base rows per
//! reconstruction (item 5) and checkpoint bytes beside the view-state total.

use crate::render::{
    cells_by_size, load, series_of, size_cell, size_label, word_of, CellsBySize, Points, SizeCell,
    Study, METRICS_B,
};
use crate::stats::Verdict;
use crate::store::Stored;
use bank_bench::score::median;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

/// The REV arms the rule is evaluated for, each separately (§7.1 item 6).
pub const REV_ARMS: &[&str] = &["N", "N'-lazy"];
/// The arms each REV arm must beat, or match (§7.1 item 6).
pub const OTHERS: &[&str] = &["P+", "H3", "P-inc"];
/// Every arm, in the order the tables list them.
pub const ARMS: &[&str] = &["N", "N'-lazy", "P+", "H3", "P-inc"];

/// E27c's pairs: each REV arm against each other arm, and the two REV arms against each
/// other (descriptive; it enters no clause).
pub const PAIRS_C: &[(&str, &str)] = &[
    ("N", "P+"),
    ("N", "H3"),
    ("N", "P-inc"),
    ("N'-lazy", "P+"),
    ("N'-lazy", "H3"),
    ("N'-lazy", "P-inc"),
    ("N", "N'-lazy"),
];

pub const E27C: Study = Study {
    name: "E27c",
    main_stem: "E27c",
    cli: " --study e27c",
    card: "Cycle 17, round 2, card C17-12 run 3; pre-registered in `docs/study/E27c-design.md` §7 (`f7178fa`), §7.1 (`cbc7efc`), §7.2 (`958191f`) and §7.3 (`d7b521d`).",
    metrics: METRICS_B,
    seed_replicated: &[
        "view_state_bytes_per_key_read",
        "pss_growth_bytes_per_key_read",
    ],
    pairs: PAIRS_C,
    deviations: DEVIATIONS_C,
    seed_gate_note: "",
    checkpoint_line: "",
    budget_line: "",
    memory_intro: "",
    view_state: true,
};

const MEMORY: &str = "view_state_bytes_per_key_read";

/// An anchor distribution (§7.1 item 5): its name and the p99 metrics of its clause.
const DISTRIBUTIONS: &[(&str, &str, [&str; 2])] = &[
    ("q1", "head", ["q1_p99_us", "c4_q1_p99_us"]),
    ("q2", "historical", ["q2_p99_us", "c4_q2_p99_us"]),
];

/// One block of the rendering: a (series, size, history) on one host, at one α.
#[derive(Clone, Debug)]
pub struct Block {
    pub host: &'static str,
    pub series: &'static str,
    pub size: u64,
    pub history: u64,
    pub alpha: &'static str,
}

impl Block {
    fn label(&self) -> String {
        let h = if self.history == 1 && self.size != 10_000 {
            String::new()
        } else {
            format!(" at {}× history", self.history)
        };
        format!("{} {}{h}", self.series, size_label(self.size))
    }
    fn subdir(&self, shape: &str) -> String {
        let mut d = format!("{shape}/a{}", self.alpha);
        if self.history != 1 {
            d += &format!("/h{}", self.history);
        }
        d
    }
}

/// A point the swap label marked (§7.1 item 7): (shape, α, history, series, size, seed),
/// with swap in use before and after, MiB.
type SwapKey = (String, String, u64, String, u64, u64);

fn swap_marked(points_tsv: &Path) -> BTreeMap<SwapKey, (String, String)> {
    let mut out = BTreeMap::new();
    let Ok(text) = std::fs::read_to_string(points_tsv) else {
        return out;
    };
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().unwrap_or("").split('\t').collect();
    let col = |n: &str| head.iter().position(|h| *h == n);
    let (Some(sh), Some(al), Some(se), Some(sz), Some(sd), Some(b), Some(a), Some(af)) = (
        col("shape"),
        col("alpha"),
        col("series"),
        col("size"),
        col("seed"),
        col("swap_used_mib_before"),
        col("swap_used_mib_after"),
        col("swap_affected"),
    ) else {
        return out;
    };
    let hf = col("history_factor");
    for l in lines {
        let f: Vec<&str> = l.split('\t').collect();
        if f.get(af) != Some(&"yes") {
            continue;
        }
        let h = hf
            .and_then(|i| f.get(i))
            .and_then(|x| x.parse().ok())
            .unwrap_or(1);
        out.insert(
            (
                f[sh].to_string(),
                f[al].to_string(),
                h,
                f[se].to_string(),
                f[sz].parse().unwrap_or(0),
                f[sd].parse().unwrap_or(0),
            ),
            (f[b].to_string(), f[a].to_string()),
        );
    }
    out
}

/// The points of one block and shape: (every point, the points the rule may compare).
fn block_points(
    root: &Path,
    b: &Block,
    shape: &str,
    swap: &BTreeMap<SwapKey, (String, String)>,
) -> (Points, Points) {
    let all: Points = load(&root.join(b.subdir(shape)))
        .into_iter()
        .filter(|(k, _)| k.0 == b.series && k.1 == b.size)
        .collect();
    let kept = all
        .iter()
        .filter(|((se, n, seed), _)| {
            !swap.contains_key(&(
                shape.to_string(),
                b.alpha.to_string(),
                b.history,
                se.clone(),
                *n,
                *seed,
            ))
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    (all, kept)
}

/// A clause's verdict, or why there is none.
enum Clause {
    Judged(Verdict),
    NotRun,
    NotEvaluable(String),
}

fn clause(points: &Points, cells: &CellsBySize, b: &Block, x: &str, y: &str, m: &str) -> Clause {
    match size_cell(points, cells, &E27C, b.series, b.size, x, y, m) {
        SizeCell::Missing => Clause::NotEvaluable(format!("{m}: 0 compared seeds, 3 required")),
        SizeCell::NotRun => Clause::NotRun,
        SizeCell::Seeds(v, k) => {
            if k < 3 {
                Clause::NotEvaluable(format!("{m}: {k} compared seeds, 3 required"))
            } else {
                Clause::Judged(v)
            }
        }
        SizeCell::Tally(v, t) => {
            let k: usize = t.iter().sum();
            if k < 3 {
                Clause::NotEvaluable(format!("{m}: {k} compared seeds, 3 required"))
            } else {
                Clause::Judged(v)
            }
        }
    }
}

fn clause_word(c: &Clause) -> String {
    match c {
        Clause::Judged(v) => word_of(v),
        Clause::NotRun => "NOT RUN".into(),
        Clause::NotEvaluable(_) => "not evaluable".into(),
    }
}

/// Median over seeds of each seed's median over its measured runs.
fn median_over_seeds(points: &Points, arm: &str, m: &str, last_only: bool) -> Option<f64> {
    let per: Vec<f64> = points
        .values()
        .filter_map(|st| {
            let v: Vec<f64> = series_of(st, arm, m).0.into_iter().flatten().collect();
            if v.is_empty() {
                None
            } else if last_only {
                v.last().copied()
            } else {
                Some(median(&v))
            }
        })
        .collect();
    (!per.is_empty()).then(|| median(&per))
}

fn state(st: &Stored, arm: &str, name: &str) -> Option<f64> {
    st.kind("state")
        .find(|r| r[1] == arm && r[2] == name)
        .and_then(|r| r[3].parse().ok())
}

fn fmt_num(x: f64) -> String {
    if x >= 100.0 {
        format!("{:.0}", x)
    } else if x >= 10.0 {
        format!("{:.1}", x)
    } else {
        format!("{:.2}", x)
    }
}

fn fmt_bytes(x: f64) -> String {
    if x >= 1e9 {
        format!("{:.2} GB", x / 1e9)
    } else if x >= 1e6 {
        format!("{:.1} MB", x / 1e6)
    } else if x >= 1e3 {
        format!("{:.1} kB", x / 1e3)
    } else {
        format!("{:.0} B", x)
    }
}

/// The blocks Host C holds (§7.1, §7.3, §7.4): the size series, then the history sweep in
/// increasing history, per α.
pub fn blocks(host: &'static str) -> Vec<Block> {
    let mut out = Vec::new();
    for alpha in ["0.6", "1.1"] {
        for (series, size, history) in [
            ("single", 100_000, 1),
            ("multi", 100_000, 1),
            ("single", 1_000_000, 1),
            ("single", 10_000, 1),
            ("single", 10_000, 4),
            ("single", 10_000, 16),
            ("single", 10_000, 64),
        ] {
            out.push(Block {
                host,
                series,
                size,
                history,
                alpha,
            });
        }
    }
    out
}

/// Render `results/E27c.md` from Host C's points (`host_root`); every E27c point is Host C's
/// since §7.4.
pub fn render(host_root: &Path, out: &Path) -> std::io::Result<String> {
    let mut s = String::new();
    let host_txt = std::fs::read_to_string(host_root.join("host.txt")).unwrap_or_default();
    let swap = swap_marked(&host_root.join("points.tsv"));
    let roots: Vec<(&'static str, &Path)> = vec![("Host C", host_root)];

    let _ = writeln!(
        s,
        "# E27c — a REV that holds only what is read: N and N′-lazy against P+, H3 and P-inc\n"
    );
    let _ = writeln!(s, "*{}*\n", E27C.card);
    let _ = writeln!(s, "*Generated by `cargo run -q --release -p comparator -- render --study e27c` from `{}`. Do not edit by hand.*\n",
        host_root.strip_prefix(out.parent().and_then(Path::parent).unwrap_or(Path::new(""))).unwrap_or(host_root).display());
    let _ = writeln!(s, "**What this measures.** Whether a REV that holds only what is read is justified against PostgreSQL with the E14 mechanism (P+), the PostgreSQL ledger with a REV sidecar (H3) and plain incremental full materialisation in PostgreSQL (P-inc). The REV is N (checkpoints on every key) and N′-lazy (checkpoint state only for keys that have been read, built through the key's history at its first read). Both answer a pinned keyed read from the ledger's checkpointed reconstruction (C17-19). E27c's N is therefore **E27b's N with C17-19**; its q2 figures are not E27b's (§7.3).\n");
    let _ = writeln!(s, "**Host C** (`host.txt`):\n\n```\n{}```\n", host_txt);

    // §1 what ran.
    let _ = writeln!(s, "## 1. What ran\n");
    let _ = writeln!(s, "Arms N, N′-lazy, P+, H3, P-inc in every point. Two shapes per (α, seed), E27b's: the main shape (3 warm-ups, 10 measured runs; the counted pass after them) and the targeted p99 shape (2,000 point reads at 1 client, q1 and q2 half each, then 4 × 500; q1 only on multi). Budget 5% of keys, LRU, C = 16. **Swap-marked points** are kept on disk and listed in §6; their latency and memory figures are not compared (§7.1 item 7): the rule's cells below are judged on the remaining seeds, and a cell with fewer than 3 is *not evaluable*.\n");
    let _ = writeln!(s, "| host | α | block | main points (compared) | p99 points (compared) |\n|---|---|---|---|---|");
    for (host, root) in &roots {
        for b in blocks(host) {
            let (ma, mk) = block_points(root, &b, "main", &swap);
            let (pa, pk) = block_points(root, &b, "p99", &swap);
            if ma.is_empty() && pa.is_empty() {
                continue;
            }
            let _ = writeln!(
                s,
                "| {host} | {} | {} | {} ({}) | {} ({}) |",
                b.alpha,
                b.label(),
                ma.len(),
                mk.len(),
                pa.len(),
                pk.len()
            );
        }
    }
    let _ = writeln!(s);

    // §2 the rule.
    let _ = writeln!(s, "## 2. The rule, evaluated (§7.1 item 6)\n");
    let _ = writeln!(s, "For each REV arm R separately, per block and per anchor distribution: **R is justified** iff it beats P+, H3 **and** P-inc by the joint gate, above the floor, on view-state bytes per key read (main shape) **and** on p99 read latency (the p99 shape's query for that distribution, at 1 and at 4 clients). **H3, P+ or P-inc is recommended** iff it matches R (*no difference*) on both. Each cell lists, against P+ · H3 · P-inc, the memory verdict, then p99 at 1 and at 4 clients.\n");
    let _ = writeln!(s, "| host | α | block | R | anchors | vs P+ | vs H3 | vs P-inc | outcome |\n|---|---|---|---|---|---|---|---|---|");
    for (host, root) in &roots {
        for b in blocks(host) {
            let (_, mk) = block_points(root, &b, "main", &swap);
            let (_, pk) = block_points(root, &b, "p99", &swap);
            let (ma, pa) = (
                block_points(root, &b, "main", &swap).0,
                block_points(root, &b, "p99", &swap).0,
            );
            if ma.is_empty() && pa.is_empty() {
                continue;
            }
            let (mc, pc) = (cells_by_size(&mk, &E27C), cells_by_size(&pk, &E27C));
            for r in REV_ARMS {
                for (_, dname, ms) in DISTRIBUTIONS {
                    let mut parts: Vec<(&str, Vec<Clause>)> = Vec::new();
                    for o in OTHERS {
                        let mut v = vec![clause(&mk, &mc, &b, r, o, MEMORY)];
                        for m in ms {
                            v.push(clause(&pk, &pc, &b, r, o, m));
                        }
                        parts.push((o, v));
                    }
                    let outcome = outcome(r, &parts);
                    let cols: Vec<String> = parts
                        .iter()
                        .map(|(_, v)| v.iter().map(clause_word).collect::<Vec<_>>().join(" · "))
                        .collect();
                    let _ = writeln!(
                        s,
                        "| {host} | {} | {} | {} | {dname} | {} | {outcome} |",
                        b.alpha,
                        b.label(),
                        r.replace('\'', "′"),
                        cols.join(" | ")
                    );
                }
            }
        }
    }
    let _ = writeln!(s);

    // §3 medians of the rule's metrics.
    let _ = writeln!(
        s,
        "## 3. The rule's metrics, medians over the compared seeds\n"
    );
    let _ = writeln!(s, "View-state bytes per key read and checkpoint bytes: each seed's last measured run (main shape). p99: each seed's median over its 10 measured runs (p99 shape), µs. A dash: no compared seed carried it.\n");
    let _ = writeln!(s, "| host | α | block | arm | view-state B/key read | checkpoint bytes | q1 p99 | q1 p99 ×4 | q2 p99 | q2 p99 ×4 |\n|---|---|---|---|---|---|---|---|---|---|");
    for (host, root) in &roots {
        for b in blocks(host) {
            let (_, mk) = block_points(root, &b, "main", &swap);
            let (_, pk) = block_points(root, &b, "p99", &swap);
            if mk.is_empty() && pk.is_empty() {
                continue;
            }
            for a in ARMS {
                let d = |x: Option<f64>, f: &dyn Fn(f64) -> String| {
                    x.map(f).unwrap_or_else(|| "—".into())
                };
                let _ = writeln!(
                    s,
                    "| {host} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                    b.alpha,
                    b.label(),
                    a.replace('\'', "′"),
                    d(median_over_seeds(&mk, a, MEMORY, true), &fmt_num),
                    d(
                        median_over_seeds(&mk, a, "checkpoint_bytes", true),
                        &fmt_bytes
                    ),
                    d(median_over_seeds(&pk, a, "q1_p99_us", false), &fmt_num),
                    d(median_over_seeds(&pk, a, "c4_q1_p99_us", false), &fmt_num),
                    d(median_over_seeds(&pk, a, "q2_p99_us", false), &fmt_num),
                    d(median_over_seeds(&pk, a, "c4_q2_p99_us", false), &fmt_num)
                );
            }
        }
    }
    let _ = writeln!(s);

    // §4 counted pass.
    let _ = writeln!(
        s,
        "## 4. Base rows per reconstruction: the counted pass (§7.1 item 5)\n"
    );
    let _ = writeln!(s, "After each main point's measured runs, untimed: 200 q1 reads then 200 q2 reads, the same keys and anchors on every arm, each checked against the oracle. Base rows per reconstruction is Δ`rows_touched` ÷ Δ`misses` per half; the share of reads that reconstructed is Δ`misses` ÷ 200. This is counted work, not latency or memory, so **every** point enters, swap-marked ones included. P+ and P-inc do not count it (*not counted*). Median over seeds, with the range.\n");
    let _ = writeln!(s, "| host | α | block | arm | q1 rows per reconstruction | q1 share reconstructed | q2 rows per reconstruction | q2 share reconstructed | divergences |\n|---|---|---|---|---|---|---|---|---|");
    for (host, root) in &roots {
        for b in blocks(host) {
            let (ma, _) = block_points(root, &b, "main", &swap);
            if ma.is_empty() {
                continue;
            }
            for a in ARMS {
                let mut cells = Vec::new();
                let mut div = 0f64;
                for q in ["q1", "q2"] {
                    let mut per = Vec::new();
                    let mut share = Vec::new();
                    for st in ma.values() {
                        let m = state(st, a, &format!("anchors {q}: misses"));
                        let r = state(st, a, &format!("anchors {q}: rows_touched"));
                        let n = state(st, a, &format!("anchors {q}: reads"));
                        div += state(st, a, &format!("anchors {q}: divergences")).unwrap_or(0.0);
                        if let (Some(m), Some(r), Some(n)) = (m, r, n) {
                            if m > 0.0 {
                                per.push(r / m);
                            }
                            if n > 0.0 {
                                share.push(m / n);
                            }
                        }
                    }
                    if per.is_empty() && share.is_empty() {
                        cells.push("not counted".to_string());
                        cells.push("not counted".to_string());
                    } else {
                        let rng = |v: &[f64]| {
                            let lo = v.iter().cloned().fold(f64::INFINITY, f64::min);
                            let hi = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                            format!("{} ({}–{})", fmt_num(median(v)), fmt_num(lo), fmt_num(hi))
                        };
                        cells.push(if per.is_empty() {
                            "—".into()
                        } else {
                            rng(&per)
                        });
                        cells.push(if share.is_empty() {
                            "—".into()
                        } else {
                            rng(&share)
                        });
                    }
                }
                let _ = writeln!(
                    s,
                    "| {host} | {} | {} | {} | {} | {:.0} |",
                    b.alpha,
                    b.label(),
                    a.replace('\'', "′"),
                    cells.join(" | "),
                    div
                );
            }
        }
    }
    let _ = writeln!(s);

    // §5 predictions.
    let _ = writeln!(s, "## 5. §5's predictions, descriptive\n");
    let _ = writeln!(s, "Prediction 1 (N's checkpoint state grows linearly in H; N′-lazy's with the history of the keys read) and prediction 3 (P+'s checkpoint relation grows linearly in H) are read across history lengths at 10⁴ accounts, all on Host C (§7.4). Prediction 2 (historical-anchor cost flat in H for N and N′-lazy, growing for H3) is read from the q2 rows per reconstruction. Checkpoint bytes: medians over compared seeds; rows per reconstruction: every point (§4).\n");
    let _ = writeln!(s, "| α | history | N checkpoint bytes | N′-lazy checkpoint bytes | P+ checkpoint bytes | q2 rows per reconstruction: N · N′-lazy · H3 |\n|---|---|---|---|---|---|");
    let mut missing = Vec::new();
    for alpha in ["0.6", "1.1"] {
        for h in [1u64, 4, 16, 64] {
            let b = Block {
                host: "Host C",
                series: "single",
                size: 10_000,
                history: h,
                alpha,
            };
            let (ma, mk) = block_points(host_root, &b, "main", &swap);
            if ma.is_empty() {
                missing.push(format!("α {alpha} at {h}×"));
                let _ = writeln!(s, "| {alpha} | {h}× | — | — | — | not run yet |");
                continue;
            }
            let cb = |a: &str| {
                median_over_seeds(&mk, a, "checkpoint_bytes", true)
                    .map(fmt_bytes)
                    .unwrap_or_else(|| "—".into())
            };
            let q2 = |a: &str| {
                let v: Vec<f64> = ma
                    .values()
                    .filter_map(|st| {
                        let m = state(st, a, "anchors q2: misses")?;
                        let r = state(st, a, "anchors q2: rows_touched")?;
                        (m > 0.0).then(|| r / m)
                    })
                    .collect();
                if v.is_empty() {
                    "—".into()
                } else {
                    fmt_num(median(&v))
                }
            };
            let _ = writeln!(
                s,
                "| {alpha} | {h}× | {} | {} | {} | {} · {} · {} |",
                cb("N"),
                cb("N'-lazy"),
                cb("P+"),
                q2("N"),
                q2("N'-lazy"),
                q2("H3")
            );
        }
    }
    if !missing.is_empty() {
        let _ = writeln!(
            s,
            "\n**Not yet readable as a trend:** no points yet for {}.",
            missing.join(", ")
        );
    }
    let _ = writeln!(s);

    // §6 swap.
    let _ = writeln!(s, "## 6. Points marked swap-affected (§7.1 item 7)\n");
    let _ = writeln!(s, "A point is marked when swap in use grew across it, by any amount, as pre-registered. Swap in use, MiB, before → after:\n");
    if swap.is_empty() {
        let _ = writeln!(s, "None.\n");
    } else {
        let _ = writeln!(s, "| shape | α | history | series | size | seed | swap MiB |\n|---|---|---|---|---|---|---|");
        for ((sh, al, h, se, n, seed), (b, a)) in &swap {
            let _ = writeln!(
                s,
                "| {sh} | {al} | {h}× | {se} | {} | {seed} | {b} → {a} |",
                size_label(*n)
            );
        }
        let _ = writeln!(s);
    }

    // §7 refusals, failures.
    let _ = writeln!(s, "## 7. Failures, divergences, refusals\n");
    let mut lines = Vec::new();
    for (host, root) in &roots {
        for b in blocks(host) {
            for shape in ["main", "p99"] {
                let (all, _) = block_points(root, &b, shape, &swap);
                for ((_, _, seed), st) in &all {
                    for r in st.kind("failure") {
                        lines.push(format!(
                            "- {host}, {shape}, α {}, {}, seed {seed}: failure — {}",
                            b.alpha,
                            b.label(),
                            r[1..].join(" ")
                        ));
                    }
                    for r in st.kind("div") {
                        lines.push(format!(
                            "- {host}, {shape}, α {}, {}, seed {seed}: divergence on {}",
                            b.alpha,
                            b.label(),
                            r[1]
                        ));
                    }
                    for r in st.kind("refused") {
                        lines.push(format!(
                            "- {host}, {shape}, α {}, {}, seed {seed}: {} refused {} — {}",
                            b.alpha,
                            b.label(),
                            r[1],
                            r.get(2).cloned().unwrap_or_default(),
                            r.get(3).cloned().unwrap_or_default()
                        ));
                    }
                }
            }
        }
    }
    if lines.is_empty() {
        let _ = writeln!(s, "None: no point recorded a failure, an oracle divergence or a refused end-of-run check.\n");
    } else {
        let _ = writeln!(s, "{}\n", lines.join("\n"));
    }

    // §8 deviations.
    let _ = writeln!(s, "## 8. Deviations and presentation choices\n");
    for d in DEVIATIONS_C {
        let _ = writeln!(s, "- {d}");
    }
    let _ = writeln!(s);
    // The arm's name in prose is N′-lazy; points and verdict words carry the ASCII spelling.
    let s = s.replace("N'-lazy", "N′-lazy");
    std::fs::write(out, &s)?;
    Ok(s)
}

/// E27c's deviations from its design, and the presentation choices made after the points
/// existed (§7.1 item 8 requires the latter to be recorded).
pub const DEVIATIONS_C: &[&str] = &[
    "**α is its own dimension.** §7.1 item 6 evaluates the rule per size range, history length and anchor distribution; α ∈ {0.6, 1.1} is reported as a further dimension, never pooled, because its two workloads differ. A presentation choice, made after the points existed.",
    "**A rule cell with fewer than 3 compared seeds is *not evaluable***, not *mixed*: the size verdict needs 3 seeds (E27b's aggregate), and swap-marked seeds are not compared. A presentation choice, made after the points existed.",
    "**The renderer was written after the points** (pre-registered, §7.1 item 8). It reuses E27b's per-seed judge, size verdict and joint gate unchanged.",
    "**E27c's N is E27b's N with C17-19** (§7.3): its pinned reads start from a checkpoint, and N′-lazy builds a key's checkpoints through its history at its first read.",
    "**10⁶ is not evaluable, by the author's decision of 2026-11-15** (§7.4): the swap label stands as pre-registered; no re-run and no post-hoc threshold.",
    "**The history sweep runs on Host C** (§7.4), not in the container as §7.1 item 1 first fixed: every history length on one host.",
];

/// The outcome of one (R, distribution) row.
fn outcome(r: &str, parts: &[(&str, Vec<Clause>)]) -> String {
    if parts
        .iter()
        .any(|(_, v)| v.iter().any(|c| matches!(c, Clause::NotRun)))
    {
        return "NOT RUN (q2 is not asked on multi at 10⁵, §7.1 item 4)".into();
    }
    let missing: Vec<String> = parts
        .iter()
        .flat_map(|(o, v)| {
            v.iter().filter_map(move |c| match c {
                Clause::NotEvaluable(w) => Some(format!("vs {o}: {w}")),
                _ => None,
            })
        })
        .collect();
    if !missing.is_empty() {
        // One reason per metric, the same for every comparator in practice: say it once.
        let reasons: BTreeSet<String> = missing
            .iter()
            .map(|w| w.split_once(": ").map(|x| x.1).unwrap_or(w).to_string())
            .collect();
        return format!(
            "not evaluable ({})",
            reasons.into_iter().collect::<Vec<_>>().join("; ")
        );
    }
    let all = |o: &str, pred: &dyn Fn(&Verdict) -> bool| {
        parts.iter().filter(|p| p.0 == o).all(|(_, v)| {
            v.iter().all(|c| match c {
                Clause::Judged(x) => pred(x),
                _ => false,
            })
        })
    };
    let wins = |v: &Verdict| *v == Verdict::Better(r.into());
    let same = |v: &Verdict| *v == Verdict::NoDifference;
    let mut holds = Vec::new();
    if OTHERS.iter().all(|o| all(o, &wins)) {
        holds.push(format!("**{} justified**", r.replace('\'', "′")));
    }
    for o in OTHERS {
        if all(o, &same) {
            holds.push(format!("**{o} recommended**"));
        }
    }
    if holds.is_empty() {
        "no clause holds".into()
    } else {
        holds.join("; ")
    }
}

#[cfg(test)]
mod tests {
    //! E27c's rendering guards: the rule's reading (§7.1 item 6) and the swap label's
    //! exclusion (item 7) are what make §2 of `results/E27c.md` true.
    use super::*;

    fn judged(v: Verdict) -> Clause {
        Clause::Judged(v)
    }

    fn three(v: impl Fn() -> Verdict) -> Vec<Clause> {
        vec![judged(v()), judged(v()), judged(v())]
    }

    #[test]
    fn the_rule_reads_as_pre_registered() {
        let win = || Verdict::Better("N".into());
        let same = || Verdict::NoDifference;
        let parts = vec![
            ("P+", three(win)),
            ("H3", three(win)),
            ("P-inc", three(win)),
        ];
        assert_eq!(outcome("N", &parts), "**N justified**");
        // One clause short of a win anywhere: not justified.
        let mut short = vec![
            ("P+", three(win)),
            ("H3", three(win)),
            ("P-inc", three(win)),
        ];
        short[1].1[0] = judged(Verdict::Better("H3".into()));
        assert_eq!(outcome("N", &short), "no clause holds");
        // H3 matches on both: H3 recommended, and only H3.
        let h3 = vec![
            ("P+", three(win)),
            ("H3", three(same)),
            ("P-inc", three(win)),
        ];
        assert_eq!(outcome("N", &h3), "**H3 recommended**");
        // BELOW FLOOR is not a match (E27b's reading: *no difference* only).
        let mut floor = vec![
            ("P+", three(same)),
            ("H3", three(win)),
            ("P-inc", three(win)),
        ];
        floor[0].1[2] = judged(Verdict::BelowFloor);
        assert_eq!(outcome("N", &floor), "no clause holds");
        // Too few compared seeds anywhere: not evaluable, never a verdict.
        let mut few = vec![
            ("P+", three(win)),
            ("H3", three(win)),
            ("P-inc", three(win)),
        ];
        few[2].1[0] = Clause::NotEvaluable("m: 2 compared seeds, 3 required".into());
        assert_eq!(
            outcome("N", &few),
            "not evaluable (m: 2 compared seeds, 3 required)"
        );
        let mut nr = vec![
            ("P+", three(win)),
            ("H3", three(win)),
            ("P-inc", three(win)),
        ];
        nr[0].1[1] = Clause::NotRun;
        assert!(outcome("N", &nr).starts_with("NOT RUN"));
    }

    #[test]
    fn a_swap_marked_point_is_kept_on_disk_and_left_out_of_the_comparison() {
        let dir = std::env::temp_dir().join(format!("e27c-swap-{}", std::process::id()));
        let pts = dir.join("main/a0.6");
        std::fs::create_dir_all(&pts).unwrap();
        for seed in [1u64, 7, 42] {
            std::fs::write(
                pts.join(format!("single-1000000-{seed}.tsv")),
                format!("meta\tseries\tsingle\nmeta\taccounts\t1000000\nmeta\tseed\t{seed}\nload\tN\t1\t2\tx\t2\tx\tn\n"),
            )
            .unwrap();
        }
        std::fs::write(
            dir.join("points.tsv"),
            "shape\talpha\thistory_factor\tseries\tsize\tseed\tstarted_utc\tseconds\tload1_before\tload1_after\tswap_used_mib_before\tswap_used_mib_after\tswap_affected\n\
             main\t0.6\t1\tsingle\t1000000\t1\tt\t1\t1\t1\t0.00\t81.12\tyes\n\
             main\t0.6\t1\tsingle\t1000000\t7\tt\t1\t1\t1\t81.12\t81.12\tno\n\
             main\t0.6\t1\tsingle\t1000000\t42\tt\t1\t1\t1\t81.12\t81.50\tyes\n",
        )
        .unwrap();
        let swap = swap_marked(&dir.join("points.tsv"));
        assert_eq!(swap.len(), 2);
        let b = Block {
            host: "Host C",
            series: "single",
            size: 1_000_000,
            history: 1,
            alpha: "0.6",
        };
        let (all, kept) = block_points(&dir, &b, "main", &swap);
        assert_eq!(all.len(), 3, "every point stays on disk and in the counts");
        assert_eq!(
            kept.keys().map(|k| k.2).collect::<Vec<_>>(),
            vec![7],
            "only the unmarked seed is compared"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
