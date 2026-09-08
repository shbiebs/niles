//! The Niles surface syntax tree.
//!
//! This is the *raw parse* level, in PostgreSQL's sense: it records what was written, not
//! what it means. Nothing here is resolved against a catalog, because in Nilestream a
//! catalog lookup is only meaningful relative to a visibility frontier — a name resolves
//! *at an epoch*. PostgreSQL separates raw parsing from analysis because catalog lookups
//! require a transaction; Niles has the strictly stronger version of that constraint, so
//! the split is forced by the consistency model rather than adopted as a style. A parse is
//! therefore total, pure and epoch-free, and every judgement that needs a schema lives in
//! `resolve`, `typecheck`, `currency_rows` and `effects`.
//!
//! Consequences that show up all over this file: a `Name` is a string and a span, never a
//! resolved id; `StageKind::Unknown` exists so an unrecognised pipeline stage parses and is
//! diagnosed later with a suggestion; and [`Expr::Error`] and [`Item::Error`] are ordinary
//! variants, so a file with a syntax error still yields a tree covering the rest of it.

use crate::lexer::{Span, TimeUnit};

/// An identifier as written, with its span. Resolution happens later, at an epoch.
#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

impl Name {
    pub fn new(text: impl Into<String>, span: Span) -> Self {
        Name {
            text: text.into(),
            span,
        }
    }
}

/// `a::b::c`
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<Name>,
    pub span: Span,
}

impl Path {
    pub fn is_ident(&self, s: &str) -> bool {
        self.segments.len() == 1 && self.segments[0].text == s
    }
    pub fn last(&self) -> &Name {
        self.segments.last().expect("paths are never empty")
    }
}

// ============================ types ============================

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    /// `i64`, `Text`, `Money<USD>`, `Signal<Money<USD>>`, `Id<Account>`
    Path {
        path: Path,
        args: Vec<Ty>,
        span: Span,
    },
    /// `&T` / `&mut T`
    Ref {
        inner: Box<Ty>,
        mutable: bool,
        span: Span,
    },
    Tuple {
        elems: Vec<Ty>,
        span: Span,
    },
    Slice {
        elem: Box<Ty>,
        span: Span,
    },
    Array {
        elem: Box<Ty>,
        len: Box<Expr>,
        span: Span,
    },
    /// `fn(A, B) -> C ! { e1, e2 }`. The effect row is part of the type, not a comment.
    Fn {
        params: Vec<Ty>,
        ret: Box<Ty>,
        effects: EffectRow,
        span: Span,
    },
    /// `dyn Trait`
    Dyn {
        path: Path,
        span: Span,
    },
    Unit(Span),
    Infer(Span),
    Error(Span),
}

impl Ty {
    pub fn span(&self) -> Span {
        match self {
            Ty::Path { span, .. }
            | Ty::Ref { span, .. }
            | Ty::Tuple { span, .. }
            | Ty::Slice { span, .. }
            | Ty::Array { span, .. }
            | Ty::Fn { span, .. }
            | Ty::Dyn { span, .. }
            | Ty::Unit(span)
            | Ty::Infer(span)
            | Ty::Error(span) => *span,
        }
    }
}

/// An effect row as written: `! { read@snapshot, append, debit<usd> }`.
///
/// Rows are open by default (a row variable is implied) so that a function can be used in
/// a context permitting more effects; `effects::solve` decides whether a particular use
/// closes the row.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectRow {
    pub effects: Vec<Effect>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Effect {
    pub name: Name,
    /// `read@snapshot` — the rung an effect is qualified by, where it takes one.
    pub at: Option<Name>,
    /// `debit<usd>` — the currency or type an effect is parameterised by.
    pub args: Vec<Name>,
    pub span: Span,
}

// ============================ items ============================

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Schema(SchemaDecl),
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Trait(TraitDecl),
    Impl(ImplDecl),
    Mod {
        name: Name,
        items: Vec<Item>,
        span: Span,
    },
    Use {
        path: Path,
        span: Span,
    },
    Const {
        name: Name,
        ty: Ty,
        value: Expr,
        is_static: bool,
        span: Span,
    },
    TypeAlias {
        name: Name,
        ty: Ty,
        span: Span,
    },
    Capability {
        name: Name,
        ty: Ty,
        span: Span,
    },
    /// A top-level view, outside any schema.
    View(ViewDecl),
    /// A parse error covering the tokens the parser skipped. Keeping it in the tree, rather
    /// than dropping the item, is what lets one bad declaration coexist with a good one.
    Error(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaDecl {
    pub name: Name,
    pub items: Vec<SchemaItem>,
    pub attrs: Vec<Attr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaItem {
    Currency(CurrencyDecl),
    /// `table` — mutable, no retained history, `update`/`delete` legal.
    Table(RelDecl),
    /// `base` / `ledger` — immutable, fully retained, epoch-ordered. A `ledger` is a
    /// `base` that additionally carries a conservation rule and money columns.
    Base(RelDecl),
    View(ViewDecl),
    Index(IndexDecl),
    Error(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurrencyDecl {
    pub name: Name,
    /// The minor-unit exponent. Carried in the type, never assumed to be 2.
    pub scale: u32,
    pub scale_span: Span,
    pub span: Span,
}

/// Whether a relation is mutable or an immutable, retained base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelKind {
    Table,
    Base,
    Ledger,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelDecl {
    pub kind: RelKind,
    pub name: Name,
    pub fields: Vec<FieldDecl>,
    pub rules: Vec<RelRule>,
    pub attrs: Vec<Attr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: Name,
    pub ty: Ty,
    pub primary_key: bool,
    pub unique: bool,
    pub default: Option<Expr>,
    /// `idem: IdemKey window 1_000_000.epochs` — the idempotency window declared on the
    /// column.
    ///
    /// **Its own field, because it used to share `default`'s.** The parser wrote both
    /// clauses into `default`, so `IdemKey default "x"` satisfied the only check that an
    /// idempotency key has a window (NL0215), and a column declaring both kept whichever
    /// came last. The check on the one construct whose absence is a business rule was
    /// vacuous in the presence of an unrelated clause.
    pub window: Option<Expr>,
    pub check: Option<Expr>,
    pub attrs: Vec<Attr>,
    pub span: Span,
}

/// A declaration-level rule attached to a relation.
#[derive(Debug, Clone, PartialEq)]
pub enum RelRule {
    /// `conserve per (txn, cur);` — the double-entry invariant, as a checkable rule.
    Conserve {
        keys: Vec<Name>,
        span: Span,
    },
    /// `retain forever;` — mandatory on a base or ledger.
    Retain {
        mode: Name,
        span: Span,
    },
    /// `bitemporal;` — declares both time axes.
    Bitemporal {
        span: Span,
    },
    /// `foreign key (a) references t (b)`
    ForeignKey {
        cols: Vec<Name>,
        target: Name,
        target_cols: Vec<Name>,
        span: Span,
    },
    /// `primary key (a, b)` in table-constraint position.
    PrimaryKey {
        cols: Vec<Name>,
        span: Span,
    },
    Error(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexDecl {
    pub name: Name,
    pub on: Name,
    pub cols: Vec<Name>,
    /// `anchor` — an anchor index, mandatory on ledger keys because reconstruction needs
    /// to reach a key's rows without scanning the ledger.
    pub anchor: bool,
    pub span: Span,
}

/// `view v = <pipeline> serve { .. };` — the REV of the theory, as a declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewDecl {
    pub name: Name,
    pub body: Expr,
    pub contract: Option<ServeContract>,
    pub public: bool,
    pub attrs: Vec<Attr>,
    pub span: Span,
}

/// The serve contract, as written. Values are still surface syntax here: `consistency:
/// bounded(epochs: 4)` is a `ContractValue::Call`, and only `resolve` turns it into an IR
/// contract, because the legality of a combination depends on the whole declaration.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ServeContract {
    pub entries: Vec<(Name, ContractValue)>,
    pub span: Span,
}

impl ServeContract {
    pub fn get(&self, key: &str) -> Option<&ContractValue> {
        self.entries
            .iter()
            .find(|(k, _)| k.text == key)
            .map(|(_, v)| v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractValue {
    Word(Name),
    Int(i128, Span),
    Duration {
        value: i128,
        unit: TimeUnit,
        span: Span,
    },
    /// `bounded(epochs: 4, millis: 200)`
    Call {
        name: Name,
        args: Vec<(Option<Name>, ContractValue)>,
        span: Span,
    },
    Error(Span),
}

impl ContractValue {
    pub fn span(&self) -> Span {
        match self {
            ContractValue::Word(n) => n.span,
            ContractValue::Int(_, s)
            | ContractValue::Duration { span: s, .. }
            | ContractValue::Call { span: s, .. }
            | ContractValue::Error(s) => *s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: Vec<Param>,
    pub ret: Option<Ty>,
    /// The declared effect row. `None` means "infer and report", not "no effects".
    pub effects: Option<EffectRow>,
    pub body: Option<Block>,
    pub public: bool,
    pub attrs: Vec<Attr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub pat: Pat,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub fields: Vec<FieldDecl>,
    pub public: bool,
    pub attrs: Vec<Attr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub variants: Vec<(Name, Vec<Ty>)>,
    pub public: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub items: Vec<FnDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplDecl {
    pub trait_: Option<Path>,
    pub self_ty: Ty,
    pub items: Vec<FnDecl>,
    pub span: Span,
}

/// `#[udf(fuel = 1_000_000, deterministic)]` or `@confidential(e2ee)`.
///
/// Two spellings for one concept because they come from two lineages, and collapsing them
/// would make `@confidential` — which is a *type-level* obligation, not a hint — look like
/// an ordinary attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct Attr {
    pub name: Name,
    pub args: Vec<AttrArg>,
    pub at_style: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrArg {
    Word(Name),
    KeyValue(Name, Expr),
}

// ============================ statements and blocks ============================

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    /// The block's value, if it ends in an expression without a semicolon.
    pub tail: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        pat: Pat,
        ty: Option<Ty>,
        init: Option<Expr>,
        span: Span,
    },
    Expr(Expr),
    Semi(Expr),
    Item(Box<Item>),
    /// SQL-surface DML, which is a statement rather than an expression because it has no
    /// value and because `update`/`delete` are legal only against a `table`.
    Dml(Dml),
    Error(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Dml {
    Insert {
        table: Name,
        cols: Vec<Name>,
        rows: Vec<Vec<Expr>>,
        span: Span,
    },
    Update {
        table: Name,
        sets: Vec<(Name, Expr)>,
        filter: Option<Expr>,
        span: Span,
    },
    Delete {
        table: Name,
        filter: Option<Expr>,
        span: Span,
    },
    Begin(Span),
    Commit(Span),
    Rollback(Span),
    Grant {
        effect: Effect,
        on: Name,
        to: Name,
        span: Span,
    },
    Revoke {
        effect: Effect,
        on: Name,
        from: Name,
        span: Span,
    },
    Backfill {
        view: Name,
        upto: Option<Expr>,
        span: Span,
    },
    Emit {
        view: Name,
        to: Name,
        span: Span,
    },
}

// ============================ patterns ============================

#[derive(Debug, Clone, PartialEq)]
pub enum Pat {
    Wild(Span),
    Bind {
        name: Name,
        mutable: bool,
        by_ref: bool,
        span: Span,
    },
    Tuple {
        elems: Vec<Pat>,
        span: Span,
    },
    /// `Some(x)`, `Outcome::Post(m)`
    TupleStruct {
        path: Path,
        elems: Vec<Pat>,
        span: Span,
    },
    Struct {
        path: Path,
        fields: Vec<(Name, Pat)>,
        rest: bool,
        span: Span,
    },
    Lit(Box<Expr>),
    Path(Path),
    Error(Span),
}

impl Pat {
    pub fn span(&self) -> Span {
        match self {
            Pat::Wild(s) | Pat::Error(s) => *s,
            Pat::Bind { span, .. }
            | Pat::Tuple { span, .. }
            | Pat::TupleStruct { span, .. }
            | Pat::Struct { span, .. } => *span,
            Pat::Lit(e) => e.span(),
            Pat::Path(p) => p.span,
        }
    }
    /// Every name this pattern binds, for the linearity check.
    pub fn bindings(&self) -> Vec<&Name> {
        let mut out = Vec::new();
        self.collect(&mut out);
        out
    }
    fn collect<'a>(&'a self, out: &mut Vec<&'a Name>) {
        match self {
            Pat::Bind { name, .. } => out.push(name),
            Pat::Tuple { elems, .. } | Pat::TupleStruct { elems, .. } => {
                elems.iter().for_each(|p| p.collect(out))
            }
            Pat::Struct { fields, .. } => fields.iter().for_each(|(_, p)| p.collect(out)),
            _ => {}
        }
    }
}

// ============================ expressions ============================

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // --- leaves ---
    Int(i128, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Str(String, Span),
    Bytes(Vec<u8>, Span),
    Unit(Span),
    /// `null` — the SQL null, and the only absence with a *value* form.
    ///
    /// It had no expression form at all, so `where(|r| r.n is null)` did not parse: the
    /// keyword was reserved, listed in the registry with `is null` as its own example, and
    /// unspellable. Kept distinct from `Option::None` and from an evicted `Hole` — the three
    /// absences the lattice of §3.3 turns on keeping apart.
    Null(Span),
    /// `10.00 usd` — the minor value at the literal's own scale, plus that scale. The
    /// scale is kept separate from the currency so that `10.001 usd` is a *scale* error
    /// naming both numbers, rather than a silent rounding.
    Money {
        minor: i128,
        scale: u32,
        currency: Name,
        span: Span,
    },
    /// `#4200`
    Epoch(u64, Span),
    /// `@2026-03-01` (system axis) / `v@2026-03-01` (valid-time axis)
    Instant {
        text: String,
        valid_axis: bool,
        span: Span,
    },
    Duration {
        value: i128,
        unit: TimeUnit,
        span: Span,
    },
    Path(Path),

    // --- composition ---
    Tuple {
        elems: Vec<Expr>,
        span: Span,
    },
    Array {
        elems: Vec<Expr>,
        span: Span,
    },
    StructLit {
        path: Path,
        fields: Vec<(Name, Expr)>,
        span: Span,
    },
    Field {
        base: Box<Expr>,
        name: Name,
        span: Span,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Arg>,
        span: Span,
    },
    /// A pipeline stage: `q.where(|r| p)`, `q |> where(|r| p)`. Both spellings produce
    /// this node; the `|>` form exists so a long query reads top-to-bottom.
    Stage {
        recv: Box<Expr>,
        kind: StageKind,
        name: Name,
        args: Vec<Arg>,
        span: Span,
    },
    Closure {
        params: Vec<(Pat, Option<Ty>)>,
        body: Box<Expr>,
        is_move: bool,
        span: Span,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
        span: Span,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    Cast {
        expr: Box<Expr>,
        ty: Ty,
        span: Span,
    },
    /// `e?` — propagate a `Result` error. The only non-local exit in query context.
    Try {
        expr: Box<Expr>,
        span: Span,
    },

    // --- control ---
    Block(Box<Block>),
    If {
        cond: Box<Expr>,
        then: Box<Block>,
        els: Option<Box<Expr>>,
        span: Span,
    },
    /// SQL's `case when .. then .. else .. end`. Kept distinct from `match` because it is
    /// not exhaustive-checked: it is an expression over predicates, not over constructors.
    Case {
        arms: Vec<(Expr, Expr)>,
        els: Option<Box<Expr>>,
        span: Span,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    While {
        cond: Box<Expr>,
        body: Box<Block>,
        span: Span,
    },
    Loop {
        body: Box<Block>,
        span: Span,
    },
    For {
        pat: Pat,
        iter: Box<Expr>,
        body: Box<Block>,
        span: Span,
    },
    Return {
        value: Option<Box<Expr>>,
        span: Span,
    },
    Break(Span),
    Continue(Span),

    // --- the novel forms ---
    /// `txn idem("k") { .. }` — the unit the conservation rule is
    /// checked over, and the unit an epoch seals.
    Txn {
        idem: Option<IdemSpec>,
        body: Box<Block>,
        span: Span,
    },
    /// `hold(acct, 20.00 usd, expires: 7.days)`
    Hold {
        args: Vec<Arg>,
        span: Span,
    },
    /// `resolve h post 18.50 usd` / `resolve h void` / `resolve h expire`
    Resolve {
        hold: Box<Expr>,
        outcome: ResolveOutcome,
        span: Span,
    },
    /// `fx { leg a: post(..), leg b: post(..), rate: r }` — two conserved legs sealed in
    /// one epoch, which is what makes cross-currency movement atomic without a currency
    /// that both sides share.
    Fx {
        legs: Vec<(Name, Expr)>,
        rate: Option<Box<Expr>>,
        span: Span,
    },
    /// `q.fixpoint(step) guard measure(depth)` — recursion with its termination witness
    /// attached. Unguarded recursion has no spelling.
    Fixpoint {
        recv: Box<Expr>,
        step: Box<Expr>,
        measure: Box<Expr>,
        span: Span,
    },
    /// `.as_of(#4200)`, `.valid_at(@2026-03-01)` — parsed as stages, kept as such.
    /// `authorize(auth, acct, amount)` — the only construct permitted to approach a floor.
    Authorize {
        args: Vec<Arg>,
        span: Span,
    },
    /// `declassify(x, auth)` — the single audited construct that lowers confidentiality.
    Declassify {
        args: Vec<Arg>,
        span: Span,
    },
    /// `explain e` / `reproduce e at #4200` / `impact r`
    Explain {
        target: Box<Expr>,
        span: Span,
    },
    Reproduce {
        target: Box<Expr>,
        at: Option<Box<Expr>>,
        span: Span,
    },
    Impact {
        target: Box<Expr>,
        span: Span,
    },
    /// `sql { select .. }` — the SQL surface, lowering to the same IR.
    Sql {
        inner: Box<Expr>,
        span: Span,
    },
    /// `exists (select ..)` — a subquery in predicate position.
    ///
    /// `not exists (..)` has no variant of its own: it parses as `Unary { Not, Exists }`,
    /// because `not` is an ordinary prefix operator and giving the pair a fused node would
    /// mean two spellings of one thing in the tree. Lowering matches on the pair.
    Exists {
        query: Box<SelectStmt>,
        span: Span,
    },
    /// A SQL `select` as written, before it is rewritten into a pipeline.
    Select(Box<SelectStmt>),

    /// A parse error. Its span covers the tokens that were skipped.
    Error(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    /// `expires: 7.days` — named arguments, because a five-argument `hold` whose third
    /// argument is a duration is unreadable positionally.
    pub name: Option<Name>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdemSpec {
    pub key: Box<Expr>,
    pub window: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolveOutcome {
    Post(Box<Expr>),
    Void(Span),
    Expire(Span),
    Error(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pat: Pat,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

/// A SQL `select`, as written. Preserved in SQL's *written* order rather than normalised,
/// so that a diagnostic can point at the clause the user typed.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStmt {
    pub distinct: bool,
    pub projections: Vec<(Expr, Option<Name>)>,
    pub from: Vec<TableRef>,
    pub filter: Option<Expr>,
    pub group_by: Vec<Expr>,
    pub having: Option<Expr>,
    pub order_by: Vec<(Expr, bool)>,
    pub limit: Option<Expr>,
    pub offset: Option<Expr>,
    pub set_op: Option<(SetOp, Box<SelectStmt>)>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOp {
    Union,
    UnionAll,
    Except,
    Intersect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableRef {
    Named {
        name: Name,
        alias: Option<Name>,
        span: Span,
    },
    Join {
        left: Box<TableRef>,
        right: Box<TableRef>,
        kind: JoinKind,
        on: Option<Expr>,
        span: Span,
    },
    Sub {
        query: Box<SelectStmt>,
        alias: Option<Name>,
        span: Span,
    },
}

impl TableRef {
    pub fn span(&self) -> Span {
        match self {
            TableRef::Named { span, .. }
            | TableRef::Join { span, .. }
            | TableRef::Sub { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// The pipeline stages the language knows. `Unknown` is not a failure: an unrecognised
/// stage parses, and `resolve` reports it with a suggestion from this list, which is a far
/// better experience than a parse error pointing at a dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageKind {
    Where,
    Map,
    Select,
    GroupBy,
    Having,
    Join,
    LeftJoin,
    RightJoin,
    FullOuterJoin,
    CrossJoin,
    Union,
    UnionAll,
    Except,
    Intersect,
    Distinct,
    DistinctBy,
    OrderBy,
    Limit,
    Offset,
    Sum,
    Count,
    Min,
    Max,
    Avg,
    Fold,
    Fixpoint,
    AsOf,
    ValidAt,
    Get,
    Range,
    /// A method call that is not a known stage — an ordinary method, or a typo.
    Unknown,
}

impl StageKind {
    pub fn from_name(s: &str) -> StageKind {
        use StageKind::*;
        match s {
            "where" | "filter" => Where,
            "map" => Map,
            "select" => Select,
            "group_by" => GroupBy,
            "having" => Having,
            "join" => Join,
            "left_join" => LeftJoin,
            "right_join" => RightJoin,
            "full_outer_join" => FullOuterJoin,
            "cross_join" => CrossJoin,
            "union" => Union,
            "union_all" => UnionAll,
            "except" => Except,
            "intersect" => Intersect,
            "distinct" => Distinct,
            "distinct_by" => DistinctBy,
            "order_by" => OrderBy,
            "limit" => Limit,
            "offset" => Offset,
            "sum" => Sum,
            "count" => Count,
            "min" => Min,
            "max" => Max,
            "avg" => Avg,
            "fold" => Fold,
            "fixpoint" => Fixpoint,
            "as_of" => AsOf,
            "valid_at" => ValidAt,
            "get" => Get,
            "range" => Range,
            _ => Unknown,
        }
    }

    /// The known stage names, for "did you mean" suggestions.
    pub fn all_names() -> &'static [&'static str] {
        &[
            "where",
            "filter",
            "map",
            "select",
            "group_by",
            "having",
            "join",
            "left_join",
            "right_join",
            "full_outer_join",
            "cross_join",
            "union",
            "union_all",
            "except",
            "intersect",
            "distinct",
            "distinct_by",
            "order_by",
            "limit",
            "offset",
            "sum",
            "count",
            "min",
            "max",
            "avg",
            "fold",
            "fixpoint",
            "as_of",
            "valid_at",
            "get",
            "range",
        ]
    }

    /// Whether the stage is *incrementally maintainable* over a Z-set without retaining
    /// the whole input. `order_by`/`limit` are not, which is why a view ending in them
    /// cannot be served at rung 5 without full materialization — a fact the planner needs
    /// and the surface syntax must not hide.
    pub fn is_incremental(self) -> bool {
        !matches!(
            self,
            StageKind::OrderBy | StageKind::Limit | StageKind::Offset
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    Deref,
    Ref,
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    /// `is null` / `is not null`
    Is,
    IsNot,
    In,
    NotIn,
    Like,
    Between,
}

impl BinOp {
    /// Binding power for the Pratt loop. Higher binds tighter.
    pub fn precedence(self) -> u8 {
        use BinOp::*;
        match self {
            Or => 1,
            And => 2,
            Eq | Ne | Lt | Le | Gt | Ge | Is | IsNot | In | NotIn | Like | Between => 3,
            BitOr => 4,
            BitXor => 5,
            BitAnd => 6,
            Add | Sub => 7,
            Mul | Div | Rem => 8,
        }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        use Expr::*;
        match self {
            Int(_, s)
            | Float(_, s)
            | Bool(_, s)
            | Str(_, s)
            | Bytes(_, s)
            | Unit(s)
            | Null(s)
            | Epoch(_, s)
            | Break(s)
            | Continue(s)
            | Error(s) => *s,
            Money { span, .. }
            | Instant { span, .. }
            | Duration { span, .. }
            | Tuple { span, .. }
            | Array { span, .. }
            | StructLit { span, .. }
            | Field { span, .. }
            | Index { span, .. }
            | Call { span, .. }
            | Stage { span, .. }
            | Closure { span, .. }
            | Unary { span, .. }
            | Binary { span, .. }
            | Assign { span, .. }
            | Cast { span, .. }
            | Try { span, .. }
            | If { span, .. }
            | Case { span, .. }
            | Match { span, .. }
            | While { span, .. }
            | Loop { span, .. }
            | For { span, .. }
            | Return { span, .. }
            | Txn { span, .. }
            | Hold { span, .. }
            | Resolve { span, .. }
            | Fx { span, .. }
            | Fixpoint { span, .. }
            | Authorize { span, .. }
            | Declassify { span, .. }
            | Explain { span, .. }
            | Reproduce { span, .. }
            | Impact { span, .. }
            | Sql { span, .. } => *span,
            Exists { span, .. } => *span,
            Path(p) => p.span,
            Block(b) => b.span,
            Select(s) => s.span,
        }
    }

    /// Whether this expression contains a parse error anywhere. Used to suppress cascading
    /// type errors: a tree that already failed to parse should not also be reported as
    /// ill-typed, because the second message is almost always noise.
    pub fn has_error(&self) -> bool {
        matches!(self, Expr::Error(_))
    }
}
