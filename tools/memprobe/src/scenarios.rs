//! The E18 scenarios: what is measured, and on what.
//!
//! Each returns a [`Row`] and nothing else — no printing, no files — so the same function
//! backs the published table and the budget test, and the two cannot measure different things
//! under one name.
//!
//! The workload matches E16's committed configuration (10,000 accounts, two rounds, a
//! residency budget of 2,500) so a memory figure and a wall-clock figure describe the same
//! engine. A scenario that measured a different shape would be answering a question nobody
//! asked of the system under test.

use crate::alloc::{count, Row};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Serving;
use proto_engine::{EvictionPolicy, Ledger, Posting, Row as LedgerRow, ViewMode};

pub const ACCOUNTS: i64 = 10_000;
pub const ROUNDS: u32 = 2;
pub const BUDGET: usize = 2_500;
/// Postings in the seeded base: two conserved legs per account per round.
pub const POSTINGS: u64 = (ACCOUNTS as u64) * (ROUNDS as u64) * 2;

fn engine() -> RevEngine {
    RevEngine::seeded(
        ACCOUNTS,
        ROUNDS,
        BUDGET,
        ViewMode::Demand,
        EvictionPolicy::Lru,
    )
}

/// Compile a wire statement exactly as `session` does, against the daemon's own schema.
///
/// Through the real compiler rather than a hand-built circuit: a memory figure for a query
/// the server would not have produced is a figure for a different query.
fn circuit(sql: &str) -> niles_ir::circuit::Circuit {
    circuit_as(sql, "auto")
}

/// The same, with the serve contract's materialization named. `full` is what makes
/// maintenance *install* a key it has not seen, which is the difference between a view that
/// holds everything and one that holds what somebody read.
fn circuit_as(sql: &str, materialize: &str) -> niles_ir::circuit::Circuit {
    let program = format!(
        "{}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: {materialize} }};\n",
        nilestream_server::daemon::DEFAULT_SCHEMA
    );
    let (prog, mut d) = niles_lang::parser::parse_program(&program);
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    d.extend(rd);
    let (_r, td) = niles_lang::typecheck::check_program(&prog, &cat);
    d.extend(td);
    assert!(
        !d.has_errors(),
        "`{sql}` does not compile: {:?}",
        d.sorted()
    );
    let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
    assert!(!ld.has_errors(), "`{sql}` has no lowering");
    assert!(niles_ir::verify::verify(&lowered.circuit).is_ok());
    lowered.circuit
}

/// **What a seeded ledger costs to hold.** The bytes-per-posting figure the project did not
/// have: the epoch records, the idempotency set and the anchor index, over the base E16 runs
/// against.
pub fn ledger_seeded() -> Row {
    let (e, counted) = count(engine);
    // Held across the reading so `live_delta` is what the structure costs rather than what a
    // temporary did.
    std::hint::black_box(&e);
    let row = Row {
        scenario: "ledger_seeded",
        unit: "posting",
        operations: POSTINGS,
        counted,
    };
    drop(e);
    row
}

/// **The Z-set the reference evaluator's base is materialised into**, per row.
///
/// Measured separately from a query because it is the cost the *representation* imposes
/// rather than one any operator asks for: a `BTreeMap<Vec<Value>, i128>` is a heap allocation
/// per row for the row, another for the tree's node, and a 16-byte payload in a 32-byte enum.
pub fn zset_base_at() -> Row {
    let mut ledger = Ledger::new();
    let mut txn = 0u64;
    for round in 0..ROUNDS {
        for a in 1..=ACCOUNTS as u64 {
            txn += 1;
            let amt = 100 + (round as i128 * 7 + a as i128 % 13);
            let _ = ledger.submit(
                &format!("seed-{txn}"),
                vec![
                    LedgerRow::Post(Posting {
                        txn,
                        acct: a,
                        cur: 0,
                        amt,
                        valid: round as i64,
                    }),
                    LedgerRow::Post(Posting {
                        txn,
                        acct: 0,
                        cur: 0,
                        amt: -amt,
                        valid: round as i64,
                    }),
                ],
            );
        }
    }
    let (z, counted) = count(|| {
        let mut z: niles_ir::eval::ZSet = Default::default();
        for e in &ledger.epochs {
            for r in &e.rows {
                if let LedgerRow::Post(p) = r {
                    niles_ir::eval::add(
                        &mut z,
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
        }
        z
    });
    let n = z.len() as u64;
    drop(z);
    Row {
        scenario: "zset_base_at",
        unit: "row",
        operations: n,
        counted,
    }
}

/// One served statement, over the wire path's own `Serving::query`.
fn served(scenario: &'static str, sql: &str, queries: u64) -> Row {
    let e = engine();
    let c = circuit(sql);
    let anchor = e.frontier();
    // Warm: the first query of a process pays for whatever the compiler and the allocator
    // have not yet touched, and that is not the per-query cost.
    let _ = e.query(&c, "__wire_result", anchor).expect("serves");
    let (_, counted) = count(|| {
        for _ in 0..queries {
            let r = e.query(&c, "__wire_result", anchor).expect("serves");
            std::hint::black_box(&r);
        }
    });
    Row {
        scenario,
        unit: "query",
        operations: queries,
        counted,
    }
}

pub fn served_group_by_cur() -> Row {
    served(
        "served_group_by_cur",
        "select cur, sum(amt) from postings group by cur",
        3,
    )
}

pub fn served_group_by_acct() -> Row {
    served(
        "served_group_by_acct",
        "select acct, sum(amt) from postings group by acct",
        3,
    )
}

pub fn served_sum_negative() -> Row {
    served(
        "served_sum_negative",
        "select sum(amt) from postings where amt < 0",
        3,
    )
}

/// **The top-ten shape: an ordering and a limit above a folded aggregate.**
///
/// Measured separately from `served_group_by_acct` because it is the one common statement
/// where the fold's ten thousand groups are *not* the answer — ten of them are — so a cost
/// proportional to the groups is a cost that should not be there. Before T-06 the limit
/// cloned every group into a vector and sorted all of them to keep ten.
pub fn served_top_ten() -> Row {
    served(
        "served_top_ten",
        "select acct, sum(amt) from postings group by acct order by sum(amt) desc limit 10",
        3,
    )
}

pub fn served_point() -> Row {
    served(
        "served_point",
        "select acct, sum(amt) from postings where acct = 4242 group by acct",
        200,
    )
}

/// **The same question with a second conjunct.**
///
/// `where acct = 4242 and cur = 0` restricts to one account exactly as `served_point` does,
/// and the engine used to scan the entire base for it: the restriction was derived from a
/// filter whose *whole* predicate had to be `acct = k`, and an `and` made it give up. A
/// second condition that narrows the query made it a thousand times more expensive.
pub fn served_point_conjunct() -> Row {
    served(
        "served_point_conjunct",
        "select acct, sum(amt) from postings where acct = 4242 and cur = 0 group by acct",
        200,
    )
}

/// **The same question spelled as a `having` over the group key.**
///
/// `group by acct having acct = 4242` selects one group, and every row of the base was read
/// to find it: a filter above the aggregate made the fold planner refuse the shape entirely,
/// so the query materialised. A `having` that names only grouping columns is a `where` in a
/// different position, and the aggregate does not need computing to know that.
pub fn served_having_on_key() -> Row {
    served(
        "served_having_on_key",
        "select acct, sum(amt) from postings group by acct having acct = 4242",
        200,
    )
}

/// **What writing a reply costs, at two sizes an order of magnitude apart.**
///
/// The claim T-32 makes is that a reply's peak memory is a constant. That is a claim about
/// *writing*, not about evaluating, so these measure the write: the engine's answer is
/// produced once, outside the counted region, and what is counted is turning it into bytes on
/// a socket.
///
/// Two sizes, because a constant is only visibly a constant against something that grew. The
/// Z-set the block borrows stays O(groups) — it is the fold's own per-group accumulator, and
/// PostgreSQL's hash aggregate holds one too — so what these compare is the envelope around
/// it.
fn wire_write(scenario: &'static str, rows: i128, reps: u64) -> Row {
    use nilestream_server::pg_wire::{self, Backend, Format, RowBlock};

    let mut z = niles_ir::eval::ZSet::new();
    for i in 0..rows {
        z.insert(
            vec![
                niles_ir::value::Value::Int(i),
                niles_ir::value::Value::Int(i * 7),
            ],
            1,
        );
    }
    let block = Backend::Rows(RowBlock {
        z,
        anchor: 4_200,
        formats: vec![Format::Text; 3],
    });
    // A sink that keeps nothing: what is being measured is what the *writer* holds, and a
    // buffer that grew to hold the whole reply would be measuring the test's own `Vec`.
    struct Discard;
    impl std::io::Write for Discard {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            std::hint::black_box(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let _ = pg_wire::write_all(&mut Discard, std::slice::from_ref(&block));
    let (_, counted) = count(|| {
        for _ in 0..reps {
            pg_wire::write_all(&mut Discard, std::slice::from_ref(&block)).expect("writes");
        }
    });
    Row {
        scenario,
        unit: "reply",
        operations: reps,
        counted,
    }
}

/// Ten thousand rows on the wire: the shape E16's `group_by_acct` sends.
pub fn wire_reply_10k() -> Row {
    wire_write("wire_reply_10k", 10_000, 5)
}

/// A hundred thousand rows, ten times the answer. **Peak must not follow it.**
pub fn wire_reply_100k() -> Row {
    wire_write("wire_reply_100k", 100_000, 5)
}

/// **The REV runtime's hit path** — the mechanism the thesis is about, which the wire path
/// does not yet use. Measured so that the difference between the two is a number.
pub fn rev_read_hit() -> Row {
    use nilestream_core::rev::{Base, Key, Policy, Runtime, Value};
    struct LedgerBase {
        ledger: Ledger,
    }
    impl Base for LedgerBase {
        fn frontier(&self) -> u64 {
            self.ledger.head()
        }
        fn reconstruct(&self, key: &Key, anchor: u64) -> (Value, u64) {
            let before = self.ledger.rows_touched();
            let v = self.ledger.reconstruct_balance(
                key[0] as u64,
                key.get(1).copied().unwrap_or(0) as u32,
                anchor,
            );
            (v, self.ledger.rows_touched() - before)
        }
        fn deltas_at(&self, e: u64) -> Vec<(Key, Value)> {
            let Some(rec) = self.ledger.epochs.get(e as usize) else {
                return Vec::new();
            };
            rec.rows
                .iter()
                .filter_map(|r| match r {
                    LedgerRow::Post(p) => Some((vec![p.acct as i64, p.cur as i64], p.amt)),
                    _ => None,
                })
                .collect()
        }
    }

    let mut ledger = Ledger::new();
    let mut txn = 0u64;
    for round in 0..ROUNDS {
        for a in 1..=ACCOUNTS as u64 {
            txn += 1;
            let amt = 100 + (round as i128 * 7 + a as i128 % 13);
            let _ = ledger.submit(
                &format!("seed-{txn}"),
                vec![
                    LedgerRow::Post(Posting {
                        txn,
                        acct: a,
                        cur: 0,
                        amt,
                        valid: round as i64,
                    }),
                    LedgerRow::Post(Posting {
                        txn,
                        acct: 0,
                        cur: 0,
                        amt: -amt,
                        valid: round as i64,
                    }),
                ],
            );
        }
    }
    let base = LedgerBase { ledger };
    let c = circuit("select acct, cur, sum(amt) from postings group by acct, cur");
    let mut rt = Runtime::install(c, Some(BUDGET as u64), Policy::Lru)
        .unwrap_or_else(|u| panic!("the keyed fragment must install: {}", u.explain()));
    let head = base.frontier();
    for e in 0..=head {
        rt.advance(&base, e);
    }
    let view = rt.view_mut("__wire_result").expect("the installed view");
    let key = vec![4242i64, 0];
    let _ = view.read(&base, &key, head);
    let reads = 1_000u64;
    let (_, counted) = count(|| {
        for _ in 0..reads {
            std::hint::black_box(view.read(&base, &key, head));
        }
    });
    Row {
        scenario: "rev_read_hit",
        unit: "read",
        operations: reads,
        counted,
    }
}

/// **An in-memory append of one conserved two-leg transaction.** The write path without the
/// `fsync`, so the allocation cost is separable from the durability cost.
pub fn append_in_memory() -> Row {
    let e = engine();
    let appends = 500u64;
    let mut n = 0u64;
    let _ = e.append(
        vec![
            LedgerRow::Post(Posting {
                txn: 800_000,
                acct: 1,
                cur: 0,
                amt: -5,
                valid: 0,
            }),
            LedgerRow::Post(Posting {
                txn: 800_000,
                acct: 2,
                cur: 0,
                amt: 5,
                valid: 0,
            }),
        ],
        "e18-warm",
    );
    let (_, counted) = count(|| {
        for _ in 0..appends {
            n += 1;
            let txn = 900_000 + n;
            e.append(
                vec![
                    LedgerRow::Post(Posting {
                        txn,
                        acct: (n % ACCOUNTS as u64) + 1,
                        cur: 0,
                        amt: -5,
                        valid: 0,
                    }),
                    LedgerRow::Post(Posting {
                        txn,
                        acct: 0,
                        cur: 0,
                        amt: 5,
                        valid: 0,
                    }),
                ],
                &format!("e18-{n}"),
            )
            .expect("a conserved append");
        }
    });
    Row {
        scenario: "append_in_memory",
        unit: "transaction",
        operations: appends,
        counted,
    }
}

/// Every scenario, in the order `memory::SCENARIOS` names them.
/// **What the view's per-key metadata costs, at twice the residency budget.**
///
/// The eviction budget bounds *values*. It does not bound the two policy maps beside them:
/// `reads_of` (per-key read counts, for the cost-aware policy) and `last_read` (per-key
/// clock, for LRU) took an entry for every key ever read and gave it back to nobody. The
/// slot map is different and must stay: an evicted entry becomes `Hole(e)`, which is honest
/// absence and the reason a miss is not a zero.
///
/// Read at `2 x BUDGET` distinct keys, so eviction is continuous and the difference between
/// "bounded by the budget" and "bounded by history" is a factor of two in this row and
/// unbounded in a running system. `live` is what the view still held when the region ended,
/// which is the figure that decides whether a long-lived view fits.
pub fn rev_metadata_2x_budget() -> Row {
    use nilestream_core::rev::{Base, Key, Policy, Runtime, Value};
    struct LedgerBase {
        ledger: Ledger,
    }
    impl Base for LedgerBase {
        fn frontier(&self) -> u64 {
            self.ledger.head()
        }
        fn reconstruct(&self, key: &Key, anchor: u64) -> (Value, u64) {
            let before = self.ledger.rows_touched();
            let v = self.ledger.reconstruct_balance(
                key[0] as u64,
                key.get(1).copied().unwrap_or(0) as u32,
                anchor,
            );
            (v, self.ledger.rows_touched() - before)
        }
        fn deltas_at(&self, e: u64) -> Vec<(Key, Value)> {
            let Some(rec) = self.ledger.epochs.get(e as usize) else {
                return Vec::new();
            };
            rec.rows
                .iter()
                .filter_map(|r| match r {
                    LedgerRow::Post(p) => Some((vec![p.acct as i64, p.cur as i64], p.amt)),
                    _ => None,
                })
                .collect()
        }
    }

    let mut ledger = Ledger::new();
    let keys = (BUDGET * 2) as u64;
    let mut txn = 0u64;
    for a in 1..=keys {
        txn += 1;
        let amt = 100 + (a as i128 % 13);
        let _ = ledger.submit(
            &format!("meta-{txn}"),
            vec![
                LedgerRow::Post(Posting {
                    txn,
                    acct: a,
                    cur: 0,
                    amt,
                    valid: 0,
                }),
                LedgerRow::Post(Posting {
                    txn,
                    acct: 0,
                    cur: 0,
                    amt: -amt,
                    valid: 0,
                }),
            ],
        );
    }
    let base = LedgerBase { ledger };
    let c = circuit("select acct, cur, sum(amt) from postings group by acct, cur");
    let mut rt = Runtime::install(c, Some(BUDGET as u64), Policy::Lru)
        .unwrap_or_else(|u| panic!("the keyed fragment must install: {}", u.explain()));
    let head = base.frontier();
    let view = rt.view_mut("__wire_result").expect("the installed view");
    let (_, counted) = count(|| {
        for a in 1..=keys {
            std::hint::black_box(view.read(&base, &vec![a as i64, 0], head));
        }
    });
    assert!(
        view.resident_count() <= BUDGET as u64,
        "the budget must bind for this row to mean anything: {} resident",
        view.resident_count()
    );
    Row {
        scenario: "rev_metadata_2x_budget",
        unit: "key",
        operations: keys,
        counted,
    }
}

/// The identity population both idempotency windows hold, in the wire's key shape.
///
/// Thirty-six characters, because that is what a client sends: a UUID, or something the
/// same size. A shorter key would understate the string heap, which is most of the cost.
fn identities(n: u64) -> Vec<String> {
    (0..n)
        .map(|i| {
            format!(
                "{:08x}-{:04x}-4{:03x}-8{:03x}-{:012x}",
                i,
                i % 65536,
                i % 4096,
                i % 4096,
                i
            )
        })
        .collect()
}

/// **The admission index: `proto_engine::Ledger::idem`, a `HashSet<String>`.**
///
/// One of *two* windows a durable daemon holds. This one answers whether a key has
/// committed; the sealer's answers at which epoch, and both hold every identity ever
/// committed. Neither is pruned by anything today, so the pair is the write path's
/// unbounded term.
///
/// Measured on the structure rather than through the sealer: the sealer owns its window on
/// its own thread, and a `#[global_allocator]`'s counters are process-wide, so a second
/// thread allocating inside a measured region lands in the numbers.
pub fn idem_admission_index() -> Row {
    let n = 100_000u64;
    let ids = identities(n);
    // The structure is *returned* from the measured region rather than forgotten: `count`
    // reads `live` before the returned value is dropped, so this reports what the window
    // holds without leaking it.
    let (_held, counted) = count(|| {
        let mut idem: std::collections::HashSet<String> = std::collections::HashSet::new();
        for k in &ids {
            idem.insert(k.clone());
        }
        std::hint::black_box(idem.len());
        idem
    });
    Row {
        scenario: "idem_admission_index",
        unit: "identity",
        operations: n,
        counted,
    }
}

/// **The sealer's window: `BTreeMap<String, u64>` in `nilestream_ledger::sequencer`.**
///
/// The second copy. It maps an identity to the epoch it committed at, because a retry must
/// be told the *original* epoch — "committed at a new epoch" would be a second transaction
/// wearing the first one's name. That is why it is a map and not a set, and why it cannot
/// simply be dropped in favour of the admission index above.
pub fn idem_window_sealer() -> Row {
    let n = 100_000u64;
    let ids = identities(n);
    let (_held, counted) = count(|| {
        let mut seen: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
        for (e, k) in ids.iter().enumerate() {
            seen.insert(k.clone(), e as u64);
        }
        std::hint::black_box(seen.len());
        seen
    });
    Row {
        scenario: "idem_window_sealer",
        unit: "identity",
        operations: n,
        counted,
    }
}

/// **Metadata per resident key, isolated — the T-05.2 figure.**
///
/// A full view is advanced through every epoch first, so every key already has a `Present`
/// slot and nothing about the slot map changes during the measured region. Then each key is
/// read once. Every read *hits*: no reconstruction, no install, no slot allocated. What is
/// left is exactly the per-key policy metadata a read creates — the read count for the
/// cost-aware policy and the clock for LRU — and `bytes/op` is therefore the metadata cost
/// of one resident key, measured in place rather than on a synthetic map.
pub fn rev_metadata_per_key() -> Row {
    use nilestream_core::rev::{Base, Key, Policy, Runtime, Value};
    struct LedgerBase {
        ledger: Ledger,
    }
    impl Base for LedgerBase {
        fn frontier(&self) -> u64 {
            self.ledger.head()
        }
        fn reconstruct(&self, key: &Key, anchor: u64) -> (Value, u64) {
            let before = self.ledger.rows_touched();
            let v = self.ledger.reconstruct_balance(
                key[0] as u64,
                key.get(1).copied().unwrap_or(0) as u32,
                anchor,
            );
            (v, self.ledger.rows_touched() - before)
        }
        fn deltas_at(&self, e: u64) -> Vec<(Key, Value)> {
            let Some(rec) = self.ledger.epochs.get(e as usize) else {
                return Vec::new();
            };
            rec.rows
                .iter()
                .filter_map(|r| match r {
                    LedgerRow::Post(p) => Some((vec![p.acct as i64, p.cur as i64], p.amt)),
                    _ => None,
                })
                .collect()
        }
    }

    let keys = BUDGET as u64;
    let mut ledger = Ledger::new();
    for a in 1..=keys {
        let amt = 100 + (a as i128 % 13);
        let _ = ledger.submit(
            &format!("perkey-{a}"),
            vec![
                LedgerRow::Post(Posting {
                    txn: a,
                    acct: a,
                    cur: 0,
                    amt,
                    valid: 0,
                }),
                LedgerRow::Post(Posting {
                    txn: a,
                    acct: 0,
                    cur: 0,
                    amt: -amt,
                    valid: 0,
                }),
            ],
        );
    }
    let base = LedgerBase { ledger };
    let c = circuit_as(
        "select acct, cur, sum(amt) from postings group by acct, cur",
        "full",
    );
    // No budget and `Materialize::Full`: every key is installed by maintenance, so the
    // measured region below allocates no slot and evicts nothing.
    let mut rt = Runtime::install(c, None, Policy::Lru)
        .unwrap_or_else(|u| panic!("the keyed fragment must install: {}", u.explain()));
    let head = base.frontier();
    for e in 0..=head {
        rt.advance(&base, e);
    }
    let view = rt.view_mut("__wire_result").expect("the installed view");
    let resident_before = view.resident_count();
    let (_, counted) = count(|| {
        for a in 1..=keys {
            std::hint::black_box(view.read(&base, &vec![a as i64, 0], head));
        }
    });
    assert_eq!(
        view.resident_count(),
        resident_before,
        "every read must hit: an install would put a slot in this row and it measures metadata"
    );
    Row {
        scenario: "rev_metadata_per_key",
        unit: "resident key",
        operations: keys,
        counted,
    }
}

pub fn all() -> Vec<Row> {
    vec![
        ledger_seeded(),
        zset_base_at(),
        served_group_by_cur(),
        served_group_by_acct(),
        served_top_ten(),
        served_sum_negative(),
        served_point(),
        wire_reply_10k(),
        wire_reply_100k(),
        served_point_conjunct(),
        served_having_on_key(),
        rev_read_hit(),
        append_in_memory(),
        rev_metadata_2x_budget(),
        rev_metadata_per_key(),
        idem_admission_index(),
        idem_window_sealer(),
    ]
}
