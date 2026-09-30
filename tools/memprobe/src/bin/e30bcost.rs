//! `e30bcost` — E30b′'s in-process check cost (cycle 15, C15-05; design
//! `docs/study/E30b-design.md` §8): parse and check time and peak heap per KLOC for the
//! upgraded SQL+C+L over representations R1 and R2, and for Niles's two surfaces, over
//! E30b′'s programs, and separately over its adversarial corpus. Each file is checked as its
//! surface's schema followed by the program. The protocol is E30's (`e30cost`): 20 warm-up
//! passes, then 200 timed passes, alternating checkers; the median pass total per thousand
//! lines; peak heap from E18's counting allocator. Writes `results/E30b-syntax/cost-inprocess.tsv`.
//!
//!     cargo run --release --manifest-path tools/memprobe/Cargo.toml --bin e30bcost

use memprobe::alloc;
use std::path::Path;
use std::time::Instant;

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
    let _ = &corpus;
    let read = |p: std::path::PathBuf| std::fs::read_to_string(p).ok();
    let e30 = |s: &'static str, schema: &'static str| -> Vec<String> {
        syntax_study::oracle::TASKS
            .iter()
            .filter(|t| syntax_study::e30b::declared_not_expressible(t, s).is_none())
            .filter_map(|t| read(syntax_study::e30b::program_path(t, s)))
            .map(|p| format!("{schema}\n{p}"))
            .collect()
    };
    let adv_dir = syntax_study::e30b::adversarial_dir();
    let cases = syntax_study::e30b::cases();
    let adv_sql = |r2: bool| -> Vec<String> {
        let mut v = Vec::new();
        for c in &cases {
            for f in if r2 {
                ["d.r2.sql", "c.r2.sql"]
            } else {
                ["d.sql", "c.sql"]
            } {
                if let Some(p) = read(adv_dir.join(&c.id).join(f)) {
                    v.push(format!("{}\n{p}", syntax_study::e30b::sql_schema(c, r2)));
                }
            }
        }
        v
    };
    let adv_niles = || -> Vec<String> {
        let mut v = Vec::new();
        for c in &cases {
            for f in ["d.nl.niles", "c.nl.niles"] {
                if let Some(p) = read(adv_dir.join(&c.id).join(f)) {
                    v.push(format!("{}\n{p}", syntax_study::e30b::niles_schema(c)));
                }
            }
        }
        v
    };
    let surfaces: Vec<(&str, Checker)> = vec![
        ("E30 corpus\tSQL-R1", sql),
        ("E30 corpus\tSQL-R2", sql),
        ("E30 corpus\tNL", niles),
        ("E30 corpus\tRS", niles),
        ("adversarial\tSQL-R1", sql),
        ("adversarial\tSQL-R2", sql),
        ("adversarial\tNL", niles),
    ];
    let files: Vec<Vec<String>> = vec![
        e30("SQL", syntax_study::pg::SCHEMA_SQL),
        e30("SQL2", syntax_study::e30b::SCHEMA_R2),
        e30("NL", syntax_study::niles::SCHEMA),
        e30("RS", syntax_study::niles::SCHEMA),
        adv_sql(false),
        adv_sql(true),
        adv_niles(),
    ];

    assert!(
        alloc::installed(),
        "e30bcost: the counting allocator is not installed"
    );
    let peaks: Vec<(usize, f64)> = surfaces
        .iter()
        .zip(&files)
        .map(|((_, f), fs)| {
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
            for (k, (_, f)) in surfaces.iter().enumerate() {
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
        cmd(
            "git",
            &[
                "-C",
                root.to_str().unwrap(),
                "rev-parse",
                "--short=12",
                "HEAD",
            ],
        ),
    );
    let dirty = !cmd(
        "git",
        &[
            "-C",
            root.to_str().unwrap(),
            "status",
            "--porcelain",
            "--",
            "crates",
            "Cargo.toml",
            "Cargo.lock",
        ],
    )
    .is_empty();
    out += &head(
        "worktree",
        if dirty {
            "MODIFIED".into()
        } else {
            "clean".into()
        },
    );
    out += &head(
        "host",
        format!(
            "{} — {} CPUs",
            cmd("uname", &["-srm"]),
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(0)
        ),
    );
    out += &head("toolchain", env!("MEMPROBE_RUSTC").to_string());
    out += &head(
        "protocol",
        format!("{warm} warm-up passes, then {reps} timed passes; each pass checks every file once with each checker, alternating; in-process, release build; peak heap from E18's counting allocator"),
    );
    out += "corpus\tsurface\tfiles\tlines\tmedian pass us\tMAD us\tus per KLOC\tpeak heap per check max bytes\tpeak heap per KLOC max bytes\n";
    for (k, (s, _)) in surfaces.iter().enumerate() {
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
    let path = root.join("results/E30b-syntax/cost-inprocess.tsv");
    std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
    std::fs::write(&path, &out).expect("write");
    print!("{out}");
    eprintln!("wrote {}", path.display());
}
