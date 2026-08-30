//! **Nilestream-Core** — the partial-state read-model runtime (thesis §6.2, Appendix D.3).
//!
//! Home of the REV: the *reconstructible epoch-anchored view*, this thesis's central
//! abstraction, as a running object rather than a definition.
//!
//! The crate is deliberately small and has exactly one dependency, `niles-ir`. It executes
//! circuits; it does not build them, store them, or decide what they should be. That
//! separation is what lets the same runtime be driven by the compiler, by a hand-built
//! plan in a test, and by a replay harness, and be measured to produce identical counted
//! work in each case.
//!
//! | Module | What it is |
//! |---|---|
//! | [`absence`] | the absence lattice, and honest absence |
//! | [`rev`] | the runtime: REVs, anchored upqueries, eviction, counted work |
//! | [`ladder`] | the consistency ladder, and what each rung costs |
//! | [`anchor_index`] | per-key indices and checkpoints over an epoch-ordered base |
//! | [`eviction`] | the policies the adaptive one is measured against |
//! | [`upquery`] | reconstruction, re-exported from the IR's path derivation |

pub mod absence;
pub mod anchor_index;
pub mod distributed;
pub mod eviction;
pub mod ladder;
pub mod rev;
pub mod upquery;

pub use absence::{Epoch, Slot};
pub use rev::{Anchored, Base, Key, Policy, Rev, Runtime, Stats, Unsupported, Value};
