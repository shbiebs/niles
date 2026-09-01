//! Accounts and parties.
//!
//! An account is an **identity and a currency**, and deliberately nothing else. It has no
//! balance, no status, and no type that carries behaviour. Everything a banking system
//! normally hangs off an account here hangs off the entries instead, because an attribute
//! stored on an account is an attribute that can disagree with the movements that produced
//! it, and reconciling the two is the work this design exists to delete.

use crate::Currency;
use std::fmt;

/// An account identifier.
///
/// Opaque and structureless on purpose. Real institutions encode a chart of accounts, a
/// legal entity, a branch and a product into an account number, and every one of those
/// encodings eventually needs to change while the accounts keep existing. Structure that
/// matters lives in the [`Chart`](crate::Chart), which is data and can be versioned.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(String);

impl AccountId {
    pub fn new(id: impl Into<String>) -> Self {
        AccountId(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Whoever an account belongs to: a customer, a counterparty, the bank itself, a fund, a
/// syndicate participant.
///
/// A party is an identity, not a record. Know-your-customer data, addresses and
/// classifications are held elsewhere and referenced; the kernel needs only to be able to
/// say that two accounts belong to the same party, which is what a netting rule, a limit
/// check, and a participation share all turn on.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Party(String);

impl Party {
    pub fn new(id: impl Into<String>) -> Self {
        Party(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Party {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An account: an identity, an owner, and exactly one currency.
///
/// **One currency, always.** A multi-currency "account" as customers experience it is a set
/// of accounts sharing a party, and movement between them is an FX transaction with two
/// conserved legs — never a conversion inside a single account. This is the kernel's most
/// consequential restriction and the one that makes per-currency conservation checkable at
/// all: if an account could hold two currencies, an entry against it would need a currency
/// of its own, and the invariant that an entry's currency *is* its account's currency
/// would be gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: AccountId,
    pub owner: Party,
    pub currency: Currency,
    /// A free-form classification the chart interprets. The kernel does not read it; it is
    /// carried so that a product library can group accounts without the kernel learning
    /// what a "nostro" is.
    pub class: String,
}

impl Account {
    pub fn new(id: impl Into<String>, owner: impl Into<String>, currency: &str) -> Self {
        Account {
            id: AccountId::new(id),
            owner: Party::new(owner),
            currency: Currency::new(currency),
            class: String::new(),
        }
    }

    pub fn classed(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_account_holds_exactly_one_currency() {
        let a = Account::new("cust-1-usd", "cust-1", "usd");
        assert_eq!(a.currency, Currency::new("USD"));
        // And the customer's euro balance is a *different account* with the same owner.
        let b = Account::new("cust-1-eur", "cust-1", "eur");
        assert_eq!(a.owner, b.owner);
        assert_ne!(a.id, b.id);
        assert_ne!(a.currency, b.currency);
    }

    #[test]
    fn identity_is_opaque_and_carries_no_structure() {
        // A test that documents a decision rather than checking behaviour: nothing in the
        // kernel parses an account id. If this ever stops being true, a chart-of-accounts
        // migration becomes a data migration.
        let a = AccountId::new("11-4820-003-GBP-NOSTRO");
        assert_eq!(a.as_str(), "11-4820-003-GBP-NOSTRO");
        assert_eq!(a, AccountId::new("11-4820-003-GBP-NOSTRO"));
    }

    #[test]
    fn accounts_have_no_balance_field() {
        // Enforced by construction rather than by assertion — `Account` has four fields and
        // none of them is a balance. The test exists so that adding one is a visible act
        // that breaks a named expectation instead of a quiet convenience.
        let a = Account::new("x", "p", "USD").classed("nostro");
        assert_eq!(a.class, "nostro");
        let rendered = format!("{a:?}");
        assert!(!rendered.contains("balance"), "a balance is a fold, never a field");
        assert!(!rendered.contains("status"), "status is a fold over events (M5)");
    }
}
