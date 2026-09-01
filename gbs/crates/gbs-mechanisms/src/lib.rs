//! **GBS mechanisms** — M2 through M7, as patterns of M1.
//!
//! None of these is a new kind of value movement. Every one produces balanced posting sets
//! and hands them to the kernel, which is what makes the claim of `ARCHITECTURE.md` §1
//! testable: a product built from these cannot reach the ledger except through M1.
//!
//! | | Mechanism | What it is |
//! |---|---|---|
//! | M2 | [`schedule`] | contingent, bitemporally-dated future posting sets, balanced by construction |
//! | M3 | [`hold`] | an affine reservation resolved exactly once — post, void, or expire |
//! | M4 | [`participation`] | an exact-rational split with a designated residual holder |
//! | M5 | [`lifecycle`] | a capability-gated state machine whose transitions are ledger events |
//! | M6 | [`signal`] | a position as a function of time, per currency or instrument |
//! | M7 | [`valuation`] | a deterministic, rate-indexed price settled as balanced postings |
//!
//! The design bet, stated so it can be wrong: twenty-odd product lines need seven
//! mechanisms. If an eighth is required, that is a design change to record rather than a
//! routine extension, and the coverage matrix in `ARCHITECTURE.md` §4 is where it shows up.

pub mod hold;
pub mod lifecycle;
pub mod participation;
pub mod schedule;
pub mod signal;
pub mod valuation;

pub use hold::{Hold, HoldId, Outcome};
pub use lifecycle::{Actor, Capability, Lifecycle, Rule, State};
pub use participation::{ParticipantSet, ParticipationError, Share};
pub use signal::{available_balance, Available, Reading, Signal};
pub use valuation::{accrue, apply_rate, cap_payoff, convert, floor_payoff, Priced, Rate, Rounding, ValuationError};
pub use schedule::{Calendar, Contingency, Observations, Occurrence, RollConvention, Schedule};

