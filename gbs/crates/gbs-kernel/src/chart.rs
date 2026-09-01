//! The chart of accounts — **data**, not code.
//!
//! Everything an institution means by "what kind of account is this" lives here rather than
//! in the kernel's types. The kernel needs the chart for exactly two decisions: whether an
//! account exists, and what currency it holds. Everything else the chart carries — nature,
//! classification, hierarchy — is for the product library and the presentation layer.
//!
//! # Nature, and why the kernel does not use it
//!
//! [`Nature`] records whether an account is an asset, a liability, equity, income or
//! expense. That distinction is what turns a signed amount into a debit or a credit for
//! presentation: a positive movement on an asset is a debit, and on a liability it is a
//! credit. The kernel never consults it, because conservation is `sum == 0` regardless —
//! which is precisely the argument for signed amounts made in `entry.rs`.
//!
//! Keeping nature here rather than in `Account` also keeps it *versionable*. A
//! reclassification is a change to the chart, and the entries it reinterprets are untouched.

use crate::{Account, AccountId, Currency};
use std::collections::BTreeMap;

/// The accounting nature of an account, for presentation and for the trial balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nature {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

impl Nature {
    /// Whether a positive movement on this nature presents as a debit.
    ///
    /// Assets and expenses increase by debit; liabilities, equity and income increase by
    /// credit. This is the entire content of the debit/credit convention, and it is four
    /// lines rather than a column on every entry.
    pub fn positive_is_debit(&self) -> bool {
        matches!(self, Nature::Asset | Nature::Expense)
    }

    /// Whether this nature appears on the balance sheet (as opposed to the income
    /// statement). Used by period-close, which is a product-library concern.
    pub fn is_balance_sheet(&self) -> bool {
        matches!(self, Nature::Asset | Nature::Liability | Nature::Equity)
    }
}

/// Which side a signed amount presents on, for a given account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Debit,
    Credit,
}

/// The chart: accounts, their natures, and nothing that behaves.
#[derive(Debug, Clone, Default)]
pub struct Chart {
    accounts: BTreeMap<AccountId, Account>,
    natures: BTreeMap<AccountId, Nature>,
}

impl Chart {
    pub fn new() -> Self {
        Chart::default()
    }

    /// Add an account with no declared nature. Legal: a clearing or suspense account that
    /// nets to zero over a period has no meaningful nature, and forcing one would be a
    /// worse lie than leaving it absent.
    pub fn with(mut self, a: Account) -> Self {
        self.accounts.insert(a.id.clone(), a);
        self
    }

    pub fn with_nature(mut self, a: Account, n: Nature) -> Self {
        self.natures.insert(a.id.clone(), n);
        self.accounts.insert(a.id.clone(), a);
        self
    }

    pub fn account(&self, id: &AccountId) -> Option<&Account> {
        self.accounts.get(id)
    }

    pub fn nature(&self, id: &AccountId) -> Option<Nature> {
        self.natures.get(id).copied()
    }

    pub fn currency_of(&self, id: &AccountId) -> Option<&Currency> {
        self.accounts.get(id).map(|a| &a.currency)
    }

    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    /// Accounts in stable order. `BTreeMap`, so a trial balance printed twice is
    /// byte-identical — the determinism obligation reaching the report layer.
    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    /// Which side a signed minor amount presents on for this account.
    ///
    /// `None` when the account has no declared nature, which is honest absence rather than
    /// a guess: a suspense account's movements have no meaningful side, and defaulting to
    /// `Debit` would put them on a report where they do not belong.
    pub fn side(&self, id: &AccountId, minor: i128) -> Option<Side> {
        let n = self.nature(id)?;
        let positive_is_debit = n.positive_is_debit();
        Some(match (minor >= 0, positive_is_debit) {
            (true, true) | (false, false) => Side::Debit,
            _ => Side::Credit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart() -> Chart {
        Chart::new()
            .with_nature(Account::new("cash", "bank", "USD"), Nature::Asset)
            .with_nature(Account::new("deposits", "bank", "USD"), Nature::Liability)
            .with_nature(Account::new("fees", "bank", "USD"), Nature::Income)
            .with(Account::new("suspense", "bank", "USD"))
    }

    #[test]
    fn the_debit_credit_convention_is_derived_and_not_stored() {
        let c = chart();
        // A positive movement on an asset is a debit; on a liability, a credit.
        assert_eq!(c.side(&AccountId::new("cash"), 100), Some(Side::Debit));
        assert_eq!(c.side(&AccountId::new("deposits"), 100), Some(Side::Credit));
        // And the signs reverse, which is the whole of the convention.
        assert_eq!(c.side(&AccountId::new("cash"), -100), Some(Side::Credit));
        assert_eq!(c.side(&AccountId::new("deposits"), -100), Some(Side::Debit));
    }

    #[test]
    fn an_account_with_no_nature_has_no_side_rather_than_a_default_one() {
        // Honest absence, applied to presentation. A suspense account defaulting to `Debit`
        // would appear on a report it has no business being on, and nothing would say why.
        assert_eq!(chart().side(&AccountId::new("suspense"), 100), None);
    }

    #[test]
    fn income_and_expense_are_not_balance_sheet_accounts() {
        assert!(Nature::Asset.is_balance_sheet());
        assert!(Nature::Liability.is_balance_sheet());
        assert!(Nature::Equity.is_balance_sheet());
        assert!(!Nature::Income.is_balance_sheet());
        assert!(!Nature::Expense.is_balance_sheet());
    }

    #[test]
    fn the_chart_answers_the_two_questions_the_kernel_actually_asks() {
        let c = chart();
        assert!(c.account(&AccountId::new("cash")).is_some());
        assert!(c.account(&AccountId::new("nope")).is_none());
        assert_eq!(c.currency_of(&AccountId::new("cash")), Some(&Currency::new("USD")));
    }

    #[test]
    fn accounts_iterate_in_stable_order() {
        let names: Vec<_> = chart().accounts().map(|a| a.id.to_string()).collect();
        assert_eq!(names, vec!["cash", "deposits", "fees", "suspense"]);
    }
}
