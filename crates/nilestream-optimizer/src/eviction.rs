//! Eviction within `demand` mode: cost-, size- and delay-aware (thesis Appendix I.4).
//!
//! Plain LRU is wrong here for two independent reasons: entries differ in reconstruction
//! cost (upquery depth varies by circuit), and reconstruction takes time during which
//! further requests queue. The credit discipline below is the file-caching generalization
//! extended by the delayed-hit weighting; plain LRU is implemented alongside it purely as
//! the comparison point for the S7 experiment.

/// Credit assigned on admission: reconstruction cost, weighted by the delayed-hit ratio
/// and the contract multiplier.
pub fn admit_credit(reconstruction_cost: f64, z: f64, contract_multiplier: f64) -> f64 {
    reconstruction_cost * (1.0 + z) * contract_multiplier
}

/// Decrement credits under budget pressure, proportional to residency size.
pub fn charge_pressure(credit: f64, delta: f64, size_bytes: u64) -> f64 {
    credit - delta * size_bytes as f64
}

/// An entry is evictable once its credit is exhausted.
pub fn evictable(credit: f64) -> bool {
    credit <= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expensive_reconstruction_earns_more_credit() {
        assert!(admit_credit(10.0, 0.0, 1.0) > admit_credit(1.0, 0.0, 1.0));
    }

    #[test]
    fn queued_requests_during_reconstruction_raise_credit() {
        assert!(admit_credit(1.0, 5.0, 1.0) > admit_credit(1.0, 0.0, 1.0));
    }

    #[test]
    fn large_entries_lose_credit_faster_under_pressure() {
        let c = admit_credit(10.0, 0.0, 1.0);
        let big = charge_pressure(c, 0.001, 10_000);
        let small = charge_pressure(c, 0.001, 100);
        assert!(big < small);
    }

    #[test]
    fn exhausted_credit_is_evictable() {
        assert!(evictable(charge_pressure(1.0, 1.0, 10)));
        assert!(!evictable(1.0));
    }
}
