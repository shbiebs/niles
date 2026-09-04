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
        match self.seq.submit(nilestream_ledger::sequencer::Txn {
            idem_key: txn_id.to_string(),
            payload,
        }) {
            Ok(e) => Ok(e),
            Err(nilestream_ledger::sequencer::Rejected::Duplicate { at_epoch }) => Ok(at_epoch),
            Err(e) => Err(format!("{e:?}")),
        }
    }
}

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
    fn frontier(&self) -> u64 {
        self.ledger.head()
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
        if let (Some(p), Some(acct)) = (planned.as_ref(), account_predicate(circuit)) {
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
                self.scan(account_predicate(circuit), anchor, |row| {
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
                let sources = match account_predicate(circuit) {
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
        // The durable sink, if one is attached, is what makes the returned epoch mean
        // something: it does not return until `fsync` has. See `DurableSink`.
        if let Some(sink) = self.durable.as_mut() {
            sink.record(txn_id, epoch.to_string().into_bytes())
                .map_err(crate::session::ServeError::NotDurable)?;
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
        Ok(self)
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
fn account_predicate(circuit: &niles_ir::circuit::Circuit) -> Option<u64> {
    use niles_ir::operator::{Op, Scalar, ScalarOp};
    /// The column `postings.acct` occupies in the source schema. Declared order:
    /// `txn, acct, cur, amt, idem`.
    const ACCT: u16 = 1;
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
    let program = format!(
        "{}\nview {BALANCE_VIEW} = sql {{ {BALANCE} }} serve {{ consistency: snapshot, materialize: auto }};\n",
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
        Some(budget as u64),
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
}
