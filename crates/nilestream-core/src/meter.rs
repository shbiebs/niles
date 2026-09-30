//! **How many bytes a derived state holds, counted rather than estimated** (E27b, C15-02, E2).
//!
//! E27 compared the memory of two things that were not alike: Nilestream's process PSS, which
//! includes its in-memory base, against a PostgreSQL cluster whose base is on disk. E27b's
//! metric is the derived state alone — a view's resident entries, their metadata, its
//! in-flight reconstructions, and the ledger's per-key checkpoints — and the question this
//! module answers is how many bytes that is.
//!
//! # How it counts, and why the counter is not here
//!
//! A figure is taken by **cloning** the structures and asking the allocator how many bytes
//! the clone holds live, then dropping it. A clone of a `BTreeMap` reproduces the original's
//! node structure exactly, so the count is exact for the trees; a cloned `Vec` is allocated
//! at its length rather than its capacity, so the figure **excludes spare vector capacity**
//! and is a lower bound by that slack. The results say so beside every number.
//!
//! The counting itself needs a `#[global_allocator]`, and a `GlobalAlloc` has no safe
//! implementation. Thesis §9.10 claims the workspace contains no `unsafe` block, and a drift
//! test holds it to that, so the allocator lives in `tools/memprobe` — outside the workspace,
//! in the one file that test allows — and reaches this crate through [`install`], a plain
//! function pointer. The metered daemon (`nilestreamd-metered`, built from the same
//! `main.rs` by `tools/memprobe`) installs one; the shipped `nilestreamd` does not, and
//! reports the figure as **absent** rather than zero, because a zero would be a claim.
//!
//! # What a meter must do
//!
//! Given a closure that builds the image, a meter counts the bytes allocated **on the calling
//! thread** while the closure runs, net of those freed, holds the image while it reads the
//! counter, and drops it afterwards. Thread-local, so a connection thread allocating during
//! the clone is not counted into it — which a process-wide counter would do silently, and by
//! a different amount each run.

use std::any::Any;
use std::sync::OnceLock;

/// A meter: builds the image, returns the bytes it held live, then drops it.
pub type Meter = fn(&mut dyn FnMut() -> Box<dyn Any>) -> u64;

static METER: OnceLock<Meter> = OnceLock::new();

/// Install the process's meter. Once: a second call is refused and returns `false`, so two
/// instruments cannot disagree about which one produced a figure.
pub fn install(m: Meter) -> bool {
    METER.set(m).is_ok()
}

/// Whether this process can count held bytes at all.
pub fn installed() -> bool {
    METER.get().is_some()
}

/// The bytes the image built by `make` holds, or `None` when no meter is installed.
///
/// `None` is not zero. A shipped binary without the instrument reports the figure as absent,
/// and a caller that printed `0` for it would be reporting a view that holds nothing.
///
/// **The image's own box is not counted.** The meter needs the image boxed, and that box
/// holds the structures' inline headers — a `BTreeMap` is three words before it has a single
/// entry — which are the same size for a view that has read nothing as for one that has read
/// everything. `Box::new` allocates exactly `size_of::<T>()`, so it is subtracted exactly,
/// and an empty view reads zero. (It read 96 the first time, which is four empty maps.)
pub fn held_bytes<T: Any>(mut make: impl FnMut() -> T) -> Option<u64> {
    let m = METER.get()?;
    let held = m(&mut || Box::new(make()) as Box<dyn Any>);
    Some(held.saturating_sub(std::mem::size_of::<T>() as u64))
}
