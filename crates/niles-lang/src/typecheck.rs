//! The body-level checker: where the currency-row solver and the effect calculus meet
//! actual source text.
//!
//! [`crate::resolve`] discharged the declaration-level rules. This phase walks function
//! and view bodies and discharges the rest: W4, W6, W7, W8, W14, W15, W16 and W17 of the
//! grammar's section 15, plus rung monotonicity.
//!
//! # What this checker is, and what it is not
//!
//! It is **not** a Hindley–Milner inference engine, and pretending otherwise would be
//! dishonest about the artifact. It is a *money-shaped* analysis: every expression is
//! classified into a small lattice — a monetary value in some currency, a linear ledger
//! half, or something else — and the interesting judgements are made over that
//! classification. That is sufficient for the four claims Contribution 4 rests on, because
//! all four are about money, currency, linearity and rung, and none is about polymorphic
//! recursion over user types.
//!
//! The boundary is stated rather than hidden: where the analysis cannot see through an
//! expression it reports *undecided* and discharges the obligation to the runtime, and
//! [`Report::runtime_obligations`] counts how often that happened. A checker that silently
//! assumed the good case for anything it could not follow would make the soundness theorem
//! vacuous.

use crate::ast::*;
use crate::currency_rows::{self as rows, Amount, Cur, Row as CurRow, Unifier, Verdict};
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::effects::{self, Effect, LinearKind, LinearValue, Row as EffRow, Rung};
use crate::lexer::Span;
use crate::resolve::Catalog;
use std::collections::HashMap;

/// A scalar the checker can name.
///
/// Not a type system — there is no inference here and no user types. It is the smallest
/// set that lets the checker say **no** to the two things a money-shaped analysis must not
/// wave through: arithmetic between values that have no arithmetic, and a `Money<c>`
/// annotation over an initializer that is plainly not money.
///
/// Before this existed, `let x: Money<usd> = "hello" + true;` passed `nilesc check`
/// reporting zero errors and zero runtime obligations, because both operands classified as
/// `Opaque` and `binary` returned `Opaque` without looking. A soundness theorem about
/// well-typed programs, over a checker that accepts that, is a theorem about the empty set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarKind {
    Int,
    Float,
    Text,
    Bool,
    Bytes,
    Unit,
    Duration,
    Instant,
    Epoch,
}

impl ScalarKind {
    fn name(self) -> &'static str {
        match self {
            ScalarKind::Int => "Int",
            ScalarKind::Float => "Float",
            ScalarKind::Text => "Text",
            ScalarKind::Bool => "Bool",
            ScalarKind::Bytes => "Bytes",
            ScalarKind::Unit => "()",
            ScalarKind::Duration => "Duration",
            ScalarKind::Instant => "Instant",
            ScalarKind::Epoch => "Epoch",
        }
    }
    /// Whether `+` and `-` are defined on two of these. Deliberately narrow: `Int + Int`
    /// and `Instant + Duration` and nothing else, because every other pair someone might
    /// want is a coercion, and a coercion in a ledger language is a silent conversion.
    fn adds_to(self, other: ScalarKind) -> bool {
        use ScalarKind::*;
        matches!(
            (self, other),
            (Int, Int)
                | (Float, Float)
                | (Instant, Duration)
                | (Duration, Instant)
                | (Duration, Duration)
                | (Epoch, Int)
                | (Epoch, Epoch)
        )
    }
    /// Whether the two may be compared. Same kind, or the two clock-shaped pairs.
    fn compares_to(self, other: ScalarKind) -> bool {
        self == other || matches!((self, other), (ScalarKind::Epoch, ScalarKind::Int))
    }
}

/// The classification an expression is given. Small on purpose.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// A monetary value: a currency (possibly a variable) and a symbolic amount.
    Money(Cur, Amount),
    /// A linear ledger half or hold, which must be consumed exactly once.
    Linear(LinearKind),
    /// A capability. **Unforgeable**, which here means: this shape has exactly two
    /// introduction forms — an `Auth<E>` parameter, and a `grant` — and no expression
    /// produces one. The string is the effect it authorises, as written.
    Auth(String),
    /// A scalar the checker can name.
    Scalar(ScalarKind),
    /// Anything else the checker does not need to look inside. Still `Opaque`, and still
    /// the honest answer for a user type or a call it cannot follow — but no longer the
    /// answer for a string literal.
    Opaque,
}

impl Shape {
    /// A short description for a diagnostic.
    fn describe(&self) -> String {
        match self {
            Shape::Money(c, _) => format!("`Money<{c}>`"),
            Shape::Linear(_) => "a ledger half".into(),
            Shape::Auth(e) => format!("`Auth<{e}>`"),
            Shape::Scalar(k) => format!("`{}`", k.name()),
            Shape::Opaque => "a value the checker cannot see inside".into(),
        }
    }
}

/// What a check concluded, beyond the diagnostics.
#[derive(Debug, Default)]
pub struct Report {
    /// Conservation obligations the checker proved statically.
    pub conservation_proved: usize,
    /// Obligations it could not decide, and handed to the runtime. Reported, never hidden.
    ///
    /// The sum of [`Report::undecided`] and [`Report::may_violate`], kept because callers
    /// have depended on it and because "how much is left for the runtime" is the number an
    /// operator wants.
    pub runtime_obligations: usize,
    /// Rows the checker could not see through: an amount it cannot decide is zero.
    ///
    /// Split out from `runtime_obligations` for the E18 measurement. The two halves mean
    /// different things — `undecided` is the checker admitting the arithmetic is beyond its
    /// domain, `may_violate` is the arithmetic failing along a path it cannot guarantee is
    /// taken — and a corpus that reported only their sum could not tell whether the analysis
    /// is weak or the programs are.
    pub undecided: usize,
    /// Rows whose arithmetic does not balance, reached across a merge or a possible abort.
    pub may_violate: usize,
    /// Must-violations: straight-line, abort-free, provably unbalanced.
    pub violates: usize,
    /// Effect rows inferred per function, for `nilesc effects` and for lowering.
    pub inferred_effects: HashMap<String, EffRow>,
    /// The weakest rung each view body reads at, which is the ceiling on what it may promise.
    pub view_rungs: HashMap<String, Option<Rung>>,
    /// The relations and views each view body reads, by name.
    ///
    /// The rung alone does not say *what* was read, and "reads no stricter than
    /// ledger_consistent" is true of a view reading the ledger directly and of one reading
    /// another view at the same rung. Telling the two apart is the whole question when the
    /// claim under test is "`available_balance` is ledger balance minus encumbrances".
    pub view_sources: HashMap<String, std::collections::BTreeSet<String>>,
}

/// What one function does, as seen from a call site.
///
/// **The object F-11(d) and F-11(e) were about.** Before this existed, a call to a function
/// declared in the same file contributed *nothing* to its caller: not its reads, so a
/// `ledger_consistent` view reading a `bounded` view through one helper passed rung
/// monotonicity; not its money, so `txn { leak_half(a, m) }` — whose callee posts one half
/// of a transfer — was neither proved conserving nor discharged to the runtime. It was not
/// counted at all. The obligation did not become a runtime check; it disappeared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamKind {
    /// `Money<usd>`, or currency-generic `Money`.
    Money(Option<String>),
    /// `Auth<E>`, as written.
    Auth(String),
    /// A shape the checker does not look inside. Not "anything goes" — it is the honest
    /// statement that this parameter's type is outside the money-shaped analysis, so no
    /// judgement is made about it here.
    Other,
}

#[derive(Debug, Clone, Default)]
pub struct FnSummary {
    /// Effects the body incurs, including everything inherited from its own callees.
    pub effects: EffRow,
    /// The body's net currency row, in terms of the parameter symbols below.
    net: CurRow,
    /// The `(currency, amount-symbol)` given to each money parameter, by position. `None`
    /// for a parameter that is not money.
    param_money: Vec<Option<(Cur, u32)>>,
    /// What each parameter's declared type is, in the shapes the checker can name, and
    /// the name it was declared under.
    ///
    /// A call site checks its arguments against this. Nothing did before: `call` collected
    /// the argument *shapes* and threw them away except to look for one money value, so
    /// `transfer(a, b, 10.00 eur)` against `m: Money<usd>` was accepted, and so was an
    /// `Auth` position filled by a call returning anything at all.
    params: Vec<ParamKind>,
    /// **Havoc.** Set when the summary could not be computed: the call graph has a cycle
    /// through this function, or the fixpoint did not settle inside its round budget.
    ///
    /// A havoc summary contributes an undecided movement rather than a proof, and the flag
    /// is what makes that visible instead of it looking like a clean answer. A fresh symbol
    /// with no havoc record is exactly the fabrication this whole pass is against.
    pub havoc: bool,
}

pub fn check_program(prog: &Program, cat: &Catalog) -> (Report, Diagnostics) {
    // Every function in the program, including those nested in `mod`s and `impl`s, by
    // name. Name collisions are the resolver's business (NL0101); here the last wins,
    // which matches what the resolver reports on.
    let mut decls: Vec<FnDecl> = Vec::new();
    for item in &prog.items {
        collect_fns(item, &mut decls);
    }
    let order: Vec<String> = decls.iter().map(|f| f.name.text.clone()).collect();

    // **The fixpoint.** Summaries start empty and are recomputed until they stop changing.
    // Effects are a finite lattice so that half always converges; the currency row can
    // grow through a recursive call, which is what the round budget is for. A function
    // still moving when the budget runs out is marked havoc — recorded, not assumed.
    let mut summaries: HashMap<String, FnSummary> = HashMap::new();
    let budget = order.len() + 2;
    let mut settled = false;
    for _ in 0..budget {
        let mut next: HashMap<String, FnSummary> = HashMap::new();
        for f in &decls {
            let mut cx = Cx::new(cat, summaries.clone());
            let s = cx.summarise(f);
            next.insert(f.name.text.clone(), s);
        }
        if summary_effects_equal(&summaries, &next) {
            summaries = next;
            settled = true;
            break;
        }
        summaries = next;
    }
    if !settled {
        for name in &order {
            if let Some(s) = summaries.get_mut(name) {
                s.havoc = true;
            }
        }
    }
    // A function that calls itself, directly or through others, cannot be summarised by
    // this scheme: its own row appears on both sides. Marked rather than guessed at.
    for name in mutually_recursive(&decls) {
        if let Some(s) = summaries.get_mut(&name) {
            s.havoc = true;
        }
    }

    // The reporting pass. Same walk, same summaries, diagnostics on.
    let mut cx = Cx::new(cat, summaries);
    cx.emit = true;
    for item in &prog.items {
        cx.item(item);
    }
    (cx.report, cx.d)
}

/// What each parameter's declared type is, in the shapes the checker can name.
fn param_kinds(f: &FnDecl) -> Vec<ParamKind> {
    f.params
        .iter()
        .map(
            |p| match (money_currency_of(&p.ty), auth_effect_of(&p.ty)) {
                (Some(cur), _) => ParamKind::Money(cur),
                (_, Some(eff)) => ParamKind::Auth(eff),
                _ => ParamKind::Other,
            },
        )
        .collect()
}

fn collect_fns(item: &Item, out: &mut Vec<FnDecl>) {
    match item {
        Item::Fn(f) => out.push(f.clone()),
        Item::Mod { items, .. } => items.iter().for_each(|i| collect_fns(i, out)),
        Item::Impl(i) => out.extend(i.items.iter().cloned()),
        _ => {}
    }
}

/// Convergence is judged on the **effect rows**, which are the half that has a finite
/// lattice and therefore the half a fixpoint can be said to reach. The currency rows ride
/// along; a function whose row is still growing is caught by the round budget and by the
/// recursion check, both of which set `havoc`.
fn summary_effects_equal(a: &HashMap<String, FnSummary>, b: &HashMap<String, FnSummary>) -> bool {
    a.len() == b.len()
        && a.iter()
            .all(|(k, v)| b.get(k).is_some_and(|w| w.effects == v.effects))
}

/// Every function on a cycle of the call graph, by name.
///
/// The call graph is read from the syntax: a name in call position that is also a declared
/// function is an edge. That over-approximates — a shadowed local of the same name would
/// count — and over-approximating here costs precision (`havoc`) rather than soundness,
/// which is the direction this whole file errs in.
fn mutually_recursive(decls: &[FnDecl]) -> Vec<String> {
    let names: std::collections::BTreeSet<&str> =
        decls.iter().map(|f| f.name.text.as_str()).collect();
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    for f in decls {
        let mut called = Vec::new();
        if let Some(b) = &f.body {
            calls_in_block(b, &mut called);
        }
        called.retain(|c| names.contains(c.as_str()));
        edges.insert(f.name.text.clone(), called);
    }
    // Transitive closure, then anything reaching itself. The graphs here are one file's
    // worth of functions, so the cubic step is not worth avoiding.
    let mut reach: HashMap<String, std::collections::BTreeSet<String>> = edges
        .iter()
        .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
        .collect();
    for _ in 0..decls.len() {
        let mut changed = false;
        let keys: Vec<String> = reach.keys().cloned().collect();
        for k in keys {
            let current: Vec<String> = reach[&k].iter().cloned().collect();
            for m in current {
                if let Some(more) = reach.get(&m).cloned() {
                    for x in more {
                        if reach.get_mut(&k).is_some_and(|s| s.insert(x)) {
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    reach
        .into_iter()
        .filter(|(k, v)| v.contains(k))
        .map(|(k, _)| k)
        .collect()
}

struct Cx<'a> {
    cat: &'a Catalog,
    d: Diagnostics,
    report: Report,
    unifier: Unifier,
    next_symbol: u32,
    /// Summaries of every function in the program, from the fixpoint above.
    summaries: HashMap<String, FnSummary>,
    /// Whether diagnostics from this walk are kept. Off during the fixpoint rounds, where
    /// the same body is walked several times and reporting each pass would multiply every
    /// error by the number of rounds.
    emit: bool,
}

/// Per-scope state: the currency row being accumulated, the effects incurred, and the
/// linear values in flight.
#[derive(Default)]
struct Scope {
    row: CurRow,
    effects: EffRow,
    linear: Vec<LinearValue>,
    /// Set inside a `txn` or an `fx` leg, where a conservation obligation is open.
    conserving: bool,
    /// What each `let`-bound name and parameter is, in the money-shaped lattice.
    ///
    /// Without this the checker sees `debit(tenant, rent)` and cannot tell what currency
    /// `rent` is, so it invents a variable and then reports the function's declared
    /// `debit<usd>` as unsatisfied. The resulting diagnostic is technically true and
    /// completely useless, which is the usual shape of an under-powered checker's output.
    bindings: HashMap<String, Shape>,
    /// Inside a loop body: how many linear values were already in flight when the body was
    /// entered, so an exit path knows which of them the body itself created.
    ///
    /// `None` outside a loop, which is what makes `break` outside a loop a parse-level
    /// question rather than one this file answers.
    loop_base: Option<usize>,
    /// The row as it stood at each `break`.
    ///
    /// A `break` leaves the loop with a *prefix* of the body's movements, and the prefix is
    /// an alternative path through the loop. Before this, `break` returned `Opaque` and the
    /// row it left behind was folded in as though the whole body had run.
    exits: Vec<CurRow>,
}

impl Scope {
    /// A child scope that inherits bindings but opens its own conservation obligation.
    /// Bindings flow in because a `txn` block sees the `let`s above it; the row does not,
    /// because the obligation belongs to the transaction.
    fn child_conserving(&self) -> Scope {
        Scope {
            conserving: true,
            bindings: self.bindings.clone(),
            loop_base: self.loop_base,
            ..Default::default()
        }
    }

    /// A scope for one arm of a branch.
    ///
    /// It inherits the bindings and the linear values in flight — a hold bound before an
    /// `if` may legitimately be resolved inside one arm — but starts with an **empty row**,
    /// because the arms' effects on the ledger are alternatives, not a sequence. Summing
    /// them was the bug: `if p { credit(a, 10) } else { credit(b, 10) }` accumulated
    /// twenty dollars of credit against ten of debit and reported a conserving program as
    /// creating money.
    fn branch(&self) -> Scope {
        Scope {
            row: CurRow::default(),
            effects: EffRow::new(),
            linear: self.linear.clone(),
            conserving: self.conserving,
            bindings: self.bindings.clone(),
            loop_base: self.loop_base,
            exits: Vec::new(),
        }
    }

    /// A scope for a loop body: a branch that also records where the loop's own linear
    /// values start.
    fn loop_body(&self) -> Scope {
        Scope {
            loop_base: Some(self.linear.len()),
            ..self.branch()
        }
    }
}

/// Merge the arms of a branch back into the parent scope.
///
/// Three things merge, each with its own rule.
///
/// * **The row** joins pointwise: agreed entries survive, disagreements go to top.
/// * **Effects** union, because an effect on any path is an effect the function has.
/// * **Linear uses** take the *maximum across arms*, not the sum. A hold resolved once in
///   each arm of an `if` is resolved once, not twice — summing them was the second half of
///   the same bug, and it reported a correct program as consuming a hold twice.
///
/// Linear values *declared inside* an arm never reach here: they are checked at the end of
/// that arm, because a hold created in one branch must be resolved in that branch.
///
/// # Judging each arm before the join
///
/// The join is deliberately lossy: where two arms disagree on a currency's entry it goes to
/// top, so the merged row is `Undecided` rather than an accusation. That is sound and it is
/// the right default — a checker that accused a program it could not follow would teach its
/// users to switch it off.
///
/// But it also **discards a real finding**. The E18 corpus made this concrete: a transaction
/// whose `else` branch is short by fifty cents merges to `Undecided`, and the user is handed a
/// runtime obligation rather than told that one path does not balance. The checker knew, and
/// said nothing.
///
/// So each arm's row is checked *before* the join, and a decided non-zero residue in one arm
/// is reported as a **may**-violation — true by construction, since some path is unbalanced,
/// and never an accusation, since which path executes is not decidable here. This is the case
/// `Verdict::MayViolate` was designed for and, until E18 measured it, the case it never saw.
fn merge_branches(
    parent: &mut Scope,
    arms: Vec<Scope>,
    at: Span,
    branch_alarms: &mut usize,
    scale_of: &dyn Fn(&str) -> Option<u32>,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if arms.is_empty() {
        return diags;
    }
    let base_len = parent.linear.len();
    let mut merged_row: Option<CurRow> = None;
    for mut arm in arms {
        // Anything the arm declared itself must be consumed within the arm.
        let arm_local: Vec<LinearValue> = arm.linear.split_off(base_len.min(arm.linear.len()));
        diags.extend(effects::check_linearity(&arm_local));

        parent.effects.union(&arm.effects);
        // Per inherited value, take the maximum use count any arm reached.
        for (i, v) in arm.linear.iter().enumerate() {
            if i < parent.linear.len() && v.uses.len() > parent.linear[i].uses.len() {
                parent.linear[i].uses = v.uses.clone();
            }
        }
        // Judge this arm on its own, before the join loses the evidence. See the doc comment.
        for v in rows::check_conservation(&arm.row) {
            if let rows::Verdict::Violates {
                currency,
                residue,
                span,
            } = v
            {
                // Downgraded: the arithmetic is definitely wrong, and whether this arm runs is
                // not something the checker can decide, so it is an alarm rather than a proof.
                let alarm_currency = currency.clone();
                let alarm = rows::Verdict::MayViolate {
                    currency,
                    residue,
                    span,
                    why: rows::Provenance::Merged,
                };
                *branch_alarms += 1;
                // The currency's *declared* scale, not 2. A hard-coded 2 printed a JPY
                // residue of 1000 as "10.00" — a wrong number in a diagnostic a reader is
                // meant to reconcile against.
                let Some(scale) = scale_of(&alarm_currency) else {
                    continue;
                };
                if let Some(d) = rows::diagnose(&alarm, scale) {
                    diags.push(d.note(
                        "this branch does not balance on its own. Whether it executes is not \
                         decidable here, so this is a warning rather than an error — but the \
                         residue is exact",
                    ));
                }
            }
        }

        merged_row = Some(match merged_row {
            None => arm.row,
            Some(acc) => acc.join(arm.row, at),
        });
    }
    if let Some(r) = merged_row {
        parent.row.merge(&r);
        // The join itself is what weakens the provenance; `merge` carries it up.
    }
    diags
}

impl<'a> Cx<'a> {
    fn new(cat: &'a Catalog, summaries: HashMap<String, FnSummary>) -> Cx<'a> {
        Cx {
            cat,
            d: Diagnostics::new(),
            report: Report::default(),
            unifier: Unifier::new(),
            next_symbol: 0,
            summaries,
            emit: false,
        }
    }

    /// Push a diagnostic, unless this is a fixpoint round.
    fn push(&mut self, d: Diagnostic) {
        if self.emit {
            self.d.push(d);
        }
    }

    /// Walk one function's body and return what a caller needs to know about it.
    ///
    /// The *same* walk the reporting pass uses, deliberately: a summary computed by a
    /// second, simpler traversal would be a second semantics, and the two would disagree
    /// on exactly the constructs that matter.
    fn summarise(&mut self, f: &FnDecl) -> FnSummary {
        let Some(body) = &f.body else {
            // A declaration without a body — a trait method, an extern. Its declared row
            // is all a caller has, and it is the honest thing to use.
            return FnSummary {
                effects: f.effects.as_ref().map(to_eff_row).unwrap_or_default(),
                net: CurRow::default(),
                param_money: Vec::new(),
                params: param_kinds(f),
                havoc: f.effects.is_none(),
            };
        };
        let mut sc = Scope::default();
        let param_money = self.seed_params(f, &mut sc);
        // The body's net row is accumulated in a conserving scope so that `debit` and
        // `credit` reach it. This is not an assertion that the function conserves — it is
        // the record of what it moves, which is precisely what a caller inside its own
        // `txn` needs in order to decide whether *it* conserves.
        sc.conserving = true;
        self.block(body, &mut sc);
        let mut net = std::mem::take(&mut sc.row);
        net.substitute(&self.unifier);
        FnSummary {
            effects: sc.effects,
            net,
            param_money,
            params: param_kinds(f),
            havoc: false,
        }
    }

    fn fresh_symbol(&mut self) -> u32 {
        self.next_symbol += 1;
        self.next_symbol
    }

    fn item(&mut self, item: &Item) {
        match item {
            Item::Schema(s) => {
                for si in &s.items {
                    if let SchemaItem::View(v) = si {
                        self.view(v);
                    }
                }
            }
            Item::Mod { items, .. } => items.iter().for_each(|i| self.item(i)),
            Item::View(v) => self.view(v),
            Item::Fn(f) => self.function(f),
            Item::Impl(i) => {
                let fns = i.items.clone();
                fns.iter().for_each(|f| self.function(f));
            }
            _ => {}
        }
    }

    // ---------------- functions ----------------

    /// Seed a scope from a signature, and report the money each parameter carries.
    ///
    /// A parameter declared `Money<usd>` is money in USD everywhere in the body, which is
    /// the point of writing the type. The returned vector is positional, and it is what
    /// makes a call site able to substitute the *caller's* amounts into the callee's row.
    fn seed_params(&mut self, f: &FnDecl, sc: &mut Scope) -> Vec<Option<(Cur, u32)>> {
        let mut out = Vec::with_capacity(f.params.len());
        for p in &f.params {
            let money = match (money_currency_of(&p.ty), first_binding(&p.pat)) {
                (Some(cur), Some(name)) => {
                    let sym = self.fresh_symbol();
                    let c = match cur {
                        Some(c) => Cur::Known(c),
                        None => self.unifier.fresh(),
                    };
                    sc.bindings
                        .insert(name, Shape::Money(c.clone(), Amount::symbol(sym)));
                    Some((c, sym))
                }
                _ => None,
            };
            out.push(money);
            // A capability is bound as a *value* the checker can follow, not only as a
            // fact about the signature. Without the binding there was no way to tell a
            // genuine `Auth` from anything else, and therefore no way to refuse a forged
            // one.
            if let (Some(eff), Some(name)) = (auth_effect_of(&p.ty), first_binding(&p.pat)) {
                sc.bindings.insert(name, Shape::Auth(eff));
            }
        }
        out
    }

    /// The capabilities a function holds: its `Auth<E>` parameters. They cannot be
    /// constructed, only received or granted, which is what makes them unforgeable.
    fn capabilities_of(&self, f: &FnDecl) -> EffRow {
        let mut caps = EffRow::new();
        for p in &f.params {
            let Ty::Path { path, args, .. } = &p.ty else {
                continue;
            };
            if path.last().text != "Auth" {
                continue;
            }
            if let Some(Ty::Path {
                path: e,
                args: eargs,
                ..
            }) = args.first()
            {
                let names: Vec<String> = eargs
                    .iter()
                    .filter_map(|t| match t {
                        Ty::Path { path, .. } => Some(path.last().text.clone()),
                        _ => None,
                    })
                    .collect();
                if let Some(eff) = Effect::parse(&e.last().text, None, &names) {
                    caps.add(eff, p.span);
                }
            }
        }
        caps
    }

    fn function(&mut self, f: &FnDecl) {
        let Some(body) = &f.body else { return };
        let mut sc = Scope::default();
        self.seed_params(f, &mut sc);
        let caps = self.capabilities_of(f);
        let tail = self.block(body, &mut sc);

        // **W7 in return position.** The same rule as the `let` annotation: a signature that
        // says `-> Money<usd>` over a body that plainly yields `Money<eur>` is a currency
        // conversion spelled as a type. Only the tail is checked here — an early `return`
        // carrying the wrong currency is a path this checker does not track, and saying so
        // is better than implying it does.
        if let Some(Some(want)) = f.ret.as_ref().map(money_currency_of) {
            if let (Some(want), Shape::Money(Cur::Known(got), _)) = (want.as_ref(), &tail) {
                if got != want {
                    let at = body.tail.as_ref().map(|t| t.span()).unwrap_or(f.name.span);
                    let written = f.ret.as_ref().map(|t| t.span()).unwrap_or(f.name.span);
                    let d = currency_annotation_mismatch(want, got, at, written, "returns");
                    self.push(d);
                }
            }
        }

        let decl_span = f.effects.as_ref().map(|r| r.span).unwrap_or(f.name.span);
        // W14: the inferred row must be permitted by the declared one.
        if let Some(declared) = &f.effects {
            let declared_row = to_eff_row(declared);
            for diag in
                effects::check_declaration(&f.name.text, &sc.effects, &declared_row, decl_span)
            {
                self.push(diag);
            }
        } else {
            // Undeclared effects are a warning, not an error: inference exists, and forcing
            // an annotation on every helper would make the language tiresome for no safety
            // gain. But a function that appends or debits and does not say so is exactly
            // the function a reviewer needs to be able to find.
            let interesting: Vec<String> = sc
                .effects
                .iter()
                .filter(|e| !matches!(e, Effect::Read(_)))
                .map(|e| e.to_string())
                .collect();
            if !interesting.is_empty() {
                self.push(
                    Diagnostic::warning(
                        "NL0313",
                        format!("`{}` has undeclared effects {}", f.name.text, sc.effects),
                    )
                    .primary(f.name.span, "no effect row declared")
                    .note("an effect row is part of the signature: a caller should be able to see from the type that this function moves money")
                    .suggest(
                        f.name.span,
                        format!("! {{ {} }}", interesting.join(", ")),
                        "declare the inferred row",
                        crate::diagnostics::Applicability::MachineApplicable,
                    ),
                );
            }
        }

        // W15/W16: authority.
        for diag in effects::check_authority(&f.name.text, &sc.effects, &caps, decl_span) {
            self.push(diag);
        }
        // W8: linearity.
        for diag in effects::check_linearity(&sc.linear) {
            self.push(diag);
        }
        self.report
            .inferred_effects
            .insert(f.name.text.clone(), sc.effects);
    }

    // ---------------- views ----------------

    fn view(&mut self, v: &ViewDecl) {
        let mut sc = Scope::default();
        self.expr(&v.body, &mut sc);
        let Some(info) = self.cat.views.get(&v.name.text) else {
            return;
        };
        // Rung monotonicity: the judgement that makes "two derived views of one ledger
        // disagreeing at the moment a decision is made" unspellable.
        for diag in effects::check_rung_monotonicity(
            &v.name.text,
            info.rung,
            &sc.effects,
            v.body.span(),
            info.contract_span,
        ) {
            self.push(diag);
        }
        self.report
            .view_rungs
            .insert(v.name.text.clone(), sc.effects.weakest_read());
        let mut sources = std::collections::BTreeSet::new();
        collect_sources(&v.body, self.cat, &mut sources);
        self.report
            .view_sources
            .insert(v.name.text.clone(), sources);
    }

    // ---------------- statements ----------------

    /// Check a block, and return the shape of its tail expression.
    ///
    /// The return value is what makes a *return position* checkable: `fn f() -> Money<usd>`
    /// with `10.00 eur` for a body is the same laundering as `let m: Money<usd> = 10.00 eur`,
    /// arriving through the one position that was not looked at. Callers that do not care
    /// about the tail ignore it, as they did when this returned nothing.
    fn block(&mut self, b: &Block, sc: &mut Scope) -> Shape {
        for s in &b.stmts {
            self.stmt(s, sc);
        }
        match &b.tail {
            Some(t) => self.expr(t, sc),
            None => Shape::Opaque,
        }
    }

    fn stmt(&mut self, s: &Stmt, sc: &mut Scope) {
        match s {
            Stmt::Let { pat, ty, init, .. } => {
                let init_shape = init.as_ref().map(|e| self.expr(e, sc));
                let mut shape = init_shape.clone().unwrap_or(Shape::Opaque);

                // **The capability rule.** `Auth<E>` has no introduction form that is an
                // expression: it arrives as a parameter or from a `grant`, and nothing
                // else produces one. Without this, `let auth: Auth<authorize<usd>> = 42;`
                // passed `nilesc check` with no diagnostic at all — so the "unforgeable"
                // in Theorem 4.4's authority clause was a claim about the prose.
                if let Some(eff) = ty.as_ref().and_then(auth_effect_of) {
                    match &init_shape {
                        Some(Shape::Auth(_)) | None => {}
                        Some(other) => {
                            self.push(
                                Diagnostic::error(
                                    "NL0330",
                                    format!("`Auth<{eff}>` cannot be constructed"),
                                )
                                .primary(
                                    init.as_ref().map(|e| e.span()).unwrap_or_default(),
                                    format!("this is {}", other.describe()),
                                )
                                .note("a capability is unforgeable: it is received as a parameter or granted, never built from a value")
                                .note("that is the whole of what `unforgeable` means here — an annotation that could be satisfied by a literal would make the authority clause of the soundness theorem vacuous")
                                .suggest(
                                    pat.span(),
                                    format!("auth: Auth<{eff}>"),
                                    "take it as a parameter of the enclosing function",
                                    crate::diagnostics::Applicability::HasPlaceholders,
                                ),
                            );
                        }
                    }
                    shape = Shape::Auth(eff);
                }

                // An explicit annotation wins over inference: `let x: Money<jpy> = f();`
                // is the programmer telling the checker something it could not see. It
                // does not win over an initializer the checker *can* see and that is
                // plainly something else — that is an annotation contradicting its own
                // right-hand side, and taking the annotation's word for it is how
                // `let x: Money<usd> = "hello";` type-checked.
                if let Some(Some(cur)) = ty.as_ref().map(money_currency_of) {
                    // **W7 in a binding position.** The paragraph above says an annotation
                    // does not overrule what the checker can see, and then looked only at
                    // scalars — so `let m: Money<usd> = 10.00 eur;` was accepted, and every
                    // later use of `m` was `Money<usd>` on the checker's word. The effect
                    // row of the enclosing function then read `debit<usd>, credit<usd>`
                    // over legs the interpreter posts in EUR. That is Theorem 4.4's
                    // currency clause failing at the checker, not in the prose.
                    if let (Some(want), Some(Shape::Money(Cur::Known(got), _))) =
                        (cur.as_ref(), &init_shape)
                    {
                        if got != want {
                            let d = currency_annotation_mismatch(
                                want,
                                got,
                                init.as_ref().map(|e| e.span()).unwrap_or_default(),
                                ty.as_ref().map(|t| t.span()).unwrap_or_default(),
                                "annotated as",
                            );
                            self.push(d);
                        }
                    }
                    if let Some(Shape::Scalar(k)) = &init_shape {
                        self.push(
                            Diagnostic::error(
                                "NL0253",
                                format!("this is `{}`, not money", k.name()),
                            )
                            .primary(
                                init.as_ref().map(|e| e.span()).unwrap_or_default(),
                                format!("`{}`", k.name()),
                            )
                            .secondary(
                                ty.as_ref().map(|t| t.span()).unwrap_or_default(),
                                "annotated as money here",
                            )
                            .note("an annotation tells the checker what it could not see; it does not overrule what it can")
                            .note("a money value carries its currency and scale: write `10.00 usd`, or convert explicitly"),
                        );
                    }
                    let sym = self.fresh_symbol();
                    let c = match cur {
                        Some(c) => Cur::Known(c),
                        None => self.unifier.fresh(),
                    };
                    shape = Shape::Money(c, Amount::symbol(sym));
                }
                if matches!(&shape, Shape::Money(..) | Shape::Auth(_) | Shape::Scalar(_)) {
                    for n in pat.bindings() {
                        sc.bindings.insert(n.text.clone(), shape.clone());
                    }
                }
                if let Shape::Linear(kind) = shape {
                    for n in pat.bindings() {
                        sc.linear.push(LinearValue {
                            name: n.text.clone(),
                            kind,
                            bound_at: n.span,
                            uses: Vec::new(),
                        });
                    }
                }
            }
            Stmt::Expr(e) | Stmt::Semi(e) => {
                self.expr(e, sc);
            }
            Stmt::Item(i) => self.item(i),
            Stmt::Dml(dml) => self.dml(dml, sc),
            Stmt::Error(_) => {}
        }
    }

    fn dml(&mut self, dml: &Dml, sc: &mut Scope) {
        // W4: `update` and `delete` are legal against a `table` only. This is the single
        // most important static difference between a base and a table, and it is what
        // "the base is never partial" means at the level of a program.
        let mutation = match dml {
            Dml::Update { table, span, .. } => Some((table, *span, "update")),
            Dml::Delete { table, span, .. } => Some((table, *span, "delete")),
            Dml::Insert { table, span, .. } => Some((table, *span, "insert")),
            _ => None,
        };
        if let Some((name, span, verb)) = mutation {
            if let Some(rel) = self.cat.relations.get(&name.text) {
                if rel.is_base() {
                    let kind = if rel.kind == RelKind::Ledger {
                        "ledger"
                    } else {
                        "base"
                    };
                    let alt = if verb == "insert" {
                        "append to it inside a `txn { .. }`"
                    } else {
                        "append a compensating entry"
                    };
                    self.push(
                        Diagnostic::error("NL0230", format!("`{verb}` is not legal against {kind} `{}`", name.text))
                            .primary(span, format!("`{}` is immutable and fully retained", name.text))
                            .secondary(rel.span, format!("declared as a {kind} here"))
                            .note("history is the authority: a fact that can be edited is not evidence, and reconstruction over an edited base is not reproduction")
                            .note(format!("to change the world, {alt}")),
                    );
                }
            }
            sc.effects.add(
                if verb == "insert" {
                    Effect::Append
                } else {
                    Effect::Mutate
                },
                span,
            );
        }
        match dml {
            Dml::Grant { span, .. } | Dml::Revoke { span, .. } | Dml::Backfill { span, .. } => {
                sc.effects.add(Effect::Admin, *span)
            }
            Dml::Emit { span, .. } => sc.effects.add(Effect::Emit, *span),
            _ => {}
        }
    }

    // ---------------- expressions ----------------

    fn expr(&mut self, e: &Expr, sc: &mut Scope) -> Shape {
        match e {
            // ---- scalars the checker can name ----
            Expr::Int(..) => Shape::Scalar(ScalarKind::Int),
            Expr::Float(..) => Shape::Scalar(ScalarKind::Float),
            Expr::Bool(..) => Shape::Scalar(ScalarKind::Bool),
            Expr::Str(..) => Shape::Scalar(ScalarKind::Text),
            Expr::Bytes(..) => Shape::Scalar(ScalarKind::Bytes),
            Expr::Unit(..) => Shape::Scalar(ScalarKind::Unit),
            Expr::Duration { .. } => Shape::Scalar(ScalarKind::Duration),
            Expr::Instant { .. } => Shape::Scalar(ScalarKind::Instant),
            Expr::Epoch(..) => Shape::Scalar(ScalarKind::Epoch),

            // ---- money ----
            Expr::Money {
                minor,
                scale,
                currency,
                span,
            } => {
                // W6: the literal's own scale must equal the currency's declared scale.
                if let Some(info) = self.cat.currencies.get(&currency.text) {
                    if info.scale != *scale {
                        self.push(
                            Diagnostic::error(
                                "NL0240",
                                format!("this literal has {scale} decimal places but `{}` has scale {}", currency.text, info.scale),
                            )
                            .primary(*span, format!("written with {scale} decimal places"))
                            .secondary(info.span, format!("`{}` declared with scale {}", currency.text, info.scale))
                            .note("the scale is part of the type; rounding a money literal silently is how a reconciliation breaks months later with no evidence of where"),
                        );
                    }
                } else {
                    self.push(
                        Diagnostic::error(
                            "NL0241",
                            format!("currency `{}` is not declared", currency.text),
                        )
                        .primary(*span, "unknown currency")
                        .note("declare it with its minor-unit scale: `currency xyz { scale: 2 }`"),
                    );
                }
                Shape::Money(Cur::Known(currency.text.clone()), Amount::constant(*minor))
            }

            // ---- the conserving forms ----
            Expr::Txn { body, span, idem } => {
                // **W19, second clause.** "An `idem` key must declare a window; a window
                // without a key is a static error." The first clause is enforced on the
                // relation (NL0215). The second had no enforcement site at all: `txn
                // idem(window: 30.days) { .. }` parsed, bound no key, and produced a
                // transaction whose identity is the empty string — so two different
                // transactions written that way are the *same* transaction to the
                // idempotency store, which is the opposite of what the window was asked for.
                if let Some(spec) = idem {
                    if matches!(&*spec.key, Expr::Error(_)) && spec.window.is_some() {
                        self.push(
                            Diagnostic::error(
                                "NL0216",
                                "an idempotency window without a key",
                            )
                            .primary(spec.span, "a `window:` but nothing to key it by")
                            .note("a window bounds how long a key is remembered; with no key there is nothing to remember, and every transaction written this way shares one identity")
                            .note("W19: an `idem` key must declare a window; a window without a key is a static error")
                            .suggest(
                                spec.span,
                                "idem(\"name\", window: 30.days)",
                                "give the transaction an identity",
                                crate::diagnostics::Applicability::HasPlaceholders,
                            ),
                        );
                    }
                }
                let mut inner = sc.child_conserving();
                self.block(body, &mut inner);
                sc.effects.union(&inner.effects);
                sc.effects.add(Effect::Append, *span);
                sc.linear.append(&mut inner.linear);
                let mut row = std::mem::take(&mut inner.row);
                self.discharge_conservation(&mut row, *span, None);
                Shape::Opaque
            }

            Expr::Fx { legs, rate, span } => {
                // Two rows, each required to be zero. There is no single currency in which
                // a cross-currency transaction sums to zero, which is precisely why this is
                // a form and not a pair of transfers.
                for (name, leg) in legs {
                    let mut inner = sc.child_conserving();
                    self.expr(leg, &mut inner);
                    sc.effects.union(&inner.effects);
                    sc.linear.append(&mut inner.linear);
                    let mut row = std::mem::take(&mut inner.row);
                    self.discharge_conservation(&mut row, name.span, Some(&name.text));
                }
                if let Some(r) = rate {
                    self.expr(r, sc);
                } else {
                    self.push(
                        Diagnostic::error("NL0242", "an `fx` form must record its rate")
                            .primary(*span, "no `rate:` given")
                            .note("the rate is what makes the two legs auditable as one conversion rather than two unrelated transactions"),
                    );
                }
                sc.effects.add(Effect::Append, *span);
                Shape::Opaque
            }

            // ---- calls, including the linear constructors ----
            Expr::Call { callee, args, span } => {
                let name = match &**callee {
                    Expr::Path(p) => p.last().text.clone(),
                    other => {
                        self.expr(other, sc);
                        String::new()
                    }
                };
                let shapes: Vec<Shape> = args.iter().map(|a| self.expr(&a.value, sc)).collect();
                self.call(&name, &shapes, args, *span, sc)
            }

            Expr::Hold { args, span } => {
                let shapes: Vec<Shape> = args.iter().map(|a| self.expr(&a.value, sc)).collect();
                let cur = shapes
                    .iter()
                    .find_map(|s| match s {
                        Shape::Money(c, _) => Some(c.to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "*".into());
                sc.effects.add(Effect::Hold(cur), *span);
                sc.effects.add(Effect::Append, *span);
                Shape::Linear(LinearKind::Hold)
            }

            Expr::Resolve {
                hold,
                outcome,
                span,
            } => {
                self.consume_linear(hold, sc, *span);
                if let ResolveOutcome::Post(a) = outcome {
                    self.expr(a, sc);
                }
                sc.effects.add(Effect::Append, *span);
                Shape::Opaque
            }

            Expr::Authorize { args, span } => {
                let shapes: Vec<Shape> = args.iter().map(|a| self.expr(&a.value, sc)).collect();
                let cur = shapes
                    .iter()
                    .find_map(|s| match s {
                        Shape::Money(c, _) => Some(c.to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "*".into());
                // W16: the only construct that may reduce a balance below zero. That it
                // *requires* authority is checked in `effects::check_authority`; what
                // happens here is that the effect is recorded, which is what makes that
                // check possible at all.
                sc.effects.add(Effect::Authorize(cur), *span);
                Shape::Opaque
            }

            Expr::Declassify { args, span } => {
                args.iter().for_each(|a| {
                    self.expr(&a.value, sc);
                });
                sc.effects.add(Effect::Declassify, *span);
                Shape::Opaque
            }

            // ---- reads ----
            Expr::Stage {
                recv,
                kind,
                args,
                span,
                ..
            } => {
                self.expr(recv, sc);
                if let Expr::Path(p) = &**recv {
                    if let Some(rung) = self.cat.source_rung(&p.last().text) {
                        sc.effects.add(Effect::Read(rung), p.span);
                    }
                }
                for a in args {
                    self.expr(&a.value, sc);
                }
                // W17: a confidential column may not appear in a predicate, a key or an
                // aggregate — the engine cannot compute on ciphertext, so a filter over an
                // encrypted column would either leak through timing or silently not filter.
                if matches!(
                    kind,
                    StageKind::Where | StageKind::GroupBy | StageKind::Sum | StageKind::Having
                ) {
                    self.check_confidential_use(recv, args, *span);
                }
                Shape::Opaque
            }

            Expr::Path(p) => {
                let name = &p.last().text;
                if let Some(rung) = self.cat.source_rung(name) {
                    sc.effects.add(Effect::Read(rung), p.span);
                }
                sc.bindings.get(name).cloned().unwrap_or(Shape::Opaque)
            }

            Expr::Fixpoint {
                recv,
                step,
                measure,
                ..
            } => {
                self.expr(recv, sc);
                self.expr(step, sc);
                self.expr(measure, sc);
                Shape::Opaque
            }

            // ---- arithmetic: where currency mismatch is caught ----
            Expr::Binary { op, lhs, rhs, span } => {
                let (a, b) = (self.expr(lhs, sc), self.expr(rhs, sc));
                self.binary(*op, a, b, *span, lhs.span(), rhs.span())
            }

            Expr::Unary { op, operand, .. } => {
                let s = self.expr(operand, sc);
                match (op, s) {
                    (UnOp::Neg, Shape::Money(c, a)) => Shape::Money(c, a.neg()),
                    (_, s) => s,
                }
            }

            // ---- everything else: recurse ----
            Expr::Block(b) => {
                self.block(b, sc);
                Shape::Opaque
            }
            Expr::If {
                cond,
                then,
                els,
                span,
            } => {
                self.expr(cond, sc);
                let mut arms = Vec::new();
                let mut a = sc.branch();
                self.block(then, &mut a);
                arms.push(a);
                match els {
                    Some(e) => {
                        let mut b = sc.branch();
                        self.expr(e, &mut b);
                        arms.push(b);
                    }
                    // A missing `else` is an arm that does nothing: the empty row. Omitting
                    // it would let a one-armed `if` look unconditional.
                    None => arms.push(sc.branch()),
                }
                let cat = self.cat;
                let scale = move |c: &str| cat.currencies.get(c).map(|x| x.scale);
                for d in merge_branches(sc, arms, *span, &mut self.report.may_violate, &scale) {
                    self.push(d);
                }
                Shape::Opaque
            }
            Expr::Case { arms, els, span } => {
                let mut branches = Vec::new();
                for (c, v) in arms {
                    // The guard is evaluated on the way in, so its effects are
                    // unconditional; only the arm's value is alternative.
                    self.expr(c, sc);
                    let mut b = sc.branch();
                    self.expr(v, &mut b);
                    branches.push(b);
                }
                match els {
                    Some(e) => {
                        let mut b = sc.branch();
                        self.expr(e, &mut b);
                        branches.push(b);
                    }
                    // `case` without `else` can fall through producing nothing, which is
                    // an arm like any other.
                    None => branches.push(sc.branch()),
                }
                let cat = self.cat;
                let scale = move |c: &str| cat.currencies.get(c).map(|x| x.scale);
                for d in merge_branches(sc, branches, *span, &mut self.report.may_violate, &scale) {
                    self.push(d);
                }
                Shape::Opaque
            }
            Expr::Match {
                scrutinee,
                arms,
                span,
            } => {
                self.expr(scrutinee, sc);
                let mut branches = Vec::new();
                for a in arms {
                    let mut b = sc.branch();
                    if let Some(g) = &a.guard {
                        self.expr(g, &mut b);
                    }
                    self.expr(&a.body, &mut b);
                    branches.push(b);
                }
                let cat = self.cat;
                let scale = move |c: &str| cat.currencies.get(c).map(|x| x.scale);
                for d in merge_branches(sc, branches, *span, &mut self.report.may_violate, &scale) {
                    self.push(d);
                }
                Shape::Opaque
            }
            // --- loops: the loop rule ---
            //
            // A row is a homomorphism from statement sequences into a free abelian group,
            // so "balanced per iteration implies balanced overall" is immediate from the
            // algebra: no widening, no trip-count reasoning, no fixpoint. A body that nets
            // non-zero contributes `n · body` for an unknown `n`, which is a product of two
            // symbolic values and therefore outside this domain — so it becomes undecided.
            //
            // That asymmetry is not a limitation to apologise for. It is exactly the
            // discipline a batch-posting loop should follow: balance each iteration.
            Expr::While { cond, body, span } => {
                self.expr(cond, sc);
                let mut b = sc.loop_body();
                self.block(body, &mut b);
                self.close_loop(sc, b, *span);
                Shape::Opaque
            }
            Expr::Loop { body, span } => {
                let mut b = sc.loop_body();
                self.block(body, &mut b);
                self.close_loop(sc, b, *span);
                Shape::Opaque
            }
            Expr::For {
                iter, body, span, ..
            } => {
                self.expr(iter, sc);
                let mut b = sc.loop_body();
                self.block(body, &mut b);
                self.close_loop(sc, b, *span);
                Shape::Opaque
            }
            Expr::Closure { body, .. } => {
                self.expr(body, sc);
                Shape::Opaque
            }
            // `e?` is an early exit from the transaction. **It does not weaken the
            // analysis**, and the reason is a guarantee the runtime provides rather than
            // anything the checker can see.
            //
            // A `txn` is sealed atomically: a path that leaves it early commits *nothing*.
            // So an abort path is not a path with a different net — it is not a path that
            // reaches the ledger at all, and the correct transfer function maps it to
            // bottom rather than to the row accumulated so far. Only the paths that reach
            // the seal are paths whose conservation is a question.
            //
            // The first version of this rule marked every `?` as weakening the verdict from
            // a proof to an alarm, and the effect was immediate: `txn { let d = debit(a,
            // 100 usd)?; let c = credit(b, 60 usd); post(d, c) }` — an unambiguous
            // forty-dollar hole — was downgraded to a warning. Atomicity is what makes the
            // stronger reading sound, which is a case of a runtime guarantee buying static
            // precision rather than the other way round.
            Expr::Try { expr, .. } => self.expr(expr, sc),
            Expr::Cast { expr, .. } => self.expr(expr, sc),
            Expr::Field { base, .. } | Expr::Index { base, .. } => {
                self.expr(base, sc);
                Shape::Opaque
            }
            Expr::Tuple { elems, .. } | Expr::Array { elems, .. } => {
                elems.iter().for_each(|x| {
                    self.expr(x, sc);
                });
                Shape::Opaque
            }
            Expr::StructLit { fields, .. } => {
                fields.iter().for_each(|(_, v)| {
                    self.expr(v, sc);
                });
                Shape::Opaque
            }
            Expr::Assign { target, value, .. } => {
                self.expr(target, sc);
                self.expr(value, sc);
                Shape::Opaque
            }
            // Same reasoning as `?`: a `return` out of a transaction abandons it, and an
            // abandoned transaction commits nothing.
            Expr::Return { value: Some(v), .. } => self.expr(v, sc),
            Expr::Return { value: None, .. } => Shape::Opaque,
            // `break` and `continue` leave a *loop*, not the transaction: the transaction
            // still commits, so what the path did to the ledger still counts.
            //
            // Both used to return `Opaque` and nothing else, which meant the row as it
            // stood at the exit was folded in as though the whole body had run, and any
            // ledger half the body had created but not yet consumed was judged only at the
            // end of the body — a point this path never reaches. So
            // `loop { let d = debit(a, m)?; if p { break; } post(d, credit(b, m)); }`
            // passed: the half is consumed on the path the checker looked at, and dropped
            // on the one it did not.
            Expr::Break(span) | Expr::Continue(span) => {
                if let Some(base) = sc.loop_base {
                    // The exit's prefix row, as an alternative path through the loop.
                    sc.exits.push(sc.row.clone());
                    // And the linear discipline on this path: anything the body created
                    // and has not yet consumed is dropped here.
                    let unconsumed: Vec<LinearValue> = sc.linear[base.min(sc.linear.len())..]
                        .iter()
                        .filter(|v| v.uses.is_empty())
                        .cloned()
                        .collect();
                    for v in unconsumed {
                        self.push(
                            Diagnostic::error(
                                "NL0322",
                                format!("`{}` is not consumed on this exit path", v.name),
                            )
                            .primary(*span, "the loop is left here")
                            .secondary(v.bound_at, "created in this iteration")
                            .note("a ledger half must be consumed exactly once on *every* path, not on the path that falls off the end of the body")
                            .note("dropping it would lose the movement silently, which is the failure mode double-entry exists to prevent"),
                        );
                    }
                }
                Shape::Opaque
            }
            Expr::Explain { target, span } | Expr::Impact { target, span } => {
                self.expr(target, sc);
                sc.effects.add(Effect::Read(Rung::Snapshot), *span);
                Shape::Opaque
            }
            Expr::Reproduce { target, at, span } => {
                self.expr(target, sc);
                if let Some(a) = at {
                    self.expr(a, sc);
                }
                sc.effects.add(Effect::Read(Rung::LedgerConsistent), *span);
                Shape::Opaque
            }
            Expr::Sql { inner, .. } => self.expr(inner, sc),
            Expr::Select(s) => {
                self.select(s, sc);
                Shape::Opaque
            }
            _ => Shape::Opaque,
        }
    }

    fn select(&mut self, s: &SelectStmt, sc: &mut Scope) {
        self.check_confidential_in_select(s);
        for t in &s.from {
            self.table_ref(t, sc);
        }
        for (e, _) in &s.projections {
            self.expr(e, sc);
        }
        if let Some(f) = &s.filter {
            self.expr(f, sc);
        }
        for g in &s.group_by {
            self.expr(g, sc);
        }
        if let Some(h) = &s.having {
            self.expr(h, sc);
        }
        if let Some((_, next)) = &s.set_op {
            let n = next.clone();
            self.select(&n, sc);
        }
    }

    fn table_ref(&mut self, t: &TableRef, sc: &mut Scope) {
        match t {
            TableRef::Named { name, .. } => {
                if let Some(r) = self.cat.source_rung(&name.text) {
                    sc.effects.add(Effect::Read(r), name.span);
                }
            }
            TableRef::Join {
                left, right, on, ..
            } => {
                self.table_ref(left, sc);
                self.table_ref(right, sc);
                if let Some(o) = on {
                    self.expr(o, sc);
                }
            }
            TableRef::Sub { query, .. } => {
                let q = query.clone();
                self.select(&q, sc);
            }
        }
    }

    /// The library calls the checker knows the meaning of. Everything else is opaque, and
    /// contributes an undecided symbol rather than an assumption.
    fn call(
        &mut self,
        name: &str,
        shapes: &[Shape],
        args: &[Arg],
        span: Span,
        sc: &mut Scope,
    ) -> Shape {
        let money = shapes.iter().find_map(|s| match s {
            Shape::Money(c, a) => Some((c.clone(), a.clone())),
            _ => None,
        });
        match name {
            "debit" | "credit" => {
                let is_debit = name == "debit";
                let (c, a) = match money {
                    Some(x) => x,
                    None => {
                        let s = self.fresh_symbol();
                        (self.unifier.fresh(), Amount::symbol(s))
                    }
                };
                let eff = if is_debit {
                    Effect::Debit(c.to_string())
                } else {
                    Effect::Credit(c.to_string())
                };
                sc.effects.add(eff, span);
                if sc.conserving {
                    sc.row.movement(c, if is_debit { a.neg() } else { a }, span);
                }
                Shape::Linear(if is_debit {
                    LinearKind::Debit
                } else {
                    LinearKind::Credit
                })
            }
            "post" => {
                // `post` is what consumes the linear halves.
                for a in args {
                    self.consume_linear(&a.value, sc, span);
                }
                sc.effects.add(Effect::Append, span);
                Shape::Opaque
            }
            _ => {
                // **A call to a function this program declares.** Its effects become the
                // caller's, and its net movement becomes part of the caller's row.
                //
                // Neither happened before. `report.inferred_effects` was written and never
                // read, so a `ledger_consistent` view reading a `bounded` view through one
                // helper passed rung monotonicity; and a callee that posted one half of a
                // transfer contributed nothing at all to the caller's conservation
                // obligation — not a violation, not an undecided, *nothing*. The obligation
                // did not move to the runtime. It ceased to exist.
                if let Some(summary) = self.summaries.get(name).cloned() {
                    self.check_arguments(name, &summary, shapes, args, span);
                    self.apply_summary(&summary, shapes, span, sc);
                }
                // A call producing money contributes an *undecided* amount: the checker
                // cannot see the returned value even when it knows what the callee moved,
                // so it records a fresh symbol rather than assuming zero. This is the
                // mechanism that keeps `is_decided` honest.
                if let Some((c, _)) = money {
                    let s = self.fresh_symbol();
                    return Shape::Money(c, Amount::symbol(s));
                }
                Shape::Opaque
            }
        }
    }

    /// Check a call's arguments against the callee's parameters.
    ///
    /// Only the shapes this checker can name are judged; a parameter it cannot see inside
    /// is `ParamKind::Other` and nothing is claimed about it. Within that boundary the
    /// judgement is total, and it is new: arguments used to be evaluated for their effects
    /// and then discarded except for a scan for the first money value, so
    /// `transfer(a, b, 10.00 eur)` against `m: Money<usd>` passed, and an `Auth<E>`
    /// position could be filled by any expression at all.
    fn check_arguments(
        &mut self,
        callee: &str,
        s: &FnSummary,
        shapes: &[Shape],
        args: &[Arg],
        span: Span,
    ) {
        if shapes.len() != s.params.len() {
            // Arity. Reported once, and the shape check is skipped, because pairing
            // arguments with parameters positionally after a count mismatch produces a
            // cascade of confident nonsense.
            self.push(
                Diagnostic::error(
                    "NL0254",
                    format!(
                        "`{callee}` takes {} argument(s), but {} were given",
                        s.params.len(),
                        shapes.len()
                    ),
                )
                .primary(span, "wrong number of arguments"),
            );
            return;
        }
        for (i, (kind, shape)) in s.params.iter().zip(shapes).enumerate() {
            let at = args.get(i).map(|a| a.value.span()).unwrap_or(span);
            match (kind, shape) {
                // A currency mismatch through a call. This is the interprocedural half of
                // W7, and the reason `Money<c>` on a signature is worth writing.
                (ParamKind::Money(Some(want)), Shape::Money(Cur::Known(got), _)) if got != want => {
                    self.push(
                        Diagnostic::error(
                            "NL0255",
                            format!("`{callee}` takes `Money<{want}>` here, not `Money<{got}>`"),
                        )
                        .primary(at, format!("this is `Money<{got}>`"))
                        .note("currencies do not convert implicitly at a call boundary any more than at an operator: an implicit one would be a silent conversion at an unrecorded rate")
                        .note("to move value between currencies, use an `fx { leg .., leg .., rate: .. }` form, which conserves each currency separately and records the rate"),
                    );
                }
                (ParamKind::Money(_), Shape::Scalar(k)) => {
                    self.push(
                        Diagnostic::error(
                            "NL0254",
                            format!("`{callee}` takes money here, not `{}`", k.name()),
                        )
                        .primary(at, format!("`{}`", k.name()))
                        .note("a money value carries its currency and scale: write `10.00 usd`"),
                    );
                }
                // **The capability position.** An `Auth<E>` parameter accepts an `Auth`
                // and nothing else — not a literal, not the result of a call the checker
                // cannot see into. Accepting an opaque value here would put the forging
                // back one level: `force_debit(a, m, granted())` would hold authority
                // because a function called `granted` exists.
                (ParamKind::Auth(want), got) => {
                    let ok = matches!(got, Shape::Auth(have) if have == want || have == "_");
                    if !ok {
                        self.push(
                            Diagnostic::error(
                                "NL0331",
                                format!("`{callee}` requires `Auth<{want}>` here"),
                            )
                            .primary(at, format!("this is {}", got.describe()))
                            .note("a capability is unforgeable: it is received as a parameter or granted, never produced by an expression")
                            .note("passing it on is the only way to delegate it, and that is what makes an authorised path traceable to the parameter that authorised it"),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    /// Join a callee's summary into the caller's scope at a call site.
    ///
    /// Two things travel, and each has its own rule.
    ///
    /// * **Effects** union in, attributed to the call rather than to a line inside the
    ///   callee: the caller's declaration is what does not admit them, and the call is
    ///   where the caller acquired them.
    /// * **The net row** is instantiated. The callee's parameter symbols are replaced by
    ///   the caller's argument amounts, its currency variables by the arguments'
    ///   currencies, and every remaining symbol — a callee local, an amount neither side
    ///   can see — by a *fresh caller symbol*, which keeps the entry undecided.
    ///
    /// A `havoc` summary contributes an undecided movement in each currency it touches,
    /// never a decided one. That is the difference between "the checker handed this to the
    /// runtime" and "the checker forgot about it", and it is the whole reason `havoc` is
    /// recorded rather than inferred from a fresh symbol appearing.
    fn apply_summary(&mut self, s: &FnSummary, args: &[Shape], span: Span, sc: &mut Scope) {
        // Callee symbol -> caller amount, for the parameters that are money.
        let mut amounts: std::collections::BTreeMap<u32, Amount> = Default::default();
        let mut currencies: std::collections::BTreeMap<u32, Cur> = Default::default();
        for (i, pm) in s.param_money.iter().enumerate() {
            let Some((callee_cur, callee_sym)) = pm else {
                continue;
            };
            match args.get(i) {
                Some(Shape::Money(c, a)) => {
                    amounts.insert(*callee_sym, a.clone());
                    if let Cur::Var(v) = callee_cur {
                        currencies.insert(*v, c.clone());
                    }
                }
                // An argument the caller cannot see either. A fresh caller symbol, so the
                // entry is undecided here rather than carrying the callee's symbol, which
                // would make two unrelated calls look like the same amount.
                _ => {
                    let fresh = self.fresh_symbol();
                    amounts.insert(*callee_sym, Amount::symbol(fresh));
                }
            }
        }
        // **Effects are instantiated too.** A currency-generic helper's inferred row says
        // `debit<?c0>`; at a call site that binds `?c0` to `eur`, the caller's row must say
        // `debit<eur>`, or the caller has to declare a variable it never wrote. An unbound
        // variable stays unknown — spelled `*` — which a caller can only cover by declaring
        // the wildcard, and that is the honest requirement: it really does not know which
        // currency moves.
        let rename: HashMap<String, String> = currencies
            .iter()
            .map(|(v, c)| (format!("?c{v}"), c.to_string()))
            .collect();
        let mut effects = EffRow::new();
        for e in s.effects.iter() {
            let at = s.effects.origin(e).unwrap_or(span);
            effects.add(instantiate_effect(e, &rename), at);
        }
        sc.effects.union_at(&effects, span);
        if !sc.conserving || s.net.is_empty() {
            return;
        }

        let mut instantiated = CurRow::default();
        for c in s.net.currencies().cloned().collect::<Vec<_>>() {
            let amount = s.net.get(&c);
            // The **call site**, not the line inside the callee. The transaction whose
            // conservation is in question is the caller's, so that is the line a reader
            // needs to look at; the callee's own body is reachable from its name.
            let at = span;
            let cur = match &c {
                Cur::Var(v) => match currencies.get(v) {
                    Some(known) => known.clone(),
                    None => self.unifier.fresh(),
                },
                known => known.clone(),
            };
            let mut a = Amount::zero();
            a.constant = amount.constant;
            for (sym, coeff) in &amount.symbols {
                match amounts.get(sym) {
                    Some(sub) => a = a.add(&sub.scale(*coeff)),
                    None => {
                        let fresh = self.fresh_symbol();
                        a = a.add(&Amount::symbol(fresh).scale(*coeff));
                    }
                }
            }
            if s.havoc {
                // Poisoned on purpose: a summary that could not be computed must not be
                // able to produce a decided entry, however the arithmetic came out.
                let fresh = self.fresh_symbol();
                a = a.add(&Amount::symbol(fresh));
            }
            instantiated.movement(cur, a, at);
        }
        sc.row.merge(&instantiated);
    }

    fn binary(&mut self, op: BinOp, a: Shape, b: Shape, span: Span, lsp: Span, rsp: Span) -> Shape {
        let arithmetic = matches!(op, BinOp::Add | BinOp::Sub);
        let scaling = matches!(op, BinOp::Mul | BinOp::Div | BinOp::Rem);
        let comparison = matches!(
            op,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
        );

        // Money and a named scalar. `Money * Int` is scaling and is how a rate is applied;
        // `Money + Int` is a category error that used to classify as `Opaque` and vanish.
        match (&a, &b) {
            (Shape::Money(c, amt), Shape::Scalar(k)) | (Shape::Scalar(k), Shape::Money(c, amt))
                if arithmetic || comparison =>
            {
                self.push(
                    Diagnostic::error(
                        "NL0252",
                        format!("cannot {} `Money<{c}>` and `{}`", verb(op), k.name()),
                    )
                    .primary(span, "these are not the same kind of thing")
                    .secondary(lsp, a.describe())
                    .secondary(rsp, b.describe())
                    .note("money is closed under addition and subtraction *within a currency*, and under integer scaling; there is no operation joining it to a plain number")
                    .note("a number that is an amount of money should be written as one: `10.00 usd`, not `1000`"),
                );
                let _ = amt;
                return Shape::Opaque;
            }
            (Shape::Money(c, amt), Shape::Scalar(ScalarKind::Int)) if scaling => {
                // Scaling by an unknown integer: the currency survives, the amount does
                // not — `n · x` is a product of two symbolic values and outside the
                // domain, so a fresh symbol, which is what keeps `is_decided` honest.
                let (c, _) = (c.clone(), amt);
                let s = self.fresh_symbol();
                return Shape::Money(c, Amount::symbol(s));
            }
            (Shape::Scalar(x), Shape::Scalar(y)) => {
                let ok = if arithmetic {
                    x.adds_to(*y)
                } else if comparison {
                    x.compares_to(*y)
                } else if scaling {
                    matches!(
                        (x, y),
                        (ScalarKind::Int, ScalarKind::Int) | (ScalarKind::Float, ScalarKind::Float)
                    )
                } else {
                    // `and`/`or`/`like`: booleans and text respectively.
                    match op {
                        BinOp::And | BinOp::Or => *x == ScalarKind::Bool && *y == ScalarKind::Bool,
                        BinOp::Like => *x == ScalarKind::Text && *y == ScalarKind::Text,
                        _ => true,
                    }
                };
                if !ok {
                    self.push(
                        Diagnostic::error(
                            "NL0252",
                            format!("cannot {} `{}` and `{}`", verb(op), x.name(), y.name()),
                        )
                        .primary(span, "no such operation")
                        .secondary(lsp, a.describe())
                        .secondary(rsp, b.describe())
                        .note("there are no implicit conversions between kinds: one would be a silent coercion, and a silent coercion in a ledger language is a rounding nobody wrote"),
                    );
                    return Shape::Opaque;
                }
                return if comparison || matches!(op, BinOp::And | BinOp::Or | BinOp::Like) {
                    Shape::Scalar(ScalarKind::Bool)
                } else {
                    Shape::Scalar(*x)
                };
            }
            _ => {}
        }

        let (Shape::Money(ca, aa), Shape::Money(cb, ab)) = (&a, &b) else {
            return Shape::Opaque;
        };
        match op {
            BinOp::Add | BinOp::Sub => {
                // W7: no operator joins two distinct currencies. This is the "cannot
                // mismatch currencies" clause, decided by unification.
                match self.unifier.unify(ca, cb) {
                    Ok(c) => {
                        let amt = if op == BinOp::Add {
                            aa.add(ab)
                        } else {
                            aa.add(&ab.neg())
                        };
                        Shape::Money(c, amt)
                    }
                    Err((x, y)) => {
                        self.push(
                            Diagnostic::error(
                                "NL0250",
                                format!("cannot {} `Money<{x}>` and `Money<{y}>`", if op == BinOp::Add { "add" } else { "subtract" }),
                            )
                            .primary(span, format!("`{x}` and `{y}` are different currencies"))
                            .secondary(lsp, format!("this is `Money<{x}>`"))
                            .secondary(rsp, format!("this is `Money<{y}>`"))
                            .note("money is closed under addition, subtraction and integer scaling *within* a currency; nothing joins two")
                            .note("to move value between currencies, use an `fx { leg .., leg .., rate: .. }` form, which conserves each currency separately and records the rate")
                            .suggest(span, "fx { leg a: .., leg b: .., rate: r }", "use an `fx` form: two conserved legs and a recorded rate", crate::diagnostics::Applicability::HasPlaceholders),
                        );
                        Shape::Opaque
                    }
                }
            }
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if let Err((x, y)) = self.unifier.unify(ca, cb) {
                    self.push(
                        Diagnostic::error("NL0251", format!("cannot compare `Money<{x}>` with `Money<{y}>`"))
                            .primary(span, "different currencies are not ordered relative to each other")
                            .secondary(lsp, format!("`Money<{x}>`"))
                            .secondary(rsp, format!("`Money<{y}>`"))
                            .note("there is no exchange-rate-free ordering between currencies, and an implicit one would be a silent conversion"),
                    );
                }
                Shape::Opaque
            }
            _ => Shape::Opaque,
        }
    }

    /// Close a loop body back into its enclosing scope, applying the loop rule.
    fn close_loop(&mut self, sc: &mut Scope, mut body: Scope, at: Span) {
        let base_len = sc.linear.len();
        let body_local: Vec<LinearValue> = body.linear.split_off(base_len.min(body.linear.len()));
        // A linear value created inside a loop body must be consumed inside it; a hold
        // that escapes an iteration would be resolved a number of times the checker cannot
        // count, which is the same problem in the linear discipline as `n · m` is in the
        // arithmetic one.
        for d in effects::check_linearity(&body_local) {
            self.push(d);
        }
        sc.effects.union(&body.effects);
        // A linear value consumed *inside* a loop is consumed an unknown number of times.
        for (i, v) in body.linear.iter().enumerate() {
            if i < sc.linear.len() && v.uses.len() > sc.linear[i].uses.len() {
                let extra: Vec<Span> = v.uses[sc.linear[i].uses.len()..].to_vec();
                // Record it twice: once is the use, and the duplicate is what makes an
                // unbounded number of uses show up as the error it is.
                sc.linear[i].uses.extend(extra.iter().copied());
                sc.linear[i].uses.extend(extra);
            }
        }
        // **The exit paths.** A `break` leaves the loop with a prefix of the body's
        // movements, so the loop's contribution is the join of the full body with each
        // prefix — alternatives, not a sequence. Where they disagree the join goes to top
        // and the verdict becomes undecided, which is the honest statement: which path ran
        // is not decidable here.
        let mut row = std::mem::take(&mut body.row);
        for exit in std::mem::take(&mut body.exits) {
            row = row.join(exit, at);
        }
        let iterated = row.iterate(at);
        sc.row.merge(&iterated);
    }

    fn consume_linear(&mut self, e: &Expr, sc: &mut Scope, at: Span) {
        if let Expr::Path(p) = e {
            let n = &p.last().text;
            if let Some(v) = sc.linear.iter_mut().find(|v| &v.name == n) {
                v.uses.push(at);
            }
        }
    }

    /// The declared minor-unit scale of a currency, for rendering a residue.
    ///
    /// `None` when the currency is not declared — and the caller then says nothing rather
    /// than printing the residue at an assumed scale. `unwrap_or(2)` rendered a JPY
    /// residue of 1000 as "10.00", and the currency being undeclared is already reported
    /// (NL0241): a second diagnostic carrying a wrong number is worse than none, because a
    /// reader reconciling against it would be reconciling against a figure the compiler
    /// invented.
    fn scale_of(&self, currency: &str) -> Option<u32> {
        self.cat.currencies.get(currency).map(|c| c.scale)
    }

    /// Close an open conservation obligation and report the verdict.
    fn discharge_conservation(&mut self, row: &mut CurRow, span: Span, leg: Option<&str>) {
        row.substitute(&self.unifier);
        if row.is_empty() {
            return;
        }
        let prov = row.provenance;
        for v in rows::check_conservation(row) {
            match &v {
                Verdict::Conserves => self.report.conservation_proved += 1,
                Verdict::Undecided { .. } => {
                    self.report.runtime_obligations += 1;
                    self.report.undecided += 1;
                }
                Verdict::MayViolate { currency, .. } => {
                    self.report.runtime_obligations += 1;
                    self.report.may_violate += 1;
                    let Some(scale) = self.scale_of(currency) else {
                        continue;
                    };
                    if let Some(d) = rows::diagnose(&v, scale) {
                        self.push(d);
                    }
                }
                Verdict::Violates { currency, .. } => {
                    self.report.violates += 1;
                    let Some(scale) = self.scale_of(currency) else {
                        continue;
                    };
                    if let Some(mut d) = rows::diagnose_with(&v, scale, prov) {
                        if let Some(name) = leg {
                            d = d.secondary(span, format!("in leg `{name}` of this `fx` form"));
                            d = d.note("each leg of an `fx` form conserves independently; a rate relates them, it does not balance them");
                        }
                        // Point at the ledger's own rule — the other half of the two-span
                        // form these diagnostics exist for.
                        // The warrant, attached conditionally: it yields to a
                        // machine-applicable fix, because a resolution is what the reader
                        // wants and a rule declaration three files away is not.
                        if let Some(rel) = self
                            .cat
                            .relations
                            .values()
                            .find(|r| r.conserve_span.is_some())
                        {
                            if let Some(cs) = rel.conserve_span {
                                d = d.warrant(
                                    cs,
                                    format!("`conserve per (..)` declared on `{}` here", rel.name),
                                );
                            }
                        }
                        self.push(d);
                    }
                }
            }
        }
    }

    fn check_confidential_use(&mut self, recv: &Expr, args: &[Arg], span: Span) {
        let Some(rel) = self.root_relation(recv) else {
            return;
        };
        let confidential = confidential_columns(rel);
        let exprs: Vec<&Expr> = args.iter().map(|a| &a.value).collect();
        self.refuse_confidential(
            &confidential,
            &exprs,
            "used in a predicate, key or aggregate",
            span,
        );
    }

    /// **The same rule for the SQL surface**, which had none.
    ///
    /// W17 ran only for pipeline stages, so with `legal_name: Text @confidential(e2ee)` the
    /// statement `select legal_name, count(id) from parties group by legal_name` passed
    /// `nilesc check` with `ok` and lowered to `aggregate(by=[1], count)` — the engine
    /// grouping by a column it is declared unable to read. Every wire client uses this
    /// surface, so the one static confidentiality guarantee the design makes was false
    /// exactly where it is relied upon.
    ///
    /// A *bare* projection of a confidential column stays legal: carrying a sealed value
    /// through to the caller is the one thing the engine can do with it. Everything else —
    /// a predicate, a grouping key, a join key, an aggregate's argument, an expression over
    /// it — is refused.
    fn check_confidential_in_select(&mut self, s: &SelectStmt) {
        let mut confidential: Vec<(String, String, Span)> = Vec::new();
        collect_relation_names(&s.from, &mut |n| {
            if let Some(rel) = self.cat.relations.get(n) {
                confidential.extend(confidential_columns(rel));
            }
        });
        if confidential.is_empty() {
            return;
        }
        if let Some(f) = &s.filter {
            self.refuse_confidential(&confidential, &[f], "used in a `where` predicate", f.span());
        }
        if let Some(h) = &s.having {
            self.refuse_confidential(
                &confidential,
                &[h],
                "used in a `having` predicate",
                h.span(),
            );
        }
        for g in &s.group_by {
            self.refuse_confidential(&confidential, &[g], "used as a grouping key", g.span());
        }
        for (e, _) in &s.projections {
            // The identity projection is the permitted use.
            if matches!(e, Expr::Path(_) | Expr::Field { .. }) {
                continue;
            }
            self.refuse_confidential(
                &confidential,
                &[e],
                "computed over in the projection",
                e.span(),
            );
        }
        collect_join_conditions(&s.from, &mut |o: &Expr| {
            let cols = confidential.clone();
            self.refuse_confidential(&cols, &[o], "used in a join condition", o.span());
        });
    }

    fn refuse_confidential(
        &mut self,
        confidential: &[(String, String, Span)],
        exprs: &[&Expr],
        what: &str,
        span: Span,
    ) {
        if confidential.is_empty() {
            return;
        }
        for e in exprs {
            let mut used = Vec::new();
            collect_fields(e, &mut used);
            for (name, span_of_use) in used {
                if let Some((_, level, decl)) = confidential.iter().find(|(c, _, _)| *c == name) {
                    self.push(
                        Diagnostic::error("NL0260", format!("`{name}` is `@confidential({level})` and cannot be used here"))
                            .primary(span_of_use, "the engine cannot compute on this column")
                            .secondary(*decl, format!("declared `@confidential({level})` here"))
                            .secondary(span, what.to_string())
                            .note("an encrypted column is opaque to the engine: a filter over it would either not filter, or leak through timing")
                            .note("to use it, `declassify(..)` it with a capability — which is audited, and is the point"),
                    );
                }
            }
        }
    }

    fn root_relation(&self, e: &Expr) -> Option<&crate::resolve::RelationInfo> {
        match e {
            Expr::Path(p) => self.cat.relations.get(&p.last().text),
            Expr::Stage { recv, .. } | Expr::Fixpoint { recv, .. } => self.root_relation(recv),
            _ => None,
        }
    }
}

fn collect_fields(e: &Expr, out: &mut Vec<(String, Span)>) {
    match e {
        Expr::Field { name, span, .. } => out.push((name.text.clone(), *span)),
        // **A bare name is a column too.** This arm was missing, and it is the mechanical
        // reason the SQL surface had no confidentiality check even where one was called: the
        // pipeline writes `|p| p.legal_name` (a `Field`) and SQL writes `legal_name` (a
        // `Path`), so every SQL use of a confidential column was invisible to the walk. The
        // callee of a `Call` is not traversed, so a function's *name* is never mistaken for a
        // column.
        Expr::Path(p) => out.push((p.last().text.clone(), p.span)),
        Expr::Closure { body, .. } => collect_fields(body, out),
        Expr::Binary { lhs, rhs, .. } => {
            collect_fields(lhs, out);
            collect_fields(rhs, out);
        }
        Expr::Unary { operand, .. } => collect_fields(operand, out),
        Expr::Tuple { elems, .. } | Expr::Array { elems, .. } => {
            elems.iter().for_each(|x| collect_fields(x, out))
        }
        Expr::Call { args, .. } | Expr::Stage { args, .. } => {
            args.iter().for_each(|a| collect_fields(&a.value, out))
        }
        _ => {}
    }
}

/// `Auth<authorize<usd>>` -> `Some("authorize<usd>")`; anything else -> `None`.
///
/// The effect is rendered rather than parsed, because the string is for a human reading a
/// diagnostic. Whether the effect *exists* is decided in [`Cx::function`], where the same
/// type is turned into an [`Effect`] by [`Effect::parse`].
fn auth_effect_of(t: &Ty) -> Option<String> {
    match t {
        Ty::Path { path, args, .. } if path.last().text == "Auth" => match args.first() {
            Some(Ty::Path { path: e, args, .. }) => {
                let inner: Vec<String> = args
                    .iter()
                    .filter_map(|a| match a {
                        Ty::Path { path, .. } => Some(path.last().text.clone()),
                        _ => None,
                    })
                    .collect();
                Some(if inner.is_empty() {
                    e.last().text.clone()
                } else {
                    format!("{}<{}>", e.last().text, inner.join(", "))
                })
            }
            _ => Some("_".into()),
        },
        Ty::Ref { inner, .. } => auth_effect_of(inner),
        _ => None,
    }
}

/// `Money<usd>` -> `Some(Some("usd"))`; `Money` -> `Some(None)`; anything else -> `None`.
/// NL0332: a `Money<c>` position filled by a value the checker can see is `Money<d>`.
///
/// One diagnostic for both positions that take a written currency and a value in one step —
/// a `let` annotation and a function's return type. The call boundary has its own (NL0255)
/// because it can name the callee and the parameter.
fn currency_annotation_mismatch(
    want: &str,
    got: &str,
    at: Span,
    written: Span,
    what: &str,
) -> Diagnostic {
    Diagnostic::error(
        "NL0332",
        format!("this is `Money<{got}>`, but the type written here is `Money<{want}>`"),
    )
    .primary(at, format!("`Money<{got}>`"))
    .secondary(written, format!("{what} `Money<{want}>`"))
    .note("an annotation tells the checker what it could not see; it does not overrule what it can, and a currency is something it can see")
    .note("taking the annotation's word here would let every later use of this value claim a currency it does not have, which is how a `debit<usd>` effect row comes to describe a EUR leg")
    .note("to move value between currencies, use an `fx { leg .., leg .., rate: .. }` form, which conserves each currency separately and records the rate")
}

fn money_currency_of(t: &Ty) -> Option<Option<String>> {
    match t {
        Ty::Path { path, args, .. } if path.last().text == "Money" => Some(match args.first() {
            Some(Ty::Path { path, .. }) => Some(path.last().text.clone()),
            _ => None,
        }),
        Ty::Ref { inner, .. } => money_currency_of(inner),
        _ => None,
    }
}

fn first_binding(p: &Pat) -> Option<String> {
    p.bindings().first().map(|n| n.text.clone())
}

fn to_eff_row(row: &EffectRow) -> EffRow {
    let mut out = EffRow::new();
    for e in &row.effects {
        let args: Vec<String> = e.args.iter().map(|a| a.text.clone()).collect();
        if let Some(eff) =
            Effect::parse(&e.name.text, e.at.as_ref().map(|a| a.text.as_str()), &args)
        {
            out.add(eff, e.span);
        }
    }
    out
}

/// The verb a binary operator is described by in a diagnostic.
fn verb(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "add",
        BinOp::Sub => "subtract",
        BinOp::Mul => "multiply",
        BinOp::Div => "divide",
        BinOp::Rem => "take the remainder of",
        BinOp::And => "conjoin",
        BinOp::Or => "disjoin",
        BinOp::Like => "match",
        _ => "compare",
    }
}

/// Every name appearing in call position in a block, for the call-graph edge set.
fn calls_in_block(b: &Block, out: &mut Vec<String>) {
    for s in &b.stmts {
        match s {
            Stmt::Let { init: Some(e), .. } | Stmt::Expr(e) | Stmt::Semi(e) => {
                calls_in_expr(e, out)
            }
            _ => {}
        }
    }
    if let Some(t) = &b.tail {
        calls_in_expr(t, out);
    }
}

fn calls_in_expr(e: &Expr, out: &mut Vec<String>) {
    if let Expr::Call { callee, args, .. } = e {
        if let Expr::Path(p) = &**callee {
            out.push(p.last().text.clone());
        }
        args.iter().for_each(|a| calls_in_expr(&a.value, out));
        return;
    }
    let (mut kids, mut blocks) = (Vec::new(), Vec::new());
    each_child(e, &mut kids, &mut blocks);
    kids.into_iter().for_each(|c| calls_in_expr(c, out));
    blocks.into_iter().for_each(|b| calls_in_block(b, out));
}

/// Walk the immediate sub-expressions and sub-blocks of an expression.
///
/// Written once so that the call-graph walk cannot fall behind the AST: a new `Expr`
/// variant that carries a body and is not added here would make a call inside it invisible
/// to the recursion check, and an unnoticed cycle is exactly what `havoc` exists to catch.
fn each_child<'e>(e: &'e Expr, exprs: &mut Vec<&'e Expr>, blocks: &mut Vec<&'e Block>) {
    match e {
        Expr::Call { callee, args, .. } => {
            exprs.push(callee);
            args.iter().for_each(|a| exprs.push(&a.value));
        }
        Expr::Hold { args, .. } | Expr::Authorize { args, .. } | Expr::Declassify { args, .. } => {
            args.iter().for_each(|a| exprs.push(&a.value))
        }
        Expr::Stage { recv, args, .. } => {
            exprs.push(recv);
            args.iter().for_each(|a| exprs.push(&a.value));
        }
        Expr::Txn { body, .. } => blocks.push(body),
        Expr::Fx { legs, rate, .. } => {
            legs.iter().for_each(|(_, l)| exprs.push(l));
            if let Some(r) = rate {
                exprs.push(r);
            }
        }
        Expr::Resolve { hold, outcome, .. } => {
            exprs.push(hold);
            if let ResolveOutcome::Post(a) = outcome {
                exprs.push(a);
            }
        }
        Expr::Fixpoint {
            recv,
            step,
            measure,
            ..
        } => {
            exprs.push(recv);
            exprs.push(step);
            exprs.push(measure);
        }
        Expr::Binary { lhs, rhs, .. } => {
            exprs.push(lhs);
            exprs.push(rhs);
        }
        Expr::Unary { operand, .. } => exprs.push(operand),
        Expr::Block(b) => blocks.push(b),
        Expr::If {
            cond, then, els, ..
        } => {
            exprs.push(cond);
            blocks.push(then);
            if let Some(e) = els {
                exprs.push(e);
            }
        }
        Expr::Case { arms, els, .. } => {
            arms.iter().for_each(|(c, v)| {
                exprs.push(c);
                exprs.push(v);
            });
            if let Some(e) = els {
                exprs.push(e);
            }
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            exprs.push(scrutinee);
            for a in arms {
                if let Some(gd) = &a.guard {
                    exprs.push(gd);
                }
                exprs.push(&a.body);
            }
        }
        Expr::While { cond, body, .. } => {
            exprs.push(cond);
            blocks.push(body);
        }
        Expr::Loop { body, .. } => blocks.push(body),
        Expr::For { iter, body, .. } => {
            exprs.push(iter);
            blocks.push(body);
        }
        Expr::Closure { body, .. } => exprs.push(body),
        Expr::Try { expr, .. } | Expr::Cast { expr, .. } => exprs.push(expr),
        Expr::Field { base, .. } | Expr::Index { base, .. } => exprs.push(base),
        Expr::Tuple { elems, .. } | Expr::Array { elems, .. } => {
            elems.iter().for_each(|x| exprs.push(x))
        }
        Expr::StructLit { fields, .. } => fields.iter().for_each(|(_, v)| exprs.push(v)),
        Expr::Assign { target, value, .. } => {
            exprs.push(target);
            exprs.push(value);
        }
        Expr::Return { value: Some(v), .. } => exprs.push(v),
        Expr::Explain { target, .. } | Expr::Impact { target, .. } => exprs.push(target),
        Expr::Reproduce { target, at, .. } => {
            exprs.push(target);
            if let Some(a) = at {
                exprs.push(a);
            }
        }
        Expr::Sql { inner, .. } => exprs.push(inner),
        _ => {}
    }
}

/// Rewrite a currency-parameterised effect's currency through a call site's bindings.
///
/// A currency that is still a variable becomes `*` — unknown — rather than keeping the
/// callee's internal variable name, which no caller could ever declare and which would leak
/// one function's numbering into another's signature.
fn instantiate_effect(e: &Effect, rename: &HashMap<String, String>) -> Effect {
    let map = |c: &String| -> String {
        match rename.get(c) {
            Some(known) => known.clone(),
            None if c.starts_with("?c") => "*".to_string(),
            None => c.clone(),
        }
    };
    match e {
        Effect::Debit(c) => Effect::Debit(map(c)),
        Effect::Credit(c) => Effect::Credit(map(c)),
        Effect::Hold(c) => Effect::Hold(map(c)),
        Effect::Authorize(c) => Effect::Authorize(map(c)),
        other => other.clone(),
    }
}

/// Every declared relation or view named in an expression.
///
/// Read from the syntax rather than from the effect row, because the row records *rungs* and
/// two different sources at the same rung are indistinguishable in it.
fn collect_sources(e: &Expr, cat: &Catalog, out: &mut std::collections::BTreeSet<String>) {
    if let Expr::Path(p) = e {
        let n = &p.last().text;
        if cat.views.contains_key(n) || cat.relations.contains_key(n) {
            out.insert(n.clone());
        }
        return;
    }
    // The SQL surface names its sources in `from`, which is a `TableRef` and not an
    // expression, so `each_child` cannot reach them. Without this arm `sql_positions`
    // reported "reads nothing" while its effect row said `ledger_consistent` — the two
    // halves of one answer disagreeing.
    if let Expr::Select(sel) = e {
        collect_sources_in_select(sel, cat, out);
        return;
    }
    let (mut kids, mut blocks) = (Vec::new(), Vec::new());
    each_child(e, &mut kids, &mut blocks);
    for k in kids {
        collect_sources(k, cat, out);
    }
    for b in blocks {
        for st in &b.stmts {
            match st {
                Stmt::Let { init: Some(x), .. } | Stmt::Expr(x) | Stmt::Semi(x) => {
                    collect_sources(x, cat, out)
                }
                _ => {}
            }
        }
        if let Some(t) = &b.tail {
            collect_sources(t, cat, out);
        }
    }
}

fn collect_sources_in_select(
    s: &SelectStmt,
    cat: &Catalog,
    out: &mut std::collections::BTreeSet<String>,
) {
    fn table_ref(t: &TableRef, cat: &Catalog, out: &mut std::collections::BTreeSet<String>) {
        match t {
            TableRef::Named { name, .. } => {
                if cat.views.contains_key(&name.text) || cat.relations.contains_key(&name.text) {
                    out.insert(name.text.clone());
                }
            }
            TableRef::Join {
                left, right, on, ..
            } => {
                table_ref(left, cat, out);
                table_ref(right, cat, out);
                if let Some(o) = on {
                    collect_sources(o, cat, out);
                }
            }
            TableRef::Sub { query, .. } => collect_sources_in_select(query, cat, out),
        }
    }
    for t in &s.from {
        table_ref(t, cat, out);
    }
    for (e, _) in &s.projections {
        collect_sources(e, cat, out);
    }
    for e in s.filter.iter().chain(s.having.iter()) {
        collect_sources(e, cat, out);
    }
    for g in &s.group_by {
        collect_sources(g, cat, out);
    }
    if let Some((_, next)) = &s.set_op {
        collect_sources_in_select(next, cat, out);
    }
}

/// The `@confidential` columns of a relation, as `(column, level, declaration span)`.
fn confidential_columns(rel: &crate::resolve::RelationInfo) -> Vec<(String, String, Span)> {
    rel.columns
        .iter()
        .filter_map(|c| c.confidential.clone().map(|l| (c.name.clone(), l, c.span)))
        .collect()
}

/// Every relation named in a from-list, including both sides of every join.
fn collect_relation_names(from: &[TableRef], f: &mut impl FnMut(&str)) {
    for t in from {
        walk_table_ref(t, f);
    }
}

fn walk_table_ref(t: &TableRef, f: &mut impl FnMut(&str)) {
    match t {
        TableRef::Named { name, .. } => f(&name.text),
        TableRef::Join { left, right, .. } => {
            walk_table_ref(left, f);
            walk_table_ref(right, f);
        }
        TableRef::Sub { query, .. } => {
            for t in &query.from {
                walk_table_ref(t, f);
            }
        }
    }
}

/// Every `on` condition in a from-list.
fn collect_join_conditions(from: &[TableRef], f: &mut impl FnMut(&Expr)) {
    for t in from {
        walk_join_conditions(t, f);
    }
}

fn walk_join_conditions(t: &TableRef, f: &mut impl FnMut(&Expr)) {
    match t {
        TableRef::Named { .. } => {}
        TableRef::Join {
            left, right, on, ..
        } => {
            walk_join_conditions(left, f);
            walk_join_conditions(right, f);
            if let Some(o) = on {
                f(o);
            }
        }
        TableRef::Sub { query, .. } => {
            for t in &query.from {
                walk_join_conditions(t, f);
            }
        }
    }
}
