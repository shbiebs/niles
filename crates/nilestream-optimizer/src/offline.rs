//! The offline optimum used to *grade* the online algorithm (thesis Appendix I.8).
//!
//! Computed by dynamic programming over a recorded trace, with a Lagrangian relaxation of
//! the global budget coupling and a sweep of the multiplier to trace the budget-feasible
//! frontier. Where the relaxation gap is non-zero it is reported, so a measured ratio is
//! never presented as tighter than the bound supports.
//!
//! Stub: implemented in Phase 3 (thesis 8.4).
