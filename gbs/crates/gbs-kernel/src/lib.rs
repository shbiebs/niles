//! **GBS kernel** — mechanism M1, and nothing else.
//!
//! One idea: a **balanced posting set** over an immutable, epoch-ordered ledger. Every
//! product in the platform — a card authorisation, a syndicated drawdown, an ETF creation
//! basket, a letter of credit amendment — is a pattern of posting sets, and this crate is
//! where the pattern's single invariant lives.
//!
//! # Why the kernel is this small
//!
//! The thesis (§6.6, §11.5.1) claims that banking is a *library over a fully general
//! relational core*, and §11.3 states its refutation condition: a product that cannot be
//! expressed without a kernel change falsifies it. That makes every addition here a
//! potential falsification, so the boundary is drawn deliberately and `ARCHITECTURE.md` §6
//! records what was kept out and why:
//!
//! * **No account type with behaviour.** An account is an identity and a currency. Asset
//!   versus liability is a property of the chart of accounts, which is data.
//! * **No balance field.** A balance is a fold over entries at an anchor. A stored balance
//!   is a second source of truth, and the whole thesis is about not having two.
//! * **No status column.** Status is a fold over events (mechanism M5). A status field is a
//!   value that can disagree with the events that produced it.
//! * **No rounding account.** Rounding is M4's designated residual holder — a named party,
//!   not a place error goes to die.
//! * **No currency conversion.** FX is two conserved legs sealed in one epoch, never an
//!   amount multiplied by a rate inside a posting. A rate applied inside a posting is how a
//!   cross-currency imbalance becomes invisible.
//!
//! # The one invariant
//!
//! A posting set balances **per currency**, not in aggregate. A transaction touching three
//! currencies carries three independent obligations, and a single scalar sum would let a
//! surplus in one currency mask a deficit in another. [`PostingSet::conservation`] returns
//! the per-currency residual so a violation names the currency and the amount rather than
//! reporting that something, somewhere, did not add up.
//!
//! # Money
//!
//! `i128` minor units at a per-currency scale. Never a float — not as a defensive
//! convention but because the determinism obligation of Appendix C.4 requires that the same
//! input produce the same bytes on every target, and floating-point summation is not
//! associative. Every arithmetic operation here is checked; there is no wrapping path.

use std::collections::BTreeMap;
use std::fmt;

pub mod account;
pub mod chart;
pub mod entry;
pub mod posting;

pub use account::{Account, AccountId, Party};
pub use chart::{Chart, Nature};
pub use entry::{Entry, EntryId};
pub use posting::{PostingSet, Residual, Sealed, TxnId};

/// Re-exported from the ledger so that no layer above the kernel needs a direct dependency
/// on the engine.
///
/// This is not tidiness. `gbs/tests/layering.rs` asserts that `gbs-products` depends on
/// nothing below `gbs-mechanisms`, and that assertion is only meaningful if a product has
/// no *reason* to reach past it. An epoch is part of the kernel's vocabulary — a stamp
/// carries one — so the kernel is where it should be visible from.
pub use nilestream_ledger::{Epoch, Minor};
/// An ISO-4217-style currency code, or any code a deployment declares.
///
/// A newtype rather than a bare `String`, because the currency is the thing conservation is
/// checked *per*, and a bare string invites the one mistake that matters: comparing a
/// currency to an account identifier and getting `false` in a way nothing notices.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Currency(String);

impl Currency {
    /// Codes are normalised to upper case on construction, so `usd` and `USD` are one
    /// currency. This is the *only* normalisation the kernel performs, and it is here
    /// because two spellings of one currency would silently defeat per-currency
    /// conservation: the residuals would balance separately and both would be zero.
    pub fn new(code: impl AsRef<str>) -> Self {
        Currency(code.as_ref().to_ascii_uppercase())
    }

    pub fn code(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An amount in one currency, in minor units at that currency's declared scale.
///
/// The scale is carried alongside rather than looked up, so that an amount is
/// self-describing on the wire and in a log. The type checker reconciles a literal's scale
/// with its currency declaration (thesis §B.2); the kernel checks that two amounts being
/// combined agree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Amount {
    pub minor: Minor,
    pub currency: Currency,
    pub scale: u32,
}

impl Amount {
    pub fn new(minor: Minor, currency: Currency, scale: u32) -> Self {
        Amount { minor, currency, scale }
    }

    /// The common case: a two-decimal currency.
    pub fn minor_2dp(minor: Minor, code: &str) -> Self {
        Amount { minor, currency: Currency::new(code), scale: 2 }
    }

    pub fn is_zero(&self) -> bool {
        self.minor == 0
    }

    pub fn negate(&self) -> Result<Amount, KernelError> {
        Ok(Amount {
            minor: self.minor.checked_neg().ok_or(KernelError::Overflow { op: "negate" })?,
            currency: self.currency.clone(),
            scale: self.scale,
        })
    }

    /// Addition, which refuses two things: a currency mismatch, and a scale mismatch within
    /// one currency.
    ///
    /// The second refusal is the less obvious one and it is the more important. Two amounts
    /// in USD at different scales are not comparable as integers — 1000 at scale 2 is ten
    /// dollars and 1000 at scale 4 is ten cents — so adding their minor units produces a
    /// number that is wrong by a factor of a hundred and looks entirely plausible. Rescaling
    /// silently would be worse still, because it either loses precision or invents it.
    pub fn add(&self, other: &Amount) -> Result<Amount, KernelError> {
        if self.currency != other.currency {
            return Err(KernelError::CurrencyMismatch {
                left: self.currency.clone(),
                right: other.currency.clone(),
            });
        }
        if self.scale != other.scale {
            return Err(KernelError::ScaleMismatch {
                currency: self.currency.clone(),
                left: self.scale,
                right: other.scale,
            });
        }
        Ok(Amount {
            minor: self.minor.checked_add(other.minor).ok_or(KernelError::Overflow { op: "add" })?,
            currency: self.currency.clone(),
            scale: self.scale,
        })
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = 10i128.pow(self.scale);
        let sign = if self.minor < 0 { "-" } else { "" };
        let a = self.minor.abs();
        write!(f, "{sign}{}.{:0width$} {}", a / d, a % d, self.currency, width = self.scale as usize)
    }
}

/// What the kernel refuses.
///
/// Every variant names the values involved rather than the fact of a failure, because a
/// conservation error reported as "transaction unbalanced" sends an operator to read a
/// thousand entries, and one reported as "USD residual +500.00 across 4 entries" does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// The posting set does not sum to zero in at least one currency.
    NotConserved { residuals: Vec<Residual> },
    CurrencyMismatch { left: Currency, right: Currency },
    ScaleMismatch { currency: Currency, left: u32, right: u32 },
    Overflow { op: &'static str },
    /// An entry names an account the chart does not declare. Refused rather than
    /// auto-created: an account that springs into existence on first use is an account
    /// nobody reconciled.
    UnknownAccount { account: AccountId },
    /// An entry's currency is not the account's currency. An account holds one currency;
    /// movement between currencies is two legs, never one entry.
    AccountCurrency { account: AccountId, account_currency: Currency, entry_currency: Currency },
    /// A posting set with no entries. Refused because an empty set trivially conserves, and
    /// a transaction that trivially conserves is one that did nothing while reporting
    /// success.
    Empty,
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelError::NotConserved { residuals } => {
                write!(f, "conservation violated: ")?;
                for (i, r) in residuals.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{} residual {} across {} entr{}",
                        r.currency, r.amount, r.entries,
                        if r.entries == 1 { "y" } else { "ies" })?;
                }
                Ok(())
            }
            KernelError::CurrencyMismatch { left, right } => {
                write!(f, "currency mismatch: {left} and {right} cannot be combined")
            }
            KernelError::ScaleMismatch { currency, left, right } => write!(
                f,
                "scale mismatch in {currency}: {left} and {right}. Minor units at different \
                 scales are not comparable; rescale explicitly"
            ),
            KernelError::Overflow { op } => write!(f, "arithmetic overflow in `{op}`"),
            KernelError::UnknownAccount { account } => {
                write!(f, "account `{account}` is not in the chart of accounts")
            }
            KernelError::AccountCurrency { account, account_currency, entry_currency } => write!(
                f,
                "account `{account}` holds {account_currency}, but the entry is in \
                 {entry_currency}; cross-currency movement is two legs, not one entry"
            ),
            KernelError::Empty => write!(f, "a posting set must contain at least one entry"),
        }
    }
}

/// A point on both temporal axes (thesis §3.8).
///
/// The distinction is not decorative and it is where most banking systems accumulate their
/// worst bugs. `epoch` is the *system* axis — when the fact was recorded, which is
/// immutable and totally ordered. `value_date` is the *world* axis — when the fact is
/// deemed to have taken effect, which is a business decision and can be in the past.
///
/// A back-valued correction is therefore a **new fact at an earlier valid time**, never a
/// mutation of an old one. That is the whole of what bitemporality buys, and it is why
/// "restate last month's interest" is an append here rather than an `UPDATE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stamp {
    /// System time: the epoch that sealed this fact.
    pub epoch: Epoch,
    /// Valid time: days since the epoch date, as a signed offset so that back-valuation is
    /// representable rather than exceptional.
    pub value_date: i64,
}

impl Stamp {
    pub fn new(epoch: Epoch, value_date: i64) -> Self {
        Stamp { epoch, value_date }
    }

    /// Whether this fact is visible to a reader anchored at `anchor` on the system axis.
    ///
    /// Strictly `<=`: a fact sealed *at* the anchor is visible at it. Anything else would
    /// make a read-your-writes rung unimplementable.
    pub fn visible_at(&self, anchor: Epoch) -> bool {
        self.epoch <= anchor
    }

    /// Whether this fact is in effect on the world axis at `as_of`.
    pub fn effective_on(&self, as_of: i64) -> bool {
        self.value_date <= as_of
    }
}

/// Per-currency sums over a set of amounts.
///
/// Separated from [`PostingSet`] so that the same folding logic serves conservation
/// checking, balance computation and position signals — three things that are the same
/// operation over different scopes, and which get written three times in most banking
/// systems and then disagree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CurrencySums {
    sums: BTreeMap<Currency, (Minor, u32, usize)>,
}

impl CurrencySums {
    pub fn new() -> Self {
        CurrencySums::default()
    }

    pub fn add(&mut self, a: &Amount) -> Result<(), KernelError> {
        match self.sums.get_mut(&a.currency) {
            None => {
                self.sums.insert(a.currency.clone(), (a.minor, a.scale, 1));
                Ok(())
            }
            Some((total, scale, count)) => {
                if *scale != a.scale {
                    return Err(KernelError::ScaleMismatch {
                        currency: a.currency.clone(),
                        left: *scale,
                        right: a.scale,
                    });
                }
                *total = total.checked_add(a.minor).ok_or(KernelError::Overflow { op: "sum" })?;
                *count += 1;
                Ok(())
            }
        }
    }

    pub fn get(&self, c: &Currency) -> Option<Amount> {
        self.sums.get(c).map(|(m, s, _)| Amount::new(*m, c.clone(), *s))
    }

    pub fn currencies(&self) -> impl Iterator<Item = &Currency> {
        self.sums.keys()
    }

    pub fn is_empty(&self) -> bool {
        self.sums.is_empty()
    }

    /// Every currency whose sum is non-zero, with the count of contributing entries.
    ///
    /// Empty means conserved. Iteration is `BTreeMap` order, so the residual list is stable
    /// across runs and a diagnostic can be compared byte for byte — the determinism
    /// obligation reaching down into an error message.
    pub fn nonzero(&self) -> Vec<Residual> {
        self.sums
            .iter()
            .filter(|(_, (m, _, _))| *m != 0)
            .map(|(c, (m, s, n))| Residual {
                currency: c.clone(),
                amount: Amount::new(*m, c.clone(), *s),
                entries: *n,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_codes_normalise_so_one_currency_is_one_currency() {
        // The failure this prevents: `usd` and `USD` balancing separately, each to zero,
        // while the transaction as a whole is unbalanced. Both residuals would be zero and
        // nothing would report a problem.
        assert_eq!(Currency::new("usd"), Currency::new("USD"));
        assert_eq!(Currency::new("Usd").code(), "USD");
        assert_ne!(Currency::new("usd"), Currency::new("eur"));
    }

    #[test]
    fn amounts_of_different_currencies_do_not_add() {
        let a = Amount::minor_2dp(1000, "USD");
        let b = Amount::minor_2dp(1000, "EUR");
        assert!(matches!(a.add(&b), Err(KernelError::CurrencyMismatch { .. })));
    }

    #[test]
    fn amounts_at_different_scales_do_not_add_even_in_one_currency() {
        // The subtle one. 1000 at scale 2 is $10.00; 1000 at scale 4 is $0.1000. Adding the
        // minor units gives 2000, which is either $20.00 or $0.2000 and is wrong either way
        // — and looks completely ordinary.
        let cents = Amount::new(1000, Currency::new("USD"), 2);
        let tenths_of_a_cent = Amount::new(1000, Currency::new("USD"), 4);
        let e = cents.add(&tenths_of_a_cent).unwrap_err();
        assert!(matches!(e, KernelError::ScaleMismatch { .. }));
        assert!(e.to_string().contains("rescale explicitly"));
    }

    #[test]
    fn addition_is_checked_and_never_wraps() {
        let big = Amount::minor_2dp(i128::MAX, "USD");
        let one = Amount::minor_2dp(1, "USD");
        assert_eq!(big.add(&one), Err(KernelError::Overflow { op: "add" }));
    }

    #[test]
    fn money_renders_at_its_own_scale() {
        assert_eq!(Amount::minor_2dp(123_456, "USD").to_string(), "1234.56 USD");
        assert_eq!(Amount::minor_2dp(-500, "EUR").to_string(), "-5.00 EUR");
        assert_eq!(Amount::new(1, Currency::new("BTC"), 8).to_string(), "0.00000001 BTC");
        // Zero is rendered, not elided. A blank where an amount should be is how a reader
        // comes to believe a leg was absent when it was zero.
        assert_eq!(Amount::minor_2dp(0, "JPY").to_string(), "0.00 JPY");
    }

    #[test]
    fn sums_are_per_currency_and_report_which_one_is_wrong() {
        // The central kernel behaviour. A three-currency transaction has three obligations.
        let mut s = CurrencySums::new();
        s.add(&Amount::minor_2dp(1000, "USD")).unwrap();
        s.add(&Amount::minor_2dp(-1000, "USD")).unwrap();
        s.add(&Amount::minor_2dp(500, "EUR")).unwrap();
        s.add(&Amount::minor_2dp(-200, "EUR")).unwrap();
        s.add(&Amount::minor_2dp(0, "GBP")).unwrap();

        let bad = s.nonzero();
        assert_eq!(bad.len(), 1, "only EUR is out");
        assert_eq!(bad[0].currency, Currency::new("EUR"));
        assert_eq!(bad[0].amount.minor, 300);
        assert_eq!(bad[0].entries, 2);
    }

    #[test]
    fn an_aggregate_sum_would_have_hidden_this_and_a_per_currency_one_does_not() {
        // The argument for per-currency conservation, as a test. +500 USD and -500 EUR sums
        // to zero if you ignore the currency, and is a five-hundred-unit hole if you do not.
        let mut s = CurrencySums::new();
        s.add(&Amount::minor_2dp(50_000, "USD")).unwrap();
        s.add(&Amount::minor_2dp(-50_000, "EUR")).unwrap();
        let bad = s.nonzero();
        assert_eq!(bad.len(), 2, "both currencies are out, and neither cancels the other");
    }

    #[test]
    fn residuals_are_reported_in_a_stable_order() {
        // Determinism reaching into diagnostics: two runs must produce byte-identical error
        // text, or a golden test on an error message is not possible.
        let mut a = CurrencySums::new();
        for c in ["ZAR", "AUD", "MXN", "JPY"] {
            a.add(&Amount::minor_2dp(1, c)).unwrap();
        }
        let mut b = CurrencySums::new();
        for c in ["JPY", "MXN", "AUD", "ZAR"] {
            b.add(&Amount::minor_2dp(1, c)).unwrap();
        }
        let names = |s: &CurrencySums| {
            s.nonzero().iter().map(|r| r.currency.to_string()).collect::<Vec<_>>()
        };
        assert_eq!(names(&a), names(&b));
        assert_eq!(names(&a), vec!["AUD", "JPY", "MXN", "ZAR"]);
    }

    #[test]
    fn the_two_temporal_axes_are_independent() {
        // A back-valued correction: recorded now, effective a month ago. Both facts are
        // true simultaneously, which is the point of having two axes rather than one date
        // column and an argument about which date it is.
        let correction = Stamp::new(Epoch(9_000), -30);
        assert!(correction.visible_at(Epoch(9_000)), "visible at the epoch that sealed it");
        assert!(!correction.visible_at(Epoch(8_999)), "and not before");
        assert!(correction.effective_on(0), "effective a month ago on the world axis");
        assert!(correction.effective_on(-30));
        assert!(!correction.effective_on(-31));
    }

    #[test]
    fn visibility_includes_the_sealing_epoch_so_read_your_writes_is_implementable() {
        let s = Stamp::new(Epoch(100), 0);
        assert!(s.visible_at(Epoch(100)));
    }
}
