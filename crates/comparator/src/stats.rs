//! Verdicts: the joint gate, the resolution floor, the refusal conditions, the seed
//! aggregation and the crossover sizes — all fixed here before the first measured run.
//!
//! * **Per (size, seed, metric, pair):** medians and MADs over the measured runs; pooled MAD
//!   √((MAD_A² + MAD_B²)/2); relative change (B − A)/A; the **floor** = 3 × the pooled MAD of
//!   the two arms' *warm-up* runs. Refused if either arm's warm-up MAD exceeds 15% of its
//!   warm-up median, if either arm diverged from the oracle at this size and seed, or if a
//!   metric is missing from a run. Otherwise BELOW FLOOR if |B − A| < floor; a direction if
//!   |rel| ≥ 10% and |B − A| ≥ 3 pooled MADs; else NO DIFFERENCE.
//! * **Per (size, metric, pair), over the seeds:** the size verdict is an arm when at least
//!   three seeds give it and none gives the other; NO DIFFERENCE when at least three seeds are
//!   NO DIFFERENCE or BELOW FLOOR; REFUSED when three or more seeds are refused; else MIXED.
//! * **Crossover:** between two adjacent size points whose size verdicts are both decisive
//!   (an arm, or NO DIFFERENCE) and differ, bracketed by those two points.

use bank_bench::score::{mad, median, pooled_mad};
use std::collections::BTreeMap;

pub const GATE_REL: f64 = 0.10;
pub const GATE_MADS: f64 = 3.0;
pub const FLOOR_MADS: f64 = 3.0;
pub const WARMUP_MAD_LIMIT: f64 = 0.15;

pub fn more_is_better(metric: &str) -> bool {
    metric == "commits_per_s"
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Better(String),
    NoDifference,
    BelowFloor,
    Refused(String),
}

impl Verdict {
    pub fn word(&self) -> String {
        match self {
            Verdict::Better(a) => format!("**{a} better**"),
            Verdict::NoDifference => "no difference".into(),
            Verdict::BelowFloor => "BELOW FLOOR".into(),
            Verdict::Refused(_) => "REFUSED".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub metric: String,
    pub a: String,
    pub b: String,
    pub accounts: u64,
    pub seed: u64,
    pub med_a: f64,
    pub med_b: f64,
    pub mad_a: f64,
    pub mad_b: f64,
    pub pooled: f64,
    pub floor: f64,
    pub rel: f64,
    pub apart: f64,
    pub verdict: Verdict,
}

/// The values of one metric for one arm: (measured, warm-up), `None` in a slot where a run
/// lacked it.
pub type Series = (Vec<Option<f64>>, Vec<Option<f64>>);

#[allow(clippy::too_many_arguments)]
pub fn judge(
    metric: &str,
    a: &str,
    b: &str,
    sa: &Series,
    sb: &Series,
    diverged: (bool, bool),
    accounts: u64,
    seed: u64,
) -> Cell {
    let all = |v: &[Option<f64>]| v.iter().copied().collect::<Option<Vec<f64>>>();
    let (ma_v, wa_v, mb_v, wb_v) = (all(&sa.0), all(&sa.1), all(&sb.0), all(&sb.1));
    let mut cell = Cell {
        metric: metric.into(),
        a: a.into(),
        b: b.into(),
        accounts,
        seed,
        med_a: f64::NAN,
        med_b: f64::NAN,
        mad_a: f64::NAN,
        mad_b: f64::NAN,
        pooled: f64::NAN,
        floor: f64::NAN,
        rel: f64::NAN,
        apart: f64::NAN,
        verdict: Verdict::Refused(String::new()),
    };
    let (Some(ma_v), Some(wa_v), Some(mb_v), Some(wb_v)) = (ma_v, wa_v, mb_v, wb_v) else {
        cell.verdict = Verdict::Refused("a run lacked this metric".into());
        return cell;
    };
    if ma_v.is_empty() || mb_v.is_empty() || wa_v.is_empty() || wb_v.is_empty() {
        cell.verdict = Verdict::Refused("no runs".into());
        return cell;
    }
    cell.med_a = median(&ma_v);
    cell.med_b = median(&mb_v);
    cell.mad_a = mad(&ma_v);
    cell.mad_b = mad(&mb_v);
    cell.pooled = pooled_mad(cell.mad_a, cell.mad_b);
    cell.floor = FLOOR_MADS * pooled_mad(mad(&wa_v), mad(&wb_v));
    let d = cell.med_b - cell.med_a;
    cell.rel = if cell.med_a != 0.0 {
        d / cell.med_a
    } else {
        f64::NAN
    };
    cell.apart = if cell.pooled > 0.0 {
        d.abs() / cell.pooled
    } else {
        f64::NAN
    };
    let noisy = |w: &[f64]| {
        let m = median(w);
        m != 0.0 && mad(w) > WARMUP_MAD_LIMIT * m.abs()
    };
    cell.verdict = if diverged.0 || diverged.1 {
        Verdict::Refused(format!(
            "oracle divergence on {}",
            match diverged {
                (true, true) => format!("{a} and {b}"),
                (true, false) => a.to_string(),
                _ => b.to_string(),
            }
        ))
    } else if noisy(&wa_v) || noisy(&wb_v) {
        Verdict::Refused(format!(
            "warm-up MAD above 15% of the median on {}",
            match (noisy(&wa_v), noisy(&wb_v)) {
                (true, true) => format!("{a} and {b}"),
                (true, false) => a.to_string(),
                _ => b.to_string(),
            }
        ))
    } else if d.abs() < cell.floor {
        Verdict::BelowFloor
    } else if cell.rel.abs() >= GATE_REL && (cell.pooled == 0.0 || cell.apart >= GATE_MADS) {
        let b_better = (d > 0.0) == more_is_better(metric);
        Verdict::Better(if b_better { b.into() } else { a.into() })
    } else {
        Verdict::NoDifference
    };
    cell
}

/// A verdict with seeds as the replicates: one value per seed per arm, the joint gate over
/// them, no floor (there are no warm-up replicates of a once-per-seed value).
pub fn judge_seeds(metric: &str, a: &str, b: &str, va: &[f64], vb: &[f64]) -> Verdict {
    if va.len() < 3 || vb.len() < 3 {
        return Verdict::Refused(format!(
            "{} and {} seeds carried the value; 3 are required",
            va.len(),
            vb.len()
        ));
    }
    let (ma, mb) = (median(va), median(vb));
    let pm = pooled_mad(mad(va), mad(vb));
    let d = mb - ma;
    let rel = if ma != 0.0 { d / ma } else { f64::NAN };
    if rel.abs() >= GATE_REL && (pm == 0.0 || d.abs() / pm >= GATE_MADS) {
        Verdict::Better(if (d > 0.0) == more_is_better(metric) {
            b.into()
        } else {
            a.into()
        })
    } else {
        Verdict::NoDifference
    }
}

/// The size verdict over seeds.
pub fn aggregate(cells: &[&Cell]) -> Verdict {
    let mut wins: BTreeMap<String, usize> = BTreeMap::new();
    let (mut none, mut refused) = (0, 0);
    for c in cells {
        match &c.verdict {
            Verdict::Better(a) => *wins.entry(a.clone()).or_default() += 1,
            Verdict::NoDifference | Verdict::BelowFloor => none += 1,
            Verdict::Refused(_) => refused += 1,
        }
    }
    if refused >= 3 {
        return Verdict::Refused(format!("{refused} of {} seeds refused", cells.len()));
    }
    if wins.len() == 1 {
        let (a, n) = wins.iter().next().unwrap();
        if *n >= 3 {
            return Verdict::Better(a.clone());
        }
    }
    if none >= 3 {
        return Verdict::NoDifference;
    }
    Verdict::Refused("mixed".into())
}

pub fn decisive(v: &Verdict) -> bool {
    matches!(v, Verdict::Better(_) | Verdict::NoDifference)
}

/// Crossovers along the sorted size points: (lower size, upper size, verdict below, above).
pub fn crossovers(by_size: &[(u64, Verdict)]) -> Vec<(u64, u64, Verdict, Verdict)> {
    by_size
        .windows(2)
        .filter(|w| decisive(&w[0].1) && decisive(&w[1].1) && w[0].1 != w[1].1)
        .map(|w| (w[0].0, w[1].0, w[0].1.clone(), w[1].1.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(m: &[f64], w: &[f64]) -> Series {
        (
            m.iter().map(|&x| Some(x)).collect(),
            w.iter().map(|&x| Some(x)).collect(),
        )
    }

    #[test]
    fn the_gate_fires_only_on_both_ten_percent_and_three_mads() {
        let a = s(&[100.0, 101.0, 99.0, 100.0, 100.5], &[100.0, 100.5, 99.5]);
        let b = s(&[80.0, 81.0, 79.0, 80.0, 80.5], &[80.0, 80.5, 79.5]);
        let c = judge("q1_p50_us", "N", "P", &a, &b, (false, false), 1000, 1);
        assert_eq!(
            c.verdict,
            Verdict::Better("P".into()),
            "lower latency is better"
        );
        let c = judge("commits_per_s", "N", "P", &a, &b, (false, false), 1000, 1);
        assert_eq!(
            c.verdict,
            Verdict::Better("N".into()),
            "more commits is better"
        );
        // 5% apart: no difference, however tight.
        let b = s(&[95.0, 95.1, 94.9, 95.0, 95.0], &[95.0, 95.2, 94.8]);
        let c = judge("q1_p50_us", "N", "P", &a, &b, (false, false), 1000, 1);
        assert_eq!(c.verdict, Verdict::NoDifference);
    }

    #[test]
    fn a_difference_under_the_warm_up_floor_has_no_direction() {
        let a = s(&[100.0, 100.0, 100.0], &[100.0, 110.0, 90.0, 105.0, 95.0]);
        let b = s(&[112.0, 112.0, 112.0], &[112.0, 122.0, 102.0, 117.0, 107.0]);
        let c = judge("q1_p50_us", "N", "P", &a, &b, (false, false), 1000, 1);
        assert_eq!(c.verdict, Verdict::BelowFloor, "{c:?}");
    }

    #[test]
    fn noisy_warm_up_divergence_and_missing_metrics_refuse() {
        let a = s(&[100.0; 5], &[50.0, 100.0, 150.0]);
        let b = s(&[10.0; 5], &[10.0, 10.0, 10.0]);
        assert!(matches!(
            judge("x", "N", "P", &a, &b, (false, false), 1, 1).verdict,
            Verdict::Refused(_)
        ));
        let a = s(&[100.0; 5], &[100.0; 3]);
        assert!(matches!(
            judge("x", "N", "P", &a, &b, (true, false), 1, 1).verdict,
            Verdict::Refused(_)
        ));
        let mut gap = a.clone();
        gap.0[2] = None;
        assert!(matches!(
            judge("x", "N", "P", &gap, &b, (false, false), 1, 1).verdict,
            Verdict::Refused(_)
        ));
    }

    #[test]
    fn seeds_aggregate_and_crossovers_are_bracketed() {
        let mk = |v: Verdict| Cell {
            metric: "m".into(),
            a: "N".into(),
            b: "P".into(),
            accounts: 0,
            seed: 0,
            med_a: 0.0,
            med_b: 0.0,
            mad_a: 0.0,
            mad_b: 0.0,
            pooled: 0.0,
            floor: 0.0,
            rel: 0.0,
            apart: 0.0,
            verdict: v,
        };
        let n = mk(Verdict::Better("N".into()));
        let p = mk(Verdict::Better("P".into()));
        let z = mk(Verdict::NoDifference);
        assert_eq!(
            aggregate(&[&n, &n, &n, &z, &z]),
            Verdict::Better("N".into())
        );
        assert!(matches!(
            aggregate(&[&n, &n, &n, &p, &z]),
            Verdict::Refused(_)
        ));
        assert_eq!(aggregate(&[&z, &z, &z, &n, &n]), Verdict::NoDifference);
        let x = crossovers(&[
            (1000, Verdict::Better("P".into())),
            (10_000, Verdict::NoDifference),
            (100_000, Verdict::Better("N".into())),
        ]);
        assert_eq!(x.len(), 2);
        assert_eq!((x[0].0, x[0].1), (1000, 10_000));
    }
}
