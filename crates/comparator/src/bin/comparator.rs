//! `comparator run | render | probe` — see the crate documentation.

use comparator::arms::{Arm, NArm, PgArm, PgKind};
use comparator::h1::H1Arm;
use comparator::h3::H3Arm;
use comparator::run::{run_point, Config};
use comparator::t::TArm;
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
                "H2" => Box::new(PgArm::new(PgKind::H2, 5456, &root)),
                "H1" => {
                    let mut a = H1Arm::new(5457, 5458, scratch.join("h1"), &root);
                    // The calibrated byte budget, when one has been measured (calibrate-h1).
                    a.calibration = comparator::h1::read_calibration(
                        &root.join("results/E27-comparator/h1-calibration.tsv"),
                    );
                    Box::new(a)
                }
                "H3" => Box::new(H3Arm::new(5461, 5462, scratch.join("h3"), &root)),
                "T" => Box::new(TArm::new(scratch.join("t"), &root)),
                "H1M" => {
                    let mut a = H1Arm::new(5459, 5460, scratch.join("h1m"), &root);
                    a.cluster = comparator::pgcluster::Cluster::new("h1m", 5459);
                    a.mutable = true;
                    Box::new(a)
                }
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
            // `--shape p99` is the targeted re-run of 2026-09-28 (Config::p99); its points go
            // to their own directory so the two shapes are never pooled.
            let p99e = arg(&args, "--shape").as_deref() == Some("p99-engine");
            let p99 = p99e || arg(&args, "--shape").as_deref() == Some("p99");
            let mut cfg = if p99 {
                Config::p99(multi)
            } else {
                Config::declared(multi)
            };
            let dir = if p99e && arg(&args, "--dir").is_none() {
                root.join("results/E27-p99-engine")
            } else if p99 && arg(&args, "--dir").is_none() {
                root.join("results/E27-p99")
            } else {
                dir.clone()
            };
            // `--q1-only`: the author's decision of 2026-09-28 for the p99 re-run on the
            // multi-currency series at 10⁵ and 10⁶ — q1 alone (1,000 reads per run at one
            // client, 4 × 250 at four), q2 recorded NOT RUN with the reason, because there
            // Nilestream answers each anchored read by folding the whole ledger (a 10⁴ point
            // took 21 minutes; 10⁵ was estimated at ~3.5 h per point).
            let q1_only = args.iter().any(|a| a == "--q1-only");
            if q1_only {
                cfg.ops = 1000;
                cfg.mix = vec![("q1", 1.0)];
                cfg.conc_reads = 250;
                cfg.conc_q1_share = 1.0;
            }
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
                    let mut p = run_point(&mut arms, &cfg, size, seed, &mut |m| eprintln!("{m}"));
                    if q1_only {
                        for a in arms.iter() {
                            p.not_run.entry(a.name().into()).or_default().push((
                                "q2".into(),
                                "the author's decision of 2026-09-28: on the multi-currency series at 10⁵ and above, Nilestream answers every anchored read by folding the whole ledger, so 1,000 q2 reads per run cost hours per point (a 10⁴ point took 21 minutes); q2's p50 verdict is in E27".into(),
                            ));
                        }
                    }
                    let mut prov = comparator::store::provenance(&root);
                    prov.push(("checkpoint_interval".into(), cfg.checkpoint.to_string()));
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
            // Every arm this comparator knows, described; the renderer lists those that appear
            // in the points (or, for the probe's mutable twins, in the probe file).
            let arms = build_arms(
                &["N", "P+", "P", "M", "H1", "H2", "H3", "T", "M+", "H1M"].map(String::from),
                false,
                Path::new("/var/tmp/e27"),
            );
            let lines: Vec<(String, String)> = arms
                .iter()
                .map(|a| (a.name().to_string(), a.describe()))
                .collect();
            let refs: Vec<&dyn Arm> = arms
                .iter()
                .filter(|a| !matches!(a.name(), "M+" | "H1M"))
                .map(|a| a.as_ref())
                .collect();
            let table = comparator::arms::sql_table(&refs);
            let p99 = arg(&args, "--shape").as_deref() == Some("p99");
            let p99e = arg(&args, "--shape").as_deref() == Some("p99-engine");
            let (shape, stem, dir) = if p99e {
                (
                    &comparator::render::P99_ENGINE,
                    "E27-p99-engine",
                    if arg(&args, "--dir").is_some() {
                        dir.clone()
                    } else {
                        root.join("results/E27-p99-engine")
                    },
                )
            } else if p99 {
                (
                    &comparator::render::P99,
                    "E27-p99",
                    if arg(&args, "--dir").is_some() {
                        dir.clone()
                    } else {
                        root.join("results/E27-p99")
                    },
                )
            } else {
                (&comparator::render::MAIN, "E27-comparator", dir.clone())
            };
            // `--stem` renders another directory (e.g. Host C's) into its own files, so a
            // rendering of one host can never overwrite another's.
            let stem = arg(&args, "--stem").unwrap_or_else(|| stem.to_string());
            let main = root.join(format!("results/{stem}.md"));
            let detail = root.join(format!("results/{stem}-detail.md"));
            comparator::render::render(&dir, &main, &detail, &lines, &table, shape)
                .expect("render");
            eprintln!("wrote {} and {}", main.display(), detail.display());
        }
        "calibrate-h1" => {
            // The author's decision of 2026-09-28: H1's --memory-limit per (series, size),
            // measured on H1 alone before the measured runs. Seed 1's universe per size.
            let sizes: Vec<u64> =
                list(&arg(&args, "--sizes").unwrap_or_else(|| "1000,10000,100000".into()));
            let series: Vec<String> = arg(&args, "--series")
                .unwrap_or_else(|| "single,multi".into())
                .split(',')
                .map(String::from)
                .collect();
            let scratch = PathBuf::from("/var/tmp/e27-cal");
            let path = root.join("results/E27-comparator/h1-calibration.tsv");
            // Written after every entry and resumed from what is on disk, so a container
            // restart costs one size, not the whole calibration.
            let mut out = std::fs::read_to_string(&path).unwrap_or_default();
            if out.is_empty() {
                for (k, v) in comparator::store::provenance(&root) {
                    out += &format!("meta\t{k}\t{v}\n");
                }
            }
            let done = comparator::h1::read_calibration(&path).unwrap_or_default();
            let mut arm = H1Arm::new(5457, 5458, scratch.join("h1"), &root);
            for se in &series {
                for &size in &sizes {
                    if done.contains_key(&(se == "multi", size)) {
                        eprintln!("H1 {se} {size}: already calibrated");
                        continue;
                    }
                    let u = comparator::universe::Universe::generate(
                        &comparator::universe::Params::declared(size, 1, 0.6, se == "multi"),
                    );
                    let c = comparator::h1::calibrate(&mut arm, &u).expect("calibrate H1");
                    eprintln!(
                        "H1 {se} {size}: allocated {} after load, {} after reading all {} keys; limit {}",
                        c.a0, c.a1, c.keys, c.limit
                    );
                    out += &format!(
                        "h1cal\t{se}\t{size}\t{}\t{}\t{}\t{}\n",
                        c.a0, c.a1, c.keys, c.limit
                    );
                    std::fs::create_dir_all(path.parent().unwrap()).expect("dir");
                    std::fs::write(&path, &out).expect("write h1-calibration.tsv");
                }
            }
            eprintln!("wrote {}", path.display());
        }
        "probe" => {
            // The five anomalies (§5.7): N, P+ and M+ in both modes, and (R2-03) H1 with its
            // mutable twin H1M. M and P are a full materialised view with no partial state and
            // no anchor on a read, so the five — anomalies of partial maintenance — have
            // nothing to act on there; E27 says so.
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
            // `--out` names the file, so arms added later (R2-03's H1) are probed into their own
            // file without re-running the arms already reported.
            let file = arg(&args, "--out").unwrap_or_else(|| "probe.tsv".into());
            std::fs::create_dir_all(&dir).expect("dir");
            std::fs::write(dir.join(&file), out).expect("write the probe file");
        }
        _ => {
            eprintln!("usage: comparator run --series single|multi --sizes 1000,10000 --seeds 1,7 [--arms N,P+,P,M] [--runs 10] [--warmups 3] [--skip-existing]");
            std::process::exit(2);
        }
    }
}
