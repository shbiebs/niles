//! One file per (series, size, seed), so a sweep that spans sessions resumes where it stopped
//! and the report is rendered from every point on disk, never from memory.
//!
//! Line-oriented and tab-separated; the first field names the record. Written once, when the
//! point finishes; a point that failed is written too, with its failures, so a refusal is on
//! disk and not only in a terminal.

use crate::run::{pct, PointResult};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

fn clean(s: &str) -> String {
    s.replace(['\t', '\n'], " ")
}

/// Where and with what a point was measured: recorded in every point file at the moment it
/// is written, so a report rendered later cannot attribute a number to the wrong build.
pub fn provenance(repo: &Path) -> Vec<(String, String)> {
    let sh = |cmd: &str| {
        std::process::Command::new("sh")
            .args(["-c", cmd])
            .current_dir(repo)
            .output()
            .ok()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .replace(['\t', '\n'], " ")
            })
            .unwrap_or_default()
    };
    vec![
        ("commit".into(), sh("git rev-parse HEAD")),
        (
            "worktree".into(),
            sh("if git diff --quiet HEAD -- crates Cargo.toml Cargo.lock; then echo clean; else echo MODIFIED; fi"),
        ),
        ("host".into(), sh("uname -srm")),
        ("cpus".into(), sh("nproc")),
        ("mem_mib".into(), sh("free -m | awk '/^Mem:/{print $2}'")),
        ("toolchain".into(), sh("cargo --version")),
        ("postgres".into(), sh("/usr/lib/postgresql/16/bin/postgres --version")),
        ("date".into(), sh("date -u +%Y-%m-%dT%H:%M:%SZ")),
    ]
}

pub fn write(
    path: &Path,
    series: &str,
    p: &PointResult,
    prov: &[(String, String)],
) -> std::io::Result<()> {
    let mut s = String::new();
    let _ = writeln!(s, "meta\tseries\t{series}");
    for (k, v) in prov {
        let _ = writeln!(s, "meta\t{k}\t{v}");
    }
    let _ = writeln!(s, "meta\taccounts\t{}", p.accounts);
    let _ = writeln!(s, "meta\tseed\t{}", p.seed);
    let _ = writeln!(s, "meta\tkeys\t{}", p.keys);
    let _ = writeln!(s, "meta\tbudget\t{}", p.budget);
    let _ = writeln!(s, "meta\thistory_txns\t{}", p.history_txns);
    let _ = writeln!(s, "meta\tbatches\t{}", p.batches);
    let _ = writeln!(s, "meta\tuniverse_legs\t{}", p.universe_checksum.0);
    let _ = writeln!(s, "meta\tuniverse_sha256\t{}", p.universe_checksum.1);
    let _ = writeln!(s, "meta\tend_legs\t{}", p.end_checksum.0);
    let _ = writeln!(s, "meta\tend_sha256\t{}", p.end_checksum.1);
    for (arm, (rep, ck, end)) in &p.loads {
        let _ = writeln!(
            s,
            "load\t{arm}\t{:.3}\t{}\t{}\t{}\t{}\t{}",
            rep.seconds,
            ck.0,
            ck.1,
            end.0,
            end.1,
            clean(&rep.notes.join("; "))
        );
    }
    for (arm, b) in &p.pss_loaded {
        let _ = writeln!(s, "pss_loaded\t{arm}\t{b}");
    }
    for (arm, st) in &p.state_end {
        for (k, v) in st {
            let _ = writeln!(s, "state\t{arm}\t{}\t{v}", clean(k));
        }
    }
    for (arm, v) in &p.not_run {
        for (q, why) in v {
            let _ = writeln!(s, "notrun\t{arm}\t{q}\t{}", clean(why));
        }
    }
    for f in &p.failures {
        let _ = writeln!(s, "failure\t{}", clean(f));
    }
    let mut pooled: BTreeMap<(String, String), Vec<f64>> = BTreeMap::new();
    for (arm, run, out) in &p.runs {
        if out.divergence.count > 0 {
            let _ = writeln!(
                s,
                "div\t{arm}\t{run}\t{}\t{}",
                out.divergence.count,
                clean(&out.divergence.examples.join(" || "))
            );
        }
        for e in &out.errors {
            let _ = writeln!(s, "err\t{arm}\t{run}\t{}", clean(e));
        }
        for (m, v) in &out.values {
            let _ = writeln!(s, "val\t{arm}\t{run}\t{m}\t{v}");
        }
        if *run >= 0 {
            for (fam, v) in &out.samples {
                pooled
                    .entry((arm.clone(), fam.clone()))
                    .or_default()
                    .extend(v);
            }
        }
    }
    for ((arm, fam), v) in &pooled {
        let _ = writeln!(
            s,
            "pooled\t{arm}\t{fam}\t{}\t{}\t{}\t{}",
            v.len(),
            pct(v, 50.0),
            pct(v, 99.0),
            pct(v, 99.9)
        );
    }
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(path, s)
}

/// A point read back from disk: the records by kind, fields as strings.
#[derive(Debug, Clone, Default)]
pub struct Stored {
    pub meta: BTreeMap<String, String>,
    pub records: Vec<Vec<String>>,
}

impl Stored {
    pub fn read(path: &Path) -> std::io::Result<Stored> {
        let text = std::fs::read_to_string(path)?;
        let mut st = Stored::default();
        for line in text.lines() {
            let f: Vec<String> = line.split('\t').map(String::from).collect();
            if f.len() >= 3 && f[0] == "meta" {
                st.meta.insert(f[1].clone(), f[2].clone());
            } else if !f.is_empty() {
                st.records.push(f);
            }
        }
        Ok(st)
    }
    pub fn kind<'a>(&'a self, k: &'a str) -> impl Iterator<Item = &'a Vec<String>> + 'a {
        self.records.iter().filter(move |r| r[0] == k)
    }
    pub fn num(&self, k: &str) -> u64 {
        self.meta.get(k).and_then(|v| v.parse().ok()).unwrap_or(0)
    }
    pub fn series(&self) -> String {
        self.meta.get("series").cloned().unwrap_or_default()
    }
}
