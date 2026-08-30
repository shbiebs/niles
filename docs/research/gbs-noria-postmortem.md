# GBS / Noria Post-Mortem: Evidence Extraction

**Source:** `/home/claude/work/niles/docs/research/noria-attempts-raw.txt` (4,565 lines, ~27,772 words)
**Method:** full read, line-numbered quotation. All line numbers refer to the raw transcript.
**Scope note:** this document is *extraction*, not argument. Section 9 states what the evidence does and does not support.

---

## Executive Summary

The transcript records a single developer ("checolino", macOS ARM64, RustRover) working with an AI assistant to build a core banking system called **GBS** (later "Global Banking System") in Rust on top of **Noria**, with a WASM/Yew browser GUI and an Axum HTTP server. The attempt ran through roughly eleven restarts and two engine changes, and ended without a single confirmed end-to-end run on any engine.

The single most important finding, and the one that most constrains what this transcript can be used to argue, is this:

> **Noria was never successfully built, started, or connected to. Not once. Zero SQL statements were ever executed against Noria. Every Noria failure in this transcript is a build, packaging, or distribution failure — not a semantic one.**

The failure chain against Noria was, in order: the `noria` crate needs nightly Rust (line 257); it uses `impl_trait_in_assoc_type` in a form modern nightly rejects (lines 284–436); its transitive `librocksdb-sys v6.20.3` panics under modern LLVM/bindgen (lines 546–578); no published Docker image exists (`mitpdos/noria:latest` → `manifest unknown`, line 2071); and building from source in Ubuntu 20.04 on `nightly-2021-01-01` still fails because *unpinned transitive deps* have moved on (`tokio-tower`, line 2364). At that point the assistant declared Noria "too unmaintained to compile cleanly" (line 2367) and pivoted to **Materialize**.

The Materialize phase produced the only *semantic* database refusals in the whole transcript — `CREATE TABLE with a primary key or unique constraint is not supported` (line 2622) — plus a long series of RBAC/permission and Docker-tag-availability problems (lines 3047, 3455, 3871, 3942). The stack was still not working when the transcript's build narrative ends; the last state is a frontend that compiles and renders but whose ping returns an error, and a backend whose schema bootstrap had not yet been observed to succeed.

Alongside this, the assistant repeatedly proposed **reimplementing database guarantees in application code**: validate double-entry in the Axum handler (line 109), "in a real banking app, you would wrap this in a SQL transaction" (line 807), generate ledger IDs with `rand::random::<i64>().abs()` (lines 811, 818) and later `transaction_id + 1` "so the primary key remains unique" (line 4313) — in a schema from which the primary key had been *deleted* two hundred lines earlier (line 2664). Balance reads map any database error to zero: `Err(_) => 0, // Account not found or zero balance` (line 839). No currency column, no overdraft check, and no audit metadata ever appear in any schema in the transcript.

Two things in the transcript actively cut *against* common versions of the thesis narrative. First, **money was never a float** — the assistant used `BIGINT` cents from the start and said why (line 738). Second, a very large fraction of the friction is neither Noria nor semantics: it is **Docker Desktop install conflicts, a chat UI that silently ate `<` and `>` from code blocks** (breaking Rust generics and Yew's `html!` macro across at least six exchanges), Homebrew binary collisions, CORS, and `trunk`'s own `lightningcss` build failure. On a straight count, incidental tooling friction is the largest single category.

**Honest verdict** (expanded in Section 9): this transcript supports *"this attempt did not reach production"* and *"the `noria` crate as published is not installable on a 2026 macOS toolchain without forking and patching"*. It does **not** support *"Noria cannot be used in production for core banking"* on semantic grounds, because no semantics of Noria were ever exercised.

---

## 1. Timeline / Narrative

### 1.1 Framing (line 1, user's own voice)

> `I had thought about creating a core baking system using Noria, but it wasn't production ready, here's what happened. Analyze, research, draft, prepare and build/write/correct/expand/refine the thesis and the repository.`

This is the user's retrospective framing, written *after* the events, when handing the transcript over. It is the user's own characterisation of the outcome ("wasn't production ready"), but it is a conclusion, not an observation, and it is worth keeping distinct from the in-transcript evidence.

### 1.2 The original goal (lines 2–8, user)

> `I want to create a core banking system using the rust programming language, the Noria database system (https://github.com/mit-pdos/noria)`
> `Begin by creating the main.rs file that will include the accounting, payments, cashflows and transactions systems. I want to use a web browser as the GUI, I need a WASM integration for best speed, but the logic must be run on the server using Rust`

Requirements as stated by the user: (a) Rust, (b) Noria, (c) accounting / payments / cashflows / transactions modules, (d) browser GUI via WASM, (e) all business logic server-side.

### 1.3 Architecture settled on

Three successive architectures, all sharing the same shape:

| | Write path | Read path | Server | GUI | Deployment |
|---|---|---|---|---|---|
| **A1** (lines 9–232) | `noria` crate `ControllerHandle::from_zk` | Noria views | axum 0.6 + tokio 1.28 | Yew/WASM served from `dist/` | local, ZooKeeper at `127.0.0.1:2181` |
| **A2** (lines 611–732, 2133–2348) | `sqlx` over Noria's **MySQL wire port 3306** | same | axum 0.6, **stable** Rust | Yew 0.20 + Trunk 0.21.14, port 8081 | docker-compose: zookeeper:3.8 + custom-built `noria-server` |
| **A3** (lines 2370–end) | `sqlx` over Materialize's **Postgres port 6875/6877** | `CREATE MATERIALIZED VIEW` | axum 0.6 | unchanged | docker-compose: `materialize/materialized:v0.68.0` |

Crates pinned across the transcript: `axum = "0.6"` (resolved 0.6.20), `tokio = "1.28"` (resolved 1.53.1), `sqlx = "0.7"` (resolved 0.7.4), `yew = "0.20"`, `reqwest = "0.11"` (0.11.27), `wasm-bindgen = "0.2"`, `trunk 0.21.14`, `rand = "0.8"` (0.8.7), `noria = "0.9"` **as requested** but resolved to **`noria-0.6.1`** in the registry (lines 31, 153 vs. line 286).

The data model never grew past this (line 3985 and passim):

```
CREATE TABLE IF NOT EXISTS public.ledger (
    id BIGINT,
    account_id BIGINT,
    amount BIGINT
);
CREATE MATERIALIZED VIEW IF NOT EXISTS public.account_balances AS
 SELECT account_id, SUM(amount) AS balance FROM public.ledger GROUP BY account_id;
```

An `accounts` table appears once (line 741) and is dropped from every subsequent restart. No currency, no posting date vs. value date, no transaction/journal grouping, no account type, no hash chain, no version or epoch column ever appears.

### 1.4 Where it stopped

The build narrative ends at line 4470. The last observed runtime facts are:

- Backend last **observed** database outcome: `permission denied for DATABASE "materialize"` (line 3942). The suggested fix (port 6877 as `mz_system`) was issued at lines 3960–4010, but the next user message (line 4015) is `cargo run could not determine which binary to run` — i.e. the backend had not yet started when that fix was applied. **No message in the transcript shows the backend successfully creating the ledger table or the view.**
- Frontend did compile and render; clicking the button produced `Error: builder error: relative URL without a base` (line 4032), then `Error: error sending request: JsValue(TypeError: Load failed undefined)` (line 4052), then further compile breakage (line 4075).
- The `POST /api/transfer` double-entry endpoint was written (lines 4270–4337, restated 4340–4470) but **never shown running**.

At line 4471 the user pivots away from building on an existing engine entirely:

> `I have this proposal for a thesis, I want to include building a dedicated database language and database management system.`

That is where the transcript's engineering narrative terminates and its research-proposal narrative begins.

---

## 2. Categorised Evidence

Category key, as requested:
**(a)** availability/maintenance · **(b)** semantic/correctness · **(c)** expressiveness · **(d)** operational · **(e)** pivot · **(f)** *incidental tooling* — added because a large share of the transcript's friction fits none of (a)–(e) and omitting it would misrepresent the record.

---

### (a) Availability / maintenance problems — 12 items

**A-1 — line 31, 153, vs. 286: requested Noria version does not exist; cargo silently resolved an older one.**
Cargo.toml asked for:
> `noria = "0.9" # Check for the latest compatible version`

but every compiler error names the resolved crate:
> `--> /Users/checolino/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/noria-0.6.1/src/controller.rs:62:19`

**A-2 — line 257–265: the published crate's dependency needs nightly.**
> ```
> error[E0554]: `#![feature]` may not be used on the stable release channel
>  --> /Users/checolino/.cargo/registry/.../assert_infrequent-0.1.0/src/lib.rs:1:1
>   |
> 1 | #![feature(test)]
> ...
> error: could not compile `assert_infrequent` (lib) due to 1 previous error
> ```
Assistant's reading (line 267):
> `This error occurs because Noria relies on older dependencies that use night-only Rust features (like #![feature(test)]inside the assert_infrequent crate), but you are currently running the stable Rust toolchain.`

**A-3 — lines 284–329: Noria uses an unstable feature form modern nightly rejects (5 sites).**
> ```
> error[E0658]: `impl Trait` in associated types is unstable
>   --> .../noria-0.6.1/src/controller.rs:62:19
>    |
> 62 |     type Future = impl Future<Output = Result<Self::Response, Self::Error>> + Send;
>    = note: see issue #63063 <https://github.com/rust-lang/rust/issues/63063> for more information
>    = help: add `#![feature(impl_trait_in_assoc_type)]` to the crate attributes to enable
>    = note: this compiler was built on 2026-08-06; consider upgrading it if it is out of date
> ```
Same error at `table.rs:218`, `table.rs:589`, `view.rs:45`, `view.rs:245`.

**A-4 — lines 330–356: unconstrained opaque types — not fixable by a feature flag.**
> ```
> error: unconstrained opaque type
>    --> .../noria-0.6.1/src/controller.rs:179:24
>     |
> 179 | type RpcFuture<A, R> = impl Future<Output = Result<R, failure::Error>>;
>     = note: `RpcFuture` must be used in combination with a concrete type within the same crate
> ```
Plus `view.rs:89` (`type Discover = impl tower_discover::Discover<...>`) and `table.rs:264`.

**A-5 — lines 357–436: three `E0308: mismatched types` inside Noria arising from the same opaque-type drift.**
> ```
> error[E0308]: mismatched types
>    --> .../noria-0.6.1/src/controller.rs:370:9
> 370 |         finalize(fut, err)
>     |         ^^^^^^^^^^^^^^^^^^ expected future, found a different future
>     = note: distinct uses of `impl Trait` result in different opaque types
> ```
> ```
> note: this item must have a `#[define_opaque(table::Discover)]` attribute to be able to define hidden types
>    --> .../noria-0.6.1/src/table.rs:258:4
> ```
Terminating line 436:
> `error: could not compile `noria` (lib) due to 11 previous errors; 11 warnings emitted`

**A-6 — line 439, assistant's explicit diagnosis: dead upstream.**
> `The Noria project hasn't been actively maintained to keep up with the latest Rust nightly breaking changes.`

**A-7 — line 507: pinning an old toolchain judged unreliable.**
> `The errors you saw earlier (unconstrained opaque type and mismatched types) indicate that Noria's specific use of unstable features is fundamentally broken on modern Rust, and attempting to downgrade the toolchain far enough back to make it work is highly unreliable.`

**A-8 — lines 546–578: native dependency (`librocksdb-sys`) panics under modern LLVM.**
> ```
> error: failed to run custom build command for `librocksdb-sys v6.20.3`
> Caused by:
>   process didn't exit successfully: .../build_script_build (exit status: 101)
>   --- stderr
>   thread 'main' (1986895) panicked at .../bindgen-0.59.2/src/ir/context.rs:878:9:
>   "enum_(unnamed_at_rocksdb/include/rocksdb/c_h_854_1)" is not a valid Ident
> ```
Assistant's diagnosis (line 580):
> `LLVM 16+ changed how it names unnamed C-enums, and the older bindgen (v0.59) used by librocksdb-sys v6.20.3 parses this new name into an invalid Rust identifier`

**A-9 — line 544: assistant labels Noria unmaintained and first floats Materialize.**
> `Noria is powerful but unmaintained; if patching becomes too cumbersome for this project, you may want to consider a modern alternative like Materialize (which is also written in Rust and built on similar dynamic dataflow principles).`

**A-10 — lines 2071–2074: no published Docker image; the assistant had invented one.**
> ```
>  ✘ Image mitpdos/noria:latest Error manifest for mitpdos/noria:latest not found: manifest unknown: manifest unknown
> Error response from daemon: manifest for mitpdos/noria:latest not found: manifest unknown: manifest unknown
> ```
> `That error is entirely my fault. I assumed the MIT team published a pre-compiled Docker image to Docker Hub under mitpdos/noria:latest, but they never actually did—they only published the source code.`

**A-11 — lines 2356–2367: the terminal Noria failure. Pinned nightly in a clean Ubuntu container still fails, because Noria's dependency tree is not itself pinned.**
> ```
>  => [3/6] RUN curl ... | sh -s -- -y --default-toolchain nightly-2021-01-01    14.2s
>  => [6/6] RUN cargo build --release --bin noria-server                         87.4s
>  => => #     = help: add `#![feature(impl_trait_in_assoc_type)]` to the crate attributes to enable
>  => => #     = note: this compiler was built on 2026-08-06; consider upgrading it if it is out of date
>  => => # error: could not compile `tokio-tower` (lib) due to 4 previous errors
> ```
Assistant (lines 2366–2367):
> `even when we use the specific older Rust compiler (nightly-2021-01-01) inside an isolated Linux environment, Noria's dependencies are fetching newer patch versions of crates (like tokio-tower) that are incompatible with that old compiler.`
> `The project is simply too unmaintained to compile cleanly without manually patching a dozen transitive dependencies, which is a nightmare for a new project.`

*(Adversarial note: the container installed the pinned toolchain but there is no `--locked` and no `Cargo.lock` respected; the failure is arguably a self-inflicted build-hygiene error as much as an upstream one. See Section 8.)*

**A-12 — lines 3452–3459: the replacement engine also has availability gaps — old Docker tags purged.**
> ```
>  ✘ Image materialize/materialized:v0.27.1 Error manifest for materialize/materialized:v0.27.1 not found: manifest unknown: manifest unknown
> ```
> `Materialize regularly purges older tags from Docker Hub, which is why v0.27.1 returned "manifest unknown."`

---

### (b) Semantic / correctness problems — 9 items

Read this section carefully: **only B-1 is an engine refusal actually observed at runtime, and it came from Materialize, not Noria.** Items B-2 through B-9 are defects *in the code the assistant produced*, not refusals by any engine. They are evidence about what an engine/language failed to prevent, which is arguably the more relevant evidence for a thesis about static checkability — but it is a different kind of evidence and must not be conflated.

**B-1 — line 2622 (user-pasted): the streaming engine refuses key/uniqueness constraints outright.**
> ```
> 2026-08-08T00:44:25.573307Z  INFO backend: Starting GBS Core Banking Backend...
> Error: error returned from database: CREATE TABLE with a primary key or unique constraint is not supported
> Caused by:
>     CREATE TABLE with a primary key or unique constraint is not supported
> ```
Assistant's explanation and remedy (lines 2657, 2664):
> `Materialize is a streaming database, not a traditional relational database. Because it processes continuous streams of data, it does not support PRIMARY KEY or UNIQUE constraints on base tables. We just need to remove that constraint.`
> ```
>    // 1. Create the immutable double-entry ledger without the PRIMARY KEY constraint
>     sqlx::query(
>         "CREATE TABLE IF NOT EXISTS ledger (
>             id BIGINT,
>             account_id BIGINT,
>             amount BIGINT
>         );"
> ```
This is the single strongest piece of semantic evidence in the transcript: the ledger's identity/idempotency key was removed from the schema because the engine could not express it, and it was never replaced by anything else.

**B-2 — line 4313 vs. line 2664: the code claims to preserve a primary key that no longer exists.**
> ```
>     // We increment the transaction ID slightly so the primary key remains unique
>     sqlx::query(
>         "INSERT INTO public.ledger (id, account_id, amount) VALUES ($1, $2, $3)"
>     )
>     .bind(transaction_id + 1)
> ```
There is no primary key on `public.ledger` (B-1). The uniqueness the comment asserts is unenforced by the engine and is not, in fact, guaranteed by the generator either — see B-3.

**B-3 — lines 4297–4301: ledger IDs are millisecond wall-clock timestamps, collision-prone under concurrency.**
> ```
>     // Generate a unique transaction ID (for a real system, use UUID or a sequence)
>     let transaction_id = std::time::SystemTime::now()
>         .duration_since(std::time::UNIX_EPOCH)
>         .unwrap()
>         .as_millis() as i64;
> ```
Two concurrent transfers in the same millisecond produce colliding `id` and `id+1` values. The `.unwrap()` also panics the handler on clock skew before the epoch. The comment itself concedes the design is not production-grade.

**B-4 — lines 810–818: earlier iteration used random IDs *and discarded every insert error*.**
> ```
>     // 1. Debit the sender
>     let _ = sqlx::query("INSERT INTO ledger (id, account_id, amount) VALUES (?, ?, ?)")
>         .bind(rand::random::<i64>().abs()) // Dummy ID generator for demo
>         .bind(payload.from_account)
>         .bind(-payload.amount_cents)
>         .execute(&state.db)
>         .await;
> ```
`let _ =` on both legs means a failed debit and a succeeded credit are indistinguishable from success, and the endpoint returns `"Transfer Processed"` (line 824) unconditionally. This is a money-creating failure mode written directly into the sample code.

**B-5 — line 807: atomicity explicitly deferred to "a real banking app".**
> ```
>     // In a real banking app, you would wrap this in a SQL transaction
>     // and generate secure UUIDs/Snowflake IDs for the ledger rows.
> ```

**B-6 — lines 838–839: a failed balance read is reported to the caller as a zero balance.**
> ```
>     let balance_cents = match row_result {
>         Ok(row) => row.try_get("balance").unwrap_or(0),
>         Err(_) => 0, // Account not found or zero balance
>     };
> ```
Repeated verbatim at 1174–1175 and 1501–1502. "Account not found", "view not yet materialized", "connection dropped", and "genuinely zero" all collapse to `0`. In a banking read path this is the canonical dangerous default.

**B-7 — no balance or authorization check anywhere.** `process_transfer` (lines 4289–4335, restated 4394–4434) never reads `account_balances` before debiting, never checks account existence, and never consults any limit. Overdraft is structurally unpreventable in the delivered design; nothing in the transcript notes this.

**B-8 — no currency anywhere.** No schema in the transcript has a currency column, and no code path carries a currency. `amount` is a bare `BIGINT`. Cross-currency mismatch is not merely unchecked, it is unrepresentable — the system silently treats all money as fungible.

**B-9 — no audit metadata, no bitemporality, no idempotency key.** `created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP` appears exactly once, in the first Noria schema (line 755), and is dropped from every subsequent iteration. No actor, no reason code, no request-id, no valid-time/transaction-time split, no hash chain, no versioning appears anywhere. "Immutability" is asserted in prose fourteen times (e.g. lines 735, 1602, 2447, 2719) but is enforced by nothing: the table is an ordinary mutable table that the application merely refrains from updating.

---

### (c) Expressiveness problems — 4 items

**C-1 — line 2622 (again, as an expressiveness fact).** The engine's SQL surface cannot express a uniqueness constraint at all. There is no alternative syntax offered; the only remedy is deletion of the requirement.

**C-2 — line 775: the compile-time-checked query path is unusable against a non-standard dialect.**
> `Tip: We use the standard sqlx::query("...") function rather than the sqlx::query!("...") macro. The macro requires a live database connection at compile-time, which can act unpredictably with Noria's custom SQL dialect.`
Consequence: **all** SQL in the project is untyped strings checked only at runtime. Every column name and type binding is unverified.

**C-3 — lines 765–766: uncertainty about basic DDL idempotency in the dialect.**
> ```
>     // NOTE: Noria might throw an error if the view already exists,
>     // so in a production app you'd check for its existence first.
> ```
Handled by `let _ = sqlx::query(...)` (line 767) — i.e. by discarding the result.

**C-4 — lines 4525, 4538 (user-supplied proposal text, near the end): the expressiveness gap stated as a thesis premise.**
> `Standard SQL has no vocabulary for streaming semantics (like lateness or watermarks) or materialization boundaries.`
> `Standard SQL hides state; your language must expose it. The language should allow a developer to define dataflow topologies where eviction policies (e.g., EVICT LRU(10GB)), upquery behaviors, and consistency requirements (e.g., REQUIRE STRICT_SERIALIZABLE) are native keywords, not afterthoughts.`
**This is a claim, not an observation.** It appears in a proposal the user pasted in, after the build attempt ended. It is not evidence produced by the attempt.

---

### (d) Operational problems — 8 items

**D-1 — lines 63–68: heavyweight external coordination dependency (ZooKeeper) baked into the client API.**
> ```
>     // Noria relies on ZooKeeper to manage its cluster state.
>     // By default, ZK runs on 127.0.0.1:2181 locally.
>     let zookeeper_addr = "127.0.0.1:2181";
>     let db = ControllerHandle::from_zk(zookeeper_addr).await?;
> ```
A `zookeeper:3.8` container is carried in every docker-compose through line 2200.

**D-2 — line 2131: source-build deployment model measured in tens of minutes.**
> `Important Timing Note: Because Docker is pulling down Ubuntu, installing LLVM/Clang, and compiling the massive Noria codebase from source on a 2021 Rust compiler, Step 5 will take roughly 10 to 20 minutes to complete.`

**D-3 — line 3047 / 3261: RBAC lockout on the default user (observed twice).**
> ```
> Error: error returned from database: permission denied for SCHEMA "materialize.public"
> ```

**D-4 — line 3871: the documented superuser escape hatch is itself blocked.**
> ```
> Error: error returned from database: unauthorized login to user 'mz_system'
> ```

**D-5 — line 3942: schema workaround also refused.**
> ```
> Error: error returned from database: permission denied for DATABASE "materialize"
> ```

**D-6 — lines 3946–3947: the operational fix is an undocumented internal admin port.**
> `To run database migrations and create tables locally, Materialize exposes a hidden internal admin port on 6877specifically for the mz_system user.`
Resulting production-shaped smell: the application's normal boot path connects as the cluster superuser over an internal port (line 3980: `postgres://mz_system@127.0.0.1:6877/materialize`).

**D-7 — line 3459: version pinning is unreliable because upstream deletes tags.** (Also A-12; it is both.)

**D-8 — no durability, recovery, backup, or observability question is ever raised in the entire transcript.** There is no `docker volume`, no persistent mount, no restart test, no snapshot, no metrics, no tracing beyond `tracing_subscriber::fmt::init()`. This is an *absence*, not a failure — worth recording precisely because a thesis about durability and recovery cannot cite this transcript as evidence about them.

---

### (e) Pivots — 11 items

Each is a moment where the plan changed to route around a limitation.

| # | Line | Trigger | Pivot |
|---|---|---|---|
| **P-1** | 267–273 | `E0554` on stable | stable Rust → `rustup override set nightly` |
| **P-2** | 440–449 | opaque-type errors on modern nightly | nightly → pinned `nightly-2021-01-01` |
| **P-3** | 462–545 | pinning judged unreliable | vendor and hand-patch Noria via `[patch.crates-io] noria = { path = "../noria/noria" }`, editing `#![feature(impl_trait_in_assoc_type)]` → `#![feature(type_alias_impl_trait)]` |
| **P-4** | 582–600 | `librocksdb-sys` bindgen panic | bump `rocksdb` inside the fork, or force `bindgen = { version = "=0.62.0" }` globally |
| **P-5** | **611–620** | everything above | **Abandon the Rust client library entirely.** Talk to Noria as a MySQL server via `sqlx` on stable Rust, in Docker |
| **P-6** | 2074–2110 | no published image | write a custom Dockerfile that builds `noria-server` from source in `ubuntu:20.04` |
| **P-7** | **2366–2375** | `tokio-tower` fails even in the pinned container | **Abandon Noria. Adopt Materialize** (`materialize/materialized:v0.68.0`) |
| **P-8** | 2657–2670 | `PRIMARY KEY … not supported` | **delete the primary key from the ledger schema** |
| **P-9** | 3192, 3875, 3947 | RBAC refusals | `public.` prefix → `mz_system` superuser → dedicated `gbs` schema → **internal admin port 6877 as `mz_system`** |
| **P-10** | 3459–3465 | `v0.27.1` tag purged | revert to `v0.68.0` and work around its RBAC instead |
| **P-11** | **4471+** | the whole preceding history | **Abandon reuse. Build a new language and a new DBMS from scratch** (the thesis) |

Bold rows are the three architecture-level pivots.

P-5's rationale, quoted in full (lines 612–613), is the clearest single statement of why the client library was dropped:
> `The noria Rust crate is broken on modern macOS because its C-bindings (bindgen and rocksdb) conflict with Apple's current LLVM compiler, and it requires an outdated Nightly Rust toolchain.`
> `The Solution: Noria natively speaks the MySQL wire protocol. Instead of compiling the broken Noria Rust client, we will run Noria and MySQL in isolated Docker containers, and write your modern Rust server using Stable Rust and sqlx(a pure-Rust async SQL crate). This gives you Noria's high-performance materialized views without any of the compilation nightmares.`

Note also the **eleven "start over" restarts** (lines 610, 1515, 1770, 2133, 2445, 2718, 3280 — the user's own repeated formula):
> `Let's start over to avoid getting the same errors. We will delete the previous broken attempts.`

Each restart begins with `rm -rf GBS`. No version control is ever used; every restart destroys the prior state.

---

### (f) Incidental tooling friction — 10 items (not attributable to any database)

This category is not in the original request but is necessary for an honest count.

**F-1 — line 233–236: IDE/cargo case-sensitivity.**
> ```
> error: package ID specification `GBS` did not match any packages
> help: a package with a similar name exists: `gbs`
> ```

**F-2 — lines 1746–1753: `trunk` will not build from source on the user's stable toolchain.**
> ```
> Some errors have detailed explanations: E0053, E0277, E0308.
> error: could not compile `lightningcss` (lib) due to 61 previous errors
> error: failed to compile `trunk v0.21.14`, intermediate artifacts can be found at `/var/folders/xg/...`
> ```

**F-3 — line 1961: Docker not installed.**
> `zsh: command not found: docker-compose`

**F-4 — lines 1979–1983, 2013–2016: Homebrew binary collisions from a prior Docker install.**
> `Error: It seems there is already a Binary at '/usr/local/bin/docker-credential-osxkeychain'.`
> `Error: It seems there is already a Binary at '/usr/local/bin/kubectl.docker'.`

**F-5 — line 2048: Docker Desktop hangs.**
> `You can't open the application "Docker.app" because it is not responding.`

**F-6 — line 2072: daemon not running.**
> `unable to get image 'mitpdos/noria:latest': failed to connect to the docker API at unix:///var/run/docker.sock; check if the path is correct and if the daemon is running: dial unix /var/run/docker.sock: connect: no such file or directory`

**F-7 — the chat-interface angle-bracket bug: at least six separate exchanges, ~15 compile errors.** This is, by error count, the single largest source of failure in the transcript.
Lines 863–866:
> ```
> error: expected parameter name, found `>`
>   --> backend/src/accounting.rs:29:32
>    |
> 29 |     Extension(state): Extension>,
> ```
Line 1010:
> ```
> 37 |         .bind(rand::random::().abs()) // Dummy ID generator for demo
>    |               ^^^^ use of unresolved module or unlinked crate `rand`
> ```
Line 3164:
> ```
> 40 |     yew::Renderer::::new().render();
>    |                    ^^ expected identifier
> ```
Line 4077:
> ```
> error: this opening fragment has no corresponding closing fragment
>   --> frontend/src/main.rs:27:9
> 27 |         <>
> ```
Assistant's admissions, lines 1441, 3189, 4169:
> `My apologies for the confusion! I am trying to use angle brackets < > for generic types, but my response format is accidentally parsing them out of the code block.`
> `I apologize. The chat interface is aggressively stripping out the angle brackets (< and >) from my code blocks, causing the generic syntax in the Rust code to become invalid.`
> `This error confirms that the chat interface you are using is aggressively censoring or deleting the closing </> tag from my code blocks before you even see them`
Confirmed empirically at lines 4192–4224, where the user pastes back their actual file and every HTML tag is gone:
> ```
>     html! {
>         <>
>                 { "Global Banking System (GBS)" }
>                     { "Ping Backend" }
>                     { "System Status: " } { (*api_response).clone() }
>     }
> ```

**F-8 — line 3273: Trunk requires a `<body>` (a casualty of F-7).**
> `2: Document has neither a <link data-trunk rel="rust"/> nor a <body>. Either one must be present.`

**F-9 — line 4032: `reqwest` under WASM requires absolute URLs.**
> `System Status: Error: builder error: relative URL without a base`

**F-10 — line 4052: CORS / origin mismatch between `localhost` and `127.0.0.1`.**
> `System Status: Error: error sending request: JsValue(TypeError: Load failed undefined)`

---

## 3. Verbatim Error / Version Inventory

Every distinct error identifier and version number in the transcript, for citation.

**Rust compiler errors:** `E0554`, `E0658` (×5 in Noria, ×4 in `tokio-tower`), `E0308` (×3 in Noria), `E0107` (×6 in the user's code), `E0214` (×2), `E0433` (×2), `E0277` (×2), `E0599` (×2), `E0412`, `E0053`.

**Panics:**
- `"enum_(unnamed_at_rocksdb/include/rocksdb/c_h_854_1)" is not a valid Ident` — `bindgen-0.59.2/src/ir/context.rs:878:9` (line 552)
- backtrace frames `proc_macro2::fallback::validate_ident` → `bindgen::ir::enum_ty::Enum as bindgen::codegen::CodeGenerator>::codegen` → `build_script_build::bindgen_rocksdb` (lines 560–580)

**Database errors (all runtime, all Materialize):**
- `CREATE TABLE with a primary key or unique constraint is not supported` (2622)
- `permission denied for SCHEMA "materialize.public"` (3047, 3261)
- `unauthorized login to user 'mz_system'` (3871)
- `permission denied for DATABASE "materialize"` (3942)

**Registry / distribution errors:**
- `manifest for mitpdos/noria:latest not found: manifest unknown: manifest unknown` (2071, 2073)
- `manifest for materialize/materialized:v0.27.1 not found: manifest unknown: manifest unknown` (3454, 3455)

**Build failures:**
- `error: could not compile `assert_infrequent` (lib) due to 1 previous error` (264)
- `error: could not compile `noria` (lib) due to 11 previous errors; 11 warnings emitted` (436)
- `error: could not compile `tokio-tower` (lib) due to 4 previous errors` (2364)
- `error: could not compile `lightningcss` (lib) due to 61 previous errors` (1750)
- `error: could not compile `backend` (bin "backend") due to 13 previous errors; 4 warnings emitted` (1046)
- `error: could not compile `backend` (bin "backend") due to 17 previous errors; 3 warnings emitted` (1436)

**Versions named:** `noria = "0.9"` (requested) / `noria-0.6.1` (resolved); `librocksdb-sys v6.20.3`; `bindgen-0.59.2`, proposed `bindgen = "=0.62.0"`; `rocksdb "0.19.0"` → proposed `"0.21.0"`; `nightly-2021-01-01` (also `nightly-2020-06-04`, `nightly-2020-10-01`, `nightly-2019-11-01` floated at line 452); rustc "built on 2026-08-06"; `assert_infrequent-0.1.0`; `axum-0.6.20`; `sqlx 0.7.4` (`sqlx-postgres v0.7.4` flagged future-incompat); `tokio 1.53.1`; `yew 0.20.0`; `rand-0.8.7`; `trunk 0.21.14`; `zookeeper:3.8`; `mysql:8.0`; `ubuntu:20.04`; `materialize/materialized:v0.68.0` and `:v0.27.1`; Docker Desktop `4.85.0,235549`; `wasm-bindgen 0.2.126`; `reqwest 0.11.27`.

**Missing / non-existent APIs and artifacts:**
- `mitpdos/noria:latest` Docker image — **never existed** (A-10)
- `materialize/materialized:v0.27.1` — purged
- Noria crate version `0.9` — not resolvable
- `#[define_opaque(table::Discover)]` — attribute modern rustc demands that Noria's source predates (line 397)

**Timestamps:** all runtime logs are dated `2026-08-08T00:44` through `2026-08-08T01:37` — the entire session's runtime phase spans roughly **53 minutes**.

---

## 4. The User's Own Voice

User turns are short, imperative, lowercase-leaning, and typo-bearing; assistant turns are long, headed, and code-heavy. Below is every substantive user statement of intent or diagnosis. This is the primary evidence.

**Statements of what they wanted:**

- Line 1 (retrospective framing): `I had thought about creating a core baking system using Noria, but it wasn't production ready, here's what happened.` — note the typo "baking" for "banking", and note that this is the *only* place the user characterises the outcome.
- Line 2: `I want to create a core banking system using the rust programming language, the Noria database system`
- Line 8: `I want to use a web browser as the GUI, I need a WASM integration for best speed, but the logic must be run on the server using Rust`
- Line 118: `remake the files taking into account the project is called GBS and is saved in /Users/checolino/RustroverProjects/GBS`
- Line 610 (and the same formula at 1515, 1770, 2133, 2445, 2718, 3280): `Let's start over to avoid getting the same errors. … Reminder: we are creating a core banking system using the rust programming language for the server logic, the Noria database system and MySQL, WASM for GUI.`
- Line 2445 — the moment the goal name changes and Noria is dropped by the *user*: `We are going to use Materialize instead of Noria. … we are creating a core banking system called GBS (Global Banking System)`
- Line 4471: `I have this proposal for a thesis, I want to include building a dedicated database language and database management system.`

**Statements about what went wrong** (near-universally the user pasting a terminal, without commentary):

- Line 233: `I got this error`
- Line 506: `give me step by step instructions, the last code I ran was rustup override set nightly-2021-01-01`
- Line 545: `I went into /Users/checolino/RustroverProjects/noria/noria/src/lib.rs and got this error`
- Lines 1746–1747: `while trying to run "rustup target add wasm32-unknown-unknown / cargo install trunk" I got this error`
- Line 1960: `While trying to run docker-compose up I got the following error`
- Line 2349: `When I ran docker compose up --build on the first terminal I got the following error`
- Line 3255 / 3264: the user pastes the two terminals **swapped** (`I got this message in the front end terminal:` followed by backend output) — a small sign of fatigue at this point in the session
- Line 3631: `I got a warning in lines 30 to 33 in the file rontend/src/main.rs` (typo "rontend")
- Line 4031 / 4051: `When I pinged the backend I got this message`
- Line 4192: `here's what I have in frontend/src/main.rs, output the new text of this whole file`
- Line 4338: `here's what I have, output the updated text of this whole file`

**Questions the user asked in their own voice — these reveal what they thought mattered:**

- Line 462: `How do I use the [patch.crates-io] feature in Cargo.toml to fix a broken dependency like Noria?`
- Line 733: `How do I set up the SQL schemas for accounts and transactions, and create the Noria materialized views via SQLx?`
- Line 1633: `How do I configure the frontend crate to compile WebAssembly and communicate with this backend API?`
- Line 4270: `How do I add the POST /api/transfer endpoint in Axum to write double-entry transactions to Materialize?`

**What the user never says, anywhere in 4,565 lines:** they never complain about consistency, staleness, transactions, serializability, recovery, durability, sharding, auditability, currency handling, or view semantics. Not once. Every user-voiced complaint is a build error, an install error, or a paste error. The semantic critique in this transcript is entirely the assistant's, or comes from the pasted proposal at line 4471+.

---

## 5. Workarounds Proposed by the Assistant

Specifically, the pattern of "reimplement the missing guarantee in application code". This is the most relevant evidence for a thesis about what a language or engine ought to provide.

**W-1 — line 109: double-entry balance is an application-layer comment.**
> ```
>     // 1. Validate the transaction (e.g., double-entry rules: debits == credits)
>     // 2. Obtain a Noria mutator: state.db.mutator("transactions").await;
>     // 3. Write to the base table
> ```
The invariant is described as a TODO in a handler. It is never implemented anywhere in the transcript.

**W-2 — line 115: the guarantee is *architectural placement*, not enforcement.**
> `Absolute Server Authority: Because the axum API handles all Noria mutators, the WASM client cannot bypass business rules (like ensuring double-entry accounting balances out). The WASM frontend only sends intents`
The claim is that putting logic on the server makes it authoritative. It does not make it *checked* — nothing in the system verifies that debits equal credits, on either side.

**W-3 — line 807: atomicity deferred.**
> ```
>     // In a real banking app, you would wrap this in a SQL transaction
>     // and generate secure UUIDs/Snowflake IDs for the ledger rows.
> ```

**W-4 — lines 811, 818: identity synthesised in application code with no uniqueness guarantee.**
> `.bind(rand::random::<i64>().abs()) // Dummy ID generator for demo`

**W-5 — lines 4297–4313: after the engine refused PRIMARY KEY, uniqueness is reimplemented as arithmetic.**
> ```
>     // Generate a unique transaction ID (for a real system, use UUID or a sequence)
>     let transaction_id = std::time::SystemTime::now() … .as_millis() as i64;
> …
>     // We increment the transaction ID slightly so the primary key remains unique
>     .bind(transaction_id + 1)
> ```
This is the purest instance of the pattern: the database said "I cannot express uniqueness"; the application answered "then I will add one to a timestamp."

**W-6 — lines 4271–4272: atomicity reimplemented as an application-held SQL transaction.**
> `To implement a robust double-entry transfer, we must enforce strict audit constraints: every transfer must insert two corresponding rows (a debit and a credit) atomically. If one fails, the entire transaction must roll back.`
> `In Materialize (which uses Postgres drivers), we achieve this by wrapping the inserts in a SQL TRANSACTION.`
Note what is asserted and not verified: that a streaming engine which refuses primary keys nonetheless provides the multi-statement transactional rollback the code depends on. This was **never tested** — the endpoint never ran.

**W-7 — lines 838–839: absence reimplemented as zero.**
> `Err(_) => 0, // Account not found or zero balance`

**W-8 — line 767, 1612, 2795, 3994 etc.: DDL errors swallowed with `let _ =`** because the assistant was unsure of the dialect's idempotency (C-3).

**W-9 — line 3875: authorization reimplemented as "create a schema you own".**
> `Since that user is locked out of the system's publicschema, we will simply instruct Rust to create a dedicated gbs schema first, which our user will own and have full permissions over.`

**W-10 — lines 3947, 3980: operate as cluster superuser over an internal port.**
> `Materialize exposes a hidden internal admin port on 6877specifically for the mz_system user.` … `let database_url = "postgres://mz_system@127.0.0.1:6877/materialize";`
The application's steady-state connection becomes a superuser connection on an internal admin port — an operational workaround with real security consequence, adopted without comment.

**W-11 — lines 461, 527–531, 582–600: fork and hand-patch the dependency.**
> `you may have to fork the Noria crate and change the feature flag at the top of their src/lib.rs from #![feature(impl_trait_in_assoc_type)] to #![feature(type_alias_impl_trait)]`
> `Note: You may need to perform a global search (Cmd+Shift+F in RustRover) for impl_trait_in_assoc_type across the Noria repository and update it in multiple lib.rs files.`
> `you must force cargo to use a newer bindgen specifically for the old librocksdb-sys.`

**Composite reading.** Between W-1, W-3, W-4, W-5, W-6 and W-7, every property a core banking ledger needs — atomicity, identity/idempotency, conservation, and honest absence — is either (i) written as a comment describing what a real system would do, (ii) synthesised in application code with no enforcement, or (iii) assumed of an engine that had just refused a weaker version of the same guarantee. That composite is the transcript's most substantive contribution to the thesis's argument, and it is an argument about *what an engine failed to make checkable*, not about *what an engine did wrong at runtime*.

---

## 6. Correct Things the Transcript Did (which the thesis should not claim otherwise)

Being adversarial requires recording what went *right*.

**R-1 — Money was never a float.** From the very first schema (line 738):
> `We will use a standard double-entry ledger format. Money is represented in cents (as BIGINT) to prevent floating-point rounding errors.`
Restated at line 4284: `pub amount: i64, // Represented in cents to avoid floating-point errors`.
If the thesis anywhere lists "money represented as float" as a failure mode this attempt hit, that is **not supported** — the opposite is true.

**R-2 — The append-only / derived-balance discipline was chosen deliberately and correctly** (line 735):
> `In core banking, you should never store an account balance as a static, updateable number. Instead, you use an immutable ledger (append-only debits and credits) and calculate the balance on the fly.`

**R-3 — The incremental-view value proposition was understood and articulated** (line 116):
> `In banking, calculating end-of-day balances or cashflow aggregations across millions of rows is expensive. With Noria, you will write a SQL materialized view like SELECT account_id, SUM(amount) FROM ledger GROUP BY account_id. Noria pre-computes this and updates it incrementally as new transactions arrive. When your WASM GUI requests an account balance, the Rust server reads it in O(1) time.`

**R-4 — Multi-statement transactions were eventually used, not avoided** (line 4293):
> `    // We explicitly use a database transaction to ensure atomicity`
`state.db.begin()` … `tx.commit()`. Whether it would have *worked* was never established, but the design did reach for the right primitive.

**R-5 — Materialize accepted `CREATE MATERIALIZED VIEW` on a `SUM … GROUP BY` without complaint.** No expressiveness refusal was ever recorded for the view itself. The only refusals were on constraints and permissions.

---

## 7. Facts That Do Not Appear (absences, not evidence)

The following are things a reader might assume the transcript shows. It does not.

1. **Noria never ran.** No Noria process ever started. No connection was ever made. No query was ever issued to Noria.
2. **No stale read, inconsistent read, or eventual-consistency anomaly was ever observed** — on Noria or on Materialize.
3. **No transaction anomaly, lost update, or torn double-entry was ever observed.** The transfer endpoint never executed.
4. **No upquery, eviction, or partial-state behaviour was ever observed.**
5. **No restart, crash, recovery, or durability test was ever performed.** No persistent volume was configured.
6. **No benchmark, throughput number, latency number, or memory measurement appears anywhere.**
7. **No sharding or multi-node deployment was attempted.**
8. **No comparison to a conventional RDBMS (Postgres/MySQL) baseline was ever run.** MySQL 8.0 appears in one docker-compose (line 681) purely as a misunderstanding of Noria's architecture, corrected at line 2075, and was never used.
9. **The proposal text at lines 4471–4565 is not evidence from the attempt.** It cites the Noria OSDI'18 paper's own stated limitations (`multi-statement transactions, which the original paper explicitly deferred and never delivered`, line 4496; `Range indexes, multi-column joins, non-primary-key predicate updates/deletes — the specific holes the original paper named`, line 4500). These are citations to *published literature*, arriving after and independent of the build attempt. They may be true and thesis-relevant, but they are not findings of this transcript, and citing them as such would be a category error.

---

## 8. Adversarial Reading: What Cuts Against the Thesis

Presented as strongly as the evidence allows.

**8.1 — The Noria failure was 100% packaging, 0% semantics.** Every Noria error in this transcript is `rustc` or `cargo` or `docker`. The thesis's interesting claims — about partial state, eviction, upqueries, consistency ladders, strict serializability, conservation of money — are untouched by a single line of this transcript. A hostile reviewer would say: *you did not fail to build a banking system on Noria; you failed to build Noria.*

**8.2 — The build failure is at least partly self-inflicted.** The Ubuntu container (lines 2160–2180) installs `nightly-2021-01-01` and then runs a bare `cargo build --release`. It does not `git checkout` a known-good commit, does not use `--locked`, and does not respect any `Cargo.lock`. The `tokio-tower` failure at line 2364 is exactly the failure mode of an unlocked dependency resolution — newer semver-compatible patch releases of transitive deps being pulled in. A competent build engineer would try `--locked` next. The transcript never does. The assistant's conclusion at line 2367 (`too unmaintained to compile cleanly without manually patching a dozen transitive dependencies`) is plausible but **untested**: no attempt at a locked build was ever made. This weakens even the availability claim.

**8.3 — Materialize's PRIMARY KEY refusal is the strongest semantic datum, and it is not about Noria.** B-1 is real, and it is genuinely on-thesis: an IVM engine's SQL surface could not express the ledger's identity constraint, and the response was to delete the requirement. But it is evidence about *Materialize*, a fully-materialized system. It cannot be used to argue anything about partial state or eviction.

**8.4 — The largest error category is a chat-UI bug.** F-7 alone accounts for more compile errors than Noria and Materialize combined. Several of the eleven "start over" restarts (line 610 onward) were triggered by accumulated damage from stripped angle brackets, not by any database property. A reader who counts errors rather than reading them would conclude the attempt failed because of markdown escaping.

**8.5 — The attempt never got far enough to encounter the interesting problems.** The system's total functional surface at the end is: one table, one view, one health endpoint, one untested transfer endpoint, one button. The absence of currency (B-8), audit metadata (B-9), and overdraft checks (B-7) are not things any engine refused — they are things the attempt never got to. Their absence is evidence of *scope reached*, not of *engine limitation*.

**8.6 — The developer's own stated diagnosis is about production-readiness, not semantics.** Line 1: `it wasn't production ready`. That is a claim about maturity and operability. It is fully supported. It is a strictly weaker claim than "Noria's design is semantically inadequate for banking", and the transcript supports only the former.

**8.7 — A counterfactual the transcript cannot rule out.** Nothing here shows that a Postgres-backed GBS would have gone worse. Indeed the two hardest *runtime* blockers (no PRIMARY KEY, RBAC lockout) are things stock Postgres would not have produced. On the evidence in this file alone, "should have used Postgres" is a live and unrefuted hypothesis.

---

## 9. What This Does and Does Not Establish

### 9.1 Structural limits of this evidence

- **n = 1.** One developer, one machine (macOS ARM64, "MacBook-Pro", Homebrew, RustRover), one session. The runtime portion spans about 53 minutes of wall-clock timestamps.
- **AI-assisted throughout**, and the assistant made at least three consequential errors of its own: inventing a Docker image that never existed (line 2074, self-admitted), a wrong architectural model of Noria requiring a backing MySQL (corrected at line 2075: `Noria does not need a background MySQL database to connect to. Noria is the database!`), and recommending a Materialize tag that had been purged (line 3459). Some of the friction is the assistant's, not the ecosystem's.
- **No controlled comparison.** No baseline engine, no second developer, no second machine, no repeated trial. There is no way to separate "Noria is hard to build" from "this environment is hard to build in" from "this operator did not use `--locked`".
- **No version control.** Every restart is `rm -rf GBS`. There is no artefact to inspect, re-run, or bisect. Nothing here is reproducible.
- **Instrument contamination.** The chat interface silently corrupted the code being transferred (F-7), across at least six exchanges. A meaningful fraction of the failure record is an artefact of the medium.
- **Retrospective framing.** Line 1 was written after the fact, by someone who had by then already decided to write a thesis about building a replacement. The transcript was selected and submitted as evidence for a conclusion already held.

### 9.2 What the transcript **does** establish

1. **The `noria` crate as published on crates.io (`0.6.1`) does not build on a 2026 Rust toolchain, stable or nightly.** Directly evidenced: A-2 through A-5, eleven distinct compiler errors, quoted verbatim. This is solid.
2. **Its native dependency chain is also broken on modern LLVM.** A-8, a reproducible bindgen panic.
3. **There is no official Noria binary distribution.** A-10, confirmed by registry response and by the assistant's own admission of having invented the image name.
4. **Pinning an old toolchain does not by itself fix it**, at least not without also locking the dependency graph — which was not tried. A-11, with the caveat in 8.2.
5. **A competent-but-not-expert developer with AI assistance, over a full session, could not get Noria to start.** Fair as a statement about accessibility and onboarding cost.
6. **At least one modern IVM engine's SQL surface cannot express a uniqueness constraint on a base table, and the practical response was to delete the constraint from a financial ledger.** B-1 + P-8, verbatim. This is genuine, on-thesis, and citable — as a fact about Materialize.
7. **When engine guarantees are absent, application code fills the gap with unverified substitutes.** W-1 through W-7, with the timestamp-plus-one primary key (W-5) as the exemplary case. This is real evidence for the thesis's *motivating* claim — that these properties belong in the language/engine, statically checked — provided it is presented as an illustration of a failure mode, not as a measurement.

### 9.3 What the transcript **does not** establish

1. **Nothing about Noria's runtime semantics.** Consistency, staleness, partial state, upqueries, eviction, transaction support, recovery, sharding — no observation of any kind. Zero.
2. **Nothing about the cost or feasibility of strong consistency under partial materialization** — the actual research question.
3. **Nothing quantitative.** No latency, throughput, memory, or recovery-time number exists in the transcript.
4. **No demonstration that a conventional RDBMS would have failed.** No baseline was run. See 8.7.
5. **No demonstration that money was ever lost, double-spent, or mis-conserved.** The transfer path never executed.
6. **No demonstration that the Noria build was actually unfixable.** `--locked` was never attempted; a pinned upstream commit was never checked out; the vendored-fork path (P-3) was abandoned mid-way when a *different* error (rocksdb) appeared.

### 9.4 The verdict on the two candidate claims

> **Claim A: "Noria cannot be used in production for core banking."**
> **NOT SUPPORTED by this transcript.** The strong reading — that Noria's *semantics* (eventual consistency, partial state, no multi-statement transactions) disqualify it for banking — may well be true, and there is published literature to cite for it, but this transcript contains no evidence for it whatsoever. Noria never ran. A weaker version — "the published `noria` crate cannot be deployed to production today without forking, patching, and pinning an entire dependency tree" — *is* supported, though even that is undercut by 8.2, since the cheapest remedy was never tried.

> **Claim B: "This attempt did not reach production."**
> **FULLY SUPPORTED**, and in fact understated: this attempt did not reach a *running system* on any engine, including the replacement engine. Eleven restarts, three architectures, two databases, zero end-to-end successes.

### 9.5 Recommended honest framing for the thesis

Cite this transcript as **motivation and illustration**, never as **result**. Two framings it can carry without overreach:

- *"Onboarding cost as a production barrier."* An artefact whose only distribution channel is a source tree pinned to a 2019–2021 nightly, with unlocked transitive dependencies and a native C++ dependency broken by later LLVM releases, is not deployable regardless of the merits of its design. Evidence: A-1 to A-11. This is an argument about *research-artefact sustainability*, and it is one this thesis is entitled to make — it argues for building a maintained system, which is precisely what Nilestream proposes.
- *"Guarantees not provided by the engine reappear as unverified application code."* When the engine could not express uniqueness, the application answered with `transaction_id + 1`; when it could not be trusted for atomicity, the code wrote a comment saying a real system would use a transaction; when a read failed, the API returned a balance of zero. Evidence: B-1 to B-6, W-1 to W-7. This is the honest, well-supported version of the motivating argument for making these properties statically checkable in the language.

What the thesis must **not** do is claim this attempt tested Noria's consistency model, its partial-state machinery, or its suitability for banking workloads. It tested none of them. If a reviewer reads this transcript — and it should be assumed they will — any claim beyond the two framings above will not survive contact with it.

---

## Appendix: Category Counts

| Category | Count |
|---|---|
| (a) availability / maintenance | 12 |
| (b) semantic / correctness | 9 *(only 1 an observed engine refusal; 8 are defects in produced code)* |
| (c) expressiveness | 4 *(1 observed, 1 tooling-derived, 1 uncertainty, 1 asserted in the pasted proposal)* |
| (d) operational | 8 *(1 of which is a global absence rather than an incident)* |
| (e) pivot | 11 *(3 architecture-level; plus 7 full "start over" restarts)* |
| (f) incidental tooling *(added category)* | 10 *(one of which, the angle-bracket bug, produced ~15 distinct compile errors)* |
| **Total distinct items** | **54** |

Distinct engine-produced runtime errors across the entire transcript: **4**, all from Materialize (lines 2622, 3047, 3871, 3942).
Distinct engine-produced runtime errors from Noria: **0**.
