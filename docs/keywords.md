# The Niles Keyword Reference

<!-- GENERATED FILE — DO NOT EDIT.
     Produced by `cargo run -p niles-lang --bin gen-keyword-ref` from the keyword
     registry in `crates/niles-lang/src/keywords.rs`. `tests/keyword_ref.rs` fails if
     this file and the registry disagree, so the two cannot drift. -->

This is the normative per-keyword reference of thesis Appendix B.19. It is **generated** from the compiler's keyword registry, not maintained alongside it: a keyword cannot exist in the lexer without an entry here, and an entry here cannot describe a keyword the lexer does not have.

**174 keywords**: 95 unreserved, 59 reserved (including reserved-for-future), 20 in the remaining two classes.

## How to read the tables

Two orthogonal axes, following PostgreSQL's `kwlist.h`, which carries both because a language with a keyword-delimited grammar and an installed base of schemas needs both.

**Category** — how far the word is reserved:

| Category | Meaning |
|---|---|
| `unreserved` | No special status outside its own clause. Usable as any identifier, anywhere. |
| `non-reserved (cannot be function or type name)` | Usable as a column or variable, but the word introduces built-in syntax that would be ambiguous with a call. |
| `reserved (can be function or type name)` | Usable as a function or type name; a bare occurrence in column position would be read as a clause. |
| `reserved` | Never an identifier without the `r#` escape. |
| `reserved for future use` | Reserved with no meaning assigned. Using it is a hard error naming the reason. |

**Label** — whether the word may be a bare output label without `as`. `select 55 check` is legal; `select 55 as from` is required. This is a second dimension, not a fifth category value.

**Escape.** Any word, however reserved, is usable as an identifier as `r#word`.

**Reservation policy.** A new keyword is `unreserved` unless a written justification records why the grammar cannot be written without reserving it. Reserved-word count is a function of parser technology, not of vocabulary size: Niles uses hand-written recursive descent with unbounded lookahead, which keeps words like `epoch`, `ledger`, `serve` and `budget` available as column names in a bank's existing schema. Every novel keyword in this reference is unreserved, and a test enforces it.

## SQL-derived keywords (71)

Taken from SQL, with SQL's meaning wherever the meaning survives. Where it does not — `update` and `delete` are legal against a `table` and meaningless against a `ledger` — the difference is stated in the entry.

| Keyword | Category | Label | Since | Description | Example |
|---|---|---|---|---|---|
| `add` | unreserved | bare | 2026 | Add a column or constraint in `alter table`. | `alter table accounts add column tier: i32;` |
| `all` | reserved | requires `as` | 2026 | Keep duplicates in a set operation, or the universal quantifier. | `let x = a.union_all(b);` |
| `alter` | unreserved | bare | 2026 | Change a declared object. Legal on `table`, never on `base` or `ledger`. | `alter table accounts add column tier: i32;` |
| `and` | reserved | requires `as` | 2026 | Short-circuiting boolean conjunction. | `where(\|r\| r.open and r.cur == usd)` |
| `any` | unreserved | bare | 2026 | Existential quantifier over a subquery or collection. | `where(\|r\| r.tags.any(\|t\| t == "vip"))` |
| `as` | reserved | requires `as` | 2026 | Rename a column, bind an output label, or coerce a value. | `select(\|r\| (r.amt as label("amount")))` |
| `asc` | unreserved | bare | 2026 | Ascending sort direction; the default. | `order_by(\|r\| asc(r.posted_at))` |
| `begin` | unreserved | bare | 2026 | Open a table-level transaction. Ledger writes use `txn` instead. | `begin; insert into accounts values (1, "a"); commit;` |
| `between` | non-reserved (cannot be function or type name) | bare | 2026 | Inclusive range test. | `where(\|r\| r.amt between (0.00 usd, 100.00 usd))` |
| `by` | reserved | requires `as` | 2026 | Introduces the grouping or ordering key; never appears alone. | `group_by(\|r\| r.acct)` |
| `case` | reserved | requires `as` | 2026 | Multi-way conditional expression. `match` is the Rust-derived alternative. | `case when r.amt > 0 then "cr" else "dr" end` |
| `cast` | non-reserved (cannot be function or type name) | bare | 2026 | Explicit conversion. Never crosses a currency: `Money<USD>` has no cast to `Money<EUR>`. | `cast(r.n as i64)` |
| `check` | unreserved | bare | 2026 | Row-level constraint on a table. | `table t { n: i32 check (n >= 0) }` |
| `column` | unreserved | bare | 2026 | Names a column in DDL. | `alter table accounts add column tier: i32;` |
| `commit` | unreserved | bare | 2026 | Close a table transaction, making its writes visible at the next epoch. | `begin; update accounts set tier = 2; commit;` |
| `create` | unreserved | bare | 2026 | Declare an object. Inside `schema` the word is optional. | `create index ix on postings (acct);` |
| `cross` | reserved (can be function or type name) | requires `as` | 2026 | Unrestricted product join. | `a.cross_join(b)` |
| `default` | unreserved | bare | 2026 | Column default. Must be a pure, epoch-independent expression. | `table t { n: i32 default 0 }` |
| `delete` | unreserved | bare | 2026 | Remove rows. Legal on `table` only; a `ledger` has no delete. | `delete from staging where done;` |
| `desc` | unreserved | bare | 2026 | Descending sort direction. | `order_by(\|r\| desc(r.amt))` |
| `distinct` | reserved | requires `as` | 2026 | Deduplicate. On a Z-set this is the canonicalising `distinct` operator. | `postings.distinct_by(\|r\| r.acct)` |
| `drop` | unreserved | bare | 2026 | Remove a declared object. Never removes ledger history. | `drop view stale_v;` |
| `else` | reserved | requires `as` | 2026 | Alternative branch of `case` or `if`. | `case when p then a else b end` |
| `end` | non-reserved (cannot be function or type name) | requires `as` | 2026 | Closes a `case` expression. Position-determined, so `end` remains usable as a column name. | `case when p then a else b end` |
| `except` | reserved | requires `as` | 2026 | Set difference. | `a.except(b)` |
| `exists` | non-reserved (cannot be function or type name) | bare | 2026 | Non-emptiness test over a subquery. | `where(\|r\| exists(holds.for_acct(r.acct)))` |
| `foreign` | unreserved | bare | 2026 | Introduces a foreign key constraint. | `foreign key (acct) references accounts (id)` |
| `from` | non-reserved (cannot be function or type name) | requires `as` | 2026 | Source relation, in the SQL surface and in `delete`. Clause-position only, so it remains usable as a variable or column name. | `sql { select id from accounts }` |
| `full` | reserved (can be function or type name) | requires `as` | 2026 | Full outer join; also the `full` materialization mode and `lineage: full`. | `a.full_outer_join(b, \|x, y\| x.k == y.k)` |
| `grant` | unreserved | bare | 2026 | Confer a capability. Niles grants an `Auth<E>`, not an ambient role. | `grant debit<usd> on postings to teller;` |
| `group` | non-reserved (cannot be function or type name) | requires `as` | 2026 | Introduces grouping; the pipeline spelling is `group_by`. Clause-position only. | `group_by(\|r\| (r.acct, r.cur))` |
| `having` | reserved | requires `as` | 2026 | Filter applied after grouping, over group aggregates. | `group_by(\|r\| r.acct).having(\|g\| g.sum > 0.00 usd)` |
| `in` | reserved | requires `as` | 2026 | Membership test, and the binder in `for x in xs`. | `where(\|r\| r.cur in [usd, eur])` |
| `index` | unreserved | bare | 2026 | Declare an anchor index. Anchor indices are mandatory on ledger keys. | `index by_acct on postings (acct) anchor;` |
| `inner` | reserved (can be function or type name) | requires `as` | 2026 | Inner join; the default join kind. | `a.join(b, \|x, y\| x.k == y.k)` |
| `insert` | unreserved | bare | 2026 | Add rows to a `table`. A `ledger` is appended to with `txn`, not `insert`. | `insert into accounts values (1, "ada");` |
| `intersect` | reserved | requires `as` | 2026 | Set intersection. | `a.intersect(b)` |
| `into` | unreserved | bare | 2026 | Target of an `insert`. | `insert into accounts values (1, "ada");` |
| `is` | reserved | requires `as` | 2026 | Identity test, chiefly `is null` and `is not null`. | `where(\|r\| r.closed_at is null)` |
| `join` | reserved (can be function or type name) | requires `as` | 2026 | Relational join. In a pipeline it is a stage; in `sql {}` it is a clause. | `postings.join(accounts, \|p, a\| p.acct == a.id)` |
| `key` | unreserved | bare | 2026 | Part of `primary key` / `foreign key`; also `lineage: key`. | `table t { id: i64 primary key }` |
| `left` | reserved (can be function or type name) | requires `as` | 2026 | Left outer join. | `a.left_join(b, \|x, y\| x.k == y.k)` |
| `like` | non-reserved (cannot be function or type name) | bare | 2026 | Pattern match on `Text`. | `where(\|r\| r.name like "ac%")` |
| `limit` | non-reserved (cannot be function or type name) | requires `as` | 2026 | Bound the result cardinality. Clause-position only, so `limit` remains usable as a column name. | `order_by(\|r\| desc(r.amt)).limit(10)` |
| `natural` | reserved (can be function or type name) | requires `as` | 2026 | Join on all like-named columns. Discouraged: it is schema-fragile. | `a.natural_join(b)` |
| `not` | reserved | requires `as` | 2026 | Boolean negation. | `where(\|r\| not r.closed)` |
| `null` | reserved | requires `as` | 2026 | The SQL null. Distinct from `Option::None` and from an evicted `Hole`. | `where(\|r\| r.closed_at is null)` |
| `offset` | non-reserved (cannot be function or type name) | requires `as` | 2026 | Skip a prefix of the result. Clause-position only. | `order_by(\|r\| r.id).offset(20).limit(10)` |
| `on` | reserved | requires `as` | 2026 | Join predicate, or the object of a `grant`. | `a.join(b, \|x, y\| x.k == y.k)` |
| `or` | reserved | requires `as` | 2026 | Short-circuiting boolean disjunction. | `where(\|r\| r.a or r.b)` |
| `order` | non-reserved (cannot be function or type name) | requires `as` | 2026 | Introduces ordering; the pipeline spelling is `order_by`. Clause-position only. | `order_by(\|r\| asc(r.id))` |
| `outer` | reserved (can be function or type name) | requires `as` | 2026 | Marks a join as outer. | `a.full_outer_join(b, \|x, y\| x.k == y.k)` |
| `primary` | unreserved | bare | 2026 | Introduces the primary key. | `table t { id: i64 primary key }` |
| `recursive` | unreserved | bare | 2026 | Marks a CTE as recursive. Niles requires a `guard measure(..)` on the recursion regardless. | `sql { with recursive r as (select 1) select * from r }` |
| `references` | unreserved | bare | 2026 | Target of a foreign key. | `foreign key (acct) references accounts (id)` |
| `revoke` | unreserved | bare | 2026 | Withdraw a capability. Recorded as a ledger event, never a silent edit. | `revoke debit<usd> on postings from teller;` |
| `right` | reserved (can be function or type name) | requires `as` | 2026 | Right outer join. | `a.right_join(b, \|x, y\| x.k == y.k)` |
| `rollback` | unreserved | bare | 2026 | Abandon a table transaction. A sealed ledger epoch cannot be rolled back. | `begin; update t set n = 1; rollback;` |
| `select` | reserved | requires `as` | 2026 | Projection, in the SQL surface. The pipeline spelling is `.map`/`.select`. | `sql { select id, owner from accounts }` |
| `set` | unreserved | bare | 2026 | Assignment list of an `update`. | `update accounts set tier = 2 where id == 1;` |
| `table` | unreserved | bare | 2026 | A mutable relation: update and delete are legal, history is not retained. | `table accounts { id: Id<Account> primary key }` |
| `then` | reserved | requires `as` | 2026 | Consequent of a `case` arm. | `case when p then a else b end` |
| `union` | reserved | requires `as` | 2026 | Set union, deduplicating. | `a.union(b)` |
| `unique` | unreserved | bare | 2026 | Uniqueness constraint. | `table t { k: Text unique }` |
| `update` | unreserved | bare | 2026 | Modify rows. Legal on `table` only; a `ledger` has no update. | `update accounts set tier = 2 where id == 1;` |
| `using` | reserved (can be function or type name) | requires `as` | 2026 | Join on named common columns. | `a.join_using(b, ["acct"])` |
| `values` | non-reserved (cannot be function or type name) | requires `as` | 2026 | Literal row constructor, in `insert`. Clause-position only. | `insert into accounts values (1, "ada");` |
| `view` | unreserved | bare | 2026 | A derived relation with a serve contract. The REV of the theory. | `view v = postings.group_by(\|p\| p.acct) serve { consistency: snapshot };` |
| `when` | reserved | requires `as` | 2026 | Guard of a `case` arm, or of a `match` arm. | `case when p then a else b end` |
| `where` | reserved | requires `as` | 2026 | Filter stage, and Rust's bound clause. The positions are disjoint. | `postings.where(\|p\| p.amt > 0.00 usd)` |
| `with` | reserved | requires `as` | 2026 | Common table expression in the SQL surface. | `sql { with t as (select 1) select * from t }` |

## Rust-derived keywords (30)

Taken from Rust, because SQL has no equivalent construct. Restrictions apply in query and transaction context: no ambient I/O, no wall clock, no `unsafe`, and recursion only through a guarded `fixpoint`.

| Keyword | Category | Label | Since | Description | Example |
|---|---|---|---|---|---|
| `Self` | reserved | requires `as` | 2026 | The implementing type. | `fn zero() -> Self { Self { amt: 0.00 usd } }` |
| `break` | reserved | requires `as` | 2026 | Leave a loop. Illegal inside a `fixpoint` body, which must terminate by measure. | `loop { if done { break; } }` |
| `const` | reserved | requires `as` | 2026 | Compile-time constant. Must be pure and epoch-independent. | `const LIMIT: Money<USD> = 500.00 usd;` |
| `continue` | reserved | requires `as` | 2026 | Next iteration of a loop. | `for p in ps { if p.zero() { continue; } post(p)?; }` |
| `crate` | reserved | requires `as` | 2026 | Path root of the current compilation unit. | `use crate::bank::transfer;` |
| `dyn` | reserved | requires `as` | 2026 | Dynamic dispatch. Forbidden in view bodies, which must be statically planned. | `let f: &dyn Rate = &fixed;` |
| `enum` | reserved | requires `as` | 2026 | Sum type. | `enum Side { Debit, Credit }` |
| `false` | reserved | bare | 2026 | Boolean literal. | `let b = false;` |
| `fn` | reserved | requires `as` | 2026 | Function item. Its effect row is part of its type. | `fn transfer(a: Acct, b: Acct) -> Result<()> ! { append, debit<usd> } { .. }` |
| `for` | reserved | requires `as` | 2026 | Iteration, or the `impl .. for ..` head. | `for p in batch { post(p)?; }` |
| `if` | reserved | requires `as` | 2026 | Conditional expression. | `if r.amt > lim { flag(r) } else { r }` |
| `impl` | reserved | requires `as` | 2026 | Inherent or trait implementation. | `impl Rate for Fixed { .. }` |
| `let` | reserved | requires `as` | 2026 | Bind a value. Linear types bound by `let` must be consumed exactly once. | `let d = debit(a, 10.00 usd)?;` |
| `loop` | reserved | requires `as` | 2026 | Unconditional loop. Not permitted in a view body. | `loop { step()?; }` |
| `match` | reserved | requires `as` | 2026 | Pattern match. Exhaustive, unlike SQL's `case`. | `match outcome { Post(m) => .., Void => .., Expire => .. }` |
| `mod` | reserved | requires `as` | 2026 | Module. | `mod bank { .. }` |
| `move` | reserved | requires `as` | 2026 | Closure captures by value. Required for closures crossing an epoch boundary. | `let f = move \|r\| r.amt;` |
| `mut` | reserved | requires `as` | 2026 | Mutable binding. Never applies to a sealed ledger row. | `let mut acc = 0.00 usd;` |
| `pub` | reserved | requires `as` | 2026 | Export from a module or schema. | `pub view balances = ..;` |
| `ref` | reserved | requires `as` | 2026 | Bind by reference in a pattern. | `match o { Some(ref v) => .., None => .. }` |
| `return` | reserved | requires `as` | 2026 | Early return. | `if p.zero() { return Ok(()); }` |
| `self` | reserved | requires `as` | 2026 | Receiver parameter or path root. | `fn amount(&self) -> Money<USD> { self.amt }` |
| `static` | reserved | requires `as` | 2026 | Program-lifetime item. Immutable; there is no mutable static. | `static ZERO: Money<USD> = 0.00 usd;` |
| `struct` | reserved | requires `as` | 2026 | Product type. | `struct Leg { acct: Acct, amt: Money<USD> }` |
| `super` | reserved | requires `as` | 2026 | Parent module in a path. | `use super::money::round;` |
| `trait` | reserved | requires `as` | 2026 | Interface with associated items. | `trait Rate { fn convert(&self, m: Money<USD>) -> Money<EUR>; }` |
| `true` | reserved | bare | 2026 | Boolean literal. | `let b = true;` |
| `type` | reserved | requires `as` | 2026 | Type alias or associated type. | `type Cents = Money<USD>;` |
| `use` | reserved | requires `as` | 2026 | Import a path. | `use std::bank::transfer;` |
| `while` | reserved | requires `as` | 2026 | Conditional loop. Not permitted in a view body. | `while !q.empty() { step()?; }` |

## Novel Niles keywords (66)

These name concepts neither SQL nor Rust has: an immutable epoch-ordered base, a conservation rule, a per-view consistency contract, a materialization mode, a linear hold, an atomic cross-currency form, a confidentiality level. Every one of them is unreserved.

| Keyword | Category | Label | Since | Description | Example |
|---|---|---|---|---|---|
| `absent` | unreserved | bare | 2026 | Materialization mode: nothing resident; every read reconstructs. | `serve { materialize: absent }` |
| `anchor` | unreserved | bare | 2026 | The epoch stamp carried by every answer, and the mandatory index kind on a ledger key. | `let e = balances.get(k)?.anchor;` |
| `as_of` | unreserved | bare | 2026 | Pin a read to a system-time epoch. The system axis of bitemporality. | `balances.as_of(#4200).get(k)` |
| `authorize` | unreserved | bare | 2026 | Check a floor against a capability. The only construct that may overdraw. | `authorize(auth, acct, 50.00 usd)?` |
| `auto` | unreserved | bare | 2026 | Delegate materialization to the optimizer, within the rest of the contract. | `serve { materialize: auto }` |
| `backfill` | unreserved | bare | 2026 | Populate a newly declared view from history, without re-deriving the base. | `backfill v upto #10000;` |
| `base` | unreserved | bare | 2026 | An immutable, fully retained, epoch-ordered authoritative relation. `ledger` is `base` plus a conservation rule. | `base events { id: u64, payload: Json }` |
| `bitemporal` | unreserved | bare | 2026 | Declares both time axes on a relation: recorded_at and valid_at. | `ledger p { .. } bitemporal;` |
| `bounded` | unreserved | bare | 2026 | Consistency rung 0: anchored no more than K epochs or T milliseconds behind. | `serve { consistency: bounded(epochs: 4, millis: 200) }` |
| `budget` | unreserved | bare | 2026 | Resident-state ceiling for a view, in entries or bytes. | `serve { materialize: demand, budget: 50_000 }` |
| `capability` | unreserved | bare | 2026 | Declare an unforgeable authority token for an effect. | `capability Overdraft: Auth<debit<usd>>;` |
| `committed` | unreserved | bare | 2026 | Confidentiality level: readable only inside the enclave that holds the key. | `owner: Text @confidential(committed)` |
| `confidential` | unreserved | bare | 2026 | Mark a column end-to-end encrypted; the engine may not compute on it. | `owner: Text @confidential(e2ee)` |
| `conserve` | unreserved | bare | 2026 | The double-entry rule: the group's amounts must sum to zero per currency. | `conserve per (txn, cur);` |
| `consistency` | unreserved | bare | 2026 | The rung a view is served at. | `serve { consistency: ledger_consistent }` |
| `currency` | unreserved | bare | 2026 | Declare a currency and its minor-unit scale. Scale lives in the type. | `currency jpy { scale: 0 }` |
| `declassify` | unreserved | bare | 2026 | The single audited construct that lowers a confidentiality level. | `declassify(row.owner, auth)?` |
| `demand` | unreserved | bare | 2026 | Materialization mode: materialize only what is read; evict the rest. | `serve { materialize: demand }` |
| `e2ee` | unreserved | bare | 2026 | Confidentiality level: end-to-end encrypted, opaque to the engine. | `owner: Text @confidential(e2ee)` |
| `emit` | unreserved | bare | 2026 | Publish a derived change downstream as a stream. | `emit v to sink;` |
| `epoch` | unreserved | bare | 2026 | The unit of visibility, versioning and hashing. Also the literal prefix `#`. | `let e: Epoch = #4200;` |
| `evictable` | unreserved | bare | 2026 | Retention of derived state: may be dropped and reconstructed. | `serve { retain: evictable }` |
| `expire` | unreserved | bare | 2026 | Resolve a hold by lapse of its window, releasing the reservation. | `resolve h expire` |
| `expires` | unreserved | bare | 2026 | The window on a hold or an idempotency key. | `hold(acct, 20.00 usd, expires: 7.days)` |
| `explain` | unreserved | bare | 2026 | Show how an answer was derived, at the view's lineage mode. | `explain balances.get(k)?;` |
| `forever` | unreserved | bare | 2026 | Retention: never evicted. Mandatory on a base or ledger. | `ledger p { .. } retain forever;` |
| `freshness` | unreserved | bare | 2026 | The staleness bound of a bounded rung, as a duration. | `serve { consistency: bounded, freshness: 200.millis }` |
| `fx` | unreserved | bare | 2026 | Atomic cross-currency form: two conserved legs sealed in one epoch. | `fx { leg a: post(du, cu), leg b: post(dm, cm), rate: r }` |
| `guard` | unreserved | bare | 2026 | Attach the termination witness to a fixpoint. Unguarded recursion is rejected. | `edges.fixpoint(step) guard measure(depth)` |
| `hold` | unreserved | bare | 2026 | Reserve funds as a ledger fact. Linear: resolvable exactly once. | `let h = hold(acct, 20.00 usd, expires: 7.days)?;` |
| `idem` | unreserved | bare | 2026 | Declare a transaction's idempotency key, or a relation's idempotency column. The window is declared on the column, in epochs. | `txn idem("ext-991") { .. }` |
| `impact` | unreserved | bare | 2026 | The inverse of `explain`: which views a base row can affect. | `impact postings.row(#4200, 7);` |
| `ledger` | unreserved | bare | 2026 | A base with a conservation rule and typed money columns. Never partial. | `ledger postings { txn: TxnId, acct: Id<Account>, amt: Money }` |
| `ledger_consistent` | unreserved | bare | 2026 | Consistency rung 5: anchored at the visibility frontier exactly. The authorization path. | `serve { consistency: ledger_consistent }` |
| `leg` | unreserved | bare | 2026 | One side of an `fx` form. | `leg a: post(d_usd, c_usd)` |
| `lineage` | unreserved | bare | 2026 | Provenance mode: off, key, or full. Off still stamps the anchor. | `serve { lineage: full }` |
| `materialize` | unreserved | bare | 2026 | The mode a view's state is kept in. | `serve { materialize: tiered }` |
| `measure` | unreserved | bare | 2026 | The well-founded quantity a guarded fixpoint decreases. | `guard measure(depth)` |
| `monotonic` | unreserved | bare | 2026 | Consistency rung 1: per session, anchors never decrease. | `serve { consistency: monotonic }` |
| `of` | unreserved | bare | 2026 | Connective in `as_of` and `impact of` forms. | `explain of balances.get(k)?;` |
| `per` | unreserved | bare | 2026 | Introduces the grouping of a conservation rule. | `conserve per (txn, cur);` |
| `pinned` | unreserved | bare | 2026 | Retention: resident and never evicted, at a declared memory cost. | `serve { retain: pinned }` |
| `post` | unreserved | bare | 2026 | Resolve a hold into a posting, capturing up to the held amount. | `resolve h post 18.50 usd` |
| `posting` | unreserved | bare | 2026 | One signed movement; the linear unit a ledger row is built from. | `let p: Posting = debit(a, 10.00 usd)?;` |
| `rate` | unreserved | bare | 2026 | The declared conversion of an `fx` form, recorded with the legs. | `fx { .., rate: r }` |
| `read_your_writes` | unreserved | bare | 2026 | Consistency rung 2: a session observes its own committed writes. | `serve { consistency: read_your_writes }` |
| `recorded_at` | unreserved | bare | 2026 | The system-time axis: when the fact was recorded. Never rewritten. | `where(\|r\| r.recorded_at <= #4200)` |
| `reproduce` | unreserved | bare | 2026 | Re-derive an answer from the base and compare, as an audit act. | `reproduce balances.get(k) at #4200;` |
| `resolve` | unreserved | bare | 2026 | Consume a hold exactly once, by post, void or expire. | `resolve h post 18.50 usd` |
| `retain` | unreserved | bare | 2026 | Retention of derived state; the base is always retained. | `serve { retain: pinned }` |
| `scale` | unreserved | bare | 2026 | The minor-unit exponent of a currency. Part of the type, never assumed. | `currency bhd { scale: 3 }` |
| `schema` | unreserved | bare | 2026 | The declaration unit: currencies, tables, bases, ledgers, views, indices. | `schema demo_bank { .. }` |
| `serializable` | unreserved | bare | 2026 | Consistency rung 4: read anchors form a serial order with transactions. | `serve { consistency: serializable }` |
| `serve` | unreserved | bare | 2026 | Attach a contract to a view. The contract is part of the view's type. | `view v = q serve { consistency: snapshot, materialize: auto };` |
| `signal` | unreserved | bare | 2026 | An epoch-indexed value: balance as a function of time. | `let b: Signal<Money<USD>> = balance_of(acct);` |
| `snapshot` | unreserved | bare | 2026 | Consistency rung 3: one anchor for the whole read set, across views. | `serve { consistency: snapshot }` |
| `spilled` | unreserved | bare | 2026 | Materialization mode: resident on secondary storage. Illegal at rung 5. | `serve { materialize: spilled }` |
| `sql` | unreserved | bare | 2026 | Escape into the SQL surface. Same IR, different syntax. | `sql { select id from accounts }` |
| `tiered` | unreserved | bare | 2026 | Materialization mode: hot entries resident, cold spilled, by the optimizer. | `serve { materialize: tiered }` |
| `txn` | unreserved | bare | 2026 | A ledger transaction: the unit the conservation rule is checked over. | `txn idem("k1") { post(d)?; post(c)?; }` |
| `udf` | unreserved | bare | 2026 | A user-defined function, fuel-metered and compiled to WASM. | `#[udf(fuel = 1_000_000, deterministic)] fn f(x: i64) -> i64 { x }` |
| `upto` | unreserved | bare | 2026 | Bound a backfill or a replay at an epoch. | `backfill v upto #10000;` |
| `valid_at` | unreserved | bare | 2026 | The valid-time axis: when the fact was true in the world. | `balances.valid_at(@2026-03-01).get(k)` |
| `value_date` | unreserved | bare | 2026 | The banking value date of a posting; drives valid-time placement. | `post(d, value_date: @2026-03-03)?` |
| `void` | unreserved | bare | 2026 | Resolve a hold by cancelling it, releasing the reservation with no posting. | `resolve h void` |
| `window` | unreserved | bare | 2026 | How long an idempotency key is remembered, in epochs, on a relation's `idem` column. | `idem: IdemKey window 1_000_000.epochs` |

## Reserved for future use (7)

Reserved with no meaning assigned in this edition. Using one is a hard error that names the reason, which is how a language keeps room to grow without a breaking change later.

| Keyword | Reason |
|---|---|
| `actor` | Reserved: no meaning assigned in this edition. |
| `async` | Reserved: query and transaction context is synchronous by design. |
| `await` | Reserved: see `async`. |
| `macro` | Reserved: no macro system in this edition. |
| `stream` | Reserved: streams are expressed as bases, not as a separate kind. |
| `unsafe` | Reserved and rejected: there is no unsafe fragment of Niles. |
| `yield` | Reserved: no generators in this edition. |

## The normative reserved list (Appendix B.16)

68 words. All require `r#` to be used as identifiers.

```
Self actor all and as async await break by case const continue crate cross distinct dyn else enum except false fn for full having if impl in inner intersect is join left let loop macro match mod move mut natural not null on or outer pub ref return right select self static stream struct super then trait true type union unsafe use using when where while with yield
```

## Words that are *not* reserved, and why that matters

95 words carry meaning in their own clause and are ordinary identifiers everywhere else. A bank migrating a schema whose columns are called `epoch`, `ledger`, `posted`, `serve`, `budget` or `scale` does not have to rename them.

```
absent add alter anchor any as_of asc authorize auto backfill base begin bitemporal bounded budget capability check column commit committed confidential conserve consistency create currency declassify default delete demand desc drop e2ee emit epoch evictable expire expires explain foreign forever freshness fx grant guard hold idem impact index insert into key ledger ledger_consistent leg lineage materialize measure monotonic of per pinned post posting primary rate read_your_writes recorded_at recursive references reproduce resolve retain revoke rollback scale schema serializable serve set signal snapshot spilled sql table tiered txn udf unique update upto valid_at value_date view void window
```

