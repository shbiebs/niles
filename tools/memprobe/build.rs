//! **The compiler that actually ran, not the version the manifest asks for.**
//!
//! `E18-memory.{csv,md}` called `CARGO_PKG_RUST_VERSION` the toolchain. That is the
//! `rust-version` field of `Cargo.toml` — the *minimum* this package declares it needs — and
//! it would read `1.95.0` on a machine compiling with anything at or above it. The byte
//! columns in those files are toolchain-scoped precisely because the standard library's own
//! type sizes move between compilers, so naming the wrong compiler defeats the reason the
//! column carries a toolchain at all.
//!
//! `rustc --version` at build time gives the compiler that produced this binary. At run time
//! it would give whatever is on the `PATH` of the machine invoking it, which need not be the
//! one that built it — the same reasoning as `crates/bank-bench/build.rs` uses for the commit.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=RUSTC");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let version = Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "rustc version not recorded".into());
    println!("cargo:rustc-env=MEMPROBE_RUSTC={version}");
}
