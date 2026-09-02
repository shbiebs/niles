//! Mode selection: rent-or-buy with a contract multiplier (thesis Appendix I.3).

use crate::estimators::RangeStats;

/// A point of the materialization mode lattice (thesis Def. 4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Absent,
    Demand,
    Full,
    Spilled,
    Tiered,
}

/// A proposed transition, carrying the evidence that produced it.
///
/// The evidence is not decoration: an adaptive component that cannot be interrogated is
/// operationally unacceptable whatever its average-case behaviour (thesis 11.2).
#[derive(Debug, Clone)]
pub struct ModeDecision {
    pub from: Mode,
    pub to: Mode,
    pub reason: &'static str,
    pub cumulative_reconstruction_spend: f64,
    pub buy_cost: f64,
    pub delayed_hit_ratio: f64,
}

/// The break-even rule of thesis I.3.
///
/// Deterministically 2-competitive against the offline optimum for the two-mode decision
/// under stationary costs; a randomized variant approaches e/(e-1) and is optimal for the
/// class. Neither guarantee survives non-stationarity, and neither extends as stated to
/// the full five-mode lattice — see thesis I.6.
pub fn should_buy(spend: f64, buy_cost: f64) -> bool {
    spend >= buy_cost
}

/// Cost of materializing and maintaining a range over the planning horizon.
pub fn buy_cost(stats: &RangeStats, horizon_epochs: f64, residency_price: f64) -> f64 {
    stats.maintenance_cost * horizon_epochs + stats.residency_bytes as f64 * residency_price
}

/// Reconstruction spend charged on a miss, weighted by the delayed-hit ratio and the
/// contract multiplier. Ignoring the delayed-hit term is provably the wrong policy when
/// reconstruction takes time (thesis 4.6(e), Appendix I.4).
pub fn miss_charge(stats: &RangeStats, contract_multiplier: f64) -> f64 {
    stats.reconstruction_cost * (1.0 + stats.delayed_hit_ratio()) * contract_multiplier
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::estimators::RangeStats;

    fn stats() -> RangeStats {
        RangeStats {
            read_rate: 10.0,
            write_rate: 1.0,
            reconstruction_cost: 5.0,
            reconstruction_latency: 0.2,
            maintenance_cost: 1.0,
            residency_bytes: 1_000,
        }
    }

    #[test]
    fn break_even_triggers_only_after_spend_exceeds_buy_cost() {
        let s = stats();
        let bc = buy_cost(&s, 10.0, 0.001);
        assert!(!should_buy(bc - 1.0, bc));
        assert!(should_buy(bc, bc));
    }

    #[test]
    fn delayed_hits_make_misses_more_expensive() {
        let mut s = stats();
        let cheap = miss_charge(&s, 1.0);
        s.reconstruction_latency = 2.0; // slower reconstruction, same arrival rate
        let dear = miss_charge(&s, 1.0);
        assert!(
            dear > cheap,
            "a slower reconstruction must cost more, not the same"
        );
    }

    #[test]
    fn strict_contracts_cost_more_than_lax_ones() {
        let s = stats();
        assert!(miss_charge(&s, 4.0) > miss_charge(&s, 1.0));
    }
}
