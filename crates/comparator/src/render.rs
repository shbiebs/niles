//! E27, rendered from the point files on disk — never from memory, never typed.
//!
//! Two files: `results/E27-comparator.md` (header, identity, verdicts per size range,
//! crossovers, medians, memory, divergences, the anomaly probe, H-E1's evidence) and
//! `results/E27-comparator-detail.md` (every cell: arm pair × metric × size × seed with
//! median, MAD, pooled MAD, floor, relative change, MADs apart and verdict).

use crate::stats::{aggregate, crossovers, judge, Cell, Series, Verdict};
use crate::store::Stored;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

/// Every metric, in the order the tables list them, with its unit and meaning.
pub const METRICS: &[(&str, &str)] = &[
    ("q1_p50_us", "q1 point at head, p50 µs, 1 client"),
    ("q1_p99_us", "q1 point at head, p99 µs, 1 client"),
    ("q2_p50_us", "q2 point at anchor, p50 µs, 1 client"),
    ("q2_p99_us", "q2 point at anchor, p99 µs, 1 client"),
    ("q3_p50_us", "q3 statement, p50 µs"),
    ("q4_p50_us", "q4 desk exposure, p50 µs"),
    ("q5_p50_us", "q5 top-10, p50 µs"),
    ("q6_p50_us", "q6 extract, p50 µs"),
    ("c2_q1_p50_us", "q1, p50 µs, 2 clients"),
    ("c2_q1_p99_us", "q1, p99 µs, 2 clients"),
    ("c2_q2_p50_us", "q2, p50 µs, 2 clients"),
    ("c2_q2_p99_us", "q2, p99 µs, 2 clients"),
    ("c4_q1_p50_us", "q1, p50 µs, 4 clients"),
    ("c4_q1_p99_us", "q1, p99 µs, 4 clients"),
    ("c4_q2_p50_us", "q2, p50 µs, 4 clients"),
    ("c4_q2_p99_us", "q2, p99 µs, 4 clients"),
    ("write_p50_us", "two-leg transfer, p50 µs (commit at fsync)"),
    ("write_p99_us", "two-leg transfer, p99 µs"),
    ("commits_per_s", "commits/s, 1 client, at fsync"),
    ("pss_mib", "process memory (PSS), MiB, after the run"),
    (
        "pss_growth_bytes_per_key_read",
        "PSS growth since load ÷ distinct keys read so far, bytes",
    ),
];

/// (series, arm A, arm B, metric) → the size verdicts along the sweep.
type SizeVerdicts = BTreeMap<(String, String, String, String), Vec<(u64, Verdict)>>;
/// (series, size) → every judged cell, with the NOT RUN reason where there is one.
type CellsBySize = BTreeMap<(String, u64), Vec<(Cell, Option<String>)>>;
/// (series, size, family, arm) → (samples, p50s, p99s, p999s) over seeds.
type Pooled = BTreeMap<(String, u64, String, String), (u64, Vec<f64>, Vec<f64>, Vec<f64>)>;
/// (series, size, arm) → (PSS after load per seed, state figures per seed).
type Memory = BTreeMap<(String, u64, String), (Vec<f64>, BTreeMap<String, Vec<f64>>)>;

/// Metrics whose runs are not replicates of one another and are judged across seeds instead,
/// on the last measured run's value. `pss_growth_bytes_per_key_read` divides by the distinct
/// keys read *so far*, which grows run by run, so its dispersion over runs is a trend, not
/// noise — gating it against a warm-up floor would refuse it by construction (the discarded
/// first attempt did exactly that at every point).
pub const SEED_REPLICATED: &[&str] = &["pss_growth_bytes_per_key_read"];

/// The pairs judged. R2-02's four, then R2-03's: the engine rule's (N and P+ against H3,
/// and against T for the write floor), and N and P+ against the two incremental-view
/// systems (H1, H2).
pub const PAIRS: &[(&str, &str)] = &[
    ("N", "P+"),
    ("N", "P"),
    ("P+", "P"),
    ("P", "M"),
    ("N", "H3"),
    ("P+", "H3"),
    ("N", "T"),
    ("P+", "T"),
    ("H3", "T"),
    ("N", "H1"),
    ("P+", "H1"),
    ("N", "H2"),
    ("P+", "H2"),
];

/// Arms whose reads come from an asynchronous replica and are checked at its bracketed
/// position (`h1.rs`, `t.rs`): their exactness figures get their own table.
pub const REPLICA_ARMS: &[&str] = &["H1", "T"];

fn f(x: f64) -> String {
    if !x.is_finite() {
        "—".into()
    } else if x.abs() >= 1000.0 {
        format!("{x:.0}")
    } else if x.abs() >= 10.0 {
        format!("{x:.1}")
    } else {
        format!("{x:.2}")
    }
}

/// A share as a percentage with enough digits that 0.1% is not printed as zero.
fn pct(x: f64) -> String {
    if !x.is_finite() {
        "—".into()
    } else if x == 0.0 {
        "0".into()
    } else {
        format!("{:.3}%", x * 100.0)
    }
}

fn size_label(n: u64) -> String {
    match n {
        1_000 => "10³".into(),
        10_000 => "10⁴".into(),
        100_000 => "10⁵".into(),
        1_000_000 => "10⁶".into(),
        n => n.to_string(),
    }
}

/// All points in a directory, by (series, accounts, seed).
pub fn load(dir: &Path) -> BTreeMap<(String, u64, u64), Stored> {
    let mut out = BTreeMap::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("tsv")
                || p.file_name().and_then(|x| x.to_str()) == Some("probe.tsv")
            {
                continue;
            }
            // Only point files: the calibration table sits in the same directory, has meta
            // lines of its own and no series.
            if let Ok(st) = Stored::read(&p) {
                if !st.series().is_empty() {
                    out.insert((st.series(), st.num("accounts"), st.num("seed")), st);
                }
            }
        }
    }
    out
}

/// The per-run values of `metric` for `arm`: (measured, warm-up), `None` where a run lacked it.
fn series_of(st: &Stored, arm: &str, metric: &str) -> Series {
    let mut runs: BTreeMap<i64, Option<f64>> = BTreeMap::new();
    for r in st.kind("val").filter(|r| r[1] == arm) {
        runs.entry(r[2].parse().unwrap_or(0)).or_insert(None);
    }
    for r in st.kind("val").filter(|r| r[1] == arm && r[3] == metric) {
        runs.insert(r[2].parse().unwrap_or(0), r[4].parse().ok());
    }
    let measured = runs
        .iter()
        .filter(|(i, _)| **i >= 0)
        .map(|(_, v)| *v)
        .collect();
    let warm = runs
        .iter()
        .filter(|(i, _)| **i < 0)
        .map(|(_, v)| *v)
        .collect();
    (measured, warm)
}

/// Every arm loaded in any point of `dir`, so a rendering's query table has a column for
/// the arms it measured and no others.
pub fn arms_in(dir: &Path) -> BTreeSet<String> {
    load(dir).values().flat_map(arms_of).collect()
}

fn arms_of(st: &Stored) -> BTreeSet<String> {
    st.kind("load").map(|r| r[1].clone()).collect()
}

fn diverged(st: &Stored, arm: &str) -> bool {
    st.kind("div").any(|r| r[1] == arm)
}

fn not_run(st: &Stored, arm: &str, metric: &str) -> Option<String> {
    let q = &metric[metric.find('q').unwrap_or(0)..];
    let q = q.get(..2)?;
    st.kind("notrun")
        .find(|r| r[1] == arm && r[2] == q)
        .map(|r| r[3].clone())
}

/// Every judged cell, for one point.
pub fn cells(st: &Stored, study: &Study) -> Vec<(Cell, Option<String>)> {
    let arms = arms_of(st);
    let mut out = Vec::new();
    let failed = st.kind("failure").next().is_some();
    for (a, b) in study.pairs {
        if !arms.contains(*a) || !arms.contains(*b) {
            continue;
        }
        for (m, _) in study.metrics {
            let nr = not_run(st, a, m).or_else(|| not_run(st, b, m));
            let (sa, sb) = (series_of(st, a, m), series_of(st, b, m));
            // A metric neither arm reported in any run is absent by the run's design (the
            // p99 shape has no 2-client phase), not refused: no row.
            let reported = |x: &Series| x.0.iter().chain(&x.1).any(Option::is_some);
            if nr.is_none() && !reported(&sa) && !reported(&sb) {
                continue;
            }
            let mut c = judge(
                m,
                a,
                b,
                &sa,
                &sb,
                (diverged(st, a), diverged(st, b)),
                st.num("accounts"),
                st.num("seed"),
            );
            if failed {
                c.verdict = Verdict::Refused("the point recorded a failure".into());
            }
            out.push((c, nr));
        }
    }
    out
}

/// One cell of §2's size table: what the seeds of one (series, size) say about a pair.
enum SizeCell {
    /// Neither arm of the pair was in these points, or the metric is absent by design.
    Missing,
    NotRun,
    /// A seed-replicated metric: the verdict across seeds and how many carried it.
    Seeds(Verdict, usize),
    /// The size verdict over per-seed cells, with the tally
    /// (A wins, B wins, no difference or below floor, refused).
    Tally(Verdict, [usize; 4]),
}

impl SizeCell {
    fn verdict(&self) -> Option<&Verdict> {
        match self {
            SizeCell::Seeds(v, _) | SizeCell::Tally(v, _) => Some(v),
            _ => None,
        }
    }
}

type Points = BTreeMap<(String, u64, u64), Stored>;

fn cells_by_size(points: &Points, study: &Study) -> CellsBySize {
    let mut all_cells: CellsBySize = BTreeMap::new();
    for ((se, n, _), st) in points {
        all_cells
            .entry((se.clone(), *n))
            .or_default()
            .extend(cells(st, study));
    }
    all_cells
}

#[allow(clippy::too_many_arguments)]
fn size_cell(
    points: &Points,
    all_cells: &CellsBySize,
    study: &Study,
    se: &str,
    n: u64,
    a: &str,
    b: &str,
    m: &str,
) -> SizeCell {
    let cs: Vec<&(Cell, Option<String>)> = all_cells
        .get(&(se.to_string(), n))
        .map(|v| {
            v.iter()
                .filter(|(c, _)| c.a == a && c.b == b && c.metric == m)
                .collect()
        })
        .unwrap_or_default();
    if cs.is_empty() {
        return SizeCell::Missing;
    }
    if cs.iter().any(|(_, nr)| nr.is_some()) {
        return SizeCell::NotRun;
    }
    if study.seed_replicated.contains(&m) {
        let last = |arm: &str| -> Vec<f64> {
            points
                .iter()
                .filter(|(k, _)| k.0 == se && k.1 == n)
                .filter_map(|(_, st)| series_of(st, arm, m).0.into_iter().flatten().last())
                .collect()
        };
        let (va, vb) = (last(a), last(b));
        let refused = points
            .iter()
            .filter(|(k, _)| k.0 == se && k.1 == n)
            .any(|(_, st)| {
                diverged(st, a) || diverged(st, b) || st.kind("failure").next().is_some()
            });
        let v = if refused {
            Verdict::Refused("oracle divergence or failure at this size".into())
        } else {
            crate::stats::judge_seeds(m, a, b, &va, &vb)
        };
        return SizeCell::Seeds(v, va.len().min(vb.len()));
    }
    let cells: Vec<&Cell> = cs.iter().map(|(c, _)| c).collect();
    let v = aggregate(&cells);
    let count = |f: &dyn Fn(&Verdict) -> bool| cells.iter().filter(|c| f(&c.verdict)).count();
    let wa = count(&|v| *v == Verdict::Better(a.into()));
    let wb = count(&|v| *v == Verdict::Better(b.into()));
    let nd = count(&|v| matches!(v, Verdict::NoDifference | Verdict::BelowFloor));
    let rf = count(&|v| matches!(v, Verdict::Refused(_)));
    SizeCell::Tally(v, [wa, wb, nd, rf])
}

/// Render both files. Returns the text of the main file.
///
/// `p99_dir` is E27b's: the targeted re-run's points, where the engine rule's p99 clause is
/// judged (design §4.2, §6). `None` for every E27 rendering.
pub fn render(
    dir: &Path,
    main: &Path,
    detail: &Path,
    arm_lines: &[(String, String)],
    sql_table: &str,
    shape: &Shape,
    p99_dir: Option<&Path>,
) -> std::io::Result<String> {
    let study = shape.study;
    let detail_name = detail
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("detail")
        .to_string();
    let points = load(dir);
    let all_arms: BTreeSet<String> = points.values().flat_map(arms_of).collect();
    let mut s = String::new();
    let mut d = String::new();
    let series: BTreeSet<String> = points.keys().map(|k| k.0.clone()).collect();
    let sizes: BTreeSet<u64> = points.keys().map(|k| k.1).collect();
    let seeds: BTreeSet<u64> = points.keys().map(|k| k.2).collect();
    let first = points.values().next();
    let meta = |k: &str| {
        first
            .and_then(|p| p.meta.get(k).cloned())
            .unwrap_or_default()
    };
    let commits: BTreeSet<String> = points
        .values()
        .filter_map(|p| p.meta.get("commit").cloned())
        .collect();
    let dirty = points
        .values()
        .any(|p| p.meta.get("worktree").map(|w| w != "clean").unwrap_or(true));

    let _ = writeln!(s, "# {}\n", shape.title);
    let _ = writeln!(s, "*Generated by `cargo run --release -p comparator -- render{}{}` from the point files in `results/{}/`. Every number below is read from those files; none is typed. {}*\n", study.cli, shape.flag, dir.file_name().and_then(|x| x.to_str()).unwrap_or("?"), study.card);
    if !shape.preface.is_empty() {
        let _ = writeln!(s, "{}\n", shape.preface);
    }
    let _ = writeln!(s, "## Provenance\n");
    let _ = writeln!(s, "| field | value |\n|---|---|");
    let _ = writeln!(
        s,
        "| commit(s) measured | {} |",
        commits
            .iter()
            .map(|c| format!("`{}`", &c[..c.len().min(12)]))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(
        s,
        "| worktree at measurement | {} |",
        if dirty {
            "**MODIFIED in at least one point — not a clean-build result**"
        } else {
            "clean (crates/, Cargo.toml, Cargo.lock) at every point"
        }
    );
    let mut by_build: BTreeMap<(String, String), usize> = BTreeMap::new();
    for p in points.values() {
        *by_build
            .entry((
                p.meta
                    .get("commit")
                    .map(|c| c[..c.len().min(7)].to_string())
                    .unwrap_or_default(),
                p.meta.get("worktree").cloned().unwrap_or_default(),
            ))
            .or_default() += 1;
    }
    let _ = writeln!(
        s,
        "| points by HEAD and worktree state when written | {} (the build that measured each is stated under Deviations) |",
        by_build
            .iter()
            .map(|((c, w), n)| format!("`{c}` {w}: {n}"))
            .collect::<Vec<_>>()
            .join("; ")
    );
    let _ = writeln!(s, "| host | {} — {} CPUs, {} MiB (the cloud container: every concurrency figure is on 2 cores) |", meta("host"), meta("cpus"), meta("mem_mib"));
    // Every distinct value, so an empty one (see Deviations) is shown beside the rest.
    let chains: BTreeSet<String> = points
        .values()
        .map(|p| {
            let t = p.meta.get("toolchain").cloned().unwrap_or_default();
            if t.is_empty() {
                "(empty)".to_string()
            } else {
                format!("`{t}`")
            }
        })
        .collect();
    let _ = writeln!(
        s,
        "| toolchain | {} |",
        chains.into_iter().collect::<Vec<_>>().join("; ")
    );
    let _ = writeln!(s, "| PostgreSQL | {} |", meta("postgres"));
    let dates: BTreeSet<String> = points
        .values()
        .filter_map(|p| p.meta.get("date").cloned())
        .collect();
    let _ = writeln!(
        s,
        "| measured | {} → {} |",
        dates.iter().next().cloned().unwrap_or_default(),
        dates.iter().last().cloned().unwrap_or_default()
    );
    let _ = writeln!(s, "| universe | declared synthetic, α = 0.6 (the author's W5: ~10% of activity in the busiest 1%, ~40% in the busiest 10%); ten transactions of history per account in batches of 200 (one epoch per batch); 3% back-valued up to 22 business days; `multi` series: 35% of accounts hold a second currency, transfers between two such accounts in it half the time (W11) |");
    let _ = writeln!(
        s,
        "| sizes | {} accounts |",
        sizes
            .iter()
            .map(|n| size_label(*n))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(
        s,
        "| seeds | {} |",
        seeds
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(
        s,
        "| series | {} |",
        series.iter().cloned().collect::<Vec<_>>().join(", ")
    );
    let _ = writeln!(s, "| checkpoint_interval | {} |", study.checkpoint_line);
    let _ = writeln!(s, "| residency budget | {} |", study.budget_line);
    if all_arms.contains("H1") {
        let _ = writeln!(s, "| licences | ReadySet (H1) is BSL 1.1, used here as non-production benchmark use; TigerBeetle (T) Apache-2.0; pg_ivm (H2) PostgreSQL Licence; RabbitMQ (T's change stream) MPL-2.0 |");
    }
    let _ = writeln!(s, "| run shape | {} |", shape.run_shape);
    let _ = writeln!(s, "| gate | ≥ 10% and ≥ 3 pooled MADs; floor = 3 × pooled MAD of the warm-up runs; refused on checksum mismatch, oracle divergence, or warm-up MAD > 15% of its median |\n");

    let _ = writeln!(s, "### The arms\n");
    let _ = writeln!(s, "| arm | what it is |\n|---|---|");
    let probed = std::fs::read_to_string(dir.join("probe.tsv")).unwrap_or_default();
    let probe_arms: BTreeSet<&str> = probed
        .lines()
        .filter(|l| l.starts_with("probe\t"))
        .filter_map(|l| l.split('\t').nth(2))
        .collect();
    let arm_lines: Vec<&(String, String)> = arm_lines
        .iter()
        .filter(|(n, _)| all_arms.contains(n) || probe_arms.contains(n.as_str()))
        .collect();
    for (n, dsc) in &arm_lines {
        let _ = writeln!(s, "| {n} | {dsc} |");
    }
    let _ = writeln!(s, "\nPostgreSQL configuration, identical on every PostgreSQL arm (§5.3), each non-default with its reason:\n");
    let _ = writeln!(s, "| setting | value | why |\n|---|---|---|");
    for (k, v, why) in crate::pgcluster::SETTINGS {
        let _ = writeln!(s, "| `{k}` | `{v}` | {why} |");
    }
    let present: BTreeSet<&str> = arm_lines.iter().map(|(n, _)| n.as_str()).collect();
    for (a, k, v, why) in crate::pgcluster::ARM_SETTINGS {
        if present.contains(a) {
            let _ = writeln!(s, "| `{k}` ({a} only) | `{v}` | {why} |");
        }
    }
    let _ = writeln!(
        s,
        "\n### The six questions, as each arm is asked them (§5.5)\n\n{sql_table}"
    );
    if !study.view_state {
        let _ = writeln!(
            s,
            "N's q3 and q4 are NOT RUN: {}.\n",
            crate::arms::N_NOT_RUN
        );
    }
    if all_arms.contains("T") {
        let _ = writeln!(s, "T's q3–q6 are NOT RUN: {}.\n", crate::t::T_NOT_RUN);
    }
    if all_arms.contains("H1") {
        let routing: BTreeSet<String> = points
            .values()
            .flat_map(|st| {
                st.kind("load")
                    .filter(|r| r[1] == "H1")
                    .filter_map(|r| r.get(7).cloned())
                    .collect::<Vec<_>>()
            })
            .filter_map(|n| {
                n.split("routing after the load (EXPLAIN LAST STATEMENT): ")
                    .nth(1)
                    .map(String::from)
            })
            .map(|r| {
                // The cache ids differ per load; the destinations are what matter.
                r.split(", ")
                    .map(|x| x.split('(').next().unwrap_or(x).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .collect();
        let _ = writeln!(s, "H1's routing, as `EXPLAIN LAST STATEMENT` reported it after every load (distinct forms): {}. A question routed `upstream` is answered by H1's PostgreSQL through ReadySet's proxy, not by a cache.\n", routing.into_iter().collect::<Vec<_>>().join(" | "));
    }

    let _ = writeln!(s, "### Deviations from the specification, and repairs\n");
    for line in study.deviations {
        let _ = writeln!(s, "- {line}");
    }
    let _ = writeln!(s);

    // Identity.
    let _ = writeln!(s, "## 1. Every arm held the same data\n");
    let _ = writeln!(s, "Per point: the universe's legs (count and SHA-256 of the canonical `acct,cur,amt` multiset), then each arm's after loading and after the last run. `=` means equal to the universe's (load) or to the oracle's (end).\n");
    let _ = writeln!(s, "| series | size | seed | keys | budget | legs loaded | SHA-256 (prefix) | legs at end | arms equal at load | arms equal at end | failures |\n|---|---|---|---|---|---|---|---|---|---|---|");
    for ((se, n, seed), st) in &points {
        let ul = st.meta.get("universe_sha256").cloned().unwrap_or_default();
        let el = st.meta.get("end_sha256").cloned().unwrap_or_default();
        let mut eq_load = Vec::new();
        let mut eq_end = Vec::new();
        for r in st.kind("load") {
            eq_load.push(format!("{} {}", r[1], if r[4] == ul { "=" } else { "≠" }));
            eq_end.push(format!(
                "{} {}",
                r[1],
                if !el.is_empty() && r[6] == el {
                    "="
                } else {
                    "≠"
                }
            ));
        }
        let fails: Vec<String> = st.kind("failure").map(|r| r[1].clone()).collect();
        let _ = writeln!(
            s,
            "| {se} | {} | {seed} | {} | {} | {} | `{}` | {} | {} | {} | {} |",
            size_label(*n),
            st.num("keys"),
            st.num("budget"),
            st.num("universe_legs"),
            &ul[..ul.len().min(16)],
            st.num("end_legs"),
            eq_load.join(", "),
            eq_end.join(", "),
            if fails.is_empty() {
                "none".into()
            } else {
                fails.join("; ")
            }
        );
    }

    // Verdicts per size range.
    let _ = writeln!(s, "\n## 2. Verdicts per size range, and crossovers\n");
    let _ = writeln!(s, "{}\n", study.seed_gate_note);
    let _ = writeln!(s, "Each cell is the size verdict over the five seeds (rule in `crates/comparator/src/stats.rs`, fixed before the first measured run): an arm when at least three seeds give it and none gives the other; *no difference* when at least three are no difference or below floor; REFUSED when three or more seeds are refused; *mixed* otherwise. In brackets, the seed tally: arm-A wins / arm-B wins / no difference or below floor / refused. Every underlying cell is in `results/{detail_name}`.\n");
    let mut size_verdicts: SizeVerdicts = BTreeMap::new();
    let all_cells = cells_by_size(&points, study);
    for se in &series {
        let _ = writeln!(s, "### Series `{se}`\n");
        let _ = write!(s, "| pair | metric |");
        for n in &sizes {
            let _ = write!(s, " {} |", size_label(*n));
        }
        let _ = writeln!(s, " crossover |");
        let _ = writeln!(s, "|---|---|{}---|", "---|".repeat(sizes.len()));
        for (a, b) in study.pairs {
            for (m, desc) in study.metrics {
                let mut row = format!("| {a} vs {b} | {desc} |");
                let mut by_size = Vec::new();
                let mut any = false;
                for n in &sizes {
                    match size_cell(&points, &all_cells, study, se, *n, a, b, m) {
                        SizeCell::Missing => row += " — |",
                        SizeCell::NotRun => {
                            any = true;
                            row += " NOT RUN |";
                        }
                        SizeCell::Seeds(v, k) => {
                            any = true;
                            row += &format!(" {} (across {k} seeds) |", v.word());
                            by_size.push((*n, v));
                        }
                        SizeCell::Tally(v, [wa, wb, nd, rf]) => {
                            any = true;
                            let word = match &v {
                                Verdict::Refused(w) if w == "mixed" => "mixed".to_string(),
                                other => other.word(),
                            };
                            row += &format!(" {word} ({wa}/{wb}/{nd}/{rf}) |");
                            by_size.push((*n, v));
                        }
                    }
                }
                if !any {
                    continue;
                }
                let xs = crossovers(&by_size);
                row += &if xs.is_empty() {
                    " none in range |".to_string()
                } else {
                    format!(
                        " {} |",
                        xs.iter()
                            .map(|(lo, hi, v0, v1)| format!(
                                "between {} and {}: {} → {}",
                                size_label(*lo),
                                size_label(*hi),
                                v0.word(),
                                v1.word()
                            ))
                            .collect::<Vec<_>>()
                            .join("; ")
                    )
                };
                let _ = writeln!(s, "{row}");
                size_verdicts.insert(
                    (se.clone(), a.to_string(), b.to_string(), m.to_string()),
                    by_size,
                );
            }
        }
        let _ = writeln!(s);
    }

    // Medians, descriptive.
    let _ = writeln!(s, "## 3. Medians (descriptive)\n");
    let _ = writeln!(s, "The median over seeds of each seed's median over its ten measured runs. Descriptive only — the verdicts are §2's.\n");
    for se in &series {
        let arms: BTreeSet<String> = points
            .iter()
            .filter(|(k, _)| &k.0 == se)
            .flat_map(|(_, st)| arms_of(st))
            .collect();
        let _ = writeln!(s, "### Series `{se}`\n");
        let _ = write!(s, "| metric | size |");
        for a in &arms {
            let _ = write!(s, " {a} |");
        }
        let _ = writeln!(s, "\n|---|---|{}", "---|".repeat(arms.len()));
        for (m, desc) in study.metrics {
            for n in &sizes {
                let mut row = format!("| {desc} | {} |", size_label(*n));
                let mut any_value = false;
                for a in &arms {
                    let meds: Vec<f64> = points
                        .iter()
                        .filter(|(k, _)| &k.0 == se && k.1 == *n)
                        .filter_map(|(_, st)| {
                            let (mv, _) = series_of(st, a, m);
                            let v: Vec<f64> = mv.into_iter().flatten().collect();
                            if v.is_empty() {
                                None
                            } else {
                                Some(bank_bench::score::median(&v))
                            }
                        })
                        .collect();
                    let nr = points
                        .iter()
                        .filter(|(k, _)| &k.0 == se && k.1 == *n)
                        .any(|(_, st)| not_run(st, a, m).is_some());
                    row += &if nr {
                        " NOT RUN |".to_string()
                    } else if meds.is_empty() {
                        " — |".to_string()
                    } else {
                        any_value = true;
                        format!(" {} |", f(bank_bench::score::median(&meds)))
                    };
                }
                // A metric no arm reported at this size is absent by design: no row.
                if any_value {
                    let _ = writeln!(s, "{row}");
                }
            }
        }
        let _ = writeln!(s);
    }

    // Tails, pooled.
    let _ = writeln!(s, "## 4. Tails (pooled, descriptive)\n");
    let _ = writeln!(s, "p999 needs a thousand samples and one run has at most ~110 of any query, so tails are pooled over the ten measured runs and the five seeds of a size. Descriptive only: a pooled figure has no per-run dispersion to gate on.\n");
    let _ = writeln!(s, "| series | size | family | arm | n | p50 µs | p99 µs | p999 µs |\n|---|---|---|---|---|---|---|---|");
    let mut pooled: Pooled = BTreeMap::new();
    for ((se, n, _), st) in &points {
        for r in st.kind("pooled") {
            let e = pooled
                .entry((se.clone(), *n, r[2].clone(), r[1].clone()))
                .or_default();
            e.0 += r[3].parse::<u64>().unwrap_or(0);
            e.1.push(r[4].parse().unwrap_or(f64::NAN));
            e.2.push(r[5].parse().unwrap_or(f64::NAN));
            e.3.push(r[6].parse().unwrap_or(f64::NAN));
        }
    }
    for ((se, n, fam, arm), (cnt, p50, p99, p999)) in &pooled {
        if !(fam == "q1" || fam == "q2" || fam == "write" || fam == "c4_q1" || fam == "c4_q2") {
            continue;
        }
        let _ = writeln!(
            s,
            "| {se} | {} | {fam} | {arm} | {cnt} | {} | {} | {} |",
            size_label(*n),
            f(bank_bench::score::median(p50)),
            f(bank_bench::score::median(p99)),
            f(bank_bench::score::median(p999))
        );
    }
    let _ = writeln!(s, "\n*Per seed the pooled p50/p99/p999 are computed over that seed's 10 runs; the table shows their median over seeds and `n` their total count.*\n");

    // Memory and state.
    let _ = writeln!(s, "## 5. Memory and state at the end of the last run\n");
    if study.view_state {
        view_state_table(&mut s, &points, &series, &sizes);
    }
    let _ = writeln!(s, "{}\n", study.memory_intro);
    let _ = writeln!(s, "| series | size | arm | PSS loaded MiB | figure | median over seeds |\n|---|---|---|---|---|---|");
    let mut mem: Memory = BTreeMap::new();
    for ((se, n, _), st) in &points {
        for r in st.kind("pss_loaded") {
            mem.entry((se.clone(), *n, r[1].clone()))
                .or_default()
                .0
                .push(r[2].parse::<f64>().unwrap_or(f64::NAN) / 1048576.0);
        }
        for r in st.kind("state") {
            mem.entry((se.clone(), *n, r[1].clone()))
                .or_default()
                .1
                .entry(r[2].clone())
                .or_default()
                .push(r[3].parse().unwrap_or(f64::NAN));
        }
    }
    for ((se, n, arm), (pl, figs)) in &mem {
        let first = format!(
            "| {se} | {} | {arm} | {} |",
            size_label(*n),
            f(bank_bench::score::median(pl))
        );
        if figs.is_empty() {
            let _ = writeln!(s, "{first} — | — |");
        }
        for (i, (k, v)) in figs.iter().enumerate() {
            if i == 0 {
                let _ = writeln!(s, "{first} {k} | {} |", f(bank_bench::score::median(v)));
            } else {
                let _ = writeln!(s, "| | | | | {k} | {} |", f(bank_bench::score::median(v)));
            }
        }
    }

    // Deadlocks.
    let _ = writeln!(s, "\n## 6a. Reads retried after a deadlock\n");
    let _ = writeln!(s, "A read chosen as a deadlock victim (SQLSTATE 40P01) is retried, up to 20 times; its latency includes every attempt and its answer is checked like any other. Total over the ten measured runs and five seeds.\n");
    let _ = writeln!(
        s,
        "| series | size | arm | reads retried |\n|---|---|---|---|"
    );
    let mut dl: BTreeMap<(String, u64, String), f64> = BTreeMap::new();
    for ((se, n, _), st) in &points {
        for r in st
            .kind("val")
            .filter(|r| r[3] == "read_deadlock_retries" && !r[2].starts_with('-'))
        {
            *dl.entry((se.clone(), *n, r[1].clone())).or_default() +=
                r[4].parse::<f64>().unwrap_or(0.0);
        }
    }
    for ((se, n, arm), v) in &dl {
        let _ = writeln!(s, "| {se} | {} | {arm} | {v:.0} |", size_label(*n));
    }

    // Divergences.
    let _ = writeln!(s, "\n## 6. Oracle divergences\n");
    let divs: Vec<String> = points
        .iter()
        .flat_map(|((se, n, seed), st)| {
            st.kind("div")
                .map(|r| {
                    format!(
                        "| {se} | {} | {seed} | {} | {} | {} | {} |",
                        size_label(*n),
                        r[1],
                        r[2],
                        r[3],
                        r[4].chars().take(300).collect::<String>()
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    if divs.is_empty() {
        let _ = writeln!(s, "**None.** Every read of every measured and warm-up run, on every arm, at every point, returned the oracle's answer.\n");
    } else {
        let _ = writeln!(s, "| series | size | seed | arm | run | count | first examples |\n|---|---|---|---|---|---|---|");
        for l in divs {
            let _ = writeln!(s, "{l}");
        }
        let _ = writeln!(s);
    }

    // Probe.
    let _ = writeln!(s, "## 7. The five anomalies (§5.7)\n");
    let probe = std::fs::read_to_string(dir.join("probe.tsv")).unwrap_or_default();
    if probe.is_empty() && !shape.flag.is_empty() {
        let _ = writeln!(
            s,
            "*Not part of this shape: the probe runs once, with the main sweep (`results/E27-comparator.md` §7).*\n"
        );
    } else if probe.is_empty() {
        let _ = writeln!(s, "*Not run yet.*\n");
    } else {
        let _ = writeln!(s, "Driver and predicates: `crates/comparator/src/probe.rs`. 10³ accounts, budget 5 against 20 hot keys, 4 readers at the head, one appender, and in the second mode one mutator rewriting a historical leg (+10⁹ / −10⁹ in one transaction). Counts are reads (or resident slots, for lost delta) matching each predicate; deadlocks are 40P01 errors on any connection. M and P are a full materialised view with no partial state and no anchor on a read, so the five — anomalies of partial maintenance — have nothing to act on; the mutable arm with partial state is M+ (P+ without the append-only trigger); H1's is H1M (H1 without it). H1's reads carry no anchor: each is bracketed by ReadySet's applied position before and after it and is right if it matches the truth at any epoch in the bracket. Since R2-03 the mutating mode holds a read to the mutations that had landed when it ran, not to the final base (see Deviations).\n");
        let _ = writeln!(s, "| arm | mode | rep | reads | appends | mutations applied/attempted | double application | skipped delta | lost delta | upquery race | upquery deadlock | other divergence | mutation refused with |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|");
        let mut exhibited: BTreeMap<(String, String), BTreeSet<&str>> = BTreeMap::new();
        for line in probe.lines() {
            let r: Vec<&str> = line.split('\t').collect();
            if r[0] != "probe" || r.len() < 17 {
                continue;
            }
            let _ = writeln!(
                s,
                "| {} | {} | {} | {} | {} | {}/{} | {} | {} | {} | {} | {} | {} | {} |",
                r[2],
                r[3],
                r[1],
                r[5],
                r[6],
                r[8],
                r[7],
                r[10],
                r[11],
                r[12],
                r[13],
                r[14],
                r[15],
                if r[9].is_empty() { "—" } else { r[9] }
            );
            let e = exhibited
                .entry((r[2].to_string(), r[3].to_string()))
                .or_default();
            for (i, name) in [
                (10, "double application"),
                (11, "skipped delta"),
                (12, "lost delta"),
                (13, "upquery race"),
                (14, "upquery deadlock"),
            ] {
                if r[i].parse::<u64>().unwrap_or(0) > 0 {
                    e.insert(name);
                }
            }
        }
        let _ = writeln!(s, "\n**Exhibited, per arm and mode, in any repetition:**\n");
        for ((arm, mode), e) in &exhibited {
            let _ = writeln!(
                s,
                "- {arm}, {mode}: {}",
                if e.is_empty() {
                    "none".into()
                } else {
                    e.iter().cloned().collect::<Vec<_>>().join(", ")
                }
            );
        }
        let failures: Vec<&str> = probe.lines().filter(|l| l.starts_with("failure")).collect();
        for l in failures {
            let _ = writeln!(s, "- probe failure: `{}`", l.replace('\t', " "));
        }
        let on = |arm: &str| {
            exhibited
                .iter()
                .filter(|((a, _), _)| a == arm)
                .flat_map(|(_, e)| e.iter().copied())
                .collect::<BTreeSet<&str>>()
        };
        let list = |e: &BTreeSet<&str>| {
            if e.is_empty() {
                "none".to_string()
            } else {
                e.iter().cloned().collect::<Vec<_>>().join(", ")
            }
        };
        let (n, mplus, h1, h1m) = (on("N"), on("M+"), on("H1"), on("H1M"));
        let _ = writeln!(
            s,
            "\nThe pre-registration (§5.7) was: at least one exhibited on M or H1, none on N. M is out of scope as stated above, and its partial-state counterpart M+ stands in for it. **Evaluated:** on N — {}; on M+ — {}; on H1 — {}; on H1M — {}. The prediction {} on N, and {} on M+ or H1.\n",
            list(&n),
            list(&mplus),
            list(&h1),
            list(&h1m),
            if n.is_empty() { "holds" } else { "fails" },
            if mplus.is_empty() && h1.is_empty() && h1m.is_empty() {
                "fails (nothing exhibited)"
            } else {
                "holds"
            },
        );
    }

    // Replica exactness.
    if REPLICA_ARMS.iter().any(|a| all_arms.contains(*a)) {
        let _ = writeln!(
            s,
            "## 7a. Answers from a replica: exactness and staleness (H1, T)\n"
        );
        let _ = writeln!(s, "H1 (ReadySet) and T (the REV fed by TigerBeetle's CDC) answer from state that trails the ledger. Each read is bracketed by the replica's applied position just before and just after it. An answer equal to the oracle at the head is fresh; one equal to it only at an earlier epoch in the bracket is **stale**, not a divergence; one equal to it at no epoch in the bracket is **inexact** — a divergence, which refuses the arm's comparisons at that point (§6). Per series, size and arm: the median over seeds of each seed's median over its measured runs, and the largest lag seen.\n");
        let _ = writeln!(s, "| series | size | arm | inexactness rate (% of reads) | stale share (% of reads) | max stale lag (epochs) |\n|---|---|---|---|---|---|");
        for se in &series {
            for n in &sizes {
                for arm in REPLICA_ARMS {
                    let at: Vec<&Stored> = points
                        .iter()
                        .filter(|(k, _)| &k.0 == se && k.1 == *n)
                        .map(|(_, st)| st)
                        .filter(|st| arms_of(st).contains(*arm))
                        .collect();
                    if at.is_empty() {
                        continue;
                    }
                    let med_over_seeds = |m: &str| -> f64 {
                        let v: Vec<f64> = at
                            .iter()
                            .filter_map(|st| {
                                let v: Vec<f64> =
                                    series_of(st, arm, m).0.into_iter().flatten().collect();
                                (!v.is_empty()).then(|| bank_bench::score::median(&v))
                            })
                            .collect();
                        if v.is_empty() {
                            f64::NAN
                        } else {
                            bank_bench::score::median(&v)
                        }
                    };
                    let lag = at
                        .iter()
                        .flat_map(|st| {
                            series_of(st, arm, "stale_lag_epochs_max")
                                .0
                                .into_iter()
                                .flatten()
                        })
                        .fold(f64::NAN, f64::max);
                    let _ = writeln!(
                        s,
                        "| {se} | {} | {arm} | {} | {} | {} |",
                        size_label(*n),
                        pct(med_over_seeds("inexactness_rate")),
                        pct(med_over_seeds("stale_read_share")),
                        f(lag)
                    );
                }
            }
        }
        let _ = writeln!(s);
    }

    // H-E1.
    if study.view_state {
        engine_rule(
            &mut s,
            &points,
            &all_cells,
            study,
            &series,
            &sizes,
            p99_dir,
            shape.flag.is_empty(),
        );
    } else {
        let _ = writeln!(s, "## 8. What this says about H-E1\n");
        let _ = writeln!(s, "§7's engine rule needs N to beat **P+ and H3** by the joint gate on resident bytes per key ever read and on p99 read latency under eviction; T informs the write floor (commits/s against N and P+). The rows below are §2's size verdicts for those pairs.{}\n", if shape.flag.is_empty() { " Their p99 rows are the main sweep's, where a run held about 110 samples of a query; the targeted re-run the author chose, `results/E27-p99-engine.md` §8, is the one that resolves p99 (about 1,000 samples of each query per run)." } else { "" });
        let _ = writeln!(
            s,
            "| series | pair | metric | {} |\n|---|---|---|{}",
            sizes
                .iter()
                .map(|n| size_label(*n))
                .collect::<Vec<_>>()
                .join(" | "),
            "---|".repeat(sizes.len())
        );
        for se in &series {
            for (a, b, m) in [
                ("N", "P+", "pss_growth_bytes_per_key_read"),
                ("N", "H3", "pss_growth_bytes_per_key_read"),
                ("N", "P+", "q1_p99_us"),
                ("N", "H3", "q1_p99_us"),
                ("N", "P+", "q2_p99_us"),
                ("N", "H3", "q2_p99_us"),
                ("N", "P+", "c4_q1_p99_us"),
                ("N", "H3", "c4_q1_p99_us"),
                ("N", "P+", "c4_q2_p99_us"),
                ("N", "H3", "c4_q2_p99_us"),
                ("N", "T", "commits_per_s"),
                ("P+", "T", "commits_per_s"),
            ] {
                if let Some(v) = size_verdicts.get(&(se.clone(), a.into(), b.into(), m.into())) {
                    let cols: Vec<String> = sizes
                        .iter()
                        .map(|n| {
                            v.iter()
                                .find(|(x, _)| x == n)
                                .map(|(_, v)| match v {
                                    Verdict::Refused(w) if w == "mixed" => "mixed".into(),
                                    o => o.word(),
                                })
                                .unwrap_or("—".into())
                        })
                        .collect();
                    let _ = writeln!(s, "| {se} | {a} vs {b} | {m} | {} |", cols.join(" | "));
                }
            }
        }
        let _ = writeln!(s);
    }

    // Detail file.
    let _ = writeln!(d, "# {} — every cell\n\n*Generated with `results/{}.md`, from the same point files. One row per (pair, metric) per point: medians and MADs over the ten measured runs, pooled MAD, floor (3 × pooled MAD of the warm-up runs), relative change (B − A)/A, MADs apart, verdict and, when refused, why.*\n", study.name, study.main_stem);
    for ((se, n, seed), st) in &points {
        let _ = writeln!(d, "## `{se}`, {} accounts, seed {seed}\n", size_label(*n));
        let _ = writeln!(d, "| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |\n|---|---|---|---|---|---|---|---|---|---|---|");
        for (c, nr) in cells(st, study) {
            let verdict = match (&nr, &c.verdict) {
                (Some(_), _) => "NOT RUN".to_string(),
                (None, Verdict::Refused(why)) => format!("REFUSED ({why})"),
                (None, v) => v.word(),
            };
            let _ = writeln!(
                d,
                "| {} vs {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                c.a,
                c.b,
                c.metric,
                f(c.med_a),
                f(c.mad_a),
                f(c.med_b),
                f(c.mad_b),
                f(c.pooled),
                f(c.floor),
                if c.rel.is_finite() {
                    format!("{:+.1}%", c.rel * 100.0)
                } else {
                    "—".into()
                },
                f(c.apart),
                verdict
            );
        }
        let _ = writeln!(d);
    }
    std::fs::write(main, &s)?;
    std::fs::write(detail, &d)?;
    Ok(s)
}

/// What distinguishes one rendering from another: the main sweep and the targeted p99 re-run
/// share every table and differ in title, run shape and preface.
/// Counted work only: the 10⁶ point on the author's Mac (the author's decision of 2026-09-28).
///
/// The Mac has 16 GB and N alone needs about 9 GB (single-currency) and 27 GB
/// (multi-currency) at 10⁶, so that host swaps. Its latency and memory figures measure the
/// swap device; they stay in the point files, labelled here as swap-affected, and are never
/// rendered or compared. What is rendered is work the engine counts rather than times — keys
/// resident, reads, hits, misses, base rows touched, relation pages — which does not depend
/// on the host, beside the container's values for the same arms at 10³–10⁵ so the series
/// can be read across sizes.
pub fn render_counted(host_dir: &Path, container_dir: &Path, out: &Path) -> std::io::Result<()> {
    let host = load(host_dir);
    let cont = load(container_dir);
    let mut s = String::new();
    let name = |d: &Path| {
        d.file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("?")
            .to_string()
    };
    let _ = writeln!(
        s,
        "# E27-hostc — counted work at 10⁶ accounts, beside the container's 10³–10⁵\n"
    );
    let _ = writeln!(s, "*Generated by `cargo run --release -p comparator -- render --shape counted` from the point files in `results/{}/` (the author's Mac, `tools/e27-hostc.sh`) and `results/{}/` (the container). Every number below is read from those files; none is typed.*\n", name(host_dir), name(container_dir));
    let _ = writeln!(s, "**Why only counted work.** The author's decision of 2026-09-28: the 10⁶ point runs on the author's Mac, which has 16 GB of memory; scaled tenfold from the container's 10⁵ measurements, N alone needs about 9 GB single-currency and 27 GB multi-currency, so the Mac swaps. Every latency, throughput and memory figure from that host therefore measures the swap device as much as the engine: those figures stay in the point files, are **swap-affected**, and are neither rendered here nor compared with anything. What is rendered is work the engines count — keys resident, reads, hits, misses, base rows touched, relation sizes — which the host does not change. Arms N and P+ only (the author's decision of 2026-09-28); P and M refresh a full view on every write and are NOT RUN at 10⁶.\n");

    let _ = writeln!(s, "## Provenance of the 10⁶ points\n");
    if host.is_empty() {
        let _ = writeln!(s, "*Not run yet.* The 10⁶ column below is empty until `tools/e27-hostc.sh` has run on the Mac and its points are copied into `results/{}/`.\n", name(host_dir));
    } else {
        let set = |k: &str| -> String {
            let v: BTreeSet<String> = host
                .values()
                .map(|p| p.meta.get(k).cloned().unwrap_or_default())
                .collect();
            v.into_iter()
                .map(|x| {
                    if x.is_empty() {
                        "(empty)".to_string()
                    } else {
                        format!("`{x}`")
                    }
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        let _ = writeln!(s, "| field | value |\n|---|---|");
        for (label, k) in [
            ("commit(s) measured", "commit"),
            ("worktree at measurement", "worktree"),
            ("host", "host"),
            ("toolchain", "toolchain"),
            ("PostgreSQL", "postgres"),
            ("checkpoint_interval", "checkpoint_interval"),
        ] {
            let _ = writeln!(s, "| {label} | {} |", set(k));
        }
        let _ = writeln!(s, "| points | {} |\n", host.len());
        let _ = writeln!(
            s,
            "### Every arm held the same data, and answered as the oracle did\n"
        );
        let _ = writeln!(s, "| series | seed | legs loaded | arms equal at load | arms equal at end | oracle divergences | failures |\n|---|---|---|---|---|---|---|");
        for ((se, _, seed), st) in &host {
            let ul = st.meta.get("universe_sha256").cloned().unwrap_or_default();
            let el = st.meta.get("end_sha256").cloned().unwrap_or_default();
            let mut eq_load = Vec::new();
            let mut eq_end = Vec::new();
            for r in st.kind("load") {
                eq_load.push(format!("{} {}", r[1], if r[4] == ul { "=" } else { "≠" }));
                let same = !el.is_empty() && r[6] == el;
                eq_end.push(format!("{} {}", r[1], if same { "=" } else { "≠" }));
            }
            let divs = st.kind("div").count();
            let fails: Vec<String> = st.kind("failure").map(|r| r[1].clone()).collect();
            let _ = writeln!(
                s,
                "| {se} | {seed} | {} | {} | {} | {divs} | {} |",
                st.num("universe_legs"),
                eq_load.join(", "),
                eq_end.join(", "),
                if fails.is_empty() {
                    "none".into()
                } else {
                    fails.join("; ")
                }
            );
        }
        let _ = writeln!(s);
    }

    let _ = writeln!(s, "## Counted work at the end of the last run, by size\n");
    let _ = writeln!(s, "Median over seeds of each point's arm state after its last run (cumulative over the point's thirteen runs). N: `nilestream_stats`. P+: its resident slots and `pg_total_relation_size` of its relations (whole 8 KiB pages; vacuum timing can move them by a page or two). *Per read* divides by the same point's reads before the median is taken. 10³–10⁵ are the container's points (`{}`); 10⁶ is the Mac's.\n", name(container_dir));
    let series: BTreeSet<String> = cont
        .keys()
        .chain(host.keys())
        .map(|k| k.0.clone())
        .collect();
    let sizes: BTreeSet<u64> = cont.keys().chain(host.keys()).map(|k| k.1).collect();
    let sizes: Vec<u64> = sizes.into_iter().collect();
    let state = |st: &Stored, arm: &str, k: &str| -> Option<f64> {
        st.kind("state")
            .find(|r| r[1] == arm && r[2] == k)
            .and_then(|r| r[3].parse().ok())
    };
    for se in &series {
        let _ = writeln!(s, "### Series `{se}`\n");
        let head: Vec<String> = sizes
            .iter()
            .map(|n| {
                if host.keys().any(|k| k.1 == *n) && !cont.keys().any(|k| k.1 == *n) {
                    format!("{} (Mac)", size_label(*n))
                } else {
                    size_label(*n)
                }
            })
            .collect();
        let _ = writeln!(
            s,
            "| arm | figure | {} |\n|---|---|{}",
            head.join(" | "),
            "---|".repeat(sizes.len())
        );
        let rows: &[(&str, &str, Option<&str>)] = &[
            ("N", "resident", None),
            ("N", "reads", None),
            ("N", "hits", None),
            ("N", "misses", None),
            ("N", "misses", Some("reads")),
            ("N", "rows_touched", None),
            ("N", "rows_touched", Some("reads")),
            ("N", "view_answers", None),
            ("N", "fallbacks", None),
            ("P+", "resident", None),
            ("P+", "bytes arm.rev", None),
            ("P+", "bytes arm.checkpoints", None),
            ("P+", "bytes arm.key_counts", None),
            ("P+", "bytes arm.postings", None),
        ];
        for (arm, k, per) in rows {
            let mut cells = Vec::new();
            for n in &sizes {
                let pts = if host.keys().any(|x| x.1 == *n) && !cont.keys().any(|x| x.1 == *n) {
                    &host
                } else {
                    &cont
                };
                let v: Vec<f64> = pts
                    .iter()
                    .filter(|(key, _)| &key.0 == se && key.1 == *n)
                    .filter_map(|(_, st)| {
                        let x = state(st, arm, k)?;
                        match per {
                            Some(d) => state(st, arm, d).filter(|y| *y > 0.0).map(|y| x / y),
                            None => Some(x),
                        }
                    })
                    .collect();
                cells.push(if v.is_empty() {
                    "—".to_string()
                } else {
                    let m = bank_bench::score::median(&v);
                    // A count is printed as one; a ratio (or a median of an even number of
                    // seeds) as `f` prints it.
                    if per.is_none() && m.fract() == 0.0 {
                        format!("{m:.0}")
                    } else {
                        f(m)
                    }
                });
            }
            let label = match per {
                Some(d) => format!("`{k}` per {}", d.trim_end_matches('s')),
                None => format!("`{k}`"),
            };
            let _ = writeln!(s, "| {arm} | {label} | {} |", cells.join(" | "));
        }
        let _ = writeln!(s);
    }
    let _ = writeln!(s, "**Reading `rows_touched` per read.** The counter includes every query, and q5 (top ten) and q6 (every balance) read the whole ledger by definition: 7% of the single-client reads (q5 5%, q6 2%), each touching every leg. Per read it therefore grows with the ledger by construction, and the point reads' share is not separable in this counter.\n");
    let _ = writeln!(s, "P+ has no hit or miss counter: its `rev_read` is the E14 mechanism's SQL, and adding a counter would change the arm that the container measured. Its work is read from what it holds.");
    std::fs::write(out, s)
}

pub struct Shape {
    pub title: &'static str,
    /// The flags that reproduce this rendering after `render`.
    pub flag: &'static str,
    pub run_shape: &'static str,
    pub preface: &'static str,
    /// Which experiment's metrics, pairs, deviations and rule this rendering reports.
    pub study: &'static Study,
}

/// **What distinguishes E27 from E27b** in a rendering: the metrics judged, the pairs, the
/// deviations, the provenance lines that name arms, and whether the view-state metric and
/// the engine rule's evaluation are reported (cycle 15, C15-02). E27's values are the
/// constants its rendering always used, so its files do not move.
pub struct Study {
    /// The experiment's name and its main file's stem, for the detail file's title.
    pub name: &'static str,
    pub main_stem: &'static str,
    /// The flag that selects it on the command line, as the header quotes it.
    pub cli: &'static str,
    /// The card and specification, in the header's italic line.
    pub card: &'static str,
    pub metrics: &'static [(&'static str, &'static str)],
    pub seed_replicated: &'static [&'static str],
    pub pairs: &'static [(&'static str, &'static str)],
    pub deviations: &'static [&'static str],
    pub seed_gate_note: &'static str,
    pub checkpoint_line: &'static str,
    pub budget_line: &'static str,
    pub memory_intro: &'static str,
    /// E27b: the view-state table (§5) and the engine rule, evaluated (§8).
    pub view_state: bool,
}

pub const E27: Study = Study {
    name: "E27",
    main_stem: "E27-comparator",
    cli: "",
    card: "Cycle 14, cards R2-02 and R2-03; specification: §5 of the round-2 work order.",
    metrics: METRICS,
    seed_replicated: SEED_REPLICATED,
    pairs: PAIRS,
    deviations: DEVIATIONS,
    seed_gate_note: "One exception to the per-seed gate, stated before the measured sweep: `pss_growth_bytes_per_key_read` is judged **across seeds** on each seed's last measured run (joint gate over the five values; no warm-up floor, since a seed has one such value), because its per-run values trend with the keys read so far and are not replicates.",
    checkpoint_line: "C = 16 on N (`nilestreamd --checkpoint 16`) and on P+ (`sql/pplus.sql`); H3's and T's upqueries have no checkpoints (`epoch <= e` on the ledger's index; TigerBeetle's account history)",
    budget_line: "5% of keys (rounded), LRU, on N, P+, H3 and T; on H1 the same share in bytes, calibrated on H1 alone (`results/E27-comparator/h1-calibration.tsv`); P, M and H2 are full",
    memory_intro: "PSS after load and after the last run (N: the daemon process; PostgreSQL: postmaster and every child). Arm state: N's `nilestream_stats` (resident keys, reads, hits, misses, base rows touched, view answers); PostgreSQL relation sizes (`pg_total_relation_size`) and P+'s resident slots. Median over seeds.",
    view_state: false,
};

/// E27's metrics, then E27b's memory metric (design §4.1).
pub const METRICS_B: &[(&str, &str)] = &[
    ("q1_p50_us", "q1 point at head, p50 µs, 1 client"),
    ("q1_p99_us", "q1 point at head, p99 µs, 1 client"),
    ("q2_p50_us", "q2 point at anchor, p50 µs, 1 client"),
    ("q2_p99_us", "q2 point at anchor, p99 µs, 1 client"),
    ("q3_p50_us", "q3 statement, p50 µs"),
    ("q4_p50_us", "q4 desk exposure, p50 µs"),
    ("q5_p50_us", "q5 top-10, p50 µs"),
    ("q6_p50_us", "q6 extract, p50 µs"),
    ("c2_q1_p50_us", "q1, p50 µs, 2 clients"),
    ("c2_q1_p99_us", "q1, p99 µs, 2 clients"),
    ("c2_q2_p50_us", "q2, p50 µs, 2 clients"),
    ("c2_q2_p99_us", "q2, p99 µs, 2 clients"),
    ("c4_q1_p50_us", "q1, p50 µs, 4 clients"),
    ("c4_q1_p99_us", "q1, p99 µs, 4 clients"),
    ("c4_q2_p50_us", "q2, p50 µs, 4 clients"),
    ("c4_q2_p99_us", "q2, p99 µs, 4 clients"),
    ("write_p50_us", "two-leg transfer, p50 µs (commit at fsync)"),
    ("write_p99_us", "two-leg transfer, p99 µs"),
    ("commits_per_s", "commits/s, 1 client, at fsync"),
    (
        "view_state_bytes_per_key_read",
        "**view-state bytes ÷ distinct keys read so far** (the engine rule's memory metric, §4.1)",
    ),
    (
        "pss_mib",
        "process memory (PSS), MiB, after the run — secondary; includes N's base",
    ),
    (
        "pss_growth_bytes_per_key_read",
        "PSS growth since load ÷ distinct keys read so far, bytes — secondary",
    ),
];

/// E27b's arms (design §3): N, P+, P, M and H3; H1, H2 and T are not re-run.
pub const PAIRS_B: &[(&str, &str)] = &[
    ("N", "P+"),
    ("N", "P"),
    ("P+", "P"),
    ("P", "M"),
    ("N", "H3"),
    ("P+", "H3"),
];

pub const E27B: Study = Study {
    name: "E27b",
    main_stem: "E27b-comparator",
    cli: " --study e27b",
    card: "Cycle 15, round 2, card C15-02 (E27b part 1); pre-registered in `docs/study/E27b-design.md` (commit `77997dd`) before any engine change.",
    metrics: METRICS_B,
    seed_replicated: &[
        "view_state_bytes_per_key_read",
        "pss_growth_bytes_per_key_read",
    ],
    pairs: PAIRS_B,
    deviations: DEVIATIONS_B,
    seed_gate_note: "Two exceptions to the per-seed gate, pre-registered (design §4.1): `view_state_bytes_per_key_read` — the engine rule's memory metric — and `pss_growth_bytes_per_key_read` (secondary) are judged **across seeds** on each seed's last measured run (joint gate over the five values; no warm-up floor, since a seed has one such value), because their per-run values trend with the keys read so far and are not replicates.",
    checkpoint_line: "C = 16 on N (`nilestreamd-metered --checkpoint 16`) and on P+ (`sql/pplus.sql`); H3's upquery has no checkpoints (`epoch <= e` on the ledger's index)",
    budget_line: "5% of keys (rounded), LRU, on N, P+ and H3; P and M are full",
    memory_intro: "**Process memory, secondary** (E27's metric; it does not enter the engine rule): PSS after load and after the last run — for N the daemon, which *includes its in-memory base*; for H3 the sidecar plus its PostgreSQL cluster; for the PostgreSQL arms the cluster (postmaster and every child). Arm state: N's `nilestream_stats` (resident keys, reads, hits, misses, base rows touched, view answers, view-state and checkpoint bytes), H3's `rev_stats`, PostgreSQL relation sizes (`pg_total_relation_size`) and resident counts. Median over seeds.",
    view_state: true,
};

/// E27's main run shape, shared by E27b's (design §5: unchanged).
const MAIN_RUN_SHAPE: &str = "3 warm-up runs then 10 measured runs per (arm, size, seed), arms visited round-robin in an order rotated each run; a run = 200 operations at 1 client (every tenth a two-leg transfer: 90% reads / 10% writes; read mix q1 60%, q2 25%, q3 5%, q4 3%, q5 5%, q6 2%), then q1/q2 reads at 2 and at 4 concurrent clients (60 per client, 70% q1)";

pub const MAIN_B: Shape = Shape {
    title: "E27b — a fair Nilestream, a fair metric: N against P+, P, M and H3, under eviction, across bank sizes",
    flag: "",
    run_shape: MAIN_RUN_SHAPE,
    preface: "**Why this file exists.** E27 (`results/E27-comparator.md`) found N justified at no size, and the cycle-15 audit (DA-3) found that it had not measured the engine as designed: a multi-currency read never used the view, the memory clause set N's process PSS (which includes its in-memory base) against a PostgreSQL cluster whose base is on disk, and q3 and q4 could not run on N. E27b measures again after exactly four engine changes, each with a guard (design §2): **E1** a read that names its currency, or is grouped by `(acct, cur)`, is answered from the `(acct, cur)`-keyed view; **E2** view-state bytes on the wire; **E3** value dates on the served relation; **E4** q3 and q4 asked of N. The engine rule's memory metric is now **view-state bytes per key read** (§4.1), counted per arm as the design fixes; process PSS is kept, secondary. E27 is not overwritten.",
    study: &E27B,
};

pub const P99_ENGINE_B: Shape = Shape {
    title: "E27b-p99-engine — read latency tails under eviction, N against P+ and H3",
    flag: " --shape p99-engine",
    run_shape: "as E27-p99-engine (design §6): 3 warm-up runs then 10 measured runs per (arm, size, seed), interleaved; a run = 2,000 point reads at 1 client (q1 and q2 half each, no writes), then 4 clients × 500 reads — on the multi-currency series at 10⁵, q1 only (the author's decision of 2026-09-28, kept by the design)",
    preface: "**Why this file exists.** The engine rule's p99 clause is judged on q1 and q2 at 1 and 4 clients in this shape (design §4.2, §6), as E27 judged it in `results/E27-p99-engine.md`. N, P+ and H3, the rule's three arms; N as changed by E27b's E1–E4 and run as the metered build. The rule is evaluated in `results/E27b-comparator.md` §8, from this file's verdicts and the main sweep's memory verdicts.",
    study: &E27B,
};

pub const MAIN: Shape = Shape {
    title: "E27 — Nilestream against its alternatives, under eviction, across bank sizes",
    flag: "",
    run_shape: MAIN_RUN_SHAPE,
    preface: "",
    study: &E27,
};

pub const P99: Shape = Shape {
    title: "E27-p99 — read latency tails under eviction, N against P+",
    flag: " --shape p99",
    run_shape: "3 warm-up runs then 10 measured runs per (arm, size, seed), interleaved as in E27; a run = 2,000 point reads at 1 client (q1 and q2 half each, no writes), then 4 clients × 500 reads (q1 and q2 half each) — about 1,000 samples of each query per run and level",
    study: &E27,
    preface: "**Why this file exists.** In E27's main sweep a run held about 110 samples of q1, so a per-run p99 was its second-largest value and most p99 comparisons were refused by the pre-registered rule (warm-up MAD above 15% of the median). p99 read latency under eviction is half of §7's engine rule, so the author chose on 2026-09-28 a targeted re-run — N against P+ only, point reads only, 1,000 samples per query per run — with the run length fixed before any of its numbers existed (`Config::p99`). Same universe, seeds, sizes, gate, floor and refusal rule as E27. Only the p99 and p50 rows of q1 and q2 are meaningful here; the other metric rows are absent by design.",
};

pub const P99_ENGINE: Shape = Shape {
    title: "E27-p99-engine — read latency tails under eviction, N against P+, H3 and T",
    flag: " --shape p99-engine",
    run_shape: "as E27-p99: 3 warm-up runs then 10 measured runs per (arm, size, seed), interleaved; a run = 2,000 point reads at 1 client (q1 and q2 half each, no writes), then 4 clients × 500 reads — on the multi-currency series at 10⁵, q1 only (the author's decision of 2026-09-28)",
    study: &E27,
    preface: "**Why this file exists.** §7's engine rule compares N with P+ **and H3** on p99 read latency under eviction, and T's reads are measured beside them. The author chose on 2026-09-28 (R2-03) to run the targeted p99 shape of `results/E27-p99.md` again with the four engine-rule arms interleaved together. `results/E27-p99.md` (N against P+ only) keeps its R2-02 points unchanged; only its rendering follows the current renderer. Same universe, seeds, sizes, gate, floor and refusal rule as E27. T's answers are checked at its bracketed frontier (E27 §7a); an inexact one refuses its comparisons.",
};

/// The deviations E27 states in its header. Each is a fact about this build, recorded in the
/// round-2 log when it was found.
pub const DEVIATIONS: &[&str] = &[
    "**What `MODIFIED` means here, and which build measured each point.** The sweep ran from a binary copied out of the tree (`/var/tmp/e27-bin/comparator`) while work continued in the tree, so a point records HEAD and the worktree as they were when the point was *written*, not the build that *measured* it. The measuring builds were: `c3b04ff` for the first 21 main points; `4bdba3d` (copied when the sweep resumed after a container restart) for the remaining main points, the probe and the first 16 p99 points; `5ad7f29` for the last 14 p99 points. The uncommitted edits present while they ran were, in order: the p99 shape and the Host C portability of `pgcluster.rs` and `provenance()` (committed in `4bdba3d`), then `render --stem` and `--q1-only` (committed in `5ad7f29`). None changes the main shape's behaviour: `Config::declared` yields the constants the earlier code used (a write every tenth operation, the same mix, 70% q1 in the concurrent phases), and under root the cluster paths are unchanged.",
    "**A first sweep attempt was discarded, and two things were changed after seeing it.** It ran at commit `fed5238` and wrote the five `single` 10³ points kept in `results/E27-comparator/discarded-attempt-1/`: it counted a read chosen as a deadlock victim (SQLSTATE 40P01, which P+'s `rev_read` produces under concurrent readers) as an oracle divergence, which refused every P+ comparison, and it gated `pss_growth_bytes_per_key_read` per run although that value trends with the keys read so far, which refused it everywhere. Before the sweep reported here: deadlocked reads are retried, with the failed attempts inside the measured latency and their count reported (§6a); the memory-per-key metric is judged across seeds (§2). Nothing else changed — not the gate, the floor, the refusal rule, the run shape or any arm.",
    "**P+ is the E14 mechanism with two repairs** (`crates/comparator/sql/pplus.sql`; the E14 files are untouched): E14's `rev_read` served a resident slot for any anchor at or below its version, so a historical read after later postings returned the current value — a wrong answer that would have refused every q2 comparison; and its eviction ordered victims by last *change*, not last use. Here a slot answers anchor e only if its last change is at or before e and the view has applied e, and eviction is LRU on a read tick, as on N.",
    "**Nilestream did not honour `order by` in a reply** until commit `e82fac3` of this card: top-10 returned the right ten rows in account order. Found by this comparator's correctness pass on its first smoke run, fixed with a test and a sabotage control before any measured run.",
    "**Nilestream's view path serves one currency only** (`answer_from_view` returns NotApplicable when the base holds two), so on the `multi` series every keyed read on N folds the base and nothing is resident. The `multi` series measures that engine as it is.",
    "**q3 and q4 are NOT RUN on N**: its served relation has no value day and no desk. PostgreSQL arms answer them from the base (P+) or the view (P, M).",
    "**P+ answers q4, q5 and q6 by folding the base**: the E14 mechanism is keyed point reads and has no view for them; P and M answer them from the materialised view.",
    "**Memory is process PSS, not bytes per resident key**: N's wire exposes a resident count and no byte figure, and PostgreSQL's cost of a key is its share of shared buffers and relation pages. The metric `pss_growth_bytes_per_key_read` divides the PSS growth since load by the distinct keys read so far; relation sizes and resident counts are reported beside it (§5).",
    "**Allocations are not measured on any arm**: N's served daemon has no allocation counter on its wire; the in-process instrument (E18) is not the served binary. §5.5 asked for allocations per read hit, reconstruction and write on N; they remain for a later card.",
    "**Latency is client-observed over the PostgreSQL wire, simple protocol, literal statement texts**, on every arm. PostgreSQL plans each statement; Nilestream compiles each (its plan cache is keyed by text, so literal keys miss it). Neither arm is given prepared statements.",
    "**P's write is two round trips** (`post_txn`, then `REFRESH MATERIALIZED VIEW CONCURRENTLY`), because the refresh cannot run inside a function; its commit latency includes both, which is P's contract (a view current after every write).",
    "**Anchors are drawn uniformly** between a key's first epoch and the head; reads draw only keys that exist at their anchor, because P+ reconstructs an absent key as 0 where N and the oracle answer no row — a semantic difference this measurement does not exercise and E27 does not score.",
    "**The 10⁶ point is not in this file**: the container holds 2 cores and 8 GB; it goes to Host C with a prepared `run.sh` (§11.3).",
    "**Concurrency phases are read-only** (writes run only in the 1-client mixed phase), so the concurrent figures are reads over a quiescent head; concurrent reads *with* writes are the anomaly probe's (§7).",
    "**R2-03 re-ran the whole sweep with eight arms.** R2-02's four-arm points (the E27 published at `de76a5a`) are kept unchanged in `results/E27-comparator/r2-02-four-arms/`; this file is rendered from the eight-arm points only.",
    "**The probe's mutating mode was misclassifying reads, and was fixed in R2-03 before its re-run.** Until then every read was held to the *final* base, so a read taken before a mutation landed counted as a skipped delta. Each read is now held to the oracle plus the mutations that had certainly landed when it began, and any that may have landed during it are allowed (`probe.rs`, `truth_range`); the upquery-race test compares only reads with no mutation of the key in flight. A one-repetition check on M+ before the re-run still found skipped delta on 1,205 of 1,214 reads (commit `81485d3`), so R2-02's M+ finding did not rest on the defect.",
    "**H1 (ReadySet) caches q1, q4, q5 and q6; q2 and q3 are proxied to its PostgreSQL**: `CREATE DEEP CACHE` refuses their range predicates (`epoch <= $3`, `value_day between …`) with \"Query contains placeholders in unsupported positions\" (measured on this build). H1's q2 and q3 latencies are PostgreSQL's plus a proxy hop, and its writes reach PostgreSQL the same way.",
    "**H1's residency budget is in bytes** (the author's decision of 2026-09-28): per series and size, on seed 1's universe and on H1 alone, `readyset_allocator_allocated_bytes` after the load (a0) and after reading every key once (a1) set `--memory-limit = a0 + (a1 − a0) × budget / keys` — the same share of keys as the other partial arms. It is ReadySet's allocator figure, not PSS, because that is what its limit is enforced against.",
    "**H3's write is acknowledged only when its epoch has arrived through logical replication and been applied to the REV**, so H3's reads are exact at the head, like N's and P+'s, and its write latency includes the replication hop. Its upquery is `epoch <= e` on the ledger's index, with no checkpoints, as §R2-03 specifies; its q3–q6 are relayed to its PostgreSQL.",
    "**T's write is acknowledged at TigerBeetle's commit** (one replica, Direct I/O), not when the view has it: T is the write floor (§7). Its reads come from the REV fed by the change stream and are checked at the sidecar's bracketed frontier (§7a); a read as of an epoch TigerBeetle has committed but the stream has not yet delivered waits for it (up to 30 s) — before that repair, found in the sweep's first minutes and before any point was written, such a read was refused and counted as a divergence. Its upquery is TigerBeetle's account balance at the epoch's timestamp — an index lookup reporting one row — so its base rows per reconstruction are not comparable with a fold's. `--cache-grid=256MiB` rather than the 1 GiB default, set before any T measurement so the eight arms fit the 8 GB host; TigerBeetle allocates about 2.3 GiB at start regardless, which is in T's memory.",
    "**H1 is absent at 10⁵ as well** (the author's decision of 2026-09-29): in the first 10⁵ point the container's memory cgroup killed ReadySet at run 7 of 13 (RSS 1.9 GB against a calibrated byte limit of about 350 MB; kernel log `task=readyset`), which refused every comparison at that point. That point is kept, unchanged, in `results/E27-comparator/failed/`; the six other arms were re-run for it. Earlier in the same aborted attempt (seed 7, a warm-up run) H1 returned one q6 extract that matched the oracle at no epoch in its bracket; no kept point contains it, and the note below on H1's replication offset bears on how to read it.",
    "**On the multi-currency series at 10⁵ only N, P+, H2 and H3 run** (the author's decision of 2026-09-29): with six arms loaded, the memory cgroup killed N at run 3 of the first point (N grows to ~2.8 GB there, as R2-02 measured). P's and M's multi-currency 10⁵ figures are R2-02's four-arm points in `results/E27-comparator/r2-02-four-arms/` — the same host, universe and seeds, a different run and build — cited there and never pooled with these.",
    "**ReadySet's replication offset runs ahead of what its readers see.** The probe's one H1 skipped delta (append + mutate mode, repetition 1, where every mutation was refused by the append-only trigger) is key (193, 0) read as −4,964,547 inside a bracket of exactly #296; recomputing the probe's oracle gives −4,991,801 at #295 and #296 and −4,964,547 at #294. The read was two epochs *staler* than the offset ReadySet reported before it, not missing a delta. The bracket (`h1.rs`) therefore bounds ReadySet's base tables, not its reader views, and an H1 answer classified as inexact may instead be stale beyond it; the main sweep recorded no inexact H1 answer, so no H1 cell depends on the difference.",
    "**The first 20 points (10³ and 10⁴) record an empty `toolchain`**: the provenance probe ran `cargo --version` inside the worktree, which resolves the pinned toolchain that does not install here. Every binary was built with `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` under `RUSTUP_TOOLCHAIN=stable`; the 10⁵ points and the p99-engine points record it, and `store::provenance` now asks for it explicitly.",
    "**T is absent at 10⁵** (the author's decision of 2026-09-28): the eight arms do not fit the 8 GB host together at 10⁵ — in a load-only check the first seven reached about 5.5 GB and the container's memory-cgroup limit killed TigerBeetle while it allocated its ~2.3 GiB (kernel log: `oom-kill: constraint=CONSTRAINT_MEMCG … task=tigerbeetle`). The other seven arms run interleaved at 10⁵; T's write floor is judged at 10³ and 10⁴; T's 10⁵ read tails are in `results/E27-p99-engine.md`, where N, P+, H3 and T fit together. Cells of T's pairs at 10⁵ show —.",
    "**Versions as installed** (`claude/cycle-14-r2-03-downloads.md`): the PostgreSQL server stayed at 16.13 (held); `postgresql-server-dev-16` and `libpq5` are 16.15, so pg_ivm 1.15 is built against 16.15 headers and loads into the 16.13 server (the module magic block checks the major version); RabbitMQ 3.12.1 from Ubuntu 24.04 left upstream community support on 29 Feb 2024 and is used only as T's change-stream transport.",
];

/// E27b's deviations, each fixed before any E27b run (design §1: "a change … after this commit
/// is a deviation, listed … with its reason"). The inherited ones are E27's, restated where
/// they still apply to E27b's arms.
pub const DEVIATIONS_B: &[&str] = &[
    "**q4 on N reads the desk from the account; no `desk` column was added** (design §2, E4, said N's served relation would gain one). The universe defines a desk as `acct % 16` (`universe::desk_of`), which is P's and M's spelling too, so a stored column would hold a function of a column N already has. N's q4 is `select cur, sum(amt) from postings where acct % 16 = {d} group by cur`, and its answers are held to the oracle like every other arm's. Decided in commit `44bb0cd`, before any run.",
    "**q3 on N writes its value-day window with `>=` and `<=`**: `between` does not lower on N's SQL surface. Design §5 provides for this (\"the smallest spelling that does is used and listed as a deviation\").",
    "**A four-value insert writes value date 0, not NULL** (design §2 said NULL): the ledger's valid-time field is an integer day, not an option. N's harness sends five values on every insert, so no measured read depends on it.",
    "**The E2 guard's third clause is false of the engine, so the guard records the behaviour instead of asserting the clause.** The clause is \"stays bounded by the budget under eviction\". Every evicted key leaves a `Hole` carrying its version (`crates/nilestream-core/src/absence.rs`). N's and H3's view-state bytes therefore grow by one hole per distinct key ever read, while their resident values and policy metadata stay at the budget (`tools/memprobe/tests/view_state_bytes.rs` asserts both halves). Whether a demand view keeps holes is LC-31, open since cycle 9. E27b measures the engine as it is. Found at the guard, before any E27b run.",
    "**N's view state counts three per-key maps besides the checkpoints** (`Ledger::checkpoint_state_image`). Two of them, the running balance and the posting count, place the checkpoints; P+ keeps the same as `arm.key_counts`, and its figure includes it. The third, `key_first`, is the first epoch per key, which E1's currency-named reads consult. It is closer to an index over the base, and it is counted anyway because counting it can only enlarge N's figure. P+'s figure adds `arm.rev_meta`, which E27's state table did not list.",
    "**The instrument, as built.** N runs as `nilestreamd-metered` and H3's sidecar as `rev-sidecar-metered`. Each is the shipped `main.rs` compiled by `tools/memprobe` over `alloc::Metered`, because §9.10 keeps `unsafe` out of the workspace. The allocator does one thread-local flag test per allocation. It counts only on the thread that asked for a figure, so neither contention nor another thread's allocations enter a figure. Its cost on latency is not measured separately and is borne by N and H3 alike. A figure is a clone's live bytes: exact for the B-trees and hash tables, and a lower bound by spare vector capacity. The PostgreSQL arms' `pg_total_relation_size` includes page slack and dead tuples. The asymmetry is stated beside every figure (§5).",
    "**H1, H2 and T are not re-run** (design §3): H1 and H2 do not enter the engine rule, and T informs the write floor only. E27's T figures stand, cited from `results/E27-comparator.md` and never pooled with these.",
    "**P+ is the E14 mechanism with two repairs** (`crates/comparator/sql/pplus.sql`; the E14 files are untouched). E14's `rev_read` served a resident slot for any anchor at or below its version, so a historical read after later postings returned the current value. That is a wrong answer, and it would have refused every q2 comparison. Its eviction also ordered victims by last *change*, not last use. Here a slot answers anchor e only if its last change is at or before e and the view has applied e, and eviction is LRU on a read tick, as on N. Unchanged from E27.",
    "**P+ answers q4, q5 and q6 by folding the base**: the E14 mechanism is keyed point reads and has no view for them; P and M answer them from the materialised view. N answers q3–q6 by folding its base (its view is keyed point balances).",
    "**H3's write is acknowledged only when its epoch has arrived through logical replication and been applied to the REV**, so H3's reads are exact at the head, like N's and P+'s, and its write latency includes the replication hop. Its upquery is `epoch <= e` on the ledger's index, with no checkpoints; its q3–q6 are relayed to its PostgreSQL. Unchanged from E27.",
    "**Latency is client-observed over the PostgreSQL wire, simple protocol, literal statement texts**, on every arm. PostgreSQL plans each statement; Nilestream compiles each (its plan cache is keyed by text, so literal keys miss it). Neither arm is given prepared statements.",
    "**P's write is two round trips** (`post_txn`, then `REFRESH MATERIALIZED VIEW CONCURRENTLY`), because the refresh cannot run inside a function; its commit latency includes both, which is P's contract.",
    "**Anchors are drawn uniformly** between a key's first epoch and the head; reads draw only keys that exist at their anchor, as in E27.",
    "**Concurrency phases are read-only** (writes run only in the 1-client mixed phase), as in E27.",
    "**Allocations per read are not measured on the served binary** (design §11): the metered build counts bytes held on request, not allocations per operation; E18 measures those in-process.",
    "**The 10⁶ point is not in this file**: it is C15-04's (Host C). The container holds 2 cores and 8 GB.",
];

/// §5's first table (E27b): the derived state each arm holds, per key read and per resident key.
fn view_state_table(
    s: &mut String,
    points: &Points,
    series: &BTreeSet<String>,
    sizes: &BTreeSet<u64>,
) {
    let _ = writeln!(
        s,
        "### View-state bytes — the engine rule's memory metric (design §4.1)\n"
    );
    let _ = writeln!(s, "At the end of each seed's last measured run; median over seeds. *Keys read* is the distinct `(acct, cur)` keys read so far (view-state bytes ÷ bytes per key read). *Resident* is the arm's own count at the end (`resident` on N and H3; `arm.rev_meta.present` on P+; every row of `arm.balances` on P and M).\n");
    let _ = writeln!(s, "| series | size | arm | view-state bytes | keys read | bytes per key read | resident keys | bytes per resident key | what is counted |\n|---|---|---|---|---|---|---|---|---|");
    let what = |arm: &str| {
        match arm {
        "N" => "the balance view's slots, metadata, pinned set and flights, plus the ledger's per-key checkpoint state; a clone's live bytes under the metering allocator (lower bound by spare vector capacity)",
        "H3" => "the sidecar REV's slots, metadata, pinned set and flights (no checkpoints); a clone's live bytes under the metering allocator (lower bound by spare vector capacity)",
        "P+" => "`pg_total_relation_size` of `arm.rev`, `arm.rev_meta`, `arm.key_counts`, `arm.checkpoints` (includes page slack and dead tuples)",
        "P" | "M" => "`pg_total_relation_size` of `arm.balances` (includes page slack and dead tuples)",
        _ => "—",
    }
    };
    for se in series {
        for n in sizes {
            let at: Vec<&Stored> = points
                .iter()
                .filter(|(k, _)| &k.0 == se && k.1 == *n)
                .map(|(_, st)| st)
                .collect();
            let arms: BTreeSet<String> = at.iter().flat_map(|st| arms_of(st)).collect();
            for arm in &arms {
                let last =
                    |st: &Stored, m: &str| series_of(st, arm, m).0.into_iter().flatten().last();
                let (mut vsb, mut keys, mut per_key, mut res, mut per_res) =
                    (vec![], vec![], vec![], vec![], vec![]);
                for st in &at {
                    let (Some(b), Some(k)) = (
                        last(st, "view_state_bytes"),
                        last(st, "view_state_bytes_per_key_read"),
                    ) else {
                        continue;
                    };
                    vsb.push(b);
                    per_key.push(k);
                    if k > 0.0 {
                        keys.push(b / k);
                    }
                    if let Some(r) = st
                        .kind("state")
                        .find(|r| &r[1] == arm && r[2] == "resident")
                        .and_then(|r| r[3].parse::<f64>().ok())
                    {
                        res.push(r);
                        if r > 0.0 {
                            per_res.push(b / r);
                        }
                    }
                }
                if vsb.is_empty() {
                    continue;
                }
                let med = |v: &[f64]| {
                    if v.is_empty() {
                        f64::NAN
                    } else {
                        bank_bench::score::median(v)
                    }
                };
                let _ = writeln!(
                    s,
                    "| {se} | {} | {arm} | {} | {} | {} | {} | {} | {} |",
                    size_label(*n),
                    f(med(&vsb)),
                    f(med(&keys)),
                    f(med(&per_key)),
                    f(med(&res)),
                    f(med(&per_res)),
                    what(arm)
                );
            }
        }
    }
    let _ = writeln!(s);
}

/// One pair's clauses: (other arm, memory verdict, p99 verdicts judged, p99 metrics NOT RUN).
type RuleParts<'a> = (
    &'a str,
    Option<Verdict>,
    Vec<(&'a str, Verdict)>,
    Vec<&'a str>,
);

/// The rule's p99 metrics (design §4.2): q1 and q2 at 1 and 4 clients.
const RULE_P99: &[&str] = &["q1_p99_us", "q2_p99_us", "c4_q1_p99_us", "c4_q2_p99_us"];
const RULE_MEMORY: &str = "view_state_bytes_per_key_read";

fn word_of(v: &Verdict) -> String {
    match v {
        Verdict::Refused(w) if w == "mixed" => "mixed".into(),
        o => o.word(),
    }
}

/// §8 of an E27b rendering: the engine rule (design §8), evaluated per series and size from
/// the size verdicts, never from a median. On the main file the memory clause is this sweep's
/// and the p99 clause is the targeted re-run's (`p99_dir`); on the re-run's own file only its
/// p99 verdicts are listed.
#[allow(clippy::too_many_arguments)]
fn engine_rule(
    s: &mut String,
    points: &Points,
    all_cells: &CellsBySize,
    study: &Study,
    series: &BTreeSet<String>,
    sizes: &BTreeSet<u64>,
    p99_dir: Option<&Path>,
    main: bool,
) {
    let p99_points = match (main, p99_dir) {
        (true, Some(d)) => load(d),
        _ => BTreeMap::new(),
    };
    let p99_cells = cells_by_size(&p99_points, study);
    let (pp, pc): (&Points, &CellsBySize) = if main {
        (&p99_points, &p99_cells)
    } else {
        (points, all_cells)
    };
    let _ = writeln!(s, "## 8. The engine rule, evaluated (H-E1)\n");
    let _ = writeln!(s, "The rule, unchanged (design §8): **N is justified as a product** iff it beats P+ **and** H3 by the joint gate, above the floor, on view-state bytes per key read **and** on p99 read latency under eviction. **The hybrid** (PostgreSQL ledger + REV sidecar) is recommended iff H3 matches N within the gate on both; **PostgreSQL with the E14 mechanism** iff P+ matches N on both. *Matches* is read as the size verdict *no difference*. The p99 clause is q1 and q2 at 1 and 4 clients from the targeted re-run (`results/E27b-p99-engine.md`); a query NOT RUN there (q2 on `multi` at 10⁵, §6) is left out of the clause and said so. A recommendation that holds at one end of the sweep and not the other is two recommendations (S-1).\n");
    let _ = writeln!(
        s,
        "| series | pair | metric | source | {} |\n|---|---|---|---|{}",
        sizes
            .iter()
            .map(|n| size_label(*n))
            .collect::<Vec<_>>()
            .join(" | "),
        "---|".repeat(sizes.len())
    );
    for se in series {
        for b in ["P+", "H3"] {
            let mut rows: Vec<(&str, &str, &Points, &CellsBySize)> = Vec::new();
            if main {
                rows.push((RULE_MEMORY, "this sweep", points, all_cells));
            }
            for m in RULE_P99 {
                rows.push((m, "p99 re-run", pp, pc));
            }
            for (m, src, ps, cs) in rows {
                let cols: Vec<String> = sizes
                    .iter()
                    .map(|n| match size_cell(ps, cs, study, se, *n, "N", b, m) {
                        SizeCell::Missing => "—".into(),
                        SizeCell::NotRun => "NOT RUN".into(),
                        c => word_of(c.verdict().expect("judged")),
                    })
                    .collect();
                let _ = writeln!(
                    s,
                    "| {se} | N vs {b} | {m} | {src} | {} |",
                    cols.join(" | ")
                );
            }
        }
    }
    let _ = writeln!(s);
    if !main {
        let _ = writeln!(s, "The rule is evaluated in `results/E27b-comparator.md` §8, from these verdicts and the main sweep's memory verdicts.\n");
        return;
    }
    if p99_points.is_empty() {
        let _ = writeln!(
            s,
            "**Not evaluated yet: the targeted p99 re-run has no points.**\n"
        );
        return;
    }
    let _ = writeln!(s, "**Evaluation**, per series and size:\n");
    for se in series {
        let mut outcomes: Vec<(u64, String)> = Vec::new();
        for n in sizes {
            let mem = |b: &str| {
                size_cell(points, all_cells, study, se, *n, "N", b, RULE_MEMORY)
                    .verdict()
                    .cloned()
            };
            let p99 = |b: &str| -> (Vec<(&str, Verdict)>, Vec<&str>) {
                let mut judged = Vec::new();
                let mut not_run = Vec::new();
                for m in RULE_P99 {
                    match size_cell(pp, pc, study, se, *n, "N", b, m) {
                        SizeCell::NotRun => not_run.push(*m),
                        c => {
                            if let Some(v) = c.verdict() {
                                judged.push((*m, v.clone()));
                            }
                        }
                    }
                }
                (judged, not_run)
            };
            let parts: Vec<RuleParts> = ["P+", "H3"]
                .into_iter()
                .map(|b| {
                    let (j, nr) = p99(b);
                    (b, mem(b), j, nr)
                })
                .collect();
            let missing: Vec<String> = parts
                .iter()
                .filter(|(_, m, j, _)| m.is_none() || j.is_empty())
                .map(|(b, m, _, _)| {
                    format!(
                        "N vs {b}: {}",
                        if m.is_none() {
                            "no memory verdict"
                        } else {
                            "no p99 verdict"
                        }
                    )
                })
                .collect();
            let line = if !missing.is_empty() {
                format!("not evaluable ({})", missing.join("; "))
            } else {
                let all = |b: &str, pred: &dyn Fn(&Verdict) -> bool| {
                    parts.iter().filter(|p| p.0 == b).all(|(_, m, j, _)| {
                        m.as_ref().map(pred).unwrap_or(false) && j.iter().all(|(_, v)| pred(v))
                    })
                };
                let n_wins = |v: &Verdict| *v == Verdict::Better("N".into());
                let same = |v: &Verdict| *v == Verdict::NoDifference;
                let mut holds = Vec::new();
                if all("P+", &n_wins) && all("H3", &n_wins) {
                    holds.push("**N is justified as a product**".to_string());
                }
                if all("H3", &same) {
                    holds.push(
                        "**the hybrid (PostgreSQL ledger + REV sidecar) is recommended**".into(),
                    );
                }
                if all("P+", &same) {
                    holds.push("**PostgreSQL with the E14 mechanism is recommended**".into());
                }
                let detail: Vec<String> = parts
                    .iter()
                    .map(|(b, m, j, nr)| {
                        let mut t = format!(
                            "N vs {b}: memory {}; {}",
                            m.as_ref().map(word_of).unwrap_or_default(),
                            j.iter()
                                .map(|(m, v)| format!("{m} {}", word_of(v)))
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        if !nr.is_empty() {
                            t += &format!(" ({} NOT RUN)", nr.join(", "));
                        }
                        t
                    })
                    .collect();
                format!(
                    "{} — {}",
                    if holds.is_empty() {
                        "no clause of the rule holds".to_string()
                    } else {
                        holds.join("; ")
                    },
                    detail.join(" · ")
                )
            };
            let _ = writeln!(s, "- `{se}`, {}: {line}", size_label(*n));
            outcomes.push((*n, line.split(" — ").next().unwrap_or("").to_string()));
        }
        let changes: Vec<String> = outcomes
            .windows(2)
            .filter(|w| w[0].1 != w[1].1)
            .map(|w| {
                format!(
                    "between {} and {}: {} → {}",
                    size_label(w[0].0),
                    size_label(w[1].0),
                    w[0].1,
                    w[1].1
                )
            })
            .collect();
        let _ = writeln!(
            s,
            "- `{se}`, crossover (S-1): {}\n",
            if changes.is_empty() {
                "none in range".to_string()
            } else {
                changes.join("; ")
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A point file as `store::write` produces it, cut down to one metric.
    fn point(n: f64, p: f64, diverged: bool) -> Stored {
        let mut t = String::from(
            "meta\tseries\tsingle\nmeta\taccounts\t1000\nmeta\tseed\t1\n\
             load\tN\t0.1\t2\tx\t2\tx\tn\nload\tP+\t0.1\t2\tx\t2\tx\tn\n",
        );
        for run in -3..10i64 {
            let j = (run.rem_euclid(3)) as f64;
            t += &format!("val\tN\t{run}\tq1_p50_us\t{}\n", n + j);
            t += &format!("val\tP+\t{run}\tq1_p50_us\t{}\n", p + j);
        }
        if diverged {
            t += "div\tP+\t2\t1\texample\n";
        }
        let dir = std::env::temp_dir().join(format!(
            "e27-fixture-{}-{n}-{p}-{diverged}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("single-1000-1.tsv");
        std::fs::write(&f, t).unwrap();
        let st = Stored::read(&f).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        st
    }

    fn verdict(st: &Stored) -> Verdict {
        cells(st, &E27)
            .into_iter()
            .find(|(c, _)| c.a == "N" && c.b == "P+" && c.metric == "q1_p50_us")
            .map(|(c, _)| c.verdict)
            .expect("the cell")
    }

    #[test]
    fn the_scorer_reads_a_point_file_and_gates_it() {
        assert_eq!(
            verdict(&point(100.0, 200.0, false)),
            Verdict::Better("N".into())
        );
        assert_eq!(
            verdict(&point(200.0, 100.0, false)),
            Verdict::Better("P+".into())
        );
        assert_eq!(verdict(&point(100.0, 101.0, false)), Verdict::BelowFloor);
    }

    #[test]
    fn a_divergence_anywhere_in_the_point_refuses_the_comparison() {
        assert!(matches!(
            verdict(&point(100.0, 200.0, true)),
            Verdict::Refused(_)
        ));
    }

    #[test]
    fn a_metric_no_arm_reported_is_absent_not_refused() {
        let st = point(100.0, 200.0, false);
        assert!(!cells(&st, &E27)
            .iter()
            .any(|(c, _)| c.metric == "c2_q1_p99_us"));
    }
}
