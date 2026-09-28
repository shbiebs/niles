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
                    comparator::store::write(&path, &series, &p).expect("write point");
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
        _ => {
            eprintln!("usage: comparator run --series single|multi --sizes 1000,10000 --seeds 1,7 [--arms N,P+,P,M] [--runs 10] [--warmups 3] [--skip-existing]");
            std::process::exit(2);
        }
    }
}
