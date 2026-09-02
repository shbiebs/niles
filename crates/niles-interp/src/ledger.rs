//! **The ledger the interpreter posts to, and the bytes a posting set encodes to.**
//!
//! This module exists so that a Niles function can be *executed* rather than only inspected.
//! Until T-18, `nilesc` could check a schema, verify its circuit and print the *shape* of a
//! function's postings — the sequence of legs, each with a direction, a parameter name and an
//! amount — and `gbs/crates/gbs-products/tests/conformance.rs` compared that shape against the
//! Rust product's. A shape comparison catches a leg dropped, a sign flipped and an amount
//! changed, and it cannot catch two implementations that agree on all of those and disagree
//! about the document they produce.
//!
//! What this module adds is the ability to run the function and hand back **the same byte
//! string the other implementation would hash**. Agreement then means agreement, and the
//! comparison is a comparison of the artefact rather than of a rendering of it.
//!
//! # Builtins, not language features
//!
//! `debit`, `credit`, `post` and `acct` are **builtins** — entries in the interpreter's free
//! function table, like `print` and `len`. They are not new syntax, they are not in the
//! grammar, and `crates/niles-lang` does not know they exist. That boundary is the T-18
//! guardrail and it is worth stating why it is the right one: `txn`, `debit`, `credit` and
//! `post` are already Niles *forms*, checked statically by the effect calculus. What was
//! missing was a *dynamic semantics* for them, and a dynamic semantics is what an interpreter
//! is. Adding grammar here would mean the language the interpreter runs is not the language
//! the compiler checks.
//!
//! # The canonical encoding is written out twice, on purpose
//!
//! [`encode`] below and `gbs_kernel::encode::encode_sealed` are two independent
//! implementations of one normative format, in two repositories that share no code. That is
//! not duplication to be removed. A conformance test comparing two callers of one encoder
//! proves that the encoder is a function; comparing two encoders proves that the *format* is
//! agreed, which is the property a hash chain across implementations actually needs.
//!
//! The format, quoted from `gbs-kernel/src/encode.rs`, which is normative:
//!
//! ```text
//! set     := u32 txn_len | txn bytes | u32 entry_count | entry*
//! entry   := u64 id
//!          | u32 account_len | account bytes
//!          | amount
//!          | stamp
//!          | u32 narrative_len | narrative bytes
//!          | u8 has_tag | (u32 tag_len | tag bytes)?
//! amount  := i128 minor (16 bytes, two's complement)
//!          | u32 currency_len | currency bytes
//!          | u32 scale
//! stamp   := u64 epoch | i64 value_date (two's complement)
//! ```
//!
//! Big-endian throughout. Entries in the order the set holds them, never sorted: leg order is
//! part of a posting set's identity.
//!
//! # What a Niles function cannot say, and what that costs
//!
//! Two fields of the format have no Niles form at all:
//!
//! * **`id`** — the kernel numbers entries; Niles has no entry identity. Every leg produced
//!   here is numbered `0`.
//! * **`narrative`** — a per-leg free-text annotation. `gbs-products` sets one on most legs
//!   ("drawdown under fac-1"); the schema has no syntax for it. Every leg produced here
//!   carries the empty string.
//!
//! Both are therefore normalised away before a conformance comparison, and the second is a
//! genuine gap rather than a formality — see `MISMATCH-T-18-normalised-fields` in the build
//! log. The alternative would have been to invent a narrative syntax, which is a language
//! change to make a test pass, and which is the thing this whole review exists to catch.

use std::collections::BTreeMap;

/// One leg of a posting set: the interpreter's `Entry`.
///
/// Deliberately the same fields as `gbs_kernel::Entry`, minus the two Niles cannot express.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leg {
    /// The kernel's entry identity. Always `0` here; Niles has no form for it.
    pub id: u64,
    pub account: String,
    /// Signed minor units. **A debit is negative**, matching the kernel: the verb implies the
    /// sign in Niles and the sign implies the verb in Rust, and this is the one place the two
    /// representations genuinely differ.
    pub minor: i128,
    pub currency: String,
    pub scale: u32,
    pub epoch: u64,
    pub value_date: i64,
    /// Always empty here; Niles has no form for it.
    pub narrative: String,
    /// A consumed linear tag. Always `None` here; `hold`/`resolve` are outside the subset.
    pub consumes: Option<String>,
}

/// A sealed posting set: an identity and its legs, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    pub txn: String,
    pub epoch: u64,
    pub legs: Vec<Leg>,
}

/// What a posting set can be refused for.
///
/// One variant, because the interpreter enforces exactly one rule: the commit rule. Everything
/// else a real ledger checks — the chart, the frontier, idempotency across restarts — is state
/// this interpreter does not have, and inventing an answer for it would be worse than not
/// answering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Some currency in the set does not sum to zero.
    Unbalanced { currency: String, residual: i128 },
    /// Two legs of one set disagree about a currency's scale.
    ScaleConflict {
        currency: String,
        first: u32,
        then: u32,
    },
    /// `post` outside a `txn` block.
    NoTransaction,
    /// An arithmetic overflow while summing a currency. Checked rather than wrapped: a set
    /// that conserves only by wraparound is not a set that conserves.
    Overflow { currency: String },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::Unbalanced { currency, residual } => write!(
                f,
                "the set does not conserve: {currency} sums to {residual} rather than zero"
            ),
            Refusal::ScaleConflict {
                currency,
                first,
                then,
            } => write!(
                f,
                "two legs disagree about {currency}'s scale: {first} then {then}"
            ),
            Refusal::NoTransaction => {
                f.write_str("`post` outside a `txn` block: there is nothing to seal it into")
            }
            Refusal::Overflow { currency } => write!(
                f,
                "summing {currency} overflowed; a set that conserves only by wraparound does not conserve"
            ),
        }
    }
}

/// The in-memory ledger a `nilesc run` posts to.
///
/// An append-only vector and an epoch counter. It is not a Nilestream ledger and does not
/// pretend to be one: there is no segment, no hash chain, no idempotency window that survives
/// anything. What it is for is giving `txn` a place to seal into, so that a function can be
/// executed and its output compared.
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    /// Accounts declared by the `--ledger` fixture: name → (currency, scale).
    ///
    /// Present so an unknown account can be *reported* rather than accepted. A run against an
    /// empty declaration accepts anything, which is the honest behaviour when nobody said what
    /// exists.
    declared: BTreeMap<String, (String, u32)>,
    /// Opening balances, by `(account, currency)`.
    opening: BTreeMap<(String, String), i128>,
    /// Everything sealed, in order.
    pub sealed: Vec<Posted>,
    /// The epoch the next sealed set takes. One-based, matching GBS's convention that an
    /// empty ledger has frontier zero.
    pub next_epoch: u64,
    /// The transaction being built, if a `txn` block is open.
    pub open: Option<Open>,
}

/// The transaction currently being built.
#[derive(Debug, Clone)]
pub struct Open {
    pub txn: String,
    pub epoch: u64,
    pub legs: Vec<Leg>,
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger {
            next_epoch: 1,
            ..Ledger::default()
        }
    }

    /// Declare an account. Returns false if it was already declared differently.
    pub fn declare(&mut self, account: &str, currency: &str, scale: u32) -> bool {
        match self.declared.get(account) {
            Some(existing) if *existing != (currency.to_string(), scale) => false,
            _ => {
                self.declared
                    .insert(account.to_string(), (currency.to_string(), scale));
                true
            }
        }
    }

    pub fn open_balance(&mut self, account: &str, currency: &str, minor: i128) {
        *self
            .opening
            .entry((account.to_string(), currency.to_string()))
            .or_insert(0) += minor;
    }

    pub fn is_declared(&self, account: &str) -> bool {
        self.declared.contains_key(account)
    }

    pub fn any_declared(&self) -> bool {
        !self.declared.is_empty()
    }

    /// The balance of `(account, currency)`: the opening balance plus every sealed leg.
    ///
    /// A fold, every time, over everything. This is a reference implementation and the whole
    /// point of it is that it has no state a product could disagree with.
    pub fn balance(&self, account: &str, currency: &str) -> i128 {
        let mut acc = *self
            .opening
            .get(&(account.to_string(), currency.to_string()))
            .unwrap_or(&0);
        for set in &self.sealed {
            for leg in &set.legs {
                if leg.account == account && leg.currency == currency {
                    acc += leg.minor;
                }
            }
        }
        acc
    }

    /// Begin a transaction. Returns the epoch it will seal into.
    pub fn begin(&mut self, txn: &str) -> u64 {
        let epoch = self.next_epoch;
        self.open = Some(Open {
            txn: txn.to_string(),
            epoch,
            legs: Vec::new(),
        });
        epoch
    }

    /// Add legs to the open transaction.
    pub fn push_legs(&mut self, legs: Vec<Leg>) -> Result<(), Refusal> {
        match &mut self.open {
            None => Err(Refusal::NoTransaction),
            Some(o) => {
                o.legs.extend(legs);
                Ok(())
            }
        }
    }

    /// Seal the open transaction, checking the commit rule.
    ///
    /// The rule is per currency and nothing else: a set moving −100 USD and +100 EUR nets to
    /// zero arithmetically and is refused, which is the whole reason the quantifier is over
    /// currency rather than over the set.
    pub fn seal(&mut self) -> Result<Posted, Refusal> {
        let Some(open) = self.open.take() else {
            return Err(Refusal::NoTransaction);
        };
        let mut sums: BTreeMap<String, i128> = BTreeMap::new();
        let mut scales: BTreeMap<String, u32> = BTreeMap::new();
        for leg in &open.legs {
            match scales.get(&leg.currency) {
                Some(s) if *s != leg.scale => {
                    return Err(Refusal::ScaleConflict {
                        currency: leg.currency.clone(),
                        first: *s,
                        then: leg.scale,
                    })
                }
                _ => {
                    scales.insert(leg.currency.clone(), leg.scale);
                }
            }
            let slot = sums.entry(leg.currency.clone()).or_insert(0);
            *slot = slot.checked_add(leg.minor).ok_or(Refusal::Overflow {
                currency: leg.currency.clone(),
            })?;
        }
        for (currency, residual) in &sums {
            if *residual != 0 {
                return Err(Refusal::Unbalanced {
                    currency: currency.clone(),
                    residual: *residual,
                });
            }
        }
        let posted = Posted {
            txn: open.txn,
            epoch: open.epoch,
            legs: open.legs,
        };
        self.next_epoch += 1;
        self.sealed.push(posted.clone());
        Ok(posted)
    }

    /// Abandon the open transaction. Called when a `txn` block's body returns an error, so a
    /// refused transaction leaves no half-built set behind.
    pub fn abandon(&mut self) {
        self.open = None;
    }
}

/// The canonical bytes of a posting set. See the module docs for the normative format.
pub fn encode(txn: &str, legs: &[Leg]) -> Vec<u8> {
    let mut out = Vec::with_capacity(64 + legs.len() * 96);
    put_str(&mut out, txn);
    out.extend_from_slice(&(legs.len() as u32).to_be_bytes());
    for leg in legs {
        out.extend_from_slice(&leg.id.to_be_bytes());
        put_str(&mut out, &leg.account);
        out.extend_from_slice(&leg.minor.to_be_bytes());
        put_str(&mut out, &leg.currency);
        out.extend_from_slice(&leg.scale.to_be_bytes());
        out.extend_from_slice(&leg.epoch.to_be_bytes());
        out.extend_from_slice(&leg.value_date.to_be_bytes());
        put_str(&mut out, &leg.narrative);
        match &leg.consumes {
            None => out.push(0),
            Some(tag) => {
                out.push(1);
                put_str(&mut out, tag);
            }
        }
    }
    out
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

/// Lower-case hex, for a one-line stdout that a shell can compare.
pub fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leg(account: &str, minor: i128) -> Leg {
        Leg {
            id: 0,
            account: account.into(),
            minor,
            currency: "USD".into(),
            scale: 2,
            epoch: 0,
            value_date: 0,
            narrative: String::new(),
            consumes: None,
        }
    }

    #[test]
    fn a_balanced_set_seals_and_an_unbalanced_one_names_the_currency() {
        let mut l = Ledger::new();
        l.begin("t1");
        l.push_legs(vec![leg("a", -100), leg("b", 100)]).unwrap();
        let p = l.seal().unwrap();
        assert_eq!(p.epoch, 1);
        assert_eq!(l.next_epoch, 2);

        l.begin("t2");
        l.push_legs(vec![leg("a", -100), leg("b", 99)]).unwrap();
        assert_eq!(
            l.seal(),
            Err(Refusal::Unbalanced {
                currency: "USD".into(),
                residual: -1
            })
        );
    }

    #[test]
    fn a_set_that_nets_to_zero_across_two_currencies_is_still_refused() {
        // The quantifier is over currency, and this is the case that shows why. Arithmetic
        // agreement is not conservation.
        let mut l = Ledger::new();
        l.begin("fx");
        let mut eur = leg("b", 100);
        eur.currency = "EUR".into();
        l.push_legs(vec![leg("a", -100), eur]).unwrap();
        assert!(matches!(l.seal(), Err(Refusal::Unbalanced { .. })));
    }

    #[test]
    fn a_set_conserving_only_by_wraparound_is_refused() {
        let mut l = Ledger::new();
        l.begin("overflow");
        l.push_legs(vec![leg("a", i128::MAX), leg("b", i128::MAX), leg("c", 2)])
            .unwrap();
        assert_eq!(
            l.seal(),
            Err(Refusal::Overflow {
                currency: "USD".into()
            })
        );
    }

    #[test]
    fn post_outside_a_transaction_is_refused_rather_than_buffered() {
        let mut l = Ledger::new();
        assert_eq!(l.push_legs(vec![leg("a", 0)]), Err(Refusal::NoTransaction));
    }

    #[test]
    fn a_balance_is_the_opening_plus_every_sealed_leg() {
        let mut l = Ledger::new();
        l.open_balance("a", "USD", 1_000);
        l.begin("t");
        l.push_legs(vec![leg("a", -250), leg("b", 250)]).unwrap();
        l.seal().unwrap();
        assert_eq!(l.balance("a", "USD"), 750);
        assert_eq!(l.balance("b", "USD"), 250);
        assert_eq!(l.balance("c", "USD"), 0);
    }

    /// The encoding is byte-exact and stated, so a change to it is a diff rather than a
    /// surprise in another repository.
    #[test]
    fn the_encoding_is_the_documented_layout_byte_for_byte() {
        let legs = vec![leg("a", -1_000)];
        let bytes = encode("t", &legs);
        let mut want = Vec::new();
        want.extend_from_slice(&1u32.to_be_bytes()); // txn_len
        want.extend_from_slice(b"t");
        want.extend_from_slice(&1u32.to_be_bytes()); // entry_count
        want.extend_from_slice(&0u64.to_be_bytes()); // id
        want.extend_from_slice(&1u32.to_be_bytes()); // account_len
        want.extend_from_slice(b"a");
        want.extend_from_slice(&(-1_000i128).to_be_bytes());
        want.extend_from_slice(&3u32.to_be_bytes()); // currency_len
        want.extend_from_slice(b"USD");
        want.extend_from_slice(&2u32.to_be_bytes()); // scale
        want.extend_from_slice(&0u64.to_be_bytes()); // epoch
        want.extend_from_slice(&0i64.to_be_bytes()); // value_date
        want.extend_from_slice(&0u32.to_be_bytes()); // narrative_len
        want.push(0); // has_tag
        assert_eq!(bytes, want);
        assert_eq!(hex(&[0x00, 0xff, 0x10]), "00ff10");
    }

    /// Leg order is part of a set's identity, so the encoder must not sort.
    #[test]
    fn two_orderings_of_one_set_encode_differently() {
        let a = encode("t", &[leg("a", -1), leg("b", 1)]);
        let b = encode("t", &[leg("b", 1), leg("a", -1)]);
        assert_ne!(
            a, b,
            "an encoder that sorted would make these hash the same"
        );
    }
}
