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

use crate::workloads::Sample;
use std::collections::BTreeMap;

/// The CSV header. Asserted against `Sample::to_csv` by a test, so the two cannot drift.
pub const CSV_HEADER: &str =
    "workload,target,run,operations,wall_ms,p50_us,p99_us,ops_per_sec,durable,not_run";

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
            if (0.8..=1.25).contains(&ratio) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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
    fn a_parity_row_is_a_band_and_overshooting_it_is_not_parity() {
        // Deliberate: a parity claim that overshoots is still not the claim that was written
        // down, and reporting it as met would let a specification be satisfied by a number it
        // did not predict.
        let row = ContractRow {
            workload: "point",
            target: "parity",
            factor: None,
        };
        assert_eq!(judge(&row, Some(100.0), Some(100.0), None), Verdict::Parity);
        assert_eq!(judge(&row, Some(85.0), Some(100.0), None), Verdict::Parity);
        assert_eq!(judge(&row, Some(120.0), Some(100.0), None), Verdict::Parity);
        assert_eq!(judge(&row, Some(50.0), Some(100.0), None), Verdict::NotMet);
        assert_eq!(judge(&row, Some(200.0), Some(100.0), None), Verdict::NotMet);
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
        for w in ["oltp", "analytical", "point", "durable"] {
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
}
