# Appendix B. The Niles Language: Complete Syntax Reference

Normative reference for Niles v0.1. The precedence rule of Section 6.22 governs every entry: Rust syntax first, SQL vocabulary second, novel forms last and never overloading inherited meaning.

## B.1 Design Principles

1. **SQL-first, Rust-fallback, novel-last**, applied per construct and auditable. Any construct SQL already expresses is written with SQL's keywords and shape; everything SQL cannot say borrows Rust's spelling; novel syntax is introduced only for what neither can express. The rule carries one exception: a SQL spelling is not reused where it would sacrifice determinism or efficiency the engine depends on — unbounded recursive CTEs are admitted only behind a guard, and `float` is not a legal money type. The rationale, and the reason this ordering was chosen over its inverse, is adjudicated in Appendix J.1.
2. **One IR.** All surfaces lower to the typed IR of Appendix D; surface constructs are notation, never semantics.
3. **Types carry the guarantees.** Money with its currency *and scale*, both time axes, consistency, materialization, lineage, retention and confidentiality live in the type system.
4. **Pipeline order.** Queries read top-to-bottom in dataflow order, never SQL's inverted clause order.
5. **No silent coercions.** No implicit currency conversion, no implicit scale change, no implicit consistency downgrade, no implicit declassification.
6. **Every derived value carries its anchor.** Reads return `Anchored<T>`.
7. **Determinism in query context.** No ambient wall clock, no unguarded recursion, no nondeterministic iteration order.

## B.2 Lexical Structure

**Character set.** UTF-8 source; identifiers use Unicode identifier classes as in Rust; keywords and operators are ASCII.

**Whitespace and comments.** Whitespace separates tokens. Comments: `// line`, `/* nested block */`, `/// doc`, `//! module doc`. SQL's `--` is accepted *only* inside `sql { … }` blocks.

**Identifiers.** Rust rules, with raw identifiers `r#serve` permitting any reserved word as a name. Convention: `snake_case` items, `CamelCase` types, `SCREAMING_SNAKE` constants.

**Literals.**

- Integers: `42`, `1_000_000`, `0xFF`, `0o77`, `0b1010`, with suffixes `42i64`, `7u32`.
- Floats: `3.14`, `6.02e23` — *banned in `Money` positions by type*, never merely by convention.
- Strings: `"…"` with Rust escapes; raw `r"…"`, `r#"…"#`; byte strings `b"…"`.
- Booleans `true`/`false`; unit `()`.
- **Money literals (novel):** `125.00 usd`, `9_990.50 mxn`, `1_000 jpy`, `12.345 kwd`. The suffix is a declared currency; the mantissa's fractional digits must match that currency's declared scale exactly — `12.345 kwd` is well-formed (scale 3), `1_000.00 jpy` is a compile error (scale 0), and `2.5 usd` is a compile error (scale 2 requires two digits or none: `2.50 usd`). This strictness exists because currency scales genuinely range 0–3 and some codes have no minor unit at all.
- **Temporal literals (novel):** instants `@2026-08-25T12:00:00Z`; dates `@2026-08-25`; **value dates** `v@2026-08-25`; epochs `#41209`; durations `3.epochs`, `250.ms`, `2.days`; valid-time intervals `@2026-01-01 ..= @2026-12-31`.
- **Idempotency key (novel):** `idem"payment-7f3a"`.

**Operators and punctuation.** The Rust set, plus novel `|>` (explicit pipeline stage) and `@` (temporal literal introducer and annotation sigil).

## B.3 Keywords

### B.3.1 SQL-Derived

`select from where group having order limit offset join left right full outer inner on union except intersect distinct as create table view index insert into values update delete begin commit rollback grant revoke primary key foreign references check default null and or not in exists between like case when then else end`

SQL-derived words are legal only in their assigned positions; `update` and `delete` are legal only against `table` data, never against a `ledger`.

### B.3.2 Rust-Derived

`fn let mut const static struct enum trait impl for while loop if else match return break continue mod use pub crate self Self super type where move ref in as dyn`

`unsafe`, `async` and `await` are reserved and rejected in v0.1. `where` serves both Rust's bound clause and the pipeline filter stage; positions are disjoint and the grammar disambiguates.

### B.3.3 Novel Niles Keywords

`base ledger posting txn hold resolve void expire schema serve consistency bounded monotonic read_your_writes snapshot serializable ledger_consistent materialize absent demand full spilled tiered budget freshness retain evictable pinned forever anchor as_of epoch valid_at value_date recorded_at bitemporal authorize capability idem window conserve currency scale fx leg confidential e2ee committed declassify lineage emit signal backfill upto guard measure explain reproduce impact`

## B.4 Type System

**Primitives.** `bool, i8…i128, u8…u128, f32, f64, Text, Bytes, ()`.

**Composites.** Tuples, arrays, slices, `Vec<T>`, `Option<T>`, `Result<T,E>`, structs, enums, shared references, and a `Json` type for semi-structured columns.

**Novel domain types.**

| Type | Meaning |
|---|---|
| `Money<CUR>` | Exact integer minor units at `CUR`'s declared scale. Closed under `+`, `−`, integer scaling. **No** operation joins distinct currencies. |
| `Currency` | Currency index kind; declared with its scale: `currency kwd { scale: 3 }`. |
| `Id<T>` | Phantom-indexed identifier. |
| `Epoch`, `Instant`, `Date`, `Duration`, `Interval<Valid>` | Temporal kinds. |
| `Signal<T>` | Epoch-indexed value (Section 6.8). |
| `Bitemporal<T>` | Value with both axes; `as_of(sys).valid_at(t)` projects. |
| `Posting`, `Debit<CUR>`, `Credit<CUR>` | Linear halves consumed exactly once. |
| `Hold<CUR>` | A reservation; resolvable exactly once by post/void/expire. |
| `Auth<E>` | Unforgeable capability for effect `E`. |
| `Anchored<T>` | A value with its epoch. The return type of every view read. |
| `IdemKey` | Idempotency key with a declared window. |
| `Lineage<T>` | Provenance annotation at the view's declared mode. |

**Effect rows** appear on function types: `fn(..) -> T ! {read@snapshot, append, debit<usd>}`.

Generics, traits and `impl` follow Rust. Linearity is enforced for `Debit`, `Credit` and `Hold` by move semantics plus a no-implicit-drop rule.

## B.5 Data Definition (SQL Reused, Novel Clauses Added)

```niles
schema bank {
    currency usd { scale: 2 }
    currency jpy { scale: 0 }
    currency kwd { scale: 3 }

    table customers {
        id: Id<Customer> primary key,
        name: Text @confidential(e2ee),
        opened: Date,
        profile: Json,
        check (opened >= @2000-01-01),
    }

    ledger postings {
        txn: TxnId,
        acct: Id<Account> references accounts.id,
        cur: Currency,
        amt: Money,
        value_date: Date,
        idem: IdemKey window 30.days,
        conserve per (txn, cur);      // the commit rule
        retain forever;               // mandatory on a ledger
        lineage full;                 // base-level provenance retention
    }

    index customers_by_name on customers(name);   // compile error: e2ee field
}
```

`base` is the general keyword; `ledger` is `base` with a `conserve` rule and monetary/bitemporal column types. Both are append-only.

## B.6 Data Manipulation (SQL Reused)

`insert into t values (…)`, `update t set … where …`, `delete from t where …` — legal on `table` only. Writes to a `base`/`ledger` go through `txn` blocks (B.10); `insert into` a ledger is a compile error naming the requirement.

## B.7 Queries — SQL Vocabulary, Pipelined Order

```niles
let exposure = postings
    .where(|p| p.cur == usd)
    .join(accounts, |p, a| p.acct == a.id)
    .group_by(|p, a| (a.desk, a.legal_entity, a.region, p.cur))
    .sum(|p, _| p.amt)
    .having(|(_, total)| total.abs() > 1_000_000.00 usd)
    .order_by(|(k, total)| desc(total))
    .limit(100);
```

**Stages.** `.where .join(other, on) .left_join .semi_join .anti_join .group_by .agg/.sum/.count/.avg/.min/.max/.top_k .having .map .order_by .limit .offset .distinct .union .except .intersect .window(w) .match_pattern(…)`.

**Temporal stages.** `.as_of(#e)`, `.valid_at(t)`, `.value_date_between(a, b)`, `.recorded_at(e)`, `.bitemporal(sys, valid)`.

**Recursive stages.** `.fixpoint(step) guard measure(m)` — the guarded least-fixpoint construct that gives Theorem 4.6(b) and serves graph traversal.

The SQL surface accepts classical `SELECT … FROM … WHERE …` for the fragment of Appendix H; both lower to α-equivalent circuits.

## B.8 Views and the Serve Contract

```niles
view available_balance = postings
    .group_by(|p| (p.acct, p.cur))
    .sum(|p| p.amt)
    .minus(holds.unresolved().group_by(|h| (h.acct, h.cur)).sum(|h| h.amount))
    serve {
        consistency:  ledger_consistent,       // l5 — the authorization path
        materialize:  demand,                  // optimizer may override upward
        budget:       share(0.10),
        freshness:    none,                    // implied by l5
        retain:       evictable,
        lineage:      key,
        backfill:     upto(#0),
    };

view desk_exposure = /* … */
    serve { consistency: bounded(2.epochs, 5.s), materialize: auto, retain: evictable };
```

**Contract terms.** `consistency` (the six rungs of Section 3.7); `checkpoint(C)` (per-key checkpoint interval; the constant in the Bounded Reconstruction Theorem, measured in §9.4.1); `materialize (`absent | demand | full | spilled | tiered | auto`, where `auto` delegates to the optimizer within the other constraints); `budget`; `freshness` (K, T); `retain` (`evictable | pinned | forever`); `lineage` (`off | key | full`); `backfill`; and confidentiality inherited from column annotations.

Reads return `Anchored<T>`:

```niles
let b = available_balance.get((acct, usd))?;   // b.value, b.anchor
```

## B.9 Transactions, Holds, and Access Control

```niles
fn authorize(card: Id<Account>, amt: Money<usd>, auth: Option<Auth<overdraw>>)
    -> Result<Hold<usd>, TxnError>
{
    txn idem"auth-{card}-{nonce()}" {
        let avail = available_balance.get((card, usd))?;   // typed l5 by the view
        hold(card, amt, expires: 7.days, auth)?            // appends a hold row
    }
}

fn capture(h: Hold<usd>, amt: Money<usd>) -> Result<TxnId, TxnError> {
    txn idem"cap-{h.id}" { resolve h post amt }            // partial capture releases remainder
}
```

`resolve h post amt | void | expire` consumes the linear `Hold` exactly once and *appends* a resolution row. Access control reuses SQL's `grant`/`revoke`, extended so capabilities are grantable objects:

```niles
grant capability overdraw(limit: 500.00 usd) on accounts to role manager;
```

## B.10 The Imperative Sublanguage (Rust Reused)

Rust statement and expression forms — `let`, `if`/`else`, `match`, `for`, `while`, `loop`, functions, closures, `?` on `Result`. The novel form is `txn`:

```niles
fn transfer(from: Id<Account>, to: Id<Account>, amt: Money<mxn>,
            auth: Option<Auth<overdraw>>) -> Result<TxnId, TxnError> {
    txn idem"tx-{from}-{to}-{nonce()}" {
        let d: Debit<mxn>  = debit(from, amt, auth)?;
        let c: Credit<mxn> = credit(to, amt);
        post(d, c)                     // types only if per-currency rows cancel
    }
}
```

## B.11 Banking Domain Library

`std::bank`: accounts, postings, holds, `fx`, statements, interest accrual, and the product modules of Section 6.21 (`::lending`, `::trade`, `::liquidity`, `::derivatives`). All are library code over B.4's features, with no compiler knowledge of money.

## B.12 UDF Tier

```niles
#[udf(fuel = 1_000_000, deterministic)]
fn risk_score(history: &[Anchored<Money<usd>>]) -> u32 { … }
```

No ambient I/O or clock; capabilities passed as parameters; compiled to wasm32 against the ABI of Appendix C.4. The `deterministic` attribute is *checked*, not trusted: NaN-producing float operations are rejected, fuel exhaustion is a deterministic abort, and no host imports outside the allowlist are permitted (Section 6.13).

## B.13 Confidentiality Annotation

`@confidential(e2ee)` — server-blind; unusable in server-side predicates, joins or aggregates. `@confidential(committed)` — additively homomorphic commitment; usable in sum-checks only. `declassify(x, auth: Auth<declassify>)` — the only crossing, always audited. Labels are **static**; dynamic labels are not offered (Section 4.5).

## B.14 Lineage and Audit Forms

```niles
explain available_balance.get((acct, usd))?;     // derivation of this answer
reproduce (answer, #41209);                      // recompute and compare
impact of postings.row(id);                      // derived entries affected
```

## B.15 Consolidated Grammar (Skeleton)

```ebnf
program     ::= item* ;
item        ::= schema | fn | struct | enum | trait | impl | mod | use | const | view | udf ;
schema      ::= "schema" ident "{" schema_item* "}" ;
schema_item ::= table | base | ledger | view | index | currency_decl ;
base        ::= ("base" | "ledger") ident "{" field+ rule_clause* "}" ;
rule_clause ::= conserve_clause | retain_clause | lineage_clause ;
conserve_clause ::= "conserve" "per" "(" ident ("," ident)* ")" ";" ;
view        ::= "view" ident "=" pipeline "serve" "{" contract "}" ";" ;
pipeline    ::= expr (("." stage_call) | ("|>" stage_call))* ;
contract    ::= kv ("," kv)* ","? ;
txn_expr    ::= "txn" idem_lit? block ;
hold_expr   ::= "hold" "(" args ")" | "resolve" expr ("post" expr | "void" | "expire") ;
fixpoint    ::= ".fixpoint" "(" expr ")" "guard" "measure" "(" expr ")" ;
money_lit   ::= decimal currency_ident ;
temporal_lit::= "@" iso8601 | "v@" iso8601 | "#" integer | number "." time_unit ;
sql_block   ::= "sql" "{" (* the fragment of Appendix H *) "}" ;
```

The full grammar is maintained in the artifact at `crates/niles-lang/grammar/niles.ebnf` as the normative machine-readable form; this skeleton is its table of contents.

## B.16 Reserved Keyword List

The union of B.3.1–B.3.3 plus reserved-for-future `async await unsafe yield macro stream actor`. All reserved words require `r#` to be used as identifiers.

## B.17 SQL ↔ Niles Mapping

| SQL | Niles |
|---|---|
| `SELECT a,b FROM t WHERE p` | `t.where(\|r\| p).map(\|r\| (r.a, r.b))` |
| `GROUP BY k HAVING h` | `.group_by(\|r\| k).having(\|g\| h)` |
| `JOIN u ON c` | `.join(u, \|t,u\| c)` |
| `ORDER BY x DESC LIMIT n` | `.order_by(\|r\| desc(r.x)).limit(n)` |
| `WITH RECURSIVE` | `.fixpoint(step) guard measure(m)` |
| `CREATE MATERIALIZED VIEW` | `view … serve { materialize: full }` |
| `AS OF SYSTEM TIME` | `.as_of(#e)` |
| `FOR SYSTEM_TIME` / period predicates | `.recorded_at`, `.valid_at`, `.bitemporal` |
| `INSERT/UPDATE/DELETE` | same, on `table` only |
| `BEGIN/COMMIT` | `txn { … }` (base) / `begin…commit` (table) |
| *(no equivalent)* | `serve` contracts, `Money<CUR>`, `conserve`, `hold`/`resolve`, `Auth`, `@confidential`, `idem`, `materialize`, `lineage` |

## B.18 Niles ↔ Rust Mapping and Novel Forms

Identical to Rust: items, expressions, patterns, generics, traits, modules, closures, `Result`/`?`. Restricted: no raw pointers, no `unsafe`, no ambient I/O or clock in query and transaction context, guarded recursion only. Novel beyond Rust: `schema/table/base/ledger/view/serve`, pipelines as queries, money/temporal/epoch/idempotency literals, effect rows, linear `Debit`/`Credit`/`Hold`, `txn`, `fx`, capabilities, confidentiality and lineage annotations.

## B.19 Keyword Reference

The normative per-keyword reference — one entry per keyword with a description and a minimal compiling sample, in three tables (SQL-derived, Rust-derived, novel) — is generated from the compiler's keyword registry into `docs/keywords.md`, so it cannot drift from the implementation. Representative entries:

- `ledger` — append-only conserved base. `ledger p { txn: TxnId, acct: Id<A>, cur: Currency, amt: Money, conserve per (txn,cur); retain forever; }`
- `serve` — attach a contract. `view v = q serve { consistency: snapshot, materialize: auto, retain: evictable };`
- `materialize` — mode hint or delegation. `serve { materialize: tiered }`
- `hold` / `resolve` — reservations as ledger facts. `let h = hold(acct, 20.00 usd, expires: 7.days, None)?; resolve h post 18.50 usd`
- `anchor` — the stamp. `let e = balances.get(k)?.anchor;`
- `fx` — atomic two-currency form. `fx { leg a: post(d_usd, c_usd), leg b: post(d_mxn, c_mxn), rate: r }`
- `guard` — well-founded recursion. `view reach = edges.fixpoint(step) guard measure(depth);`
- `lineage` — provenance mode. `serve { lineage: full }`
- `explain` — derivation of an answer. `explain balances.get(k)?;`

## B.20 A Complete Worked Program

```niles
schema demo_bank {
    currency usd { scale: 2 }

    table accounts { id: Id<Account> primary key, owner: Text @confidential(e2ee) }

    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        value_date: Date, idem: IdemKey window 30.days,
        conserve per (txn, cur);
        retain forever;
    }

    ledger holds {
        id: HoldId, acct: Id<Account>, cur: Currency, amount: Money,
        expires: Instant, idem: IdemKey window 7.days,
        retain forever;
    }

    view ledger_balance = postings
        .group_by(|p| (p.acct, p.cur)).sum(|p| p.amt)
        serve { consistency: read_your_writes, materialize: auto, retain: evictable };

    view available_balance = ledger_balance
        .minus(holds.unresolved().group_by(|h| (h.acct, h.cur)).sum(|h| h.amount))
        serve { consistency: ledger_consistent, materialize: demand, lineage: key };

    view statement_mtd = postings
        .value_date_between(month_start(), today())
        .group_by(|p| p.acct)
        serve { consistency: bounded(10.epochs, 30.s), materialize: demand, retain: evictable };
}

fn pay_rent(tenant: Id<Account>, landlord: Id<Account>) -> Result<TxnId, TxnError> {
    let rent = 850.00 usd;
    txn idem"rent-2026-08-{tenant}" {
        let d = debit(tenant, rent, None)?;     // fails if overdraft and no Auth
        let c = credit(landlord, rent);
        post(d, c)
    }
}

fn main() -> Result<(), TxnError> {
    let _t = pay_rent(acct!(1001), acct!(2002))?;
    let b = available_balance.get((acct!(1001), usd))?;
    println!("available {} as of epoch {}", b.value, b.anchor);
    Ok(())
}
```

## B.21 Standard-Library Catalogue

**Aggregates (incrementally maintained).** `sum count count_distinct avg min max top_k arg_min arg_max stddev var percentile_sketch` — each documenting its delta form, and `min`/`max`/percentiles documenting their behaviour under retraction, which is where naive incremental aggregates are wrong.

**Money and FX.** `Money::zero(cur) abs allocate(parts) split_pro_rata(weights) round_to(scale, mode) fx_convert(m, rate) -> FxLegs`. `allocate` exists because dividing an amount into parts is the canonical place where naive arithmetic *destroys money* through rounding; it distributes remainders deterministically so the parts sum exactly to the whole, and returns a value whose type obliges the caller to use all parts.

**Temporal and identity (epoch-deterministic).** `now_anchor() epoch_of(instant) value_date_of(row) valid_window(from,to) accrue(signal, rate, daycount) window(tumbling|sliding|session) seq_id() id_at(anchor)` — all deterministic given the anchor; no wall-clock reads.

**Scalar, string, hashing.** `len concat split trim lower upper regex_match parse_int format hash digest hex`.

**Collections (Rust-sourced).** `map filter fold zip enumerate chunk sort_by dedup first last get windows` — restricted to pure closures in query context.

**Semi-structured, graph, search.** `json_get json_path json_exists` for document access; `neighbors reachable shortest_path` over edge bases via guarded fixpoint; `tokenize match_terms rank_bm25` for inverted-index views.
