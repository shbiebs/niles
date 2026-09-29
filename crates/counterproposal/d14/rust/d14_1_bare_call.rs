//! D14, bare call: the hold is made in statement position; its value is never bound.
//! Rust arm (round 1's class-B probe, measured again in the run): `#![deny(unused_must_use)]`,
//! then `clippy-driver -D warnings`.
#![deny(unused_must_use)]

#[must_use = "a hold must be resolved"]
pub struct Hold(pub i64);

pub fn hold(amount: i64) -> Hold {
    Hold(amount)
}

pub fn resolve(h: Hold) -> i64 {
    h.0
}

pub fn f() {
    hold(2000);
}

fn main() {
    f();
    println!("{}", resolve(hold(5)));
}
