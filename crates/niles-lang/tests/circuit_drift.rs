//! **The circuit-drift guard** (cycle 15, C15-01).
//!
//! Every committed Niles program is lowered, and its circuit is compared with the one
//! recorded in `tests/circuit_drift.golden`. A lowering change that moves any committed
//! program's circuit fails this test, whether or not any answer-level test notices.
//!
//! **Why it exists.** Cycle 14's R2-06 changed the lowering six times ("Niles fix 1/5" …
//! "6/6") while measured experiments stood on it. Nothing recorded whether any *already
//! measured* program lowered differently afterwards; the cycle-15 audit had to answer that
//! by hand, building `nilesc` at `50bac5c` and at `f59cb16` and diffing 92 programs (no
//! change except the one refused on purpose). This test is that probe, kept: the rule of
//! the round is that a measured system is not changed during its own measurement without
//! re-running what it touches, and a change nobody can see cannot be re-run.
//!
//! **What is lowered.**
//!
//! - Every `.niles` file in the repository outside `target/` and the E30 corpus
//!   (`crates/syntax-study/corpus/`, whose programs are the study's measured objects and are
//!   run by its own harness). Fragments are completed the way their own tests complete
//!   them: `tests/golden/` bodies are wrapped as `sql_golden.rs` wraps them (the `.sql`
//!   bodies there too, since they are committed programs on the SQL surface),
//!   `tests/mutants/` get `mutants/schema.niles` in front, and `tests/solver_corpus/` gets
//!   `preamble.niles`.
//! - The daemon's served view shapes: the balance view `install_balance_view` compiles
//!   (with `materialize: auto` and `full`), and the statements the E27 comparator's N arm
//!   sends, each wrapped as the engine wraps a wire statement. The schema is read out of
//!   `nilestream-server/src/daemon.rs`, and every statement template is asserted to occur
//!   in the source it was copied from, so the copy cannot drift from what is served.
//!
//! **What is recorded,** per program: the diagnostic codes of parse, resolve and lower (a
//! change in what is refused is a change too), then every node in full (its operator with
//! predicates, inputs, key, arity, anchor, contract, conservation and lineage), then the
//! outputs sorted by name.
//!
//! **An intended change** re-records the file in the same commit, with the reason in the
//! commit message:
//!
//! ```sh
//! NILES_BLESS_CIRCUITS=1 cargo test -p niles-lang --test circuit_drift
//! ```

use niles_lang::{lower, parser, resolve};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

const GOLDEN: &str = "crates/niles-lang/tests/circuit_drift.golden";
const SERVE: &str = "serve { consistency: snapshot, materialize: auto }";

/// Lower one program and describe it: codes, then the circuit.
fn describe(src: &str) -> String {
    let (prog, mut d) = parser::parse_program(src);
    let (cat, rd) = resolve::resolve_program(&prog, 0);
    d.extend(rd);
    let (lowered, ld) = lower::lower_program(&prog, &cat);
    d.extend(ld);
    let mut codes: Vec<String> = d.items.iter().map(|x| x.code.to_string()).collect();
    codes.sort();
    let c = &lowered.circuit;
    let mut s = format!(
        "codes: [{}]\ncircuit ({} nodes):\n",
        codes.join(", "),
        c.nodes.len()
    );
    // Every node in full, in id order, not `explain()`'s summary: `explain` prints `filter`
    // without its predicate, so F7's self-join defect (`Column(2) = Column(2)` for
    // `q.cur = p.cur`) would have lowered to an identical explain line. The operator's
    // `Debug` carries its predicates, keys and aggregates. The `Checked` fields are read
    // with `peek`, which records no access; outputs are sorted, `outputs` being a `HashMap`.
    for n in &c.nodes {
        s.push_str(&format!(
            "  {} {:?} in={:?} key={:?} arity={} anchor={:?} contract={:?} conserving={:?} lineage={:?} // {}\n",
            n.id,
            n.op,
            n.inputs,
            n.key,
            n.arity,
            n.anchor.peek(),
            n.contract.peek(),
            n.conservation_transparent.peek(),
            n.lineage.peek(),
            n.label
        ));
    }
    let mut outs: Vec<_> = c.outputs.iter().collect();
    outs.sort();
    for (name, id) in outs {
        s.push_str(&format!("  output {name} = node {id}\n"));
    }
    s
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn niles_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if name == "target" || name.starts_with('.') {
                continue;
            }
            niles_files(&p, out);
        } else if name.ends_with(".niles") {
            out.push(p);
        }
    }
}

/// Every program this test lowers, as (name, source), in a fixed order.
fn programs() -> Vec<(String, String)> {
    let root = root();
    let mut files = Vec::new();
    niles_files(&root, &mut files);
    let corpus = root.join("crates/syntax-study/corpus");
    let golden = root.join("crates/niles-lang/tests/golden");
    let mutants = root.join("crates/niles-lang/tests/mutants");
    let solver = root.join("crates/niles-lang/tests/solver_corpus");

    let mut out = Vec::new();
    for f in files {
        if f.starts_with(&corpus) {
            continue;
        }
        let rel = f
            .strip_prefix(&root)
            .expect("under the root")
            .display()
            .to_string();
        let src = read(&f);
        let dir = f.parent().expect("a file has a parent");
        let is_schema = f.file_name().is_some_and(|n| n == "schema.niles");
        let full = if dir == golden && !is_schema {
            format!(
                "{}\nview golden = {src}\n    {SERVE};\n",
                read(&golden.join("schema.niles"))
            )
        } else if dir == mutants && !is_schema {
            format!("{}\n{src}\n", read(&mutants.join("schema.niles")))
        } else if dir == solver && f.file_name().is_some_and(|n| n != "preamble.niles") {
            format!("{}\n{src}\n", read(&solver.join("preamble.niles")))
        } else {
            src
        };
        out.push((rel, full));
    }

    // The SQL surface's golden bodies.
    let mut sqls: Vec<PathBuf> = std::fs::read_dir(&golden)
        .expect("golden dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "sql"))
        .collect();
    sqls.sort();
    let schema = read(&golden.join("schema.niles"));
    for f in sqls {
        let body = read(&f);
        let rel = f.strip_prefix(&root).expect("root").display().to_string();
        out.push((
            rel,
            format!(
                "{schema}\nview golden = sql {{ {} }} {SERVE};\n",
                body.trim()
            ),
        ));
    }

    out.extend(daemon_shapes());
    out
}

/// Take a `pub const NAME: &str = "\ … ";` literal out of a Rust source file.
fn rust_str_const(src: &str, name: &str) -> String {
    let head = format!("pub const {name}: &str = \"\\\n");
    let start = src
        .find(&head)
        .unwrap_or_else(|| panic!("`{name}` is no longer written as `{head}`"))
        + head.len();
    let len = src[start..]
        .find("\";\n")
        .unwrap_or_else(|| panic!("`{name}` has no closing `\";`"));
    let lit = &src[start..start + len];
    assert!(
        !lit.contains('\\') && !lit.contains('"'),
        "`{name}` now carries escapes; read it with a real parser"
    );
    lit.to_string()
}

/// The shapes the daemon lowers when it serves the E27 workload.
fn daemon_shapes() -> Vec<(String, String)> {
    let root = root();
    let daemon = read(&root.join("crates/nilestream-server/src/daemon.rs"));
    let engine = read(&root.join("crates/nilestream-server/src/rev_engine.rs"));
    let arms = read(&root.join("crates/comparator/src/arms.rs"));
    let schema = rust_str_const(&daemon, "DEFAULT_SCHEMA");

    // Copied from the sources named; each must still occur there, verbatim.
    let balance = "select acct, cur, sum(amt) from postings group by acct, cur";
    assert!(
        engine.contains(&format!("const BALANCE: &str = \"{balance}\";")),
        "rev_engine.rs's balance view is no longer `{balance}`: update this test"
    );
    let wire_wrapper = "view __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};";
    assert!(
        engine.contains(wire_wrapper),
        "rev_engine.rs no longer wraps a wire statement as `{wire_wrapper}`: update this test"
    );
    // (template in arms.rs, the instance lowered here)
    let n_arm = [
        (
            "select acct, sum(amt) from postings as of system time {e} where acct = 1 and cur = 0 group by acct",
            "select acct, sum(amt) from postings as of system time 5 where acct = 1 and cur = 0 group by acct",
        ),
        (
            "select acct, cur, sum(amt) from postings{asof} where acct = {a} and cur = {c} group by acct, cur",
            "select acct, cur, sum(amt) from postings where acct = 7 and cur = 0 group by acct, cur",
        ),
        (
            "select acct, cur, sum(amt) from postings{asof} where acct = {a} and cur = {c} group by acct, cur",
            "select acct, cur, sum(amt) from postings as of system time 5 where acct = 7 and cur = 0 group by acct, cur",
        ),
        (
            "select acct, sum(amt) from postings{asof} where acct = {a} group by acct",
            "select acct, sum(amt) from postings where acct = 7 group by acct",
        ),
        (
            "select acct, sum(amt) from postings where cur = {c} group by acct order by sum(amt) desc limit 10",
            "select acct, sum(amt) from postings where cur = 0 group by acct order by sum(amt) desc limit 10",
        ),
        (
            "select acct, cur, sum(amt) from postings group by acct, cur",
            "select acct, cur, sum(amt) from postings group by acct, cur",
        ),
    ];

    let mut out = Vec::new();
    for m in ["auto", "full"] {
        out.push((
            format!("daemon: balance view, materialize {m}"),
            format!(
                "{schema}\nview ledger_balance = sql {{ {balance} }} serve {{ consistency: snapshot, materialize: {m} }};\n"
            ),
        ));
    }
    for (template, instance) in n_arm {
        assert!(
            arms.contains(template),
            "the comparator's N arm no longer sends `{template}`: update this test"
        );
        out.push((
            format!("daemon: {instance}"),
            format!("{schema}\nview __wire_result = sql {{ {instance} }} {SERVE};\n"),
        ));
    }
    out
}

fn render() -> String {
    let mut s = String::from(
        "# Generated by crates/niles-lang/tests/circuit_drift.rs; do not edit by hand.\n\
         # Re-record an intended change: NILES_BLESS_CIRCUITS=1 cargo test -p niles-lang --test circuit_drift\n",
    );
    for (name, src) in programs() {
        s.push_str(&format!("\n=== {name}\n{}", describe(&src)));
    }
    s
}

#[test]
fn every_committed_program_lowers_to_its_recorded_circuit() {
    let now = render();
    let path = root().join(GOLDEN);
    if std::env::var_os("NILES_BLESS_CIRCUITS").is_some() {
        std::fs::write(&path, &now).expect("write the golden file");
        return;
    }
    let was = std::fs::read_to_string(&path).unwrap_or_default();
    if was == now {
        return;
    }
    // Name the programs that moved, and show the first difference.
    let split = |s: &str| -> Vec<(String, String)> {
        s.split("\n=== ")
            .skip(1)
            .map(|b| {
                let (h, rest) = b.split_once('\n').unwrap_or((b, ""));
                (h.to_string(), rest.to_string())
            })
            .collect()
    };
    let (a, b) = (split(&was), split(&now));
    let mut moved = Vec::new();
    for (name, body) in &b {
        match a.iter().find(|(n, _)| n == name) {
            None => moved.push(format!("new:     {name}")),
            Some((_, old)) if old != body => moved.push(format!("changed: {name}")),
            _ => {}
        }
    }
    for (name, _) in &a {
        if !b.iter().any(|(n, _)| n == name) {
            moved.push(format!("gone:    {name}"));
        }
    }
    let first = b
        .iter()
        .find_map(|(name, body)| {
            let old = a.iter().find(|(n, _)| n == name).map(|(_, o)| o.as_str())?;
            if old == body {
                return None;
            }
            let line = old
                .lines()
                .zip(body.lines())
                .position(|(x, y)| x != y)
                .unwrap_or_else(|| old.lines().count().min(body.lines().count()));
            Some(format!(
                "first difference, in `{name}`, line {}:\n  recorded: {}\n  now:      {}",
                line + 1,
                old.lines().nth(line).unwrap_or("<end>"),
                body.lines().nth(line).unwrap_or("<end>")
            ))
        })
        .unwrap_or_default();
    panic!(
        "{} lowered program(s) differ from {GOLDEN}:\n{}\n\n{first}\n\n\
         If the change is intended, re-record the file in the same commit and say why:\n\
         NILES_BLESS_CIRCUITS=1 cargo test -p niles-lang --test circuit_drift",
        moved.len(),
        moved.join("\n")
    );
}

#[test]
fn the_guard_lowers_what_it_says_it_lowers() {
    // A guard over an empty set passes forever. The floor is the count at this card's head:
    // 90 `.niles` files at `f59cb16` (the audit's 92 were those 90 plus `gbs.niles`, which
    // lives in the other repository, and one file of daemon shapes), plus the two NL0257
    // mutants C15-01 adds. It is a floor, so a walker that silently stopped descending
    // fails rather than reports a quiet tree; the SQL golden bodies and the eight daemon
    // shapes come on top.
    let ps = programs();
    let niles = ps.iter().filter(|(n, _)| n.ends_with(".niles")).count();
    let daemon = ps.iter().filter(|(n, _)| n.starts_with("daemon: ")).count();
    assert!(
        niles >= 92,
        "only {niles} .niles programs found; there were 92 at C15-01"
    );
    assert_eq!(
        daemon, 8,
        "two balance-view contracts and six N-arm statements"
    );
    // And the lowering must actually produce circuits, not refuse everything: the daemon's
    // shapes are the programs the engine serves, so each must lower cleanly.
    for (name, src) in ps.iter().filter(|(n, _)| n.starts_with("daemon: ")) {
        let d = describe(src);
        assert!(
            d.starts_with("codes: []\n"),
            "{name} no longer lowers cleanly:\n{d}"
        );
    }
}
