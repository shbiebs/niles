//! The cost model used to compare materialization strategies.
//!
//! Costs are expressed in **counted work units**, not seconds: resident entry-epochs for
//! memory, per-key delta applications for maintenance, and base rows read for
//! reconstruction. A ratio computed in these units is a property of the algorithms and the
//! workload, so it is reproducible on any machine — which is what a phase diagram needs to
//! be useful to a reader who does not have the author's hardware.
//!
//! The weights are the model's parameters, and the phase diagram is reported *as a function
//! of them* rather than at one arbitrary setting, so that a reader whose reconstruction is
//! cheaper or whose memory is dearer can locate their own operating point.

#[derive(Debug, Clone, Copy)]
pub struct CostModel {
    /// Cost per resident entry per epoch.
    pub memory: f64,
    /// Cost per delta applied to a resident key.
    pub maintenance: f64,
    /// Cost per base row read during reconstruction.
    pub reconstruction: f64,
}

impl Default for CostModel {
    fn default() -> Self {
        // Unit weights: one resident entry-epoch, one delta application, and one base-row
        // read each cost 1. Every reported ratio states the weights it used.
        Self { memory: 1.0, maintenance: 1.0, reconstruction: 1.0 }
    }
}

impl CostModel {
    pub fn new(memory: f64, maintenance: f64, reconstruction: f64) -> Self {
        Self { memory, maintenance, reconstruction }
    }
}
