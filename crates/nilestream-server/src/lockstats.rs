//! **How long the engine mutex is held, and how long connections wait for it.**
//!
//! `daemon::serve` takes one `Arc<Mutex<RevEngine>>` around the whole of `Session::handle` —
//! parse, plan, execute, frame — and on a write that critical section contains the `fsync`,
//! because `RevEngine::append` blocks in the sequencer until the epoch is on stable storage.
//! So every other connection, reads included, waits behind one disk barrier.
//!
//! Nothing measured it. The published contract numbers are single-connection, where a lock
//! nobody contends for costs nothing, and the one experiment that varied connections (E19)
//! was read as a scaling result rather than as a lock diagnosis. This module is the
//! instrument that makes the diagnosis a number instead of an argument.
//!
//! # Why a bucketed histogram rather than samples
//!
//! Recording every hold would allocate on the hot path and make the measurement change what
//! it measures. Buckets are power-of-two microseconds, updated with three relaxed atomic
//! operations, so the cost is a few nanoseconds and does not depend on how many samples have
//! already been taken. The reported percentiles are therefore **bucket boundaries, not
//! interpolated values** — `p99 ≤ 2048µs` is an honest statement and `p99 = 1873µs` would
//! not be.
//!
//! Relaxed ordering throughout: these are counters, no other memory is published through
//! them, and a percentile that races is a percentile off by one sample.

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

/// 1µs, 2µs, 4µs … about 8.4 seconds. Anything slower lands in the last bucket, which is
/// itself the finding.
const BUCKETS: usize = 24;

/// Waiting to acquire, and holding once acquired, are different questions: a long hold with
/// no wait is a slow query nobody is queued behind, and a short hold with a long wait is
/// contention. Reporting one number for both would hide which.
pub struct LockStats {
    wait: [AtomicU64; BUCKETS],
    hold: [AtomicU64; BUCKETS],
    acquisitions: AtomicU64,
    wait_ns: AtomicU64,
    hold_ns: AtomicU64,
    max_wait_ns: AtomicU64,
    max_hold_ns: AtomicU64,
}

#[allow(clippy::declare_interior_mutable_const)]
const ZERO: AtomicU64 = AtomicU64::new(0);

impl LockStats {
    pub const fn new() -> LockStats {
        LockStats {
            wait: [ZERO; BUCKETS],
            hold: [ZERO; BUCKETS],
            acquisitions: ZERO,
            wait_ns: ZERO,
            hold_ns: ZERO,
            max_wait_ns: ZERO,
            max_hold_ns: ZERO,
        }
    }

    fn bucket(ns: u64) -> usize {
        let us = ns / 1_000;
        if us == 0 {
            return 0;
        }
        // floor(log2(us)) + 1, clamped.
        ((64 - us.leading_zeros()) as usize).min(BUCKETS - 1)
    }

    fn bump(slot: &AtomicU64, max: &AtomicU64, hist: &[AtomicU64; BUCKETS], ns: u64) {
        slot.fetch_add(ns, Relaxed);
        hist[Self::bucket(ns)].fetch_add(1, Relaxed);
        max.fetch_max(ns, Relaxed);
    }

    pub fn record(&self, wait_ns: u64, hold_ns: u64) {
        self.acquisitions.fetch_add(1, Relaxed);
        Self::bump(&self.wait_ns, &self.max_wait_ns, &self.wait, wait_ns);
        Self::bump(&self.hold_ns, &self.max_hold_ns, &self.hold, hold_ns);
    }

    /// The upper bound of the bucket the `q`th quantile falls in, in microseconds.
    fn quantile(hist: &[AtomicU64; BUCKETS], q: f64) -> u64 {
        let total: u64 = hist.iter().map(|b| b.load(Relaxed)).sum();
        if total == 0 {
            return 0;
        }
        let want = (total as f64 * q).ceil() as u64;
        let mut seen = 0u64;
        for (i, b) in hist.iter().enumerate() {
            seen += b.load(Relaxed);
            if seen >= want {
                return if i == 0 { 1 } else { 1u64 << (i - 1) };
            }
        }
        1u64 << (BUCKETS - 2)
    }

    /// `(acquisitions, wait_p50µs, wait_p99µs, wait_maxµs, hold_p50µs, hold_p99µs,
    /// hold_maxµs, hold_totalµs)`.
    ///
    /// `hold_total` is there so a reader can compute occupancy — the fraction of a run's
    /// wall clock during which the engine was locked — which is the number that says whether
    /// the mutex is the ceiling or merely present.
    pub fn snapshot(&self) -> (u64, u64, u64, u64, u64, u64, u64, u64) {
        (
            self.acquisitions.load(Relaxed),
            Self::quantile(&self.wait, 0.50),
            Self::quantile(&self.wait, 0.99),
            self.max_wait_ns.load(Relaxed) / 1_000,
            Self::quantile(&self.hold, 0.50),
            Self::quantile(&self.hold, 0.99),
            self.max_hold_ns.load(Relaxed) / 1_000,
            self.hold_ns.load(Relaxed) / 1_000,
        )
    }
}

impl Default for LockStats {
    fn default() -> Self {
        LockStats::new()
    }
}

/// One per process, because there is one base per process.
///
/// **What this counts changed with the reader-writer split, and the name did not.** It used
/// to be the daemon's single engine mutex, taken once around the whole of `Session::handle`.
/// It is now the lock over the *base* — taken shared by every read and exclusively by every
/// append — which is the same question asked of the structure that replaced it: how long is
/// the base held, and how long does a connection wait for it. A shared acquisition that
/// waits for nothing records a zero wait, which is the shape a working reader-writer lock
/// produces and the shape a mutex cannot.
pub static ENGINE_LOCK: LockStats = LockStats::new();

/// Acquire, timing both the wait and the hold, and record on drop.
///
/// A guard rather than two calls, so a path that returns early — a refused statement, a
/// closed socket — cannot leave the hold uncounted and make contention look smaller than it
/// is.
pub struct Timed<'a, T> {
    guard: Option<std::sync::MutexGuard<'a, T>>,
    since: Instant,
    waited_ns: u64,
    stats: &'static LockStats,
}

impl<'a, T> Timed<'a, T> {
    pub fn acquire(m: &'a std::sync::Mutex<T>, stats: &'static LockStats) -> Timed<'a, T> {
        let asked = Instant::now();
        // A poisoned engine mutex means a panic inside a critical section; the daemon's
        // existing behaviour is to unwrap, and changing that is not this instrument's job.
        let guard = m.lock().unwrap();
        let waited = asked.elapsed();
        Timed {
            guard: Some(guard),
            since: Instant::now(),
            waited_ns: waited.as_nanos() as u64,
            stats,
        }
    }
}

impl<T> std::ops::Deref for Timed<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard.as_ref().expect("held")
    }
}

impl<T> std::ops::DerefMut for Timed<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.as_mut().expect("held")
    }
}

impl<T> Drop for Timed<'_, T> {
    fn drop(&mut self) {
        let held = self.since.elapsed().as_nanos() as u64;
        // Release before recording, so the counter update is not itself inside the section
        // it measures.
        drop(self.guard.take());
        self.stats.record(self.waited_ns, held);
    }
}

/// A shared acquisition of a reader-writer lock, timed like [`Timed`].
///
/// Separate from `Timed` rather than generic over the guard, because the two guards have no
/// common trait in `std` and a hand-rolled one would be more machinery than the two structs.
pub struct TimedRead<'a, T> {
    guard: Option<std::sync::RwLockReadGuard<'a, T>>,
    since: Instant,
    waited_ns: u64,
    stats: &'static LockStats,
}

impl<'a, T> TimedRead<'a, T> {
    pub fn acquire(l: &'a std::sync::RwLock<T>, stats: &'static LockStats) -> TimedRead<'a, T> {
        let asked = Instant::now();
        let guard = l.read().expect("the base lock is not poisoned");
        let waited = asked.elapsed();
        TimedRead {
            guard: Some(guard),
            since: Instant::now(),
            waited_ns: waited.as_nanos() as u64,
            stats,
        }
    }
}

impl<T> std::ops::Deref for TimedRead<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard.as_ref().expect("held")
    }
}

impl<T> Drop for TimedRead<'_, T> {
    fn drop(&mut self) {
        let held = self.since.elapsed().as_nanos() as u64;
        drop(self.guard.take());
        self.stats.record(self.waited_ns, held);
    }
}

/// An exclusive acquisition of a reader-writer lock, timed like [`Timed`].
pub struct TimedWrite<'a, T> {
    guard: Option<std::sync::RwLockWriteGuard<'a, T>>,
    since: Instant,
    waited_ns: u64,
    stats: &'static LockStats,
}

impl<'a, T> TimedWrite<'a, T> {
    pub fn acquire(l: &'a std::sync::RwLock<T>, stats: &'static LockStats) -> TimedWrite<'a, T> {
        let asked = Instant::now();
        let guard = l.write().expect("the base lock is not poisoned");
        let waited = asked.elapsed();
        TimedWrite {
            guard: Some(guard),
            since: Instant::now(),
            waited_ns: waited.as_nanos() as u64,
            stats,
        }
    }
}

impl<T> std::ops::Deref for TimedWrite<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard.as_ref().expect("held")
    }
}

impl<T> std::ops::DerefMut for TimedWrite<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.as_mut().expect("held")
    }
}

impl<T> Drop for TimedWrite<'_, T> {
    fn drop(&mut self) {
        let held = self.since.elapsed().as_nanos() as u64;
        drop(self.guard.take());
        self.stats.record(self.waited_ns, held);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bucket_is_the_power_of_two_microsecond_band_the_duration_falls_in() {
        assert_eq!(LockStats::bucket(999), 0, "under a microsecond");
        assert_eq!(LockStats::bucket(1_000), 1, "exactly one");
        assert_eq!(LockStats::bucket(1_999), 1);
        assert_eq!(LockStats::bucket(2_000), 2);
        assert_eq!(LockStats::bucket(4_000), 3);
        assert_eq!(
            LockStats::bucket(u64::MAX),
            BUCKETS - 1,
            "anything absurd lands in the last bucket rather than out of bounds"
        );
    }

    /// **Nearest-rank, and the difference from an interpolated percentile matters here.**
    ///
    /// The first version of this test asserted that one slow hold in a hundred moves the
    /// p99. It does not, and the code was right: the 99th percentile of 100 samples is the
    /// 99th smallest, and the outlier is the 100th. A reader watching `hold_p99` for a rare
    /// stall would be watching the wrong column — `hold_max` is the one that carries it, and
    /// that is why both are reported.
    #[test]
    fn one_outlier_in_a_hundred_moves_the_maximum_and_not_the_p99() {
        let s = LockStats::new();
        for _ in 0..99 {
            s.record(0, 1_000);
        }
        s.record(0, 8_000_000);
        let (n, _, _, _, p50, p99, max, total) = s.snapshot();
        assert_eq!(n, 100);
        assert_eq!(p50, 1, "the median hold is in the 1µs band");
        assert_eq!(
            p99, 1,
            "the 99th of 100 samples is still a fast hold; the outlier is the 100th"
        );
        assert_eq!(max, 8_000, "the maximum is exact, not bucketed");
        assert_eq!(total, 99 + 8_000, "microseconds held, summed exactly");
    }

    /// Once the tail is wider than 1% of the samples the p99 does move, which is the
    /// property that makes it worth reporting for a contended lock.
    #[test]
    fn a_tail_wider_than_one_percent_moves_the_p99() {
        let s = LockStats::new();
        for _ in 0..989 {
            s.record(0, 1_000);
        }
        for _ in 0..11 {
            s.record(0, 8_000_000);
        }
        let (n, _, _, _, p50, p99, _, _) = s.snapshot();
        assert_eq!(n, 1_000);
        assert_eq!(p50, 1, "the median is unmoved by a tail");
        assert!(
            p99 >= 4_096,
            "with 11 slow holds in 1,000 the p99 must be in the slow band, not {p99}"
        );
    }

    /// The guard records even when the critical section returns early.
    #[test]
    fn a_hold_is_counted_when_the_guard_is_dropped_on_any_path() {
        static S: LockStats = LockStats::new();
        let m = std::sync::Mutex::new(0u32);
        fn early(m: &std::sync::Mutex<u32>) -> Option<u32> {
            let g = Timed::acquire(m, &S);
            if *g == 0 {
                return None; // the path that used to lose the measurement
            }
            Some(*g)
        }
        assert_eq!(early(&m), None);
        assert_eq!(S.snapshot().0, 1, "the early return still counted its hold");
    }
}
