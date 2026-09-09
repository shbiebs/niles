//! **The read path, served by the mechanism the thesis is about.**
//!
//! `nilestreamd` used to answer every query from a `HashMap` and say so in its startup
//! banner. That was honest but it made the server unmeasurable: when the E16 wall-clock
//! harness compared it to PostgreSQL, the `point` row read `PARITY` and the document had to
//! explain at length that the number was the protocol path rather than an engine result.
//!
//! This module closes that. A wire query is answered by
//! [`PartialView::read`](proto_engine::PartialView::read) over an immutable, hash-chained
//! [`Ledger`](proto_engine::Ledger) — partial materialisation, the absence lattice, an
//! anchored upquery on a miss — so the same harness now measures the thing the specification
//! makes claims about.
//!
//! # What is still not a database
//!
//! Everything `proto-engine`'s own module docs say: in-memory, single-threaded, no durability,
//! no consensus, no planner, two view shapes. That list has not shrunk and this module does
//! not pretend otherwise. What has changed is narrower and worth having: a latency measured
//! here is the latency of *a partial view answering an anchored read*, and a miss is a real
//! reconstruction over a real base, so the miss rate is reportable alongside it.
//!
//! # Why the miss rate is reported and not just the latency
//!
//! A parity result at a 0% miss rate and a parity result at a 40% miss rate are different
//! findings, and the phase diagram of thesis §9.3 is built from exactly that difference. A
//! server that reported only latency would let the more interesting of the two disappear, so
//! [`RevEngine::stats`] carries it out to the harness.

use proto_engine::{EvictionPolicy, Ledger, Posting, Row, ViewMode};

/// **A rendezvous inside `report_from_view`**, between deciding the plan's shape and taking
/// the view to certify and copy it.
///
/// It exists so the A10-02 hazard can be *witnessed* rather than argued about. In the
/// repaired code a thread paused here holds nothing and has certified nothing, so an append
/// landing while it is parked is seen by the single guard below and the report declines. In
/// the code this replaces, the certification had already happened above this point — so the
/// same append moves the view under a decision already taken, and the copy returns the new
/// values under the old anchor's label.
///
/// The same rendezvous, in the same place, tells the two builds apart. Compiled only under
/// `cfg(test)` and keyed to one thread, for the reasons `stats_order_hook` gives.
#[cfg(test)]
pub(crate) mod report_race_hook {
    use std::sync::{Condvar, Mutex};
    use std::thread::ThreadId;

    static ARMED_FOR: Mutex<Option<ThreadId>> = Mutex::new(None);
    static REACHED: Mutex<bool> = Mutex::new(false);
    static REACHED_CV: Condvar = Condvar::new();
    static RELEASED: Mutex<bool> = Mutex::new(false);
    static RELEASED_CV: Condvar = Condvar::new();

    pub(crate) fn arm_this_thread() {
        *REACHED.lock().expect("not poisoned") = false;
        *RELEASED.lock().expect("not poisoned") = false;
        *ARMED_FOR.lock().expect("not poisoned") = Some(std::thread::current().id());
    }

    pub(crate) fn disarm() {
        *ARMED_FOR.lock().expect("not poisoned") = None;
        release();
    }

    pub(crate) fn pause() {
        let armed = *ARMED_FOR.lock().expect("not poisoned") == Some(std::thread::current().id());
        if !armed {
            return;
        }
        {
            let mut r = REACHED.lock().expect("not poisoned");
            *r = true;
            REACHED_CV.notify_all();
        }
        let mut g = RELEASED.lock().expect("not poisoned");
        while !*g {
            g = RELEASED_CV.wait(g).expect("not poisoned");
        }
    }

    pub(crate) fn wait_until_reached(within: std::time::Duration) -> bool {
        let deadline = std::time::Instant::now() + within;
        let mut g = REACHED.lock().expect("not poisoned");
        while !*g {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() {
                return false;
            }
            let (next, timeout) = REACHED_CV.wait_timeout(g, left).expect("not poisoned");
            g = next;
            if timeout.timed_out() && !*g {
                return false;
            }
        }
        true
    }

    pub(crate) fn release() {
        let mut g = RELEASED.lock().expect("not poisoned");
        *g = true;
        RELEASED_CV.notify_all();
    }
}

#[cfg(not(test))]
pub(crate) mod report_race_hook {
    #[inline(always)]
    pub(crate) fn pause() {}
}

/// **A rendezvous inside `read_stats`, between its base phase and its view phase.**
///
/// It exists so the deadlock of A10-01 can be *witnessed* rather than argued about. The two
/// statements the hook sits between are the ones whose order is the bug: with V taken first
/// and B reached for underneath it, a thread paused here holds V and is about to want B,
/// which is exactly the state an appender holding B and wanting V cannot be reconciled with.
/// With the order repaired, a thread paused here holds nothing, and the same test that hangs
/// on the broken build completes on the repaired one.
///
/// Compiled only under `cfg(test)`; in a release build `pause` is not called at all, so this
/// costs the serving path nothing. Keyed to one thread rather than armed globally, because
/// the test binary runs tests in parallel and a global arm would park an unrelated test's
/// stats query.
#[cfg(test)]
pub(crate) mod stats_order_hook {
    use std::sync::{Condvar, Mutex};
    use std::thread::ThreadId;

    static ARMED_FOR: Mutex<Option<ThreadId>> = Mutex::new(None);
    static REACHED: Mutex<bool> = Mutex::new(false);
    static REACHED_CV: Condvar = Condvar::new();
    static RELEASED: Mutex<bool> = Mutex::new(false);
    static RELEASED_CV: Condvar = Condvar::new();

    /// Called by the thread that is about to run `read_stats`, before it does.
    pub(crate) fn arm_this_thread() {
        *REACHED.lock().expect("not poisoned") = false;
        *RELEASED.lock().expect("not poisoned") = false;
        *ARMED_FOR.lock().expect("not poisoned") = Some(std::thread::current().id());
    }

    pub(crate) fn disarm() {
        *ARMED_FOR.lock().expect("not poisoned") = None;
        release();
    }

    /// The rendezvous itself. A no-op on every thread but the armed one.
    pub(crate) fn pause() {
        let armed = *ARMED_FOR.lock().expect("not poisoned") == Some(std::thread::current().id());
        if !armed {
            return;
        }
        {
            let mut r = REACHED.lock().expect("not poisoned");
            *r = true;
            REACHED_CV.notify_all();
        }
        let mut g = RELEASED.lock().expect("not poisoned");
        while !*g {
            g = RELEASED_CV.wait(g).expect("not poisoned");
        }
    }

    /// Block until the armed thread reaches the rendezvous, or the deadline passes.
    pub(crate) fn wait_until_reached(within: std::time::Duration) -> bool {
        let deadline = std::time::Instant::now() + within;
        let mut g = REACHED.lock().expect("not poisoned");
        while !*g {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() {
                return false;
            }
            let (next, timeout) = REACHED_CV.wait_timeout(g, left).expect("not poisoned");
            g = next;
            if timeout.timed_out() && !*g {
                return false;
            }
        }
        true
    }

    pub(crate) fn release() {
        let mut g = RELEASED.lock().expect("not poisoned");
        *g = true;
        RELEASED_CV.notify_all();
    }
}

/// In a non-test build the rendezvous does not exist and nothing calls it.
#[cfg(not(test))]
pub(crate) mod stats_order_hook {
    #[inline(always)]
    pub(crate) fn pause() {}
}

/// What one attempt at answering a keyed read from the maintained view produced.
///
/// Three cases, because the two-phase read has three outcomes and collapsing them into an
/// `Option` would lose the one that matters: a reader that found a reconstruction already
/// in flight at its own anchor must **wait holding nothing**, and a function that can only
/// say "no answer" would have to either wait inside itself — under the base guard, which
/// blocks every append — or throw the shared fold away and start a second one.
enum ViewAnswer {
    /// The view answered, exactly at the anchor asked for.
    Rows(crate::session::Rows),
    /// Not a shape this path serves, or the view could not answer it. The fold answers.
    NotApplicable,
    /// Someone else is folding this key at this anchor. Wait — with no lock held — and then
    /// **use that flight's answer**, which is exact at this anchor by construction.
    ///
    /// Everything the caller needs to shape the reply travels with the ticket, so a joined
    /// read never re-enters the view and never re-acquires the base. It used to discard the
    /// answer and go round the loop, which re-took B for the same logical read and turned a
    /// join — the mechanism whose whole point is that the second reader pays nothing — into
    /// a wait *plus* a full second pass (A10-04).
    Wait(WaitPlan),
}

/// **Bounded, because a retry loop with no ceiling is a spin.** A waiter wakes when the
/// flight it joined publishes, and the next attempt hits the entry that flight installed. It
/// can fail to: the owner was cancelled, or the entry was evicted between the wake and the
/// retry under a budget smaller than the number of keys in flight. A handful of rounds covers
/// those; past them the fold answers, which is slower and exact, and never a spin under load.
const MAX_JOIN_ATTEMPTS: u32 = 8;

/// One attempt at a query, and what it needs from its caller.
enum QueryStep {
    Done(Result<crate::session::Rows, crate::session::ServeError>),
    /// A flight is already folding this key at this anchor. The caller waits — holding
    /// whatever it likes, including nothing — and then calls `resolve_join`.
    Wait(WaitPlan),
}

/// A join, and the shape its answer must be given.
struct WaitPlan {
    ticket: nilestream_core::rev::WaitTicket,
    /// The reply's column names, built before the wait so the join costs no work after it.
    columns: Vec<String>,
    acct: u64,
    cur: u32,
    with_currency: bool,
}

/// What a read cost, in the units that distinguish one parity result from another.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadStats {
    pub reads: u64,
    pub hits: u64,
    pub misses: u64,
    /// Base rows touched by reconstructions. The counted-work figure the prototype was built
    /// to produce, now available alongside a wall-clock one for the same run.
    pub rows_touched: u64,
    pub resident: usize,
    /// **Keyed reads the maintained view answered, and keyed reads whose view answer was
    /// discarded.**
    ///
    /// The pair and not a ratio, because they are counted at different places and a ratio
    /// computed from one would hide which. `fallbacks` counts only the anchor mismatch — the
    /// view was consulted and its answer thrown away because the entry was not exact at the
    /// anchor asked for — and not the shapes the view is never asked about, which are a
    /// different fact belonging to `serve_path`.
    ///
    /// This is the number `BLOCKED-fallback-rate` was raised about, twice. It was correct
    /// behaviour and an uncounted cost: 87.4% of keyed reads over the wire under four
    /// readers and two writers, which an audit could only infer from a latency distribution
    /// because no surface in the server could be asked.
    pub view_answers: u64,
    pub fallbacks: u64,
    /// **The two structures whose size is a function of history, made askable.**
    ///
    /// `view_metadata_keys` is how many keys the view's eviction-policy metadata describes;
    /// the residency budget is supposed to bound it and did not, at 159 B per key ever read.
    /// `idem_window_keys` is how many identities the base's idempotency index holds, which
    /// the declared window is supposed to bound and did not, at 68.8 B each forever.
    ///
    /// Both are columns rather than a memory figure because a bound nobody can ask about is
    /// a bound nobody can hold: a view whose metadata is linear in history and a ledger whose
    /// index is linear in history are the two ways this engine stops fitting, and until T-05
    /// (cycle 8) neither had a surface.
    pub view_metadata_keys: u64,
    pub idem_window_keys: u64,
    /// **The absence lattice's fourth state, made askable.**
    ///
    /// `pending_joins` counts keyed reads that found a reconstruction already in flight at
    /// their own exact anchor and shared it instead of starting a second one — the number
    /// that was zero by construction while the fold happened inside the view lock, and the
    /// evidence `MISMATCH-pending-unreachable` was raised for.
    ///
    /// `uninstalled_folds` counts exact answers that were deliberately not written to the
    /// view: a flight at another anchor owned the key, or the completion arrived after its
    /// generation was superseded. `pinned_installs` counts reconstructions that landed
    /// below the frontier the view had already applied and therefore entered pinned.
    /// `flights_refused` counts reads the flight table had no room for, which folded
    /// anyway — an overload that shows up only as latency is one nobody can be asked about.
    pub pending_joins: u64,
    pub uninstalled_folds: u64,
    pub pinned_installs: u64,
    pub flights_refused: u64,
    /// **The counters the wire could not be asked for.** `deferred_merges` reads zero until
    /// the merge lands and is reported anyway, because an absent counter and a zero counter
    /// are different claims and only one of them is checkable. `waiters_refused` is the
    /// *other* capacity refusal, kept apart from `flights_refused`. The two gap totals are
    /// the lag a read started with and the lag it ended with; their difference is the only
    /// part its own fold caused (A10-08, A10-11).
    pub deferred_merges: u64,
    /// **What the merge cost, beside what it achieved.** `deferred_merges` alone reports a
    /// benefit with no price; these are the rows the merges really walked, the epochs they
    /// really spanned, and the late landings that pinned instead — split by cause, because an
    /// over-budget suffix, an unretained one and merging switched off are three different
    /// findings with three different remedies.
    pub merge_rows_visited: u64,
    pub merge_epochs_merged: u64,
    /// The bounds in force, so a transcript names its own arm.
    pub merge_max_epochs: u64,
    pub merge_max_rows: u64,
    pub merges_refused_epochs: u64,
    pub merges_refused_rows: u64,
    pub merges_refused_unavailable: u64,
    pub waiters_refused: u64,
    pub joins_answered: u64,
    pub joins_retried: u64,
    pub gap_at_begin_total: u64,
    pub gap_at_finish_total: u64,
    pub gap_at_finish_max: u64,
    /// The two totals' denominators. A total reported without its count invites the reader
    /// to divide by the nearest counter to hand, which is what the first harness run did.
    pub gap_begin_samples: u64,
    pub gap_finish_samples: u64,
    pub flights_behind_at_begin: u64,
    pub flights_that_fell_behind: u64,
}

impl ReadStats {
    pub fn miss_rate(&self) -> f64 {
        if self.reads == 0 {
            return 0.0;
        }
        self.misses as f64 / self.reads as f64
    }

    /// The share of keyed reads that consulted the view and fell back to the fold.
    ///
    /// Over `view_answers + fallbacks` rather than over `reads`: a fold that was never a
    /// candidate for the view is not a fallback, and dividing by every read would report a
    /// number that improves when the workload shifts away from keyed reads.
    pub fn fallback_rate(&self) -> f64 {
        let keyed = self.view_answers + self.fallbacks;
        if keyed == 0 {
            return 0.0;
        }
        self.fallbacks as f64 / keyed as f64
    }
}

/// A ledger, **one** read model over it, and the mapping from a wire query to a read.
///
/// "One" is the repair. `RevEngine` used to hold two: `proto_engine::PartialView`, a
/// hard-coded balance cache keyed `(account, currency)`, and the REV runtime that the thesis
/// is actually about. The wire path read the runtime; `stats()`, `read_point` and
/// `evict_all` read the other one; and nothing in the type said which of the two a caller was
/// looking at. A test that warmed one and measured the other would have passed.
pub struct RevEngine {
    /// **The base, behind a reader-writer lock.**
    ///
    /// The base is append-only and a read of it is a fold over a frozen prefix: two folds
    /// have nothing to say to each other, and serialising them was worth a measured factor.
    /// What needs exclusion is the append — `submit` chains a hash and mutates five indexes
    /// — and the write guard is also what keeps epochs contiguous, because it serialises
    /// submitters in the order they will be assigned ids.
    ledger: std::sync::RwLock<Ledger>,
    /// The currency index every wire query uses. The demo schema declares one currency, and a
    /// server that silently answered in whichever currency it found first would be making the
    /// per-currency conservation rule invisible from the outside.
    currency: u32,
    views: Vec<(String, u32)>,
    /// Counted work spent evaluating circuits, so the scan surface's cost is reported
    /// rather than hidden inside a latency number.
    scan_work: std::sync::atomic::AtomicU64,
    /// Queries served by evaluating a circuit, and base rows materialised for them.
    ///
    /// **Every one of these is a reconstruction.** The served path evaluates the compiled
    /// circuit over a source scan; the partially materialised view is not consulted, so
    /// there is no resident entry to hit. The counters are reported as what they are rather
    /// than as a hit/miss ratio over a view nothing reads — which is what the CSV's
    /// `miss_rate` column would otherwise be silently reporting.
    ///
    /// **Atomics, for the same reason `Ledger::rows_touched` is one.** These three counters
    /// were the only things the scan path wrote, and a `&mut self` taken to increment a
    /// counter is a `&mut self` taken over the whole engine — which is how a fold of twenty
    /// thousand postings came to exclude every other fold. Counting through relaxed atomics
    /// costs an add and buys `&self` on the read path.
    served: std::sync::atomic::AtomicU64,
    served_rows: std::sync::atomic::AtomicU64,
    /// **The incremental REV runtime, over the compiled circuit for a balance.**
    ///
    /// This is the mechanism the thesis is about — partial materialisation under a budget,
    /// the absence lattice, an anchored upquery on a miss — and until now the wire path did
    /// not use it. `read_stats` reported `hits = 0, misses = reads` *by construction*, so
    /// the `miss_rate` column of every benchmark row was 1.00 whatever the engine did, and
    /// the phase diagram's mechanism was measured by nothing the daemon ran.
    ///
    /// `None` when the view could not be installed, which is a refusal rather than a
    /// fallback: the fold answers, and the counters say the runtime served nothing.
    /// **Behind a mutex, because a read of the view is a write to it.** A miss reconstructs
    /// and installs, and an install may evict: partial materialisation mutates on read by
    /// construction, and no lock discipline makes that untrue. What the mutex buys is that
    /// the mutation is scoped to the *view* rather than to the engine — a keyed read holds
    /// it for microseconds, while a fold, which never touches the view at all, holds
    /// nothing and runs concurrently with every other fold.
    runtime: Option<std::sync::Mutex<nilestream_core::rev::Runtime>>,
    /// Currencies the base holds. The installed view is keyed `(account, currency)` and a
    /// wire query asking for one account's balance names no currency, so the runtime can
    /// answer only while there is exactly one to name. More than one and the fold answers —
    /// because picking a currency for the caller is how a per-currency conservation rule
    /// becomes invisible from outside.
    currencies: std::sync::RwLock<std::collections::BTreeSet<u32>>,
    /// **Keyed reads the view answered, and keyed reads whose view answer was discarded.**
    ///
    /// The pair, not a ratio, because the two are counted at different places and a ratio
    /// computed from one of them would hide which. `view_fallbacks` counts only the anchor
    /// mismatch — the view was consulted and its answer thrown away — and not the shapes the
    /// view was never asked about, which are a different fact and belong to `serve_path`.
    view_answers: std::sync::atomic::AtomicU64,
    view_fallbacks: std::sync::atomic::AtomicU64,
    /// **How a join ended.** The wait happens in the query loop with no lock held, which is
    /// the whole point of the split and also the reason these are atomics on the engine
    /// rather than fields on the view: counting them there would cost a third view
    /// acquisition on the one path that exists to avoid holding it. A join that wakes to
    /// find its flight gone is not an error and is not free; the old surface reported
    /// neither outcome (A10-04, A10-08).
    joins_answered: std::sync::atomic::AtomicU64,
    joins_retried: std::sync::atomic::AtomicU64,
    /// The durable sink, when the daemon was started with one. `None` for the in-memory
    /// engine the tests and the benchmark's warm-up use.
    ///
    /// Its presence is the difference between "an epoch was assigned" and "an epoch is on
    /// stable storage", and `append` does not return until the second is true.
    durable: Option<DurableSink>,
    /// **The highest epoch whose record is on stable storage.**
    ///
    /// The ledger's own [`Frontier`](nilestream_ledger::frontiers::Frontier) has carried this
    /// distinction — `sealed` versus `visible` — since it was written, and the engine did not.
    /// It did not need to: `append` blocked on the barrier while holding the engine's lock,
    /// so no reader could observe the gap between an epoch existing and being durable,
    /// because no reader could run at all.
    ///
    /// Moving the barrier out of the lock creates that gap, and this is what closes it.
    /// Rows are applied to the base under the lock — in lock order, so epochs stay totally
    /// ordered and contiguous — and become **visible** only when their record has been
    /// fsynced. A read anchors at [`frontier`](Self::frontier), which reports this rather
    /// than the base's head, so durable-before-visible holds for exactly the same reason it
    /// held before: not by luck, and now not by exclusion either.
    ///
    /// `None` when there is no durable sink, where head and visible are the same thing.
    visible: Option<std::sync::Arc<std::sync::atomic::AtomicU64>>,
}

/// An applied-but-not-yet-durable epoch, and what makes it visible.
pub struct Pending {
    epoch: u64,
    token: PendingDurable,
    visible: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl Pending {
    /// Wait for the barrier, then publish.
    ///
    /// **Publication is `fetch_max`, and that it is monotone is not an accident.** Epochs are
    /// assigned under the engine's lock and submitted in the same order, and the sealer
    /// completes batches in order — so a later epoch's barrier cannot return before an
    /// earlier one's, and taking the maximum can never expose a gap. The alternative, a
    /// counter that assumes it is always advancing by one, would be wrong the first time two
    /// transactions shared a batch.
    pub fn wait(self) -> Result<u64, String> {
        let outcome = self
            .token
            .recv()
            .map_err(|_| "the sealer stopped before the epoch was durable".to_string())?;
        let e = DurableSink::interpret(outcome)?;
        self.visible
            .fetch_max(self.epoch, std::sync::atomic::Ordering::AcqRel);
        Ok(e)
    }
}

/// **A receipt whose barrier can never return — the failure `serve` must not acknowledge.**
///
/// Test-only. A sealer that dies between a submission and its `fsync` looks exactly like this
/// to the caller: the sender end of the reply channel is gone, so `recv` fails and `wait`
/// reports it. There is no way to ask real storage to fail on demand, and a durability test
/// that cannot produce a failing barrier is a test of the happy path wearing the name of the
/// guarantee.
#[cfg(test)]
pub(crate) fn a_barrier_that_will_never_return(epoch: u64) -> Pending {
    let (tx, rx) = std::sync::mpsc::channel();
    drop(tx);
    Pending {
        epoch,
        token: rx,
        visible: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
    }
}

/// A receipt that has already succeeded, for the same reason.
#[cfg(test)]
pub(crate) fn a_barrier_that_has_returned(epoch: u64) -> Pending {
    let (tx, rx) = std::sync::mpsc::channel();
    tx.send(Ok(epoch)).expect("the receiver is alive");
    Pending {
        epoch,
        token: rx,
        visible: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
    }
}

/// **Test-only shorthands for the two things `serve` does with an append.**
///
/// `serve` calls `Serving::append`, keeps the receipt on its own session, and waits on it
/// before writing the reply. A test that wants a durable epoch wants that whole sequence, and
/// a test that wants to inspect the barrier wants the receipt. Naming both keeps the receipt's
/// ownership visible at every call site: there is no list to drain and nothing to take that
/// another caller could have taken first (A9-F01).
#[cfg(test)]
impl RevEngine {
    /// Append and wait for **this append's own** barrier, returning the epoch.
    pub(crate) fn append_durable(
        &self,
        rows: Vec<Row>,
        txn_id: &str,
    ) -> Result<u64, crate::session::ServeError> {
        let a = <Self as crate::session::Serving>::append(self, rows, txn_id)?;
        if let Some(r) = a.receipt {
            r.wait().map_err(crate::session::ServeError::NotDurable)?;
        }
        Ok(a.epoch)
    }

    /// Append without waiting, handing back the epoch and its receipt.
    pub(crate) fn append_pending(
        &self,
        rows: Vec<Row>,
        txn_id: &str,
    ) -> Result<(u64, Option<Pending>), crate::session::ServeError> {
        let a = <Self as crate::session::Serving>::append(self, rows, txn_id)?;
        Ok((a.epoch, a.receipt))
    }
}

/// A write-ahead sink that does not acknowledge until `fsync` has returned.
///
/// Wrapped around `nilestream_ledger::Sequencer` with [`SyncPolicy::Always`], which is the
/// only policy the daemon accepts. The ordering is the entire content of the guarantee:
/// returning an epoch before the sync would let a client observe a transaction that a crash
/// then erases, and in a ledger an observation that is later erased is not a stale read —
/// it is a transaction a customer saw succeed and that no longer exists.
pub struct DurableSink {
    seq: nilestream_ledger::sequencer::Sequencer,
}

impl DurableSink {
    /// What the sealer beneath this sink has done: epochs, transactions, `fsync`s, and the
    /// **largest batch it has ever formed**.
    ///
    /// Plumbed because the counter that decides whether group commit is reaching the wire
    /// existed and nothing could read it. `Sequencer` has batched to 4,096 since it was
    /// written and is tested at sixteen concurrent submitters; the daemon holds one mutex
    /// across `Session::handle`, so submitters reach `submit` one at a time, the sealer's
    /// drain loop always finds an empty queue, and `max_batch` is 1 for the life of the
    /// process. That is a one-number diagnosis and no surface reported the number.
    pub fn stats(&self) -> nilestream_ledger::sequencer::SequencerStats {
        self.seq.stats()
    }

    /// Open a durable sink at `path`. Refuses any policy but `Always`, which the sequencer
    /// itself also refuses — stated twice on purpose, because the daemon is the layer a
    /// deployment configures and a server that quietly accepted `Never` would be a server
    /// whose durability claim depended on a flag nobody read.
    pub fn open(path: impl AsRef<std::path::Path>) -> std::io::Result<DurableSink> {
        Ok(Self::open_recovered(path)?.0)
    }

    /// `open`, and the records the segment held, so the base can be rebuilt from them.
    pub fn open_recovered(
        path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<(DurableSink, nilestream_ledger::segment::Recovery)> {
        Self::open_bounded(path, None)
    }

    /// The same, with the schema's declared idempotency window, in epochs.
    pub fn open_bounded(
        path: impl AsRef<std::path::Path>,
        idem_window: Option<u64>,
    ) -> std::io::Result<(DurableSink, nilestream_ledger::segment::Recovery)> {
        let (seq, recovery) = nilestream_ledger::sequencer::Sequencer::open_bounded(
            path,
            nilestream_ledger::segment::SyncPolicy::Always,
            idem_window,
        )?;
        Ok((DurableSink { seq }, recovery))
    }

    /// Seal the transaction and return only once `fsync` has.
    ///
    /// A `Duplicate` is not an error: the caller retried, and the correct response is the
    /// original outcome rather than a second transaction. That distinction is the reason
    /// this returns the epoch rather than a unit.
    /// The same call the append path makes, exposed so a test can assert the *ordering*
    /// rather than only the outcome. An append that returned before `fsync` would look
    /// identical in a successful run, so the property needs a test that reads the file.
    pub fn record_for_test(&mut self, txn_id: &str, epoch: u64) -> Result<u64, String> {
        self.record(txn_id, epoch.to_string().into_bytes())
    }

    fn record(&mut self, txn_id: &str, payload: Vec<u8>) -> Result<u64, String> {
        Self::interpret(
            self.record_pending(txn_id, payload)?
                .recv()
                .map_err(|_| "the sealer stopped before answering".to_string())?,
        )
    }

    /// Hand the record to the sealer without waiting for the barrier.
    ///
    /// The caller holds the engine's lock; the wait must not happen there. See
    /// [`Sequencer::submit_pending`](nilestream_ledger::sequencer::Sequencer::submit_pending).
    fn record_pending(&self, txn_id: &str, payload: Vec<u8>) -> Result<PendingDurable, String> {
        self.seq
            .submit_pending(nilestream_ledger::sequencer::Txn {
                idem_key: txn_id.to_string(),
                payload,
            })
            .map_err(|e| format!("{e:?}"))
    }

    /// A duplicate is not an error: the caller retried, and the correct answer is the epoch
    /// the original committed at.
    fn interpret(r: Result<u64, nilestream_ledger::sequencer::Rejected>) -> Result<u64, String> {
        match r {
            Ok(e) => Ok(e),
            // **A duplicate at the sink is not a commit here.** The sequencer's window says
            // "this key committed at epoch *e*"; the in-memory base has its own window and
            // has already accepted these rows as fresh, so returning `Ok` let the same
            // transaction be applied twice — once now, once when the original was applied —
            // and after a restart, when the sink's window survived and the base's did not,
            // the two disagreed on every retried key. The base's window is the one `append`
            // consults before it gets here; if it said fresh and the sink says duplicate, the
            // two are out of step and that is the fact to report.
            Err(nilestream_ledger::sequencer::Rejected::Duplicate { at_epoch }) => Err(format!(
                "the sink has already committed this idempotency key at epoch {at_epoch}, \
                 while the base accepted it as new. The two idempotency windows disagree."
            )),
            Err(e) => Err(format!("{e:?}")),
        }
    }
}

/// A submitted transaction whose barrier has not yet returned.
type PendingDurable =
    std::sync::mpsc::Receiver<Result<u64, nilestream_ledger::sequencer::Rejected>>;

/// `parent ‖ hash ‖ canon(rows)` — the inverse of what `append` writes.
///
/// Strict: anything shorter than the two links, or whose row bytes are not exactly
/// `encode_rows`'s output, is `None` and stops recovery. A decoder that guessed would rebuild
/// a ledger nobody wrote, and the hash check downstream would then be comparing two things
/// this process had invented.
fn decode_record(p: &[u8]) -> Option<([u8; 32], [u8; 32], Vec<proto_engine::Row>)> {
    let parent: [u8; 32] = p.get(..32)?.try_into().ok()?;
    let hash: [u8; 32] = p.get(32..64)?.try_into().ok()?;
    let rows = proto_engine::ledger::decode_rows(p.get(64..)?)?;
    Some((parent, hash, rows))
}

impl RevEngine {
    /// Build an engine holding `accounts` accounts, each seeded with `postings_per_account`
    /// balanced transfers against a house account.
    ///
    /// Seeded rather than empty because a benchmark against an empty server reports excellent
    /// latencies for queries that return nothing, and because a partial view over a base with
    /// no history has nothing to reconstruct — the miss path, which is the interesting one,
    /// would never run.
    /// Apply the schema's declared idempotency window, in epochs, to the base's admission
    /// index.
    ///
    /// The other half is [`with_durable_bounded`](Self::with_durable_bounded), which gives
    /// the same window to the sealer. Both are needed: a durable daemon holds two indexes and
    /// every committed identity is in both.
    pub fn with_idem_window(self, epochs: Option<u64>) -> Self {
        if let Some(w) = epochs {
            let ledger = std::mem::take(&mut *self.ledger.write().expect("not poisoned"));
            *self.ledger.write().expect("not poisoned") = ledger.with_idem_window(w);
        }
        self
    }

    /// How many identities the base's idempotency index holds, and the window it was
    /// declared with. `nilestream_stats.idem_window_keys`.
    pub fn idem_window(&self) -> (usize, Option<u64>) {
        let l = self.base();
        (l.idem_window_keys(), l.idem_window())
    }

    pub fn seeded(
        accounts: i64,
        postings_per_account: u32,
        budget: usize,
        mode: ViewMode,
        policy: EvictionPolicy,
    ) -> RevEngine {
        let mut ledger = Ledger::new();
        let mut txn = 0u64;
        for round in 0..postings_per_account {
            for a in 1..=accounts as u64 {
                txn += 1;
                // Two conserved legs: the account and the house. The commit rule quantifies
                // over currency, so an unbalanced pair would be refused here rather than
                // discovered as a wrong answer later.
                let amt = 100 + (round as i128 * 7 + a as i128 % 13);
                let rows = vec![
                    Row::Post(Posting {
                        txn,
                        acct: a,
                        cur: 0,
                        amt,
                        valid: round as i64,
                    }),
                    Row::Post(Posting {
                        txn,
                        acct: 0,
                        cur: 0,
                        amt: -amt,
                        valid: round as i64,
                    }),
                ];
                let _ = ledger.submit(&format!("seed-{txn}"), rows);
            }
        }

        let _ = (mode, policy);
        // The REV runtime, dragged to the same frontier. `advance` applies each epoch's
        // deltas to whichever entries are *resident*, which is the whole saving partiality
        // buys and is why this is cheap over twenty thousand epochs on an empty view.
        let mut runtime = install_balance_view(budget);
        if let Some(rt) = runtime.as_mut() {
            let head = {
                use nilestream_core::rev::Base;
                ledger.frontier()
            };
            for e in 0..=head {
                rt.advance(&ledger, e);
            }
        }

        RevEngine {
            ledger: std::sync::RwLock::new(ledger),
            currency: 0,
            views: vec![("__wire_result".to_string(), 2)],
            scan_work: std::sync::atomic::AtomicU64::new(0),
            served: std::sync::atomic::AtomicU64::new(0),
            served_rows: std::sync::atomic::AtomicU64::new(0),
            runtime: runtime.map(std::sync::Mutex::new),
            currencies: std::sync::RwLock::new(std::collections::BTreeSet::from([0])),
            view_answers: std::sync::atomic::AtomicU64::new(0),
            view_fallbacks: std::sync::atomic::AtomicU64::new(0),
            joins_answered: std::sync::atomic::AtomicU64::new(0),
            joins_retried: std::sync::atomic::AtomicU64::new(0),
            durable: None,
            visible: None,
        }
    }

    /// Drop everything the read model holds, so the next reads all miss.
    ///
    /// The lever the phase diagram is swept with: measuring at a 0% miss rate and at a high
    /// one are different experiments, and a server that could only be measured warm would
    /// only ever produce the flattering half.
    ///
    /// It now wipes **the view the wire path reads**. It used to wipe the other one, so a
    /// test could evict, observe a rising miss rate, and be measuring a cache no query
    /// consulted.
    pub fn evict_all(&mut self) {
        if let Some(rt) = self.runtime.as_mut() {
            if let Some(v) = rt
                .get_mut()
                .expect("the view lock is not poisoned")
                .view_mut(BALANCE_VIEW)
            {
                v.wipe();
            }
        }
    }

    pub fn head(&self) -> u64 {
        self.base().head()
    }

    /// **The currencies this fold would add together, if it would add any.**
    ///
    /// `Some(list)` when the plan sums `amt`, groups without the currency column, and does not
    /// pin one in its predicate, over a base that holds more than one. `None` — meaning "serve
    /// it" — in every other case, and each exclusion is a way the sum is well defined rather
    /// than a way it is convenient:
    ///
    /// * the aggregate is not `sum(amt)` — a `count` or a `min` over mixed currencies is a
    ///   question about rows, not about money;
    /// * the group key contains the currency column, so each group holds one currency;
    /// * the predicate pins `cur = k`, so the scan sees one currency whatever the key is;
    /// * the base holds one currency, so there is nothing to mismatch.
    ///
    /// Note what is *not* here: the schema's declared set. This asks what the base actually
    /// holds, because a schema may declare three currencies and a base hold one, and refusing
    /// that fold would refuse the ordinary case for a hypothetical.
    fn cross_currency_fold(&self, p: &crate::scan_fold::FoldPlan) -> Option<Vec<u32>> {
        use niles_ir::operator::{Agg, Scalar};
        const CUR: u16 = 2;
        const AMT: u16 = 3;

        if p.aggs() != [(Agg::Sum, Scalar::Column(AMT))] {
            return None;
        }
        if p.group_key().contains(&CUR) {
            return None;
        }
        if p.column_restriction(CUR).is_some() {
            return None;
        }
        let held = self
            .currencies
            .read()
            .expect("the currency set is not poisoned");
        (held.len() > 1).then(|| held.iter().copied().collect())
    }

    /// How many currencies the base holds.
    fn currency_count(&self) -> usize {
        self.currencies
            .read()
            .expect("the currency set is not poisoned")
            .len()
    }

    /// The one currency in the base, or `None` where there is not exactly one.
    fn sole_currency(&self) -> Option<u32> {
        let c = self
            .currencies
            .read()
            .expect("the currency set is not poisoned");
        (c.len() == 1).then(|| *c.iter().next().expect("one currency"))
    }

    /// One attempt at a query: everything the public `query` does, except that a join is
    /// **returned** rather than waited on.
    ///
    /// The split exists so that a caller reaching this engine through a lock can drop that
    /// lock before it waits — see `query`'s own comment (A10-04). `try_view` is `false` once
    /// the bounded retries are spent, which sends the read to the fold: slower, exact, and
    /// never a spin.
    fn query_step(
        &self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
        try_view: bool,
    ) -> QueryStep {
        // **Predicate pushdown into the source scan.** Without it every query materialises
        // the whole base: the point workload went from ~12,000 ops/s to 68 when the server
        // started evaluating the circuit instead of answering from a hard-coded fold, and
        // 68 is what a full scan of twenty thousand postings per query costs.
        //
        // This is not a second answer. Restricting the source to the rows a `Filter` in the
        // circuit would keep cannot change what the circuit denotes — the filter is still
        // there and still applied — and the restriction goes through the anchor index,
        // which is the mechanism §9.4.1 measures at two orders of magnitude.
        // `the_pushdown_and_the_full_scan_agree` holds the two to the same answer.
        // **The fast path: fold the base, never materialise it.**
        //
        // `scan_fold::plan` recognises the keyed-aggregate fragment and refuses everything
        // else, so this is a *fast path for one shape* rather than a second engine. What it
        // computes is the aggregate node's value; everything above that — `order by`,
        // `limit` — is evaluated by the reference from that value, so those operators keep
        // one semantics and cannot drift from the golden corpus.
        let planned = crate::scan_fold::plan(circuit, output);
        // **A single account's balance is a read of the maintained view, not a scan.**
        //
        // This is the mechanism the phase diagram is about, and until now nothing on the wire
        // used it: every read was an anchored reconstruction and the `miss_rate` column read
        // 1.00 by construction rather than by measurement.
        // **The account restriction comes from the plan where there is one.** The
        // circuit-level version below understands only a filter whose *whole* predicate is
        // `acct = k`; the plan's version splits conjuncts and sees a `having` that has been
        // pushed below the aggregate, which is two cliffs the same query used to fall off.
        let restricted = planned
            .as_ref()
            .and_then(|p| p.column_restriction(ACCT_COL))
            .or_else(|| account_predicate(circuit));
        // **The view answers only when the account is the whole of the restriction.**
        // `restricted` above is enough to narrow a *scan*, because the circuit's filters are
        // applied to whatever the scan returns. It is not enough to answer from the balance
        // view, which is keyed by account and currency and knows nothing about a query's
        // other conditions: `where acct = 7 and cur = 99` selects no rows, and answering it
        // from the view returned account 7's balance. A test found that, not a reading.
        //
        // The condition is `serve_path`'s, so `explain` and the engine cannot describe
        // different engines: one function decides, and both read it.
        // **A sum over a base that holds more than one currency is refused, not folded.**
        //
        // Contribution 4 says money cannot be mismatched. The compiler discharges that for
        // programs; this is the same obligation on data, and it was undischarged: with 324 USD
        // and 500 of a second currency in the base, `sum(amt) group by acct` folded both into
        // "824" and served it as an account's balance. Adding two currencies is not a slow
        // answer or an imprecise one — it is a number that denotes nothing, and
        // `conserve per (txn, cur)` is quantified per currency precisely so that it never has
        // to be computed.
        //
        // `report_from_view` already refused this shape and fell through to here, which is why
        // the refusal has to live on the fold rather than on the view: the fold is what the
        // view falls back *to*. The query is answerable in two ways and both are offered by
        // name — group by the currency, or name one in the predicate — so this narrows what
        // the server will answer without narrowing what a caller can ask.
        if let Some(p) = planned.as_ref() {
            if let Some(offending) = self.cross_currency_fold(p) {
                return QueryStep::Done(Err(crate::session::ServeError::CrossCurrency(offending)));
            }
        }
        // **A report first**, because a fully maintained view answers one without touching
        // the base at all. Refused for every shape that is not one — see `report_from_view`.
        if let Some(p) = planned.as_ref() {
            if let Some(rows) = self.report_from_view(p, circuit, output, anchor) {
                self.served
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return QueryStep::Done(Ok(rows));
            }
        }
        let sole = planned
            .as_ref()
            .and_then(|p| p.sole_account_filter(ACCT_COL));
        if let (true, Some(p), Some(acct), ServePath::View) = (
            try_view,
            planned.as_ref(),
            sole,
            serve_path_of(planned.as_ref(), circuit, output),
        ) {
            match self.answer_from_view(p, circuit, output, acct, anchor) {
                ViewAnswer::Rows(rows) => return QueryStep::Done(Ok(rows)),
                // The fold below answers it.
                ViewAnswer::NotApplicable => {}
                // **Handed out, not waited on here.** This function may be running under a
                // caller's lock; the wait belongs where nothing is held.
                ViewAnswer::Wait(plan) => return QueryStep::Wait(plan),
            }
        }
        // Counted here rather than on entry: a query the maintained view answered is not a
        // read of the scan surface, and adding it to both totals double-counted every one.
        self.served
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (z, work) = match planned {
            Some(p) if p.relation == "postings" => {
                // **Streamed, not buffered.** Collecting the base into a `Vec` first cost
                // 10MB of vector growth per query — more than the Z-set this path exists to
                // avoid building — so the scan pushes rows into the fold one at a time and
                // nothing holds the base at all.
                let mut folder = crate::scan_fold::Folder::new(&p);
                let mut scanned = 0u64;
                self.scan(restricted, anchor, |row| {
                    scanned += 1;
                    folder.row(&row[..]);
                });
                self.served_rows
                    .fetch_add(scanned, std::sync::atomic::Ordering::Relaxed);
                // **A refused fold is a refused query, over the wire, with a diagnostic.**
                // The alternative this replaces was not an alternative: the fold could not
                // fail, because `x / 0` evaluated to `0` and was summed into the answer.
                let (folded, w) = match folder.finish() {
                    Ok(v) => v,
                    Err(e) => {
                        return QueryStep::Done(Err(crate::session::ServeError::Eval(
                            e.to_string(),
                        )))
                    }
                };
                // **When the aggregate *is* the output, the fold has already answered.**
                //
                // Handing it to the reference evaluator as a precomputed node and asking for
                // the output cost a full deep clone of the Z-set — ten thousand row vectors
                // and a tree rebuilt — because `try_run_node_with` returns a `Cow` and
                // `into_owned` at its boundary copies a borrowed one. That is the right
                // shape for the general case and pure waste for the common one, which is a
                // plain `group by` with nothing above it.
                //
                // Where something *is* above it — an `order by`, a `limit` — the reference
                // still evaluates it from the folded value, so those operators keep exactly
                // one semantics.
                if circuit.outputs.get(output) == Some(&p.node) {
                    (folded, w)
                } else {
                    let mut given = std::collections::BTreeMap::new();
                    given.insert(p.node, folded);
                    let r = niles_ir::eval::try_run_with(
                        circuit,
                        output,
                        &std::collections::BTreeMap::new(),
                        &given,
                    )
                    .map_err(|e| crate::session::ServeError::Eval(e.to_string()));
                    let (z, w2) = match r {
                        Ok(v) => v,
                        Err(e) => return QueryStep::Done(Err(e)),
                    };
                    (z, w + w2)
                }
            }
            // The materialising path, still here and still correct: it is what answers every
            // shape outside the fragment, and it is the oracle the fast path is tested
            // against.
            _ => {
                let sources = match restricted {
                    Some(acct) => self.base_for_account(acct, anchor),
                    None => self.base_at(anchor),
                };
                self.served_rows.fetch_add(
                    sources.values().map(|z| z.len() as u64).sum::<u64>(),
                    std::sync::atomic::Ordering::Relaxed,
                );
                match niles_ir::eval::try_run(circuit, output, &sources)
                    .map_err(|e| crate::session::ServeError::Eval(e.to_string()))
                {
                    Ok(v) => v,
                    Err(e) => return QueryStep::Done(Err(e)),
                }
            }
        };
        self.scan_work
            .fetch_add(work, std::sync::atomic::Ordering::Relaxed);

        // Column names from the lowering are not available here — the circuit is the
        // contract between the two, and it carries indices rather than names — so the
        // columns are named positionally and the *anchor* is appended, because a served
        // answer without the moment it is true at is a number a dispute cannot use.
        //
        // **The width comes from the circuit, not from the first row.** It was
        // `z.keys().next().map(|r| r.len()).unwrap_or(0)`, so a query that returned nothing
        // described *one* column where the same query with rows described three: a client
        // saw a different schema for the same statement depending on the data. A result set
        // is described by the query, and an empty one is still a result set.
        let width = circuit
            .outputs
            .get(output)
            .and_then(|id| circuit.nodes.iter().find(|n| n.id == *id))
            .map(|n| n.arity as usize)
            .filter(|w| *w > 0)
            .unwrap_or_else(|| z.keys().next().map(|r| r.len()).unwrap_or(0));
        let mut columns: Vec<String> = (0..width).map(|i| format!("c{i}")).collect();
        columns.push("anchor".into());
        // **The Z-set is handed over, not copied out.** Rendering it here meant a `Vec` per
        // row and a `String` per cell — ten thousand rows rebuilt out of integers that were
        // already integers, immediately before the wire layer parsed them back. The framer
        // reads them where they lie; `Rows::text` is still there for a caller that wants the
        // text form, and the tests use it.
        QueryStep::Done(Ok(crate::session::Rows {
            columns,
            rows: crate::session::RowSource::Evaluated { z, anchor },
        }))
    }

    /// Count how a join ended. Split out so an implementation that waits outside its own
    /// lock can still report the outcome through a short, separate acquisition.
    fn note_join(&self, answered: bool) {
        let counter = if answered {
            &self.joins_answered
        } else {
            &self.joins_retried
        };
        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if answered {
            self.view_answers
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Wait for a joined flight with **nothing held**, and turn its answer into the reply.
    ///
    /// `None` means the flight went away or published at another anchor; the caller tries
    /// again, and past [`MAX_JOIN_ATTEMPTS`] the fold answers.
    fn resolve_join(&self, plan: WaitPlan, anchor: u64) -> Option<crate::session::Rows> {
        let WaitPlan {
            ticket,
            columns,
            acct,
            cur,
            with_currency,
        } = plan;
        match ticket.wait() {
            Some(joined) => {
                self.note_join(true);
                // **The joined answer is the answer.** It is exact at this caller's anchor —
                // `WaitTicket::wait` returns `None` for a completion published at any other —
                // so returning it is not an optimisation over asking again; it is the only
                // reading in which joining costs the second reader less than folding.
                //
                // No base read: `base_rows` travelled with the answer, which is what lets
                // this distinguish "no such account" from "a balance of zero" without
                // acquiring B a second time for one logical read.
                Some(Self::rows_from_answer(
                    columns,
                    acct,
                    cur,
                    with_currency,
                    joined,
                    anchor,
                ))
            }
            None => {
                self.note_join(false);
                None
            }
        }
    }

    /// A shared borrow of the base. Every read path goes through here, and every one of
    /// them is counted, so `select nilestream_lock` describes the structure that is actually
    /// there rather than the mutex that used to be.
    fn base(&self) -> crate::lockstats::TimedRead<'_, Ledger> {
        crate::lockstats::TimedRead::acquire(&self.ledger, &crate::lockstats::ENGINE_LOCK)
    }
}

impl crate::session::Serving for RevEngine {
    fn serve_path_now(
        &self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> &'static str {
        self.serve_path_at(circuit, output, anchor).as_str()
    }

    /// **The frontier a read anchors at, which is the durable one.**
    ///
    /// Not `ledger.head()`. The base may hold an epoch whose barrier has not returned; that
    /// epoch is sealed and not yet visible, and answering a read at it would let a client
    /// observe a transaction a crash could still erase. In a ledger an observation that is
    /// later erased is not a stale read — it is a transaction a customer saw succeed and
    /// that no longer exists.
    fn frontier(&self) -> u64 {
        match &self.visible {
            Some(v) => v.load(std::sync::atomic::Ordering::Acquire),
            // No durable sink: nothing to be durable *before*, so the base's head is the
            // frontier and always was.
            None => self.base().head(),
        }
    }

    /// **Evaluate the circuit the client's query compiled to.**
    ///
    /// This is what F-16 was about. The old path compiled the query, verified it, and then
    /// *discarded the circuit*: `pick_view` returned the constant `"__wire_result"`, this
    /// method ignored the view name entirely, read `key[0]`, and folded `sum(amt)` for
    /// currency 0. Every query over one account returned the same number, whatever it
    /// asked for. The compiler was decoration on a hard-coded answer, and the §6.9 claim —
    /// "a wire protocol is a surface, not a semantics" — was false in the direction that
    /// matters: the surface was accepting queries the semantics never saw.
    ///
    /// The base is materialised as a Z-set at the anchor and the circuit is evaluated over
    /// it by the same `niles_ir::eval` the golden corpus uses. That is a full fold, and it
    /// is the honest cost of an arbitrary query against a partial-state engine: the
    /// partially-materialised view is a *fast path for one shape*, not a general answer, and
    /// `query_step` below is where it is spent; it answers through the same REV runtime the
    /// phase diagram measures rather than through a cache beside it.
    ///
    /// **The retry loop is out here, and one attempt is one call — A10-04.**
    ///
    /// It used to live inside the body, which meant a caller reaching this engine *through a
    /// lock* held that lock for every attempt, including the wait.
    /// `Serving for RwLock<RevEngine>` does exactly that: `self.read()`, then delegate. The
    /// benchmark's hosted daemon builds one, and its sweep calls `reseed`, which takes the
    /// same lock **exclusively** — so a session parked on another reader's fold held O shared
    /// while a reseed queued behind it, and `std::sync::RwLock` is writer-preferring on both
    /// hosts this project measures (170/200 on Linux, `c10-rwlock.sh`), which puts every
    /// later reader behind that writer.
    ///
    /// With one attempt per call, the outer implementation drops its guard, waits, and takes
    /// the guard again. Re-deriving the plan per attempt is the cost: bounded, and paid only
    /// on the rare join that has to retry at all.
    fn query(
        &self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> Result<crate::session::Rows, crate::session::ServeError> {
        let mut attempts = 0u32;
        loop {
            attempts += 1;
            match self.query_step(circuit, output, anchor, attempts <= MAX_JOIN_ATTEMPTS) {
                QueryStep::Done(r) => return r,
                QueryStep::Wait(plan) => {
                    if let Some(rows) = self.resolve_join(plan, anchor) {
                        return Ok(rows);
                    }
                }
            }
        }
    }

    fn append(
        &self,
        rows: Vec<Row>,
        txn_id: &str,
    ) -> Result<crate::session::Appended, crate::session::ServeError> {
        // **The write guard, taken once and held across the apply.** It serialises
        // submitters, which is what makes the epoch ids it assigns contiguous, and it is
        // released before the barrier is waited on — the whole of what T-05 established.
        let mut base =
            crate::lockstats::TimedWrite::acquire(&self.ledger, &crate::lockstats::ENGINE_LOCK);
        let epoch = base.submit(txn_id, rows).map_err(|e| match e {
            // The two are different failures and a client acts on them differently: a
            // duplicate means the earlier attempt succeeded and the retry must stop; an
            // unbalanced set means the caller built something that does not conserve.
            proto_engine::Reject::Duplicate => {
                crate::session::ServeError::Duplicate(format!("`{txn_id}` already committed"))
            }
            other => crate::session::ServeError::Rejected(format!("{other:?}")),
        })?;
        // **The barrier is submitted here and waited for somewhere else.**
        //
        // This used to be `sink.record(...)`, which blocks until the epoch is on stable
        // storage — inside the engine's lock, because `append` is called with the engine
        // locked. Two things followed. Every other connection, reads included, waited behind
        // one disk barrier; and no *second* submitter could reach the sealer, so its drain
        // loop always found an empty queue and group commit — built, and tested at sixteen
        // concurrent submitters — never formed a batch. `select nilestream_sealer` reported
        // `max_batch` 1 and 1.00 transactions per fsync at every connection count.
        //
        // The submission is non-blocking; the token is returned to the caller, which waits
        // on it *after* releasing the lock and before writing the reply. So the
        // acknowledgement still follows the barrier — the client is told "committed" only
        // once it is — while the lock is held for the apply alone.
        let mut receipt: Option<Pending> = None;
        if let Some(sink) = self.durable.as_ref() {
            // **The payload is the epoch, not its number.** `parent ‖ hash ‖ canon(rows)`:
            // the rows, so a restart can rebuild the base, and the ledger's own chain link,
            // so a restart can *verify* what it rebuilt rather than take it on trust. The
            // segment's chain protects these bytes; the bytes carry the ledger's chain; both
            // are the same hash over the same canonical encoding.
            //
            // It used to be `epoch.to_string()`. A restart recovered the idempotency window
            // and not one row.
            let link = base
                .epochs
                .get(epoch as usize)
                .expect("the epoch just submitted is the last one");
            let mut payload = Vec::with_capacity(64 + link.rows.len() * 48);
            payload.extend_from_slice(&link.parent);
            payload.extend_from_slice(&link.hash);
            payload.extend_from_slice(&proto_engine::ledger::encode_rows(&link.rows));
            let token = sink
                .record_pending(txn_id, payload)
                .map_err(crate::session::ServeError::NotDurable)?;
            // **To the caller, not to a list on the engine.** The list was drained by
            // whichever connection asked next (A9-F01); a value returned from this call
            // belongs to the call.
            receipt = Some(Pending {
                epoch,
                token,
                visible: self
                    .visible
                    .clone()
                    .expect("a durable sink implies a visible frontier"),
            });
        }
        // The maintained view moves with the ledger, or the next read answers at an anchor
        // the base has already passed. `advance` touches only *resident* entries, which is
        // the saving partiality buys and is why this is not a per-epoch scan of the key
        // space.
        if let Some(rt) = self.runtime.as_ref() {
            crate::lockstats::Timed::acquire(rt, &crate::lockstats::VIEW_LOCK)
                .advance(&*base, epoch);
        }
        if let Some(e) = base.epochs.get(epoch as usize) {
            let mut cur = self
                .currencies
                .write()
                .expect("the currency set is not poisoned");
            for r in &e.rows {
                if let proto_engine::Row::Post(p) = r {
                    cur.insert(p.cur);
                }
            }
        }
        Ok(crate::session::Appended { epoch, receipt })
    }

    fn views(&self) -> Vec<(String, u32)> {
        self.views.clone()
    }

    fn sealer_stats(&self) -> Option<(u64, u64, u64, u64, u64)> {
        let s = self.durable.as_ref()?.stats();
        Some((
            s.epochs_sealed,
            s.txns_committed,
            s.fsyncs,
            s.duplicates_absorbed,
            s.max_batch,
        ))
    }

    fn durability(&self) -> &'static str {
        match self.durable {
            Some(_) => "always",
            None => "none",
        }
    }

    /// `(reads, hits, misses, rows_touched, resident)` for the **served** path.
    ///
    /// Hits are zero and misses equal reads, and that is not a placeholder: the served path
    /// evaluates the compiled circuit over a source scan through the anchor index, so every
    /// read is an anchored reconstruction and no resident entry is consulted. Reporting the
    /// `PartialView`'s counters here would report a view the wire path does not read — they
    /// would all be zero, and a benchmark would record `n/a` and move on.
    fn read_stats(&self) -> ReadStats {
        // **Hits are counted now, where they used to be zero by construction.**
        //
        // This returned `(served, 0, served, rows, resident)` with a paragraph explaining
        // that the served path never consults a resident entry — true when it was written,
        // and it made the `miss_rate` column of every benchmark row 1.00 whatever the engine
        // did. A single-account read is now answered by the REV runtime, so its hits and
        // misses are the runtime's, and the scan surface's reads are added to both totals so
        // that a mixed workload's rate is over everything the server answered.
        let served = self.served.load(std::sync::atomic::Ordering::Relaxed);
        let served_rows = self.served_rows.load(std::sync::atomic::Ordering::Relaxed);
        // **The base phase, complete before the view is touched — A10-01 / F-10-01.**
        //
        // These two statements used to be the other way round: V was acquired, and then
        // `self.base()` reached for B underneath it. `append` takes B exclusively and then
        // takes V, so a stats request from one connection and an append from another form
        // the cycle B→V / V→B and neither ever returns. It is reachable from the wire —
        // `select nilestream_stats` is an ordinary statement — and the whole of cycle 9's
        // mixed sweep queries this table between levels, so the harness that measures the
        // engine could hang the engine it was measuring.
        //
        // The order is B < P < V < C and this path now takes a subsequence of it: B is
        // acquired, read and released on the line below, and V is acquired after. Not nested
        // at all, which is stronger than nested in the right order.
        //
        // **What that costs, stated rather than hidden: this snapshot is not atomic.** The
        // idempotency-window size is read at one instant and the view's counters at another,
        // and an append may land between them, so the row can pair a window size with view
        // counters that are one epoch newer. That is the correct trade for a diagnostic
        // table — the alternative is holding both locks across the read, which is the
        // deadlock — and it is stated here because a reader who assumes atomicity would
        // read a one-epoch skew as a counter that does not reconcile.
        let idem_keys = self.base().idem_window_keys() as u64;
        crate::rev_engine::stats_order_hook::pause();
        let guard = self
            .runtime
            .as_ref()
            .map(|rt| crate::lockstats::Timed::acquire(rt, &crate::lockstats::VIEW_LOCK));
        let view_answers = self.view_answers.load(std::sync::atomic::Ordering::Relaxed);
        let fallbacks = self
            .view_fallbacks
            .load(std::sync::atomic::Ordering::Relaxed);
        // Read from the runtime rather than from the environment a second time: what the
        // transcript must name is the value the views are actually running with.
        let v_caps = guard.as_ref().map(|rt| rt.merge_caps()).unwrap_or_default();
        match guard.as_ref().and_then(|rt| rt.view(BALANCE_VIEW)) {
            Some(v) => {
                let s = &v.stats;
                ReadStats {
                    reads: s.reads + served,
                    hits: s.hits,
                    misses: s.misses + served,
                    rows_touched: s.base_rows_read + served_rows,
                    resident: v.resident_count() as usize,
                    view_answers,
                    fallbacks,
                    view_metadata_keys: v.metadata_len() as u64,
                    idem_window_keys: idem_keys,
                    pending_joins: s.pending_joins,
                    uninstalled_folds: s.uninstalled_folds,
                    pinned_installs: s.pinned_installs,
                    flights_refused: s.flights_refused,
                    deferred_merges: s.deferred_merges,
                    merge_rows_visited: s.merge_rows_visited,
                    merge_epochs_merged: s.merge_epochs_merged,
                    merge_max_epochs: v_caps.max_epochs,
                    merge_max_rows: v_caps.max_rows,
                    merges_refused_epochs: s.merges_refused_epochs,
                    merges_refused_rows: s.merges_refused_rows,
                    merges_refused_unavailable: s.merges_refused_unavailable,
                    waiters_refused: s.waiters_refused,
                    joins_answered: self
                        .joins_answered
                        .load(std::sync::atomic::Ordering::Relaxed),
                    joins_retried: self
                        .joins_retried
                        .load(std::sync::atomic::Ordering::Relaxed),
                    gap_at_begin_total: s.gap_at_begin_total,
                    gap_at_finish_total: s.gap_at_finish_total,
                    gap_at_finish_max: s.gap_at_finish_max,
                    gap_begin_samples: s.gap_begin_samples,
                    gap_finish_samples: s.gap_finish_samples,
                    flights_behind_at_begin: s.flights_behind_at_begin,
                    flights_that_fell_behind: s.flights_that_fell_behind,
                }
            }
            // No runtime: the server has no partial state at all, so nothing is resident
            // and every served read is a reconstruction over the base.
            None => ReadStats {
                reads: served,
                misses: served,
                rows_touched: served_rows,
                view_answers,
                fallbacks,
                idem_window_keys: idem_keys,
                ..Default::default()
            },
        }
    }
}

/// **A swappable engine, for a harness that measures more than one size.**
///
/// The daemon holds its engine behind a shared borrow and never replaces it; a benchmark
/// sweeping the base across two decades has to. This adapter is the whole difference: it
/// forwards every `Serving` method through a shared borrow — so reads still run
/// concurrently — and `reseed` takes the exclusive one to put a different engine in place.
impl crate::session::Serving for std::sync::RwLock<RevEngine> {
    fn frontier(&self) -> u64 {
        self.read().expect("not poisoned").frontier()
    }
    /// **The outer lock is dropped before the wait — A10-04.**
    ///
    /// This was `self.read().expect(..).query(..)`, one call, so the shared acquisition of
    /// **O** was held for the whole of the inner retry loop *including* the wait on another
    /// reader's fold. Nothing deadlocks on that on its own; what it does is hold O shared for
    /// the length of somebody else's reconstruction. The benchmark's hosted daemon builds one
    /// of these and its sweep calls `reseed`, which takes O **exclusively** — and
    /// `std::sync::RwLock` is writer-preferring on both hosts this project measures (170/200
    /// on Linux, `c10-rwlock.sh`), so the queued reseed then stops every reader that arrives
    /// behind it, for as long as one parked reader's flight takes.
    ///
    /// One attempt per acquisition. The guard is taken, a step is run, the guard is dropped —
    /// and only then does this wait. `query_step` re-derives the plan each time, which is the
    /// price of not holding a lock across another thread's work.
    fn query(
        &self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> Result<crate::session::Rows, crate::session::ServeError> {
        let mut attempts = 0u32;
        loop {
            attempts += 1;
            // The guard's scope is this statement and nothing more.
            let step = self.read().expect("not poisoned").query_step(
                circuit,
                output,
                anchor,
                attempts <= MAX_JOIN_ATTEMPTS,
            );
            match step {
                QueryStep::Done(r) => return r,
                QueryStep::Wait(plan) => {
                    // **O is released before the wait, and taken again only to count.**
                    // Destructuring here is what makes that possible: the ticket, the
                    // columns and the key are owned values, so nothing in the wait borrows
                    // through a guard.
                    let WaitPlan {
                        ticket,
                        columns,
                        acct,
                        cur,
                        with_currency,
                    } = plan;
                    let joined = ticket.wait();
                    self.read()
                        .expect("not poisoned")
                        .note_join(joined.is_some());
                    if let Some(j) = joined {
                        return Ok(RevEngine::rows_from_answer(
                            columns,
                            acct,
                            cur,
                            with_currency,
                            j,
                            anchor,
                        ));
                    }
                }
            }
        }
    }
    fn append(
        &self,
        rows: Vec<Row>,
        txn_id: &str,
    ) -> Result<crate::session::Appended, crate::session::ServeError> {
        self.read().expect("not poisoned").append(rows, txn_id)
    }
    fn views(&self) -> Vec<(String, u32)> {
        self.read().expect("not poisoned").views()
    }
    fn durability(&self) -> &'static str {
        self.read().expect("not poisoned").durability()
    }
    fn read_stats(&self) -> ReadStats {
        self.read().expect("not poisoned").read_stats()
    }
    fn sealer_stats(&self) -> Option<(u64, u64, u64, u64, u64)> {
        self.read().expect("not poisoned").sealer_stats()
    }
    fn serve_path_now(
        &self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> &'static str {
        self.read()
            .expect("not poisoned")
            .serve_path_now(circuit, output, anchor)
    }
}

impl RevEngine {
    /// Attach a durable sink, so every append reaches stable storage before it is
    /// acknowledged. Refuses any policy but `Always`, which the sequencer also refuses.
    pub fn with_durable(self, path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        self.with_durable_bounded(path, None)
    }

    /// `with_durable`, with the idempotency window the schema declared, in epochs.
    ///
    /// The window reaches **both** indexes: the ledger's admission map here and the sealer's
    /// own, through [`DurableSink::open_bounded`]. Two structures held every identity ever
    /// committed until T-05 (cycle 8), at a measured 68.8 B and 99.6 B each (E18), and a
    /// window that bounded one of them would have moved the leak rather than closed it.
    pub fn with_durable_bounded(
        mut self,
        path: impl AsRef<std::path::Path>,
        idem_window: Option<u64>,
    ) -> std::io::Result<Self> {
        // **The seeded prefix, before anything is replayed.** Seeding is not durable and is
        // not meant to be: it is a deterministic function of `(accounts, rounds)` that both
        // the original process and this one ran. Every recovered transaction lands on top of
        // it, in order, so the k-th recovered transaction is ledger epoch `seed_epochs + k`,
        // where `seed_epochs` is how many the seeded prefix has — not its head, which is one
        // less.
        //
        // **A record is a batch and a batch is many transactions**, so a record's own epoch is
        // not that index. Counting records here would place every recovered row at the wrong
        // epoch under group commit — which is to say, under every workload the sealer exists
        // for.
        let seed_epochs = self.base().epochs.len() as u64;
        let (sink, recovery) = DurableSink::open_bounded(path, idem_window)?;
        // **The checked form.** `recover_txns` returns what it could read; a restart must
        // refuse what it could not. A record whose batch envelope ends early loses the
        // transactions after the truncation, and replaying the remainder onto a prefix that
        // is missing an epoch rebuilds a ledger nobody wrote — silently, and with a success
        // exit code, which is the worst way for a durability claim to be false.
        let txns = nilestream_ledger::sequencer::Sequencer::recover_txns_checked(&recovery)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "recovered transaction stream is not decodable: {e}. Recovery stops \
                         here rather than replaying the part it could read."
                    ),
                )
            })?;

        {
            let mut base = self.ledger.write().expect("the base lock is not poisoned");
            for (k, (idem_key, payload)) in txns.iter().enumerate() {
                let expect = seed_epochs + k as u64;
                let Some((parent, hash, rows)) = decode_record(payload) else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "recovered transaction {k} (`{idem_key}`) is not a decodable epoch \
                             record. Recovery stops here rather than continuing past a record \
                             it cannot read: replaying a suffix onto a prefix missing an epoch \
                             rebuilds a ledger nobody wrote."
                        ),
                    ));
                };
                let got = base.submit(idem_key, rows).map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("recovered transaction {k} (`{idem_key}`) was refused: {e:?}"),
                    )
                })?;
                if got != expect {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "recovered transaction {k} landed at epoch {got}, not {expect} = \
                             seed_epochs({seed_epochs}) + {k}. The seeded prefix this process \
                             built is not the one these records were written against."
                        ),
                    ));
                }
                // **Verified, not merely rebuilt.** `submit` recomputed the chain link from
                // the rows and the prefix; the record says what that link was when the epoch
                // was committed. Equality means every earlier epoch is also as it was, because
                // the link is taken over the parent. A chain rebuilt on replay and compared to
                // nothing protects nothing.
                let rec = base.epochs.get(got as usize).expect("just submitted");
                if rec.parent != parent || rec.hash != hash {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "the chain does not verify at recovered epoch {got} (`{idem_key}`): \
                             the record commits to a different link than this prefix computes. \
                             The segment, the seeded prefix, or a row has changed since it was \
                             written."
                        ),
                    ));
                }
            }
        }

        // The visible frontier starts at the recovered head: everything replayed above was
        // acknowledged before the restart, so it is durable by definition. Everything
        // appended from here earns its visibility by being fsynced.
        let head = self.base().head();
        self.durable = Some(sink);
        self.visible = Some(std::sync::Arc::new(std::sync::atomic::AtomicU64::new(head)));
        Ok(self)
    }

    /// Whether this query is a report the maintained view can answer, and in which spelling.
    ///
    /// `Some(true)` for `group by acct, cur`, `Some(false)` for `group by acct`, `None` when
    /// the fold must answer. Split out of [`report_from_view`] so `explain` can say which
    /// path a statement will take **without running it and without a second opinion**: the
    /// benchmark's report row claims to be served by a maintained view, and a reader who
    /// wants to check that should be able to ask the server rather than infer it from a
    /// latency.
    fn report_shape(
        &self,
        p: &crate::scan_fold::FoldPlan,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
    ) -> Option<bool> {
        use niles_ir::operator::{Agg, Scalar};
        const ACCT: u16 = 1;
        const CUR: u16 = 2;
        const AMT: u16 = 3;

        if circuit.outputs.get(output) != Some(&p.node) || !p.steps_are_empty() {
            return None;
        }
        if p.aggs() != [(Agg::Sum, Scalar::Column(AMT))] {
            return None;
        }
        let with_currency = match p.group_key() {
            [ACCT] => false,
            [ACCT, CUR] => true,
            _ => return None,
        };
        if !with_currency && self.currency_count() != 1 {
            return None;
        }
        // **No lock is taken here, and no state is consulted — A10-02.**
        //
        // This used to acquire the view, check `is_full() && applied_through() == anchor`,
        // release the view, and return `Some`. `report_from_view` then acquired the view a
        // *second* time and copied whatever was resident. An append landing between the two
        // acquisitions advances the view, so the rows copied were the values at some
        // `e > anchor` while the reply carried `anchor` — a report labelled with a snapshot
        // it is not, and nothing in the reply says so.
        //
        // The split is along the line between what moves and what does not. A plan's shape
        // is a property of the circuit and cannot change under a reader; whether the view is
        // full and how far it has been advanced can change on every append. So the shape is
        // decided here, without a lock, and the state is decided by
        // [`report_certified_at`] inside whichever guard is about to *use* it.
        Some(with_currency)
    }

    /// Is this view, right now, a full report at exactly this anchor?
    ///
    /// **Called only with the view guard already held, and its answer used without releasing
    /// it.** That is the whole of the A10-02 repair: the question is about state that an
    /// append moves, so an answer carried across a lock release is a claim about a moment
    /// that has passed.
    fn report_certified_at(view: &nilestream_core::rev::Rev, anchor: u64) -> bool {
        view.is_full() && view.applied_through() == anchor
    }

    /// **The serve path this engine would take, as opposed to the one the circuit implies.**
    ///
    /// [`serve_path`] reads the circuit alone, which is right for every class it names and
    /// wrong for one: whether a report is answered from a maintained view depends on the
    /// view's contract and on how far it has been advanced, neither of which is in the
    /// circuit. `explain` reported `fold` for a statement the engine was about to answer
    /// without touching a base row.
    pub fn serve_path_at(
        &self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> ServePath {
        let planned = crate::scan_fold::plan(circuit, output);
        // In the order `serve` takes them, so `explain` cannot describe a different engine:
        // the refusal is checked first there and must be checked first here.
        if let Some(p) = planned.as_ref() {
            if self.cross_currency_fold(p).is_some() {
                return ServePath::Refused;
            }
            if self.report_shape(p, circuit, output).is_some()
                && self
                    .runtime
                    .as_ref()
                    .map(|rt| {
                        let g = crate::lockstats::Timed::acquire(rt, &crate::lockstats::VIEW_LOCK);
                        g.view(BALANCE_VIEW)
                            .is_some_and(|v| Self::report_certified_at(v, anchor))
                    })
                    .unwrap_or(false)
            {
                return ServePath::Report;
            }
        }
        serve_path_of(planned.as_ref(), circuit, output)
    }

    /// **A whole report, out of a fully maintained view.**
    ///
    /// The other end of the trade `answer_from_view` is one end of. That answers one key from
    /// partial state; this answers *every* key from state that has been maintained through
    /// every append since it was installed, so a report over ten thousand accounts touches no
    /// base rows at all.
    ///
    /// `None` — meaning "fold the base" — whenever any condition of that claim fails, and
    /// each of them is a way the answer could otherwise be wrong rather than slow:
    ///
    /// * **The view must be full.** A partial view's resident entries are the accounts that
    ///   happen to be in memory. Serving those as "every account's balance" is a different
    ///   question presented as the one that was asked, and it is the single most dangerous
    ///   thing this file could do.
    /// * **There must be no restriction.** A filtered query is a point read, and
    ///   `answer_from_view` is its path.
    /// * **The shape must match** — `group by acct` or `group by acct, cur` summing `amt`,
    ///   with the aggregate as the output. Anything above it is evaluated by the reference
    ///   from a folded value, and this path does not produce one.
    /// * **One currency**, for the `group by acct` spelling, for the reason
    ///   `answer_from_view` gives: a server that picked a currency for the caller would make
    ///   the per-currency conservation rule invisible from outside.
    /// * **The anchor must be the one the view is true at.** A report is a set of rows true
    ///   at one moment; serving it at another would answer as of a moment nobody asked about.
    fn report_from_view(
        &self,
        p: &crate::scan_fold::FoldPlan,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> Option<crate::session::Rows> {
        use niles_ir::value::Value;

        let with_currency = self.report_shape(p, circuit, output)?;
        crate::rev_engine::report_race_hook::pause();

        // **One guard, and the certification is inside it — A10-02.**
        //
        // The check that the view is applied exactly through `anchor` and the copy of the
        // rows are now the same critical section. Split across two acquisitions, an append
        // between them left this returning the values at a later epoch under the earlier
        // epoch's label; nothing in the reply said so, and a caller has no way to tell.
        //
        // Failing the check is a `None`, which sends the statement down the general fold
        // path — slower, and exact at the anchor it was asked for. A stale label is not the
        // cheaper alternative to that; it is a wrong answer.
        let rt =
            crate::lockstats::Timed::acquire(self.runtime.as_ref()?, &crate::lockstats::VIEW_LOCK);
        let view = rt.view(BALANCE_VIEW)?;
        if !Self::report_certified_at(view, anchor) {
            return None;
        }

        let mut columns: Vec<String> = (0..p.width()).map(|i| format!("c{i}")).collect();
        columns.push("anchor".into());

        // Built straight into the Z-set the framer streams, in key order — which is Z-set
        // order, so this is a bulk build and not ten thousand tree insertions. No
        // intermediate row vector, and no base row read.
        let mut rows: Vec<(Vec<Value>, i128)> = Vec::with_capacity(view.resident_count() as usize);
        for (key, value, _at) in view.iter_resident() {
            let (Some(acct), Some(cur)) = (key.first(), key.get(1)) else {
                return None;
            };
            let mut row = vec![Value::Int(*acct as i128)];
            if with_currency {
                row.push(Value::Int(*cur as i128));
            }
            row.push(Value::Int(value));
            rows.push((row, 1));
        }

        Some(crate::session::Rows {
            columns,
            rows: crate::session::RowSource::Evaluated {
                z: rows.into_iter().collect(),
                anchor,
            },
        })
    }

    /// **One account's balance, out of the maintained REV** — or `None`, meaning this query
    /// is not that question and the general path must answer it.
    ///
    /// Every condition below is a way the maintained view could be a *different* answer from
    /// the one that was asked for, and each is checked rather than assumed:
    ///
    /// * the aggregate must be the circuit's output, so nothing sits above it that this
    ///   would skip;
    /// * the chain from the base must be filters only — a `Map` changes what is aggregated,
    ///   and a view maintained over unmapped rows would answer a different query;
    ///   `account_predicate` has already established that every filter is `acct = k`;
    /// * the aggregate must be exactly `sum(amt)`, which is what the installed view holds;
    /// * the base must hold exactly one currency. The view is keyed `(account, currency)`
    ///   and this query names no currency, so with two the server would have to pick one —
    ///   and a server that picks a currency for the caller makes the per-currency
    ///   conservation rule invisible from outside. With two currencies the fold answers,
    ///   correctly and more slowly, which is the right way round.
    fn answer_from_view(
        &self,
        p: &crate::scan_fold::FoldPlan,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        acct: u64,
        anchor: u64,
    ) -> ViewAnswer {
        use niles_ir::operator::{Agg, Scalar};
        /// Column positions in `postings`: `txn, acct, cur, amt, idem`.
        const ACCT: u16 = 1;
        const CUR: u16 = 2;
        const AMT: u16 = 3;

        if circuit.outputs.get(output) != Some(&p.node) || !p.filters_only() {
            return ViewAnswer::NotApplicable;
        }
        if p.aggs() != [(Agg::Sum, Scalar::Column(AMT))] {
            return ViewAnswer::NotApplicable;
        }
        // `group by acct` and `group by acct, cur` are the two spellings of "this account's
        // balance"; the first is only the same question while there is one currency to mean.
        let with_currency = match p.group_key() {
            [ACCT] => false,
            [ACCT, CUR] => true,
            _ => return ViewAnswer::NotApplicable,
        };
        if self.currency_count() != 1 {
            return ViewAnswer::NotApplicable;
        }
        let Some(cur) = self.sole_currency() else {
            return ViewAnswer::NotApplicable;
        };

        // **The base, and then the view — in that order, because `append` takes them in that
        // order and there is only one safe answer to which comes first.**
        //
        // This read used to take the view lock and reach for the base underneath it, while
        // `append` took the base exclusively and reached for the view underneath *that*. Two
        // locks, two paths, opposite orders: a point read holding the view and waiting for
        // the base, against an append holding the base and waiting for the view, is a
        // deadlock, and it was reachable in the shipped daemon the moment a client read
        // while another client wrote. No test saw it, because every concurrency test in this
        // workspace drives one workload at a time.
        //
        // One guard is taken for the whole function rather than one per use. `RwLock` is not
        // reentrant: a second `read()` on the same thread deadlocks against a writer that
        // arrived in between, so two guards would trade one hang for a rarer one. Holding it
        // across the keyed read is cheap — the read is microseconds — and it buys the base
        // and the view being read at one instant instead of two.
        // **The parts of a keyed read, timed, so the tail can be attributed to one of
        // them.** A mixed workload's read maximum is 12-13 ms on the reference host against
        // a p99 of 246 us and a base-lock wait that never exceeds 1.7 ms: a handful of reads
        // per run go somewhere, and an aggregate histogram is the wrong instrument for a
        // handful. Four clock reads on a path that already takes two locks, offered to a
        // bounded table that keeps the slowest sixteen.
        let Some(runtime) = self.runtime.as_ref() else {
            return ViewAnswer::NotApplicable;
        };
        let key = vec![acct as i64, cur as i64];

        // **Phase one: decide under the view, then let go of it.**
        //
        // This used to be one `Rev::read` call, which took the view lock, missed,
        // folded the base *inside* the hold, installed, and returned. Every other keyed read
        // in the process queued behind that fold whether or not it wanted the same key. The
        // hold is not the fold's fault — a reconstruction *is* expensive — it is the fault of
        // doing it while holding the one lock every reader needs for microseconds, with no
        // bound on how long that is.
        //
        // **The speed argument for this split did not survive its measurement, and the split
        // stays anyway.** The 12–13 ms read maximum and the 22 ms view wait it was undertaken
        // against came from an uncorrected harness — one daemon for a whole session, lock
        // histograms never reset between levels, lifetime counters, and a keyed read folding
        // the base twice on 87.4% of attempts before the certification interval was repaired.
        // Re-measured properly (C9-06.2), the predecessor build does not collapse at all: it
        // rises, 147k/157k/161k reads/s across 6r3w/9r5w/12r6w, and this split is worth
        // +1.8%. The slowest reads on *both* arms are dominated by `base wait`, not by this
        // lock. What the split is kept for is on the tin: the reconstruction happens with the
        // view released, which makes `Slot::Pending` reachable — the lattice's fourth state,
        // which the thesis describes and nothing could construct — and bounds a hold that had
        // no bound. See `docs/audit/cycle-9/hostc/c9-pending-results.md`.
        let read_began = std::time::Instant::now();
        let base = self.base();
        let base_ready = std::time::Instant::now();
        let mut rt = crate::lockstats::Timed::acquire(runtime, &crate::lockstats::VIEW_LOCK);
        let view_ready = std::time::Instant::now();
        let outcome = match rt.view_mut(BALANCE_VIEW) {
            Some(view) => view.begin_read(&key, anchor),
            None => return ViewAnswer::NotApplicable,
        };
        drop(rt);
        let decided = std::time::Instant::now();
        // **Every phase named, because a residual is not a measurement.** The old table had
        // four timestamps taken consecutively inside this function, so its fifth column was
        // integer-division truncation wearing the name of the wire (A9-F07), and the second
        // view acquisition — the one this read takes to *install* — was outside all of them
        // (A10-08). These are the phases the design questions of cycle 10 are about.
        let mut view_wait2 = std::time::Duration::ZERO;
        let mut install_hold = std::time::Duration::ZERO;
        let mut fold_time = std::time::Duration::ZERO;
        let mut trace_outcome = crate::lockstats::ReadOutcome::Hit;
        let mut gap_begin = 0u64;
        let mut gap_finish = 0u64;

        let answered = match outcome {
            nilestream_core::rev::ReadOutcome::Hit(a) => a,
            // **A reconstruction for this exact key at this exact anchor is already out.**
            // Return, which drops the base guard on the way, and let the caller wait holding
            // nothing at all. Waiting here — under B, and having just released V — would
            // block every append in the process behind another reader's fold, which is a
            // worse serialisation than the one this task removes.
            nilestream_core::rev::ReadOutcome::Join(w) => {
                // **Offered to the tail table before it goes, so a joined read can appear in
                // it at all.** The path whose cost is *another thread's* fold was the one
                // path the slowest-16 could never show; the caller reports the wait when it
                // returns, and this row says the read reached a join and how long deciding
                // that took.
                let now = std::time::Instant::now();
                let total_ns = now.duration_since(read_began).as_nanos() as u64;
                crate::lockstats::SLOW_READS.offer(
                    total_ns,
                    crate::lockstats::ReadTrace {
                        total_us: total_ns / 1_000,
                        base_wait_us: base_ready.duration_since(read_began).as_micros() as u64,
                        view_wait_us: view_ready.duration_since(base_ready).as_micros() as u64,
                        view_hold_us: decided.duration_since(view_ready).as_micros() as u64,
                        view_wait2_us: 0,
                        view_hold2_us: 0,
                        fold_us: 0,
                        outcome: crate::lockstats::ReadOutcome::Joined,
                        gap_begin: 0,
                        gap_finish: 0,
                    },
                );
                // **The shape travels with the ticket.** Built here, where `with_currency`
                // and the sole currency are already known and the columns are already the
                // ones the fold would have produced, so the joined reader does no work after
                // its wait and touches neither the view nor the base again.
                let mut columns: Vec<String> = (0..p.width()).map(|i| format!("c{i}")).collect();
                columns.push("anchor".into());
                return ViewAnswer::Wait(WaitPlan {
                    ticket: w,
                    columns,
                    acct,
                    cur,
                    with_currency,
                });
            }
            nilestream_core::rev::ReadOutcome::Fold(t) => {
                // **The fold, with the view released and the base still held.** B is not
                // re-acquired — `RwLock` is not reentrant and a keyed read takes it exactly
                // once — and V is not held, so a concurrent reader of any other key runs
                // straight through while this one reconstructs.
                trace_outcome = if t.installs() {
                    crate::lockstats::ReadOutcome::FoldOwned
                } else {
                    crate::lockstats::ReadOutcome::FoldAlone
                };
                gap_begin = t.gap_begin();
                let fold_from = std::time::Instant::now();
                let (value, rows) = {
                    use nilestream_core::rev::Base as _;
                    base.reconstruct(t.key(), t.anchor())
                };
                fold_time = fold_from.elapsed();
                let asked2 = std::time::Instant::now();
                let mut rt =
                    crate::lockstats::Timed::acquire(runtime, &crate::lockstats::VIEW_LOCK);
                let held_from = std::time::Instant::now();
                view_wait2 = held_from.duration_since(asked2);
                let a = match rt.view_mut(BALANCE_VIEW) {
                    Some(view) => {
                        gap_finish = view.applied_through().saturating_sub(t.anchor());
                        // **The base is passed, and it is the one already held.** A late
                        // landing merges the deltas it missed instead of pinning; the suffix
                        // is read through the shared guard this read has held since before
                        // `begin_read`, so no lock is taken here and the order B < V is
                        // unchanged. What does change is the length of the second view hold,
                        // which is why `install_hold` is measured around it — the merge's
                        // price is paid under V and has to be visible there.
                        view.finish_fold_merging(t, value, rows, Some(&*base))
                    }
                    // The view went away between the two phases. The ticket is dropped
                    // un-settled, which cancels the flight and releases anyone who joined
                    // it to retry; this caller falls back to the fold path.
                    None => return ViewAnswer::NotApplicable,
                };
                // **The hold ends when the guard is dropped, not when the work is done.**
                // `install_hold` was taken while `rt` was still alive, so releasing the
                // second view guard fell outside every phase and landed in the residual
                // column — which is how a row in this harness's first run showed 1,160 µs of
                // "rounding" on a table whose divisions can only lose six. The drop is
                // explicit and the elapsed time is read after it.
                drop(rt);
                install_hold = held_from.elapsed();
                a
            }
        };
        let view_done = std::time::Instant::now();
        let total_ns = view_done.duration_since(read_began).as_nanos() as u64;
        crate::lockstats::SLOW_READS.offer(
            total_ns,
            crate::lockstats::ReadTrace {
                total_us: total_ns / 1_000,
                base_wait_us: base_ready.duration_since(read_began).as_micros() as u64,
                view_wait_us: view_ready.duration_since(base_ready).as_micros() as u64,
                // **The first hold only.** The second is its own column: adding them made
                // one number out of two questions, and the install's wait was in neither.
                view_hold_us: decided.duration_since(view_ready).as_micros() as u64,
                view_wait2_us: view_wait2.as_micros() as u64,
                view_hold2_us: install_hold.as_micros() as u64,
                fold_us: fold_time.as_micros() as u64,
                outcome: trace_outcome,
                gap_begin,
                gap_finish,
            },
        );

        // **The answer must be true at the anchor that was asked for, not merely at least as
        // fresh.** A hit reports the entry's *effective* version, which can be later than the
        // requested anchor — a value that includes writes the caller's snapshot excludes.
        //
        // This used to say the two were always equal, because the write path held the engine
        // lock and the frontier could not move between `observe` and here. That reasoning
        // died with the engine lock: an append now runs concurrently with this read, so the
        // mismatch is a normal event and not a defensive check against an impossible one.
        // The response is unchanged and still correct — fall back to the fold, which
        // reconstructs at the requested anchor exactly — but it is now a *cost*, and how
        // often it is paid is not counted anywhere. See `BLOCKED-fallback-rate`.
        if answered.anchor != anchor {
            // **Counted, because an uncounted cost is one nobody can be asked about.**
            // `BLOCKED-fallback-rate` was raised against exactly this line: the response is
            // correct — the fold reconstructs at the requested anchor — but how often the
            // engine paid for a view lookup and then threw the answer away was not visible
            // from any surface, and an audit measured 87.4% of keyed reads taking it under
            // concurrent writers. `select nilestream_stats` reports it now.
            self.view_fallbacks
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return ViewAnswer::NotApplicable;
        }
        self.view_answers
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut columns: Vec<String> = (0..p.width()).map(|i| format!("c{i}")).collect();
        columns.push("anchor".into());

        // **An account the base has never posted to has no balance, and that is not zero.**
        // The absence lattice's distinction, at the layer where it would be quietest to
        // lose: a keyed read answers with a number for a key nothing has ever touched, and a
        // group that does not exist must produce no row at all — which is what the fold and
        // the reference evaluator both do, and what this must agree with.
        if base.key_update_count(acct, anchor.min(base.head())) == 0 {
            return ViewAnswer::Rows(crate::session::Rows {
                columns,
                rows: crate::session::RowSource::Evaluated {
                    z: niles_ir::eval::ZSet::new(),
                    anchor,
                },
            });
        }

        // The same shape the fold would have produced, so the two paths are
        // indistinguishable to everything above them — including the framer, which appends
        // the anchor itself rather than being handed it as a cell.
        ViewAnswer::Rows(Self::rows_from_answer(
            columns,
            acct,
            cur,
            with_currency,
            nilestream_core::rev::Joined {
                answer: answered,
                // The owner reached this line through a hit or its own fold, and the
                // no-such-account case was decided above from the base. One row.
                base_rows: 1,
            },
            anchor,
        ))
    }

    /// **One shaping, used by the reader that folded and by the reader that joined it.**
    ///
    /// Two spellings of this would disagree the first time either changed, and the
    /// disagreement would be between a read that waited and a read that did not — the one
    /// difference a client must never be able to see.
    ///
    /// `base_rows == 0` is "this key has no history in this prefix", which produces **no
    /// row**. A joined reader has no other way to know that: the value it receives is zero
    /// either way, and a zero balance and a missing account are different answers.
    fn rows_from_answer(
        columns: Vec<String>,
        acct: u64,
        cur: u32,
        with_currency: bool,
        joined: nilestream_core::rev::Joined,
        anchor: u64,
    ) -> crate::session::Rows {
        use niles_ir::value::Value;
        if joined.base_rows == 0 {
            return crate::session::Rows {
                columns,
                rows: crate::session::RowSource::Evaluated {
                    z: niles_ir::eval::ZSet::new(),
                    anchor,
                },
            };
        }
        let mut row = vec![Value::Int(acct as i128)];
        if with_currency {
            row.push(Value::Int(cur as i128));
        }
        row.push(Value::Int(joined.answer.value));
        let mut z = niles_ir::eval::ZSet::new();
        z.insert(row, 1);
        crate::session::Rows {
            columns,
            rows: crate::session::RowSource::Evaluated { z, anchor },
        }
    }

    /// **The base's posting records, in the schema a lowered circuit indexes into** —
    /// `(txn, acct, cur, amt, idem)` — written into a caller's buffer so a query costs one
    /// allocation for the scan rather than two per row.
    ///
    /// `idem` is a text column and has no integer form, so it is null. That is not a
    /// placeholder: an idempotency key is not something a read model computes with, and a
    /// circuit filtering on one would be filtering on an absence, which the three-valued
    /// rules already handle.
    fn scan(&self, acct: Option<u64>, anchor: u64, mut f: impl FnMut([niles_ir::value::Value; 5])) {
        use niles_ir::value::Value;
        let base = self.base();
        let upto = anchor.min(base.head());
        let row = |p: &Posting| {
            [
                Value::Int(p.txn as i128),
                Value::Int(p.acct as i128),
                Value::Int(p.cur as i128),
                Value::Int(p.amt),
                Value::Null,
            ]
        };
        match acct {
            // Through the anchor index: the cost is that account's own postings rather than
            // the length of history, which is the mechanism §9.4.1 measures.
            Some(a) => {
                for p in base.postings_for(a, upto) {
                    f(row(&p));
                }
            }
            None => {
                for e in &base.epochs {
                    if e.id > upto {
                        break;
                    }
                    for r in &e.rows {
                        if let Row::Post(p) = r {
                            f(row(p));
                        }
                    }
                }
            }
        }
    }

    /// The `postings` base as a Z-set visible at `anchor`, in the schema the lowering gives
    /// a source node: `(txn, acct, cur, amt, idem)` — the declared column order, so a
    /// circuit's column indices mean here what they meant when it was lowered.
    ///
    /// `idem` is a text column in the schema and has no integer form, so it is `null`. That
    /// is not a placeholder for a value: an idempotency key is not something a read model
    /// computes with, and a circuit that filtered on one would be filtering on an absence,
    /// which the three-valued rules already handle correctly.
    fn base_at(&self, anchor: u64) -> std::collections::BTreeMap<String, niles_ir::eval::ZSet> {
        use niles_ir::value::Value;
        let mut z: niles_ir::eval::ZSet = Default::default();
        let base = self.base();
        let head = base.head();
        let upto = anchor.min(head);
        for e in &base.epochs {
            if e.id > upto {
                break;
            }
            for r in &e.rows {
                if let Row::Post(p) = r {
                    let row = vec![
                        Value::Int(p.txn as i128),
                        Value::Int(p.acct as i128),
                        Value::Int(p.cur as i128),
                        Value::Int(p.amt),
                        Value::Null,
                    ];
                    niles_ir::eval::add(&mut z, row, 1);
                }
            }
        }
        let mut m = std::collections::BTreeMap::new();
        m.insert("postings".to_string(), z);
        m
    }

    /// The base restricted to one account, through the anchor index.
    fn base_for_account(
        &self,
        acct: u64,
        anchor: u64,
    ) -> std::collections::BTreeMap<String, niles_ir::eval::ZSet> {
        use niles_ir::value::Value;
        let mut z: niles_ir::eval::ZSet = Default::default();
        let base = self.base();
        for p in base.postings_for(acct, anchor.min(base.head())) {
            niles_ir::eval::add(
                &mut z,
                vec![
                    Value::Int(p.txn as i128),
                    Value::Int(p.acct as i128),
                    Value::Int(p.cur as i128),
                    Value::Int(p.amt),
                    Value::Null,
                ],
                1,
            );
        }
        let mut m = std::collections::BTreeMap::new();
        m.insert("postings".to_string(), z);
        m
    }
}

/// The single account a circuit's filters restrict it to, if there is exactly one.
///
/// Conservative by construction: it looks for a `Filter` whose predicate is
/// `Column(acct) = <literal>` and returns `None` for anything else, including a disjunction
/// or a second filter naming a different account. A pushdown that guessed would restrict a
/// scan the circuit does not restrict, and the answer would be missing rows — which is the
/// one thing an optimisation must never do.
/// The column `postings.acct` occupies in the source schema. Declared order:
/// `txn, acct, cur, amt, idem`.
pub const ACCT_COL: u16 = 1;

/// **How the engine would answer a statement.**
///
/// Four classes, ordered from cheapest to most expensive, and the difference between the
/// first and the last is three orders of magnitude. Until this existed, the only way to find
/// out which one a query would take was to measure it: the decision was spread over three
/// `if let`s inside `query`, and a query that fell off a cliff — `where acct = k and cur = 0`
/// used to, and `group by acct having acct = k` used to — looked exactly like one that did
/// not.
///
/// [`serve_path`] computes this from the same functions `query` branches on, and `query` now
/// branches on *this*, so the two cannot drift into describing different engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServePath {
    /// **Every key of a fully maintained view, read without touching the base.**
    ///
    /// The other end of the trade `View` is one end of, and the one the `report` row of E16
    /// measures. Reachable only while the view's contract is `materialize: full` and it has
    /// been advanced to the anchor being asked about — so it is a property of the engine at
    /// this moment, not of the circuit, and only `serve_path_now` can report it.
    Report,
    /// A maintained REV entry: one key, read from partial state, reconstructed on a miss.
    /// This is the mechanism the phase diagram characterises.
    View,
    /// A fold over the rows of **one account**, reached through the anchor index.
    IndexFold,
    /// A fold over the whole base, in one pass, with no intermediate Z-set.
    Fold,
    /// The whole base materialised as a Z-set and the circuit evaluated over it: the honest
    /// cost of a shape outside the keyed-aggregate fragment.
    Materialise,
    /// **No path: the query would add two currencies and the engine refuses it.**
    ///
    /// A path in this enum is a claim about what the engine will do, and "it will refuse" is
    /// as much a fact about the next execution as "it will fold". Leaving it out meant
    /// `explain` promised a `fold` for a statement that was about to raise `22000` — which is
    /// the specific failure this enum exists to prevent, one function deciding and both
    /// readers agreeing.
    Refused,
}

impl ServePath {
    pub fn as_str(self) -> &'static str {
        match self {
            ServePath::Report => "report-from-view",
            ServePath::View => "view",
            ServePath::IndexFold => "index-fold",
            ServePath::Fold => "fold",
            ServePath::Materialise => "materialise",
            ServePath::Refused => "refused-cross-currency",
        }
    }

    /// The same line, from the class name a `Serving` implementation reported.
    ///
    /// The trait returns a `&'static str` rather than a `ServePath` so a server with no
    /// partial state need not depend on this enum; this turns it back.
    pub fn describe_class(name: &str) -> &'static str {
        for p in [
            ServePath::Report,
            ServePath::View,
            ServePath::IndexFold,
            ServePath::Fold,
            ServePath::Materialise,
        ] {
            if p.as_str() == name {
                return p.describe();
            }
        }
        "an unnamed serve class — this server reported a path this build does not know"
    }

    /// One line saying what the class means and what it costs, for `explain`.
    pub fn describe(self) -> &'static str {
        match self {
            ServePath::Report => {
                "every key of a fully maintained view, in key order, at the anchor the view is true at; touches no base rows at all"
            }
            ServePath::View => {
                "a maintained REV entry, read at the requested anchor and reconstructed on a miss; touches no base rows on a hit"
            }
            ServePath::IndexFold => {
                "one pass over one account's postings, reached through the anchor index; O(that account's history)"
            }
            ServePath::Fold => {
                "one pass over the base with a per-group accumulator and no intermediate Z-set; O(base rows + groups)"
            }
            ServePath::Materialise => {
                "the base materialised as a Z-set and the circuit evaluated over it — the shape is outside the keyed-aggregate fragment; O(base rows) with an allocation per row"
            }
            ServePath::Refused => {
                "refused: this `sum(amt)` groups without `cur` over a base holding more than one currency, so it would add amounts that are not comparable. Add `cur` to the `group by`, or restrict with `cur = k`"
            }
        }
    }
}

/// The class [`RevEngine::query`] would choose for this statement.
///
/// **Static**: it answers from the circuit alone. `View` additionally requires conditions the
/// engine knows and a compiler does not — that the balance view is installed, that the base
/// holds exactly one currency, and that the requested anchor is the one the entry is true at
/// — and each of those falls back to `IndexFold`, which is why `explain` says "view" for a
/// shape that *can* be a view read rather than for one that certainly will be.
pub fn serve_path(circuit: &niles_ir::circuit::Circuit, output: &str) -> ServePath {
    serve_path_of(
        crate::scan_fold::plan(circuit, output).as_ref(),
        circuit,
        output,
    )
}

/// [`serve_path`] for a caller that has already planned.
///
/// `query` has, and planning twice per query would make `explain` cost the engine an
/// allocation to agree with itself.
pub fn serve_path_of(
    plan: Option<&crate::scan_fold::FoldPlan<'_>>,
    circuit: &niles_ir::circuit::Circuit,
    output: &str,
) -> ServePath {
    use niles_ir::operator::{Agg, Scalar};
    const CUR: u16 = 2;
    const AMT: u16 = 3;
    let Some(p) = plan else {
        return ServePath::Materialise;
    };
    let sole = p.sole_account_filter(ACCT_COL);
    if sole.is_some()
        && circuit.outputs.get(output) == Some(&p.node)
        && p.filters_only()
        && p.aggs() == [(Agg::Sum, Scalar::Column(AMT))]
        && matches!(p.group_key(), [ACCT_COL] | [ACCT_COL, CUR])
    {
        return ServePath::View;
    }
    if p.column_restriction(ACCT_COL).is_some() {
        return ServePath::IndexFold;
    }
    ServePath::Fold
}

fn account_predicate(circuit: &niles_ir::circuit::Circuit) -> Option<u64> {
    use niles_ir::operator::{Op, Scalar, ScalarOp};
    const ACCT: u16 = ACCT_COL;
    let mut found: Option<u64> = None;
    for n in &circuit.nodes {
        let Op::Filter { predicate } = &n.op else {
            continue;
        };
        let Scalar::Binary {
            op: ScalarOp::Eq,
            lhs,
            rhs,
        } = predicate
        else {
            // A filter this does not understand may keep rows a restricted scan would have
            // dropped, so the restriction is abandoned rather than narrowed.
            return None;
        };
        let acct = match (&**lhs, &**rhs) {
            (Scalar::Column(ACCT), Scalar::LitInt(k))
            | (Scalar::LitInt(k), Scalar::Column(ACCT)) => u64::try_from(*k).ok()?,
            _ => return None,
        };
        match found {
            Some(prev) if prev != acct => return None,
            _ => found = Some(acct),
        }
    }
    found
}

/// **Install the balance view the wire path reads from.**
///
/// The circuit is compiled from Niles source against the daemon's own schema rather than
/// hand-built, because that is the claim: *source → typed IR → running partial-state
/// engine*, with nothing hand-assembled in between. A hand-built circuit here would make the
/// pipeline a diagram again.
///
/// Keyed `(account, currency)` because a balance is per currency, and folding across
/// currencies is exactly the thing the commit rule refuses on the write side.
///
/// Returns `None` if the view will not install — an aggregate outside the runtime's fragment,
/// or a compile error — and the caller then serves by folding. A refusal, not a fallback that
/// answers something else.
fn install_balance_view(budget: usize) -> Option<nilestream_core::rev::Runtime> {
    const BALANCE: &str = "select acct, cur, sum(amt) from postings group by acct, cur";
    // **The contract is what decides whether this view holds everything.** `full` makes the
    // runtime install a key on its first delta rather than waiting for a read, which is what
    // "every account's balance" needs and what `auto` does not give: a view with no budget
    // that has never been read holds nothing, and a report over it would return the empty
    // set. The budget sentinel is how the caller asks, and the contract is how the runtime is
    // told — the two must agree or `is_full` refuses and the fold answers.
    let materialize = if budget == usize::MAX { "full" } else { "auto" };
    let program = format!(
        "{}\nview {BALANCE_VIEW} = sql {{ {BALANCE} }} serve {{ consistency: snapshot, materialize: {materialize} }};\n",
        crate::daemon::DEFAULT_SCHEMA
    );
    let (prog, d) = niles_lang::parser::parse_program(&program);
    if d.has_errors() {
        return None;
    }
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    if rd.has_errors() {
        return None;
    }
    let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
    if ld.has_errors() || !niles_ir::verify::verify(&lowered.circuit).is_ok() {
        return None;
    }
    nilestream_core::rev::Runtime::install(
        lowered.circuit,
        // `usize::MAX` is the harness's way of asking for a view that never evicts, which is
        // what a *report* needs: a partial view's resident subset is the accounts that happen
        // to be in memory, not every account, and answering "every balance" from it would be
        // answering a different question.
        (budget != usize::MAX).then_some(budget as u64),
        nilestream_core::rev::Policy::Lru,
    )
    .ok()
    .map(|mut rt| {
        rt.set_merge_caps(merge_caps_from_env());
        rt
    })
}

/// The deferred-merge bounds this process runs with, read once from the environment.
///
/// `NILESTREAM_MERGE_CAPS` takes `off`, `default`, or `<epochs>:<rows>`. **This exists so
/// that one binary can run both arms of the measurement**: the merging build and the pinned
/// control differ in this value and in nothing else, so a difference between them cannot be
/// a difference between two compilations.
///
/// An unparseable value is a **refusal**, not a silent fall back to the default. A run whose
/// arm was decided by a typo is a run that scored the control against itself and said
/// otherwise, and the caps it actually used are reported on the wire for the same reason.
fn merge_caps_from_env() -> nilestream_core::rev::MergeCaps {
    parse_merge_caps(std::env::var("NILESTREAM_MERGE_CAPS").ok().as_deref()).unwrap_or_else(|raw| {
        panic!(
            "NILESTREAM_MERGE_CAPS={raw:?} is not `off`, `default` or `<epochs>:<rows>`. \
                 Refusing rather than defaulting: a run whose arm was decided by a typo would \
                 report the merging build's numbers under the control's name."
        )
    })
}

/// The parse, with no environment in it.
///
/// **Separated because the test for it cannot own the process.** The first version of this
/// read `std::env::var` inside the function and the test set the variable to each bad value
/// in turn; `cargo test` runs tests in parallel, so a `RevEngine` built by an unrelated test
/// saw `NILESTREAM_MERGE_CAPS="8:64:2"` and the refusal — which is the right behaviour —
/// killed it. That is MF-4's shape exactly: a guard that mutates process-wide state cannot
/// isolate what it is testing. The environment is read in one place and the decision is a
/// pure function of a string.
fn parse_merge_caps(raw: Option<&str>) -> Result<nilestream_core::rev::MergeCaps, String> {
    use nilestream_core::rev::MergeCaps;
    let Some(raw) = raw else {
        return Ok(MergeCaps::default());
    };
    match raw.trim() {
        "" | "default" => Ok(MergeCaps::default()),
        "off" => Ok(MergeCaps::OFF),
        other => other
            .split_once(':')
            .and_then(|(e, r)| {
                Some(MergeCaps {
                    max_epochs: e.trim().parse().ok()?,
                    max_rows: r.trim().parse().ok()?,
                })
            })
            .ok_or_else(|| other.to_string()),
    }
}

/// The name of that view inside the runtime.
pub const BALANCE_VIEW: &str = "__balance";

/// **The arm selector, and that it refuses rather than defaults.**
///
/// T04.2 compares a merging build against a pinned control, and the two are this binary with
/// two different values of `NILESTREAM_MERGE_CAPS`. Every property that makes that comparison
/// trustworthy is a property of this function: `off` is exactly [`MergeCaps::OFF`], an absent
/// variable is the preregistered default, and anything unparseable stops the process instead
/// of quietly running the default under the control's name.
#[cfg(test)]
mod merge_caps_env_tests {
    use nilestream_core::rev::MergeCaps;

    /// The parse is a pure function of a string, so this test touches no process state at
    /// all — see `parse_merge_caps`'s own note for what happened when it did not.
    #[test]
    fn the_arm_selector_parses_what_it_promises_and_refuses_the_rest() {
        let parse = super::parse_merge_caps;

        assert_eq!(parse(None).unwrap(), MergeCaps::default());
        assert_eq!(parse(Some("default")).unwrap(), MergeCaps::default());
        assert_eq!(parse(Some("  ")).unwrap(), MergeCaps::default());
        assert_eq!(
            parse(Some("off")).unwrap(),
            MergeCaps::OFF,
            "`off` must be exactly the pinned control, not a small budget that merges a little"
        );
        assert_eq!(
            parse(Some("8:64")).unwrap(),
            MergeCaps {
                max_epochs: 8,
                max_rows: 64
            }
        );

        for bad in ["on", "8", "8:", ":64", "eight:64", "-1:64", "8:64:2"] {
            assert!(
                parse(Some(bad)).is_err(),
                "`{bad}` must be refused. Falling back to the default here would run the \
                 merging build and label the transcript with whatever the invocation \
                 intended, which is a control scored against itself"
            );
        }
    }

    /// The default is the preregistered pair, stated here so a change to it fails a test
    /// rather than moving a number the report already quotes.
    #[test]
    fn the_preregistered_caps_are_the_ones_the_report_names() {
        assert_eq!(
            MergeCaps::default(),
            MergeCaps {
                max_epochs: 32,
                max_rows: 4_096
            },
            "cycle 10 preregistered 32 epochs and 4,096 rows, from a measured arrival gap of \
             4.6-5.7 epochs with a maximum of 17. Changing them is allowed; changing them \
             silently is not"
        );
    }
}

#[cfg(test)]
mod tests {

    /// Compile a wire statement against the daemon's schema, exactly as a session does.
    pub(super) fn compile(sql: &str) -> niles_lang::lower::Lowered {
        let program = format!(
            "{}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};\n",
            crate::daemon::DEFAULT_SCHEMA
        );
        let (prog, d) = niles_lang::parser::parse_program(&program);
        assert!(!d.has_errors(), "`{sql}`: {:?}", d.items);
        let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
        assert!(!rd.has_errors(), "`{sql}`: {:?}", rd.items);
        let (_r, td) = niles_lang::typecheck::check_program(&prog, &cat);
        assert!(!td.has_errors(), "`{sql}`: {:?}", td.items);
        let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
        assert!(!ld.has_errors(), "`{sql}`: {:?}", ld.items);
        assert!(niles_ir::verify::verify(&lowered.circuit).is_ok());
        lowered
    }

    /// Render a Z-set the way `Serving::query` renders one, so the two can be compared.
    fn rendered(z: &niles_ir::eval::ZSet, anchor: u64) -> Vec<Vec<Option<String>>> {
        let mut rows = Vec::new();
        for (r, w) in z {
            if *w <= 0 {
                continue;
            }
            for _ in 0..*w {
                let mut cells: Vec<Option<String>> = r
                    .iter()
                    .map(|v| match v {
                        niles_ir::value::Value::Null => None,
                        niles_ir::value::Value::Int(i) => Some(i.to_string()),
                    })
                    .collect();
                cells.push(Some(anchor.to_string()));
                rows.push(cells);
            }
        }
        rows
    }

    /// **The obligation the fold carries.**
    ///
    /// Answering a keyed aggregate by folding the base cannot change what the circuit
    /// denotes, and this is what says so rather than the module's docstring: every statement
    /// is answered by the fold and by the reference evaluator over a materialised base, and
    /// the two must be identical — rows, order and nulls.
    ///
    /// The cases are chosen to cover what the fragment's edges are made of: a grouped and an
    /// ungrouped aggregate, a filter that keeps some rows and one that keeps none, a `sum`
    /// over no non-null rows (which is null and not zero), an ordering and a limit above the
    /// aggregate, a pushed-down key, and shapes the plan must *refuse* so that they take the
    /// materialising path and still answer.
    #[test]
    fn the_fold_and_the_oracle_agree() {
        use crate::session::Serving;
        // Small enough to be a unit test, seeded densely enough that groups, filters and
        // limits all have something to do.
        let e = RevEngine::seeded(60, 3, 20, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();

        let folded = [
            "select cur, sum(amt) from postings group by cur",
            "select acct, sum(amt) from postings group by acct",
            "select sum(amt) from postings where amt < 0",
            "select sum(amt) from postings",
            "select acct, sum(amt) from postings group by acct order by sum(amt) desc limit 5",
            "select acct, sum(amt) from postings group by acct order by acct limit 3",
            "select acct, sum(amt) from postings where acct = 7 group by acct",
            "select acct, sum(amt) from postings where acct = 999999 group by acct",
            "select cur, sum(amt) from postings where amt > 1000000 group by cur",
            "select acct, count(amt) from postings group by acct",
            "select count(amt) from postings where amt < 0",
        ];
        for sql in folded {
            let lowered = compile(sql);
            let plan = crate::scan_fold::plan(&lowered.circuit, "__wire_result");
            assert!(
                plan.is_some(),
                "`{sql}` is inside the fragment and must be folded, or this case is \
                 comparing the materialising path with itself"
            );
            let by_fold = e
                .query(&lowered.circuit, "__wire_result", anchor)
                .expect("the fold answers");
            let sources = e.base_at(anchor);
            let (oracle, _) = niles_ir::eval::try_run(&lowered.circuit, "__wire_result", &sources)
                .expect("the reference answers");
            assert_eq!(
                by_fold.text(),
                rendered(&oracle, anchor),
                "`{sql}`: the fold and the reference evaluator denote different things"
            );
        }

        // Shapes outside the fragment: the plan must refuse them — silently answering one
        // with a sum is the failure this design is arranged to prevent — and the query must
        // still be served, by the path that materialises.
        for sql in [
            "select acct, min(amt) from postings group by acct",
            "select acct, avg(amt) from postings group by acct",
            "select acct, amt from postings where amt < 0",
            "select distinct acct from postings",
        ] {
            let lowered = compile(sql);
            assert!(
                crate::scan_fold::plan(&lowered.circuit, "__wire_result").is_none(),
                "`{sql}` is outside the fragment and must be refused by the planner"
            );
            let served = e
                .query(&lowered.circuit, "__wire_result", anchor)
                .expect("the materialising path still answers");
            let sources = e.base_at(anchor);
            let (oracle, _) = niles_ir::eval::try_run(&lowered.circuit, "__wire_result", &sources)
                .expect("the reference answers");
            assert_eq!(served.text(), rendered(&oracle, anchor), "`{sql}`");
        }
    }

    /// **The scalar-key fold denotes exactly what the generic one did.**
    ///
    /// T-04 gave a one-column group key its own accumulator table, keyed by `Option<i128>`
    /// instead of by a boxed one-element `Vec<Value>`, and made `finish` insert straight into
    /// the Z-set on the strength of the two orderings agreeing. Three things could go wrong
    /// and none of them would be visible in a throughput number:
    ///
    /// * a **null key** could sort into the wrong place, or worse, merge with a group;
    /// * the **emission order** could stop being Z-set order, which the `insert` shortcut
    ///   assumes and which nothing else would notice, because a `BTreeMap` re-sorts what is
    ///   put into it — the rows would be right and the shortcut's premise would be false;
    /// * a `Map` between the source and the aggregate could shift the key column, so the
    ///   fold would group by the wrong one.
    ///
    /// Each is compared against the reference evaluator over the same base, which is the only
    /// judge either path answers to.
    #[test]
    fn the_scalar_key_fold_agrees_with_the_reference_on_the_cases_it_specialises() {
        use crate::session::Serving;
        let e = RevEngine::seeded(40, 2, 15, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();

        for sql in [
            // A single integer key, the shape the specialisation exists for.
            "select acct, sum(amt) from postings group by acct",
            "select acct, count(amt) from postings group by acct",
            // **A key that is null for every row.** `idem` is the text column, materialised
            // as null, so this forms exactly one group whose key is `Null` — the slot the
            // scalar path keeps separately, as `None`, and which must sort before every
            // integer key and merge with nothing.
            "select idem, sum(amt) from postings group by idem",
            "select idem, count(amt) from postings group by idem",
            // Two key columns: the generic path, kept working, and the reason `Groups` still
            // has a `Wide` arm.
            "select acct, cur, sum(amt) from postings group by acct, cur",
            // No key at all: also the generic path.
            "select sum(amt) from postings",
            // A single key with a filter that keeps a scattered subset, so the groups are
            // sparse and the tree is not being walked in insertion order.
            "select acct, sum(amt) from postings where amt < 0 group by acct",
            // Ordering and limiting above a scalar-keyed fold: the reference evaluates these
            // from the folded node, so this is where a wrong emission order would surface.
            "select acct, sum(amt) from postings group by acct order by acct limit 7",
            "select acct, sum(amt) from postings group by acct order by sum(amt) desc limit 5",
        ] {
            let lowered = compile(sql);
            let by_fold = e
                .query(&lowered.circuit, "__wire_result", anchor)
                .expect("the fold answers");
            let sources = e.base_at(anchor);
            let (oracle, _) = niles_ir::eval::try_run(&lowered.circuit, "__wire_result", &sources)
                .expect("the reference answers");
            assert_eq!(
                by_fold.text(),
                rendered(&oracle, anchor),
                "`{sql}`: the scalar-key fold and the reference denote different things"
            );
        }

        // **The premise of the `insert` shortcut, checked directly.** `finish` puts rows into
        // the output Z-set with `insert` rather than `eval::add`, which is only sound if the
        // groups arrive in ascending row order. A `BTreeMap` would hide a violation — the
        // answer would still be right — so the order is asserted here rather than inferred
        // from the answers above.
        let lowered = compile("select acct, sum(amt) from postings group by acct");
        let plan = crate::scan_fold::plan(&lowered.circuit, "__wire_result").expect("in fragment");
        let sources = e.base_at(anchor);
        let base: Vec<Vec<niles_ir::value::Value>> = sources
            .get("postings")
            .expect("the base is there")
            .keys()
            .cloned()
            .collect();
        let (z, _) = crate::scan_fold::fold(&plan, base.iter().map(|r| r.as_slice()))
            .expect("this fixture contains no failing arithmetic");
        let keys: Vec<&Vec<niles_ir::value::Value>> = z.keys().collect();
        assert!(
            keys.windows(2).all(|w| w[0] < w[1]),
            "the fold emitted its groups out of order, so `insert` was standing in for `add` \
             on a premise that no longer holds"
        );
        assert!(keys.len() > 1, "the case is trivial if there is one group");
    }

    /// **The two cliffs, and the property that makes removing them safe.**
    ///
    /// `where acct = 4242 and cur = 0` and `group by acct having acct = 4242` both restrict
    /// to one account, and both used to read every row of the base: the first because the
    /// scan restriction understood only a filter whose whole predicate was `acct = k`, the
    /// second because a filter above the aggregate made the fold planner refuse the shape.
    ///
    /// Restricting a scan is only sound if it cannot change the answer, so every shape here
    /// is compared against the reference evaluator over the *unrestricted* base. The
    /// negative cases matter as much: a disjunction over two accounts and a pair of
    /// contradictory conjuncts must **not** be restricted, and would silently lose rows if
    /// they were.
    #[test]
    fn a_restricted_scan_and_the_full_one_answer_the_same_question() {
        use crate::session::Serving;
        let e = RevEngine::seeded(60, 3, 20, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();

        for sql in [
            // The two shapes this task made cheap.
            "select acct, sum(amt) from postings where acct = 7 and cur = 0 group by acct",
            "select acct, sum(amt) from postings group by acct having acct = 7",
            // A conjunct that excludes everything the account has: the restriction is right
            // and the answer is still empty.
            "select acct, sum(amt) from postings where acct = 7 and cur = 99 group by acct",
            // The conjunct in the other order, and with the literal on the left.
            "select acct, sum(amt) from postings where cur = 0 and acct = 7 group by acct",
            // A `having` that names a key *and* an aggregate: not pushable, and it must
            // still answer.
            "select acct, sum(amt) from postings group by acct having acct = 7",
            // A `having` over the aggregate alone: never pushable.
            "select acct, sum(amt) from postings group by acct having sum(amt) > 0",
            // **Must not be restricted**: neither is a necessary condition on a row.
            "select acct, sum(amt) from postings where acct = 7 or acct = 9 group by acct",
            // Contradictory conjuncts: no row survives, and a scan restricted to one of the
            // two accounts would be answering a different query.
            "select acct, sum(amt) from postings where acct = 7 and acct = 9 group by acct",
        ] {
            let lowered = compile(sql);
            let served = e
                .query(&lowered.circuit, "__wire_result", anchor)
                .expect("answers");
            let sources = e.base_at(anchor);
            let (oracle, _) = niles_ir::eval::try_run(&lowered.circuit, "__wire_result", &sources)
                .expect("the reference answers");
            assert_eq!(
                served.text(),
                rendered(&oracle, anchor),
                "`{sql}`: the restricted scan and the full base denote different things"
            );
        }
    }

    /// The restriction itself, asserted directly rather than inferred from an answer.
    ///
    /// A scan that is restricted when it should not be gives a wrong answer, which the test
    /// above catches. A scan that is *not* restricted when it could be is merely slow, and
    /// nothing catches that except a budget — so the decision is asserted here too, where a
    /// reader can see which shapes are expected to be cheap.
    #[test]
    fn the_scan_is_restricted_exactly_where_a_conjunct_makes_it_safe() {
        let restriction = |sql: &str| -> Option<u64> {
            let lowered = compile(sql);
            let p = crate::scan_fold::plan(&lowered.circuit, "__wire_result")?;
            p.column_restriction(ACCT_COL)
        };
        for (sql, want) in [
            (
                "select acct, sum(amt) from postings where acct = 7 group by acct",
                Some(7),
            ),
            (
                "select acct, sum(amt) from postings where acct = 7 and cur = 0 group by acct",
                Some(7),
            ),
            (
                "select acct, sum(amt) from postings where cur = 0 and acct = 7 group by acct",
                Some(7),
            ),
            (
                "select acct, sum(amt) from postings group by acct having acct = 7",
                Some(7),
            ),
            // A disjunction is one leaf: neither side is necessary, so no restriction.
            (
                "select acct, sum(amt) from postings where acct = 7 or acct = 9 group by acct",
                None,
            ),
            // Two accounts required at once: nothing survives, and guessing one of them
            // would answer a different query.
            (
                "select acct, sum(amt) from postings where acct = 7 and acct = 9 group by acct",
                None,
            ),
            (
                "select acct, sum(amt) from postings where cur = 0 group by acct",
                None,
            ),
            ("select acct, sum(amt) from postings group by acct", None),
        ] {
            assert_eq!(restriction(sql), want, "`{sql}`");
        }
    }

    /// **The cliff, measured in base rows rather than in allocations.**
    ///
    /// `served_point_conjunct`'s allocation budget does not catch this one, and that is worth
    /// saying rather than leaving as a gap: an unrestricted fold streams the base without
    /// allocating per row, so reading forty thousand postings to answer a one-account
    /// question costs *time* and not *memory*. The counted-work figure is what moves — 1,014µs
    /// against 0.8µs in the audit — and `read_stats().rows_touched` is where it is visible.
    ///
    /// So this asserts the number of base rows the three spellings of one question touch. It
    /// is deterministic, unlike a wall clock, and it fails if either cliff comes back.
    #[test]
    fn one_account_is_answered_by_reading_one_account() {
        use crate::session::Serving;
        let accounts = 200i64;
        let rows_for = |sql: &str| -> u64 {
            let e = RevEngine::seeded(accounts, 2, 50, ViewMode::Demand, EvictionPolicy::Lru);
            let anchor = e.frontier();
            let lowered = compile(sql);
            let before = e.read_stats().rows_touched;
            e.query(&lowered.circuit, "__wire_result", anchor)
                .expect("answers");
            e.read_stats().rows_touched - before
        };

        // The whole base is 200 accounts x 2 rounds x 2 legs = 800 postings. A query that
        // names one account must not read all of them.
        let whole = rows_for("select acct, sum(amt) from postings group by acct");
        assert_eq!(whole, 800, "the control reads the base");

        for sql in [
            "select acct, sum(amt) from postings where acct = 7 and cur = 0 group by acct",
            "select acct, sum(amt) from postings group by acct having acct = 7",
            "select acct, count(amt) from postings where acct = 7 and cur = 0 group by acct",
        ] {
            let touched = rows_for(sql);
            assert!(
                touched * 20 < whole,
                "`{sql}` touched {touched} of {whole} base rows: the scan was not restricted \
                 to the account the query names"
            );
        }

        // And the shapes that must *not* be restricted still read the base, because
        // restricting them would drop rows.
        for sql in [
            "select acct, sum(amt) from postings where acct = 7 or acct = 9 group by acct",
            "select acct, sum(amt) from postings where cur = 0 group by acct",
        ] {
            assert_eq!(
                rows_for(sql),
                whole,
                "`{sql}` was restricted, and neither of its conditions is necessary"
            );
        }
    }

    /// **A warm report touches no base rows, and that is counted rather than timed.**
    ///
    /// The claim T-33 exists to make testable: a view maintained through every append already
    /// holds the answer, so producing it reads nothing. A timing would be flaky and would not
    /// say *why* it was fast; `rows_touched` says exactly which path served the query.
    #[test]
    fn a_warm_report_is_served_by_the_maintained_view() {
        use crate::session::Serving;
        // A budget of `usize::MAX` is the ask for a view that never evicts. Without that the
        // view is partial and a report must not come from it — see the case below.
        let e = RevEngine::seeded(200, 2, usize::MAX, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();
        let lowered = compile("select acct, sum(amt) from postings group by acct");

        let before = e.read_stats().rows_touched;
        let served = e
            .query(&lowered.circuit, "__wire_result", anchor)
            .expect("answers");
        let touched = e.read_stats().rows_touched - before;

        assert_eq!(
            touched, 0,
            "a warm report read {touched} base rows; the maintained view is supposed to hold \
             the answer already, and a report that folds is the cold path wearing the warm \
             path's name"
        );

        // And it is the *right* answer, judged by the reference evaluator over the base —
        // fast and wrong is the only outcome worse than slow.
        let sources = e.base_at(anchor);
        let (oracle, _) = niles_ir::eval::try_run(&lowered.circuit, "__wire_result", &sources)
            .expect("the reference answers");
        assert_eq!(
            served.text(),
            rendered(&oracle, anchor),
            "the maintained view and the base disagree about every account's balance"
        );
        assert!(served.len() > 100, "and it is a report, not a row");
    }

    /// **`explain` names the path the engine will actually take, not the circuit's.**
    ///
    /// The report row of E16 claims to be answered out of a maintained view; a reader who
    /// wants to check that should be able to ask the server. Before this, `explain` read the
    /// circuit alone and answered `fold` for a statement the engine was about to answer
    /// without touching a base row — which is the one case where a static class is wrong,
    /// because the view's contract and how far it has been advanced are not in the circuit.
    #[test]
    fn explain_names_the_report_path_only_when_the_engine_would_take_it() {
        use crate::session::Serving;
        let report = compile("select acct, sum(amt) from postings group by acct");

        let full = RevEngine::seeded(200, 2, usize::MAX, ViewMode::Demand, EvictionPolicy::Lru);
        let at = full.frontier();
        assert_eq!(
            Serving::serve_path_now(&full, &report.circuit, "__wire_result", at),
            "report-from-view",
            "a full view at the anchor being asked about answers a report without the base"
        );

        let partial = RevEngine::seeded(200, 2, 20, ViewMode::Demand, EvictionPolicy::Lru);
        let at = partial.frontier();
        assert_eq!(
            Serving::serve_path_now(&partial, &report.circuit, "__wire_result", at),
            "fold",
            "and a partial view does not, however many entries happen to be resident"
        );

        // The static answer is what it always was for the shapes that do not depend on the
        // engine, so this did not become a second opinion about the other four classes.
        let point = compile("select acct, sum(amt) from postings where acct = 7 group by acct");
        assert_eq!(
            Serving::serve_path_now(&full, &point.circuit, "__wire_result", at),
            serve_path(&point.circuit, "__wire_result").as_str()
        );
    }

    /// **A report's certification and its rows come from the same instant — A10-02.**
    ///
    /// The check that the view is applied exactly through `anchor` and the copy of the
    /// resident rows used to be two separate acquisitions of the view lock. An append
    /// landing between them advances the view, so the values copied were those at some
    /// `e > anchor` while the reply carried `anchor`. Nothing in the reply says so, and a
    /// client that asked for a snapshot got a later one under the earlier one's name — the
    /// quietest way a bitemporal system can lie.
    ///
    /// This drives the sequence directly rather than racing for it: certify at `anchor`,
    /// append (which advances the view), then ask for the report at `anchor`. A build that
    /// certifies once and copies later serves the post-append values; this one declines and
    /// the general fold path answers exactly at `anchor`.
    #[test]
    fn a_report_is_not_served_from_a_view_that_moved_after_it_was_certified() {
        use crate::session::Serving;
        let e = RevEngine::seeded(8, 2, usize::MAX, ViewMode::Demand, EvictionPolicy::Lru);
        let report = compile("select acct, sum(amt) from postings group by acct");
        let anchor = e.frontier();
        assert_eq!(
            Serving::serve_path_now(&e, &report.circuit, "__wire_result", anchor),
            "report-from-view",
            "PRECONDITION UNMET: the view must be able to serve this report at `anchor`, or \
             the certification being tested never happens"
        );
        let before = e
            .query(&report.circuit, "__wire_result", anchor)
            .expect("serves")
            .text();

        // The append advances the view past `anchor`. This is the event that used to slip
        // between the certification and the copy.
        e.append(
            vec![
                Row::Post(proto_engine::Posting {
                    txn: 4242,
                    acct: 1,
                    cur: 0,
                    amt: 100_000,
                    valid: 0,
                }),
                Row::Post(proto_engine::Posting {
                    txn: 4242,
                    acct: 2,
                    cur: 0,
                    amt: -100_000,
                    valid: 0,
                }),
            ],
            "report-race",
        )
        .expect("a balanced transfer is admitted");
        assert!(
            e.frontier() > anchor,
            "PRECONDITION UNMET: the append did not move the frontier, so there is no \
             divergence for this test to be about"
        );

        // The same question, at the same anchor, after the view moved.
        let after = e
            .query(&report.circuit, "__wire_result", anchor)
            .expect("serves")
            .text();
        assert_eq!(
            before, after,
            "the report at anchor {anchor} changed when the view advanced past it"
        );

        // And the engine says so rather than serving a stale label: at an anchor the view
        // has passed, the report path declines and the fold answers.
        assert_eq!(
            Serving::serve_path_now(&e, &report.circuit, "__wire_result", anchor),
            "fold",
            "once the view is past `anchor`, the maintained report is a different question \
             and must not claim to answer this one"
        );
    }

    /// **The append lands between the certification and the copy — A10-02, witnessed.**
    ///
    /// The test above shows that a build deciding the report path from `is_full()` alone
    /// claims `report-from-view` for an anchor the view has passed. It does *not* witness the
    /// divergence in the values, because in that build the certification simply fails and the
    /// fold answers correctly. The hazard is narrower and needs the append to land in the
    /// window **between** a certification that succeeded and the copy that follows it, and a
    /// window is not something to race for.
    ///
    /// So it is latched. `report_race_hook` parks the reporting thread at the point between
    /// deciding the plan's shape and taking the view. In this build nothing has been
    /// certified when it parks, so the append is seen by the single guard below and the
    /// report declines — the fold answers exactly at `anchor`. In the build where the
    /// certification happens above that point, the same append moves the view under a
    /// decision already made and the copy returns the new values under the old label.
    ///
    /// Bounded: the reporting thread reports through a channel with a deadline, so a build
    /// that parks forever fails rather than hanging the suite.
    #[test]
    fn a_report_certified_before_an_append_does_not_serve_the_values_after_it() {
        use crate::session::Serving;
        use std::time::Duration;

        let e = std::sync::Arc::new(RevEngine::seeded(
            8,
            2,
            usize::MAX,
            ViewMode::Demand,
            EvictionPolicy::Lru,
        ));
        let report = compile("select acct, sum(amt) from postings group by acct");
        let anchor = e.frontier();
        assert_eq!(
            Serving::serve_path_now(&*e, &report.circuit, "__wire_result", anchor),
            "report-from-view",
            "PRECONDITION UNMET: the view must be able to serve this report at `anchor`"
        );
        let expected = e
            .query(&report.circuit, "__wire_result", anchor)
            .expect("serves")
            .text();

        let (tx, rx) = std::sync::mpsc::channel();
        let e2 = e.clone();
        let circuit = report.circuit.clone();
        std::thread::spawn(move || {
            report_race_hook::arm_this_thread();
            let rows = e2
                .query(&circuit, "__wire_result", anchor)
                .map(|r| r.text());
            let _ = tx.send(rows);
        });
        assert!(
            report_race_hook::wait_until_reached(Duration::from_secs(10)),
            "PRECONDITION UNMET: the reporting thread never reached the rendezvous, so this \
             run witnesses nothing either way"
        );

        // Parked. Now move the view past `anchor`.
        e.append(
            vec![
                Row::Post(proto_engine::Posting {
                    txn: 5252,
                    acct: 1,
                    cur: 0,
                    amt: 777_000,
                    valid: 0,
                }),
                Row::Post(proto_engine::Posting {
                    txn: 5252,
                    acct: 2,
                    cur: 0,
                    amt: -777_000,
                    valid: 0,
                }),
            ],
            "report-race-latched",
        )
        .expect("a balanced transfer is admitted");
        assert!(
            e.frontier() > anchor,
            "PRECONDITION UNMET: the append did not move the frontier"
        );
        report_race_hook::release();

        let got = rx
            .recv_timeout(Duration::from_secs(10))
            .unwrap_or_else(|_| {
                report_race_hook::disarm();
                panic!("the reporting thread never returned within 10s")
            })
            .expect("the query is served one way or the other");
        report_race_hook::disarm();

        assert_eq!(
            got, expected,
            "the report answered at anchor {anchor} with the values the view holds AFTER an \
             append that moved it past {anchor}. The certification and the copy were \
             separated by that append, so the reply is a snapshot that never existed and is \
             labelled with one that did."
        );
    }

    /// **An anchor the view has not been advanced to is not answered from the view.**
    ///
    /// A report is a set of rows true at one moment. Serving the view's own moment for a
    /// different one asked about would be answering as of a time nobody asked about, which
    /// is the quietest way a bitemporal system can lie.
    #[test]
    fn a_report_at_an_anchor_the_view_has_not_reached_is_not_served_from_it() {
        use crate::session::Serving;
        let e = RevEngine::seeded(200, 2, usize::MAX, ViewMode::Demand, EvictionPolicy::Lru);
        let report = compile("select acct, sum(amt) from postings group by acct");
        let now = e.frontier();
        assert_eq!(
            Serving::serve_path_now(&e, &report.circuit, "__wire_result", now),
            "report-from-view"
        );
        assert_eq!(
            Serving::serve_path_now(&e, &report.circuit, "__wire_result", now.saturating_sub(1)),
            "fold",
            "an earlier anchor is a different question and the fold answers it"
        );
    }

    /// **A partial view never serves a report.**
    ///
    /// Its resident entries are the accounts that happen to be in memory. Serving those as
    /// "every account's balance" would be answering a different question in the shape of the
    /// one that was asked — the single most dangerous thing the engine could do with partial
    /// state, and the reason `is_full` exists.
    #[test]
    fn a_partial_view_never_serves_a_report() {
        use crate::session::Serving;
        // A budget well below the key count, so the view evicts and is never full.
        let e = RevEngine::seeded(200, 2, 20, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();

        // Warm it, so it has resident entries a careless report would happily return.
        let point = compile("select acct, sum(amt) from postings where acct = 7 group by acct");
        for _ in 0..30 {
            let _ = e.query(&point.circuit, "__wire_result", anchor);
        }

        let lowered = compile("select acct, sum(amt) from postings group by acct");
        let before = e.read_stats().rows_touched;
        let served = e
            .query(&lowered.circuit, "__wire_result", anchor)
            .expect("answers");
        let touched = e.read_stats().rows_touched - before;

        assert!(
            touched > 0,
            "a report over a partial view was answered without reading the base, which means \
             it was answered from whichever accounts were resident"
        );
        let sources = e.base_at(anchor);
        let (oracle, _) = niles_ir::eval::try_run(&lowered.circuit, "__wire_result", &sources)
            .expect("the reference answers");
        assert_eq!(
            served.text(),
            rendered(&oracle, anchor),
            "and it is still every account, folded from the base"
        );
    }
    /// A `sum` over no non-null rows is **null**, and the fold must say so too.
    ///
    /// Called out separately because it is the one place a one-pass accumulator most easily
    /// goes wrong: an `i128` starting at zero reports zero for a group that summed nothing,
    /// and a balance of "nothing was ever posted" rendered as `0.00` is §1.1.1's defect
    /// wearing an aggregate's clothes.
    #[test]
    fn a_sum_over_no_rows_is_absent_rather_than_zero() {
        use crate::session::Serving;
        let e = RevEngine::seeded(20, 2, 10, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();
        let lowered = compile("select cur, sum(amt) from postings where amt > 100000 group by cur");
        assert!(crate::scan_fold::plan(&lowered.circuit, "__wire_result").is_some());
        let out = e
            .query(&lowered.circuit, "__wire_result", anchor)
            .expect("answers");
        assert!(
            out.is_empty(),
            "no row passes the filter, so there is no group and no row: {:?}",
            out.rows
        );

        // And a group that exists but whose aggregated expression is null everywhere: `idem`
        // is the text column, materialised as null, so summing it forms groups with no
        // non-null value in them.
        let lowered = compile("select cur, sum(idem) from postings group by cur");
        assert!(crate::scan_fold::plan(&lowered.circuit, "__wire_result").is_some());
        let by_fold = e
            .query(&lowered.circuit, "__wire_result", anchor)
            .expect("answers");
        let sources = e.base_at(anchor);
        let (oracle, _) =
            niles_ir::eval::try_run(&lowered.circuit, "__wire_result", &sources).expect("oracle");
        assert_eq!(by_fold.text(), rendered(&oracle, anchor));
        assert_eq!(
            by_fold.text(),
            vec![vec![Some("0".into()), None, Some(anchor.to_string())]],
            "the group is there and its sum is absent, not zero"
        );
    }

    /// **The wire path reads the maintained view, and the miss rate is a measurement.**
    ///
    /// `read_stats` used to return `(served, 0, served, …)` — hits zero *by construction* —
    /// so every benchmark row reported a miss rate of 1.00 whatever the engine did, and the
    /// phase diagram's mechanism was measured by nothing the daemon ran. This asserts the
    /// three things that had to become true: a single-account read is answered by the REV
    /// runtime, its hits are counted, and a budget smaller than the key space produces
    /// misses that are real reconstructions rather than a number nobody can move.
    #[test]
    fn a_single_account_read_is_served_by_the_maintained_view() {
        use crate::session::Serving;
        // A budget of a quarter of the key space, as the benchmark configures: small enough
        // that eviction happens and the miss path is exercised.
        let e = RevEngine::seeded(200, 2, 50, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();

        let hot = compile("select acct, sum(amt) from postings where acct = 7 group by acct");
        // Reading one key repeatedly: the first is a miss, the rest are hits, and the answer
        // is the same every time.
        let first = e
            .query(&hot.circuit, "__wire_result", anchor)
            .expect("answers");
        for _ in 0..50 {
            let again = e
                .query(&hot.circuit, "__wire_result", anchor)
                .expect("answers");
            assert_eq!(again.rows, first.rows, "a hit must answer what a miss did");
        }
        let s = e.read_stats();
        let (reads, hits, misses, resident) = (s.reads, s.hits, s.misses, s.resident);
        assert_eq!(reads, 51, "every read reached the view");
        assert!(hits >= 50, "a warm key must hit: {hits} of {reads}");
        assert!(misses >= 1, "and the first read of it must not");
        assert!(resident > 0, "a read materialises the key it answered");

        // And the answer is the one the reference evaluator gives, so serving from the view
        // is a faster route to the same denotation rather than a second one.
        let sources = e.base_at(anchor);
        let (oracle, _) =
            niles_ir::eval::try_run(&hot.circuit, "__wire_result", &sources).expect("oracle");
        assert_eq!(first.text(), rendered(&oracle, anchor));

        // Sweeping the whole key space against a budget of 50 forces eviction, so the rate
        // is a property of the workload rather than of the wiring.
        for acct in 1..=200i64 {
            let sql =
                format!("select acct, sum(amt) from postings where acct = {acct} group by acct");
            let c = compile(&sql);
            let got = e
                .query(&c.circuit, "__wire_result", anchor)
                .expect("answers");
            let sources = e.base_at(anchor);
            let (oracle, _) =
                niles_ir::eval::try_run(&c.circuit, "__wire_result", &sources).expect("oracle");
            assert_eq!(got.text(), rendered(&oracle, anchor), "acct {acct}");
        }
        let s = e.read_stats();
        let (reads, hits, misses, resident) = (s.reads, s.hits, s.misses, s.resident);
        assert_eq!(reads, hits + misses);
        assert!(
            misses > 51,
            "a budget below the key space must evict and reconstruct: {misses} misses in \
             {reads} reads"
        );
        assert!(
            resident <= 50,
            "the residency budget must bind: {resident} entries held against a budget of 50"
        );
        let rate = misses as f64 / reads as f64;
        assert!(
            rate > 0.0 && rate < 1.0,
            "the miss rate must be a measurement rather than a constant: {rate}"
        );
    }

    /// An account nothing has posted to produces **no row**, not a balance of zero, on the
    /// view path as on every other.
    #[test]
    fn an_untouched_account_has_no_balance_on_the_view_path_either() {
        use crate::session::Serving;
        let e = RevEngine::seeded(20, 2, 10, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();
        let c = compile("select acct, sum(amt) from postings where acct = 999999 group by acct");
        let got = e
            .query(&c.circuit, "__wire_result", anchor)
            .expect("answers");
        assert!(
            got.is_empty(),
            "an account with no postings forms no group: {:?}",
            got.rows
        );
        let sources = e.base_at(anchor);
        let (oracle, _) =
            niles_ir::eval::try_run(&c.circuit, "__wire_result", &sources).expect("oracle");
        assert_eq!(got.text(), rendered(&oracle, anchor));
    }

    /// A write is visible to the next read, which is what `advance` on the append path buys.
    #[test]
    fn a_read_after_a_write_sees_it() {
        use crate::session::Serving;
        let e = RevEngine::seeded(50, 2, 20, ViewMode::Demand, EvictionPolicy::Lru);
        let c = compile("select acct, sum(amt) from postings where acct = 3 group by acct");
        let before_anchor = e.frontier();
        let before = e
            .query(&c.circuit, "__wire_result", before_anchor)
            .expect("answers");
        let was: i128 = before.text()[0][1].as_ref().unwrap().parse().unwrap();

        e.append(
            vec![
                Row::Post(Posting {
                    txn: 77_777,
                    acct: 3,
                    cur: 0,
                    amt: -250,
                    valid: 0,
                }),
                Row::Post(Posting {
                    txn: 77_777,
                    acct: 0,
                    cur: 0,
                    amt: 250,
                    valid: 0,
                }),
            ],
            "t-05-write",
        )
        .expect("a conserved append");

        let anchor = e.frontier();
        let after = e
            .query(&c.circuit, "__wire_result", anchor)
            .expect("answers");
        let now: i128 = after.text()[0][1].as_ref().unwrap().parse().unwrap();
        assert_eq!(
            now,
            was - 250,
            "a maintained view that did not advance would answer the old balance, which is \
             the failure mode a cached read model has and a reconstruction does not"
        );
        let sources = e.base_at(anchor);
        let (oracle, _) =
            niles_ir::eval::try_run(&c.circuit, "__wire_result", &sources).expect("oracle");
        assert_eq!(after.text(), rendered(&oracle, anchor));
    }

    #[test]
    fn the_pushdown_and_the_full_scan_agree() {
        // **The obligation the optimisation carries.** Restricting the source to the rows a
        // `Filter` would keep cannot change what the circuit denotes, and this is the test
        // that says so rather than the comment: the same circuit is evaluated over the
        // restricted base and over the whole one, and the two must be identical.
        //
        // Without the restriction the point workload measured 68 operations per second —
        // a full scan of twenty thousand postings per query. With it, and with no
        // difference in the answer, it is back in the thousands.
        use crate::session::Serving;
        let e = RevEngine::seeded(200, 3, 50, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();
        for acct in [1i64, 7, 42, 199] {
            let sql =
                format!("select acct, sum(amt) from postings where acct = {acct} group by acct");
            let program = format!(
                "{}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};\n",
                crate::daemon::DEFAULT_SCHEMA
            );
            let (prog, d) = niles_lang::parser::parse_program(&program);
            assert!(!d.has_errors(), "{:?}", d.items);
            let (cat, _) = niles_lang::resolve::resolve_program(&prog, 0);
            let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
            assert!(!ld.has_errors(), "{:?}", ld.items);

            // The pushdown must have fired, or the test is comparing one path with itself.
            assert_eq!(
                account_predicate(&lowered.circuit),
                Some(acct as u64),
                "the predicate on `acct = {acct}` must be recognised"
            );

            let pushed = e
                .query(&lowered.circuit, "__wire_result", anchor)
                .expect("pushed");
            let full = {
                let sources = e.base_at(anchor);
                let (z, _) = niles_ir::eval::try_run(&lowered.circuit, "__wire_result", &sources)
                    .expect("full");
                z
            };
            let from_full: Vec<Vec<Option<String>>> = full
                .iter()
                .filter(|(_, w)| **w > 0)
                .map(|(r, _)| {
                    let mut cells: Vec<Option<String>> = r
                        .iter()
                        .map(|v| match v {
                            niles_ir::value::Value::Null => None,
                            niles_ir::value::Value::Int(i) => Some(i.to_string()),
                        })
                        .collect();
                    cells.push(Some(anchor.to_string()));
                    cells
                })
                .collect();
            assert_eq!(
                pushed.text(),
                from_full,
                "the restricted scan and the full scan must denote the same thing for acct {acct}"
            );
        }
    }

    #[test]
    fn a_predicate_the_pushdown_does_not_understand_abandons_the_restriction() {
        // The safe direction. A pushdown that guessed would restrict a scan the circuit
        // does not restrict, and the answer would be missing rows — the one thing an
        // optimisation must never do. Two different accounts in one circuit, a predicate on
        // a column that is not the key, and an inequality all fall back to the full scan.
        use niles_ir::operator::{Op, Scalar, ScalarOp};
        let filter = |p: Scalar| {
            let mut c = niles_ir::circuit::Circuit::new();
            let contract = niles_ir::circuit::internal_contract();
            let src = c.add(
                Op::Source {
                    relation: "postings".into(),
                    is_base: true,
                    anchor_key: vec![1],
                    confidential: Vec::new(),
                },
                vec![],
                contract,
                "postings",
            );
            c.add(Op::Filter { predicate: p }, vec![src], contract, "where");
            c
        };
        let eq = |col: u16, k: i128| Scalar::Binary {
            op: ScalarOp::Eq,
            lhs: Box::new(Scalar::Column(col)),
            rhs: Box::new(Scalar::LitInt(k)),
        };
        assert_eq!(account_predicate(&filter(eq(1, 7))), Some(7));
        assert_eq!(
            account_predicate(&filter(eq(3, 7))),
            None,
            "a predicate on `amt` says nothing about which accounts are needed"
        );
        assert_eq!(
            account_predicate(&filter(Scalar::Binary {
                op: ScalarOp::Gt,
                lhs: Box::new(Scalar::Column(1)),
                rhs: Box::new(Scalar::LitInt(7)),
            })),
            None,
            "an inequality names a range, not a key"
        );
    }
    use super::*;
    use crate::session::Serving;

    fn engine() -> RevEngine {
        RevEngine::seeded(100, 3, 1_000, ViewMode::Demand, EvictionPolicy::Lru)
    }

    /// **One account's balance, through the wire path.**
    ///
    /// The tests below used to call `RevEngine::read_point`, which read a `PartialView` that
    /// sat beside the REV runtime and that no query ever consulted. They warmed one read
    /// model and asserted about the other, and would have passed with the served path
    /// disconnected entirely. They now ask the same question the way a client does.
    ///
    /// `None` is "this account has no balance", which is not zero: an aggregate over no rows
    /// forms no group, so the answer is no row at all.
    fn balance(e: &mut RevEngine, acct: i64, anchor: u64) -> Option<i128> {
        use crate::session::Serving;
        let lowered = compile(&format!(
            "select acct, sum(amt) from postings where acct = {acct} group by acct"
        ));
        let rows = e
            .query(&lowered.circuit, "__wire_result", anchor)
            .expect("serves");
        let text = rows.text();
        let r = text.first()?;
        r[1].as_ref().map(|v| v.parse().expect("an integer"))
    }

    /// `read_stats`, which **is** the named struct now: this used to re-wrap a five-tuple so
    /// a test would read like a claim rather than like a positional destructure, and the
    /// trait returns the struct itself since T-02.
    fn stats(e: &RevEngine) -> ReadStats {
        use crate::session::Serving;
        e.read_stats()
    }

    #[test]
    fn a_read_is_answered_by_the_read_model_over_the_ledger() {
        let mut e = engine();
        // Epochs are zero-based, so 300 transfers put the head at 299.
        assert_eq!(
            e.frontier(),
            299,
            "one epoch per seeded transfer, zero-based"
        );
        let head = e.frontier();
        let v = balance(&mut e, 7, head).expect("account 7 has postings");
        assert!(v > 0, "a real fold, not a placeholder: {v}");
        assert!(stats(&e).reads > 0);
    }

    #[test]
    fn an_account_the_base_never_touched_has_no_value_rather_than_zero() {
        // The absence lattice reaching the wire. A server that answered zero here would make
        // "this account does not exist" indistinguishable from "this account has no money",
        // which is the §1.1.1 defect at the protocol boundary.
        let mut e = engine();
        let head = e.frontier();
        assert_eq!(balance(&mut e, 9_999, head), None);
        assert!(balance(&mut e, 1, head).is_some());
    }

    #[test]
    fn a_read_at_a_past_anchor_answers_what_was_true_then() {
        let mut e = engine();
        let head = e.frontier();
        let late = balance(&mut e, 5, head).unwrap();
        let early = balance(&mut e, 5, 5).unwrap();
        assert!(early < late, "the past holds less: {early} vs {late}");

        // And it is stable: asking again gives the same answer, because the prefix cannot
        // change and the reconstruction is a pure function of (key, anchor).
        for _ in 0..20 {
            assert_eq!(balance(&mut e, 5, 5), Some(early));
        }
    }

    #[test]
    fn eviction_forces_a_reconstruction_and_the_answer_is_unchanged() {
        // The property Contribution 1 is about, exercised through the wire path: eviction
        // followed by reconstruction can neither create nor destroy money.
        let mut e = engine();
        let head = e.frontier();
        let warm: Vec<Option<i128>> = (1..=20).map(|a| balance(&mut e, a, head)).collect();
        let hits_before = stats(&e).hits;

        e.evict_all();
        assert_eq!(
            stats(&e).resident,
            0,
            "and it wipes the view the wire path reads, not one beside it"
        );

        let cold: Vec<Option<i128>> = (1..=20).map(|a| balance(&mut e, a, head)).collect();
        assert_eq!(warm, cold, "eviction changed an answer");
        assert!(
            stats(&e).misses > 0,
            "and the cold reads really did reconstruct"
        );
        assert!(stats(&e).rows_touched > 0, "touching base rows to do it");
        assert!(stats(&e).hits >= hits_before);
    }

    #[test]
    fn the_miss_rate_is_reportable_because_a_parity_result_depends_on_it() {
        // A parity result at a 0% miss rate and one at a 40% miss rate are different
        // findings. A server that reported only latency would let the more interesting of
        // the two disappear.
        //
        // Through `query` and `read_stats`, so the rate reported is the rate of the reads
        // the server actually answered. The version this replaced drove `read_point` and
        // read a `PartialView` the wire path never touched: the three rates below were true
        // of a cache no query consulted.
        let mut e = engine();
        let head = e.frontier();
        let pass = |e: &mut RevEngine| {
            for a in 1..=50 {
                balance(e, a, head);
            }
        };

        // Pass one is cold: in demand mode a key is materialised when it is first read, so
        // every one of these misses and reconstructs.
        pass(&mut e);
        let cold = stats(&e);
        assert_eq!(cold.misses, 50, "every first touch is a miss");
        assert!((cold.miss_rate() - 1.0).abs() < 1e-9);

        // Pass two is warm: the same keys, now resident, so the cumulative rate halves.
        pass(&mut e);
        let warm = stats(&e);
        assert_eq!(warm.misses, 50, "no new misses");
        assert_eq!(warm.hits, 50, "and fifty hits");
        assert!(
            (warm.miss_rate() - 0.5).abs() < 1e-9,
            "{}",
            warm.miss_rate()
        );

        // Eviction puts it back: the third pass reconstructs everything again.
        e.evict_all();
        pass(&mut e);
        let after = stats(&e);
        assert!(
            after.miss_rate() > warm.miss_rate(),
            "eviction raises the miss rate"
        );
        assert_eq!(after.misses, 100);
        assert!(
            after.rows_touched > cold.rows_touched,
            "and it paid for it in base rows"
        );
        assert!((0.0..=1.0).contains(&after.miss_rate()));
    }

    #[test]
    fn an_anchor_beyond_the_head_is_answered_at_the_head_rather_than_invented() {
        let mut e = engine();
        let head = e.frontier();
        let at_head = balance(&mut e, 3, head);
        assert_eq!(balance(&mut e, 3, u64::MAX), at_head);
    }

    #[test]
    fn an_empty_ledger_answers_nothing_rather_than_zero() {
        let mut e = RevEngine::seeded(0, 0, 10, ViewMode::Demand, EvictionPolicy::Lru);
        assert_eq!(balance(&mut e, 1, 0), None);
    }

    #[test]
    fn epoch_zero_is_a_real_record_and_is_readable() {
        // Zero-based epochs, and an engine that treated 0 as "nothing" would make the
        // ledger's first transaction invisible — an off-by-one a conservation suite finds
        // three months later.
        let mut e = RevEngine::seeded(3, 1, 100, ViewMode::Demand, EvictionPolicy::Lru);
        assert_eq!(e.frontier(), 2, "three transfers, epochs 0..=2");
        assert!(
            balance(&mut e, 1, 0).is_some(),
            "account 1 posted at epoch 0"
        );
        assert_eq!(balance(&mut e, 2, 0), None, "account 2 has not yet");
        assert!(balance(&mut e, 2, 1).is_some());
    }

    #[test]
    fn a_historical_read_reconstructs_rather_than_reusing_a_fresher_slot() {
        // A partially materialised view treats a slot anchored later as a hit for an earlier
        // read — correct for a "no older than X" rung, wrong for "as of X", which is what a
        // dispute asks. Warming the view at the head must not change what a past anchor
        // answers, and `answer_from_view` enforces that by falling back to the fold whenever
        // the entry's effective anchor is not the one that was requested.
        let mut e = engine();
        let head = e.frontier();
        let past = 5;
        let cold = balance(&mut e, 5, past).unwrap();

        for a in 1..=50 {
            balance(&mut e, a, head);
        }
        assert_eq!(
            balance(&mut e, 5, past),
            Some(cold),
            "a warm view answered a historical query with a fresher value"
        );
        assert_ne!(balance(&mut e, 5, head), Some(cold), "and the head differs");
    }

    /// **An empty answer describes the same columns as a non-empty one.**
    ///
    /// The width came from the first row, so a statement that matched nothing described one
    /// column and the same statement with rows described three. A client that prepared once
    /// and executed twice would see its result schema change with the data — and a benchmark
    /// or a driver that trusted the description would be reading a different shape from the
    /// one it was told about.
    #[test]
    fn an_empty_answer_describes_its_columns() {
        use crate::session::Serving;
        let e = RevEngine::seeded(50, 1, 25, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();
        let full = compile("select acct, sum(amt) from postings where acct = 7 group by acct");
        let empty =
            compile("select acct, sum(amt) from postings where acct = 999999 group by acct");
        let with_rows = e
            .query(&full.circuit, "__wire_result", anchor)
            .expect("serves");
        let without = e
            .query(&empty.circuit, "__wire_result", anchor)
            .expect("serves");
        assert!(!with_rows.is_empty(), "the control has rows");
        assert!(without.is_empty(), "and the case has none");
        assert_eq!(
            with_rows.columns.len(),
            without.columns.len(),
            "the same statement shape must describe the same number of columns whether or \
             not it matched anything"
        );
        assert_eq!(
            with_rows.columns, without.columns,
            "and the same names: a client that reads the descriptor to lay out a report must \
             not have to guess the width of an empty answer"
        );
    }

    /// **`explain` and the engine agree, by construction and by test.**
    ///
    /// `serve_path` exists so a reader can find out what a statement costs without measuring
    /// it, and that is only true if it describes the engine rather than a second opinion
    /// about it. `query` branches on the same three functions, so the two agree by
    /// construction; this pins the classification itself, which is what a reader actually
    /// relies on — including the two shapes that used to be classified `materialise` and cost
    /// three orders of magnitude more than the question deserved.
    #[test]
    fn explain_names_the_path_the_engine_takes() {
        for (sql, want) in [
            (
                "select acct, sum(amt) from postings where acct = 7 group by acct",
                ServePath::View,
            ),
            (
                "select acct, cur, sum(amt) from postings where acct = 7 group by acct, cur",
                ServePath::View,
            ),
            // One account, but not a plain balance: the extra conjunct means the view — keyed
            // by account and currency — cannot answer it, so the scan is restricted and
            // folded. This used to be `materialise`.
            (
                "select acct, sum(amt) from postings where acct = 7 and cur = 0 group by acct",
                ServePath::IndexFold,
            ),
            // A `having` over the group key is a `where` in a different position. This used
            // to be `materialise` too.
            (
                "select acct, sum(amt) from postings group by acct having acct = 7",
                ServePath::IndexFold,
            ),
            // A count is in the fragment but is not a balance, so no view read.
            (
                "select acct, count(amt) from postings where acct = 7 group by acct",
                ServePath::IndexFold,
            ),
            // No account restriction: the whole base, in one pass.
            (
                "select acct, sum(amt) from postings group by acct",
                ServePath::Fold,
            ),
            (
                "select cur, sum(amt) from postings group by cur",
                ServePath::Fold,
            ),
            // An ordering above the aggregate means the output is not the aggregate, so the
            // view cannot answer even for one account — the reference evaluates the rest.
            (
                "select acct, sum(amt) from postings group by acct order by sum(amt) desc limit 10",
                ServePath::Fold,
            ),
            // Outside the fragment: `min` is not a running fold over a signed multiset.
            (
                "select acct, min(amt) from postings group by acct",
                ServePath::Materialise,
            ),
            ("select distinct acct from postings", ServePath::Materialise),
            // A disjunction is not a restriction, and the shape is otherwise a plain fold.
            (
                "select acct, sum(amt) from postings where acct = 7 or acct = 9 group by acct",
                ServePath::Fold,
            ),
        ] {
            let lowered = compile(sql);
            assert_eq!(
                serve_path(&lowered.circuit, "__wire_result"),
                want,
                "`{sql}`"
            );
        }
    }
}

#[cfg(test)]
mod sealer_stats_tests {
    //! **The counter that decides whether group commit reaches the wire.**
    //!
    //! `Sequencer` batches to 4,096 and is tested at sixteen concurrent submitters. The
    //! daemon holds one mutex across `Session::handle`, and `append` blocks inside that
    //! section until the sealer has fsynced — so submitters arrive at `submit` one at a
    //! time, the drain loop always finds an empty queue, and `max_batch` is 1 however many
    //! connections are open.
    //!
    //! Measured through the wire with this instrument, at 1/2/4/8 connections against a
    //! `--durable` daemon: 3,600 transactions, **3,600 fsyncs, `max_batch` 1, 1.00
    //! transactions per fsync at every level**, lock wait p50 256µs rising to a p99 of 16ms,
    //! and 2.33 seconds of the run spent with the engine locked. The sealer alone, driven
    //! directly, reaches `max_batch` 16 at 9.65 transactions per fsync on the same host.
    //!
    //! This module pins the instrument, not the defect: T-05 releases the lock and the
    //! scaling gate then requires `max_batch >= 8`. What must not happen in between is the
    //! counter silently going away.

    use super::*;
    use crate::session::Serving;

    fn tiny() -> RevEngine {
        RevEngine::seeded(
            8,
            1,
            4,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        )
    }

    /// The guard: remove the durable sink's stats plumbing and this fails.
    #[test]
    fn a_durable_engine_reports_the_sealers_counters_after_an_append() {
        let dir = std::env::temp_dir().join("nilestream-sealer-stats-test");
        let _ = std::fs::create_dir_all(&dir);
        let seg = dir.join("counters.seg");
        let _ = std::fs::remove_file(&seg);

        let e = tiny().with_durable(&seg).expect("durable sink");
        assert!(
            e.sealer_stats().is_some(),
            "a durable engine must expose its sealer's counters"
        );

        let before = e.sealer_stats().expect("durable").2;
        let applied = e
            .append(
                vec![proto_engine::Row::Post(proto_engine::Posting {
                    txn: 77,
                    acct: 1,
                    cur: 0,
                    amt: 0,
                    valid: 0,
                })],
                "sealer-stats-1",
            )
            .expect("appends");
        let applied_epoch = applied.epoch;

        // **`append` no longer waits for the barrier, so this must.**
        //
        // Before T-05 the fsync happened inside `append`, inside the engine's lock, and this
        // test could read `fsyncs` straight afterwards. That is the protocol that changed:
        // the caller applies, takes the token, releases the lock, and *then* waits — which is
        // what the daemon does between framing a reply and writing it. A test that skipped
        // the wait would be asserting on a barrier that had not happened yet.
        let receipt = applied
            .receipt
            .expect("a durable engine hands the append its own barrier receipt");
        receipt.wait().expect("the epoch reaches stable storage");
        assert_eq!(
            e.frontier(),
            applied_epoch,
            "the visible frontier advances to the epoch once, and only once, it is durable"
        );

        let (epochs, txns, fsyncs, _dups, max_batch) = e.sealer_stats().expect("durable");

        assert!(
            fsyncs > before,
            "an append under SyncPolicy::Always must have fsynced: {fsyncs} <= {before}"
        );
        assert!(epochs >= 1 && txns >= 1, "{epochs} epochs, {txns} txns");
        assert!(
            max_batch >= 1,
            "a batch of at least one transaction was sealed"
        );
        let _ = std::fs::remove_file(&seg);
    }

    /// **`None` and all-zeroes are different facts.**
    ///
    /// A server that cannot batch and one that has not yet batched are not the same, and a
    /// benchmark reading `max_batch = 0` must be able to tell which it is looking at.
    #[test]
    fn a_volatile_engine_reports_no_sealer_rather_than_a_sealer_that_did_nothing() {
        assert_eq!(
            tiny().sealer_stats(),
            None,
            "an engine with no durable sink has no sealer, and must not report one idling"
        );
    }
}

#[cfg(test)]
mod lock_order_tests {
    //! **One order for the two locks, and a test that hangs if it is two.**
    //!
    //! `RevEngine` holds four things behind locks. In the order a thread may acquire them:
    //!
    //! | | what | taken by |
    //! |---|---|---|
    //! | **B** | the base, `RwLock<Ledger>` | shared by every read, exclusively by `append` |
    //! | **P** | the pending barriers, `Mutex<Vec<Pending>>` | `append`, `take_pending` |
    //! | **V** | the read model, `Mutex<Runtime>` | `append`, and every read of the view |
    //! | **C** | the currency set, `RwLock<BTreeSet<u32>>` | `append`; read paths take and release it before V |
    //! | **F** | one flight's completion, a condvar inside the view | `finish_fold` to publish; a joined reader to receive |
    //!
    //! **F is strictly below all of them and is never held while anything is acquired.** A
    //! keyed read that finds a reconstruction already in flight at its own anchor returns
    //! from `answer_from_view` — dropping the base guard on the way out — and waits in the
    //! caller with nothing held at all. Waiting inside the read would park a thread holding
    //! **B**, which blocks every append in the process behind another reader's fold: a worse
    //! serialisation than the one the two-phase split removes, and the reason the wait is
    //! the caller's and not the read's.
    //!
    //! **B < P < V < C**, and every path takes a subsequence of that. The one that did not
    //! was `answer_from_view`, which took V and then reached for B underneath it while
    //! `append` took B and then reached for V — an AB–BA inversion between a point read and
    //! a concurrent append, reachable in the shipped daemon, and invisible to every test in
    //! this workspace because they all drive one workload at a time.
    //!
    //! Two guards, because neither alone is enough. The behavioural one only deadlocks when
    //! the interleaving happens, so it can pass on a lucky run; the source-level one holds
    //! whether or not the scheduler cooperates, and it is the one that will still be here
    //! when someone adds a fifth lock.

    use super::*;

    /// A copy of `s` with every `//` comment removed, line by line.
    ///
    /// **Every source-reading guard in this module needs it, and one of them learned why the
    /// hard way.** These guards read the *positions* of lock acquisitions and calls, and a
    /// comment that names one moves the position. The A10-01 repair carries a paragraph
    /// explaining that `read_stats` used to reach for `self.base()` under the view; with
    /// comments included, that sentence sat where the acquisition used to be, and the guard
    /// passed against the reverted deadlocking build on the strength of the prose describing
    /// the deadlock.
    fn code_only(s: &str) -> String {
        s.lines()
            .map(|l| match l.find("//") {
                Some(i) => &l[..i],
                None => l,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    use crate::session::Serving;

    /// **The deadlock of A10-01, witnessed and bounded — the stats cycle.**
    ///
    /// `read_stats` took V and then reached for B beneath it while `append` takes B and then
    /// V. Two connections — one running `select nilestream_stats`, one committing — is the
    /// whole of the schedule, and it is reachable from the wire. Cycle 9's own mixed sweep
    /// queries that table between levels, so the harness that measured the engine could hang
    /// the engine it was measuring, which is why this is repaired before anything is scored.
    ///
    /// **Latched, not raced.** A test that merely runs stats queries beside appends passes on
    /// a lucky schedule and says nothing. The rendezvous in `stats_order_hook` pauses the
    /// stats thread at the exact point between the two acquisitions, the appender is started
    /// while it is parked there, and only then is it released. On the broken order the
    /// paused thread holds V and wants B while the appender holds B and wants V; on the
    /// repaired order it holds nothing, because B was taken and released before the pause.
    ///
    /// **Bounded, so a red test is a failure and not a hung suite.** Each thread reports
    /// through a channel and the assertions are `recv_timeout`s. A deadlocked pair is left
    /// parked on an engine this test owns and nothing else can reach; the process is not
    /// waited on it, and no other test's locks are involved.
    #[test]
    fn a_stats_snapshot_and_a_concurrent_append_both_finish() {
        use std::sync::mpsc;
        use std::time::Duration;

        // Long enough that a slow machine is not called a deadlock, short enough that a
        // deadlock is not called a slow machine. The repaired path answers in microseconds.
        const DEADLINE: Duration = Duration::from_secs(10);

        let engine = std::sync::Arc::new(RevEngine::seeded(
            64,
            2,
            256,
            ViewMode::Demand,
            EvictionPolicy::Lru,
        ));

        let (stats_tx, stats_rx) = mpsc::channel();
        let (append_tx, append_rx) = mpsc::channel();

        let e = engine.clone();
        std::thread::spawn(move || {
            stats_order_hook::arm_this_thread();
            let s = Serving::read_stats(&*e);
            let _ = stats_tx.send(s.reads);
        });

        assert!(
            stats_order_hook::wait_until_reached(DEADLINE),
            "PRECONDITION UNMET: the stats thread never reached the rendezvous between its \
             base phase and its view phase, so this run witnesses nothing either way"
        );

        // Parked at the rendezvous. Now the appender, which takes B exclusively and then V.
        let e = engine.clone();
        std::thread::spawn(move || {
            let r = e.append(
                vec![
                    Row::Post(proto_engine::Posting {
                        txn: 77,
                        acct: 1,
                        cur: 0,
                        amt: 5,
                        valid: 0,
                    }),
                    Row::Post(proto_engine::Posting {
                        txn: 77,
                        acct: 2,
                        cur: 0,
                        amt: -5,
                        valid: 0,
                    }),
                ],
                "deadlock-witness",
            );
            let _ = append_tx.send(r.is_ok());
        });

        // Long enough for the appender to have taken B and to be inside or waiting for V.
        // If it has not started at all the test still holds: releasing the stats thread then
        // simply lets both run, which is the outcome being asserted.
        std::thread::sleep(Duration::from_millis(200));
        stats_order_hook::release();

        let appended = append_rx.recv_timeout(DEADLINE).unwrap_or_else(|_| {
            stats_order_hook::disarm();
            panic!(
                "the append never returned within {DEADLINE:?}. It holds the base \
                 exclusively and is waiting for the view; the stats snapshot holds the view \
                 and is waiting for the base. That is the cycle of A10-01, and no answer is \
                 ever wrong on the way into it — the connection simply never replies."
            )
        });
        let reads = stats_rx.recv_timeout(DEADLINE).unwrap_or_else(|_| {
            stats_order_hook::disarm();
            panic!(
                "the stats snapshot never returned within {DEADLINE:?}, having been \
                 released at the rendezvous: it is waiting for the base under the view"
            )
        });
        stats_order_hook::disarm();

        assert!(appended, "the witness transaction must actually commit");
        // The snapshot is not atomic across the two locks and this asserts nothing about
        // which side of the append it landed on; what it asserts is that it landed.
        let _ = reads;
    }

    /// The order is a property of the source, and reading it there is not a weaker test —
    /// it is the only one that cannot pass by luck.
    ///
    /// **Every function, not a list of two.** This guard named `answer_from_view` and
    /// `append`, which were the two paths that took both locks when it was written. A third
    /// appeared — `read_stats`, which took V and then reached for B underneath it — and the
    /// guard said nothing, because a hand-written list of the paths that must obey a rule
    /// only covers the paths someone remembered (A10-01). It now derives the list: every
    /// `fn` in this file that acquires both is checked, so the next path to take both is
    /// covered on the commit that introduces it rather than on the commit that finds it.
    ///
    /// A function that takes neither, or only one, is not the rule's business and is skipped
    /// — but the count of functions that take both is asserted to be non-trivial, so a
    /// change to how the locks are spelled cannot empty the scan and leave a green test
    /// asserting nothing.
    #[test]
    fn the_base_is_acquired_before_the_view_on_every_path_that_takes_both() {
        let src = include_str!("rev_engine.rs");
        // Bodies are delimited by the `    fn ` / `\n    }` shape every method in this file
        // has. A free function or a nested closure is inside some method's body and is
        // checked as part of it, which is what the rule is about anyway: the order of
        // acquisitions on one thread's path.
        let mut checked = 0;
        let mut names = Vec::new();
        for chunk in src.split("\n    fn ").skip(1) {
            let name = chunk
                .split(['(', '<'])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let body = &chunk[..chunk.find("\n    }").unwrap_or(chunk.len())];
            // **Code only.** The scan reads positions of lock acquisitions, and a comment
            // that *names* one moves that position. This is not hypothetical: the repair for
            // A10-01 carries a paragraph explaining that `read_stats` used to reach for
            // `self.base()` under the view, and with comments included that sentence sat
            // where the acquisition used to be — so the guard passed against the reverted,
            // deadlocking build, on the strength of the prose describing the deadlock.
            let body = code_only(body);
            let body = body.as_str();
            // The test module below contains this very test, whose text mentions both
            // spellings in prose and in assertions; scanning it would check the guard
            // against itself. Everything above `mod tests` is the engine.
            if src.find(&format!("\n    fn {name}")).unwrap_or(0)
                > src.find("\nmod tests {").unwrap_or(src.len())
            {
                continue;
            }
            let base = body
                .find("self.base()")
                .or_else(|| body.find("TimedWrite::acquire"))
                .or_else(|| body.find("TimedRead::acquire"));
            let view = body
                .find("VIEW_LOCK")
                .or_else(|| body.find(".lock()"))
                .or_else(|| body.find("view_mut("));
            let (Some(base), Some(view)) = (base, view) else {
                continue;
            };
            checked += 1;
            names.push(name.clone());
            assert!(
                base < view,
                "`{name}` takes the view at {view} and the base at {base}. Every path must \
                 take the base first: a thread holding the view while it waits for the base, \
                 against an append holding the base while it waits for the view, is a \
                 deadlock and no answer is ever wrong on the way into it."
            );
        }
        assert!(
            checked >= 3,
            "PRECONDITION UNMET: only {checked} function(s) were found to take both locks \
             ({names:?}). `answer_from_view`, `append` and `read_stats` all do, so a scan \
             that finds fewer is not recognising an acquisition and is asserting nothing."
        );
        for required in ["answer_from_view", "append", "read_stats"] {
            assert!(
                names.iter().any(|n| n == required),
                "the scan did not recognise `{required}` as taking both locks; it does, so \
                 the spellings this test matches on have fallen behind the source. Found: \
                 {names:?}"
            );
        }
    }

    /// A keyed read is not permitted to take the base twice.
    ///
    /// `RwLock` is not reentrant. A second `read()` on the same thread blocks behind a
    /// writer that arrived in between, so the fix for the inversion must be one guard held
    /// across the function and not two taken in the right order.
    #[test]
    fn a_keyed_read_takes_the_base_exactly_once() {
        let src = include_str!("rev_engine.rs");
        let body = src
            .split("fn answer_from_view(")
            .nth(1)
            .expect("the function");
        let body = &body[..body.find("\n    }").unwrap_or(body.len())];
        assert_eq!(
            body.matches("self.base()").count(),
            1,
            "`answer_from_view` acquires the base more than once. `RwLock` is not reentrant: \
             a writer arriving between the two makes the second acquisition block on a lock \
             this thread already holds."
        );
    }

    /// **A joined read takes the base once, and uses the answer it waited for — A10-04.**
    ///
    /// The `Wait` arm discarded `w.wait()`'s value and went round the loop. The retry
    /// re-entered `answer_from_view`, which acquires the base — so one logical keyed read
    /// acquired B twice, and a join, whose whole point is that the second reader pays
    /// nothing, cost a wait *plus* a full second pass. The answer it threw away was already
    /// exact at its own anchor: `WaitTicket::wait` returns `None` for a completion published
    /// at any other, so there was nothing left to check.
    ///
    /// Two things are asserted, and the second is the one that makes the first safe. The
    /// joined reader's rows must equal the owner's — a client must not be able to tell
    /// whether its read waited — and a joined value of **zero must not become a row** for an
    /// account the base has never posted to. That distinction cannot be re-derived after the
    /// wait without going back to the base, which is the acquisition being removed, so the
    /// flight publishes its `base_rows` alongside the value and the joiner reads it there.
    #[test]
    fn a_joined_read_answers_from_the_flight_and_never_reopens_the_base() {
        // **`resolve_join` is where a join becomes an answer**, so that is what is scanned.
        // The `Wait` arm is now one line in each of two implementations and the substance is
        // in the shared resolver.
        let src = include_str!("rev_engine.rs");
        let arm = src
            .split("\n    fn resolve_join(")
            .nth(1)
            .expect("`resolve_join` is where a join becomes an answer");
        let arm = &arm[..arm.find("\n    }").unwrap_or(arm.len())];
        assert!(
            arm.contains("rows_from_answer"),
            "the joined answer must be shaped and returned here. Discarding it and going \
             round the loop re-enters `query_step`, which acquires the base a second time \
             for one logical read (A10-04): {arm}"
        );
        assert!(
            !arm.contains("self.base()") && !arm.contains("key_update_count"),
            "the joined path must not consult the base. `base_rows` travels with the \
             answer precisely so that it does not have to: {arm}"
        );

        // **And the behaviour, not only the shape of the source.** A key with no history
        // must produce no row whether the reader folded it or joined someone who did.
        use nilestream_core::rev::Joined;
        let columns = vec!["c0".to_string(), "c1".to_string(), "anchor".to_string()];
        let missing = RevEngine::rows_from_answer(
            columns.clone(),
            4242,
            0,
            false,
            Joined {
                answer: nilestream_core::rev::Anchored {
                    value: 0,
                    anchor: 7,
                },
                base_rows: 0,
            },
            7,
        );
        assert!(
            missing.text().is_empty(),
            "an account with no history in this prefix has no balance, and that is not a \
             balance of zero. A joined reader learns the difference from `base_rows`; \
             without it, every unknown account would come back as a row containing 0."
        );
        let present = RevEngine::rows_from_answer(
            columns,
            4242,
            0,
            false,
            Joined {
                answer: nilestream_core::rev::Anchored {
                    value: 0,
                    anchor: 7,
                },
                base_rows: 3,
            },
            7,
        );
        assert_eq!(
            present.text().len(),
            1,
            "an account whose history sums to zero DOES have a balance, and it is zero. \
             The two cases differ only in `base_rows`, which is why it is published."
        );
    }

    /// **A reader that joins a flight waits with no lock held.**
    ///
    /// The two-phase read's whole benefit is that the reconstruction happens outside the
    /// view; a waiter parked inside `answer_from_view` would be parked under the base guard,
    /// which is worse than the hold it replaces. The wait is therefore the caller's: the
    /// read *returns* a ticket, and the base guard is dropped by that return.
    #[test]
    fn a_joined_reader_waits_outside_the_read_and_holds_nothing() {
        let src = include_str!("rev_engine.rs");
        let body = src
            .split("fn answer_from_view(")
            .nth(1)
            .expect("the function");
        let body = &body[..body.find("\n    }").unwrap_or(body.len())];
        assert!(
            !body.contains(".wait()"),
            "`answer_from_view` waits on a flight inside itself. It holds the base for its \
             whole body, so a thread parked there blocks every append in the process behind \
             another reader's fold. Return `ViewAnswer::Wait` and let the caller wait."
        );
        assert!(
            body.contains("return ViewAnswer::Wait(WaitPlan {"),
            "`answer_from_view` must hand the join out rather than resolving it, or the \
             `Join` case has nowhere to go but a second fold of the same prefix. It hands \
             out a `WaitPlan`: a bare ticket sends the joined reader back through this \
             function, which re-acquires the base for the same logical read (A10-04)."
        );
        // And the wait itself holds nothing. Two implementations of `Serving` reach it and
        // both must be clean, so both are checked: `resolve_join`, which the engine's own
        // loop calls, and the `RwLock` wrapper, which must drop its outer guard before
        // waiting (A10-04).
        let resolver = src
            .split("\n    fn resolve_join(")
            .nth(1)
            .expect("`resolve_join` exists");
        let resolver = &resolver[..resolver.find("\n    }").unwrap_or(resolver.len())];
        assert!(
            resolver.contains("ticket.wait()"),
            "`resolve_join` must be where the wait happens"
        );
        for forbidden in ["Timed::acquire", "self.base()", ".read()", ".write()"] {
            assert!(
                !resolver.contains(forbidden),
                "`resolve_join` contains `{forbidden}`: the wait is only cheap because it \
                 is taken holding nothing"
            );
        }

        // The wrapper's arm: its guard's scope must close before `ticket.wait()`.
        let wrapper = src
            .split("impl crate::session::Serving for std::sync::RwLock<RevEngine> {")
            .nth(1)
            .expect("the RwLock implementation");
        let arm = wrapper
            .split("QueryStep::Wait(plan) => {")
            .nth(1)
            .expect("the wrapper's wait arm");
        let arm = &arm[..arm.find("\n                }").unwrap_or(arm.len())];
        let wait_at = arm
            .find("ticket.wait()")
            .expect("the wrapper waits on the ticket");
        assert!(
            arm[..wait_at].find(".read()").is_none(),
            "the `RwLock` wrapper takes its own lock and then waits. O would be held shared \
             for the length of another reader's fold, and a `reseed` queued behind it stops \
             every later reader — `std::sync::RwLock` is writer-preferring on both hosts \
             this project measures."
        );
    }

    /// **The install is a second, separate hold — and the fold is between them.**
    ///
    /// The finding this task repairs is not that the view lock existed; it is that
    /// `Rev::read` folded the base *inside* it. A single `begin_read`/`finish_fold` pair with
    /// no `drop` between them would restore that exactly, and pass every behavioural test.
    #[test]
    fn the_reconstruction_happens_with_the_view_released() {
        let src = include_str!("rev_engine.rs");
        let body = src
            .split("fn answer_from_view(")
            .nth(1)
            .expect("the function");
        let body = &body[..body.find("\n    }").unwrap_or(body.len())];
        // **Code only, for the reason the scan above gives**: this guard reads *positions*,
        // and a comment naming one of these calls moves the position it reads. The function
        // it scans now carries a paragraph about where the base comes from, which is exactly
        // the shape that defeated the A10-01 guard.
        let body = code_only(body);
        let body = body.as_str();
        let begin = body.find("begin_read(").expect("phase one");
        let dropped = body.find("drop(rt);").expect("the view must be released");
        let fold = body.find("base.reconstruct(").expect("the fold");
        // `finish_fold` or `finish_fold_merging`: the install is one call under two names —
        // the second passes the base so a late landing can merge instead of pinning — and
        // this guard is about *when* it happens, not which of them is used. Matched without
        // the open parenthesis so a rename of the call cannot make the guard vacuous by
        // making it panic; with comments stripped, prose cannot satisfy it either.
        let finish = body.find("finish_fold").expect("phase two");
        assert!(
            begin < dropped && dropped < fold && fold < finish,
            "the keyed read must decide under the view (at {begin}), release it (at \
             {dropped}), fold (at {fold}) and only then take it again to install (at \
             {finish}). Folding under the hold is the 12-13 ms read maximum this task \
             exists to remove, and no answer is ever wrong on the way into it."
        );
        assert!(
            !body.contains("view.read(&*base"),
            "`Rev::read` folds inside the view lock by construction. The served path must \
             use the two-phase API."
        );
    }

    /// **The interleaving itself, under a deadline.**
    ///
    /// Readers whose budget guarantees misses — so the view's read path reaches the base —
    /// against a writer appending continuously. On the inverted order this hangs; here it
    /// finishes. The deadline is what turns a hang into a failure, because a test that hangs
    /// is a test nobody can read the output of.
    #[test]
    fn a_keyed_read_and_a_concurrent_append_do_not_deadlock() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::{mpsc, Arc};

        // A budget far below the key count, so nearly every keyed read misses and
        // reconstructs — which is the path that reaches the base from under the view.
        let engine = Arc::new(RevEngine::seeded(
            256,
            2,
            4,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        ));
        let done = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();

        let mut threads = Vec::new();
        for t in 0..4u64 {
            let e = Arc::clone(&engine);
            let tx = tx.clone();
            threads.push(std::thread::spawn(move || {
                let lowered = tests::compile(
                    "select acct, sum(amt) from postings where acct = 7 group by acct",
                );
                for i in 0..400u64 {
                    let anchor = e.frontier();
                    let _ = e.query(&lowered.circuit, "__wire_result", anchor);
                    let _ = (t, i);
                }
                let _ = tx.send(());
            }));
        }
        {
            let e = Arc::clone(&engine);
            let done = Arc::clone(&done);
            let tx = tx.clone();
            threads.push(std::thread::spawn(move || {
                let mut n = 0u64;
                while !done.load(Ordering::Relaxed) && n < 2_000 {
                    n += 1;
                    let rows = vec![
                        proto_engine::Row::Post(proto_engine::Posting {
                            txn: 900_000 + n,
                            acct: 7,
                            cur: 0,
                            amt: 5,
                            valid: 0,
                        }),
                        proto_engine::Row::Post(proto_engine::Posting {
                            txn: 900_000 + n,
                            acct: 8,
                            cur: 0,
                            amt: -5,
                            valid: 0,
                        }),
                    ];
                    let _ = e.append(rows, &format!("deadlock-probe-{n}"));
                }
                let _ = tx.send(());
            }));
        }
        drop(tx);

        // Five threads must each report in. Thirty seconds is far more than the work needs
        // and far less than a hung suite costs.
        let deadline = std::time::Duration::from_secs(30);
        for i in 0..5 {
            if rx.recv_timeout(deadline).is_err() {
                done.store(true, Ordering::Relaxed);
                panic!(
                    "only {i} of 5 threads finished within {deadline:?}. A keyed read and a \
                     concurrent append are deadlocked: one holds the view and wants the \
                     base, the other holds the base and wants the view."
                );
            }
        }
        done.store(true, Ordering::Relaxed);
        // **A panic inside one of these threads is a result, not noise.** `let _ = t.join()`
        // discarded it: a reader that panicked mid-run left the deadline satisfied — it had,
        // after all, finished — and the test passed while one of its five participants died.
        for (i, t) in threads.into_iter().enumerate() {
            t.join()
                .unwrap_or_else(|_| panic!("thread {i} panicked during the deadlock probe"));
        }
    }
}

#[cfg(test)]
mod visibility_tests {
    //! **Durable before visible, now that the two can differ.**
    //!
    //! Before T-05 the barrier happened inside `append`, inside the engine's lock, so an
    //! epoch could not be observed between existing and being durable — no reader could run
    //! at all. That made the guarantee true by exclusion, and moving the barrier out of the
    //! lock removes the exclusion. What replaces it is the split the ledger's own `Frontier`
    //! has always modelled: rows are applied to the base under the lock, in lock order, and
    //! become *visible* only when their record has been fsynced.
    //!
    //! These tests are the difference between that being a design and being a property.

    use super::*;
    use crate::session::Serving;

    fn engine_at(name: &str) -> (RevEngine, std::path::PathBuf) {
        let dir = std::env::temp_dir().join("nilestream-visibility-tests");
        let _ = std::fs::create_dir_all(&dir);
        let seg = dir.join(format!("{name}.seg"));
        let _ = std::fs::remove_file(&seg);
        let e = RevEngine::seeded(
            8,
            1,
            4,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        )
        .with_durable(&seg)
        .expect("durable sink");
        (e, seg)
    }

    fn post(txn: u64, acct: u64) -> Vec<proto_engine::Row> {
        vec![proto_engine::Row::Post(proto_engine::Posting {
            txn,
            acct,
            cur: 0,
            amt: 0,
            valid: 0,
        })]
    }

    /// **The gap the old design could not have: applied, and not yet visible.**
    ///
    /// This is the whole of what T-05 changed, asserted directly. Between `append` returning
    /// and the token being waited on, the rows are in the base and the frontier has not moved
    /// — so a read anchored at the frontier cannot see them, which is what stops a client
    /// observing a transaction a crash could still erase.
    #[test]
    fn an_applied_epoch_is_not_visible_until_its_barrier_returns() {
        let (e, seg) = engine_at("not-visible-yet");
        let before = e.frontier();

        let (epoch, receipt) = e.append_pending(post(1, 1), "v-1").expect("applies");
        assert!(
            epoch > before,
            "the base advanced: epoch {epoch} follows {before}"
        );
        assert_eq!(
            e.frontier(),
            before,
            "and the frontier did NOT: an epoch whose barrier has not returned is sealed, \
             not visible. If this reads {epoch}, durable-before-visible has been lost and a \
             client can observe a transaction a crash would erase."
        );

        receipt
            .expect("a durable engine returns a receipt")
            .wait()
            .expect("durable");
        assert_eq!(
            e.frontier(),
            epoch,
            "and only now, after the barrier, is it visible"
        );
        let _ = std::fs::remove_file(&seg);
    }

    /// **A receipt belongs to the connection that made the append — A9-F01, C9-02.1.**
    ///
    /// The theft window, reproduced and then closed. Connection A appends and has not yet
    /// waited; connection B handles a read and drains what it is entitled to. Under the old
    /// design both drained one engine-wide vector, so B took A's token: B then waited on a
    /// barrier it had not caused (and could be told 58030 for a write it never made), while A
    /// found the list empty and wrote `INSERT 0 2` with its own record still in flight.
    ///
    /// Nothing about timing is needed to show it. The property is that the receipt is
    /// *reachable* only from the session that made the append, and a session is one
    /// connection's.
    #[test]
    fn a_receipt_is_reachable_only_from_the_session_that_appended() {
        let (e, seg) = engine_at("receipt-ownership");
        let schema = crate::daemon::DEFAULT_SCHEMA.to_string();
        let mut a = crate::session::Session::new("a".into(), "bank".into(), schema.clone());
        let mut b = crate::session::Session::new("b".into(), "bank".into(), schema);

        let replies = a.handle(
            crate::pg_wire::Frontend::Query(
                "insert into postings values (910001, 1, 0, -5), (910001, 2, 0, 5)".into(),
            ),
            &e,
        );
        assert!(
            replies.iter().any(|m| matches!(
                m,
                crate::pg_wire::Backend::CommandComplete(t) if t.starts_with("INSERT")
            )),
            "the append must have been accepted for this test to mean anything: {replies:?}"
        );

        // B runs *while A's barrier is outstanding* — the whole of the window.
        let _ = b.handle(
            crate::pg_wire::Frontend::Query(
                "select acct, sum(amt) from postings where acct = 1 group by acct".into(),
            ),
            &e,
        );
        let stolen = b.take_receipts();
        assert!(
            stolen.is_empty(),
            "a connection that appended nothing took {} outstanding barrier(s); under the \
             engine-wide list this is where another connection's receipt went",
            stolen.len()
        );

        let mine = a.take_receipts();
        assert_eq!(
            mine.len(),
            1,
            "the appending connection must still hold its own receipt after another \
             connection has served a whole statement"
        );
        for p in mine {
            p.wait().expect("durable");
        }
        let _ = std::fs::remove_file(&seg);
    }

    /// **A barrier that never returns must not move the frontier.**
    ///
    /// The failure injected between apply and publish. Dropping the token without waiting is
    /// what a crashed or shut-down sealer looks like to the caller: the rows are in the base,
    /// and the epoch must stay invisible forever rather than being published by anything
    /// other than a completed fsync.
    #[test]
    fn a_barrier_that_is_never_awaited_never_publishes_its_epoch() {
        let (e, seg) = engine_at("never-awaited");
        let before = e.frontier();
        let (epoch, receipt) = e.append_pending(post(2, 2), "v-2").expect("applies");

        // The injection: drop this append's own receipt unread.
        assert!(
            receipt.is_some(),
            "a durable engine hands the append its receipt; there is nothing else to drop"
        );
        drop(receipt);

        assert_eq!(
            e.frontier(),
            before,
            "no wait, no publication — the frontier cannot advance to {epoch} without a \
             barrier having returned"
        );
        let _ = std::fs::remove_file(&seg);
    }

    /// **Every epoch at or below the visible frontier is in the base.**
    ///
    /// The other half of the ordering, and the one a naive publication scheme gets wrong.
    /// Rows are applied under the lock before their token is created, so the frontier can
    /// never name an epoch whose rows are missing — there is no window in which a read at the
    /// published frontier could find a gap.
    #[test]
    fn the_visible_frontier_never_names_an_epoch_the_base_has_not_applied() {
        let (e, seg) = engine_at("no-gap");
        // Each append waits on its own receipt before the next, as `serve` does per statement.
        for i in 0..8u64 {
            e.append_durable(post(100 + i, i % 4), &format!("g-{i}"))
                .expect("applies");
        }
        let visible = e.frontier();
        assert!(
            visible <= e.head(),
            "the visible frontier {visible} is beyond the base's head {} — it is naming an \
             epoch whose rows are not applied",
            e.head()
        );
        assert_eq!(
            visible,
            e.head(),
            "with every barrier returned, the two must agree"
        );
        let _ = std::fs::remove_file(&seg);
    }

    /// **One window, two structures, and a named retry across its edge — T00a.5, F-10-12.**
    ///
    /// The idempotency window is enforced twice: `proto_engine::Ledger` refuses a duplicate on
    /// admission, and the sequencer behind the durable sink refuses one on the way to disk.
    /// They are separate structures, and until C9-02 they counted different things under one
    /// declaration — the base kept the last W *transactions*, the sink kept the last W
    /// *batches*, and a batch is up to 4,096 transactions. A retry landing in that gap was
    /// **new to admission and duplicate to the sink**: applied to the base, an epoch sealed,
    /// and then unacknowledgeable. That is an applied transaction the client is never told
    /// about, which is the worst shape a durability bug takes.
    ///
    /// **The target here is `accept`, not `refuse`.** The obvious-looking repair is to make
    /// both sides refuse the retry, and it is wrong: an identity that has aged out of a
    /// declared window is one the system has *promised to forget*, and a retry of it is a new
    /// transaction that must commit. Making both refuse would turn a lost acknowledgement
    /// into a silently dropped payment, which is the same class of error with the sign
    /// reversed. So this asserts the two agree in both directions:
    ///
    ///   * an identity **inside** the window is refused, by both, consistently;
    ///   * an identity that has **aged out** is accepted, by both, and lands durably.
    ///
    /// And nothing is acknowledged before it is durable on either path: the epoch the
    /// accepted retry gets is asserted to come back from the segment on a fresh open.
    #[test]
    fn a_retry_after_the_window_expires_is_accepted_by_admission_and_by_the_sealer() {
        const W: u64 = 3;
        let dir = std::env::temp_dir().join("nilestream-visibility-tests");
        let _ = std::fs::create_dir_all(&dir);
        let seg = dir.join("idem-window-pair.seg");
        let _ = std::fs::remove_file(&seg);
        let e = RevEngine::seeded(
            8,
            1,
            4,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        )
        .with_idem_window(Some(W))
        .with_durable_bounded(&seg, Some(W))
        .expect("durable sink");

        // W + 1 named transactions. The first is then exactly one transaction past the edge.
        let mut epochs = Vec::new();
        for i in 0..=W {
            let epoch = e
                .append_durable(post(300 + i, i), &format!("win-{i}"))
                .unwrap_or_else(|err| panic!("`win-{i}` must commit: {err:?}"));
            epochs.push(epoch);
        }
        assert_eq!(
            e.idem_window(),
            (W as usize, Some(W)),
            "PRECONDITION UNMET: the base is not holding exactly the declared window, so \
             neither half of this test is about the edge it names"
        );

        // **Inside the window: refused, and by both.** `win-{W}` was the last one committed.
        let inside = format!("win-{W}");
        let err = e
            .append_durable(post(999, 1), &inside)
            .expect_err("an identity inside the window is a duplicate");
        assert!(
            matches!(err, crate::session::ServeError::Duplicate(_)),
            "an in-window retry must be refused as a duplicate and not as something else: \
             {err:?}"
        );

        // **Aged out: accepted, and by both.** `win-0` fell off the front when `win-{W}`
        // was committed. Admission no longer knows it, and the sealer must not either — if
        // the sealer still held it, this call would apply to the base and then fail on the
        // way to disk, which is the gap this test exists for.
        let head_before = e.head();
        let epoch = e
            .append_durable(post(1_000, 2), "win-0")
            .unwrap_or_else(|err| {
                panic!(
                    "`win-0` has aged out of the declared window of {W} transactions, so it \
                     is a new transaction and must commit. It was refused with {err:?}. A \
                     both-refuse outcome for an expired identity is not the repair — it \
                     turns a lost acknowledgement into a dropped payment."
                )
            });
        assert!(
            epoch > head_before,
            "the accepted retry must seal a new epoch, not return the original one: {epoch} \
             against a head of {head_before}"
        );

        // **Acknowledged only if durable**, checked in the segment's own coordinates.
        //
        // The epoch `append_durable` returns is a *base* epoch and the segment counts
        // *records*; the first draft of this compared the two and failed against a correct
        // build, because `seeded` starts the base above zero. What the segment can be asked
        // is which identities are in it and in what order, and that is the question anyway.
        let frontier = crate::session::Serving::frontier(&e);
        assert!(
            frontier >= epoch,
            "the frontier {frontier} is behind the epoch {epoch} that was acknowledged, so \
             something was acknowledged before it was visible"
        );
        drop(e);
        let recovery = nilestream_ledger::segment::recover(&seg).expect("the segment decodes");
        let seen = nilestream_ledger::sequencer::Sequencer::recover_seen_checked(&recovery)
            .expect("the segment's identities decode");
        let mut keys: Vec<&str> = seen.keys().map(|k| k.as_str()).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["win-0", "win-1", "win-2", "win-3"],
            "every identity this test committed must be in the segment, and no others"
        );
        // **The retry is a later record than the original**, which is what says it was
        // written durably as a new commit rather than answered from the sink's memory.
        // `recover_seen_checked` keeps the last epoch it saw for a key, so `win-0`'s must now
        // be past `win-{W}`'s — the retry came after every original.
        let win0 = seen["win-0"];
        let last_original = seen[&format!("win-{W}")];
        assert!(
            win0 > last_original,
            "`win-0` is recorded at {win0} and the last original at {last_original}: the \
             accepted retry did not reach the segment as a new record, so it was \
             acknowledged without being durable"
        );
        let _ = std::fs::remove_file(&seg);
    }

    /// **Publication is monotone even when barriers return for a batch at once.**
    ///
    /// `Pending::wait` publishes with `fetch_max`, and the argument that this is safe is that
    /// epochs are assigned under the engine's lock and submitted in the same order, so a
    /// later epoch's barrier cannot return before an earlier one's. Waiting out of order
    /// here is the adversarial case: even then the frontier must never go backwards, and must
    /// never exceed the base.
    #[test]
    fn waiting_out_of_order_cannot_move_the_frontier_backwards() {
        let (e, seg) = engine_at("monotone");
        let mut pending = Vec::new();
        for i in 0..6u64 {
            let (_epoch, receipt) = e
                .append_pending(post(200 + i, i), &format!("m-{i}"))
                .expect("applies");
            pending.extend(receipt);
        }
        pending.reverse(); // newest first: the order the argument does not rely on
        let mut seen = e.frontier();
        for p in pending {
            p.wait().expect("durable");
            let now = e.frontier();
            assert!(
                now >= seen,
                "the frontier went backwards, {seen} then {now}"
            );
            assert!(
                now <= e.head(),
                "the frontier {now} passed the base's head {}",
                e.head()
            );
            seen = now;
        }
        assert_eq!(seen, e.head());
        let _ = std::fs::remove_file(&seg);
    }
}

#[cfg(test)]
mod recovery_tests {
    //! **Durable means the rows come back.**
    //!
    //! Before T-01 the durable record carried `epoch.to_string()` — the epoch *number*. An
    //! audit ran eight writers against `nilestreamd --durable`, killed it with `SIGKILL` after
    //! two seconds, and restarted it on the same segment: 25,416 acknowledged inserts were
    //! gone, the frontier was back at the seed, every writer's account had no balance, and a
    //! retry of an acknowledged transaction was accepted as new. Every `durable` row this
    //! project has ever published measured the cost of a barrier on a record that could not
    //! reconstruct the ledger.
    //!
    //! The segment layer underneath was never the problem: it recovers payloads correctly and
    //! is tested for it. The server handed it a number.
    //!
    //! These tests are the in-process form of that protocol. `probes/crash.rs` in
    //! `docs/audit/cycle-7/probes/` is the out-of-process one, and the two agree.

    use super::*;
    use crate::session::Serving;

    fn seg(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("nilestream-recovery-tests");
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join(format!("{name}.seg"));
        let _ = std::fs::remove_file(&p);
        p
    }

    fn engine(seg: &std::path::Path) -> RevEngine {
        RevEngine::seeded(
            8,
            1,
            4,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        )
        .with_durable(seg)
        .expect("durable sink")
    }

    /// A conserved pair on `acct`, so the commit rule admits it.
    fn pair(txn: u64, acct: u64, amt: i128) -> Vec<proto_engine::Row> {
        vec![
            proto_engine::Row::Post(proto_engine::Posting {
                txn,
                acct,
                cur: 0,
                amt,
                valid: 0,
            }),
            proto_engine::Row::Post(proto_engine::Posting {
                txn,
                acct: 999,
                cur: 0,
                amt: -amt,
                valid: 0,
            }),
        ]
    }

    /// The account's balance now, seed included; the tests compare deltas across a reopen.
    fn balance(e: &RevEngine, acct: u64) -> i128 {
        let base = e.base();
        let head = base.head();
        base.postings_for(acct, head)
            .into_iter()
            .map(|p| p.amt)
            .sum()
    }

    /// **The whole of T-01, in one assertion.** Acknowledge, drop the process, reopen.
    #[test]
    fn every_acknowledged_row_is_there_after_a_reopen() {
        let path = seg("rows-come-back");
        let mut acked = 0i128;
        let seeded;
        {
            let e = engine(&path);
            seeded = balance(&e, 3);
            for i in 1..=40u64 {
                // The barrier, waited on exactly as `serve` waits on it: on this append's
                // own receipt, before the next statement.
                let epoch = e
                    .append_durable(pair(700_000 + i, 3, 7), &format!("t-{i}"))
                    .unwrap();
                acked += 7;
                assert!(epoch > 0);
            }
            assert_eq!(balance(&e, 3) - seeded, acked, "before the reopen");
        }

        let after = engine(&path);
        assert_eq!(
            balance(&after, 3) - seeded,
            acked,
            "after the reopen. Every one of these was acknowledged to a client; a `durable` \
             row that does not survive the process is a receipt, not a log."
        );
        assert!(
            after.frontier() >= 40,
            "the visible frontier must cover the recovered epochs, not restart at the seed"
        );
    }

    /// A retry of an acknowledged transaction is refused **by the daemon**, not accepted.
    #[test]
    fn a_retry_of_an_acknowledged_transaction_is_refused_after_a_reopen() {
        let path = seg("retry-refused");
        {
            let e = engine(&path);
            e.append(pair(700_001, 4, 5), "t-1").unwrap();
        }
        let after = engine(&path);
        let before = balance(&after, 4);
        let again = after.append_durable(pair(700_001, 4, 5), "t-1");
        assert!(
            matches!(again, Err(crate::session::ServeError::Duplicate(_))),
            "a retried transaction must be refused as a duplicate after a restart, not taken \
             again: {again:?}"
        );
        assert_eq!(balance(&after, 4), before, "and nothing moved");
    }

    /// **The chain is verified on replay, not rebuilt and trusted.**
    #[test]
    fn a_tampered_record_stops_recovery_rather_than_being_replayed() {
        let path = seg("tamper");
        let seeded;
        {
            let e = engine(&path);
            seeded = balance(&e, 5);
            for i in 1..=3u64 {
                e.append(pair(700_100 + i, 5, 11), &format!("t-{i}"))
                    .unwrap();
            }
        }
        // Flip one byte of the last record's row bytes. The segment's own CRC covers the
        // body, so this is caught there first — which is the outer of the two checks and
        // exactly as good. What matters is that recovery *stops* rather than replaying a
        // ledger nobody wrote.
        let mut bytes = std::fs::read(&path).unwrap();
        let n = bytes.len();
        bytes[n - 12] ^= 0xff;
        std::fs::write(&path, &bytes).unwrap();

        let after = RevEngine::seeded(
            8,
            1,
            4,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        )
        .with_durable(&path);
        match after {
            Err(e) => {
                let m = e.to_string();
                assert!(
                    m.contains("chain does not verify")
                        || m.contains("not a decodable")
                        || m.contains("was refused"),
                    "recovery must refuse, naming why: {m}"
                );
            }
            Ok(engine) => {
                // Truncated at the damaged record is also correct — the prefix before it is
                // whole, and that is what recovery is for. What is forbidden is replaying
                // past the damage.
                let replayed = balance(&engine, 5) - seeded;
                assert!(
                    replayed < 33,
                    "a damaged record must not be replayed as though it were intact: {replayed} \
                     of 33 came back, so all three records were taken"
                );
            }
        }
    }
}

#[cfg(test)]
mod chain_verification_tests {
    //! **A chain that is rebuilt and compared to nothing protects nothing.**
    //!
    //! The segment's CRC catches a *corrupt* record — a flipped byte, a torn write — and
    //! truncates there. That is the outer of two checks and it is why the crash tests pass
    //! whether or not the ledger's link is verified. It is not what the hash chain is for.
    //!
    //! The chain is for a record that is internally consistent and *wrong*: one whose CRC and
    //! whose own segment hash both check out, but whose rows are not the rows the epoch
    //! committed. That is the difference between a checksum and tamper-evidence, and until
    //! T-01 the daemon had only the first, because the two chains were hashes of two
    //! different things and nothing ever compared them.

    use super::*;
    use crate::session::Serving;

    fn seg(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("nilestream-chain-tests");
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join(format!("{name}.seg"));
        let _ = std::fs::remove_file(&p);
        p
    }

    fn engine(seg: &std::path::Path) -> std::io::Result<RevEngine> {
        RevEngine::seeded(
            8,
            1,
            4,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        )
        .with_durable(seg)
    }

    /// **A record that stops in the middle is refused, and the message names which one —
    /// T-14.3.**
    ///
    /// The forged-record case is `a_forged_record_is_refused_by_the_chain` below: a record
    /// whose CRC and segment hash are both valid and whose *rows* are wrong. This is the
    /// other failure a replay can meet — a record the decoder cannot finish reading, because
    /// the envelope claims a length the payload does not have.
    ///
    /// Two things are asserted, and the second is the one an operator needs. Recovery must
    /// **refuse**, rather than replay the prefix it could read: a suffix applied onto a
    /// prefix missing an epoch rebuilds a ledger nobody wrote. And the refusal must name the
    /// record, because "recovery failed" on a segment of a hundred thousand records is not
    /// something anyone can act on.
    #[test]
    fn a_record_that_cannot_be_decoded_is_refused_and_named() {
        let p = seg("truncated-record");
        {
            let e = engine(&p).expect("a durable engine");
            for i in 0..6u64 {
                e.append(
                    vec![
                        proto_engine::Row::Post(proto_engine::Posting {
                            txn: i,
                            acct: 1,
                            cur: 0,
                            amt: 100,
                            valid: 0,
                        }),
                        proto_engine::Row::Post(proto_engine::Posting {
                            txn: i,
                            acct: 2,
                            cur: 0,
                            amt: -100,
                            valid: 0,
                        }),
                    ],
                    &format!("t{i}"),
                )
                .expect("balanced");
            }
        }
        // Truncate the *payload* of the last record while leaving the framing intact: the
        // segment's own CRC is recomputed, so this is not a corrupt record — it is a
        // well-formed record carrying an envelope that ends early, which is the case the
        // batch decoder has to refuse rather than half-read.
        let mut recovered = nilestream_ledger::segment::recover(&p).expect("recoverable");
        // **One record is one batch and one batch is many transactions.** Six appends can
        // land in six records or in one, depending on whether the sealer drained them
        // together — which under a loaded test runner it does. The test must not care: it
        // truncates the last record, whatever the batching produced. An earlier version
        // asserted six records and failed one run in three for that reason, which is the
        // same class of load-dependent assertion T-13 found in the concurrent differential.
        assert!(
            !recovered.records.is_empty(),
            "at least one record must have been written"
        );
        let last = recovered.records.pop().expect("a last record");
        let mut payload = last.payload.clone();
        assert!(payload.len() > 12, "the envelope has a header to keep");
        payload.truncate(payload.len() - 4);
        let _ = std::fs::remove_file(&p);
        {
            // Re-sealed through `Segment::append`, exactly as the forgery test does: the
            // record's own hash and CRC are recomputed from the bytes given, so every check
            // the *storage* layer makes passes. What fails is the batch decoder, one layer
            // up, which is the layer this test is about.
            let (mut fresh, _) = nilestream_ledger::segment::Segment::open(
                &p,
                nilestream_ledger::segment::SyncPolicy::Always,
            )
            .expect("a fresh segment");
            for rec in recovered.records {
                fresh.append(rec.payload).expect("re-seal");
            }
            fresh.append(payload).expect("re-seal the short one");
            fresh.sync().expect("flush");
        }

        let err = match engine(&p) {
            Ok(_) => panic!("recovery must refuse a record it cannot decode"),
            Err(e) => e,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("segment record") && msg.contains("envelope"),
            "the refusal must name the record it stopped at and what was short, or an \
             operator has a segment and no place to look: {msg}"
        );
        let _ = std::fs::remove_file(&p);
    }

    /// Rebuild a sealed batch envelope with account 6's postings redirected to account 7,
    /// leaving every key, every amount and the framing itself intact. Returns `None` if the
    /// envelope does not parse or holds nothing to redirect, so a malformed record is skipped
    /// rather than silently counted as forged.
    fn redirect_postings_in_envelope(env: &[u8], altered: &mut usize) -> Option<Vec<u8>> {
        fn u32_at(b: &[u8], o: &mut usize) -> Option<usize> {
            let raw = b.get(*o..*o + 4)?;
            *o += 4;
            Some(u32::from_le_bytes(raw.try_into().ok()?) as usize)
        }

        let mut o = 0usize;
        let count = u32_at(env, &mut o)?;
        let mut out = Vec::with_capacity(env.len());
        out.extend_from_slice(&(count as u32).to_le_bytes());
        let mut touched = false;
        for _ in 0..count {
            let klen = u32_at(env, &mut o)?;
            let key = env.get(o..o + klen)?.to_vec();
            o += klen;
            let plen = u32_at(env, &mut o)?;
            let payload = env.get(o..o + plen)?;
            o += plen;

            // Inside one transaction's payload: parent(32) ‖ hash(32) ‖ encode_rows(rows).
            let forged = if payload.len() >= 64 {
                match proto_engine::ledger::decode_rows(&payload[64..]) {
                    Some(mut rows) => {
                        for row in &mut rows {
                            if let proto_engine::Row::Post(p) = row {
                                if p.acct == 6 {
                                    p.acct = 7;
                                    touched = true;
                                    *altered += 1;
                                }
                            }
                        }
                        let mut v = payload[..64].to_vec();
                        v.extend_from_slice(&proto_engine::ledger::encode_rows(&rows));
                        v
                    }
                    None => payload.to_vec(),
                }
            } else {
                payload.to_vec()
            };

            out.extend_from_slice(&(klen as u32).to_le_bytes());
            out.extend_from_slice(&key);
            out.extend_from_slice(&(forged.len() as u32).to_le_bytes());
            out.extend_from_slice(&forged);
        }
        touched.then_some(out)
    }

    /// Re-seal a segment record with altered rows, so every checksum the storage layer knows
    /// about is correct and only the ledger's link disagrees. This is the adversary the chain
    /// exists for, and it is trivial for anyone who can write the file.
    #[test]
    fn a_resealed_record_with_different_rows_is_refused_by_the_chain() {
        let path = seg("resealed");
        {
            let e = engine(&path).expect("durable sink");
            for i in 1..=3u64 {
                e.append(
                    vec![
                        proto_engine::Row::Post(proto_engine::Posting {
                            txn: 800_000 + i,
                            acct: 6,
                            cur: 0,
                            amt: 100,
                            valid: 0,
                        }),
                        proto_engine::Row::Post(proto_engine::Posting {
                            txn: 800_000 + i,
                            acct: 999,
                            cur: 0,
                            amt: -100,
                            valid: 0,
                        }),
                    ],
                    &format!("t-{i}"),
                )
                .unwrap();
            }
        }

        // **Redirect the credit leg to a different account, and leave every amount alone.**
        //
        // The choice of forgery matters, and the first one tried here was wrong: rewriting an
        // amount makes the transaction unbalanced, so `interpret` refuses it on the
        // double-entry invariant and the test passes with the chain check deleted. That is a
        // real refusal, but it is the *ledger's* refusal, and it proves nothing about the
        // link. Moving money between two accounts keeps the sum at zero, keeps both currencies
        // and both transaction ids, and is accepted by every check the ledger makes — which is
        // precisely the theft the hash chain is the only defence against.
        //
        // The rows are decoded and re-encoded rather than byte-patched, so the forgery is
        // exact: the record still carries the parent and hash the honest epoch committed, and
        // only the rows underneath them have moved.
        //
        // A segment record is not one transaction's payload — it is the *batch envelope* the
        // sequencer seals, `count | (key_len, key, payload_len, payload)*`, because one fsync
        // covers many transactions and recovery has to recover each one's idempotency key as
        // well as its rows. So the forgery unwraps the envelope, redirects the posting inside
        // each transaction's payload, and re-frames. Patching the record bytes directly was
        // the second wrong turn here: at offset 64 of the *envelope* there are no rows at all.
        let (mut recovered, _) = nilestream_ledger::segment::Segment::open(
            &path,
            nilestream_ledger::segment::SyncPolicy::Always,
        )
        .map(|(s, r)| (r, s))
        .expect("reopen to read records");
        let mut altered = 0usize;
        for rec in &mut recovered.records {
            let Some(forged) = redirect_postings_in_envelope(&rec.payload, &mut altered) else {
                continue;
            };
            rec.payload = forged;
        }
        assert!(
            altered > 0,
            "the probe must have found a posting to redirect"
        );
        let _ = std::fs::remove_file(&path);
        {
            let (mut fresh, _) = nilestream_ledger::segment::Segment::open(
                &path,
                nilestream_ledger::segment::SyncPolicy::Always,
            )
            .expect("a fresh segment");
            for rec in recovered.records {
                // `append` re-seals: it computes the record's own hash and CRC from the bytes
                // it is given, so the forged record is indistinguishable from an honest one
                // to every check the storage layer makes.
                fresh.append(rec.payload).expect("re-seal");
            }
            fresh.sync().expect("flush");
        }

        let reopened = engine(&path);
        let err = reopened
            .err()
            .expect("a re-sealed record with different rows must not open");
        let m = err.to_string();
        assert!(
            m.contains("chain does not verify"),
            "recovery must refuse on the ledger's chain, not on a checksum — the storage \
             layer's own checks all pass here, which is the point. Got: {m}"
        );
    }
}

#[cfg(test)]
mod anchor_after_barrier_tests {
    //! **A session may not observe an epoch whose barrier has not returned.**
    //!
    //! `append` returns the *applied* epoch. Raising the session's anchor to it meant that
    //! when the barrier then failed — and `serve` correctly answered `58030` rather than a
    //! commit tag — the session was still anchored past the visible frontier, and its next
    //! read found the rows in the base and served them. Read-your-own-failed-write, on the
    //! rung the thesis names read-your-writes.
    //!
    //! The window is small and the test does not need a storage fault to close it: between
    //! `append` returning and the barrier being waited on, the applied epoch and the visible
    //! frontier differ *by construction*, and that is exactly the interval in which the
    //! session must not have moved.

    use super::*;
    use crate::session::Serving;

    #[test]
    fn the_visible_frontier_lags_the_applied_epoch_until_the_barrier_returns() {
        let dir = std::env::temp_dir().join("nilestream-anchor-tests");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("lag.seg");
        let _ = std::fs::remove_file(&path);
        let e = RevEngine::seeded(
            8,
            1,
            4,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        )
        .with_durable(&path)
        .expect("durable sink");

        let (applied, receipt) = e
            .append_pending(
                vec![
                    proto_engine::Row::Post(proto_engine::Posting {
                        txn: 900_001,
                        acct: 2,
                        cur: 0,
                        amt: 50,
                        valid: 0,
                    }),
                    proto_engine::Row::Post(proto_engine::Posting {
                        txn: 900_001,
                        acct: 999,
                        cur: 0,
                        amt: -50,
                        valid: 0,
                    }),
                ],
                "anchor-lag",
            )
            .expect("applied");

        // The barrier has not been waited on. This is the interval the session's anchor must
        // not cross: `frontier()` is what `insert` now observes, and it is behind.
        assert!(
            e.frontier() < applied,
            "the visible frontier ({}) must lag the applied epoch ({applied}) until the \
             barrier returns — otherwise there is no interval in which a failed barrier can \
             be caught, and `insert` observing the applied epoch would be harmless only by \
             accident",
            e.frontier()
        );

        receipt
            .expect("a durable engine returns a receipt")
            .wait()
            .expect("durable");
        assert_eq!(
            e.frontier(),
            applied,
            "and it catches up exactly when the barrier returns"
        );
    }

    /// The source-level half: `insert` must not pass `append`'s epoch to `observe`.
    #[test]
    fn insert_observes_the_frontier_and_not_the_epoch_it_just_applied() {
        let src = include_str!("session.rs");
        let body = src
            .split("fn insert(&mut self, sql: &str, engine: &dyn Serving)")
            .nth(1)
            .expect("`insert` is where the anchor is raised");
        let body = &body[..body.find("\n    }").unwrap_or(body.len())];
        assert!(
            body.contains("self.observe(engine.frontier())"),
            "`insert` must raise the session's anchor from the visible frontier. Observing \
             the epoch `append` returned anchors the session past a barrier that may still \
             fail, and the next read then serves rows the client was told did not commit."
        );
    }
}

#[cfg(test)]
mod currency_premise_tests {
    //! **The F-41 witness, as a test.**
    //!
    //! Contribution 4 says a well-typed program cannot mismatch currencies. The compiler
    //! discharges that for programs, and every benchmark row is *data*: the wire accepted an
    //! `insert` in currency 999 against a schema declaring only `usd`, and then served
    //! 324 USD + 500 of currency 999 as **"824"**, the balance of account 1. A conservation
    //! rule quantified per currency is vacuous over a currency nobody declared, and a sum
    //! across currencies is not an imprecise number — it is a number that denotes nothing.
    //!
    //! The two halves are checked separately because they fail separately: the ingress check
    //! keeps an undeclared currency out of the base, and the fold refusal covers the case the
    //! ingress check cannot — a schema that declares two currencies, both legitimately
    //! present, summed by a query that names neither.

    use super::*;
    use crate::session::Serving;

    fn engine() -> RevEngine {
        RevEngine::seeded(20, 3, 8, ViewMode::Demand, EvictionPolicy::Lru)
    }

    fn transfer(txn: u64, cur: u32, amt: i128) -> Vec<Row> {
        vec![
            Row::Post(proto_engine::Posting {
                txn,
                acct: 1,
                cur,
                amt,
                valid: 0,
            }),
            Row::Post(proto_engine::Posting {
                txn,
                acct: 2,
                cur,
                amt: -amt,
                valid: 0,
            }),
        ]
    }

    /// The witness's second `select`: over a base holding two currencies, a `sum(amt)` that
    /// groups without `cur` is **refused**, where it used to answer 824.
    #[test]
    fn a_sum_without_the_currency_is_refused_over_a_multi_currency_base() {
        let e = engine();
        e.append(transfer(777_003, 1, 500), "w-1")
            .expect("balanced");
        let lowered =
            tests::compile("select acct, sum(amt) from postings where acct = 1 group by acct");
        let err = e
            .query(&lowered.circuit, "__wire_result", e.frontier())
            .expect_err("a cross-currency sum must not answer");
        assert!(
            matches!(err, crate::session::ServeError::CrossCurrency(_)),
            "and must be refused by name rather than by some generic failure: {err:?}"
        );
        assert_eq!(
            err.sqlstate(),
            "22000",
            "a data exception: the operands are not comparable. Not `23000` — nothing about \
             the stored data violates a constraint"
        );
        assert!(
            err.detail().contains("group by") && err.detail().contains("cur = k"),
            "the refusal must name both remedies, or it is a wall rather than an answer"
        );
    }

    /// The two spellings that *are* answerable stay answerable. A refusal that also refused
    /// these would have narrowed what a caller can ask rather than what the server will
    /// silently get wrong.
    #[test]
    fn grouping_by_the_currency_or_naming_one_still_answers() {
        let e = engine();
        e.append(transfer(777_003, 1, 500), "w-1")
            .expect("balanced");
        let anchor = e.frontier();

        let grouped = tests::compile(
            "select acct, cur, sum(amt) from postings where acct = 1 group by acct, cur",
        );
        let rows = e
            .query(&grouped.circuit, "__wire_result", anchor)
            .expect("grouping by the currency is well defined");
        assert_eq!(
            rows.len(),
            2,
            "both currencies must be reported, each as its own row"
        );

        let pinned = tests::compile(
            "select acct, sum(amt) from postings where acct = 1 and cur = 0 group by acct",
        );
        let rows = e
            .query(&pinned.circuit, "__wire_result", anchor)
            .expect("a query that names one currency sums within it");
        assert_eq!(rows.len(), 1);
    }

    /// One currency in the base is the ordinary case and must be untouched — including when
    /// the *schema* declares more than one. The check asks what the base holds.
    #[test]
    fn a_single_currency_base_is_unaffected() {
        let e = engine();
        e.append(transfer(777_005, 0, 700), "w-2")
            .expect("balanced");
        let lowered =
            tests::compile("select acct, sum(amt) from postings where acct = 1 group by acct");
        assert!(
            e.query(&lowered.circuit, "__wire_result", e.frontier())
                .is_ok(),
            "with one currency in the base there is nothing to mismatch, and refusing here \
             would refuse every ordinary balance read"
        );
    }

    /// `explain` must report the refusal. A path is a claim about what the next execution
    /// does, and "it will refuse" is as much a fact as "it will fold" — promising `fold` for
    /// a statement about to raise `22000` is the exact drift `ServePath` exists to prevent.
    #[test]
    fn explain_reports_the_refusal_rather_than_a_fold() {
        let e = engine();
        e.append(transfer(777_003, 1, 500), "w-1")
            .expect("balanced");
        let lowered =
            tests::compile("select acct, sum(amt) from postings where acct = 1 group by acct");
        assert_eq!(
            e.serve_path_at(&lowered.circuit, "__wire_result", e.frontier()),
            ServePath::Refused,
            "`explain` and the engine must describe the same engine"
        );
    }
}

#[cfg(test)]
mod fallback_rate_tests {
    //! **How often a keyed read consults the view and throws the answer away.**
    //!
    //! `answer_from_view` returns `None` when the entry it found is not exact at the anchor
    //! that was asked for, and folds instead. That is correct — the fold reconstructs at the
    //! requested anchor — and it costs roughly a hundred times a view read. Under concurrent
    //! writers an audit measured **87.4%** of keyed reads taking it, against 0.0% with readers
    //! alone, and no surface in the server could be asked, so the audit had to infer it from a
    //! latency distribution (`probes/mixed.rs`, `FOLD_FACTOR`). `select nilestream_stats`
    //! counts it directly now, and this is the test that holds it down.
    //!
    //! Four readers and two writers, because that is the shape the finding was measured in.
    //! The number is a ratio over keyed reads that reached the view, not over all reads: a
    //! query the view was never asked about is a different fact and belongs to `serve_path`.

    use super::*;
    use crate::session::Serving;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    const ACCOUNTS: i64 = 200;
    const READS_PER_READER: u64 = 1_500;

    /// Run four readers and two writers over one engine and return the delta in
    /// `ReadStats` across the run.
    fn mixed_full(readers: usize, writers: usize) -> (ReadStats, u64) {
        let engine = std::sync::Arc::new(RevEngine::seeded(
            ACCOUNTS,
            3,
            usize::MAX,
            ViewMode::Full,
            EvictionPolicy::Lru,
        ));
        let before = engine.read_stats();
        // **How far the writers moved the base while the readers ran.** Every keyed read
        // that has to reconstruct needs an epoch that landed between the view's applied
        // point and the read's anchor, so this is the term any honest bound on misses is
        // written against. Sampled before the threads start and after they join, which is
        // exactly the interval the reads happened in.
        let frontier_before = crate::session::Serving::frontier(&*engine);
        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let done = std::sync::Arc::new(AtomicU64::new(0));

        let mut threads = Vec::new();
        for w in 0..writers {
            let (e, stop) = (engine.clone(), stop.clone());
            threads.push(std::thread::spawn(move || {
                let mut n = 0u64;
                while !stop.load(Ordering::Relaxed) {
                    n += 1;
                    let txn = 900_000 + (w as u64) * 1_000_000 + n;
                    let acct = n % ACCOUNTS as u64;
                    let _ = e.append(
                        vec![
                            Row::Post(proto_engine::Posting {
                                txn,
                                acct,
                                cur: 0,
                                amt: 1,
                                valid: 0,
                            }),
                            Row::Post(proto_engine::Posting {
                                txn,
                                acct: 999_999,
                                cur: 0,
                                amt: -1,
                                valid: 0,
                            }),
                        ],
                        &format!("mix-{w}-{n}"),
                    );
                }
            }));
        }

        for r in 0..readers {
            let (e, done) = (engine.clone(), done.clone());
            threads.push(std::thread::spawn(move || {
                let lowered = tests::compile(
                    "select acct, sum(amt) from postings where acct = 7 group by acct",
                );
                for _ in 0..READS_PER_READER {
                    // The anchor a session would hold: sampled from the frontier, then used.
                    // The gap between the two is the whole of the window this measures, and
                    // it is a real window — a session observes the frontier when it writes or
                    // reads, and a concurrent writer moves it afterwards.
                    let anchor = e.frontier();
                    let _ = e.query(&lowered.circuit, "__wire_result", anchor);
                }
                done.fetch_add(r as u64, Ordering::Relaxed);
            }));
        }

        // Readers finish on their own count; the writers run until they do.
        let reader_handles = threads.split_off(writers);
        for t in reader_handles {
            t.join().expect("a reader panicked");
        }
        stop.store(true, Ordering::Relaxed);
        for t in threads {
            t.join().expect("a writer panicked");
        }

        let epochs_sealed =
            crate::session::Serving::frontier(&*engine).saturating_sub(frontier_before);
        let after = engine.read_stats();
        let delta = ReadStats {
            reads: after.reads - before.reads,
            hits: after.hits - before.hits,
            misses: after.misses - before.misses,
            rows_touched: after.rows_touched - before.rows_touched,
            resident: after.resident,
            view_answers: after.view_answers - before.view_answers,
            fallbacks: after.fallbacks - before.fallbacks,
            pending_joins: after.pending_joins - before.pending_joins,
            uninstalled_folds: after.uninstalled_folds - before.uninstalled_folds,
            pinned_installs: after.pinned_installs - before.pinned_installs,
            flights_refused: after.flights_refused - before.flights_refused,
            deferred_merges: after.deferred_merges - before.deferred_merges,
            merge_rows_visited: after.merge_rows_visited - before.merge_rows_visited,
            merge_epochs_merged: after.merge_epochs_merged - before.merge_epochs_merged,
            // Not a difference: the caps are a setting, not a counter, and subtracting them
            // would report zero for a run that had them the whole time.
            merge_max_epochs: after.merge_max_epochs,
            merge_max_rows: after.merge_max_rows,
            merges_refused_epochs: after.merges_refused_epochs - before.merges_refused_epochs,
            merges_refused_rows: after.merges_refused_rows - before.merges_refused_rows,
            merges_refused_unavailable: after.merges_refused_unavailable
                - before.merges_refused_unavailable,
            waiters_refused: after.waiters_refused - before.waiters_refused,
            joins_answered: after.joins_answered - before.joins_answered,
            joins_retried: after.joins_retried - before.joins_retried,
            gap_at_begin_total: after.gap_at_begin_total - before.gap_at_begin_total,
            gap_at_finish_total: after.gap_at_finish_total - before.gap_at_finish_total,
            gap_begin_samples: after.gap_begin_samples - before.gap_begin_samples,
            gap_finish_samples: after.gap_finish_samples - before.gap_finish_samples,
            flights_behind_at_begin: after.flights_behind_at_begin - before.flights_behind_at_begin,
            flights_that_fell_behind: after.flights_that_fell_behind
                - before.flights_that_fell_behind,
            // A maximum over the window is not a difference of two maxima — the larger of
            // the two could belong entirely to the run before this one. Reported as the
            // level it is, and read as "the largest gap this process has ever installed at".
            gap_at_finish_max: after.gap_at_finish_max,
            // Levels, not deltas: both are sizes of a structure, and the difference between
            // two sizes is not a size.
            view_metadata_keys: after.view_metadata_keys,
            idem_window_keys: after.idem_window_keys,
        };
        (delta, epochs_sealed)
    }

    fn mixed(readers: usize, writers: usize) -> (u64, u64) {
        let (s, _epochs) = mixed_full(readers, writers);
        (s.view_answers, s.fallbacks)
    }

    /// **A fallback traded for a reconstruction would be no win at all**, and the ratio alone
    /// cannot tell the two apart: a read that stops falling back because it now *misses* and
    /// folds the account's history through the anchor index has moved the cost, not removed
    /// it. So this reports the whole picture, and asserts on the part that would hide.
    #[test]
    fn the_reads_that_stopped_falling_back_are_served_and_not_reconstructed() {
        let (s, epochs) = mixed_full(4, 2);
        let keyed = s.view_answers + s.fallbacks;
        assert!(
            keyed > 1_000,
            "PRECONDITION UNMET: only {keyed} keyed reads reached the view"
        );
        eprintln!(
            "  4r/2w: reads={} hits={} misses={} rows_touched={} view_answers={} \
             fallbacks={} ({:.1}%) epochs_sealed={epochs}",
            s.reads,
            s.hits,
            s.misses,
            s.rows_touched,
            s.view_answers,
            s.fallbacks,
            s.fallbacks as f64 / keyed as f64 * 100.0
        );
        // **The bound the code actually promises, against a measured write count.**
        //
        // This asserted `hits * 4 > keyed * 3` — three quarters of keyed reads must hit —
        // which is not a property of the engine at all: it is a property of the *ratio of
        // reads to writes the scheduler produced*. Two writers that get a great deal of CPU
        // move the frontier past more anchors, every one of which is a legitimate
        // reconstruction, and the test fails for the engine behaving correctly under a
        // different load.
        //
        // What the engine promises is narrower and does not mention load: a keyed read
        // reconstructs only when an epoch landed between the view's applied point and the
        // read's anchor. Each such epoch can cost at most one miss per reader — a reader
        // that misses installs the value at its own anchor — so misses are bounded by
        // `readers × epochs_sealed`, and hits by the complement. With no writers the bound
        // collapses to "every keyed read hits", which is the control below.
        const READERS: u64 = 4;
        let allowed = READERS.saturating_mul(epochs);
        assert!(
            s.hits + allowed >= keyed,
            "keyed reads that did not hit exceed what the writers can explain: {} hits over \
             {keyed} keyed reads with {epochs} epochs sealed by 2 writers ({READERS} readers \
             × {epochs} epochs = {allowed} reconstructions accounted for). A miss beyond that \
             is a resident entry being rebuilt for no reason a write can justify.",
            s.hits
        );
    }

    fn rate(answers: u64, fallbacks: u64) -> f64 {
        let total = answers + fallbacks;
        if total == 0 {
            return 0.0;
        }
        fallbacks as f64 / total as f64
    }

    /// **The finding, as a bound.** 87.4% before; the entry is exact throughout its
    /// certification interval and the read now says so, which leaves only the keys a writer
    /// touched between the anchor and the read.
    #[test]
    fn the_fallback_rate_under_four_readers_and_two_writers_is_low() {
        let (answers, fallbacks) = mixed(4, 2);
        let r = rate(answers, fallbacks);
        assert!(
            answers + fallbacks > 1_000,
            "PRECONDITION UNMET: the measurement did not reach the view: {answers} answered, \
             {fallbacks} fell back"
        );
        // **Zero, not a rate.** This asserted `<= 5%`, a threshold chosen when the rate was
        // 87.4% and the repair had just landed. Since T-12.3 the number cannot be anything
        // but zero: `Rev::read` returns the anchor it was asked for on both the hit and the
        // miss path, so `answer_from_view`'s `answered.anchor != anchor` branch — the only
        // thing that increments this counter — is unreachable. A tolerance around an
        // unreachable event is a threshold nobody can breach and nobody can learn from; an
        // equality is a witness that the branch is still unreachable, and it fails the moment
        // a change puts a compensating fold back on the read path.
        assert_eq!(
            fallbacks,
            0,
            "a keyed read fell back to the fold {fallbacks} times of {} ({:.1}%). `Rev::read` \
             answers at the anchor it was asked for, so this branch is unreachable; reaching \
             it means a caller reintroduced the compensating fold the read exists to remove",
            answers + fallbacks,
            r * 100.0
        );
    }

    /// Readers alone were already at 0.0%, and must stay there: a change that lowered the
    /// mixed rate by loosening what counts as exact would show up here as nothing at all,
    /// so this is a control rather than a second measurement.
    #[test]
    fn readers_alone_never_fall_back() {
        let (s, epochs) = mixed_full(4, 0);
        let keyed = s.view_answers + s.fallbacks;
        assert!(
            s.view_answers > 1_000,
            "PRECONDITION UNMET: the readers did not reach the view"
        );
        assert_eq!(
            epochs, 0,
            "PRECONDITION UNMET: no writer was started, so the frontier must not have moved: \
             {epochs} epochs sealed"
        );
        assert_eq!(
            s.fallbacks, 0,
            "with no writer moving the frontier, every entry is exact at every anchor asked \
             for, and a fallback here would mean the read is refusing its own state"
        );
        // **The sharp half of the pair, and the one that is load-free.** The mixed test can
        // only bound its misses by what the writers did, because that is all the engine
        // promises under concurrency. With no writer there is nothing to explain a
        // reconstruction with: after the first read of the key, every later read of it must
        // be served from the resident entry. This is what "the reads that stopped falling
        // back are served and not reconstructed" was really trying to say, and here it can be
        // said exactly.
        assert!(
            s.misses <= 1,
            "with no writer, at most the first keyed read may reconstruct; {} of {keyed} \
             keyed reads rebuilt the value from the base, which means resident entries are \
             being discarded rather than served",
            s.misses
        );
    }
}
