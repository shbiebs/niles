# Work Order 7 — structural pass (Fable, second pass)

**Auditor:** Fable (Claude Fable 5.1), structural pass, after the GPT 6 Astra brief was refused.
**Audited:** Niles `d9149a8` on `c6/audit-cycle-7` (engine identical to `d9c8699`), GBS `e803b7d`.
**Method:** reading, with three live probes where a reading needed a witness. No re-measurement of
anything the empirical pass measured. Nothing written into either repository.
**Attribution for the executor:** from the executing session's own instructions, or "none required".

---

## 0. Preflight

Host D for this pass is the same 2-core container class as the empirical pass's Host A:

```
uname   Linux 6.18.44-fc-v24 x86_64      cores 2 (no cgroup quota file)   mem 7.8 GiB
mount   /dev/vda ext4 rw,relatime         barrier fdatasync 109.6 µs -> 9,125/s  VERDICT plausible
rustc   1.95.0 (== pin)  clippy/rustfmt/valgrind/strace present  PostgreSQL 16.13 fdatasync
trees   niles d9149a8 c6/audit-cycle-7 (clean)   gbs e803b7d c6/07-hold-index (clean)
```

| admissibility (D) | |
|---|---|
| storage evidence | yes for this container; none used — this pass measured nothing durable |
| >3-core curves | no; none used |
| toolchain newer than pin | no |
| what this pass may claim | source-level facts (HI); three probe witnesses on D, each a yes/no |

---

## 1. Executive judgement

**On the empirical work order.** Every finding I could check from the source, I confirmed: F-26's
payload is the epoch string and nothing replays (`rev_engine.rs:615`, `with_durable`); F-27's cause
is the applied-versus-visible gap and the fix it proposes is **licensed by the algebra** (§2, T-02
verdict); F-30's lock enumeration is complete and the channel under B is unbounded; F-29's
instance-fragility is exactly what the thesis's own contract table invites, because the thesis
prints absolute figures from an unnamed instance. I overturn one thing the empirical pass and both
cycle-7 briefs repeated: **the hash chain is not a placeholder.** It has been SHA-256 since
ADR 0003; the README and the thesis's implementation table are what is stale (F-38). And I found
four things a reading was needed for, three of them correctness.

**The definitions.** *An efficient language* is one whose checked facts are **trusted by the engine
and enforced at every ingress** — measured as (a) run-time checks that duplicate a static proof
(cost, the redundancy the brief expected) and (b) static proofs the engine fails to protect (a hole,
which is what I actually found: F-41). Its memory efficiency is **the bytes per row its value model
mandates over the schema's packed width**: `Value::Int(i128)` is 16 bytes plus a discriminant, a
five-column row as `Vec<Value>` is ~144 bytes against a 40-byte posting — which is most of the
173 B/row the empirical pass measured. The overhead is the language's, not the engine's, and LC-03
is a sub-question of it. *An efficient core-banking system* cannot be defined yet: with no product
trace (F-24) and state that does not survive a restart (F-12), the only admissible GBS task is the
one that makes measurement possible, and I confirm the empirical pass's refusal.

**On the goals.** Nilestream's speed under concurrency is now hardware-bound and its memory
sub-linear; its durability is a stub (F-26) with two further holes on the failure path (F-39,
F-40); its currency safety — Contribution 4's headline — **is undischarged at the wire** (F-41).
Niles the language is sound for programs and silent about data. GBS is not measurable and, since
T-06, not buildable against current Niles (F-47).

**What outranks everything.** F-41: the daemon accepted a currency the schema never declared and
then answered a "balance" that was USD plus that currency. It is one `INSERT`. The compiler proves
it cannot happen in a Niles program; the PostgreSQL wire is not a Niles program.

---

## 2. Findings — continuing from F-38

EV = impact × confidence ÷ cost (1–5 each; ties favour correctness). All HI unless marked.

### F-41 — The wire ingress accepts an undeclared currency, and a `group by acct` then sums across currencies — EV 25 (5×5÷1) — correctness, Contribution 4

**Witness** (`nilestreamd`, default schema, which declares exactly `currency usd { scale: 2 }`):

```
select acct, sum(amt) from postings where acct = 1 group by acct      -> 1 | 324 | 2999
insert into postings values (777002, 1, 999, 500), (777002, 2, 999, -500)   -> INSERT 0 2
select acct, sum(amt) from postings where acct = 1 group by acct      -> 1 | 824 | 3000
select acct, cur, sum(amt) ... group by acct, cur                     -> 1|0|324   1|999|500
```

**324 USD + 500 of currency 999 = "824"**, served as the balance of account 1.

**Mechanism.** `session.rs` `parse_insert` reads `cur` as a bare `u32` (`:1250`); `Ledger::submit`
checks `Duplicate`, `Overflow`, `Unbalanced` per `(txn, cur)` and nothing else (`ledger.rs:188–204`);
no site compares `cur` against the catalog's declared currencies. The compiler accepted `sum(amt)
group by acct` **because the schema declares one currency** — the currency-row solver's proof is
correct and its premise ("every posting is USD") is a schema invariant the engine does not enforce at
ingress. `answer_from_view` correctly refuses when `currency_count() != 1` (it saw 2) — and the fold
path then evaluates the SQL as written, which is exactly the cross-currency sum the thesis says is
impossible. `explain` still reports `serve path | view` for that statement, because `serve_path_at`
classifies from the circuit alone.

**This is the sharpest instance of "compiler knows, engine does not re-establish."** Not
redundancy — a hole. Contribution 4 ("a well-typed program cannot mismatch currencies") is true of
programs and silent about the wire, and every benchmark row is driven over the wire.

**Repair.** At the wire ingress: every `cur` must be a declared currency, refused by name otherwise
(`NL…` diagnostic, SQLSTATE `22023`); every `amt` must fit the declared scale. At the engine: a
`group by` that omits `cur` over a base holding more than one currency must **refuse**, not fold —
the same refusal the compiler would give if the schema declared two. `explain` must classify from
`serve_path_at`'s actual decision. **Gives up:** nothing the thesis wants to keep.

### F-39 — A session anchors itself at an epoch that has not been published, before the barrier returns — EV 20 (4×5÷1) — correctness

`session.rs` `insert`: `match engine.append(rows, &txn) { Ok(epoch) => { self.observe(epoch); … } }`.
`append` returns the **applied** epoch before its barrier; `observe` raises the session anchor to it.
`serve` then waits on the barrier and, on failure, replaces the reply with `58030` — but the session's
anchor already points past `visible`. The session's next read is anchored at an epoch no barrier
confirmed; the scan path reads `upto = anchor.min(base.head())` and the rows **are** in the base.
**Read-your-own-failed-write.** Small window (a failed barrier), but it is durable-before-visible
violated on the one path the thesis singles out as a rung (ℓ₂, read-your-writes).

**Repair.** `observe` after `wait()` succeeds — either `serve` re-anchors the session from the
visible frontier after the wait, or `append` returns the token and the session observes on
success. **Gives up:** nothing.

### F-40 — A failed barrier is masked by the next successful one — EV 10 (5×4÷2) — correctness

`Pending::wait` publishes with `fetch_max(epoch)` over **ledger** epochs, which are contiguous.
Epoch *n* fails its barrier: its rows are applied, its record is not on disk (or half is). Epoch
*n+1* succeeds: `fetch_max(n+1)` makes *n* visible to everyone. Meanwhile the sealer
(`sequencer.rs`, the `Err(e)` arm) replies `Io` to the batch and **continues** with the next. Today
this is moot — nothing recovers — but under T-01 a replay would meet either a gap or a record that
fails its checksum, stop there (`TruncationCause`), and lose every acknowledged epoch after it.

**Repair.** A barrier failure is **fail-stop**: the sealer stops, every later submit gets
`ShuttingDown`, the daemon refuses appends until restarted-and-recovered. Then `fetch_max`'s
contiguity assumption is true. **Gives up:** availability under a storage fault — which a ledger
should give up. Route as **LC-21** because it is a policy.

### F-38 — Two hash chains that never meet, and a placeholder that is not one — EV 15 (3×5÷1) — claim-versus-code, both directions

**Direction one.** `README.md:55` and `thesis/07-implementation.md:69` say "built with a
**placeholder hasher** (ADR 0002)". `nilestream-ledger/src/chain.rs` is FIPS 180-4 SHA-256 with the
NIST vectors, since **ADR 0003**, whose text says why. The empirical work order, both cycle-7 briefs
and the cycle-6 audit repeated the README. The artefact is *better* than its documentation and three
audits believed the documentation.

**Direction two, which matters for T-01.** There are two chains. The in-memory `Ledger` chains
`H(parent ‖ rows)` (`ledger.rs:140`); the segment chains `H(parent ‖ epoch ‖ payload)`
(`segment.rs:81`) and **verifies it forward on recovery** (`recover`, `:63`). They share a hasher and
nothing else: the segment stores neither the ledger's hash nor its rows, and today its payload is
the epoch string, so the verified chain protects a sequence of integers. T-01 as the empirical pass
wrote it would put rows in the payload and replay through `Ledger::submit` — which **rebuilds** the
ledger's chain from the seed rather than verifying it against anything. A chain rebuilt on replay
protects nothing across a restart whatever the hasher.

**Repair.** One chain: the ledger's `EpochRec.hash` **is** the segment record's hash, over one
canonical row encoding, and replay **verifies** each recomputed epoch hash against the record's
before applying it. Fix the README and the thesis table. **Gives up:** nothing.

### F-47 — T-06 changed a public trait and GBS no longer builds against Niles — EV 20 (4×5÷1) — correctness/process

`make gate` in GBS: `error[E0053]: method reconstruct has an incompatible type for trait` in
`gbs-nilestream/src/evicting.rs:163` (and `deltas_at`, and a `&mut base`). `Base::reconstruct` and
`deltas_at` went `&mut self → &self` in `d681f0d`; the adapter implements the old signature. It is a
three-line fix. The finding is that **no check exists in either direction**: the Niles gate never
builds the adapter, GBS's gate was never run in cycle 6 or 7 until now, and work order 6's rule —
"adapter commits follow their Niles producer commit and record its SHA" — was not applied to T-06
because nobody listed `Base` as a public trait with a downstream. **Repair:** fix the adapter; add a
Niles-side test that builds `gbs-nilestream` when `GBS_ROOT` is set, skipping by name otherwise; list
the traits GBS implements in `docs/SPEC-ENGINE.md`. **Gives up:** nothing.

### F-44 — The eviction budget bounds resident values, not the per-key metadata — EV 15 (3×5÷1) — memory, structural

`rev.rs:171,174`: `reads_of: BTreeMap<Key, u64>` and `last_read: BTreeMap<Key, u64>` are inserted on
every read (`:196–197`) and **never removed** — not on eviction, not ever. `Hole(e)` retaining a
version per evicted key is by design and the thesis says so ("honest absence retains only a
version"); two more maps of `Vec<i64>` keys plus counters are not. Over 10,000 accounts this is
bounded by |K|; the thesis's key space is unbounded (F1). **The "memory under a budget" claim has an
unbounded shadow.** **Repair:** evict the metadata with the entry, or bound it with the budget.
**Gives up:** LRU/LFU accuracy for keys evicted and re-read, which is the trade the budget already
makes.

### F-43 — `idem: IdemKey window 30.days` is a declaration with no runtime — EV 10 (4×5÷2) — correctness/memory

`window` appears in no crate below the compiler (grep of `niles-ir`, `nilestream-core`,
`nilestream-ledger`, `proto-engine`). The sequencer's "window" is `seen: BTreeMap<String, u64>`,
appended forever and **rebuilt from the whole segment on every open** (`sequencer.rs:164–191`); the
in-memory `Ledger` has its own, also unbounded. So the declared window is infinite, memory grows at
~100 B per committed transaction forever, and reopen is O(history). The thesis (§3.20) says the key
has "a defined admission semantics"; the defined part is the compiler's, the semantics are the
engine's, and they do not meet. **Repair:** carry the window through the IR to the sequencer; prune
by epoch age; make the retained set's size a stats column. **Gives up:** a retry older than the
window is admitted as new — which is what "window" means.

### F-42 — Stale reasoning, including the daemon's own banner — EV 10 (2×5÷1) — claim-versus-code

`main.rs:191–192` prints, on every start: *"appends are durable; the read side is in-memory and
serialises on one engine mutex."* Both halves are false since T-05/T-06/F-26. Also stale:
`lockstats.rs:1` (module header), `rev_engine.rs:188` ("the daemon holds one mutex"), `:230` ("the
caller holds the engine's lock" — it holds B), `daemon.rs:197` (the guard module's doc). The
codebase carries its reasoning in comments; a reader trusting the banner would conclude the
opposite of the truth on both counts. **Repair:** a source-level test that the banner names no
mutex and does not say "durable" until T-01 lands. **Gives up:** nothing.

### F-46 — Appendix C's determinism obligation has a test for the compiler and none for the engine — EV 12 (3×4÷1) — instrument

C.5 states: "for every (IR circuit, base prefix, anchor), the emitted answer bytes are identical on
every supported target." The only cross-target evidence in the project is the compiler corpus
(`counterproposal`, 4/4 on arm64 in `run4.sh`, table identical to x86_64). No engine answer has ever
been compared across targets. **Repair:** a fixture — the E1 conservation run's rendered output
hashed — committed from x86_64 and asserted on arm64 by the existing Host C scripts. Costs one file.
**Gives up:** nothing. Also discharges §6.6 of the brief: no further Host C script is needed.

### F-45 — The extended-protocol plan cache is bounded only by `Close` — EV 4 (1×4÷1) — memory, minor

`extended.rs:149`: `statements: HashMap<String, Prepared>` removed only by `close_statement`. A
long-lived connection preparing named statements without closing grows it without bound. Per
session, dropped on disconnect. Note it; do not spend the cycle on it.

### Verified, not findings

- **T-02's fix is licensed by the algebra.** Definition 3.3 certifies `M(k) = (v, e)` at *e*; the
  proof of Theorem 4.1 step (3) advances a certified entry by exactly the deltas in `(e_old, e]`;
  `apply_epoch` restamps on every delta (`rev.rs:311`). So a resident, unpinned `Present(v, stamp)`
  with no delta in `(stamp, applied]` is certified at **every** anchor in `[stamp, applied]`, and
  `stamp ≤ a ≤ applied` is exactly that condition. The pinned path (certified at its stamp only), the
  `Materialize::Full` first-delta install (stamped at the delta's epoch, so `stamp > a` for earlier
  anchors), and a resident key whose first delta in `(a, applied]` restamps it above *a* all fall on
  the reconstruct side of the condition. **LC-17 closes**: the residual is keys with a delta after
  the anchor, which reconstruct and are correct. **Add to T-02:** `hits` must not count a discarded
  hit.
- **The lock order** O < B < P < V < C, S a leaf: confirmed from the source at every site; the
  `append`→sealer channel is `std::sync::mpsc::channel()`, unbounded. The guard's `.lock()` search is
  the only fragility; name the lock in the test (`runtime` + `.lock()`).
- **`e803b7d`'s invariant** (map says consumed at *e* ⇒ an entry in *e* consumes it) holds by
  construction: both writers of `consumed` (`view.rs:333`, `:422`) derive from `e.consumes` on the
  entry being appended. Recommend a fold-equals-map property test as the guard.
- **Appendix E is honest**, and the brief was stale: `bootstrap/parser.niles` (1,647 lines) exists
  and is tested; the type-checker in Niles does not. E.0 says exactly that.
- **§3's data-race-freedom text is properly hedged** ("bounded model check, which has not been
  done") but says "two lock disciplines"; there are now six locks and four atomics.
  `MISMATCH-lock-count`, minor.
- **The corpus on arm64** (§6.6): discharged by `run4.sh`'s first T-03 line — `counterproposal`
  4/4 including the table assertion. No Host C script from this pass.

### `MISMATCH` register from this pass

| id | thesis | code |
|---|---|---|
| `MISMATCH-durability-restart` | 06:38, 07:19 "recovery replays… chain verified forward"; 09:457 "a reopened ledger's read model agrees with a fold of the recovered journal" | true of `nilestream-ledger`; false of the daemon E16/E19 measure (F-26) |
| `MISMATCH-contract-instance` | 09:686–702 prints absolute E16 figures from an unnamed instance as the contract | flips MET/NOT MET across instances of one class (F-29) |
| `MISMATCH-placeholder-hasher` | 07:69 "placeholder hasher (ADR 0002)" | SHA-256 since ADR 0003 (F-38) |
| `MISMATCH-currency-at-wire` | Contribution 4, "cannot mismatch currencies" | undischarged at the wire ingress (F-41) |
| `MISMATCH-idem-window` | §3.20 "a defined admission semantics" for the idempotency key | the window never reaches the engine (F-43) |
| `MISMATCH-lock-count` | 03:231 "two lock disciplines" | six locks, four atomics |

---

## 3. Tasks — integrated with the empirical order

The empirical pass's T-01…T-04a stand. This pass inserts **T-00** ahead of them, adds to T-01 and
T-02, and appends T-05…T-08. **Cut line after T-04a.** Every task: guard fails on the reverted
change, transcript in the report; the report carries §3 verbatim with a status per line (F-32).

### T-00 — GBS builds against Niles again, and a check exists — closes F-47

*Files:* `gbs-nilestream/src/evicting.rs` (`&self` on `reconstruct`, `deltas_at`; `&base`), a
Niles test `adapter_builds_when_present` gated on `GBS_ROOT`, `docs/SPEC-ENGINE.md` (public traits
with downstreams: `Base`, `Serving`).
*Target:* GBS `make gate` green against Niles `d9149a8`; the Niles test skips **by name** when
`GBS_ROOT` is unset and fails when the adapter does not compile.
*Guard:* revert the adapter signature → the Niles test fails with the E0053 text.

### T-01 — Durable means the rows come back — closes F-26; **extended by F-38, F-39, F-40**

As written by the empirical pass, plus: **(a)** one chain — the ledger's `EpochRec.hash` is the
segment record's hash over the canonical row encoding, and replay **verifies** each recomputed hash
against the record before applying (F-38); **(b)** the session observes only after `wait()` succeeds
(F-39); **(c)** a barrier failure is fail-stop — sealer stops, later submits get `ShuttingDown`, the
daemon refuses appends until reopened (F-40, pending LC-21); **(d)** the invariant between the two
epoch clocks — `record.epoch == ledger_epoch − seed_head` — stated and asserted on replay.
*Additional guards:* inject a barrier failure at epoch *n* then succeed at *n+1* → a read at the
visible frontier must not see *n*'s rows (fails on `fetch_max` alone); tamper one byte of one
record's payload → reopen refuses at that record rather than rebuilding past it.

### T-01b — The wire enforces the schema's currency premise — closes F-41

*Files:* `session.rs` (`parse_insert`, `insert`), `rev_engine.rs` (the fold path's refusal,
`serve_path_at`), `niles-lang` diagnostics (a named code).
*Baseline:* the witness in F-41.
*Target:* an `INSERT` naming an undeclared currency is refused by name; an `amt` outside the declared
scale is refused; `sum(amt) group by acct` over a base with >1 currency is **refused** at the engine
with the compiler's own diagnostic, not folded; `explain` reports the path `query` will take.
*Acceptance:* the F-41 witness as a test — the second `select` must refuse, the `group by acct, cur`
form must answer.
*Guard:* remove the ingress check → the witness test fails on "824".
*Guardrails:* no change to the compiler's solver; the schema stays the single source of declared
currencies.

### T-02 — The view answers exactly at the anchor when it can — closes F-27; **algebra confirmed**

As written, plus: `hits` counts only answers served; the discarded-hit count is its own column.

### T-03, T-04, T-04a — as written by the empirical pass.

---
*Cut line.*

### T-05 — The window is a window — closes F-43, F-44

Carry `window` through the IR to the sequencer and the ledger's `seen`; prune by epoch age; evict
`reads_of`/`last_read` with the entry; stats columns for both retained sets. Guard: a test that
commits 2× budget distinct keys and asserts the metadata maps' length ≤ budget; a test that a key
older than the window is admitted as new.

### T-06 — Say what is true — closes F-42, F-38's docs half, the `MISMATCH` register

Banner, `lockstats.rs` header, the four comments; `README.md:55`; `thesis/07:69`; each `MISMATCH`
either resolved in prose or left as a marked `MISMATCH-<id>` sentence in the chapter. Guard: a
source test that the banner names no mutex.

### T-07 — The engine's determinism fixture — closes F-46

E1's rendered output hashed, committed from x86_64, asserted by `run4.sh`'s successor on arm64.

### T-08 — GBS: lifecycle and holds on the ledger — F-12, as F-26's twin

Unchanged from work order 6; sequenced after T-01 so the two replay designs share one encoding
discipline. **No GBS efficiency task is admissible before this and T-14.**

---

## 4. Branch stacks and merge order

Niles: `c6/audit-cycle-7` (`d9149a8`) → `c7/00-adapter-check` → `c7/01-durable-rows` →
`c7/01b-currency-at-wire` → `c7/02-exact-at-anchor` → `c7/03-mixed-row` → `c7/04-contract-ab` →
`c7/04a-lints`. GBS: `c6/07-hold-index` → `c7/00-adapter` (three lines, records Niles's SHA) →
`c7/08-lifecycle-ledger`. **T-00's two halves merge in one window**; T-01b before T-02 so the mixed
row never measures a cross-currency fold.

## 5. Validation protocol

| command | green means | red means |
|---|---|---|
| preflight | environment stated | nothing later interpretable |
| fmt, clippy on 1.95.0 **and** 1.97.1 | the gate is about the code | it is about the calendar |
| `cargo test --offline --workspace` both trees, serial, `PGPORT` | semantics | no performance claim |
| GBS `make gate` against the named Niles SHA | the pair interoperates | cross-repo claims blocked (F-47) |
| the F-41 witness test | the wire enforces the schema | Contribution 4 is a program-only claim |
| the T-01 crash test + the injected-failure test | rows survive; a failed epoch is never visible | every `durable` row is void |
| T-02's fallback-rate test ≤5% | the view serves under writes | the mechanism is off |
| `make reproduce`, `make fsync-proof` | reproducible; barrier reaches the kernel | stale; volatile |

## 6. Open questions

- **LC-17 — closed** (T-02's condition is the algebra's own certification interval).
- **LC-19 — sharpened.** Within a process the split preserves the wire claim. Across a restart it is
  void until T-01. On the failure path it is void until F-39 and F-40 are closed. State it as three
  clauses in the thesis.
- **LC-20 — to the thesis.** A bounded-staleness rung anchored at *applied* rather than *visible*
  would let the view serve without any fallback and would be exactly ℓ₀ of §3.7 with K = the
  in-flight batch depth. The ladder already has the rung; nothing lets a session ask for it.
- **LC-21 (new) — fail-stop on barrier failure?** The alternative is a marked-void epoch with a
  tombstone in the segment. Fail-stop is simpler and is what a ledger should do; it costs
  availability under a storage fault. Decide before T-01.
- **LC-03 — framed.** It is one term of the language's mandated bytes-per-row (§1); measure it
  against the 173 B/row figure, not on its own.
- **LC-06 — recommend** the thesis quote allocator-level bytes per row (deterministic) and report
  process RSS per host as context, never as a ratio.
- **LC-16** — moot until T-01; then durable by default with `--volatile` for benchmarks.
- LC-05, LC-07, LC-13 — unchanged; LC-07 gains the note that F-27's cost surfaces on E23's base axis.

## 7. Host C scripts

**None from this pass.** §6.6 is discharged by `run4.sh`; the empirical pass's re-run of `run4.sh`
after T-01/T-02 stands.

## 8. Reporting requirements for the executor

The empirical pass's §8, plus: the `MISMATCH` register with each entry resolved or carried; the
F-41 witness before and after T-01b; GBS `make gate` output against the named Niles SHA; the two
T-01 additional-guard transcripts; and **exactly three material facts found while executing that
neither work order covered**.

## 9. Three things this pass found that its brief did not cover

1. **The hash chain is real and the documentation is what is wrong** (F-38). Three audits and both
   cycle-7 briefs believed `README.md:55`; nobody opened `chain.rs`.
2. **T-06 broke a downstream repository and no gate on either side could see it** (F-47). A public
   trait with an out-of-tree implementor is an API, and the project has no list of them.
3. **The single most dangerous input to this system is an `INSERT` with a currency the schema never
   heard of** (F-41) — and it is accepted with `INSERT 0 2`, which is the opposite of honest refusal,
   the project's own rule.
