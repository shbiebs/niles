//! `e30cost` — E30's in-process check cost (cycle 14, R2-06; design §6, speed): parse and
//! check time and peak heap per KLOC for SQL+C+L (`nilescheck-sql`) and for Niles's two
//! surfaces (`nilesc check`'s front end: parse, resolve, typecheck, lower), over every E30
//! program each surface has. Each file is checked as its surface's schema followed by the
//! program, as the study's harness checks it, and a line is a line the checker reads.
//!
//! The protocol is E14's (`checkcost`): 20 warm-up passes, then 200 timed passes, each pass
//! checking every file once with each checker, alternating; the median pass total per
//! thousand lines; peak heap from E18's counting allocator, the high-water mark above the
//! bytes live when a file's check starts. Writes `results/E30-syntax/cost-inprocess.tsv`.
//!
//!     cargo run --release --manifest-path tools/memprobe/Cargo.toml --bin e30cost

use memprobe::alloc;
use std::path::Path;
use std::time::Instant;
use syntax_study::tasks;

#[global_allocator]
static COUNTING: alloc::Counting = alloc::Counting;

fn niles(src: &str) -> usize {
    let (prog, mut d) = niles_lang::parser::parse_program(src);
    let (cat, r) = niles_lang::resolve::resolve_program(&prog, 0);
    d.extend(r);
    let (_, t) = niles_lang::typecheck::check_program(&prog, &cat);
    d.extend(t);
    let (_, l) = niles_lang::lower::lower_program(&prog, &cat);
    d.items.len() + l.items.len()
}

fn sql(src: &str) -> usize {
    match nilescheck_sql::parse(src) {
        Ok((stmts, _)) => nilescheck_sql::check_all(&stmts).len(),
        Err(_) => usize::MAX,
    }
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn mad(v: &[f64]) -> f64 {
    let mut w = v.to_vec();
    let m = median(&mut w);
    let mut d: Vec<f64> = v.iter().map(|x| (x - m).abs()).collect();
    median(&mut d)
}

fn cmd(c: &str, args: &[&str]) -> String {
    std::process::Command::new(c)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root");
    let corpus = root.join("crates/syntax-study/corpus");
    type Checker = fn(&str) -> usize;
    let surfaces: [(&str, Checker, &str); 3] = [
        ("SQL", sql, syntax_study::pg::SCHEMA_SQL),
        ("NL", niles, syntax_study::niles::SCHEMA),
        ("RS", niles, syntax_study::niles::SCHEMA),
    ];
    let files: Vec<Vec<String>> = surfaces
        .iter()
        .map(|(s, _, schema)| {
            syntax_study::oracle::TASKS
                .iter()
                .filter(|t| tasks::in_scope(s, t))
                .filter_map(|t| std::fs::read_to_string(corpus.join(tasks::program_file(t, s))).ok())
                .map(|p| format!("{schema}\n{p}"))
                .collect()
        })
        .collect();

    assert!(alloc::installed(), "e30cost: the counting allocator is not installed");
    let peaks: Vec<(usize, f64)> = surfaces
        .iter()
        .zip(&files)
        .map(|((_, f, _), fs)| {
            let mut max_b = 0usize;
            let mut max_k = 0f64;
            for s in fs {
                let (_, c) = alloc::count(|| std::hint::black_box(f(s)));
                max_b = max_b.max(c.peak_delta);
                max_k = max_k.max(c.peak_delta as f64 * 1000.0 / s.lines().count() as f64);
            }
            (max_b, max_k)
        })
        .collect();

    let (warm, reps) = (20, 200);
    let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(reps); surfaces.len()];
    let longest = files.iter().map(Vec::len).max().unwrap_or(0);
    for r in 0..warm + reps {
        let mut pass = vec![0f64; surfaces.len()];
        for i in 0..longest {
            for (k, (_, f, _)) in surfaces.iter().enumerate() {
                if let Some(s) = files[k].get(i) {
                    let t = Instant::now();
                    std::hint::black_box(f(s));
                    pass[k] += t.elapsed().as_secs_f64();
                }
            }
        }
        if r >= warm {
            for k in 0..surfaces.len() {
                times[k].push(pass[k] * 1e6);
            }
        }
    }

    let mut out = String::new();
    let head = |k: &str, v: String| format!("# {k}: {v}\n");
    out += &head(
        "commit",
        cmd("git", &["-C", root.to_str().unwrap(), "rev-parse", "--short=12", "HEAD"]),
    );
    let dirty = !cmd(
        "git",
        &["-C", root.to_str().unwrap(), "status", "--porcelain", "--", "crates", "Cargo.toml", "Cargo.lock"],
    )
    .is_empty();
    out += &head("worktree", if dirty { "MODIFIED".into() } else { "clean".into() });
    out += &head(
        "host",
        format!(
            "{} — {} CPUs",
            cmd("uname", &["-srm"]),
            std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0)
        ),
    );
    out += &head("toolchain", env!("MEMPROBE_RUSTC").to_string());
    out += &head(
        "protocol",
        format!("{warm} warm-up passes, then {reps} timed passes; each pass checks every file once with each checker, alternating; in-process, release build; peak heap from E18's counting allocator"),
    );
    out += "surface\tfiles\tlines\tmedian pass us\tMAD us\tus per KLOC\tpeak heap per check max bytes\tpeak heap per KLOC max bytes\n";
    for (k, (s, _, _)) in surfaces.iter().enumerate() {
        let lines: usize = files[k].iter().map(|f| f.lines().count()).sum();
        let m = mad(&times[k]);
        let med = median(&mut times[k]);
        out += &format!(
            "{s}\t{}\t{lines}\t{med:.0}\t{m:.0}\t{:.0}\t{}\t{:.0}\n",
            files[k].len(),
            med * 1000.0 / lines as f64,
            peaks[k].0,
            peaks[k].1
        );
    }
    let path = root.join("results/E30-syntax/cost-inprocess.tsv");
    std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
    std::fs::write(&path, &out).expect("write");
    print!("{out}");
    eprintln!("wrote {}", path.display());
}
