//! `checkcost` — check time and peak heap per KLOC for the two checkers of E14 (cycle 14,
//! R2-05: "record check time and peak memory per KLOC for both checkers"), measured
//! in-process so the figure is the checker's and not a process start.
//!
//! * **Niles**: `nilesc check`'s front end — parse, resolve at epoch 0, typecheck, lower —
//!   over `crates/counterproposal/niles/*.niles` and `d14/niles/d14_*.niles`.
//! * **SQL+C+L**: `nilescheck_sql::parse` and `check_all` over `sql-checked/_preamble.sql`
//!   followed by each `sql-checked/d*.sql` and `d14/sql/d14_*.sql` — the preamble is counted
//!   in the lines, because it is checked every time.
//!
//! 20 warm-up passes, then 200 timed passes; in each pass every file is checked once by each
//! checker, alternating, so drift lands on both. Time: the median over passes of one pass's
//! total, per thousand lines. Peak heap: E18's counting allocator (`memprobe::alloc::count`),
//! the high-water mark above the bytes live when each file's check starts. It lives in this
//! package, outside the workspace, because a counting allocator needs `unsafe` and §9.10
//! confines that to the memory instrument. Writes `results/E14-checkcost.md`.
//!
//!     cargo run --release --manifest-path tools/memprobe/Cargo.toml --bin checkcost

use memprobe::alloc;
use std::path::{Path, PathBuf};
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

fn files(dir: &Path, prefix: &str, ext: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let n = p.file_name().unwrap().to_string_lossy();
            n.starts_with(prefix) && n.ends_with(ext)
        })
        .collect();
    v.sort();
    v
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
        .expect("the repository root is two levels above this manifest");
    let cp = root.join("crates/counterproposal");
    let pre = std::fs::read_to_string(cp.join("sql-checked/_preamble.sql")).unwrap();
    let niles_src: Vec<(String, String)> = files(&cp.join("niles"), "d", ".niles")
        .into_iter()
        .chain(files(&cp.join("d14/niles"), "d14_", ".niles"))
        .map(|p| {
            (
                p.file_stem().unwrap().to_string_lossy().into(),
                std::fs::read_to_string(&p).unwrap(),
            )
        })
        .collect();
    let sql_src: Vec<(String, String)> = files(&cp.join("sql-checked"), "d", ".sql")
        .into_iter()
        .chain(files(&cp.join("d14/sql"), "d14_", ".sql"))
        .map(|p| {
            (
                p.file_stem().unwrap().to_string_lossy().into(),
                format!("{pre}\n{}", std::fs::read_to_string(&p).unwrap()),
            )
        })
        .collect();
    let lines = |v: &[(String, String)]| v.iter().map(|(_, s)| s.lines().count()).sum::<usize>();
    let (nl, sl) = (lines(&niles_src), lines(&sql_src));

    // Peak heap per file, one check each, before any timing. Single-threaded, as E18 is.
    assert!(
        alloc::installed(),
        "checkcost: the counting allocator is not installed; every peak would read zero"
    );
    let peak = |f: &dyn Fn(&str) -> usize, src: &str| -> usize {
        let (_, c) = alloc::count(|| std::hint::black_box(f(src)));
        c.peak_delta
    };
    let per_kloc = |bytes: usize, src: &str| bytes as f64 * 1000.0 / src.lines().count() as f64;
    let np: Vec<(usize, f64)> = niles_src
        .iter()
        .map(|(_, s)| {
            let b = peak(&niles, s);
            (b, per_kloc(b, s))
        })
        .collect();
    let sp: Vec<(usize, f64)> = sql_src
        .iter()
        .map(|(_, s)| {
            let b = peak(&sql, s);
            (b, per_kloc(b, s))
        })
        .collect();

    let (warm, reps) = (20, 200);
    let mut nt = Vec::with_capacity(reps);
    let mut st = Vec::with_capacity(reps);
    for r in 0..warm + reps {
        let (mut a, mut b) = (0f64, 0f64);
        for i in 0..niles_src.len().max(sql_src.len()) {
            if let Some((_, s)) = niles_src.get(i) {
                let t = Instant::now();
                std::hint::black_box(niles(s));
                a += t.elapsed().as_secs_f64();
            }
            if let Some((_, s)) = sql_src.get(i) {
                let t = Instant::now();
                std::hint::black_box(sql(s));
                b += t.elapsed().as_secs_f64();
            }
        }
        if r >= warm {
            nt.push(a * 1e6);
            st.push(b * 1e6);
        }
    }
    let (nmad, smad) = (mad(&nt), mad(&st));
    let (nmed, smed) = (median(&mut nt), median(&mut st));
    let nk = nmed * 1000.0 / nl as f64;
    let sk = smed * 1000.0 / sl as f64;
    let max_by = |v: &[(usize, f64)]| v.iter().map(|x| x.1).fold(0f64, f64::max);
    let max_b = |v: &[(usize, f64)]| v.iter().map(|x| x.0).max().unwrap_or(0);

    let mut out = String::new();
    out += "# E14 — check cost per KLOC, Niles against SQL+C+L\n\n";
    out += "*Written by `cargo run --release --manifest-path tools/memprobe/Cargo.toml --bin checkcost` (cycle 14, R2-05). Machine-dependent: every figure is a measurement on the host below, and only the ratio between the two checkers on one host means anything.*\n\n";
    out += "| field | value |\n|---|---|\n";
    out += &format!(
        "| commit | `{}` |\n",
        cmd(
            "git",
            &[
                "-C",
                root.to_str().unwrap(),
                "rev-parse",
                "--short=12",
                "HEAD"
            ]
        )
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
    out += &format!(
        "| worktree | {} (crates/, Cargo.toml, Cargo.lock) |\n",
        if dirty { "MODIFIED" } else { "clean" }
    );
    out += &format!(
        "| host | {} — {} CPUs |\n",
        cmd("uname", &["-srm"]),
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0)
    );
    // The compiler that built this binary (build.rs), not whatever `rustc` is on the PATH.
    out += &format!("| toolchain | `{}` |\n", env!("MEMPROBE_RUSTC"));
    out += "| checkpoint_interval | n/a (no engine runs: a checker's parse and check) |\n";
    out += &format!("| protocol | {warm} warm-up passes, then {reps} timed passes; each pass checks every file once with each checker, alternating; in-process, release build; peak heap from E18's counting allocator |\n\n");
    out += "| checker | corpus | files | lines | median pass µs (MAD) | µs per KLOC | peak heap per check, max over files, bytes | peak heap per KLOC, max over files, bytes |\n|---|---|--:|--:|--:|--:|--:|--:|\n";
    out += &format!(
        "| Niles (`nilesc check` front end) | 13 classes + 5 d14 spellings | {} | {nl} | {nmed:.0} ({nmad:.0}) | {nk:.0} | {} | {:.0} |\n",
        niles_src.len(),
        max_b(&np),
        max_by(&np)
    );
    out += &format!(
        "| SQL+C+L (`nilescheck-sql`, preamble included per file) | 13 classes + 5 d14 spellings | {} | {sl} | {smed:.0} ({smad:.0}) | {sk:.0} | {} | {:.0} |\n\n",
        sql_src.len(),
        max_b(&sp),
        max_by(&sp)
    );
    let ratio = sk / nk;
    out += &format!(
        "**Time per KLOC, SQL+C+L over Niles: {ratio:.2}×.** The pre-registered rule reports a checker whose time per KLOC is above 2× the other's as a cost, not a veto; {}.\n",
        if ratio > 2.0 {
            "SQL+C+L is above 2× Niles's"
        } else if ratio < 0.5 {
            "Niles is above 2× SQL+C+L's"
        } else {
            "neither is above 2× the other's"
        }
    );
    out += &format!(
        "\n**What a line is here.** Each SQL file is checked as the {}-line preamble followed by the defect, so most of SQL+C+L's lines are the preamble's declarations, re-checked every time; each Niles file carries its own nine-line schema. The two corpora state the same eighteen defects, and the figure is time over the lines each checker actually reads — not over the lines a programmer would write for the defect alone.\n",
        pre.lines().count()
    );
    let path = root.join("results/E14-checkcost.md");
    std::fs::write(&path, &out).expect("write");
    print!("{out}");
    eprintln!("wrote {}", path.display());
}
