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
}

impl ReadStats {
    pub fn miss_rate(&self) -> f64 {
        if self.reads == 0 {
            return 0.0;
        }
        self.misses as f64 / self.reads as f64
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
    ledger: Ledger,
    /// The currency index every wire query uses. The demo schema declares one currency, and a
    /// server that silently answered in whichever currency it found first would be making the
    /// per-currency conservation rule invisible from the outside.
    currency: u32,
    views: Vec<(String, u32)>,
    /// Counted work spent evaluating circuits, so the scan surface's cost is reported
    /// rather than hidden inside a latency number.
    scan_work: u64,
    /// Queries served by evaluating a circuit, and base rows materialised for them.
    ///
    /// **Every one of these is a reconstruction.** The served path evaluates the compiled
    /// circuit over a source scan; the partially materialised view is not consulted, so
    /// there is no resident entry to hit. The counters are reported as what they are rather
    /// than as a hit/miss ratio over a view nothing reads — which is what the CSV's
    /// `miss_rate` column would otherwise be silently reporting.
    served: u64,
    served_rows: u64,
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
    runtime: Option<nilestream_core::rev::Runtime>,
    /// Currencies the base holds. The installed view is keyed `(account, currency)` and a
    /// wire query asking for one account's balance names no currency, so the runtime can
    /// answer only while there is exactly one to name. More than one and the fold answers —
    /// because picking a currency for the caller is how a per-currency conservation rule
    /// becomes invisible from outside.
    currencies: std::collections::BTreeSet<u32>,
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
    /// Epochs applied to the base whose barrier has not yet returned.
    ///
    /// Drained by the daemon after it releases the lock; see [`Serving::take_pending`].
    pending: Vec<Pending>,
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
        let seq = nilestream_ledger::sequencer::Sequencer::open(
            path,
            nilestream_ledger::segment::SyncPolicy::Always,
        )?;
        Ok(DurableSink { seq })
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
            Err(nilestream_ledger::sequencer::Rejected::Duplicate { at_epoch }) => Ok(at_epoch),
            Err(e) => Err(format!("{e:?}")),
        }
    }
}

/// A submitted transaction whose barrier has not yet returned.
type PendingDurable =
    std::sync::mpsc::Receiver<Result<u64, nilestream_ledger::sequencer::Rejected>>;

impl RevEngine {
    /// Build an engine holding `accounts` accounts, each seeded with `postings_per_account`
    /// balanced transfers against a house account.
    ///
    /// Seeded rather than empty because a benchmark against an empty server reports excellent
    /// latencies for queries that return nothing, and because a partial view over a base with
    /// no history has nothing to reconstruct — the miss path, which is the interesting one,
    /// would never run.
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
                rt.advance(&mut ledger, e);
            }
        }

        RevEngine {
            ledger,
            currency: 0,
            views: vec![("__wire_result".to_string(), 2)],
            scan_work: 0,
            served: 0,
            served_rows: 0,
            runtime,
            currencies: std::collections::BTreeSet::from([0]),
            durable: None,
            visible: None,
            pending: Vec::new(),
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
        if let Some(v) = self
            .runtime
            .as_mut()
            .and_then(|rt| rt.view_mut(BALANCE_VIEW))
        {
            v.wipe();
        }
    }

    pub fn head(&self) -> u64 {
        self.ledger.head()
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
            None => self.ledger.head(),
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
    /// `answer_from_view` below is where it is spent, and it answers through the same REV
    /// runtime the phase diagram measures rather than through a cache beside it.
    fn query(
        &mut self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> Result<crate::session::Rows, crate::session::ServeError> {
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
            .and_then(|p| p.account_restriction(ACCT_COL))
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
        // **A report first**, because a fully maintained view answers one without touching
        // the base at all. Refused for every shape that is not one — see `report_from_view`.
        if let Some(p) = planned.as_ref() {
            if let Some(rows) = self.report_from_view(p, circuit, output, anchor) {
                self.served += 1;
                return Ok(rows);
            }
        }
        let sole = planned
            .as_ref()
            .and_then(|p| p.sole_account_filter(ACCT_COL));
        if let (Some(p), Some(acct), ServePath::View) = (
            planned.as_ref(),
            sole,
            serve_path_of(planned.as_ref(), circuit, output),
        ) {
            if let Some(rows) = self.answer_from_view(p, circuit, output, acct, anchor) {
                return Ok(rows);
            }
        }
        // Counted here rather than on entry: a query the maintained view answered is not a
        // read of the scan surface, and adding it to both totals double-counted every one.
        self.served += 1;
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
                self.served_rows += scanned;
                let (folded, w) = folder.finish();
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
                    let (z, w2) = niles_ir::eval::try_run_with(
                        circuit,
                        output,
                        &std::collections::BTreeMap::new(),
                        &given,
                    )
                    .map_err(|e| crate::session::ServeError::Eval(e.to_string()))?;
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
                self.served_rows += sources.values().map(|z| z.len() as u64).sum::<u64>();
                niles_ir::eval::try_run(circuit, output, &sources)
                    .map_err(|e| crate::session::ServeError::Eval(e.to_string()))?
            }
        };
        self.scan_work += work;

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
        Ok(crate::session::Rows {
            columns,
            rows: crate::session::RowSource::Evaluated { z, anchor },
        })
    }

    fn append(&mut self, rows: Vec<Row>, txn_id: &str) -> Result<u64, crate::session::ServeError> {
        let epoch = self.ledger.submit(txn_id, rows).map_err(|e| match e {
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
        // The submission is non-blocking; the token goes to `pending`, and the daemon waits
        // on it *after* releasing the lock and before writing the reply. So the
        // acknowledgement still follows the barrier — the client is told "committed" only
        // once it is — while the lock is held for the apply alone.
        if let Some(sink) = self.durable.as_ref() {
            let token = sink
                .record_pending(txn_id, epoch.to_string().into_bytes())
                .map_err(crate::session::ServeError::NotDurable)?;
            self.pending.push(Pending {
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
        if let Some(rt) = self.runtime.as_mut() {
            rt.advance(&mut self.ledger, epoch);
        }
        if let Some(e) = self.ledger.epochs.get(epoch as usize) {
            for r in &e.rows {
                if let proto_engine::Row::Post(p) = r {
                    self.currencies.insert(p.cur);
                }
            }
        }
        Ok(epoch)
    }

    fn views(&self) -> Vec<(String, u32)> {
        self.views.clone()
    }

    fn take_pending(&mut self) -> Vec<Pending> {
        std::mem::take(&mut self.pending)
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
    fn read_stats(&self) -> (u64, u64, u64, u64, usize) {
        // **Hits are counted now, where they used to be zero by construction.**
        //
        // This returned `(served, 0, served, rows, resident)` with a paragraph explaining
        // that the served path never consults a resident entry — true when it was written,
        // and it made the `miss_rate` column of every benchmark row 1.00 whatever the engine
        // did. A single-account read is now answered by the REV runtime, so its hits and
        // misses are the runtime's, and the scan surface's reads are added to both totals so
        // that a mixed workload's rate is over everything the server answered.
        match self.runtime.as_ref().and_then(|rt| rt.view(BALANCE_VIEW)) {
            Some(v) => {
                let s = &v.stats;
                (
                    s.reads + self.served,
                    s.hits,
                    s.misses + self.served,
                    s.base_rows_read + self.served_rows,
                    v.resident_count() as usize,
                )
            }
            // No runtime: the server has no partial state at all, so nothing is resident
            // and every served read is a reconstruction over the base.
            None => (self.served, 0, self.served, self.served_rows, 0),
        }
    }
}

impl RevEngine {
    /// Attach a durable sink, so every append reaches stable storage before it is
    /// acknowledged. Refuses any policy but `Always`, which the sequencer also refuses.
    pub fn with_durable(mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        self.durable = Some(DurableSink::open(path)?);
        // The base already holds its seeded epochs and they were never written to a sink, so
        // the visible frontier starts where the base is. Everything appended from here earns
        // its visibility by being fsynced.
        self.visible = Some(std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            self.ledger.head(),
        )));
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
        anchor: u64,
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
        if !with_currency && self.currencies.len() != 1 {
            return None;
        }
        let view = self.runtime.as_ref()?.view(BALANCE_VIEW)?;
        if !view.is_full() || view.applied_through() != anchor {
            return None;
        }
        Some(with_currency)
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
        if let Some(p) = planned.as_ref() {
            if self.report_shape(p, circuit, output, anchor).is_some() {
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
        &mut self,
        p: &crate::scan_fold::FoldPlan,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> Option<crate::session::Rows> {
        use niles_ir::value::Value;

        let with_currency = self.report_shape(p, circuit, output, anchor)?;
        let rt = self.runtime.as_ref()?;
        let view = rt.view(BALANCE_VIEW)?;

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
        &mut self,
        p: &crate::scan_fold::FoldPlan,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        acct: u64,
        anchor: u64,
    ) -> Option<crate::session::Rows> {
        use niles_ir::operator::{Agg, Scalar};
        /// Column positions in `postings`: `txn, acct, cur, amt, idem`.
        const ACCT: u16 = 1;
        const CUR: u16 = 2;
        const AMT: u16 = 3;

        if circuit.outputs.get(output) != Some(&p.node) || !p.filters_only() {
            return None;
        }
        if p.aggs() != [(Agg::Sum, Scalar::Column(AMT))] {
            return None;
        }
        // `group by acct` and `group by acct, cur` are the two spellings of "this account's
        // balance"; the first is only the same question while there is one currency to mean.
        let with_currency = match p.group_key() {
            [ACCT] => false,
            [ACCT, CUR] => true,
            _ => return None,
        };
        if self.currencies.len() != 1 {
            return None;
        }
        let cur = *self.currencies.iter().next()?;
        let rt = self.runtime.as_mut()?;
        let view = rt.view_mut(BALANCE_VIEW)?;
        let answered = view.read(&mut self.ledger, &vec![acct as i64, cur as i64], anchor);

        // **The answer must be true at the anchor that was asked for, not merely at least as
        // fresh.** A hit reports the entry's *effective* version, which can be later than the
        // requested anchor — a value that includes writes the caller's snapshot excludes.
        // In this server the write path holds the engine's lock, so the frontier cannot move
        // between `observe` and here and the two are always equal; that is a property of the
        // current concurrency and not of the read, so it is checked rather than relied on. A
        // difference falls back to the fold, which reconstructs at the anchor exactly.
        if answered.anchor != anchor {
            return None;
        }

        let mut columns: Vec<String> = (0..p.width()).map(|i| format!("c{i}")).collect();
        columns.push("anchor".into());

        // **An account the base has never posted to has no balance, and that is not zero.**
        // The absence lattice's distinction, at the layer where it would be quietest to
        // lose: a keyed read answers with a number for a key nothing has ever touched, and a
        // group that does not exist must produce no row at all — which is what the fold and
        // the reference evaluator both do, and what this must agree with.
        if self
            .ledger
            .key_update_count(acct, anchor.min(self.ledger.head()))
            == 0
        {
            return Some(crate::session::Rows {
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
        use niles_ir::value::Value;
        let mut row = vec![Value::Int(acct as i128)];
        if with_currency {
            row.push(Value::Int(cur as i128));
        }
        row.push(Value::Int(answered.value));
        let mut z = niles_ir::eval::ZSet::new();
        z.insert(row, 1);
        Some(crate::session::Rows {
            columns,
            rows: crate::session::RowSource::Evaluated { z, anchor },
        })
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
        let upto = anchor.min(self.ledger.head());
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
                for p in self.ledger.postings_for(a, upto) {
                    f(row(&p));
                }
            }
            None => {
                for e in &self.ledger.epochs {
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
        let head = self.ledger.head();
        let upto = anchor.min(head);
        for e in &self.ledger.epochs {
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
        for p in self
            .ledger
            .postings_for(acct, anchor.min(self.ledger.head()))
        {
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
}

impl ServePath {
    pub fn as_str(self) -> &'static str {
        match self {
            ServePath::Report => "report-from-view",
            ServePath::View => "view",
            ServePath::IndexFold => "index-fold",
            ServePath::Fold => "fold",
            ServePath::Materialise => "materialise",
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
    if p.account_restriction(ACCT_COL).is_some() {
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
}

/// The name of that view inside the runtime.
pub const BALANCE_VIEW: &str = "__balance";

#[cfg(test)]
mod tests {

    /// Compile a wire statement against the daemon's schema, exactly as a session does.
    fn compile(sql: &str) -> niles_lang::lower::Lowered {
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
        let mut e = RevEngine::seeded(60, 3, 20, ViewMode::Demand, EvictionPolicy::Lru);
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
        let mut e = RevEngine::seeded(40, 2, 15, ViewMode::Demand, EvictionPolicy::Lru);
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
        let (z, _) = crate::scan_fold::fold(&plan, base.iter().map(|r| r.as_slice()));
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
        let mut e = RevEngine::seeded(60, 3, 20, ViewMode::Demand, EvictionPolicy::Lru);
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
            p.account_restriction(ACCT_COL)
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
            let mut e = RevEngine::seeded(accounts, 2, 50, ViewMode::Demand, EvictionPolicy::Lru);
            let anchor = e.frontier();
            let lowered = compile(sql);
            let before = e.read_stats().3;
            e.query(&lowered.circuit, "__wire_result", anchor)
                .expect("answers");
            e.read_stats().3 - before
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
        let mut e = RevEngine::seeded(200, 2, usize::MAX, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();
        let lowered = compile("select acct, sum(amt) from postings group by acct");

        let before = e.read_stats().3;
        let served = e
            .query(&lowered.circuit, "__wire_result", anchor)
            .expect("answers");
        let touched = e.read_stats().3 - before;

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
        let mut e = RevEngine::seeded(200, 2, 20, ViewMode::Demand, EvictionPolicy::Lru);
        let anchor = e.frontier();

        // Warm it, so it has resident entries a careless report would happily return.
        let point = compile("select acct, sum(amt) from postings where acct = 7 group by acct");
        for _ in 0..30 {
            let _ = e.query(&point.circuit, "__wire_result", anchor);
        }

        let lowered = compile("select acct, sum(amt) from postings group by acct");
        let before = e.read_stats().3;
        let served = e
            .query(&lowered.circuit, "__wire_result", anchor)
            .expect("answers");
        let touched = e.read_stats().3 - before;

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
        let mut e = RevEngine::seeded(20, 2, 10, ViewMode::Demand, EvictionPolicy::Lru);
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
        let mut e = RevEngine::seeded(200, 2, 50, ViewMode::Demand, EvictionPolicy::Lru);
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
        let (reads, hits, misses, _rows, resident) = e.read_stats();
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
        let (reads, hits, misses, _, resident) = e.read_stats();
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
        let mut e = RevEngine::seeded(20, 2, 10, ViewMode::Demand, EvictionPolicy::Lru);
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
        let mut e = RevEngine::seeded(50, 2, 20, ViewMode::Demand, EvictionPolicy::Lru);
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
        let mut e = RevEngine::seeded(200, 3, 50, ViewMode::Demand, EvictionPolicy::Lru);
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

    /// `read_stats` as a named struct, so a test reads like a claim rather than like a tuple.
    fn stats(e: &RevEngine) -> ReadStats {
        use crate::session::Serving;
        let (reads, hits, misses, rows_touched, resident) = e.read_stats();
        ReadStats {
            reads,
            hits,
            misses,
            rows_touched,
            resident,
        }
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
        let mut e = RevEngine::seeded(50, 1, 25, ViewMode::Demand, EvictionPolicy::Lru);
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

        let mut e = tiny().with_durable(&seg).expect("durable sink");
        assert!(
            e.sealer_stats().is_some(),
            "a durable engine must expose its sealer's counters"
        );

        let before = e.sealer_stats().expect("durable").2;
        let epoch = e
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

        // **`append` no longer waits for the barrier, so this must.**
        //
        // Before T-05 the fsync happened inside `append`, inside the engine's lock, and this
        // test could read `fsyncs` straight afterwards. That is the protocol that changed:
        // the caller applies, takes the token, releases the lock, and *then* waits — which is
        // what the daemon does between framing a reply and writing it. A test that skipped
        // the wait would be asserting on a barrier that had not happened yet.
        let pending = crate::session::Serving::take_pending(&mut e);
        assert_eq!(pending.len(), 1, "one append, one outstanding barrier");
        for p in pending {
            p.wait().expect("the epoch reaches stable storage");
        }
        assert_eq!(
            e.frontier(),
            epoch,
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
        let (mut e, seg) = engine_at("not-visible-yet");
        let before = e.frontier();

        let epoch = e.append(post(1, 1), "v-1").expect("applies");
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

        for p in e.take_pending() {
            p.wait().expect("durable");
        }
        assert_eq!(
            e.frontier(),
            epoch,
            "and only now, after the barrier, is it visible"
        );
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
        let (mut e, seg) = engine_at("never-awaited");
        let before = e.frontier();
        let epoch = e.append(post(2, 2), "v-2").expect("applies");

        // The injection: take the tokens and drop them unread.
        let pending = e.take_pending();
        assert_eq!(pending.len(), 1);
        drop(pending);

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
        let (mut e, seg) = engine_at("no-gap");
        for i in 0..8u64 {
            e.append(post(100 + i, i % 4), &format!("g-{i}"))
                .expect("applies");
        }
        // Wait in submission order, as the daemon does.
        for p in e.take_pending() {
            p.wait().expect("durable");
        }
        let visible = e.frontier();
        assert!(
            visible <= e.ledger.head(),
            "the visible frontier {visible} is beyond the base's head {} — it is naming an \
             epoch whose rows are not applied",
            e.ledger.head()
        );
        assert_eq!(
            visible,
            e.ledger.head(),
            "with every barrier returned, the two must agree"
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
        let (mut e, seg) = engine_at("monotone");
        for i in 0..6u64 {
            e.append(post(200 + i, i), &format!("m-{i}"))
                .expect("applies");
        }
        let mut pending = e.take_pending();
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
                now <= e.ledger.head(),
                "the frontier {now} passed the base's head {}",
                e.ledger.head()
            );
            seen = now;
        }
        assert_eq!(seen, e.ledger.head());
        let _ = std::fs::remove_file(&seg);
    }
}
