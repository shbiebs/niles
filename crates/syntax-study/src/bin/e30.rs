//! `e30` — the syntax study's driver (design `docs/study/E30-syntax-design.md`).
//!
//! ```text
//! e30 gen                      write the dataset's fixture files into corpus/data/
//! e30 run [TASK] [SURFACE]     check and execute each program, compare with the oracle
//! ```

use std::path::{Path, PathBuf};
use syntax_study::answer::Answer;
use syntax_study::run::{verdict, Ctx, Outcome};
use syntax_study::{data, oracle, tasks};

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

fn work() -> PathBuf {
    std::env::var_os("E30_WORK")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("e30-work"))
}

fn results() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../results/E30-syntax")
}

pub fn program_path(task: &str, surface: &str) -> PathBuf {
    corpus().join(tasks::program_file(task, surface))
}

fn describe(o: &Outcome, want: &Answer) -> String {
    match o {
        Outcome::Answer(a) => match want.diff(a) {
            None => "CORRECT".into(),
            Some(d) => format!("WRONG: {d}"),
        },
        Outcome::Static(c) => format!("REFUSED {}", c.join(" ")),
        Outcome::Runtime(e) => format!("RAISED {e}"),
        Outcome::Unexecuted(e) => format!("UNEXECUTED {e}"),
        Outcome::Blocked(e) => format!("BLOCKED {e}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let d = data::generate(1);
    match args.first().map(String::as_str) {
        Some("gen") => {
            let dir = corpus().join("data");
            std::fs::create_dir_all(&dir).expect("mkdir");
            for (name, text) in d.files() {
                std::fs::write(dir.join(name), text).expect("write");
            }
            eprintln!("wrote {}", dir.display());
        }
        Some("run") => {
            let only_task = args.get(1).filter(|t| t.as_str() != "-").cloned();
            let only_surface = args.get(2).cloned();
            let mut ctx = Ctx::new(d.clone(), work());
            for t in oracle::TASKS {
                if only_task.as_deref().is_some_and(|x| x != *t) {
                    continue;
                }
                let want = oracle::expected(t, &d);
                for (s, _) in tasks::SURFACES {
                    if only_surface.as_deref().is_some_and(|x| x != *s) || !tasks::in_scope(s, t) {
                        continue;
                    }
                    let p = program_path(t, s);
                    let Ok(src) = std::fs::read_to_string(&p) else {
                        let reason = std::fs::read_to_string(format!("{}.none", p.display()));
                        match reason {
                            Ok(r) => println!(
                                "{t}\t{s}\tNOT EXPRESSIBLE {}",
                                r.lines().next().unwrap_or("")
                            ),
                            Err(_) => println!("{t}\t{s}\t(no program)"),
                        }
                        continue;
                    };
                    let o = verdict(&mut ctx, t, s, &src, &want);
                    println!("{t}\t{s}\t{}", describe(&o, &want));
                }
            }
        }
        Some("keywords") => {
            let mut pg = syntax_study::pg::connect().unwrap_or_else(|e| panic!("{e:?}"));
            let prqlc = syntax_study::ext::prqlc().expect("prqlc");
            let dir = corpus().join("keywords");
            std::fs::create_dir_all(&dir).expect("mkdir");
            for (name, text) in syntax_study::keywords::generate(&mut pg, &prqlc).expect("keywords")
            {
                std::fs::write(dir.join(name), text).expect("write");
            }
            eprintln!("wrote {}", dir.display());
        }
        Some("sites") => {
            // The mutants each program has, without running any: for reviewing the patterns.
            let only_task = args.get(1).filter(|t| t.as_str() != "-").cloned();
            let only_surface = args.get(2).cloned();
            for t in oracle::TASKS {
                if only_task.as_deref().is_some_and(|x| x != *t) {
                    continue;
                }
                for (s, _) in tasks::SURFACES {
                    if only_surface.as_deref().is_some_and(|x| x != *s) || !tasks::in_scope(s, t) {
                        continue;
                    }
                    let Ok(src) = std::fs::read_to_string(program_path(t, s)) else {
                        continue;
                    };
                    for m in syntax_study::mutate::mutants(&src, s) {
                        println!("{t}\t{s}\t{}#{}\tline {}\t{}", m.op, m.site, m.line, m.what);
                    }
                }
            }
        }
        Some("mutate") => {
            // Every mutant of every expressible program, checked, executed and classified
            // (design §6): static, runtime, silent, equivalent, or unexecuted (A1).
            let only_task = args.get(1).filter(|t| t.as_str() != "-").cloned();
            let only_surface = args.get(2).cloned();
            let mut ctx = Ctx::new(d.clone(), work());
            let mut rows =
                vec!["task\tsurface\top\tsite\tline\tmutation\tclass\tdetail".to_string()];
            for t in oracle::TASKS {
                if only_task.as_deref().is_some_and(|x| x != *t) {
                    continue;
                }
                let want = oracle::expected(t, &d);
                for (s, _) in tasks::SURFACES {
                    if only_surface.as_deref().is_some_and(|x| x != *s) || !tasks::in_scope(s, t) {
                        continue;
                    }
                    let Ok(src) = std::fs::read_to_string(program_path(t, s)) else {
                        continue;
                    };
                    let base = verdict(&mut ctx, t, s, &src, &want);
                    let base_answer = match &base {
                        Outcome::Answer(a) if want.diff(a).is_none() => Some(a.clone()),
                        Outcome::Unexecuted(_) => None,
                        other => panic!(
                            "{t} {s}: the unmutated program is not correct: {}",
                            describe(other, &want)
                        ),
                    };
                    for m in syntax_study::mutate::mutants(&src, s) {
                        let (class, detail) = syntax_study::run::classify(
                            &mut ctx,
                            t,
                            s,
                            &m.text,
                            base_answer.as_ref(),
                            &want,
                        );
                        eprintln!("{t} {s} {}#{} {class}", m.op, m.site);
                        rows.push(format!(
                            "{t}\t{s}\t{}\t{}\t{}\t{}\t{class}\t{}",
                            m.op,
                            m.site,
                            m.line,
                            m.what.replace(['\t', '\n'], " "),
                            detail.replace(['\t', '\n'], " ")
                        ));
                    }
                }
            }
            let dir = results();
            std::fs::create_dir_all(&dir).expect("mkdir");
            let name = if only_task.is_some() || only_surface.is_some() {
                "mutants-partial.tsv"
            } else {
                "mutants.tsv"
            };
            std::fs::write(dir.join(name), rows.join("\n") + "\n").expect("write");
            eprintln!("wrote {}", dir.join(name).display());
        }
        Some("measure") => {
            // Expressiveness per program (design §6): verdict, tokens, constructs.
            let mut ctx = Ctx::new(d.clone(), work());
            let kwdir = corpus().join("keywords");
            let mut rows = vec![
                "task\tsurface\tverdict\tdetail\tlines\ttokens\tkeywords\toperators".to_string(),
            ];
            for t in oracle::TASKS {
                let want = oracle::expected(t, &d);
                for (s, _) in tasks::SURFACES {
                    if !tasks::in_scope(s, t) {
                        continue;
                    }
                    let p = program_path(t, s);
                    let Ok(src) = std::fs::read_to_string(&p) else {
                        let why = std::fs::read_to_string(format!("{}.none", p.display()))
                            .unwrap_or_else(|_| panic!("{t} {s}: neither a program nor a reason"));
                        rows.push(format!(
                            "{t}\t{s}\tnot expressible\t{}\t\t\t\t",
                            why.lines().next().unwrap_or("").replace('\t', " ")
                        ));
                        continue;
                    };
                    let o = verdict(&mut ctx, t, s, &src, &want);
                    let (v, detail) = match &o {
                        Outcome::Answer(a) => match want.diff(a) {
                            None => ("correct", String::new()),
                            Some(x) => ("wrong", x),
                        },
                        Outcome::Unexecuted(e) => ("unexecuted", e.clone()),
                        Outcome::Static(c) => ("refused", c.join(" ")),
                        Outcome::Runtime(e) => ("raised", e.clone()),
                        Outcome::Blocked(e) => panic!("{t} {s}: blocked: {e}"),
                    };
                    let toks = syntax_study::tokens::tokenize(&src, s);
                    let kw = syntax_study::keywords::load(&kwdir, s);
                    let (k, o) = syntax_study::tokens::constructs(&toks, s, &kw);
                    let lines = src.lines().filter(|l| !l.trim().is_empty()).count();
                    rows.push(format!(
                        "{t}\t{s}\t{v}\t{}\t{lines}\t{}\t{k}\t{o}",
                        detail.replace(['\t', '\n'], " "),
                        toks.len()
                    ));
                }
            }
            let mut schema = vec!["surface\tfile\tlines\ttokens".to_string()];
            for (s, f, lex) in [
                ("SQL", "schema.sql", "SQL"),
                ("NL", "schema.niles", "NL"),
                ("RS", "schema.niles", "RS"),
                ("PRQL", "schema_plain.sql", "SQL"),
                ("DL", "schema.dl", "DL"),
            ] {
                let src = std::fs::read_to_string(corpus().join("schema").join(f)).expect("schema");
                let n = syntax_study::tokens::tokenize(&src, lex).len();
                let lines = src.lines().filter(|l| !l.trim().is_empty()).count();
                schema.push(format!("{s}\t{f}\t{lines}\t{n}"));
            }
            let dir = results();
            std::fs::create_dir_all(&dir).expect("mkdir");
            std::fs::write(dir.join("programs.tsv"), rows.join("\n") + "\n").expect("write");
            std::fs::write(dir.join("schemas.tsv"), schema.join("\n") + "\n").expect("write");
            eprintln!("wrote programs.tsv and schemas.tsv in {}", dir.display());
        }
        Some("cost-ext") => {
            // Check cost of the two external checkers (design §6, speed): wall time per
            // invocation over the corpus minus the same binary's time on an empty program,
            // reported apart from the in-process figures and never ranked against them.
            // 5 warm-up passes, then 30 timed; per file, the median over passes.
            let ctx = Ctx::new(d.clone(), work());
            let mut rows = vec![
                "surface\tbinary\tfiles\tprogram lines\tmedian ms per invocation (mean over files)\tempty program ms\tnet ms per KLOC".to_string(),
            ];
            for s in ["PRQL", "DL"] {
                let progs: Vec<String> = oracle::TASKS
                    .iter()
                    .filter(|t| tasks::in_scope(s, t))
                    .filter_map(|t| std::fs::read_to_string(program_path(t, s)).ok())
                    .collect();
                let time = |src: &str| -> f64 {
                    let t0 = std::time::Instant::now();
                    let _ = syntax_study::run::check(&ctx, s, src);
                    t0.elapsed().as_secs_f64() * 1e3
                };
                let median = |mut v: Vec<f64>| -> f64 {
                    v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
                    let n = v.len();
                    if n % 2 == 1 {
                        v[n / 2]
                    } else {
                        (v[n / 2 - 1] + v[n / 2]) / 2.0
                    }
                };
                for _ in 0..5 {
                    time("");
                    progs.iter().for_each(|p| {
                        time(p);
                    });
                }
                let mut per: Vec<Vec<f64>> = vec![Vec::new(); progs.len()];
                let mut empty = Vec::new();
                for _ in 0..30 {
                    empty.push(time(""));
                    for (i, p) in progs.iter().enumerate() {
                        per[i].push(time(p));
                    }
                }
                let e = median(empty);
                let meds: Vec<f64> = per.into_iter().map(median).collect();
                let mean = meds.iter().sum::<f64>() / meds.len() as f64;
                let net: f64 = meds.iter().map(|m| m - e).sum();
                let lines: usize = progs
                    .iter()
                    .map(|p| p.lines().filter(|l| !l.trim().is_empty()).count())
                    .sum();
                let bin = if s == "PRQL" {
                    ctx.prqlc.clone()
                } else {
                    ctx.souffle.clone()
                }
                .expect("binary");
                rows.push(format!(
                    "{s}\t{}\t{}\t{lines}\t{mean:.2}\t{e:.2}\t{:.1}",
                    syntax_study::ext::version(&bin),
                    progs.len(),
                    net / lines as f64 * 1000.0
                ));
            }
            let dir = results();
            std::fs::create_dir_all(&dir).expect("mkdir");
            std::fs::write(dir.join("cost-external.tsv"), rows.join("\n") + "\n").expect("write");
            eprintln!("wrote cost-external.tsv");
        }
        Some("load-check") => {
            // Design §4: every surface loads the same rows, asserted by count and SHA-256.
            use syntax_study::load;
            let ctx = Ctx::new(d.clone(), work());
            let mut c = syntax_study::pg::connect().unwrap_or_else(|e| panic!("{e:?}"));
            let loaded: Vec<(&str, load::Rows)> = vec![
                (
                    "SQL",
                    load::from_pg(&mut c, true, &ctx.data_typed).expect("SQL"),
                ),
                (
                    "PRQL",
                    load::from_pg(&mut c, false, &ctx.data_plain).expect("PRQL"),
                ),
                (
                    "DL",
                    load::from_dl(
                        ctx.souffle.as_ref().expect("souffle"),
                        &ctx.facts,
                        &ctx.work,
                    )
                    .expect("DL"),
                ),
                ("NL, RS", syntax_study::niles::loaded_rows(&d)),
            ];
            let mut rows = vec!["surface\trelation\trows\tsha256\tequals the dataset".to_string()];
            let mut ok = true;
            for (s, got) in &loaded {
                let want = load::expected(&d, got.contains_key("epochs"));
                for (rel, w) in &want {
                    let g = got.get(rel).cloned().unwrap_or_default();
                    let (n, h) = load::digest(&g);
                    let same = load::digest(w) == (n, h.clone());
                    ok &= same;
                    rows.push(format!(
                        "{s}\t{rel}\t{n}\t{h}\t{}",
                        if same { "yes" } else { "NO" }
                    ));
                }
                if !got.contains_key("epochs") {
                    rows.push(format!(
                        "{s}\tepochs\t\t\tn/a: system time, not a column (A1)"
                    ));
                }
            }
            let dir = results();
            std::fs::create_dir_all(&dir).expect("mkdir");
            std::fs::write(dir.join("load.tsv"), rows.join("\n") + "\n").expect("write");
            println!("{}", rows.join("\n"));
            if !ok {
                eprintln!("a surface did not load the dataset's rows");
                std::process::exit(1);
            }
        }
        Some("report") => {
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let log = std::process::Command::new("git")
                .args([
                    "-C",
                    root.to_str().expect("utf-8"),
                    "log",
                    "--reverse",
                    "--format=%h %s",
                    "--",
                    "docs/study/E30-syntax-design.md",
                ])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();
            let design = log
                .lines()
                .map(|l| {
                    let (h, subj) = l.split_once(' ').unwrap_or((l, ""));
                    let what = if subj.contains("A1") {
                        "amendment A1"
                    } else if subj.contains("A2") {
                        "amendment A2"
                    } else {
                        "the pre-registration"
                    };
                    format!("`{h}` {what}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            let md = syntax_study::report::render(&syntax_study::report::Inputs {
                dir: &results(),
                design: &design,
            });
            let out = root.join("results/E30-syntax.md");
            std::fs::write(&out, md).expect("write");
            eprintln!("wrote {}", out.display());
        }
        Some("circuit") => {
            // A debugging aid: the lowered circuit of a Niles program, operator by operator.
            let src = std::fs::read_to_string(&args[1]).expect("read");
            let full = format!("{}\n{src}", syntax_study::niles::SCHEMA);
            let (prog, _) = niles_lang::parser::parse_program(&full);
            let (cat, _) = niles_lang::resolve::resolve_program(&prog, 0);
            let (l, _) = niles_lang::lower::lower_program(&prog, &cat);
            for n in &l.circuit.nodes {
                println!("{} {:?} <- {:?}", n.id, n.op, n.inputs);
            }
        }
        _ => {
            eprintln!("usage: e30 gen | e30 run [TASK|-] [SURFACE]");
            std::process::exit(2);
        }
    }
}
