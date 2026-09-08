# Cycle 9 — consolidated work order (Fable + Astra audits)

**Auditors:** Claude Fable 5.1 (cloud container, 2 cores; ran the gate, the probes and the corpus) and
Astra (source reading of the Mac trees; gate run by the author on request). This document merges the
two work orders: Fable's measurements and findings are §0–§2, Astra's findings — each marked with
what Fable verified in source — are §2A, and everything from §3 on is a single consolidated plan.
**Executor:** the next Opus session.
**Trees audited:** niles `5a64595` (`c9/00-briefs`, on top of `5eb0bb2`), gbs `963e4d9`.
**Brief:** `docs/audit/cycle-9/fable-audit-prompt.md` (§12 is the shape of this document).

Every figure below says which host it came from. Counts (instructions, allocations, bytes, test
verdicts, corpus tallies) travel; wall clock does not and is labelled *this host* where it appears.

---

## 0. Preflight, verbatim, and the admissibility table

`bash docs/audit/cycle-9/preflight.sh /home/claude/work/niles /home/claude/work/gbs`, run at
`5a64595` / `963e4d9` on 2026-09-08. Two defects in the script itself are reported first,
because an instrument is audited before the code it measures (F-74): its banner still says
"cycle-7 audit preflight" (both copies, line 22), and section F still says "the FOUR untracked
files" — there have been five since cycle 8 (`niles/.DS_Store`). Neither changes a measurement.

```
cycle-7 audit preflight — 2026-09-08T04:36:18Z

=== A. host
uname            : Linux 6.18.44-fc-v24 x86_64 GNU/Linux
cores (nproc)    : 2
cpu model        : Intel(R) Xeon(R) Processor @ 2.80GHz
memtotal         : 7.8 GiB
cgroup cpu.max   : n/a
cgroup mem.max   : n/a

=== B. filesystem under the tree — THE decisive section
path             : /home/claude/work/niles
mount            : /dev/vda ext4   rw,relatime,resv_strict,resuid=65534,resgid=65534
free space       : 18G avail of 252G
barrier          : fdatasync
median           : 126.2 us  ->  7,923 barriers/s
spread           : 124.4-870.8 us
VERDICT          : plausible as a real barrier for THIS container. Still a
                   host-shaped number: usable for within-host ratios only.

=== C. toolchain
pin              : channel = "1.95.0" — does NOT resolve here — every probe uses RUSTUP_TOOLCHAIN=stable
rustc            : rustc 1.95.0 (59807616e 2026-04-14)   (toolchain: stable)
cargo            : cargo 1.95.0 (f2d3ce0bd 2026-03-21)
host triple      : x86_64-unknown-linux-gnu
rustfmt          : rustfmt 1.9.0-stable        clippy : clippy 0.1.95
1.97.1           : not installed here — the author runs that gate on the Mac
valgrind         : valgrind-3.22.0     python3 : 3.11.15     git : 2.43.0     strace : present
CARGO_TARGET_DIR : <unset, which is correct>

=== D. PostgreSQL
psql / postgres  : PostgreSQL 16.13 (Ubuntu 16.13-0ubuntu0.24.04.1)
pg_isready       : /var/run/postgresql:5432 - accepting connections
bench PostgreSQL : 127.0.0.1:5433 - no response
settings (5432)  : fsync=on synchronous_commit=on full_page_writes=on wal_level=replica
                   shared_buffers=16384 work_mem=4096 max_wal_size=1024 wal_sync_method=fdatasync

=== E. network egress
github.com 400 · crates.io 403 · static.rust-lang.org FAIL   (the executor has none of these either)

=== F. trees
/home/claude/work/niles  HEAD 5a64595 (c9/00-briefs)  dirty: 0 entries
/home/claude/work/gbs    HEAD 963e4d9 (c8/01-idem-window-epochs)  dirty: 0 entries
```

### 0.1 Admissibility (this container)

| question | answer |
|---|---|
| cores actually granted | **2** (`cpu.max` absent → nproc governs; no cgroup quota to divide) |
| barrier + median + rate | fdatasync · 126.2 µs · 7,923/s (7 probes, spread 124.4–870.8 µs) |
| storage evidence? | yes — ext4 on `/dev/vda`, `rw`; `make fsync-proof` shows `fdatasync(3) = 0` reaching the kernel |
| may publish durability rows? | **NO** — container; `--publish` is Host C only, CSVs included |
| may publish >3-core curves? | **NO** — two cores; every connection-count curve here is a ratio at best |
| toolchain newer than 1.95.0? | no — `stable` *is* 1.95.0; the pin does not resolve, so every build is `RUSTUP_TOOLCHAIN=stable` |
| valgrind attribution possible | **yes** — callgrind 3.22, per-function inclusive counts used in §2 (F-68, F-69) |
| PostgreSQL comparison possible | in-process only, 16.13 on 5432 under `wal_sync_method=fdatasync`; `numeric_binary_oracle` green against it; nothing from it is publishable |

### 0.2 The gate, both trees, at the heads above (§8.1) — evidence class: **run, recorded**

| row | niles `5a64595` | gbs `963e4d9` |
|---|---|---|
| `cargo fmt --all -- --check` | exit 0 | exit 0 |
| `cargo clippy --offline --all-targets -- -D warnings` | exit 0 | exit 0 |
| `cargo test --offline --workspace` (default threads) | **994 passed, 0 failed, 7 ignored** (all seven are declared measurements/generators) | **504 passed, 0 failed, 1 ignored** (fixture generator) |
| `make reproduce` | exit 0, clean diff (2 m 28 s this host) | exit 0, clean diff |
| `make fsync-proof` | exit 0 — `fdatasync(3) = 0` | exit 0 |
| `-- --test-threads=8`, run 1 | 994/0 | 504/0 |
| `-- --test-threads=8`, run 2 | **1 FAILED** — `frontiers::tests::a_reader_never_observes_an_epoch_that_was_not_published` ("the reader saw nothing at all; the test proved nothing"); the run stopped at `nilestream-ledger`, 72 passed in that crate | 504/0 |
| `-- --test-threads=8`, run 3 | 994/0 | 504/0 |
| `taskset -c 0 … --test-threads=1` | 994/0 (65 s this host) | not run (gbs was green four times) |

One test changed verdict between runs. It is F-63, and it is the class §9 of the brief named:
the reader spins 200,000 times and asserts it saw *something*; on two cores with eight test
threads the writer thread was not scheduled once before the reader finished. The cycle-8
differential was cured of the same disease by bounding on progress; this test was not. Four more
tests carry an assertion the scheduler decides (§2, F-63) and passed four times here — passing is
not the same as deterministic.

---

## 1. Executive judgement

**Where the three artefacts stand.**

*Niles (the language).* The static half is sound where it is claimed and measured (E18/9.14.3,
E26); the run-time half — what a checked fact is worth once the engine holds it — was unmeasured
until this cycle, and the first measurement is unflattering: on E16's `oltp` statement the session
**re-parses and re-resolves the entire schema on every `INSERT`** to learn the currency table
(F-68: 74.5k of 113.5k instructions per insert, 61%, callgrind, deterministic). The compiler's
verdict is right; the engine does not trust it. The calculus also spells idempotency twice with
different rigor (F-67): the column form is checked to the epoch (NL0215/NL0217); the transaction
form `idem("k", window: 30.days)` is parsed, its key bound, its window counted-and-ignored by the
interpreter and dropped by lowering, and the typechecker's own NL0216 *suggestion text* prescribes
the wall-clock spelling NL0217 refuses. Twenty-five files carry it.

*Nilestream (the engine).* Correct on every input the suite asks; wrong in structure in four places
this cycle found by asking the class rather than the instance. (0) **The acknowledgement** — found
by Astra, confirmed by Fable in source (§2A, A9-F01): durability receipts sit in one engine-wide
vector and are drained by whichever connection asks next, so a client can be told `INSERT 1` before
its own record's fsync returns; and the two idempotency windows count different things called
"epoch" (A9-F02). Those two outrank everything below. (1) **Durability of the record:** a single flipped
bit in a record's *length prefix* — the four bytes the CRC does not cover — makes segment recovery
read the damaged record as a torn tail, truncate to the previous record, and open cleanly: a
ten-record ledger with one bit flipped in record 0's prefix restarts empty with a success code
(F-61; 216 of 8,840 single-bit flips in a corpus, plus every flip in the last record, which the
format cannot tell from a crash). `open_bounded` also builds its idempotency window from the
*unchecked* decoder, so a short envelope yields an empty window and a running sequencer (F-62);
only the daemon's later checked replay saves the daemon, and nothing saves the other callers.
(2) **The lattice's fourth state** is still unreachable (LC-23, `MISMATCH-pending-unreachable`),
and the hole's version — the third state's whole content — is read nowhere (F-66): the runtime
implements ⊥ and Present with a 125-byte-per-key ghost in between, 12.7 MB at 100k keys read
under a 2,500 budget, 98% of the view's live bytes. (3) **The served daemon has no checkpoints**
(F-64): every wall-clock figure in §9.14 came from a history-length fold that §9.4.1 says the cost
law does not hold under, and §9 does not say so.

*GBS.* Unchanged from cycle 8: 23 of 29 product-evidenced, 6 generic-path-only, no product trace
(F-24), lifecycles that do not survive a restart (F-12). Inadmissible for an efficiency claim. The
Loan IQ / Calypso mapping (§2, F-79) finds one shape that is admissible under the brief's rule
because it is also (ii): a pro-rata distribution to a thousand lender shares, as a *trace* the
engine benchmark can replay — the largest single-epoch batch the system would ever seal, and a
conservation obligation with a rounding remainder.

**The three questions of brief §1.**

1. *Is Niles a highly efficient query language, in memory and speed?* Statically, yes as far as
   measured: 15.7 µs to compile the point statement (this host, release, median of 200; 649k
   instructions whole-process), a `Lowered` of 2,281 B. At run time, **not yet**: the engine
   re-derives at least one checked fact per write at 61% of the write's instruction budget, and
   the plan cache keys on literal text so 15–20% of E16's point reads pay a full front end
   (F-69). Both are one-task fixes; neither is measured by any committed row today.
2. *Is Nilestream a highly efficient engine?* On the reference host it beats PostgreSQL on every
   E16 row except the durable one, and its read throughput collapses 4.6× from six to twelve
   connections on one mutex (LC-23). Efficiency is not the binding constraint this cycle;
   **correctness of the durability claim is** (A9-F01, A9-F02, F-61, F-62), and by the brief's own
   rule it outranks everything else in this order. The Pending repair (C9-06) is the efficiency task and
   is designed end to end below.
3. *Is GBS poised to replace Loan IQ / Calypso?* No, and it should not claim to be this cycle.
   What it can claim after C9-G01 is a product trace with a thousand-way pro-rata split whose
   conservation is checked by the ledger rather than by the product — something neither vendor
   document claims to check structurally. The arithmetic for the reachability claim: GBS's
   syndicate today is eleven lenders (`lending.rs:11`); one distribution to 1,000 shares is
   1,001 postings in one epoch, ~532 B/txn in memory (E18 `append_in_memory`) × 1,001 ≈ 533 KB for the epoch — one
   envelope, one fsync, one conservation proof, and a `max_batch` the sealer has never seen. It is
   reachable in one task and it is the *only* product task in the plan — below the cut line, as
   both audits place it.

**What the cycle should do, in one sentence:** make the gate deterministic (C9-00), make the durability claim true — receipts owned by their
request (C9-02) and a recovery that cannot mistake damage for a torn tail (C9-03) — make the
catalog deterministic (C9-04), stop re-checking what the compiler proved (C9-05), then reach the
lattice's fourth state (C9-06); below the line, delete its ghost third (C9-08) — and republish on C
with the daemon configured the way the thesis describes.

---

## 2. Findings

EV = impact × confidence ÷ cost, each 1–5; ties go to correctness. **HI** = engine/instrument,
**HS** = language/semantics. Line numbers are at `5a64595` / `963e4d9`. "Container" figures are
counts unless marked *this host*.

| id | class | one line | EV | H |
|---|---|---|--:|---|
| F-62 | correctness | `open_bounded` builds the idempotency window with the unchecked decoder — an undecodable envelope gives an *empty* window and a running sequencer | 4×5÷1 = 20 | HI |
| F-68 | efficiency (checked-twice) | every `INSERT` re-parses and re-resolves the whole schema: 61% of the insert path's instructions | 4×5÷1 = 20 | HS |
| F-63 | liveness of the gate | five tests assert a count the scheduler decides; one failed 1-in-3 at `--test-threads=8` | 3×5÷1 = 15 | HI |
| F-65 | wrong-measurement | two of T-05's four E18 rows measure stand-in structures, not the ones T-05 changed | 3×5÷1 = 15 | HI |
| F-70 | correctness (silent default) | `nilestreamd` `--port/--accounts/--rounds/--budget` fall back to the default on a parse failure | 3×5÷1 = 15 | HI |
| F-72 | wrong-measurement | E19 `durable`/`mixed` and run4's probe write one zero-amount leg; E16 `oltp` writes a two-leg transfer | 3×5÷1 = 15 | HI |
| F-61 | correctness (durability) | a damaged length prefix is recovered as a torn tail: a prefix with a success code | 5×5÷2 = 12.5 | HI |
| F-71 | correctness (nondeterminism) | `declared_idem_window` takes the first window of a `HashMap` iteration | 2×5÷1 = 10 | HI |
| F-73 | stale-claim | thesis §9.14.1 says the point miss rate "is 0.30"; the committed CSV says 0.14–0.16 | 2×5÷1 = 10 | HI |
| F-74 | stale-claim (bundle) | six comments/scripts that describe a previous cycle's code | 2×5÷1 = 10 | HI |
| F-75 | instrument-gap | `ENGINE_LOCK`/`VIEW_LOCK` histograms never reset: every run6 histogram is cumulative across levels | 2×5÷1 = 10 | HI |
| F-64 | stale-claim → guarantee-bounded | the served daemon runs with `checkpoint_interval = 0`; §9 does not say which configuration its wall-clock tables came from | 5×5÷3 = 8.3 | HI |
| F-66 | guarantee-bounded / design (LC-31) | a hole's version is read nowhere; holes cost 125 B each and are 98% of a demand view's memory at 100k keys read | 4×4÷2 = 8 | HI |
| F-67 | correctness (calculus) | `idem("k", window: …)` on a `txn` is parsed, half-checked, ignored, and recommended by NL0216's own suggestion | 3×5÷2 = 7.5 | HS |
| F-69 | efficiency (LC-33) | the plan cache keys on literal text; 15–20% of E16's point reads compile, at 25× a served read | 3×5÷2 = 7.5 | HS |
| F-78 | stale-branch | `answer_from_view`'s `answered.anchor != anchor` fallback is unreachable since T-12.3 and is still a fallback | 2×5÷1 = 10 | HI |
| F-79 | GBS mapping (LC-34) | one Loan IQ shape is admissible under the §6.5 rule; the rest are classified below | 3×4÷3 = 4 | HI |
| F-76 | **negative** | the concurrent differential cannot overlap a fold with an `advance` at HEAD, by construction; a 300-pass busy fold passes in 306 s | — | HI |
| F-77 | **negative** | one cached `Lowered` is 2,281 B; the plan cache is not a memory term (≤ 584 KB/session, 9.3 MB at 16) | — | HS |
| F-80 | **negative** | checkpoints cost 32 B per *C* postings per key — 2 B/posting at C = 16, 0.4% of the base's 539 B/posting; not the kind of term T-05 bounded | — | HI |

### F-61 — a damaged length prefix is a "torn tail" (correctness, durability) — EV 12.5, HI

**Evidence.** `crates/nilestream-ledger/src/segment.rs`: the record is `len: u32 | body | crc32`,
and the CRC is over the body only — `crc32(&out[4..])` at the writer (line 102) and
`crc32(&with_len)` at the reader (line 386), whose buffer, despite its name, holds no length.
`recover` (lines 372–376) turns a length that runs past the file into `ShortTail`; `Segment::open`
(lines 180–215) then asks `declared_record_len` — *the damaged prefix itself* — whether further
bytes follow, and when the corrupted length exceeds the file it concludes "tail", `set_len`s to
the last valid record, syncs, and opens cleanly.

**Measured (container, deterministic; probe: a ten-record, 1,105-byte segment written by the
sequencer under `SyncPolicy::Always`, every mutation opened with `Sequencer::open_bounded` and
replayed with `recover_txns_checked`).**

| mutation | cases | refused | exact | **prefix with success** |
|---|--:|--:|--:|--:|
| truncation at every byte offset | 1,105 | 0 | 0 | 1,105 (by design: a torn tail) |
| single-bit flip, every bit | 8,840 | 7,720 | 0 | **1,120** |
| — of which in a non-final record's length prefix (bytes 1–3 of the prefix) | 216 | 0 | 0 | **216** |
| — of which anywhere in the final record | 920 | 16 | 0 | 904 |
| duplicated last record | 1 | 0 | 1 (the duplicate is `OutOfOrder` and trimmed as a tail) | 0 |
| reordered pair (records 4, 5) | 1 | 1 | 0 | 0 |
| 30-byte junk tail | 1 | 0 | 1 | 0 |

Byte 1, bit 2 of the file — record 0's length prefix — yields "0 of 10 txns, window 0 keys,
cause ShortTail { at_offset: 0 }, truncated 1105": **ten acknowledged, fsynced epochs discarded
and a clean open.** The comment at `segment.rs:180` describes exactly this event as fixed; the fix
covered the CRC-detectable case and not the prefix the CRC does not cover.

The 904 final-record cases are a different fact: the format cannot distinguish a damaged last
record from a crash mid-write, because nothing records that the writer *finished*. That is a
design limit, positioned as **LC-36** rather than folded into the repair.

**Cost of repair.** 4 bytes per record and a format bump. No committed segment exists to keep
readable: the `.seg` files under `results/E16-wallclock/` are zero-byte, untracked leftovers.

**Repair (C9-03).** A self-checking header: `len: u32 | len_check: u32` where `len_check =
!len ^ 0xA5A5_A5A5` (or a CRC of the four bytes — the executor's choice; the test is the corpus).
`recover` classifies a header that fails its own check as `BadHeader { at_offset }` — corruption,
never a tail — and only a header *shorter than 8 bytes* remains `ShortTail`. `Segment::open`'s
tail decision then uses a validated length or refuses. **What it gives up:** a real crash that
tears *inside* the 8-byte header with a coincidentally self-consistent pair is a 2⁻³² event; a
segment written by the old format does not open (state the version in the error).

### F-62 — the unchecked window (correctness, "no default-on-error") — EV 20, HI

**Evidence.** `sequencer.rs:114–116`: `recover_seen` is `recover_seen_checked(..).unwrap_or_default()`.
`open_bounded` (line 178) calls it, and `start_bounded` runs a sealer whose window is whatever
came back — on any envelope the decoder refuses, **an empty map**: every identity committed before
the restart is new again. The daemon survives only because `with_durable_bounded`
(`rev_engine.rs:1004–1013`) calls `recover_txns_checked` *afterwards* and returns `Err`; the
sequencer thread started with the empty window is dropped on that path. `Sequencer::open`,
`open_recovered`, the durability bench and every test that opens a segment get the empty window
and no error. The corpus above confirms the pairing: every `PrefixWithSuccess` row's "window N
keys" equals its recovered prefix, i.e. the window silently matches whatever the decoder managed.

**Repair (C9-03).** `open_bounded` uses the checked form and returns `InvalidData`; the unchecked
`recover_seen`/`recover_txns` become `pub(crate)` behind the inspection tool only, or are deleted.
**Gives up:** nothing a ledger may keep.

### F-63 — assertions the scheduler decides (liveness of the gate) — EV 15, HI

**Evidence.** Run 2 of 3 at `--test-threads=8`:
`frontiers::tests::a_reader_never_observes_an_epoch_that_was_not_published` panicked at
`frontiers.rs:199` — "the reader saw nothing at all; the test proved nothing". The reader loops
`for _ in 0..200_000` (line 181) with no bound on the writer's progress; on two cores under eight
test threads the writer thread had not run when the reader finished. The same test passed under
`taskset -c 0 --test-threads=1`, where preemption interleaves them. **The class, swept:**

| test | file:line | the scheduler-decided assertion |
|---|---|---|
| `a_reader_never_observes_an_epoch_that_was_not_published` | `frontiers.rs:160–203` | `observed > 0` after a fixed 200,000 spins — **failed 1/3** |
| `group_commit_amortises_the_fsync` | `sequencer.rs:~653–685` | `max_batch > 1` — true only if two submitters queue before the sealer drains |
| `eight_concurrent_submitters_all_commit_and_at_least_one_batch_is_shared` | `sequencer.rs:1029–1088` | `max_batch > 1`, same |
| `the_reads_that_stopped_falling_back_are_served_and_not_reconstructed` | `rev_engine.rs:4113–4136` | `hits × 4 > keyed × 3` under 4r/2w — depends on how many epochs land between anchor and read |
| `the_fallback_rate_under_four_readers_and_two_writers_is_low` | `rev_engine.rs:4149–4165` | `rate ≤ 0.05`, same dependence |

The last four passed four times here; a machine that starves readers or never queues two
submissions will fail them, and the failure will look like a regression in the code.

**Repair (C9-00).** The frontier test: bound the reader on the writer's progress (`while
f.visible() < 199 || n < MIN_READS`, with a ceiling), as the cycle-8 differential does. The two
`max_batch` tests: hold the sealer behind a `Barrier` until all submitters have queued (the
sequencer already has the pending queue; a test-only `SyncPolicy`-independent gate, or submit
via `submit_pending` and release together), so `max_batch ≥ N` is a *property* and not a race
outcome. The two ratio tests: make the writers' epoch count between anchor and read a controlled
quantity (writers gated per reader round) or assert the ratio against a *measured* write count
(`hits ≥ keyed − epochs_sealed_during_reads × readers`), which is the inequality the code actually
promises. **Guard:** the reverted frontier test fails within three runs under two `yes >/dev/null`
hogs on the executing host — the transcript of that failure is the guard's proof, since the class
is defined by the load.

### F-64 — the served daemon has no checkpoints, and §9 does not say so — EV 8.3, HI

**Evidence.** `rev_engine.rs:381`: `RevEngine::seeded` builds `Ledger::new()`;
`proto-engine/src/ledger.rs:268–271`: `new()` leaves `checkpoint_interval` at 0 (`with_checkpoints`
exists at line 273 and the daemon never calls it). `reconstruct_balance` (lines 513–525) starts
from a checkpoint only when the interval is non-zero; otherwise `start = 0`. Thesis §9.4.1 (Table
9.7, line 270) and §9.13.1 (line 531): "**The cost law holds, but only with checkpointing**".
§9.14.1 (E16) names the budget and the miss rate and never the interval; §9.14.5 (E23) and the E19
document likewise. The within-level decay in cycle 8's data (94,038 → 84,384 reads/s over five
runs as the base grew, LC-23 attribution table) is what a history-length fold looks like.

**Memory (F-80, container).** A checkpoint is `(Epoch, Minor)` = 32 B; per key, one per *C*
postings; plus `running` and `posting_seen` per key. At C = 16 that is 2 B per posting against
539 B per posting for the base (E18 `ledger_seeded`) — 0.4%. Not a Θ(history) term of the kind
T-05 removed; a term proportional to the base at a chosen constant.

**Repair (C9-07).** `nilestreamd --checkpoint-interval N` with default **16** (the constant §9.4.1
measured to C/2 + 1 and called "a number the designer chooses"), printed in the banner, carried
into every generated document's provenance header, and a `MISMATCH-daemon-checkpoints` marker in
§9.14.1/§9.14.5/E19 until the C republish (C9-12). Whether the interval becomes a schema declaration
like the window did is **LC-32**; this cycle it is a flag, because a schema declaration would
put a cost constant in the language's contract before a measurement says it belongs there.
**Gives up:** the committed E16/E19/E23 tables become "measured at C = 0" by label; that is
already true of them.

### F-65 — E18's idempotency rows measure stand-ins (wrong-measurement) — EV 15, HI

**Re-derived from `tools/memprobe/src/scenarios.rs` (§8.3), row by row.**

| row | what it builds | what T-05 shipped | measures what its name says? |
|---|---|---|---|
| `rev_metadata_per_key` (lines 632–719) | a full view advanced through every epoch, then one hitting read per key; asserts `resident_count` unchanged | `Rev.meta: BTreeMap<Key, Meta>` | **yes** — 94.1 B live per resident key is the metadata, isolated by construction |
| `rev_metadata_2x_budget` (lines 466–550) | 5,000 distinct reads under a 2,500 budget; asserts the budget bound | the same | **yes, but mislabelled**: the 243 B/key it reports is metadata *plus* 2,500 holes *plus* 2,500 values; the doc comment (line 60) says the slot map "must stay" because a hole is "the reason a miss is not a zero", which §6.2/F-66 shows is not what makes a miss not a zero |
| `idem_admission_index` (lines 577–600) | `HashSet<String>`, 100k UUID-shaped keys | `Ledger.idem: HashMap<Arc<str>, Epoch>` + `idem_order: VecDeque<Arc<str>>` when a window is declared | **no** — a different container, a different key type, no epoch, no deque; its comment says "Neither is pruned by anything today" |
| `idem_window_sealer` (lines 605–626) | `BTreeMap<String, u64>` | `BTreeMap<Arc<str>, u64>` + `VecDeque<(Arc<str>, u64)>` (`sequencer.rs:320–327`) | **no** — no `Arc`, no order deque |

The two rows the T-05 report quotes as "68.8 B/identity" and "99.6 B/identity" are figures for
containers the code no longer holds. The cost of the real structures appeared only indirectly, as
the +10.9 B/posting in `ledger_seeded` (528.3 → 539.2) and +17 B/txn in `append_in_memory`. **Repair
(C9-09):** build the rows on `proto_engine::Ledger::with_idem_window(n)` and on the sealer's own
window type (expose a `pub fn window_bytes_probe` or construct the exact pair of containers), keep
the old rows under `_stand_in` names for one cycle so the two can be compared, and add the two
rows this audit measured by hand (below). Fix the three doc comments.

### F-66 — the hole's version is dead information (LC-31) — EV 8, HI

**Reading `Rev::read` against chapter 3 and Theorem 4.1 (§8.2).** `rev.rs:245–306`: the only
resident state the read consults is `Slot::Present(v, e)`; a `Hole(e)` and a `Bottom` fall through
to `base.reconstruct(key, anchor)` — anchored at the *requested* anchor, never at `e`. `absence.rs:35`
says a hole "knows from `e` exactly which prefix to fold"; it cannot: without the value at `e`
there is no shorter fold than the one from genesis (or from a checkpoint, which the base finds
without the hole's help). Theorem 4.1's step (3) (thesis `04:33`) is stated for "a never-evicting
execution's entry" and uses the certification invariant maintained by `apply`; honest absence is
invoked only in the interleaving clause ("preserves the anchor across eviction so ρ's target is
well-defined") — but ρ_{k,a}'s target is (k, a), fixed by the read, and would be equally
well-defined over ⊥. **So the theorem is stated for the three-state lattice the runtime has**, and
Pending appears in the proof nowhere: the join semantics of C9-06 need their own clause.

**Measured (container, deterministic, `Runtime::install(.., Some(2_500), Lru)`, one read per key).**

| keys read | allocations | bytes | live at end | resident | live per key |
|--:|--:|--:|--:|--:|--:|
| 5,000 | 19,164 | 1,214,712 | 859,352 | 2,500 | 171.9 |
| 20,000 | 84,156 | 4,978,656 | 2,733,440 | 2,500 | 136.7 |
| 100,000 | 430,785 | 25,056,952 | **12,729,528** | 2,500 | 127.3 |

Slope 5k→20k: 124.9 B per additional key; 20k→100k: **125.0 B per hole.** At 100k keys read the
2,500 resident entries and their metadata are ≈ 0.55 MB of a 12.7 MB view; the rest is holes.
G3 wipes every view to ⊥ before each sweep and conservation holds (29/29, `as_of_reconstructions
= 0`) — the evidence for (b) and (c) the brief anticipated.

**Decision.** (b) and (c) together: honest absence is an *audit* property — "eviction never
reports a value it does not hold, and never lowers a pinned or in-flight entry" — and under full
retention a hole may be compacted to ⊥ with no loss to any answer. **Repair (C9-08):** eviction
removes the slot (`slots.remove`), keeps `evictions` as the audit count, and the thesis's §3 lattice
paragraph and `absence.rs` are rewritten to say what the version is for; `Slot::Hole` stays in the
type for the *audit* path (a `--trace-evictions` log records `(key, e)` if wanted) and is no longer
what a demand view retains. `MISMATCH-hole-version-unused` marks §3 until the rewrite lands.
**Gives up:** a per-key record of the epoch an entry was certified through when it left, resident
for free; recoverable from an eviction log at the same 125 B when someone asks for it.

### F-67 — one concept, two spellings, two rigors (calculus) — EV 7.5, HS

**Evidence.** `parser.rs:2523–2550`: `txn idem(<key>, window: <expr>)` binds both.
`typecheck.rs:1015–1041`: only "window without key" is checked (NL0216) — and its `.suggest(..)`
at line 1034 writes `idem("name", window: 30.days)`, the wall-clock unit NL0217 refuses on the
column. `niles-interp/src/lib.rs:1448–1450`: `if spec.window.is_some() { self.ignored_windows += 1 }`.
`lower.rs`: no use of `idem` (grep). 25 files carry the call form with `30.days`/`24.hours`,
including `tests/solver_corpus/sound.niles` and `calculus_mutants.rs`. LC-28 settled the unit; it
settled it for one spelling.

**Repair (C9-04).** The transaction form takes the key only: `txn idem("k") { .. }`. A `window:` on
it is **NL0218** "the idempotency window is declared on the relation's `idem` column, in epochs";
the suggestion text of NL0216 changes accordingly; the 25 usages are converted (source and
corpus only — never `docs/audit/*`, as cycle 8 learned); a mutant `idem_window_on_txn.niles` is
added; the key must reach the IR as the posted row's `idem` value (or the checker refuses a `txn
idem(..)` that posts to a relation without an `idem` column — **NL0219**). **Gives up:** a per-
transaction window, which the runtime never implemented.

### F-68 — the engine re-establishes the currency table per `INSERT` (checked-twice) — EV 20, HS

**Evidence.** `session.rs:1278–1281`: `Session::insert` calls `declared_currencies(&self.schema)`,
which (`1503–1521`) runs `parse_program` and `resolve_program` over the **entire schema text** to
extract `(code → scale)`. The same schema was parsed, resolved and type-checked when the session
compiled its first statement, and its identity is already a key (`schema_key()`).

**Measured (container, callgrind, inclusive, per function; the probe drives 2,000 E16-shaped
statements through `Session::handle` against `RevEngine::seeded(10_000, 1, 2_500, Demand, Lru)`;
seed-only baseline 197,226,140 instructions subtracted).**

| path | instructions per statement | of which |
|---|--:|---|
| `oltp` insert (two-leg transfer) | **121.6k** (`Session::insert` 113.5k) | `parse_program` 65.5k + `resolve_program` 9.0k = **74.5k (61%)** re-deriving the currency table; `Serving::append` 18.5k; `Ledger::submit` 16.8k; `Runtime::advance` 4.5k |
| `point` read, hot set of 100 keys (5% compile miss) | 29.8k | `compile_cached` 10.5k avg (210k per miss); `Serving::query` 7.1k |
| `point` read, cold (97.6% miss) | 193.5k | `compile_cached` 175k per miss: lexer 72k, parser 33k, lower 19k, typecheck 10.7k, resolve 10.7k, verifier 4.4k |

Wall clock (*this host*, release, `nilesc time`, median of 200): the whole point-statement
compile is 15.7 µs — lex 6.6, parse 2.4, resolve 1.8, typecheck 1.5, lower 2.7, verify 0.6.
`parse_insert` and `minor_units` are inlined into `Session::insert` and are not the term.

**Repair (C9-05).** A `currencies: Option<(SchemaKey, BTreeMap<u32,u32>)>` on the session,
rebuilt when `schema_key()` changes (the same rule the plan cache uses), and the same for
`declared_idem_window` at daemon start (it is called once; fine). Target: `Session::insert` ≤ 45k
instructions on the probe. **Gives up:** nothing — the refusals (22023 when the schema does not
resolve, 22003/0A000 on the amount) are unchanged because they act on the cached table's absence
or on the statement, not on the parse. The wire contract is the same set of SQLSTATEs.

### F-69 — the plan cache keys on literal text (LC-33) — EV 7.5, HS

**Evidence.** `session.rs:484`: key `(schema_key, sql.to_string())`. E16's point statement inlines
the account (`workloads.rs:205`), so every distinct key is a distinct plan. **Simulated with the
bench's own generator** (`Rng::skewed_key(10_000, 0.9)`, seeds `0xB0A7 ^ run`, 2,000 operations,
one session, FIFO of 256): miss rate 14.5–20.0% across runs 1–10 (mean **17.2%**); at 200k
operations steady state 15.3% FIFO, **9.8% LRU** (same cache size). A miss is 175k instructions
(above) against 7.1k for a served read. **Memory (F-77):** one `Lowered` for the point statement is
2,281 B live in 58 allocations (3 nodes, 3 schemas) — 584 KB at the 256 limit, 9.3 MB across 16
sessions; the size is not the problem, the key is. **Position for LC-33:** FIFO→LRU is a one-line
third off the miss rate; the real repair is a literal-templated key (`where acct = ?` with the
literal bound at execution) which is a lowering with parameters and belongs with the extended
protocol's `Bind` — **below the cut line (C9-11)**, because it changes what a cached plan *is*.

### F-70 — the daemon's flags fall back silently (correctness) — EV 15, HI

`main.rs:79–83`: `--port`, `--accounts`, `--rounds`, `--budget` use `parse().unwrap_or(default)`.
`--budget 2,500` (a comma) runs the phase diagram's lever at its default and says nothing; a
mistyped `--port` serves on 5434. The constraint is "no default-on-error / honest refusal over
silent fallback" and the shipped binary's own front door breaks it. **Repair (C9-04):** refuse
with the flag name and the text that did not parse. **Gives up:** nothing.

### F-71 — which window? the first one a `HashMap` yields (nondeterminism) — EV 10, HI

`session.rs:1494–1500`: `cat.relations.values().flat_map(..).find_map(..)`; `resolve.rs:161`:
`relations: HashMap<String, RelationInfo>` (std `RandomState`). Two relations with different
windows → the daemon's window differs between two starts of the same binary. The default schema
has one relation, so nothing has seen it. **Repair (C9-04):** the window of the relation the wire
writes to (`postings`), and a refusal if any other relation declares a different one.

### F-72 — the concurrent writers post one zero-amount leg (wrong-measurement) — EV 15, HI

`bench.rs:2212` and `2252` (E19 `durable` and `mixed` levels) and `docs/audit/cycle-7/probes/
mixed.rs:90` write `insert into postings values ({id}, {acct}, 0, 0)`; E16's `oltp`
(`workloads.rs:270`) writes `({txn},{from},0,-{amount}),({txn},{to},0,{amount})`. The single-leg
statement seals a real epoch and pays a real fsync, but: the envelope carries one row instead of
two; the per-currency zero-sum rule is satisfied by a single zero; every view delta is 0, so no
resident value ever changes under the readers — the mixed levels measured lock interference and
never a value moving under a read. The 4r/2w in-process tests seed real amounts; the wire levels
do not. **Repair (C9-01):** a two-leg non-zero transfer from `scaling_key` pairs, identical on both
arms; `run4.sh`'s crate is an audit artefact and is annotated, not edited. **Gives up:** the E19
rows must be re-measured (they are `not_run` already, LC-27).

### F-73, F-74, F-75, F-78 — prose, comments, instruments, a dead branch — EV 10 each, HI

- **F-73** thesis `09-evaluation.md:735`: "it is 0.30 … reconstructing three reads in ten";
  `results/E16-wallclock/point.csv`: `miss_rate` 0.1414, 0.1444, 0.1528, 0.1593, 0.1429, … The
  generated `E16-wallclock.md:98` correctly says the rate "is rendered from [the CSV], not a range
  typed into this sentence"; the thesis typed one in. Repair: the sentence reads the column or is
  deleted (C9-10).
- **F-74** (a) `preflight.sh:22` "cycle-7" banner (cycle-8 and cycle-9 copies); (b) `preflight.sh:183`
  "FOUR untracked files"; (c) `run6.sh:57` excludes four names, so `niles/.DS_Store` prints as
  "unexpected"; (d) `session.rs:432` "before the cache is emptied" and `:477` "**Unbounded**" — the
  cache is FIFO-bounded at 256 since T-13; (e) `absence.rs:35` "knows from `e` exactly which prefix
  to fold" (F-66); (f) `scenarios.rs:60, 581–583, 607` (F-65). Repair (C9-01 / C9-09).
- **F-75** `lockstats.rs`: `ENGINE_LOCK`, `VIEW_LOCK` are process-global histograms with no reset;
  `run6.sh` prints them after every level, so level *n*'s histogram contains levels 1..n-1.
  `SLOW_READS` got `reset` in cycle 8; these did not. Repair (C9-01): `select nilestream_lockstats
  reset`, called by the harness before each level, and the histogram output labelled "since reset".
- **F-78** `rev_engine.rs:1300–1321`: `if answered.anchor != anchor { view_fallbacks += 1; return
  None }` — since T-12.3 `Rev::read` returns the anchor it was asked (postcondition tested), so the
  branch is unreachable, and it is still a *compensating fallback* — the shape the standing
  constraint forbids callers to reintroduce. Repair (C9-10): the branch becomes a refusal counted
  under `view_fallbacks` with a `nilestream_stats` row asserted 0 by the E16 harness — the G3
  as-of pattern — rather than a silent fold.

### F-76, F-77, F-80 — negative findings

- **F-76.** `rev.rs:1770–1870`: readers, the advancer and the writer all serialise on one
  `Mutex<Runtime>`; `view.read(..)` reconstructs *inside* the guard. Replacing `SharedBase::
  reconstruct` with a busy fold (300 passes over the history) passes in 306 s; 20 passes in
  22.8 s (*this host*). The differential cannot exercise a fold overlapping an `advance` today,
  so it is **not yet a guard** for C9-06; C9-06's guard is the two-phase version below.
- **F-77.** Above (F-69). The plan cache's memory is bounded and small.
- **F-80.** Above (F-64). Checkpoints are 0.4% of the base at C = 16.

### F-79 — Loan IQ and Calypso onto GBS's 29 rows (LC-34)

Classification against `results/G3-verdict.md` at `963e4d9`. **P** = present with G3 evidence
(`ProductSpecific`), **G** = generic path only, **A** = absent; the last column is the invariant an
absent shape would stress, which is the admissibility test.

| vendor capability | GBS row(s) | status | invariant it would stress / why it is or is not admissible |
|---|---|---|---|
| deal → facility → outstanding; bilateral and syndicated lending; drawdowns, schedules | Lending — revolving / term / syndicated | **P** (11-lender syndicate, `lending.rs`) | already evidenced |
| agency servicing for "over a thousand lenders"; pro-rata distribution | Lending — syndicated | **A** at that scale | **one epoch of 1,001 postings**: batch envelope, sealer drain, E23 cost-per-row, and the rounding-remainder obligation (`ParticipantSumMismatch`, `lending.rs:283`). Admissible under (i) *and* (ii) as a product trace → **C9-G01** |
| interest and fee accrual, tiered pricing, discount loans | Lending rows | **G** (accrual is a schedule, not a posting set) | balance-as-a-function-of-time; no invariant beyond conservation; not admissible alone |
| PIK capitalisation | — | **A** | a posting the *schedule* creates: the ledger writing to itself, epoch-anchored and deterministic. Niles has no spelling for a scheduled posting (no `at epoch`/`every` form; the temporal library is read-side). Stresses determinism of self-generated epochs. Admissible only after F-12 (lifecycles survive restart) — below the line |
| adjustments, amendments, reversals | all posting rows | **G** | a reversal is a compensating set (expressible: any row); an amendment is a new valid-time interval (`valid:` exists on `Posting`); **neither has a first-class spelling** — `reverse(txn)` / `amend(txn, valid: ..)` do not exist. Stresses bitemporal audit. Admissible as a *language* task, not a product task |
| secondary trades: assignments, participations, settlement | Trading and operations; Clearing — prime brokerage | **P** / **G** | conservation across counterparties; already evidenced generically |
| letters of credit, guarantees, export finance | Letters of credit; Trade loans; Supply-chain finance | **P** | the LC row exists; a contingent liability is modelled with a **hold with expiry** — the brief's question "is a hold the right primitive?" is answered *provisionally yes* by G3 (55 reads, 6 wipes, passes); an LC needs no third state until an *amendment* of an undrawn LC is required, which is the bitemporal gap above |
| collateral, cross-collateralisation, covenants, document tracking | — | **A** | document tracking stresses no ledger invariant; cross-collateralisation is a *constraint over holds across accounts* — a `conserve per (..)`-shaped rule. Not admissible this cycle |
| online accounting, real-time debits/credits, multi-branch GL mapping, audit trail | every row | **P** by construction (the ledger *is* this) | — |
| unitranche, non-pro-rata, sustainability-linked | Lending rows | **A** | pricing variants; no new invariant |
| **Calypso** multi-asset trade capture over shared curves | Securities ×3, Multi-asset, ETF, Forwards/swaps, OTC | **P** | already evidenced |
| market risk, CCR, limits, P&L, XVA | Risk analytics | **P** as a *read-only* row (posts nothing; a test fails the build if it does) | derived views over market data — the first REV whose base is not the ledger. Stresses the "fully general relational core" claim. Admissible as an *engine* task (a non-ledger `Base` implementation), not a GBS product task |
| margin: CSA, IM/VM, SIMM, optimisation | — | **A** | a variation-margin call = a conditional hold whose amount is a function of a price → the same non-ledger base; settles by posting. Blocked on the engine task above |
| CCP clearing: SPAN2/PRISMA/IRM2 margin, ISA/OSA/NOSA/GOSA account structures, position limits | Clearing and prime brokerage | **G** | house/client segregation = a partition of the key space with cross-partition conservation *forbidden*: `conserve per (book, cur)` — Niles has `conserve per (txn, cur)` (`ast.rs:256`, `currency_rows.rs`); whether a *partition that must not net across itself* can be declared as a second clause is the language question. Admissible as a language task |
| post-trade STP: confirmation, matching, settlement, corporate actions | Trading and operations; Securities | **P** (`matching.rs`) | — |
| treasury: cash positions, funding, liquidity across banking/trading/investment books | Funds sweep; Cash pooling; Zero-balance; Cash management | **P** | **no book dimension** on any account key (keys are `(acct, cur)`): a third key component. Stresses nothing until `conserve per (book)` exists |
| regulatory reporting (EMIR, Dodd-Frank, MiFID II, FRTB, SA-CCR) | — | **A** | reporting is a read model; no invariant |
| unified trade record | the ledger | **P** by construction | — |

**LC-34 position:** the first product task is the thousand-lender distribution as a trace (C9-G01);
every other admissible item is a language or engine task (book dimension + `conserve per (book)`;
`reverse`/`amend` forms; a non-ledger `Base`) and is listed for the author's ordering in §7.
The refusal "no GBS efficiency task until F-24 and F-12 close" **stands**; C9-G01 is the first half
of closing F-24.

---

## 2A. Astra's findings, and what Fable verified in source

Astra read the Mac trees at `5eb0bb2`/`963e4d9` and ran nothing; the author ran the gate on request.
Fable re-read every source claim below in the container tree at `5a64595` (runtime source identical
to `5eb0bb2`; the one commit between them touches five audit files). **Verdict** is Fable's:
*confirmed* (the code does what Astra says), *confirmed-narrowed* (true, with a scope note), or
*not re-read* (carried on Astra's evidence class). Where the two audits found the same thing the
ids are paired; the consolidated task that repairs it is in the last column.

| Astra | Fable | what it is | Fable's verdict | task |
|---|---|---|---|---|
| A9-F01 | — | **a connection can drain another request's durability receipts**: `append` pushes a `Pending` token into an engine-wide `Mutex<Vec<Pending>>` (`rev_engine.rs:188–189`); the daemon calls `take_pending` *after* `session.handle` returns (`daemon.rs:143–144`), unpaired with the append. Thread B's `take_pending` between A's append and A's drain takes A's token; A acknowledges with nothing to wait on. | **confirmed.** The reply is written before the fsync of its own record: `INSERT 1` on the wire, the row applied, the barrier still in flight on another connection. The visible frontier is still gated by the token, so no *read* sees it — but the client was told it committed, and a crash before the barrier loses an acknowledged transaction. A failed barrier is reported to the wrong connection (58030 on B's read; A's insert acked). This is the durability claim being false in the served daemon, and it outranks every efficiency item. | **C9-02** |
| A9-F02 | — | **two windows, two coordinates named "epoch"**: `prune_idem` (`ledger.rs:374–381`) bounds the admission index to the last *W transactions* (one transaction = one proto epoch); the sealer (`sequencer.rs:433–452`) drops an identity `W` *segment records* behind the record just written, and one record is a batch of up to 4,096 transactions. | **confirmed.** For W declared, admission forgets an identity after W transactions; the sealer remembers it for W batches (≥ W, ≤ 4,096·W transactions). A retry in the gap is *new* to admission (applied to the base, an epoch sealed) and *Duplicate* to the sink; the session maps `Duplicate` to an error (`rev_engine.rs:308–323`): an applied, un-acknowledged, un-durable epoch. Whether a later `fetch_max` publishes past it is the witness C9-02 must build; the coordinate mismatch is a fact today. | **C9-02** |
| A9-F03 | F-61 | damaged length prefix → destructive tail recovery | **confirmed and measured** (§2, F-61: 216 + 904 corpus cases). | **C9-03** |
| A9-F04 | F-62 | unchecked recovery wrappers still on the startup path; lossy UTF-8; no trailing-byte check in the envelope | **confirmed** (F-62) and extended: `decode_envelope` uses `from_utf8_lossy` and does not require `o == payload.len()` at the end, so an envelope with trailing bytes decodes cleanly. | **C9-03** |
| A9-F05 | (F-71 is the same class) | **the IR's currency code is HashMap iteration order**: `lower.rs:834` `self.cat.currencies.keys().position(..)` over `resolve.rs:160` `HashMap<String, CurrencyInfo>` (std `RandomState`); the wire's `declared_currencies` sorts by declaration span (`session.rs:1512–1513`). | **confirmed.** With two or more declared currencies, `Scalar::LitMoney { currency: ix }` in a compiled circuit names a currency by a per-process random order while the wire names it by declaration index — the standing constraint "a currency's wire code is its declaration index" is broken inside the compiler. Fable's F-71 (`declared_idem_window` over the same map) is the same defect class. One currency in the default schema is why nothing has seen it. | **C9-04** |
| A9-F06 | — | **gate scratch files collide**: `nilesc/tests/run.rs:26–34` names a temp file by `(name, SystemTime nanos, pid)`; `File::create` truncates an existing file; `Drop` removes it; `args_file` uses the fixed name `args` for every test. Two tests in one process within one clock tick share a file. The author's third `--test-threads=8` run on the Mac failed `an_account_is_named_by_the_rendering_of_what_was_passed` with another test's arguments in its file, then "No such file or directory". | **confirmed** by reading; not reproduced here (Linux's clock resolution is finer). This is F-63's class with a different mechanism — a name the clock decides. | **C9-00** |
| A9-F07 | — | **the "unaccounted" residual is an identity, and the lock histogram's quantile is a lower edge**: `answer_from_view` takes four consecutive timestamps, so `total − (base_wait + view_wait + view_hold)` is rounding (`rev_engine.rs:1288–1300`, `session.rs:925–945`); `lockstats.rs:73–108` buckets by `floor(log2 µs)+1` and `quantile` returns `1 << (i−1)` — the bucket's *lower* edge — under the name "upper bound". | **confirmed, and it corrects Fable's own cycle-8 attribution.** "0–2 µs on everything else" in `lc-23-attribution.md` and in thesis §3:41 measured nothing: the wire, framing and scheduling sit *outside* `read_began..view_done` and were never in the sum. The V-wait figures stand (they are measured differences); the exclusion of everything else does not. Every `p99` the histogram has printed is a lower bound up to 2× low. | **C9-01** |
| A9-F08 | (F-72, F-75 overlap) | run6 section D: shapes labelled `4r2w/8r1w/8r4w` are 6r3w/9r5w/12r6w (the bench derives writers from connections); the two arms reuse one segment path per port; output directories reused; grep-as-verdict; history grows across levels. | **confirmed** — the label defect is in `run6.sh:165–180`; Fable's F-72 (zero-amount writers) and F-75 (cumulative histograms) are the other two instrument defects in the same section. | **C9-01** |
| A9-F09 | F-65 | E18 idempotency rows model obsolete containers | **confirmed and re-derived** (§2, F-65). Astra's occupancy note stands: W batch epochs can hold up to 4,096·W identities, so "≈ 100 MB at 1,000,000 epochs" was an extrapolation of the stand-in, not a size. | **C9-09** |
| A9-F10 | (LC-23) | Pending unreachable; the fold owns V | **confirmed** (§6.1 design in C9-06). | **C9-06** |
| A9-F11 | F-66 | holes retain history; victim selection scans it | **confirmed and measured** (125 B/hole). Astra adds a scope note Fable accepts: **Full mode** initialises absent entries from deltas, so compaction must be demand/evictable views only, or completeness must be tracked. | **C9-08** |
| A9-F12 | F-64 | served checkpoints off; **the physical scan index is per account while checkpoints are per (account, currency)** (`ledger.rs:108`, `by_account: HashMap<Acct, Vec<RowRef>>`; `checkpoints: HashMap<(Acct, Cur), ..>`) | **confirmed.** A key `(a, usd)` reconstructs by scanning every row of account `a` in every currency from its last `(a, usd)` checkpoint — so the C/2 + 1 bound is a bound on *target-key* rows, not on rows visited; a heavy-`eur`, quiet-`usd` account is the counterexample. The thesis's "no cost term depends on base length" needs both qualifications. | **C9-07** |
| A9-F13 | F-68 (narrower) | compiler facts do not authorise deleting data validation; cache the schema instead | **agreed.** Fable's measurement (§2, F-68) is exactly the schema re-parse; the SQLSTATE refusals stay. | **C9-05** |
| A9-F14 | F-67 | idem presence checked, meaning not enforced | **confirmed** (F-67), with Astra's correction to the brief accepted: the call form is not *wholly* unvalidated — NL0216 checks presence; unit and lowering are the gaps. | **C9-04** |
| A9-F15 | — | **`Runtime::install` certifies the output node only**: `rev.rs:557–598` checks that each output is a keyed `Sum`/`Count` aggregate and never walks the input graph; thesis 4:35 says "the runtime refuses to install a circuit outside Q_lin". | **confirmed.** A `sum` over a join installs and `apply_epoch` then applies the base's per-key deltas to it — a well-formed wrong answer. The *daemon* is guarded by `FoldPlan::filters_only()` in `answer_from_view`; the core API, which G3 and every in-process test use, is not. | **C9-08** |
| A9-F16 | — | G3's `wire` and `crash` columns come from separate fixtures (`over_the_wire.rs` has no durable sink; "crash" is a same-process reopen of a completed segment), assigned by row name (`WIRE_ROWS`). | **not re-read in full**; the `WIRE_ROWS` name-assignment and the same-process reopen are visible in `g3.rs:148–151, 339–420` and Fable accepts the classification: the columns are honest about what ran but the row reads as one joint witness. | **C9-G01** |
| A9-F17 | (F-12 carried) | lifecycle state mutates product objects before the caller commits and does not replay | **confirmed** at `gbs-mechanisms/src/lifecycle.rs:215–318` (a `Vec` of events on the product; no durable form). This *is* F-12. | **C9-G01** |
| A9-F18 | — | **`rows_touched` is a process-global counter and per-read cost is its difference** (`ledger.rs:138, 694–700`): under B-shared, concurrent reconstructions contaminate each other's counts. | **confirmed.** Every `rows_touched`-derived figure taken under concurrency (E19 mixed, the 4r/2w tests' `rows_touched`) is a sum over overlapping folds, not a per-read cost. Single-threaded rows (E1–E12, E18) are unaffected. | **C9-01** |
| A9-F19 | F-69, F-77 | FIFO capacity known, usefulness and memory not | **superseded by measurement**: hit rate simulated with the bench's own generator (17.2% miss), `Lowered` = 2,281 B (§2, F-69/F-77). Astra's request for a *traced* hit rate on a real session stands as a C9-09 row. | **C9-09 / C9-11** |

**Where the two audits disagree, and the consolidated position.**

- *Checkpoint default.* Fable: 16 (the value §9.4.1 measured to C/2 + 1 flat, 2 B/posting). Astra: 64
  (less memory, no optimality claim). **Consolidated:** ship the flag with default **16**, the
  ablation `0`, and let `c9-checkpoints.sh` measure 0/16/64/256 on C before LC-32 is decided; the
  default is a flag today and is the author's to move. Both audits agree it is not a schema
  declaration this cycle.
- *Pending's performance target.* Fable: ≥ 2.5× at 12r6w and worst V-wait < 2 ms. Astra: ≥ 1.25×
  median with the ≥ 10% ∧ ≥ 3 pooled-MAD gate, preregistered, on a *corrected* baseline. **Consolidated:**
  the pass line is Astra's (1.25×, significance-gated, corrected harness, same session); Fable's
  2.5× and the 2 ms V-wait are the *expected* magnitudes and are reported, not required. A curve
  that clears 1.25× and not 2.5× is `done` with the number beside it.
- *The cut line.* Fable: ten tasks. Astra: five (through Pending). **Consolidated:** the cut sits
  after Pending (C9-06), as Astra placed it — durability and determinism before speed — with two
  cheap correctness tasks Fable adds above it (C9-04, C9-05: one to two days each, EV 20 and 10–20)
  because they are refusals and a cache, not new protocols. C9-07…C9-10 follow in order below the
  line and are taken only when C9-00…C9-06 are green.
- *Base guard across `advance`.* Both: leave it alone in C9-06; measure after.
- *`txn idem(..)`.* Both: one typed epoch-window meaning; Fable's specific repair (key-only call form,
  NL0218/NL0219) is adopted; Astra's `MISMATCH-idem-fingerprint` (the chapter promises a canonical
  fingerprint the key-only index does not keep) is carried as a marker.
- *Recovery of a complete lost suffix.* Both: a segment cut at a record boundary is indistinguishable
  from a shorter log without a retained tip. Fable's LC-36 and Astra's `BLOCKED-recovery-tip` are the
  same item; carried as **LC-36** with the marker.

**Astra's chapter 3/4 sentence ledger (its §2.5)** is adopted whole as the input to C9-10 and is not
reproduced here; its verdict notation (T / F / C/S) and its instruction — update each sentence with
the guard that justifies it, add `MISMATCH-A9-<finding>` where a discrepancy remains — are binding.
Fable's own reading (§2, F-66) agrees with its line 3:47–49 and 4:27–33 verdicts and adds nothing they
lack. The MISMATCH inventory Astra found incomplete (ch. 4 `MISMATCH-F-07`, `MISMATCH-T-11-overdraw`;
ch. 9 F-08/F-09) is C9-10's first deliverable.

### 2A.1 The Mac gate, run by the author for Astra (evidence class: measured by the author on request)

Pinned sibling worktrees under `~/Documents/niles-hostc/c9-gate/{niles,GBS}` at `5eb0bb2`/`963e4d9`:
fmt (1.95.0) exit 0 both; clippy (1.97.1) exit 0 both; GBS `test` 604 passed / 1 ignored, three
times; niles `test --no-fail-fast -- --test-threads=8`: 993/1/7, 993/1/7, **992/2/7**. The constant
red is `our_numeric_bytes_are_postgresqls_numeric_bytes` — no PostgreSQL on the Mac's 5432 (PATH has
PostgreSQL 18.6; PG16 not established) — **RED, blocked prerequisite**, not a defect. The changing
red is A9-F06. `make reproduce` and `make fsync-proof` were not run on the Mac (`strace` absent;
`fsync-proof` is a Linux instrument — the Mac's witness is `F_FULLFSYNC`, which `c9-storage.sh`
covers). The five protected files measured 10,244 / 16,639 / 6,148 / 8,196 / 816,110 bytes.

One count does not reconcile and is left as a fact for the executor: the Mac's GBS run reports **604**
passed and the container's reports **504** (559 `#[test]` attributes in the tree; no `cfg(target_os)`
gating found). Whichever host is counting a hundred tests the other does not, the executor lists them
by name in the report (C9-00's gate matrix) before the number is quoted anywhere.

The Mac preflight also showed the instrument defects F-74 (a/b) and two more: section A prints `?`
for cores and memory on Darwin (no `nproc`/`/proc/meminfo`), and section C reported the pinned
toolchain as "resolves" while every tool read `ABSENT` — the probe calls the tools through
`rustup run` with a `timeout` the Mac does not have. Both are fixed in C9-01 alongside the rest.

---

## 3. Consolidated tasks, in dependency order

### 3.0 Preconditions and guardrails every task inherits

Baselines `5eb0bb2` (niles source; `5a64595` is its documentation-only descendant and the
integration parent) and `963e4d9` (gbs) are immutable. Work in isolated paired sibling checkouts
(`/home/claude/work/c9-wt-<task>` and, for GBS, `/home/claude/work/gbs-wt-<task>` beside a niles
worktree, because GBS's manifest reaches `../../../niles`); never in the author's trees; never touch
the five protected files. `RUSTUP_TOOLCHAIN=stable` where the pin does not resolve, and
`RUSTUP_AUTO_INSTALL=0` everywhere, so rustup never downloads. Offline; nothing installed; zero new
dependencies; no `unsafe`; no default-on-error; a red test is a result and is never deleted,
ignored or loosened. Python edits to Rust are line-based.

Preserve: durable-before-visible; apply-before-publish; one sealer per ledger; one total *logical*
epoch order; full retention; honest absence as C9-08 restates it; fold-never-field; product purity;
self-describing amounts; GBS layering; `Rev::read` answers at the anchor asked and no caller
reintroduces a compensating branch; a currency's wire code is its declaration index; the window is
counted in epochs; lock order **O < B < P < V < C, S a leaf**; no waiting on a flight, receipt,
channel or I/O while holding a lock its producer needs.

Every optimisation ships its guard in the same commit; the executor proves in a disposable worktree
that the guard fails with the operative change reverted and the guard kept (a compile error is not
the witness), then green restored — transcript with worktree name, commands, exit codes. Correctness
repairs need a failing baseline witness the same way. The full gate per task:

```
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline --workspace --no-fail-fast
cargo test --offline --workspace --no-fail-fast -- --test-threads=8     (×3)
make reproduce && git --no-optional-locks status --porcelain
make fsync-proof
```

README's test count moves with every test. `docs/audit/*` is evidence: never edited by a sweep.
Benchmarks write outside worktrees; committed CSVs/docs change only with `--publish`, only on C.
`BLOCKED-<id>` on any missing tool, contract or precondition; continue with independent tasks.

**Cut line:** after C9-06. C9-00 … C9-06 are the cycle; C9-07 … C9-10 follow in order only when all
of those are green; C9-11, C9-G01 and C9-12 are specified backlog. If a safety task is unresolved,
C9-06 is `not done: prerequisite unresolved` — it is not replaced by below-line work.

### C9-00 — a deterministic gate (A9-F06, F-63) — `c9/01-gate-fixtures`

**Closes:** the changing verdicts on both hosts. **Files:** `crates/nilesc/tests/run.rs:22–45,
113–115`; `frontiers.rs:160–203`; `sequencer.rs` (`group_commit_amortises_the_fsync`,
`eight_concurrent_submitters_…`); `rev_engine.rs:4005–4165`; any other match of the §2.6 sweep
(timer-named scratch, sleep-as-readiness, fixed-count observation, global counters shared across
tests, ignored join errors) that is *evidenced*, not merely matched.
**Baseline:** Mac: 1 changed verdict in 3 (`run.rs:430`); container: 1 in 3 (`frontiers.rs:199`).
**Method.** `Temp::new`: a process-local atomic counter in the name and `File::options().
create_new(true)`, retrying a bounded number of candidates; remove only a file this `Temp` owns.
Frontier test: bound the reader on the writer's progress (`visible() ≥ 199 && n ≥ MIN_READS`, with
a ceiling whose message names the unmet precondition). `max_batch` tests: N submitters behind a
`Barrier`, the sink's first barrier held by a test gate until all N are queued → `max_batch ≥ N` is
a property. Ratio tests: measure epochs sealed during the read phase and assert `hits ≥ keyed −
readers × epochs_sealed`; keep the old ratio as a printed measurement.
**Targets.**
- C9-00.1 *The forced temporary-name collision guard passes without overwriting or deleting another
  live fixture, and three complete `--no-fail-fast -- --test-threads=8` workspace runs on the
  executing host show no changed verdict.*
- C9-00.2 *Every test that spawns a thread and asserts on a count states its precondition as its own
  assertion message, distinct from the property, and the same four runs (three at 8 threads, one
  pinned to one core at 1 thread) pass under two `yes >/dev/null &` hogs.*
**Guard/reversion.** `Temp`: inject equal salt for two fixtures with different contents, drop one,
read the other — baseline construction goes red by content or by lifetime. Frontier: the reverted
test under the hogs at 8 threads, run until it fails (transcript). `max_batch`: reverted gate, 20 runs
under hogs; report the count of `max_batch = 1` outcomes even if it is zero.
**Host C:** *the author must now run `bash ~/Documents/niles-hostc/c9-gates.sh` and paste the
output* (§6, C-GATES) — this is the landing that must be green on both hosts before anything else.

### C9-01 — instruments before code (A9-F07, A9-F08, A9-F18, F-72, F-74, F-75, and the disclosures) — `c9/02-instruments`

**Closes:** the wrong-measurement findings; puts a `MISMATCH` beside every below-line discrepancy
now, so no chapter waits on a repair to stop being wrong.
**Files:** `rev_engine.rs:1288–1300` and `session.rs:925–945` (`unaccounted_us` → measured
external boundaries or removed); `lockstats.rs:73–108` (bucket bounds, open-ended top bucket,
`reset`); `ledger.rs:694–700` (`reconstruct` returns a *local* visited count, adds to the global
total once); `bench.rs:2212, 2252` (two-leg non-zero writers); `bench.rs` scaling labels
(readers/writers named from the harness, not from connection arithmetic); `docs/audit/cycle-9/
preflight.sh` (banner, "five", Darwin cores/memory, `rt()` without `timeout`); `scenarios.rs` doc
comments; `session.rs:432, 477`; `absence.rs:35`; thesis §3:41 (withdraw "0–2 µs on everything
else"), §9.14.1:735 (F-73), and `MISMATCH-A9-F01`, `-F02`, `-F05`, `-F12`, `-F15`, `-idem-fingerprint`,
`MISMATCH-daemon-checkpoints`, `MISMATCH-hole-version-unused` placed at their sentences.
`run6.sh` is **not edited** (evidence); `c9-pending.sh` (§6) replaces it as the D entry point.
**Baseline:** telescoping residual reported as a measurement; p99s as lower edges; global
`rows_touched` deltas under concurrency; single-leg zero writes; four/five-file confusions.
**Method.** Add the two boundaries that are *not* inside the four timestamps — request decoded → 
`handle` entered, and `handle` returned → reply written (receipt wait included once C9-02 lands) —
and report each as its own column; delete `unaccounted_us` or define it as `wire + queue`, never as
the sum's remainder. Histogram: `quantile` returns the bucket's upper edge; the last bucket prints
as `≥ 2^k` and is never reported as a finite bound; `select nilestream_lockstats reset`, called
before each level. `Ledger::reconstruct` counts locally. Writers: `({txn},{a},0,-{amt}),({txn},{b},0,
{amt})` from `scaling_key` pairs on both arms. Labels: the harness prints the reader and writer
counts it actually spawned. Preflight: `nproc` → `sysctl -n hw.logicalcpu` fallback; `MemTotal` →
`sysctl -n hw.memsize`; `rt()` uses a Python deadline, not `timeout`.
**Targets.**
- C9-01.1 *Every mixed row names the readers and writers actually spawned, its start and end epoch,
  and phase-local (since-reset) lock counters; no printed residual or histogram bound claims time
  outside its measured boundaries.*
- C9-01.2 *Two overlapping reconstructions each report their own visited-row count and the global
  total is their sum, asserted by a test.*
- C9-01.3 *Every E19 write is a two-leg transfer with a non-zero amount on both arms, asserted on
  the statement generator.*
- C9-01.4 *Every discrepancy this work order names below the cut line has a `MISMATCH-<id>` at its
  sentence, and the §9.14.1 miss-rate sentence reads the CSV's column.*
- C9-01.5 *`preflight.sh` prints "cycle-9", "five untracked files", real core and memory figures on
  Darwin, and finds an installed toolchain without `timeout`.*
**Guard/reversion.** Fake-clock test: a 3 µs wait must not report an upper bound of 2; the top
bucket must not report a finite bound; revert each and its test goes red. `rows_touched`: two
threads folding concurrently; revert to the global difference and the per-fold assertion fails.
Generator test for the writers.
**Host C:** *the author must now run `bash ~/Documents/niles-hostc/c9-pending.sh --baseline-only`
and paste the output* — the corrected D baseline C9-06 is measured against.

### C9-02 — receipts owned by their request; one logical epoch coordinate (A9-F01, A9-F02) — `c9/03-owned-receipts`

**Closes:** the two ways the served daemon can acknowledge what is not durable.
**Files:** `rev_engine.rs:188–189, 753–847` (`append`, `take_pending`), `daemon.rs:140–175`,
`session.rs` (the response envelope carries its receipts; simple and extended protocol; multi-
statement), `ledger.rs:374–381` (`prune_idem`), `sequencer.rs:433–452` (window pruning),
`docs/SPEC-ENGINE.md` (Part 0: the two coordinates named and one chosen), thesis §3:53–61.
**Baseline (read from source, both audits; no reproduced loss):** the receipt-theft schedule and
the window-gap schedule of Astra §2.3.
**Method.** (1) `Serving::append` returns its `Pending` *to the caller*; the session attaches it to
the reply it frames for that statement; the daemon waits on the reply's own receipts before writing
it. `take_pending` is deleted (no global drain). A failed barrier is reported to the statement that
caused it and to no other; a disconnected owner still completes or fails its own barrier (the sealer
does not know who is listening) and *visibility* still requires the contiguous durable prefix — an
owner vanishing is not permission to publish out of order. (2) One coordinate: the window is counted
in **logical transactions** (= proto epochs = what the language's `N.epochs` has always meant at
the column) on *both* indexes; the sealer's `order` deque carries the logical epoch of each identity
(it has it: the base assigns it before `submit`), and prunes by that; the record/batch sequence is
renamed `batch_seq` in types and names so it can never again be read as an epoch. Decide the
inclusion inequality once (`identity is remembered iff head − its_epoch < W`), and test W−1 / W /
W+1 under forced batch sizes 1 and > 1. (3) Validate admission *before* irreversible application
where the order allows; where it does not, an application the sink refuses stops further acceptance
(fail-stop, LC-21) rather than being reported as an error and left applied.
**Targets.**
- C9-02.1 *Each successful reply is written only after every receipt of the statements it answers
  has returned, and a receipt can be observed by no other connection — proved by the delayed-barrier
  two-connection test on the daemon's real bytes.*
- C9-02.2 *A failed barrier is reported as 58030 to the statement that caused it and to no other
  connection, tested with a failing injected barrier and a concurrent reader.*
- C9-02.3 *Admission, sealer and replay agree on one declared window in logical epochs at W−1, W and
  W+1 under batch sizes 1 and > 1, and the segment record's sequence number is named `batch_seq`
  everywhere it appears.*
- C9-02.4 *No path applies a transaction the sink then refuses and continues accepting; the
  witness for Astra §2.3's window gap either reproduces the applied-undurable state and shows the
  daemon stops, or is recorded as "not reproduced" with the invariant that prevents it named.*
**Guard/reversion.** Latch-driven tests (no sleeps): A appends behind a held barrier, B reads and
would have drained; assert B's reply carries no wait and A's reply arrives after release. Revert the
pairing → B's read blocks on A's barrier or A's `INSERT 1` precedes the release: red. Revert the
coordinate → the W boundary tests disagree between indexes: red.
**Gives up:** the global convenience API; unversioned recovery metadata if the sealer's order
entries change shape (`BLOCKED-envelope-version` if an existing segment cannot be read — never a
silent rewrite).
**Host C:** *the author must now run `bash ~/Documents/niles-hostc/c9-durability.sh` and paste
the output* (needs `c9-storage.sh` first — §6).

### C9-03 — recovery that cannot mistake damage for a torn tail (F-61, F-62, A9-F03, A9-F04) — `c9/04-strict-recovery`

**Files, baseline, method, targets, guard:** as Fable's design in §2 (F-61/F-62) plus Astra's
additions: validate the whole segment *and every envelope* read-only before a writable handle is
taken or a frontier published; refuse invalid UTF-8 identities, trailing envelope bytes, count/length
arithmetic that does not consume the payload exactly, duplicate and out-of-order records with
further bytes; the corpus includes a *second family* with CRCs recomputed so the storage layer
passes and only the envelope is malformed. The corpus is a committed test
(`crates/nilestream-ledger/tests/torn_corpus.rs`), exhaustive over offsets and bits of a ≤ 2 KiB
fixture with ≥ 2 records and ≥ 2 transactions in one record, printing its tally.
**Targets.**
- C9-03.1 *Every single-bit flip in a non-final record's header, body or CRC is refused with the
  input file's bytes unchanged, and the committed corpus reports zero prefix-with-success cases
  outside the final record.*
- C9-03.2 *`Sequencer::open_bounded` returns `Err(InvalidData)` on an envelope it cannot decode
  exactly (short, trailing bytes, invalid UTF-8), and no public constructor can start a sequencer
  with a window built by `unwrap_or_default`.*
- C9-03.3 *`docs/SPEC-ENGINE.md` states the self-checking record header, the three-way
  classification (clean end / torn tail / damage) and the version byte, and the thesis sentence on
  the record layout carries no `MISMATCH`.*
- C9-03.4 *A segment cut exactly at a record boundary is recorded as `LC-36` / `BLOCKED-recovery-tip`:
  indistinguishable from a shorter log without a retained tip, which is the author's contract to
  give — never claimed solved by the parser.*
**Guard/reversion.** Revert the header classification → ≥ 216 prefix-with-success: red. Revert the
checked window → "window 0 keys, Ok" cases: red. Control: a *valid* shorter fixture still opens
(so "refuse everything" fails). `crash_recovery.rs` unchanged and green.
**Host C:** *the author must now run `bash ~/Documents/niles-hostc/c9-durability.sh` and paste
the output* with the C9-03 candidate SHA.

### C9-04 — deterministic catalog facts and loud defaults (A9-F05, F-67, F-70, F-71, LC-16, LC-35) — `c9/05-contracts`

**Files:** `resolve.rs:160` (currencies keep a declaration index: `Vec<CurrencyInfo>` in span
order + name map, or an `index` field assigned once); `lower.rs:834` (use it); `session.rs:1494–
1521` (window and currencies from the catalog's index, not from map iteration); `main.rs:79–85`
(flag parsing refuses; `--durable`/`--volatile`; `--idem-window`); `parser.rs:2523–2550`,
`typecheck.rs:1015–1041`, `niles-interp` (`ignored_windows` goes), `lower.rs` (the txn key reaches
the posting), 25 usages, `tests/mutants/idem_window_on_txn.niles`, appendix B, `docs/SPEC-LANG*`.
**Baseline:** codes by `RandomState`; four flags silently default; call-form window ignored;
volatile by default; undeclared window keeps everything.
**Method.** As §2 (F-67, F-70, F-71) and §2A (A9-F05). **LC-16 decided:** refuse to start without
`--durable <segment>` unless `--volatile` is explicit; the banner's first line says which. **LC-35
decided:** a wire relation without an `idem` column is served only with an explicit `--idem-window
N` or `--idem-window unbounded` (printed as such); otherwise refused. `MISMATCH-idem-fingerprint`
stays a marker: fingerprint conflict detection is a separate author decision.
**Targets.**
- C9-04.1 *Every currency code in the IR and on the wire is the currency's declaration index, tested
  by lowering a three-currency schema under permuted `HashMap` insertion and asserting identical
  `LitMoney` codes and wire codes each time.*
- C9-04.2 *`nilestreamd --budget 2,500` (and each other flag with a non-integer) exits non-zero
  naming the flag and the text.*
- C9-04.3 *Two relations declaring different windows are refused at startup naming both; one
  relation gives its window deterministically across ten starts.*
- C9-04.4 *`txn idem("k", window: 30.days)` is NL0218 in every corpus that used it (25 files, none
  under `docs/audit/`); a `txn idem(..)` posting to a relation without an `idem` column is NL0219;
  the mutant is refused; `ignored_windows` no longer exists.*
- C9-04.5 *`nilestreamd` with neither `--durable` nor `--volatile` exits non-zero; with `--volatile`
  the banner says VOLATILE; a schema without an `idem` column on the wire relation is refused unless
  `--idem-window` is given.*
**Guard/reversion.** Each refusal has a test asserting the exit or diagnostic; each reversion
yields a silent success and fails it. C9-04.1's reversion is the `keys().position` line.
**Host C:** *the author must now run `bash ~/Documents/niles-hostc/c9-gates.sh` and paste the
output.*

### C9-05 — the currency table once per schema epoch (F-68, A9-F13) — `c9/06-currency-table`

As Fable's design in §2 (F-68): a `(schema_key, table)` cache on the session, a `schema_parses`
counter in `nilestream_stats`, the `checked-twice` probe binary and `make checked-twice` committed
as a counting tool. Refusals unchanged (22023 / 22003 / 0A000 / duplicate).
**Targets.**
- C9-05.1 *`schema_parses` after 2,000 inserts in one session at one schema epoch reads 1; after a
  schema change it reads 2; tested.*
- C9-05.2 *`Session::insert` on the `checked-twice oltp` probe costs at most 45,000 instructions per
  insert under callgrind on the executing host (was 113,500), stated with the command.*
- C9-05.3 *The four SQLSTATE refusals on the insert path each still have a passing test.*
**Guard/reversion.** Revert the cache → C9-05.1 reads 2,000: red.

### C9-06 — `Slot::Pending`: anchored flights, exact anchors, the first efficiency result (LC-23, LC-30, A9-F10) — `c9/07-pending`

**The design** is Fable's §6.1 specification (the two-phase API, join on equal `(key, anchor)`,
independent fold on a different anchor without install, `advance` skips Pending and why that is
safe, install-with-pinning when `applied` moved, the deferred-delta merge as a *second* commit,
cancellation via `Drop`, eviction never touches Pending, the base guard left alone) **with Astra's
additions adopted:** the flight carries an immutable `(key, anchor, generation)`; `install_fold`
installs only if the generation still matches and the result is eligible (a newer eligible
resident or flight is never overwritten by an older completion — the older completion still
answers its own callers); a same-anchor waiter holds none of O/B/P/V/C while waiting (B is
re-acquired after the wake, in order); active flights and waiters are **bounded** with an explicit
overload refusal (a SQLSTATE, never a silent fold under V) and a zero/small-budget test; a stale
owner cannot clear a successor generation; the lock-order source guard learns the condvar wait.
Checkpoints stay at 0 on both arms of the measurement so Pending is isolated (C9-07 is separate).

**Baseline:** the cycle-8 curve (94,038 / 48,310 / 20,507; V-wait max 22,160 µs) is *inferred from a
document* — the pass line is measured against the **corrected** baseline from `c9-pending.sh
--baseline-only` (C9-01), same session, interleaved arms.
**Targets.**
- C9-06.1 *Every concurrent read equals an independent fold at its own requested anchor while a
  delayed reconstruction overlaps an `advance`, and same-key same-anchor readers share exactly one
  fold without holding V while folding or waiting — proved by the latch-driven two-phase
  differential whose precondition counter (`pinned_installs + deferred_merges + pending_joins > 0`)
  is asserted.*
- C9-06.2 *At the corrected 12r6w Host C point, Pending delivers at least 1.25× the baseline median
  read throughput under the ≥ 10% ∧ ≥ 3 pooled-MAD gate, with no contract or durability regression
  and the slowest-16 table's `view_wait_us` maximum reported beside it (expected: ≥ 2.5× and < 2 ms;
  reported, not required).*
- C9-06.3 *`nilestream_stats` reports `pending_joins`, `uninstalled_folds`, `pinned_installs`,
  `flights_refused`; the 12r6w run shows `pending_joins > 0`; `MISMATCH-pending-unreachable` is
  removed only then.*
- C9-06.4 *Theorem 4.1's proof carries the Pending clause; §3's lattice paragraph and its read-path
  rules (3:160, 3:162) state the exact-anchor interval `stamp ≤ a ≤ effective`, the join on equal
  anchors and the generation-owned install as implemented.*
**Guard/reversion.** The two-phase differential with the 20-pass busy fold (F-76) and latches
(pause the owner's fold, complete an `advance`, let same- and different-anchor readers reach their
paths, release). Reversion 1: install unpinned when `anchor < applied` → divergence. Reversion 2:
`advance` applies to a Pending slot → double count. Reversion 3: install ignoring generation → an
older completion overwrites a newer resident → divergence. Each red, each transcript. No timing
assertion in a unit test.
**Host C:** *the author must now run `bash ~/Documents/niles-hostc/c9-pending.sh` and paste the
output* (interleaved baseline/candidate).

### ——— cut line ———

### C9-07 — checkpoints in the served daemon (F-64, A9-F12, LC-32) — `c9/08-checkpoints`

Fable's C9-07 design (§2, F-64: flag, default 16, ablation 0, banner, provenance header,
`results_headers.rs`, the 1,024-posting test at ≤ 17 rows) plus Astra's two corrections: the
*physical* scan index is per account while checkpoints are per (account, currency), so either the
index is keyed `(acct, cur)` or the stated bound is narrowed to target-key rows; and checkpoint
memory is O(K + N/C) (32 B per tuple, plus `running` and `posting_seen` per key) and is *measured*
in an E18 row, not assumed.
**Targets.**
- C9-07.1 *`nilestreamd` without flags serves with `checkpoint_interval = 16`, prints it, carries it
  in every provenance header, and a head read of a 1,024-posting key touches at most 17 base rows.*
- C9-07.2 *A `(a, usd)` read on an account with 1,000 `eur` postings and 16 `usd` postings visits at
  most 17 rows (index keyed by the reconstruction key) — or the thesis bound is restated as
  target-key rows and the adversary case is a committed test that reports the visited count.*
- C9-07.3 *An E18 row measures checkpoint bytes per posting at C = 16 (expected ≈ 2 B), and §9.14.1,
  §9.14.5, `E19-scaling.md` state the interval their tables were measured at.*
**Guard:** revert the constructor → 1,025 rows: red; revert the index key → C9-07.2 red.
**Host C:** *the author must now run `bash ~/Documents/niles-hostc/c9-checkpoints.sh` and paste
the output* (C = 0 / 16 / 64 / 256; LC-32 is decided on it).

### C9-08 — holes compact to ⊥, and `install` certifies the whole graph (F-66, A9-F11, A9-F15, LC-31) — `c9/09-bounded-view`

Fable's design (§2, F-66) with Astra's scope: demand/evictable views only — Full mode initialises
absent entries from deltas and must either track completeness or refuse compaction; `Runtime::
install` walks the reachable input graph and refuses anything but source → filters → keyed
`Sum`/`Count` (no join engine is built to make a guard pass).
**Targets.**
- C9-08.1 *The `rev_holes_100k` E18 row's live bytes are at most 2× the `rev_holes_5k` row's (was
  14.8×), both rows published with budgets, with active flights quiescent.*
- C9-08.2 *A circuit with a `sum` over a join is refused by `Runtime::install` naming the node, and
  the thesis sentence "the runtime refuses to install a circuit outside Q_lin" is true of the core
  API.*
- C9-08.3 *Chapter 3 states honest absence as the audit property and Θ(budget + active flights)
  view memory under full retention; `MISMATCH-hole-version-unused` is removed; G3 passes unchanged.*
**Guard:** revert compaction → ratio ≥ 14×: red; revert the graph walk → the join circuit installs:
red.
**Host C:** *the author must now run `bash ~/Documents/niles-hostc/c9-memory.sh` and paste the
output.*

### C9-09 — E18 on the production structures (F-65, A9-F09, F-77) — `c9/10-e18-rows`

Fable's C9-09 rows (§2, F-65: the two idem rows on `Ledger::with_idem_window` and the sealer's real
pair; `rev_holes_5k/100k`; `plan_lowered_point`; stand-ins kept one cycle under `_stand_in`) plus
Astra's: a traced plan-cache hit rate on named traces (repeated prepared statement; 256/257 working
sets; 1,000 distinct twice; the C9-G01 trace when it exists), reported as compilations / hits /
misses / evictions / bytes; per-session state × 16 sessions as *live bytes*, with E24's RSS kept
separate and never inferred from struct sizes.
**Targets.**
- C9-09.1 *`results/E18-memory.md` carries the two idempotency rows measured on the shipped types
  beside their stand-ins, and the report states the delta between each pair.*
- C9-09.2 *`rev_holes_5k`, `rev_holes_100k`, `plan_lowered_point`, `checkpoints_per_posting` are
  rows with budgets, and the plan-cache trace table is in `docs/BENCHMARK.md` with its commands.*
**Guard:** each row's budget test; a `type_name` assertion that the row builds the shipped type.

### C9-10 — the sentence ledger and the MISMATCH inventory (F-73, F-74, F-78, Astra §2.5) — `c9/11-prose`

Astra's chapter 3/4 sentence-cluster ledger is the input: every T/F/C-S verdict becomes a sentence
edit *with the guard that justifies it* or a `MISMATCH-A9-<id>`; the full inventory of literal
`MISMATCH` markers in ch. 3, 4, 9 and their generators (including `MISMATCH-F-07`,
`MISMATCH-T-11-overdraw`, F-08/F-09) with resolved / still-running status; the dead fallback branch
(F-78) becomes a counted refusal asserted 0 by the E16 harness; the generated-prose inventory
(Astra §2.6's table) is checked artefact by artefact.
**Targets.**
- C9-10.1 *No sentence in §9.14 states a figure that differs from the committed CSV beside it; the
  executor lists every figure checked.*
- C9-10.2 *`view_fallbacks` is asserted 0 by `e16_nilestream.rs` and the branch that incremented it
  no longer folds the base.*
- C9-10.3 *Every literal `MISMATCH` in chapters 3, 4 and 9 is in one table with its status, and none
  was removed without cited evidence.*

### C9-11 — a literal-templated plan key (F-69, LC-33) — backlog
LRU as the interim (simulated 15.3% → 9.8%); the templated key is a lowering with parameters and
belongs with `Bind` — design first, in `docs/`.

### C9-G01 — one durable syndicated lifecycle trace and a joint G3 witness (F-79, A9-F16, A9-F17, F-12, F-24, LC-34) — gbs `c9/01-g3-witness` then `c9/02-syndicate-lifecycle`

Fable's thousand-lender trace (§2, F-79) merged with Astra's A9-G01: (1) G3's `wire`, `durable`,
`crash` columns computed from the run that exercised each (a subprocess daemon with a durable sink,
the wire in the same run, a real kill/restart; the old same-process reopen relabelled "completed-log
reopen"); (2) lifecycle transitions persisted atomically with their posting sets and *derived* on
replay — no product field is commit authority; (3) the 1,000-share payment as a machine-readable
trace (`results/traces/syndicated-1000.csv`: event, logical anchor, fan-out, legs, currencies,
anchors read, visited rows, hits/folds, transitions, bytes/batch/barriers) replayable by `bench
--trace`; leg count and sealer-transaction count measured separately.
**Target.** *A product-specific 1,000-share syndicated payment trace survives process restart with
lifecycle and posting state reconstructed at every requested anchor (as-of rebuilds > 0 by
construction, not by hand), conserves to the minor unit with a deterministic residual, and G3
reports wire, durable, eviction and crash evidence from the actual witness that exercised each.*
**Guard:** drop the residual → `ParticipantSumMismatch` and G3 red; revert durable lifecycle
derivation → the restart/anchor guard red; a fixture with no durable sink must show `durable: no`.
No GBS efficiency target. **Host C:** *the author must now run `bash ~/Documents/niles-hostc/
c9-gbs.sh` and paste the output.*

### C9-12 — determinism, matched arms and the C republish (carried T-07 / T-08 / T-09 / T-15.1 / T-15.3; LC-07, LC-22, LC-24, LC-26, LC-27, LC-29) — `c9/12-contract-evidence`

Astra's A9-09 a/b/c as written: both SHA-256 streams across C and the container on one fixture;
two arms in one benchmark process with sample-level arm identity and matched writer counts; E23
refit with range and error; p99 at 16 re-read after C9-06 while measuring the remaining B wait;
E24 RSS under the workload; LC-22 first as a transcript diff. Republish only rows whose PG16,
concurrency, storage and noise prerequisites passed; closes `MISMATCH-e16-header`,
`MISMATCH-daemon-checkpoints`, LC-27.
**Targets.**
- C9-12.1 *The `chain` and `write_rows` byte streams agree across the pinned C and container builds
  on the identical corpus, with host and toolchain provenance attached.*
- C9-12.2 *One benchmark process records matched baseline and candidate arms, and Host C republishes
  only contract rows whose PostgreSQL version, concurrency, storage and noise prerequisites passed.*
- C9-12.3 *E23 slopes, the 16-connection p99 and E24 RSS are reported from explicit configurations
  as measured, blocked, not run or noise-limited, without borrowing another host's absolutes.*
**Host C:** *the author must now run `bash ~/Documents/niles-hostc/c9-determinism.sh`, then
`c9-wire.sh`, then `c9-publish.sh` (dry run), then `c9-publish.sh --publish` after review, and
paste each output.*

---

## 4. Branch stacks, merge order and landings

The executor has no remote and cannot push. `5eb0bb2` and `963e4d9` are immutable baselines; the
integration parent for niles is `5a64595` (the author's `c7/01-durable-rows` is at it — the Mac
preflight shows that branch checked out there), so every `c9/*` branch below descends from
`5a64595`, in this order, each on the previous:

```
niles  5eb0bb2 → 5a64595 (integration parent)
  → c9/00-audit              this document (Fable) — landed first
  → c9/01-gate-fixtures      C9-00
  → c9/02-instruments        C9-01 (+ every MISMATCH disclosure)
  → c9/03-owned-receipts     C9-02 (two commits: receipts, then the coordinate)
  → c9/04-strict-recovery    C9-03
  → c9/05-contracts          C9-04
  → c9/06-currency-table     C9-05
  → c9/07-pending            C9-06 (pinned install first; deferred-delta merge as a second commit)
  ——— cut ———
  → c9/08-checkpoints        C9-07
  → c9/09-bounded-view       C9-08
  → c9/10-e18-rows           C9-09
  → c9/11-prose              C9-10
  → c9/12-contract-evidence  C9-12 (after any GBS trace it depends on)
gbs    963e4d9
  → c9/01-g3-witness         C9-G01 (witness columns; narrows claims, closes nothing)
  → c9/02-syndicate-lifecycle C9-G01 (durable lifecycle + trace; needs the niles API landed and accepted)
```

Never reuse a branch name for unrelated work (`BLOCKED-branch-collision`). Before each task record
the integration SHA, the audit baseline and the preceding task's tip. GBS lands after the niles API
it needs is *accepted by the author*, never before.

**Every landing is a bundle and an immediate sync block**, written to
`~/Documents/niles-sync/cycle-9/` on the Mac through the desktop bridge, verified with `git bundle
verify`, with its SHA-256, base, tip and commit list in the report. Bundle names:
`niles-<nn>-<branch-suffix>.bundle` (`niles-01-gate-fixtures.bundle`, …, `niles-12-contract-
evidence.bundle`; an ordinal suffix `-2`, `-3` when a task lands more than once) and
`gbs-01-g3-witness.bundle`, `gbs-02-syndicate-lifecycle.bundle`. The executor sends this block with
the literals substituted **at the moment the landing exists**, not at the end (no `#` comments —
the author's shell rejects them):

```
cd ~/Documents/niles
git fetch ~/Documents/niles-sync/cycle-9/niles-01-gate-fixtures.bundle c9/01-gate-fixtures
git merge --ff-only FETCH_HEAD
git branch -f c9/01-gate-fixtures HEAD
git push origin c9/01-gate-fixtures c7/01-durable-rows
RUSTUP_AUTO_INSTALL=0 cargo +1.97.1 clippy --offline --all-targets -- -D warnings
```

(`cd ~/Documents/GBS`, `gbs-01-g3-witness.bundle`, `c9/01-g3-witness`, `c7/00-adapter` for GBS.)
`git branch -f` is against the task branch, never the checked-out one; if `--ff-only` refuses,
report `BLOCKED-sync-divergence` — never merge, rebase or reset for the author. A 1.97.1 lint the
container cannot see is fixed from the pasted output as a follow-up commit, never by rewriting an
accepted one.

---

## 5. Validation protocol

Per task, in the task's worktree, `RUSTUP_TOOLCHAIN=stable RUSTUP_AUTO_INSTALL=0`:

```
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline --workspace --no-fail-fast
cargo test --offline --workspace --no-fail-fast -- --test-threads=8      (three times)
taskset -c 0 cargo test --offline --workspace --no-fail-fast -- --test-threads=1
make reproduce ; git --no-optional-locks status --porcelain
make fsync-proof
```

PostgreSQL 16 on 5432 with the `bench` role (start it as §0 records; `PGPORT` if elsewhere) so
`numeric_binary_oracle` runs; its absence is **RED, blocked prerequisite**, never "flaky". On C the
author adds `cargo +1.97.1 clippy` (1.95.0 for everything else); C has no `strace`, so
`fsync-proof` is `unsupported` there and `c9-storage.sh` is the durability witness.

| outcome | meaning and next action |
|---|---|
| green | the exact command exited 0, the expected cases ran, pins matched, clean diff, guards passed; ignored/skipped counts stated |
| red — implementation or guard | keep the log and the failing input; fix the behaviour or mark the target `not done`; never delete, ignore or loosen |
| red — missing prerequisite | `BLOCKED-<id>` with the exact tool/service/version; the contract stays unvalidated; independent work continues |
| unsupported | the operation cannot execute under the allowed tools on this host; never simulated |
| not run | no execution evidence; name the dependency and the next command |
| noise-limited | a timing change fails the preregistered gate (≥ 10% ∧ ≥ 3 × pooled MAD, `sqrt((MAD_A² + MAD_B²)/2)`); samples kept; no win claimed |
| changed verdict | list each run's result; repair the precondition or the cause, never the threshold |

Deterministic counters (instructions, allocations, visited rows, `max_batch`, corpus tallies,
compiler verdicts) gate on one controlled run; allocation attribution is single-threaded or
per-operation, never a global difference. Wall clock: two warm-ups, ≥ 5 measured runs per arm and
condition, AB/BA interleaved, equal initial history, median / MAD / range, raw samples kept;
tails and error counts reported with throughput; a comparison stops on a wrong SHA, dirty source,
wrong server configuration, harness errors, an unverified barrier or a missing sample — the output
is kept with `blocked`. Fixed unoptimised controls and refused-input controls stay in every timed
suite, so refusing every query cannot be the fastest path.

---

## 6. Host C scripts — the executor writes them, the author runs them

None of these exists yet. Opus writes each under `docs/audit/cycle-9/hostc/` in the tree (tracked,
so its text is evidence), the author copies it to `~/Documents/niles-hostc/`, and every "the author
must run this now" marker in §3 names one of them. Estimates are planning figures.

**Shared contract (every script).** `set -euo pipefail`; a Python 3 supervisor (the Mac has no
`timeout`) with per-command and global monotonic deadlines, `start_new_session=True`, cleanup of
*owned* process groups only; non-interactive; no network, installs or sudo. It prints its own
SHA-256, the commands, the caps and the intended writes before doing anything. An **embedded
manifest** carries the full 40-character source and candidate SHAs (resolved when authored, never
at run time), the paired GBS/niles SHAs where needed, configuration (seed, rounds, budget,
checkpoint interval, idem window, barrier profile), host and sample counts; a resolved object that
differs is a refusal. A script-owned run root outside both repositories, one new bounded output
directory per invocation, free space checked first, nothing historical deleted. **Retarget-and-
refuse**: only script-owned sibling worktrees, verified clean *before* retargeting to the embedded
detached SHA and clean *again* after; the author's checkouts are never retargeted, cleaned or
pruned; the five protected files are never touched; unexpected untracked files refuse. Builds are
offline with the true exit captured; no stale `target/release` fallback. Any section that needs
`nilestreamd` starts **its own** child with explicit flags and a fresh segment per arm and
replicate, refuses an occupied port, waits for protocol readiness plus the expected banner
configuration, and reaps only that child. Any section that needs PostgreSQL checks **once, up
front**, an explicitly identified PG16 (`SHOW server_version_num` in 160000–169999) at
127.0.0.1:5433 with the `bench` role and records `fsync`, `synchronous_commit`, `full_page_writes`,
`wal_level`, `shared_buffers`, `work_mem`, `max_wal_size`, **`wal_sync_method`**, `max_connections`;
absent → `BLOCKED-PG16` with complete `initdb`/`pg_ctl`/`createdb` lines for a script-owned data
directory using the *verified* PG16 path (never PATH's 18.6, never `pg_ctlcluster` on macOS). Every
status is printed (blocked / unsupported / not run / noise-limited); grep output is never a verdict;
the exit code is non-zero if a required section did not run; tracked CSVs/docs are never written
without `--publish`, and `--publish` runs in an isolated publication worktree and emits a diff and
bundle for the author to land — it never pushes.

| script | what it does | est. / cap | marker |
|---|---|---|---|
| `c9-gates.sh` | both trees at the embedded SHAs; §5 gate with `--no-fail-fast`, three 8-thread runs, 1.97.1 lint, changed-verdict summary, protected-file sizes; PG precheck reports the oracle separately | 5–15 min / **45 min** | after C9-00, C9-04, and every landing the author wants re-gated |
| `c9-storage.sh` | verify the tree's real mount; the platform barrier probe with `F_FULLFSYNC` beside plain `fsync`; the candidate's receipt guard on that path; ≤ 16 MiB probe data | 1–3 min / **10 min** | before any new durability row; before `c9-durability.sh` |
| `c9-durability.sh` | C9-02/C9-03 candidates against their predecessor controls: latch-driven delayed and failing barriers on the daemon's bytes (two connections; two writers; a disconnected owner), the W−1/W/W+1 window boundary under batch sizes 1 and > 1, the strict corpus (exhaustive, tally printed), a real kill/reopen | 5–15 min / **30 min**, each witness ≤ 60 s | after C9-02; again after C9-03 |
| `c9-pending.sh` | the corrected section D: own daemon per replicate, `--checkpoint-interval 0` on both arms, same budget/seed/window/storage, exact 6r3w / 9r5w / 12r6w, 30 s mixed intervals, `nilestream_lockstats reset` before each level, phase-local counters, `--baseline-only` stage; interleaved baseline/candidate with 2 warm-ups + 5 measured per arm and shape | 25–35 min / **45 min** (baseline-only ≈ 12 min) | `--baseline-only` after C9-01; full after C9-06 |
| `c9-memory.sh` | E18 rows on production structures; slot-map growth to 100k keys at fixed budget; plan cache 1/256 plans, 1/16 sessions; window occupancy at declared W within a ≤ 2 GiB cap (refuses 1,000,000 if over); own daemon only for RSS | 5–15 min / **30 min** | after C9-08; after C9-09 |
| `c9-checkpoints.sh` | own daemon at C = 0 / 16 / 64 / 256 on one immutable multi-currency history with historical anchors; deterministic visited/lookup counts first, then paired wall clock C0/C16/C64, memory separately | 10–20 min / **40 min** | after C9-07; LC-32 is decided on it |
| `c9-gbs.sh` | paired SHAs; the 1,000-share trace through a subprocess durable wire daemon at a binding budget; kill/restart at declared points; independent fold and allocation oracle; replay/lifecycle/anchor counters | 5–15 min / **30 min** | after C9-G01 |
| `c9-determinism.sh` | `chain` and `write_rows` streams on the shipped fixture, compared byte-for-byte with the container's manifest | 1–5 min / **15 min** | C9-12.1 |
| `c9-wire.sh` | own daemon; PG16 oracle; the old/new declared-scale transcript for LC-22, transcript-only first | 2–5 min / **15 min** | before any LC-22 change |
| `c9-publish.sh` | C9-12: paired arms in one process, E16/E19 at matched concurrency, E23 refit, E24 RSS; dry run by default; `--publish` explicit and still gated on every prerequisite | 60–120 min / **180 min** | C9-12.2/3, dry run first |

`run6.sh` is a template for the retarget logic and a finding for everything else (labels, shared
segment, grep verdicts, four-file exclusion); it is not copied unchanged. `run4.sh` is not run.

---

## 7. The open-questions ledger

**Settled, not reopened:** LC-01, 02, 04, 08–12, 14, 15, 17, 18, 21 (fail-stop), 28 (epochs).
**Closed this audit:** **LC-23** — the tail is the view mutex holding V across the fold (LC-23
attribution, confirmed by both audits); the *exclusion* of everything else is withdrawn (A9-F07);
the repair is C9-06 with its own two statuses.

| LC | position |
|---|---|
| 03 AccountId String→u64 | carry; C9-04's key packing at the view is the same decision, not proof for GBS; measure with cross-repo canonical-byte guards after C9-00 |
| 05 rendered NULL / refusals | carry; no data check removed (A9-F13); LC-22's transcript first |
| 06 allocator / process boundary | carry; E18 allocated/live/peak vs E24 RSS kept separate (C9-09, C9-12) |
| 07 E23 slope | carry; refit only on new admissible C data (C9-12.3) |
| 13 release checks | carry; the gate and the author's 1.97.1 at every landing |
| **16 durable by default** | **decided in C9-04:** refuse without `--durable` unless `--volatile` is explicit — after C9-02, because a durable banner without owned receipts is not a durable daemon |
| 19 the wire claim's three clauses | not closed from the brief's "now all true": the engine's wire capabilities and G3's joint witness are different scopes; fold into one ch. 6 sentence at the tested scope after C9-G01 |
| 20 | `BLOCKED-LC20-definition` — the brief carries "not decided" with no subject; the author supplies it |
| 22 declared scale on the wire | carry; `c9-wire.sh` transcript diff first; declaration-index codes are non-negotiable (C9-04.1) |
| 24 p99 at 16 | carry; V is a candidate cause, not the proven sole one; measure the remaining B wait after C9-06 (C9-12.3) |
| 25 concurrent crash | prior "no" stands for a broad task; C9-02's witnesses are the narrow ones this cycle needs |
| 26 same-process A/B | not done; script existence is not two arm identities in one sample stream (C9-12.2) |
| 27 E19 CSVs | `not_run` until the C republish |
| 29 oltp verdict concurrency | into SPEC-ENGINE Part 0 with C9-12.2 |
| **30 Pending, different anchor** | **proposed:** join only on identical `(key, anchor)`; other anchors fold independently and do not install; bounded flights; generation-owned completion (C9-06) — the author confirms the protocol |
| **31 what a hole is** | **proposed:** an audit fact; demand/evictable holes compact to ⊥ under full retention; ⊥ means "no cached assertion", never zero; Full mode tracks completeness (C9-08) |
| **32 checkpoint interval** | **proposed:** a flag, default 16 (Fable) — Astra's 64 is in the C matrix; explicit 0 ablation; not a schema declaration this cycle; O(K + N/C) memory measured (C9-07) |
| **33 plan-cache policy** | keep FIFO 256 until the traced hit rate (C9-09) decides; the templated key is C9-11 |
| **34 first product** | **proposed:** the 1,000-share syndicated payment as a durable lifecycle trace (C9-G01); nothing else above any line |
| **35 undeclared window** | **decided in C9-04:** refusal unless `--idem-window` is explicit |
| **36 (new) a damaged final record after a clean shutdown** | the format cannot tell it from a torn write without a retained tip; `BLOCKED-recovery-tip` until the author gives the contract (a clean-close marker, or a tip kept outside the segment) |

Markers carried until their tasks close: `MISMATCH-pending-unreachable` (C9-06.3),
`MISMATCH-e16-header` (C9-12), `MISMATCH-daemon-checkpoints` (C9-07/C9-12),
`MISMATCH-hole-version-unused` (C9-08), `MISMATCH-idem-fingerprint` (author decision),
`MISMATCH-A9-F01/F02/F05/F12/F15` (placed in C9-01, closed by their tasks), and the pre-existing
`MISMATCH-F-07`, `MISMATCH-T-11-overdraw`, F-08/F-09 (inventoried in C9-10).

---

## 8. Reporting requirements for the executor

One execution report (`docs/audit/cycle-9/execution-report.md`), results only, with at the top the
audit baselines, the integration parent, every task tip, the measured binary SHAs and both trees'
states. Evidence classes on every figure: *measured on the executing host*, *measured by the author
on request*, *read from source*, *inferred from a document*.

**The checklist — every target line verbatim, with `done` / `not done: why` and its evidence.**

| id | target |
|---|---|
| C9-00.1 | The forced temporary-name collision guard passes without overwriting or deleting another live fixture, and three complete `--no-fail-fast -- --test-threads=8` workspace runs on the executing host show no changed verdict. |
| C9-00.2 | Every test that spawns a thread and asserts on a count states its precondition as its own assertion message, distinct from the property, and the same four runs (three at 8 threads, one pinned to one core at 1 thread) pass under two `yes >/dev/null &` hogs. |
| C9-01.1 | Every mixed row names the readers and writers actually spawned, its start and end epoch, and phase-local (since-reset) lock counters; no printed residual or histogram bound claims time outside its measured boundaries. |
| C9-01.2 | Two overlapping reconstructions each report their own visited-row count and the global total is their sum, asserted by a test. |
| C9-01.3 | Every E19 write is a two-leg transfer with a non-zero amount on both arms, asserted on the statement generator. |
| C9-01.4 | Every discrepancy this work order names below the cut line has a `MISMATCH-<id>` at its sentence, and the §9.14.1 miss-rate sentence reads the CSV's column. |
| C9-01.5 | `preflight.sh` prints "cycle-9", "five untracked files", real core and memory figures on Darwin, and finds an installed toolchain without `timeout`. |
| C9-02.1 | Each successful reply is written only after every receipt of the statements it answers has returned, and a receipt can be observed by no other connection — proved by the delayed-barrier two-connection test on the daemon's real bytes. |
| C9-02.2 | A failed barrier is reported as 58030 to the statement that caused it and to no other connection, tested with a failing injected barrier and a concurrent reader. |
| C9-02.3 | Admission, sealer and replay agree on one declared window in logical epochs at W−1, W and W+1 under batch sizes 1 and > 1, and the segment record's sequence number is named `batch_seq` everywhere it appears. |
| C9-02.4 | No path applies a transaction the sink then refuses and continues accepting; the witness for Astra §2.3's window gap either reproduces the applied-undurable state and shows the daemon stops, or is recorded as "not reproduced" with the invariant that prevents it named. |
| C9-03.1 | Every single-bit flip in a non-final record's header, body or CRC is refused with the input file's bytes unchanged, and the committed corpus reports zero prefix-with-success cases outside the final record. |
| C9-03.2 | `Sequencer::open_bounded` returns `Err(InvalidData)` on an envelope it cannot decode exactly (short, trailing bytes, invalid UTF-8), and no public constructor can start a sequencer with a window built by `unwrap_or_default`. |
| C9-03.3 | `docs/SPEC-ENGINE.md` states the self-checking record header, the three-way classification (clean end / torn tail / damage) and the version byte, and the thesis sentence on the record layout carries no `MISMATCH`. |
| C9-03.4 | A segment cut exactly at a record boundary is recorded as `LC-36` / `BLOCKED-recovery-tip`: indistinguishable from a shorter log without a retained tip, which is the author's contract to give — never claimed solved by the parser. |
| C9-04.1 | Every currency code in the IR and on the wire is the currency's declaration index, tested by lowering a three-currency schema under permuted `HashMap` insertion and asserting identical `LitMoney` codes and wire codes each time. |
| C9-04.2 | `nilestreamd --budget 2,500` (and each other flag with a non-integer) exits non-zero naming the flag and the text. |
| C9-04.3 | Two relations declaring different windows are refused at startup naming both; one relation gives its window deterministically across ten starts. |
| C9-04.4 | `txn idem("k", window: 30.days)` is NL0218 in every corpus that used it (25 files, none under `docs/audit/`); a `txn idem(..)` posting to a relation without an `idem` column is NL0219; the mutant is refused; `ignored_windows` no longer exists. |
| C9-04.5 | `nilestreamd` with neither `--durable` nor `--volatile` exits non-zero; with `--volatile` the banner says VOLATILE; a schema without an `idem` column on the wire relation is refused unless `--idem-window` is given. |
| C9-05.1 | `schema_parses` after 2,000 inserts in one session at one schema epoch reads 1; after a schema change it reads 2; tested. |
| C9-05.2 | `Session::insert` on the `checked-twice oltp` probe costs at most 45,000 instructions per insert under callgrind on the executing host (was 113,500), stated with the command. |
| C9-05.3 | The four SQLSTATE refusals on the insert path each still have a passing test. |
| C9-06.1 | Every concurrent read equals an independent fold at its own requested anchor while a delayed reconstruction overlaps an `advance`, and same-key same-anchor readers share exactly one fold without holding V while folding or waiting — proved by the latch-driven two-phase differential whose precondition counter (`pinned_installs + deferred_merges + pending_joins > 0`) is asserted. |
| C9-06.2 | At the corrected 12r6w Host C point, Pending delivers at least 1.25× the baseline median read throughput under the ≥ 10% ∧ ≥ 3 pooled-MAD gate, with no contract or durability regression and the slowest-16 table's `view_wait_us` maximum reported beside it (expected: ≥ 2.5× and < 2 ms; reported, not required). |
| C9-06.3 | `nilestream_stats` reports `pending_joins`, `uninstalled_folds`, `pinned_installs`, `flights_refused`; the 12r6w run shows `pending_joins > 0`; `MISMATCH-pending-unreachable` is removed only then. |
| C9-06.4 | Theorem 4.1's proof carries the Pending clause; §3's lattice paragraph and its read-path rules (3:160, 3:162) state the exact-anchor interval `stamp ≤ a ≤ effective`, the join on equal anchors and the generation-owned install as implemented. |
| — | *cut line* — |
| C9-07.1 | `nilestreamd` without flags serves with `checkpoint_interval = 16`, prints it, carries it in every provenance header, and a head read of a 1,024-posting key touches at most 17 base rows. |
| C9-07.2 | A `(a, usd)` read on an account with 1,000 `eur` postings and 16 `usd` postings visits at most 17 rows (index keyed by the reconstruction key) — or the thesis bound is restated as target-key rows and the adversary case is a committed test that reports the visited count. |
| C9-07.3 | An E18 row measures checkpoint bytes per posting at C = 16 (expected ≈ 2 B), and §9.14.1, §9.14.5, `E19-scaling.md` state the interval their tables were measured at. |
| C9-08.1 | The `rev_holes_100k` E18 row's live bytes are at most 2× the `rev_holes_5k` row's (was 14.8×), both rows published with budgets, with active flights quiescent. |
| C9-08.2 | A circuit with a `sum` over a join is refused by `Runtime::install` naming the node, and the thesis sentence "the runtime refuses to install a circuit outside Q_lin" is true of the core API. |
| C9-08.3 | Chapter 3 states honest absence as the audit property and Θ(budget + active flights) view memory under full retention; `MISMATCH-hole-version-unused` is removed; G3 passes unchanged. |
| C9-09.1 | `results/E18-memory.md` carries the two idempotency rows measured on the shipped types beside their stand-ins, and the report states the delta between each pair. |
| C9-09.2 | `rev_holes_5k`, `rev_holes_100k`, `plan_lowered_point`, `checkpoints_per_posting` are rows with budgets, and the plan-cache trace table is in `docs/BENCHMARK.md` with its commands. |
| C9-10.1 | No sentence in §9.14 states a figure that differs from the committed CSV beside it; the executor lists every figure checked. |
| C9-10.2 | `view_fallbacks` is asserted 0 by `e16_nilestream.rs` and the branch that incremented it no longer folds the base. |
| C9-10.3 | Every literal `MISMATCH` in chapters 3, 4 and 9 is in one table with its status, and none was removed without cited evidence. |
| C9-G01 | A product-specific 1,000-share syndicated payment trace survives process restart with lifecycle and posting state reconstructed at every requested anchor (as-of rebuilds > 0 by construction, not by hand), conserves to the minor unit with a deterministic residual, and G3 reports wire, durable, eviction and crash evidence from the actual witness that exercised each. |
| C9-12.1 | The `chain` and `write_rows` byte streams agree across the pinned C and container builds on the identical corpus, with host and toolchain provenance attached. |
| C9-12.2 | One benchmark process records matched baseline and candidate arms, and Host C republishes only contract rows whose PostgreSQL version, concurrency, storage and noise prerequisites passed. |
| C9-12.3 | E23 slopes, the 16-connection p99 and E24 RSS are reported from explicit configurations as measured, blocked, not run or noise-limited, without borrowing another host's absolutes. |

**Also required.** (1) The gate matrix with full `--no-fail-fast` counts per run, changed verdicts by
name, ignored tests by name; the Mac's oracle red and its cause. (2) For every repair and
optimisation, the guard transcript: worktree name, candidate SHA, the reverted lines, the intended
red assertion, the restored green, exit codes; GBS transcripts name the sibling niles path. (3)
Timing tables with a host column, raw samples, warm-ups, interleave order, median/MAD/range, the
noise rule, configuration and history at start and end, error counts; E18 (allocated/live/peak) and
E24 (RSS) never in one column. (4) The updated sentence ledger (Astra §2.5), the MISMATCH inventory,
the generated-prose inventory, and the LC ledger with the author's decisions. (5) **Exactly three
material facts found while executing that this work order does not cover**, each with evidence
class, file/line or command, consequence and whether it moves the cut — negative findings qualify;
nothing already in §2/§2A may be recycled; fewer than three is reported as `not done: only N`. (6)
Worktree status before and after, separating task edits from the five protected files (report their
sizes again: 10,244 / 16,639 / 6,148 / 8,196 / 816,110 bytes at audit time — a discrepancy is
referred to the author, never corrected); every SHA; every bundle with hash, base, tip; the expanded
sync block for every landing; the author's 1.97.1 result. No executor push; no baked-in trailer —
attribution is the executing session's own.

**Audit boundary.** Fable ran the gate, the probes, the corpus prototype and the callgrind
attribution in the container and read the Mac trees; Astra read source and had the author run the
Mac gate. Nothing in §3 is implemented; no script in §6 exists; every figure labelled "expected" is
a prediction.
