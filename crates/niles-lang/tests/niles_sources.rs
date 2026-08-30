//! Every `.niles` file in the repository must compile, verify, and stay compiling.
//!
//! This is the test that makes "Nilestream is written in Niles, as far as the bootstrap
//! reaches" a checkable statement rather than an aspiration. It walks `niles/` and
//! `examples/`, runs the full front end plus lowering plus the IR verifier over each file,
//! and fails on any error.
//!
//! It also enforces the reservation policy against real code rather than against a test
//! fixture: `niles/std/bank.niles` uses `from`, `to`, `order` and `limit` as ordinary
//! parameter and column names, and that file is where the policy is either true or a
//! slogan. It was a slogan until this file was written — `from` was reserved, and the most
//! natural parameter name in a transfer function did not compile.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn niles_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            niles_files(&p, out);
        } else if p.extension().map_or(false, |x| x == "niles") {
            out.push(p);
        }
    }
}

struct Outcome {
    diags: String,
    errors: usize,
    verified: bool,
    verify_report: String,
    proved: usize,
    obligations: usize,
    views: usize,
}

fn compile(path: &Path) -> Outcome {
    let src = std::fs::read_to_string(path).unwrap();
    let name = path.file_name().unwrap().to_string_lossy().to_string();
    let (prog, mut d) = niles_lang::parser::parse_program(&src);
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    d.extend(rd);
    let (report, td) = niles_lang::typecheck::check_program(&prog, &cat);
    d.extend(td);
    let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
    d.extend(ld);
    let vr = niles_ir::verify::verify(&lowered.circuit);
    Outcome {
        diags: d.render(&src, &name),
        errors: d.error_count(),
        verified: vr.is_ok(),
        verify_report: vr.render(),
        proved: report.conservation_proved,
        obligations: report.runtime_obligations,
        views: cat.views.len(),
    }
}

#[test]
fn every_niles_source_in_the_repository_compiles_and_verifies() {
    let root = repo_root();
    let mut files = Vec::new();
    niles_files(&root.join("niles"), &mut files);
    niles_files(&root.join("examples"), &mut files);
    files.sort();
    assert!(files.len() >= 3, "expected the Niles sources to be found, got {files:?}");

    let mut total_views = 0usize;
    let mut total_proved = 0usize;
    for f in &files {
        let o = compile(f);
        assert_eq!(o.errors, 0, "{} does not compile:\n{}", f.display(), o.diags);
        assert!(o.verified, "{} does not verify:\n{}", f.display(), o.verify_report);
        total_views += o.views;
        total_proved += o.proved;
    }
    assert!(total_views >= 10, "only {total_views} views across the sources");
    assert!(total_proved >= 5, "only {total_proved} conservation obligations proved");
}

#[test]
fn the_banking_layer_needs_no_kernel_construct() {
    // Section 6.11's claim: banking is a library over a general relational core. The test
    // is negative and therefore weak on its own, but it is the strongest form available:
    // the file uses only constructs any Niles program may use, and it compiles.
    let o = compile(&repo_root().join("niles/std/bank.niles"));
    assert_eq!(o.errors, 0, "{}", o.diags);
    assert!(o.proved >= 5, "the domain layer should prove its conservation obligations, proved {}", o.proved);
    assert_eq!(o.obligations, 0, "and discharge none to the runtime");
}

#[test]
fn the_reservation_policy_survives_contact_with_real_code() {
    // `bank.niles` uses `from` as a parameter name in three functions. Before the registry
    // was corrected this did not compile, which is the whole argument for keeping the
    // reserved set small: the most natural name for the source account in a transfer is
    // `from`, and a language that forbids it is a language a bank rewrites its code for.
    let src = std::fs::read_to_string(repo_root().join("niles/std/bank.niles")).unwrap();
    assert!(src.contains("fn transfer(from:"), "the test's premise has moved");
    let o = compile(&repo_root().join("niles/std/bank.niles"));
    assert_eq!(o.errors, 0, "{}", o.diags);
}

#[test]
fn the_engine_observes_itself_through_its_own_language() {
    // The first real bootstrap step: Nilestream's telemetry views are Niles source,
    // compiled by the same compiler and executed by the same runtime as a user query.
    let o = compile(&repo_root().join("niles/nilestream/observability.niles"));
    assert_eq!(o.errors, 0, "{}", o.diags);
    assert!(o.verified, "{}", o.verify_report);
    assert!(o.views >= 4, "expected the telemetry views, found {}", o.views);
}
