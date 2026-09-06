//! Turning samples into the performance-contract table — **and nothing else into it**.
//!
//! `SPEC-ENGINE.md` Part 0 states four targets relative to PostgreSQL. Until this module
//! existed, that table was a prediction in the typography of a result: every experiment in
//! `results/` reported counted work inside the prototype, and the one wall-clock table in the
//! thesis was in-memory, single-threaded and compared to nothing.
//!
//! So the rule here is mechanical. Every cell of the **Measured** column comes from a sample
//! in a CSV this repository produced. A row with no sample reads `NOT RUN` and carries the
//! reason the target gave. Nothing is typed in by hand, which is why `bench --render` exists
//! rather than a paragraph explaining how to fill the table in.

use crate::workloads::{Sample, ScalingSample};
use std::collections::BTreeMap;

/// The CSV header. Asserted against `Sample::to_csv` by a test, so the two cannot drift.
pub const CSV_HEADER: &str =
    "workload,target,run,operations,wall_ms,p50_us,p99_us,ops_per_sec,durable,not_run,protocol_path,miss_rate";

/// One row of the performance contract.
pub struct ContractRow {
    /// The workload class, matching the `workload` column of the CSV.
    pub workload: &'static str,
    /// How `SPEC-ENGINE.md` Part 0 words the target.
    pub target: &'static str,
    /// The multiple of PostgreSQL the specification claims, or `None` where it claims parity.
    pub factor: Option<f64>,
}

/// The contract, transcribed once.
///
/// Transcribed rather than parsed out of the specification: a parser would silently produce
/// an empty table if the document's formatting changed, and an empty table is the one failure
/// mode that looks like success.
pub fn contract() -> Vec<ContractRow> {
    vec![
        ContractRow {
            workload: "oltp",
            target: "5–10× PostgreSQL",
            factor: Some(5.0),
        },
        ContractRow {
            workload: "analytical",
            target: "10–12× PostgreSQL",
            factor: Some(10.0),
        },
        ContractRow {
            workload: "point",
            target: "parity with PostgreSQL",
            factor: None,
        },
        ContractRow {
            workload: "durable",
            target: "parity with PostgreSQL",
            factor: None,
        },
        // **The row the analytical contract was written for.** `analytical` measures a *cold
        // reconstruction*: the fold runs over the whole base every time, which is one end of
        // the phase diagram and not the end the thesis claims. `report` is the other — the
        // same statement answered out of a view the write path is keeping current, under
        // concurrent appends so that "warm" means maintained rather than stale.
        //
        // Its target is stated as a multiple rather than as parity because a maintained view
        // that were merely at parity with a recompute would be evidence *against* the
        // structural claim: the whole argument for holding derived state is that reading it
        // is cheaper than deriving it. 2.6x is the figure T-33 derives from the measured
        // parts — frame, client, socket — against PostgreSQL's measured recompute, not a
        // number chosen to be reachable.
        ContractRow {
            workload: "report",
            target: "≥ 2.6× PostgreSQL (maintained view vs recompute)",
            factor: Some(2.6),
        },
    ]
}

/// What a row's measurement says about its target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Met,
    NotMet,
    Parity,
    NotRun(String),
}

impl Verdict {
    pub fn label(&self) -> &str {
        match self {
            Verdict::Met => "MET",
            Verdict::NotMet => "NOT MET",
            Verdict::Parity => "PARITY",
            Verdict::NotRun(_) => "NOT RUN",
        }
    }
}

/// The median of a set of samples' throughput, ignoring runs that did not happen.
fn median_ops(samples: &[&Sample]) -> Option<f64> {
    let mut v: Vec<f64> = samples
        .iter()
        .filter(|s| s.not_run.is_none())
        .map(|s| s.ops_per_second())
        .collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(v[v.len() / 2])
}

/// The median absolute deviation of a set of values, in the values' own unit.
///
/// Reported beside every median because a 2-core container is a noisy host and a difference
/// smaller than the spread is not a finding. MAD rather than a standard deviation: one slow
/// run should not widen the reported spread of the other four.
fn mad(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = v[v.len() / 2];
    let mut d: Vec<f64> = v.iter().map(|x| (x - med).abs()).collect();
    d.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(d[d.len() / 2])
}

/// Per-run throughput of the runs that happened.
fn ops_series(samples: &[&Sample]) -> Vec<f64> {
    samples
        .iter()
        .filter(|s| s.not_run.is_none())
        .map(|s| s.ops_per_second())
        .collect()
}

fn median_p99_us(samples: &[&Sample]) -> Option<f64> {
    let mut v: Vec<f64> = samples
        .iter()
        .filter(|s| s.not_run.is_none())
        .map(|s| s.p99.as_nanos() as f64 / 1000.0)
        .collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(v[v.len() / 2])
}

/// Judge one row.
///
/// **Parity is a band, not a point.** A system within ±20% of the baseline has neither beaten
/// it nor lost to it in any sense an operator would act on, and forcing a verdict either way
/// would be reporting noise as a finding. Outside the band the answer is `NOT MET`, including
/// when the engine is *faster* than a parity target — a parity claim that overshoots is still
/// a claim that was not what was written down.
pub fn judge(
    row: &ContractRow,
    ours: Option<f64>,
    baseline: Option<f64>,
    gap: Option<String>,
) -> Verdict {
    if let Some(reason) = gap {
        return Verdict::NotRun(reason);
    }
    let (ours, baseline) = match (ours, baseline) {
        (Some(a), Some(b)) if b > 0.0 => (a, b),
        _ => return Verdict::NotRun("no measurement was produced for this row".into()),
    };
    let ratio = ours / baseline;
    match row.factor {
        Some(f) => {
            if ratio >= f {
                Verdict::Met
            } else {
                Verdict::NotMet
            }
        }
        None => {
            // **One-sided, because the specification is one-sided.** SPEC-ENGINE Part 0 says
            // the engine "MUST NOT regress below parity on point lookups and selective indexed
            // access". A two-sided band reported `NOT MET` on runs where Nilestream's p99 was
            // 1.30x and 1.46x *better* than PostgreSQL's — a contract that was met, recorded as
            // failed, in two runs out of three. The overshoot is reported as the number it is.
            if ratio >= 0.8 {
                Verdict::Parity
            } else {
                Verdict::NotMet
            }
        }
    }
}

/// Render the Part 0 table from samples.
///
/// The output is markdown, and the caller writes it to `results/E16-wallclock.md`. No
/// argument to this function can put a number in the table that did not come from a sample.
pub fn contract_table(samples: &[Sample], gaps: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    out.push_str("| Workload | Contract (SPEC-ENGINE Part 0) | PostgreSQL | Nilestream | Ratio | Verdict |\n");
    out.push_str("|---|---|---|---|---|---|\n");

    for row in contract() {
        let of = |target: &str| -> Vec<&Sample> {
            samples
                .iter()
                .filter(|s| s.workload == row.workload && s.target == target)
                .collect()
        };
        let pg = of("postgres");
        let nls = of("nilestream");

        // For a latency row the honest comparison is the tail, not the throughput: the
        // contract's point-lookup claim is about how long one lookup takes.
        let latency_row = row.factor.is_none() && row.workload == "point";
        let (pg_value, nls_value, unit) = if latency_row {
            (median_p99_us(&pg), median_p99_us(&nls), "µs p99")
        } else {
            (median_ops(&pg), median_ops(&nls), "ops/s")
        };

        // A latency row inverts: lower is better, so the ratio that means "as fast as" is
        // baseline over ours.
        let (a, b) = if latency_row {
            (pg_value, nls_value)
        } else {
            (nls_value, pg_value)
        };
        let gap = gaps
            .get(row.workload)
            .cloned()
            .or_else(|| nls.iter().find_map(|s| s.not_run.clone()));
        let verdict = judge(&row, a, b, gap);

        let show = |v: Option<f64>| match v {
            Some(x) if x >= 1000.0 => format!("{x:.0}"),
            Some(x) => format!("{x:.1}"),
            None => "—".to_string(),
        };
        let ratio = match (a, b) {
            (Some(x), Some(y)) if y > 0.0 => format!("{:.2}×", x / y),
            _ => "—".to_string(),
        };
        out.push_str(&format!(
            "| {} | {} | {} {} | {} {} | {} | **{}** |\n",
            row.workload,
            row.target,
            show(pg_value),
            unit,
            show(nls_value),
            unit,
            ratio,
            verdict.label()
        ));
    }

    // Every `NOT RUN` carries its reason below the table rather than in a cell, because the
    // reasons are sentences and a table that swallowed them would leave a reader guessing.
    let mut notes: Vec<(String, String)> = Vec::new();
    for row in contract() {
        let gap = gaps.get(row.workload).cloned().or_else(|| {
            samples
                .iter()
                .find(|s| s.workload == row.workload && s.target == "nilestream")
                .and_then(|s| s.not_run.clone())
        });
        if let Some(g) = gap {
            notes.push((row.workload.to_string(), g));
        }
    }
    if !notes.is_empty() {
        out.push_str("\nWhy a row is `NOT RUN`:\n\n");
        for (w, why) in notes {
            out.push_str(&format!("* **{w}** — {why}\n"));
        }
    }
    out
}

/// **The analytical workload, statement by statement.**
///
/// The composite ratio says how far apart the two engines are; it cannot say *which operator*
/// the distance is in, and a composite attributed to a single cause is a guess with a number
/// attached. This table is the attribution: one row per statement, median of the per-run
/// medians with the spread beside it, and the coverage difference stated underneath rather
/// than folded into the ratio.
pub fn statement_table(samples: &[Sample]) -> String {
    let mut out = String::new();
    out.push_str("### The analytical workload, statement by statement\n\n");
    out.push_str(
        "Median of the per-run medians, with the median absolute deviation beside it. \
         The **composite** row above is computed from the statements marked `common` \
         and from nothing else.\n\n",
    );
    out.push_str(
        "| Statement | In the ratio | PostgreSQL | Nilestream | Nilestream speed ÷ PostgreSQL |\n",
    );
    out.push_str("|---|---|---|---|---|\n");

    let ms = |s: &Sample| s.p50.as_nanos() as f64 / 1_000_000.0;
    for st in crate::workloads::ANALYTICAL_STATEMENTS {
        let w = format!("analytical:{}", st.id);
        let of = |target: &str| -> Vec<f64> {
            samples
                .iter()
                .filter(|s| s.workload == w && s.target == target && s.not_run.is_none())
                .map(ms)
                .collect()
        };
        let (pg, nls) = (of("postgres"), of("nilestream"));
        let cell = |v: &[f64]| match (median_of(v), mad(v)) {
            (Some(m), Some(d)) => format!("{m:.2} ± {d:.2} ms"),
            _ => "—".to_string(),
        };
        let ratio = match (median_of(&pg), median_of(&nls)) {
            (Some(p), Some(n)) if n > 0.0 => format!("{:.2}×", p / n),
            _ => "—".to_string(),
        };
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            st.id,
            if st.nls.is_some() { "common" } else { "no" },
            cell(&pg),
            cell(&nls),
            ratio
        ));
    }

    let blocked: Vec<&crate::workloads::AnalyticalStatement> =
        crate::workloads::ANALYTICAL_STATEMENTS
            .iter()
            .filter(|s| s.nls.is_none())
            .collect();
    if !blocked.is_empty() {
        out.push_str(&format!(
            "\n**Coverage.** {} of {} statements are outside Nilestream's lowered fragment. \
             They are measured on PostgreSQL — so their cost is on the record — and excluded \
             from the ratio, because a composite that averaged a statement one side cannot \
             express is not a comparison:\n\n",
            blocked.len(),
            crate::workloads::ANALYTICAL_STATEMENTS.len()
        ));
        for st in blocked {
            let why = crate::target::analytical_blocked_reason(st.id)
                .unwrap_or("no reason is recorded, which is itself the finding");
            out.push_str(&format!("* `{}` — {why}\n", st.id));
        }
    }
    out
}

fn median_of(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut v = v.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(v[v.len() / 2])
}

/// **Runs and spread for the contract rows**, which the four-column table above cannot carry.
///
/// A median without its spread cannot be acted on: on this host the same probe varies by tens
/// of percent between invocations, and a reader comparing two commits needs to know which
/// differences are larger than that.
pub fn spread_table(samples: &[Sample]) -> String {
    let mut out = String::new();
    out.push_str("### Runs and spread\n\n");
    out.push_str("| Workload | Target | Runs | Median | MAD | MAD as % of median |\n");
    out.push_str("|---|---|---|---|---|---|\n");
    for row in contract() {
        for target in ["postgres", "nilestream"] {
            let rows: Vec<&Sample> = samples
                .iter()
                .filter(|s| s.workload == row.workload && s.target == target)
                .collect();
            let series = ops_series(&rows);
            let (m, d) = (median_of(&series), mad(&series));
            let pct = match (m, d) {
                (Some(m), Some(d)) if m > 0.0 => format!("{:.1}%", d / m * 100.0),
                _ => "—".into(),
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                row.workload,
                target,
                series.len(),
                m.map(|x| format!("{x:.1} ops/s")).unwrap_or("—".into()),
                d.map(|x| format!("{x:.1}")).unwrap_or("—".into()),
                pct
            ));
        }
    }
    out
}

/// **The E19 scaling table.**
///
/// A separate document from the contract table, and the separation is the point: the four
/// contract rows are single-connection measurements, and a reader who found a 4-connection
/// figure in them would be reading a number the specification does not state a target for.
///
/// Rows are grouped by `(workload, target)` and ordered by connection count, with each
/// level's throughput expressed **relative to that target's own 1-connection level**. The
/// absolute figures are hardware; the ratio is the finding — whether a second and a fourth
/// connection buy anything, or whether the server serialises them.
pub fn scaling_table(samples: &[ScalingSample]) -> String {
    let mut s = String::new();
    s.push_str(
        "| Workload | Target | Connections | Median ops/s | MAD | vs 1 connection | Median p99 |\n\
         |---|---|---|---|---|---|---|\n",
    );

    // BTreeMap so the ordering is the data's, not a hash's: (workload, target, connections).
    let mut by: BTreeMap<(String, String), BTreeMap<u32, Vec<&ScalingSample>>> = BTreeMap::new();
    for x in samples {
        by.entry((x.workload.clone(), x.target.clone()))
            .or_default()
            .entry(x.connections)
            .or_default()
            .push(x);
    }

    for ((workload, target), levels) in &by {
        // The denominator of the `vs 1 connection` column, taken from this target's own
        // single-connection level. Never from the other target's: the two run on the same
        // host but not on the same code, and a cross-target ratio here would be the contract
        // table's job, done without the contract table's care.
        let one = levels
            .get(&1)
            .and_then(|v| median_scaling_ops(v))
            .filter(|x| *x > 0.0);
        for (conns, rows) in levels {
            let ran: Vec<&&ScalingSample> = rows.iter().filter(|r| r.not_run.is_none()).collect();
            if ran.is_empty() {
                let why = rows
                    .iter()
                    .find_map(|r| r.not_run.clone())
                    .unwrap_or_else(|| "no sample".into());
                s.push_str(&format!(
                    "| {workload} | {target} | {conns} | **NOT RUN** | — | — | {why} |\n"
                ));
                continue;
            }
            let series: Vec<f64> = ran.iter().map(|r| r.ops_per_second()).collect();
            let med = median_of(&series).unwrap_or(0.0);
            let spread = mad(&series).unwrap_or(0.0);
            let mut p99s: Vec<f64> = ran
                .iter()
                .map(|r| r.p99.as_nanos() as f64 / 1000.0)
                .collect();
            p99s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p99 = p99s[p99s.len() / 2];
            let rel = match one {
                Some(base) => format!("{:.2}×", med / base),
                None => "—".into(),
            };
            s.push_str(&format!(
                "| {workload} | {target} | {conns} | {med:.0} | {spread:.0} | {rel} | {p99:.0} µs |\n"
            ));
        }
    }
    s
}

/// **Does the widest level buy anything over the one below it?**
///
/// The question E19 exists to answer, rendered rather than left to a reader to compute from
/// twelve rows. The step from 1 to 2 connections is not the interesting one — a
/// single-connection level is round-trip bound, so filling the pipeline flatters it — and the
/// top step is: at that point both cores are already busy, and a target that keeps rising is
/// parallelising while one that falls is contending on something.
///
/// The bands are ±10%, and they are bands for the same reason the contract table's parity is:
/// a 2-core container is a noisy host, and a difference smaller than the run-to-run spread is
/// not a finding.
pub fn scaling_verdicts(samples: &[ScalingSample]) -> String {
    let mut by: BTreeMap<(String, String), BTreeMap<u32, Vec<&ScalingSample>>> = BTreeMap::new();
    for x in samples {
        by.entry((x.workload.clone(), x.target.clone()))
            .or_default()
            .entry(x.connections)
            .or_default()
            .push(x);
    }
    let mut s = String::new();
    s.push_str("| Workload | Target | Top step | Ratio | Reading |\n|---|---|---|---|---|\n");
    for ((workload, target), levels) in &by {
        let mut ks: Vec<u32> = levels.keys().copied().collect();
        ks.sort_unstable();
        if ks.len() < 2 {
            s.push_str(&format!(
                "| {workload} | {target} | — | — | only one connection level was run |\n"
            ));
            continue;
        }
        let (below, top) = (ks[ks.len() - 2], ks[ks.len() - 1]);
        let a = levels.get(&below).and_then(|v| median_scaling_ops(v));
        let b = levels.get(&top).and_then(|v| median_scaling_ops(v));
        let (Some(a), Some(b)) = (a, b) else {
            s.push_str(&format!(
                "| {workload} | {target} | {below} → {top} | — | **NOT RUN** at one of the two levels |\n"
            ));
            continue;
        };
        if a <= 0.0 {
            s.push_str(&format!(
                "| {workload} | {target} | {below} → {top} | — | the lower level answered nothing |\n"
            ));
            continue;
        }
        let r = b / a;
        let reading = if r > 1.10 {
            "**rises** — the added connection is doing work"
        } else if r < 0.90 {
            "**falls** — the added connection costs more than it brings"
        } else {
            "**flat** — the added connection buys nothing measurable"
        };
        s.push_str(&format!(
            "| {workload} | {target} | {below} → {top} | {r:.2}× | {reading} |\n"
        ));
    }
    s
}

fn median_scaling_ops(rows: &[&ScalingSample]) -> Option<f64> {
    let v: Vec<f64> = rows
        .iter()
        .filter(|r| r.not_run.is_none())
        .map(|r| r.ops_per_second())
        .collect();
    median_of(&v)
}

/// **The E19 mixed table, which refuses a row it cannot vouch for.**
///
/// A `mixed` row exists to carry one number: the share of keyed reads that consulted the
/// maintained view and fell back to the fold. Readers alone measure 0.0% of it whatever the
/// engine does — the anchor mismatch is a concurrency phenomenon — so a mixed row *without*
/// that column is not a weaker result, it is the same table E19 already had, published under a
/// name that claims otherwise.
///
/// So a sample whose `fallback_rate` is `None` is rendered as **REFUSED** with its reason, and
/// its throughput figures are not printed at all. Printing them beside an `n/a` would invite
/// exactly the reading the column exists to prevent: that the row was measured and the
/// mechanism was fine. `mixed_table_refusals` is the count, and the caller writes it into the
/// document so a refusal cannot be scrolled past.
pub fn mixed_table(samples: &[crate::workloads::MixedSample]) -> String {
    use std::collections::BTreeMap;
    let mut s = String::new();
    s.push_str(
        "| Target | Readers | Writers | Median reads/s | Read p50 | Read p99 | Median writes/s \
         | Write p99 | Fallback | max_batch | Lock wait p99 | Base epochs |\n\
         |---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|\n",
    );
    // BTreeMap so the ordering is the data's and not a hash's.
    let mut by: BTreeMap<(String, u32, u32), Vec<&crate::workloads::MixedSample>> = BTreeMap::new();
    for x in samples {
        by.entry((x.target.clone(), x.readers, x.writers))
            .or_default()
            .push(x);
    }
    for ((target, readers, writers), rows) in &by {
        let ran: Vec<&&crate::workloads::MixedSample> =
            rows.iter().filter(|r| r.not_run.is_none()).collect();
        if ran.is_empty() {
            let why = rows
                .iter()
                .find_map(|r| r.not_run.clone())
                .unwrap_or_else(|| "no sample".into());
            s.push_str(&format!(
                "| {target} | {readers} | {writers} | NOT RUN | | | | | | | | {why} |\n"
            ));
            continue;
        }
        // **The refusal.** Not a missing cell in an otherwise complete row: the whole row goes,
        // because the throughput figures without the fallback rate are the `point` and
        // `durable` rows again and would read as a result about the mixed workload.
        if ran.iter().any(|r| r.fallback_rate.is_none()) {
            s.push_str(&format!(
                "| {target} | {readers} | {writers} | **REFUSED** | | | | | \
                 *the server could not be asked for its fallback rate; a mixed row without it \
                 is the `point` row under another name* | | | |\n"
            ));
            continue;
        }
        let med = |mut v: Vec<f64>| -> f64 {
            v.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a measured rate"));
            v[v.len() / 2]
        };
        let reads = med(ran.iter().map(|r| r.reads_per_second()).collect());
        let writes = med(ran.iter().map(|r| r.writes_per_second()).collect());
        let us = |f: &dyn Fn(&crate::workloads::MixedSample) -> std::time::Duration| -> f64 {
            med(ran
                .iter()
                .map(|r| f(r).as_nanos() as f64 / 1000.0)
                .collect())
        };
        // `unwrap_or(0.0)` and not `expect`, deliberately. The refusal above has already
        // returned for any level missing a rate, so this cannot be reached — and writing it as
        // a panic would mean that deleting the refusal produces a crash rather than a wrong
        // row. It should produce the wrong row: **0.00% fallback beside a full set of
        // throughput figures**, which is exactly the publishable falsehood the refusal exists
        // to prevent, and exactly what the guard test has to be able to observe.
        let fb = med(ran.iter().map(|r| r.fallback_rate.unwrap_or(0.0)).collect());
        let batch = med(ran
            .iter()
            .map(|r| r.max_batch.unwrap_or(0) as f64)
            .collect());
        let wait = med(ran
            .iter()
            .map(|r| r.lock_wait_p99_us.unwrap_or(0) as f64)
            .collect());
        // **Without this column two mixed rows are not comparable.** A level's read rate falls
        // with accumulated history — measured 23,232 -> 18,998 -> 12,208 reads/s across
        // 6,925 -> 20,436 -> 38,466 sealed epochs, with nothing about the engine changing — so
        // a row that omitted it would invite a comparison that is 1.9x off for reasons that
        // have nothing to do with what is being compared.
        let epochs = med(ran
            .iter()
            .map(|r| r.base_epochs.unwrap_or(0) as f64)
            .collect());
        s.push_str(&format!(
            "| {target} | {readers} | {writers} | {reads:.0} | {:.0} µs | {:.0} µs | \
             {writes:.0} | {:.0} µs | {:.2}% | {batch:.0} | {wait:.0} µs | {epochs:.0} |\n",
            us(&|r| r.read_p50),
            us(&|r| r.read_p99),
            us(&|r| r.write_p99),
            fb * 100.0,
        ));
    }
    s
}

/// How many mixed levels were refused for want of a fallback rate. Zero is the only publishable
/// value, and the caller says so in the document rather than leaving it to be noticed.
pub fn mixed_table_refusals(samples: &[crate::workloads::MixedSample]) -> usize {
    use std::collections::BTreeSet;
    let mut refused = BTreeSet::new();
    for x in samples {
        if x.not_run.is_none() && x.fallback_rate.is_none() {
            refused.insert((x.target.clone(), x.readers, x.writers));
        }
    }
    refused.len()
}

/// **Where a number came from, so two of them are never compared by accident.**
///
/// F-29 is what this exists for. E16's `report` row read 2.68× MET on one instance of host class
/// A and 2.17× NOT MET on another, and the three arms measured *on one instance* were within 8%
/// of each other with MADs ≤4%: the change did not move the row, the instance did. A 2.6×
/// threshold sitting inside cross-instance variance of one host class is not a contract, it is a
/// coin — and nothing in the document said which coin had been tossed.
///
/// Every field here is one a reader needs to know whether two tables may be compared:
///
/// * **host and instance** — the same host *class* is not the same machine. PostgreSQL itself ran
///   35% slower on the second A instance (oltp 4,519 → 2,706 ops/s), which is most of F-29.
/// * **session** — both arms of a ratio must come from one invocation. Across sessions the
///   machine may be differently loaded, and the ratio inherits that rather than the engine.
/// * **barrier** — the durable row is an fsync rate, and the same probe measured 255/s to
///   5,825/s across this project's hosts. A durable ratio without it says nothing.
/// * **commit** — which code produced the numbers, stamped at build time (see `build.rs`).
/// * **baseline commit** — what it is being compared *against*, when the caller names one.
pub struct Provenance {
    host: String,
    instance: String,
    session: String,
    commit: &'static str,
    dirty: bool,
    baseline: Option<String>,
}

impl Provenance {
    pub fn gather(baseline: Option<String>) -> Provenance {
        let read = |p: &str| {
            std::fs::read_to_string(p)
                .ok()
                .map(|s| s.trim().to_string())
        };
        // The boot id distinguishes two instances of one host class, which hostname does not:
        // the two A instances in F-29 answered to the same name.
        let instance = read("/proc/sys/kernel/random/boot_id")
            .map(|b| b.chars().take(8).collect::<String>())
            .or_else(|| read("/etc/machine-id").map(|m| m.chars().take(8).collect()))
            .unwrap_or_else(|| "unknown".into());
        let host = read("/etc/hostname")
            .or_else(|| std::env::var("HOSTNAME").ok())
            .unwrap_or_else(|| "unknown".into());
        // One id per invocation, from the clock. It does not need to be unique across the
        // world, only to differ between two runs whose tables might be laid side by side.
        let session = format!(
            "{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        );
        Provenance {
            host,
            instance,
            session,
            commit: env!("NILES_COMMIT"),
            dirty: env!("NILES_DIRTY") == "true",
            baseline,
        }
    }

    pub fn render(&self, ceiling: Option<crate::storage::Ceiling>) -> String {
        let mut s = String::new();
        s.push_str(
            "## Where these numbers came from

",
        );
        s.push_str(
            "**A ratio in this table may be compared only with another taken in the same              session on the same instance.** E16's `report` row once read 2.68× MET on one              instance of host class A and 2.17× NOT MET on another, and the three engine              versions measured on one instance were within 8% of each other — the code did not              move the row, the machine did. PostgreSQL itself ran 35% slower on the second              instance. A threshold sitting inside cross-instance variance is not a contract.

",
        );
        s.push_str(
            "| | |
|---|---|
",
        );
        s.push_str(&format!(
            "| Host | `{}` |
",
            self.host
        ));
        s.push_str(&format!(
            "| Instance | `{}` — *not* the host class; two instances of one class differ by              more than this experiment's thresholds |
",
            self.instance
        ));
        s.push_str(&format!(
            "| Session | `{}` — both arms of every ratio below come from this one invocation |
",
            self.session
        ));
        s.push_str(&format!(
            "| Engine commit | `{}`{} |
",
            self.commit,
            if self.dirty {
                " — **the tree was dirty; this hash does not identify the code that ran**"
            } else {
                ""
            }
        ));
        s.push_str(&format!(
            "| Baseline commit | {} |
",
            match &self.baseline {
                Some(b) => format!("`{b}` — the A/B this run is one arm of"),
                None =>
                    "*none named*: this run is an absolute measurement, not an A/B. Pass                      `--baseline <commit>` when comparing two engine versions, and measure both                      arms in one session on one instance"
                        .to_string(),
            }
        ));
        match ceiling {
            Some(c) => s.push_str(&format!(
                "| Barrier | **{:.0}** durable commits/s per connection at the median (MAD                  {:.0}) — the `durable` row is an fsync rate, and this project has measured the                  same probe from 255/s to 5,825/s across hosts |
",
                c.median, c.mad
            )),
            None => s.push_str(
                "| Barrier | *not probed in this run* — read no durable row here as a                  device-independent figure |
",
            ),
        }
        s.push('\n');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// **And the document actually renders it.** `Provenance::render` producing the right block
    /// is half the claim; the other half is that E16's builder calls it, and that builder lives
    /// in the binary where a unit test cannot reach it. Asserted at the source, which is exactly
    /// as strong here — there is no behaviour to observe if the call is not there, which is the
    /// failure being guarded against.
    #[test]
    fn the_e16_document_renders_the_provenance_block_under_its_title() {
        let src = include_str!("bin/bench.rs");
        let title = src
            .find(r##""# E16 — The performance contract, measured"##)
            .expect("E16's title");
        let after = &src[title..title + 400];
        assert!(
            after.contains("Provenance::gather"),
            "the provenance block must be rendered immediately under E16's title. Anywhere \
             later and a reader has already read the ratios; not at all and the table is the \
             instance-fragile one F-29 was raised about. Got: {after}"
        );
    }
    /// **The E16 header names the commit that produced it — the guard F-29 asked for.**
    ///
    /// The finding was not that a number was wrong. It was that `report` read 2.68× MET on one
    /// instance of host class A and 2.17× NOT MET on another, the three engine versions measured
    /// on one instance agreed within 8%, and nothing in the document let a reader see that the
    /// two tables were about different machines. A contract ratio without a host, a session and
    /// a commit beside it is a coin toss with a threshold drawn on it.
    #[test]
    fn the_contract_header_names_its_host_session_and_commit() {
        let p = Provenance::gather(None);
        let block = p.render(None);
        for wanted in [
            "Host",
            "Instance",
            "Session",
            "Engine commit",
            "Baseline commit",
        ] {
            assert!(
                block.contains(wanted),
                "the E16 header must name `{wanted}`, or two tables from different machines \
                 look comparable: {block}"
            );
        }
        assert!(
            block.contains(env!("NILES_COMMIT")),
            "and must carry the commit stamped into this binary at build time: {block}"
        );
        assert!(
            block.contains("not probed in this run"),
            "a run with no barrier probe must say so beside the durable row rather than leave \
             the reader to assume a device: {block}"
        );
    }

    /// Naming a baseline says which A/B the table belongs to; not naming one says the opposite,
    /// out loud. The second half matters more: an absolute measurement read as an A/B is exactly
    /// how a 2.68× from one instance came to be compared with a 2.17× from another.
    #[test]
    fn a_run_with_no_baseline_says_it_is_not_an_ab() {
        let named = Provenance::gather(Some("15425b5".into())).render(None);
        assert!(named.contains("15425b5"), "{named}");
        assert!(named.contains("the A/B this run is one arm of"), "{named}");

        let bare = Provenance::gather(None).render(None);
        assert!(
            bare.contains("none named") && bare.contains("not an A/B"),
            "a run with no baseline must say it is an absolute measurement: {bare}"
        );
    }

    /// A verdict needs both arms. `judge` returns `NOT RUN` rather than a `MET` computed against
    /// a baseline that is not there — the structural half of the same-session rule, since both
    /// arms of every ratio are drawn from one invocation's samples.
    #[test]
    fn no_verdict_without_a_baseline_arm() {
        let row = ContractRow {
            workload: "report",
            target: "2.6x PostgreSQL",
            factor: Some(2.6),
        };
        assert!(matches!(
            judge(&row, Some(1000.0), None, None),
            Verdict::NotRun(_)
        ));
        assert!(matches!(
            judge(&row, Some(1000.0), Some(0.0), None),
            Verdict::NotRun(_)
        ));
        assert!(matches!(
            judge(&row, Some(1000.0), Some(100.0), None),
            Verdict::Met
        ));
    }
    /// **A mixed row without its fallback rate does not publish.**
    ///
    /// The column is the row's whole reason for existing: readers alone measure 0.0% of the
    /// anchor-mismatch fallback whatever the engine does, so a mixed row that cannot report it
    /// is the `point` row with a different label. Rendering the throughput figures beside an
    /// `n/a` would invite precisely the reading the column exists to prevent — that the level
    /// ran and the mechanism was fine.
    #[test]
    fn a_mixed_row_without_a_fallback_rate_is_refused_rather_than_rendered() {
        use crate::workloads::MixedSample;
        use std::time::Duration;

        let sample = |fallback: Option<f64>| MixedSample {
            target: "nilestream".into(),
            run: 1,
            readers: 4,
            writers: 2,
            reads: 400_000,
            writes: 2_000,
            wall: Duration::from_secs(4),
            read_p50: Duration::from_micros(27),
            read_p99: Duration::from_micros(180),
            write_p50: Duration::from_micros(7990),
            write_p99: Duration::from_micros(9003),
            fallback_rate: fallback,
            max_batch: Some(3),
            lock_wait_p99_us: Some(1),
            base_epochs: Some(6_925),
            duplicates: 0,
            errors: 0,
            not_run: None,
        };

        let measured = mixed_table(&[sample(Some(0.002))]);
        assert!(
            measured.contains("0.20%"),
            "a measured level must report its rate: {measured}"
        );
        assert!(
            measured.contains("100000"),
            "and its throughput: {measured}"
        );
        assert_eq!(mixed_table_refusals(&[sample(Some(0.002))]), 0);

        let unmeasured = mixed_table(&[sample(None)]);
        assert!(
            unmeasured.contains("REFUSED"),
            "a level with no fallback rate must be refused by name: {unmeasured}"
        );
        assert!(
            !unmeasured.contains("100000"),
            "and its throughput must not be printed — a reader who sees reads/s beside a \
             missing rate will take the level as measured and the mechanism as healthy, which \
             is the one conclusion this column exists to block: {unmeasured}"
        );
        assert_eq!(
            mixed_table_refusals(&[sample(None)]),
            1,
            "and the refusal must be counted, so the document can state it rather than leave \
             it to be noticed"
        );
    }

    /// A level that did not run at all is a third thing, distinct from both: it reports its
    /// reason and is not counted as a refusal, because nothing was measured to refuse.
    #[test]
    fn a_level_that_did_not_run_says_so_and_is_not_a_refusal() {
        let skipped = crate::workloads::mixed_skipped(
            "nilestream",
            4,
            2,
            1,
            "--nls-only: no PostgreSQL arm".into(),
        );
        let t = mixed_table(std::slice::from_ref(&skipped));
        assert!(t.contains("NOT RUN"), "{t}");
        assert!(t.contains("no PostgreSQL arm"), "{t}");
        assert_eq!(mixed_table_refusals(&[skipped]), 0);
    }

    fn sample(workload: &str, target: &str, ops: f64, p99_us: u64) -> Sample {
        Sample {
            workload: workload.into(),
            target: target.into(),
            run: 1,
            operations: 1_000,
            wall: Duration::from_secs_f64(1_000.0 / ops),
            p50: Duration::from_micros(p99_us / 2),
            p99: Duration::from_micros(p99_us),
            durable: true,
            not_run: None,
            protocol_path: crate::workloads::PROTOCOL_PATH,
            miss_rate: None,
        }
    }

    #[test]
    fn the_csv_header_matches_what_a_sample_writes() {
        // The two would otherwise drift, and a CSV whose header disagrees with its rows is
        // one every downstream reader misparses in a different way.
        let s = sample("point", "postgres", 1_000.0, 100);
        assert_eq!(CSV_HEADER.split(',').count(), s.to_csv().split(',').count());
    }

    #[test]
    fn a_throughput_row_is_met_only_at_the_stated_multiple() {
        let row = ContractRow {
            workload: "oltp",
            target: "5–10×",
            factor: Some(5.0),
        };
        assert_eq!(
            judge(&row, Some(5_000.0), Some(1_000.0), None),
            Verdict::Met
        );
        assert_eq!(
            judge(&row, Some(4_999.0), Some(1_000.0), None),
            Verdict::NotMet
        );
        // Faster than claimed still meets a *lower bound*.
        assert_eq!(
            judge(&row, Some(50_000.0), Some(1_000.0), None),
            Verdict::Met
        );
    }

    #[test]
    fn a_parity_row_is_a_floor_because_the_specification_is_a_floor() {
        // `docs/SPEC-ENGINE.md`: "MUST NOT regress below parity on point lookups and selective
        // indexed access". A floor, not a band. The band this replaced reported `NOT MET` on a
        // run where the engine was 46% faster than the baseline it is held to.
        let row = ContractRow {
            workload: "point",
            target: "parity",
            factor: None,
        };
        assert_eq!(judge(&row, Some(100.0), Some(100.0), None), Verdict::Parity);
        assert_eq!(judge(&row, Some(85.0), Some(100.0), None), Verdict::Parity);
        assert_eq!(judge(&row, Some(120.0), Some(100.0), None), Verdict::Parity);
        assert_eq!(judge(&row, Some(50.0), Some(100.0), None), Verdict::NotMet);
        // Faster than the baseline is parity met, not parity missed.
        assert_eq!(judge(&row, Some(200.0), Some(100.0), None), Verdict::Parity);
        assert_eq!(judge(&row, Some(146.0), Some(100.0), None), Verdict::Parity);
    }

    #[test]
    fn a_row_with_no_measurement_is_not_run_rather_than_zero() {
        // The failure this whole module exists to prevent: an absent measurement rendered as
        // a number, which is indistinguishable in a table from a measured one.
        let row = ContractRow {
            workload: "oltp",
            target: "5–10×",
            factor: Some(5.0),
        };
        assert!(matches!(
            judge(&row, None, Some(1_000.0), None),
            Verdict::NotRun(_)
        ));
        assert!(matches!(
            judge(&row, Some(1.0), None, None),
            Verdict::NotRun(_)
        ));
        assert!(matches!(
            judge(&row, Some(1.0), Some(0.0), None),
            Verdict::NotRun(_)
        ));
    }

    #[test]
    fn a_declared_gap_wins_over_any_measurement() {
        // If a target says it cannot serve a workload, no number may appear for it — even if
        // something was measured under that name.
        let row = ContractRow {
            workload: "oltp",
            target: "5–10×",
            factor: Some(5.0),
        };
        let v = judge(
            &row,
            Some(99_999.0),
            Some(1.0),
            Some("no write surface".into()),
        );
        assert_eq!(v.label(), "NOT RUN");
    }

    #[test]
    fn the_table_reports_measured_rows_and_names_the_missing_ones() {
        let samples = vec![
            sample("point", "postgres", 5_000.0, 200),
            sample("point", "nilestream", 5_500.0, 210),
            crate::workloads::skipped("oltp", "nilestream", 1, "no write surface".into()),
            sample("oltp", "postgres", 900.0, 1_500),
        ];
        let table = contract_table(&samples, &BTreeMap::new());
        assert!(table.contains("PARITY"), "{table}");
        assert!(table.contains("NOT RUN"), "{table}");
        assert!(
            table.contains("no write surface"),
            "the reason is in the notes: {table}"
        );
        // Every contract row appears, whether or not it was measured.
        for w in ["oltp", "analytical", "point", "durable", "report"] {
            assert!(table.contains(w), "{w} missing from {table}");
        }
    }

    #[test]
    fn the_median_ignores_runs_that_did_not_happen() {
        let a = sample("point", "postgres", 1_000.0, 100);
        let b = sample("point", "postgres", 3_000.0, 100);
        let c = crate::workloads::skipped("point", "postgres", 3, "x".into());
        let m = median_ops(&[&a, &b, &c]).unwrap();
        assert!((m - 3_000.0).abs() < 1.0, "median of two is the upper: {m}");
        assert_eq!(
            median_ops(&[&c]),
            None,
            "and all-skipped is no measurement at all"
        );
    }

    /// **A scaling row names its connection count, and pools every thread.**
    ///
    /// The guard T-02 exists to leave behind. Two properties, both of which the audit's
    /// scratch harness got wrong:
    ///
    /// * The connection count reaches the table. Without it E19 is a column of unlabelled
    ///   throughput figures in which a reader cannot tell 1 from 4 — which is the entire
    ///   question the experiment asks.
    /// * The throughput of a level is its **pooled** operations over the level's wall clock,
    ///   not the mean of its threads. A four-connection level that answered at the speed of
    ///   one must show `1.00×`, and one that scaled must show `4.00×`; averaging per-thread
    ///   rates reports a serialising server and a scaling one identically.
    #[test]
    fn a_scaling_row_names_its_connection_count_and_pools_every_thread() {
        use crate::workloads::ScalingSample;
        use std::time::Duration;

        let level = |connections: u32, operations: u64, ms: u64| ScalingSample {
            workload: "point".into(),
            target: "nilestream".into(),
            connections,
            run: 1,
            operations,
            wall: Duration::from_millis(ms),
            p50: Duration::from_micros(100),
            p99: Duration::from_micros(400),
            durable: false,
            fallback_rate: None,
            not_run: None,
        };

        // One connection: 1,000 operations in a second. Four connections: 4,000 operations,
        // still in a second — a server that scaled perfectly.
        let scaled = vec![level(1, 1_000, 1_000), level(4, 4_000, 1_000)];
        let t = scaling_table(&scaled);
        assert!(
            t.contains("| point | nilestream | 1 |") && t.contains("| point | nilestream | 4 |"),
            "the connection count is not in the row: {t}"
        );
        assert!(
            t.contains("4.00×"),
            "a level that answered four times the work in the same wall clock must read \
             4.00×, or the table cannot distinguish a scaling server from a serialising one: \
             {t}"
        );

        // The same 4,000 operations taking four times as long: a server that serialises.
        // The per-thread rate is unchanged between these two cases and the pooled rate is
        // not, which is why the pooled one is what is reported.
        let serial = vec![level(1, 1_000, 1_000), level(4, 4_000, 4_000)];
        let t = scaling_table(&serial);
        assert!(
            t.contains("1.00×"),
            "a level that took four times as long for four times the work bought nothing, \
             and the table must say so: {t}"
        );
        assert!(
            !t.contains("4.00×"),
            "a serialising server was reported as scaling: {t}"
        );

        // The top-step reading is computed from the same pooled rates, and it is the
        // sentence the experiment exists to produce: a server that kept rising at its widest
        // level parallelises, and one that fell contends.
        let v = scaling_verdicts(&scaled);
        assert!(v.contains("1 → 4") && v.contains("rises"), "{v}");
        let v = scaling_verdicts(&serial);
        assert!(v.contains("flat"), "{v}");
        let dropping = vec![level(2, 2_000, 1_000), level(4, 4_000, 4_000)];
        let v = scaling_verdicts(&dropping);
        assert!(v.contains("2 → 4") && v.contains("falls"), "{v}");

        // A level that did not run is a row with its reason, never an omission: a missing
        // row invites a reader to assume the number was unremarkable.
        let skipped = vec![
            level(1, 1_000, 1_000),
            crate::workloads::scaling_skipped(
                "point",
                "nilestream",
                4,
                1,
                "the server refused a fourth connection".into(),
            ),
        ];
        let t = scaling_table(&skipped);
        assert!(t.contains("| point | nilestream | 4 | **NOT RUN**"), "{t}");
        assert!(t.contains("refused a fourth connection"), "{t}");
    }

    /// The scaling CSV's header matches what a `ScalingSample` writes.
    #[test]
    fn the_scaling_csv_header_matches_its_rows() {
        use crate::workloads::{ScalingSample, SCALING_CSV_HEADER};
        use std::time::Duration;
        let s = ScalingSample {
            workload: "durable".into(),
            target: "postgres".into(),
            connections: 2,
            run: 3,
            operations: 100,
            wall: Duration::from_millis(50),
            p50: Duration::from_micros(400),
            p99: Duration::from_micros(900),
            durable: true,
            fallback_rate: Some(0.031),
            not_run: None,
        };
        assert_eq!(
            SCALING_CSV_HEADER.split(',').count(),
            s.to_csv().split(',').count(),
            "header: {SCALING_CSV_HEADER}\nrow: {}",
            s.to_csv()
        );
        assert!(
            SCALING_CSV_HEADER.contains("connections"),
            "the column that makes the row mean something is not in the schema"
        );
    }
}

/// **The two asymptotic verdicts (E-2a, E-2b), as functions rather than as inline branches.**
///
/// Pulled out of the E23 renderer because both of them are judgements a test should be able
/// to make fail. One of them shipped wrong: with the control's own slope buried in noise, the
/// parity comparison read a *negative* PostgreSQL slope as Nilestream winning — a benchmark
/// flattering itself with the baseline's measurement error.
pub mod asymptotic {
    use crate::fit::{Fit, NoFit};

    type Fitted = Result<Fit, NoFit>;

    /// E-2b: per row of base, the warm slope must be flat and the control's must be positive.
    ///
    /// Both halves are required. "Flat" alone would be satisfied by a measurement too noisy
    /// to show anything, and the control having a positive slope is what says the experiment
    /// had the resolution to have found one.
    pub fn per_row_of_base(control: &Fitted, warm: &Fitted) -> &'static str {
        match (control, warm) {
            (Ok(c), Ok(w)) => {
                if w.distinguishable_from_zero() {
                    "**NOT MET** (the warm slope is not flat)"
                } else if c.distinguishable_from_zero() && c.slope > 0.0 {
                    "**MET**"
                } else {
                    "**INCONCLUSIVE** (the control's own slope is not positive, so a flat                      result could be an experiment with no resolution rather than a system                      with no growth)"
                }
            }
            _ => "**NOT RUN**",
        }
    }

    /// E-2a: per row of answer, the warm slope must not exceed the control's.
    ///
    /// Compared against the **sum** of the two standard errors: a difference smaller than the
    /// two error bars together is not a difference either way, and calling it one in the
    /// favourable direction is how a benchmark flatters itself.
    pub fn per_row_of_answer(control: &Fitted, warm: &Fitted) -> &'static str {
        match (control, warm) {
            (Ok(c), Ok(w)) => {
                if !c.distinguishable_from_zero() {
                    "**INCONCLUSIVE** (the control's own slope is not distinguishable from                      zero, so there is nothing to be at parity with — widen the answer's                      range)"
                } else if w.slope <= c.slope + (c.stderr + w.stderr) {
                    "**MET** (parity at the floor)"
                } else {
                    "**NOT MET** (cost per row of answer is above PostgreSQL's)"
                }
            }
            _ => "**NOT RUN**",
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::fit::fit;

        fn line(sizes: &[f64], slope: f64, jitter: f64) -> Fitted {
            fit(&sizes
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    // A deterministic wobble, so a verdict test cannot pass or fail by luck.
                    let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                    (*x, 1.0 + slope * x + sign * jitter)
                })
                .collect::<Vec<_>>())
        }

        const SIZES: [f64; 6] = [
            20_000.0,
            200_000.0,
            2_000_000.0,
            20_000.0,
            200_000.0,
            2_000_000.0,
        ];

        #[test]
        fn the_base_row_needs_both_halves_and_says_which_one_failed() {
            let growing = line(&SIZES, 8.4e-5, 0.02);
            let flat = line(&SIZES, 0.0, 0.05);
            assert_eq!(per_row_of_base(&growing, &flat), "**MET**");

            // The warm side grows: not met, whatever the control did.
            assert!(per_row_of_base(&growing, &growing).contains("NOT MET"));

            // The control does not grow either. Nothing was measured well enough to say
            // anything, and calling that a win is the failure this branch exists for.
            assert!(per_row_of_base(&flat, &flat).contains("INCONCLUSIVE"));
        }

        #[test]
        fn the_answer_row_refuses_a_verdict_when_the_control_has_no_slope() {
            // **The defect, reproduced.** E23's first output-axis run held the base at
            // 200,000 rows while the answer grew from 201 to 1,001, so PostgreSQL's cost was
            // dominated by an unchanging scan and its fitted slope came out negative. A
            // comparison of point estimates reads that as parity met.
            let noisy_control = line(&[201.0, 501.0, 1001.0, 201.0, 501.0, 1001.0], -3.3e-3, 6.0);
            let real = line(&[201.0, 501.0, 1001.0, 201.0, 501.0, 1001.0], 3.0e-4, 0.01);
            assert!(
                noisy_control
                    .as_ref()
                    .is_ok_and(|c| !c.distinguishable_from_zero()),
                "the fixture must reproduce the buried control"
            );
            assert!(per_row_of_answer(&noisy_control, &real).contains("INCONCLUSIVE"));

            // With a control that does have a slope, the same warm series passes.
            let control = line(
                &[1_001.0, 10_001.0, 100_001.0, 1_001.0, 10_001.0, 100_001.0],
                8.6e-4,
                0.5,
            );
            let warm = line(
                &[1_001.0, 10_001.0, 100_001.0, 1_001.0, 10_001.0, 100_001.0],
                2.5e-4,
                0.05,
            );
            assert!(per_row_of_answer(&control, &warm).contains("MET"));
            assert!(!per_row_of_answer(&control, &warm).contains("NOT MET"));

            // And a warm series that is genuinely worse is not excused by the error bars.
            let worse = line(
                &[1_001.0, 10_001.0, 100_001.0, 1_001.0, 10_001.0, 100_001.0],
                5.0e-3,
                0.05,
            );
            assert!(per_row_of_answer(&control, &worse).contains("NOT MET"));
        }

        #[test]
        fn a_missing_fit_is_not_run_rather_than_a_pass() {
            let ok = line(&SIZES, 8.4e-5, 0.02);
            let none = fit(&[(1.0, 1.0), (2.0, 2.0)]);
            assert_eq!(per_row_of_base(&none, &ok), "**NOT RUN**");
            assert_eq!(per_row_of_answer(&ok, &none), "**NOT RUN**");
        }
    }
}
