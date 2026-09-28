//! The syntax tree the checker reads: PostgreSQL 16's statements, expressions and PL/pgSQL.
//!
//! Shaped for checking, not for execution: every node keeps its span so a diagnostic points
//! at the text, names keep their quoting (a quoted identifier is never a keyword), and
//! constructs this parser recognises but does not model in detail are kept as
//! [`Stmt::Other`] with their leading keywords and span — *recognised and not analysed*,
//! which the checker reports as such rather than passing silently.

use crate::lex::Span;

/// A possibly-qualified name, each part already case-folded when unquoted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(pub Vec<String>);

impl Name {
    pub fn last(&self) -> &str {
        self.0.last().map(String::as_str).unwrap_or("")
    }
    pub fn dotted(&self) -> String {
        self.0.join(".")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeName {
    pub name: Name,
    /// Type modifiers: `numeric(18, 2)` → [18, 2]; `varchar(10)` → [10].
    pub mods: Vec<Expr>,
    /// Array dimensions: `int[]`, `int[3][]`.
    pub array_dims: usize,
    /// `%TYPE` / `%ROWTYPE` (PL/pgSQL declarations).
    pub percent: Option<String>,
    /// `timestamp with time zone`, `double precision` and the other multi-word names are
    /// normalised into `name` (`timestamptz`, `float8`); the words written are kept here.
    pub written: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(String),
    Num(String),
    Str(String),
    BitStr(String),
    HexStr(String),
    Bool(bool),
    Null,
    /// `interval '1 day'`, `date '2026-01-01'`: a type name applied to a string.
    Typed(Box<TypeName>, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Literal, Span),
    /// A column or variable reference, possibly qualified; `*` as the last part for `t.*`.
    Col(Name, Span),
    Star(Span),
    Param(u32, Span),
    /// A binary operator, including the keyword operators (`and`, `or`, `like`, `is distinct
    /// from`, …), normalised to lower case, and `OPERATOR(schema.op)`.
    Bin(Box<Expr>, String, Box<Expr>, Span),
    /// A prefix operator: `-x`, `not x`, `~x`, `@x`.
    Un(String, Box<Expr>, Span),
    /// `x is null`, `x is not true`, `x is document`, … — the predicate normalised.
    Is(Box<Expr>, String, Span),
    Cast(Box<Expr>, Box<TypeName>, Span),
    Func(Box<FuncCall>),
    Case {
        operand: Option<Box<Expr>>,
        whens: Vec<(Expr, Expr)>,
        otherwise: Option<Box<Expr>>,
        span: Span,
    },
    /// `x between a and b`, with `not` and `symmetric`.
    Between {
        e: Box<Expr>,
        lo: Box<Expr>,
        hi: Box<Expr>,
        negated: bool,
        symmetric: bool,
        span: Span,
    },
    /// `x in (list)`.
    InList(Box<Expr>, Vec<Expr>, bool, Span),
    /// `x in (select …)`, `x op any (select …)`.
    InSub(Box<Expr>, Box<Query>, bool, Span),
    /// `x op any/some/all (array or subquery)`.
    Quantified(Box<Expr>, String, String, Box<Expr>, Span),
    Exists(Box<Query>, Span),
    /// A scalar subquery `(select …)`.
    Sub(Box<Query>, Span),
    /// `array[…]` or `array(select …)`.
    Array(Vec<Expr>, Span),
    ArraySub(Box<Query>, Span),
    /// `row(a, b)` or `(a, b)`.
    Row(Vec<Expr>, Span),
    /// `e[i]`, `e[a:b]`.
    Subscript(Box<Expr>, Box<Option<Expr>>, Box<Option<Expr>>, bool, Span),
    /// `(e).field`, `(e).*`.
    Field(Box<Expr>, String, Span),
    /// `e collate "C"`.
    Collate(Box<Expr>, Name, Span),
    /// `e at time zone z`.
    AtTimeZone(Box<Expr>, Box<Expr>, Span),
    /// `grouping(a, b)` and the other special forms with their own syntax, kept as calls.
    Default(Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Lit(_, s)
            | Expr::Col(_, s)
            | Expr::Star(s)
            | Expr::Param(_, s)
            | Expr::Bin(_, _, _, s)
            | Expr::Un(_, _, s)
            | Expr::Is(_, _, s)
            | Expr::Cast(_, _, s)
            | Expr::InList(_, _, _, s)
            | Expr::InSub(_, _, _, s)
            | Expr::Quantified(_, _, _, _, s)
            | Expr::Exists(_, s)
            | Expr::Sub(_, s)
            | Expr::Array(_, s)
            | Expr::ArraySub(_, s)
            | Expr::Row(_, s)
            | Expr::Subscript(_, _, _, _, s)
            | Expr::Field(_, _, s)
            | Expr::Collate(_, _, s)
            | Expr::AtTimeZone(_, _, s)
            | Expr::Default(s) => *s,
            Expr::Func(f) => f.span,
            Expr::Case { span, .. } | Expr::Between { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncCall {
    pub name: Name,
    /// Positional or named (`name => value`, `name := value`) arguments.
    pub args: Vec<(Option<String>, Expr)>,
    pub star: bool,
    pub distinct: bool,
    pub variadic: bool,
    /// `agg(x order by y)`.
    pub order_by: Vec<OrderItem>,
    /// `within group (order by …)`.
    pub within_group: Vec<OrderItem>,
    pub filter: Option<Expr>,
    pub over: Option<Window>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    /// `over w` or `over (w …)`.
    pub base: Option<String>,
    pub partition_by: Vec<Expr>,
    pub order_by: Vec<OrderItem>,
    /// The frame clause, as written (`rows between unbounded preceding and current row`):
    /// recognised and kept; the checker has no rule that reads it.
    pub frame: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderItem {
    pub expr: Expr,
    pub desc: bool,
    /// `using op`.
    pub using: Option<String>,
    pub nulls_first: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectItem {
    pub expr: Expr,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FromItem {
    Table {
        name: Name,
        only: bool,
        alias: Option<Alias>,
        tablesample: Option<String>,
        span: Span,
    },
    Sub {
        lateral: bool,
        query: Box<Query>,
        alias: Option<Alias>,
        span: Span,
    },
    Func {
        lateral: bool,
        call: Box<Expr>,
        with_ordinality: bool,
        alias: Option<Alias>,
        span: Span,
    },
    Join {
        kind: JoinKind,
        natural: bool,
        left: Box<FromItem>,
        right: Box<FromItem>,
        on: Option<Expr>,
        using: Vec<String>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alias {
    pub name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetOp {
    Union,
    Intersect,
    Except,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryBody {
    Select(Box<Select>),
    Values(Vec<Vec<Expr>>),
    /// `table t`.
    Table(Name),
    SetOp {
        op: SetOp,
        all: bool,
        left: Box<QueryBody>,
        right: Box<QueryBody>,
    },
    /// A parenthesised query used as a set-operation operand.
    Nested(Box<Query>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cte {
    pub name: String,
    pub columns: Vec<String>,
    pub materialized: Option<bool>,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub with: Vec<Cte>,
    pub recursive: bool,
    pub body: QueryBody,
    pub order_by: Vec<OrderItem>,
    pub limit: Option<Expr>,
    /// `fetch first n rows with ties` sets `with_ties`.
    pub with_ties: bool,
    pub offset: Option<Expr>,
    /// `for update`, `for share`, … as written.
    pub locking: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    pub distinct: bool,
    pub distinct_on: Vec<Expr>,
    pub items: Vec<SelectItem>,
    /// `select … into x` (the SQL form; PL/pgSQL's `into` is handled by the PL/pgSQL parser).
    pub into: Vec<Name>,
    /// `into strict` (PL/pgSQL: exactly one row, or an error).
    pub into_strict: bool,
    pub from: Vec<FromItem>,
    pub where_: Option<Expr>,
    /// Plain expressions; `rollup`, `cube`, `grouping sets` and `()` are kept as calls.
    pub group_by: Vec<Expr>,
    pub group_distinct: bool,
    pub having: Option<Expr>,
    pub windows: Vec<(String, Window)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Insert {
    pub with: Vec<Cte>,
    pub table: Name,
    pub alias: Option<String>,
    pub columns: Vec<String>,
    /// `values …`, a query, or `None` for `default values`.
    pub source: Option<Box<Query>>,
    pub on_conflict: Option<OnConflict>,
    pub returning: Vec<SelectItem>,
    pub span: Span,
}

/// `set (a, b) = e` / `set a = e`: the target columns and the value.
pub type Assignment = (Vec<String>, Expr);

#[derive(Debug, Clone, PartialEq)]
pub struct OnConflict {
    pub target: Vec<Expr>,
    pub constraint: Option<String>,
    /// `None` for `do nothing`.
    pub update: Option<(Vec<Assignment>, Option<Expr>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub with: Vec<Cte>,
    pub table: Name,
    pub only: bool,
    pub alias: Option<String>,
    /// `set a = e`, `set (a, b) = (e, f)` / `= (select …)`: target columns and the value.
    pub set: Vec<(Vec<String>, Expr)>,
    pub from: Vec<FromItem>,
    pub where_: Option<Expr>,
    /// `where current of cursor`.
    pub current_of: Option<String>,
    pub returning: Vec<SelectItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Delete {
    pub with: Vec<Cte>,
    pub table: Name,
    pub only: bool,
    pub alias: Option<String>,
    pub using: Vec<FromItem>,
    pub where_: Option<Expr>,
    pub current_of: Option<String>,
    pub returning: Vec<SelectItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub ty: TypeName,
    pub not_null: bool,
    pub default: Option<Expr>,
    pub primary_key: bool,
    pub unique: bool,
    pub check: Option<Expr>,
    pub references: Option<(Name, Vec<String>)>,
    /// `generated always as (e) stored` / `generated … as identity`.
    pub generated: Option<String>,
    pub collate: Option<Name>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableConstraint {
    PrimaryKey(Vec<String>),
    Unique(Vec<String>),
    Check(Expr),
    ForeignKey(Vec<String>, Name, Vec<String>),
    Exclude(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTable {
    pub name: Name,
    pub temporary: bool,
    pub unlogged: bool,
    pub if_not_exists: bool,
    pub columns: Vec<ColumnDef>,
    pub constraints: Vec<(Option<String>, TableConstraint)>,
    /// `create table t as select …`.
    pub as_query: Option<Box<Query>>,
    pub inherits: Vec<Name>,
    pub partition_by: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateView {
    pub name: Name,
    pub or_replace: bool,
    pub materialized: bool,
    pub recursive: bool,
    pub columns: Vec<String>,
    /// `with (security_barrier, check_option = local, …)`: the reloptions, raw key and value.
    pub options: Vec<(String, Option<String>)>,
    pub query: Box<Query>,
    /// `with [cascaded | local] check option`.
    pub check_option: Option<String>,
    /// Materialised views: `with no data`.
    pub with_data: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncArg {
    /// `in`, `out`, `inout`, `variadic`, or `None`.
    pub mode: Option<String>,
    pub name: Option<String>,
    pub ty: TypeName,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FuncBody {
    /// A `as '…'` / `as $$…$$` string, with the language that interprets it.
    Text(String, Span),
    /// SQL-standard body: `begin atomic … end` or `return e`.
    Atomic(Vec<Stmt>),
    Return(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateFunction {
    pub name: Name,
    pub or_replace: bool,
    pub procedure: bool,
    pub args: Vec<FuncArg>,
    /// `returns t`, `returns setof t`, `returns table (…)`, `returns trigger`.
    pub returns: Option<Returns>,
    pub language: Option<String>,
    /// `immutable` / `stable` / `volatile`.
    pub volatility: Option<String>,
    pub strict: bool,
    pub security_definer: bool,
    pub leakproof: bool,
    pub parallel: Option<String>,
    /// Other attributes (`cost`, `rows`, `set …`, `support`, `window`), as written.
    pub other: Vec<String>,
    pub body: Option<FuncBody>,
    /// When the language is `plpgsql` and the body is text: the parsed block.
    pub plpgsql: Option<crate::plpgsql::Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Returns {
    Type { setof: bool, ty: TypeName },
    Table(Vec<(String, TypeName)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTrigger {
    pub name: String,
    pub or_replace: bool,
    pub constraint: bool,
    /// `before`, `after`, `instead of`.
    pub timing: String,
    /// `insert`, `update [of cols]`, `delete`, `truncate`.
    pub events: Vec<String>,
    pub table: Name,
    pub deferrable: bool,
    pub initially_deferred: bool,
    pub for_each_row: bool,
    pub when: Option<Expr>,
    pub function: Name,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Query(Box<Query>),
    Insert(Box<Insert>),
    Update(Box<Update>),
    Delete(Box<Delete>),
    CreateTable(Box<CreateTable>),
    CreateView(Box<CreateView>),
    CreateFunction(Box<CreateFunction>),
    CreateTrigger(Box<CreateTrigger>),
    CreateIndex {
        name: Option<String>,
        unique: bool,
        table: Name,
        columns: Vec<OrderItem>,
        where_: Option<Expr>,
        span: Span,
    },
    CreateType {
        name: Name,
        /// Composite: `as (a t, …)`; enum: `as enum ('a', …)`; others as written.
        kind: String,
        attributes: Vec<(String, TypeName)>,
        labels: Vec<String>,
        span: Span,
    },
    CreateDomain {
        name: Name,
        ty: TypeName,
        checks: Vec<Expr>,
        not_null: bool,
        default: Option<Expr>,
        span: Span,
    },
    CreateSchema {
        name: String,
        if_not_exists: bool,
        span: Span,
    },
    /// `comment on column t.c is '…'` and the other `comment on` forms: object kind, name, text.
    Comment {
        kind: String,
        target: Name,
        text: Option<String>,
        span: Span,
    },
    /// `drop <kind> [if exists] names [cascade|restrict]`.
    Drop {
        kind: String,
        names: Vec<Name>,
        if_exists: bool,
        cascade: bool,
        span: Span,
    },
    /// `alter table t disable trigger x` and every other `alter`: object kind, name, and the
    /// action text, which the checker scans for the few actions it has rules about.
    Alter {
        kind: String,
        name: Name,
        action: String,
        span: Span,
    },
    /// `truncate [table] [only] t, … [restart identity] [cascade]`.
    Truncate {
        tables: Vec<Name>,
        cascade: bool,
        span: Span,
    },
    Merge(Box<Merge>),
    /// `call proc(args)`.
    Call(Box<Expr>, Span),
    /// `explain [(options)] [analyze] [verbose] statement`.
    Explain {
        options: String,
        stmt: Box<Stmt>,
        span: Span,
    },
    /// `grant … on … to …` and `revoke … on … from …`; a role grant has kind `role`.
    Grant {
        revoke: bool,
        privileges: Vec<String>,
        kind: String,
        objects: Vec<Name>,
        grantees: Vec<String>,
        span: Span,
    },
    /// `create sequence` (options as written).
    CreateSequence {
        name: Name,
        options: String,
        span: Span,
    },
    /// `create [or replace] rule r as on event to t [where c] do [also | instead]
    /// { nothing | command | ( commands ) }` — the commands parsed, because a rule can turn
    /// an insert on a ledger into an update.
    CreateRule {
        name: String,
        event: String,
        table: Name,
        where_: Option<Expr>,
        instead: bool,
        actions: Vec<Stmt>,
        span: Span,
    },
    /// The objects PostgreSQL defines with a `(key = value, …)` list or a fixed clause:
    /// `create operator`, `aggregate`, `text search …`, `collation`, `cast`,
    /// `operator class`, `operator family`, `access method`, `extension`, `policy`,
    /// `role`, `server`, … — kind, name and the definition's items as written.
    Define {
        kind: String,
        name: Name,
        items: Vec<(String, String)>,
        span: Span,
    },
    /// `do [language plpgsql] $$ … $$` — an anonymous PL/pgSQL block, parsed like a body.
    Do {
        block: Box<crate::plpgsql::Block>,
        span: Span,
    },
    /// `begin`, `commit`, `rollback`, `savepoint`, `set`, `grant`, … — recognised by
    /// their leading keywords, kept with their text.
    Other {
        keywords: Vec<String>,
        text: String,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Query(q) => q.span,
            Stmt::Insert(s) => s.span,
            Stmt::Update(s) => s.span,
            Stmt::Delete(s) => s.span,
            Stmt::CreateTable(s) => s.span,
            Stmt::CreateView(s) => s.span,
            Stmt::CreateFunction(s) => s.span,
            Stmt::CreateTrigger(s) => s.span,
            Stmt::Merge(m) => m.span,
            Stmt::CreateIndex { span, .. }
            | Stmt::CreateType { span, .. }
            | Stmt::CreateDomain { span, .. }
            | Stmt::CreateSchema { span, .. }
            | Stmt::Comment { span, .. }
            | Stmt::Drop { span, .. }
            | Stmt::Alter { span, .. }
            | Stmt::Do { span, .. }
            | Stmt::Truncate { span, .. }
            | Stmt::Call(_, span)
            | Stmt::Explain { span, .. }
            | Stmt::Grant { span, .. }
            | Stmt::CreateSequence { span, .. }
            | Stmt::CreateRule { span, .. }
            | Stmt::Define { span, .. }
            | Stmt::Other { span, .. } => *span,
        }
    }
}

/// `merge into t [as a] using source on cond when [not] matched [and c] then action …`.
#[derive(Debug, Clone, PartialEq)]
pub struct Merge {
    pub with: Vec<Cte>,
    pub table: Name,
    pub alias: Option<String>,
    pub source: FromItem,
    pub on: Expr,
    pub whens: Vec<MergeWhen>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MergeWhen {
    pub matched: bool,
    pub condition: Option<Expr>,
    pub action: MergeAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergeAction {
    Update(Vec<(Vec<String>, Expr)>),
    Delete,
    Insert {
        columns: Vec<String>,
        values: Option<Vec<Expr>>,
    },
    DoNothing,
}
