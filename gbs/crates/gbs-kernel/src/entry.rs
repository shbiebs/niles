//! Entries — the individual movements a posting set is made of.
//!
//! An entry is a signed amount against one account at one bitemporal stamp. There is no
//! "debit" and "credit" pair of fields, and that omission is deliberate enough to justify a
//! paragraph.
//!
//! # Why signed amounts rather than debit/credit columns
//!
//! Classical double-entry stores a positive amount and a side. It has one real advantage —
//! the amount is never negative, so a sign error is impossible — and two costs that matter
//! more here. First, conservation becomes "sum of debits equals sum of credits", which is
//! two folds and a comparison rather than one fold against zero, and the two-fold version
//! is where an unbalanced set can hide when a currency appears on only one side. Second,
//! and decisively, whether a movement is a debit or a credit depends on the account's
//! *nature* — a credit increases a liability and decreases an asset — so the debit/credit
//! encoding entangles the movement with the chart of accounts, and the kernel would have to
//! know what an asset is.
//!
//! With signed amounts, conservation is `sum == 0` per currency, the kernel needs no notion
//! of asset or liability, and the debit/credit *presentation* is recovered by the chart at
//! the point of display (see [`Chart::side`](crate::Chart::side)). The accountant's view is
//! preserved; it is just derived rather than stored.

use crate::{AccountId, Amount, Stamp};
use std::fmt;

/// A stable identifier for one entry.
///
/// Assigned by the ledger, not by the application. The GBS/Noria postmortem (thesis §1.1.1)
/// records what happens otherwise: when the engine could not supply uniqueness, the
/// application generated it as `transaction_id + 1`, with a comment explaining that the
/// increment keeps the key unique. Identity is the ledger's job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(pub u64);

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

/// One movement: a signed amount against one account, stamped on both temporal axes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: EntryId,
    pub account: AccountId,
    /// Signed. Positive increases the account's balance in its own currency; negative
    /// decreases it. What that means in accounting terms is the chart's business.
    pub amount: Amount,
    pub stamp: Stamp,
    /// Free text carried for audit. Never parsed by the kernel — a system that makes
    /// decisions by reading a narrative field has a schema it did not admit to having.
    pub narrative: String,
}

impl Entry {
    pub fn new(id: u64, account: impl Into<String>, amount: Amount, stamp: Stamp) -> Self {
        Entry {
            id: EntryId(id),
            account: AccountId::new(account),
            amount,
            stamp,
            narrative: String::new(),
        }
    }

    pub fn narrated(mut self, n: impl Into<String>) -> Self {
        self.narrative = n.into();
        self
    }

    /// The mirror of this entry against another account: same amount, opposite sign.
    ///
    /// The two-entry transfer is the overwhelmingly common case, and building it from a
    /// negation rather than from two independently written amounts removes the single most
    /// common way a transfer comes out unbalanced — someone typing the figure twice and
    /// getting it wrong once.
    pub fn mirrored_to(&self, id: u64, account: impl Into<String>) -> Result<Entry, crate::KernelError> {
        Ok(Entry {
            id: EntryId(id),
            account: AccountId::new(account),
            amount: self.amount.negate()?,
            stamp: self.stamp,
            narrative: self.narrative.clone(),
        })
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.account, self.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Currency;
    use nilestream_ledger::Epoch;

    fn stamp() -> Stamp {
        Stamp::new(Epoch(1), 0)
    }

    #[test]
    fn a_mirror_cannot_disagree_with_its_original() {
        // The property: a transfer built by mirroring is balanced by construction, so the
        // most common cause of an unbalanced transfer — the amount typed twice — has no
        // way to occur.
        let debit = Entry::new(1, "a", Amount::minor_2dp(-25_000, "USD"), stamp());
        let credit = debit.mirrored_to(2, "b").unwrap();
        assert_eq!(credit.amount.minor, 25_000);
        assert_eq!(credit.amount.currency, Currency::new("USD"));
        assert_eq!(debit.amount.minor + credit.amount.minor, 0);
        assert_eq!(credit.stamp, debit.stamp, "both legs share one stamp");
    }

    #[test]
    fn a_mirror_of_the_extreme_negative_reports_overflow_rather_than_wrapping() {
        // i128::MIN has no positive counterpart. Wrapping here would produce a mirror equal
        // to the original, which balances to i128::MIN rather than to zero — an unbalanced
        // transfer that passes a naive check.
        let e = Entry::new(1, "a", Amount::minor_2dp(i128::MIN, "USD"), stamp());
        assert!(e.mirrored_to(2, "b").is_err());
    }

    #[test]
    fn entries_carry_no_side_and_conservation_needs_none() {
        let d = format!("{:?}", Entry::new(1, "a", Amount::minor_2dp(1, "USD"), stamp()));
        assert!(!d.contains("debit"), "the side is derived from the chart, not stored");
        assert!(!d.contains("credit"));
    }

    #[test]
    fn narrative_is_carried_and_never_interpreted() {
        let e = Entry::new(1, "a", Amount::minor_2dp(1, "USD"), stamp())
            .narrated("REVERSAL of e17 per ticket OPS-3341");
        assert!(e.narrative.contains("REVERSAL"));
        // If the kernel ever needed to *read* this to behave correctly, the thing it was
        // reading would deserve a field.
    }
}
