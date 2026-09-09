//! **The arithmetic, done once, by the program that owns the data.**
//!
//! Cycle 10's harness ended with `note "Paste this whole transcript"` and a list of the
//! statistics a reader was expected to compute from it. Two auditors then computed them by
//! hand from the same logs and disagreed — one of them (this one) published six pooled MADs
//! that were wrong, having taken the mean of two MADs where the pooled MAD is their RMS. The
//! transcript was not at fault and neither reader was careless; the arrangement was, because
//! it made the last step of every measurement a manual one and then scored pass lines against
//! its output.
//!
//! `bench --score <dir>` reads the per-replicate CSVs a run wrote and prints the table. The
//! harness prints *that* table and performs no arithmetic of its own.
//!
//! # What it computes, and the definitions it commits to
//!
//! Per arm, working point, level and metric, over the measured replicates:
//! median, MAD (median absolute deviation about that median), the range, and the count.
//! Between two arms:
//!
//! * **pooled MAD** = `sqrt((mad_a^2 + mad_b^2) / 2)` — the RMS of the two, not their mean.
//! * **relative change** = `(median_b - median_a) / median_a`, signed, against arm A as the
//!   reference.
//! * **MADs apart** = `|median_b - median_a| / pooled_mad`.
//! * **the gate**: a row fires only when `|relative change| >= 10%` **and**
//!   `MADs apart >= 3`. Either alone is `noise-limited`, which is a result and is printed as
//!   one. Both thresholds are printed on every row, so a row carries the gate it was scored
//!   against instead of the gate the reader assumes.
//!
//! # What it refuses
//!
//! * An arm with fewer than [`MIN_REPLICATES`] measured replicates. Five is the harness's
//!   own floor; a median of three is a middle value and its MAD is barely a dispersion.
//! * A pooled MAD of zero with a non-zero difference: dividing by it would print `inf` MADs
//!   apart and fire every gate. The row is `refused` and says so.
//! * A metric that is `n/a` in any replicate of either arm. A median over "the ones that
//!   answered" is a median over a population selected by whether the measurement worked.
//!
//! Refusals are rows, not omissions: a row that could not be scored is printed with
//! `verdict=refused` and a reason, because a table that silently drops what it could not
//! score is a table whose length depends on its own failures.

use crate::summary;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The smallest number of measured replicates an arm may be scored on.
pub const MIN_REPLICATES: usize = 5;

/// The relative-change half of the joint gate.
pub const GATE_REL: f64 = 0.10;

/// The dispersion half of the joint gate, in pooled MADs.
pub const GATE_MADS: f64 = 3.0;

/// The metrics scored, and whether more is better.
///
/// Both directions are here on purpose. A read-throughput win that costs writer throughput or
/// write tail latency is not a win, and cycle 10's merge arm is exactly that shape: reads
/// unchanged within noise, writers down 30%, write p99 up 53–80%. A scorer that printed only
/// the read row would have reported it as "no effect".
pub const METRICS: &[(&str, bool)] = &[
    ("reads_per_second", true),
    ("writes_per_second", true),
    ("read_p99_us", false),
    ("write_p99_us", false),
    ("read_p50_us", false),
    ("write_p50_us", false),
];

/// One replicate's row for one level.
#[derive(Debug, Clone)]
pub struct Row {
    pub arm: String,
    pub point: String,
    pub rep: String,
    pub target: String,
    pub readers: u32,
    pub writers: u32,
    /// Column name to value, `None` when the CSV said `n/a`.
    pub values: BTreeMap<String, Option<f64>>,
    /// `Some(reason)` when the level did not run.
    pub not_run: Option<String>,
}

impl Row {
    pub fn level(&self) -> String {
        format!("{}r/{}w", self.readers, self.writers)
    }
}

/// What one comparison came to.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Fires,
    NoiseLimited,
    Refused(String),
}

impl Verdict {
    pub fn word(&self) -> &'static str {
        match self {
            Verdict::Fires => "fires",
            Verdict::NoiseLimited => "noise-limited",
            Verdict::Refused(_) => "refused",
        }
    }
}

/// One scored row: one metric, one level, one working point, two arms.
#[derive(Debug, Clone)]
pub struct Scored {
    pub point: String,
    pub level: String,
    pub readers: u32,
    pub writers: u32,
    pub metric: String,
    pub arm_a: String,
    pub arm_b: String,
    pub n_a: usize,
    pub n_b: usize,
    pub median_a: f64,
    pub median_b: f64,
    pub mad_a: f64,
    pub mad_b: f64,
    pub min_a: f64,
    pub max_a: f64,
    pub min_b: f64,
    pub max_b: f64,
    pub pooled_mad: f64,
    pub rel_change: f64,
    pub mads_apart: f64,
    pub verdict: Verdict,
    /// `faster` / `slower` / `none`, in the metric's own sense of better.
    pub direction: &'static str,
}

/// The median of a non-empty slice. Even lengths take the mean of the two middles.
pub fn median(xs: &[f64]) -> f64 {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let n = v.len();
    if n == 0 {
        return f64::NAN;
    }
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// The median absolute deviation about the median. Not scaled to a normal σ: this project's
/// gate counts raw MADs and a 1.4826 factor would silently move it.
pub fn mad(xs: &[f64]) -> f64 {
    let m = median(xs);
    let d: Vec<f64> = xs.iter().map(|x| (x - m).abs()).collect();
    median(&d)
}

/// `sqrt((a^2 + b^2) / 2)` — the RMS of the two arms' MADs.
pub fn pooled_mad(a: f64, b: f64) -> f64 {
    ((a * a + b * b) / 2.0).sqrt()
}

/// Parse one output root: every `results-<arm>-<point>-<rep>/E19-scaling/mixed.csv` under it.
///
/// The directory name is the only place the arm and the working point are recorded, so a
/// directory this parser cannot name is reported rather than skipped — a run whose arms were
/// silently halved is a run whose medians are over the wrong population.
pub fn read_rows(root: &Path) -> (Vec<Row>, Vec<String>) {
    let mut rows = Vec::new();
    let mut problems = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        problems.push(format!("cannot read {}", root.display()));
        return (rows, problems);
    };
    let mut dirs: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    for d in dirs {
        let name = d
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let Some(rest) = name.strip_prefix("results-") else {
            continue;
        };
        // `<arm>-<point>-<rep>`; the arm may not contain `-`, and none of the arm names this
        // harness uses does. A name that does not split into three is named, not guessed at.
        let parts: Vec<&str> = rest.split('-').collect();
        if parts.len() != 3 {
            problems.push(format!(
                "{name}: not `results-<arm>-<point>-<rep>`, so its arm cannot be read from it"
            ));
            continue;
        }
        let csv = d.join("E19-scaling").join("mixed.csv");
        let Ok(text) = std::fs::read_to_string(&csv) else {
            problems.push(format!("{name}: no E19-scaling/mixed.csv"));
            continue;
        };
        match parse_mixed_csv(&text, parts[0], parts[1], parts[2]) {
            Ok(mut r) => rows.append(&mut r),
            Err(e) => problems.push(format!("{name}: {e}")),
        }
    }
    (rows, problems)
}

/// Parse a `mixed.csv`, checking its header against the one this build writes.
///
/// A header this build does not write is refused outright. Reading a column by position out
/// of a file written to a different schema is how a `fallback_rate` column that did not exist
/// became four cycles of comparisons between two schemas; reading it by *name* out of a stale
/// header is the same defect with an extra step.
pub fn parse_mixed_csv(text: &str, arm: &str, point: &str, rep: &str) -> Result<Vec<Row>, String> {
    let mut lines = text.lines();
    let header = lines.next().ok_or("empty file")?.trim_end();
    let want = crate::workloads::MIXED_CSV_HEADER;
    if header != want {
        return Err(format!(
            "header is not the one this build writes.\n      file: {header}\n      build: {want}"
        ));
    }
    let names: Vec<&str> = want.split(',').collect();
    let mut out = Vec::new();
    for (i, l) in lines.enumerate() {
        if l.trim().is_empty() {
            continue;
        }
        let cells: Vec<&str> = l.split(',').collect();
        if cells.len() != names.len() {
            return Err(format!(
                "row {} has {} cells and the header has {}",
                i + 2,
                cells.len(),
                names.len()
            ));
        }
        let cell = |n: &str| -> &str {
            let idx = names.iter().position(|x| *x == n).expect("in the header");
            cells[idx].trim()
        };
        let num = |n: &str| -> Option<f64> {
            let c = cell(n);
            if c == summary::NOT_MEASURED {
                None
            } else {
                c.parse::<f64>().ok()
            }
        };
        let readers = cell("readers").parse::<u32>().map_err(|e| e.to_string())?;
        let writers = cell("writers").parse::<u32>().map_err(|e| e.to_string())?;
        let mut values = BTreeMap::new();
        for n in &names {
            if matches!(*n, "target" | "not_run") {
                continue;
            }
            values.insert((*n).to_string(), num(n));
        }
        let nr = cell("not_run");
        out.push(Row {
            arm: arm.to_string(),
            point: point.to_string(),
            rep: rep.to_string(),
            target: cell("target").to_string(),
            readers,
            writers,
            values,
            not_run: (!nr.is_empty()).then(|| nr.to_string()),
        });
    }
    Ok(out)
}

/// Score every (point, level, metric) for which two arms have rows.
///
/// Arm A is the reference. `baseline` is A when it is present, then `pinned`, then the
/// alphabetically first name — fixed rather than incidental, because the sign of every
/// relative change depends on it and a table whose reference depended on directory ordering
/// would flip its own conclusions between runs.
pub fn score(rows: &[Row]) -> Vec<Scored> {
    let mut arms: Vec<String> = rows.iter().map(|r| r.arm.clone()).collect();
    arms.sort();
    arms.dedup();
    let mut out = Vec::new();
    if arms.len() < 2 {
        return out;
    }
    let pick_a = |arms: &[String]| -> String {
        for preferred in ["baseline", "pinned", "control"] {
            if let Some(a) = arms.iter().find(|a| *a == preferred) {
                return a.clone();
            }
        }
        arms[0].clone()
    };
    let a_name = pick_a(&arms);
    for b_name in arms.iter().filter(|a| **a != a_name) {
        let mut points: Vec<String> = rows.iter().map(|r| r.point.clone()).collect();
        points.sort();
        points.dedup();
        for point in &points {
            let mut levels: Vec<(u32, u32)> = rows.iter().map(|r| (r.readers, r.writers)).collect();
            levels.sort();
            levels.dedup();
            for (readers, writers) in &levels {
                for (metric, more_is_better) in METRICS {
                    let take = |arm: &str| -> (Vec<f64>, usize, usize) {
                        let sel: Vec<&Row> = rows
                            .iter()
                            .filter(|r| {
                                r.arm == arm
                                    && r.point == *point
                                    && r.readers == *readers
                                    && r.writers == *writers
                            })
                            .collect();
                        let total = sel.len();
                        let mut vals = Vec::new();
                        let mut absent = 0;
                        for r in sel {
                            if r.not_run.is_some() {
                                absent += 1;
                                continue;
                            }
                            match r.values.get(*metric).copied().flatten() {
                                Some(v) => vals.push(v),
                                None => absent += 1,
                            }
                        }
                        (vals, total, absent)
                    };
                    let (va, ta, aa) = take(&a_name);
                    let (vb, tb, ab) = take(b_name);
                    if ta == 0 && tb == 0 {
                        continue;
                    }
                    let (ma, mb) = (median(&va), median(&vb));
                    let (da, db) = (mad(&va), mad(&vb));
                    let pm = pooled_mad(da, db);
                    let rel = if ma != 0.0 { (mb - ma) / ma } else { f64::NAN };
                    let apart = if pm > 0.0 {
                        (mb - ma).abs() / pm
                    } else {
                        f64::NAN
                    };
                    let verdict = if va.len() < MIN_REPLICATES || vb.len() < MIN_REPLICATES {
                        Verdict::Refused(format!(
                            "{} of {} and {} of {} replicates carried this metric; \
                             {MIN_REPLICATES} are required",
                            va.len(),
                            ta,
                            vb.len(),
                            tb
                        ))
                    } else if aa > 0 || ab > 0 {
                        Verdict::Refused(format!(
                            "{aa} arm-A and {ab} arm-B replicates reported this metric as \
                             `n/a` or did not run; a median over the ones that answered is a \
                             median over a population selected by whether the measurement \
                             worked"
                        ))
                    } else if !ma.is_finite() || !mb.is_finite() {
                        Verdict::Refused("a median is not finite".into())
                    } else if pm == 0.0 && ma != mb {
                        Verdict::Refused(
                            "both arms have a zero MAD and different medians: the dispersion \
                             this gate divides by is zero, so `MADs apart` is not a number"
                                .into(),
                        )
                    } else if rel.abs() >= GATE_REL && apart >= GATE_MADS {
                        Verdict::Fires
                    } else {
                        Verdict::NoiseLimited
                    };
                    let better = if mb == ma {
                        "none"
                    } else if (mb > ma) == *more_is_better {
                        "better"
                    } else {
                        "worse"
                    };
                    out.push(Scored {
                        point: point.clone(),
                        level: format!("{readers}r/{writers}w"),
                        readers: *readers,
                        writers: *writers,
                        metric: (*metric).to_string(),
                        arm_a: a_name.clone(),
                        arm_b: b_name.clone(),
                        n_a: va.len(),
                        n_b: vb.len(),
                        median_a: ma,
                        median_b: mb,
                        mad_a: da,
                        mad_b: db,
                        min_a: va.iter().copied().fold(f64::INFINITY, f64::min),
                        max_a: va.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                        min_b: vb.iter().copied().fold(f64::INFINITY, f64::min),
                        max_b: vb.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                        pooled_mad: pm,
                        rel_change: rel,
                        mads_apart: apart,
                        verdict,
                        direction: better,
                    });
                }
            }
        }
    }
    out
}

impl Scored {
    /// The machine-readable rendering. One line, [`summary::SCORE_HEADER`]'s fields.
    pub fn summary_line(&self) -> String {
        let f = |x: f64| {
            if x.is_finite() {
                format!("{x:.4}")
            } else {
                summary::NOT_MEASURED.to_string()
            }
        };
        summary::line(
            "score",
            summary::SCORE_HEADER,
            &[
                self.point.clone(),
                self.level.clone(),
                self.readers.to_string(),
                self.writers.to_string(),
                self.metric.clone(),
                self.arm_a.clone(),
                self.arm_b.clone(),
                self.n_a.to_string(),
                self.n_b.to_string(),
                f(self.median_a),
                f(self.median_b),
                f(self.mad_a),
                f(self.mad_b),
                f(self.min_a),
                f(self.max_a),
                f(self.min_b),
                f(self.max_b),
                f(self.pooled_mad),
                f(self.rel_change),
                f(self.mads_apart),
                f(GATE_REL),
                f(GATE_MADS),
                self.verdict.word().to_string(),
                self.direction.to_string(),
            ],
        )
    }
}

/// The per-arm counter table, read from the `flights` summary lines of a run's logs.
///
/// Returns `(arm, point, shape, line)` in directory order. Logs that carry no summary line —
/// every log written before this cycle — contribute nothing and are reported by the caller as
/// *missing*, which is the honest description of a build that did not print them.
pub fn read_flight_lines(root: &Path) -> Vec<(String, String, String, summary::Line)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return out;
    };
    let mut files: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    files.sort();
    for p in files {
        let name = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let Some(rest) = name
            .strip_prefix("bench-")
            .and_then(|s| s.strip_suffix(".log"))
        else {
            continue;
        };
        let parts: Vec<&str> = rest.split('-').collect();
        if parts.len() != 3 {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        for l in summary::Line::all_of(&text, "flights") {
            let shape = l.text("shape").unwrap_or_default();
            out.push((parts[0].to_string(), parts[1].to_string(), shape, l));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pooled_mad_is_the_rms_and_not_the_mean() {
        // 6.0 and 8.0: mean 7.0, RMS sqrt((36+64)/2) = sqrt(50) = 7.0710678...
        let p = pooled_mad(6.0, 8.0);
        assert!((p - 50f64.sqrt()).abs() < 1e-12, "{p}");
        assert!(
            (p - 7.0).abs() > 0.07,
            "the mean would have been 7.0, and it is not"
        );
    }

    #[test]
    fn median_and_mad_on_a_known_sample() {
        let x = [1.0, 2.0, 4.0, 8.0, 16.0];
        assert_eq!(median(&x), 4.0);
        // deviations 3,2,0,4,12 -> median 3
        assert_eq!(mad(&x), 3.0);
    }

    #[test]
    fn an_arm_with_four_replicates_is_refused_and_still_printed() {
        let mk = |arm: &str, n: usize, v: f64| -> Vec<Row> {
            (0..n)
                .map(|i| Row {
                    arm: arm.into(),
                    point: "full".into(),
                    rep: i.to_string(),
                    target: "nilestream".into(),
                    readers: 6,
                    writers: 3,
                    values: METRICS
                        .iter()
                        .map(|(m, _)| ((*m).to_string(), Some(v + i as f64)))
                        .collect(),
                    not_run: None,
                })
                .collect()
        };
        let mut rows = mk("baseline", 4, 100.0);
        rows.extend(mk("candidate", 5, 100.0));
        let s = score(&rows);
        assert!(!s.is_empty());
        for r in &s {
            assert!(matches!(r.verdict, Verdict::Refused(_)), "{:?}", r.verdict);
        }
    }

    #[test]
    fn ten_percent_alone_does_not_fire_and_neither_do_three_mads_alone() {
        let mk = |arm: &str, vals: &[f64]| -> Vec<Row> {
            vals.iter()
                .enumerate()
                .map(|(i, v)| Row {
                    arm: arm.into(),
                    point: "full".into(),
                    rep: i.to_string(),
                    target: "nilestream".into(),
                    readers: 6,
                    writers: 3,
                    values: [("reads_per_second".to_string(), Some(*v))]
                        .into_iter()
                        .collect(),
                    not_run: None,
                })
                .collect()
        };
        // Wide dispersion, 12% apart: clears the relative half, not the MAD half.
        let mut rows = mk("baseline", &[80.0, 90.0, 100.0, 110.0, 120.0]);
        rows.extend(mk("candidate", &[92.0, 102.0, 112.0, 122.0, 132.0]));
        let s: Vec<Scored> = score(&rows)
            .into_iter()
            .filter(|r| r.metric == "reads_per_second")
            .collect();
        assert_eq!(s.len(), 1);
        assert!(s[0].rel_change.abs() >= GATE_REL, "{}", s[0].rel_change);
        assert!(s[0].mads_apart < GATE_MADS, "{}", s[0].mads_apart);
        assert_eq!(s[0].verdict, Verdict::NoiseLimited);

        // Tight dispersion, 2% apart: clears the MAD half, not the relative half. Both arms
        // have a *non-zero* MAD on purpose — a zero-MAD pair is the separate refusal below,
        // and using one here would have tested that path while claiming to test this one.
        let mut rows = mk("baseline", &[100.0, 101.0, 99.0, 100.0, 100.5]);
        rows.extend(mk("candidate", &[102.0, 103.0, 101.0, 102.0, 102.5]));
        let s = score(&rows);
        let r = s
            .iter()
            .find(|r| r.metric == "reads_per_second")
            .expect("a row");
        assert!(r.rel_change.abs() < GATE_REL, "{}", r.rel_change);
        assert!(r.mads_apart >= GATE_MADS, "{}", r.mads_apart);
        assert_eq!(r.verdict, Verdict::NoiseLimited);
    }

    #[test]
    fn a_zero_mad_with_different_medians_is_refused_and_not_infinite() {
        let mk = |arm: &str, v: f64| -> Vec<Row> {
            (0..5)
                .map(|i| Row {
                    arm: arm.into(),
                    point: "full".into(),
                    rep: i.to_string(),
                    target: "nilestream".into(),
                    readers: 6,
                    writers: 3,
                    values: [("reads_per_second".to_string(), Some(v))]
                        .into_iter()
                        .collect(),
                    not_run: None,
                })
                .collect()
        };
        let mut rows = mk("baseline", 100.0);
        rows.extend(mk("candidate", 200.0));
        let s = score(&rows);
        let r = s
            .iter()
            .find(|r| r.metric == "reads_per_second")
            .expect("a row");
        assert!(matches!(r.verdict, Verdict::Refused(_)), "{:?}", r.verdict);
    }

    #[test]
    fn a_stale_header_is_refused_rather_than_read_by_position() {
        let stale = "target,run,readers,writers,reads,writes,wall_ms,reads_per_second\n\
                     nilestream,1,6,3,10,10,1000.0,10.0\n";
        let e = parse_mixed_csv(stale, "baseline", "full", "1").expect_err("refused");
        assert!(e.contains("header is not the one this build writes"), "{e}");
    }
}
