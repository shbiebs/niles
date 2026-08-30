//! **Stage 0 execution**: a tree-walking interpreter for Niles's imperative subset
//! (thesis Appendix E.1, E.3).
//!
//! # Why an interpreter, and why it is the missing piece
//!
//! Appendix E specifies a three-stage bootstrap: stage 0 is a compiler written in the
//! host language that accepts the whole of Niles; stage 1 is the Niles-written compiler
//! *compiled by* stage 0; stage 2 is stage 1 recompiling the same sources; stage 3 is the
//! fixpoint check. E.0 recorded, honestly, that stage 1 had no input, because the stage-0
//! compiler had no code generation for the imperative tier: it could lower a *query* to
//! the IR, and it could type-check a function, but it could not **run** one. A bootstrap
//! needs execution, not compilation. Nothing in the definition of stage 0 requires it to
//! emit machine code; it requires it to *evaluate* Niles programs. That is what this
//! crate provides, and it is what turns E.1 from a plan into a thing with an exit status.
//!
//! This is not a workaround. It is how bootstraps are normally started — a first-stage
//! interpreter, then a compiler written in the language and run under it, then the
//! compiler compiling itself. Treating "stage 0 must be a native compiler" as a
//! requirement would have been an error of the thesis's own making.
//!
//! # What it accepts, precisely
//!
//! The **imperative subset**: functions, `let` with patterns, `if`/`else`, `while`,
//! `loop`, `for`, `match`, `break`/`continue`/`return`, blocks with tail values, integer
//! and string and boolean arithmetic, comparison, arrays with index and mutation, structs
//! and their fields, tuple-variant enums, closures, and calls.
//!
//! It does **not** accept the relational tier — `view`, pipelines, `txn`, `hold`, `fx`,
//! `fixpoint` — and does not pretend to. Those forms are rejected by name with
//! [`Error::NotInSubset`], because a bootstrap that silently evaluated `txn` as a plain
//! block would be a bootstrap that had quietly abandoned the conservation guarantee the
//! rest of the thesis is about. The refusal is the honest behaviour and the tests pin it.
//!
//! # Determinism is a property this crate must have, not merely hope for
//!
//! Appendix C.4 imposes a cross-target determinism obligation: the same input must
//! produce byte-identical output on every target. An interpreter is where that obligation
//! is easiest to break — hash-map iteration order, address-derived identity, float
//! formatting, wall-clock reads. Three decisions here address it directly. Environments
//! are `BTreeMap`, so scope iteration is by name and not by hash seed. There is no
//! floating-point arithmetic in the subset at all. And there is no clock, no
//! randomness, and no I/O beyond an explicit output buffer the caller owns, so a program
//! run twice cannot differ. [`determinism_gate`] is the runnable form of that claim.

use niles_lang::ast::*;
use niles_lang::lexer::Span;
use std::collections::BTreeMap;
use std::rc::Rc;

/// A runtime value.
///
/// Deliberately small. Money, epochs and instants are *carried* but not operated on:
/// arithmetic on them belongs to the checked tier, and an interpreter that added two
/// `Money` values without consulting the currency row would be re-implementing the very
/// thing Contribution 4 makes static. They are here so a program can pass them through.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i128),
    Bool(bool),
    Str(Rc<String>),
    Bytes(Rc<Vec<u8>>),
    Array(Rc<Vec<Value>>),
    Tuple(Rc<Vec<Value>>),
    /// A struct or an enum's struct-variant: a name and ordered fields.
    Record { name: String, fields: Rc<BTreeMap<String, Value>> },
    /// `Some(x)`, `Tok::Ident(s)` — a path and positional payload.
    Variant { path: String, payload: Rc<Vec<Value>> },
    Closure(Rc<ClosureVal>),
    /// Carried, not computed on. See the type docs.
    Money { minor: i128, scale: u32, currency: String },
    Epoch(u64),
}

#[derive(Debug, PartialEq)]
pub struct ClosureVal {
    pub params: Vec<Pat>,
    pub body: Expr,
    /// Captured by value at creation. The subset has no interior mutability, so a
    /// snapshot is sound and avoids the reference cycles a shared environment would need.
    pub captured: BTreeMap<String, Value>,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Unit => "unit",
            Value::Int(_) => "int",
            Value::Bool(_) => "bool",
            Value::Str(_) => "str",
            Value::Bytes(_) => "bytes",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::Record { .. } => "struct",
            Value::Variant { .. } => "variant",
            Value::Closure(_) => "closure",
            Value::Money { .. } => "money",
            Value::Epoch(_) => "epoch",
        }
    }

    /// A total, deterministic rendering. Used by `print` and by the determinism gate, so
    /// it must never depend on iteration order or on an address.
    pub fn render(&self) -> String {
        match self {
            Value::Unit => "()".into(),
            Value::Int(i) => i.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => (**s).clone(),
            Value::Bytes(b) => format!("b\"{}\"", String::from_utf8_lossy(b)),
            Value::Array(xs) => {
                format!("[{}]", xs.iter().map(|v| v.render()).collect::<Vec<_>>().join(", "))
            }
            Value::Tuple(xs) => {
                format!("({})", xs.iter().map(|v| v.render()).collect::<Vec<_>>().join(", "))
            }
            // BTreeMap iteration is by key, so this is stable across runs and targets.
            Value::Record { name, fields } => format!(
                "{name} {{ {} }}",
                fields.iter().map(|(k, v)| format!("{k}: {}", v.render())).collect::<Vec<_>>().join(", ")
            ),
            Value::Variant { path, payload } if payload.is_empty() => path.clone(),
            Value::Variant { path, payload } => format!(
                "{path}({})",
                payload.iter().map(|v| v.render()).collect::<Vec<_>>().join(", ")
            ),
            Value::Closure(_) => "<closure>".into(),
            Value::Money { minor, scale, currency } => {
                let d = 10i128.pow(*scale);
                format!("{}.{:0width$} {currency}", minor / d, (minor % d).abs(), width = *scale as usize)
            }
            Value::Epoch(e) => format!("#{e}"),
        }
    }

    fn truthy(&self, at: Span) -> Result<bool, Error> {
        match self {
            Value::Bool(b) => Ok(*b),
            // No coercion. A language whose conditions accept integers is a language in
            // which `if balance` means something, and it should not.
            other => Err(Error::TypeMismatch {
                want: "bool".into(),
                got: other.type_name().into(),
                at,
            }),
        }
    }
}

/// What can go wrong. Every variant carries a span, because a stage-0 failure that cannot
/// be located is a stage-0 failure nobody can fix.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A construct outside the imperative subset. Named, never silently approximated.
    NotInSubset { form: &'static str, at: Span },
    Unbound { name: String, at: Span },
    TypeMismatch { want: String, got: String, at: Span },
    NoField { name: String, at: Span },
    IndexOutOfBounds { index: i128, len: usize, at: Span },
    DivideByZero { at: Span },
    /// Integer overflow. Checked rather than wrapping: a ledger interpreter that wrapped
    /// silently would be the exact defect Contribution 4 exists to exclude.
    Overflow { op: &'static str, at: Span },
    WrongArity { want: usize, got: usize, at: Span },
    NoMatchingArm { at: Span },
    /// The fuel ran out. See [`Interp::with_fuel`].
    OutOfFuel,
    NotCallable { got: String, at: Span },
}

impl Error {
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::NotInSubset { at, .. }
            | Error::Unbound { at, .. }
            | Error::TypeMismatch { at, .. }
            | Error::NoField { at, .. }
            | Error::IndexOutOfBounds { at, .. }
            | Error::DivideByZero { at }
            | Error::Overflow { at, .. }
            | Error::WrongArity { at, .. }
            | Error::NoMatchingArm { at }
            | Error::NotCallable { at, .. } => Some(*at),
            Error::OutOfFuel => None,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Error::NotInSubset { form, .. } => format!(
                "`{form}` is outside the stage-0 imperative subset; the relational tier is \
                 compiled, not interpreted"
            ),
            Error::Unbound { name, .. } => format!("`{name}` is not bound here"),
            Error::TypeMismatch { want, got, .. } => format!("expected {want}, found {got}"),
            Error::NoField { name, .. } => format!("no field `{name}`"),
            Error::IndexOutOfBounds { index, len, .. } => {
                format!("index {index} is out of bounds for a length of {len}")
            }
            Error::DivideByZero { .. } => "division by zero".into(),
            Error::Overflow { op, .. } => format!("integer overflow in `{op}`"),
            Error::WrongArity { want, got, .. } => {
                format!("expected {want} argument(s), found {got}")
            }
            Error::NoMatchingArm { .. } => "no arm matched".into(),
            Error::OutOfFuel => "evaluation exceeded its fuel budget".into(),
            Error::NotCallable { got, .. } => format!("`{got}` is not callable"),
        }
    }
}

/// Non-local control flow, threaded through evaluation as an error-like signal.
enum Flow {
    Err(Error),
    Return(Value),
    Break,
    Continue,
}

impl From<Error> for Flow {
    fn from(e: Error) -> Flow {
        Flow::Err(e)
    }
}

type Eval<T> = Result<T, Flow>;

/// A lexical scope chain. `BTreeMap` rather than `HashMap` for the reason in the module
/// docs: iteration order must not depend on a hash seed.
#[derive(Debug, Default, Clone)]
struct Env {
    frames: Vec<BTreeMap<String, Value>>,
}

impl Env {
    fn new() -> Self {
        Env { frames: vec![BTreeMap::new()] }
    }
    fn push(&mut self) {
        self.frames.push(BTreeMap::new());
    }
    fn pop(&mut self) {
        self.frames.pop();
    }
    fn define(&mut self, name: &str, v: Value) {
        self.frames.last_mut().expect("at least one frame").insert(name.into(), v);
    }
    fn get(&self, name: &str) -> Option<&Value> {
        self.frames.iter().rev().find_map(|f| f.get(name))
    }
    fn set(&mut self, name: &str, v: Value) -> bool {
        for f in self.frames.iter_mut().rev() {
            if let Some(slot) = f.get_mut(name) {
                *slot = v;
                return true;
            }
        }
        false
    }
    /// Everything visible, innermost shadowing outermost. The capture set of a closure.
    fn flatten(&self) -> BTreeMap<String, Value> {
        let mut out = BTreeMap::new();
        for f in &self.frames {
            for (k, v) in f {
                out.insert(k.clone(), v.clone());
            }
        }
        out
    }
}

/// The interpreter.
pub struct Interp {
    fns: BTreeMap<String, Rc<FnDecl>>,
    /// Enum variants, so `Tok::Ident` resolves as a constructor rather than a variable.
    variants: BTreeMap<String, usize>,
    /// Struct field lists, for positional construction and for arity checking.
    structs: BTreeMap<String, Vec<String>>,
    /// Everything the program printed. The only output channel; there is no `stdout`
    /// here, because a determinism gate that compares terminal output compares the
    /// terminal too.
    pub output: Vec<String>,
    /// A step budget. Guarded recursion (§6.7) is a *language* guarantee for the
    /// relational tier; the imperative tier has ordinary `while`, so an interpreter
    /// running a compiler needs a way to stop rather than hang a test suite.
    fuel: u64,
}

impl Default for Interp {
    fn default() -> Self {
        Interp::new()
    }
}

impl Interp {
    pub fn new() -> Self {
        Interp {
            fns: BTreeMap::new(),
            variants: BTreeMap::new(),
            structs: BTreeMap::new(),
            output: Vec::new(),
            fuel: 50_000_000,
        }
    }

    pub fn with_fuel(mut self, fuel: u64) -> Self {
        self.fuel = fuel;
        self
    }

    /// Load a parsed program's items. Relational items are *skipped*, not rejected: a
    /// source file may legitimately declare a schema and a helper function, and the
    /// interpreter's business is the function.
    pub fn load(&mut self, prog: &Program) {
        for item in &prog.items {
            self.load_item(item);
        }
    }

    fn load_item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => {
                self.fns.insert(f.name.text.clone(), Rc::new(f.clone()));
            }
            Item::Struct(s) => {
                self.structs
                    .insert(s.name.text.clone(), s.fields.iter().map(|f| f.name.text.clone()).collect());
            }
            Item::Enum(e) => {
                for (v, tys) in &e.variants {
                    self.variants.insert(format!("{}::{}", e.name.text, v.text), tys.len());
                    // The bare form too, so `Ident(s)` works inside the enum's own module,
                    // matching Rust's `use Tok::*`.
                    self.variants.entry(v.text.clone()).or_insert(tys.len());
                }
            }
            Item::Impl(i) => {
                for f in &i.items {
                    self.fns.insert(f.name.text.clone(), Rc::new(f.clone()));
                }
            }
            Item::Mod { items, .. } => {
                for it in items {
                    self.load_item(it);
                }
            }
            _ => {}
        }
    }

    pub fn function_names(&self) -> Vec<String> {
        self.fns.keys().cloned().collect()
    }

    /// Call a top-level function by name.
    pub fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, Error> {
        let at = Span { start: 0, end: 0 };
        let f = self
            .fns
            .get(name)
            .cloned()
            .ok_or_else(|| Error::Unbound { name: name.into(), at })?;
        match self.call_decl(&f, args, at) {
            Ok(v) => Ok(v),
            Err(Flow::Err(e)) => Err(e),
            Err(Flow::Return(v)) => Ok(v),
            Err(_) => Err(Error::NotInSubset { form: "break outside a loop", at }),
        }
    }

    fn call_decl(&mut self, f: &FnDecl, args: Vec<Value>, at: Span) -> Eval<Value> {
        if args.len() != f.params.len() {
            return Err(Error::WrongArity { want: f.params.len(), got: args.len(), at }.into());
        }
        let body = match &f.body {
            Some(b) => b.clone(),
            None => return Err(Error::NotInSubset { form: "function without a body", at }.into()),
        };
        // A fresh environment, not the caller's: the subset has no dynamic scope.
        let mut env = Env::new();
        for (p, v) in f.params.iter().zip(args) {
            self.bind(&mut env, &p.pat, v)?;
        }
        match self.block(&mut env, &body) {
            Ok(v) => Ok(v),
            Err(Flow::Return(v)) => Ok(v),
            Err(other) => Err(other),
        }
    }

    fn burn(&mut self) -> Eval<()> {
        if self.fuel == 0 {
            return Err(Error::OutOfFuel.into());
        }
        self.fuel -= 1;
        Ok(())
    }

    // ── statements and blocks ────────────────────────────────────────────────────────

    fn block(&mut self, env: &mut Env, b: &Block) -> Eval<Value> {
        env.push();
        let r = self.block_inner(env, b);
        env.pop();
        r
    }

    fn block_inner(&mut self, env: &mut Env, b: &Block) -> Eval<Value> {
        for s in &b.stmts {
            self.stmt(env, s)?;
        }
        match &b.tail {
            Some(e) => self.expr(env, e),
            None => Ok(Value::Unit),
        }
    }

    fn stmt(&mut self, env: &mut Env, s: &Stmt) -> Eval<()> {
        self.burn()?;
        match s {
            Stmt::Let { pat, init, span, .. } => {
                let v = match init {
                    Some(e) => self.expr(env, e)?,
                    None => Value::Unit,
                };
                let _ = span;
                self.bind(env, pat, v)
            }
            Stmt::Expr(e) | Stmt::Semi(e) => {
                self.expr(env, e)?;
                Ok(())
            }
            Stmt::Item(i) => {
                self.load_item(i);
                Ok(())
            }
            Stmt::Dml(d) => Err(Error::NotInSubset {
                form: match d {
                    Dml::Insert { .. } => "insert",
                    Dml::Update { .. } => "update",
                    Dml::Delete { .. } => "delete",
                    Dml::Begin(_) => "begin",
                    Dml::Commit(_) => "commit",
                    Dml::Rollback(_) => "rollback",
                    Dml::Grant { .. } => "grant",
                    Dml::Revoke { .. } => "revoke",
                    Dml::Backfill { .. } => "backfill",
                    Dml::Emit { .. } => "emit",
                },
                at: Span { start: 0, end: 0 },
            }
            .into()),
            Stmt::Error(sp) => {
                Err(Error::NotInSubset { form: "a statement that did not parse", at: *sp }.into())
            }
        }
    }

    // ── patterns ─────────────────────────────────────────────────────────────────────

    fn bind(&mut self, env: &mut Env, p: &Pat, v: Value) -> Eval<()> {
        match self.try_bind(env, p, &v) {
            Ok(true) => Ok(()),
            Ok(false) => Err(Error::NoMatchingArm { at: p.span() }.into()),
            Err(e) => Err(e),
        }
    }

    /// Attempt a match, binding on success. Returns `false` for a refutable pattern that
    /// did not match, which is how `match` arms are tried in order.
    fn try_bind(&mut self, env: &mut Env, p: &Pat, v: &Value) -> Eval<bool> {
        match p {
            Pat::Wild(_) => Ok(true),
            Pat::Bind { name, .. } => {
                env.define(&name.text, v.clone());
                Ok(true)
            }
            Pat::Tuple { elems, span } => match v {
                Value::Tuple(xs) | Value::Array(xs) if xs.len() == elems.len() => {
                    for (pe, ve) in elems.iter().zip(xs.iter()) {
                        if !self.try_bind(env, pe, ve)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => {
                    let _ = span;
                    Ok(false)
                }
            },
            Pat::TupleStruct { path, elems, .. } => {
                let want = path_text(path);
                match v {
                    Value::Variant { path: got, payload }
                        if variant_matches(&want, got) && payload.len() == elems.len() =>
                    {
                        for (pe, ve) in elems.iter().zip(payload.iter()) {
                            if !self.try_bind(env, pe, ve)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Pat::Struct { path, fields, .. } => {
                let want = path_text(path);
                match v {
                    Value::Record { name, fields: got } if variant_matches(&want, name) => {
                        for (fname, fpat) in fields {
                            match got.get(&fname.text) {
                                Some(fv) => {
                                    let fv = fv.clone();
                                    if !self.try_bind(env, fpat, &fv)? {
                                        return Ok(false);
                                    }
                                }
                                None => return Ok(false),
                            }
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Pat::Lit(e) => {
                let lv = self.expr(env, e)?;
                Ok(&lv == v)
            }
            Pat::Path(path) => {
                let want = path_text(path);
                // A bare lowercase identifier in pattern position with no known variant is
                // a binding, matching Rust. An uppercase one, or one that names a known
                // variant, is a unit-variant test.
                let known = self.variants.contains_key(&want);
                let uppercase = want.chars().next().is_some_and(|c| c.is_ascii_uppercase());
                if known || uppercase || want.contains("::") {
                    match v {
                        Value::Variant { path: got, payload } if payload.is_empty() => {
                            Ok(variant_matches(&want, got))
                        }
                        _ => Ok(false),
                    }
                } else {
                    env.define(&want, v.clone());
                    Ok(true)
                }
            }
            Pat::Error(sp) => {
                Err(Error::NotInSubset { form: "a pattern that did not parse", at: *sp }.into())
            }
        }
    }

    // ── expressions ──────────────────────────────────────────────────────────────────

    fn expr(&mut self, env: &mut Env, e: &Expr) -> Eval<Value> {
        self.burn()?;
        match e {
            // --- leaves ---
            Expr::Int(i, _) => Ok(Value::Int(*i)),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Str(s, _) => Ok(Value::Str(Rc::new(s.clone()))),
            Expr::Bytes(b, _) => Ok(Value::Bytes(Rc::new(b.clone()))),
            Expr::Unit(_) => Ok(Value::Unit),
            Expr::Money { minor, scale, currency, .. } => Ok(Value::Money {
                minor: *minor,
                scale: *scale,
                currency: currency.text.clone(),
            }),
            Expr::Epoch(n, _) => Ok(Value::Epoch(*n)),
            // Floats are excluded from the subset on purpose: cross-target determinism
            // (Appendix C.4) and money never being a float are the same commitment.
            Expr::Float(_, sp) => {
                Err(Error::NotInSubset { form: "floating-point literal", at: *sp }.into())
            }
            // Temporal literals belong to the bitemporal tier, which is checked and
            // lowered rather than interpreted. Carrying them as opaque values would let a
            // Niles program compare two instants without the axis discipline of §3.8.
            Expr::Instant { span, .. } => {
                Err(Error::NotInSubset { form: "temporal literal", at: *span }.into())
            }
            Expr::Duration { span, .. } => {
                Err(Error::NotInSubset { form: "duration literal", at: *span }.into())
            }

            Expr::Path(p) => {
                let name = path_text(p);
                if let Some(v) = env.get(&name) {
                    return Ok(v.clone());
                }
                if let Some(&arity) = self.variants.get(&name) {
                    if arity == 0 {
                        return Ok(Value::Variant { path: name, payload: Rc::new(vec![]) });
                    }
                }
                Err(Error::Unbound { name, at: p.span }.into())
            }

            // --- composition ---
            Expr::Tuple { elems, .. } => {
                let vs = self.eval_all(env, elems)?;
                Ok(Value::Tuple(Rc::new(vs)))
            }
            Expr::Array { elems, .. } => {
                let vs = self.eval_all(env, elems)?;
                Ok(Value::Array(Rc::new(vs)))
            }
            Expr::StructLit { path, fields, .. } => {
                let mut m = BTreeMap::new();
                for (n, fe) in fields {
                    let v = self.expr(env, fe)?;
                    m.insert(n.text.clone(), v);
                }
                Ok(Value::Record { name: path_text(path), fields: Rc::new(m) })
            }
            Expr::Field { base, name, span } => {
                let b = self.expr(env, base)?;
                match &b {
                    Value::Record { fields, .. } => fields
                        .get(&name.text)
                        .cloned()
                        .ok_or_else(|| Error::NoField { name: name.text.clone(), at: *span }.into()),
                    // Tuple field access, `t.0`.
                    Value::Tuple(xs) => match name.text.parse::<usize>() {
                        Ok(i) if i < xs.len() => Ok(xs[i].clone()),
                        _ => Err(Error::NoField { name: name.text.clone(), at: *span }.into()),
                    },
                    other => Err(Error::TypeMismatch {
                        want: "struct".into(),
                        got: other.type_name().into(),
                        at: *span,
                    }
                    .into()),
                }
            }
            Expr::Index { base, index, span } => {
                let b = self.expr(env, base)?;
                let i = self.expr(env, index)?;
                let idx = match i {
                    Value::Int(n) => n,
                    other => {
                        return Err(Error::TypeMismatch {
                            want: "int".into(),
                            got: other.type_name().into(),
                            at: *span,
                        }
                        .into())
                    }
                };
                match &b {
                    Value::Array(xs) | Value::Tuple(xs) => {
                        if idx < 0 || idx as usize >= xs.len() {
                            Err(Error::IndexOutOfBounds { index: idx, len: xs.len(), at: *span }.into())
                        } else {
                            Ok(xs[idx as usize].clone())
                        }
                    }
                    Value::Bytes(bs) => {
                        if idx < 0 || idx as usize >= bs.len() {
                            Err(Error::IndexOutOfBounds { index: idx, len: bs.len(), at: *span }.into())
                        } else {
                            Ok(Value::Int(bs[idx as usize] as i128))
                        }
                    }
                    other => Err(Error::TypeMismatch {
                        want: "array".into(),
                        got: other.type_name().into(),
                        at: *span,
                    }
                    .into()),
                }
            }

            Expr::Closure { params, body, .. } => Ok(Value::Closure(Rc::new(ClosureVal {
                params: params.iter().map(|(p, _)| p.clone()).collect(),
                body: (**body).clone(),
                captured: env.flatten(),
            }))),

            Expr::Call { callee, args, span } => self.call_expr(env, callee, args, *span),

            Expr::Unary { op, operand, span } => {
                let v = self.expr(env, operand)?;
                match (op, &v) {
                    (UnOp::Neg, Value::Int(i)) => {
                        i.checked_neg().map(Value::Int).ok_or_else(|| Error::Overflow { op: "-", at: *span }.into())
                    }
                    (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    // References are a no-op here: the subset has no aliasing, because
                    // values are cloned. §6.6's memory model is a compile-time claim, and
                    // an interpreter that faked ownership would be evidence for nothing.
                    (UnOp::Ref | UnOp::RefMut | UnOp::Deref, _) => Ok(v),
                    _ => Err(Error::TypeMismatch {
                        want: "a numeric or boolean operand".into(),
                        got: v.type_name().into(),
                        at: *span,
                    }
                    .into()),
                }
            }

            Expr::Binary { op, lhs, rhs, span } => self.binary(env, *op, lhs, rhs, *span),

            Expr::Assign { target, value, span } => {
                let v = self.expr(env, value)?;
                match &**target {
                    Expr::Path(p) => {
                        let name = path_text(p);
                        if env.set(&name, v) {
                            Ok(Value::Unit)
                        } else {
                            Err(Error::Unbound { name, at: *span }.into())
                        }
                    }
                    // `xs[i] = v`. Arrays are `Rc<Vec<_>>`, so this reads, mutates a
                    // clone, and rebinds — O(n) per write. Correct and slow, which is the
                    // right trade for a stage-0 interpreter whose successor is a compiler.
                    Expr::Index { base, index, .. } => {
                        let idx = match self.expr(env, index)? {
                            Value::Int(n) => n,
                            other => {
                                return Err(Error::TypeMismatch {
                                    want: "int".into(),
                                    got: other.type_name().into(),
                                    at: *span,
                                }
                                .into())
                            }
                        };
                        let name = match &**base {
                            Expr::Path(p) => path_text(p),
                            _ => {
                                return Err(Error::NotInSubset {
                                    form: "assignment to a computed place",
                                    at: *span,
                                }
                                .into())
                            }
                        };
                        let cur = env
                            .get(&name)
                            .cloned()
                            .ok_or_else(|| Flow::Err(Error::Unbound { name: name.clone(), at: *span }))?;
                        match cur {
                            Value::Array(xs) => {
                                if idx < 0 || idx as usize >= xs.len() {
                                    return Err(Error::IndexOutOfBounds {
                                        index: idx,
                                        len: xs.len(),
                                        at: *span,
                                    }
                                    .into());
                                }
                                let mut new = (*xs).clone();
                                new[idx as usize] = v;
                                env.set(&name, Value::Array(Rc::new(new)));
                                Ok(Value::Unit)
                            }
                            other => Err(Error::TypeMismatch {
                                want: "array".into(),
                                got: other.type_name().into(),
                                at: *span,
                            }
                            .into()),
                        }
                    }
                    Expr::Field { base, name, .. } => {
                        let vname = match &**base {
                            Expr::Path(p) => path_text(p),
                            _ => {
                                return Err(Error::NotInSubset {
                                    form: "assignment to a computed place",
                                    at: *span,
                                }
                                .into())
                            }
                        };
                        let cur = env
                            .get(&vname)
                            .cloned()
                            .ok_or_else(|| Flow::Err(Error::Unbound { name: vname.clone(), at: *span }))?;
                        match cur {
                            Value::Record { name: rn, fields } => {
                                let mut m = (*fields).clone();
                                m.insert(name.text.clone(), v);
                                env.set(&vname, Value::Record { name: rn, fields: Rc::new(m) });
                                Ok(Value::Unit)
                            }
                            other => Err(Error::TypeMismatch {
                                want: "struct".into(),
                                got: other.type_name().into(),
                                at: *span,
                            }
                            .into()),
                        }
                    }
                    _ => Err(Error::NotInSubset { form: "assignment to this place", at: *span }.into()),
                }
            }

            // A cast in the subset is between integers and is therefore the identity;
            // `as u8`-style truncation is not modelled, because a silent truncation in a
            // ledger interpreter is exactly the class of defect this thesis argues against.
            Expr::Cast { expr, .. } => self.expr(env, expr),

            Expr::Try { expr, span } => {
                let v = self.expr(env, expr)?;
                match &v {
                    Value::Variant { path, payload } if variant_matches("Ok", path) => {
                        Ok(payload.first().cloned().unwrap_or(Value::Unit))
                    }
                    Value::Variant { path, .. } if variant_matches("Err", path) => {
                        Err(Flow::Return(v))
                    }
                    Value::Variant { path, payload } if variant_matches("Some", path) => {
                        Ok(payload.first().cloned().unwrap_or(Value::Unit))
                    }
                    Value::Variant { path, .. } if variant_matches("None", path) => {
                        Err(Flow::Return(v))
                    }
                    other => Err(Error::TypeMismatch {
                        want: "Result or Option".into(),
                        got: other.type_name().into(),
                        at: *span,
                    }
                    .into()),
                }
            }

            // --- control ---
            Expr::Block(b) => self.block(env, b),

            Expr::If { cond, then, els, span } => {
                let c = self.expr(env, cond)?.truthy(*span)?;
                if c {
                    self.block(env, then)
                } else {
                    match els {
                        Some(e) => self.expr(env, e),
                        None => Ok(Value::Unit),
                    }
                }
            }

            Expr::Case { arms, els, span } => {
                for (pred, body) in arms {
                    if self.expr(env, pred)?.truthy(*span)? {
                        return self.expr(env, body);
                    }
                }
                match els {
                    Some(e) => self.expr(env, e),
                    // SQL's `case` with no `else` yields null. The subset has no null, so
                    // this is unit, and the difference is documented rather than hidden.
                    None => Ok(Value::Unit),
                }
            }

            Expr::Match { scrutinee, arms, span } => {
                let v = self.expr(env, scrutinee)?;
                for arm in arms {
                    env.push();
                    let matched = self.try_bind(env, &arm.pat, &v)?;
                    if matched {
                        let guard_ok = match &arm.guard {
                            Some(g) => self.expr(env, g)?.truthy(*span)?,
                            None => true,
                        };
                        if guard_ok {
                            let r = self.expr(env, &arm.body);
                            env.pop();
                            return r;
                        }
                    }
                    env.pop();
                }
                Err(Error::NoMatchingArm { at: *span }.into())
            }

            Expr::While { cond, body, span } => {
                loop {
                    self.burn()?;
                    if !self.expr(env, cond)?.truthy(*span)? {
                        break;
                    }
                    match self.block(env, body) {
                        Ok(_) | Err(Flow::Continue) => {}
                        Err(Flow::Break) => break,
                        Err(other) => return Err(other),
                    }
                }
                Ok(Value::Unit)
            }

            Expr::Loop { body, .. } => {
                loop {
                    self.burn()?;
                    match self.block(env, body) {
                        Ok(_) | Err(Flow::Continue) => {}
                        Err(Flow::Break) => break,
                        Err(other) => return Err(other),
                    }
                }
                Ok(Value::Unit)
            }

            Expr::For { pat, iter, body, span } => {
                let it = self.expr(env, iter)?;
                let items: Vec<Value> = match &it {
                    Value::Array(xs) | Value::Tuple(xs) => (**xs).clone(),
                    Value::Bytes(bs) => bs.iter().map(|b| Value::Int(*b as i128)).collect(),
                    Value::Record { name, fields } if name == "Range" => {
                        let lo = as_int(fields.get("start"), *span)?;
                        let hi = as_int(fields.get("end"), *span)?;
                        (lo..hi).map(Value::Int).collect()
                    }
                    other => {
                        return Err(Error::TypeMismatch {
                            want: "an iterable".into(),
                            got: other.type_name().into(),
                            at: *span,
                        }
                        .into())
                    }
                };
                for v in items {
                    self.burn()?;
                    env.push();
                    let bound = self.try_bind(env, pat, &v);
                    let r = match bound {
                        Ok(true) => self.block_inner(env, body).map(|_| ()),
                        Ok(false) => Err(Flow::Err(Error::NoMatchingArm { at: *span })),
                        Err(e) => Err(e),
                    };
                    env.pop();
                    match r {
                        Ok(()) | Err(Flow::Continue) => {}
                        Err(Flow::Break) => break,
                        Err(other) => return Err(other),
                    }
                }
                Ok(Value::Unit)
            }

            Expr::Return { value, .. } => {
                let v = match value {
                    Some(e) => self.expr(env, e)?,
                    None => Value::Unit,
                };
                Err(Flow::Return(v))
            }
            Expr::Break(_) => Err(Flow::Break),
            Expr::Continue(_) => Err(Flow::Continue),

            // --- the relational tier: named, refused, never approximated ---
            Expr::Stage { span, .. } => self.stage_or_method(env, e, *span),
            Expr::Txn { span, .. } => Err(Error::NotInSubset { form: "txn", at: *span }.into()),
            Expr::Hold { span, .. } => Err(Error::NotInSubset { form: "hold", at: *span }.into()),
            Expr::Resolve { span, .. } => Err(Error::NotInSubset { form: "resolve", at: *span }.into()),
            Expr::Fx { span, .. } => Err(Error::NotInSubset { form: "fx", at: *span }.into()),
            Expr::Fixpoint { span, .. } => {
                Err(Error::NotInSubset { form: "fixpoint", at: *span }.into())
            }
            Expr::Authorize { span, .. } => {
                Err(Error::NotInSubset { form: "authorize", at: *span }.into())
            }
            Expr::Declassify { span, .. } => {
                Err(Error::NotInSubset { form: "declassify", at: *span }.into())
            }
            Expr::Explain { span, .. } => Err(Error::NotInSubset { form: "explain", at: *span }.into()),
            Expr::Reproduce { span, .. } => {
                Err(Error::NotInSubset { form: "reproduce", at: *span }.into())
            }
            Expr::Impact { span, .. } => Err(Error::NotInSubset { form: "impact", at: *span }.into()),
            Expr::Sql { span, .. } => Err(Error::NotInSubset { form: "sql", at: *span }.into()),
            Expr::Select(s) => Err(Error::NotInSubset { form: "select", at: s.span }.into()),
            Expr::Error(sp) => {
                Err(Error::NotInSubset { form: "an expression that did not parse", at: *sp }.into())
            }
        }
    }

    fn eval_all(&mut self, env: &mut Env, es: &[Expr]) -> Eval<Vec<Value>> {
        // Left to right, always. Evaluation order is observable through `print`, so it is
        // part of the determinism obligation and not an implementation detail.
        es.iter().map(|e| self.expr(env, e)).collect()
    }

    /// A `.method(..)` reaches the parser as a `Stage`. In the imperative subset those
    /// are the collection and string builtins.
    fn stage_or_method(&mut self, env: &mut Env, e: &Expr, span: Span) -> Eval<Value> {
        let (recv, name, args) = match e {
            Expr::Stage { recv, name, args, .. } => (recv, name.text.as_str(), args),
            _ => unreachable!("stage_or_method called on a non-stage"),
        };
        let r = self.expr(env, recv)?;
        let mut vs = Vec::new();
        for a in args {
            vs.push(self.expr(env, &a.value)?);
        }
        builtin_method(&r, name, &vs, span).map_err(Flow::Err)
    }

    fn call_expr(&mut self, env: &mut Env, callee: &Expr, args: &[Arg], span: Span) -> Eval<Value> {
        let name = match callee {
            Expr::Path(p) => Some(path_text(p)),
            _ => None,
        };
        let mut vs = Vec::new();
        for a in args {
            vs.push(self.expr(env, &a.value)?);
        }

        if let Some(n) = &name {
            // A variant constructor: `Tok::Ident("x")`.
            if let Some(&arity) = self.variants.get(n) {
                if arity == vs.len() {
                    return Ok(Value::Variant { path: n.clone(), payload: Rc::new(vs) });
                }
            }
            // A struct built positionally.
            if let Some(fields) = self.structs.get(n).cloned() {
                if fields.len() == vs.len() {
                    let m: BTreeMap<String, Value> =
                        fields.into_iter().zip(vs.clone()).collect();
                    return Ok(Value::Record { name: n.clone(), fields: Rc::new(m) });
                }
            }
            // A user function.
            if let Some(f) = self.fns.get(n).cloned() {
                return self.call_decl(&f, vs, span);
            }
            // A free builtin.
            if let Some(v) = self.builtin_free(n, &vs, span)? {
                return Ok(v);
            }
            // A closure held in a variable.
            if let Some(Value::Closure(c)) = env.get(n).cloned() {
                return self.call_closure(&c, vs, span);
            }
            return Err(Error::Unbound { name: n.clone(), at: span }.into());
        }

        match self.expr(env, callee)? {
            Value::Closure(c) => self.call_closure(&c, vs, span),
            other => Err(Error::NotCallable { got: other.type_name().into(), at: span }.into()),
        }
    }

    fn call_closure(&mut self, c: &Rc<ClosureVal>, args: Vec<Value>, span: Span) -> Eval<Value> {
        if args.len() != c.params.len() {
            return Err(Error::WrongArity { want: c.params.len(), got: args.len(), at: span }.into());
        }
        let mut env = Env::new();
        for (k, v) in &c.captured {
            env.define(k, v.clone());
        }
        env.push();
        for (p, v) in c.params.iter().zip(args) {
            self.bind(&mut env, p, v)?;
        }
        match self.expr(&mut env, &c.body) {
            Ok(v) => Ok(v),
            Err(Flow::Return(v)) => Ok(v),
            Err(other) => Err(other),
        }
    }

    /// Free functions. Deliberately few: this is the "100-line host shim" of Appendix E.3,
    /// and every entry is a capability the Niles-written compiler would otherwise have to
    /// obtain from somewhere unprincipled.
    fn builtin_free(&mut self, name: &str, vs: &[Value], span: Span) -> Eval<Option<Value>> {
        let v = match (name, vs) {
            ("print", [v]) => {
                self.output.push(v.render());
                Value::Unit
            }
            ("len", [v]) => Value::Int(value_len(v, span).map_err(Flow::Err)? as i128),
            ("panic", [v]) => {
                return Err(Error::TypeMismatch {
                    want: "no panic".into(),
                    got: v.render(),
                    at: span,
                }
                .into())
            }
            ("Some", [v]) => Value::Variant { path: "Some".into(), payload: Rc::new(vec![v.clone()]) },
            ("Ok", [v]) => Value::Variant { path: "Ok".into(), payload: Rc::new(vec![v.clone()]) },
            ("Err", [v]) => Value::Variant { path: "Err".into(), payload: Rc::new(vec![v.clone()]) },
            ("range", [Value::Int(a), Value::Int(b)]) => {
                let mut m = BTreeMap::new();
                m.insert("start".to_string(), Value::Int(*a));
                m.insert("end".to_string(), Value::Int(*b));
                Value::Record { name: "Range".into(), fields: Rc::new(m) }
            }
            ("chr", [Value::Int(i)]) => {
                Value::Str(Rc::new(((*i as u8) as char).to_string()))
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    fn binary(&mut self, env: &mut Env, op: BinOp, l: &Expr, r: &Expr, span: Span) -> Eval<Value> {
        // Short-circuit before evaluating the right side, or `i < len(s) && s[i] == 32`
        // reads out of bounds at the end of every string in the Niles lexer.
        if matches!(op, BinOp::And | BinOp::Or) {
            let lv = self.expr(env, l)?.truthy(span)?;
            return match (op, lv) {
                (BinOp::And, false) => Ok(Value::Bool(false)),
                (BinOp::Or, true) => Ok(Value::Bool(true)),
                _ => Ok(Value::Bool(self.expr(env, r)?.truthy(span)?)),
            };
        }
        let a = self.expr(env, l)?;
        let b = self.expr(env, r)?;
        arith(op, &a, &b, span).map_err(Flow::Err)
    }
}

fn as_int(v: Option<&Value>, at: Span) -> Eval<i128> {
    match v {
        Some(Value::Int(i)) => Ok(*i),
        Some(other) => Err(Error::TypeMismatch {
            want: "int".into(),
            got: other.type_name().into(),
            at,
        }
        .into()),
        None => Err(Error::NoField { name: "start/end".into(), at }.into()),
    }
}

fn value_len(v: &Value, at: Span) -> Result<usize, Error> {
    match v {
        Value::Str(s) => Ok(s.len()),
        Value::Bytes(b) => Ok(b.len()),
        Value::Array(xs) | Value::Tuple(xs) => Ok(xs.len()),
        other => Err(Error::TypeMismatch { want: "a sized value".into(), got: other.type_name().into(), at }),
    }
}

/// Whether a pattern's path names this runtime variant. `Tok::Ident` matches a value
/// tagged `Ident` or `Tok::Ident`, so a program can `use` the enum or not.
fn variant_matches(want: &str, got: &str) -> bool {
    want == got
        || want.rsplit("::").next() == got.rsplit("::").next()
}

fn path_text(p: &Path) -> String {
    p.segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("::")
}

/// Arithmetic and comparison. Every integer operation is checked.
fn arith(op: BinOp, a: &Value, b: &Value, at: Span) -> Result<Value, Error> {
    use BinOp::*;
    match (op, a, b) {
        (Add, Value::Int(x), Value::Int(y)) => {
            x.checked_add(*y).map(Value::Int).ok_or(Error::Overflow { op: "+", at })
        }
        (Sub, Value::Int(x), Value::Int(y)) => {
            x.checked_sub(*y).map(Value::Int).ok_or(Error::Overflow { op: "-", at })
        }
        (Mul, Value::Int(x), Value::Int(y)) => {
            x.checked_mul(*y).map(Value::Int).ok_or(Error::Overflow { op: "*", at })
        }
        (Div, Value::Int(_), Value::Int(0)) => Err(Error::DivideByZero { at }),
        (Div, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
        (Rem, Value::Int(_), Value::Int(0)) => Err(Error::DivideByZero { at }),
        (Rem, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x % y)),
        (BitAnd, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x & y)),
        (BitOr, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x | y)),
        (BitXor, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x ^ y)),

        (Add, Value::Str(x), Value::Str(y)) => Ok(Value::Str(Rc::new(format!("{x}{y}")))),

        // Money arithmetic is *not* provided. Adding two `Money` values requires the
        // currency check that the type system performs; doing it here would create a
        // second, unchecked path to the same operation, which is the seam §6.9 refuses.
        (Add | Sub, Value::Money { .. }, _) | (Add | Sub, _, Value::Money { .. }) => {
            Err(Error::NotInSubset { form: "money arithmetic (checked tier only)", at })
        }

        (Eq, _, _) => Ok(Value::Bool(a == b)),
        (Ne, _, _) => Ok(Value::Bool(a != b)),
        (Lt | Le | Gt | Ge, _, _) => {
            let ord = match (a, b) {
                (Value::Int(x), Value::Int(y)) => x.cmp(y),
                (Value::Str(x), Value::Str(y)) => x.cmp(y),
                _ => {
                    return Err(Error::TypeMismatch {
                        want: "two comparable values of the same type".into(),
                        got: format!("{} and {}", a.type_name(), b.type_name()),
                        at,
                    })
                }
            };
            Ok(Value::Bool(match op {
                Lt => ord.is_lt(),
                Le => ord.is_le(),
                Gt => ord.is_gt(),
                _ => ord.is_ge(),
            }))
        }

        (In, x, Value::Array(xs)) => Ok(Value::Bool(xs.contains(x))),
        (NotIn, x, Value::Array(xs)) => Ok(Value::Bool(!xs.contains(x))),

        _ => Err(Error::TypeMismatch {
            want: format!("operands for `{op:?}`"),
            got: format!("{} and {}", a.type_name(), b.type_name()),
            at,
        }),
    }
}

/// Methods on built-in values. The second half of the host shim.
fn builtin_method(recv: &Value, name: &str, args: &[Value], at: Span) -> Result<Value, Error> {
    match (recv, name, args) {
        // --- universal ---
        (v, "len", []) => Ok(Value::Int(value_len(v, at)? as i128)),
        (v, "to_string", []) => Ok(Value::Str(Rc::new(v.render()))),

        // --- strings ---
        (Value::Str(s), "byte_at", [Value::Int(i)]) => {
            let b = s.as_bytes();
            if *i < 0 || *i as usize >= b.len() {
                Err(Error::IndexOutOfBounds { index: *i, len: b.len(), at })
            } else {
                Ok(Value::Int(b[*i as usize] as i128))
            }
        }
        (Value::Str(s), "slice", [Value::Int(a), Value::Int(b)]) => {
            let bytes = s.as_bytes();
            let (lo, hi) = (*a.max(&0) as usize, (*b).clamp(0, bytes.len() as i128) as usize);
            if lo > hi || hi > bytes.len() {
                return Err(Error::IndexOutOfBounds { index: *b, len: bytes.len(), at });
            }
            Ok(Value::Str(Rc::new(String::from_utf8_lossy(&bytes[lo..hi]).into_owned())))
        }
        (Value::Str(s), "starts_with", [Value::Str(p)]) => Ok(Value::Bool(s.starts_with(&**p))),
        (Value::Str(s), "contains", [Value::Str(p)]) => Ok(Value::Bool(s.contains(&**p))),
        // ASCII-only, deliberately: a locale-dependent case fold would make the lexer's
        // keyword lookup depend on the environment, which Appendix C.4 forbids.
        (Value::Str(s), "to_lower", []) => Ok(Value::Str(Rc::new(s.to_ascii_lowercase()))),
        (Value::Str(s), "to_upper", []) => Ok(Value::Str(Rc::new(s.to_ascii_uppercase()))),
        (Value::Str(s), "bytes", []) => Ok(Value::Bytes(Rc::new(s.as_bytes().to_vec()))),
        (Value::Str(s), "parse_int", []) => match s.trim().parse::<i128>() {
            Ok(i) => Ok(Value::Variant { path: "Some".into(), payload: Rc::new(vec![Value::Int(i)]) }),
            Err(_) => Ok(Value::Variant { path: "None".into(), payload: Rc::new(vec![]) }),
        },

        // --- arrays. Persistent: `push` returns a new array rather than mutating, which
        // is what lets the subset have no aliasing at all. ---
        (Value::Array(xs), "push", [v]) => {
            let mut new = (**xs).clone();
            new.push(v.clone());
            Ok(Value::Array(Rc::new(new)))
        }
        (Value::Array(xs), "get", [Value::Int(i)]) => {
            if *i < 0 || *i as usize >= xs.len() {
                Ok(Value::Variant { path: "None".into(), payload: Rc::new(vec![]) })
            } else {
                Ok(Value::Variant {
                    path: "Some".into(),
                    payload: Rc::new(vec![xs[*i as usize].clone()]),
                })
            }
        }
        (Value::Array(xs), "last", []) => match xs.last() {
            Some(v) => Ok(Value::Variant { path: "Some".into(), payload: Rc::new(vec![v.clone()]) }),
            None => Ok(Value::Variant { path: "None".into(), payload: Rc::new(vec![]) }),
        },

        // --- character classification. In the shim rather than in Niles because the
        // alternative is a Niles-side table, and a bootstrap should not begin by
        // transcribing ASCII. ---
        (Value::Int(c), "is_digit", []) => Ok(Value::Bool((b'0' as i128..=b'9' as i128).contains(c))),
        (Value::Int(c), "is_alpha", []) => Ok(Value::Bool(
            (b'a' as i128..=b'z' as i128).contains(c) || (b'A' as i128..=b'Z' as i128).contains(c),
        )),
        (Value::Int(c), "is_ident_start", []) => Ok(Value::Bool(
            (b'a' as i128..=b'z' as i128).contains(c)
                || (b'A' as i128..=b'Z' as i128).contains(c)
                || *c == b'_' as i128,
        )),
        (Value::Int(c), "is_ident_continue", []) => Ok(Value::Bool(
            (b'a' as i128..=b'z' as i128).contains(c)
                || (b'A' as i128..=b'Z' as i128).contains(c)
                || (b'0' as i128..=b'9' as i128).contains(c)
                || *c == b'_' as i128,
        )),
        (Value::Int(c), "is_space", []) => {
            Ok(Value::Bool(matches!(*c as u8, b' ' | b'\t' | b'\n' | b'\r')))
        }

        // --- Option/Result ---
        (Value::Variant { path, payload }, "unwrap_or", [d]) => {
            if variant_matches("Some", path) || variant_matches("Ok", path) {
                Ok(payload.first().cloned().unwrap_or_else(|| d.clone()))
            } else {
                Ok(d.clone())
            }
        }
        (Value::Variant { path, .. }, "is_some", []) => Ok(Value::Bool(variant_matches("Some", path))),
        (Value::Variant { path, .. }, "is_none", []) => Ok(Value::Bool(variant_matches("None", path))),

        (r, n, _) => Err(Error::NoField { name: format!("{n} on {}", r.type_name()), at }),
    }
}

// ── the determinism gate (Appendix C.4, E.18) ────────────────────────────────────────

/// The result of running a program twice and comparing.
#[derive(Debug, Clone, PartialEq)]
pub struct DeterminismReport {
    pub runs: usize,
    pub identical: bool,
    /// The output of the first run, for the record.
    pub output: Vec<String>,
    /// The first index at which two runs disagreed, if any.
    pub first_divergence: Option<usize>,
}

/// Run `entry` `runs` times over freshly loaded copies of the same program and check that
/// every run produced byte-identical output.
///
/// This is the runnable form of Appendix C.4's obligation, scoped to what stage 0 can
/// actually check: it establishes *run-to-run* determinism on one target. It does **not**
/// establish cross-target determinism, which needs the same program run on ARM64 and
/// x86-64 and the outputs compared, and which this gate cannot perform from inside one
/// process. §E.19 should say so, and the return type does not pretend otherwise.
pub fn determinism_gate(prog: &Program, entry: &str, runs: usize) -> Result<DeterminismReport, Error> {
    let mut first: Option<Vec<String>> = None;
    let mut divergence = None;
    for _ in 0..runs {
        let mut it = Interp::new();
        it.load(prog);
        it.call(entry, vec![])?;
        match &first {
            None => first = Some(it.output.clone()),
            Some(f) => {
                if *f != it.output {
                    divergence = Some(
                        f.iter().zip(&it.output).position(|(a, b)| a != b).unwrap_or(f.len().min(it.output.len())),
                    );
                }
            }
        }
    }
    let output = first.unwrap_or_default();
    Ok(DeterminismReport { runs, identical: divergence.is_none(), output, first_divergence: divergence })
}

#[cfg(test)]
mod tests {
    use super::*;
    use niles_lang::parser;

    fn run(src: &str, entry: &str) -> Result<Value, Error> {
        let (prog, d) = parser::parse_program(src);
        assert!(!d.has_errors(), "parse errors: {:?}", d.items);
        let mut it = Interp::new();
        it.load(&prog);
        it.call(entry, vec![])
    }

    fn run_out(src: &str, entry: &str) -> Vec<String> {
        let (prog, d) = parser::parse_program(src);
        assert!(!d.has_errors(), "parse errors: {:?}", d.items);
        let mut it = Interp::new();
        it.load(&prog);
        it.call(entry, vec![]).expect("should run");
        it.output
    }

    // ── arithmetic and control flow ─────────────────────────────────────────────────

    #[test]
    fn arithmetic_and_let_work() {
        assert_eq!(run("fn main() -> i64 { let x = 2; let y = 3; x * y + 1 }", "main"), Ok(Value::Int(7)));
    }

    #[test]
    fn if_else_selects_a_branch() {
        let src = "fn f(n: i64) -> i64 { if n > 10 { 1 } else { 2 } }
                   fn main() -> i64 { f(20) + f(1) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(3)));
    }

    #[test]
    fn while_loops_and_accumulates() {
        let src = "fn main() -> i64 {
            let mut i = 0;
            let mut total = 0;
            while i < 10 { total = total + i; i = i + 1; }
            total
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(45)));
    }

    #[test]
    fn break_and_continue_do_what_they_say() {
        let src = "fn main() -> i64 {
            let mut i = 0;
            let mut n = 0;
            loop {
                i = i + 1;
                if i > 100 { break; }
                if i % 2 == 0 { continue; }
                n = n + 1;
            }
            n
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(50)));
    }

    #[test]
    fn recursion_works_which_is_what_a_parser_needs() {
        let src = "fn fact(n: i64) -> i64 { if n <= 1 { 1 } else { n * fact(n - 1) } }
                   fn main() -> i64 { fact(10) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(3_628_800)));
    }

    #[test]
    fn early_return_leaves_the_function_not_the_block() {
        let src = "fn f(n: i64) -> i64 { if n == 0 { return 42; } 7 }
                   fn main() -> i64 { f(0) * 100 + f(1) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(4207)));
    }

    // ── data ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn arrays_index_push_and_iterate() {
        let src = "fn main() -> i64 {
            let mut xs = [1, 2, 3];
            xs = xs.push(4);
            let mut total = 0;
            for x in xs { total = total + x; }
            total * 10 + xs[0]
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(101)));
    }

    #[test]
    fn structs_carry_and_update_fields() {
        let src = "struct P { a: i64, b: i64 }
                   fn main() -> i64 {
                       let mut p = P { a: 1, b: 2 };
                       p.a = 10;
                       p.a + p.b
                   }";
        assert_eq!(run(src, "main"), Ok(Value::Int(12)));
    }

    #[test]
    fn enums_construct_and_match_with_payloads() {
        let src = "enum Tok { Ident(str), Num(i64), Eof }
            fn describe(t: Tok) -> i64 {
                match t {
                    Tok::Num(n) => n,
                    Tok::Ident(s) => len(s),
                    Tok::Eof => 0,
                }
            }
            fn main() -> i64 { describe(Tok::Num(5)) * 100 + describe(Tok::Ident(\"abcd\")) * 10 + describe(Tok::Eof) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(540)));
    }

    #[test]
    fn match_guards_are_honoured_and_order_matters() {
        let src = "fn f(n: i64) -> i64 {
            match n {
                x if x < 0 => 0,
                0 => 1,
                x => x * 2,
            }
        }
        fn main() -> i64 { f(-5) * 100 + f(0) * 10 + f(3) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(16)));
    }

    #[test]
    fn closures_capture_by_value_at_creation() {
        // A snapshot capture, stated in the type docs. If this ever became by-reference,
        // the interpreter would acquire aliasing the language does not have.
        let src = "fn main() -> i64 {
            let mut n = 1;
            let f = |x| x + n;
            n = 100;
            f(5)
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(6)));
    }

    // ── strings and the lexer primitives ─────────────────────────────────────────────

    #[test]
    fn string_primitives_are_enough_to_scan_text() {
        let src = "fn main() -> i64 {
            let s = \"let x = 42\";
            let mut i = 0;
            let mut digits = 0;
            while i < len(s) {
                let c = s.byte_at(i);
                if c.is_digit() { digits = digits + 1; }
                i = i + 1;
            }
            digits
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(2)));
    }

    #[test]
    fn slice_and_concatenation_build_tokens() {
        let src = "fn main() -> str { let s = \"abcdef\"; s.slice(1, 3) + \"|\" + s.slice(4, 6) }";
        assert_eq!(run(src, "main"), Ok(Value::Str(Rc::new("bc|ef".into()))));
    }

    // ── the refusals ────────────────────────────────────────────────────────────────

    #[test]
    fn the_relational_tier_is_refused_by_name_and_never_approximated() {
        // The property that keeps stage 0 honest. If `txn` evaluated as a block, a
        // Niles-written program could appear to conserve money while nothing checked it.
        let src = "fn main() -> i64 { txn { 1 } }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new();
        it.load(&prog);
        match it.call("main", vec![]) {
            Err(Error::NotInSubset { form, .. }) => assert_eq!(form, "txn"),
            other => panic!("txn must be refused by name, got {other:?}"),
        }
    }

    #[test]
    fn money_arithmetic_is_refused_because_the_check_lives_in_the_type_system() {
        let src = "fn main() -> i64 { let a = 10.00 usd; let b = 5.00 eur; a + b }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new();
        it.load(&prog);
        assert!(matches!(
            it.call("main", vec![]),
            Err(Error::NotInSubset { form: "money arithmetic (checked tier only)", .. })
        ));
    }

    #[test]
    fn floats_have_no_spelling_in_the_subset() {
        // Cross-target determinism and money-is-not-a-float are the same commitment.
        let src = "fn main() -> i64 { let x = 1.5; 0 }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new();
        it.load(&prog);
        assert!(matches!(
            it.call("main", vec![]),
            Err(Error::NotInSubset { form: "floating-point literal", .. })
        ));
    }

    #[test]
    fn overflow_is_an_error_and_never_a_wrap() {
        let src = "fn main() -> i64 { let big = 170141183460469231731687303715884105727; big + 1 }";
        assert!(matches!(run(src, "main"), Err(Error::Overflow { op: "+", .. })));
    }

    #[test]
    fn division_by_zero_is_an_error_with_a_span() {
        let src = "fn main() -> i64 { let z = 0; 1 / z }";
        let e = run(src, "main").unwrap_err();
        assert!(matches!(e, Error::DivideByZero { .. }));
        assert!(e.span().is_some(), "a stage-0 failure must be locatable");
    }

    #[test]
    fn an_out_of_bounds_index_names_the_index_and_the_length() {
        let src = "fn main() -> i64 { let xs = [1, 2]; xs[5] }";
        let e = run(src, "main").unwrap_err();
        assert!(matches!(e, Error::IndexOutOfBounds { index: 5, len: 2, .. }));
        // And the span must point at the indexing expression, not at the function. We
        // check the text it covers rather than its offsets, so the assertion survives an
        // edit to the string above.
        let sp = e.span().expect("locatable");
        assert_eq!(&src[sp.start as usize..sp.end as usize], "xs[5]");
    }

    #[test]
    fn an_unbound_name_is_reported_rather_than_defaulted() {
        assert!(matches!(run("fn main() -> i64 { nope }", "main"), Err(Error::Unbound { .. })));
    }

    #[test]
    fn a_non_boolean_condition_is_rejected_rather_than_coerced() {
        let src = "fn main() -> i64 { if 1 { 2 } else { 3 } }";
        assert!(matches!(run(src, "main"), Err(Error::TypeMismatch { .. })));
    }

    #[test]
    fn an_infinite_loop_runs_out_of_fuel_instead_of_hanging_the_suite() {
        let src = "fn main() -> i64 { loop { } }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new().with_fuel(1000);
        it.load(&prog);
        assert_eq!(it.call("main", vec![]), Err(Error::OutOfFuel));
    }

    // ── determinism ──────────────────────────────────────────────────────────────────

    #[test]
    fn evaluation_order_is_left_to_right_and_observable() {
        let src = "fn a() -> i64 { print(\"a\"); 1 }
                   fn b() -> i64 { print(\"b\"); 2 }
                   fn main() -> i64 { a() + b() }";
        assert_eq!(run_out(src, "main"), vec!["a", "b"]);
    }

    #[test]
    fn the_short_circuit_really_short_circuits() {
        // Not a nicety: the Niles lexer's bounds checks depend on it.
        let src = "fn t() -> bool { print(\"evaluated\"); true }
                   fn main() -> i64 { if false && t() { 1 } else { 0 } }";
        assert_eq!(run_out(src, "main"), Vec::<String>::new());
    }

    #[test]
    fn the_determinism_gate_passes_on_a_deterministic_program() {
        let src = "fn main() -> i64 {
            let mut i = 0;
            while i < 5 { print(i.to_string()); i = i + 1; }
            0
        }";
        let (prog, _) = parser::parse_program(src);
        let r = determinism_gate(&prog, "main", 8).unwrap();
        assert!(r.identical);
        assert_eq!(r.runs, 8);
        assert_eq!(r.output, vec!["0", "1", "2", "3", "4"]);
        assert_eq!(r.first_divergence, None);
    }

    #[test]
    fn scope_iteration_is_by_name_so_a_hash_seed_cannot_leak_into_output() {
        // The concrete determinism hazard this design closes. Were `Env` a `HashMap`,
        // nothing in the language would change, and yet a struct's rendering could
        // reorder between processes.
        let src = "struct S { zeta: i64, alpha: i64, mu: i64 }
                   fn main() -> i64 { print(S { zeta: 1, alpha: 2, mu: 3 }.to_string()); 0 }";
        let out = run_out(src, "main");
        assert_eq!(out, vec!["S { alpha: 2, mu: 3, zeta: 1 }"]);
    }

    #[test]
    fn the_gate_reports_the_number_of_runs_it_actually_did() {
        let src = "fn main() -> i64 { print(\"x\"); 0 }";
        let (prog, _) = parser::parse_program(src);
        assert_eq!(determinism_gate(&prog, "main", 3).unwrap().runs, 3);
    }

    #[test]
    fn a_failure_inside_the_gate_surfaces_rather_than_being_counted_as_identical() {
        let src = "fn main() -> i64 { let z = 0; 1 / z }";
        let (prog, _) = parser::parse_program(src);
        assert!(determinism_gate(&prog, "main", 2).is_err());
    }
}
