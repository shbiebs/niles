//! **How long the engine's locks are held, and how long connections wait for them.**
//!
//! This module was written when `daemon::serve` took one `Arc<Mutex<RevEngine>>` around the
//! whole of `Session::handle` — parse, plan, execute, frame — and that critical section
//! contained the `fsync`, because `RevEngine::append` blocked in the sequencer until the
//! epoch was on stable storage. Every other connection, reads included, waited behind one
//! disk barrier. Nothing measured it: the published contract numbers were
//! single-connection, where a lock nobody contends for costs nothing, and the one
//! experiment that varied connections (E19) was read as a scaling result rather than as a
//! lock diagnosis. This module is the instrument that made the diagnosis a number instead
//! of an argument, and the number is what justified the two changes that followed.
//!
//! **Neither of those sentences describes the engine today, which is why they are in the
//! past tense.** Since cycle 6 the engine is behind an `RwLock`, so readers run
//! concurrently with each other; since cycle 7's T-05 the barrier is submitted inside the
//! base guard and *waited on* outside it, so no lock in this process is held across an
//! `fsync`. What the histograms measure now is the base guard: an exclusive hold across an
//! append's apply, and a shared hold across a keyed read.
//!
//! What is **not** instrumented, and is the open question the audit leaves: the view mutex
//! (`V` in the lock order `O < B < P < V < C`). A mixed workload's read *maximum* is 12–13
//! ms on the reference host while the base guard's longest wait is 1.7 ms, so the tail is
//! somewhere this file cannot see.
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
    /// **Where a shared acquisition of this lock is *also* recorded**, and where an
    /// exclusive one is.
    ///
    /// A reader-writer lock reported as one histogram gives a p99 over a shared mode and an
    /// exclusive mode that belongs to neither (A10-08). The split is declared here, on the
    /// aggregate, rather than decided at the recording site: the first attempt tested for
    /// the aggregate's address inside `TimedRead::drop` and `TimedWrite::drop`, which made
    /// "the modes sum to the aggregate" a property of two `if`s in two `Drop` impls and
    /// therefore a property a test had to check — and the test that checked it passed
    /// against a build with the shared arm deleted, because the run it checked happened to
    /// contain no shared acquisition.
    ///
    /// Declared, the sum is arithmetic: every `record` on the aggregate goes to exactly one
    /// of these when the acquisition's mode is known, and to neither when it is not (a plain
    /// mutex has no modes and leaves both `None`).
    shared: Option<&'static LockStats>,
    exclusive: Option<&'static LockStats>,
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
            shared: None,
            exclusive: None,
        }
    }

    /// A lock whose two modes are counted apart as well as together.
    pub const fn with_modes(
        shared: &'static LockStats,
        exclusive: &'static LockStats,
    ) -> LockStats {
        LockStats {
            wait: [ZERO; BUCKETS],
            hold: [ZERO; BUCKETS],
            acquisitions: ZERO,
            wait_ns: ZERO,
            hold_ns: ZERO,
            max_wait_ns: ZERO,
            max_hold_ns: ZERO,
            shared: Some(shared),
            exclusive: Some(exclusive),
        }
    }

    /// The histogram a shared acquisition of this lock is also recorded into, if any.
    pub fn shared_mode(&self) -> Option<&'static LockStats> {
        self.shared
    }

    /// The histogram an exclusive acquisition of this lock is also recorded into, if any.
    pub fn exclusive_mode(&self) -> Option<&'static LockStats> {
        self.exclusive
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

    /// **The upper bound of the bucket the `q`th quantile falls in, in microseconds.**
    ///
    /// It said that and returned the *lower* edge. `bucket` puts a sample of `us`
    /// microseconds in bucket `floor(log2(us)) + 1`, so bucket `i ≥ 1` holds
    /// `[2^(i-1), 2^i - 1]`; the old body returned `1 << (i - 1)`, which is that interval's
    /// **floor**. Every `p50` and `p99` this file has ever printed — in `run6.sh`'s tables,
    /// in the audit records that quote them, in `E19-scaling` — was therefore a lower bound
    /// presented under the name "upper bound", understating the quantile by up to a factor
    /// of two. A9-F07.
    ///
    /// Two corrections. The edge is `2^i - 1`. And it is **clamped by the largest sample
    /// actually observed**, which is both tighter and the only honest answer for the top
    /// bucket: that one is open-ended (`us ≥ 2^22`, about 4.2 s and up) and has no finite
    /// edge, so the observed maximum is the smallest true upper bound available. The
    /// clamp costs nothing for the other buckets and can only make the number smaller and
    /// still correct.
    ///
    /// Bucket 0 is everything under one microsecond; its upper bound, truncated to
    /// microseconds, is 0. `acquisitions` is what distinguishes that from no data.
    fn quantile(hist: &[AtomicU64; BUCKETS], q: f64, max_ns: u64) -> u64 {
        let total: u64 = hist.iter().map(|b| b.load(Relaxed)).sum();
        if total == 0 {
            return 0;
        }
        let max_us = max_ns / 1_000;
        let want = (total as f64 * q).ceil() as u64;
        let mut seen = 0u64;
        for (i, b) in hist.iter().enumerate() {
            seen += b.load(Relaxed);
            if seen >= want {
                if i == BUCKETS - 1 {
                    // Open-ended: no finite edge exists, so the observed maximum is it.
                    return max_us;
                }
                let edge = if i == 0 {
                    0
                } else {
                    (1u64 << i).saturating_sub(1)
                };
                return edge.min(max_us);
            }
        }
        max_us
    }

    /// `(acquisitions, wait_p50µs, wait_p99µs, wait_maxµs, hold_p50µs, hold_p99µs,
    /// hold_maxµs, hold_totalµs)`.
    ///
    /// `hold_total` is there so a reader can compute occupancy — the fraction of a run's
    /// wall clock during which the engine was locked — which is the number that says whether
    /// the mutex is the ceiling or merely present.
    pub fn snapshot(&self) -> (u64, u64, u64, u64, u64, u64, u64, u64) {
        let wmax = self.max_wait_ns.load(Relaxed);
        let hmax = self.max_hold_ns.load(Relaxed);
        (
            self.acquisitions.load(Relaxed),
            Self::quantile(&self.wait, 0.50, wmax),
            Self::quantile(&self.wait, 0.99, wmax),
            wmax / 1_000,
            Self::quantile(&self.hold, 0.50, hmax),
            Self::quantile(&self.hold, 0.99, hmax),
            hmax / 1_000,
            self.hold_ns.load(Relaxed) / 1_000,
        )
    }

    /// **Zero every counter — the phase boundary these histograms never had.**
    ///
    /// `ENGINE_LOCK` and `VIEW_LOCK` are process-global and were never cleared, so a harness
    /// that printed them after each level of a sweep printed level *n*'s numbers *plus every
    /// level before it*: a `max` that can only rise, a `p99` weighted by the earliest and
    /// least loaded phase, and an occupancy total spanning phases that were supposed to be
    /// compared. `SLOW_READS` was given a reset in cycle 8 for exactly this reason and these
    /// two were left; every histogram `run6.sh` has printed is cumulative (F-75, A9-F08).
    ///
    /// Not atomic across the whole structure, and it does not need to be: the caller is a
    /// harness at a quiescent phase boundary, not a concurrent reader. What it must not do is
    /// pretend — a snapshot taken while traffic is running is a snapshot of a moving
    /// structure whether or not this function exists.
    pub fn reset(&self) {
        for b in &self.wait {
            b.store(0, Relaxed);
        }
        for b in &self.hold {
            b.store(0, Relaxed);
        }
        self.acquisitions.store(0, Relaxed);
        self.wait_ns.store(0, Relaxed);
        self.hold_ns.store(0, Relaxed);
        self.max_wait_ns.store(0, Relaxed);
        self.max_hold_ns.store(0, Relaxed);
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
pub static ENGINE_LOCK: LockStats = LockStats::with_modes(&BASE_READ, &BASE_WRITE);

/// **The base lock's two modes, separately — `base_read` and `base_write`.**
///
/// `ENGINE_LOCK` above is the sum, and the sum is what could not answer the question cycle 10
/// opened. A `RwLock` is held shared by every keyed read across its reconstruction and
/// exclusively by every append across its apply; one histogram over both reports a p99 and a
/// maximum that belong to neither, and the design question — *is the base wait caused by
/// readers' folds or by the writer's section* — is unanswerable from it (A10-08 / F-10-02).
///
/// **What the split is not.** Summing overlapping shared holds is reader-lock-time, not
/// exclusive occupancy: N readers holding the lock for one microsecond each contribute N
/// microseconds to `hold_total` while occupying the lock for one. A ratio of the two totals is
/// therefore not a statement about which mode causes a tail, and the audit that first reported
/// 2.9× said so. What the split *can* do is show that a wait arrived while the exclusive mode
/// was held, or that it did not — which is the falsification the aggregate could not offer.
pub static BASE_READ: LockStats = LockStats::new();
pub static BASE_WRITE: LockStats = LockStats::new();

/// **The view mutex — `V` in the lock order `O < B < P < V < C`.**
///
/// The one lock in this engine that nothing measured. `ENGINE_LOCK` covers the base; the
/// view is taken inside it by an append (to `advance` the maintained state) and taken
/// alone by a keyed read, a report and a stats query. It was added because a mixed
/// workload's read maximum was orders of magnitude above its p99 and the instrumented lock
/// could not explain it, so "somewhere" was as precise as this project could be — the only
/// other candidates being this mutex and the scheduler, and one of them had no counter.
///
/// **Having a counter turned out to exonerate it.** With the harness corrected (one daemon
/// per replicate, histograms reset per level, level-local counters) and the read split so
/// the fold no longer runs under this lock, the interleaved two-arm run of C9-06.2 finds
/// the tail is dominated by `base wait` on both arms and that removing the long view hold
/// is worth +1.8%. The instrument earned its place by falsifying the guess that motivated
/// it, which is the only thing an instrument is for.
///
/// Both histograms use the same 24 power-of-two buckets, so a wait here and a wait on the
/// base are directly comparable; the top bucket is about 8.4 seconds, well past the 32 ms
/// the tail lives under.
pub static VIEW_LOCK: LockStats = LockStats::new();

/// **The statement boundary: `Session::handle` entered to `Session::handle` returned.**
///
/// Not a lock, and the type is reused deliberately — the histogram, the reset and the
/// clamped quantiles are exactly what is wanted, and `wait` is left at zero because nothing
/// is waited for. This exists because of what the slow-read table could *not* say. Its four
/// timestamps are taken inside `answer_from_view`, one after another, so
/// `total - (base_wait + view_wait + view_hold)` is arithmetic, not a measurement: three
/// consecutive intervals partition the fourth by construction and the remainder is the
/// truncation of four microsecond divisions. The audit reported "0–2 µs on everything else"
/// from that column and it measured nothing at all (A9-F07).
///
/// The parts genuinely outside the engine's own section — compiling or looking up the plan,
/// framing the reply, and everything the connection does around them — are outside those
/// four timestamps, so they need their own boundaries. This is the inner one.
pub static STATEMENT: LockStats = LockStats::new();

/// **The connection boundary: a message decoded to its reply written.**
///
/// The outer of the two. `WIRE.hold` minus `STATEMENT.hold` is the framing, the socket and
/// whatever the scheduler did between them — the term that was being *asserted* to be
/// 0–2 µs and had never been measured. Recorded by `daemon::serve` around one iteration of
/// its loop, so it includes the durability barrier a reply waits on, which is the point:
/// a reply that waits 8 ms for an fsync is not a slow engine and the two must be
/// distinguishable.
pub static WIRE: LockStats = LockStats::new();

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
        // One clock read, not two: see `Timed::acquire`.
        let acquired = Instant::now();
        TimedRead {
            guard: Some(guard),
            waited_ns: acquired.duration_since(asked).as_nanos() as u64,
            since: acquired,
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
        // **The mode scope, beside the scope the caller named.** A shared acquisition of a
        // lock that declares its modes lands in the shared one as well as in the aggregate,
        // so the aggregate keeps the meaning every prior cycle read it with and the two
        // modes are separable for the first time.
        //
        // The lock says which histogram that is; this code does not test for a particular
        // static's address. Two consequences, and both are the point: a second reader-writer
        // lock gets the split by declaring it rather than by editing this `Drop`, and "the
        // modes sum to the aggregate" stops being a claim about the branch below.
        //
        // Recorded here rather than at the call site because a guard that returns early must
        // still be counted, which is why this type exists at all.
        if let Some(mode) = self.stats.shared_mode() {
            mode.record(self.waited_ns, held);
        }
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
        // One clock read, not two: see `Timed::acquire`.
        let acquired = Instant::now();
        TimedWrite {
            guard: Some(guard),
            waited_ns: acquired.duration_since(asked).as_nanos() as u64,
            since: acquired,
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
        if let Some(mode) = self.stats.exclusive_mode() {
            mode.record(self.waited_ns, held);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A declared mode is recorded into, and the aggregate is recorded into as well.**
    ///
    /// Deterministic because the lock it exercises is this test's own: `ENGINE_LOCK` is
    /// process-global and every other test in the binary contributes to it, so a check
    /// written against it either races or — as the first draft of this guard did — compares
    /// three numbers that the run under test never touched and passes for that reason.
    ///
    /// The property: one shared acquisition and one exclusive acquisition leave the
    /// aggregate at two and each mode at one. That is what makes `base_read + base_write ==
    /// base` arithmetic rather than a hope, and what a `Drop` impl that stopped routing
    /// would break.
    #[test]
    fn a_shared_acquisition_is_counted_in_the_aggregate_and_in_the_shared_mode() {
        static AGG_SHARED: LockStats = LockStats::new();
        static AGG_EXCLUSIVE: LockStats = LockStats::new();
        static AGG: LockStats = LockStats::with_modes(&AGG_SHARED, &AGG_EXCLUSIVE);

        let lock = std::sync::RwLock::new(0u32);
        drop(TimedRead::acquire(&lock, &AGG));
        drop(TimedWrite::acquire(&lock, &AGG));

        let (agg, ..) = AGG.snapshot();
        let (shared, ..) = AGG_SHARED.snapshot();
        let (exclusive, ..) = AGG_EXCLUSIVE.snapshot();
        assert_eq!(
            shared, 1,
            "the shared acquisition reached `base_read`'s analogue"
        );
        assert_eq!(
            exclusive, 1,
            "the exclusive acquisition reached `base_write`'s analogue"
        );
        assert_eq!(
            agg, 2,
            "both acquisitions are still in the aggregate, so a cycle that quoted the \
             aggregate is quoting the same number it always was"
        );
        assert_eq!(
            shared + exclusive,
            agg,
            "the modes partition the aggregate; this is the identity the wire's `base`, \
             `base_read` and `base_write` columns are read with"
        );
    }

    /// A lock with no declared modes records into itself and nowhere else.
    #[test]
    fn a_lock_without_modes_records_only_the_aggregate() {
        static PLAIN: LockStats = LockStats::new();
        assert!(PLAIN.shared_mode().is_none());
        assert!(PLAIN.exclusive_mode().is_none());
        let lock = std::sync::RwLock::new(0u32);
        drop(TimedRead::acquire(&lock, &PLAIN));
        let (n, ..) = PLAIN.snapshot();
        assert_eq!(n, 1);
    }

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

    /// **A reported quantile is an upper bound on the band, not its floor — A9-F07.**
    ///
    /// A single 3 µs hold lands in the band `[2, 3] µs`. The bound to report is 3. The old
    /// body returned `1 << (i - 1)` — 2 — and called it "the upper bound of the bucket",
    /// which is a number no sample in that band can be under. Every published `p50` and
    /// `p99` from this file was low by up to a factor of two.
    #[test]
    fn a_reported_quantile_is_never_below_a_sample_it_summarises() {
        let s = LockStats::new();
        s.record(0, 3_000);
        let (_, _, _, _, p50, p99, max, _) = s.snapshot();
        assert_eq!(max, 3, "the maximum is exact");
        assert_eq!(
            p50, 3,
            "a 3µs hold is in the [2,3]µs band and its bound is 3; reporting 2 claims a \
             ceiling under the only sample there is"
        );
        assert_eq!(p99, 3);
        // And the clamp: with one 3µs sample the band's edge is 3, which is also the max.
        // With a sample at the bottom of a wide band the observed maximum is the tighter
        // and still-true bound.
        let t = LockStats::new();
        t.record(0, 4_000);
        let (_, _, _, _, tp50, _, tmax, _) = t.snapshot();
        assert_eq!(tmax, 4);
        assert_eq!(
            tp50, 4,
            "the [4,7]µs band's edge is 7, but nothing over 4µs was seen, so 4 is the \
             smallest true upper bound"
        );
    }

    /// **The open-ended band has no finite edge, and must not be given one.**
    ///
    /// The last bucket holds everything from about 4.2 s upward. `1 << (BUCKETS - 2)` is its
    /// *floor*, and returning it says "the p99 was 4.19 s" about a hold that may have been
    /// an hour. The only true upper bound available is the largest sample seen.
    #[test]
    fn the_open_ended_band_reports_the_observed_maximum_rather_than_a_made_up_ceiling() {
        let s = LockStats::new();
        s.record(0, 30_000_000_000);
        let (_, _, _, _, p50, p99, max, _) = s.snapshot();
        assert_eq!(max, 30_000_000, "thirty seconds, exact");
        assert_eq!(
            p50, 30_000_000,
            "the top band is open-ended; its bound is what was actually seen"
        );
        assert_eq!(p99, 30_000_000);
        assert!(
            p99 > 1u64 << (BUCKETS - 2),
            "reporting the band's floor ({}) would understate a {max}µs hold",
            1u64 << (BUCKETS - 2)
        );
    }

    /// **A reset makes a histogram phase-local — F-75.**
    #[test]
    fn a_reset_leaves_a_lock_histogram_reporting_only_what_came_after_it() {
        let s = LockStats::new();
        for _ in 0..100 {
            s.record(0, 9_000_000);
        }
        assert_eq!(s.snapshot().6, 9_000, "the first phase is in there");
        s.reset();
        assert_eq!(s.snapshot().0, 0, "no acquisitions survive a reset");
        assert_eq!(s.snapshot().6, 0, "no maximum survives a reset");
        s.record(0, 2_000);
        let (n, _, _, _, _, _, max, total) = s.snapshot();
        assert_eq!(n, 1);
        assert_eq!(
            max, 2,
            "the second phase's maximum must not inherit the first phase's 9ms"
        );
        assert_eq!(total, 2);
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

#[cfg(test)]
mod view_lock_tests {
    use super::*;

    /// **A hold on the view must land in the view's histogram.**
    ///
    /// The instrument's own failure mode: a static that nothing records into reports zeros,
    /// and zeros read exactly like a lock nobody waits for — which is the answer this
    /// experiment is trying to distinguish from. So the guard is a deliberate 5 ms hold,
    /// asserted to appear.
    /// A table that is not emptied is the previous level's table with a few rows added.
    #[test]
    fn a_reset_table_reports_only_what_came_after_it() {
        SLOW_READS.reset();
        SLOW_READS.offer(
            9_000_000,
            ReadTrace {
                total_us: 9_000,
                ..Default::default()
            },
        );
        assert_eq!(SLOW_READS.snapshot().len(), 1);
        SLOW_READS.reset();
        assert!(
            SLOW_READS.snapshot().is_empty(),
            "a reset table must report nothing, or every level after the first is mostly a \
             copy of the one before it"
        );
        // And the floor comes down with it: a 9 ms sample left behind would keep every
        // ordinary read out of the next level's table.
        SLOW_READS.offer(
            50_000,
            ReadTrace {
                total_us: 50,
                ..Default::default()
            },
        );
        assert_eq!(
            SLOW_READS.snapshot().len(),
            1,
            "the floor must reset with the table"
        );
        SLOW_READS.reset();
    }

    #[test]
    fn a_long_hold_on_the_view_lands_in_the_view_histogram() {
        let m = std::sync::Mutex::new(0u32);
        let before = VIEW_LOCK.snapshot();
        {
            let mut g = Timed::acquire(&m, &VIEW_LOCK);
            *g += 1;
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let after = VIEW_LOCK.snapshot();
        assert!(
            after.0 > before.0,
            "the acquisition was not counted: {before:?} -> {after:?}"
        );
        assert!(
            after.6 >= 4_000,
            "a 5 ms hold must show in `view_hold_max_us`, which read {} — a histogram that \
             does not see a hold it was handed cannot rule the view out of a 12 ms tail",
            after.6
        );
    }
}

/// **What the instrument costs, so that a run with it can be compared with one without.**
///
/// `#[ignore]`d: a wall-clock figure, run explicitly.
///
/// ```sh
/// cargo test --release -p nilestream-server --lib -- --ignored --test-threads=1 instrument_cost
/// ```
#[cfg(test)]
mod instrument_cost {
    use super::*;

    #[test]
    #[ignore = "a measurement: run with --release -- --ignored --test-threads=1"]
    fn timing_an_acquisition_is_not_a_measurable_share_of_a_read() {
        const N: u32 = 200_000;
        let m = std::sync::Mutex::new(0u64);
        // Interleaved, five measured runs after one warm-up, medians reported — the
        // protocol every wall-clock figure in this project is held to, at this scale
        // because the difference being measured is nanoseconds and a single run of
        // anything on a shared two-core host is not evidence.
        let mut bare = Vec::new();
        let mut timed = Vec::new();
        for run in 0..6 {
            let t0 = std::time::Instant::now();
            for _ in 0..N {
                let mut g = m.lock().unwrap();
                *g += 1;
            }
            let b = t0.elapsed().as_nanos() as f64 / N as f64;
            let t1 = std::time::Instant::now();
            for _ in 0..N {
                let mut g = Timed::acquire(&m, &VIEW_LOCK);
                *g += 1;
            }
            let t = t1.elapsed().as_nanos() as f64 / N as f64;
            if run > 0 {
                bare.push(b);
                timed.push(t);
            }
        }
        let med = |mut v: Vec<f64>| {
            v.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
            v[v.len() / 2]
        };
        let (b, t) = (med(bare), med(timed));
        eprintln!(
            "bare {b:.1} ns/acq, timed {t:.1} ns/acq, delta {:.1} ns",
            t - b
        );
        assert!(
            t - b < 200.0,
            "timing an acquisition costs {:.1} ns. A keyed read is tens of microseconds, so \
             the instrument may not be a measurable share of one; above this it changes what \
             it measures",
            t - b
        );
    }
}

/// **The slowest keyed reads, broken into where the time went.**
///
/// The histograms above say how long the base and the view were *held and waited for*, in
/// aggregate. They cannot answer the question the mixed workload actually poses: a read
/// maximum orders of magnitude above the p99 is a handful of reads per run, and an
/// aggregate is exactly the wrong instrument for a handful. This keeps the slowest few,
/// whole, with their parts — and it is what showed that those reads are waiting on the
/// *base*, not on the view, which no aggregate in this file could have said.
///
/// Bounded and cheap: a fixed array behind one mutex, and a relaxed atomic floor read on
/// every read so the common case — a read faster than the slowest kept — takes no lock at
/// all. Sixteen samples is enough to see whether the tail is one event or a population, and
/// small enough that the table fits in a wire reply.
pub struct SlowReads {
    floor_ns: AtomicU64,
    /// A fixed array and a length, not a `Vec`: a measurement instrument that allocates
    /// shows up in the measurement. The first version grew a `Vec` to sixteen and cost
    /// `served_point` three allocations and 896 live bytes in E18 — small, and exactly the
    /// kind of small that makes a memory row unreproducible for a reason unrelated to the
    /// program.
    samples: std::sync::Mutex<([ReadTrace; SlowReads::KEEP], usize)>,
}

/// One keyed read, and the three waits inside it. Microseconds, because that is the unit
/// every other latency in this system is reported in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReadTrace {
    pub total_us: u64,
    pub base_wait_us: u64,
    pub view_wait_us: u64,
    pub view_hold_us: u64,
    /// The **second** view acquisition — the one `finish_fold` takes to install. The
    /// two-phase read takes V twice and the old table counted one of them, so a read that
    /// waited to install was reported as a read that waited for nothing (A10-08).
    pub view_wait2_us: u64,
    pub view_hold2_us: u64,
    /// The reconstruction itself, with no lock on the view. Named rather than left in a
    /// residual, because "the fold" is the term every design question this cycle is about.
    pub fold_us: u64,
    /// How this read was answered: [`ReadOutcome`] as a small integer. A joined read is
    /// offered to this table now; it never was, so the one path whose cost is *another
    /// thread's* fold could not appear in the tail at all.
    pub outcome: ReadOutcome,
    /// `applied − anchor` when the flight was authorised, and again when it landed. The
    /// first says the read started behind; the difference says the frontier moved under it.
    /// One number could not tell those apart, and the merge decision turns on which it is.
    pub gap_begin: u64,
    pub gap_finish: u64,
}

/// How a keyed read was answered. Ordered so a larger number is a costlier path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadOutcome {
    #[default]
    Hit,
    /// Folded and owned the key's flight.
    FoldOwned,
    /// Folded because another anchor owned the key, or the flight table was full. Exact,
    /// unshared, and installed nothing.
    FoldAlone,
    /// Shared another reader's fold at the same anchor.
    Joined,
    /// Joined a flight that went away, and went round again.
    JoinRetried,
}

impl ReadOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            ReadOutcome::Hit => "hit",
            ReadOutcome::FoldOwned => "fold_owned",
            ReadOutcome::FoldAlone => "fold_alone",
            ReadOutcome::Joined => "joined",
            ReadOutcome::JoinRetried => "join_retried",
        }
    }
}

impl SlowReads {
    pub const KEEP: usize = 16;

    pub const fn new() -> SlowReads {
        SlowReads {
            floor_ns: AtomicU64::new(0),
            samples: std::sync::Mutex::new((
                [ReadTrace {
                    total_us: 0,
                    base_wait_us: 0,
                    view_wait_us: 0,
                    view_hold_us: 0,
                    view_wait2_us: 0,
                    view_hold2_us: 0,
                    fold_us: 0,
                    outcome: ReadOutcome::Hit,
                    gap_begin: 0,
                    gap_finish: 0,
                }; SlowReads::KEEP],
                0,
            )),
        }
    }

    /// Offer a completed read. Kept only if it is slower than the slowest already kept, or
    /// if there is room.
    pub fn offer(&self, total_ns: u64, t: ReadTrace) {
        if total_ns < self.floor_ns.load(Relaxed) {
            return;
        }
        let Ok(mut g) = self.samples.lock() else {
            return;
        };
        let (buf, len) = &mut *g;
        if *len < Self::KEEP {
            buf[*len] = t;
            *len += 1;
        } else if t.total_us > buf[Self::KEEP - 1].total_us {
            buf[Self::KEEP - 1] = t;
        } else {
            return;
        }
        buf[..*len].sort_by_key(|s| std::cmp::Reverse(s.total_us));
        if *len == Self::KEEP {
            self.floor_ns
                .store(buf[Self::KEEP - 1].total_us * 1_000, Relaxed);
        }
    }

    /// Forget everything kept so far.
    ///
    /// **A level's table must be that level's.** Without this the table is cumulative over
    /// the process, so a 12-connection level's worst reads sit in the 6-connection level's
    /// output and every table after the first is mostly a copy of the one before it. The
    /// first Host C run printed ten tables of which six were identical, and the shape a
    /// reader needed — how the tail *grows* with connections — was the one thing they could
    /// not show.
    pub fn reset(&self) {
        self.floor_ns.store(0, Relaxed);
        if let Ok(mut g) = self.samples.lock() {
            g.1 = 0;
        }
    }

    pub fn snapshot(&self) -> Vec<ReadTrace> {
        self.samples
            .lock()
            .map(|g| g.0[..g.1].to_vec())
            .unwrap_or_default()
    }
}

impl Default for SlowReads {
    fn default() -> Self {
        SlowReads::new()
    }
}

/// The slowest keyed reads this process has served.
pub static SLOW_READS: SlowReads = SlowReads::new();
