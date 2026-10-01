//! **A power-law fitter, after Clauset, Shalizi and Newman** (SIAM Review 51(4), 2009,
//! "Power-law distributions in empirical data"), for the universe's public calibration
//! (cycle 16, C16-11; `docs/universe.md` §5).
//!
//! Three steps, each the paper's:
//!
//! 1. **The exponent by maximum likelihood**, for a given lower bound `xmin`:
//!    α̂ = 1 + n · [Σ ln(xᵢ / xmin)]⁻¹ over the tail xᵢ ≥ xmin (the paper's eq. 3.1, continuous).
//!    For discrete data the paper's approximation replaces `xmin` with `xmin − ½` in the
//!    denominator (eq. 3.7), accurate for `xmin` of about 6 and above; [`Kind::Discrete`] uses it.
//! 2. **`xmin` by Kolmogorov–Smirnov**: every distinct observed value is tried as `xmin`, and the
//!    one whose fitted tail is closest to the empirical tail in KS distance is kept.
//! 3. **A confidence interval by bootstrap**: the data are resampled with replacement, steps 1
//!    and 2 are rerun on each resample, and the 2.5th and 97.5th percentiles of α̂ are reported.
//!    200 resamples by default, as the work order fixes.
//!
//! What it does not do: the paper's goodness-of-fit p-value (a second, semiparametric bootstrap)
//! and its likelihood-ratio comparisons with alternatives (lognormal, exponential). A fitted
//! exponent here says *if* the tail is a power law, this is its exponent; it does not say the
//! tail is one. `results/E29-skew.md` must say the same.
//!
//! No dependencies, and no floating-point randomness: the resampler is SplitMix64 from a seed.

/// Continuous data, or integer data (counts, degrees) under the paper's discrete approximation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Continuous,
    Discrete,
}

/// A fitted tail.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub alpha: f64,
    pub xmin: f64,
    /// Observations at or above `xmin`.
    pub n_tail: usize,
    /// The KS distance at the chosen `xmin`.
    pub ks: f64,
    /// The paper's asymptotic standard error, (α̂ − 1)/√n_tail.
    pub stderr: f64,
}

/// A fit with its bootstrap interval.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Estimate {
    pub fit: Fit,
    pub ci_low: f64,
    pub ci_high: f64,
    pub resamples: usize,
}

/// Why there is no fit.
#[derive(Debug, Clone, PartialEq)]
pub enum FitError {
    /// Fewer than `MIN_TAIL` positive observations: no exponent is estimable.
    TooFew { n: usize },
    /// A value that is not finite, or not positive, which a power law cannot hold.
    BadValue { value: f64 },
}

impl std::fmt::Display for FitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FitError::TooFew { n } => write!(
                f,
                "{n} observations; at least {MIN_TAIL} are needed in a tail to fit an exponent"
            ),
            FitError::BadValue { value } => {
                write!(f, "{value} is not a finite positive number")
            }
        }
    }
}

impl std::error::Error for FitError {}

/// The smallest tail a candidate `xmin` may leave. Below it the KS distance is noise and the
/// search would pick an `xmin` that fits three points perfectly.
pub const MIN_TAIL: usize = 10;

fn alpha_for(tail: &[f64], xmin: f64, kind: Kind) -> f64 {
    let shift = match kind {
        Kind::Continuous => xmin,
        Kind::Discrete => xmin - 0.5,
    };
    let s: f64 = tail.iter().map(|x| (x / shift).ln()).sum();
    1.0 + tail.len() as f64 / s
}

/// KS distance between the empirical CDF of `tail` (sorted ascending) and the fitted
/// continuous power-law CDF 1 − (x/xmin)^(1−α).
fn ks_for(tail: &[f64], xmin: f64, alpha: f64) -> f64 {
    let n = tail.len() as f64;
    let mut d: f64 = 0.0;
    for (i, &x) in tail.iter().enumerate() {
        let model = 1.0 - (x / xmin).powf(1.0 - alpha);
        let below = i as f64 / n;
        let at = (i + 1) as f64 / n;
        d = d.max((model - below).abs()).max((model - at).abs());
    }
    d
}

/// The most `xmin` candidates tried. The paper tries every distinct value; with thousands of
/// distinct values and 200 bootstrap resamples that is billions of operations, so the search
/// takes at most this many, spread evenly over the distinct values in order. Stated as a
/// departure from the paper rather than hidden: a true `xmin` between two candidates is
/// approximated by the nearer one.
pub const MAX_CANDIDATES: usize = 100;

/// Steps 1 and 2: the exponent and `xmin`, on data that are already validated and sorted.
fn fit_sorted(sorted: &[f64], kind: Kind) -> Result<Fit, FitError> {
    // The first index of each distinct value whose tail is long enough.
    let mut starts = Vec::new();
    let mut i = 0;
    while i < sorted.len() && sorted.len() - i >= MIN_TAIL {
        starts.push(i);
        let mut j = i + 1;
        while j < sorted.len() && sorted[j] == sorted[i] {
            j += 1;
        }
        i = j;
    }
    if starts.len() > MAX_CANDIDATES {
        let step = starts.len() as f64 / MAX_CANDIDATES as f64;
        starts = (0..MAX_CANDIDATES)
            .map(|k| starts[(k as f64 * step) as usize])
            .collect();
    }
    let mut best: Option<Fit> = None;
    for i in starts {
        let xmin = sorted[i];
        let tail = &sorted[i..];
        // An xmin at or below ½ makes the discrete shift non-positive.
        if kind == Kind::Discrete && xmin <= 0.5 {
            continue;
        }
        let alpha = alpha_for(tail, xmin, kind);
        if alpha.is_finite() && alpha > 1.0 {
            let ks = ks_for(tail, xmin, alpha);
            if best.is_none_or(|b| ks < b.ks) {
                best = Some(Fit {
                    alpha,
                    xmin,
                    n_tail: tail.len(),
                    ks,
                    stderr: (alpha - 1.0) / (tail.len() as f64).sqrt(),
                });
            }
        }
    }
    best.ok_or(FitError::TooFew { n: sorted.len() })
}

fn validated(data: &[f64]) -> Result<Vec<f64>, FitError> {
    let mut v = Vec::with_capacity(data.len());
    for &x in data {
        if !x.is_finite() || x <= 0.0 {
            return Err(FitError::BadValue { value: x });
        }
        v.push(x);
    }
    v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    Ok(v)
}

/// Steps 1 and 2.
pub fn fit(data: &[f64], kind: Kind) -> Result<Fit, FitError> {
    fit_sorted(&validated(data)?, kind)
}

/// SplitMix64.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in (0, 1]: never zero, so it can be inverted through a power law.
    pub fn unit_open(&mut self) -> f64 {
        ((self.next_u64() >> 11) + 1) as f64 / (1u64 << 53) as f64
    }
}

/// Steps 1–3: the fit and its bootstrap interval from `resamples` resamples.
pub fn estimate(
    data: &[f64],
    kind: Kind,
    resamples: usize,
    seed: u64,
) -> Result<Estimate, FitError> {
    let sorted = validated(data)?;
    let point = fit_sorted(&sorted, kind)?;
    let mut rng = Rng::new(seed);
    let mut alphas = Vec::with_capacity(resamples);
    let mut buf = vec![0.0; sorted.len()];
    for _ in 0..resamples {
        for b in buf.iter_mut() {
            *b = sorted[(rng.next_u64() % sorted.len() as u64) as usize];
        }
        buf.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        // A resample whose tail is too short has no exponent; it is skipped, and the count of
        // resamples used is reported rather than the count asked for.
        if let Ok(f) = fit_sorted(&buf, kind) {
            alphas.push(f.alpha);
        }
    }
    alphas.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let pct = |p: f64| -> f64 {
        if alphas.is_empty() {
            return f64::NAN;
        }
        let i = ((alphas.len() - 1) as f64 * p).round() as usize;
        alphas[i]
    };
    Ok(Estimate {
        fit: point,
        ci_low: pct(0.025),
        ci_high: pct(0.975),
        resamples: alphas.len(),
    })
}

/// `n` draws from a continuous power law with exponent `alpha` above `xmin`, by inverting its
/// CDF. For the tests, and for anyone checking the fitter against a known answer.
pub fn sample(n: usize, alpha: f64, xmin: f64, seed: u64) -> Vec<f64> {
    let mut rng = Rng::new(seed);
    (0..n)
        .map(|_| xmin * rng.unit_open().powf(-1.0 / (alpha - 1.0)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fitter recovers a known exponent: on 1,000 draws with α = 2.5 above xmin = 1, the
    /// estimate is within three of its standard errors of the truth, and the bootstrap
    /// interval contains it.
    #[test]
    fn a_known_exponent_is_recovered() {
        for (alpha, seed) in [(2.5, 1u64), (1.8, 7), (3.2, 42)] {
            let x = sample(1_000, alpha, 1.0, seed);
            let e = estimate(&x, Kind::Continuous, 200, seed).unwrap();
            assert!(
                (e.fit.alpha - alpha).abs() < 3.0 * e.fit.stderr.max(0.02),
                "α = {alpha}: fitted {} ± {}",
                e.fit.alpha,
                e.fit.stderr
            );
            assert!(
                e.ci_low <= alpha && alpha <= e.ci_high,
                "α = {alpha} outside [{}, {}]",
                e.ci_low,
                e.ci_high
            );
            assert!(e.resamples > 190, "only {} resamples fitted", e.resamples);
        }
    }

    /// The KS search finds the bend: a body that is not a power law below 10, and a power-law
    /// tail above it. An xmin far below 10 would fit the body and be wrong. Within the candidate
    /// grid's resolution: with 100 candidates over 4,000 values, a candidate falls every ~40.
    #[test]
    fn xmin_is_found_where_the_power_law_starts() {
        let mut x = sample(2_000, 2.5, 10.0, 3);
        let mut rng = Rng::new(9);
        x.extend((0..2_000).map(|_| 1.0 + 9.0 * rng.unit_open()));
        let f = fit(&x, Kind::Continuous).unwrap();
        assert!(f.xmin >= 8.0, "xmin {} is inside the uniform body", f.xmin);
        assert!((f.alpha - 2.5).abs() < 0.2, "α {}", f.alpha);
    }

    #[test]
    fn integer_data_use_the_discrete_approximation() {
        // Rounded continuous draws above 20, where the paper says the approximation holds.
        let x: Vec<f64> = sample(2_000, 2.2, 20.0, 11)
            .into_iter()
            .map(f64::round)
            .collect();
        let d = fit(&x, Kind::Discrete).unwrap();
        assert!((d.alpha - 2.2).abs() < 0.15, "discrete α {}", d.alpha);
    }

    #[test]
    fn what_cannot_be_fitted_is_refused() {
        assert!(matches!(
            fit(&[1.0, 2.0, 3.0], Kind::Continuous),
            Err(FitError::TooFew { .. })
        ));
        assert!(matches!(
            fit(&[1.0, 0.0, 3.0], Kind::Continuous),
            Err(FitError::BadValue { .. })
        ));
        assert!(matches!(
            fit(&[1.0, f64::NAN], Kind::Continuous),
            Err(FitError::BadValue { .. })
        ));
    }

    #[test]
    fn the_bootstrap_is_a_pure_function_of_its_seed() {
        let x = sample(1_000, 2.0, 1.0, 5);
        assert_eq!(
            estimate(&x, Kind::Continuous, 50, 99).unwrap(),
            estimate(&x, Kind::Continuous, 50, 99).unwrap()
        );
    }
}
