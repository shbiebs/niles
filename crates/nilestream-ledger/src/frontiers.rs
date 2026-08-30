//! The visibility frontier: the single published number that makes consistency statable.
//!
//! Every read in the system is anchored at an epoch, and the frontier is the epoch a read
//! at the strictest rung is anchored *at*. It is one atomic counter, and its whole design
//! is contained in one rule:
//!
//! > **The frontier advances only after the epoch is on stable storage.**
//!
//! Publishing first would let a reader observe an epoch a crash then erases. In a cache
//! that is a stale read; in a ledger it is a transaction the customer watched succeed and
//! that no longer exists. The two are not the same kind of wrong, and the ordering here is
//! the only thing separating them.
//!
//! # Why one number is enough
//!
//! Strict serializability is famously **not** a local property: composing two
//! strictly-serializable objects does not give a strictly-serializable system. This design
//! does not escape that theorem — it avoids needing to compose. There is one frontier, and
//! every derived view anchors to it, so there are no two objects whose orders could
//! disagree. Composition is rescued by the shared visibility timeline, not by the effect
//! system and not by any property of the views themselves. That distinction matters,
//! because it says exactly what would break the guarantee: a second, independently
//! advancing frontier.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// The published frontier. Cheap to read from any thread; advanced by exactly one writer.
#[derive(Debug, Default)]
pub struct Frontier {
    /// The highest epoch that is durable and visible. `u64::MAX` is reserved for "no
    /// epochs yet", encoded as zero with a separate `started` flag would cost a second
    /// atomic, so epoch numbering starts at 1 and 0 means empty.
    visible: AtomicU64,
    /// The highest epoch that has been *sealed* but may not yet be durable. Never read by
    /// a query path; used by the sealer to know where it is, and by tests to observe the
    /// window between sealing and publishing.
    sealed: AtomicU64,
}

impl Frontier {
    pub fn new() -> Arc<Frontier> {
        Arc::new(Frontier::default())
    }

    /// The epoch a strict read anchors at. `0` means no epoch is visible yet.
    pub fn visible(&self) -> u64 {
        self.visible.load(Ordering::Acquire)
    }

    pub fn sealed(&self) -> u64 {
        self.sealed.load(Ordering::Acquire)
    }

    /// Mark an epoch sealed but **not yet visible**. Called before the fsync.
    pub fn seal(&self, e: u64) {
        self.sealed.fetch_max(e, Ordering::AcqRel);
    }

    /// Publish. **Callers must have fsynced first**, and the `Release` ordering is what
    /// makes that meaningful: it guarantees a reader who sees this value also sees every
    /// write that preceded it, so "durable then visible" holds across threads and not just
    /// in the writer's own program order.
    pub fn publish(&self, e: u64) {
        self.visible.fetch_max(e, Ordering::Release);
    }

    /// A snapshot: the frontier as of now.
    ///
    /// Snapshot isolation over an immutable base needs nothing more than this number.
    /// There is no read set to validate and no writer to conflict with, because the prefix
    /// a snapshot names cannot change — which is the cheapest thing about building a read
    /// path over an append-only history.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot { anchor: self.visible() }
    }

    /// The number of epochs sealed but not yet published: the durability window. A healthy
    /// system under `SyncPolicy::Always` holds this at zero or one.
    pub fn pending(&self) -> u64 {
        self.sealed().saturating_sub(self.visible())
    }
}

/// A read anchor. Immutable, copyable, and meaningful for as long as the base retains the
/// prefix it names — which, the base being fully retained, is forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Snapshot {
    pub anchor: u64,
}

impl Snapshot {
    pub fn at(anchor: u64) -> Snapshot {
        Snapshot { anchor }
    }
    /// Whether an epoch is inside this snapshot.
    pub fn includes(&self, epoch: u64) -> bool {
        epoch <= self.anchor
    }
    /// Monotonicity, per session: a session's anchors never decrease. This is rung 1, and
    /// it is one `max`, which is the cheapest rung on the ladder for a reason.
    pub fn advance_to(self, other: Snapshot) -> Snapshot {
        Snapshot { anchor: self.anchor.max(other.anchor) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::thread;

    #[test]
    fn an_epoch_is_sealed_before_it_is_visible() {
        let f = Frontier::new();
        f.seal(7);
        assert_eq!(f.sealed(), 7);
        assert_eq!(f.visible(), 0, "sealing must not publish");
        assert_eq!(f.pending(), 7, "the durability window is observable");
        f.publish(7);
        assert_eq!(f.visible(), 7);
        assert_eq!(f.pending(), 0);
    }

    #[test]
    fn the_frontier_never_goes_backwards() {
        // A frontier that could retreat would let a reader observe an epoch and then be
        // told it does not exist, which is the failure the whole ordering rule prevents.
        let f = Frontier::new();
        f.publish(10);
        f.publish(4);
        assert_eq!(f.visible(), 10);
    }

    #[test]
    fn a_snapshot_names_a_prefix_that_cannot_change() {
        let f = Frontier::new();
        f.publish(5);
        let s = f.snapshot();
        f.publish(50);
        assert_eq!(s.anchor, 5, "a taken snapshot is unaffected by later commits");
        assert!(s.includes(5) && !s.includes(6));
    }

    #[test]
    fn session_anchors_never_decrease() {
        let a = Snapshot::at(9);
        assert_eq!(a.advance_to(Snapshot::at(3)).anchor, 9);
        assert_eq!(a.advance_to(Snapshot::at(20)).anchor, 20);
    }

    #[test]
    fn a_reader_never_observes_an_epoch_that_was_not_published() {
        // The concurrency property, tested with a real thread rather than argued for. The
        // writer publishes only after "durability" (here, setting a flag); the reader must
        // never see a frontier whose corresponding durability flag is unset.
        let f = Frontier::new();
        let durable: Arc<Vec<AtomicBool>> = Arc::new((0..200).map(|_| AtomicBool::new(false)).collect());
        let (fw, dw) = (Arc::clone(&f), Arc::clone(&durable));

        let writer = thread::spawn(move || {
            for e in 1..200u64 {
                fw.seal(e);
                // "fsync"
                dw[e as usize].store(true, Ordering::Release);
                fw.publish(e);
            }
        });

        let (fr, dr) = (Arc::clone(&f), Arc::clone(&durable));
        let reader = thread::spawn(move || {
            let mut observed = 0u64;
            let mut last = 0u64;
            for _ in 0..200_000 {
                let v = fr.visible();
                assert!(v >= last, "the frontier went backwards: {last} -> {v}");
                last = v;
                if v > 0 {
                    assert!(
                        dr[v as usize].load(Ordering::Acquire),
                        "epoch {v} was visible before it was durable"
                    );
                    observed = observed.max(v);
                }
            }
            observed
        });

        writer.join().unwrap();
        let observed = reader.join().unwrap();
        assert!(observed > 0, "the reader saw nothing at all; the test proved nothing");
    }
}
