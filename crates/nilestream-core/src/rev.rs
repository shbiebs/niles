//! The REV runtime: reconstructible epoch-anchored views, executing a typed IR circuit.
//!
//! This is where the thesis's central abstraction becomes a running object. A [`Rev`] is a
//! versioned, partially materialized derived state over an immutable, epoch-ordered base:
//! it materializes only what is read, evicts the rest under a budget, and reconstructs on
//! demand through an anchored upquery.
//!
//! # What closes the loop
//!
//! The circuit this runtime executes is the one `niles-lang` lowers from Niles source
//! text. Nothing is hand-built between the two. That is what makes the pipeline
//! *source → typed IR → running partial-state engine → measured result* a single artifact
//! rather than a diagram.
//!
//! # Scope, stated before any measurement is read
//!
//! This runtime executes the **key-aggregate fragment** of the IR: a `Source`, optional
//! `Filter` and `Map`, and one `Aggregate` keyed by a derivable group key. That is the
//! shape of a balance, a position, a rollup and a running total — which is to say, of the
//! views the thesis's measurements are about — but it is a fragment, and a circuit outside
//! it is rejected by [`Runtime::install`] rather than silently mis-executed. Joins,
//! fixpoints and ordering stages lower and verify but do not execute here.
//!
//! The counters are **counted work** — base rows read, deltas applied, resident
//! entry-epochs — not wall-clock. They are properties of the algorithm and the workload,
//! reproducible on any machine, which is what makes a phase diagram built from them
//! something a reader can check rather than something they must trust.

use crate::absence::{Epoch, Slot};
use niles_ir::circuit::{Circuit, NodeId};
use niles_ir::operator::{Agg, Op};
use niles_ir::{Consistency, Materialize, Retention};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

/// A view key. Concretely a small tuple of integers, which is what a `(acct, cur)` group
/// key lowers to.
pub type Key = Vec<i64>;

/// A materialized value. The fragment this runtime executes produces one aggregate per
/// key, in exact integer minor units — never floating point, because a balance that is
/// only approximately conserved is not conserved.
pub type Value = i128;

/// An answer, with the epoch it is true at. Every read returns one of these: an answer
/// without its anchor is not checkable, and "the same question, asked twice, answered
/// consistently" would be unstatable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anchored {
    pub value: Value,
    pub anchor: Epoch,
}

/// **The lattice's fourth state, and the reason it exists.**
///
/// `Slot::Pending(e)` has been in [`crate::absence`] since the lattice was written and no
/// code path could reach it, because reconstruction happened *inside* `read` while the
/// caller held the view: there was no moment at which a second reader could arrive and find
/// a flight to join, because no second reader could run at all. The daemon's tail was that
/// hold — one mutex serialising every keyed read behind the slowest fold on the machine —
/// and the lattice's own documentation described a mechanism the runtime did not have.
///
/// The repair is a two-phase read. [`Rev::begin_read`] decides, under the view lock, what
/// this caller must do; the caller then **releases the view** and folds; [`Rev::finish_fold`]
/// takes the view again and installs. What a second reader finds in between is a
/// `Pending(e)`, and what it does about it is this enum's three cases.
pub enum ReadOutcome {
    /// A resident entry certified across the requested anchor. Nothing else to do.
    Hit(Anchored),
    /// Fold the base and come back. The ticket carries an immutable `(key, anchor,
    /// generation)`; nothing about it can be changed by the time it returns, which is what
    /// makes a late completion identifiable rather than merely old.
    Fold(FoldTicket),
    /// Someone is already folding **this key at this exact anchor**. Wait for their answer.
    ///
    /// The join is on the *pair*, never on the key alone. A flight at a different anchor is
    /// computing a different number over a different prefix, and joining it would be the
    /// wrong-anchor answer the certification interval exists to make impossible; a reader
    /// that finds one folds independently and installs nothing.
    Join(WaitTicket),
}

/// **What a flight publishes**: the answer, and the evidence a joiner needs to shape it.
///
/// The answer alone is not enough, and the gap is not academic. A keyed read of an account
/// the base has never posted to must produce **no row**, not a row containing zero — the
/// absence lattice's distinction, at the layer where losing it is quietest. The owner learns
/// that from its own reconstruction, which reports how many base rows it read; a joiner that
/// received only the value would have to go back to the base to find out, which is a second
/// acquisition of B for one logical read and the thing T01.2 forbids (A10-04).
///
/// So the flight publishes the row count with the value, and the joiner needs no base at all.
#[derive(Debug, Clone, Copy)]
pub struct Joined {
    pub answer: Anchored,
    /// Base rows the owner's reconstruction read. **Zero means the key has no history in
    /// this prefix**, which is a different answer from a value of zero.
    pub base_rows: u64,
}

/// The published state of one flight. The only lock **below** the view.
#[derive(Debug, Default)]
struct CompletionState {
    answer: Option<Joined>,
    /// The owner went away without an answer — a cancelled statement, a dropped session, a
    /// panicking fold. Waiters wake and retry rather than waiting forever on a thread that
    /// is not coming back.
    cancelled: bool,
    waiters: u32,
}

/// **The rendezvous between the reader that folds and the readers that joined it.**
///
/// A waiter parked here holds no engine lock at all: `begin_read` returns *after* the view
/// guard is dropped, and the wait re-acquires nothing. That is what makes this lock a leaf.
/// The engine's documented order — O < B < P < V < C, S a leaf — gains one rung strictly
/// below the view: **V < F**, taken by `finish_fold` to publish and by a waiter to receive,
/// and never held while anything else is acquired.
///
/// Poisoning is recovered rather than propagated. The guarded state is an `Option`, a flag
/// and a counter; a panic in a fold leaves them consistent, and turning a reader's panic
/// into every subsequent reader's panic would convert one failed statement into a dead view.
#[derive(Debug, Default)]
pub struct Completion {
    state: Mutex<CompletionState>,
    woken: Condvar,
    /// **Whether anyone has ever joined this flight**, and therefore whether the mutex and
    /// the condvar below are needed at all.
    ///
    /// Set under the view lock, in the one place a completion is handed to a second reader.
    /// A flight nobody joined has no rendezvous to hold: its answer goes back through
    /// `finish_fold`'s return value, and its cancellation is read from `cancelled` below.
    ///
    /// This is not a micro-optimisation of a lock that was cheap. On a platform whose
    /// `Mutex` and `Condvar` are futexes the saving is two atomic operations; on macOS,
    /// where Rust's std lazily boxes a `pthread_mutex_t` and a `pthread_cond_t` on first
    /// use, it is **two heap allocations and 112 bytes per flight** — 64 for the mutex and
    /// 48 for the condition variable. E18's `rev_metadata_2x_budget` folds 5,000 uncontended
    /// keys and measured 44,165 allocations on Host C against 34,165 committed from Linux:
    /// exactly 10,000, exactly two per key (A10-19). The gate failure was a real cost, paid
    /// on every uncontended fold, that the committed budget could not see because the
    /// machine it was measured on does not charge for it.
    joined: AtomicBool,
    /// **The cancellation flag, outside the mutex.**
    ///
    /// `reap_cancelled` asks every miss whether this key's flight is dead, and a dropped
    /// ticket answers by setting it. Both were inside the mutex, so both took the lock —
    /// which on macOS meant the reaping path allocated the pthread objects that the publish
    /// path had just been taught not to. A flag read and written by a `Drop` and by a
    /// reader is exactly what an atomic is for.
    cancelled: AtomicBool,
    /// How many times [`Completion::lock`] has been called on this completion.
    locks: std::sync::atomic::AtomicU64,
}

/// **How many times any completion's mutex has been taken, process-wide.**
///
/// The cost this counts is not a duration. On macOS, Rust's std lazily boxes a
/// `pthread_mutex_t` and a `pthread_cond_t` the first time a `Mutex`/`Condvar` is used, so
/// *the first* acquisition of a completion's lock costs two heap allocations and 112 bytes
/// and every later one costs almost nothing — which makes a timing a bad instrument for it
/// and a count an exact one. A guard written against this counter fails identically on every
/// platform, including the ones where the saving is two atomics.
///
/// Public so the memory probe and the guards can read it; `Relaxed` because it is a
/// diagnostic total with no ordering obligations to anything.
pub static COMPLETION_LOCKS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Completion {
    fn lock(&self) -> MutexGuard<'_, CompletionState> {
        // **Counted, per completion.**
        //
        // The cost being counted is not a duration. On macOS, Rust's std lazily boxes a
        // `pthread_mutex_t` and a `pthread_cond_t` the first time a `Mutex`/`Condvar` is
        // used, so the *first* acquisition costs two heap allocations and 112 bytes and
        // every later one costs almost nothing — which makes a timing a bad instrument for
        // it and a count an exact one.
        //
        // Per completion rather than in a process-wide static: the first draft used a static
        // and its guard passed alone and failed under `cargo test`'s parallelism, because
        // another test's joined flight incremented the same counter between the two reads.
        // A counter a guard cannot isolate reports someone else's work.
        self.locks.fetch_add(1, Ordering::Relaxed);
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// How many times this completion's mutex has been taken. Zero for a flight nobody
    /// joined, which is the property the E18 budget turns on.
    fn locks(&self) -> u64 {
        self.locks.load(Ordering::Relaxed)
    }

    /// Called under the view lock when this completion is handed to a joining reader.
    ///
    /// Ordering: the join happens under **V**, and so does `finish_fold`. A joiner that
    /// arrives first sets this and only then waits; a `finish_fold` that runs first removes
    /// the flight under V, so no later reader can find it to join. There is no interleaving
    /// in which a waiter exists and `joined` is false.
    fn mark_joined(&self) {
        self.joined.store(true, Ordering::Release);
    }

    fn was_joined(&self) -> bool {
        self.joined.load(Ordering::Acquire)
    }

    fn publish(&self, answer: Joined) {
        // **Nobody is waiting, so there is nothing to publish to.** The owner's answer
        // reaches its own caller as `finish_fold`'s return value; the rendezvous exists only
        // for readers who joined, and there are none.
        if !self.was_joined() {
            return;
        }
        self.lock().answer = Some(answer);
        self.woken.notify_all();
    }

    fn cancel(&self) {
        // The flag first, and unconditionally: `reap_cancelled` reads it on every miss and
        // must see a dead flight whether or not anyone joined it.
        self.cancelled.store(true, Ordering::Release);
        if !self.was_joined() {
            return;
        }
        // A waiter parked on the condvar needs waking, and it reads `cancelled` from the
        // guarded state, so that copy is kept in step for the one case that reads it.
        self.lock().cancelled = true;
        self.woken.notify_all();
    }

    /// The owner gave up. The flight is reapable and no waiter will ever be answered by it.
    fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    /// Give back a place in the queue that was taken and will not be used.
    fn release_waiter(&self) {
        let mut s = self.lock();
        s.waiters = s.waiters.saturating_sub(1);
    }

    /// Block until the owner publishes or gives up. **Holds nothing but this leaf.**
    fn join(&self) -> Option<Joined> {
        let mut s = self.lock();
        while s.answer.is_none() && !s.cancelled {
            s = self.woken.wait(s).unwrap_or_else(|e| e.into_inner());
        }
        s.waiters = s.waiters.saturating_sub(1);
        s.answer
    }
}

/// A reconstruction this view has authorised and is waiting on.
struct InFlight {
    anchor: Epoch,
    generation: u64,
    done: Arc<Completion>,
    /// The slot this flight replaced with `Pending`, so a cancelled flight can put back
    /// exactly what was there. `None` when the slot was left alone — a `Present` entry stays
    /// resident and keeps serving its own certification interval while a read at an anchor
    /// outside that interval is reconstructed, because evicting a good answer to make room
    /// for a marker would be paying memory pressure for bookkeeping.
    prior: Option<Slot<Value>>,
}

/// Permission to fold, and the identity that permission was granted under.
///
/// Immutable by construction: `key`, `anchor` and `generation` are set once, in
/// `begin_read`, under the view lock. `finish_fold` installs only if the view still has a
/// flight for this key at this generation — so a completion that arrives after its flight
/// was superseded answers its own caller (its value is exact at its own anchor) and touches
/// no resident state.
pub struct FoldTicket {
    key: Key,
    anchor: Epoch,
    generation: u64,
    /// Whether this caller owns the key's flight. `false` for a fold that found a flight at
    /// a different anchor, or that was refused a flight under overload: an honest,
    /// uninstalled reconstruction, which is slower and correct, and never a second writer
    /// to a slot someone else owns.
    install: bool,
    done: Option<Arc<Completion>>,
    settled: bool,
    /// `applied − anchor` at authorisation, carried so `finish_fold` can separate the lag a
    /// read started with from the lag its own fold introduced.
    gap_begin: u64,
}

impl FoldTicket {
    /// The key to fold. Borrowed, not cloned: the fold does not need to own it.
    pub fn key(&self) -> &Key {
        &self.key
    }
    /// The anchor to fold at — the one the caller asked for, and the one its answer will
    /// carry.
    pub fn anchor(&self) -> Epoch {
        self.anchor
    }
    /// Whether a successful fold will be installed. Reported so a caller can count the
    /// uninstalled ones without inspecting the view.
    pub fn installs(&self) -> bool {
        self.install
    }
    /// `applied − anchor` when this flight was authorised.
    pub fn gap_begin(&self) -> u64 {
        self.gap_begin
    }
}

impl Drop for FoldTicket {
    /// **A fold that never returns must not strand the readers waiting on it.**
    ///
    /// A cancelled statement, a dropped connection or a panicking fold destroys the ticket
    /// without calling `finish_fold`. The waiters are woken with no answer and retry; the
    /// view's `in_flight` entry and its `Pending` marker are reaped by the next
    /// `begin_read` for that key, which is the first moment anything holds the view lock
    /// again. Cancellation cannot take the view here — `Drop` runs wherever the ticket
    /// died, which may be inside a section that already holds it — and a lock taken in a
    /// destructor is a deadlock waiting for the one caller who does that.
    fn drop(&mut self) {
        if !self.settled {
            if let Some(d) = &self.done {
                d.cancel();
            }
        }
    }
}

/// A reader that found a flight at its own exact anchor and is waiting for that answer.
pub struct WaitTicket {
    done: Arc<Completion>,
    anchor: Epoch,
    /// Whether [`WaitTicket::wait`] consumed this ticket. A ticket that was waited on has
    /// already released its place; one that was not is released by `Drop`.
    consumed: bool,
}

impl WaitTicket {
    /// Block until the flight publishes. **The caller must hold no engine lock**: this is
    /// the whole point of the two-phase read, and holding the view across it would restore
    /// the serialisation the split exists to remove.
    ///
    /// `None` means the owner went away, or published an answer at an anchor this waiter
    /// did not ask for — neither is an answer, and the caller goes around again.
    pub fn wait(mut self) -> Option<Joined> {
        self.consumed = true;
        self.done.join().filter(|j| j.answer.anchor == self.anchor)
    }
}

impl Drop for WaitTicket {
    /// **A place in the queue that is never taken must be given back — A10-05.**
    ///
    /// `begin_read` increments `waiters` under the completion's lock and hands out this
    /// ticket; `join` decrements it on the way out. A ticket that is *dropped* — a cancelled
    /// statement, a client that hung up, a caller that decided to fold instead — decremented
    /// nothing, so the flight's queue kept a reservation for a reader that was never coming.
    /// `MAX_WAITERS` of those and every later reader is refused a join it could have had,
    /// with the refusal counted against a queue that is empty.
    ///
    /// Taking the completion's lock in a destructor is safe here and nowhere else in this
    /// engine: **F is a leaf**, below the view and below everything else, and is never held
    /// while another lock is acquired. The `FoldTicket`'s destructor deliberately does *not*
    /// take the view for exactly the opposite reason.
    fn drop(&mut self) {
        if !self.consumed {
            self.done.release_waiter();
        }
    }
}

/// **Whether the caller of a read is in a position to wait for another reader's flight.**
///
/// Not a tuning knob: it is a statement about the caller's lock discipline. A caller holding
/// the view — `Rev::read` does, for its whole body — cannot wait on a flight whose owner must
/// take the view to publish, so it says `Alone` and folds its own (A10-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadMode {
    /// The caller holds nothing and may wait; a flight at this exact anchor may be joined.
    Shared,
    /// The caller cannot wait. A flight at this anchor is left alone and this read folds
    /// independently, installing nothing.
    Alone,
}

/// How many reconstructions one view will have outstanding at once.
///
/// A bound, not a tuning knob. Each flight costs a `Pending` marker, a map entry and a
/// condvar; an unbounded flight table is a per-connection allocation with no ceiling, which
/// is the shape of every memory term this cycle removed. Past the bound `begin_read`
/// **refuses the flight and folds anyway** — the caller still gets an exact answer, it is
/// simply not shared — and counts `flights_refused`, because an overload that shows up only
/// as latency is one nobody can be asked about.
const MAX_FLIGHTS: usize = 256;

/// How many readers may queue on one flight before the rest fold independently.
const MAX_WAITERS: u32 = 256;

/// What the runtime needs from a base. Deliberately narrow: three operations, all of them
/// reads of an immutable, epoch-ordered history.
///
/// Keeping this a trait rather than a concrete type is not abstraction for its own sake.
/// It is what lets the same runtime run over the in-memory research prototype that
/// produced Chapter 9's numbers and over a durable, segment-backed store, and be
/// *measured to produce the same counted work* over both. A runtime welded to one storage
/// layer could not make that comparison.
pub trait Base {
    /// The current visibility frontier.
    fn frontier(&self) -> Epoch;

    /// Fold the aggregate for `key` over the prefix ending at `anchor`.
    ///
    /// The implementation is expected to start from the newest checkpoint at or before
    /// `anchor` and fold only the suffix — which is what makes reconstruction cost bounded
    /// by the checkpoint interval rather than by history length (SC7).
    ///
    /// Returns the value **and** the number of base rows it had to read, because the row
    /// count is the measurement.
    fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64);

    /// The per-key deltas sealed in epoch `e`. Empty for most keys in most epochs, which
    /// is exactly why partial materialization can pay.
    fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)>;

    /// The earliest epoch whose per-key deltas this base can still produce.
    ///
    /// **Zero by default, because a fully retained base is what F4 promises**, and every
    /// base in this repository is one: the history is immutable and nothing is compacted
    /// away, so any epoch at or below the frontier can be re-read. A base that truncates or
    /// compacts a prefix must override this, and the reason it matters is
    /// [`Rev::finish_fold`]: a deferred merge folds the deltas in `(anchor, applied]`, and
    /// `deltas_at` reports an epoch it no longer holds as **empty**, which is
    /// indistinguishable from an epoch in which nothing happened. Merging across that
    /// boundary would silently drop real money. The merge refuses a suffix that begins below
    /// this epoch and pins instead.
    fn deltas_available_from(&self) -> Epoch {
        0
    }
}

/// The bounds a deferred merge is allowed to spend, fixed before anything was measured.
///
/// # Why these are constants and not a heuristic
///
/// A merge that adapts its own budget to what it finds is a merge whose cost is a function
/// of the workload, and the phase diagram this thesis is for would then be measuring the
/// heuristic rather than the mechanism. These are **preregistered**: written down with their
/// reasons, and a run that wants different ones states them in its transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeCaps {
    /// The largest suffix, in epochs, a landing may merge across.
    pub max_epochs: Epoch,
    /// The largest number of physical delta rows a single merge may visit.
    pub max_rows: u64,
}

impl MergeCaps {
    /// **Merging off.** No suffix is at most zero epochs long when the gap is at least one,
    /// so every late landing pins — which is the policy this cycle measures against, and the
    /// reason the ablation is a value rather than a `cfg`.
    pub const OFF: MergeCaps = MergeCaps {
        max_epochs: 0,
        max_rows: 0,
    };
}

impl Default for MergeCaps {
    /// **32 epochs and 4,096 rows.**
    ///
    /// The epoch cap comes from the cycle-10 baseline: the arrival gap on both hosts is
    /// 4.6–5.7 epochs with a maximum of 17 across 45,681 flights. 32 covers that
    /// distribution with roughly a factor of two in hand and still refuses a genuinely large
    /// gap — a reader whose anchor is hundreds of epochs old is asking a historical
    /// question, and pinning is the right answer to it.
    ///
    /// The row cap bounds the *physical* work independently, because epochs are not a
    /// constant amount of reading: an epoch's delta list is the whole epoch's rows across
    /// every key, and one epoch of a bulk load can be larger than thirty of a payment
    /// stream. 4,096 rows over at most 32 epochs is 128 rows per epoch, which is well above
    /// anything the mixed benchmark seals and well below the cost of the reconstruction the
    /// merge is trying to save.
    fn default() -> MergeCaps {
        MergeCaps {
            max_epochs: 32,
            max_rows: 4_096,
        }
    }
}

/// Why a late landing did not merge, when it did not.
///
/// Kept apart rather than summed into one "declined" counter: an over-budget suffix is a
/// tuning question, an unavailable one is a retention question, and a merge that is simply
/// switched off is neither. A refusal reported under the wrong cause is a refusal nobody can
/// act on — the lesson `waiters_refused` and `flights_refused` already carry (A10-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeRefusal {
    /// The suffix is longer than [`MergeCaps::max_epochs`].
    TooManyEpochs,
    /// Visiting the suffix would read more than [`MergeCaps::max_rows`] delta rows.
    TooManyRows,
    /// The suffix begins below [`Base::deltas_available_from`].
    SuffixUnavailable,
}

/// Eviction policy. `CostAware` is the adaptive one; the other two are the baselines it is
/// measured against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Lru,
    Random,
    /// Evict the entry whose reconstruction is cheapest relative to how rarely it is read.
    /// The measured margin over LRU is real but uneven — large on reconstruction work,
    /// small on aggregate delay — and the runtime records both so the claim stays honest.
    CostAware,
}

/// Counted work. Machine-independent, and the unit every claim in Chapter 9 is stated in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub reads: u64,
    pub hits: u64,
    pub misses: u64,
    pub upqueries: u64,
    /// Base rows the upqueries had to fold. The headline reconstruction cost.
    pub base_rows_read: u64,
    /// Deltas applied to resident entries. Where a consistency rung's cost actually falls.
    pub deltas_applied: u64,
    /// Deltas skipped because their entry was not resident — the saving partiality buys.
    pub deltas_skipped: u64,
    pub evictions: u64,
    /// Maintenance passes: the number of times the view was dragged to a frontier.
    ///
    /// Distinct from `deltas_applied`, and the distinction is the point. A bounded rung
    /// batches maintenance, so it does *fewer, larger* passes; it does not apply fewer
    /// deltas, because the deltas it batched over still have to be folded in. Counting
    /// only deltas made a lax rung look cheap by making it wrong.
    pub maintenance_passes: u64,
    pub peak_resident: u64,
    /// The integral of residency over time: sum over epochs of the resident entry count.
    ///
    /// This, and not peak residency, is the memory term in the cost model. An early
    /// version of this instrument charged `peak x epochs` and produced a phase diagram in
    /// which partial materialization won every cell — an artifact of the metric, not a
    /// result. The correction is recorded in the thesis because the mistake is an easy one
    /// and the corrected figure is the one the phase boundary depends on.
    pub resident_entry_epochs: u64,
    /// Readers that found a flight at their own exact anchor and shared its fold.
    ///
    /// **The counter that says the lattice's fourth state is reachable.** It was zero by
    /// construction until the read was split in two, and `MISMATCH-pending-unreachable`
    /// stood against the chapter that described joining as a mechanism the runtime had.
    pub pending_joins: u64,
    /// Folds whose result answered their caller and was not installed: a flight at another
    /// anchor already owned the key, a flight was refused under overload, or the completion
    /// arrived after its generation had been superseded.
    ///
    /// Not waste, and not an error. Each one is an exact answer at its own anchor; what it
    /// declines to do is write to a slot another anchor owns.
    pub uninstalled_folds: u64,
    /// Installs of a reconstruction anchored **below** the view's applied frontier, which
    /// enter pinned. A flight that overlapped an `advance` lands here, and the pin is what
    /// stops the entry inheriting a frontier it never saw the deltas for.
    pub pinned_installs: u64,
    /// Reads refused a flight because the view already had `MAX_FLIGHTS` outstanding. They
    /// folded anyway, uninstalled.
    pub flights_refused: u64,
    /// Deltas held back during a flight and folded into its result on landing.
    ///
    /// **Zero until the merge lands, and reported so it can be seen to be zero.** A counter
    /// that is absent from the wire and a counter that reads zero are different claims, and
    /// only one of them can be checked; this one was absent (A10-08).
    pub deferred_merges: u64,
    /// Delta rows a merge actually visited, and epochs it actually merged across.
    ///
    /// The cost side of the same event. `deferred_merges` alone says a merge happened and
    /// says nothing about what it cost, and a mechanism whose benefit is reported without
    /// its price is not a measurement.
    pub merge_rows_visited: u64,
    pub merge_epochs_merged: u64,
    /// Late landings that did **not** merge, by cause. An over-budget suffix is a tuning
    /// question, an unavailable one is a retention question, and merging switched off is
    /// neither; a refusal reported under the wrong cause is one nobody can act on.
    pub merges_refused_epochs: u64,
    pub merges_refused_rows: u64,
    pub merges_refused_unavailable: u64,
    /// Readers turned away from an existing flight because its waiter list was full. They
    /// folded alone, exactly. Counted apart from `flights_refused`, which is the *other*
    /// capacity: a refusal with one cause reported under another is a refusal nobody can act
    /// on (A10-05).
    pub waiters_refused: u64,
    /// Reads that could have joined a flight at their own anchor and did not, because the
    /// caller said it was in no position to wait. A third refusal reason, kept apart from
    /// the two capacity ones: this is lock discipline, not overload.
    pub joins_declined_by_caller: u64,
    /// **The two anchor gaps, kept apart.** `gap_at_begin_total` sums `applied − anchor` at
    /// the moment a flight is authorised: the read was already behind before it folded.
    /// `gap_at_finish_total` sums it again when the flight lands. Their difference is the
    /// only part the fold itself caused, and one number could not distinguish them — which
    /// is the measurement the merge decision turns on (A10-11).
    pub gap_at_begin_total: u64,
    pub gap_at_finish_total: u64,
    pub gap_at_finish_max: u64,
    /// **The denominators, because a total without its count is not a mean.**
    ///
    /// The first harness run that printed these divided both totals by `pending_joins`, the
    /// nearest counter to hand, and reported a mean gap of 386 epochs beside a maximum of
    /// 16 — a mean above the maximum, which is the arithmetic saying the divisor is wrong.
    /// The two totals are accumulated over different sets of events (every authorised
    /// flight; every *installing* one), so they need two counts and neither is any other
    /// counter in this struct.
    pub gap_begin_samples: u64,
    pub gap_finish_samples: u64,
    /// Flights that were already behind when they began, and flights that fell further
    /// behind while folding.
    pub flights_behind_at_begin: u64,
    pub flights_that_fell_behind: u64,
    /// **How many times this view's flights took their rendezvous lock.**
    ///
    /// A flight nobody joined has no rendezvous to take, and taking one costs two heap
    /// allocations and 112 bytes on macOS the first time — Rust's std boxes the
    /// `pthread_mutex_t` and the `pthread_cond_t` lazily. E18's `rev_metadata_2x_budget`
    /// folds 5,000 uncontended keys and measured 44,165 allocations on Host C against a
    /// 34,165 budget: exactly two per key (A10-19). This counter is what a guard asserts on,
    /// and it asserts identically on a platform where the same saving is two atomic
    /// operations and no allocation at all.
    pub completion_locks: u64,
}

impl Stats {
    pub fn hit_ratio(&self) -> f64 {
        if self.reads == 0 {
            0.0
        } else {
            self.hits as f64 / self.reads as f64
        }
    }
    /// Price the raw counters after the fact. Pricing at the end rather than during the run
    /// is what lets one measured run be re-scored under many cost models, which is how the
    /// phase diagram sweeps the price of memory without re-running anything.
    pub fn cost(&self, memory: f64, maintenance: f64, reconstruction: f64) -> f64 {
        self.resident_entry_epochs as f64 * memory
            + self.deltas_applied as f64 * maintenance
            + self.base_rows_read as f64 * reconstruction
    }
}

/// Per-key eviction-policy metadata: the read count `CostAware` ranks by, and the clock
/// `Lru` ranks by. Sixteen bytes beside the key, rather than two maps each holding their own
/// copy of it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Meta {
    reads: u64,
    last_read: u64,
}

/// One reconstructible epoch-anchored view.
pub struct Rev {
    pub node: NodeId,
    pub name: String,
    slots: BTreeMap<Key, Slot<Value>>,
    resident: u64,
    /// Resident-entry budget. `None` means full materialization.
    budget: Option<u64>,
    policy: Policy,
    /// The epoch through which deltas have been applied to this view as a whole.
    applied: Epoch,
    /// The next epoch this view has yet to see.
    ///
    /// Separate from `applied` because **epoch 0 is a real record**, and a single field
    /// cannot say whether `applied == 0` means "through epoch zero" or "nothing yet".
    /// `advance` computed its range as `applied + 1 ..= e`, so the very first call —
    /// `advance(base, 0)` — iterated `1..=0`, which is empty: the ledger's first transaction
    /// was never folded into any view. It was invisible while nothing was resident, because
    /// a demand view's first read reconstructs from the base and picks the epoch up on the
    /// way; it became a wrong answer the moment a view was asked to hold everything.
    next_to_apply: Epoch,
    /// Keys whose resident entry is **not** certified through `applied`, because it was
    /// installed at an anchor below the frontier the view had already applied. Such an
    /// entry has not seen the deltas between its own anchor and `applied`, so it may
    /// serve only reads at or below its own stamp, and a delta must not be folded into
    /// it — it is missing earlier ones, and adding a later one would compound the error
    /// rather than correct it.
    pinned: BTreeSet<Key>,
    /// Per-key policy metadata, **bounded by the budget and evicted with the entry**.
    ///
    /// One map, not two. It was `reads_of: BTreeMap<Key, u64>` beside
    /// `last_read: BTreeMap<Key, u64>`, which held two copies of every key — and a `Key` is
    /// a heap-allocated `Vec<i64>`, so the second copy was most of the second map. Merging
    /// them took the metadata of a resident key from 159 B to 80 B, measured by E18's
    /// `rev_metadata_per_key`.
    ///
    /// **It is removed when the entry is evicted**, which is the property the budget is
    /// supposed to give and did not. Both maps took an entry for every key *ever read* and
    /// gave it back to nobody: at twice the residency budget the view held 284 B per key
    /// read against a budget bounding 2,500 values, and in a long-lived view the term is
    /// linear in history rather than in the budget. Honest absence is unaffected — the
    /// *slot* keeps the version, which is what a miss needs; what goes is the read count and
    /// the clock of a key that is no longer resident, neither of which any answer depends
    /// on: `choose_victim` reads them only for keys that are.
    ///
    /// What it gives up: a key evicted and read again starts its count at one, so under
    /// `CostAware` a returning hot key is briefly as evictable as a cold one. The
    /// alternative is a counter that outlives every entry it describes, which is the
    /// unbounded term this replaces.
    meta: BTreeMap<Key, Meta>,
    /// **Reconstructions this view has authorised and has not yet seen land.**
    ///
    /// One entry per key, because a key has at most one *owned* flight: a second reader at
    /// the same anchor joins it, and a reader at a different anchor folds independently
    /// without claiming ownership. Bounded by `MAX_FLIGHTS`, and every entry is removed by
    /// exactly one of three events — its own completion, a supersession, or a reap after
    /// cancellation.
    in_flight: BTreeMap<Key, InFlight>,
    /// Monotone, never reused, and the identity a late completion is checked against. A
    /// counter rather than a timestamp because the question is only ever "is this still the
    /// flight the view is waiting for", and equality of a `u64` answers it without a clock.
    flight_generation: u64,
    clock: u64,
    /// The rung this view promises, and the mode it was planned in. Both are read from the
    /// circuit's checked fields, so an engine that ignored them fails the IR audit.
    pub rung: Consistency,
    pub mode: Materialize,
    pub stats: Stats,
    /// The bounds this view's deferred merges may spend. [`MergeCaps::OFF`] is the pinned
    /// control the merge is measured against.
    merge_caps: MergeCaps,
}

impl Rev {
    pub fn resident_count(&self) -> u64 {
        self.resident
    }

    /// How many keys the per-key policy metadata holds an entry for.
    ///
    /// **The number the eviction budget is supposed to bound and did not.** Exposed so a
    /// test can assert it rather than infer it from a memory figure, and reported over the
    /// wire as `view_metadata_keys`: a view whose metadata is linear in history rather than
    /// in its budget is a view that does not fit, and nothing said so before.
    pub fn metadata_len(&self) -> usize {
        self.meta.len()
    }

    /// Read a key at the given anchor.
    ///
    /// The four cases are the absence lattice: a `Present` entry whose certification interval
    /// contains the anchor is a hit; anything else reconstructs. `Bottom` reconstructs too — a
    /// key never seen is not a key with no postings, and answering it with the aggregate's
    /// identity would be the money-from-memory-pressure bug the lattice exists to prevent.
    ///
    /// # The postcondition
    ///
    /// **The returned `Anchored` always carries the anchor that was asked for.** Both paths
    /// establish it: a hit only fires inside `[stamp, effective]`, where the value is
    /// unchanged and so is the value at `anchor`; a miss reconstructs over the prefix ending
    /// at `anchor`. Held by `every_answer_is_stamped_with_the_anchor_it_was_asked_for`.
    ///
    /// This was not true before T-02 — a hit reported `effective`, which can be later — and
    /// every caller in this project independently wrote the same compensating branch:
    /// *if the anchor came back different, throw the answer away and rebuild*. The daemon
    /// paid it on 87.4% of keyed reads under concurrent writers; GBS's ledger adapter counted
    /// it as `as_of_reconstructions`. Those branches are now unreachable, which is the
    /// intended outcome: a caller should not have to check that an engine answered the
    /// question it was asked.
    /// **This call cannot join another reader's flight, and that is a safety property rather
    /// than an optimisation — A10-03.**
    ///
    /// `read` takes `&mut self` for its whole body. Waiting on someone else's flight from
    /// here would park a thread holding the view exclusively, and the flight's owner needs
    /// that same `&mut Rev` to call `finish_fold` and publish — so the waiter waits for a
    /// publication that cannot happen. The engine's own two-phase path avoids this by
    /// returning the ticket and waiting in a caller that holds nothing; this one has no
    /// caller to return to.
    ///
    /// It used to loop on `Join` with a comment arguing the case was unreachable: "a
    /// single-threaded caller cannot reach it: it holds the view for the whole call, so no
    /// other flight can have started". That is true of a single-threaded caller and false of
    /// this function, which is public and is reached through a `Mutex<Runtime>` that several
    /// threads share. One thread owns a flight — having released the mutex to fold — while a
    /// second calls `read` through the mutex, joins that flight, and waits holding the lock
    /// the first needs. Both threads are then permanent.
    ///
    /// So the flight is not joined: `ReadMode::Alone` folds this key independently, which is
    /// slower and exact and installs nothing that belongs to somebody else. The `Join` arm is
    /// gone rather than made unreachable, because "cannot happen on this path" was exactly
    /// the claim that was wrong.
    pub fn read(&mut self, base: &dyn Base, key: &Key, anchor: Epoch) -> Anchored {
        // **The whole read, expressed in the two-phase API, so there is one read path and
        // not two.** A second implementation of the certification-interval rule would
        // disagree with this one the first time either changed, and the disagreement would
        // be between a unit test's engine and the daemon's.
        match self.begin_read_with(key, anchor, ReadMode::Alone) {
            ReadOutcome::Hit(a) => a,
            ReadOutcome::Fold(t) => {
                let (value, rows) = base.reconstruct(t.key(), t.anchor());
                self.finish_fold_merging(t, value, rows, Some(base))
            }
            // `ReadMode::Alone` does not produce this outcome. Returning the ticket's own
            // exact answer is not possible without waiting, so the only honest thing left is
            // to say which invariant broke, rather than to wait and hang.
            ReadOutcome::Join(_) => unreachable!(
                "`Rev::read` asked for `ReadMode::Alone` and was handed a join. Waiting here \
                 holds the view against the owner that must publish through it (A10-03), so \
                 this is a deadlock the moment it is taken."
            ),
        }
    }

    /// **Phase one: decide, under the view lock, and then let go of it.**
    ///
    /// Everything that must be atomic with respect to the view happens here — the hit test,
    /// the flight table, the `Pending` marker — and none of it folds. The caller drops the
    /// view guard before doing any of the three things this returns.
    ///
    /// The counters `reads`, `hits`, `misses` and `upqueries` are charged here, because this
    /// is where the decision is made; `base_rows_read` is charged in [`finish_fold`], because
    /// that is where the rows are actually read.
    pub fn begin_read(&mut self, key: &Key, anchor: Epoch) -> ReadOutcome {
        self.begin_read_with(key, anchor, ReadMode::Shared)
    }

    /// `begin_read`, with the caller stating whether it is in a position to wait.
    ///
    /// A caller that holds the view — or any lock the flight's owner needs to publish through
    /// — cannot wait for that flight, and asking for [`ReadMode::Alone`] says so. What it
    /// gets instead is an independent, uninstalled fold: slower, exact, and touching no slot
    /// that belongs to another flight.
    pub fn begin_read_with(&mut self, key: &Key, anchor: Epoch, mode: ReadMode) -> ReadOutcome {
        self.stats.reads += 1;
        self.clock += 1;
        let now = self.clock;
        let m = self.meta.entry(key.clone()).or_default();
        m.reads += 1;
        m.last_read = now;

        if let Some(a) = self.hit(key, anchor) {
            self.stats.hits += 1;
            return ReadOutcome::Hit(a);
        }

        self.stats.misses += 1;
        self.stats.upqueries += 1;

        // A flight whose owner died leaves its marker behind; the first reader to hold the
        // view again puts the slot back and takes over. Reaping here rather than in `Drop`
        // is what keeps the destructor lock-free.
        self.reap_cancelled(key);

        enum Decision {
            Join(Arc<Completion>),
            Alone,
            Own,
        }
        let decision = match self.in_flight.get(key) {
            // **A caller that cannot wait does not take a place in the queue.** Counted
            // apart from both capacity refusals: this is not an overloaded flight table and
            // not a full waiter list, it is a reader whose own lock discipline forbids the
            // wait, and reporting it as either would be a refusal nobody can act on
            // (A10-05).
            Some(f) if f.anchor == anchor && mode == ReadMode::Alone => {
                let _ = f;
                self.stats.joins_declined_by_caller += 1;
                Decision::Alone
            }
            Some(f) if f.anchor == anchor => {
                // **This is the one place a completion becomes a rendezvous.** Marked before
                // the lock is taken, and under the view lock, so that a `finish_fold` racing
                // this join cannot decide the flight is uncontended after the decision to
                // join has been made. Both run under V, so they are ordered by V.
                f.done.mark_joined();
                let mut st = f.done.lock();
                if st.answer.is_none() && !st.cancelled && st.waiters < MAX_WAITERS {
                    st.waiters += 1;
                    drop(st);
                    Decision::Join(f.done.clone())
                } else {
                    // **A full waiter list is a different refusal from a full flight
                    // table**, and reporting one under the other is a refusal nobody can
                    // act on. A settled flight is not a refusal at all: this reader simply
                    // arrived after the answer and folds its own.
                    if st.answer.is_none() && !st.cancelled {
                        drop(st);
                        self.stats.waiters_refused += 1;
                    }
                    Decision::Alone
                }
            }
            // **A flight at a different anchor is a different question.** It folds a
            // different prefix and will install a value certified at its own anchor;
            // waiting for it would be answering this caller with someone else's snapshot.
            Some(_) => Decision::Alone,
            None if self.in_flight.len() >= MAX_FLIGHTS => {
                self.stats.flights_refused += 1;
                Decision::Alone
            }
            None => Decision::Own,
        };

        match decision {
            Decision::Join(done) => {
                self.stats.pending_joins += 1;
                ReadOutcome::Join(WaitTicket {
                    done,
                    anchor,
                    consumed: false,
                })
            }
            Decision::Alone => ReadOutcome::Fold(FoldTicket {
                key: key.clone(),
                anchor,
                generation: 0,
                install: false,
                done: None,
                settled: false,
                gap_begin: self.applied.saturating_sub(anchor),
            }),
            Decision::Own => {
                self.flight_generation += 1;
                let generation = self.flight_generation;
                // **The lag this read started with, before it folded anything.** A flight
                // authorised at an anchor already below `applied` was behind on arrival; one
                // authorised level with `applied` falls behind only if the frontier moves
                // under it. `finish_fold` subtracts to say which happened (A10-11).
                let gap_begin = self.applied.saturating_sub(anchor);
                self.stats.gap_at_begin_total += gap_begin;
                self.stats.gap_begin_samples += 1;
                if gap_begin > 0 {
                    self.stats.flights_behind_at_begin += 1;
                }
                let done = Arc::new(Completion::default());
                // **The marker replaces an absence, never a value.** A `Present` entry that
                // could not answer *this* anchor can still answer every anchor inside its
                // own interval, and overwriting it with `Pending` would evict a good answer
                // to record that somebody is looking for a different one.
                //
                // **So "the slot is `Pending` for the duration of a flight" is false, and it
                // was claimed** — A10-12. A flight over a key whose entry is `Present` at
                // some other anchor leaves that entry exactly where it is: `prior` is `None`
                // precisely because there is nothing to put back. Anything reasoning from
                // "a key in flight is a key whose slot is `Pending`" is reasoning about the
                // other case.
                //
                // The three properties this arm actually establishes, which the one sentence
                // ran together:
                //
                //   * **the answer** returned to *this* reader is exact at *its* anchor,
                //     which follows from the fold's precondition and not from any slot;
                //   * **the certification** of whatever is resident is unchanged — a
                //     retained `Present` keeps its own stamp and its own interval, and a
                //     read inside that interval is still a hit while the flight is out;
                //   * **the ownership** of the slot, which is what `generation` decides, and
                //     which is about who may *write* it, not about what it holds.
                let prior = match self.slots.get_mut(key) {
                    Some(Slot::Present(..)) => None,
                    // In place where the map already holds the key: `insert` would clone it
                    // and drop the clone, which is a heap allocation per miss for nothing.
                    Some(slot) => {
                        let p = std::mem::replace(slot, Slot::Pending(anchor));
                        Some(p)
                    }
                    None => {
                        self.slots.insert(key.clone(), Slot::Pending(anchor));
                        Some(Slot::Bottom)
                    }
                };
                self.in_flight.insert(
                    key.clone(),
                    InFlight {
                        anchor,
                        generation,
                        done: done.clone(),
                        prior,
                    },
                );
                ReadOutcome::Fold(FoldTicket {
                    key: key.clone(),
                    anchor,
                    generation,
                    install: true,
                    done: Some(done),
                    settled: false,
                    gap_begin,
                })
            }
        }
    }

    /// **Phase two: take the view again and install, if this flight is still the one.**
    ///
    /// `value` is the fold of `ticket.key()` over the prefix ending at `ticket.anchor()`,
    /// and `rows` the base rows it read. The answer returned is exact at the ticket's anchor
    /// whatever the view decides to do with it: an install is an optimisation for the *next*
    /// reader, never a condition on this one's correctness.
    ///
    /// # Why a generation, and what it is not for
    ///
    /// The check is not about the value. A ticket's `(key, anchor)` is fixed at
    /// `begin_read`, so a late completion's value is still the correct value at its own
    /// anchor, and the certification interval keeps a stale *stamp* from ever answering a
    /// later anchor. What the generation prevents is a superseded owner writing over the
    /// entry a newer flight installed — moving a resident stamp backwards, which costs every
    /// subsequent reader a refold — and clearing an `in_flight` entry it does not own, which
    /// would strand the successor's `Pending` marker with nobody to publish it.
    pub fn finish_fold(&mut self, ticket: FoldTicket, value: Value, rows: u64) -> Anchored {
        self.finish_fold_merging(ticket, value, rows, None)
    }

    /// `finish_fold`, with the base available so a late landing can merge instead of pinning.
    ///
    /// # What a deferred merge is, and what it deliberately is not
    ///
    /// A flight folds the prefix ending at its anchor `a`. If the view advanced to `e > a`
    /// while it was out, the landing entry holds this key's value as of `a` and has not seen
    /// the deltas in `(a, e]`. `install` therefore **pins** it: the entry keeps its own
    /// stamp, serves only `[a, a]`, and every later reader reconstructs. That is sound and
    /// it is the weaker of the two available landings.
    ///
    /// The merge is the other one. Fold this key's deltas over `(a, e]` into the landing
    /// value and install at `e`, so the entry lands **current** and the next reader at the
    /// frontier hits instead of reconstructing.
    ///
    /// **What the caller receives does not change, and this is the invariant the whole thing
    /// rests on.** The reader asked at `a` and is answered at `a` — the returned `Anchored`,
    /// and the `Joined` published to every same-anchor waiter, both carry the fold's own
    /// value and its own anchor. The merge changes what is written into the view, never what
    /// is handed back. A merge that returned the merged value would be answering a question
    /// nobody asked, and every certification-interval guarantee in Chapter 3 is stated about
    /// the anchor that was requested.
    ///
    /// # Why it is bounded, and why it refuses rather than truncating
    ///
    /// The merge reads `deltas_at` once per epoch in the suffix, and each call returns that
    /// epoch's rows across **every** key, not just this one. So the work is a function of the
    /// suffix's length *and* of how busy those epochs were, and both are bounded separately
    /// by [`MergeCaps`]. A suffix that exceeds either cap is not partially merged: a partial
    /// merge installed at `e` would be a value missing some of its own history under a
    /// current stamp, which is the one thing a pin exists to prevent. It pins, and the
    /// refusal is counted under its own cause.
    ///
    /// The third refusal is retention. `deltas_at` reports an epoch the base no longer holds
    /// as *empty*, which is indistinguishable from an epoch in which nothing happened, so a
    /// merge across a compacted boundary would silently drop money rather than fail.
    /// [`Base::deltas_available_from`] is what makes that case a refusal.
    ///
    /// Ownership is unchanged: only a landing that still holds its generation installs at
    /// all, and only such a landing can merge. A superseded completion writes nothing,
    /// merged or not.
    pub fn finish_fold_merging(
        &mut self,
        ticket: FoldTicket,
        value: Value,
        rows: u64,
        base: Option<&dyn Base>,
    ) -> Anchored {
        let mut ticket = ticket;
        ticket.settled = true;
        self.stats.base_rows_read += rows;
        let answer = Anchored {
            value,
            anchor: ticket.anchor,
        };

        // **The gap again, at landing.** Recorded for the folds that install — the ones
        // whose answer becomes resident state, which is what a merge decision is about. The
        // comment here used to say "installed or not", which the `if` below has never done;
        // a fold that does not install has no landing to be late for, and counting it would
        // mix two populations under one mean.
        //
        // `gap_finish_samples` is the divisor. Without it the total is a total, and the
        // first run to print a mean from it divided by the wrong counter and reported a mean
        // above the maximum.
        if ticket.install {
            let gap_finish = self.applied.saturating_sub(ticket.anchor);
            self.stats.gap_at_finish_total += gap_finish;
            self.stats.gap_finish_samples += 1;
            self.stats.gap_at_finish_max = self.stats.gap_at_finish_max.max(gap_finish);
            if gap_finish > ticket.gap_begin {
                self.stats.flights_that_fell_behind += 1;
            }
        }
        let mine = ticket.install
            && self
                .in_flight
                .get(&ticket.key)
                .is_some_and(|f| f.generation == ticket.generation);
        if mine {
            self.in_flight.remove(&ticket.key);
            // Moved, not cloned. The ticket owns this key and is about to be dropped.
            let key = std::mem::take(&mut ticket.key);
            let merged = if ticket.anchor < self.applied {
                base.and_then(|b| self.merge_suffix(b, &key, ticket.anchor))
            } else {
                // Not late. There is no suffix, and asking for one would visit the base for
                // nothing on the overwhelmingly common path.
                None
            };
            match merged {
                Some(delta) => {
                    self.stats.deferred_merges += 1;
                    let at = self.applied;
                    self.install(key, value + delta, at);
                }
                None => {
                    if ticket.anchor < self.applied {
                        self.stats.pinned_installs += 1;
                    }
                    self.install(key, value, ticket.anchor);
                }
            }
        } else {
            self.stats.uninstalled_folds += 1;
        }

        if let Some(d) = &ticket.done {
            d.publish(Joined {
                answer,
                base_rows: rows,
            });
            // **Folded into the view's stats as the flight ends**, which is the last moment
            // this completion is reachable from the view. A flight nobody joined contributes
            // zero, and that zero is the whole of the E18 repair (A10-19).
            self.stats.completion_locks += d.locks();
        }
        answer
    }

    /// The hit test, and the whole of the certification interval, stated once.
    ///
    /// # The postcondition
    ///
    /// **The returned `Anchored` always carries the anchor that was asked for.** Both paths
    /// establish it: a hit only fires inside `[stamp, effective]`, where the value is
    /// unchanged and so is the value at `anchor`; a miss reconstructs over the prefix ending
    /// at `anchor`. Held by `every_answer_is_stamped_with_the_anchor_it_was_asked_for`.
    ///
    /// This was not true before T-02 — a hit reported `effective`, which can be later — and
    /// every caller in this project independently wrote the same compensating branch:
    /// *if the anchor came back different, throw the answer away and rebuild*. The daemon
    /// paid it on 87.4% of keyed reads under concurrent writers; GBS's ledger adapter counted
    /// it as `as_of_reconstructions`. Those branches are now unreachable, which is the
    /// intended outcome: a caller should not have to check that an engine answered the
    /// question it was asked.
    fn hit(&self, key: &Key, anchor: Epoch) -> Option<Anchored> {
        // The effective version of a resident entry is the later of its own stamp and the
        // view-wide applied epoch: an entry that received no delta in an epoch is still
        // current through that epoch, and rewriting every resident entry's stamp on every
        // epoch would make maintenance O(resident) instead of O(deltas).
        //
        // The inheritance is sound only for an entry that was resident for every epoch in
        // `(stamp, applied]` — that is, one whose stamp is at or after the frontier the
        // view had already applied when the entry was installed. An entry installed at an
        // *older* anchor than `applied` (a historical read, or a reconstruction that
        // raced maintenance) has not seen the deltas in between, so promoting it to
        // `applied` would serve a stale value under a fresh anchor. It keeps its own
        // stamp, and a read above that stamp reconstructs.
        let Some(Slot::Present(v, e)) = self.slots.get(key) else {
            // `Pending`, `Hole` and `Bottom` all reconstruct, and for the same reason: none
            // of them holds a value. A `Pending` is not an answer-in-progress this reader
            // may wait on blindly — whether it may wait is decided by the anchor, above.
            return None;
        };
        let effective = if self.certified_through_applied(key, *e) {
            (*e).max(self.applied)
        } else {
            *e
        };
        // **The certification interval, and the answer is exact everywhere inside it.**
        //
        // The entry holds this key's value as of `stamp`, and it received no delta
        // between `stamp` and `effective` — that is what `effective` means. So the value
        // is unchanged across the whole closed interval `[stamp, effective]`, and it is
        // the correct answer at *every* anchor in it, not merely at `effective`. This is
        // Theorem 4.1's own step (3), and the read now says so by returning the anchor it
        // was asked for.
        //
        // It used to return `anchor: effective`, which is honest and useless: the caller
        // wants an answer true at its snapshot, an answer stamped later includes writes
        // that snapshot excludes *as far as the caller can tell*, and so
        // `answer_from_view` discarded it and folded the base instead — at roughly a
        // hundred times the cost, for 24.3% of keyed reads in-process and 87.4% over the
        // wire under four readers and two writers. The value was right the whole time;
        // the engine could not tell, because the read reported the wrong end of the
        // interval.
        //
        // The lower bound is not decoration. `anchor < stamp` is a read *below* the
        // entry — a delta landed in `(anchor, stamp]` that this anchor must not see — and
        // it has to reconstruct. It used to be served, with `effective` attached, and was
        // correct only because the caller then threw it away; counting it as a hit is
        // what made `hits` a count of answers rather than of answers *served*.
        if *e <= anchor && anchor <= effective {
            return Some(Anchored { value: *v, anchor });
        }
        None
    }

    /// Put back what a cancelled flight replaced, and let the next reader take over.
    fn reap_cancelled(&mut self, key: &Key) {
        let dead = self.in_flight.get(key).is_some_and(|f| f.done.cancelled());
        if !dead {
            return;
        }
        if let Some(f) = self.in_flight.remove(key) {
            // A cancelled flight ends here rather than in `finish_fold`, so its rendezvous is
            // accounted for here or nowhere.
            self.stats.completion_locks += f.done.locks();
            if let Some(prior) = f.prior {
                match self.slots.get_mut(key) {
                    Some(slot) => *slot = prior,
                    None => {
                        self.slots.insert(key.clone(), prior);
                    }
                }
            }
        }
    }

    /// Is this key's resident entry current through the view's `applied` frontier?
    ///
    /// True for an entry installed at or after the frontier the view had applied at the
    /// time, and for one last written by `apply_epoch`; false for a historical read.
    fn certified_through_applied(&self, key: &Key, _stamp: Epoch) -> bool {
        !self.pinned.contains(key)
    }

    /// This key's total delta over `(anchor, applied]`, or `None` if the merge is refused.
    ///
    /// Two passes are deliberately **not** taken: the suffix is walked once, accumulating the
    /// key's deltas and the physical rows visited together, and the row cap is checked as it
    /// goes. Walking once to count and again to sum would double the cost of the thing being
    /// measured. When the cap is exceeded the accumulated sum is discarded — a merge is all
    /// or nothing, because a partial one installs a value missing part of its own history
    /// under a current stamp.
    ///
    /// The rows visited are counted **whether or not the merge completes**, because they were
    /// really read: a refusal that reports zero cost is a refusal that looks free.
    fn merge_suffix(&mut self, base: &dyn Base, key: &Key, anchor: Epoch) -> Option<Value> {
        let caps = self.merge_caps;
        let span = self.applied.saturating_sub(anchor);
        if span > caps.max_epochs {
            self.stats.merges_refused_epochs += 1;
            return None;
        }
        // `deltas_at` cannot distinguish an epoch it no longer holds from an empty one, so
        // the boundary is asked for rather than inferred.
        if anchor + 1 < base.deltas_available_from() {
            self.stats.merges_refused_unavailable += 1;
            return None;
        }

        let mut delta: Value = 0;
        let mut visited: u64 = 0;
        for e in (anchor + 1)..=self.applied {
            let rows = base.deltas_at(e);
            visited += rows.len() as u64;
            if visited > caps.max_rows {
                self.stats.merge_rows_visited += visited;
                self.stats.merges_refused_rows += 1;
                return None;
            }
            for (k, d) in rows {
                if k == *key {
                    delta += d;
                }
            }
        }
        self.stats.merge_rows_visited += visited;
        self.stats.merge_epochs_merged += span;
        Some(delta)
    }

    fn install(&mut self, key: Key, value: Value, anchor: Epoch) {
        let was_resident = self.slots.get(&key).is_some_and(|s| s.is_resident());
        // A reconstruction anchored below the view's applied frontier is a historical
        // answer. It is worth keeping — it is what a dispute asks for — but it is not
        // current, and must never inherit `applied`.
        if anchor < self.applied {
            self.pinned.insert(key.clone());
        } else {
            self.pinned.remove(&key);
        }
        self.slots.insert(key, Slot::Present(value, anchor));
        if !was_resident {
            self.resident += 1;
        }
        self.stats.peak_resident = self.stats.peak_resident.max(self.resident);
        self.enforce_budget();
    }

    fn enforce_budget(&mut self) {
        let Some(b) = self.budget else { return };
        while self.resident > b {
            let Some(victim) = self.choose_victim() else {
                break;
            };
            if let Some(s) = self.slots.get_mut(&victim) {
                // Honest absence: the value goes, the version stays.
                if s.evict() {
                    self.resident -= 1;
                    self.stats.evictions += 1;
                    self.pinned.remove(&victim);
                    // The metadata goes with the value. The version stays in the slot, which
                    // is what honest absence requires; the read count and the clock describe
                    // an entry that no longer exists and are read only for resident keys.
                    self.meta.remove(&victim);
                }
            }
        }
    }

    fn choose_victim(&self) -> Option<Key> {
        let resident = || self.slots.iter().filter(|(_, s)| s.is_resident());
        match self.policy {
            Policy::Random => resident().next().map(|(k, _)| k.clone()),
            Policy::Lru => resident()
                .min_by_key(|(k, _)| self.meta.get(*k).map_or(0, |m| m.last_read))
                .map(|(k, _)| k.clone()),
            Policy::CostAware => {
                // Keep what is read often; drop what is cheap to rebuild. The score is
                // reads per unit of reconstruction work, and the entry with the lowest
                // score goes. Approximating reconstruction cost by 1 here is deliberate:
                // with per-key checkpoints the cost is bounded by the interval and is
                // nearly uniform, which is itself a consequence of SC7.
                resident()
                    .min_by_key(|(k, _)| self.meta.get(*k).map_or(0, |m| m.reads))
                    .map(|(k, _)| k.clone())
            }
        }
    }

    /// Apply one epoch's deltas. **O(deltas), not O(resident)** — an entry that receives no
    /// delta is not touched, and its currency is derived on read from `applied`.
    ///
    /// This is where a consistency rung's cost actually falls, which was itself a measured
    /// finding: instrumenting only the read path showed no difference between rungs at all.
    pub fn apply_epoch(&mut self, base: &dyn Base, e: Epoch) {
        for (key, delta) in base.deltas_at(e) {
            // An entry pinned to a historical anchor is missing the deltas between that
            // anchor and now; folding this one in would compound the gap rather than
            // close it. Treated exactly like a non-resident key: skipped, and rebuilt by
            // the next read that needs it fresh.
            if self.pinned.contains(&key) {
                self.stats.deltas_skipped += 1;
                continue;
            }
            match self.slots.get_mut(&key) {
                // A delta at or below the entry's own stamp is already in the value —
                // an entry reconstructed ahead of the applied frontier has it — and
                // adding it again is the double-application anomaly.
                Some(Slot::Present(_, stamp)) if *stamp >= e => {
                    self.stats.deltas_skipped += 1;
                }
                Some(Slot::Present(v, stamp)) => {
                    *v += delta;
                    *stamp = e;
                    self.stats.deltas_applied += 1;
                }
                // **A view that materialises everything installs the key.** Under
                // `Materialize::Full` there is no budget and nothing is ever evicted, so a
                // key that is not resident is one this view has not seen before — and a
                // full view is exactly the promise that it will hold every key. Installing
                // it here is what makes the view's contents *every* key rather than every
                // key somebody happened to read, which is the difference between a report
                // and a sample of one.
                //
                // Sound because a key's first delta is its whole value: the base is
                // append-only and this runtime sees every epoch in order, so there is no
                // earlier history for the new entry to be missing.
                // **A reconstruction for this key is in flight, and this epoch is not
                // folded into it here.** There is no value in a `Pending` slot to add a
                // delta to, and the flight will return a fold over its *own* prefix — which
                // already contains this epoch if its anchor is at or above `e`, and must not
                // contain it otherwise. Treating the marker as an entry and writing
                // `Present(delta, e)` into it publishes one epoch's delta as the key's
                // balance: every reader whose anchor falls in the resulting interval takes
                // it as a hit, and the account is wrong by its entire history until the
                // flight lands.
                //
                // Skipping is the same treatment a non-resident key gets, and for the same
                // reason: the next read that needs the epoch reconstructs and picks it up.
                Some(Slot::Pending(_)) => {
                    self.stats.deltas_skipped += 1;
                }
                None if self.mode == Materialize::Full => {
                    self.slots.insert(key, Slot::Present(delta, e));
                    self.resident += 1;
                    self.stats.deltas_applied += 1;
                }
                // Not resident: the delta is skipped, and correctly so. The entry's hole
                // still carries its old version, so a later read reconstructs from the
                // base and picks this delta up along the way. This is the saving.
                _ => self.stats.deltas_skipped += 1,
            }
        }
        self.applied = e;
        self.stats.resident_entry_epochs += self.resident;
    }

    /// Drop everything, keeping no versions. Used only by the rebuild-from-base check,
    /// where the point is to prove the derived layer holds no information the base does not.
    pub fn wipe(&mut self) {
        self.slots.clear();
        self.pinned.clear();
        self.meta.clear();
        // Outstanding flights are forgotten, not cancelled: their owners still hold live
        // tickets and will still answer their own callers exactly, at their own anchors.
        // What they lose is the right to install — `finish_fold` finds no entry at their
        // generation — which is precisely the guarantee a wipe is asking for.
        self.in_flight.clear();
        self.resident = 0;
    }

    pub fn slot(&self, key: &Key) -> Slot<Value> {
        self.slots.get(key).cloned().unwrap_or(Slot::Bottom)
    }

    /// Whether this view holds **every** key the base has ever had a delta for.
    ///
    /// Three conditions, and the first is the one a careless version would leave out.
    /// `Materialize::Full` is what makes `apply_epoch` *install* a key it has not seen
    /// rather than skip it; without that a view with no budget still holds only what
    /// somebody has read, and a report over it would return whichever accounts happened to
    /// be warm. A test found that, not a reading of this file.
    ///
    /// The precondition for serving a *report* from the view rather than from the base. A
    /// partial view's resident subset is not an answer to "every account's balance": it is
    /// the accounts that happen to be in memory, which is a different question and one
    /// nobody asked. A report over a partial view is answered by folding the base, and the
    /// benchmark row says which path served it.
    pub fn is_full(&self) -> bool {
        self.mode == Materialize::Full && self.budget.is_none() && self.stats.evictions == 0
    }

    /// Every resident entry, in key order, with the epoch each is true at.
    ///
    /// **The whole view, by iteration** — what a report needs and what nothing could ask for
    /// before. `read` answers one key and is the mechanism the phase diagram is about;
    /// this is the other end of the same trade, where the view has been maintained through
    /// every append and the answer is already there.
    ///
    /// Only `Present` entries are yielded. A `Hole` is an evicted value whose version is
    /// kept, and including it — or its version without its value — would be inventing a
    /// number; a caller wanting a report over a view with holes must fold the base instead,
    /// which is what [`is_full`](Self::is_full) is for.
    ///
    /// The effective anchor of an entry is the later of its own stamp and the view-wide
    /// `applied` epoch, for entries that are not pinned — the same inheritance rule `read`
    /// uses, and stated once here rather than twice, because two copies of that rule would
    /// disagree the first time either changed.
    pub fn iter_resident(&self) -> impl Iterator<Item = (&Key, Value, Epoch)> + '_ {
        self.slots.iter().filter_map(move |(k, s)| match s {
            Slot::Present(v, stamp) => {
                let anchor = if self.pinned.contains(k) {
                    *stamp
                } else {
                    (*stamp).max(self.applied)
                };
                Some((k, *v, anchor))
            }
            _ => None,
        })
    }

    /// The epoch through which this view as a whole has had deltas applied.
    ///
    /// A report's rows are all true at one moment, and this is that moment. A report that
    /// stamped each row with its own entry's anchor would be a set of answers to different
    /// questions presented as one table.
    pub fn applied_through(&self) -> Epoch {
        self.applied
    }
}

/// Why a circuit could not be installed on this runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum Unsupported {
    /// Outside the key-aggregate fragment.
    Shape { node: NodeId, op: &'static str },
    /// No derivable key, so there is nothing to materialize per key.
    NoKey { node: NodeId },
    /// The aggregate is not one the runtime maintains.
    Aggregate { node: NodeId, agg: &'static str },
    /// A node the output depends on is outside the fragment, even though the output itself
    /// is inside it.
    Upstream {
        output: NodeId,
        node: NodeId,
        op: &'static str,
    },
    /// The aggregate reads a derived source: a view this runtime does not hold, and whose
    /// rows no `Base` can fold.
    DerivedSource { node: NodeId, relation: String },
}

impl Unsupported {
    pub fn explain(&self) -> String {
        match self {
            Unsupported::Shape { node, op } => format!(
                "node {node} is a `{op}`, which lowers and verifies but is outside the \
                 key-aggregate fragment this runtime executes"
            ),
            Unsupported::NoKey { node } => {
                format!(
                    "node {node} has no derivable key, so there is nothing to materialize per key"
                )
            }
            Unsupported::Aggregate { node, agg } => {
                format!("node {node} aggregates with `{agg}`, which this runtime does not maintain")
            }
            Unsupported::Upstream { output, node, op } => format!(
                "output node {output} is a keyed aggregate this runtime maintains, but it \
                 reads node {node}, a `{op}`. Every answer comes from `Base::reconstruct`, \
                 which is told a key and an epoch and nothing else, so a stage between the \
                 source and the aggregate is not executed — it is dropped, and the view \
                 answers as though it were not there"
            ),
            Unsupported::DerivedSource { node, relation } => format!(
                "node {node} reads `{relation}`, which is derived rather than a base. This \
                 runtime folds a base through `Base::reconstruct`; it does not hold the \
                 intermediate a derived source names"
            ),
        }
    }
}

/// The runtime: a circuit, and one REV per named output.
pub struct Runtime {
    pub circuit: Circuit,
    pub views: Vec<Rev>,
    pub epoch: Epoch,
}

impl Runtime {
    /// Install a compiled circuit. Rejects anything outside the executable fragment rather
    /// than silently mis-executing it.
    ///
    /// # The fragment, and why the theorem stops where this function does
    ///
    /// What is accepted is a keyed aggregate over a linear operator — `sum` and `count` —
    /// which is **Q_lin** of Definition 4.1.1, and Theorem 4.1 (Epoch-Anchored
    /// Reconstruction) is stated for exactly that fragment. The theorem's proof needs
    /// linearity in one step and one only: advancing a *resident* key's certified entry by
    /// an epoch's delta requires that the delta to that key's output be computable from the
    /// epoch's rows and the key's own current value.
    ///
    /// A join or a `max` breaks that step rather than the theorem's other clauses.
    /// Reconstruction stays pure and total for them — the upquery paths exist for every
    /// circuit — but the delta to a resident key's output can depend on input rows belonging
    /// to keys that were evicted, so an incremental advance would have to either re-read the
    /// base (which makes `apply` an upquery and changes the cost model of C3) or keep state
    /// for absent keys (which is full materialization under another name). Refusing here is
    /// what keeps Chapter 9 inside the fragment the theorem covers; Open case 4.1.α states
    /// the general case as a conjecture rather than pretending it is proved.
    ///
    /// # What is checked, and where it used to stop
    ///
    /// **The whole graph reachable from each output**, and until cycle 10 it was the output
    /// node alone (A10-07's sibling, A10-06). The paragraph above was true of what this
    /// function *meant* and false of what it did: `Source → Join → Aggregate(sum)` presented
    /// a keyed `sum` as its output and installed, and `Q_lin` was a claim in a doc comment
    /// rather than a property of the artifact. `thesis_drift::theorem_4_1_names_its_fragment`
    /// asserted the restriction was true of the code on the strength of this comment.
    ///
    /// It matters more than a missing refusal usually would, because of what happens after
    /// this function returns: nothing reads the circuit again. Every answer comes from
    /// `Base::reconstruct`, which is given a key and an epoch. So an unsupported stage is not
    /// executed badly — it is not executed at all, and the view answers as though the stage
    /// were absent. A `Filter` under the aggregate is the sharpest case: no error, no
    /// approximation, just the unfiltered sum, which is precisely what an answer-level test
    /// expects to see.
    ///
    /// The accepted fragment is therefore stated as what a `Base` can actually answer: one
    /// keyed `sum`/`count` directly over one immutable base source. The server's planner
    /// (`nilestream_server::scan_fold::plan`) walks the same shape and returns `None` rather
    /// than folding what it cannot; this is the other door into the same runtime, the one a
    /// hand-built `Circuit` comes through, and the two now agree.
    pub fn install(
        circuit: Circuit,
        budget: Option<u64>,
        policy: Policy,
    ) -> Result<Runtime, Unsupported> {
        let mut views = Vec::new();
        let outputs: Vec<(String, NodeId)> = circuit
            .outputs
            .iter()
            .map(|(n, i)| (n.clone(), *i))
            .collect();
        for (name, id) in outputs {
            let n = circuit.node(id);
            match &n.op {
                Op::Aggregate { aggs, .. } => {
                    for (a, _) in aggs {
                        if !matches!(a, Agg::Sum | Agg::Count) {
                            return Err(Unsupported::Aggregate {
                                node: id,
                                agg: a.as_str(),
                            });
                        }
                    }
                }
                other => {
                    return Err(Unsupported::Shape {
                        node: id,
                        op: other.name(),
                    })
                }
            }
            if n.key.is_none() {
                return Err(Unsupported::NoKey { node: id });
            }

            // **The whole reachable graph, not only the output node — A10-06.**
            //
            // This function used to look at `circuit.outputs` and stop. An output that was a
            // keyed `sum` installed, whatever fed it — and after `install` returns, the
            // circuit is never consulted again: every answer comes from `Base::reconstruct`,
            // which is handed a key and an epoch and is told nothing about the graph. So a
            // `Source → Filter → Aggregate` circuit installed happily and then answered as
            // if the filter were not there. Not a crash, not a refusal: a well-formed wrong
            // number from a public API, and no answer-level test could catch it, because the
            // answer is exactly what the *unfiltered* circuit should give.
            //
            // The server has a planner that walks this properly (`scan_fold::plan`) and
            // returns `None` on anything it cannot fold. This is the other door into the
            // same runtime — the one a hand-built `Circuit` comes through — and it had no
            // such check.
            //
            // The accepted fragment is therefore stated as what `Base` can actually answer:
            // one keyed aggregate directly over one immutable base. A `Filter` or `Map`
            // between them is not "nearly supported"; it is a stage nothing executes.
            let mut seen = BTreeSet::new();
            let mut stack = vec![id];
            while let Some(cur) = stack.pop() {
                if !seen.insert(cur) {
                    continue;
                }
                let m = circuit.node(cur);
                match &m.op {
                    // The output aggregate itself, already checked above.
                    Op::Aggregate { .. } if cur == id => {}
                    Op::Source {
                        is_base, relation, ..
                    } => {
                        if !*is_base {
                            return Err(Unsupported::DerivedSource {
                                node: cur,
                                relation: relation.clone(),
                            });
                        }
                    }
                    other => {
                        return Err(Unsupported::Upstream {
                            output: id,
                            node: cur,
                            op: other.name(),
                        })
                    }
                }
                stack.extend(m.inputs.iter().copied());
            }
            // Read the checked fields. This is not incidental: the IR's accessed-field
            // audit fails if a consumer plans without consulting them, and a runtime that
            // ignored the rung would serve stale state and return a well-formed wrong
            // answer that no answer-level test would catch.
            let contract = *n.contract.get();
            let _ = n.anchor.get();
            let _ = n.conservation_transparent.get();
            let _ = n.lineage.get();

            // The contract decides the budget: a `full` or `pinned` view is not evictable,
            // whatever budget the caller suggested.
            let effective_budget = match (contract.materialize, contract.retain) {
                (Materialize::Full, _) | (_, Retention::Pinned) | (_, Retention::Forever) => None,
                _ => budget,
            };
            views.push(Rev {
                node: id,
                name,
                slots: BTreeMap::new(),
                resident: 0,
                budget: effective_budget,
                policy,
                applied: 0,
                next_to_apply: 0,
                pinned: BTreeSet::new(),
                meta: BTreeMap::new(),
                in_flight: BTreeMap::new(),
                flight_generation: 0,
                clock: 0,
                rung: contract.consistency,
                mode: contract.materialize,
                stats: Stats::default(),
                merge_caps: MergeCaps::default(),
            });
        }
        views.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Runtime {
            circuit,
            views,
            epoch: 0,
        })
    }

    /// Set every view's merge bounds. [`MergeCaps::OFF`] is the pinned control.
    ///
    /// A runtime-wide setter rather than an `install` parameter: the ablation is a property
    /// of a *run*, and threading it through nine construction sites would put a measurement
    /// knob in the signature every caller has to answer.
    pub fn set_merge_caps(&mut self, caps: MergeCaps) {
        for v in &mut self.views {
            v.merge_caps = caps;
        }
    }

    /// The merge bounds in force, so a transcript can say which arm produced its numbers.
    ///
    /// **Reported rather than assumed.** The measurement compares a merging build against a
    /// pinned one, and the two are the same binary with a different value here. A run that
    /// cannot state which value it had is a run whose arm is a matter of trusting the
    /// invocation, which is how a control ends up scored against itself.
    pub fn merge_caps(&self) -> MergeCaps {
        self.views.first().map(|v| v.merge_caps).unwrap_or_default()
    }

    pub fn view_mut(&mut self, name: &str) -> Option<&mut Rev> {
        self.views.iter_mut().find(|v| v.name == name)
    }

    pub fn view(&self, name: &str) -> Option<&Rev> {
        self.views.iter().find(|v| v.name == name)
    }

    /// Advance every view to epoch `e`.
    ///
    /// The rung is honoured here, and this is where its cost lands. A `Bounded { epochs: k }`
    /// view need only be maintained every k epochs; a `ledger_consistent` view must be
    /// maintained at every one. That difference is a factor of k in maintenance work, and
    /// it is invisible on the read path — which is why an earlier version of this
    /// experiment, instrumented only on reads, measured no difference between rungs at all
    /// and reported a null.
    pub fn advance(&mut self, base: &dyn Base, e: Epoch) {
        self.epoch = e;
        for v in &mut self.views {
            let stride = match v.rung {
                Consistency::Bounded { epochs, .. } => epochs.max(1),
                _ => 1,
            };
            if e.is_multiple_of(stride) {
                // Every epoch since the last maintenance, in order. A bounded rung
                // batches maintenance; it does not discard the deltas it batched over.
                // Applying only `e` would leave the view certified through `e` while
                // the `stride - 1` epochs before it had never been folded into anyone,
                // which is a wrong answer rather than a stale one.
                for epoch in v.next_to_apply..=e {
                    v.apply_epoch(base, epoch);
                }
                v.next_to_apply = e.saturating_add(1);
                v.stats.maintenance_passes += 1;
            } else {
                // Still accrue the memory integral: the entries are resident whether or
                // not this epoch touched them, and charging only maintained epochs would
                // make a lax rung look free rather than cheap.
                v.stats.resident_entry_epochs += v.resident;
            }
        }
    }

    /// Total counted work across every view.
    pub fn stats(&self) -> Stats {
        let mut s = Stats::default();
        for v in &self.views {
            s.reads += v.stats.reads;
            s.hits += v.stats.hits;
            s.misses += v.stats.misses;
            s.upqueries += v.stats.upqueries;
            s.base_rows_read += v.stats.base_rows_read;
            s.deltas_applied += v.stats.deltas_applied;
            s.deltas_skipped += v.stats.deltas_skipped;
            s.evictions += v.stats.evictions;
            s.maintenance_passes += v.stats.maintenance_passes;
            s.peak_resident = s.peak_resident.max(v.stats.peak_resident);
            s.resident_entry_epochs += v.stats.resident_entry_epochs;
            s.pending_joins += v.stats.pending_joins;
            s.uninstalled_folds += v.stats.uninstalled_folds;
            s.pinned_installs += v.stats.pinned_installs;
            s.flights_refused += v.stats.flights_refused;
            s.deferred_merges += v.stats.deferred_merges;
            s.merge_rows_visited += v.stats.merge_rows_visited;
            s.merge_epochs_merged += v.stats.merge_epochs_merged;
            s.merges_refused_epochs += v.stats.merges_refused_epochs;
            s.merges_refused_rows += v.stats.merges_refused_rows;
            s.merges_refused_unavailable += v.stats.merges_refused_unavailable;
            s.waiters_refused += v.stats.waiters_refused;
            s.joins_declined_by_caller += v.stats.joins_declined_by_caller;
            s.gap_at_begin_total += v.stats.gap_at_begin_total;
            s.gap_at_finish_total += v.stats.gap_at_finish_total;
            s.gap_begin_samples += v.stats.gap_begin_samples;
            s.gap_finish_samples += v.stats.gap_finish_samples;
            s.completion_locks += v.stats.completion_locks;
            s.gap_at_finish_max = s.gap_at_finish_max.max(v.stats.gap_at_finish_max);
            s.flights_behind_at_begin += v.stats.flights_behind_at_begin;
            s.flights_that_fell_behind += v.stats.flights_that_fell_behind;
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use niles_ir::circuit::internal_contract;
    use niles_ir::operator::Scalar;
    use niles_ir::{Lineage, ServeContract};
    /// A base whose whole history is in a vector. Deliberately unoptimised: it is a
    /// definition of the right answer, not an implementation of a fast one.
    ///
    /// **`BTreeMap`, in a test.** These two maps are never iterated, so a hash map would give
    /// the same answers — and they are ordered anyway, because the reference base a
    /// determinism test is checked against must not be the one thing in the experiment whose
    /// order depends on a hash seed. GC-12's rule is "no `HashMap` in this file", and a rule
    /// with an exception for the parts a reader is least likely to check is not a rule.
    #[derive(Default)]
    pub(super) struct FoldBase {
        /// (epoch, key, delta)
        rows: Vec<(Epoch, Key, Value)>,
        head: Epoch,
        /// Per-key running balances every `interval` rows, which is the mechanism SC7 is
        /// about: reconstruction folds only the suffix after the newest checkpoint.
        interval: usize,
        checkpoints: BTreeMap<Key, Vec<(Epoch, Value)>>,
        counts: BTreeMap<Key, usize>,
    }

    impl FoldBase {
        pub(super) fn new(interval: usize) -> Self {
            FoldBase {
                interval,
                ..Default::default()
            }
        }
        pub(super) fn seal(&mut self, key: Key, delta: Value) -> Epoch {
            self.head += 1;
            self.rows.push((self.head, key.clone(), delta));
            let c = self.counts.entry(key.clone()).or_insert(0);
            *c += 1;
            if self.interval > 0 && (*c).is_multiple_of(self.interval) {
                let running: Value = self
                    .rows
                    .iter()
                    .filter(|(_, k, _)| *k == key)
                    .map(|(_, _, d)| d)
                    .sum();
                self.checkpoints
                    .entry(key)
                    .or_default()
                    .push((self.head, running));
            }
            self.head
        }
    }

    impl Base for FoldBase {
        fn frontier(&self) -> Epoch {
            self.head
        }
        fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
            let (mut acc, mut from) = (0i128, 0u64);
            if let Some(cps) = self.checkpoints.get(key) {
                if let Some((e, v)) = cps.iter().rev().find(|(e, _)| *e <= anchor) {
                    acc = *v;
                    from = *e;
                }
            }
            let mut rows = 0u64;
            for (e, k, d) in &self.rows {
                if *e > from && *e <= anchor && k == key {
                    acc += d;
                    rows += 1;
                }
            }
            (acc, rows)
        }
        fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
            self.rows
                .iter()
                .filter(|(re, _, _)| *re == e)
                .map(|(_, k, d)| (k.clone(), *d))
                .collect()
        }
    }

    pub(super) fn circuit(mat: Materialize, rung: Consistency) -> Circuit {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "postings".into(),
                is_base: true,
                anchor_key: vec![0],
                confidential: Vec::new(),
            },
            vec![],
            internal_contract(),
            "postings",
        );
        let agg = c.add(
            Op::Aggregate {
                group_key: vec![0],
                aggs: vec![(Agg::Sum, Scalar::Column(1))],
            },
            vec![src],
            ServeContract {
                consistency: rung,
                materialize: mat,
                retain: Retention::Evictable,
                lineage: Lineage::Key,
            },
            "balance",
        );
        c.set_output("balance", agg);
        c
    }

    /// **The budget must bound the metadata, not only the values — T-05.1.**
    ///
    /// The eviction budget bounded `slots`' *resident* count and nothing else. Two per-key
    /// maps beside it — a read count and a clock — took an entry for every key ever read and
    /// released none, so a view's memory was linear in history where the design promises it
    /// is linear in the budget. Every answer was correct throughout, which is why four audit
    /// cycles went past it: the defect is in the shape, not in the output.
    ///
    /// Read at twice the budget, so eviction is continuous, and assert the three things that
    /// together say the bound is real: residency is capped, the metadata is capped with it,
    /// and — the one a careless fix would break — every key ever read still has a *slot*,
    /// because an evicted entry is `Hole(e)` and forgetting it would turn honest absence into
    /// `Bottom` and a miss into a zero.
    #[test]
    fn metadata_is_bounded_by_the_budget() {
        const BUDGET: u64 = 8;
        const KEYS: i64 = 16;
        let mut base = FoldBase::new(0);
        for k in 0..KEYS {
            base.seal(vec![k], 100 + k as i128);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(BUDGET),
            Policy::Lru,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();
        for k in 0..KEYS {
            v.read(&base, &vec![k], anchor);
        }

        assert!(
            v.stats.evictions > 0,
            "the budget must bite or this test asserts nothing: {:?}",
            v.stats
        );
        assert!(
            v.resident_count() <= BUDGET,
            "{} entries resident against a budget of {BUDGET}",
            v.resident_count()
        );
        assert!(
            v.metadata_len() as u64 <= BUDGET,
            "the policy metadata holds {} keys against a budget of {BUDGET}: it is bounded by \
             history, not by the budget, and a long-lived view does not fit",
            v.metadata_len()
        );
        // Honest absence survives the bound: the value went, the version stayed.
        for k in 0..KEYS {
            assert!(
                v.slot(&vec![k]).version().is_some(),
                "key {k} has no version after eviction: absence of value became absence of \
                 history, which is how a miss becomes a zero"
            );
        }
    }

    /// A key that comes back is a key the metadata describes again.
    ///
    /// The other half of the bound: dropping metadata on eviction must not make a returning
    /// key unrankable. It is re-created on the next read, at one, which is what the doc on
    /// `meta` says it gives up.
    #[test]
    fn an_evicted_key_read_again_is_ranked_again() {
        let mut base = FoldBase::new(0);
        for k in 0..4i64 {
            base.seal(vec![k], 10);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(1),
            Policy::Lru,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();
        for k in 0..4i64 {
            v.read(&base, &vec![k], anchor);
        }
        assert_eq!(
            v.metadata_len(),
            1,
            "one resident entry, one metadata entry"
        );
        let first = v.read(&base, &vec![0], anchor).value;
        assert_eq!(v.metadata_len(), 1);
        assert_eq!(
            first,
            v.read(&base, &vec![0], anchor).value,
            "the value must not depend on whether its metadata survived"
        );
    }

    #[test]
    fn a_read_after_eviction_returns_the_same_value() {
        // Reconstruction equivalence, on the running engine: the central corollary of
        // Contribution 1, which is that memory pressure can never change an answer.
        let mut base = FoldBase::new(0);
        for i in 0..50 {
            base.seal(vec![i % 5], 100 + i as i128);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(2),
            Policy::Lru,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();

        let mut first = BTreeMap::new();
        for k in 0..5i64 {
            first.insert(k, v.read(&base, &vec![k], anchor).value);
        }
        // With a budget of 2 and 5 keys, the sweep above already evicted three of them.
        assert!(
            v.stats.evictions >= 3,
            "the budget must actually bite: {:?}",
            v.stats
        );
        for k in 0..5i64 {
            let again = v.read(&base, &vec![k], anchor).value;
            assert_eq!(again, first[&k], "key {k} changed across an eviction");
        }
    }

    /// **The lower bound of the certification interval, which is the whole of its safety.**
    ///
    /// `read` serves a resident entry at the anchor it was asked for when
    /// `stamp <= anchor <= effective`. The upper bound is the saving; the *lower* bound is
    /// what keeps it honest, and a version of this change without it would pass every
    /// fallback-rate measurement while serving a value from the future at a historical
    /// anchor. Here the key receives a delta strictly after the anchor being read, so the
    /// resident entry is stamped above it and the read must go back to the base.
    #[test]
    fn a_key_with_a_delta_after_the_anchor_still_reconstructs() {
        let mut base = FoldBase::new(0);
        base.seal(vec![7], 100);
        let early = base.frontier();
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        let v = rt.view_mut("balance").unwrap();

        // Make the entry resident and current, then move the key underneath it.
        base.seal(vec![7], 400);
        let late = base.frontier();
        assert_eq!(v.read(&base, &vec![7], late).value, 500);
        assert_eq!(v.stats.misses, 1, "the first read reconstructs");

        // A read at the earlier anchor must not see the 400.
        let before = v.stats.misses;
        let answered = v.read(&base, &vec![7], early);
        assert_eq!(
            answered.value, 100,
            "a read below the entry's stamp must reconstruct at its own anchor, not serve              the later value"
        );
        assert_eq!(
            answered.anchor, early,
            "and must report the anchor it was asked for"
        );
        assert_eq!(
            v.stats.misses,
            before + 1,
            "it must have reconstructed rather than been counted as a hit — an answer the              caller then discards is not a hit, and counting it as one is what made `hits` a              count of answers rather than of answers served"
        );
    }

    /// A pinned entry is one installed at an anchor *below* the frontier the view had
    /// applied, so it never saw the deltas in between and may not inherit `applied`. Its
    /// `effective` is its own stamp, and a read above that stamp must reconstruct — the
    /// certification interval collapses to a point and the new upper bound must not widen it.
    ///
    /// **Merging is switched off here, and that is the finding rather than a workaround.**
    /// This test went red when the deferred merge landed, because on this fixture the late
    /// landing is two epochs behind and now merges: the entry installs current, and the read
    /// below is served from it instead of rebuilding. The value was right either way — the
    /// merge is not a correctness change — but the fixture was written when pinning was the
    /// only landing, so as written it asserts the *policy* and not the property it is named
    /// for. The property is about what a pinned entry may answer, so the pin is made
    /// explicit; `a_merged_entry_answers_the_frontier_it_landed_at` is the same fixture with
    /// merging on, and the two together say what changed.
    #[test]
    fn a_pinned_entry_read_above_its_stamp_still_reconstructs() {
        let mut base = FoldBase::new(0);
        base.seal(vec![7], 100);
        let early = base.frontier();
        base.seal(vec![7], 400);
        base.seal(vec![9], 1);
        let late = base.frontier();

        let mut rt = Runtime::install(
            circuit(Materialize::Full, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        rt.set_merge_caps(MergeCaps::OFF);
        // Drag the view forward, then read historically: `install` pins the entry because it
        // is anchored below `applied`. `advance` is the runtime's, so it is called before the
        // view is borrowed out of it.
        rt.advance(&base, late);
        let v = rt.view_mut("balance").unwrap();
        assert_eq!(v.read(&base, &vec![7], early).value, 100);
        assert_eq!(v.stats.pinned_installs, 1, "the fixture must produce a pin");

        let before = v.stats.misses;
        let answered = v.read(&base, &vec![7], late);
        assert_eq!(
            answered.value, 500,
            "a pinned entry has not seen the deltas between its anchor and `applied`, so a              read above its stamp must rebuild"
        );
        assert_eq!(
            v.stats.misses,
            before + 1,
            "and must do so by reconstructing, not by inheriting a frontier it never saw"
        );
    }

    /// **The same fixture, with merging on: the entry lands current and the next read hits.**
    ///
    /// This is the whole of T04's benefit on the smallest case that shows it, and it is worth
    /// stating beside the pin rather than only in the merge module: the two tests differ in
    /// one value, both answer 100 at `early` and 500 at `late`, and the difference between
    /// them is a reconstruction that does not happen.
    #[test]
    fn a_merged_entry_answers_the_frontier_it_landed_at() {
        let mut base = FoldBase::new(0);
        base.seal(vec![7], 100);
        let early = base.frontier();
        base.seal(vec![7], 400);
        base.seal(vec![9], 1);
        let late = base.frontier();

        let mut rt = Runtime::install(
            circuit(Materialize::Full, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        rt.advance(&base, late);
        let v = rt.view_mut("balance").unwrap();
        assert_eq!(
            v.read(&base, &vec![7], early).value,
            100,
            "the historical reader is still answered at its own anchor, with its own value"
        );
        assert_eq!(v.stats.deferred_merges, 1);
        assert_eq!(v.stats.pinned_installs, 0);

        let before = v.stats.misses;
        let answered = v.read(&base, &vec![7], late);
        assert_eq!(
            answered.value, 500,
            "the merged entry is the value at the frontier"
        );
        assert_eq!(answered.anchor, late);
        assert_eq!(
            v.stats.misses, before,
            "and it is served from the entry: the reconstruction the pinned arm pays is the \
             one this arm saves"
        );
    }

    /// The upper bound, stated on its own: an entry that received no delta since its stamp
    /// answers every anchor between the two, and answers *at* the anchor asked for. This is
    /// the claim the fallback rate rests on, and it is worth one test that does not involve
    /// threads.
    #[test]
    fn an_entry_answers_every_anchor_in_its_certification_interval() {
        let mut base = FoldBase::new(0);
        base.seal(vec![7], 100);
        let stamp = base.frontier();
        let mut rt = Runtime::install(
            circuit(Materialize::Full, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        {
            let v = rt.view_mut("balance").unwrap();
            assert_eq!(v.read(&base, &vec![7], stamp).value, 100);
        }

        // Epochs pass in which this key receives nothing.
        for i in 0..5 {
            base.seal(vec![8], i);
        }
        rt.advance(&base, base.frontier());
        let v = rt.view_mut("balance").unwrap();

        let hits = v.stats.hits;
        for a in stamp..=base.frontier() {
            let answered = v.read(&base, &vec![7], a);
            assert_eq!(answered.value, 100, "the key did not change at epoch {a}");
            assert_eq!(
                answered.anchor, a,
                "and the answer must be stamped with the anchor it was asked for, or the                  caller cannot tell it apart from one that is merely fresher"
            );
        }
        assert_eq!(
            v.stats.hits - hits,
            base.frontier() - stamp + 1,
            "every anchor in the interval must be a hit; a reconstruction here is the 24.3%              fallback in miniature"
        );
    }

    /// **The postcondition, over every shape that reaches `read`.**
    ///
    /// A hit, a miss, an eviction, a pinned historical entry, a key never seen: whatever
    /// happens inside, the answer is stamped with the anchor that was asked for. This is the
    /// property that lets a caller use the value without checking, and its absence is what
    /// made every caller in this project write the same compensating branch.
    #[test]
    fn every_answer_is_stamped_with_the_anchor_it_was_asked_for() {
        let mut base = FoldBase::new(0);
        for i in 0..40 {
            base.seal(vec![i % 6], 10 + i as i128);
        }
        let mut rt = Runtime::install(
            // A budget below the key count, so eviction bites; each key is read twice in a row,
            // so the second read is a hit on the entry the first installed. Without the repeat the
            // budget evicts every entry before it is asked again and the sweep exercises the miss
            // path alone — asserting the postcondition over one branch while looking thorough.
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(4),
            Policy::Lru,
        )
        .unwrap();
        let head = base.frontier();

        // Interleave maintenance with reads at every anchor, so entries are variously fresh,
        // stale, pinned and evicted when they are asked.
        for a in 0..=head {
            if a.is_multiple_of(3) {
                rt.advance(&base, a);
            }
            let v = rt.view_mut("balance").unwrap();
            for k in 0..7i64 {
                // Key 6 is never sealed: the unknown-key path reaches `reconstruct` too.
                for _ in 0..2 {
                    let answered = v.read(&base, &vec![k], a);
                    assert_eq!(
                        answered.anchor, a,
                        "key {k} at anchor {a} came back stamped {} — a caller cannot use an                      answer about a moment it did not ask about, and every one of them ends                      up writing the same branch to throw it away",
                        answered.anchor
                    );
                }
            }
        }
        let v = rt.view_mut("balance").unwrap();
        assert!(
            v.stats.hits > 0 && v.stats.misses > 0 && v.stats.evictions > 0,
            "the sweep must actually have exercised hits, misses and evictions, or it is              asserting the postcondition over one path: {:?}",
            v.stats
        );
    }

    #[test]
    fn an_evicted_entry_keeps_its_version_and_never_answers_zero() {
        let mut base = FoldBase::new(0);
        base.seal(vec![7], 500);
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(1),
            Policy::Lru,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();
        assert_eq!(v.read(&base, &vec![7], anchor).value, 500);
        // Force it out.
        base.seal(vec![8], 900);
        let head = base.frontier();
        v.read(&base, &vec![8], head);
        assert!(
            matches!(v.slot(&vec![7]), Slot::Hole(_)),
            "must be a hole, not gone: {}",
            v.slot(&vec![7])
        );
        // And reading it back reconstructs the real value, not the aggregate's identity.
        let head = base.frontier();
        assert_eq!(v.read(&base, &vec![7], head).value, 500);
    }

    #[test]
    fn every_answer_carries_its_anchor() {
        let mut base = FoldBase::new(0);
        base.seal(vec![1], 10);
        base.seal(vec![1], 20);
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        let v = rt.view_mut("balance").unwrap();
        let a = v.read(&base, &vec![1], 1);
        let b = v.read(&base, &vec![1], 2);
        assert_eq!(
            (a.value, a.anchor),
            (10, 1),
            "an as-of read sees the prefix, not the head"
        );
        assert_eq!((b.value, b.anchor), (30, 2));
    }

    #[test]
    fn deltas_to_non_resident_entries_are_skipped_and_that_is_the_saving() {
        let mut base = FoldBase::new(0);
        for i in 0..20 {
            base.seal(vec![i % 10], 1);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(2),
            Policy::Lru,
        )
        .unwrap();
        {
            let v = rt.view_mut("balance").unwrap();
            v.read(&base, &vec![0], 1);
        }
        for e in 1..=20 {
            rt.advance(&base, e);
        }
        let s = rt.stats();
        assert!(
            s.deltas_skipped > s.deltas_applied,
            "partiality must skip more than it applies: {s:?}"
        );
    }

    #[test]
    fn a_full_view_ignores_the_budget() {
        // The contract overrides the caller: `materialize: full` is a promise the runtime
        // keeps even when someone hands it a budget.
        let mut base = FoldBase::new(0);
        for i in 0..10 {
            base.seal(vec![i], 1);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Full, Consistency::Snapshot),
            Some(1),
            Policy::Lru,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();
        for k in 0..10i64 {
            v.read(&base, &vec![k], anchor);
        }
        assert_eq!(
            v.stats.evictions, 0,
            "a fully materialized view must not evict"
        );
        assert_eq!(v.resident_count(), 10);
    }

    #[test]
    fn a_lax_rung_batches_maintenance_and_applies_exactly_the_same_deltas() {
        // What a consistency rung actually costs, once maintenance stops dropping the
        // deltas it batched over.
        //
        // The earlier version of this test asserted that a strict rung applies more than
        // four times the deltas of a bounded one, and it passed — because `advance` was
        // folding only the boundary epoch and discarding the `stride - 1` epochs before
        // it. Those deltas were never applied to anyone, and the view was nevertheless
        // certified through the boundary. The "66x rung tax" was the count of deltas
        // thrown away.
        //
        // Corrected, the saving is real but it is a *batching* saving: the same deltas,
        // in fewer passes. That is what a bounded-staleness contract buys — you may drag
        // the view to the frontier less often — and it is a materially weaker claim than
        // the one the thesis reported.
        let mut runs = Vec::new();
        for rung in [
            Consistency::Bounded {
                epochs: 8,
                millis: 0,
            },
            Consistency::LedgerConsistent,
        ] {
            let mut base = FoldBase::new(0);
            for i in 0..200 {
                base.seal(vec![i % 4], 1);
            }
            let mut rt =
                Runtime::install(circuit(Materialize::Demand, rung), None, Policy::Lru).unwrap();
            {
                let v = rt.view_mut("balance").unwrap();
                for k in 0..4i64 {
                    v.read(&base, &vec![k], 1);
                }
            }
            for e in 1..=200 {
                rt.advance(&base, e);
            }
            runs.push(rt.stats());
        }
        let (lax, strict) = (runs[0], runs[1]);
        assert!(
            strict.maintenance_passes > lax.maintenance_passes * 4,
            "the strict rung must be dragged to the frontier materially more often: \
             {lax:?} vs {strict:?}"
        );
        assert_eq!(
            lax.deltas_applied, strict.deltas_applied,
            "and it must apply exactly the same deltas: a rung that applied fewer would \
             be serving a value the ledger does not have"
        );
        assert_eq!(
            lax.reads, strict.reads,
            "and the read counts must be indistinguishable"
        );
    }

    // ── the certification invariant: the three defects, pinned ──────────────────────

    /// An independent fold. Deliberately not the engine's own `reconstruct`: comparing a
    /// value against the function that produced it proves nothing.
    fn truth(base: &FoldBase, key: &Key, anchor: Epoch) -> Value {
        base.rows
            .iter()
            .filter(|(e, k, _)| *e <= anchor && k == key)
            .map(|(_, _, d)| d)
            .sum()
    }

    #[test]
    fn f01_stale_hit_is_not_promoted() {
        // A historical read installs an entry anchored below the applied frontier. It is
        // a correct answer *at that anchor* and must never be served for a later one:
        // promoting it to `applied` returns a value the ledger does not have, as a hit.
        let mut base = FoldBase::new(0);
        base.seal(vec![1], 10);
        base.seal(vec![1], 20);
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        rt.advance(&base, 1);
        rt.advance(&base, 2);
        let v = rt.view_mut("balance").unwrap();

        let historical = v.read(&base, &vec![1], 1);
        assert_eq!(historical.value, truth(&base, &vec![1], 1));
        assert_eq!(historical.anchor, 1);

        let current = v.read(&base, &vec![1], 2);
        assert_eq!(
            current.value,
            truth(&base, &vec![1], 2),
            "a read at the head must not be answered from an entry anchored below it"
        );
        assert_eq!(current.anchor, 2);
    }

    #[test]
    fn f02_delta_is_applied_once() {
        // An entry reconstructed *ahead* of the applied frontier already carries the
        // deltas of the epochs the view has not yet folded. Folding them in again is the
        // double-application anomaly the thesis says is dissolved by construction.
        let mut base = FoldBase::new(0);
        base.seal(vec![1], 10);
        base.seal(vec![1], 20);
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        {
            let v = rt.view_mut("balance").unwrap();
            let ahead = v.read(&base, &vec![1], 2);
            assert_eq!(ahead.value, truth(&base, &vec![1], 2));
        }
        rt.advance(&base, 1);
        rt.advance(&base, 2);
        let v = rt.view_mut("balance").unwrap();
        let after = v.read(&base, &vec![1], 2);
        assert_eq!(
            after.value,
            truth(&base, &vec![1], 2),
            "the deltas were already in the value; applying them again doubles the money"
        );
    }

    #[test]
    fn f03_bounded_rung_applies_every_delta() {
        // A bounded rung batches maintenance. Batching is not discarding: every epoch in
        // the window still has to be folded, or the view is certified through a frontier
        // whose deltas nobody applied.
        let mut base = FoldBase::new(0);
        for _ in 0..16 {
            base.seal(vec![1], 1);
        }
        let mut rt = Runtime::install(
            circuit(
                Materialize::Demand,
                Consistency::Bounded {
                    epochs: 8,
                    millis: 0,
                },
            ),
            None,
            Policy::Lru,
        )
        .unwrap();
        {
            let v = rt.view_mut("balance").unwrap();
            v.read(&base, &vec![1], 0);
        }
        for e in 1..=16 {
            rt.advance(&base, e);
        }
        let v = rt.view_mut("balance").unwrap();
        let got = v.read(&base, &vec![1], 16);
        assert_eq!(
            got.value,
            truth(&base, &vec![1], 16),
            "16 epochs of +1 is 16, not the 2 a stride that folds only its boundary gives"
        );
    }

    #[test]
    fn cert_holds_under_interleaved_reads_evictions_and_batched_maintenance() {
        // The certification invariant as a property: every value served as a hit equals
        // an independent fold at the anchor it was served with, under every rung, with
        // reads at arbitrary anchors interleaved with eviction and batched maintenance.
        for rung in [
            Consistency::Bounded {
                epochs: 8,
                millis: 0,
            },
            Consistency::Snapshot,
            Consistency::LedgerConsistent,
        ] {
            for seed in [1u64, 7, 42, 100, 2024] {
                let mut lcg = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let mut next = || {
                    lcg = lcg
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    (lcg >> 33) as i64
                };
                let mut base = FoldBase::new(0);
                for i in 0..120 {
                    base.seal(vec![i % 6], ((i % 5) - 2) as i128);
                }
                // A budget that binds: six keys, three slots, so eviction is continuous.
                let mut rt =
                    Runtime::install(circuit(Materialize::Demand, rung), Some(3), Policy::Lru)
                        .unwrap();
                let mut checked = 0u64;
                for e in 1..=120u64 {
                    rt.advance(&base, e);
                    let v = rt.view_mut("balance").unwrap();
                    for _ in 0..3 {
                        let k = vec![next().rem_euclid(6)];
                        // Anchors anywhere in the retained history, not only at the head.
                        let anchor = (next().rem_euclid(e as i64 + 1)) as Epoch;
                        let got = v.read(&base, &k, anchor);
                        assert!(
                            got.anchor >= anchor,
                            "an answer must not be older than the anchor it was asked for"
                        );
                        assert_eq!(
                            got.value,
                            truth(&base, &k, got.anchor),
                            "rung {rung:?} seed {seed}: served value disagrees with the \
                             fold at the anchor it was served with"
                        );
                        checked += 1;
                    }
                }
                assert!(checked >= 300);
            }
        }
    }

    #[test]
    fn the_five_partial_state_anomalies_are_each_absent() {
        // Thesis section 4.2 tabulates five anomalies that partial-state dataflow over a
        // mutable base must exclude by protocol, and argues each is dissolved here by
        // construction. A table is not a test; this is.
        let mut base = FoldBase::new(0);
        for _ in 0..4 {
            base.seal(vec![1], 5);
        }
        let mk = || {
            Runtime::install(
                circuit(Materialize::Demand, Consistency::Snapshot),
                Some(1),
                Policy::Lru,
            )
            .unwrap()
        };

        // 1. Double application — a delta already in the value is not folded again.
        {
            let mut rt = mk();
            {
                let v = rt.view_mut("balance").unwrap();
                v.read(&base, &vec![1], 4);
            }
            for e in 1..=4 {
                rt.advance(&base, e);
            }
            let v = rt.view_mut("balance").unwrap();
            assert_eq!(v.read(&base, &vec![1], 4).value, 20);
        }
        // 2. Skipped deltas — reconstruction is anchored, so a delta at or below the
        //    anchor is included whether or not the resident state ever saw it.
        {
            let mut rt = mk();
            for e in 1..=4 {
                rt.advance(&base, e);
            }
            let v = rt.view_mut("balance").unwrap();
            assert_eq!(v.read(&base, &vec![1], 4).value, 20);
        }
        // 3. Upquery races — two reconstructions at different anchors are mutually
        //    consistent: each answer is exact at the anchor it is *returned* with, and
        //    the higher anchor wins by monotone anchoring.
        //
        //    Note what this does and does not say. `read(key, anchor)` means "at least as
        //    fresh as `anchor`", so once the entry sits at epoch 4 a request for epoch 2
        //    is answered at 4 — correctly, and with 4 in the anchor. That is a hit rule
        //    for a freshness contract, and it is the wrong rule for an as-of read, which
        //    is why `nilestream-server::rev_engine` reconstructs historical reads instead
        //    of consulting the view. The distinction is a seam, and it is recorded here
        //    rather than hidden behind an assertion that reads as if it were not.
        {
            let mut rt = mk();
            let v = rt.view_mut("balance").unwrap();
            let lo = v.read(&base, &vec![1], 2);
            let hi = v.read(&base, &vec![1], 4);
            assert_eq!((lo.anchor, lo.value), (2, 10));
            assert_eq!((hi.anchor, hi.value), (4, 20));
            let again = v.read(&base, &vec![1], 2);
            assert!(again.anchor >= 2);
            assert_eq!(
                again.value,
                truth(&base, &vec![1], again.anchor),
                "every answer is exact at the anchor it carries"
            );
        }
        // 4. Lost deltas — a delta addressed to an evicted key is discarded with no
        //    downstream notice, and the next read still answers correctly.
        {
            let mut rt = mk();
            {
                let v = rt.view_mut("balance").unwrap();
                v.read(&base, &vec![1], 0);
                v.read(&base, &vec![2], 0); // evicts key 1: the budget is one slot
            }
            for e in 1..=4 {
                rt.advance(&base, e);
            }
            let v = rt.view_mut("balance").unwrap();
            assert!(v.stats.deltas_skipped > 0, "the saving must be real");
            assert_eq!(v.read(&base, &vec![1], 4).value, 20);
        }
        // 5. Upquery deadlock — reconstruction is a pull over an immutable prefix and
        //    never waits on the update path, so a read during maintenance terminates.
        {
            let mut rt = mk();
            for e in 1..=4 {
                rt.advance(&base, e);
                let v = rt.view_mut("balance").unwrap();
                v.read(&base, &vec![1], e);
            }
        }
    }

    #[test]
    fn eviction_is_reproducible_from_the_seed_for_every_policy() {
        // The thesis states that a seed is a reproducibility guarantee rather than a
        // hope. It was not: victims were chosen by iterating a hash map, so three runs
        // of one seed gave three different resident sets.
        for policy in [Policy::Lru, Policy::Random, Policy::CostAware] {
            let run = || {
                let mut base = FoldBase::new(0);
                for i in 0..80 {
                    base.seal(vec![i % 8], 1);
                }
                let mut rt = Runtime::install(
                    circuit(Materialize::Demand, Consistency::Snapshot),
                    Some(3),
                    policy,
                )
                .unwrap();
                for e in 1..=80u64 {
                    rt.advance(&base, e);
                    let v = rt.view_mut("balance").unwrap();
                    v.read(&base, &vec![(e as i64 * 7) % 8], e);
                }
                let v = rt.view("balance").unwrap();
                let resident: Vec<Key> = v
                    .slots
                    .iter()
                    .filter(|(_, s)| s.is_resident())
                    .map(|(k, _)| k.clone())
                    .collect();
                (rt.stats(), resident)
            };
            let a = run();
            let b = run();
            let c = run();
            assert_eq!(a, b, "{policy:?} is not reproducible");
            assert_eq!(b, c, "{policy:?} is not reproducible");
        }
    }

    #[test]
    fn checkpoints_bound_reconstruction_by_the_interval_not_by_history() {
        // SC7, the Bounded Reconstruction Theorem, on the running engine. Without
        // checkpoints the fold grows with history; with them it does not.
        let mut costs = Vec::new();
        for interval in [0usize, 16] {
            let mut base = FoldBase::new(interval);
            for i in 0..2000 {
                base.seal(vec![i % 4], 1);
            }
            let mut rt = Runtime::install(
                circuit(Materialize::Demand, Consistency::Snapshot),
                Some(1),
                Policy::Lru,
            )
            .unwrap();
            let anchor = base.frontier();
            let v = rt.view_mut("balance").unwrap();
            for _ in 0..8 {
                for k in 0..4i64 {
                    v.read(&base, &vec![k], anchor);
                }
            }
            costs.push(v.stats.base_rows_read as f64 / v.stats.upqueries as f64);
        }
        let (unbounded, bounded) = (costs[0], costs[1]);
        assert!(
            unbounded > 400.0,
            "without checkpoints the fold is history-length: {unbounded}"
        );
        // Predicted bound: C/2 + 1 on average, so 9 for C = 16. Allow the interval itself
        // as slack, since the last checkpoint's position within the run varies.
        assert!(
            bounded <= 16.0,
            "with C=16 the fold must be bounded near C/2+1 = 9, measured {bounded}"
        );
    }

    /// **The guard corpus: every unsupported reachable graph, refused at the public door.**
    ///
    /// A10-06. `install` inspected `circuit.outputs` and stopped, so a keyed `sum` was
    /// enough to install *whatever fed it*. That is not a near-miss. After `install`
    /// returns, nothing reads the circuit again — `Base::reconstruct` is handed a key and an
    /// epoch — so an intervening `Filter` is not approximated or partially applied, it is
    /// dropped, and the view answers the question the circuit does **not** ask. The failure
    /// is a well-formed correct-looking number, which is why it survived four audit cycles:
    /// every answer-level assertion in the tree passes, because the answer is exactly what
    /// the graph-without-the-filter should produce.
    ///
    /// Each entry below is a graph that lowers and verifies. `Join` and `Fixpoint` are the
    /// cases Theorem 4.1's fragment excludes for a stated reason; `Filter`, `Map` and
    /// `Distinct` are the ones that look harmless and are not.
    #[test]
    fn every_unsupported_reachable_graph_is_refused_at_install() {
        use niles_ir::operator::{JoinKind, ScalarOp};

        fn src(c: &mut Circuit, relation: &str, is_base: bool) -> NodeId {
            c.add(
                Op::Source {
                    relation: relation.into(),
                    is_base,
                    anchor_key: vec![0],
                    confidential: Vec::new(),
                },
                vec![],
                internal_contract(),
                relation,
            )
        }
        fn agg_over(c: &mut Circuit, input: NodeId) -> NodeId {
            c.add(
                Op::Aggregate {
                    group_key: vec![0],
                    aggs: vec![(Agg::Sum, Scalar::Column(1))],
                },
                vec![input],
                internal_contract(),
                "balance",
            )
        }

        // Each case builds a circuit and names the op the refusal must point at: what the
        // graph is, the `Op::name()` the refusal must carry, and how to build it.
        type Case = (&'static str, &'static str, Box<dyn Fn() -> Circuit>);
        let cases: Vec<Case> = vec![
            (
                "a filter between the source and the aggregate",
                "filter",
                Box::new(|| {
                    let mut c = Circuit::new();
                    let s = src(&mut c, "postings", true);
                    let f = c.add(
                        Op::Filter {
                            predicate: Scalar::Binary {
                                op: ScalarOp::Eq,
                                lhs: Box::new(Scalar::Column(0)),
                                rhs: Box::new(Scalar::LitInt(4242)),
                            },
                        },
                        vec![s],
                        internal_contract(),
                        "where",
                    );
                    let a = agg_over(&mut c, f);
                    c.set_output("balance", a);
                    c
                }),
            ),
            (
                "a map between the source and the aggregate",
                "map",
                Box::new(|| {
                    let mut c = Circuit::new();
                    let s = src(&mut c, "postings", true);
                    let m = c.add(
                        Op::Map {
                            exprs: vec![
                                Scalar::Column(0),
                                Scalar::Neg(Box::new(Scalar::Column(1))),
                            ],
                        },
                        vec![s],
                        internal_contract(),
                        "map",
                    );
                    let a = agg_over(&mut c, m);
                    c.set_output("balance", a);
                    c
                }),
            ),
            (
                "a join under the aggregate",
                "join",
                Box::new(|| {
                    let mut c = Circuit::new();
                    let l = src(&mut c, "postings", true);
                    let r = src(&mut c, "accounts", true);
                    let j = c.add(
                        Op::Join {
                            kind: JoinKind::Inner,
                            left_key: vec![0],
                            right_key: vec![0],
                            residual: None,
                        },
                        vec![l, r],
                        internal_contract(),
                        "join",
                    );
                    let a = agg_over(&mut c, j);
                    c.set_output("balance", a);
                    c
                }),
            ),
            (
                "a distinct under the aggregate",
                "distinct",
                Box::new(|| {
                    let mut c = Circuit::new();
                    let s = src(&mut c, "postings", true);
                    let dd = c.add(Op::Distinct, vec![s], internal_contract(), "distinct");
                    let a = agg_over(&mut c, dd);
                    c.set_output("balance", a);
                    c
                }),
            ),
            (
                "a guarded fixpoint under the aggregate",
                "fixpoint",
                Box::new(|| {
                    let mut c = Circuit::new();
                    let s = src(&mut c, "postings", true);
                    let fx = c.add(
                        Op::Fixpoint {
                            measure: Scalar::Column(1),
                            max_rounds: 4,
                        },
                        vec![s],
                        internal_contract(),
                        "fix",
                    );
                    let a = agg_over(&mut c, fx);
                    c.set_output("balance", a);
                    c
                }),
            ),
        ];

        for (what, op, build) in cases {
            match Runtime::install(build(), None, Policy::Lru) {
                Ok(_) => panic!(
                    "installed a circuit with {what}. Nothing below `install` reads the \
                     circuit again, so that stage would be silently dropped and the view \
                     would answer a question it was not asked"
                ),
                Err(Unsupported::Upstream { op: got, .. }) => assert_eq!(
                    got, op,
                    "the refusal for {what} must name the node it refused"
                ),
                Err(other) => panic!("{what}: refused for the wrong reason: {other:?}"),
            }
        }

        // A derived source is refused with its own reason: not "a stage was dropped" but
        // "this runtime does not hold the thing you are reading".
        let mut c = Circuit::new();
        let s = src(&mut c, "some_view", false);
        let a = agg_over(&mut c, s);
        c.set_output("balance", a);
        match Runtime::install(c, None, Policy::Lru) {
            Err(Unsupported::DerivedSource { relation, .. }) => assert_eq!(relation, "some_view"),
            Err(other) => panic!("a derived source must be refused by name: {other:?}"),
            Ok(_) => panic!("a derived source installed: nothing here can fold a view's rows"),
        }
    }

    /// The other half of T02.1, and the half that makes the refusals worth having: the
    /// supported shape still installs, and still agrees with a fold that shares no state with
    /// it — **after** an advance and after eviction, which is where a partial view could
    /// diverge without any refusal noticing.
    #[test]
    fn a_supported_keyed_balance_still_agrees_with_an_independent_fold_after_advance_and_eviction()
    {
        const BUDGET: u64 = 4;
        const KEYS: i64 = 12;

        let mut base = FoldBase::new(0);
        // An independent adder: the same rows, summed by something that is not the runtime.
        let mut expected: BTreeMap<Key, Value> = BTreeMap::new();
        for round in 0..3 {
            for k in 0..KEYS {
                let delta = 100 + k as i128 + round * 7;
                base.seal(vec![k], delta);
                *expected.entry(vec![k]).or_insert(0) += delta;
            }
        }

        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(BUDGET),
            Policy::Lru,
        )
        .expect("the keyed balance circuit is inside the fragment");
        let head = base.frontier();
        rt.advance(&base, head);

        // Read every key twice, in opposite orders. With a budget of 4 against 12 keys the
        // view evicts continuously, so most of these answers are reconstructions rather than
        // resident reads — which is the case the agreement has to hold in.
        let order: Vec<i64> = (0..KEYS).chain((0..KEYS).rev()).collect();
        for k in order {
            let got = rt
                .view_mut("balance")
                .expect("balance")
                .read(&base, &vec![k], head);
            assert_eq!(
                got.value,
                expected[&vec![k]],
                "key {k} disagreed with an independent fold at epoch {head}"
            );
        }

        let v = rt.view("balance").expect("balance");
        assert!(
            v.stats.evictions > 0,
            "the agreement was measured without ever evicting, so it says nothing about \
             reconstruction: budget {BUDGET} against {KEYS} keys"
        );
    }

    #[test]
    fn a_circuit_outside_the_fragment_is_rejected_not_mis_executed() {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "p".into(),
                is_base: true,
                anchor_key: vec![0],
                confidential: Vec::new(),
            },
            vec![],
            internal_contract(),
            "",
        );
        let f = c.add(
            Op::Filter {
                predicate: Scalar::LitBool(true),
            },
            vec![src],
            internal_contract(),
            "",
        );
        c.set_output("v", f);
        let e = match Runtime::install(c, None, Policy::Lru) {
            Err(e) => e,
            Ok(_) => panic!("a filter-terminated circuit must be rejected, not installed"),
        };
        assert!(
            matches!(e, Unsupported::Shape { op: "filter", .. }),
            "{e:?}"
        );
        assert!(e.explain().contains("outside the key-aggregate fragment"));
    }

    #[test]
    fn the_runtime_reads_every_checked_field_of_the_nodes_it_installs() {
        // The GoogleSQL discipline, enforced against this runtime: if `install` stopped
        // consulting the rung, the audit would say so.
        let c = circuit(Materialize::Demand, Consistency::Snapshot);
        let rt = Runtime::install(c, None, Policy::Lru).unwrap();
        let unread: Vec<_> = rt
            .circuit
            .audit_access()
            .unread
            .into_iter()
            .filter(|(n, _)| rt.views.iter().any(|v| v.node == *n))
            .collect();
        assert!(
            unread.is_empty(),
            "the runtime ignored a semantic field: {unread:?}"
        );
    }

    #[test]
    fn wiping_the_derived_layer_loses_nothing_the_base_does_not_have() {
        // Rebuild-from-base: the derived layer holds no information the base does not.
        let mut base = FoldBase::new(8);
        for i in 0..300 {
            base.seal(vec![i % 6], (i as i128 % 7) - 3);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(3),
            Policy::CostAware,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();
        let before: Vec<Value> = (0..6i64)
            .map(|k| v.read(&base, &vec![k], anchor).value)
            .collect();
        v.wipe();
        let after: Vec<Value> = (0..6i64)
            .map(|k| v.read(&base, &vec![k], anchor).value)
            .collect();
        assert_eq!(
            before, after,
            "a full rebuild from the base must reproduce every balance"
        );
    }
}

#[cfg(test)]
mod two_phase {
    //! **The latch-driven differential for the two-phase read.**
    //!
    //! The concurrent differential below runs threads and compares answers, which finds a
    //! wrong rule only on the interleavings the scheduler happens to produce. This one
    //! *constructs* the interleaving: an owner is stopped in the middle of its fold, an
    //! epoch is sealed and applied while it is stopped, and a same-anchor reader and a
    //! different-anchor reader are each taken to their own path before the owner is
    //! released. Every step is a latch, so the run is the same on a loaded container and an
    //! idle laptop, and the three reversions the repair is guarded by each turn it red.
    //!
    //! The precondition is asserted rather than assumed: a run in which no flight was
    //! joined, none landed pinned and none was deferred did not exercise the mechanism, and
    //! a pass under those conditions would be a pass for the old code.

    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::RwLock;

    impl Rev {
        /// Reap this key's flight as if its owner had been cancelled.
        ///
        /// It calls exactly the path a real cancellation takes: the point of a guard against
        /// a stale completion is that the supersession happen the way it happens in
        /// production, not that a test hand-build a second generation.
        ///
        /// **Defined here rather than in `impl Rev` above, and the reason is not style.**
        /// `thesis/gen-appendix-d.py` reads each crate's public surface by cutting the file
        /// at its first `#[cfg(test)]`; a test-only method in the middle of `impl Rev` moved
        /// that cut above `Unsupported` and `Runtime` and silently deleted both from the
        /// generated appendix. The generator's rule is fragile and this file must not be the
        /// thing that trips it.
        pub(in crate::rev) fn force_reap(&mut self, key: &Key) {
            if let Some(f) = self.in_flight.get(key) {
                f.done.cancel();
            }
            self.reap_cancelled(key);
        }
    }

    /// A history a test can extend and fold, with no view in the way.
    #[derive(Default)]
    struct Hist {
        rows: RwLock<Vec<(Epoch, Key, Value)>>,
        head: AtomicU64,
    }

    impl Hist {
        fn seal(&self, key: &Key, delta: Value) -> Epoch {
            let mut rows = self.rows.write().expect("not poisoned");
            let e = self.head.fetch_add(1, Ordering::SeqCst) + 1;
            rows.push((e, key.clone(), delta));
            e
        }
        /// The oracle: a fold over the frozen prefix, computed from nothing the view holds.
        fn fold(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
            let rows = self.rows.read().expect("not poisoned");
            let (mut acc, mut read) = (0i128, 0u64);
            for (e, k, d) in rows.iter() {
                if *e <= anchor && k == key {
                    acc += *d;
                    read += 1;
                }
            }
            (acc, read)
        }
    }

    impl Base for Hist {
        fn frontier(&self) -> Epoch {
            self.head.load(Ordering::SeqCst)
        }
        fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
            self.fold(key, anchor)
        }
        fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
            let rows = self.rows.read().expect("not poisoned");
            rows.iter()
                .filter(|(x, _, _)| *x == e)
                .map(|(_, k, d)| (k.clone(), *d))
                .collect()
        }
    }

    /// A one-shot latch. Deterministic where a sleep would be a guess.
    #[derive(Default)]
    struct Gate {
        open: Mutex<bool>,
        woken: Condvar,
    }

    impl Gate {
        fn open(&self) {
            *self.open.lock().expect("not poisoned") = true;
            self.woken.notify_all();
        }
        fn wait(&self) {
            let mut g = self.open.lock().expect("not poisoned");
            while !*g {
                g = self.woken.wait(g).expect("not poisoned");
            }
        }
    }

    type Shared = Arc<Mutex<Runtime>>;

    fn view(rt: &Shared) -> std::sync::MutexGuard<'_, Runtime> {
        rt.lock().expect("not poisoned")
    }

    /// This view's rendezvous-lock count, read the way a guard must: from the view under
    /// test, not from a process-wide total another test in the same binary contributes to.
    fn completion_locks(rt: &Shared) -> u64 {
        view(rt)
            .view("balance")
            .expect("installed")
            .stats
            .completion_locks
    }

    fn runtime(budget: Option<u64>) -> Shared {
        Arc::new(Mutex::new(
            Runtime::install(
                super::tests::circuit(Materialize::Demand, Consistency::Snapshot),
                budget,
                Policy::Lru,
            )
            .expect("installs"),
        ))
    }

    /// A whole keyed read done the way the daemon does it: decide under the view, release
    /// it, fold, take it again to install. Nothing is held across the fold.
    fn read_split(rt: &Shared, hist: &Hist, key: &Key, anchor: Epoch) -> Anchored {
        loop {
            let outcome = view(rt)
                .view_mut("balance")
                .expect("installed")
                .begin_read(key, anchor);
            match outcome {
                ReadOutcome::Hit(a) => return a,
                ReadOutcome::Fold(t) => {
                    let (v, rows) = hist.fold(t.key(), t.anchor());
                    return view(rt)
                        .view_mut("balance")
                        .expect("installed")
                        .finish_fold(t, v, rows);
                }
                ReadOutcome::Join(w) => {
                    if let Some(j) = w.wait() {
                        return j.answer;
                    }
                }
            }
        }
    }

    /// **The differential: one flight, one advance, two readers, three claims.**
    ///
    /// While a reconstruction anchored at `a0` is stopped mid-fold and an epoch for its own
    /// key is sealed and applied:
    ///
    /// 1. the `Pending` slot absorbs nothing — the epoch is skipped, not written into it;
    /// 2. a reader at `a0` **joins** the flight and folds nothing;
    /// 3. a reader at the new frontier folds **independently** and installs nothing;
    ///
    /// and when the flight lands it installs **pinned**, so no later anchor is ever served
    /// from an entry that never saw the epochs in between.
    #[test]
    fn a_flight_that_overlaps_an_advance_is_joined_shared_and_installed_pinned() {
        let hist = Arc::new(Hist::default());
        let k: Key = vec![7];
        let other: Key = vec![8];
        for i in 0..5i128 {
            hist.seal(&k, 10 + i);
            hist.seal(&other, 1);
        }
        let a0 = hist.frontier();
        let rt = runtime(Some(64));
        view(&rt).advance(&*hist, a0);

        let entered = Arc::new(Gate::default());
        let release = Arc::new(Gate::default());

        // The owner: it takes the flight, announces that it is inside the fold, and waits.
        let owner = {
            let (rt, hist, k, entered, release) = (
                rt.clone(),
                hist.clone(),
                k.clone(),
                entered.clone(),
                release.clone(),
            );
            std::thread::spawn(move || {
                let t = match view(&rt)
                    .view_mut("balance")
                    .expect("installed")
                    .begin_read(&k, a0)
                {
                    ReadOutcome::Fold(t) => t,
                    _ => panic!("the first reader of a cold key owns its flight"),
                };
                assert!(t.installs(), "the owner of a flight installs its result");
                entered.open();
                release.wait();
                let (v, rows) = hist.fold(t.key(), t.anchor());
                view(&rt)
                    .view_mut("balance")
                    .expect("installed")
                    .finish_fold(t, v, rows)
            })
        };
        entered.wait();

        assert_eq!(
            view(&rt).view("balance").expect("installed").slot(&k),
            Slot::Pending(a0),
            "a key whose reconstruction is in flight is `Pending` at the anchor being folded \
             — the lattice's fourth state, which no path could reach while the fold happened \
             inside the view lock"
        );

        // **An epoch for this very key, sealed and applied while the flight is out.**
        hist.seal(&k, 1_000_000);
        let e1 = hist.frontier();
        view(&rt).advance(&*hist, e1);

        assert_eq!(
            view(&rt).view("balance").expect("installed").slot(&k),
            Slot::Pending(a0),
            "`advance` wrote into a `Pending` slot. There is no value there to fold a delta \
             into, and publishing one epoch's delta as the key's balance makes every reader \
             whose anchor falls in the resulting interval take it as a hit: the account is \
             wrong by its entire history until the flight lands"
        );

        // **A reader at the flight's own anchor joins it.**
        let joined = {
            let outcome = view(&rt)
                .view_mut("balance")
                .expect("installed")
                .begin_read(&k, a0);
            match outcome {
                ReadOutcome::Join(w) => std::thread::spawn(move || w.wait()),
                _ => panic!(
                    "a reader of the same key at the same anchor must join the flight, not \
                     start a second fold of the same prefix"
                ),
            }
        };

        // **A reader at a different anchor folds alone and installs nothing.**
        let at_e1 = {
            let outcome = view(&rt)
                .view_mut("balance")
                .expect("installed")
                .begin_read(&k, e1);
            let t = match outcome {
                ReadOutcome::Fold(t) => t,
                _ => panic!(
                    "a flight at another anchor is computing a different number over a \
                     different prefix; joining it would answer this reader with someone \
                     else's snapshot"
                ),
            };
            assert!(
                !t.installs(),
                "a fold that does not own the key's flight must not write to its slot"
            );
            let (v, rows) = hist.fold(t.key(), t.anchor());
            view(&rt)
                .view_mut("balance")
                .expect("installed")
                .finish_fold(t, v, rows)
        };
        assert_eq!(at_e1.anchor, e1);
        assert_eq!(
            at_e1.value,
            hist.fold(&k, e1).0,
            "the independent fold must equal the oracle at its own anchor"
        );

        release.open();
        let owned = owner.join().expect("the owner panicked");
        let joined = joined
            .join()
            .expect("the joiner panicked")
            .expect("a waiter on a live flight is answered by it");

        assert_eq!(owned.anchor, a0);
        assert_eq!(
            owned.value,
            hist.fold(&k, a0).0,
            "the owner's fold is exact at `a0`"
        );
        assert_eq!(
            joined.answer, owned,
            "a joined reader receives the flight's answer, not a second fold's"
        );
        // And the evidence that shapes it: the owner's own reconstruction row count, which
        // is what lets a joined reader tell a key with no history from a balance of zero
        // without going back to the base (A10-04).
        assert!(
            joined.base_rows > 0,
            "this key has history, and the flight must publish that alongside the value"
        );

        // **The install is pinned, because the frontier moved under it.**
        let g = view(&rt);
        let v = g.view("balance").expect("installed");
        assert_eq!(
            v.slot(&k),
            Slot::Present(owned.value, a0),
            "the flight installs at its own anchor"
        );
        assert_eq!(v.stats.pending_joins, 1, "exactly one reader joined");
        assert_eq!(
            v.stats.pinned_installs, 1,
            "the flight landed below `applied`"
        );
        assert!(
            v.stats.uninstalled_folds >= 1,
            "the different-anchor fold answered and installed nothing"
        );
        assert!(
            v.stats.pinned_installs + v.stats.deferred_merges + v.stats.pending_joins > 0,
            "the run did not exercise the mechanism it is named for, so a pass here would \
             be a pass for the code this replaces"
        );
        drop(g);

        // **And the pin is what makes the next reader right.** An unpinned entry would
        // inherit `applied = e1` and serve the value at `a0` to a reader asking for `e1`.
        let fresh = read_split(&rt, &hist, &k, e1);
        assert_eq!(fresh.anchor, e1);
        assert_eq!(
            fresh.value,
            hist.fold(&k, e1).0,
            "a read at the new frontier was served the value from before the epoch that \
             landed during the flight. An entry installed at an anchor below `applied` has \
             not seen the deltas in between and must not inherit the frontier"
        );
    }

    /// **A completion that arrives after its flight was superseded installs nothing.**
    ///
    /// Not because its value is wrong — a ticket's `(key, anchor)` is fixed at `begin_read`,
    /// so its answer is exact at its own anchor and it still answers its own caller. What it
    /// must not do is write over the entry a *newer* flight installed: that moves a resident
    /// stamp backwards, costing every later reader a reconstruction, and clears an
    /// `in_flight` entry it does not own, stranding the successor's `Pending` marker with
    /// nobody left to publish it.
    #[test]
    fn a_superseded_completion_never_moves_a_resident_stamp_backwards() {
        let hist = Hist::default();
        let k: Key = vec![3];
        for i in 0..4i128 {
            hist.seal(&k, 100 + i);
        }
        let a0 = hist.frontier();
        let rt = runtime(Some(64));

        let stale = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&k, a0)
        {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the first reader owns the flight"),
        };

        // The owner is cancelled — a dropped connection, a killed statement — and the next
        // reader through the view reaps its marker and takes over with a new generation.
        view(&rt)
            .view_mut("balance")
            .expect("installed")
            .force_reap(&k);
        hist.seal(&k, 500);
        let a1 = hist.frontier();

        let fresh = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&k, a1)
        {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the reaped key is free for a new flight"),
        };
        let (v1, r1) = hist.fold(fresh.key(), fresh.anchor());
        let newer = view(&rt)
            .view_mut("balance")
            .expect("installed")
            .finish_fold(fresh, v1, r1);
        assert_eq!(newer.anchor, a1);

        // Now the superseded owner returns, with an answer that was exact when it started.
        let (v0, r0) = hist.fold(stale.key(), stale.anchor());
        let late = view(&rt)
            .view_mut("balance")
            .expect("installed")
            .finish_fold(stale, v0, r0);
        assert_eq!(
            late,
            Anchored {
                value: hist.fold(&k, a0).0,
                anchor: a0
            },
            "a superseded completion still answers its own caller, exactly, at its own anchor"
        );

        let g = view(&rt);
        let v = g.view("balance").expect("installed");
        assert_eq!(
            v.slot(&k),
            Slot::Present(newer.value, a1),
            "the older completion overwrote the newer resident entry. Its value is right at \
             its own anchor and wrong as a resident stamp: every read above `{a1}` now \
             reconstructs, and the flight table entry it cleared belonged to somebody else"
        );
        assert!(
            v.stats.uninstalled_folds >= 1,
            "a fold that installed nothing must be counted, or the cost is one nobody can \
             be asked about"
        );
    }

    /// **A flight over a resident key leaves the entry resident, and it still answers —
    /// A10-12.**
    ///
    /// "Between `begin_read` and `finish_fold` the slot is `Pending`" was stated as a
    /// property of a flight. It is a property of *one* case. A key whose entry is `Present`
    /// at an anchor that cannot answer this reader keeps that entry — `prior` is `None`
    /// because there is nothing to put back — so during the flight the slot holds a value,
    /// and a third reader inside that entry's certification interval is served a **hit**
    /// while the reconstruction is still out.
    ///
    /// That is correct behaviour and it is worth having a test say so, because the sentence
    /// it replaces would have made this run look like a bug. The three claims are asserted
    /// separately, which is the whole of the repair:
    ///
    ///   * **the slot** — still `Present`, at its own stamp, not `Pending`;
    ///   * **the certification** — a read inside that interval hits, and the value it gets
    ///     equals an independent fold of the base at that anchor;
    ///   * **the flight** — outstanding the whole time, and its own answer exact at its own
    ///     anchor when it lands.
    #[test]
    fn a_flight_over_a_resident_key_leaves_the_entry_present_and_answering() {
        let rt = runtime(Some(64));
        let hist = Hist::default();
        let k: Key = vec![11];
        hist.seal(&k, 5);
        let a0 = hist.frontier();

        // Install an entry at `a0` the ordinary way.
        //
        // **The outcome is bound to a local before it is matched**, and that is not style.
        // A temporary in a `match` scrutinee lives until the end of the whole `match`, so
        // `match view(&rt)...begin_read(..) { .. }` holds the view guard inside every arm —
        // and an arm that calls `view(&rt)` again self-deadlocks on a `Mutex` that is not
        // reentrant. The first draft of this test did exactly that and hung the suite until
        // it was killed, which is the same mistake, in a test, that this cycle is repairing
        // in the engine. `read_split` above binds first for the same reason.
        let outcome = view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&k, a0);
        let first = match outcome {
            ReadOutcome::Fold(t) => {
                let (v, rows) = hist.fold(t.key(), t.anchor());
                view(&rt)
                    .view_mut("balance")
                    .expect("installed")
                    .finish_fold(t, v, rows)
            }
            _ => panic!("a cold key folds"),
        };
        assert_eq!(first.value, hist.fold(&k, a0).0);

        // Move the base and start a flight at the newer anchor. The entry at `a0` cannot
        // answer it, so a reconstruction is authorised — and the entry stays.
        hist.seal(&k, 7);
        let a1 = hist.frontier();
        let outcome = view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&k, a1);
        let ticket = match outcome {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the newer anchor is outside the entry's interval, so it folds"),
        };

        // **The slot**, stated on its own.
        {
            let g = view(&rt);
            let v = g.view("balance").expect("installed");
            assert_eq!(
                v.slot(&k),
                Slot::Present(first.value, a0),
                "a flight over a key that already has a good answer must not evict it to \
                 record that somebody is looking for a different one. The claim that the \
                 slot is `Pending` for the duration of a flight is false here, and it was \
                 stated without this case in view (A10-12)."
            );
        }

        // **The certification**, stated on its own: the retained entry still answers inside
        // its own interval, and what it answers equals an independent fold.
        let outcome = view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&k, a0);
        let hit = match outcome {
            ReadOutcome::Hit(a) => a,
            _ => panic!(
                "the retained entry must still serve a read inside its own certification \
                 interval while the flight for a later anchor is outstanding"
            ),
        };
        assert_eq!(
            hit.anchor, a0,
            "a hit answers at the anchor it was asked for"
        );
        assert_eq!(
            hit.value,
            hist.fold(&k, a0).0,
            "and its value is the one an independent fold of the base gives at that anchor"
        );

        // **The flight**, stated on its own: still outstanding, and exact when it lands.
        let (v, rows) = hist.fold(ticket.key(), ticket.anchor());
        let landed = view(&rt)
            .view_mut("balance")
            .expect("installed")
            .finish_fold(ticket, v, rows);
        assert_eq!(landed.anchor, a1);
        assert_eq!(
            landed.value,
            hist.fold(&k, a1).0,
            "the flight's own answer is exact at its own anchor, which follows from the \
             fold's precondition and not from anything about the slot"
        );
    }

    /// **The legacy read cannot join, so it cannot wait on a publication it is blocking —
    /// A10-03.**
    ///
    /// The schedule: one thread owns a flight for a key and has released the view to fold.
    /// A second thread calls `Rev::read` for that same key at that same anchor, through the
    /// same `Mutex<Runtime>`. `read` takes the view exclusively for its whole body, and the
    /// owner needs that view to call `finish_fold` — so a `read` that joined would wait for
    /// a publication only it is preventing, and both threads would be permanent.
    ///
    /// **Bounded, because the failure is a hang.** The second thread's work is done behind a
    /// channel with a deadline; a build that joins never sends, and the assertion fails
    /// instead of stopping the suite. The stranded thread is left parked on a runtime this
    /// test owns.
    #[test]
    fn the_legacy_read_folds_alone_rather_than_waiting_for_a_flight_it_blocks() {
        use std::sync::mpsc;
        use std::time::Duration;

        let rt = runtime(Some(64));
        let hist = Hist::default();
        hist.seal(&vec![5], 11);
        let anchor = hist.frontier();

        // Thread one owns the flight and holds nothing: `begin_read` returned and the guard
        // it was taken under is dropped here.
        let ticket = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&vec![5], anchor)
        {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the first reader owns the flight"),
        };

        // Thread two goes through `Rev::read`, which takes the view for its whole body.
        let (tx, rx) = mpsc::channel();
        let rt2 = rt.clone();
        std::thread::spawn(move || {
            let hist2 = Hist::default();
            hist2.seal(&vec![5], 11);
            let a =
                view(&rt2)
                    .view_mut("balance")
                    .expect("installed")
                    .read(&hist2, &vec![5], anchor);
            let _ = tx.send(a);
        });

        let answered = rx.recv_timeout(Duration::from_secs(10)).expect(
            "`Rev::read` did not return within 10s. It holds the view exclusively for its \
             whole body, so if it joined the flight owned by this test it is waiting for a \
             `finish_fold` that needs the very view it is holding — the deadlock of A10-03. \
             It must fold alone instead.",
        );
        assert_eq!(
            answered.anchor, anchor,
            "an independent fold still answers at the anchor it was asked for"
        );
        assert_eq!(answered.value, 11, "and with the value the base holds");

        // The owner can still finish, which it could not if the view were held against it.
        let (v, rows) = hist.fold(ticket.key(), ticket.anchor());
        let owner = view(&rt)
            .view_mut("balance")
            .expect("installed")
            .finish_fold(ticket, v, rows);
        assert_eq!(owner.value, answered.value, "both folds see the same base");

        let declined = view(&rt)
            .view("balance")
            .expect("installed")
            .stats
            .joins_declined_by_caller;
        assert!(
            declined >= 1,
            "the declined join must be counted under its own reason and not as a capacity \
             refusal — an overloaded flight table and a caller that may not wait are \
             different findings with different remedies"
        );
    }

    /// **A wait ticket that is dropped gives its place back — A10-05.**
    ///
    /// `begin_read` reserves a place in the flight's queue under the completion's lock and
    /// hands out a ticket; `join` gives it back. A ticket that is *dropped* — a cancelled
    /// statement, a client that hung up, a caller that chose to fold instead — gave nothing
    /// back, so the queue held a reservation for a reader that was never coming.
    /// `MAX_WAITERS` of those and every later reader is refused a join it could have had,
    /// against a queue that is in fact empty.
    ///
    /// The observable is the refusal counter: with the leak, the reader after the abandoned
    /// ones is refused and `waiters_refused` rises; without it, that reader joins.
    #[test]
    fn an_abandoned_wait_ticket_releases_its_place_in_the_queue() {
        let rt = runtime(Some(64));
        let hist = Hist::default();
        hist.seal(&vec![3], 7);
        let anchor = hist.frontier();

        let ticket = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&vec![3], anchor)
        {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the first reader owns the flight"),
        };

        // Take and abandon a place, more times than the queue can hold. Each `WaitTicket`
        // is dropped without being waited on, which is the whole case.
        for _ in 0..(MAX_WAITERS as usize + 8) {
            match view(&rt)
                .view_mut("balance")
                .expect("installed")
                .begin_read(&vec![3], anchor)
            {
                ReadOutcome::Join(w) => drop(w),
                ReadOutcome::Fold(_) => panic!(
                    "a reader at the owner's exact anchor must be offered a join; being told \
                     to fold means the queue is already full of places nobody is standing in"
                ),
                ReadOutcome::Hit(_) => panic!("nothing has been installed yet"),
            }
        }

        let refused = view(&rt)
            .view("balance")
            .expect("installed")
            .stats
            .waiters_refused;
        assert_eq!(
            refused, 0,
            "{refused} reader(s) were refused a join. Every ticket above was dropped without \
             being waited on, so the queue should be empty — a refusal here is a place held \
             for a reader that is not coming."
        );

        // And the flight still works: the owner publishes, and a waiter that does wait is
        // answered. Without this the test would pass against a build that never queues.
        let waiter = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&vec![3], anchor)
        {
            ReadOutcome::Join(w) => w,
            _ => panic!("the queue must still admit a real waiter"),
        };
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(waiter.wait());
        });
        let (v, rows) = hist.fold(ticket.key(), ticket.anchor());
        let owner = view(&rt)
            .view_mut("balance")
            .expect("installed")
            .finish_fold(ticket, v, rows);
        let joined = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("the waiter is answered within the deadline")
            .expect("and with an answer rather than a cancellation");
        assert_eq!(joined.answer.value, owner.value);
    }

    /// **A fold nobody joined never touches its rendezvous — A10-19.**
    ///
    /// The cost being guarded is a heap allocation, not a wait. Rust's `Mutex` and `Condvar`
    /// on macOS lazily box a `pthread_mutex_t` and a `pthread_cond_t` on first use — 64 and
    /// 48 bytes — so the first acquisition of a completion's lock costs two allocations and
    /// 112 bytes, and every subsequent one costs nothing. E18's `rev_metadata_2x_budget`
    /// folds 5,000 uncontended keys and measured **44,165 allocations on Host C against
    /// 34,165 committed**: a difference of exactly 10,000, exactly two per key, with `live`
    /// and `peak` identical because the objects are freed with the flight. The budget could
    /// not see it because the machine it was measured on does not charge for it.
    ///
    /// So the guard is a **count**, not a duration: it fails the same way on Linux, where
    /// the saving is two atomic operations and no allocation at all. Ten sequential misses
    /// on one thread create ten flights, and none of them may lock anything.
    ///
    /// This is not a claim that locking is expensive. It is a claim that a rendezvous with
    /// nobody at it should not be used, and that the platform where that is nearly free is
    /// not the only platform this runs on.
    #[test]
    fn an_uncontended_fold_never_locks_its_completion() {
        let rt = runtime(Some(64));
        let hist = Hist::default();
        for k in 0..10i64 {
            hist.seal(&vec![k], 10 + k as i128);
        }
        let anchor = hist.frontier();

        let before = completion_locks(&rt);
        for k in 0..10i64 {
            let outcome = view(&rt)
                .view_mut("balance")
                .expect("installed")
                .begin_read(&vec![k], anchor);
            match outcome {
                ReadOutcome::Fold(t) => {
                    let (v, rows) = hist.fold(t.key(), t.anchor());
                    view(&rt)
                        .view_mut("balance")
                        .expect("installed")
                        .finish_fold(t, v, rows);
                }
                _ => panic!("a cold key on one thread must fold alone"),
            }
        }
        let after = completion_locks(&rt);
        assert_eq!(
            after,
            before,
            "ten uncontended folds took {} completion lock(s). Each first acquisition costs \
             two heap allocations and 112 bytes on macOS, which is the whole of the E18 gate \
             failure at `rev_metadata_2x_budget`: 44,165 allocations against a 34,165 \
             budget, two per key over 5,000 keys.",
            after - before
        );
    }

    /// **And a fold that IS joined still uses it** — the control without which the guard
    /// above is satisfied by a rendezvous that never works at all.
    ///
    /// The fast path is a claim about flights with no waiter. A build that skipped the
    /// rendezvous unconditionally would strand every joiner forever and would pass the count
    /// guard perfectly, so the count is asserted to *rise* here, and the joined reader is
    /// asserted to actually receive the owner's answer.
    #[test]
    fn a_joined_fold_does_use_its_completion_and_answers_the_waiter() {
        let rt = runtime(Some(64));
        let hist = Hist::default();
        hist.seal(&vec![7], 42);
        let anchor = hist.frontier();

        let ticket = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&vec![7], anchor)
        {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the first reader owns the flight"),
        };
        let before = completion_locks(&rt);
        let waiter = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&vec![7], anchor)
        {
            ReadOutcome::Join(w) => w,
            _ => panic!("the second reader at the same anchor joins"),
        };

        let (v, rows) = hist.fold(ticket.key(), ticket.anchor());
        let owner_answer = view(&rt)
            .view_mut("balance")
            .expect("installed")
            .finish_fold(ticket, v, rows);

        // **The wait is bounded, because the failure this control is for is a hang.**
        //
        // A build that applied the uncontended fast path to every flight — by never marking
        // a joined completion — publishes to nobody and the waiter parks forever. Reverting
        // `mark_joined` and running this test proved exactly that, by hanging the suite
        // until it was killed: a red test that never returns is not a result, it is a
        // machine to be interrupted. So the wait happens on its own thread and the assertion
        // is a deadline. The stranded thread is left parked on a completion this test owns
        // and nothing else can reach.
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(waiter.wait());
        });
        let joined_answer = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap_or_else(|_| {
                panic!(
                    "the joined reader was never answered within 10s. Its owner has already \
                     finished the fold, so the rendezvous was skipped for a flight that had \
                     a waiter — which is the fast path applied where it does not hold, and \
                     the waiter is stranded rather than slow."
                )
            })
            .expect("the waiter is answered, not stranded");

        // Read after the flight has ended, because that is when a completion's count is
        // folded into the view — it is the last moment the completion is reachable from the
        // view at all.
        eprintln!("DBG before={before} after={}", completion_locks(&rt));
        assert!(
            completion_locks(&rt) > before,
            "a joined flight must take its rendezvous. A build that skipped it \
             unconditionally would pass the uncontended guard perfectly and strand every \
             waiter, so this control is what makes that guard mean what it says."
        );
        assert_eq!(
            joined_answer.answer.value, owner_answer.value,
            "a joined reader receives the owner's answer, which is the point of joining"
        );
        assert_eq!(
            joined_answer.answer.anchor, owner_answer.anchor,
            "and at the owner's anchor, which is its own"
        );
    }

    /// **A flight whose owner never returns releases its waiters and puts the slot back.**
    ///
    /// Honest absence survives cancellation: what goes back is the exact `Hole(e)` the
    /// marker replaced, version and all, and not a `⊥` that would make the key look never
    /// seen.
    #[test]
    fn a_cancelled_flight_restores_the_absence_it_replaced_and_frees_its_waiters() {
        let hist = Hist::default();
        let (k, filler): (Key, Key) = (vec![1], vec![2]);
        hist.seal(&k, 42);
        hist.seal(&filler, 7);
        let a = hist.frontier();
        // A budget of one, so reading the filler evicts `k` and leaves a hole with a version.
        let rt = runtime(Some(1));
        read_split(&rt, &hist, &k, a);
        read_split(&rt, &hist, &filler, a);
        let hole = view(&rt).view("balance").expect("installed").slot(&k);
        assert_eq!(hole, Slot::Hole(a), "the evicted entry keeps its version");

        let doomed = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&k, a)
        {
            ReadOutcome::Fold(t) => t,
            _ => panic!("a hole is reconstructed, and the first reader owns the flight"),
        };
        let waiter = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&k, a)
        {
            ReadOutcome::Join(w) => std::thread::spawn(move || w.wait()),
            _ => panic!("the second reader at the same anchor joins"),
        };

        drop(doomed);
        assert!(
            waiter.join().expect("the waiter panicked").is_none(),
            "a waiter on a flight whose owner died must wake and retry. Waiting on a thread \
             that is not coming back turns one cancelled statement into a stuck connection"
        );

        view(&rt)
            .view_mut("balance")
            .expect("installed")
            .force_reap(&k);
        assert_eq!(
            view(&rt).view("balance").expect("installed").slot(&k),
            hole,
            "a cancelled flight puts back exactly the absence it replaced. A `⊥` here would \
             say the key was never seen, which is the one thing honest absence exists to \
             prevent"
        );
    }

    /// **Past the ceiling a read folds alone, exactly, and the refusal is counted.**
    ///
    /// The flight table is bounded because an unbounded one is a per-connection allocation
    /// with no ceiling. Over the bound the answer is still exact — it is simply not shared —
    /// and there is no compensating branch: no silent fold under the view lock, no second
    /// path with different rules.
    #[test]
    fn a_view_past_its_flight_ceiling_folds_alone_and_says_so() {
        let hist = Hist::default();
        for i in 0..(MAX_FLIGHTS as i64 + 1) {
            hist.seal(&vec![i], 1);
        }
        let a = hist.frontier();
        let rt = runtime(None);

        let mut held = Vec::new();
        for i in 0..MAX_FLIGHTS as i64 {
            match view(&rt)
                .view_mut("balance")
                .expect("installed")
                .begin_read(&vec![i], a)
            {
                ReadOutcome::Fold(t) => held.push(t),
                _ => panic!("each cold key owns its own flight"),
            }
        }
        let over = match view(&rt)
            .view_mut("balance")
            .expect("installed")
            .begin_read(&vec![MAX_FLIGHTS as i64], a)
        {
            ReadOutcome::Fold(t) => t,
            _ => panic!("a refused flight still folds"),
        };
        assert!(
            !over.installs(),
            "a read the flight table had no room for must not claim a slot it does not own"
        );
        assert_eq!(
            view(&rt)
                .view("balance")
                .expect("installed")
                .stats
                .flights_refused,
            1,
            "an overload that shows up only as latency is one nobody can be asked about"
        );
    }
}

#[cfg(test)]
mod concurrent_differential {
    //! **The view's answer, against an independent fold, while a writer moves the frontier.**
    //!
    //! T-02 made `read` serve a resident entry at the anchor it was asked for whenever
    //! `stamp ≤ anchor ≤ effective`. Every test that gates it is either single-threaded or
    //! asserts a *rate* — the fallback counters say how often the view answered, not whether the
    //! answer was right. A rate cannot falsify a correctness change.
    //!
    //! This is the differential that can. A writer seals epochs; readers sample an anchor, read
    //! the key through the view, and compare against `Base::reconstruct` at that same anchor —
    //! a fold over the frozen prefix, computed independently of anything the view holds. The
    //! comparison is the whole claim of the certification interval: *the value is unchanged
    //! across `[stamp, effective]`, so serving it at any anchor inside is exact.* If the upper
    //! bound were too loose the reader would see a value from the future; if the lower bound
    //! were missing it would see one from after its own anchor. Either way the fold disagrees.
    //!
    //! The eviction budget is deliberately below the key count, so the same key is answered from
    //! a resident entry on one read and rebuilt on the next, and both must agree with the fold.

    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::RwLock;

    const KEYS: i64 = 24;

    /// A base a writer can extend while readers fold it.
    ///
    /// `FoldBase` above is `&mut` to seal, which is right for a definition of the answer and
    /// useless here: the race being tested is between an append and a read. The rows are behind
    /// an `RwLock`, and every method of `Base` takes `&self` already — that was T-06's change,
    /// and it is what makes this test expressible at all.
    #[derive(Default)]
    struct SharedBase {
        rows: RwLock<Vec<(Epoch, Key, Value)>>,
        head: AtomicU64,
    }

    impl SharedBase {
        fn seal(&self, key: Key, delta: Value) {
            let mut rows = self.rows.write().expect("not poisoned");
            let e = self.head.fetch_add(1, Ordering::SeqCst) + 1;
            rows.push((e, key, delta));
        }
    }

    impl Base for SharedBase {
        fn frontier(&self) -> Epoch {
            self.head.load(Ordering::SeqCst)
        }
        fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
            let rows = self.rows.read().expect("not poisoned");
            let mut acc = 0i128;
            let mut read = 0u64;
            for (e, k, d) in rows.iter() {
                if *e <= anchor && k == key {
                    acc += *d;
                    read += 1;
                }
            }
            (acc, read)
        }
        fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
            let rows = self.rows.read().expect("not poisoned");
            rows.iter()
                .filter(|(x, _, _)| *x == e)
                .map(|(_, k, d)| (k.clone(), *d))
                .collect()
        }
    }

    fn circuit() -> Circuit {
        super::tests::circuit(Materialize::Demand, Consistency::Snapshot)
    }

    #[test]
    fn every_answer_matches_an_independent_fold_at_its_own_anchor() {
        let base = std::sync::Arc::new(SharedBase::default());
        // A prefix, so readers have something to hit before the writer starts.
        for i in 0..(KEYS * 4) {
            base.seal(vec![i % KEYS], 10 + i as i128);
        }
        let rt = std::sync::Arc::new(std::sync::Mutex::new(
            Runtime::install(circuit(), Some(8), Policy::Lru).expect("installs"),
        ));
        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let compared = std::sync::Arc::new(AtomicU64::new(0));
        /// How far the frontier must move under the readers for the run to be the
        /// experiment it claims to be.
        const SEAL_TARGET: Epoch = 120;
        /// Reads per reader, so four readers clear the comparison floor below.
        const READS_EACH: u64 = 400;
        let start_head = base.frontier();

        let writer = {
            let (base, stop) = (base.clone(), stop.clone());
            std::thread::spawn(move || {
                let mut i = 0i64;
                while !stop.load(Ordering::Relaxed) {
                    i += 1;
                    base.seal(vec![i % KEYS], 1);
                    // Advance the view from the writer, as the engine does under its own lock.
                    std::thread::yield_now();
                }
                i
            })
        };

        let advancer = {
            let (base, rt, stop) = (base.clone(), rt.clone(), stop.clone());
            std::thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    let f = base.frontier();
                    rt.lock().expect("not poisoned").advance(&*base, f);
                    std::thread::yield_now();
                }
            })
        };

        let readers: Vec<_> = (0..4)
            .map(|t| {
                let (base, rt, compared) = (base.clone(), rt.clone(), compared.clone());
                std::thread::spawn(move || {
                    let mut divergences: Vec<String> = Vec::new();
                    // **Bounded by the writer's progress, not by a fixed count.** The
                    // experiment is "answers taken while the frontier moves", so the loop
                    // ends when the frontier has moved far enough — 400 reads on a machine
                    // that gave the writer no CPU is 400 reads of a static base, which is a
                    // different experiment reported under this one's name. On a loaded
                    // two-core container the writer sealed 3 epochs against these readers'
                    // 1,600 reads, and the run failed its own precondition rather than
                    // passing vacuously; now the readers wait for it.
                    //
                    // **Both preconditions, or neither is met.** Stopping as soon as the
                    // frontier had moved made the run end after nine comparisons when the
                    // writer got a burst of CPU, which failed the *other* precondition: an
                    // experiment that compares nine answers has not looked for a race
                    // either. The loop therefore runs until the frontier has moved *and*
                    // this reader has taken its share of reads.
                    let mut n = 0u64;
                    while (base.frontier() < start_head + SEAL_TARGET || n < READS_EACH)
                        && n < 20_000
                    {
                        n += 1;
                        let key = vec![((t * 7 + n as i64) % KEYS).max(0)];
                        // The anchor a session would hold: sampled, then used. Everything the
                        // writer does after this sample is outside the read's snapshot.
                        let anchor = base.frontier();
                        // **Read the way the daemon reads: decide under the view, release
                        // it, fold, take it again.** Driving `read` here would hold the view
                        // across the reconstruction, which is exactly the shape this task
                        // removed — the differential would then be testing a path no server
                        // takes, and the `Join` case could never arise because no second
                        // reader could run.
                        let answered = loop {
                            let outcome = {
                                let mut g = rt.lock().expect("not poisoned");
                                let v = g.view_mut("balance").expect("installed");
                                v.begin_read(&key, anchor)
                            };
                            match outcome {
                                ReadOutcome::Hit(a) => break a,
                                ReadOutcome::Fold(t) => {
                                    let (value, rows) = base.reconstruct(t.key(), t.anchor());
                                    let mut g = rt.lock().expect("not poisoned");
                                    let v = g.view_mut("balance").expect("installed");
                                    break v.finish_fold(t, value, rows);
                                }
                                // Nothing is held here, which is the property the whole
                                // split exists for and the one a deadlock would expose.
                                ReadOutcome::Join(w) => {
                                    if let Some(j) = w.wait() {
                                        break j.answer;
                                    }
                                }
                            }
                        };
                        // The oracle: a fold over the frozen prefix ending at the anchor the
                        // answer says it is true at. Independent of every resident entry.
                        let (expected, _) = base.reconstruct(&key, answered.anchor);
                        compared.fetch_add(1, Ordering::Relaxed);
                        if answered.value != expected {
                            divergences.push(format!(
                                "key {key:?} at anchor {anchor}: view said {} stamped {}, the \
                                 fold at {} says {expected}",
                                answered.value, answered.anchor, answered.anchor
                            ));
                        }
                        if answered.anchor != anchor {
                            divergences.push(format!(
                                "key {key:?}: asked for anchor {anchor}, answered at {}",
                                answered.anchor
                            ));
                        }
                    }
                    divergences
                })
            })
            .collect();

        let mut divergences = Vec::new();
        for r in readers {
            divergences.extend(r.join().expect("a reader panicked"));
        }
        stop.store(true, Ordering::Relaxed);
        let sealed = writer.join().expect("the writer panicked");
        advancer.join().expect("the advancer panicked");

        assert!(
            base.frontier() >= start_head + SEAL_TARGET,
            "the readers gave up before the frontier moved {SEAL_TARGET} epochs ({sealed} \
             sealed, head {} from {start_head}). Every reader hit its 20,000-read ceiling, \
             which means the writer thread was not scheduled — the machine could not run \
             this experiment, and a pass here would be a pass for a static base.",
            base.frontier()
        );
        assert!(
            compared.load(Ordering::Relaxed) >= 1_500,
            "the differential must actually compare: {} comparisons. Each of the four \
             readers runs until the frontier has moved and it has taken {READS_EACH} reads, \
             so falling short means a reader hit its 20,000-read ceiling with the writer \
             starved",
            compared.load(Ordering::Relaxed)
        );
        assert!(
            divergences.is_empty(),
            "{} divergences under concurrent append and read. First: {}",
            divergences.len(),
            divergences.first().expect("non-empty")
        );

        // **Reported, not asserted.** Whether four readers overlapped on one key at one
        // anchor is the scheduler's business, and a test that required it would be a test
        // that fails on an idle machine. The deterministic claim is the latched differential
        // above; this line says what this run happened to see.
        let s = rt.lock().expect("not poisoned").stats();
        eprintln!(
            "two-phase, under concurrency: {} joins, {} uninstalled folds, {} pinned \
             installs, {} flights refused, over {} reads",
            s.pending_joins, s.uninstalled_folds, s.pinned_installs, s.flights_refused, s.reads
        );
    }
}

/// **Two counters that answer different questions, and were read as one.**
///
/// `flights_that_fell_behind` increments when the view's `applied` epoch moves *between* a
/// flight's begin and its landing. `pinned_installs` increments when the landing anchor is
/// below `applied` at all — which includes every flight that **began** behind and never moved
/// an inch. They are not two views of the same event, and the cycle-10 baseline separates
/// them sharply: `flights_that_fell_behind` is 0 in every level of every replicate on both
/// hosts, while pinned installs are 45,282 of 45,681 — **99.1% of installs** — with an
/// arrival gap of four to five epochs.
///
/// The zero is structural and this module proves it: on the served path a fold holds the base
/// shared for its whole duration and `append` needs it exclusively, so no epoch can be applied
/// inside a flight. Nothing falls behind while folding.
///
/// What that zero does **not** say is that a deferred merge has nothing to merge. The suffix
/// a merge would fold is `(anchor, applied]` at landing, and that interval is non-empty for
/// almost every install here — not because folds are slow, but because readers arrive with a
/// frontier already a few epochs behind the view's. So the merge window is wide open; it is
/// simply opened by arrival lag rather than by fold duration, which is a different mechanism
/// with a different cost model and a different bound.
///
/// The first test below is the half that was missing before any of this could be reported:
/// **nothing asserted the counter could ever be non-zero.** An instrument reading zero
/// because it is dead reads exactly like an instrument reading zero because the event does
/// not occur. The second states the structural exclusion as arithmetic rather than as a
/// schedule that happened to be observed.
///
/// The separate question of whether the shared hold across the fold should be released at all
/// is A10-10, and it is not answered here: releasing it lets a fold span an append, which
/// needs the fold to be provably strict-serialisable at its anchor rather than merely bounded
/// by `[stamp, effective]` — a snapshot or a per-key version, not a lock release.
#[cfg(test)]
mod deferred_merge_window_tests {
    use super::tests::{circuit, FoldBase};
    use super::*;

    /// The instrument is live: when a view *does* advance under a flight, the counter says so.
    ///
    /// Driven through the two-phase API directly, because that interleaving is the one the
    /// served path structurally prevents — the only way to produce it is to be the thing the
    /// served path is not.
    #[test]
    fn a_flight_that_the_view_outruns_is_counted() {
        let mut base = FoldBase::new(0);
        for k in 0..4 {
            base.seal(vec![k], 100);
        }
        let early = base.frontier();
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        rt.advance(&base, early);

        // Begin a fold at the epoch the view is at.
        let v = rt.view_mut("balance").expect("balance");
        let ticket = match v.begin_read_with(&vec![0], early, ReadMode::Alone) {
            ReadOutcome::Fold(t) => t,
            ReadOutcome::Hit(_) => panic!("a cold key must fold, not hit"),
            ReadOutcome::Join(_) => panic!("`Alone` does not produce a join"),
        };

        // The view moves on while the fold is outstanding. This is what the served path
        // cannot do, and what a released base hold would make possible.
        for k in 0..4 {
            base.seal(vec![k], 5);
        }
        let later = base.frontier();
        rt.advance(&base, later);

        let (value, rows) = base.reconstruct(&vec![0], early);
        let v = rt.view_mut("balance").expect("balance");
        let answer = v.finish_fold(ticket, value, rows);

        assert_eq!(
            answer.anchor, early,
            "a late landing must certify at the anchor it folded, not at the view's epoch"
        );
        assert_eq!(
            v.stats.flights_that_fell_behind, 1,
            "the counter the merge decision rests on must be able to move; if this is 0 the \
             zeroes measured in the baseline mean nothing"
        );
        assert!(
            v.stats.pinned_installs >= 1,
            "the late landing is pinned at its anchor, which is the policy a deferred merge \
             would replace"
        );
    }

    /// The structural exclusion, stated as arithmetic rather than as an observed schedule.
    ///
    /// `flights_that_fell_behind` increments exactly when `applied` at landing exceeds
    /// `applied` at begin. `applied` moves only in `advance`, which the served path calls
    /// while holding the base exclusively, and a flight holds it shared from begin to
    /// landing. So on that path the two values are equal at every landing — and a run of
    /// folds interleaved with nothing shows it: the counter stays at zero however many
    /// epochs the base has, because none of them arrive *during* a fold.
    #[test]
    fn a_flight_that_nothing_advances_under_never_falls_behind() {
        let mut base = FoldBase::new(0);
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(2),
            Policy::Lru,
        )
        .unwrap();
        for round in 0..8 {
            for k in 0..6 {
                base.seal(vec![k], 10 + round);
            }
            let head = base.frontier();
            // Advance and read, never overlapping: the serialisation the base guard imposes.
            rt.advance(&base, head);
            for k in 0..6 {
                rt.view_mut("balance").unwrap().read(&base, &vec![k], head);
            }
        }
        let v = rt.view("balance").expect("balance");
        assert!(
            v.stats.reads >= 48 && v.stats.evictions > 0,
            "the run must actually fold and evict, or it excludes nothing: {:?}",
            (v.stats.reads, v.stats.evictions)
        );
        assert_eq!(
            v.stats.flights_that_fell_behind, 0,
            "with no append inside a flight there is no suffix to merge, which is why T04 is \
             recorded as a negative experiment rather than built"
        );
    }
}

/// **T04.1 — the bounded, certified deferred merge.**
///
/// A flight folds the prefix ending at its anchor `a`. When the view has advanced to `e > a`
/// by the time it lands, the entry has not seen `(a, e]` and `install` pins it: it serves
/// only `[a, a]`, and the next reader at the frontier reconstructs the whole key again. The
/// cycle-10 baseline says how often that happens — **45,282 pinned installs of 45,681**, with
/// a suffix of four to six epochs — so the pin is not an edge case, it is the ordinary
/// landing, and the reconstruction it forces is the ordinary cost.
///
/// The merge folds `(a, e]` into the landing value and installs at `e`. Three things have to
/// hold for that to be an improvement rather than a bug, and each has a test here:
///
/// 1. **It merges exactly `(a, e]`** — the merged entry equals an independent reconstruction
///    at `e`, and the next read at `e` is a hit rather than a fold.
/// 2. **Nobody's answer changes.** The owner and every same-anchor waiter receive the fold's
///    own value at `a`. The merge writes the view; it does not answer the reader.
/// 3. **A suffix it cannot merge exactly is pinned, not truncated.** Over either cap, or
///    below the base's retention boundary, the landing pins and the refusal is counted under
///    its own cause. A partially merged entry installed at `e` would be a value missing part
///    of its own history under a current stamp — precisely what the pin exists to prevent.
///
/// `MergeCaps::OFF` is the pinned control the measurement is taken against, and it is a value
/// rather than a `cfg` so that one binary can run both arms.
#[cfg(test)]
mod deferred_merge_tests {
    use super::tests::{circuit, FoldBase};
    use super::*;

    /// A view holding one key, advanced to `head`, with a flight open at `head`.
    fn open_flight(rt: &mut Runtime, key: i64, anchor: Epoch) -> FoldTicket {
        match rt.view_mut("balance").expect("balance").begin_read_with(
            &vec![key],
            anchor,
            ReadMode::Alone,
        ) {
            ReadOutcome::Fold(t) => t,
            ReadOutcome::Hit(_) => panic!("a cold key must fold, not hit"),
            ReadOutcome::Join(_) => panic!("`Alone` does not produce a join"),
        }
    }

    /// The setup every case here shares: a base with `KEYS` keys, a view advanced to some
    /// early epoch, a flight opened there, and then the base and the view moved on — so the
    /// flight is late by construction when it lands.
    fn late_landing(caps: MergeCaps, extra_rounds: i128) -> (FoldBase, Runtime, FoldTicket, Epoch) {
        let mut base = FoldBase::new(0);
        for k in 0..4 {
            base.seal(vec![k], 100 + k as i128);
        }
        let early = base.frontier();
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        rt.set_merge_caps(caps);
        rt.advance(&base, early);
        let ticket = open_flight(&mut rt, 0, early);
        for round in 0..extra_rounds {
            for k in 0..4 {
                base.seal(vec![k], 7 + round);
            }
        }
        let late = base.frontier();
        rt.advance(&base, late);
        (base, rt, ticket, early)
    }

    /// **Clause 1.** The merged entry is the value at `e`, and the reader that follows hits.
    #[test]
    fn an_eligible_late_landing_merges_exactly_the_deltas_it_missed() {
        let (base, mut rt, ticket, early) = late_landing(MergeCaps::default(), 3);
        let late = base.frontier();
        let (value, rows) = base.reconstruct(&vec![0], early);

        let v = rt.view_mut("balance").expect("balance");
        v.finish_fold_merging(ticket, value, rows, Some(&base as &dyn Base));
        assert_eq!(v.stats.deferred_merges, 1, "the landing was eligible");
        assert_eq!(
            v.stats.pinned_installs, 0,
            "a merged landing is current, so nothing was pinned"
        );
        assert_eq!(
            v.stats.merge_epochs_merged,
            late - early,
            "the merge must span exactly the suffix the flight missed"
        );

        // The independent fold: what a reader at the frontier is owed.
        let (expected, _) = base.reconstruct(&vec![0], late);

        // The next read at the frontier must be served from the entry, not from a new fold —
        // which is the entire benefit, and is invisible in a value assertion alone.
        let folds_before = v.stats.base_rows_read;
        let got = match v.begin_read_with(&vec![0], late, ReadMode::Alone) {
            ReadOutcome::Hit(a) => a,
            _ => panic!(
                "the read after a merged landing reconstructed. The merge installed at the \
                 frontier precisely so that it would not"
            ),
        };
        assert_eq!(
            got.value, expected,
            "the merged entry is the value at {late}"
        );
        assert_eq!(got.anchor, late);
        assert_eq!(
            v.stats.base_rows_read, folds_before,
            "the hit must have read no base rows"
        );
    }

    /// **Clause 2, the owner.** The reader asked at `a` and is answered at `a`, with the
    /// fold's own value — not the merged one, which is a different question.
    #[test]
    fn the_owner_is_answered_at_its_own_anchor_and_not_at_the_merged_one() {
        let (base, mut rt, ticket, early) = late_landing(MergeCaps::default(), 3);
        let (value, rows) = base.reconstruct(&vec![0], early);
        let (at_late, _) = base.reconstruct(&vec![0], base.frontier());
        assert_ne!(value, at_late, "the fixture must actually move the key");

        let v = rt.view_mut("balance").expect("balance");
        let answer = v.finish_fold_merging(ticket, value, rows, Some(&base as &dyn Base));
        assert_eq!(
            answer.anchor, early,
            "the answer carries the anchor asked for"
        );
        assert_eq!(
            answer.value, value,
            "the owner receives its own fold. Returning the merged value would answer a \
             question nobody asked, at an anchor the caller did not request"
        );
    }

    /// **Clause 2, the waiter.** A reader that joined the flight at the same anchor receives
    /// the same thing the owner does.
    #[test]
    fn a_same_anchor_waiter_receives_the_answer_at_that_anchor() {
        let mut base = FoldBase::new(0);
        for k in 0..4 {
            base.seal(vec![k], 100 + k as i128);
        }
        let early = base.frontier();
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        rt.advance(&base, early);

        let v = rt.view_mut("balance").expect("balance");
        let ticket = match v.begin_read_with(&vec![0], early, ReadMode::Shared) {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the first reader owns the flight"),
        };
        let waiter = match v.begin_read_with(&vec![0], early, ReadMode::Shared) {
            ReadOutcome::Join(w) => w,
            _ => panic!("a second reader at the same anchor must join"),
        };

        for k in 0..4 {
            base.seal(vec![k], 11);
        }
        let late = base.frontier();
        rt.advance(&base, late);

        let (value, rows) = base.reconstruct(&vec![0], early);
        let v = rt.view_mut("balance").expect("balance");
        v.finish_fold_merging(ticket, value, rows, Some(&base as &dyn Base));
        assert_eq!(v.stats.deferred_merges, 1);

        // Published before the wait, so this returns without blocking.
        let joined = waiter
            .wait()
            .expect("the flight published at this waiter's anchor");
        assert_eq!(joined.answer.anchor, early);
        assert_eq!(
            joined.answer.value, value,
            "a waiter is served the flight's answer at its own anchor, whatever the view \
             then installs"
        );
    }

    /// **Clause 3, the epoch cap.** A suffix longer than the cap pins, and says why.
    #[test]
    fn a_suffix_longer_than_the_epoch_cap_pins_and_is_counted_as_such() {
        let caps = MergeCaps {
            max_epochs: 2,
            max_rows: u64::MAX,
        };
        let (base, mut rt, ticket, early) = late_landing(caps, 4);
        let (value, rows) = base.reconstruct(&vec![0], early);
        let v = rt.view_mut("balance").expect("balance");
        v.finish_fold_merging(ticket, value, rows, Some(&base as &dyn Base));

        assert_eq!(v.stats.deferred_merges, 0);
        assert_eq!(v.stats.merges_refused_epochs, 1);
        assert_eq!(v.stats.pinned_installs, 1, "the refusal lands as a pin");
        // Pinned means exactly this: the entry serves its own anchor and nothing above it.
        assert!(
            matches!(
                v.begin_read_with(&vec![0], base.frontier(), ReadMode::Alone),
                ReadOutcome::Fold(_)
            ),
            "a pinned entry must not answer above its own stamp"
        );
    }

    /// **Clause 3, the row cap** — the bound that is not a function of epochs. And the rows a
    /// refused merge really did read are counted, because a refusal that reports zero cost
    /// looks free.
    #[test]
    fn a_suffix_over_the_row_cap_pins_and_still_reports_what_it_read() {
        let caps = MergeCaps {
            max_epochs: u64::MAX,
            max_rows: 2,
        };
        let (base, mut rt, ticket, early) = late_landing(caps, 4);
        let (value, rows) = base.reconstruct(&vec![0], early);
        let v = rt.view_mut("balance").expect("balance");
        v.finish_fold_merging(ticket, value, rows, Some(&base as &dyn Base));

        assert_eq!(v.stats.deferred_merges, 0);
        assert_eq!(v.stats.merges_refused_rows, 1);
        assert_eq!(
            v.stats.merges_refused_epochs, 0,
            "the epoch cap was not the cause"
        );
        assert_eq!(v.stats.pinned_installs, 1);
        assert!(
            v.stats.merge_rows_visited > 0,
            "the walk read rows before it gave up, and a refusal reporting zero cost is one \
             a reader would take for free"
        );
        assert_eq!(
            v.stats.merge_epochs_merged, 0,
            "nothing was merged, so no epochs were merged"
        );
    }

    /// **Clause 3, retention.** `deltas_at` reports an epoch the base no longer holds as
    /// *empty*, which is indistinguishable from an epoch in which nothing happened. Merging
    /// across that boundary would drop real money and report success, so the boundary is
    /// asked for rather than inferred.
    #[test]
    fn a_suffix_below_the_retention_boundary_pins_rather_than_dropping_money() {
        /// A base that has forgotten its early deltas but can still reconstruct.
        struct Compacted {
            inner: FoldBase,
            from: Epoch,
        }
        impl Base for Compacted {
            fn frontier(&self) -> Epoch {
                self.inner.frontier()
            }
            fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
                self.inner.reconstruct(key, anchor)
            }
            fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
                if e < self.from {
                    Vec::new()
                } else {
                    self.inner.deltas_at(e)
                }
            }
            fn deltas_available_from(&self) -> Epoch {
                self.from
            }
        }

        let mut inner = FoldBase::new(0);
        for k in 0..4 {
            inner.seal(vec![k], 100 + k as i128);
        }
        let early = inner.frontier();
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();

        let base = Compacted {
            inner,
            from: early + 2,
        };
        rt.advance(&base, early);
        let ticket = open_flight(&mut rt, 0, early);
        let mut base = base;
        for k in 0..4 {
            base.inner.seal(vec![k], 9);
        }
        let late = base.frontier();
        rt.advance(&base, late);

        let (value, rows) = base.reconstruct(&vec![0], early);
        let v = rt.view_mut("balance").expect("balance");
        v.finish_fold_merging(ticket, value, rows, Some(&base as &dyn Base));

        assert_eq!(v.stats.merges_refused_unavailable, 1);
        assert_eq!(v.stats.deferred_merges, 0);
        assert_eq!(v.stats.pinned_installs, 1);
    }

    /// **The control.** `MergeCaps::OFF` is the pinned policy, so the arm the measurement
    /// compares against is a value in one binary rather than a second build.
    #[test]
    fn merging_off_is_exactly_the_pinned_policy() {
        let (base, mut rt, ticket, early) = late_landing(MergeCaps::OFF, 3);
        let (value, rows) = base.reconstruct(&vec![0], early);
        let v = rt.view_mut("balance").expect("balance");
        let answer = v.finish_fold_merging(ticket, value, rows, Some(&base as &dyn Base));

        assert_eq!(v.stats.deferred_merges, 0);
        assert_eq!(v.stats.pinned_installs, 1);
        assert_eq!(
            v.stats.merge_rows_visited, 0,
            "the control reads no suffix at all"
        );
        assert_eq!(answer.anchor, early);
    }

    /// **Does the merge help or hurt a reader that arrives behind the frontier?**
    ///
    /// Printed rather than asserted: this is the mechanism behind T04.2's measured
    /// regression, and it is a *shape*, not a threshold. Run with
    /// `cargo test -p nilestream-core --lib the_merge_against_the_arrival_gap -- --ignored --nocapture`.
    #[test]
    #[ignore = "a measurement, not a gate"]
    fn the_merge_against_the_arrival_gap() {
        const KEYS: i64 = 200;
        const ROUNDS: i128 = 60;
        for gap in [0u64, 1, 3, 5, 8] {
            let mut line = format!("gap {gap}: ");
            for caps in [MergeCaps::OFF, MergeCaps::default()] {
                let mut base = FoldBase::new(0);
                for k in 0..KEYS {
                    base.seal(vec![k], 100);
                }
                let mut rt = Runtime::install(
                    circuit(Materialize::Demand, Consistency::Snapshot),
                    None,
                    Policy::Lru,
                )
                .unwrap();
                rt.set_merge_caps(caps);
                for round in 0..ROUNDS {
                    for k in 0..KEYS {
                        base.seal(vec![k], 1 + round);
                    }
                    let head = base.frontier();
                    rt.advance(&base, head);
                    let anchor = head.saturating_sub(gap);
                    for k in 0..KEYS {
                        rt.view_mut("balance")
                            .unwrap()
                            .read(&base, &vec![k], anchor);
                    }
                }
                let v = rt.view("balance").unwrap();
                let name = if caps == MergeCaps::OFF {
                    "pinned"
                } else {
                    "merge"
                };
                line.push_str(&format!(
                    "{name} hit {:.1}% rows {}   ",
                    100.0 * v.stats.hits as f64 / v.stats.reads as f64,
                    v.stats.base_rows_read
                ));
            }
            println!("{line}");
        }
    }

    /// A caller with no base cannot merge, and must pin rather than guess. `finish_fold`'s
    /// own signature is this case, and it is the one every existing caller takes.
    #[test]
    fn a_landing_with_no_base_pins() {
        let (base, mut rt, ticket, early) = late_landing(MergeCaps::default(), 3);
        let (value, rows) = base.reconstruct(&vec![0], early);
        let v = rt.view_mut("balance").expect("balance");
        v.finish_fold(ticket, value, rows);
        assert_eq!(v.stats.deferred_merges, 0);
        assert_eq!(v.stats.pinned_installs, 1);
    }

    /// **Ownership is unchanged by any of this.** A landing whose generation was superseded
    /// writes nothing — merged or not — and a merge must not become a second way to write
    /// over an entry a newer flight installed.
    #[test]
    fn a_superseded_landing_merges_nothing_and_writes_nothing() {
        let (base, mut rt, ticket, early) = late_landing(MergeCaps::default(), 3);
        let late = base.frontier();
        let v = rt.view_mut("balance").expect("balance");

        // The owner is superseded: its marker is reaped — a dropped connection, a killed
        // statement — and the next reader through the view takes the key over with a new
        // generation.
        v.force_reap(&vec![0]);
        let newer = match v.begin_read_with(&vec![0], late, ReadMode::Alone) {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the successor owns the key now"),
        };
        let (nv, nr) = base.reconstruct(&vec![0], late);
        v.finish_fold_merging(newer, nv, nr, Some(&base as &dyn Base));
        let after_successor = v.stats.deferred_merges;

        let (value, rows) = base.reconstruct(&vec![0], early);
        let answer = v.finish_fold_merging(ticket, value, rows, Some(&base as &dyn Base));

        assert_eq!(
            answer.anchor, early,
            "a superseded landing still answers its caller"
        );
        assert_eq!(
            v.stats.deferred_merges, after_successor,
            "a landing that owns nothing must not merge: the merge writes the view, and this \
             one has no right to"
        );
        assert!(v.stats.uninstalled_folds >= 1);
    }
}
