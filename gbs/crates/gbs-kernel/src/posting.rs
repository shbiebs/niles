//! Posting sets — mechanism **M1**, the atom every product is built from.
//!
//! A posting set is a group of entries sealed together in one epoch. Its single invariant
//! is that it sums to zero **per currency**, and [`PostingSet::seal`] is the only way to
//! obtain a [`Sealed`] one. There is no way to construct a `Sealed` that has not been
//! checked, which is the type-level form of the runtime guarantee: the thing that reaches
//! the ledger is the thing that passed.

use crate::{Amount, Chart, Currency, CurrencySums, Entry, KernelError};
use nilestream_ledger::Epoch;
use std::fmt;

/// A transaction identifier, and the idempotency key.
///
/// One value serves both roles deliberately. A transaction that arrives twice — a retried
/// payment instruction, a redelivered message — must post once, and the identity of the
/// business event is precisely what decides whether two arrivals are the same event. A
/// separate idempotency key is a second identity that can disagree with the first.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxnId(String);

impl TxnId {
    pub fn new(id: impl Into<String>) -> Self {
        TxnId(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TxnId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A currency in which a posting set failed to conserve, and by how much.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Residual {
    pub currency: Currency,
    pub amount: Amount,
    /// How many entries contributed to this currency. Included because "USD residual
    /// +500.00 across 2 entries" is a bug you can find and "across 4,000 entries" is a
    /// reconciliation, and knowing which before you start is worth a field.
    pub entries: usize,
}

/// A posting set under construction. Not yet checked, and not yet acceptable to the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostingSet {
    pub txn: TxnId,
    pub entries: Vec<Entry>,
}

/// A posting set that **has** been checked. The only thing the ledger accepts.
///
/// The field is private and there is no constructor other than [`PostingSet::seal`]. That
/// is the whole design: a value of this type is evidence that per-currency conservation was
/// verified, and the evidence cannot be forged by an application in a hurry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sealed {
    inner: PostingSet,
    epoch: Epoch,
}

impl Sealed {
    pub fn entries(&self) -> &[Entry] {
        &self.inner.entries
    }
    pub fn txn(&self) -> &TxnId {
        &self.inner.txn
    }
    pub fn epoch(&self) -> Epoch {
        self.epoch
    }

    /// Re-derive the per-currency sums. Every one must be zero; this is here so that a
    /// caller downstream of the seal can re-verify rather than trust, and so that the
    /// conservation suite can check the property it claims rather than the flag.
    pub fn verify(&self) -> Result<(), KernelError> {
        self.inner.conservation().and_then(|r| {
            if r.is_empty() {
                Ok(())
            } else {
                Err(KernelError::NotConserved { residuals: r })
            }
        })
    }
}

impl PostingSet {
    pub fn new(txn: impl Into<String>) -> Self {
        PostingSet { txn: TxnId::new(txn), entries: Vec::new() }
    }

    pub fn with(mut self, e: Entry) -> Self {
        self.entries.push(e);
        self
    }

    pub fn push(&mut self, e: Entry) {
        self.entries.push(e);
    }

    /// The per-currency residuals. Empty means conserved.
    ///
    /// Returns the residuals rather than a boolean, because every caller that gets `false`
    /// immediately needs to know which currency and how much, and a caller that has to ask
    /// twice will eventually not ask.
    pub fn conservation(&self) -> Result<Vec<Residual>, KernelError> {
        let mut sums = CurrencySums::new();
        for e in &self.entries {
            sums.add(&e.amount)?;
        }
        Ok(sums.nonzero())
    }

    /// Check every entry against the chart: the account exists, and its currency is the
    /// account's currency.
    ///
    /// The second check is what keeps conservation meaningful. An entry in EUR against a USD
    /// account would balance the EUR column and leave the account holding a currency it does
    /// not have, so the arithmetic would be right and the position would be nonsense.
    pub fn check_against(&self, chart: &Chart) -> Result<(), KernelError> {
        for e in &self.entries {
            let acct = chart
                .account(&e.account)
                .ok_or_else(|| KernelError::UnknownAccount { account: e.account.clone() })?;
            if acct.currency != e.amount.currency {
                return Err(KernelError::AccountCurrency {
                    account: e.account.clone(),
                    account_currency: acct.currency.clone(),
                    entry_currency: e.amount.currency.clone(),
                });
            }
        }
        Ok(())
    }

    /// Seal the set at an epoch. The only way to obtain a [`Sealed`].
    ///
    /// Runs, in order: non-emptiness, the chart check, then conservation. The order matters
    /// for the diagnostic rather than the outcome — an entry against an unknown account
    /// would also usually show up as a residual, and "account `x` is not in the chart" is a
    /// far better first thing to read than "USD residual +100.00".
    pub fn seal(self, chart: &Chart, epoch: Epoch) -> Result<Sealed, KernelError> {
        if self.entries.is_empty() {
            return Err(KernelError::Empty);
        }
        self.check_against(chart)?;
        let residuals = self.conservation()?;
        if !residuals.is_empty() {
            return Err(KernelError::NotConserved { residuals });
        }
        Ok(Sealed { inner: self, epoch })
    }

    /// The currencies this set touches, in stable order.
    pub fn currencies(&self) -> Vec<Currency> {
        let mut cs: Vec<Currency> = self.entries.iter().map(|e| e.amount.currency.clone()).collect();
        cs.sort();
        cs.dedup();
        cs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Account, Stamp};

    fn chart() -> Chart {
        Chart::new()
            .with(Account::new("cash.usd", "bank", "USD"))
            .with(Account::new("cust.usd", "cust-1", "USD"))
            .with(Account::new("fees.usd", "bank", "USD"))
            .with(Account::new("cash.eur", "bank", "EUR"))
            .with(Account::new("cust.eur", "cust-1", "EUR"))
    }

    fn st() -> Stamp {
        Stamp::new(Epoch(42), 0)
    }

    #[test]
    fn a_balanced_transfer_seals() {
        let ps = PostingSet::new("txn-1")
            .with(Entry::new(1, "cust.usd", Amount::minor_2dp(-10_000, "USD"), st()))
            .with(Entry::new(2, "cash.usd", Amount::minor_2dp(10_000, "USD"), st()));
        let sealed = ps.seal(&chart(), Epoch(42)).expect("balanced");
        assert_eq!(sealed.epoch(), Epoch(42));
        assert_eq!(sealed.entries().len(), 2);
        sealed.verify().expect("still conserved after sealing");
    }

    #[test]
    fn an_unbalanced_set_names_the_currency_and_the_amount() {
        // The diagnostic quality this crate exists to provide. Not "unbalanced" — which
        // currency, how much, and across how many entries.
        let ps = PostingSet::new("txn-2")
            .with(Entry::new(1, "cust.usd", Amount::minor_2dp(-10_000, "USD"), st()))
            .with(Entry::new(2, "cash.usd", Amount::minor_2dp(9_500, "USD"), st()));
        let e = ps.seal(&chart(), Epoch(42)).unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("USD"), "{msg}");
        assert!(msg.contains("-5.00"), "the residual, at its own scale: {msg}");
        assert!(msg.contains("2 entries"), "{msg}");
    }

    #[test]
    fn a_three_legged_set_with_a_fee_balances() {
        // The everyday shape: customer pays 100.00, the bank keeps 0.25.
        let ps = PostingSet::new("txn-3")
            .with(Entry::new(1, "cust.usd", Amount::minor_2dp(-10_000, "USD"), st()))
            .with(Entry::new(2, "cash.usd", Amount::minor_2dp(9_975, "USD"), st()))
            .with(Entry::new(3, "fees.usd", Amount::minor_2dp(25, "USD"), st()));
        assert!(ps.seal(&chart(), Epoch(42)).is_ok());
    }

    #[test]
    fn a_multi_currency_set_must_balance_in_every_currency_independently() {
        // The FX shape, and the property that makes it safe: USD balances, EUR balances,
        // and neither is allowed to compensate for the other.
        let good = PostingSet::new("fx-1")
            .with(Entry::new(1, "cust.usd", Amount::minor_2dp(-10_000, "USD"), st()))
            .with(Entry::new(2, "cash.usd", Amount::minor_2dp(10_000, "USD"), st()))
            .with(Entry::new(3, "cash.eur", Amount::minor_2dp(-9_200, "EUR"), st()))
            .with(Entry::new(4, "cust.eur", Amount::minor_2dp(9_200, "EUR"), st()));
        assert!(good.seal(&chart(), Epoch(42)).is_ok());

        // And the failure a naive aggregate check would miss entirely: +100 USD against
        // -100 EUR sums to zero if the currency is ignored.
        let bad = PostingSet::new("fx-2")
            .with(Entry::new(1, "cust.usd", Amount::minor_2dp(10_000, "USD"), st()))
            .with(Entry::new(2, "cust.eur", Amount::minor_2dp(-10_000, "EUR"), st()));
        let e = bad.seal(&chart(), Epoch(42)).unwrap_err();
        match e {
            KernelError::NotConserved { residuals } => {
                assert_eq!(residuals.len(), 2, "both currencies are out: {residuals:?}");
            }
            other => panic!("expected a conservation failure, got {other}"),
        }
    }

    #[test]
    fn an_entry_against_an_unknown_account_is_refused_rather_than_creating_one() {
        let ps = PostingSet::new("txn-4")
            .with(Entry::new(1, "cust.usd", Amount::minor_2dp(-100, "USD"), st()))
            .with(Entry::new(2, "somewhere.new", Amount::minor_2dp(100, "USD"), st()));
        assert!(matches!(
            ps.seal(&chart(), Epoch(42)),
            Err(KernelError::UnknownAccount { .. })
        ));
    }

    #[test]
    fn an_entry_in_the_wrong_currency_for_its_account_is_refused() {
        // Balances perfectly in EUR, and would leave a USD account holding euros. The
        // arithmetic is right and the position is nonsense, which is exactly the class of
        // error a per-currency sum alone cannot catch.
        let ps = PostingSet::new("txn-5")
            .with(Entry::new(1, "cust.usd", Amount::minor_2dp(-100, "EUR"), st()))
            .with(Entry::new(2, "cust.eur", Amount::minor_2dp(100, "EUR"), st()));
        let e = ps.seal(&chart(), Epoch(42)).unwrap_err();
        assert!(matches!(e, KernelError::AccountCurrency { .. }));
        assert!(e.to_string().contains("two legs, not one entry"));
    }

    #[test]
    fn an_empty_set_is_refused_because_it_would_conserve_trivially() {
        // The subtle one. An empty set sums to zero in every currency, so a naive check
        // passes it — and a transaction that does nothing while reporting success is worse
        // than one that fails.
        assert_eq!(PostingSet::new("txn-6").seal(&chart(), Epoch(1)), Err(KernelError::Empty));
    }

    #[test]
    fn a_sealed_set_cannot_be_constructed_without_checking() {
        // Not expressible as a runtime assertion — it is a property of the module boundary:
        // `Sealed`'s fields are private and `seal` is the only constructor. This test
        // documents the intent, and the compiler enforces it.
        let ps = PostingSet::new("txn-7")
            .with(Entry::new(1, "cust.usd", Amount::minor_2dp(-1, "USD"), st()))
            .with(Entry::new(2, "cash.usd", Amount::minor_2dp(1, "USD"), st()));
        let sealed = ps.seal(&chart(), Epoch(9)).unwrap();
        assert!(sealed.verify().is_ok());
    }

    #[test]
    fn currencies_are_reported_in_stable_order() {
        let ps = PostingSet::new("t")
            .with(Entry::new(1, "cust.eur", Amount::minor_2dp(0, "EUR"), st()))
            .with(Entry::new(2, "cust.usd", Amount::minor_2dp(0, "USD"), st()))
            .with(Entry::new(3, "cash.eur", Amount::minor_2dp(0, "EUR"), st()));
        assert_eq!(
            ps.currencies().iter().map(|c| c.to_string()).collect::<Vec<_>>(),
            vec!["EUR", "USD"]
        );
    }

    #[test]
    fn a_zero_amount_entry_is_legal_and_still_counted() {
        // Zero-value legs are real — a fee waived, a leg present for audit symmetry. They
        // must not be silently dropped, because a reader counting legs would then see a
        // different transaction from the one that was posted.
        let ps = PostingSet::new("t")
            .with(Entry::new(1, "cust.usd", Amount::minor_2dp(0, "USD"), st()));
        let sealed = ps.seal(&chart(), Epoch(1)).expect("zero conserves");
        assert_eq!(sealed.entries().len(), 1);
    }
}
