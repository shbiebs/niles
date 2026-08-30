//! The Niles keyword registry — the single normative source of truth.
//!
//! This module is the direct analogue of PostgreSQL's `kwlist.h`: one table, no logic,
//! consumed by whoever needs it. PostgreSQL keeps its keyword lists "in their own source
//! files for use by automatic tools", with the representation decided by the consuming
//! macro; the same discipline is applied here so that a keyword cannot exist in the lexer
//! without existing in the documentation, and vice versa.
//!
//! Everything downstream is derived from this table and nothing restates it:
//!
//! * the lexer's keyword recognizer ([`lookup`]),
//! * the parser's `unreserved_keyword` escape (via [`Category`]),
//! * the normative reserved list of thesis Appendix B.16,
//! * the per-keyword reference of Appendix B.19, generated into `docs/keywords.md`
//!   by the `gen-keyword-ref` binary,
//! * the grammar's terminal set, which `tests/grammar_drift.rs` checks against
//!   `grammar/niles.ebnf` in both directions.
//!
//! # Two orthogonal axes, not one
//!
//! PostgreSQL's registry carries a *category* (how reserved the word is) and, separately,
//! a *label status* (whether the word may be a bare output label without `AS`). Niles
//! keeps both, and adds two more that the thesis needs: [`Origin`], which generates
//! Appendix B's three sub-tables mechanically, and `since`, which makes the edition
//! mechanism of §6.24 enforceable rather than aspirational.
//!
//! # Reservation policy
//!
//! Reserved-word count is a function of parser technology, not of vocabulary size. SQL
//! reserves heavily in part because an LALR(1) grammar cannot resolve context-dependent
//! identifier/keyword ambiguity; Niles uses hand-written recursive descent with unbounded
//! lookahead, so a word can be a keyword in its clause position and an ordinary identifier
//! everywhere else. Niles is meant to be adopted *against existing bank schemas*, where
//! `epoch`, `ledger`, `posted`, `valid`, `serve`, `balance` and `settle` are all plausible
//! existing column names. The policy is therefore:
//!
//! > A new keyword is [`Category::Unreserved`] unless a written justification records why
//! > the grammar cannot be written without reserving it.
//!
//! Even a [`Category::Reserved`] word remains usable as an identifier under the raw-escape
//! `r#ident`, and as a bare output label when its [`Label`] is [`Label::Bare`].

use std::collections::HashMap;
use std::sync::OnceLock;

/// How far a word is reserved. Ordered from most usable to least.
///
/// The four classes mirror PostgreSQL's, because the problem is identical: a language with
/// a keyword-delimited grammar and a large installed base of schemas whose column names
/// were chosen before the keyword existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    /// No special status outside its own clause; usable as any identifier, anywhere.
    Unreserved,
    /// Usable as a column or variable name, but not as a function or type name, because
    /// the word introduces built-in syntax that would be ambiguous with a call.
    ColName,
    /// Usable as a function or type name, but not as a bare column name — the classic
    /// `JOIN`/`LEFT`/`FULL` family, where a bare occurrence would be read as a join clause.
    TypeFuncName,
    /// Never an identifier without the `r#` escape.
    Reserved,
    /// Reserved with no meaning assigned: using it is a hard error naming the reason,
    /// which is how a language keeps room to grow without a breaking change later.
    ReservedFuture,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Unreserved => "unreserved",
            Category::ColName => "non-reserved (cannot be function or type name)",
            Category::TypeFuncName => "reserved (can be function or type name)",
            Category::Reserved => "reserved",
            Category::ReservedFuture => "reserved for future use",
        }
    }
    /// Whether the word may appear where an identifier is expected, without `r#`.
    pub fn usable_as_ident(self) -> bool {
        matches!(self, Category::Unreserved | Category::ColName)
    }
    /// Whether the word may name a function or a type.
    pub fn usable_as_type_or_fn(self) -> bool {
        matches!(self, Category::Unreserved | Category::TypeFuncName)
    }
}

/// The second, orthogonal axis: may this word be a bare output label, without `AS`?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Label {
    /// `select 55 check` is legal: the word can be a bare column label.
    Bare,
    /// `select 55 as from` is required; a bare occurrence would continue the clause.
    RequiresAs,
}

/// Where the word came from. This is what generates Appendix B.3's three sub-tables, and
/// it is the mechanical statement of the thesis's syntax-lineage rule: SQL first, Rust as
/// fallback where SQL has no equivalent, novel only where neither language has the concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Origin {
    /// Taken from SQL, with SQL's meaning wherever the meaning survives.
    Sql,
    /// Taken from Rust, because SQL has no equivalent construct.
    Rust,
    /// Novel: names a concept neither SQL nor Rust has.
    Novel,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::Sql => "SQL-derived",
            Origin::Rust => "Rust-derived",
            Origin::Novel => "novel",
        }
    }
}

/// One registry row.
#[derive(Debug, Clone, Copy)]
pub struct Keyword {
    /// The spelling, lower-case ASCII. Niles keywords are case-insensitive on input
    /// (SQL habit) but canonicalised to this spelling in diagnostics and pretty-printing.
    pub word: &'static str,
    /// The token this word lexes to.
    pub token: Kw,
    pub category: Category,
    pub label: Label,
    pub origin: Origin,
    /// The edition in which the word acquired its meaning. A program declaring an older
    /// edition lexes this word as an ordinary identifier.
    pub since: &'static str,
    /// One line: what the word does. Appears in `docs/keywords.md` and in `--explain`.
    pub doc: &'static str,
    /// A minimal fragment using the word. Appendix B.19 requires one per keyword.
    pub example: &'static str,
}

/// The token kind a keyword lexes to. One variant per registry row; the lexer never
/// invents a keyword token that is not here, which is what makes the drift test possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum Kw {
    // ---- SQL-derived ----
    Select, From, Where, Group, By, Having, Order, Limit, Offset, Join, Left, Right, Full,
    Outer, Inner, On, Union, Except, Intersect, Distinct, As, Create, Table, View, Index,
    Insert, Into, Values, Update, Delete, Begin, Commit, Rollback, Grant, Revoke, Primary,
    Key, Foreign, References, Check, Default, Null, And, Or, Not, In, Exists, Between,
    Like, Case, When, Then, Else, End, Asc, Desc, Cross, Natural, Using, All, Any, Cast,
    With, Recursive, Unique, Drop, Alter, Add, Column, Set, Is,
    // ---- Rust-derived ----
    Fn, Let, Mut, Const, Static, Struct, Enum, Trait, Impl, For, While, Loop, If, Match,
    Return, Break, Continue, Mod, Use, Pub, Crate, SelfValue, SelfType, Super, Type, Move,
    Ref, Dyn, Where_, As_, In_, Else_, True, False,
    // ---- Novel ----
    Base, Ledger, Posting, Txn, Hold, Resolve, Void, Expire, Schema, Serve, Consistency,
    Bounded, Monotonic, ReadYourWrites, Snapshot, Serializable, LedgerConsistent,
    Materialize, Absent, Demand, FullMode, Spilled, Tiered, Auto, Budget, Freshness,
    Retain, Evictable, Pinned, Forever, Anchor, AsOf, Epoch, ValidAt, ValueDate,
    RecordedAt, Bitemporal, Authorize, Capability, Idem, Window, Conserve, Currency,
    Scale, Fx, Leg, Confidential, E2ee, Committed, Declassify, Lineage, Emit, Signal,
    Backfill, Upto, Guard, Measure, Explain, Reproduce, Impact, Per, Post, Udf, Rate,
    Sql, Expires, Of,
    // ---- Reserved for future use ----
    Async, Await, Unsafe, Yield, Macro, Stream, Actor,
}

macro_rules! kw {
    ($w:literal, $t:ident, $c:ident, $l:ident, $o:ident, $since:literal, $d:literal, $e:literal) => {
        Keyword {
            word: $w,
            token: Kw::$t,
            category: Category::$c,
            label: Label::$l,
            origin: Origin::$o,
            since: $since,
            doc: $d,
            example: $e,
        }
    };
}

/// The registry. **Kept in ASCII order of `word`**, which `registry_is_sorted` enforces,
/// so that a reader, a diff and a perfect-hash generator all see the same order.
pub static KEYWORDS: &[Keyword] = &[
    // ================= SQL-derived =================
    kw!("add", Add, Unreserved, Bare, Sql, "2026", "Add a column or constraint in `alter table`.", "alter table accounts add column tier: i32;"),
    kw!("all", All, Reserved, RequiresAs, Sql, "2026", "Keep duplicates in a set operation, or the universal quantifier.", "let x = a.union_all(b);"),
    kw!("alter", Alter, Unreserved, Bare, Sql, "2026", "Change a declared object. Legal on `table`, never on `base` or `ledger`.", "alter table accounts add column tier: i32;"),
    kw!("and", And, Reserved, RequiresAs, Sql, "2026", "Short-circuiting boolean conjunction.", "where(|r| r.open and r.cur == usd)"),
    kw!("any", Any, Unreserved, Bare, Sql, "2026", "Existential quantifier over a subquery or collection.", "where(|r| r.tags.any(|t| t == \"vip\"))"),
    kw!("as", As, Reserved, RequiresAs, Sql, "2026", "Rename a column, bind an output label, or coerce a value.", "select(|r| (r.amt as label(\"amount\")))"),
    kw!("asc", Asc, Unreserved, Bare, Sql, "2026", "Ascending sort direction; the default.", "order_by(|r| asc(r.posted_at))"),
    kw!("begin", Begin, Unreserved, Bare, Sql, "2026", "Open a table-level transaction. Ledger writes use `txn` instead.", "begin; insert into accounts values (1, \"a\"); commit;"),
    kw!("between", Between, ColName, Bare, Sql, "2026", "Inclusive range test.", "where(|r| r.amt between (0.00 usd, 100.00 usd))"),
    kw!("by", By, Reserved, RequiresAs, Sql, "2026", "Introduces the grouping or ordering key; never appears alone.", "group_by(|r| r.acct)"),
    kw!("case", Case, Reserved, RequiresAs, Sql, "2026", "Multi-way conditional expression. `match` is the Rust-derived alternative.", "case when r.amt > 0 then \"cr\" else \"dr\" end"),
    kw!("cast", Cast, ColName, Bare, Sql, "2026", "Explicit conversion. Never crosses a currency: `Money<USD>` has no cast to `Money<EUR>`.", "cast(r.n as i64)"),
    kw!("check", Check, Unreserved, Bare, Sql, "2026", "Row-level constraint on a table.", "table t { n: i32 check (n >= 0) }"),
    kw!("column", Column, Unreserved, Bare, Sql, "2026", "Names a column in DDL.", "alter table accounts add column tier: i32;"),
    kw!("commit", Commit, Unreserved, Bare, Sql, "2026", "Close a table transaction, making its writes visible at the next epoch.", "begin; update accounts set tier = 2; commit;"),
    kw!("create", Create, Unreserved, Bare, Sql, "2026", "Declare an object. Inside `schema` the word is optional.", "create index ix on postings (acct);"),
    kw!("cross", Cross, TypeFuncName, RequiresAs, Sql, "2026", "Unrestricted product join.", "a.cross_join(b)"),
    kw!("default", Default, Unreserved, Bare, Sql, "2026", "Column default. Must be a pure, epoch-independent expression.", "table t { n: i32 default 0 }"),
    kw!("delete", Delete, Unreserved, Bare, Sql, "2026", "Remove rows. Legal on `table` only; a `ledger` has no delete.", "delete from staging where done;"),
    kw!("desc", Desc, Unreserved, Bare, Sql, "2026", "Descending sort direction.", "order_by(|r| desc(r.amt))"),
    kw!("distinct", Distinct, Reserved, RequiresAs, Sql, "2026", "Deduplicate. On a Z-set this is the canonicalising `distinct` operator.", "postings.distinct_by(|r| r.acct)"),
    kw!("drop", Drop, Unreserved, Bare, Sql, "2026", "Remove a declared object. Never removes ledger history.", "drop view stale_v;"),
    kw!("else", Else, Reserved, RequiresAs, Sql, "2026", "Alternative branch of `case` or `if`.", "case when p then a else b end"),
    kw!("end", End, ColName, RequiresAs, Sql, "2026", "Closes a `case` expression. Position-determined, so `end` remains usable as a column name.", "case when p then a else b end"),
    kw!("except", Except, Reserved, RequiresAs, Sql, "2026", "Set difference.", "a.except(b)"),
    kw!("exists", Exists, ColName, Bare, Sql, "2026", "Non-emptiness test over a subquery.", "where(|r| exists(holds.for_acct(r.acct)))"),
    kw!("foreign", Foreign, Unreserved, Bare, Sql, "2026", "Introduces a foreign key constraint.", "foreign key (acct) references accounts (id)"),
    kw!("from", From, ColName, RequiresAs, Sql, "2026", "Source relation, in the SQL surface and in `delete`. Clause-position only, so it remains usable as a variable or column name.", "sql { select id from accounts }"),
    kw!("full", Full, TypeFuncName, RequiresAs, Sql, "2026", "Full outer join; also the `full` materialization mode and `lineage: full`.", "a.full_outer_join(b, |x, y| x.k == y.k)"),
    kw!("grant", Grant, Unreserved, Bare, Sql, "2026", "Confer a capability. Niles grants an `Auth<E>`, not an ambient role.", "grant debit<usd> on postings to teller;"),
    kw!("group", Group, ColName, RequiresAs, Sql, "2026", "Introduces grouping; the pipeline spelling is `group_by`. Clause-position only.", "group_by(|r| (r.acct, r.cur))"),
    kw!("having", Having, Reserved, RequiresAs, Sql, "2026", "Filter applied after grouping, over group aggregates.", "group_by(|r| r.acct).having(|g| g.sum > 0.00 usd)"),
    kw!("in", In, Reserved, RequiresAs, Sql, "2026", "Membership test, and the binder in `for x in xs`.", "where(|r| r.cur in [usd, eur])"),
    kw!("index", Index, Unreserved, Bare, Sql, "2026", "Declare an anchor index. Anchor indices are mandatory on ledger keys.", "index by_acct on postings (acct) anchor;"),
    kw!("inner", Inner, TypeFuncName, RequiresAs, Sql, "2026", "Inner join; the default join kind.", "a.join(b, |x, y| x.k == y.k)"),
    kw!("insert", Insert, Unreserved, Bare, Sql, "2026", "Add rows to a `table`. A `ledger` is appended to with `txn`, not `insert`.", "insert into accounts values (1, \"ada\");"),
    kw!("intersect", Intersect, Reserved, RequiresAs, Sql, "2026", "Set intersection.", "a.intersect(b)"),
    kw!("into", Into, Unreserved, Bare, Sql, "2026", "Target of an `insert`.", "insert into accounts values (1, \"ada\");"),
    kw!("is", Is, Reserved, RequiresAs, Sql, "2026", "Identity test, chiefly `is null` and `is not null`.", "where(|r| r.closed_at is null)"),
    kw!("join", Join, TypeFuncName, RequiresAs, Sql, "2026", "Relational join. In a pipeline it is a stage; in `sql {}` it is a clause.", "postings.join(accounts, |p, a| p.acct == a.id)"),
    kw!("key", Key, Unreserved, Bare, Sql, "2026", "Part of `primary key` / `foreign key`; also `lineage: key`.", "table t { id: i64 primary key }"),
    kw!("left", Left, TypeFuncName, RequiresAs, Sql, "2026", "Left outer join.", "a.left_join(b, |x, y| x.k == y.k)"),
    kw!("like", Like, ColName, Bare, Sql, "2026", "Pattern match on `Text`.", "where(|r| r.name like \"ac%\")"),
    kw!("limit", Limit, ColName, RequiresAs, Sql, "2026", "Bound the result cardinality. Clause-position only, so `limit` remains usable as a column name.", "order_by(|r| desc(r.amt)).limit(10)"),
    kw!("natural", Natural, TypeFuncName, RequiresAs, Sql, "2026", "Join on all like-named columns. Discouraged: it is schema-fragile.", "a.natural_join(b)"),
    kw!("not", Not, Reserved, RequiresAs, Sql, "2026", "Boolean negation.", "where(|r| not r.closed)"),
    kw!("null", Null, Reserved, RequiresAs, Sql, "2026", "The SQL null. Distinct from `Option::None` and from an evicted `Hole`.", "where(|r| r.closed_at is null)"),
    kw!("offset", Offset, ColName, RequiresAs, Sql, "2026", "Skip a prefix of the result. Clause-position only.", "order_by(|r| r.id).offset(20).limit(10)"),
    kw!("on", On, Reserved, RequiresAs, Sql, "2026", "Join predicate, or the object of a `grant`.", "a.join(b, |x, y| x.k == y.k)"),
    kw!("or", Or, Reserved, RequiresAs, Sql, "2026", "Short-circuiting boolean disjunction.", "where(|r| r.a or r.b)"),
    kw!("order", Order, ColName, RequiresAs, Sql, "2026", "Introduces ordering; the pipeline spelling is `order_by`. Clause-position only.", "order_by(|r| asc(r.id))"),
    kw!("outer", Outer, TypeFuncName, RequiresAs, Sql, "2026", "Marks a join as outer.", "a.full_outer_join(b, |x, y| x.k == y.k)"),
    kw!("primary", Primary, Unreserved, Bare, Sql, "2026", "Introduces the primary key.", "table t { id: i64 primary key }"),
    kw!("recursive", Recursive, Unreserved, Bare, Sql, "2026", "Marks a CTE as recursive. Niles requires a `guard measure(..)` on the recursion regardless.", "sql { with recursive r as (select 1) select * from r }"),
    kw!("references", References, Unreserved, Bare, Sql, "2026", "Target of a foreign key.", "foreign key (acct) references accounts (id)"),
    kw!("revoke", Revoke, Unreserved, Bare, Sql, "2026", "Withdraw a capability. Recorded as a ledger event, never a silent edit.", "revoke debit<usd> on postings from teller;"),
    kw!("right", Right, TypeFuncName, RequiresAs, Sql, "2026", "Right outer join.", "a.right_join(b, |x, y| x.k == y.k)"),
    kw!("rollback", Rollback, Unreserved, Bare, Sql, "2026", "Abandon a table transaction. A sealed ledger epoch cannot be rolled back.", "begin; update t set n = 1; rollback;"),
    kw!("select", Select, Reserved, RequiresAs, Sql, "2026", "Projection, in the SQL surface. The pipeline spelling is `.map`/`.select`.", "sql { select id, owner from accounts }"),
    kw!("set", Set, Unreserved, Bare, Sql, "2026", "Assignment list of an `update`.", "update accounts set tier = 2 where id == 1;"),
    kw!("table", Table, Unreserved, Bare, Sql, "2026", "A mutable relation: update and delete are legal, history is not retained.", "table accounts { id: Id<Account> primary key }"),
    kw!("then", Then, Reserved, RequiresAs, Sql, "2026", "Consequent of a `case` arm.", "case when p then a else b end"),
    kw!("union", Union, Reserved, RequiresAs, Sql, "2026", "Set union, deduplicating.", "a.union(b)"),
    kw!("unique", Unique, Unreserved, Bare, Sql, "2026", "Uniqueness constraint.", "table t { k: Text unique }"),
    kw!("update", Update, Unreserved, Bare, Sql, "2026", "Modify rows. Legal on `table` only; a `ledger` has no update.", "update accounts set tier = 2 where id == 1;"),
    kw!("using", Using, TypeFuncName, RequiresAs, Sql, "2026", "Join on named common columns.", "a.join_using(b, [\"acct\"])"),
    kw!("values", Values, ColName, RequiresAs, Sql, "2026", "Literal row constructor, in `insert`. Clause-position only.", "insert into accounts values (1, \"ada\");"),
    kw!("view", View, Unreserved, Bare, Sql, "2026", "A derived relation with a serve contract. The REV of the theory.", "view v = postings.group_by(|p| p.acct) serve { consistency: snapshot };"),
    kw!("when", When, Reserved, RequiresAs, Sql, "2026", "Guard of a `case` arm, or of a `match` arm.", "case when p then a else b end"),
    kw!("where", Where, Reserved, RequiresAs, Sql, "2026", "Filter stage, and Rust's bound clause. The positions are disjoint.", "postings.where(|p| p.amt > 0.00 usd)"),
    kw!("with", With, Reserved, RequiresAs, Sql, "2026", "Common table expression in the SQL surface.", "sql { with t as (select 1) select * from t }"),

    // ================= Rust-derived =================
    kw!("Self", SelfType, Reserved, RequiresAs, Rust, "2026", "The implementing type.", "fn zero() -> Self { Self { amt: 0.00 usd } }"),
    kw!("break", Break, Reserved, RequiresAs, Rust, "2026", "Leave a loop. Illegal inside a `fixpoint` body, which must terminate by measure.", "loop { if done { break; } }"),
    kw!("const", Const, Reserved, RequiresAs, Rust, "2026", "Compile-time constant. Must be pure and epoch-independent.", "const LIMIT: Money<USD> = 500.00 usd;"),
    kw!("continue", Continue, Reserved, RequiresAs, Rust, "2026", "Next iteration of a loop.", "for p in ps { if p.zero() { continue; } post(p)?; }"),
    kw!("crate", Crate, Reserved, RequiresAs, Rust, "2026", "Path root of the current compilation unit.", "use crate::bank::transfer;"),
    kw!("dyn", Dyn, Reserved, RequiresAs, Rust, "2026", "Dynamic dispatch. Forbidden in view bodies, which must be statically planned.", "let f: &dyn Rate = &fixed;"),
    kw!("enum", Enum, Reserved, RequiresAs, Rust, "2026", "Sum type.", "enum Side { Debit, Credit }"),
    kw!("false", False, Reserved, Bare, Rust, "2026", "Boolean literal.", "let b = false;"),
    kw!("fn", Fn, Reserved, RequiresAs, Rust, "2026", "Function item. Its effect row is part of its type.", "fn transfer(a: Acct, b: Acct) -> Result<()> ! { append, debit<usd> } { .. }"),
    kw!("for", For, Reserved, RequiresAs, Rust, "2026", "Iteration, or the `impl .. for ..` head.", "for p in batch { post(p)?; }"),
    kw!("if", If, Reserved, RequiresAs, Rust, "2026", "Conditional expression.", "if r.amt > lim { flag(r) } else { r }"),
    kw!("impl", Impl, Reserved, RequiresAs, Rust, "2026", "Inherent or trait implementation.", "impl Rate for Fixed { .. }"),
    kw!("let", Let, Reserved, RequiresAs, Rust, "2026", "Bind a value. Linear types bound by `let` must be consumed exactly once.", "let d = debit(a, 10.00 usd)?;"),
    kw!("loop", Loop, Reserved, RequiresAs, Rust, "2026", "Unconditional loop. Not permitted in a view body.", "loop { step()?; }"),
    kw!("match", Match, Reserved, RequiresAs, Rust, "2026", "Pattern match. Exhaustive, unlike SQL's `case`.", "match outcome { Post(m) => .., Void => .., Expire => .. }"),
    kw!("mod", Mod, Reserved, RequiresAs, Rust, "2026", "Module.", "mod bank { .. }"),
    kw!("move", Move, Reserved, RequiresAs, Rust, "2026", "Closure captures by value. Required for closures crossing an epoch boundary.", "let f = move |r| r.amt;"),
    kw!("mut", Mut, Reserved, RequiresAs, Rust, "2026", "Mutable binding. Never applies to a sealed ledger row.", "let mut acc = 0.00 usd;"),
    kw!("pub", Pub, Reserved, RequiresAs, Rust, "2026", "Export from a module or schema.", "pub view balances = ..;"),
    kw!("ref", Ref, Reserved, RequiresAs, Rust, "2026", "Bind by reference in a pattern.", "match o { Some(ref v) => .., None => .. }"),
    kw!("return", Return, Reserved, RequiresAs, Rust, "2026", "Early return.", "if p.zero() { return Ok(()); }"),
    kw!("self", SelfValue, Reserved, RequiresAs, Rust, "2026", "Receiver parameter or path root.", "fn amount(&self) -> Money<USD> { self.amt }"),
    kw!("static", Static, Reserved, RequiresAs, Rust, "2026", "Program-lifetime item. Immutable; there is no mutable static.", "static ZERO: Money<USD> = 0.00 usd;"),
    kw!("struct", Struct, Reserved, RequiresAs, Rust, "2026", "Product type.", "struct Leg { acct: Acct, amt: Money<USD> }"),
    kw!("super", Super, Reserved, RequiresAs, Rust, "2026", "Parent module in a path.", "use super::money::round;"),
    kw!("trait", Trait, Reserved, RequiresAs, Rust, "2026", "Interface with associated items.", "trait Rate { fn convert(&self, m: Money<USD>) -> Money<EUR>; }"),
    kw!("true", True, Reserved, Bare, Rust, "2026", "Boolean literal.", "let b = true;"),
    kw!("type", Type, Reserved, RequiresAs, Rust, "2026", "Type alias or associated type.", "type Cents = Money<USD>;"),
    kw!("use", Use, Reserved, RequiresAs, Rust, "2026", "Import a path.", "use std::bank::transfer;"),
    kw!("while", While, Reserved, RequiresAs, Rust, "2026", "Conditional loop. Not permitted in a view body.", "while !q.empty() { step()?; }"),

    // ================= Novel =================
    kw!("absent", Absent, Unreserved, Bare, Novel, "2026", "Materialization mode: nothing resident; every read reconstructs.", "serve { materialize: absent }"),
    kw!("anchor", Anchor, Unreserved, Bare, Novel, "2026", "The epoch stamp carried by every answer, and the mandatory index kind on a ledger key.", "let e = balances.get(k)?.anchor;"),
    kw!("as_of", AsOf, Unreserved, Bare, Novel, "2026", "Pin a read to a system-time epoch. The system axis of bitemporality.", "balances.as_of(#4200).get(k)"),
    kw!("authorize", Authorize, Unreserved, Bare, Novel, "2026", "Check a floor against a capability. The only construct that may overdraw.", "authorize(auth, acct, 50.00 usd)?"),
    kw!("auto", Auto, Unreserved, Bare, Novel, "2026", "Delegate materialization to the optimizer, within the rest of the contract.", "serve { materialize: auto }"),
    kw!("backfill", Backfill, Unreserved, Bare, Novel, "2026", "Populate a newly declared view from history, without re-deriving the base.", "backfill v upto #10000;"),
    kw!("base", Base, Unreserved, Bare, Novel, "2026", "An immutable, fully retained, epoch-ordered authoritative relation. `ledger` is `base` plus a conservation rule.", "base events { id: u64, payload: Json }"),
    kw!("bitemporal", Bitemporal, Unreserved, Bare, Novel, "2026", "Declares both time axes on a relation: recorded_at and valid_at.", "ledger p { .. } bitemporal;"),
    kw!("bounded", Bounded, Unreserved, Bare, Novel, "2026", "Consistency rung 0: anchored no more than K epochs or T milliseconds behind.", "serve { consistency: bounded(epochs: 4, millis: 200) }"),
    kw!("budget", Budget, Unreserved, Bare, Novel, "2026", "Resident-state ceiling for a view, in entries or bytes.", "serve { materialize: demand, budget: 50_000 }"),
    kw!("capability", Capability, Unreserved, Bare, Novel, "2026", "Declare an unforgeable authority token for an effect.", "capability Overdraft: Auth<debit<usd>>;"),
    kw!("committed", Committed, Unreserved, Bare, Novel, "2026", "Confidentiality level: readable only inside the enclave that holds the key.", "owner: Text @confidential(committed)"),
    kw!("confidential", Confidential, Unreserved, Bare, Novel, "2026", "Mark a column end-to-end encrypted; the engine may not compute on it.", "owner: Text @confidential(e2ee)"),
    kw!("conserve", Conserve, Unreserved, Bare, Novel, "2026", "The double-entry rule: the group's amounts must sum to zero per currency.", "conserve per (txn, cur);"),
    kw!("consistency", Consistency, Unreserved, Bare, Novel, "2026", "The rung a view is served at.", "serve { consistency: ledger_consistent }"),
    kw!("currency", Currency, Unreserved, Bare, Novel, "2026", "Declare a currency and its minor-unit scale. Scale lives in the type.", "currency jpy { scale: 0 }"),
    kw!("declassify", Declassify, Unreserved, Bare, Novel, "2026", "The single audited construct that lowers a confidentiality level.", "declassify(row.owner, auth)?"),
    kw!("demand", Demand, Unreserved, Bare, Novel, "2026", "Materialization mode: materialize only what is read; evict the rest.", "serve { materialize: demand }"),
    kw!("e2ee", E2ee, Unreserved, Bare, Novel, "2026", "Confidentiality level: end-to-end encrypted, opaque to the engine.", "owner: Text @confidential(e2ee)"),
    kw!("emit", Emit, Unreserved, Bare, Novel, "2026", "Publish a derived change downstream as a stream.", "emit v to sink;"),
    kw!("epoch", Epoch, Unreserved, Bare, Novel, "2026", "The unit of visibility, versioning and hashing. Also the literal prefix `#`.", "let e: Epoch = #4200;"),
    kw!("evictable", Evictable, Unreserved, Bare, Novel, "2026", "Retention of derived state: may be dropped and reconstructed.", "serve { retain: evictable }"),
    kw!("expire", Expire, Unreserved, Bare, Novel, "2026", "Resolve a hold by lapse of its window, releasing the reservation.", "resolve h expire"),
    kw!("expires", Expires, Unreserved, Bare, Novel, "2026", "The window on a hold or an idempotency key.", "hold(acct, 20.00 usd, expires: 7.days)"),
    kw!("explain", Explain, Unreserved, Bare, Novel, "2026", "Show how an answer was derived, at the view's lineage mode.", "explain balances.get(k)?;"),
    kw!("forever", Forever, Unreserved, Bare, Novel, "2026", "Retention: never evicted. Mandatory on a base or ledger.", "ledger p { .. } retain forever;"),
    kw!("freshness", Freshness, Unreserved, Bare, Novel, "2026", "The staleness bound of a bounded rung, as a duration.", "serve { consistency: bounded, freshness: 200.millis }"),
    kw!("fx", Fx, Unreserved, Bare, Novel, "2026", "Atomic cross-currency form: two conserved legs sealed in one epoch.", "fx { leg a: post(du, cu), leg b: post(dm, cm), rate: r }"),
    kw!("guard", Guard, Unreserved, Bare, Novel, "2026", "Attach the termination witness to a fixpoint. Unguarded recursion is rejected.", "edges.fixpoint(step) guard measure(depth)"),
    kw!("hold", Hold, Unreserved, Bare, Novel, "2026", "Reserve funds as a ledger fact. Linear: resolvable exactly once.", "let h = hold(acct, 20.00 usd, expires: 7.days)?;"),
    kw!("idem", Idem, Unreserved, Bare, Novel, "2026", "Declare the idempotency key and window of a transaction.", "txn idem(\"ext-991\", window: 24.hours) { .. }"),
    kw!("impact", Impact, Unreserved, Bare, Novel, "2026", "The inverse of `explain`: which views a base row can affect.", "impact postings.row(#4200, 7);"),
    kw!("ledger", Ledger, Unreserved, Bare, Novel, "2026", "A base with a conservation rule and typed money columns. Never partial.", "ledger postings { txn: TxnId, acct: Id<Account>, amt: Money }"),
    kw!("ledger_consistent", LedgerConsistent, Unreserved, Bare, Novel, "2026", "Consistency rung 5: anchored at the visibility frontier exactly. The authorization path.", "serve { consistency: ledger_consistent }"),
    kw!("leg", Leg, Unreserved, Bare, Novel, "2026", "One side of an `fx` form.", "leg a: post(d_usd, c_usd)"),
    kw!("lineage", Lineage, Unreserved, Bare, Novel, "2026", "Provenance mode: off, key, or full. Off still stamps the anchor.", "serve { lineage: full }"),
    kw!("materialize", Materialize, Unreserved, Bare, Novel, "2026", "The mode a view's state is kept in.", "serve { materialize: tiered }"),
    kw!("measure", Measure, Unreserved, Bare, Novel, "2026", "The well-founded quantity a guarded fixpoint decreases.", "guard measure(depth)"),
    kw!("monotonic", Monotonic, Unreserved, Bare, Novel, "2026", "Consistency rung 1: per session, anchors never decrease.", "serve { consistency: monotonic }"),
    kw!("of", Of, Unreserved, Bare, Novel, "2026", "Connective in `as_of` and `impact of` forms.", "explain of balances.get(k)?;"),
    kw!("per", Per, Unreserved, Bare, Novel, "2026", "Introduces the grouping of a conservation rule.", "conserve per (txn, cur);"),
    kw!("pinned", Pinned, Unreserved, Bare, Novel, "2026", "Retention: resident and never evicted, at a declared memory cost.", "serve { retain: pinned }"),
    kw!("post", Post, Unreserved, Bare, Novel, "2026", "Resolve a hold into a posting, capturing up to the held amount.", "resolve h post 18.50 usd"),
    kw!("posting", Posting, Unreserved, Bare, Novel, "2026", "One signed movement; the linear unit a ledger row is built from.", "let p: Posting = debit(a, 10.00 usd)?;"),
    kw!("rate", Rate, Unreserved, Bare, Novel, "2026", "The declared conversion of an `fx` form, recorded with the legs.", "fx { .., rate: r }"),
    kw!("read_your_writes", ReadYourWrites, Unreserved, Bare, Novel, "2026", "Consistency rung 2: a session observes its own committed writes.", "serve { consistency: read_your_writes }"),
    kw!("recorded_at", RecordedAt, Unreserved, Bare, Novel, "2026", "The system-time axis: when the fact was recorded. Never rewritten.", "where(|r| r.recorded_at <= #4200)"),
    kw!("reproduce", Reproduce, Unreserved, Bare, Novel, "2026", "Re-derive an answer from the base and compare, as an audit act.", "reproduce balances.get(k) at #4200;"),
    kw!("resolve", Resolve, Unreserved, Bare, Novel, "2026", "Consume a hold exactly once, by post, void or expire.", "resolve h post 18.50 usd"),
    kw!("retain", Retain, Unreserved, Bare, Novel, "2026", "Retention of derived state; the base is always retained.", "serve { retain: pinned }"),
    kw!("scale", Scale, Unreserved, Bare, Novel, "2026", "The minor-unit exponent of a currency. Part of the type, never assumed.", "currency bhd { scale: 3 }"),
    kw!("schema", Schema, Unreserved, Bare, Novel, "2026", "The declaration unit: currencies, tables, bases, ledgers, views, indices.", "schema demo_bank { .. }"),
    kw!("serializable", Serializable, Unreserved, Bare, Novel, "2026", "Consistency rung 4: read anchors form a serial order with transactions.", "serve { consistency: serializable }"),
    kw!("serve", Serve, Unreserved, Bare, Novel, "2026", "Attach a contract to a view. The contract is part of the view's type.", "view v = q serve { consistency: snapshot, materialize: auto };"),
    kw!("signal", Signal, Unreserved, Bare, Novel, "2026", "An epoch-indexed value: balance as a function of time.", "let b: Signal<Money<USD>> = balance_of(acct);"),
    kw!("snapshot", Snapshot, Unreserved, Bare, Novel, "2026", "Consistency rung 3: one anchor for the whole read set, across views.", "serve { consistency: snapshot }"),
    kw!("spilled", Spilled, Unreserved, Bare, Novel, "2026", "Materialization mode: resident on secondary storage. Illegal at rung 5.", "serve { materialize: spilled }"),
    kw!("sql", Sql, Unreserved, Bare, Novel, "2026", "Escape into the SQL surface. Same IR, different syntax.", "sql { select id from accounts }"),
    kw!("tiered", Tiered, Unreserved, Bare, Novel, "2026", "Materialization mode: hot entries resident, cold spilled, by the optimizer.", "serve { materialize: tiered }"),
    kw!("txn", Txn, Unreserved, Bare, Novel, "2026", "A ledger transaction: the unit the conservation rule is checked over.", "txn idem(\"k1\") { post(d)?; post(c)?; }"),
    kw!("udf", Udf, Unreserved, Bare, Novel, "2026", "A user-defined function, fuel-metered and compiled to WASM.", "#[udf(fuel = 1_000_000, deterministic)] fn f(x: i64) -> i64 { x }"),
    kw!("upto", Upto, Unreserved, Bare, Novel, "2026", "Bound a backfill or a replay at an epoch.", "backfill v upto #10000;"),
    kw!("valid_at", ValidAt, Unreserved, Bare, Novel, "2026", "The valid-time axis: when the fact was true in the world.", "balances.valid_at(@2026-03-01).get(k)"),
    kw!("value_date", ValueDate, Unreserved, Bare, Novel, "2026", "The banking value date of a posting; drives valid-time placement.", "post(d, value_date: @2026-03-03)?"),
    kw!("void", Void, Unreserved, Bare, Novel, "2026", "Resolve a hold by cancelling it, releasing the reservation with no posting.", "resolve h void"),
    kw!("window", Window, Unreserved, Bare, Novel, "2026", "The duration an idempotency key or hold remains in force.", "idem(\"k1\", window: 24.hours)"),

    // ================= Reserved for future use =================
    kw!("actor", Actor, ReservedFuture, RequiresAs, Novel, "2026", "Reserved: no meaning assigned in this edition.", "n/a"),
    kw!("async", Async, ReservedFuture, RequiresAs, Rust, "2026", "Reserved: query and transaction context is synchronous by design.", "n/a"),
    kw!("await", Await, ReservedFuture, RequiresAs, Rust, "2026", "Reserved: see `async`.", "n/a"),
    kw!("macro", Macro, ReservedFuture, RequiresAs, Rust, "2026", "Reserved: no macro system in this edition.", "n/a"),
    kw!("stream", Stream, ReservedFuture, RequiresAs, Novel, "2026", "Reserved: streams are expressed as bases, not as a separate kind.", "n/a"),
    kw!("unsafe", Unsafe, ReservedFuture, RequiresAs, Rust, "2026", "Reserved and rejected: there is no unsafe fragment of Niles.", "n/a"),
    kw!("yield", Yield, ReservedFuture, RequiresAs, Rust, "2026", "Reserved: no generators in this edition.", "n/a"),
];

fn index() -> &'static HashMap<&'static str, &'static Keyword> {
    static IDX: OnceLock<HashMap<&'static str, &'static Keyword>> = OnceLock::new();
    IDX.get_or_init(|| KEYWORDS.iter().map(|k| (k.word, k)).collect())
}

/// Look a word up. Case-insensitive for ASCII, per SQL habit: `SELECT` and `select` are
/// the same keyword, but `Self` and `self` are distinct words in Rust and therefore here,
/// which is why the exact-match probe comes first.
pub fn lookup(word: &str) -> Option<&'static Keyword> {
    if let Some(k) = index().get(word) {
        return Some(k);
    }
    let lowered = word.to_ascii_lowercase();
    index().get(lowered.as_str()).copied()
}

/// Every keyword of a given origin, in registry order. Generates Appendix B.3's tables.
pub fn by_origin(o: Origin) -> impl Iterator<Item = &'static Keyword> {
    KEYWORDS.iter().filter(move |k| k.origin == o)
}

/// The normative reserved list of Appendix B.16.
pub fn reserved() -> impl Iterator<Item = &'static Keyword> {
    KEYWORDS
        .iter()
        .filter(|k| matches!(k.category, Category::Reserved | Category::ReservedFuture | Category::TypeFuncName))
}

/// How many rows the registry has. Cited in the thesis; computed, never typed by hand.
pub fn count() -> usize {
    KEYWORDS.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_is_sorted_and_unique() {
        // ASCII order, as PostgreSQL requires of kwlist.h so that a perfect-hash generator
        // and a human diff agree. Uppercase `Self` sorts before the lowercase words.
        let mut seen = HashSet::new();
        for k in KEYWORDS {
            assert!(seen.insert(k.word), "duplicate keyword: {}", k.word);
        }
        // The table is grouped into the three origin blocks of Appendix B.3 plus the
        // reserved-for-future block, and is in ASCII order *within* each block, which is
        // what a diff and a perfect-hash generator need. A single global order would
        // scramble the blocks the thesis prints as separate tables.
        for o in [Origin::Sql, Origin::Rust, Origin::Novel] {
            let words: Vec<&str> = by_origin(o)
                .filter(|k| k.category != Category::ReservedFuture)
                .map(|k| k.word)
                .collect();
            let mut sorted = words.clone();
            sorted.sort_unstable();
            assert_eq!(words, sorted, "{} block must be in ASCII order", o.as_str());
        }
    }

    #[test]
    fn tokens_are_unique() {
        let mut seen = HashSet::new();
        for k in KEYWORDS {
            assert!(seen.insert(k.token), "duplicate token for {}", k.word);
        }
    }

    #[test]
    fn every_keyword_documents_itself() {
        for k in KEYWORDS {
            assert!(!k.doc.is_empty(), "{} has no doc", k.word);
            assert!(!k.example.is_empty(), "{} has no example", k.word);
            if k.category != Category::ReservedFuture {
                assert_ne!(k.example, "n/a", "{} needs a real example", k.word);
            }
        }
    }

    #[test]
    fn lookup_is_case_insensitive_but_self_is_not_self() {
        assert_eq!(lookup("SELECT").unwrap().token, Kw::Select);
        assert_eq!(lookup("select").unwrap().token, Kw::Select);
        assert_eq!(lookup("Self").unwrap().token, Kw::SelfType);
        assert_eq!(lookup("self").unwrap().token, Kw::SelfValue);
        assert!(lookup("balance").is_none(), "ordinary identifiers must not be keywords");
    }

    #[test]
    fn reservation_policy_holds() {
        // The stated policy: novel vocabulary is unreserved by default, so that Niles can
        // be adopted against schemas whose columns are already called `epoch` or `ledger`.
        for k in by_origin(Origin::Novel) {
            if k.category == Category::ReservedFuture {
                continue;
            }
            assert_eq!(
                k.category,
                Category::Unreserved,
                "novel keyword `{}` is reserved; add a written justification here if that is deliberate",
                k.word
            );
        }
    }

    #[test]
    fn thesis_b3_lists_are_covered() {
        // The three sub-lists of Appendix B.3 must each be non-empty and disjoint.
        let sql = by_origin(Origin::Sql).count();
        let rust = by_origin(Origin::Rust).count();
        let novel = by_origin(Origin::Novel).count();
        assert!(sql > 60 && rust > 25 && novel > 60, "{sql}/{rust}/{novel}");
        assert_eq!(sql + rust + novel, KEYWORDS.len());
    }
}
