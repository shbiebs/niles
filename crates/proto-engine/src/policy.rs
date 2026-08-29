//! Eviction policies.
//!
//! Three are implemented so the comparison in E6 has a proper baseline set rather than
//! only the system compared against itself:
//!   * `Lru` — the classical recency policy, k-competitive for page faults;
//!   * `Random` — the policy the original partial-state work used ("eviction ... is
//!     randomized"), included so the improvement is measured against what exists;
//!   * `CostAware` — a credit discipline weighted by reconstruction cost *and* by the
//!     delayed-hit factor, i.e. ranking by expected aggregate delay rather than by reuse
//!     probability alone.

use std::collections::HashMap;

use crate::view::{policy_meta::Meta, Slot};
use crate::{Acct, Cur};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    Lru,
    Random,
    CostAware,
}

impl EvictionPolicy {
    pub fn choose_victim(
        &self,
        slots: &HashMap<(Acct, Cur), Slot>,
        meta: &HashMap<(Acct, Cur), Meta>,
        clock: u64,
    ) -> Option<(Acct, Cur)> {
        let present: Vec<(Acct, Cur)> = slots
            .iter()
            .filter(|(_, s)| matches!(s, Slot::Present(_, _)))
            .map(|(k, _)| *k)
            .collect();
        if present.is_empty() {
            return None;
        }
        match self {
            EvictionPolicy::Lru => present
                .into_iter()
                .min_by_key(|k| meta.get(k).map(|m| m.last_used).unwrap_or(0)),
            EvictionPolicy::Random => {
                // Deterministic pseudo-random selection driven by the logical clock, so
                // runs remain reproducible from a seed.
                let i = (clock.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
                    >> 33) as usize
                    % present.len();
                Some(present[i])
            }
            EvictionPolicy::CostAware => present.into_iter().min_by(|a, b| {
                let ca = meta.get(a).map(|m| m.credit).unwrap_or(0.0);
                let cb = meta.get(b).map(|m| m.credit).unwrap_or(0.0);
                ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
            }),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EvictionPolicy::Lru => "lru",
            EvictionPolicy::Random => "random",
            EvictionPolicy::CostAware => "cost_aware",
        }
    }
}
