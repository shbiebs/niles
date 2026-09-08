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
use crate::lexer::{Span, TimeUnit};
use std::collections::{BTreeSet, HashMap};

/// The epoch a catalog was read at. A resolution is a statement about the schema *as of*
/// this epoch and no other.
pub type Epoch = u64;

#[derive(Debug, Clone)]
pub struct CurrencyInfo {
    pub name: String,
    /// The minor-unit exponent. Carried, never assumed: JPY is 0, most are 2, BHD is 3.
    pub scale: u32,
    pub span: Span,
    /// **The currency's wire and IR code: its position among the schema's `currency`
    /// declarations, counting from zero.**
    ///
    /// Assigned once, here, from declaration order. It was derived twice and differently:
    /// the wire sorted the catalog's currencies by span (`session::declared_currencies`),
    /// and the compiler took `cat.currencies.keys().position(..)` over a `HashMap` with the
    /// standard hasher (`lower::scalar`), so a money literal in a compiled circuit named a
    /// currency by an order that changed between processes while the wire named it by
    /// declaration index. A currency code is written into the ledger, where it is permanent
    /// (A9-F05). With one declared currency the two agree at zero, which is why nothing had
    /// seen it.
    pub code: u32,
}

#[derive(Debug, Clone)]
pub struct RelationInfo {
    pub name: String,
    pub kind: RelKind,
    pub columns: Vec<ColumnInfo>,
    /// The conservation grouping, if any. Present exactly on ledgers (W2).
    pub conserve_keys: Vec<String>,
    pub conserve_span: Option<Span>,
    /// The spans of every `conserve` clause after the first, which W2b refuses.
    pub extra_conserve_spans: Vec<Span>,
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
    /// The column naming whose value this is, from `@confidential(<level>, subject = <col>)`.
    ///
    /// `None` on a column that is not confidential, and a refusal (NL0261) on one that is:
    /// a sealed value whose owner the schema does not name is a value no erasure can find.
    pub confidential_subject: Option<String>,
    /// The idempotency window this column declares, if it declares one.
    ///
    /// Carried rather than reduced to a flag: it used to be `bool`, so the window was
    /// checked for *existence* and then dropped, and nothing below the compiler knew how
    /// long an identity was supposed to be remembered. Both idempotency indexes therefore
    /// kept every identity ever committed — 68.8 B and 99.6 B each, forever (E18) — which is
    /// the one structure in the write path whose size was a function of history.
    pub idem_window: Option<IdemWindow>,
    pub span: Span,
}

/// A declared idempotency window.
///
/// **Epochs are the only unit the engine can honour.** An epoch carries no wall clock: the
/// durable record is `parent ‖ hash ‖ canon(rows)`, and a timestamp under that hash would
/// make every committed chain hash time-dependent. So a wall-clock window is refused
/// (NL0217) rather than converted with an assumed epoch quantum — the conversion would be a
/// number the compiler invented, and the window would be wrong by whatever the write rate
/// turned out to be. Decided by the author as LC-28, cycle 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdemWindow {
    /// `window 1_000_000.epochs` — the form the ledger and the sealer both enforce.
    Epochs(u64),
    /// `window 30.days` — declared in wall-clock time, which this system has no coordinate
    /// for. Kept in the catalogue so the diagnostic can say what was written.
    WallClock { millis: i128 },
}

/// The two numbers a `bounded(..)` rung is parameterised by.
///
/// Carried from the source into the IR. They used to be discarded twice over — `resolve`
/// kept only the *word* `bounded` from a `ContractValue::Call`, and `lower` then wrote
/// `{ epochs: 4, millis: 1000 }` into every bounded contract in the language. So
/// `bounded(epochs: 8)` and `bounded(1.epochs, 1.s)` produced byte-identical circuits, and
/// the staleness a view promised its readers had no relation to the staleness it was
/// written with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Staleness {
    pub epochs: u64,
    pub millis: u64,
}

impl Staleness {
    /// What a bare `bounded` means: the rung with no parameters given.
    ///
    /// Stated once, here, rather than typed into the lowering — and *named*, so a reader
    /// can tell a default from a measurement. Any view that says `bounded(..)` overrides
    /// both numbers.
    pub const UNPARAMETERISED: Staleness = Staleness {
        epochs: 4,
        millis: 1_000,
    };
}

#[derive(Debug, Clone)]
pub struct ViewInfo {
    pub name: String,
    pub rung: Rung,
    /// Present exactly when `rung == Rung::Bounded`. `None` on every other rung, because
    /// the other rungs are not parameterised and carrying a bound on them would invite a
    /// reader to believe one was honoured.
    pub staleness: Option<Staleness>,
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
        Catalog {
            epoch,
            ..Default::default()
        }
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
                FnInfo {
                    name: f.name.text.clone(),
                    declared_effects: f.effects.clone(),
                    span: f.name.span,
                },
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
            // **The code is assigned here and nowhere else.** Declaration order, counting
            // from zero, taken from how many currencies the catalog already holds — which is
            // the order they were collected in, which is the order they were written in.
            // Every consumer reads this field instead of re-deriving it (A9-F05).
            let code = cat.currencies.len() as u32;
            insert_unique(
                &mut cat.currencies,
                c.name.text.clone(),
                CurrencyInfo {
                    name: c.name.text.clone(),
                    scale: c.scale,
                    span: c.name.span,
                    code,
                },
                |x| x.span,
                "currency",
                c.name.span,
                d,
            );
        }
        SchemaItem::Table(r) | SchemaItem::Base(r) => {
            let info = relation_info(r);
            insert_unique(
                &mut cat.relations,
                r.name.text.clone(),
                info,
                |x| x.span,
                "relation",
                r.name.span,
                d,
            );
        }
        SchemaItem::View(v) => collect_view(v, cat, d),
        SchemaItem::Index(ix) => {
            if ix.anchor {
                if let Some(rel) = cat.relations.get_mut(&ix.on.text) {
                    rel.anchor_indices
                        .push(ix.cols.iter().map(|c| c.text.clone()).collect());
                }
            }
        }
        SchemaItem::Error(_) => {}
    }
}

/// Read `bounded(epochs: 4, millis: 200)` — or `bounded(10.epochs, 30.s)`, or
/// `bounded(epochs: 8)` — into the two numbers the engine needs.
///
/// Both spellings are in the repository's own examples, so both are read here rather than
/// one being quietly preferred. An argument that is neither is refused: the previous
/// behaviour discarded the whole argument list, so every one of these forms produced the
/// same contract and nothing said so.
fn staleness_of(
    args: &[(Option<Name>, ContractValue)],
    span: Span,
    view: &str,
    d: &mut Diagnostics,
) -> Staleness {
    let mut out = Staleness::UNPARAMETERISED;
    let (mut saw_epochs, mut saw_millis) = (false, false);
    for (key, value) in args {
        // Which of the two numbers this is: from the label if there is one, and otherwise
        // from the unit, because `10.epochs` and `30.s` name themselves.
        let (is_epochs, n, at) = match (key.as_ref().map(|k| k.text.as_str()), value) {
            (Some("epochs"), ContractValue::Int(n, s)) => (true, *n, *s),
            (Some("millis"), ContractValue::Int(n, s)) => (false, *n, *s),
            (
                _,
                ContractValue::Duration {
                    value: n,
                    unit: TimeUnit::Epochs,
                    span: s,
                },
            ) => (true, *n, *s),
            (
                _,
                ContractValue::Duration {
                    value: n,
                    unit,
                    span: s,
                },
            ) => match unit.millis() {
                Some(m) => (false, n * m, *s),
                None => (true, *n, *s),
            },
            (Some(other), v) => {
                d.push(
                    Diagnostic::error("NL0202", format!("`bounded` has no parameter `{other}`"))
                        .primary(v.span(), "unknown parameter")
                        .note("`bounded` takes `epochs` and `millis`, in either order, by name or by unit"),
                );
                continue;
            }
            (None, v) => {
                d.push(
                    Diagnostic::error("NL0202", "this bound has no unit")
                        .primary(v.span(), "cannot tell epochs from milliseconds")
                        .note("write `epochs: 8` or `8.epochs`, `millis: 200` or `200.ms`")
                        .note("a bare number here used to be discarded silently, along with the rest of the argument list"),
                );
                continue;
            }
        };
        if n < 0 {
            d.push(
                Diagnostic::error("NL0202", "a staleness bound cannot be negative")
                    .primary(at, format!("`{n}`"))
                    .note("the bound is how far *behind* the frontier a read may be; a negative one would be a promise to read the future"),
            );
            continue;
        }
        let n = u64::try_from(n).unwrap_or(u64::MAX);
        if is_epochs {
            out.epochs = n;
            saw_epochs = true;
        } else {
            out.millis = n;
            saw_millis = true;
        }
    }
    // A bound given on one axis only leaves the other at its unparameterised value, which
    // is a real decision and not obviously the one intended — so it is said out loud.
    if (saw_epochs || saw_millis) && !(saw_epochs && saw_millis) {
        let (given, missing, keep) = if saw_epochs {
            ("epochs", "millis", out.millis)
        } else {
            ("millis", "epochs", out.epochs)
        };
        d.push(
            Diagnostic::warning(
                "NL0203",
                format!("`{view}` bounds `{given}` but not `{missing}`"),
            )
            .primary(span, format!("`{missing}` stays at {keep}"))
            .note("a bounded rung is bounded on both axes: whichever is reached first is the one that binds")
            .suggest(
                span,
                format!("bounded({given}: .., {missing}: {keep})"),
                "state both, so the one that binds is visible",
                Applicability::MaybeIncorrect,
            ),
        );
    }
    out
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
        let span = c
            .and_then(|c| c.get("consistency"))
            .map(|x| x.span())
            .unwrap_or(v.span);
        let mut diag = Diagnostic::error("NL0200", format!("`{rung_word}` is not a consistency rung"))
            .primary(span, "unknown rung")
            .note("the ladder is: bounded, monotonic, read_your_writes, snapshot, serializable, ledger_consistent");
        if let Some(s) = closest(&rung_word, Rung::all().iter().map(|r| r.as_str())) {
            diag = diag.suggest(
                span,
                s,
                format!("did you mean `{s}`?"),
                Applicability::MachineApplicable,
            );
        }
        d.push(diag);
    }
    // The `bounded(..)` parameters, read here rather than discarded here.
    let staleness = if rung == Rung::Bounded {
        Some(match c.and_then(|c| c.get("consistency")) {
            Some(ContractValue::Call { args, span, .. }) => {
                staleness_of(args, *span, &v.name.text, d)
            }
            _ => Staleness::UNPARAMETERISED,
        })
    } else {
        // A parameterised rung other than `bounded` is a contract that reads as if it
        // constrains something it does not. Refused rather than ignored.
        if let Some(ContractValue::Call { name, span, .. }) = c.and_then(|c| c.get("consistency")) {
            if Rung::parse(&name.text).is_some() {
                d.push(
                    Diagnostic::error(
                        "NL0201",
                        format!("`{}` takes no parameters", name.text),
                    )
                    .primary(*span, "parameters given here")
                    .note("only `bounded` is parameterised: it is the one rung defined by how far behind the frontier a read may be")
                    .note("the others are defined by an ordering property, which has no knob"),
                );
            }
        }
        None
    };
    let info = ViewInfo {
        name: v.name.text.clone(),
        rung,
        staleness,
        materialize: word("materialize", "auto"),
        retain: word("retain", "evictable"),
        lineage: word("lineage", "off"),
        contract_span: c.map(|c| c.span).unwrap_or(v.span),
        span: v.span,
        tail_is_incremental: tail_stage(&v.body).is_none_or(|k| k.is_incremental()),
    };
    insert_unique(
        &mut cat.views,
        v.name.text.clone(),
        info,
        |x| x.span,
        "view",
        v.name.span,
        d,
    );
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
        extra_conserve_spans: Vec::new(),
        retain_forever: false,
        bitemporal: false,
        anchor_indices: Vec::new(),
        span: r.name.span,
    };
    for f in &r.fields {
        let money_currency = match &f.ty {
            Ty::Path { path, args, .. } if path.last().text == "Money" => {
                Some(match args.first() {
                    Some(Ty::Path { path, .. }) => Some(path.last().text.clone()),
                    _ => None,
                })
            }
            _ => None,
        };
        let conf_attr = f.attrs.iter().find(|a| a.name.text == "confidential");
        let confidential = conf_attr.map(|a| match a.args.first() {
            Some(AttrArg::Word(w)) => w.text.clone(),
            _ => "e2ee".to_string(),
        });
        // **The subject: whose value this is.**
        //
        // A confidential value is encrypted under a key, and a key belongs to somebody. Until
        // the schema says who, an erasure has no way to name the values it must destroy: it
        // would have to scan every column of every row and guess. `subject = <column>` states
        // it once, where the column is declared, and the erasure path reads it.
        let confidential_subject = conf_attr.and_then(|a| {
            a.args.iter().find_map(|arg| match arg {
                AttrArg::KeyValue(k, v) if k.text == "subject" => match v {
                    Expr::Path(p) => Some(p.last().text.clone()),
                    Expr::Field { name, .. } => Some(name.text.clone()),
                    _ => None,
                },
                _ => None,
            })
        });
        info.columns.push(ColumnInfo {
            name: f.name.text.clone(),
            ty: f.ty.clone(),
            money_currency,
            confidential,
            confidential_subject,
            idem_window: f.window.as_ref().and_then(|w| match w {
                Expr::Duration {
                    value,
                    unit: crate::lexer::TimeUnit::Epochs,
                    ..
                } => Some(IdemWindow::Epochs((*value).max(0) as u64)),
                Expr::Duration { value, unit, .. } => Some(IdemWindow::WallClock {
                    millis: unit.millis().unwrap_or(0) * *value,
                }),
                _ => None,
            }),
            span: f.name.span,
        });
    }
    for rule in &r.rules {
        match rule {
            RelRule::Conserve { keys, span } => {
                // **First wins, and a second is a refusal rather than a replacement.** This
                // used to assign unconditionally, so `conserve per (txn, cur); conserve per
                // (desk);` compiled clean and the *first* rule — the one a reader would say
                // the ledger declares — was the one thrown away. The refusal is raised in
                // `check_relation` where the diagnostics live; recording the first span is
                // what lets it point at both.
                if info.conserve_span.is_none() {
                    info.conserve_keys = keys.iter().map(|k| k.text.clone()).collect();
                    info.conserve_span = Some(*span);
                } else {
                    info.extra_conserve_spans.push(*span);
                }
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
        SchemaItem::Index(ix) if !cat.relations.contains_key(&ix.on.text) => {
            unknown_name(d, &ix.on, "relation", cat.relation_names());
        }
        _ => {}
    }
}

/// The single column of `r` whose declared type names `ty`, if there is exactly one.
///
/// `Err(n)` when there are `n != 1`: zero means the ledger cannot be conserved the way
/// admission conserves, and more than one means "the transaction column" does not name
/// anything in particular, so neither can be silently chosen.
fn sole_column_of_type<'a>(info: &'a RelationInfo, ty: &str) -> Result<&'a ColumnInfo, usize> {
    let hits: Vec<&ColumnInfo> = info
        .columns
        .iter()
        .filter(|c| matches!(&c.ty, Ty::Path { path, .. } if path.last().text == ty))
        .collect();
    match hits.len() {
        1 => Ok(hits[0]),
        n => Err(n),
    }
}

fn check_relation(r: &RelDecl, cat: &Catalog, d: &mut Diagnostics) {
    let info = cat.relations.get(&r.name.text);
    let Some(info) = info else { return };

    // **A sealed value must name whose it is.**
    //
    // `@confidential(<level>)` says the engine may not read a column. It does not say who the
    // value belongs to, and without that an erasure has nothing to aim at: destroying "this
    // person's data" means destroying the values encrypted under this person's key, and the
    // key is chosen by the subject. A column declared confidential and unattributed is a value
    // that can be sealed and never lawfully destroyed, which is the worst of both properties.
    for c in &info.columns {
        let Some(level) = &c.confidential else {
            continue;
        };
        match &c.confidential_subject {
            None => d.push(
                Diagnostic::error(
                    "NL0261",
                    format!("`{}` is `@confidential({level})` and names no subject", c.name),
                )
                .primary(c.span, "no `subject = <column>`")
                .note("a confidential value is encrypted under a key, and a key belongs to somebody; the subject is the column that says who")
                .suggest(
                    c.span,
                    format!("@confidential({level}, subject = <column>)"),
                    "name the column this value belongs to",
                    Applicability::HasPlaceholders,
                ),
            ),
            Some(subject) => {
                if !info.columns.iter().any(|x| &x.name == subject) {
                    d.push(
                        Diagnostic::error(
                            "NL0261",
                            format!("`{}`'s subject `{subject}` is not a column of `{}`", c.name, r.name.text),
                        )
                        .primary(c.span, "no such column")
                        .note(format!(
                            "the columns of `{}` are: {}",
                            r.name.text,
                            info.columns.iter().map(|x| x.name.as_str()).collect::<Vec<_>>().join(", ")
                        )),
                    );
                } else if info
                    .columns
                    .iter()
                    .any(|x| &x.name == subject && x.confidential.is_some())
                {
                    d.push(
                        Diagnostic::error(
                            "NL0261",
                            format!("`{}`'s subject `{subject}` is itself confidential", c.name),
                        )
                        .primary(c.span, "the subject must be readable")
                        .note("an erasure has to look the subject up to find the keys it must destroy; a subject the engine cannot read is a subject it cannot find"),
                    );
                }
            }
        }
    }

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
            Diagnostic::error(
                "NL0211",
                format!("ledger `{}` declares no conservation rule", r.name.text),
            )
            .primary(r.name.span, "no `conserve per (..)`")
            .note(
                "a ledger is a base plus a conservation rule; without one, declare it as a `base`",
            )
            .suggest(
                r.span,
                "conserve per (txn, cur);",
                "add the double-entry rule",
                Applicability::HasPlaceholders,
            ),
        );
    }
    if r.kind != RelKind::Ledger && !info.conserve_keys.is_empty() {
        let span = info.conserve_span.unwrap_or(r.name.span);
        d.push(
            Diagnostic::error(
                "NL0212",
                format!(
                    "`{}` is a {} and cannot conserve",
                    r.name.text,
                    kind_word(r.kind)
                ),
            )
            .primary(span, "conservation rule on a non-ledger")
            .note("conservation is checked at the seal of an epoch; a table has no seal"),
        );
    }

    // W3: every column named in the conservation rule must exist.
    for key in &info.conserve_keys {
        if info.column(key).is_none() {
            let span = info.conserve_span.unwrap_or(r.name.span);
            let names: Vec<&str> = info.columns.iter().map(|c| c.name.as_str()).collect();
            let mut diag = Diagnostic::error(
                "NL0213",
                format!("`{key}` is not a column of `{}`", r.name.text),
            )
            .primary(span, "unknown column in the conservation key");
            if let Some(s) = closest(key, names.into_iter()) {
                diag = diag.suggest(
                    span,
                    s,
                    format!("did you mean `{s}`?"),
                    Applicability::MachineApplicable,
                );
            }
            d.push(diag);
        }
    }

    // W2b: a ledger declares its conservation rule once.
    //
    // The parser accepts any number of `conserve` clauses and `relation_info` used to assign
    // the last one over the first, silently. So a ledger could carry two contradictory rules,
    // compile clean, and be conserved by neither the rule a reader would name nor a rule
    // anyone wrote down.
    for extra in &info.extra_conserve_spans {
        let mut diag = Diagnostic::error(
            "NL0224",
            format!("`{}` declares more than one conservation rule", r.name.text),
        )
        .primary(*extra, "a second `conserve per (..)` on the same ledger")
        .note(
            "a ledger has one conservation rule; two rules are two different claims about \
             what a sealed epoch must satisfy, and nothing decides between them",
        );
        if let Some(first) = info.conserve_span {
            diag = diag.secondary(first, "the first rule is declared here");
        }
        d.push(diag);
    }

    // W3b: the declared grouping must be the grouping admission enforces.
    //
    // **A10-07.** `conserve per (..)` parsed, name-checked its columns, and then reached
    // nothing: `conserve_keys` had no consumer anywhere below the compiler, while admission
    // conserves per (transaction, currency) unconditionally — `crates/niles-interp/src/
    // ledger.rs` sums each open transaction's legs by currency and refuses a non-zero
    // residual. So `conserve per (txn, desk)` compiled, read as a promise of per-desk
    // segregation, and bought nothing at all.
    //
    // The refusal is deliberately narrow, and the narrowness is the point. Per-group zero
    // sums are *weaker* than forbidding cross-group flow: two opposite cross-desk transfers
    // in one transaction cancel within each desk and the partition is still violated. A real
    // segregation rule needs an admissible-edge policy with a stated allowance for FX and
    // linked legs, and that policy is the author's to choose (LC-41). Until it exists, the
    // honest thing is to accept exactly what is enforced and refuse the rest by name —
    // not to invent a policy here, and not to keep accepting a declaration that means
    // nothing.
    //
    // The keys are compared as a *set*: `(cur, txn)` and `(txn, cur)` describe the same
    // partition, and refusing one of them would be a statement about writing order rather
    // than about conservation.
    if r.kind == RelKind::Ledger && !info.conserve_keys.is_empty() {
        let span = info.conserve_span.unwrap_or(r.name.span);
        let every_key_exists = info.conserve_keys.iter().all(|k| info.column(k).is_some());
        // W3 has already reported an unknown column; a second diagnostic about the same
        // typo would be noise.
        if every_key_exists {
            let txn = sole_column_of_type(info, "TxnId");
            let cur = sole_column_of_type(info, "Currency");
            let enforced: Option<Vec<&str>> = match (&txn, &cur) {
                (Ok(t), Ok(c)) => Some(vec![t.name.as_str(), c.name.as_str()]),
                _ => None,
            };
            let declared: BTreeSet<&str> = info.conserve_keys.iter().map(|k| k.as_str()).collect();

            match enforced {
                Some(e)
                    if declared == e.iter().copied().collect::<BTreeSet<&str>>()
                        && declared.len() == info.conserve_keys.len() =>
                {
                    // The declared rule is the enforced rule. Nothing to say.
                }
                Some(e) => {
                    let repeated = declared.len() != info.conserve_keys.len();
                    let mut diag = Diagnostic::error(
                        "NL0225",
                        format!(
                            "`{}` declares a conservation grouping that is not enforced",
                            r.name.text
                        ),
                    )
                    .primary(
                        span,
                        if repeated {
                            "a column is named twice in the conservation key"
                        } else {
                            "this grouping has no admission rule behind it"
                        },
                    )
                    .note(format!(
                        "the seal enforces exactly one grouping: per transaction, per \
                         currency — here `conserve per ({}, {})`",
                        e[0], e[1]
                    ))
                    .note(
                        "a grouping the seal does not check is a promise the ledger does not \
                         keep. Per-group zero sums would also be weaker than they look: two \
                         opposite cross-group legs in one transaction cancel inside each \
                         group, so segregation needs an admissible-edge rule that does not \
                         exist yet",
                    );
                    if !repeated {
                        diag = diag.suggest(
                            span,
                            format!("conserve per ({}, {});", e[0], e[1]),
                            "declare the rule the seal enforces",
                            Applicability::MachineApplicable,
                        );
                    }
                    d.push(diag);
                }
                None => {
                    let why = match (txn, cur) {
                        (Err(n), _) if n != 1 => format!(
                            "`{}` has {} columns of type `TxnId`",
                            r.name.text,
                            if n == 0 {
                                "no".to_string()
                            } else {
                                n.to_string()
                            }
                        ),
                        (_, Err(n)) => format!(
                            "`{}` has {} columns of type `Currency`",
                            r.name.text,
                            if n == 0 {
                                "no".to_string()
                            } else {
                                n.to_string()
                            }
                        ),
                        _ => unreachable!("enforced is None only when a lookup failed"),
                    };
                    d.push(
                        Diagnostic::error(
                            "NL0225",
                            format!(
                                "`{}` cannot be conserved the way the seal conserves",
                                r.name.text
                            ),
                        )
                        .primary(span, "no enforceable grouping for this ledger")
                        .note(format!("{why}, so \"per transaction, per currency\" does not name a grouping of it"))
                        .note(
                            "the seal sums each open transaction's legs by currency; a ledger \
                             it can check declares exactly one `TxnId` column and exactly one \
                             `Currency` column",
                        ),
                    );
                }
            }
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
                    diag = diag.suggest(
                        c.span,
                        s,
                        format!("did you mean `{s}`?"),
                        Applicability::MachineApplicable,
                    );
                }
                d.push(diag);
            }
        }
        // W19: an idempotency key without a window is not idempotent, merely unique.
        if matches!(&c.ty, Ty::Path { path, .. } if path.last().text == "IdemKey") {
            match c.idem_window {
                None => d.push(
                    Diagnostic::error("NL0215", format!("`{}` is an `IdemKey` with no window", c.name))
                        .primary(c.span, "no `window` clause")
                        .note("without a window a key is unique forever, which is a uniqueness constraint, not an idempotency window")
                        .suggest(c.span, "window 1_000_000.epochs", "declare the window", Applicability::HasPlaceholders),
                ),
                // NL0217: a window this system has no clock to measure.
                Some(IdemWindow::WallClock { .. }) => d.push(
                    Diagnostic::error(
                        "NL0217",
                        format!("`{}` declares its idempotency window in wall-clock time", c.name),
                    )
                    .primary(c.span, "an epoch carries no clock")
                    .note("the ledger's only time coordinate is the epoch. The durable record is `parent | hash | canon(rows)`, and a timestamp under that hash would make every committed chain hash depend on when it was written")
                    .note("converting days to epochs here would mean inventing a write rate, and the window would then be wrong by however much the real rate differed")
                    .suggest(c.span, "window 1_000_000.epochs", "count the window in epochs", Applicability::HasPlaceholders),
                ),
                Some(IdemWindow::Epochs(_)) => {}
            }
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
    let Some(info) = cat.views.get(&v.name.text) else {
        return;
    };
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
            Diagnostic::error(
                "NL0221",
                format!("view `{}` cannot be `pinned` and `absent`", v.name.text),
            )
            .primary(cspan, "pinned state is resident by definition")
            .note("`retain: pinned` says never evict; `materialize: absent` says never resident"),
        );
    }

    // W11: `order_by`, `limit` and `offset` are not incrementally maintainable over a
    // Z-set without retaining the whole input. Serving such a view on demand at a strict
    // rung is not something the planner can honour, and promoting it silently to `full`
    // would breach the memory budget the contract implies.
    if !info.tail_is_incremental && info.materialize == "demand" && info.rung >= Rung::Serializable
    {
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
    //
    // A view that is *never* evicted has no reconstruction to be slow, so the warning does
    // not apply to it. Emitting it anyway put a permanent, unfixable line in the gate's
    // output — the suggested remedy is an anchor index on a column the ledger does not
    // have — and a warning nobody can act on is one everybody learns to scroll past.
    let evictable = info.retain != "pinned" && info.materialize != "full";
    if let Some((rel, keys, span)) = view_group_key(&v.body, cat) {
        if evictable && rel.is_base() && !keys.is_empty() {
            let covered = rel
                .anchor_indices
                .iter()
                .any(|ix| keys.iter().all(|k| ix.contains(k)));
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
        Expr::Stage {
            recv,
            kind: StageKind::GroupBy,
            args,
            span,
            ..
        } => {
            let rel = root_relation(recv, cat)?;
            let keys = args
                .first()
                .map(|a| key_fields(&a.value))
                .unwrap_or_default();
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
        diag = diag.suggest(
            n.span,
            s,
            format!("did you mean `{s}`?"),
            Applicability::MachineApplicable,
        );
    }
    d.push(diag);
}

#[cfg(test)]
mod confidentiality_tests {
    use crate::{parser, resolve};

    fn codes(src: &str) -> Vec<String> {
        let (prog, mut d) = parser::parse_program(src);
        let (_cat, rd) = resolve::resolve_program(&prog, 0);
        d.extend(rd);
        d.items
            .iter()
            .filter(|x| x.severity == crate::diagnostics::Severity::Error)
            .map(|x| x.code.to_string())
            .collect()
    }

    const HEAD: &str = "schema s { currency usd { scale: 2 }\n";

    /// **A sealed value must say whose it is.**
    ///
    /// Not a style rule: `erase(subject)` has to enumerate the values encrypted under that
    /// subject's key, and the only thing that can tell it which rows those are is a column
    /// the schema names. A confidential column with no subject is a value that can be sealed
    /// and never lawfully destroyed.
    #[test]
    fn a_confidential_column_must_name_a_subject() {
        assert!(codes(&format!(
            "{HEAD} table p {{ id: Id<A> primary key, owner: Text @confidential(e2ee) }} }}"
        ))
        .contains(&"NL0261".to_string()));

        assert!(
            codes(&format!(
                "{HEAD} table p {{ id: Id<A> primary key, owner: Text @confidential(e2ee, subject = id) }} }}"
            ))
            .is_empty(),
            "a subject that resolves is accepted"
        );
    }

    #[test]
    fn a_subject_must_be_a_column_of_the_same_relation() {
        assert!(codes(&format!(
            "{HEAD} table p {{ id: Id<A> primary key, owner: Text @confidential(e2ee, subject = nope) }} }}"
        ))
        .contains(&"NL0261".to_string()));
    }

    /// The subject has to be readable, because the erasure path looks it up.
    #[test]
    fn a_subject_may_not_itself_be_confidential() {
        assert!(codes(&format!(
            "{HEAD} table p {{ id: Id<A> primary key, \
             who: Text @confidential(e2ee, subject = id), \
             owner: Text @confidential(e2ee, subject = who) }} }}"
        ))
        .contains(&"NL0261".to_string()));
    }
}

/// **The idempotency window: that it is declared, and in a unit the engine can honour.**
#[cfg(test)]
mod idem_window_tests {
    use crate::{parser, resolve};

    fn codes(src: &str) -> Vec<String> {
        let (prog, mut d) = parser::parse_program(src);
        let (_cat, rd) = resolve::resolve_program(&prog, 0);
        d.extend(rd);
        d.items
            .iter()
            .filter(|x| x.severity == crate::diagnostics::Severity::Error)
            .map(|x| x.code.to_string())
            .collect()
    }

    fn ledger_with(col: &str) -> String {
        format!(
            "schema s {{ currency usd {{ scale: 2 }}\n\
             ledger postings {{ txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money, \
             {col}, conserve per (txn, cur); retain forever; }}\n\
             index ix on postings (acct) anchor; }}"
        )
    }

    #[test]
    fn a_window_in_epochs_is_accepted() {
        assert!(codes(&ledger_with("idem: IdemKey window 1_000_000.epochs")).is_empty());
    }

    #[test]
    fn a_window_in_wall_clock_is_refused() {
        assert!(codes(&ledger_with("idem: IdemKey window 30.days")).contains(&"NL0217".to_string()));
    }

    #[test]
    fn no_window_at_all_is_still_refused() {
        assert!(codes(&ledger_with("idem: IdemKey")).contains(&"NL0215".to_string()));
    }

    /// **The check on the window was vacuous in the presence of an unrelated clause.**
    ///
    /// `window <expr>` and `default <expr>` were parsed into the same `FieldDecl` slot, and
    /// the only thing NL0215 asked was whether that slot was occupied. So an `IdemKey` with
    /// a `default` and no window passed the one check whose whole purpose is to say that an
    /// idempotency key without a window is a uniqueness constraint on all of history — and a
    /// column declaring both kept whichever came last, silently. The window now has its own
    /// field, and this is the case that was accepted before it did.
    #[test]
    fn a_default_is_not_a_window() {
        assert!(
            codes(&ledger_with("idem: IdemKey default \"x\"")).contains(&"NL0215".to_string()),
            "an `IdemKey` with a `default` and no window must still be refused: a default is \
             not a window, and reading one as the other made the check vacuous"
        );
    }
}

/// **The conservation grouping: that it is declared once, and that it is the one enforced.**
///
/// A10-07. Before cycle 10 `conserve per (..)` was parsed, its columns were name-checked, and
/// then `conserve_keys` reached no consumer anywhere below the compiler. Admission conserves
/// per (transaction, currency) unconditionally, so `conserve per (txn, desk)` compiled clean,
/// read as a promise of per-desk segregation, and bought nothing. A repeated clause was worse
/// still: the second overwrote the first in silence, and neither rule was the one enforced.
///
/// The refusals here are narrow on purpose. Refusing an unenforced grouping is not the same
/// as implementing segregation, and this cycle does not implement it: per-group zero sums are
/// weaker than forbidding cross-group flow (two opposite cross-group legs in one transaction
/// cancel inside each group), so a real rule needs an admissible-edge policy with a stated
/// allowance for FX and linked legs. That policy is the author's to choose — LC-41 — and the
/// honest position until it exists is to accept exactly what the seal checks.
#[cfg(test)]
mod conserve_grouping_tests {
    use crate::{parser, resolve};

    fn codes(src: &str) -> Vec<String> {
        let (prog, mut d) = parser::parse_program(src);
        let (_cat, rd) = resolve::resolve_program(&prog, 0);
        d.extend(rd);
        d.items
            .iter()
            .filter(|x| x.severity == crate::diagnostics::Severity::Error)
            .map(|x| x.code.to_string())
            .collect()
    }

    /// A ledger whose columns are the usual four, with `rule` as its conservation clause(s).
    fn with_rule(rule: &str) -> String {
        format!(
            "schema s {{ currency usd {{ scale: 2 }}\n\
             ledger postings {{ txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money, \
             idem: IdemKey window 1_000_000.epochs, {rule} retain forever; }}\n\
             index ix on postings (acct) anchor; }}"
        )
    }

    #[test]
    fn the_enforced_grouping_is_accepted() {
        assert!(
            codes(&with_rule("conserve per (txn, cur);")).is_empty(),
            "the rule the seal enforces must compile"
        );
    }

    /// The grouping is a set. `(cur, txn)` and `(txn, cur)` are the same partition, and
    /// refusing one would be a statement about writing order rather than about conservation.
    #[test]
    fn the_enforced_grouping_is_accepted_in_either_order() {
        assert!(codes(&with_rule("conserve per (cur, txn);")).is_empty());
    }

    /// **The column names are not what is checked — the *types* are.**
    ///
    /// The conserved quantity is a graded abelian group, and a grade is declared with
    /// `currency` because that is the domain the notation was designed in. Nothing requires
    /// the column holding it to be spelled `cur`, and a rule keyed on that spelling would
    /// refuse the one file in the corpus that tests whether this machinery is general at all
    /// — the H-S8 falsifier of §9.11.1, which conserves a non-monetary quantity and whose
    /// standing claim is that no source file under `crates/niles-lang`, `crates/niles-ir` or
    /// `crates/nilestream-core` contains a word from its domain. Hence a neutral fixture
    /// here — the test in `crates/conservation-suite` that enforces that claim reads this
    /// file, and naming the domain in this comment would break it, which is the claim
    /// working.
    #[test]
    fn the_currency_column_need_not_be_called_cur() {
        assert!(
            codes(
                "schema s { currency aaa { scale: 0 }\n\
                 ledger movements { txn: TxnId, place: Id<Account>, grade: Currency, qty: Money, \
                 conserve per (txn, grade); retain forever; }\n\
                 index ix on movements (place) anchor; }"
            )
            .is_empty(),
            "the enforced grouping is (the TxnId column, the Currency column), whatever they \
             are named"
        );
    }

    #[test]
    fn a_grouping_the_seal_does_not_enforce_is_refused() {
        let c = codes(&with_rule("conserve per (txn, acct);"));
        assert!(
            c.contains(&"NL0225".to_string()),
            "`conserve per (txn, acct)` promises per-account segregation and the seal checks \
             per-currency zero sums; it must not compile. got {c:?}"
        );
    }

    #[test]
    fn a_grouping_of_the_wrong_arity_is_refused() {
        assert!(codes(&with_rule("conserve per (txn);")).contains(&"NL0225".to_string()));
        assert!(codes(&with_rule("conserve per (txn, cur, acct);")).contains(&"NL0225".to_string()));
    }

    /// Naming the same column twice is not the enforced grouping either, and the diagnostic
    /// says which of the two things went wrong rather than offering a rewrite that would
    /// silently drop a key.
    #[test]
    fn a_repeated_key_inside_one_clause_is_refused() {
        assert!(codes(&with_rule("conserve per (txn, txn);")).contains(&"NL0225".to_string()));
    }

    /// **The silent overwrite.** `resolve` assigned the last clause over the first, so this
    /// program used to compile and be conserved by neither rule in it.
    #[test]
    fn a_second_conservation_rule_is_refused_rather_than_replacing_the_first() {
        let c = codes(&with_rule("conserve per (txn, cur); conserve per (acct);"));
        assert!(
            c.contains(&"NL0224".to_string()),
            "two conservation rules are two claims about what a sealed epoch satisfies, and \
             nothing decides between them. got {c:?}"
        );
    }

    /// A ledger with no `Currency` column has no grouping the seal can check, and saying so
    /// is better than reporting the *keys* as wrong — they may be exactly what the author
    /// meant, on a relation that cannot carry the rule.
    #[test]
    fn a_ledger_with_no_currency_column_is_refused_by_shape_not_by_key() {
        let c = codes(
            "schema s { ledger m { txn: TxnId, acct: Id<Account>, n: Int, \
             conserve per (txn, acct); retain forever; }\n\
             index ix on m (acct) anchor; }",
        );
        assert!(c.contains(&"NL0225".to_string()), "got {c:?}");
    }

    /// An unknown column is W3's diagnostic (NL0213) and must not also draw W3b's: two
    /// errors for one typo is noise, and the second one would recommend a rewrite of a rule
    /// the author has not finished writing.
    #[test]
    fn an_unknown_column_reports_once() {
        let c = codes(&with_rule("conserve per (txn, nope);"));
        assert!(c.contains(&"NL0213".to_string()), "got {c:?}");
        assert!(
            !c.contains(&"NL0225".to_string()),
            "a typo must not also be reported as an unenforced grouping: {c:?}"
        );
    }
}
