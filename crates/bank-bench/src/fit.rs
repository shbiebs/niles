//! **Ordinary least squares, with the standard error the slope is judged against.**
//!
//! E23 asks a question no single measurement can answer: does a maintained view's cost grow
//! with the size of the base it was derived from? The thesis's foundational hypothesis F1
//! says it must not — a stream-first system's per-answer cost cannot grow with accumulated
//! input — and the only honest way to test "does not grow" is to fit a slope and ask whether
//! it is distinguishable from zero.
//!
//! Hence the standard error. A slope of 0.000003 ms per row is not evidence of anything on
//! its own; a slope of 0.000003 ± 0.000001 is a positive slope, and 0.000003 ± 0.000020 is
//! not. Publishing a point estimate and calling it flat is the specific mistake this module
//! exists to prevent — and it is an easy one to make, because a slope near zero always
//! *looks* like the answer the hypothesis wanted.
//!
//! Three points minimum, and the reason is not conservatism: with two points the residual
//! degrees of freedom are zero, the fit is exact, and the standard error is 0/0. A two-point
//! "slope with an error bar" would report a false precision that no amount of care in the
//! surrounding prose could undo, so [`fit`] refuses rather than produces one.

/// A fitted line and what is known about its slope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    /// The slope, in the y unit per x unit.
    pub slope: f64,
    /// The intercept, in the y unit.
    pub intercept: f64,
    /// The standard error of the slope. `0.0` only when every residual is zero.
    pub stderr: f64,
    /// How many points were fitted.
    pub n: usize,
    /// Coefficient of determination. Reported because a slope from a fit that explains
    /// nothing is a slope through noise, however small its standard error.
    pub r2: f64,
}

impl Fit {
    /// Whether the slope is distinguishable from zero at two standard errors.
    ///
    /// The criterion the thesis states for H-F1, applied literally: *not* distinguishable is
    /// the finding the hypothesis predicts, so the test is written to be failable — a slope
    /// this call reports as flat is one whose confidence interval contains zero, not one
    /// that happened to come out small.
    pub fn distinguishable_from_zero(&self) -> bool {
        self.slope.abs() > 2.0 * self.stderr
    }

    /// The verdict as a word, for a results table.
    pub fn verdict(&self) -> &'static str {
        if !self.distinguishable_from_zero() {
            "flat"
        } else if self.slope > 0.0 {
            "positive"
        } else {
            "negative"
        }
    }
}

/// Why a fit could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoFit {
    /// Fewer than three distinct x values.
    TooFewPoints(usize),
    /// Every point has the same x, so there is no slope to speak of.
    NoSpread,
}

impl std::fmt::Display for NoFit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoFit::TooFewPoints(n) => write!(
                f,
                "{n} distinct sizes; a slope needs three. With two the fit is exact, the \
                 residual degrees of freedom are zero, and the standard error is 0/0 — a \
                 number that would look like certainty and mean nothing"
            ),
            NoFit::NoSpread => write!(f, "every point has the same x, so there is no slope"),
        }
    }
}

/// Fit `y = a + b·x` and report the slope with its standard error.
pub fn fit(points: &[(f64, f64)]) -> Result<Fit, NoFit> {
    let mut xs: Vec<f64> = points.iter().map(|(x, _)| *x).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).expect("no NaN among the sizes"));
    xs.dedup();
    if xs.len() < 3 {
        return Err(NoFit::TooFewPoints(xs.len()));
    }

    let n = points.len() as f64;
    let mx = points.iter().map(|(x, _)| *x).sum::<f64>() / n;
    let my = points.iter().map(|(_, y)| *y).sum::<f64>() / n;
    let sxx: f64 = points.iter().map(|(x, _)| (x - mx) * (x - mx)).sum();
    if sxx <= 0.0 {
        return Err(NoFit::NoSpread);
    }
    let sxy: f64 = points.iter().map(|(x, y)| (x - mx) * (y - my)).sum();
    let slope = sxy / sxx;
    let intercept = my - slope * mx;

    let ss_res: f64 = points
        .iter()
        .map(|(x, y)| {
            let e = y - (intercept + slope * x);
            e * e
        })
        .sum();
    let ss_tot: f64 = points.iter().map(|(_, y)| (y - my) * (y - my)).sum();
    // n - 2: one degree of freedom spent on the slope and one on the intercept.
    let dof = (points.len() as f64 - 2.0).max(1.0);
    let stderr = (ss_res / dof / sxx).sqrt();
    let r2 = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };

    Ok(Fit {
        slope,
        intercept,
        stderr,
        n: points.len(),
        r2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_is_recovered_exactly_and_its_error_is_zero() {
        let pts: Vec<(f64, f64)> = (0..5).map(|i| (i as f64, 3.0 + 2.0 * i as f64)).collect();
        let f = fit(&pts).expect("five points on a line");
        assert!((f.slope - 2.0).abs() < 1e-9, "{f:?}");
        assert!((f.intercept - 3.0).abs() < 1e-9, "{f:?}");
        assert!(f.stderr < 1e-9, "an exact fit has no residual: {f:?}");
        assert!((f.r2 - 1.0).abs() < 1e-9);
        assert!(f.distinguishable_from_zero());
        assert_eq!(f.verdict(), "positive");
    }

    #[test]
    fn a_flat_series_with_noise_is_reported_as_flat_rather_than_as_a_tiny_slope() {
        // The case the whole module is for. The measurements wobble; the underlying cost
        // does not grow. A point estimate alone would be a small positive number, and
        // reporting *that* as "cost grows with the base" is the misreading H-F1 invites.
        let pts = vec![
            (20_000.0, 1.50),
            (200_000.0, 1.47),
            (2_000_000.0, 1.55),
            (20_000.0, 1.44),
            (200_000.0, 1.58),
            (2_000_000.0, 1.46),
        ];
        let f = fit(&pts).expect("three sizes");
        assert!(
            !f.distinguishable_from_zero(),
            "a wobble around a constant was called a trend: slope {} +- {}",
            f.slope,
            f.stderr
        );
        assert_eq!(f.verdict(), "flat");
    }

    #[test]
    fn a_real_growth_is_not_hidden_by_the_error_bar() {
        // The other half: the criterion must be able to say "positive", or it is a rubber
        // stamp for the hypothesis rather than a test of it. PostgreSQL's recompute is the
        // series this must catch.
        let pts = vec![
            (20_000.0, 6.5),
            (200_000.0, 61.0),
            (2_000_000.0, 605.0),
            (20_000.0, 6.7),
            (200_000.0, 59.0),
            (2_000_000.0, 611.0),
        ];
        let f = fit(&pts).expect("three sizes");
        assert!(
            f.distinguishable_from_zero() && f.slope > 0.0,
            "a thirty-fold growth across two decades was called flat: {f:?}"
        );
        assert!(f.r2 > 0.99, "and the line explains the data: {f:?}");
    }

    #[test]
    fn two_sizes_are_refused_rather_than_fitted() {
        let pts = vec![(1.0, 1.0), (2.0, 2.0), (1.0, 1.1), (2.0, 2.1)];
        assert_eq!(fit(&pts), Err(NoFit::TooFewPoints(2)));
        // And the refusal says why, because "not enough data" without a reason invites
        // someone to add a fourth point at one of the two sizes and think it helped.
        let why = fit(&pts).unwrap_err().to_string();
        assert!(why.contains("three"), "{why}");
    }

    #[test]
    fn one_size_repeated_is_not_a_slope() {
        let pts = vec![(5.0, 1.0), (5.0, 2.0), (5.0, 3.0)];
        assert_eq!(fit(&pts), Err(NoFit::TooFewPoints(1)));
    }
}
