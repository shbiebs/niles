//! D14, `mem::forget`, with a destructor: as `d14_5_forget.rs`, but `Hold` implements `Drop`
//! (the drop bomb of round 1's option (a)), so clippy's `forget_non_drop` no longer applies.
//! Rust arm (round 1's class-B probe, measured again in the run): `#![deny(unused_must_use)]`,
//! then `clippy-driver -D warnings`.
#![deny(unused_must_use)]

#[must_use = "a hold must be resolved"]
pub struct Hold(pub i64);

impl Drop for Hold {
    // A drop bomb would panic here when the hold was not resolved; the body is irrelevant
    // to the static question this file asks.
    fn drop(&mut self) {}
}

pub fn hold(amount: i64) -> Hold {
    Hold(amount)
}

pub fn resolve(h: Hold) -> i64 {
    h.0
}

pub fn f() {
    let h = hold(2000);
    std::mem::forget(h);
}

fn main() {
    f();
    println!("{}", resolve(hold(5)));
}
