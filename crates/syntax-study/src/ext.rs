//! **The external surfaces** (design §2): PRQL 0.13.14 and Soufflé 2.5, run as the binaries the
//! author approved on 2026-09-29 (`/opt/arms`, or `PRQLC` / `SOUFFLE`). A missing binary is a
//! blocked surface, reported as absent — never a verdict on a program.

use crate::data::Dataset;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const SCHEMA_DL: &str = include_str!("../corpus/schema/schema.dl");

pub fn prqlc() -> Option<PathBuf> {
    let p = std::env::var_os("PRQLC")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/opt/arms/prqlc/prqlc"));
    p.is_file().then_some(p)
}

pub fn souffle() -> Option<PathBuf> {
    let p = std::env::var_os("SOUFFLE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/opt/arms/souffle/usr/bin/souffle"));
    p.is_file().then_some(p)
}

/// The version line of an external binary, for provenance.
pub fn version(bin: &Path) -> String {
    Command::new(bin)
        .arg("--version")
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .find(|l| !l.trim().is_empty() && !l.starts_with('-'))
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .unwrap_or_default()
}

/// `prqlc compile --target sql.postgres`: the SQL, or the compiler's error text.
pub fn prql_compile(bin: &Path, program: &str) -> Result<String, String> {
    let mut child = Command::new(bin)
        .args([
            "compile",
            "--hide-signature-comment",
            "--target",
            "sql.postgres",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run prqlc: {e}"))?;
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(program.as_bytes())
        .map_err(|e| e.to_string())?;
    let o = child.wait_with_output().map_err(|e| e.to_string())?;
    if o.status.success() {
        Ok(String::from_utf8_lossy(&o.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
    }
}

/// The dataset as Soufflé fact files in `dir` (tab-separated, one file per input relation).
pub fn write_facts(d: &Dataset, dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let day0 = crate::niles::day0();
    let w = |name: &str, rows: Vec<String>| std::fs::write(dir.join(name), rows.join("\n") + "\n");
    w(
        "parties.facts",
        d.parties.iter().map(|p| p.id.to_string()).collect(),
    )?;
    w(
        "parent.facts",
        d.parties
            .iter()
            .filter_map(|p| p.parent.map(|x| format!("{}\t{x}", p.id)))
            .collect(),
    )?;
    w(
        "accounts.facts",
        d.accounts
            .iter()
            .map(|a| format!("{}\t{}\t{}", a.id, a.owner, a.desk))
            .collect(),
    )?;
    w(
        "postings.facts",
        d.postings
            .iter()
            .map(|p| {
                format!(
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    p.txn,
                    p.acct,
                    p.cur,
                    p.amt,
                    p.epoch,
                    day0 + p.value_date
                )
            })
            .collect(),
    )?;
    w(
        "holds.facts",
        d.holds
            .iter()
            .map(|h| {
                format!(
                    "{}\t{}\t{}\t{}\t{}",
                    h.id, h.acct, h.cur, h.amount, h.open as u8
                )
            })
            .collect(),
    )?;
    Ok(())
}

/// Why a Soufflé run produced no answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DlErr {
    /// Soufflé's front end refused the program (parse or semantic error): before evaluating.
    Refused(String),
    /// It evaluated and failed.
    Failed(String),
}

/// Run a Datalog program (after the schema) over the facts; the rows of `answer`.
pub fn souffle_run(
    bin: &Path,
    facts: &Path,
    work: &Path,
    program: &str,
) -> Result<Vec<Vec<String>>, DlErr> {
    std::fs::create_dir_all(work).map_err(|e| DlErr::Failed(e.to_string()))?;
    let prog = work.join("p.dl");
    let out = work.join("out");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| DlErr::Failed(e.to_string()))?;
    std::fs::write(&prog, format!("{SCHEMA_DL}\n{program}"))
        .map_err(|e| DlErr::Failed(e.to_string()))?;
    let o = Command::new(bin)
        .arg("-F")
        .arg(facts)
        .arg("-D")
        .arg(&out)
        .arg(&prog)
        .output()
        .map_err(|e| DlErr::Failed(format!("cannot run souffle: {e}")))?;
    let err = String::from_utf8_lossy(&o.stderr).to_string();
    if !o.status.success() || err.contains("Error:") {
        // Soufflé reports front-end errors as `Error: …` with a source location and exits
        // before evaluating; anything else is an evaluation failure.
        return Err(if err.contains("Error:") {
            DlErr::Refused(err.trim().to_string())
        } else {
            DlErr::Failed(err.trim().to_string())
        });
    }
    let text = std::fs::read_to_string(out.join("answer.csv")).unwrap_or_default();
    Ok(text
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.split('\t').map(String::from).collect())
        .collect())
}
