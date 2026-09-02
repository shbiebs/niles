//! Per-range estimators (thesis Appendix I.2).
//!
//! The decisive parameter is Z, the delayed-hit ratio: reconstruction latency times
//! arrival rate. It is *measured*, never assumed — a phase-diagram point reported
//! without Z is uninterpretable (thesis 5.5).

/// Exponentially weighted moving average.
#[derive(Debug, Clone, Copy)]
pub struct Ewma {
    value: f64,
    alpha: f64,
    initialized: bool,
}

impl Ewma {
    pub fn new(alpha: f64) -> Self {
        Self {
            value: 0.0,
            alpha,
            initialized: false,
        }
    }

    pub fn update(&mut self, sample: f64) -> f64 {
        if self.initialized {
            self.value = self.alpha * sample + (1.0 - self.alpha) * self.value;
        } else {
            self.value = sample;
            self.initialized = true;
        }
        self.value
    }

    pub fn get(&self) -> f64 {
        self.value
    }
}

/// Statistics for one (view, key range), as of the last review.
#[derive(Debug, Clone, Copy)]
pub struct RangeStats {
    /// Read arrival rate (reads per unit time).
    pub read_rate: f64,
    /// Write rate touching this range.
    pub write_rate: f64,
    /// Measured reconstruction work per upquery.
    pub reconstruction_cost: f64,
    /// Measured reconstruction *service time* — the term classical caching omits.
    pub reconstruction_latency: f64,
    /// Per-epoch maintenance cost if materialized.
    pub maintenance_cost: f64,
    /// Resident size.
    pub residency_bytes: u64,
}

impl RangeStats {
    /// Z = reconstruction latency x arrival rate (thesis Appendix I.2).
    ///
    /// Under delayed hits the objective is aggregate delay rather than miss count, and
    /// policies optimal for hit rate are not optimal for latency.
    pub fn delayed_hit_ratio(&self) -> f64 {
        self.reconstruction_latency * self.read_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ewma_starts_at_first_sample_not_at_zero() {
        let mut e = Ewma::new(0.3);
        assert_eq!(e.update(10.0), 10.0);
        assert!(e.get() > 0.0);
    }

    #[test]
    fn ewma_tracks_toward_new_level() {
        let mut e = Ewma::new(0.5);
        e.update(0.0);
        for _ in 0..20 {
            e.update(100.0);
        }
        assert!(e.get() > 99.0);
    }

    #[test]
    fn z_is_latency_times_arrival_rate() {
        let s = RangeStats {
            read_rate: 50.0,
            write_rate: 1.0,
            reconstruction_cost: 1.0,
            reconstruction_latency: 0.02,
            maintenance_cost: 1.0,
            residency_bytes: 0,
        };
        assert!((s.delayed_hit_ratio() - 1.0).abs() < 1e-9);
    }
}
