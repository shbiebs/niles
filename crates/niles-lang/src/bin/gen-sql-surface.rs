//! Regenerate the SQL ↔ Niles status sentence in `docs/SPEC-LANGUAGE.md` (L-5 / L-23).
//!
//! # Why this is a program and not a paragraph
//!
//! The paragraph it replaces said "**Status: Built** for the declared fragment" and had said
//! so since before the fragment was checkable. Meanwhile `sql_surface.rs` marked
//! `SELECT a, b FROM t` as `Lowered` while the lowering emitted no projection at all, and
//! `DISTINCT` as `Lowered` while the keyword was parsed and ignored. Three statements about
//! one thing, none of them derived from the others, and the two that were wrong were wrong
//! in the direction that flatters the artifact.
//!
//! There is now one source — `sql_surface::MAPPING`, whose every non-excluded row is backed
//! by a case in `tests/golden/` (enforced by `sql_golden.rs`) — and this program writes the
//! sentence from it.
//!
//! ```sh
//! cargo run -q -p niles-lang --bin gen-sql-surface            # rewrite the file
//! cargo run -q -p niles-lang --bin gen-sql-surface -- --check # exit 1 if it would change
//! ```
//!
//! `--check` is what the gate runs, so a status that drifts from the code fails a build
//! rather than being noticed in review.

use niles_lang::sql_surface::{Status, MAPPING};

const BEGIN: &str = "<!-- BEGIN:sql-surface -->";
const END: &str = "<!-- END:sql-surface -->";

fn main() -> std::process::ExitCode {
    let check = std::env::args().any(|a| a == "--check");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/SPEC-LANGUAGE.md");

    let (mut equivalent, mut lowered, mut refused, mut untested, mut specified, mut excluded) =
        (0, 0, 0, 0, 0, 0);
    for m in MAPPING {
        match m.status {
            Status::Equivalent => equivalent += 1,
            Status::Lowered => lowered += 1,
            Status::Refused(_) => refused += 1,
            Status::Untested(_) => untested += 1,
            Status::Specified => specified += 1,
            Status::Excluded(_) => excluded += 1,
        }
    }
    let total = MAPPING.len();

    let generated = format!(
        "{BEGIN}\n\
         *Status, generated from `sql_surface::MAPPING` by \
         `cargo run -q -p niles-lang --bin gen-sql-surface`. Do not edit between the \
         markers.*\n\n\
         The stated fragment has **{total} forms**. Of those, **{equivalent}** are\n\
         *equivalent* — the SQL and pipeline spellings denote the same Z-set on the golden\n\
         corpus's dataset, which is a stronger claim than the structural circuit equality\n\
         this line used to rest on: two circuits can differ and denote the same thing, and\n\
         agree while both are wrong. **{lowered}** are *lowered* with a golden case fixing\n\
         what they denote but written in one surface only. **{refused}** are *refused*, each\n\
         with the diagnostic code that refuses it — that is what narrowing the fragment looks\n\
         like from inside the compiler. **{untested}** are lowered with nothing checking what\n\
         they compute, and say so. **{specified}** are specified and not built.\n\
         **{excluded}** are deliberate exclusions, each carrying its reason.\n\n\
         Every non-excluded row names a case in `crates/niles-lang/tests/golden/`, and\n\
         `the_status_of_every_form_is_backed_by_a_corpus_case` fails the build if one does\n\
         not.\n\
         {END}"
    );

    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("gen-sql-surface: cannot read {path}: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    let (Some(b), Some(e)) = (src.find(BEGIN), src.find(END)) else {
        eprintln!(
            "gen-sql-surface: {path} has no `{BEGIN}` … `{END}` block. Add one where the \
             status sentence belongs."
        );
        return std::process::ExitCode::from(2);
    };
    let updated = format!("{}{}{}", &src[..b], generated, &src[e + END.len()..]);

    if updated == src {
        if !check {
            println!("gen-sql-surface: up to date");
        }
        return std::process::ExitCode::SUCCESS;
    }
    if check {
        eprintln!(
            "gen-sql-surface: SPEC-LANGUAGE.md is out of date with `sql_surface::MAPPING`.\n\
             Run `cargo run -q -p niles-lang --bin gen-sql-surface` and commit the result."
        );
        return std::process::ExitCode::from(1);
    }
    match std::fs::write(path, updated) {
        Ok(()) => {
            println!("gen-sql-surface: wrote {path}");
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("gen-sql-surface: cannot write {path}: {e}");
            std::process::ExitCode::from(2)
        }
    }
}
