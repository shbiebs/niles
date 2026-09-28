//! `comparator run | render | probe` — see the crate documentation.

use comparator::arms::{Arm, NArm, PgArm, PgKind};
use comparator::run::{run_point, Config};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf()
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn list<T: std::str::FromStr>(s: &str) -> Vec<T> {
    s.split(',').filter_map(|x| x.trim().parse().ok()).collect()
}

fn build_arms(names: &[String], multi: bool, scratch: &Path) -> Vec<Box<dyn Arm>> {
    let root = repo_root();
    let bin = root.join("target/release/nilestreamd");
    names
        .iter()
        .map(|n| -> Box<dyn Arm> {
            match n.as_str() {
                "N" => Box::new(NArm::new(5450, scratch.join("n"), bin.clone(), multi, 16)),
                "P" => Box::new(PgArm::new(PgKind::P, 5451, &root)),
                "P+" => Box::new(PgArm::new(PgKind::PPlus, 5452, &root)),
                "M" => Box::new(PgArm::new(PgKind::M, 5453, &root)),
                "M+" => Box::new(PgArm::new(PgKind::MPlus, 5454, &root)),
                other => panic!("unknown arm {other}"),
            }
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    let root = repo_root();
    let dir = PathBuf::from(
        arg(&args, "--dir")
            .unwrap_or_else(|| root.join("results/E27-comparator").display().to_string()),
    );
    match cmd {
        "run" => {
            let series = arg(&args, "--series").unwrap_or_else(|| "single".into());
            let multi = match series.as_str() {
                "single" => false,
                "multi" => true,
                s => panic!("--series single|multi, not {s}"),
            };
            let sizes: Vec<u64> = list(&arg(&args, "--sizes").unwrap_or_else(|| "1000".into()));
            let seeds: Vec<u64> =
                list(&arg(&args, "--seeds").unwrap_or_else(|| "1,7,42,100,2024".into()));
            let names: Vec<String> = arg(&args, "--arms")
                .unwrap_or_else(|| "N,P+,P,M".into())
                .split(',')
                .map(String::from)
                .collect();
            let mut cfg = Config::declared(multi);
            if let Some(r) = arg(&args, "--runs") {
                cfg.runs = r.parse().expect("--runs");
            }
            if let Some(w) = arg(&args, "--warmups") {
                cfg.warmups = w.parse().expect("--warmups");
            }
            let skip = args.iter().any(|a| a == "--skip-existing");
            let scratch =
                PathBuf::from(arg(&args, "--scratch").unwrap_or_else(|| "/var/tmp/e27".into()));
            let bin = root.join("target/release/nilestreamd");
            if names.iter().any(|n| n == "N") && !bin.exists() {
                eprintln!("comparator: {} is missing; run `cargo build --release -p nilestream-server --bin nilestreamd` first", bin.display());
                std::process::exit(2);
            }
            let mut arms = build_arms(&names, multi, &scratch);
            for &size in &sizes {
                for &seed in &seeds {
                    let path = dir.join(format!("{series}-{size}-{seed}.tsv"));
                    if skip && path.exists() {
                        eprintln!("skip {}", path.display());
                        continue;
                    }
                    let t0 = std::time::Instant::now();
                    let p = run_point(&mut arms, &cfg, size, seed, &mut |m| eprintln!("{m}"));
                    let prov = comparator::store::provenance(&root);
                    comparator::store::write(&path, &series, &p, &prov).expect("write point");
                    eprintln!(
                        "wrote {} in {:.0}s{}",
                        path.display(),
                        t0.elapsed().as_secs_f64(),
                        if p.failures.is_empty() {
                            String::new()
                        } else {
                            format!(" — FAILURES: {:?}", p.failures)
                        }
                    );
                }
            }
            for a in arms.iter_mut() {
                a.stop();
            }
        }
        "render" => {
            let arms = build_arms(
                &["N".into(), "P+".into(), "P".into(), "M".into(), "M+".into()],
                false,
                Path::new("/var/tmp/e27"),
            );
            let lines: Vec<(String, String)> = arms
                .iter()
                .map(|a| (a.name().to_string(), a.describe()))
                .collect();
            let refs: Vec<&dyn Arm> = arms.iter().take(4).map(|a| a.as_ref()).collect();
            let table = comparator::arms::sql_table(&refs);
            let main = root.join("results/E27-comparator.md");
            let detail = root.join("results/E27-comparator-detail.md");
            comparator::render::render(&dir, &main, &detail, &lines, &table).expect("render");
            eprintln!("wrote {} and {}", main.display(), detail.display());
        }
        "probe" => {
            // The five anomalies (§5.7): N, P+ and M+ in both modes. M and P are a full
            // materialised view with no partial state and no anchor on a read, so the five —
            // anomalies of partial maintenance — have nothing to act on there; E27 says so.
            let scratch = PathBuf::from("/var/tmp/e27-probe");
            let names: Vec<String> = arg(&args, "--arms")
                .unwrap_or_else(|| "N,P+,M+".into())
                .split(',')
                .map(String::from)
                .collect();
            let mut arms = build_arms(&names, false, &scratch);
            let cfg = comparator::probe::ProbeConfig::default();
            let reps: usize = arg(&args, "--reps")
                .map(|r| r.parse().expect("--reps"))
                .unwrap_or(5);
            let mut out = String::new();
            for (k, v) in comparator::store::provenance(&root) {
                out += &format!("meta\t{k}\t{v}\n");
            }
            for arm in arms.iter_mut() {
                for mutate in [false, true] {
                    for rep in 0..reps {
                        match comparator::probe::probe(arm.as_mut(), &cfg, mutate) {
                            Ok(r) => {
                                eprintln!(
                                    "{} {} rep {rep}: exhibited {:?}; reads {}",
                                    r.arm,
                                    r.mode,
                                    r.exhibited(),
                                    r.reads
                                );
                                out += &comparator::probe::to_tsv(rep, &r);
                            }
                            Err(e) => {
                                eprintln!("{} probe failed: {e}", arm.name());
                                out += &format!(
                                    "failure\t{rep}\t{}\t{}\n",
                                    arm.name(),
                                    e.replace(['\t', '\n'], " ")
                                );
                            }
                        }
                    }
                }
                arm.stop();
            }
            std::fs::create_dir_all(&dir).expect("dir");
            std::fs::write(dir.join("probe.tsv"), out).expect("write probe.tsv");
        }
        _ => {
            eprintln!("usage: comparator run --series single|multi --sizes 1000,10000 --seeds 1,7 [--arms N,P+,P,M] [--runs 10] [--warmups 3] [--skip-existing]");
            std::process::exit(2);
        }
    }
}
