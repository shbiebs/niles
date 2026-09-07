//! `nilesc` — the Niles compiler driver.
//!
//! Six subcommands, each of which exposes one stage of the pipeline, because a compiler
//! whose intermediate results cannot be inspected is a compiler nobody can review:
//!
//! ```text
//! nilesc check   FILE     lex -> parse -> resolve -> typecheck -> lower; report diagnostics
//! nilesc parse   FILE     the item structure, for debugging the parser
//! nilesc explain FILE     the lowered circuit, with keys, anchors, rungs and contracts
//! nilesc upquery FILE V   the reconstruction path for view V, and whether it is anchored
//! nilesc verify  FILE     run the IR verifier over the lowered circuit
//! nilesc effects FILE     the inferred effect row of every function
//! nilesc postings FILE FN the legs a function declares, in source order
//! nilesc run     FILE FN  *execute* FN and print the canonical bytes it posts
//! nilesc plan    FILE     the materialization plan the optimizer would choose
//! nilesc time    FILE     what each stage of the pipeline costs, and on what
//! ```
//!
//! `postings` and `run` are the two halves of conformance and the difference between them is
//! the whole of finding F-25. `postings` reports the shape a function *declares* — a rendering
//! — and two implementations can agree on a rendering while producing different documents.
//! `run` executes the function over an in-memory ledger and prints the canonical encoding of
//! the set it sealed, which is the artefact a hash chain would cover. Agreement on that is
//! agreement.
//!
//! Exit code 0 iff no errors. Warnings do not fail the build; that is what makes the
//! missing-contract warning usable during migration rather than a wall.

use niles_interp::ledger;
use niles_interp::{Error as InterpError, Interp, Value};
use niles_ir::{upquery_path, verify};
use niles_lang::{lower, parser, resolve, typecheck};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("{}", USAGE);
        return ExitCode::from(2);
    }
    let (cmd, path) = (args[1].as_str(), args[2].as_str());
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("nilesc: cannot read {path}: {e}");
            return ExitCode::from(2);
        }
    };

    // ---- front end. Always runs in full: a later stage's diagnostics are often the
    // useful ones, and stopping at the first error would hide them.
    let (prog, mut diags) = parser::parse_program(&src);
    // Epoch 0: the schema as of its own declarations. The server path passes the live
    // visibility frontier here instead, and it is the same code.
    let (cat, rdiags) = resolve::resolve_program(&prog, 0);
    diags.extend(rdiags);
    let (report, tdiags) = typecheck::check_program(&prog, &cat);
    diags.extend(tdiags);

    // Mutable because lowering runs later and its diagnostics are diagnostics: a driver
    // that printed an error and exited 0 told every script that called it the file was
    // fine. `nilesc explain` on a view with an unlowerable `where` clause did exactly
    // that, and the `make schema` gate that ran it was green for as long as it existed.
    let mut failed = diags.has_errors();
    if !diags.items.is_empty() {
        eprint!("{}", diags.render(&src, path));
    }

    match cmd {
        "check" => {
            // **`check` lowers.** It used to stop after typechecking, so a view whose
            // `group by` named nothing, whose projection dropped a column, or whose `order by`
            // key did not resolve was reported `ok` by the command a `make` target and a CI
            // job run — while `explain` on the same file printed three errors. A checker that
            // is green on a program the compiler refuses is worse than no checker: it is a
            // green light nobody can act on. Lowering is 2.6 µs on a view; the cost of not
            // running it was a whole class of refusal nothing surfaced.
            let (_lowered, ldiags) = lower::lower_program(&prog, &cat);
            let lowering_errors = ldiags.error_count();
            if !ldiags.items.is_empty() {
                eprint!("{}", ldiags.render(&src, path));
            }
            failed |= ldiags.has_errors();
            if failed {
                eprintln!("nilesc: {} error(s)", diags.error_count() + lowering_errors);
            } else {
                println!(
                    "ok: {} relation(s), {} view(s), {} function(s); \
                     {} conservation obligation(s) proved statically, {} discharged to the runtime",
                    cat.relations.len(),
                    cat.views.len(),
                    cat.functions.len(),
                    report.conservation_proved,
                    report.runtime_obligations
                );
            }
        }
        // **The cheap half of H-S6, which had no measurement of any kind.**
        //
        // The hypothesis is that static checkability subsumes runtime policing. Its
        // residual-violation half is addressed by the mutant corpus; the other half asks
        // what *fraction* of a real program's conservation obligations the checker
        // discharges, and nothing counted. One line of machine-readable output, so a corpus
        // can be summed by a script rather than by reading prose.
        //
        // It reports on a file that does not check, too, with `ok=0`: a corpus that silently
        // dropped its failures would report a flattering fraction over the files that
        // happened to compile.
        "report-obligations" => {
            println!(
                "file={path} ok={} static={} runtime={}",
                u8::from(!failed),
                report.conservation_proved,
                report.runtime_obligations
            );
        }
        // **What the compiler costs, which nothing had ever asked.**
        //
        // H-S6 names *compile time* as a dependent variable (`thesis/01`, §1.5) and E14
        // measures the other two. Eight cycles of work on this language and the only figure
        // anyone had for its compiler was the 2.6 µs in `check`'s comment above, measured
        // once, by hand, and never again.
        //
        // Two instruments, because neither alone is honest. **Wall clock per phase** says
        // where the time goes — it is the only way to separate six stages inside one process
        // — but it is host-shaped and noisy, so it is reported as a median over repeats with
        // its spread beside it. **Instruction counts** are deterministic and comparable
        // across runs and hosts of one architecture, and `callgrind` can only count a whole
        // process: run `valgrind --tool=callgrind nilesc check FILE` for that number, which
        // is what `experiments e26` does over a corpus. A phase table and a total that
        // cannot be added together are still the two facts worth having.
        "time" => {
            let repeat: u32 = args
                .get(3)
                .and_then(|s| s.strip_prefix("--repeat="))
                .and_then(|s| s.parse().ok())
                .unwrap_or(200);
            let med = |mut v: Vec<u128>| -> u128 {
                v.sort_unstable();
                v[v.len() / 2]
            };
            let mut lex_ns = Vec::new();
            let mut parse_ns = Vec::new();
            let mut resolve_ns = Vec::new();
            let mut check_ns = Vec::new();
            let mut lower_ns = Vec::new();
            let mut verify_ns = Vec::new();
            for _ in 0..repeat.max(1) {
                let t = std::time::Instant::now();
                let lexed = niles_lang::lexer::lex(&src);
                lex_ns.push(t.elapsed().as_nanos());
                std::hint::black_box(&lexed);

                let t = std::time::Instant::now();
                let (prog, _) = parser::parse_program(&src);
                parse_ns.push(t.elapsed().as_nanos());

                let t = std::time::Instant::now();
                let (cat, _) = resolve::resolve_program(&prog, 0);
                resolve_ns.push(t.elapsed().as_nanos());

                let t = std::time::Instant::now();
                let checked = typecheck::check_program(&prog, &cat);
                check_ns.push(t.elapsed().as_nanos());
                std::hint::black_box(&checked);

                let t = std::time::Instant::now();
                let (lowered, _) = lower::lower_program(&prog, &cat);
                lower_ns.push(t.elapsed().as_nanos());

                let t = std::time::Instant::now();
                let v = verify::verify(&lowered.circuit);
                verify_ns.push(t.elapsed().as_nanos());
                std::hint::black_box(&v);
            }
            // Parsing includes lexing — `parse_program` lexes for itself — so the parser's
            // own cost is the difference. Reported that way rather than as two numbers that
            // do not add up to the whole.
            let (l, p_, r, c, lo, v) = (
                med(lex_ns),
                med(parse_ns),
                med(resolve_ns),
                med(check_ns),
                med(lower_ns),
                med(verify_ns),
            );
            let total = p_ + r + c + lo + v;
            let items = counts(&prog);
            println!("file={path} repeats={repeat}");
            println!("stage,median_ns,share");
            let row = |name: &str, ns: u128| {
                println!(
                    "{name},{ns},{:.1}%",
                    if total == 0 {
                        0.0
                    } else {
                        100.0 * ns as f64 / total as f64
                    }
                );
            };
            row("lex", l);
            row("parse_incl_lex", p_);
            row("resolve", r);
            row("typecheck", c);
            row("lower", lo);
            row("verify_ir", v);
            println!("total,{total},100.0%");
            println!("items,{}", items.total());
            println!(
                "by_class,view={},function={},relation={},currency={},other={}",
                items.views, items.functions, items.relations, items.currencies, items.other
            );
            if items.total() > 0 {
                println!("per_item_ns,{}", total / items.total() as u128);
            }
            println!(
                "note,a wall-clock figure is this host's. For a deterministic count run: \
                 valgrind --tool=callgrind --callgrind-out-file=/dev/null nilesc check {path}"
            );
        }
        "parse" => {
            for item in &prog.items {
                println!("{}", describe_item(item));
            }
        }
        "effects" => {
            let mut names: Vec<&String> = report.inferred_effects.keys().collect();
            names.sort();
            for n in names {
                println!("  fn {n:<24} ! {}", report.inferred_effects[n]);
            }
            let mut views: Vec<&String> = report.view_rungs.keys().collect();
            views.sort();
            for v in views {
                // The sources by name, not only the rung. Two views at one rung read
                // differently, and "does `available_balance` read `encumbrances`" is a
                // question the rung alone cannot answer.
                let from = report
                    .view_sources
                    .get(v)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.iter().cloned().collect::<Vec<_>>().join(", "))
                    .unwrap_or_else(|| "nothing".into());
                match report.view_rungs[v] {
                    Some(r) => {
                        println!("  view {v:<22} reads {from} — no stricter than {r}")
                    }
                    None => println!("  view {v:<22} reads {from}"),
                }
            }
        }
        // `nilesc postings FILE [FN]` — the legs a function declares, in source order.
        //
        // The conformance surface. GBS has two implementations of every product — Rust in
        // `gbs-products`, Niles in `gbs.niles` — and thesis §6.9 argues against exactly that
        // seam. This is what lets the two be compared without either repository depending on
        // the other's types: a documented text format, produced by the compiler that owns the
        // schema, checked against a fixture produced by the Rust that owns the product.
        //
        // It reports the *declared shape*, not an evaluation. What that does and does not
        // cover is in `niles_lang::postings`, and the output carries its own caveats —
        // `branched`, `fx` — so a fixture cannot rely on them silently.
        "postings" => {
            use niles_lang::postings;
            match args.get(3) {
                Some(function) => match postings::shape_of(&prog, function) {
                    Some(shape) => print!("{}", shape.render()),
                    None => {
                        // Distinguished from a function with no legs, which prints a header
                        // and nothing else. A conformance test that could not tell the two
                        // apart would pass against a typo in the function name.
                        eprintln!("nilesc: {path} declares no function `{function}`");
                        return ExitCode::from(2);
                    }
                },
                None => {
                    for shape in postings::all_shapes(&prog) {
                        print!("{}", shape.render());
                        println!();
                    }
                }
            }
        }
        "explain" | "verify" | "upquery" | "plan" => {
            let (lowered, ldiags) = lower::lower_program(&prog, &cat);
            if !ldiags.items.is_empty() {
                eprint!("{}", ldiags.render(&src, path));
            }
            failed |= ldiags.has_errors();
            match cmd {
                "explain" => {
                    println!("circuit ({} nodes):", lowered.circuit.nodes.len());
                    print!("{}", lowered.circuit.explain());
                    // **What the engine would do with each view, not only what it lowered
                    // to.** A circuit dump says what the query means; it says nothing about
                    // whether answering it reads one maintained entry or materialises the
                    // whole base, and those differ by three orders of magnitude. The class
                    // comes from `nilestream_server::rev_engine::serve_path`, which is the
                    // function the engine itself branches on.
                    let mut named: Vec<&String> = lowered.circuit.outputs.keys().collect();
                    named.sort();
                    if !named.is_empty() {
                        println!("\nserve path:");
                        for out in named {
                            let p =
                                nilestream_server::rev_engine::serve_path(&lowered.circuit, out);
                            println!("  {out}: {} — {}", p.as_str(), p.describe());
                        }
                    }
                }
                "plan" => {
                    // The observed load a running engine would supply. Offline, these are
                    // stated defaults rather than measurements, and the output says so —
                    // a plan presented as if it were measured would be the worst of both.
                    let obs = nilestream_optimizer::offline::Observed {
                        read_rate: 100.0,
                        write_rate: 50.0,
                        working_set: 10_000.0,
                        reconstruction_rows: 9.0,
                        residency_price: 0.002,
                    };
                    println!("plan (offline: load figures are defaults, not measurements)\n");
                    for p in nilestream_optimizer::offline::plan(&lowered.circuit, obs) {
                        println!("  {}", p.explanation);
                        for (m, why) in &p.excluded {
                            println!("      ruled out {m:?}: {why}");
                        }
                        println!();
                    }
                }
                "verify" => {
                    let r = verify::verify(&lowered.circuit);
                    print!("{}", r.render());
                    if !r.is_ok() {
                        return ExitCode::from(1);
                    }
                }
                _ => {
                    let Some(view) = args.get(3) else {
                        eprintln!("nilesc upquery FILE VIEW");
                        return ExitCode::from(2);
                    };
                    let Some(id) = lowered.circuit.outputs.get(view.as_str()) else {
                        eprintln!("nilesc: no view `{view}`");
                        return ExitCode::from(2);
                    };
                    match upquery_path::derive(&lowered.circuit, *id, 0) {
                        Ok(p) => {
                            println!("{}", p.render());
                            for h in &p.hops {
                                println!(
                                    "    {:<12} key={:?} reads={:?}",
                                    h.op_name, h.key, h.base_columns
                                );
                            }
                        }
                        Err(e) => {
                            println!("no upquery path: {}", e.explain());
                            return ExitCode::from(1);
                        }
                    }
                }
            }
        }
        // Execute a function and print the canonical bytes of the set it posted.
        //
        // Exit 2 on `NotInSubset`, naming the construct, because the caller's next move
        // depends on which it was: a language gap is a `BLOCKED-T-18-<function>` in a build
        // log, and a wrong answer is a defect. Exit 1 on any other refusal.
        "run" => {
            if failed {
                eprintln!("nilesc: {path} does not check; refusing to run it");
                return ExitCode::from(1);
            }
            let Some(function) = args.get(3) else {
                eprintln!("nilesc: `run` needs a function name\n{USAGE}");
                return ExitCode::from(2);
            };
            return run_function(&prog, path, function, &args);
        }
        other => {
            eprintln!("nilesc: unknown command `{other}`\n{USAGE}");
            return ExitCode::from(2);
        }
    }

    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

const USAGE: &str = "\
nilesc — the Niles compiler

USAGE:
    nilesc check   FILE        lex, parse, resolve, typecheck
    nilesc parse   FILE        item structure
    nilesc explain FILE        the lowered circuit
    nilesc upquery FILE VIEW   the reconstruction path for VIEW
    nilesc verify  FILE        run the IR verifier
    nilesc effects FILE        inferred effect rows
    nilesc report-obligations FILE
                               one machine-readable line: how many conservation
                               obligations were proved statically and how many were
                               discharged to the runtime seal (H-S6)
    nilesc postings FILE [FN]  the legs a function declares, for conformance
    nilesc run     FILE FN     execute FN; print the canonical encoding of its posting set
                               [--ledger FIXTURE] [--args FIXTURE]
";

/// What a program declares, by class — the axis `nilesc time` reports cost against.
///
/// A "statement class" in this language is an *item*: a view is compiled once and served
/// many times, which is the whole shape of the system, so per-statement cost is per-item
/// cost and not per-query cost.
#[derive(Default)]
struct ItemCounts {
    views: usize,
    functions: usize,
    relations: usize,
    currencies: usize,
    other: usize,
}

impl ItemCounts {
    fn total(&self) -> usize {
        self.views + self.functions + self.relations + self.currencies + self.other
    }
}

fn counts(p: &niles_lang::ast::Program) -> ItemCounts {
    use niles_lang::ast::{Item, SchemaItem};
    let mut c = ItemCounts::default();
    for i in &p.items {
        match i {
            Item::Schema(s) => {
                for si in &s.items {
                    match si {
                        SchemaItem::Currency(_) => c.currencies += 1,
                        SchemaItem::Table(_) | SchemaItem::Base(_) => c.relations += 1,
                        SchemaItem::View(_) => c.views += 1,
                        _ => c.other += 1,
                    }
                }
            }
            Item::View(_) => c.views += 1,
            Item::Fn(_) => c.functions += 1,
            _ => c.other += 1,
        }
    }
    c
}

fn describe_item(i: &niles_lang::ast::Item) -> String {
    use niles_lang::ast::{Item, SchemaItem};
    match i {
        Item::Schema(s) => {
            let mut out = format!("schema {} ({} items)", s.name.text, s.items.len());
            for si in &s.items {
                out.push_str(&match si {
                    SchemaItem::Currency(c) => {
                        format!("\n  currency {} scale {}", c.name.text, c.scale)
                    }
                    SchemaItem::Table(r) | SchemaItem::Base(r) => {
                        format!(
                            "\n  {:?} {} ({} cols, {} rules)",
                            r.kind,
                            r.name.text,
                            r.fields.len(),
                            r.rules.len()
                        )
                    }
                    SchemaItem::View(v) => format!(
                        "\n  view {} ({})",
                        v.name.text,
                        v.contract
                            .as_ref()
                            .map_or("no contract".into(), |c| format!(
                                "{} contract entries",
                                c.entries.len()
                            ))
                    ),
                    SchemaItem::Index(ix) => {
                        format!(
                            "\n  index {} on {} {}",
                            ix.name.text,
                            ix.on.text,
                            if ix.anchor { "(anchor)" } else { "" }
                        )
                    }
                    SchemaItem::Error(_) => "\n  <parse error>".into(),
                });
            }
            out
        }
        Item::Fn(f) => format!(
            "fn {}({}) {}",
            f.name.text,
            f.params.len(),
            f.effects
                .as_ref()
                .map_or(String::new(), |e| format!("! {} effects", e.effects.len()))
        ),
        Item::View(v) => format!("view {}", v.name.text),
        Item::Struct(s) => format!("struct {}", s.name.text),
        Item::Enum(e) => format!("enum {}", e.name.text),
        Item::Impl(i) => format!("impl ({} fns)", i.items.len()),
        Item::Trait(t) => format!("trait {}", t.name.text),
        Item::Error(_) => "<parse error>".into(),
        _ => "item".into(),
    }
}

// ── `nilesc run`: executing conformance ─────────────────────────────────────────────

/// Execute `function` over an in-memory ledger and print the canonical encoding of the
/// posting set it sealed, as lower-case hex, on one line.
///
/// # Why one line of hex on stdout
///
/// The consumer is a test in another repository that has just built the same transaction with
/// a different implementation and encoded it with a different encoder. What it needs is a
/// value it can compare for equality and print in a failure message. Hex on stdout is a value
/// a shell, a test harness and a person can each handle; a binary stream is none of those, and
/// a structured format would be a third thing to keep in agreement.
///
/// # Exit codes, and why `NotInSubset` gets its own
///
/// * `0` — the function ran and its set is on stdout.
/// * `1` — the function ran and was refused (the set does not conserve), or the file does not
///   check.
/// * `2` — the function could not be run: no such function, a bad fixture, or a construct
///   outside the interpretable subset. The last is printed as `NotInSubset: <form>` so a
///   caller can distinguish "this language cannot express the run" from "this run was wrong" —
///   the first is a gap to record, the second is a defect to fix, and a single exit code for
///   both would let a gap be reported as a failure and a failure as a gap.
fn run_function(
    prog: &niles_lang::ast::Program,
    path: &str,
    function: &str,
    args: &[String],
) -> ExitCode {
    let mut it = Interp::new();
    it.load(prog);
    if !it.function_names().iter().any(|n| n == function) {
        // Distinguished from a function that posts nothing, which succeeds and prints the
        // encoding of an empty set. A test that could not tell the two apart would pass
        // against a typo in the function name.
        eprintln!("nilesc: {path} declares no function `{function}`");
        return ExitCode::from(2);
    }

    if let Some(f) = flag(args, "--ledger") {
        match std::fs::read_to_string(&f) {
            Ok(text) => {
                if let Err(e) = seed_ledger(&mut it, &text) {
                    eprintln!("nilesc: {f}: {e}");
                    return ExitCode::from(2);
                }
            }
            Err(e) => {
                eprintln!("nilesc: cannot read {f}: {e}");
                return ExitCode::from(2);
            }
        }
    }

    let call_args = match flag(args, "--args") {
        None => Vec::new(),
        Some(f) => match std::fs::read_to_string(&f) {
            Ok(text) => match parse_args(&text) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("nilesc: {f}: {e}");
                    return ExitCode::from(2);
                }
            },
            Err(e) => {
                eprintln!("nilesc: cannot read {f}: {e}");
                return ExitCode::from(2);
            }
        },
    };

    match it.call(function, call_args) {
        Ok(_) => {}
        Err(InterpError::NotInSubset { form, .. }) => {
            eprintln!("nilesc: NotInSubset: {form}");
            return ExitCode::from(2);
        }
        Err(e) => {
            eprintln!("nilesc: {function}: {}", e.message());
            return ExitCode::from(1);
        }
    }

    let sealed = &it.ledger.sealed;
    if sealed.len() != 1 {
        // A conformance fixture compares *one* posting set. A function that sealed none or
        // several is not a failure of the function — `main` legitimately seals several — but
        // it is a question this command cannot answer, and answering it with the first or the
        // last would be a silent choice.
        eprintln!(
            "nilesc: {function} sealed {} posting sets; `run` compares one",
            sealed.len()
        );
        return ExitCode::from(2);
    }
    let set = &sealed[0];
    println!("{}", ledger::hex(&ledger::encode(&set.txn, &set.legs)));
    if it.ignored_windows() > 0 {
        // Not a warning that changes the answer, and said out loud anyway: the window is part
        // of what the schema declares and nothing here honoured it.
        eprintln!(
            "nilesc: note: {} `idem` window(s) were not honoured; this interpreter has no \
             idempotency store, and the identity is what the encoding carries",
            it.ignored_windows()
        );
    }
    ExitCode::SUCCESS
}

/// The value after a `--flag`.
fn flag(args: &[String], name: &str) -> Option<String> {
    let i = args.iter().position(|a| a == name)?;
    args.get(i + 1).cloned()
}

/// Seed the ledger from a fixture.
///
/// The format is one directive per line; `#` starts a comment and blank lines are ignored:
///
/// ```text
/// account <name> <currency> <scale>
/// open    <name> <currency> <minor>
/// ```
///
/// Deliberately not a general-purpose format. It exists so a run can start from a stated
/// world rather than from nothing, and every field it carries is a field a posting set needs.
fn seed_ledger(it: &mut Interp, text: &str) -> Result<(), String> {
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        let at = n + 1;
        match f.as_slice() {
            ["account", name, currency, scale] => {
                let scale: u32 = scale
                    .parse()
                    .map_err(|_| format!("line {at}: `{scale}` is not a scale"))?;
                if !it.ledger.declare(name, currency, scale) {
                    return Err(format!(
                        "line {at}: {name} was already declared with a different currency or scale"
                    ));
                }
            }
            ["open", name, currency, minor] => {
                let minor: i128 = minor
                    .parse()
                    .map_err(|_| format!("line {at}: `{minor}` is not an amount in minor units"))?;
                it.ledger.open_balance(name, currency, minor);
            }
            _ => return Err(format!("line {at}: `{line}` is not a directive")),
        }
    }
    Ok(())
}

/// Parse the argument fixture into interpreter values.
///
/// One argument per line, tagged with its type, in the order the function declares them:
///
/// ```text
/// acct  loan.fac-1.agent      # an Id<Account>: carried as its name
/// money -25000 USD 2          # signed minor units, currency, scale
/// int   7
/// str   hello
/// ```
///
/// Tagged rather than inferred. An untagged `100` could be an integer or an amount, and an
/// interpreter that guessed would produce a posting set that balanced by coincidence.
fn parse_args(text: &str) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        let at = n + 1;
        let v = match f.as_slice() {
            ["acct", name] | ["str", name] => Value::Str(std::rc::Rc::new((*name).to_string())),
            ["int", i] => Value::Int(
                i.parse()
                    .map_err(|_| format!("line {at}: `{i}` is not an integer"))?,
            ),
            ["money", minor, currency, scale] => Value::Money {
                minor: minor
                    .parse()
                    .map_err(|_| format!("line {at}: `{minor}` is not minor units"))?,
                scale: scale
                    .parse()
                    .map_err(|_| format!("line {at}: `{scale}` is not a scale"))?,
                currency: (*currency).to_string(),
            },
            _ => return Err(format!("line {at}: `{line}` is not an argument")),
        };
        out.push(v);
    }
    Ok(out)
}
