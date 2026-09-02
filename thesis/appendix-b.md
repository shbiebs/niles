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

<!-- BEGIN:kw-sql docs/keywords.md#kwsql -->

*Generated from `docs/keywords.md`. Do not edit by hand.*

`add` `all` `alter` `and` `any` `as` `asc` `begin` `between` `by` `case` `cast` `check` `column` `commit` `create` `cross` `default` `delete` `desc` `distinct` `drop` `else` `end` `except` `exists` `foreign` `from` `full` `grant` `group` `having` `in` `index` `inner` `insert` `intersect` `into` `is` `join` `key` `left` `like` `limit` `natural` `not` `null` `offset` `on` `or` `order` `outer` `primary` `recursive` `references` `revoke` `right` `rollback` `select` `set` `table` `then` `union` `unique` `update` `using` `values` `view` `when` `where` `with`

<!-- END:kw-sql -->

SQL-derived words are legal only in their assigned positions; `update` and `delete` are legal only against `table` data, never against a `ledger`.

### B.3.2 Rust-Derived

<!-- BEGIN:kw-rust docs/keywords.md#kwrust -->

*Generated from `docs/keywords.md`. Do not edit by hand.*

`Self` `break` `const` `continue` `crate` `dyn` `enum` `false` `fn` `for` `if` `impl` `let` `loop` `match` `mod` `move` `mut` `pub` `ref` `return` `self` `static` `struct` `super` `trait` `true` `type` `use` `while`

<!-- END:kw-rust -->

`unsafe`, `async` and `await` are reserved and rejected in v0.1. `where` serves both Rust's bound clause and the pipeline filter stage; positions are disjoint and the grammar disambiguates.

### B.3.3 Novel Niles Keywords

<!-- BEGIN:kw-novel docs/keywords.md#kwnovel -->

*Generated from `docs/keywords.md`. Do not edit by hand.*

`absent` `anchor` `as_of` `authorize` `auto` `backfill` `base` `bitemporal` `bounded` `budget` `capability` `committed` `confidential` `conserve` `consistency` `currency` `declassify` `demand` `e2ee` `emit` `epoch` `evictable` `expire` `expires` `explain` `forever` `freshness` `fx` `guard` `hold` `idem` `impact` `ledger` `ledger_consistent` `leg` `lineage` `materialize` `measure` `monotonic` `of` `per` `pinned` `post` `posting` `rate` `read_your_writes` `recorded_at` `reproduce` `resolve` `retain` `scale` `schema` `serializable` `serve` `signal` `snapshot` `spilled` `sql` `tiered` `txn` `udf` `upto` `valid_at` `value_date` `void` `window`

<!-- END:kw-novel -->

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

The SQL surface accepts classical `SELECT … FROM … WHERE …` for the fragment of Appendix H; both denote the same Z-set, which is what the golden corpus checks and what Theorem 4.6(c) claims.

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

### B.10.1 Operator Precedence and Associativity

Normative. Higher binding power binds tighter; every level is left-associative except assignment, which is right-associative and binds loosest of all. Where a word and a symbol denote the same operator — `and`/`&&`, `or`/`||` — they share a level and produce the same tree, because a program's meaning must not depend on which of its two ancestries the author was thinking in.

| Power | Operators | Associativity |
|---|---|---|
| *(loosest)* | `=` | **right** |
| 1 | `or` `\|\|` | left |
| 2 | `and` `&&` | left |
| 3 | `==` `!=` `<` `<=` `>` `>=` `is` `is not` `in` `not in` `like` `between` | left |
| 4 | `\|` | left |
| 5 | `^` | left |
| 6 | `&` | left |
| 7 | `+` `-` | left |
| 8 | `*` `/` `%` | left |
| *(tightest)* | prefix `-` `!` `not` `*` `&` `&mut`; postfix `.f` `.m(..)` `\|> m(..)` `(..)` `[..]` `?` `as T` | — |

Two consequences are worth stating because they are where hand-written parsers go wrong, and where this table earned its place. First, `a - b - c` is `(a - b) - c`; a Pratt loop that recurses on the right at the *same* power instead of one above it produces the other tree, and no test that inspects only the token stream can tell. Second, `a = b = c` is `a = (b = c)`, following Rust: the reference parser recursed for the right-hand side at power 1 rather than 0, which put assignment outside its own guard and made it silently left-associative — a defect that survived until `bootstrap/parser.niles` was written against this table and the two implementations disagreed. The table exists so that a disagreement of that kind has an arbiter, and Appendix E's stage-1 equivalence gate is what forces the question to be asked.

Bitwise operators sit *between* comparison and arithmetic, which is C's ordering rather than Python's, and is inherited from Rust along with the rest of the expression grammar. `between` takes a tuple — `x between (lo, hi)` — so that the parse is uniform and the operator needs no special case in the table above.


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

The full grammar is maintained in the artifact at `crates/niles-lang/grammar/niles.ebnf` as the normative machine-readable form; this skeleton is its table of contents. That file defines **205 named rules**, which is **496 productions** when expanded to one production per alternative. Both figures are computed from the file by `crates/niles-lang/tests/grammar_drift.rs` and asserted against the file's own header, so neither can drift from what is written; and both are reported because "how big is a grammar" has no single answer, and an uncounted one is folklore. No comparable published figure for SQL is cited here: PostgreSQL's `gram.y` is 21,016 lines, but no authoritative production count for the SQL-92 or SQL:2016 BNF could be verified, and this thesis declines to quote a number it has not counted itself.

The same test file enforces the grammar against the compiler in both directions. Every keyword in the registry must appear in some production, or it is dead vocabulary; every alphabetic terminal in the grammar must be a registry keyword or a known library name, or the production is unreachable; the pipeline-stage vocabulary of grammar section 12 must equal the compiler's `StageKind::all_names()`, because that list is the set of operators the IR must implement; and the twenty well-formedness rules of section 15 must be contiguous. The grammar's last section is the honest part of it: W1–W20 are the obligations deliberately *not* encoded as syntax, because a grammar that tried to say "a hold is consumed exactly once" would be neither readable nor decidable. Each is a claim that a later phase enforces it, and each has a test in the phase that does.

## B.16 Reserved Keyword List

Generated from the compiler's keyword registry, like B.3.1–B.3.3 above. The three lists there and this one used to be maintained by hand beside a registry that already generated `docs/keywords.md`, so a word added to the lexer and not to the appendix was a word the normative grammar did not have.

<!-- BEGIN:kw-reserved docs/keywords.md#kwreserved -->

*Generated from `docs/keywords.md`. Do not edit by hand.*

68 words. All require `r#` to be used as identifiers.

```
Self actor all and as async await break by case const continue crate cross distinct dyn else enum except false fn for full having if impl in inner intersect is join left let loop macro match mod move mut natural not null on or outer pub ref return right select self static stream struct super then trait true type union unsafe use using when where while with yield
```

<!-- END:kw-reserved -->

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

The normative per-keyword reference — one entry per keyword with a description and a minimal compiling sample, in three tables (SQL-derived, Rust-derived, novel) — is generated from the compiler's keyword registry into `docs/keywords.md`, so it cannot drift from the implementation.

The registry is `crates/niles-lang/src/keywords.rs`, and it is the direct analogue of PostgreSQL's `kwlist.h`: one table, no logic, consumed by whoever needs it. PostgreSQL keeps its keyword lists "in their own source files for use by automatic tools" and generates from that one table the perfect-hash lookup, the parser's token declarations *and* the documentation, so that none of the three can drift from the others. Niles copies the discipline and carries four axes rather than PostgreSQL's two. The first two are PostgreSQL's own and are orthogonal: a **category** (`unreserved`, `non-reserved-cannot-be-function-or-type-name`, `reserved-can-be-function-or-type-name`, `reserved`, plus `reserved-for-future`) and a **label status**, recording whether the word may be a bare output label without `as` — `select 55 check` is legal, `select 55 as from` is required. The two Niles adds are what this thesis needs: **origin** (SQL-derived, Rust-derived, novel), which generates the three sub-tables of B.3 mechanically rather than by hand, and **edition**, which makes the syntax-lineage rule of §6.25 enforceable instead of aspirational, since a word's edition records when it entered the language and therefore which programs a later addition may break.

The registry holds **174 keywords**: 70 SQL-derived, 31 Rust-derived, 66 novel and 7 reserved for future use. Of these, 95 are unreserved and 66 are reserved. Those figures are asserted by `tests/grammar_drift.rs`, so a keyword added without updating this paragraph fails the build.

**The reservation policy, and why the numbers matter.** Reserved-word count is a function of parser technology, not of vocabulary size. SQL reserves heavily in part because an LALR(1) grammar cannot resolve context-dependent identifier/keyword ambiguity, so words are reserved to remove conflicts. Niles's parser is hand-written recursive descent with unbounded lookahead, which buys the freedom to keep a word meaningful in its clause and ordinary everywhere else. The policy that follows is written into the registry: *a new keyword is unreserved unless a written justification records why the grammar cannot be written without reserving it*, and a test fails if a novel keyword is reserved without one. The measured result is that **all 66 novel keywords are unreserved**. This is not a stylistic preference. Niles is meant to be adopted against schemas that already exist, and a bank's ledger table is very likely to have columns called `epoch`, `ledger`, `posted`, `valid`, `serve`, `budget` or `scale`. Reserving those would mean a migration renames columns before it can begin, and the reserved set is the single most under-examined adoption cost a new query language imposes. Two further mechanisms soften what remains: any word at all is usable as an identifier under the raw escape `r#word`, and in *member position* — after `.` or `|>`, and as a struct-literal field — every keyword is legal without escape, because those positions have exactly one reading. That is why `p.where(..)`, `p.order`, `p.select` and `p.check` all parse.

Representative entries:

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
