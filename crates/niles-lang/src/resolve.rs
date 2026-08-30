//! Name resolution and the catalog — the *analysis* half of the parse/analyse split.
//!
//! Everything the parser deliberately declined to know is decided here: whether a name
//! denotes a table, a ledger, a view or a column; whether a currency exists and at what
//! scale; whether a view's contract is a combination the engine can actually serve.
//!
//! # Why this is a separate phase, and why it takes an epoch
//!
//! PostgreSQL separates raw parsing from semantic analysis because "system catalog lookups
//! can only be done within a transaction, and we do not wish to start a transaction
//! immediately upon receiving a query string." Nilestream has the stronger version of that
//! constraint: **a name resolves only relative to a visibility frontier.** `postings` at
//! epoch 4,200 and `postings` at epoch 9,000 may have different columns, because a
//! migration is itself a ledger fact. So [`Catalog::at`] carries the epoch it was read at,
//! and a resolution is only meaningful together with that epoch. A compiler that resolved
//! names without one would be answering a question that has no answer.
//!
//! For a single-file compilation the epoch is the head of the schema's own declarations,
//! which is why `resolve_program` can be called without a running engine — but the
//! signature keeps the epoch explicit so that the server path and the offline path are the
//! same code.

use crate::ast::*;
use crate::diagnostics::{closest, Applicability, Diagnostic, Diagnostics};
use crate::effects::Rung;
use crate::lexer::Span;
use std::collections::HashMap;

/// The epoch a catalog was read at. A resolution is a statement about the schema *as of*
/// this epoch and no other.
pub type Epoch = u64;

#[derive(Debug, Clone)]
pub struct CurrencyInfo {
    pub name: String,
    /// The minor-unit exponent. Carried, never assumed: JPY is 0, most are 2, BHD is 3.
    pub scale: u32,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RelationInfo {
    pub name: String,
    pub kind: RelKind,
    pub columns: Vec<ColumnInfo>,
    /// The conservation grouping, if any. Present exactly on ledgers (W2).
    pub conserve_keys: Vec<String>,
    pub conserve_span: Option<Span>,
    pub retain_forever: bool,
    pub bitemporal: bool,
    /// Anchor-indexed column sets on this relation (W12).
    pub anchor_indices: Vec<Vec<String>>,
    pub span: Span,
}

impl RelationInfo {
    pub fn is_base(&self) -> bool {
        matches!(self.kind, RelKind::Base | RelKind::Ledger)
    }
    pub fn column(&self, name: &str) -> Option<&ColumnInfo> {
        self.columns.iter().find(|c| c.name == name)
    }
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub ty: Ty,
    /// `Some(c)` when the column's type is `Money<c>`; `None` for a currency-generic
    /// `Money`, whose currency comes from a sibling `Currency` column.
    pub money_currency: Option<Option<String>>,
    pub confidential: Option<String>,
    pub idem_window: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ViewInfo {
    pub name: String,
    pub rung: Rung,
    pub materialize: String,
    pub retain: String,
    pub lineage: String,
    pub contract_span: Span,
    pub span: Span,
    /// Whether the view's last stage is incrementally maintainable over a Z-set (W11).
    pub tail_is_incremental: bool,
}

#[derive(Debug, Clone)]
pub struct FnInfo {
    pub name: String,
    pub declared_effects: Option<EffectRow>,
    pub span: Span,
}

/// The resolved schema, as of an epoch.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub epoch: Epoch,
    pub currencies: HashMap<String, CurrencyInfo>,
    pub relations: HashMap<String, RelationInfo>,
    pub views: HashMap<String, ViewInfo>,
    pub functions: HashMap<String, FnInfo>,
    pub capabilities: HashMap<String, Span>,
}

impl Catalog {
    /// The catalog as of an epoch. The epoch is not decoration: it is what makes a
    /// resolution well defined.
    pub fn at(epoch: Epoch) -> Self {
        Catalog { epoch, ..Default::default() }
    }
    pub fn relation_names(&self) -> Vec<&str> {
        self.relations.keys().map(|s| s.as_str()).collect()
    }
    /// A relation or a view — the two things a pipeline can start from.
    pub fn source_rung(&self, name: &str) -> Option<Rung> {
        if let Some(v) = self.views.get(name) {
            return Some(v.rung);
        }
        // A base is the authoritative history: reading it directly is ledger-consistent by
        // construction, since it is never partial.
        self.relations.get(name).map(|_| Rung::LedgerConsistent)
    }
}

/// Resolve a program into a catalog, reporting every declaration-level error.
///
/// The well-formedness rules discharged here are W1, W2, W3, W5, W9, W10, W11, W12 and
/// W19 of the grammar's section 15. The body-level rules (W4, W6, W7, W8, W13–W18) need
/// expressions and are discharged in [`crate::typecheck`].
pub fn resolve_program(prog: &Program, epoch: Epoch) -> (Catalog, Diagnostics) {
    let mut cat = Catalog::at(epoch);
    let mut d = Diagnostics::new();
    // Two passes: declarations first, so that a view may refer to a relation declared
    // after it. Schemas are not ordered, and requiring them to be would be an artefact of
    // the compiler rather than a property of the language.
    for item in &prog.items {
        collect_item(item, &mut cat, &mut d);
    }
    for item in &prog.items {
        check_item(item, &cat, &mut d);
    }
    (cat, d)
}

fn collect_item(item: &Item, cat: &mut Catalog, d: &mut Diagnostics) {
    match item {
        Item::Schema(s) => {
            for si in &s.items {
                collect_schema_item(si, cat, d);
            }
        }
        Item::Mod { items, .. } => items.iter().for_each(|i| collect_item(i, cat, d)),
        Item::Fn(f) => {
            insert_unique(
                &mut cat.functions,
                f.name.text.clone(),
                FnInfo { name: f.name.text.clone(), declared_effects: f.effects.clone(), span: f.name.span },
                |x| x.span,
                "function",
                f.name.span,
                d,
            );
        }
        Item::View(v) => collect_view(v, cat, d),
        Item::Capability { name, .. } => {
            cat.capabilities.insert(name.text.clone(), name.span);
        }
        _ => {}
    }
}

fn collect_schema_item(si: &SchemaItem, cat: &mut Catalog, d: &mut Diagnostics) {
    match si {
        SchemaItem::Currency(c) => {
            insert_unique(
                &mut cat.currencies,
                c.name.text.clone(),
                CurrencyInfo { name: c.name.text.clone(), scale: c.scale, span: c.name.span },
                |x| x.span,
                "currency",
                c.name.span,
                d,
            );
        }
        SchemaItem::Table(r) | SchemaItem::Base(r) => {
            let info = relation_info(r);
            insert_unique(&mut cat.relations, r.name.text.clone(), info, |x| x.span, "relation", r.name.span, d);
        }
        SchemaItem::View(v) => collect_view(v, cat, d),
        SchemaItem::Index(ix) => {
            if ix.anchor {
                if let Some(rel) = cat.relations.get_mut(&ix.on.text) {
                    rel.anchor_indices.push(ix.cols.iter().map(|c| c.text.clone()).collect());
                }
            }
        }
        SchemaItem::Error(_) => {}
    }
}

fn collect_view(v: &ViewDecl, cat: &mut Catalog, d: &mut Diagnostics) {
    let c = v.contract.as_ref();
    let word = |key: &str, default: &str| -> String {
        c.and_then(|c| c.get(key))
            .map(|x| match x {
                ContractValue::Word(n) => n.text.clone(),
                ContractValue::Call { name, .. } => name.text.clone(),
                other => format!("{other:?}"),
            })
            .unwrap_or_else(|| default.to_string())
    };
    let rung_word = word("consistency", "snapshot");
    let rung = Rung::parse(&rung_word).unwrap_or(Rung::Snapshot);
    if c.is_some() && Rung::parse(&rung_word).is_none() {
        let span = c.and_then(|c| c.get("consistency")).map(|x| x.span()).unwrap_or(v.span);
        let mut diag = Diagnostic::error("NL0200", format!("`{rung_word}` is not a consistency rung"))
            .primary(span, "unknown rung")
            .note("the ladder is: bounded, monotonic, read_your_writes, snapshot, serializable, ledger_consistent");
        if let Some(s) = closest(&rung_word, Rung::all().iter().map(|r| r.as_str())) {
            diag = diag.suggest(span, s, format!("did you mean `{s}`?"), Applicability::MachineApplicable);
        }
        d.push(diag);
    }
    let info = ViewInfo {
        name: v.name.text.clone(),
        rung,
        materialize: word("materialize", "auto"),
        retain: word("retain", "evictable"),
        lineage: word("lineage", "off"),
        contract_span: c.map(|c| c.span).unwrap_or(v.span),
        span: v.span,
        tail_is_incremental: tail_stage(&v.body).map_or(true, |k| k.is_incremental()),
    };
    insert_unique(&mut cat.views, v.name.text.clone(), info, |x| x.span, "view", v.name.span, d);
}

fn tail_stage(e: &Expr) -> Option<StageKind> {
    match e {
        Expr::Stage { kind, .. } => Some(*kind),
        Expr::Sql { inner, .. } => tail_stage(inner),
        _ => None,
    }
}

fn relation_info(r: &RelDecl) -> RelationInfo {
    let mut info = RelationInfo {
        name: r.name.text.clone(),
        kind: r.kind,
        columns: Vec::new(),
        conserve_keys: Vec::new(),
        conserve_span: None,
        retain_forever: false,
        bitemporal: false,
        anchor_indices: Vec::new(),
        span: r.name.span,
    };
    for f in &r.fields {
        let money_currency = match &f.ty {
            Ty::Path { path, args, .. } if path.last().text == "Money" => Some(match args.first() {
                Some(Ty::Path { path, .. }) => Some(path.last().text.clone()),
                _ => None,
            }),
            _ => None,
        };
        let confidential = f
            .attrs
            .iter()
            .find(|a| a.name.text == "confidential")
            .map(|a| match a.args.first() {
                Some(AttrArg::Word(w)) => w.text.clone(),
                _ => "e2ee".to_string(),
            });
        info.columns.push(ColumnInfo {
            name: f.name.text.clone(),
            ty: f.ty.clone(),
            money_currency,
            confidential,
            idem_window: f.default.is_some() && matches!(&f.ty, Ty::Path { path, .. } if path.last().text == "IdemKey"),
            span: f.name.span,
        });
    }
    for rule in &r.rules {
        match rule {
            RelRule::Conserve { keys, span } => {
                info.conserve_keys = keys.iter().map(|k| k.text.clone()).collect();
                info.conserve_span = Some(*span);
            }
            RelRule::Retain { mode, .. } if mode.text == "forever" => info.retain_forever = true,
            RelRule::Bitemporal { .. } => info.bitemporal = true,
            _ => {}
        }
    }
    info
}

fn insert_unique<V>(
    map: &mut HashMap<String, V>,
    key: String,
    value: V,
    span_of: impl Fn(&V) -> Span,
    what: &str,
    span: Span,
    d: &mut Diagnostics,
) {
    if let Some(prev) = map.get(&key) {
        d.push(
            Diagnostic::error("NL0201", format!("{what} `{key}` is declared twice"))
                .primary(span, "redeclared here")
                .secondary(span_of(prev), "first declared here"),
        );
        return;
    }
    map.insert(key, value);
}

// ===================== declaration-level well-formedness =====================

fn check_item(item: &Item, cat: &Catalog, d: &mut Diagnostics) {
    match item {
        Item::Schema(s) => {
            for si in &s.items {
                check_schema_item(si, cat, d);
            }
        }
        Item::Mod { items, .. } => items.iter().for_each(|i| check_item(i, cat, d)),
        Item::View(v) => check_view(v, cat, d),
        _ => {}
    }
}

fn check_schema_item(si: &SchemaItem, cat: &Catalog, d: &mut Diagnostics) {
    match si {
        SchemaItem::Base(r) | SchemaItem::Table(r) => check_relation(r, cat, d),
        SchemaItem::View(v) => check_view(v, cat, d),
        SchemaItem::Index(ix) => {
            if !cat.relations.contains_key(&ix.on.text) {
                unknown_name(d, &ix.on, "relation", cat.relation_names());
            }
        }
        _ => {}
    }
}

fn check_relation(r: &RelDecl, cat: &Catalog, d: &mut Diagnostics) {
    let info = cat.relations.get(&r.name.text);
    let Some(info) = info else { return };

    // W1: a base or ledger must be fully retained. Proposition 3.4 makes full retention
    // *necessary*, not merely sufficient, for reconstructibility — a base you can evict
    // from is a base you cannot reconstruct a view over.
    if info.is_base() && !info.retain_forever {
        d.push(
            Diagnostic::error("NL0210", format!("`{}` is a {} and must declare `retain forever`", r.name.text, kind_word(r.kind)))
                .primary(r.name.span, "no retention declared")
                .note("reconstruction folds the base from a checkpoint forward; a base with evictable history cannot be folded")
                .note("Proposition 3.4: full retention is necessary, not merely sufficient, for reconstructibility")
                .suggest(r.span, "retain forever;", "add the retention clause", Applicability::MachineApplicable),
        );
    }

    // W2: a ledger must declare exactly one conservation rule. Without it, it is a `base`
    // and should say so — the difference is the whole double-entry claim.
    if r.kind == RelKind::Ledger && info.conserve_keys.is_empty() {
        d.push(
            Diagnostic::error("NL0211", format!("ledger `{}` declares no conservation rule", r.name.text))
                .primary(r.name.span, "no `conserve per (..)`")
                .note("a ledger is a base plus a conservation rule; without one, declare it as a `base`")
                .suggest(r.span, "conserve per (txn, cur);", "add the double-entry rule", Applicability::HasPlaceholders),
        );
    }
    if r.kind != RelKind::Ledger && !info.conserve_keys.is_empty() {
        let span = info.conserve_span.unwrap_or(r.name.span);
        d.push(
            Diagnostic::error("NL0212", format!("`{}` is a {} and cannot conserve", r.name.text, kind_word(r.kind)))
                .primary(span, "conservation rule on a non-ledger")
                .note("conservation is checked at the seal of an epoch; a table has no seal"),
        );
    }

    // W3: every column named in the conservation rule must exist.
    for key in &info.conserve_keys {
        if info.column(key).is_none() {
            let span = info.conserve_span.unwrap_or(r.name.span);
            let names: Vec<&str> = info.columns.iter().map(|c| c.name.as_str()).collect();
            let mut diag = Diagnostic::error("NL0213", format!("`{key}` is not a column of `{}`", r.name.text))
                .primary(span, "unknown column in the conservation key");
            if let Some(s) = closest(key, names.into_iter()) {
                diag = diag.suggest(span, s, format!("did you mean `{s}`?"), Applicability::MachineApplicable);
            }
            d.push(diag);
        }
    }

    for c in &info.columns {
        // W5: every `Money<C>` needs its currency declared, because the scale lives in the
        // declaration and a wrong scale is a money bug, not a formatting one.
        if let Some(Some(cur)) = &c.money_currency {
            if !cat.currencies.contains_key(cur) {
                let names: Vec<&str> = cat.currencies.keys().map(|s| s.as_str()).collect();
                let mut diag = Diagnostic::error("NL0214", format!("currency `{cur}` is not declared"))
                    .primary(c.span, format!("`Money<{cur}>` needs `currency {cur} {{ scale: n }}` in scope"))
                    .note("the minor-unit scale is part of the type: JPY is 0, BHD is 3, and defaulting to 2 would be a silent hundred-fold error");
                if let Some(s) = closest(cur, names.into_iter()) {
                    diag = diag.suggest(c.span, s, format!("did you mean `{s}`?"), Applicability::MachineApplicable);
                }
                d.push(diag);
            }
        }
        // W19: an idempotency key without a window is not idempotent, merely unique.
        if matches!(&c.ty, Ty::Path { path, .. } if path.last().text == "IdemKey") && !c.idem_window {
            d.push(
                Diagnostic::error("NL0215", format!("`{}` is an `IdemKey` with no window", c.name))
                    .primary(c.span, "no `window` clause")
                    .note("without a window a key is unique forever, which is a uniqueness constraint, not an idempotency window")
                    .suggest(c.span, "window 24.hours", "declare the window", Applicability::HasPlaceholders),
            );
        }
    }
}

fn kind_word(k: RelKind) -> &'static str {
    match k {
        RelKind::Table => "table",
        RelKind::Base => "base",
        RelKind::Ledger => "ledger",
    }
}

fn check_view(v: &ViewDecl, cat: &Catalog, d: &mut Diagnostics) {
    let Some(info) = cat.views.get(&v.name.text) else { return };
    let cspan = info.contract_span;

    // W9: a strictly-served view may not be backed by state whose read path is an I/O
    // round trip on the critical path.
    if info.rung == Rung::LedgerConsistent && info.materialize == "spilled" {
        d.push(
            Diagnostic::error("NL0220", format!("view `{}` cannot be `ledger_consistent` and `spilled`", v.name.text))
                .primary(cspan, "this combination is not serveable")
                .note("the top rung must answer at the visibility frontier; a spilled read is an I/O round trip on the critical path")
                .suggest(cspan, "materialize: demand", "use demand materialization", Applicability::MachineApplicable),
        );
    }

    // W10: a pinned view is by definition resident.
    if info.retain == "pinned" && info.materialize == "absent" {
        d.push(
            Diagnostic::error("NL0221", format!("view `{}` cannot be `pinned` and `absent`", v.name.text))
                .primary(cspan, "pinned state is resident by definition")
                .note("`retain: pinned` says never evict; `materialize: absent` says never resident"),
        );
    }

    // W11: `order_by`, `limit` and `offset` are not incrementally maintainable over a
    // Z-set without retaining the whole input. Serving such a view on demand at a strict
    // rung is not something the planner can honour, and promoting it silently to `full`
    // would breach the memory budget the contract implies.
    if !info.tail_is_incremental && info.materialize == "demand" && info.rung >= Rung::Serializable {
        d.push(
            Diagnostic::error("NL0222", format!("view `{}` ends in a non-incremental stage and cannot be demand-materialized at `{}`", v.name.text, info.rung))
                .primary(cspan, "this contract cannot be honoured")
                .secondary(v.body.span(), "the final stage retains its whole input")
                .note("`order_by`, `limit` and `offset` are not incrementally maintainable over a Z-set: an upquery would have to reconstruct the entire ordering")
                .suggest(cspan, "materialize: full", "materialize it fully, and accept the memory", Applicability::MaybeIncorrect),
        );
    }

    // W12: a view over a ledger key needs an anchor index on that key, or reconstruction
    // degrades to a scan of history. The measured cost of the difference is two orders of
    // magnitude in constant factor (thesis §9.4.1).
    if let Some((rel, keys, span)) = view_group_key(&v.body, cat) {
        if rel.is_base() && !keys.is_empty() {
            let covered = rel.anchor_indices.iter().any(|ix| keys.iter().all(|k| ix.contains(k)));
            if !covered {
                d.push(
                    Diagnostic::warning("NL0223", format!("no anchor index on `{}` covers ({})", rel.name, keys.join(", ")))
                        .primary(span, "reconstruction for this key will scan history")
                        .secondary(rel.span, "declared here")
                        .note("an anchor index buys a measured 80-112x constant factor on reconstruction; it does not change the asymptote, which is what checkpoints are for")
                        .suggest(rel.span, format!("index ix_{} on {} ({}) anchor;", rel.name, rel.name, keys.join(", ")), "declare an anchor index", Applicability::MachineApplicable),
                );
            }
        }
    }
}

/// The relation and grouping key a view's pipeline reduces to, when it is simple enough to
/// see. Deliberately conservative: a shape this cannot read yields no warning rather than
/// a wrong one.
fn view_group_key<'a>(e: &Expr, cat: &'a Catalog) -> Option<(&'a RelationInfo, Vec<String>, Span)> {
    match e {
        Expr::Stage { recv, kind: StageKind::GroupBy, args, span, .. } => {
            let rel = root_relation(recv, cat)?;
            let keys = args.first().map(|a| key_fields(&a.value)).unwrap_or_default();
            Some((rel, keys, *span))
        }
        Expr::Stage { recv, .. } => view_group_key(recv, cat),
        Expr::Fixpoint { recv, .. } => view_group_key(recv, cat),
        _ => None,
    }
}

fn root_relation<'a>(e: &Expr, cat: &'a Catalog) -> Option<&'a RelationInfo> {
    match e {
        Expr::Path(p) => cat.relations.get(&p.last().text),
        Expr::Stage { recv, .. } | Expr::Fixpoint { recv, .. } => root_relation(recv, cat),
        _ => None,
    }
}

/// The field names a `|p| (p.a, p.b)` closure projects.
fn key_fields(e: &Expr) -> Vec<String> {
    fn walk(e: &Expr, out: &mut Vec<String>) {
        match e {
            Expr::Field { name, .. } => out.push(name.text.clone()),
            Expr::Tuple { elems, .. } => elems.iter().for_each(|x| walk(x, out)),
            Expr::Closure { body, .. } => walk(body, out),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(e, &mut out);
    out
}

fn unknown_name(d: &mut Diagnostics, n: &Name, what: &str, candidates: Vec<&str>) {
    let mut diag = Diagnostic::error("NL0202", format!("unknown {what} `{}`", n.text))
        .primary(n.span, format!("no {what} with this name is declared"));
    if let Some(s) = closest(&n.text, candidates.into_iter()) {
        diag = diag.suggest(n.span, s, format!("did you mean `{s}`?"), Applicability::MachineApplicable);
    }
    d.push(diag);
}
