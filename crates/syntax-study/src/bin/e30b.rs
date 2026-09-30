//! `e30b` — E30b′'s driver (design `docs/study/E30b-design.md`, cycle 15, C15-05).
//!
//! ```text
//! e30b run [TASK] [SURFACE]   check and execute each program, compare with the oracle
//! e30b measure                results/E30b-syntax/programs.tsv: verdict, tokens, constructs
//! e30b mutate                 results/E30b-syntax/mutants.tsv: every mutant, classified
//! e30b adversarial            results/E30b-syntax/adversarial.tsv: the rule's corpus
//! e30b report                 results/E30b-syntax.md, rendered from the rows
//! ```

use std::path::{Path, PathBuf};
use syntax_study::answer::Answer;
use syntax_study::e30b::{self, SURFACES};
use syntax_study::run::{classify, verdict, Ctx, Outcome};
use syntax_study::{data, oracle};

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

fn work() -> PathBuf {
    std::env::var_os("E30_WORK")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("e30b-work"))
}

fn results() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../results/E30b-syntax")
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

/// A task's program on a surface: its text, or why there is none.
fn program(task: &str, surface: &str) -> Result<String, String> {
    if let Some(why) = e30b::declared_not_expressible(task, surface) {
        return Err(why.to_string());
    }
    let p = e30b::program_path(task, surface);
    std::fs::read_to_string(&p).map_err(|_| {
        std::fs::read_to_string(format!("{}.none", p.display()))
            .map(|r| r.lines().next().unwrap_or("").to_string())
            .unwrap_or_else(|_| format!("no program at {}", p.display()))
    })
}

fn clean(s: &str) -> String {
    s.replace(['\t', '\n'], " ")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let d = data::generate(1);
    match args.first().map(String::as_str) {
        Some("run") => {
            let only_task = args.get(1).filter(|t| t.as_str() != "-").cloned();
            let only_surface = args.get(2).cloned();
            let mut ctx = Ctx::new(d.clone(), work());
            for t in oracle::TASKS {
                if only_task.as_deref().is_some_and(|x| x != *t) {
                    continue;
                }
                let want = oracle::expected(t, &d);
                for s in SURFACES {
                    if only_surface.as_deref().is_some_and(|x| x != *s) {
                        continue;
                    }
                    match program(t, s) {
                        Err(why) => println!("{t}\t{s}\tNOT EXPRESSIBLE {why}"),
                        Ok(src) => {
                            let o = verdict(&mut ctx, t, s, &src, &want);
                            println!("{t}\t{s}\t{}", describe(&o, &want));
                        }
                    }
                }
            }
        }
        Some("measure") => {
            let mut ctx = Ctx::new(d.clone(), work());
            let kwdir = corpus().join("keywords");
            let mut rows = vec![
                "task\tsurface\tverdict\tdetail\tlines\ttokens\tkeywords\toperators".to_string(),
            ];
            for t in oracle::TASKS {
                let want = oracle::expected(t, &d);
                for s in SURFACES {
                    let src = match program(t, s) {
                        Ok(src) => src,
                        Err(why) => {
                            rows.push(format!(
                                "{t}\t{s}\tnot expressible\t{}\t\t\t\t",
                                clean(&why)
                            ));
                            continue;
                        }
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
                    let lex = e30b::lexis(s);
                    let toks = syntax_study::tokens::tokenize(&src, lex);
                    let kw = syntax_study::keywords::load(&kwdir, lex);
                    let (k, o) = syntax_study::tokens::constructs(&toks, lex, &kw);
                    let lines = src.lines().filter(|l| !l.trim().is_empty()).count();
                    rows.push(format!(
                        "{t}\t{s}\t{v}\t{}\t{lines}\t{}\t{k}\t{o}",
                        clean(&detail),
                        toks.len()
                    ));
                }
            }
            write("programs.tsv", &rows);
        }
        Some("mutate") => {
            let only_task = args.get(1).filter(|t| t.as_str() != "-").cloned();
            let only_surface = args.get(2).cloned();
            let mut ctx = Ctx::new(d.clone(), work());
            let mut rows = vec![
                "task\tsurface\top\tsite\tline\tin_annotation\tmutation\tclass\tdetail".to_string(),
            ];
            for t in oracle::TASKS {
                if only_task.as_deref().is_some_and(|x| x != *t) {
                    continue;
                }
                let want = oracle::expected(t, &d);
                for s in SURFACES {
                    if only_surface.as_deref().is_some_and(|x| x != *s) {
                        continue;
                    }
                    let Ok(src) = program(t, s) else { continue };
                    let base = verdict(&mut ctx, t, s, &src, &want);
                    let base_answer = match &base {
                        Outcome::Answer(a) if want.diff(a).is_none() => Some(a.clone()),
                        Outcome::Unexecuted(_) => None,
                        other => panic!(
                            "{t} {s}: the unmutated program is not correct: {}",
                            describe(other, &want)
                        ),
                    };
                    for m in e30b::mutants(&src, s) {
                        let (class, detail) =
                            classify(&mut ctx, t, s, &m.text, base_answer.as_ref(), &want);
                        let ann = e30b::in_annotation(&src, &m.text, s);
                        eprintln!("{t} {s} {}#{} {class}", m.op, m.site);
                        rows.push(format!(
                            "{t}\t{s}\t{}\t{}\t{}\t{}\t{}\t{class}\t{}",
                            m.op,
                            m.site,
                            m.line,
                            if ann { "yes" } else { "no" },
                            clean(&m.what),
                            clean(&detail)
                        ));
                    }
                }
            }
            let name = if only_task.is_some() || only_surface.is_some() {
                "mutants-partial.tsv"
            } else {
                "mutants.tsv"
            };
            write(name, &rows);
        }
        Some("adversarial") => {
            let mut pg = syntax_study::pg::connect().unwrap_or_else(|e| panic!("{e:?}"));
            let mut rows =
                vec!["case\tclass\tsurface\ttwin\toutcome\tcodes\texecution".to_string()];
            let dir = e30b::adversarial_dir();
            for c in e30b::cases() {
                let cd = dir.join(&c.id);
                // SQL: R1 always; R2 where the case has R2 twins.
                for (surface, r2) in [("SQL-R1", false), ("SQL-R2", true)] {
                    for (twin, file, call) in [
                        (
                            "defective",
                            if r2 { "d.r2.sql" } else { "d.sql" },
                            &c.d_sql_args,
                        ),
                        (
                            "correct",
                            if r2 { "c.r2.sql" } else { "c.sql" },
                            &c.c_sql_args,
                        ),
                    ] {
                        let Ok(src) = std::fs::read_to_string(cd.join(file)) else {
                            continue;
                        };
                        let schema = e30b::sql_schema(&c, r2);
                        let verdict = syntax_study::run::check_sql(schema, &src);
                        let (outcome, codes) = match &verdict {
                            Some(v) => ("refused", v.join(" ")),
                            None => ("accepted", String::new()),
                        };
                        let exec = if twin == "correct" {
                            match e30b::run_sql_twin(&mut pg, &d, schema, r2, &src, call) {
                                Ok(n) => format!("runs; {n} postings after"),
                                Err(e) => format!("RAISES {e}"),
                            }
                        } else {
                            String::new()
                        };
                        rows.push(format!(
                            "{}\t{}\t{surface}\t{twin}\t{outcome}\t{codes}\t{}",
                            c.id,
                            c.class,
                            clean(&exec)
                        ));
                    }
                }
                for (twin, spec, args) in [
                    ("defective", &c.niles_defective, &c.d_niles_args),
                    ("correct", &c.niles_correct, &c.c_niles_args),
                ] {
                    if let Some(why) = spec.strip_prefix("n/a: ") {
                        rows.push(format!(
                            "{}\t{}\tNL\t{twin}\tnot expressible\t\t{}",
                            c.id,
                            c.class,
                            clean(why)
                        ));
                        continue;
                    }
                    let src = std::fs::read_to_string(cd.join(spec)).expect("a Niles twin");
                    let schema = e30b::niles_schema(&c);
                    let v = syntax_study::niles::check_with(schema, &src);
                    let (outcome, codes) = if v.refused() {
                        ("refused", v.errors.join(" "))
                    } else {
                        ("accepted", String::new())
                    };
                    let exec = if twin == "correct" && !v.refused() {
                        match syntax_study::niles::run_fn_with(
                            schema,
                            &src,
                            "task",
                            &e30b::niles_args(args),
                            &d,
                        ) {
                            Ok(a) => format!("runs; {} legs", a.rows.len()),
                            Err(syntax_study::niles::RunErr::NotInSubset(f)) => {
                                format!("unexecuted: niles-interp refuses `{f}`")
                            }
                            Err(syntax_study::niles::RunErr::Failed(e)) => format!("RAISES {e}"),
                        }
                    } else {
                        String::new()
                    };
                    rows.push(format!(
                        "{}\t{}\tNL\t{twin}\t{outcome}\t{codes}\t{}",
                        c.id,
                        c.class,
                        clean(&exec)
                    ));
                }
            }
            write("adversarial.tsv", &rows);
        }
        Some("report") => {
            let md = syntax_study::e30b_report::render(&results());
            let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../results/E30b-syntax.md");
            std::fs::write(&out, md).expect("write");
            eprintln!("wrote {}", out.display());
        }
        _ => {
            eprintln!("usage: e30b run|measure|mutate|adversarial|report");
            std::process::exit(2);
        }
    }
}

fn write(name: &str, rows: &[String]) {
    let dir = results();
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(dir.join(name), rows.join("\n") + "\n").expect("write");
    eprintln!("wrote {}", dir.join(name).display());
}
