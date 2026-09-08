//! One client session: the path from a wire query to a served answer.
//!
//! This module is where the adoption argument of §6.9 is either true or not. A PostgreSQL
//! client sends a string; that string is parsed as Niles's SQL surface, lowered to the same
//! IR as any other query, verified by the same verifier, and served from the same REV
//! runtime under the same contract. **There is no compatibility layer with its own
//! execution path**, which matters because a second execution path is exactly the seam this
//! thesis argues against everywhere else: two ways to compute an answer is two answers that
//! can disagree.
//!
//! # The session's anchor, and why it exists
//!
//! Every session holds a [`Session::anchor`], and it only ever moves forward. That single
//! `max` is consistency rung 1 — monotonic reads — and it is the cheapest rung on the
//! ladder for a reason: over an append-only base, a session's guarantee that it never sees
//! time run backwards costs one comparison. A session that read the live frontier on every
//! query would give a *weaker* guarantee for the same work, because two queries could
//! straddle nothing at all and still be served out of order under retry.

use crate::pg_wire::{self, Backend, Field, Frontend};
use niles_lang::diagnostics::Severity;

/// What the session can serve from. Kept as a trait so the session can be tested without a
/// running engine, and so the same code serves the in-memory prototype and a durable one.
/// **An applied epoch and the receipt for its barrier.**
///
/// One value, so an append's epoch and the token that says the epoch reached stable storage
/// cannot become separated — which is exactly what happened when the token went to a vector
/// on the engine and the epoch went to the caller (A9-F01).
///
/// `receipt` is `None` when the server has no durable sink: there is no barrier to be after,
/// and `None` says so rather than an empty list that could equally mean "someone else took
/// it".
pub struct Appended {
    pub epoch: u64,
    pub receipt: Option<crate::rev_engine::Pending>,
}

pub trait Serving {
    /// The current visibility frontier.
    fn frontier(&self) -> u64;

    /// **Evaluate a compiled circuit and return its rows.**
    ///
    /// The signature is the finding. It used to be
    /// `read(&mut self, view: &str, key: &[i64], anchor: u64) -> Option<i128>` — one
    /// number, for one key, from a view named by a string — and the implementation ignored
    /// the string, took `key[0]`, and folded `sum(amt)` for currency 0. The session
    /// compiled the client's SQL, verified the circuit, and then threw it away.
    ///
    /// A trait that cannot *accept* a circuit cannot serve one, so the shape of this
    /// method is what made the shortcut invisible: nothing in the type said the answer was
    /// unrelated to the query.
    fn query(
        &self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> Result<Rows, ServeError>;

    /// Append rows as one sealed epoch, returning the epoch and **this append's own
    /// durability receipt**.
    ///
    /// The receipt is the whole of A9-F01. It used to be pushed onto a vector owned by the
    /// *engine*, and the connection loop drained the vector — not its own token — after
    /// `Session::handle` returned. Between one connection's append and its drain, another
    /// connection's drain took the first one's receipt: the first was then acknowledged with
    /// nothing to wait on, while its record's barrier was still in flight, and a barrier that
    /// failed was reported to whichever connection happened to be draining. Visibility was
    /// still gated by the token, so no read observed the epoch; what was wrong was the
    /// acknowledgement, which is the thing a client acts on.
    ///
    /// Returning it makes the ownership a type fact. A caller cannot take another's receipt
    /// because it never has a reference to one.
    fn append(&self, rows: Vec<proto_engine::Row>, txn_id: &str) -> Result<Appended, ServeError>;

    /// The views this server knows, and the scale each one's money column carries.
    fn views(&self) -> Vec<(String, u32)>;

    /// What this server does with an append before returning: `"always"` when every epoch
    /// is on stable storage before it is acknowledged, `"none"` when it is not.
    ///
    /// Asked over the wire so a benchmark reports what the server *is* rather than what its
    /// harness believes. A `durable` row measured against a non-durable append is the
    /// single most common way a durability number is inflated.
    fn durability(&self) -> &'static str;

    /// What the read model did, or zeroes where the target has no partial state.
    ///
    /// A named struct and not a tuple. It was `(u64, u64, u64, u64, usize)`, which every
    /// caller destructured positionally and one caller had already re-wrapped in a struct so
    /// its tests would read like claims; adding the two fallback columns would have made it a
    /// seven-tuple, and a seven-tuple of counters is a place transposition errors live.
    fn read_stats(&self) -> crate::rev_engine::ReadStats;

    /// What the sealer beneath this server has done, when there is one.
    ///
    /// `(epochs_sealed, txns_committed, fsyncs, duplicates_absorbed, max_batch)`. `None` for
    /// a server with no durable sink, which is not the same as all zeroes: a server that
    /// cannot batch and one that has not yet batched are different facts, and a benchmark
    /// reading `max_batch = 0` should be able to tell them apart.
    ///
    /// Exists because `max_batch` is the one number that says whether group commit is
    /// reaching the wire, the sealer has maintained it since it was written, and no surface
    /// could read it.
    fn sealer_stats(&self) -> Option<(u64, u64, u64, u64, u64)> {
        None
    }

    /// **What this server would do with this circuit, right now**, as one short class name.
    ///
    /// On the trait rather than as a free function over the circuit, because one of the
    /// classes is not a property of the circuit at all: whether a report is answered from a
    /// maintained view depends on the view's contract and on how far the write path has
    /// advanced it. `explain` reported `fold` for statements the engine answered without
    /// reading a single base row.
    ///
    /// The default is the circuit's own answer, which is right for a server with no
    /// maintained state to consult.
    fn serve_path_now(
        &self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        _anchor: u64,
    ) -> &'static str {
        crate::rev_engine::serve_path(circuit, output).as_str()
    }
}

/// A served result: column names and the rows, in whatever form the answer already had them.
///
/// `None` is a SQL null and is kept distinct from a zero all the way to the wire, because
/// the whole absence argument is worthless if the last layer collapses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rows {
    pub columns: Vec<String>,
    pub rows: RowSource,
}

/// **Where a served answer's rows are, rather than a copy of them.**
///
/// This used to be `Vec<Vec<Option<String>>>` and nothing else, so answering a `group by
/// acct` meant taking a Z-set the fold had just built and rebuilding it: a `Vec` per row and
/// a `String` per cell, ten thousand times, to hold integers that were already sitting in
/// memory as integers. With the framing costs above it, that was six allocations per row of
/// a reply whose content is a handful of numbers.
///
/// The evaluated path therefore hands over **the Z-set itself**, and the framer reads it
/// where it lies. The text form stays for the diagnostic statements — `nilestream_stats`,
/// the catalog queries — whose cells are genuinely strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowSource {
    /// Rows that are text: the diagnostic and catalog statements.
    Text(Vec<Vec<Option<String>>>),
    /// An evaluated answer, with the anchor each row is stamped with.
    ///
    /// The anchor is a column of the reply and not of the Z-set, because it is not part of
    /// what the query denotes: it is the moment the answer is true at, which a dispute needs
    /// and a `select` did not ask for.
    Evaluated {
        z: niles_ir::eval::ZSet,
        anchor: u64,
    },
}

impl Rows {
    /// How many rows a client will receive — the number `CommandComplete` reports.
    ///
    /// A Z-set weight above one is a row that appears that many times, so this is a sum of
    /// weights and not a count of entries. Collapsing them would be a `distinct` nobody
    /// wrote.
    pub fn len(&self) -> usize {
        match &self.rows {
            RowSource::Text(r) => r.len(),
            RowSource::Evaluated { z, .. } => {
                z.values().filter(|w| **w > 0).map(|w| *w as usize).sum()
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The rows as text, the way a client decodes them.
    ///
    /// For tests and for the in-process callers that want to compare answers rather than
    /// bytes. Deliberately not what the wire path uses: rendering here would put back exactly
    /// the per-cell allocations this type exists to avoid.
    pub fn text(&self) -> Vec<Vec<Option<String>>> {
        match &self.rows {
            RowSource::Text(r) => r.clone(),
            RowSource::Evaluated { z, anchor } => {
                let mut out = Vec::new();
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
                        out.push(cells);
                    }
                }
                out
            }
        }
    }

    /// A text answer, for the diagnostic statements and the test doubles.
    pub fn text_rows(columns: Vec<String>, rows: Vec<Vec<Option<String>>>) -> Rows {
        Rows {
            columns,
            rows: RowSource::Text(rows),
        }
    }
}

/// Why a query or an append could not be served.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServeError {
    /// The circuit did not evaluate — a non-terminating fixpoint, most usefully.
    Eval(String),
    /// The ledger refused the rows because the set does not conserve, or names a hold
    /// that is not there.
    Rejected(String),
    /// This identity already committed. **Not the same failure**, and not the same code: a
    /// driver retries an integrity violation differently from a duplicate, and a retried
    /// payment that came back as "unbalanced" would be retried again.
    Duplicate(String),
    /// The rows were sealed and the sync did not return. The epoch is **not** published and
    /// the client is told so, because a caller that wanted durability and received an `Ok`
    /// would stop keeping its own copy and find out at the worst possible moment.
    NotDurable(String),
    /// **The query would have added two currencies together.**
    ///
    /// Its own variant and not a `Rejected`, because the caller's remedy is different and
    /// specific: group by the currency, or name one. The message carries the codes the base
    /// actually holds, so the remedy can be written without a second query.
    CrossCurrency(Vec<u32>),
}

impl ServeError {
    /// The SQLSTATE a PostgreSQL client will interpret.
    ///
    /// Real codes rather than a single generic one: a client's retry logic reads this, and
    /// `40001` (serialization failure) means "retry" while `23505` (unique violation) means
    /// "you already did this". Answering `XX000` to both would make an idempotent retry
    /// look like an outage.
    pub fn sqlstate(&self) -> &'static str {
        match self {
            ServeError::Eval(_) => "22000", // data exception
            // The set does not conserve: an integrity constraint, and the constraint is
            // the one this whole system is about.
            ServeError::Rejected(_) => "23000", // integrity_constraint_violation
            ServeError::Duplicate(_) => "23505", // unique_violation: a repeated identity
            ServeError::NotDurable(_) => "58030", // io_error
            // `22000` is the data exception the sum *is*: the operands are not comparable.
            // Deliberately not `23000` — nothing about the stored data violates a constraint,
            // and telling a client its ledger is corrupt when its query is malformed sends it
            // looking in the wrong place.
            ServeError::CrossCurrency(_) => "22000", // data_exception
        }
    }
    /// **The `ERROR:` line a client shows before it shows anything else.**
    ///
    /// It was `"this query could not be evaluated"` for every failure, which is true and
    /// tells a caller nothing: a query refused for adding two currencies and a query whose
    /// fixpoint did not terminate are different mistakes with different remedies, and the
    /// distinction sat in `DETAIL` where a terse client does not print it.
    pub fn headline(&self) -> &'static str {
        match self {
            ServeError::Eval(_) => "this query could not be evaluated",
            ServeError::Rejected(_) => "this query was refused by the ledger",
            ServeError::Duplicate(_) => "this identity has already committed",
            ServeError::NotDurable(_) => "this epoch is not on stable storage",
            ServeError::CrossCurrency(_) => "this `sum` would add amounts in different currencies",
        }
    }

    /// The detail line, owned because one variant composes it.
    pub fn detail(&self) -> std::borrow::Cow<'_, str> {
        match self {
            ServeError::Eval(m)
            | ServeError::Rejected(m)
            | ServeError::Duplicate(m)
            | ServeError::NotDurable(m) => std::borrow::Cow::Borrowed(m),
            ServeError::CrossCurrency(held) => {
                let list: Vec<String> = held.iter().map(|c| c.to_string()).collect();
                std::borrow::Cow::Owned(format!(
                    "the base holds currencies [{}] and this `sum(amt)` groups without `cur`, so it would add amounts in different currencies into one number. Add `cur` to the `group by`, or restrict with `cur = k`. `conserve per (txn, cur)` is quantified per currency exactly so this sum never has to be taken.",
                    list.join(", ")
                ))
            }
        }
    }
}

/// Whether a `Bind` asked for any column in binary.
///
/// The message is: portal name, statement name, `n` parameter format codes, `m` parameters,
/// then `k` **result** format codes. A `k` of zero means every column is text; a `k` of one
/// applies that code to every column; otherwise there is one per column.
///
/// This server negotiates per *session* rather than per column, because every column of a
/// served answer is an integer kind today and a mixed request has no shape it could take
/// advantage of. A request in which *any* result column is binary therefore turns the session
/// binary, and `RowDescription` reports what was actually chosen — so a client is never told
/// one thing and sent another, whatever it asked for.
fn binds_binary(body: &[u8]) -> bool {
    let mut at = 0usize;
    let _portal = pg_wire::get_cstr(body, &mut at);
    let _statement = pg_wire::get_cstr(body, &mut at);
    let read_i16 = |b: &[u8], at: &mut usize| -> Option<i16> {
        if *at + 2 > b.len() {
            return None;
        }
        let v = i16::from_be_bytes([b[*at], b[*at + 1]]);
        *at += 2;
        Some(v)
    };
    let read_i32 = |b: &[u8], at: &mut usize| -> Option<i32> {
        if *at + 4 > b.len() {
            return None;
        }
        let v = i32::from_be_bytes([b[*at], b[*at + 1], b[*at + 2], b[*at + 3]]);
        *at += 4;
        Some(v)
    };
    let Some(n_param_formats) = read_i16(body, &mut at) else {
        return false;
    };
    for _ in 0..n_param_formats.max(0) {
        if read_i16(body, &mut at).is_none() {
            return false;
        }
    }
    let Some(n_params) = read_i16(body, &mut at) else {
        return false;
    };
    for _ in 0..n_params.max(0) {
        match read_i32(body, &mut at) {
            // -1 is a null parameter and carries no bytes.
            Some(len) if len >= 0 => at += len as usize,
            Some(_) => {}
            None => return false,
        }
    }
    let Some(n_result_formats) = read_i16(body, &mut at) else {
        return false;
    };
    let mut any = false;
    for _ in 0..n_result_formats.max(0) {
        match read_i16(body, &mut at) {
            Some(1) => any = true,
            Some(_) => {}
            None => return false,
        }
    }
    any
}

/// The scale a money column is reported at.
///
/// The demo schema declares `currency usd { scale: 2 }`, and until column kinds reach the
/// wire (T-23) there is one. Named rather than written as `2` in three places, so that when
/// the schema's own scale arrives there is one line to change and a reader can see there was
/// an assumption here.
pub const MONEY_SCALE: u32 = 2;

pub struct Session {
    pub user: String,
    pub database: String,
    /// Rung 1: this only ever increases.
    pub anchor: u64,
    pub in_transaction: bool,
    pub failed: bool,
    /// Rows accumulated inside a `BEGIN` … `COMMIT`, sealed as **one** epoch at commit.
    ///
    /// Held rather than appended per statement, because a transaction whose statements
    /// sealed independently would have an epoch in which half of it had happened — which is
    /// precisely what atomicity means here, and what the old `begin`/`commit` pair did not
    /// provide: it flipped a flag and said so in a notice.
    pending: Vec<proto_engine::Row>,
    /// The identities of the statements in the open transaction, joined into the sealed
    /// set's own idempotency key so a retried transaction is refused as a whole.
    pending_ids: Vec<String>,
    /// Prepared statements and portals, keyed by the schema epoch they were compiled at.
    pub plans: crate::extended::PlanCache,
    /// The schema text this session's queries are compiled against.
    pub schema: String,
    /// **Whether this session's results are sent in binary.**
    ///
    /// Off by default and off for every client that does not ask, which is what keeps `psql`
    /// and the conformance transcript byte-identical. The extended protocol negotiates it per
    /// column in `Bind`, as the protocol specifies; the simple protocol has no place to carry
    /// a format code, so `set nilestream.binary = on` is the extension — documented as one,
    /// and named so that nobody mistakes it for something PostgreSQL has.
    pub binary: bool,
    /// **Compiled circuits, keyed by statement text.**
    ///
    /// The simple query path compiled every statement afresh — parse, resolve, typecheck,
    /// lower, verify — and `callgrind` says what that costs: **177,064 instructions against
    /// 7,693 to actually serve a point read**, so 96% of the daemon's own work on that
    /// workload was recompiling a statement it had just compiled. The wall-clock share is
    /// much smaller (a round trip is ~120µs against ~22µs of compilation) which is why this
    /// was worth measuring before believing either number: "the compiler is 0.5% of the
    /// budget" and "the compiler is 96% of the engine" are both true, of different budgets.
    ///
    /// Keyed by `(schema, statement text)` and not by an epoch, because **a session compiles
    /// against one schema for its whole life**: `Session::schema` is set at construction and
    /// never assigned. A key carrying the schema makes that a property of the cache rather
    /// than a fact a reader has to go and check, and leaves the door open for a session whose
    /// schema can move.
    compiled: std::collections::HashMap<(u64, String), niles_lang::lower::Lowered>,
    /// Compile order, so a full cache evicts its oldest entry instead of emptying itself.
    ///
    /// **It used to empty itself, and that made the cache worse than no cache for the one
    /// workload it mattered on.** A cache keyed by arbitrary client text is unbounded memory
    /// with a friendly name, so a limit is right; clearing all 256 entries to admit the
    /// 257th is what turned a working set of 1,000 statements into 1,000 compilations *per
    /// pass* — 2,000 for two passes over the same thousand, measured, where 1,000 is the
    /// number a cache exists to produce. The old note said "a session that issues more than
    /// this many distinct ones is not one a plan cache was going to help", which is true of
    /// a session that issues 10,000 unrelated statements once each and false of every
    /// session whose working set is a little larger than the limit — and the second is the
    /// ordinary shape of generated SQL.
    ///
    /// First-in-first-out and not LRU: one `VecDeque` and no bookkeeping on the hit path,
    /// where LRU would touch a recency structure on every *hit*. FIFO holds a working set
    /// that fits and degrades gracefully for one that does not, which is the whole
    /// difference from clearing.
    compile_order: std::collections::VecDeque<(u64, String)>,
    /// How many plans were evicted to make room. Was `compiled_cleared`, a count of
    /// wholesale emptyings.
    compiled_evicted: u64,
    pub compile_hits: u64,
    pub compile_misses: u64,
    pub queries_served: u64,
    /// **Durability receipts for the appends this session has made and not yet answered.**
    ///
    /// One `Session` belongs to one connection, so a receipt placed here can be taken by
    /// exactly one caller: the connection that made the append. The receipts used to live in
    /// a `Mutex<Vec<Pending>>` on the *engine*, drained by whichever connection called
    /// `take_pending` next — so a connection could acknowledge its insert while its own
    /// barrier was still in flight on somebody else's thread, and a failed barrier could be
    /// reported as an error to a connection that had only read (A9-F01).
    ///
    /// Emptied by [`Session::take_receipts`] once per wire message, immediately before the
    /// reply for that message is written.
    receipts: Vec<crate::rev_engine::Pending>,
    /// **The declared currencies and their scales, compiled once per schema epoch.**
    ///
    /// `Some((key, table))` once this session has resolved its schema; the `key` is
    /// `schema_key()`, the same identity the plan cache uses, so a `set schema` invalidates
    /// both by the same rule. `Some((key, None))` records that the schema at that key does
    /// *not* resolve — an answer worth caching, because the alternative is re-parsing a
    /// broken schema on every statement to rediscover that it is broken.
    ///
    /// Before this, `insert` called `declared_currencies(&self.schema)`, which runs
    /// `parse_program` and `resolve_program` over the whole schema text. 61% of the insert
    /// path's instructions were spent re-deriving a table the session had already compiled
    /// (F-68).
    currencies: Option<(u64, Option<std::collections::BTreeMap<u32, u32>>)>,
    /// How many times this session has parsed its schema. A `nilestream_stats` column,
    /// because the defect above was invisible from every surface the server had.
    pub schema_parses: u64,
}

/// How many compiled statements one session holds before the oldest is evicted.
///
/// "before the cache is emptied" until cycle 8's T-13, when the policy stopped being
/// clear-all and became FIFO. The comment outlived the code by a cycle (F-74).
const PLAN_CACHE_LIMIT: usize = 256;

impl Session {
    pub fn new(user: String, database: String, schema: String) -> Session {
        Session {
            user,
            database,
            anchor: 0,
            in_transaction: false,
            failed: false,
            pending: Vec::new(),
            pending_ids: Vec::new(),
            plans: crate::extended::PlanCache::new(),
            schema,
            binary: false,
            compiled: std::collections::HashMap::new(),
            compile_order: std::collections::VecDeque::new(),
            compiled_evicted: 0,
            compile_hits: 0,
            compile_misses: 0,
            queries_served: 0,
            receipts: Vec::new(),
            currencies: None,
            schema_parses: 0,
        }
    }

    /// The identity of the schema a plan was compiled against.
    ///
    /// A hash of the text rather than a ledger epoch. The extended path uses
    /// `engine.frontier()` as its `schema_epoch`, which invalidates every prepared statement
    /// on every *append* — conservative, and useless as a cache, since the schema does not
    /// change when a posting is written. That is left alone here and reported rather than
    /// changed under an efficiency task, because `Prepared::schema_epoch` is the mechanism a
    /// future migration story is meant to hang on.
    fn schema_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.schema.hash(&mut h);
        h.finish()
    }

    /// **Compile a statement, or return the circuit compiled for it earlier.**
    ///
    /// One entry point, used by the simple path and by the extended one, so the two cannot
    /// come to disagree about what a statement means — which is the compatibility-layer
    /// failure this crate's own module docs argue against, in miniature.
    /// How many compiled plans this session holds. **Bounded at `PLAN_CACHE_LIMIT`**, FIFO,
    /// since cycle 8's T-13; this said "**Unbounded**" and pointed at a test that no longer
    /// exists (F-74). One `Lowered` for E16's point statement is 2,281 live bytes, so a full
    /// cache is about 584 KB per session — measured, not estimated, and the reason the limit
    /// is not the memory question. What the *key* is remains open (LC-33).
    pub fn compiled_len(&self) -> usize {
        self.compiled.len()
    }

    fn compile_cached(&mut self, sql: &str) -> Result<&niles_lang::lower::Lowered, Vec<Backend>> {
        let key = (self.schema_key(), sql.to_string());
        if self.compiled.contains_key(&key) {
            self.compile_hits += 1;
            return Ok(&self.compiled[&key]);
        }
        self.compile_misses += 1;
        let program = format!(
            "{}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};\n",
            self.schema
        );
        let (prog, mut diags) = niles_lang::parser::parse_program(&program);
        let (cat, rd) = niles_lang::resolve::resolve_program(&prog, self.anchor);
        diags.extend(rd);
        let (_r, td) = niles_lang::typecheck::check_program(&prog, &cat);
        diags.extend(td);
        if diags.has_errors() {
            let first = diags
                .sorted()
                .into_iter()
                .find(|d| d.severity == Severity::Error)
                .expect("has_errors implies one exists");
            let detail = first.notes.first().cloned();
            self.failed = true;
            return Err(vec![pg_wire::diagnostic_error(
                first.code,
                &first.msg,
                detail.as_deref(),
            )]);
        }
        let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
        if ld.has_errors() {
            // The lowering's own code and message, not a generic one. A client told "this
            // query has no lowering" cannot tell a `limit` it cannot read from a stage that
            // does not exist.
            let first = ld
                .sorted()
                .into_iter()
                .find(|d| d.severity == Severity::Error)
                .expect("has_errors implies one exists");
            let detail = first.notes.first().cloned();
            self.failed = true;
            return Err(vec![pg_wire::diagnostic_error(
                first.code,
                &first.msg,
                detail.as_deref(),
            )]);
        }
        // **The verifier stands between the compiler and the engine on this path too**, and
        // it runs before the plan is cached — so a circuit that does not verify is never
        // stored, and a cache hit is a hit on something that passed the gate. A client cannot
        // be given a way around it, or the trusted base has a hole in it shaped like a
        // network socket.
        let vr = niles_ir::verify::verify(&lowered.circuit);
        if !vr.is_ok() {
            let first = vr
                .violations
                .first()
                .map(|v| v.msg.clone())
                .unwrap_or_default();
            self.failed = true;
            return Err(vec![pg_wire::diagnostic_error(
                "IR000",
                "the compiled circuit did not verify",
                Some(&first),
            )]);
        }
        while self.compiled.len() >= PLAN_CACHE_LIMIT {
            match self.compile_order.pop_front() {
                Some(old) => {
                    self.compiled.remove(&old);
                    self.compiled_evicted += 1;
                }
                // The order queue and the map disagree, which cannot happen through this
                // function. Emptying is the safe response and is the old behaviour.
                None => {
                    self.compiled.clear();
                    break;
                }
            }
        }
        self.compile_order.push_back(key.clone());
        Ok(self.compiled.entry(key).or_insert(lowered))
    }

    /// The transaction-status byte a `ReadyForQuery` carries.
    pub fn status(&self) -> u8 {
        if self.failed {
            b'E'
        } else if self.in_transaction {
            b'T'
        } else {
            b'I'
        }
    }

    /// Advance the session anchor. Monotone by construction.
    pub fn observe(&mut self, frontier: u64) -> u64 {
        self.anchor = self.anchor.max(frontier);
        self.anchor
    }

    /// **The declared currencies and their scales for this session's current schema.**
    ///
    /// Compiled on first use at a given schema epoch and reused until that epoch changes.
    /// `None` when the schema does not resolve, which is cached too: a broken schema is a
    /// fact about the schema, not about the statement that happened to notice.
    fn currencies(&mut self) -> Option<std::collections::BTreeMap<u32, u32>> {
        let key = self.schema_key();
        match &self.currencies {
            Some((k, table)) if *k == key => table.clone(),
            _ => {
                self.schema_parses += 1;
                let table = declared_currencies(&self.schema);
                self.currencies = Some((key, table.clone()));
                table
            }
        }
    }

    /// **The durability receipts for the appends this message made.**
    ///
    /// Called once per wire message, after `handle` and before the reply is written. Its
    /// contract is the one the engine-wide list could not offer: what comes back was put
    /// here by *this* session, so waiting on it waits for this connection's own barriers and
    /// for no others (A9-F01).
    pub fn take_receipts(&mut self) -> Vec<crate::rev_engine::Pending> {
        std::mem::take(&mut self.receipts)
    }

    /// Handle one frontend message, producing the messages to send back.
    ///
    /// **Timed into `lockstats::STATEMENT`**, which is the inner of the two boundaries the
    /// slow-read table could not see past. Its four timestamps live inside
    /// `answer_from_view`; everything this function does around that — splitting the string,
    /// compiling or finding the plan, framing rows — is outside them, and the "unaccounted"
    /// column that purported to cover it was the truncation residual of those same four
    /// reads (A9-F07). One `Instant::now()` pair per message, on a path that already frames
    /// a reply.
    pub fn handle(&mut self, msg: Frontend, engine: &dyn Serving) -> Vec<Backend> {
        let began = std::time::Instant::now();
        let out = self.handle_inner(msg, engine);
        crate::lockstats::STATEMENT.record(0, began.elapsed().as_nanos() as u64);
        out
    }

    fn handle_inner(&mut self, msg: Frontend, engine: &dyn Serving) -> Vec<Backend> {
        match msg {
            Frontend::Query(sql) => {
                // **A simple query string may hold several statements**, and PostgreSQL
                // executes them in order with one `ReadyForQuery` at the end. The server
                // used to treat the whole string as one query, so
                // `psql -c "begin; insert …; commit"` — the ordinary way anyone scripts a
                // transaction — came back as a syntax error on the word `begin`. The
                // transaction machinery existed and there was no way to reach it from a
                // real client.
                let mut out = Vec::new();
                for stmt in split_statements(&sql) {
                    self.queries_served += 1;
                    out.extend(self.query(&stmt, engine));
                    // PostgreSQL abandons the rest of a multi-statement string after an
                    // error, which is what makes `begin; …; commit` safe to send in one
                    // go: a failing statement does not leave the following `commit` to
                    // seal a half-built transaction.
                    if self.failed {
                        break;
                    }
                }
                if out.is_empty() {
                    out.push(Backend::EmptyQueryResponse);
                }
                out.push(Backend::ReadyForQuery(self.status()));
                out
            }
            Frontend::Terminate => Vec::new(),
            // **The extended protocol, served by the plan cache that was written for it.**
            //
            // `extended.rs` is a complete module with eleven tests and a written answer to
            // the epoch-invalidation question — a plan is valid exactly while no schema
            // epoch has occurred after the one it was compiled at, which is a comparison of
            // two integers. It was reachable from nothing: the decoder discarded the
            // message body, so the statement text never arrived, and every `Parse` was
            // answered with a refusal citing a design question the module beside it had
            // already answered.
            Frontend::Extended(tag, body) => self.extended(tag, &body, engine),
            Frontend::Password(_) => vec![
                Backend::AuthenticationOk,
                Backend::ReadyForQuery(self.status()),
            ],
            Frontend::Unknown(t) => {
                self.failed = true;
                vec![
                    pg_wire::unsupported(&format!("message type `{}`", t as char), "unrecognised"),
                    Backend::ReadyForQuery(self.status()),
                ]
            }
            Frontend::Startup { .. } | Frontend::SslRequest => Vec::new(),
        }
    }

    fn query(&mut self, sql: &str, engine: &dyn Serving) -> Vec<Backend> {
        let trimmed = sql.trim().trim_end_matches(';').trim();
        if trimmed.is_empty() {
            return vec![Backend::EmptyQueryResponse];
        }
        let lower = trimmed.to_ascii_lowercase();

        // Transaction control, and the honest limits of it. A `begin`/`commit` pair over a
        // *base* is not a table transaction: the base is appended to inside a `txn` block,
        // and there is no rollback of a sealed epoch. So the session tracks the state a
        // client expects and says plainly what it does not provide.
        match lower.as_str() {
            "begin" | "start transaction" => {
                self.in_transaction = true;
                self.pending.clear();
                self.pending_ids.clear();
                return vec![
                    Backend::NoticeResponse {
                        message:
                            "reads in this session are served at a session anchor rather than \
                                  from a snapshot held open; writes are buffered and sealed as one \
                                  epoch at COMMIT"
                                .into(),
                    },
                    Backend::CommandComplete("BEGIN".into()),
                ];
            }
            // **One epoch, or none.** The buffered rows are sealed together, so there is no
            // moment at which half the transaction is visible. This used to flip a flag and
            // return, which is why the notice above had to apologise for it.
            "commit" | "end" => {
                self.in_transaction = false;
                let rows = std::mem::take(&mut self.pending);
                let ids = std::mem::take(&mut self.pending_ids);
                if rows.is_empty() {
                    self.failed = false;
                    return vec![Backend::CommandComplete("COMMIT".into())];
                }
                let txn = ids.join("+");
                return match engine.append(rows, &txn) {
                    Ok(applied) => {
                        let epoch = applied.epoch;
                        // **This session's own receipt.** Held here, waited on by this
                        // connection before its reply is written, and reachable by nobody
                        // else (A9-F01).
                        self.receipts.extend(applied.receipt);
                        self.observe(epoch);
                        self.failed = false;
                        vec![Backend::CommandComplete(format!("COMMIT {epoch}"))]
                    }
                    Err(e) => {
                        self.failed = true;
                        vec![pg_wire::sqlstate_error(
                            e.sqlstate(),
                            "the transaction did not commit",
                            Some(&e.detail()),
                        )]
                    }
                };
            }
            // A rollback discards the buffer. Nothing was sealed, so there is nothing to
            // compensate — which is the whole reason the rows are held rather than appended.
            "rollback" | "abort" => {
                self.in_transaction = false;
                self.failed = false;
                let n = self.pending.len();
                self.pending.clear();
                self.pending_ids.clear();
                return vec![Backend::CommandComplete(format!("ROLLBACK {n}"))];
            }
            _ => {}
        }

        // **`set nilestream.binary = on|off`: the simple protocol's only way to ask.**
        //
        // The extended protocol carries a result format code per column in `Bind`, which is
        // where a format belongs and is what every driver uses. The simple protocol has no
        // such field, so a session setting is the extension — named `nilestream.` so that
        // nobody mistakes it for something PostgreSQL has, and answered with the same
        // `SET`/`ShowResponse` shape a client already knows.
        if let Some(rest) = lower.strip_prefix("set nilestream.binary") {
            let on = rest.contains("on") || rest.contains("true") || rest.contains('1');
            self.binary = on;
            return vec![Backend::CommandComplete("SET".into())];
        }
        if lower == "show nilestream.binary" {
            let mut buf = Vec::new();
            pg_wire::encode_into(
                &mut buf,
                &Backend::DataRow(vec![Some(if self.binary { "on" } else { "off" }.into())]),
            );
            return vec![
                Backend::RowDescription(vec![Field::text("nilestream.binary")]),
                Backend::Raw(buf),
                Backend::CommandComplete("SHOW".into()),
            ];
        }

        // **`explain <statement>`: what this server would do to answer it, before it does.**
        //
        // The four serve classes differ by three orders of magnitude, and the only way to
        // find out which one a statement took used to be to measure it — so a query that
        // fell off a cliff (`where acct = k and cur = 0` did; `group by acct having
        // acct = k` did) looked exactly like one that did not. The class comes from
        // `rev_engine::serve_path`, which is the function `query` itself branches on, so
        // this cannot become a second opinion about the engine.
        if lower.starts_with("explain ") {
            let inner = trimmed[..].split_at("explain ".len()).1.trim();
            let lowered = match self.compile_cached(inner) {
                Ok(l) => l,
                Err(e) => return e,
            };
            // The engine's answer, not the circuit's. A report served from a fully
            // maintained view is decided by the view's contract and by how far it has been
            // advanced, and `serve_path` — which reads the circuit alone — reported `fold`
            // for statements the engine answered without reading a base row.
            let path = engine.serve_path_now(&lowered.circuit, "__wire_result", engine.frontier());
            let rows = vec![
                vec![Some("serve path".to_string()), Some(path.to_string())],
                vec![
                    Some("what it costs".into()),
                    Some(crate::rev_engine::ServePath::describe_class(path).into()),
                ],
                vec![
                    Some("circuit nodes".into()),
                    Some(lowered.circuit.nodes.len().to_string()),
                ],
            ];
            let n = rows.len();
            let mut buf = Vec::new();
            for r in &rows {
                pg_wire::encode_into(&mut buf, &Backend::DataRow(r.clone()));
            }
            return vec![
                Backend::RowDescription(vec![Field::text("property"), Field::text("value")]),
                Backend::Raw(buf),
                Backend::CommandComplete(format!("EXPLAIN {n}")),
            ];
        }

        // Two introspection queries, because a client that cannot list what it can read is
        // not usable from `psql`.
        if lower == "\\d" || lower.starts_with("select * from nilestream_views") {
            let mut rows: Vec<Backend> = vec![Backend::RowDescription(vec![
                Field::text("view"),
                Field::int8("scale"),
            ])];
            for (name, scale) in engine.views() {
                rows.push(Backend::DataRow(vec![Some(name), Some(scale.to_string())]));
            }
            let n = rows.len() - 1;
            rows.push(Backend::CommandComplete(format!("SELECT {n}")));
            return rows;
        }
        // Whether this server's appends are durable, asked rather than assumed. The
        // benchmark used to hard-code `false` for this target with a comment explaining
        // that the read side is an in-memory demo — which was true, and meant a `durable`
        // row could be *measured* against a non-durable append with nothing but a comment
        // standing between that and a fabricated durability number.
        // The read-model counters, so a benchmark can record the miss rate beside the
        // latency. A parity result at a zero miss rate says a warm view is fast; one at a
        // real miss rate says *reconstruction* is, which is the claim the thesis makes —
        // and the CSV had `n/a` in that column because nothing could ask.
        if lower.starts_with("select") && lower.contains("nilestream_stats") {
            let s = engine.read_stats();
            return vec![
                Backend::RowDescription(vec![
                    Field::int8("reads"),
                    Field::int8("hits"),
                    Field::int8("misses"),
                    Field::int8("rows_touched"),
                    Field::int8("resident"),
                    // **The two columns `BLOCKED-fallback-rate` was raised about.** A keyed
                    // read that consulted the view and then discarded its answer is a cost
                    // the server pays and could not be asked about: 87.4% of keyed reads
                    // took it under concurrent writers, and no surface said so.
                    Field::int8("view_answers"),
                    Field::int8("fallbacks"),
                    // **The two bounded-by-nothing structures, asked about.** A residency
                    // budget that bounds values and not their metadata, and a declared
                    // idempotency window that reached no crate below the compiler, were both
                    // invisible: the only surface for either was a memory profile of the
                    // whole process.
                    Field::int8("view_metadata_keys"),
                    Field::int8("idem_window_keys"),
                    // **How many times this session has parsed its own schema.**
                    //
                    // One per schema epoch is the contract. It was one per `INSERT`, and no
                    // surface could see it: the front end ran on the serving path and the
                    // only way to notice was a profiler (F-68).
                    Field::int8("schema_parses"),
                    // **The absence lattice's fourth state, reported.**
                    //
                    // `pending_joins` counts keyed reads that found a reconstruction already
                    // in flight at their own exact anchor and shared it. It was zero by
                    // construction while the fold ran inside the view lock — there was no
                    // moment at which a second reader could arrive, because no second reader
                    // could run — and `MISMATCH-pending-unreachable` stood against every
                    // sentence describing joining as a mechanism this runtime had.
                    //
                    // The other three are what the split costs and what it refuses.
                    // `uninstalled_folds`: exact answers deliberately not written to the
                    // view, because a flight at another anchor owned the key or the
                    // completion was superseded. `pinned_installs`: reconstructions that
                    // landed below the frontier the view had already applied.
                    // `flights_refused`: reads the bounded flight table had no room for,
                    // which folded anyway.
                    Field::int8("pending_joins"),
                    Field::int8("uninstalled_folds"),
                    Field::int8("pinned_installs"),
                    Field::int8("flights_refused"),
                    // **A counter that reads zero and a counter that is absent are different
                    // claims, and only one of them can be checked.** `deferred_merges` was
                    // absent from every surface while the thesis described the mechanism it
                    // counts (A10-08). `waiters_refused` is the *other* capacity refusal:
                    // reporting it under `flights_refused` is a refusal nobody can act on.
                    Field::int8("deferred_merges"),
                    Field::int8("waiters_refused"),
                    // How a join ended. A retried join is not an error and is not free.
                    Field::int8("joins_answered"),
                    Field::int8("joins_retried"),
                    // **The two anchor gaps, kept apart.** The first is the lag a read
                    // started with; the second is the lag it ended with. Their difference is
                    // the only part the fold itself caused, and one number could not say
                    // which — the measurement the merge decision turns on (A10-11).
                    Field::int8("gap_at_begin_total"),
                    Field::int8("gap_at_finish_total"),
                    Field::int8("gap_at_finish_max"),
                    Field::int8("flights_behind_at_begin"),
                    Field::int8("flights_that_fell_behind"),
                ]),
                Backend::DataRow(vec![
                    Some(s.reads.to_string()),
                    Some(s.hits.to_string()),
                    Some(s.misses.to_string()),
                    Some(s.rows_touched.to_string()),
                    Some(s.resident.to_string()),
                    Some(s.view_answers.to_string()),
                    Some(s.fallbacks.to_string()),
                    Some(s.view_metadata_keys.to_string()),
                    Some(s.idem_window_keys.to_string()),
                    Some(self.schema_parses.to_string()),
                    Some(s.pending_joins.to_string()),
                    Some(s.uninstalled_folds.to_string()),
                    Some(s.pinned_installs.to_string()),
                    Some(s.flights_refused.to_string()),
                    Some(s.deferred_merges.to_string()),
                    Some(s.waiters_refused.to_string()),
                    Some(s.joins_answered.to_string()),
                    Some(s.joins_retried.to_string()),
                    Some(s.gap_at_begin_total.to_string()),
                    Some(s.gap_at_finish_total.to_string()),
                    Some(s.gap_at_finish_max.to_string()),
                    Some(s.flights_behind_at_begin.to_string()),
                    Some(s.flights_that_fell_behind.to_string()),
                ]),
                Backend::CommandComplete("SELECT 1".into()),
            ];
        }
        // **The sealer's own counters, and the engine mutex's.**
        //
        // `max_batch` is the diagnosis in one number: the sequencer batches to 4,096 and is
        // tested at sixteen concurrent submitters, but the daemon holds one mutex across
        // this whole method, so submitters reach `submit` one at a time and the drain loop
        // always finds an empty queue. A `max_batch` of 1 under load means group commit is
        // built and unreachable; `txns_per_fsync` is the same fact as a ratio.
        //
        // The lock columns are the other half. `hold_p50` on a durable workload should sit
        // in the device's `fsync` band, because the fsync happens inside the section; if it
        // does, the write path's ceiling is one transaction per barrier and no amount of
        // concurrency will move it.
        if lower.starts_with("select") && lower.contains("nilestream_sealer") {
            let (epochs, txns, fsyncs, dups, max_batch) =
                engine.sealer_stats().unwrap_or((0, 0, 0, 0, 0));
            let durable = engine.sealer_stats().is_some();
            let (acq, wp50, wp99, wmax, hp50, hp99, hmax, htotal) =
                crate::lockstats::ENGINE_LOCK.snapshot();
            // **The view mutex, which nothing measured.** The base's longest wait is under
            // 2 ms on the reference host while a mixed workload's read maximum is 12-13 ms,
            // so the tail is not where the instrumented lock is. These columns are the other
            // candidate, made a number instead of an argument.
            let (vacq, vwp50, vwp99, vwmax, vhp50, vhp99, vhmax, vhtotal) =
                crate::lockstats::VIEW_LOCK.snapshot();
            let per_fsync = if fsyncs == 0 {
                0.0
            } else {
                txns as f64 / fsyncs as f64
            };
            return vec![
                Backend::RowDescription(vec![
                    Field::text("durable"),
                    Field::int8("epochs_sealed"),
                    Field::int8("txns_committed"),
                    Field::int8("fsyncs"),
                    Field::int8("duplicates_absorbed"),
                    Field::int8("max_batch"),
                    Field::text("txns_per_fsync"),
                    Field::int8("lock_acquisitions"),
                    Field::int8("lock_wait_p50_us"),
                    Field::int8("lock_wait_p99_us"),
                    Field::int8("lock_wait_max_us"),
                    Field::int8("lock_hold_p50_us"),
                    Field::int8("lock_hold_p99_us"),
                    Field::int8("lock_hold_max_us"),
                    Field::int8("lock_hold_total_us"),
                    Field::int8("view_acquisitions"),
                    Field::int8("view_wait_p50_us"),
                    Field::int8("view_wait_p99_us"),
                    Field::int8("view_wait_max_us"),
                    Field::int8("view_hold_p50_us"),
                    Field::int8("view_hold_p99_us"),
                    Field::int8("view_hold_max_us"),
                    Field::int8("view_hold_total_us"),
                ]),
                Backend::DataRow(vec![
                    Some(if durable { "yes" } else { "no" }.to_string()),
                    Some(epochs.to_string()),
                    Some(txns.to_string()),
                    Some(fsyncs.to_string()),
                    Some(dups.to_string()),
                    Some(max_batch.to_string()),
                    Some(format!("{per_fsync:.2}")),
                    Some(acq.to_string()),
                    Some(wp50.to_string()),
                    Some(wp99.to_string()),
                    Some(wmax.to_string()),
                    Some(hp50.to_string()),
                    Some(hp99.to_string()),
                    Some(hmax.to_string()),
                    Some(htotal.to_string()),
                    Some(vacq.to_string()),
                    Some(vwp50.to_string()),
                    Some(vwp99.to_string()),
                    Some(vwmax.to_string()),
                    Some(vhp50.to_string()),
                    Some(vhp99.to_string()),
                    Some(vhmax.to_string()),
                    Some(vhtotal.to_string()),
                ]),
                Backend::CommandComplete("SELECT 1".into()),
            ];
        }
        // **Every histogram this process keeps, with a phase boundary.**
        //
        // `ENGINE_LOCK` and `VIEW_LOCK` were process-global and never cleared, so a sweep
        // that printed them after each level printed that level *plus every level before
        // it*: a maximum that can only rise, and a p99 weighted by the lightest phase
        // (F-75). `select nilestream_lockstats reset` zeroes all six after reading, so the
        // next level's table is the next level's — the same contract
        // `nilestream_slow_reads reset` has had since cycle 8.
        //
        // Nested: `wire` (a message decoded to its reply written) contains `statement`
        // (`Session::handle`), which contains `base` — reported whole and again split into
        // `base_read` and `base_write` — and `view`. A reader can
        // subtract to get the framing and the barrier wait, which is what the slow-read
        // table's residual column was wrongly believed to give.
        if lower.starts_with("select") && lower.contains("nilestream_lockstats") {
            // **Six scopes, and two of them are the same lock in its two modes.** `base` is
            // the aggregate every prior cycle read, kept so its meaning does not change under
            // a reader; `base_read` and `base_write` are the split the design question of
            // cycle 10 needs, because one histogram over a shared mode and an exclusive mode
            // reports a p99 that belongs to neither (A10-08 / F-10-02).
            //
            // A caution that travels with the rows: summing overlapping shared holds is
            // reader-lock-time, not exclusive occupancy. N readers holding for one microsecond
            // each add N microseconds to `hold_total` while occupying the lock for one, so a
            // ratio of the two totals is not a statement about which mode causes a tail.
            let scopes: [(&str, &crate::lockstats::LockStats); 6] = [
                ("base", &crate::lockstats::ENGINE_LOCK),
                ("base_read", &crate::lockstats::BASE_READ),
                ("base_write", &crate::lockstats::BASE_WRITE),
                ("view", &crate::lockstats::VIEW_LOCK),
                ("statement", &crate::lockstats::STATEMENT),
                ("wire", &crate::lockstats::WIRE),
            ];
            let mut rows = Vec::new();
            for (name, st) in scopes {
                let (acq, wp50, wp99, wmax, hp50, hp99, hmax, htotal) = st.snapshot();
                rows.push(Backend::DataRow(vec![
                    Some(name.to_string()),
                    Some(acq.to_string()),
                    Some(wp50.to_string()),
                    Some(wp99.to_string()),
                    Some(wmax.to_string()),
                    Some(hp50.to_string()),
                    Some(hp99.to_string()),
                    Some(hmax.to_string()),
                    Some(htotal.to_string()),
                ]));
            }
            if lower.contains("reset") {
                for (_, st) in scopes {
                    st.reset();
                }
            }
            let n = rows.len();
            let mut out = vec![Backend::RowDescription(vec![
                Field::text("scope"),
                Field::int8("acquisitions"),
                Field::int8("wait_p50_us"),
                Field::int8("wait_p99_us"),
                Field::int8("wait_max_us"),
                Field::int8("hold_p50_us"),
                Field::int8("hold_p99_us"),
                Field::int8("hold_max_us"),
                Field::int8("hold_total_us"),
            ])];
            out.extend(rows);
            out.push(Backend::CommandComplete(format!("SELECT {n}")));
            return out;
        }
        // **The slowest keyed reads, with their parts.** The histograms above are
        // aggregates, and a read maximum of 12-13 ms against a p99 of 246 us is a handful of
        // reads: an aggregate cannot say whether that handful waited for the base, waited
        // for the view, or waited for nothing this process can see — which is the difference
        // between a lock to fix and a scheduler to stop blaming the engine for.
        // `select nilestream_slow_reads reset` empties the table after reading it, so the
        // next level's table is the next level's.
        if lower.starts_with("select") && lower.contains("nilestream_slow_reads") {
            let mut rows = Vec::new();
            for (i, t) in crate::lockstats::SLOW_READS.snapshot().iter().enumerate() {
                rows.push(Backend::DataRow(vec![
                    Some(i.to_string()),
                    Some(t.total_us.to_string()),
                    Some(t.base_wait_us.to_string()),
                    Some(t.view_wait_us.to_string()),
                    Some(t.view_hold_us.to_string()),
                    Some(t.view_wait2_us.to_string()),
                    Some(t.view_hold2_us.to_string()),
                    Some(t.fold_us.to_string()),
                    Some(t.outcome.as_str().to_string()),
                    Some(t.gap_begin.to_string()),
                    Some(t.gap_finish.to_string()),
                    // **Rounding, and the slivers between phases — nothing else.** This
                    // column was called `unaccounted_us` and described as "what the engine
                    // cannot see: the wire, the framing, and whatever the scheduler did
                    // between them". It is none of those. Every timestamp it is computed
                    // from is taken inside `answer_from_view`, so the six named phases
                    // partition the total up to (a) the truncation of six microsecond
                    // divisions and (b) the two un-timed hand-offs between them — the
                    // instruction between `decided` and the fold's start, and the one
                    // between the install hold ending and `view_done`. Cycle 8's
                    // attribution quoted this column as evidence that everything outside
                    // the view wait was under 2 µs; it was evidence of integer division
                    // (A9-F07), and it was computed against three phases while three more
                    // existed unmeasured (A10-08), which is why it is not evidence of
                    // anything on its own.
                    //
                    // The wire and the framing are measured now, by `STATEMENT` and `WIRE`
                    // in `select nilestream_lockstats`, which have boundaries that actually
                    // contain them.
                    Some(
                        t.total_us
                            .saturating_sub(
                                t.base_wait_us
                                    + t.view_wait_us
                                    + t.view_hold_us
                                    + t.fold_us
                                    + t.view_wait2_us
                                    + t.view_hold2_us,
                            )
                            .to_string(),
                    ),
                ]));
            }
            if lower.contains("reset") {
                crate::lockstats::SLOW_READS.reset();
            }
            let n = rows.len();
            let mut out = vec![Backend::RowDescription(vec![
                Field::int8("rank"),
                Field::int8("total_us"),
                Field::int8("base_wait_us"),
                Field::int8("view_wait_us"),
                Field::int8("view_hold_us"),
                Field::int8("view_wait2_us"),
                Field::int8("view_hold2_us"),
                Field::int8("fold_us"),
                Field::text("outcome"),
                Field::int8("gap_begin"),
                Field::int8("gap_finish"),
                Field::int8("rounding_us"),
            ])];
            out.extend(rows);
            out.push(Backend::CommandComplete(format!("SELECT {n}")));
            return out;
        }
        if lower.starts_with("select") && lower.contains("nilestream_durability") {
            let mode = engine.durability();
            return vec![
                Backend::RowDescription(vec![Field::text("durability")]),
                Backend::DataRow(vec![Some(mode.to_string())]),
                Backend::CommandComplete("SELECT 1".into()),
            ];
        }
        if lower.starts_with("select") && lower.contains("nilestream_frontier") {
            let f = engine.frontier();
            self.observe(f);
            return vec![
                Backend::RowDescription(vec![
                    Field::int8("frontier"),
                    Field::int8("session_anchor"),
                ]),
                Backend::DataRow(vec![Some(f.to_string()), Some(self.anchor.to_string())]),
                Backend::CommandComplete("SELECT 1".into()),
            ];
        }

        // **A write.** `INSERT INTO postings VALUES (txn, acct, cur, amt)` becomes a
        // sealed epoch, and inside a transaction it is buffered until `COMMIT`. The old
        // path wrapped the statement in a view and rejected it, so the wire surface was
        // read-only and the §6.9 claim covered half a database.
        if lower.starts_with("insert") {
            return self.insert(trimmed, engine);
        }
        if lower.starts_with("update") || lower.starts_with("delete") {
            self.failed = true;
            return vec![pg_wire::sqlstate_error(
                "0A000",
                "`update` and `delete` are not legal against a ledger",
                Some(
                    "history is the authority: a fact that can be edited is not evidence, and a \
                     reconstruction over an edited base is not a reproduction. Append a \
                     compensating entry.",
                ),
            )];
        }

        // The anchor is observed *before* compiling, and the ordering is not arbitrary: a
        // cached circuit was compiled at some earlier anchor, so the cache is sound only
        // because a lowering does not depend on one. `Catalog::epoch` is carried and never
        // read by `lower`; `as_of` takes a literal epoch. `a_cached_plan_does_not_depend_on
        // _the_anchor_it_was_compiled_at` holds that, because it is the assumption the whole
        // cache rests on.
        let anchor = self.observe(engine.frontier());

        // The real path: compile the client's SQL as Niles, against this session's schema —
        // or take the circuit compiled for it earlier. `compile_cached` runs the verifier
        // before it stores anything, so a hit is a hit on a circuit that passed the gate.
        //
        // Serving it means **evaluating that circuit**, which is the whole of F-16: the
        // three lines this replaced picked a view by name, scraped integers out of the query
        // text with a digit scanner, and asked the engine for `sum(amt)` on the first of
        // them. The compiler ran, the verifier ran, and neither had any bearing on the
        // answer.
        let (served, named) = match self.compile_cached(trimmed) {
            Ok(lowered) => {
                let served = engine.query(&lowered.circuit, "__wire_result", anchor);
                // **The column names come from the lowering**, which is the only place that
                // knows them: a circuit carries indices, so the engine can only name columns
                // positionally. Taking them from `Lowered::schemas` means a client sees the
                // names it wrote rather than `c0`, `c1`.
                let named: Vec<String> = lowered
                    .circuit
                    .outputs
                    .get("__wire_result")
                    .and_then(|id| lowered.schemas.get(id))
                    .cloned()
                    .unwrap_or_default();
                (served, named)
            }
            Err(e) => return e,
        };
        let rows = match served {
            Ok(r) => r,
            Err(e) => {
                self.failed = true;
                return vec![pg_wire::sqlstate_error(
                    e.sqlstate(),
                    e.headline(),
                    Some(&e.detail()),
                )];
            }
        };
        let mut rows = rows;
        if named.len() + 1 == rows.columns.len() {
            rows.columns = named;
            rows.columns.push("anchor".into());
        }

        // **The format each column is sent in, chosen once and reported once.** A description
        // that announced text and then sent eight binary bytes would desynchronise every
        // client that believed it, so the same list builds both.
        let fields: Vec<Field> = rows
            .columns
            .iter()
            .map(|c| {
                // Money as `numeric`, never `float8`. Exactness that survived the type
                // system must survive the wire.
                let mut f = if c == "anchor" {
                    Field::int8("anchor")
                } else {
                    Field::numeric(c)
                };
                if self.binary {
                    f.format = if f.type_oid == 20 {
                        pg_wire::Format::BinaryInt8
                    } else {
                        // The scale a client needs to read the digits back. Reported in the
                        // type modifier as PostgreSQL does — `atttypmod` for `numeric` is
                        // `((precision << 16) | scale) + 4` — so a driver sees a declared
                        // `numeric(38, s)` rather than an unconstrained one.
                        f.type_modifier = ((38i32 << 16) | MONEY_SCALE as i32) + 4;
                        pg_wire::Format::BinaryNumeric(MONEY_SCALE)
                    };
                }
                f
            })
            .collect();
        let formats: Vec<pg_wire::Format> = fields.iter().map(|f| f.format).collect();
        let description = Backend::RowDescription(fields);
        let n = rows.len();

        // **The rows are framed once, into one buffer.**
        //
        // The description and the completion stay ordinary messages either side of it: an
        // `Execute` filters the description out because the client already has it from
        // `Describe`, and a reply that fused the three would break that and be harder to read
        // in a capture. What is removed is the per-row cost — a `Vec` for the row, a `String`
        // per cell, and `encode`'s two vectors per message — for a reply whose content is
        // integers that were already integers.
        let framed = match rows.rows {
            // **Handed over, not rendered.** The writer turns the Z-set into bytes a bounded
            // buffer at a time, so the peak memory of a reply is a constant rather than a
            // function of how many rows it has. This used to build the whole reply here: a
            // hundred-thousand-row answer was a hundred-thousand-row allocation before the
            // first byte reached the socket.
            RowSource::Evaluated { z, anchor } => {
                Backend::Rows(pg_wire::RowBlock { z, anchor, formats })
            }
            // The diagnostic statements: a handful of rows whose cells are text.
            RowSource::Text(text) => {
                let mut buf = Vec::new();
                for r in &text {
                    pg_wire::encode_into(&mut buf, &Backend::DataRow(r.clone()));
                }
                Backend::Raw(buf)
            }
        };
        vec![
            description,
            framed,
            Backend::CommandComplete(format!("SELECT {n}")),
        ]
    }

    /// Serve one extended-protocol message.
    ///
    /// The messages are the standard ones and the state machine is the standard one: a
    /// `Parse` compiles and caches, a `Bind` makes a portal, an `Execute` serves it, and a
    /// `Sync` ends the implicit transaction and emits `ReadyForQuery`. Only `Sync` emits
    /// `ReadyForQuery`, which is the rule that distinguishes the extended protocol from the
    /// simple one and the one a client notices immediately if it is broken.
    fn extended(&mut self, tag: u8, body: &[u8], engine: &dyn Serving) -> Vec<Backend> {
        use crate::extended::ExtError;
        let mut at = 0usize;
        match tag {
            // Parse: statement name, query, parameter type OIDs.
            b'P' => {
                let name = pg_wire::get_cstr(body, &mut at);
                let sql = pg_wire::get_cstr(body, &mut at);
                let epoch = engine.frontier();
                // The fields are not known until the query is compiled, and compiling it
                // here is what makes a `Describe` before `Execute` answerable.
                match self.compile(&sql) {
                    Ok(fields) => {
                        let n = sql.matches('$').count();
                        self.plans.parse(&name, &sql, epoch, fields, n);
                        vec![Backend::ParseComplete]
                    }
                    Err(b) => {
                        self.failed = true;
                        vec![b]
                    }
                }
            }
            // Bind: portal, statement, parameters.
            b'B' => {
                let portal = pg_wire::get_cstr(body, &mut at);
                let statement = pg_wire::get_cstr(body, &mut at);
                // **The result format codes, which used to be skipped.** `Bind` carries, at
                // its end, either no codes (every column text), one code (applied to every
                // column), or one per column. The body was read for two strings and the rest
                // discarded, so a driver that asked for binary was answered in text and
                // would have read eight bytes of `int8` as eight bytes of digits.
                //
                // Read per the protocol; a per-column list is honoured as far as it goes and
                // any column beyond it is text, which is the protocol's own rule.
                self.binary = binds_binary(body);
                match self.plans.bind(&portal, &statement, Vec::new(), 0, false) {
                    Ok(()) => vec![Backend::BindComplete],
                    Err(e) => {
                        self.failed = true;
                        vec![e.to_backend()]
                    }
                }
            }
            b'D' => {
                let _kind = if body.is_empty() { b'S' } else { body[0] };
                at = 1;
                let name = pg_wire::get_cstr(body, &mut at);
                match self.plans.describe(&name) {
                    Ok(fields) => vec![Backend::RowDescription(fields)],
                    Err(e) => {
                        self.failed = true;
                        vec![e.to_backend()]
                    }
                }
            }
            b'E' => {
                let portal = pg_wire::get_cstr(body, &mut at);
                let sql = match self.plans.portal(&portal) {
                    Ok(p) => {
                        let stmt = p.statement.clone();
                        match self.plans.lookup(&stmt, engine.frontier()) {
                            Ok(prep) => prep.sql.clone(),
                            // A stale plan is **recompiled**, not refused: the client's
                            // prepared name keeps working across a migration, which is the
                            // entire point of having prepared it.
                            Err(Some(sql)) => sql,
                            Err(None) => {
                                self.failed = true;
                                return vec![ExtError::UnknownStatement(stmt).to_backend()];
                            }
                        }
                    }
                    Err(e) => {
                        self.failed = true;
                        return vec![e.to_backend()];
                    }
                };
                self.plans.note_execution(&portal);
                self.queries_served += 1;
                // The rows only: an `Execute` does not re-send `RowDescription`, which the
                // client already got from `Describe`.
                self.query(&sql, engine)
                    .into_iter()
                    .filter(|m| !matches!(m, Backend::RowDescription(_)))
                    .collect()
            }
            b'S' => vec![Backend::ReadyForQuery(self.status())],
            b'C' => {
                let kind = if body.is_empty() { b'S' } else { body[0] };
                at = 1;
                let name = pg_wire::get_cstr(body, &mut at);
                if kind == b'P' {
                    self.plans.close_portal(&name);
                } else {
                    self.plans.close_statement(&name);
                }
                vec![Backend::CloseComplete]
            }
            b'H' => Vec::new(), // Flush: everything is written eagerly.
            other => {
                self.failed = true;
                vec![pg_wire::unsupported(
                    &format!("extended message `{}`", other as char),
                    "not part of the protocol's message set",
                )]
            }
        }
    }

    /// Compile a statement far enough to know its output columns, without serving it.
    fn compile(&mut self, sql: &str) -> Result<Vec<Field>, Backend> {
        let program = format!(
            "{}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};\n",
            self.schema
        );
        let (prog, mut diags) = niles_lang::parser::parse_program(&program);
        let (cat, rd) = niles_lang::resolve::resolve_program(&prog, self.anchor);
        diags.extend(rd);
        let (_r, td) = niles_lang::typecheck::check_program(&prog, &cat);
        diags.extend(td);
        let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
        diags.extend(ld);
        if diags.has_errors() {
            let first = diags
                .sorted()
                .into_iter()
                .find(|d| d.severity == Severity::Error)
                .expect("has_errors implies one exists");
            return Err(pg_wire::diagnostic_error(
                first.code,
                &first.msg,
                first.notes.first().map(|x| x.as_str()),
            ));
        }
        let mut fields: Vec<Field> = lowered
            .circuit
            .outputs
            .get("__wire_result")
            .and_then(|id| lowered.schemas.get(id))
            .map(|cols| cols.iter().map(|c| Field::numeric(c)).collect())
            .unwrap_or_default();
        fields.push(Field::int8("anchor"));
        Ok(fields)
    }

    /// `INSERT INTO postings VALUES (txn, acct, cur, amt), (…)` — the write path.
    ///
    /// Parsed here rather than through the lowering because an insert is not a query and has
    /// no circuit: the IR describes read models. The statement's shape is narrow on purpose
    /// and the narrowness is *stated* — a wider parser would be inventing a DML surface the
    /// language has not specified, and thesis §11.3's rule is to narrow publicly.
    fn insert(&mut self, sql: &str, engine: &dyn Serving) -> Vec<Backend> {
        // The declared currencies and their scales — **from the session's cache, compiled
        // once per schema epoch.** The ingress check below needs the set and the amount
        // parser needs each code's scale, and this used to obtain both by running
        // `parse_program` and `resolve_program` over the *entire schema text* on every
        // `INSERT`. Measured on E16's `oltp` statement with callgrind: 74,500 of the insert
        // path's 113,500 instructions, 61%, spent re-deriving a table the session had
        // already compiled and holds a key for (F-68).
        //
        // The refusals are unchanged, and that is the point of caching rather than eliding:
        // 22023 still fires when the schema does not resolve — now from the cached absence
        // instead of a fresh parse — and 22003 and 0A000 still act on the statement.
        let declared = match self.currencies() {
            Some(d) => d,
            None => {
                self.failed = true;
                return vec![pg_wire::sqlstate_error(
                    "22023",
                    "this session's schema does not resolve, so no currency can be checked",
                    Some(
                        "an `insert` is refused rather than admitted while the declared \
                         currencies are unknown.",
                    ),
                )];
            }
        };

        let rows = match parse_insert(sql, &declared) {
            // **An amount the currency cannot hold, refused as that** and not as a syntax
            // error. `22003` is `numeric_value_out_of_range`: the statement's shape is fine and
            // its value is not, which is a different thing for a caller to be told and sends
            // them somewhere else to fix it.
            Err(r) => {
                self.failed = true;
                let written = r.written.clone();
                let digits = written.split_once('.').map(|(_, f)| f.len()).unwrap_or(0);
                return vec![pg_wire::sqlstate_error(
                    "22003",
                    &format!(
                        "`{written}` has more fractional digits than currency {} declares",
                        r.currency
                    ),
                    Some(&format!(
                        "the schema declares scale {} for that currency, so it holds {} \
                         fractional digit(s); `{written}` has {digits}. The amount is refused \
                         rather than rounded — rounding would move money silently, and the \
                         amount you wrote is the amount you meant. Write it in minor units, or \
                         to the declared scale.",
                        r.declared, r.declared
                    )),
                )];
            }
            Ok(None) => {
                self.failed = true;
                return vec![pg_wire::sqlstate_error(
                    "0A000",
                    "this `insert` is outside the supported form",
                    Some(
                        "the form is `INSERT INTO postings VALUES (txn, acct, cur, amt)[, …]`. \
                         `amt` is minor units, or a decimal at the currency's declared scale. \
                         Anything else is refused rather than partly understood.",
                    ),
                )];
            }
            Ok(Some(r)) => r,
        };
        if rows.is_empty() {
            return vec![Backend::CommandComplete("INSERT 0 0".into())];
        }

        // **The schema's currency premise, enforced at the wire.**
        //
        // Contribution 4 says a well-typed program cannot mismatch currencies, and the
        // compiler discharges that for programs. It said nothing about *data*, and every
        // benchmark row is data: an `insert` naming currency 999 against a schema declaring
        // only `usd` was accepted, and `sum(amt) group by acct` then served 324 + 500 = "824"
        // as one account's balance. A conservation rule quantified over currencies is
        // vacuous for a currency nobody declared.
        //
        // Refused here rather than deeper because this is the boundary the premise enters at:
        // the ledger's `Cur` is a `u32` and cannot be narrowed without changing the base's
        // type, and the engine below has no schema. `22023` is `invalid_parameter_value`.
        let offending = rows.iter().find_map(|r| match r {
            proto_engine::Row::Post(p) if !declared.contains_key(&p.cur) => Some(p.cur),
            _ => None,
        });
        if let Some(cur) = offending {
            self.failed = true;
            let names: Vec<String> = declared.keys().map(|c| c.to_string()).collect();
            return vec![pg_wire::sqlstate_error(
                "22023",
                &format!("currency {cur} is not declared by this schema"),
                Some(&format!(
                    "declared currency codes are [{}] — a currency's code is its position among the schema's `currency` declarations, counting from zero. Declare it in the schema rather than inserting it: a currency with no declaration has no scale, so its amounts have no meaning and `conserve per (txn, cur)` cannot hold over it.",
                    names.join(", ")
                )),
            )];
        }

        let n = rows.len();
        // The identity: the transaction numbers in the statement. An insert with no
        // identity would not be idempotent, and a retried insert over a wire that dropped
        // an acknowledgement is the ordinary case rather than the exotic one.
        let txn = rows
            .iter()
            .map(|r| match r {
                proto_engine::Row::Post(p) => p.txn.to_string(),
                _ => "x".into(),
            })
            .collect::<Vec<_>>()
            .join("-");

        if self.in_transaction {
            self.pending.extend(rows);
            self.pending_ids.push(txn);
            return vec![Backend::CommandComplete(format!("INSERT 0 {n}"))];
        }
        match engine.append(rows, &txn) {
            Ok(applied) => {
                // This statement's receipt, on this session, for this connection to wait on.
                self.receipts.extend(applied.receipt);
                // **The session does not observe the epoch it just wrote.** `append` returns
                // the *applied* epoch, before its barrier has returned; raising the anchor to
                // it here meant that when the barrier then failed — and `serve` correctly
                // answered `58030` instead of a commit tag — the session was still anchored
                // past the visible frontier, and its next read found the rows in the base and
                // served them. Read-your-own-failed-write, on the rung the thesis names
                // read-your-writes.
                //
                // The anchor is raised from `frontier()` instead, which moves only when a
                // barrier returns. On a durable engine that is one round trip later than it
                // used to be; on a volatile one the two are the same number.
                self.observe(engine.frontier());
                vec![Backend::CommandComplete(format!("INSERT 0 {n}"))]
            }
            Err(e) => {
                self.failed = true;
                vec![pg_wire::sqlstate_error(
                    e.sqlstate(),
                    "the insert did not commit",
                    Some(&e.detail()),
                )]
            }
        }
    }
}

/// Split a simple-query string into statements at top-level semicolons.
///
/// Quotes and parentheses are tracked, because a `;` inside a string literal or a value
/// list is not a statement boundary and splitting there would cut a query in half — which
/// is worse than not splitting at all, since the halves might each parse.
fn split_statements(sql: &str) -> Vec<String> {
    let (mut out, mut cur) = (Vec::new(), String::new());
    let (mut in_str, mut depth) = (false, 0i32);
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                // A doubled quote inside a literal is an escaped quote, not a close.
                if in_str && chars.peek() == Some(&'\'') {
                    cur.push(c);
                    cur.push(chars.next().expect("peeked"));
                    continue;
                }
                in_str = !in_str;
                cur.push(c);
            }
            '(' if !in_str => {
                depth += 1;
                cur.push(c);
            }
            ')' if !in_str => {
                depth -= 1;
                cur.push(c);
            }
            ';' if !in_str && depth <= 0 => {
                if !cur.trim().is_empty() {
                    out.push(cur.trim().to_string());
                }
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// The rows an `INSERT INTO postings VALUES (…)` names.
///
/// `None` for anything outside the form. Deliberately not lenient: a partly-understood
/// insert would put rows in the ledger that do not match what was written, and the ledger is
/// the one place in this system where that cannot be corrected by an update.
/// **The currency codes this schema declares, in declaration order.**
///
/// The wire carries a currency as an integer (`Cur = u32`) and the language declares one by
/// name (`currency usd { scale: 2 }`), so something has to relate the two. The rule is the
/// narrowest one that invents no syntax: **a declared currency's code is its position in the
/// schema, counting from zero.** A schema declaring only `usd` therefore admits code `0` and
/// nothing else, which is exactly what the seeded base has always used.
///
/// Order comes from the declaration's span and not from `HashMap` iteration, because a code
/// that depended on hash order would differ between runs of the same binary on the same
/// schema — and a currency code is written into the ledger, where it is permanent.
///
/// Returns `None` if the schema does not resolve; the caller then refuses rather than
/// admitting everything, because "the schema is unreadable" is not a reason to accept a
/// currency no schema declared.
/// The idempotency window the schema declares on its ledger's `IdemKey` column, in epochs.
///
/// **In epochs, because the ledger has no other clock** — a wall-clock window is refused by
/// the compiler (NL0217), so anything that reaches here is already in the unit the engine
/// enforces. `None` means the schema did not compile or declares no ledger with an
/// `IdemKey`, in which case both idempotency indexes keep every identity, which is what they
/// did unconditionally before T-05 (cycle 8).
/// **The idempotency window the wire relation declares, in epochs.**
///
/// `Ok(None)` means no relation declares one. `Err` means two do and they disagree, which
/// used to be resolved by taking whichever the `HashMap` yielded first: with the standard
/// hasher that is a per-process choice, so the same binary on the same schema ran with
/// different windows on different starts (F-71). A window is a durability-adjacent contract
/// and cannot be decided by a hash seed.
pub fn declared_idem_window(schema: &str) -> Result<Option<u64>, String> {
    let (prog, d) = niles_lang::parser::parse_program(schema);
    if d.has_errors() {
        return Ok(None);
    }
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    if rd.has_errors() {
        return Ok(None);
    }
    // Ordered by name so the *diagnostic* is stable too: an error message whose two named
    // relations swap between runs is the same defect one level up.
    let mut declared: Vec<(&str, u64)> = Vec::new();
    let mut names: Vec<&String> = cat.relations.keys().collect();
    names.sort();
    for name in names {
        let r = &cat.relations[name];
        for c in &r.columns {
            if let Some(niles_lang::resolve::IdemWindow::Epochs(n)) = c.idem_window {
                declared.push((name.as_str(), n));
            }
        }
    }
    match declared.as_slice() {
        [] => Ok(None),
        [(_, n)] => Ok(Some(*n)),
        many => {
            if many.iter().all(|(_, n)| *n == many[0].1) {
                return Ok(Some(many[0].1));
            }
            Err(format!(
                "the schema declares more than one idempotency window and they disagree: {}. \
                 One daemon holds one window, and picking one of them by hash-map order made \
                 the same binary run with different windows on different starts. Declare one \
                 window, or the same window on every relation.",
                many.iter()
                    .map(|(r, n)| format!("`{r}` says {n} epochs"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
    }
}

fn declared_currencies(schema: &str) -> Option<std::collections::BTreeMap<u32, u32>> {
    let (prog, d) = niles_lang::parser::parse_program(schema);
    if d.has_errors() {
        return None;
    }
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    if rd.has_errors() {
        return None;
    }
    // Code -> declared scale, **read from the catalog rather than re-derived**. This sorted
    // the currencies by span and numbered them, which is the right answer computed the wrong
    // way: the compiler was numbering them differently (by `HashMap` iteration), and two
    // derivations of one fact is how they came to disagree (A9-F05). One assignment, at
    // declaration, and both consumers read it.
    //
    // The scale travels with the code because it is the *currency's* property:
    // `currency jpy { scale: 0 }` and `currency bhd { scale: 3 }` are both real, and a wire
    // that assumed 2 would accept a third decimal place in yen.
    Some(cat.currencies.values().map(|c| (c.code, c.scale)).collect())
}

/// Why an `amt` was refused, when it was refused for its value rather than its shape.
#[derive(Debug, PartialEq, Eq)]
pub struct ScaleRefusal {
    pub currency: u32,
    pub declared: u32,
    pub written: String,
}

fn parse_insert(
    sql: &str,
    scales: &std::collections::BTreeMap<u32, u32>,
) -> Result<Option<Vec<proto_engine::Row>>, ScaleRefusal> {
    // `?` is not available on the shape path any more: the function distinguishes "this is not
    // the supported form" (`Ok(None)`) from "the form is right and the value is not"
    // (`Err`), and collapsing the second into the first is what let an out-of-scale amount be
    // reported as a syntax error.
    macro_rules! shape {
        ($e:expr) => {
            match $e {
                Some(v) => v,
                None => return Ok(None),
            }
        };
    }

    let lower = sql.to_ascii_lowercase();
    let into = shape!(lower.find("into"));
    let values = shape!(lower.find("values"));
    let target = lower[into + 4..values].trim();
    // The column list, if written, is accepted only in the declared order — an insert
    // naming columns in another order would be silently permuted otherwise.
    let target = shape!(target.split('(').next()).trim();
    if target != "postings" {
        return Ok(None);
    }
    let mut out = Vec::new();
    let mut rest = &sql[values + 6..];
    while let Some(open) = rest.find('(') {
        let close = shape!(rest[open..].find(')')) + open;
        let raw: Vec<&str> = rest[open + 1..close].split(',').map(|f| f.trim()).collect();
        if raw.len() != 4 {
            return Ok(None);
        }
        let whole = |t: &str| t.parse::<i128>().ok();
        let txn = u64::try_from(shape!(whole(raw[0]))).ok();
        let acct = u64::try_from(shape!(whole(raw[1]))).ok();
        let cur = u32::try_from(shape!(whole(raw[2]))).ok();
        let (txn, acct, cur) = (shape!(txn), shape!(acct), shape!(cur));
        let amt = match minor_units(raw[3], scales.get(&cur).copied()) {
            Ok(Some(a)) => a,
            Ok(None) => return Ok(None),
            Err(written) => {
                return Err(ScaleRefusal {
                    currency: cur,
                    declared: scales.get(&cur).copied().unwrap_or(0),
                    written,
                })
            }
        };
        out.push(proto_engine::Row::Post(proto_engine::Posting {
            txn,
            acct,
            cur,
            amt,
            valid: 0,
        }));
        rest = &rest[close + 1..];
    }
    if out.is_empty() {
        return Ok(None);
    }
    Ok(Some(out))
}

/// **An amount, in the currency's own minor units.**
///
/// The wire's `amt` is an integer of minor units, so a bare `-250` is 250 minor units and
/// always was. What was missing is the other spelling: a client that writes `2.50` means the
/// same thing, and a client that writes `2.505` against `currency usd { scale: 2 }` means
/// something the currency cannot hold.
///
/// Until now `2.505` failed `parse::<i128>()` and came back as *"this `insert` is outside the
/// supported form"* — a true sentence about the wrong problem, which sends a caller to check
/// their syntax when their money is the thing that does not fit. The scale is the currency's
/// (`currency jpy { scale: 0 }` and `currency bhd { scale: 3 }` are both real), so the check is
/// per row rather than per statement.
///
/// `Ok(None)` for something that is not a number at all — that is a shape problem and the
/// caller reports it as one. `Err(text)` for a number with more fractional digits than the
/// currency declares: **refused, never rounded.** Rounding here would silently move money, and
/// the amount a client wrote is the amount it meant.
fn minor_units(text: &str, scale: Option<u32>) -> Result<Option<i128>, String> {
    let scale = match scale {
        Some(s) => s as usize,
        // An undeclared currency: the ingress check refuses the row by name a moment later, and
        // guessing a scale for it here would decide which error the caller sees.
        None => return Ok(text.parse::<i128>().ok()),
    };
    let (int_part, frac) = match text.split_once('.') {
        None => (text, ""),
        Some((i, f)) => (i, f),
    };
    if int_part.is_empty() || !frac.chars().all(|c| c.is_ascii_digit()) {
        return Ok(None);
    }
    let whole: i128 = match int_part.parse() {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    if !text.contains('.') {
        // Already minor units, which is the form every existing client sends.
        return Ok(Some(whole));
    }
    // **One branch, and it is the refusal.** This was two: a `frac.len() > scale` guard, and
    // below it `scale - frac.len()`. Deleting the guard to prove it mattered did not change the
    // answer — the subtraction underflowed, `checked_pow` on the wrapped value returned `None`,
    // and the same `Err` came back for a different reason. The test passed with the check gone,
    // which made it evidence of nothing. Merging them leaves no second path to the same answer.
    let pad = match scale.checked_sub(frac.len()) {
        Some(p) => p,
        // More fractional digits than the currency holds. Refused, never rounded: rounding
        // moves money silently, and the amount a client wrote is the amount it meant.
        None => return Err(text.to_string()),
    };
    let frac_units: i128 = if frac.is_empty() {
        0
    } else {
        match frac.parse::<i128>() {
            Ok(v) => v,
            Err(_) => return Ok(None),
        }
    };
    let mult = 10i128
        .checked_pow(scale as u32)
        .ok_or_else(|| text.to_string())?;
    let scaled = frac_units
        .checked_mul(
            10i128
                .checked_pow(pad as u32)
                .ok_or_else(|| text.to_string())?,
        )
        .ok_or_else(|| text.to_string())?;
    let magnitude = whole
        .checked_mul(mult)
        .ok_or_else(|| text.to_string())?
        .checked_add(if int_part.starts_with('-') {
            -scaled
        } else {
            scaled
        })
        .ok_or_else(|| text.to_string())?;
    Ok(Some(magnitude))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA: &str = "\
schema bank {
    currency usd { scale: 2 }
    ledger postings { txn: TxnId, acct: Id<A>, cur: Currency, amt: Money,
        idem: IdemKey window 50_000.epochs, conserve per (txn, cur); retain forever; }
    index ix on postings (acct) anchor;
}";

    /// A test double that **evaluates the circuit**, like the real engine.
    ///
    /// The double it replaces held a `HashMap<(view, key), value>` and answered from it, so
    /// the session's tests could not have caught F-16: a session that ignored the circuit
    /// and a double that ignored the circuit agreed perfectly.
    struct MemoryEngine {
        /// **Behind a mutex, because `Serving` is a shared-borrow trait.** The real engine
        /// puts its base behind a reader-writer lock so that folds run concurrently; this
        /// one has no concurrency to win and takes the simplest interior mutability that
        /// satisfies the same signature.
        state: std::sync::Mutex<MemoryState>,
        views: Vec<(String, u32)>,
    }

    #[derive(Default)]
    struct MemoryState {
        frontier: u64,
        postings: niles_ir::eval::ZSet,
        appended: Vec<(String, usize)>,
    }

    impl MemoryEngine {
        fn state(&self) -> std::sync::MutexGuard<'_, MemoryState> {
            self.state.lock().expect("not poisoned")
        }
    }

    impl Serving for MemoryEngine {
        fn frontier(&self) -> u64 {
            self.state().frontier
        }
        fn query(
            &self,
            circuit: &niles_ir::circuit::Circuit,
            output: &str,
            anchor: u64,
        ) -> Result<Rows, ServeError> {
            let mut src = std::collections::BTreeMap::new();
            src.insert("postings".to_string(), self.state().postings.clone());
            let (z, _) = niles_ir::eval::try_run(circuit, output, &src)
                .map_err(|e| ServeError::Eval(e.to_string()))?;
            let width = z.keys().next().map(|r| r.len()).unwrap_or(0);
            let mut columns: Vec<String> = (0..width).map(|i| format!("c{i}")).collect();
            columns.push("anchor".into());
            let mut rows = Vec::new();
            for (r, w) in &z {
                for _ in 0..(*w).max(0) {
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
            Ok(Rows::text_rows(columns, rows))
        }
        fn append(
            &self,
            rows: Vec<proto_engine::Row>,
            txn_id: &str,
        ) -> Result<Appended, ServeError> {
            let mut st = self.state();
            if st.appended.iter().any(|(t, _)| t == txn_id) {
                return Err(ServeError::Duplicate(format!(
                    "`{txn_id}` already committed"
                )));
            }
            st.appended.push((txn_id.to_string(), rows.len()));
            for r in rows {
                if let proto_engine::Row::Post(p) = r {
                    niles_ir::eval::add(
                        &mut st.postings,
                        vec![
                            niles_ir::value::Value::Int(p.txn as i128),
                            niles_ir::value::Value::Int(p.acct as i128),
                            niles_ir::value::Value::Int(p.cur as i128),
                            niles_ir::value::Value::Int(p.amt),
                            niles_ir::value::Value::Null,
                        ],
                        1,
                    );
                }
            }
            st.frontier += 1;
            // No durable sink here, so no barrier and no receipt: `None` says that, where an
            // empty list could equally have meant "someone else took it".
            Ok(Appended {
                epoch: st.frontier,
                receipt: None,
            })
        }
        fn views(&self) -> Vec<(String, u32)> {
            self.views.clone()
        }
        fn durability(&self) -> &'static str {
            "none"
        }
        fn read_stats(&self) -> crate::rev_engine::ReadStats {
            crate::rev_engine::ReadStats::default()
        }
    }

    fn engine() -> MemoryEngine {
        // Two accounts, so a query naming one must not answer for the other.
        let postings = niles_ir::eval::zset(&[
            (&[1, 1001, 0, 85_000, 0], 1),
            (&[2, 2002, 0, 12_500, 0], 1),
            (&[3, 1001, 0, 500, 0], 1),
        ]);
        let mut postings: niles_ir::eval::ZSet = postings;
        // The `idem` column is text in the schema and has no integer form; it is null.
        let nulled: Vec<(Vec<niles_ir::value::Value>, i128)> = postings
            .iter()
            .map(|(r, w)| {
                let mut r = r.clone();
                r[4] = niles_ir::value::Value::Null;
                (r, *w)
            })
            .collect();
        postings = nulled.into_iter().collect();
        MemoryEngine {
            state: std::sync::Mutex::new(MemoryState {
                frontier: 4200,
                postings,
                appended: Vec::new(),
            }),
            views: vec![("ledger_balance".into(), 2)],
        }
    }

    fn session() -> Session {
        Session::new("ada".into(), "bank".into(), SCHEMA.into())
    }

    /// The data rows in a reply.
    /// The rows a client would decode from a reply.
    ///
    /// Through `pg_wire::decoded_rows` rather than by matching on `Backend::DataRow`,
    /// because a served answer's rows are framed once into a `Backend::Raw` buffer and a
    /// diagnostic statement's are not. A test that matched on the representation would be
    /// asserting which code path ran; this asserts what reaches the client, which is what
    /// the test is about.
    fn rows_of(out: &[Backend]) -> Vec<Vec<Option<String>>> {
        pg_wire::decoded_rows(out)
    }

    /// **A statement is compiled once, and the IR verifier runs once with it — T-13.3.**
    ///
    /// H-S6 claims static checking subsumes runtime policing *at no measurable runtime
    /// cost*, and that claim is only true if the checking happens off the serving path.
    /// Nothing had asserted it. A verifier that ran per execution would put the whole front
    /// end — parse, resolve, typecheck, lower, verify, 243 µs on a 158-line program (E26) —
    /// inside every query, and the hypothesis would be false for a reason no experiment in
    /// this project was looking at.
    ///
    /// A thousand distinct statements and a thousand repeats of them: the compile count must
    /// be a thousand and not two thousand.
    #[test]
    fn the_front_end_runs_once_per_distinct_statement_and_not_per_execution() {
        let (mut s, e) = (session(), engine());
        // Inside the cache's limit, so this measures the front end's invocation count and
        // not the eviction policy — which is the next test's subject.
        const N: u64 = 200;
        for i in 0..N {
            let _ = s.handle(
                Frontend::Query(format!("select sum(amt) from postings where acct = {i}")),
                &e,
            );
        }
        let after_first = (s.compile_misses, s.compile_hits);
        assert_eq!(
            after_first.0, N,
            "{N} distinct statements must compile {N} times, not {}",
            after_first.0
        );
        for i in 0..N {
            let _ = s.handle(
                Frontend::Query(format!("select sum(amt) from postings where acct = {i}")),
                &e,
            );
        }
        assert_eq!(
            s.compile_misses,
            N,
            "a repeat must not recompile: {} compilations after {} executions of {N} \
             statements. The front end would then be on the serving path, and H-S6's \
             \"no measurable runtime cost\" would be a claim about a compiler nobody runs",
            s.compile_misses,
            2 * N
        );
        assert_eq!(
            s.compile_hits, N,
            "every repeat must be a cache hit: {} hits",
            s.compile_hits
        );
    }

    /// **One new statement must not cost a session every plan it holds — T-13.3's other
    /// half.**
    ///
    /// The limit is 256 plans per session, and the policy used to be *empty the cache* to
    /// admit the 257th. Under that policy a single novel statement — one generated `where`
    /// clause, one ad-hoc query from a human — threw away every plan the session had, so a
    /// workload of a stable hot set plus occasional one-offs recompiled its whole hot set
    /// after every one-off. Measured before the change: 1,000 distinct statements executed
    /// twice cost **2,000** compilations, where a cache exists to produce 1,000.
    ///
    /// The property, minimally: fill the cache, admit one novel statement, and ask for a hot
    /// statement that is not the one evicted. FIFO answers from the cache; clearing does
    /// not. Both policies lose a strict cyclic scan of `limit + 1` statements — that is
    /// Bélády's worst case and LRU shares it — so the test does not claim otherwise.
    #[test]
    fn one_new_statement_evicts_one_plan_and_not_the_whole_cache() {
        let (mut s, e) = (session(), engine());
        let limit = super::PLAN_CACHE_LIMIT as u64;
        let send = |s: &mut Session, sql: String| {
            let _ = s.handle(Frontend::Query(sql), &e);
        };
        let hot = |i: u64| format!("select sum(amt) from postings where acct = {i}");
        for i in 0..limit {
            send(&mut s, hot(i));
        }
        assert_eq!(s.compiled_len(), limit as usize, "the cache is full");
        // One novel statement. Exactly one plan may leave.
        send(
            &mut s,
            "select sum(amt) from postings where acct = 999999".into(),
        );
        assert_eq!(
            s.compiled_len(),
            limit as usize,
            "the cache must stay at its limit, not below it"
        );
        let before = s.compile_hits;
        // The newest hot statement, which no policy has any reason to have evicted.
        send(&mut s, hot(limit - 1));
        assert_eq!(
            s.compile_hits,
            before + 1,
            "a hot plan did not survive one unrelated statement. Emptying the cache to admit \
             one entry makes it worse than no cache for every session whose working set is a \
             little larger than the limit — it pays to compile *and* to store."
        );
        // And the oldest is the one that went.
        let before = s.compile_misses;
        send(&mut s, hot(0));
        assert_eq!(
            s.compile_misses,
            before + 1,
            "the evicted plan must be the oldest: first in, first out"
        );
    }

    #[test]
    fn a_query_compiles_through_the_same_front_end_as_any_other() {
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 1001 group by acct".into(),
            ),
            &e,
        );
        assert!(matches!(out[0], Backend::RowDescription(_)), "{out:?}");
        assert!(!pg_wire::decoded_rows(&out).is_empty());
        assert!(matches!(out.last(), Some(Backend::ReadyForQuery(b'I'))));
    }

    #[test]
    fn a_compiler_error_reaches_the_client_with_its_code_intact() {
        // The point of not having a compatibility layer: the client gets the real
        // diagnostic, not a generic syntax error that says the wrong thing.
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("select nope from nonexistent_table".into()),
            &e,
        );
        let Some(Backend::ErrorResponse { message, .. }) = out.first() else {
            panic!("{out:?}")
        };
        assert!(
            message.starts_with("[NL"),
            "the NL code must survive to the wire: {message}"
        );
        assert_eq!(
            s.status(),
            b'E',
            "and the session must enter the failed state"
        );
    }

    #[test]
    fn the_session_anchor_only_moves_forward() {
        // Rung 1, monotonic reads, in one `max`. A session that read the live frontier each
        // time would give a weaker guarantee for the same work.
        let mut s = session();
        assert_eq!(s.observe(100), 100);
        assert_eq!(
            s.observe(50),
            100,
            "a lower frontier must not move the session back"
        );
        assert_eq!(s.observe(200), 200);
    }

    #[test]
    fn a_missing_key_is_null_and_not_zero() {
        // The absence lattice reaching the client intact. A view with no entry for a key is
        // not a view whose balance is zero, and a wire format that conflated them would
        // undo the distinction the whole engine maintains.
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 9999 group by acct".into(),
            ),
            &e,
        );
        // **The old assertion was wrong, and its wrongness is the finding.** It said this
        // query returns one row whose value is NULL. It does not: a grouped aggregate over
        // an account with no postings produces *no group*, and SQL returns no row. The old
        // server manufactured one — it took the key out of the query text and emitted a row
        // for it whether or not the data had one, which is a fabricated result row wearing
        // the absence argument's clothes.
        //
        // The distinction the old test was defending is real and is enforced one layer
        // down, where it belongs: `RevEngine::read_point` returns `Option`, and
        // `a_key_the_base_has_never_seen_has_no_value` holds it.
        let rows = pg_wire::decoded_rows(&out);
        assert!(
            rows.is_empty(),
            "an account with no postings forms no group, so there is no row to be null: {out:?}"
        );
        assert!(
            out.iter()
                .any(|m| matches!(m, Backend::CommandComplete(t) if t == "SELECT 0")),
            "{out:?}"
        );
    }

    #[test]
    fn every_answer_carries_the_anchor_it_was_true_at() {
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 1001 group by acct".into(),
            ),
            &e,
        );
        let Some(Backend::RowDescription(fields)) = out.first() else {
            panic!()
        };
        assert_eq!(
            fields[2].name, "anchor",
            "the anchor is a column, not a footnote"
        );
        let row = pg_wire::decoded_rows(&out).into_iter().next();
        assert_eq!(row.unwrap()[2], Some("4200".into()));
    }

    #[test]
    fn money_is_described_as_numeric_on_this_path_too() {
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 1001 group by acct".into(),
            ),
            &e,
        );
        let Some(Backend::RowDescription(fields)) = out.first() else {
            panic!()
        };
        assert_eq!(
            fields[1].type_oid, 1700,
            "the money column must be numeric, not float8"
        );
    }

    #[test]
    fn an_unkeyed_query_is_answered_by_the_scan_surface() {
        // **Inverted, and the inversion is the finding.** This used to assert that an
        // unkeyed query was *refused*, on the reasoning that "scanning defeats the
        // mechanism being measured". That is a benchmark's reason, not a database's: a
        // server that refuses `group by` because the answer would be uninteresting to an
        // experiment cannot answer the analytical half of its own Part 0 table.
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("select acct, sum(amt) from postings group by acct".into()),
            &e,
        );
        let rows = pg_wire::decoded_rows(&out);
        assert_eq!(rows.len(), 2, "both accounts: {out:?}");
        assert!(
            !out.iter()
                .any(|m| matches!(m, Backend::NoticeResponse { .. })),
            "and no apology"
        );
    }

    #[test]
    fn the_extended_protocol_parses_binds_and_executes() {
        // **Inverted, and the inversion is the finding.** This used to assert that `Parse`
        // was refused with "that design question is open" — while `extended.rs`, in the
        // same crate, contained the answer, a plan cache keyed by schema epoch, and eleven
        // tests. The module was reachable from nothing.
        let (mut s, e) = (session(), engine());
        let mut parse = Vec::new();
        push_cstr(&mut parse, "st1");
        push_cstr(
            &mut parse,
            "select acct, sum(amt) from postings where acct = 1001 group by acct",
        );
        parse.extend_from_slice(&0u16.to_be_bytes());
        let out = s.handle(Frontend::Extended(b'P', parse), &e);
        assert!(
            matches!(out.first(), Some(Backend::ParseComplete)),
            "{out:?}"
        );

        let mut bind = Vec::new();
        push_cstr(&mut bind, "po1");
        push_cstr(&mut bind, "st1");
        let out = s.handle(Frontend::Extended(b'B', bind), &e);
        assert!(
            matches!(out.first(), Some(Backend::BindComplete)),
            "{out:?}"
        );

        // Describe answers from the plan, before any row is fetched — which is the whole
        // point of having compiled at `Parse` time.
        let mut desc = vec![b'S'];
        push_cstr(&mut desc, "st1");
        let out = s.handle(Frontend::Extended(b'D', desc), &e);
        let Some(Backend::RowDescription(fields)) = out.first() else {
            panic!("{out:?}")
        };
        assert_eq!(
            fields.last().map(|f| f.name.as_str()),
            Some("anchor"),
            "every answer carries the moment it is true at"
        );

        let mut exec = Vec::new();
        push_cstr(&mut exec, "po1");
        exec.extend_from_slice(&0u32.to_be_bytes());
        let out = s.handle(Frontend::Extended(b'E', exec), &e);
        assert!(
            pg_wire::decoded_rows(&out)
                .iter()
                .any(|r| r[1] == Some("85500".into())),
            "{out:?}"
        );
        assert!(
            !out.iter().any(|m| matches!(m, Backend::RowDescription(_))),
            "an Execute does not re-send the row description the client already has"
        );

        // Only `Sync` emits `ReadyForQuery`. A client notices immediately if this is wrong.
        let out = s.handle(Frontend::Extended(b'S', Vec::new()), &e);
        assert!(
            matches!(out.first(), Some(Backend::ReadyForQuery(_))),
            "{out:?}"
        );
    }

    #[test]
    fn a_prepared_statement_survives_a_schema_epoch_moving() {
        // The answer `extended.rs` was written around: a plan is valid exactly while no
        // schema epoch has occurred after the one it was compiled at. A stale plan is
        // recompiled, not refused, so the client's prepared name keeps working.
        let (mut s, e) = (session(), engine());
        let mut parse = Vec::new();
        push_cstr(&mut parse, "st1");
        push_cstr(
            &mut parse,
            "select acct, sum(amt) from postings where acct = 1001 group by acct",
        );
        parse.extend_from_slice(&0u16.to_be_bytes());
        s.handle(Frontend::Extended(b'P', parse), &e);
        let mut bind = Vec::new();
        push_cstr(&mut bind, "po1");
        push_cstr(&mut bind, "st1");
        s.handle(Frontend::Extended(b'B', bind), &e);

        // The frontier moves, so the plan's schema epoch no longer matches.
        e.state().frontier += 10;
        let mut exec = Vec::new();
        push_cstr(&mut exec, "po1");
        exec.extend_from_slice(&0u32.to_be_bytes());
        let out = s.handle(Frontend::Extended(b'E', exec), &e);
        assert!(
            !pg_wire::decoded_rows(&out).is_empty(),
            "a plan whose schema epoch moved is recompiled, not refused: {out:?}"
        );
    }

    fn push_cstr(b: &mut Vec<u8>, s: &str) {
        b.extend_from_slice(s.as_bytes());
        b.push(0);
    }

    #[test]
    fn transaction_control_says_what_it_does_and_does_not_provide() {
        // The notice no longer apologises for a rollback that does not exist — writes are
        // buffered and sealed as one epoch at `COMMIT`, so a `ROLLBACK` discards them and
        // there is nothing to compensate. What it still says is the part that remains true:
        // reads are served at a session anchor rather than from a snapshot held open.
        let (mut s, e) = (session(), engine());
        let out = s.handle(Frontend::Query("begin".into()), &e);
        assert!(
            out.iter().any(|m| matches!(m, Backend::NoticeResponse { message } if message.contains("session anchor"))),
            "a client must be told how its reads are anchored: {out:?}"
        );
        assert!(
            out.iter().any(|m| matches!(m, Backend::NoticeResponse { message } if message.contains("one epoch at COMMIT"))),
            "and how its writes are sealed: {out:?}"
        );
        assert_eq!(s.status(), b'T');
        s.handle(Frontend::Query("commit".into()), &e);
        assert_eq!(s.status(), b'I');
    }

    #[test]
    fn an_empty_query_gets_the_empty_response_not_an_error() {
        let (mut s, e) = (session(), engine());
        let out = s.handle(Frontend::Query("   ".into()), &e);
        assert!(matches!(out[0], Backend::EmptyQueryResponse), "{out:?}");
    }

    /// **A statement is compiled once per session, and the second ask is a cache hit.**
    ///
    /// `callgrind` on the point path: 177,064 instructions to compile a statement against
    /// 7,693 to serve it, so 96% of the daemon's own work on that workload was recompiling
    /// something it had just compiled. Both of the things previously said about this are
    /// true and neither is the whole story — compilation is 0.5% of a *wall-clock* budget
    /// dominated by a 120µs round trip, and 96% of the engine's own — which is why the
    /// decision to build this was gated on measuring it rather than on either intuition.
    #[test]
    fn a_repeated_statement_is_compiled_once() {
        let (mut s, e) = (session(), engine());
        let sql = "select acct, sum(amt) from postings where acct = 1001 group by acct";
        let first = s.handle(Frontend::Query(sql.into()), &e);
        assert!(
            !first
                .iter()
                .any(|m| matches!(m, Backend::ErrorResponse { .. })),
            "{first:?}"
        );
        assert_eq!((s.compile_hits, s.compile_misses), (0, 1));

        for _ in 0..20 {
            let again = s.handle(Frontend::Query(sql.into()), &e);
            assert_eq!(
                rows_of(&again),
                rows_of(&first),
                "a cached plan must answer what the compiled one did"
            );
        }
        assert_eq!(
            (s.compile_hits, s.compile_misses),
            (20, 1),
            "twenty-one asks, one compilation"
        );

        // A *different* statement is a different key, and still compiles.
        let other = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 9 group by acct".into(),
            ),
            &e,
        );
        assert!(!other
            .iter()
            .any(|m| matches!(m, Backend::ErrorResponse { .. })));
        assert_eq!((s.compile_hits, s.compile_misses), (20, 2));
    }

    /// **A statement that does not compile is not cached, and says so every time.**
    ///
    /// The failure a plan cache invites: storing something the verifier refused, or
    /// answering the second ask from a cache entry that should not exist. A refusal is not a
    /// plan.
    #[test]
    fn a_refused_statement_is_refused_again_and_never_cached() {
        let (mut s, e) = (session(), engine());
        for _ in 0..3 {
            let mut fresh = session();
            let out = fresh.handle(
                Frontend::Query("select nope from nonexistent_table".into()),
                &e,
            );
            assert!(
                out.iter()
                    .any(|m| matches!(m, Backend::ErrorResponse { .. })),
                "a statement that does not compile must be refused: {out:?}"
            );
            assert_eq!(fresh.compile_hits, 0, "a refusal is not a plan");
        }
        let _ = &mut s;
    }

    /// **The lowering's refusals reach the wire with their codes.**
    ///
    /// The three shapes T-03 repaired all *compiled* before it, and the daemon served their
    /// wrong answers: an aggregating projection with no aggregate in it, a `group by` over a
    /// name that resolves to nothing, and a projection column outside the grouping. A
    /// refusal that only `nilesc` produces is a refusal a client never sees, and the client
    /// is what a bank runs.
    #[test]
    fn the_aggregating_refusals_reach_the_wire_with_their_codes() {
        for (sql, code) in [
            (
                "select acct, sum(amt) * 2 from postings group by acct",
                "NL0517",
            ),
            (
                "select acct, sum(amt) from postings group by nope",
                "NL0509",
            ),
            (
                "select acct, cur, sum(amt) from postings group by acct",
                "NL0517",
            ),
        ] {
            let (mut s, e) = (session(), engine());
            let out = s.handle(Frontend::Query(sql.into()), &e);
            let found = out.iter().any(|m| {
                matches!(m, Backend::ErrorResponse { message, detail, .. }
                    if message.contains(code)
                        || detail.as_deref().is_some_and(|d| d.contains(code)))
            });
            assert!(
                found,
                "`{sql}` must be refused on the wire with {code}, got {out:?}"
            );
            assert!(
                pg_wire::decoded_rows(&out).is_empty(),
                "`{sql}` must serve no row at all"
            );
        }
    }

    /// **The assumption the cache rests on**: a lowering does not depend on the anchor it
    /// was compiled at.
    ///
    /// The cache is keyed by `(schema, statement text)` and not by the session anchor, which
    /// would be a different key on every write and no cache at all. That is sound only
    /// because `Catalog::epoch` is carried into the lowering and never read by it — `as_of`
    /// takes a *literal* epoch. If that ever stops being true, this fails here rather than
    /// by serving a stale plan.
    #[test]
    fn a_cached_plan_does_not_depend_on_the_anchor_it_was_compiled_at() {
        let sql = "select acct, sum(amt) from postings where acct = 1001 group by acct";
        let circuit_at = |anchor: u64| {
            let program = format!(
                "{SCHEMA}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};\n"
            );
            let (prog, d) = niles_lang::parser::parse_program(&program);
            assert!(!d.has_errors());
            let (cat, _) = niles_lang::resolve::resolve_program(&prog, anchor);
            let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
            assert!(!ld.has_errors());
            format!("{:?}", lowered.circuit)
        };
        assert_eq!(
            circuit_at(0),
            circuit_at(4_200),
            "the same statement lowered to different circuits at two anchors, so a plan \
             cached at one is not valid at another and this cache is unsound"
        );
        assert_eq!(circuit_at(4_200), circuit_at(u64::MAX / 2));
    }

    #[test]
    fn two_queries_over_one_key_return_different_answers() {
        // **The test that would have caught F-16, and did not exist.** The served answer
        // used to be `sum(amt)` for `key[0]`, whatever was asked: `pick_view` returned a
        // constant, `extract_keys` scraped digit runs out of the query *text*, and the
        // circuit the compiler had just verified was dropped on the floor. Two different
        // questions about account 1001 came back with the same number.
        let (mut s, e) = (session(), engine());
        let rows = |out: &[Backend]| -> Vec<Vec<Option<String>>> { pg_wire::decoded_rows(out) };
        let sum = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 1001 group by acct".into(),
            ),
            &e,
        );
        let count = s.handle(
            Frontend::Query(
                "select acct, count(amt) from postings where acct = 1001 group by acct".into(),
            ),
            &e,
        );
        let (a, b) = (rows(&sum), rows(&count));
        assert_eq!(a.len(), 1, "{sum:?}");
        assert_eq!(b.len(), 1, "{count:?}");
        assert_eq!(a[0][1], Some("85500".into()), "the sum of 85000 and 500");
        assert_eq!(b[0][1], Some("2".into()), "two postings on that account");
        assert_ne!(a, b, "two queries, two answers");
    }

    #[test]
    fn a_query_naming_one_account_does_not_answer_for_another() {
        // The other half: the key comes from the *predicate the compiler lowered*, not from
        // a digit scanner over the query text.
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 2002 group by acct".into(),
            ),
            &e,
        );
        let rows = pg_wire::decoded_rows(&out);
        assert_eq!(rows.len(), 1, "{out:?}");
        assert_eq!(rows[0][0], Some("2002".into()));
        assert_eq!(rows[0][1], Some("12500".into()));
    }

    #[test]
    fn an_insert_appends_and_is_visible_to_the_next_read() {
        // The wire surface used to be read-only: an `INSERT` was wrapped in a view and
        // rejected, so half a database was unreachable over the protocol §6.9's adoption
        // argument rests on.
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("insert into postings values (4, 3003, 0, 700)".into()),
            &e,
        );
        assert!(
            out.iter()
                .any(|m| matches!(m, Backend::CommandComplete(t) if t == "INSERT 0 1")),
            "{out:?}"
        );
        let read = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 3003 group by acct".into(),
            ),
            &e,
        );
        assert!(
            pg_wire::decoded_rows(&read)
                .iter()
                .any(|r| r[1] == Some("700".into())),
            "{read:?}"
        );
    }

    /// **An amount the currency cannot hold is refused as a value, not as a syntax error.**
    ///
    /// The schema declares `currency usd { scale: 2 }`, so `2.505` is not a USD amount. Until
    /// this it failed `parse::<i128>()` and came back as *"this `insert` is outside the
    /// supported form"* — a true sentence about the wrong problem, which sends a caller to
    /// check their syntax when their money is what does not fit.
    #[test]
    fn an_amount_finer_than_the_declared_scale_is_refused_by_name() {
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("insert into postings values (770001, 1, 0, 2.505)".into()),
            &e,
        );
        let msg = format!("{out:?}");
        assert!(
            !out.iter()
                .any(|m| matches!(m, Backend::CommandComplete(t) if t.starts_with("INSERT"))),
            "an amount below the declared scale must not commit: {msg}"
        );
        assert!(
            msg.contains("22003"),
            "and must be `numeric_value_out_of_range` — the shape is fine and the value is \
             not, which is a different thing to be told: {msg}"
        );
        assert!(
            msg.contains("2.505") && msg.contains("fractional digits"),
            "and must name the amount and what is wrong with it: {msg}"
        );
        assert!(
            msg.contains("rather than rounded"),
            "and must say it was refused rather than rounded — silently rounding an amount is \
             moving money: {msg}"
        );
    }

    /// The two spellings that *are* the same amount both commit, and to the same minor units.
    /// A check that refused decimals outright would pass the test above and be useless.
    #[test]
    fn a_decimal_at_the_declared_scale_is_the_same_amount_as_its_minor_units() {
        for (sql, label) in [
            (
                "insert into postings values (770010, 1, 0, 250)",
                "minor units",
            ),
            (
                "insert into postings values (770011, 1, 0, 2.50)",
                "a decimal",
            ),
            (
                "insert into postings values (770012, 1, 0, -2.50)",
                "a negative decimal",
            ),
        ] {
            let (mut s, e) = (session(), engine());
            let out = s.handle(Frontend::Query(sql.into()), &e);
            assert!(
                out.iter()
                    .any(|m| matches!(m, Backend::CommandComplete(t) if t == "INSERT 0 1")),
                "{label} must commit: {out:?}"
            );
        }
        // And the value is the same one, which the parser is the only place to check.
        let scales = std::collections::BTreeMap::from([(0u32, 2u32)]);
        let of = |sql: &str| match parse_insert(sql, &scales) {
            Ok(Some(rows)) => match rows.first() {
                Some(proto_engine::Row::Post(p)) => p.amt,
                _ => panic!("a posting"),
            },
            other => panic!("{sql}: {other:?}"),
        };
        assert_eq!(of("insert into postings values (1, 1, 0, 250)"), 250);
        assert_eq!(of("insert into postings values (1, 1, 0, 2.50)"), 250);
        assert_eq!(of("insert into postings values (1, 1, 0, -2.50)"), -250);
        assert_eq!(of("insert into postings values (1, 1, 0, 2.5)"), 250);
        assert_eq!(of("insert into postings values (1, 1, 0, 0.01)"), 1);
    }

    /// The scale is the **currency's**, not the wire's. `currency jpy { scale: 0 }` holds no
    /// fractional digit at all, and a wire that assumed 2 would take a fraction of a yen.
    #[test]
    fn the_scale_that_applies_is_the_one_that_currency_declares() {
        let scales = std::collections::BTreeMap::from([(0u32, 2u32), (1u32, 0u32)]);
        // Currency 1 is scale 0: one fractional digit is one too many.
        assert!(matches!(
            parse_insert("insert into postings values (1, 1, 1, 2.5)", &scales),
            Err(ScaleRefusal {
                currency: 1,
                declared: 0,
                ..
            })
        ));
        // The same text against currency 0 is fine.
        assert!(matches!(
            parse_insert("insert into postings values (1, 1, 0, 2.5)", &scales),
            Ok(Some(_))
        ));
    }

    /// **The F-41 witness at the wire: an undeclared currency is refused.**
    ///
    /// The schema declares one currency, so code `0` is the only one it admits, and
    /// `INSERT … (777002, 1, 999, 500)` used to answer `INSERT 0 2`. It reached the base, and
    /// the next `sum(amt) group by acct` added 324 USD to 500 of currency 999 and served
    /// "824". Contribution 4 was proved for programs and undischarged for data.
    /// **The columns the benchmark's reader looks up by name.**
    ///
    /// `bank-bench` reads `view_answers` and `fallbacks` out of this reply *by name* and
    /// renders `n/a` when they are absent, which is the right degradation and a silent one:
    /// deleting them here would turn every E19 fallback figure into "the question was not
    /// asked" without failing a build. This is the other half of that chain, and it is the
    /// half that fails loudly.
    #[test]
    fn nilestream_stats_names_the_columns_the_benchmark_reads() {
        let (mut s, e) = (session(), engine());
        let out = s.handle(Frontend::Query("select nilestream_stats".into()), &e);
        let names: Vec<String> = out
            .iter()
            .find_map(|m| match m {
                Backend::RowDescription(f) => Some(f.iter().map(|x| x.name.clone()).collect()),
                _ => None,
            })
            .expect("a row description");
        for wanted in [
            "reads",
            "hits",
            "misses",
            "rows_touched",
            "resident",
            "view_answers",
            "fallbacks",
            "view_metadata_keys",
            "idem_window_keys",
            "schema_parses",
            "pending_joins",
            "uninstalled_folds",
            "pinned_installs",
            "flights_refused",
            // **Cycle 10's instruments (T00.2).** Each is read by the baseline script by
            // name; a dropped column renders as `n/a` rather than failing, so the name is
            // the contract and this list is where it is kept.
            "deferred_merges",
            "waiters_refused",
            "joins_answered",
            "joins_retried",
            "gap_at_begin_total",
            "gap_at_finish_total",
            "gap_at_finish_max",
            "flights_behind_at_begin",
            "flights_that_fell_behind",
        ] {
            assert!(
                names.iter().any(|n| n == wanted),
                "`select nilestream_stats` must name `{wanted}` — the benchmark looks it up                  by name and renders `n/a` when it is missing, so dropping it here would                  quietly unmeasure the column rather than break anything. Got: {names:?}"
            );
        }
    }

    /// **The schema is compiled once per epoch, not once per `INSERT` — F-68, C9-05.1.**
    ///
    /// `insert` needed the declared currencies and their scales, and obtained them by running
    /// `parse_program` and `resolve_program` over the whole schema text every time. Callgrind
    /// on E16's `oltp` statement: 74,500 of 113,500 instructions per insert, 61%, spent
    /// re-deriving a table the session had already compiled and holds a key for. The compiler
    /// was right; the engine did not trust it.
    #[test]
    fn the_schema_is_parsed_once_per_epoch_and_not_once_per_insert() {
        let (mut s, e) = (session(), engine());
        const N: u64 = 2_000;
        for i in 0..N {
            let out = s.handle(
                Frontend::Query(format!(
                    "insert into postings values ({}, 1, 0, -1), ({}, 2, 0, 1)",
                    900_000 + i,
                    900_000 + i
                )),
                &e,
            );
            assert!(
                out.iter().any(|m| matches!(
                    m,
                    Backend::CommandComplete(t) if t.starts_with("INSERT")
                )),
                "insert {i} must be accepted for this measurement to mean anything: {out:?}"
            );
        }
        assert_eq!(
            s.schema_parses, 1,
            "{N} inserts at one schema epoch parsed the schema {} times",
            s.schema_parses
        );

        // A schema change invalidates it, by the same key the plan cache uses.
        s.schema.push('\n');
        let _ = s.handle(
            Frontend::Query("insert into postings values (990001, 1, 0, 0)".into()),
            &e,
        );
        assert_eq!(
            s.schema_parses, 2,
            "a changed schema must be compiled again rather than answered from a stale table"
        );
    }

    /// **Every slow-read column is declared, and the declaration matches the row — T00.2.**
    ///
    /// This guard exists because the failure it catches happened. The six phase columns
    /// added this cycle were emitted as `DataRow` values while the `RowDescription` still
    /// declared five: a `psql` reading the table would have shown eleven values under six
    /// names, silently shifting every column's meaning by five positions — the residual
    /// read as `view_hold_us`, and so on. Nothing in the build objects to a row that is
    /// wider than its description; only a client does, and only by lying.
    ///
    /// So the assertion is not "the names I want are present" but "the description and
    /// the row have the same width", which is the property that was actually violated.
    #[test]
    fn the_slow_read_table_declares_every_column_it_emits() {
        let (mut s, e) = (session(), engine());
        // A read that reaches the view, so the table has at least one row to be wrong about.
        for i in 0..8u64 {
            let _ = s.handle(
                Frontend::Query(format!("select bal from balances where acct = {}", i % 3)),
                &e,
            );
        }
        let out = s.handle(Frontend::Query("select nilestream_slow_reads".into()), &e);
        let names: Vec<String> = out
            .iter()
            .find_map(|m| match m {
                Backend::RowDescription(f) => Some(f.iter().map(|x| x.name.clone()).collect()),
                _ => None,
            })
            .expect("a row description");
        for wanted in [
            "rank",
            "total_us",
            "base_wait_us",
            "view_wait_us",
            "view_hold_us",
            "view_wait2_us",
            "view_hold2_us",
            "fold_us",
            "outcome",
            "gap_begin",
            "gap_finish",
            "rounding_us",
        ] {
            assert!(
                names.iter().any(|n| n == wanted),
                "`select nilestream_slow_reads` must name `{wanted}`; got {names:?}"
            );
        }
        for (i, row) in rows_of(&out).iter().enumerate() {
            assert_eq!(
                row.len(),
                names.len(),
                "row {i} carries {} values under {} declared names — a client would read \
                 every column after the first extra one as the wrong quantity",
                row.len(),
                names.len()
            );
        }
    }

    /// **`select nilestream_lockstats` names six nested scopes and can be reset — F-75,
    /// A10-08.**
    ///
    /// The reset is the point. Both lock histograms are process-global and a sweep that
    /// prints them per level was printing every earlier level with each one.
    ///
    /// Six, not four: `base` is one histogram over an `RwLock` taken in two modes, and a
    /// p99 over a shared hold and an exclusive hold belongs to neither. `base_read` and
    /// `base_write` are the same acquisitions counted apart; `base` is kept unchanged so
    /// that a number quoted by an earlier cycle still means what it meant.
    #[test]
    fn the_lock_histograms_name_six_scopes_and_a_reset_empties_them() {
        let (mut s, e) = (session(), engine());
        // Something in every scope: a statement through `handle` fills `STATEMENT`, and the
        // query it runs takes the base.
        for _ in 0..3 {
            let _ = s.handle(
                Frontend::Query("select acct, sum(amt) from postings group by acct".into()),
                &e,
            );
        }
        let out = s.handle(Frontend::Query("select nilestream_lockstats".into()), &e);
        let names: Vec<String> = out
            .iter()
            .find_map(|m| match m {
                Backend::RowDescription(f) => Some(f.iter().map(|x| x.name.clone()).collect()),
                _ => None,
            })
            .expect("a row description");
        for wanted in [
            "scope",
            "acquisitions",
            "wait_p99_us",
            "hold_p99_us",
            "hold_max_us",
            "hold_total_us",
        ] {
            assert!(
                names.iter().any(|n| n == wanted),
                "`select nilestream_lockstats` must name `{wanted}`; got {names:?}"
            );
        }
        let scopes: Vec<String> = rows_of(&out)
            .iter()
            .filter_map(|r| r.first().cloned().flatten())
            .collect();
        assert_eq!(
            scopes,
            vec![
                "base",
                "base_read",
                "base_write",
                "view",
                "statement",
                "wire"
            ],
            "six scopes: the aggregate base, its two modes, then the nested scopes \
             outermost last"
        );
        // **The split must add up to the aggregate it splits.** If a `TimedRead` or a
        // `TimedWrite` stopped feeding both histograms, the modes would silently under-count
        // and every attribution drawn from them would be wrong by an unknown amount.
        let acq = |name: &str| -> u64 {
            rows_of(&out)
                .iter()
                .find(|r| r.first().cloned().flatten().as_deref() == Some(name))
                .and_then(|r| r[1].clone())
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| panic!("the {name} row carries an acquisition count"))
        };
        assert_eq!(
            acq("base_read") + acq("base_write"),
            acq("base"),
            "every base acquisition is either shared or exclusive and is counted once in \
             each of the two places"
        );
        let statement_before: u64 = rows_of(&out)
            .iter()
            .find(|r| r.first().cloned().flatten().as_deref() == Some("statement"))
            .and_then(|r| r[1].clone())
            .and_then(|v| v.parse().ok())
            .expect("the statement row carries an acquisition count");
        assert!(
            statement_before > 0,
            "`Session::handle` must be counted; it is the inner boundary the slow-read \
             table cannot see past"
        );
        // The reset, and then a fresh read of the same table.
        let _ = s.handle(
            Frontend::Query("select nilestream_lockstats reset".into()),
            &e,
        );
        let after = s.handle(Frontend::Query("select nilestream_lockstats".into()), &e);
        let statement_after: u64 = rows_of(&after)
            .iter()
            .find(|r| r.first().cloned().flatten().as_deref() == Some("statement"))
            .and_then(|r| r[1].clone())
            .and_then(|v| v.parse().ok())
            .expect("the statement row is still there");
        assert!(
            statement_after < statement_before,
            "a reset must drop the counters: {statement_before} before, {statement_after} \
             after (the one or two statements since the reset are the only ones left)"
        );
    }

    /// **The declared window reaches the engine — T-05.3.**
    ///
    /// `idem: IdemKey window N.epochs` was checked for existence by the compiler and read by
    /// nothing: both idempotency indexes kept every identity ever committed. The daemon now
    /// reads it out of the schema it already compiles at start-up, and hands it to the base
    /// and to the sealer. A schema with no window still keeps everything, which the banner
    /// says out loud.
    #[test]
    fn the_daemons_schema_declares_an_idempotency_window_in_epochs() {
        assert_eq!(
            super::declared_idem_window(crate::daemon::DEFAULT_SCHEMA),
            Ok(Some(1_000_000)),
            "the shipped schema must declare a window the engine can honour, in epochs"
        );
        assert_eq!(
            super::declared_idem_window("schema s { currency usd { scale: 2 } }"),
            Ok(None),
            "a schema with no ledger declares no window, and the daemon refuses to choose \
             one silently rather than assuming"
        );
    }

    /// **One daemon, one window — and it is not chosen by a hash seed (F-71).**
    ///
    /// Two relations declaring different windows used to resolve to whichever the catalog's
    /// `HashMap` yielded first, so the same binary on the same schema ran with different
    /// windows on different starts. Ten resolutions in one process, because the order this
    /// depends on is seeded per process and would take luck to catch in one.
    #[test]
    fn two_relations_declaring_different_windows_are_refused_by_name() {
        const TWO: &str = "\
schema s {
    currency usd { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 1_000_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
    ledger journal {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 250_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
}
";
        let mut answers = std::collections::BTreeSet::new();
        for _ in 0..10 {
            let r = super::declared_idem_window(TWO);
            let e = r.expect_err("two disagreeing windows must be refused, not chosen between");
            assert!(
                e.contains("postings") && e.contains("journal"),
                "the refusal must name both relations, and name them in a stable order: {e}"
            );
            answers.insert(e);
        }
        assert_eq!(
            answers.len(),
            1,
            "the diagnostic itself must not depend on hash order: {answers:?}"
        );

        // Agreeing declarations are not a disagreement.
        let agree = TWO.replace("250_000.epochs", "1_000_000.epochs");
        assert_eq!(super::declared_idem_window(&agree), Ok(Some(1_000_000)));
    }

    #[test]
    fn an_insert_naming_an_undeclared_currency_is_refused_by_name() {
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "insert into postings values (777002, 1, 999, 500), (777002, 2, 999, -500)".into(),
            ),
            &e,
        );
        let msg = format!("{out:?}");
        assert!(
            !out.iter()
                .any(|m| matches!(m, Backend::CommandComplete(t) if t.starts_with("INSERT"))),
            "an undeclared currency must not commit: {msg}"
        );
        assert!(
            msg.contains("22023"),
            "and must be refused with `invalid_parameter_value`, which is what the value is:              {msg}"
        );
        assert!(
            msg.contains("999") && msg.contains("not declared"),
            "and must name the currency it refused, or the caller cannot act on it: {msg}"
        );
    }

    /// The declared currency still commits. A check that refused everything would pass the
    /// test above and be useless.
    #[test]
    fn an_insert_in_the_declared_currency_still_commits() {
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("insert into postings values (777010, 1, 0, 500)".into()),
            &e,
        );
        assert!(
            out.iter()
                .any(|m| matches!(m, Backend::CommandComplete(t) if t == "INSERT 0 1")),
            "{out:?}"
        );
    }

    #[test]
    fn a_transaction_seals_as_one_epoch_and_a_rollback_seals_nothing() {
        let (mut s, e) = (session(), engine());
        s.handle(Frontend::Query("begin".into()), &e);
        s.handle(
            Frontend::Query("insert into postings values (10, 4004, 0, 100)".into()),
            &e,
        );
        s.handle(
            Frontend::Query("insert into postings values (11, 4004, 0, -100)".into()),
            &e,
        );
        let before = e.frontier();
        s.handle(Frontend::Query("commit".into()), &e);
        assert_eq!(
            e.frontier(),
            before + 1,
            "two statements, one epoch: there is no moment in which half of it happened"
        );
        assert_eq!(e.state().appended.len(), 1);
        assert_eq!(
            e.state().appended[0].1,
            2,
            "both rows in the one sealed set"
        );

        // And the rollback: buffered rows are discarded, and nothing was sealed to
        // compensate for.
        s.handle(Frontend::Query("begin".into()), &e);
        s.handle(
            Frontend::Query("insert into postings values (12, 5005, 0, 1)".into()),
            &e,
        );
        let f = e.frontier();
        let out = s.handle(Frontend::Query("rollback".into()), &e);
        assert_eq!(e.frontier(), f, "a rollback seals nothing");
        assert!(
            out.iter()
                .any(|m| matches!(m, Backend::CommandComplete(t) if t == "ROLLBACK 1")),
            "and says how many rows it discarded: {out:?}"
        );
    }

    #[test]
    fn a_repeated_transaction_identity_is_refused_with_a_unique_violation() {
        // Idempotency over a wire that can drop an acknowledgement. The SQLSTATE matters:
        // a driver reads it, and `23505` says "you already did this" while the `42P01`
        // this server used to answer to everything says "no such table".
        let (mut s, e) = (session(), engine());
        s.handle(
            Frontend::Query("insert into postings values (7, 6006, 0, 5)".into()),
            &e,
        );
        let again = s.handle(
            Frontend::Query("insert into postings values (7, 6006, 0, 5)".into()),
            &e,
        );
        let Some(Backend::ErrorResponse { code, .. }) = again.first() else {
            panic!("{again:?}")
        };
        assert_eq!(code, "23505");
    }

    #[test]
    fn an_update_against_the_ledger_is_refused_with_feature_not_supported() {
        let (mut s, e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("update postings set amt = 0 where acct = 1001".into()),
            &e,
        );
        let Some(Backend::ErrorResponse { code, detail, .. }) = out.first() else {
            panic!("{out:?}")
        };
        assert_eq!(code, "0A000");
        assert!(detail
            .as_deref()
            .unwrap_or_default()
            .contains("compensating entry"));
    }

    #[test]
    fn a_diagnostic_carries_a_sqlstate_from_its_own_family() {
        // Every Niles diagnostic used to reach the client as `42P01` — *undefined_table* —
        // so a driver's retry logic was told the same untrue thing about a syntax error, a
        // currency mismatch and a rung violation alike.
        assert_eq!(pg_wire::sqlstate_for("NL0001"), "42601");
        assert_eq!(pg_wire::sqlstate_for("NL0250"), "42804");
        assert_eq!(pg_wire::sqlstate_for("NL0300"), "23000");
        assert_eq!(
            pg_wire::sqlstate_for("NL0312"),
            "42501",
            "a missing capability is a privilege failure, not an integrity one: a driver \
             retries the second and must not retry the first"
        );
        assert_eq!(pg_wire::sqlstate_for("NL0400"), "42P20");
        assert_eq!(pg_wire::sqlstate_for("NL0501"), "0A000");
        assert_eq!(pg_wire::sqlstate_for("IR013"), "58000");
        assert_eq!(
            pg_wire::sqlstate_for("ZZ9999"),
            "XX000",
            "an unclassified code is an internal error, not a guess"
        );
    }

    /// **`explain` reaches the wire and names a class a client can act on.**
    #[test]
    fn explain_over_the_wire_names_the_serve_path() {
        let e = engine();
        let mut s = session();
        for (sql, want) in [
            (
                "explain select acct, sum(amt) from postings where acct = 7 group by acct",
                "view",
            ),
            (
                "explain select acct, sum(amt) from postings where acct = 7 and cur = 0 group by acct",
                "index-fold",
            ),
            (
                "explain select acct, sum(amt) from postings group by acct having acct = 7",
                "index-fold",
            ),
            (
                "explain select acct, sum(amt) from postings group by acct",
                "fold",
            ),
            ("explain select distinct acct from postings", "materialise"),
        ] {
            let out = s.handle(Frontend::Query(sql.into()), &e);
            let rows = pg_wire::decoded_rows(&out);
            assert_eq!(rows[0][0], Some("serve path".into()), "`{sql}`: {out:?}");
            assert_eq!(rows[0][1], Some(want.into()), "`{sql}`");
            // And the second row says what the class costs, so a reader who does not know
            // the four names is not left with a word.
            assert!(
                rows[1][1].as_deref().unwrap_or("").len() > 30,
                "`{sql}`: the class is named and not explained"
            );
        }
        // A statement that does not compile is a diagnostic, not a class.
        let out = s.handle(
            Frontend::Query("explain select nope from postings".into()),
            &e,
        );
        assert!(
            out.iter()
                .any(|m| matches!(m, Backend::ErrorResponse { .. })),
            "{out:?}"
        );
    }
    /// **A `Bind` asking for binary is read, and one that does not is not.**
    ///
    /// The result format codes sit at the end of the message, after the parameter formats and
    /// the parameters themselves — so reading them means walking the whole body correctly,
    /// and a parser that guessed at the offsets would answer plausibly and wrongly.
    #[test]
    fn the_result_format_codes_are_read_from_where_the_protocol_puts_them() {
        // portal, statement, n param formats, n params, n result formats
        let bind = |param_formats: &[i16], params: &[Option<&[u8]>], results: &[i16]| -> Vec<u8> {
            let mut b = Vec::new();
            b.extend_from_slice(b"p\0");
            b.extend_from_slice(b"s\0");
            b.extend_from_slice(&(param_formats.len() as i16).to_be_bytes());
            for f in param_formats {
                b.extend_from_slice(&f.to_be_bytes());
            }
            b.extend_from_slice(&(params.len() as i16).to_be_bytes());
            for p in params {
                match p {
                    None => b.extend_from_slice(&(-1i32).to_be_bytes()),
                    Some(v) => {
                        b.extend_from_slice(&(v.len() as i32).to_be_bytes());
                        b.extend_from_slice(v);
                    }
                }
            }
            b.extend_from_slice(&(results.len() as i16).to_be_bytes());
            for f in results {
                b.extend_from_slice(&f.to_be_bytes());
            }
            b
        };

        // No result codes: every column text, which is the protocol's default and what every
        // client that has never heard of this gets.
        assert!(!binds_binary(&bind(&[], &[], &[])));
        assert!(!binds_binary(&bind(&[], &[], &[0])));
        // One code, applied to every column.
        assert!(binds_binary(&bind(&[], &[], &[1])));
        // One per column, mixed.
        assert!(binds_binary(&bind(&[], &[], &[0, 1, 0])));
        assert!(!binds_binary(&bind(&[], &[], &[0, 0, 0])));
        // **With parameters in the way**, which is where an offset error would show. A null
        // parameter carries no bytes and a present one carries its length first.
        assert!(binds_binary(&bind(
            &[0, 0],
            &[Some(b"1234"), None, Some(b"x")],
            &[1]
        )));
        assert!(!binds_binary(&bind(
            &[0, 0],
            &[Some(b"1234"), None, Some(b"x")],
            &[0]
        )));
        // A truncated message is not binary rather than a panic: a client can send anything.
        assert!(!binds_binary(b"p\0s\0"));
        assert!(!binds_binary(b""));
    }
}
