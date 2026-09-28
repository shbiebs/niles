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

pub const PAIRS: &[(&str, &str)] = &[("N", "P+"), ("N", "P"), ("P+", "P"), ("P", "M")];

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
            if let Ok(st) = Stored::read(&p) {
                out.insert((st.series(), st.num("accounts"), st.num("seed")), st);
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
pub fn cells(st: &Stored) -> Vec<(Cell, Option<String>)> {
    let arms = arms_of(st);
    let mut out = Vec::new();
    let failed = st.kind("failure").next().is_some();
    for (a, b) in PAIRS {
        if !arms.contains(*a) || !arms.contains(*b) {
            continue;
        }
        for (m, _) in METRICS {
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

/// Render both files. Returns the text of the main file.
pub fn render(
    dir: &Path,
    main: &Path,
    detail: &Path,
    arm_lines: &[(String, String)],
    sql_table: &str,
    shape: &Shape,
) -> std::io::Result<String> {
    let detail_name = detail
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("detail")
        .to_string();
    let points = load(dir);
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
    let _ = writeln!(s, "*Generated by `cargo run --release -p comparator -- render{}` from the point files in `results/{}/`. Every number below is read from those files; none is typed. Cycle 14, card R2-02; specification: §5 of the round-2 work order.*\n", shape.flag, dir.file_name().and_then(|x| x.to_str()).unwrap_or("?"));
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
    let _ = writeln!(s, "| toolchain | {} |", meta("toolchain"));
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
    let _ = writeln!(s, "| checkpoint_interval | C = 16 on N (`nilestreamd --checkpoint 16`) and on P+ (`sql/pplus.sql`) |");
    let _ = writeln!(
        s,
        "| residency budget | 5% of keys (rounded), LRU, on N and P+ |"
    );
    let _ = writeln!(s, "| run shape | {} |", shape.run_shape);
    let _ = writeln!(s, "| gate | ≥ 10% and ≥ 3 pooled MADs; floor = 3 × pooled MAD of the warm-up runs; refused on checksum mismatch, oracle divergence, or warm-up MAD > 15% of its median |\n");

    let _ = writeln!(s, "### The arms\n");
    let _ = writeln!(s, "| arm | what it is |\n|---|---|");
    for (n, dsc) in arm_lines {
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
    let _ = writeln!(
        s,
        "N's q3 and q4 are NOT RUN: {}.\n",
        crate::arms::N_NOT_RUN
    );

    let _ = writeln!(s, "### Deviations from the specification, and repairs\n");
    for line in DEVIATIONS {
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
    let _ = writeln!(s, "One exception to the per-seed gate, stated before the measured sweep: `pss_growth_bytes_per_key_read` is judged **across seeds** on each seed's last measured run (joint gate over the five values; no warm-up floor, since a seed has one such value), because its per-run values trend with the keys read so far and are not replicates.\n");
    let _ = writeln!(s, "Each cell is the size verdict over the five seeds (rule in `crates/comparator/src/stats.rs`, fixed before the first measured run): an arm when at least three seeds give it and none gives the other; *no difference* when at least three are no difference or below floor; REFUSED when three or more seeds are refused; *mixed* otherwise. In brackets, the seed tally: arm-A wins / arm-B wins / no difference or below floor / refused. Every underlying cell is in `results/{detail_name}`.\n");
    let mut size_verdicts: SizeVerdicts = BTreeMap::new();
    let mut all_cells: CellsBySize = BTreeMap::new();
    for ((se, n, _), st) in &points {
        all_cells
            .entry((se.clone(), *n))
            .or_default()
            .extend(cells(st));
    }
    for se in &series {
        let _ = writeln!(s, "### Series `{se}`\n");
        let _ = write!(s, "| pair | metric |");
        for n in &sizes {
            let _ = write!(s, " {} |", size_label(*n));
        }
        let _ = writeln!(s, " crossover |");
        let _ = writeln!(s, "|---|---|{}---|", "---|".repeat(sizes.len()));
        for (a, b) in PAIRS {
            for (m, desc) in METRICS {
                let mut row = format!("| {a} vs {b} | {desc} |");
                let mut by_size = Vec::new();
                let mut any = false;
                for n in &sizes {
                    let cs: Vec<&(Cell, Option<String>)> = all_cells
                        .get(&(se.clone(), *n))
                        .map(|v| {
                            v.iter()
                                .filter(|(c, _)| c.a == *a && c.b == *b && c.metric == *m)
                                .collect()
                        })
                        .unwrap_or_default();
                    if cs.is_empty() {
                        row += " — |";
                        continue;
                    }
                    any = true;
                    if let Some(why) = cs.iter().find_map(|(_, nr)| nr.clone()) {
                        let _ = why;
                        row += " NOT RUN |";
                        continue;
                    }
                    if SEED_REPLICATED.contains(m) {
                        let last = |arm: &str| -> Vec<f64> {
                            points
                                .iter()
                                .filter(|(k, _)| &k.0 == se && k.1 == *n)
                                .filter_map(|(_, st)| {
                                    series_of(st, arm, m).0.into_iter().flatten().last()
                                })
                                .collect()
                        };
                        let (va, vb) = (last(a), last(b));
                        let refused = points.iter().filter(|(k, _)| &k.0 == se && k.1 == *n).any(
                            |(_, st)| {
                                diverged(st, a)
                                    || diverged(st, b)
                                    || st.kind("failure").next().is_some()
                            },
                        );
                        let v = if refused {
                            Verdict::Refused("oracle divergence or failure at this size".into())
                        } else {
                            crate::stats::judge_seeds(m, a, b, &va, &vb)
                        };
                        row +=
                            &format!(" {} (across {} seeds) |", v.word(), va.len().min(vb.len()));
                        by_size.push((*n, v));
                        continue;
                    }
                    let cells: Vec<&Cell> = cs.iter().map(|(c, _)| c).collect();
                    let v = aggregate(&cells);
                    let wa = cells
                        .iter()
                        .filter(|c| c.verdict == Verdict::Better((*a).into()))
                        .count();
                    let wb = cells
                        .iter()
                        .filter(|c| c.verdict == Verdict::Better((*b).into()))
                        .count();
                    let nd = cells
                        .iter()
                        .filter(|c| {
                            matches!(c.verdict, Verdict::NoDifference | Verdict::BelowFloor)
                        })
                        .count();
                    let rf = cells
                        .iter()
                        .filter(|c| matches!(c.verdict, Verdict::Refused(_)))
                        .count();
                    let word = match &v {
                        Verdict::Refused(w) if w == "mixed" => "mixed".to_string(),
                        other => other.word(),
                    };
                    row += &format!(" {word} ({wa}/{wb}/{nd}/{rf}) |");
                    by_size.push((*n, v));
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
        for (m, desc) in METRICS {
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
    let _ = writeln!(s, "PSS after load and after the last run (N: the daemon process; PostgreSQL: postmaster and every child). Arm state: N's `nilestream_stats` (resident keys, reads, hits, misses, base rows touched, view answers); PostgreSQL relation sizes (`pg_total_relation_size`) and P+'s resident slots. Median over seeds.\n");
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
    if probe.is_empty() {
        let _ = writeln!(s, "*Not run yet.*\n");
    } else {
        let _ = writeln!(s, "Driver and predicates: `crates/comparator/src/probe.rs`. 10³ accounts, budget 5 against 20 hot keys, 4 readers at the head, one appender, and in the second mode one mutator rewriting a historical leg (+10⁹ / −10⁹ in one transaction). Counts are reads (or resident slots, for lost delta) matching each predicate; deadlocks are 40P01 errors on any connection. M and P are a full materialised view with no partial state and no anchor on a read, so the five — anomalies of partial maintenance — have nothing to act on; the mutable arm with partial state is M+ (P+ without the append-only trigger).\n");
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
        let _ = writeln!(s, "\nThe pre-registration (§5.7) was: at least one exhibited on M or H1, none on N. M is out of scope as stated above; its partial-state counterpart M+ is the arm the prediction is evaluated on here, and H1 waits for R2-03.\n");
    }

    // H-E1.
    let _ = writeln!(s, "## 8. What this says about H-E1\n");
    let _ = writeln!(s, "§7's engine rule needs N to beat **P+ and H3** by the joint gate on resident bytes per key ever read and on p99 read latency under eviction. H3 is R2-03's; this table is the P+ half, per series and size, as §2 found it.{}\n", if shape.flag.is_empty() { " Its p99 rows are the main sweep's, where a run held ~110 samples of a query; the targeted re-run the author chose, `results/E27-p99.md`, is the one that resolves p99." } else { "" });
    let _ = writeln!(
        s,
        "| series | metric | {} |\n|---|---|{}",
        sizes
            .iter()
            .map(|n| size_label(*n))
            .collect::<Vec<_>>()
            .join(" | "),
        "---|".repeat(sizes.len())
    );
    for se in &series {
        for m in [
            "pss_growth_bytes_per_key_read",
            "q1_p99_us",
            "q2_p99_us",
            "c4_q1_p99_us",
            "c4_q2_p99_us",
        ] {
            if let Some(v) = size_verdicts.get(&(se.clone(), "N".into(), "P+".into(), m.into())) {
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
                let _ = writeln!(s, "| {se} | {m} | {} |", cols.join(" | "));
            }
        }
    }
    let _ = writeln!(s);

    // Detail file.
    let _ = writeln!(d, "# E27 — every cell\n\n*Generated with `results/E27-comparator.md`, from the same point files. One row per (pair, metric) per point: medians and MADs over the ten measured runs, pooled MAD, floor (3 × pooled MAD of the warm-up runs), relative change (B − A)/A, MADs apart, verdict and, when refused, why.*\n");
    for ((se, n, seed), st) in &points {
        let _ = writeln!(d, "## `{se}`, {} accounts, seed {seed}\n", size_label(*n));
        let _ = writeln!(d, "| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |\n|---|---|---|---|---|---|---|---|---|---|---|");
        for (c, nr) in cells(st) {
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
pub struct Shape {
    pub title: &'static str,
    /// The flags that reproduce this rendering after `render`.
    pub flag: &'static str,
    pub run_shape: &'static str,
    pub preface: &'static str,
}

pub const MAIN: Shape = Shape {
    title: "E27 — Nilestream against its alternatives, under eviction, across bank sizes",
    flag: "",
    run_shape: "3 warm-up runs then 10 measured runs per (arm, size, seed), arms visited round-robin in an order rotated each run; a run = 200 operations at 1 client (every tenth a two-leg transfer: 90% reads / 10% writes; read mix q1 60%, q2 25%, q3 5%, q4 3%, q5 5%, q6 2%), then q1/q2 reads at 2 and at 4 concurrent clients (60 per client, 70% q1)",
    preface: "",
};

pub const P99: Shape = Shape {
    title: "E27-p99 — read latency tails under eviction, N against P+",
    flag: " --shape p99",
    run_shape: "3 warm-up runs then 10 measured runs per (arm, size, seed), interleaved as in E27; a run = 2,000 point reads at 1 client (q1 and q2 half each, no writes), then 4 clients × 500 reads (q1 and q2 half each) — about 1,000 samples of each query per run and level",
    preface: "**Why this file exists.** In E27's main sweep a run held about 110 samples of q1, so a per-run p99 was its second-largest value and most p99 comparisons were refused by the pre-registered rule (warm-up MAD above 15% of the median). p99 read latency under eviction is half of §7's engine rule, so the author chose on 2026-09-28 a targeted re-run — N against P+ only, point reads only, 1,000 samples per query per run — with the run length fixed before any of its numbers existed (`Config::p99`). Same universe, seeds, sizes, gate, floor and refusal rule as E27. Only the p99 and p50 rows of q1 and q2 are meaningful here; the other metric rows are absent by design.",
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
];

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
        cells(st)
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
        assert!(!cells(&st).iter().any(|(c, _)| c.metric == "c2_q1_p99_us"));
    }
}
