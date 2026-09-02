//! `nilesc` — the Niles compiler driver.
//!
//! Six subcommands, each of which exposes one stage of the pipeline, because a compiler
//! whose intermediate results cannot be inspected is a compiler nobody can review:
//!
//! ```text
//! nilesc check   FILE     lex -> parse -> resolve -> typecheck; report diagnostics
//! nilesc parse   FILE     the item structure, for debugging the parser
//! nilesc explain FILE     the lowered circuit, with keys, anchors, rungs and contracts
//! nilesc upquery FILE V   the reconstruction path for view V, and whether it is anchored
//! nilesc verify  FILE     run the IR verifier over the lowered circuit
//! nilesc effects FILE     the inferred effect row of every function
//! nilesc postings FILE FN the legs a function declares, in source order
//! nilesc plan    FILE     the materialization plan the optimizer would choose
//! ```
//!
//! Exit code 0 iff no errors. Warnings do not fail the build; that is what makes the
//! missing-contract warning usable during migration rather than a wall.

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

    let failed = diags.has_errors();
    if !diags.items.is_empty() {
        eprint!("{}", diags.render(&src, path));
    }

    match cmd {
        "check" => {
            if failed {
                eprintln!("nilesc: {} error(s)", diags.error_count());
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
                match report.view_rungs[v] {
                    Some(r) => println!("  view {v:<22} reads no stricter than {r}"),
                    None => println!("  view {v:<22} reads nothing"),
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
            match cmd {
                "explain" => {
                    println!("circuit ({} nodes):", lowered.circuit.nodes.len());
                    print!("{}", lowered.circuit.explain());
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
    nilesc postings FILE [FN]  the legs a function declares, for conformance
";

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
