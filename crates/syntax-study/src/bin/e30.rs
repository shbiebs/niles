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

pub fn program_path(task: &str, ext: &str) -> PathBuf {
    corpus()
        .join(task)
        .join(format!("{}.{ext}", task.to_lowercase()))
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
                for (s, ext) in tasks::SURFACES {
                    if only_surface.as_deref().is_some_and(|x| x != *s) || !tasks::in_scope(s, t) {
                        continue;
                    }
                    let p = program_path(t, ext);
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
