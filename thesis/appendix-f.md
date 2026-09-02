# Appendix F. A Runnable Conservation-of-Money Reference Model

This appendix specifies the reference oracle 𝒪 (Section 3.11) as executable code. **This is the one component of the system that is implemented and passing tests today.** The full program ships in the artifact; reproduced here is its semantic core, which is short enough to audit by reading — and that shortness *is* the point. Correctness of the large system is defined as agreement with a small program anyone can check.

## F.1 Design Rules

1. **No optimization of any kind.** The base is a vector of epochs; every read folds a prefix. Any cleverness here would make the oracle a second system that could itself be wrong.
2. **No shared mutable state.** Single-threaded, message in, answer out.
3. **Exact integer money at the currency's own scale.** Never floating point; never a hard-coded scale of two, since real currency scales range from zero to three.
4. **It implements the LTS of Section 3.11, transition by transition, and nothing else.**

## F.2 Semantic Core

```rust
// ---- Values ---------------------------------------------------------
type Minor = i128;                       // exact minor units at the currency's scale

struct Cur(u32);                         // currency index; scale declared per currency
struct Acct(u64);

struct Posting { txn: u64, acct: Acct, cur: Cur, amt: Minor, valid: i64 }

enum Row {
    Post(Posting),
    Hold  { id: u64, acct: Acct, cur: Cur, amount: Minor, valid: i64 },
    Resolve { hold: u64, outcome: Outcome, valid: i64 },   // Post(amt) | Void | Expire
}

struct EpochRec { id: u64, parent: [u8; 32], hash: [u8; 32], rows: Vec<Row> }

// ---- The base -------------------------------------------------------
struct Oracle { epochs: Vec<EpochRec>, idem: HashSet<String> }

impl Oracle {
    // Admission: idempotency, then the commit rule (per (txn, currency) balance),
    // then seal immediately. The oracle's epoch is one batch; tau is irrelevant here.
    fn submit(&mut self, key: &str, rows: Vec<Row>) -> Result<u64, Reject> {
        if self.idem.contains(key) { return Err(Reject::Duplicate); }
        let mut sums: HashMap<(u64, Cur), Minor> = HashMap::new();
        for r in &rows { if let Row::Post(p) = r {
            *sums.entry((p.txn, p.cur)).or_insert(0) += p.amt;
        }}
        if sums.values().any(|s| *s != 0) { return Err(Reject::Unbalanced); }
        // ... chain, append, record key ...
    }

    // Every read is a fold of a prefix. No caches, no indexes, no cleverness.
    fn ledger_balance(&self, a: Acct, c: Cur, anchor: u64) -> Minor { /* fold postings */ }

    // Available balance = posted minus unresolved holds. The regulator's distinction,
    // expressed as two folds over the same base.
    fn available_balance(&self, a: Acct, c: Cur, anchor: u64) -> Minor {
        self.ledger_balance(a, c, anchor) - self.unresolved_holds(a, c, anchor)
    }

    // "What did we believe at system epoch `sys` about valid time <= `valid_upto`?"
    fn bitemporal(&self, a: Acct, c: Cur, sys: u64, valid_upto: i64) -> Minor { /* … */ }

    // The global invariant, checkable at every epoch, per currency.
    fn conservation_ok(&self, anchor: u64) -> bool { /* all currency sums are zero */ }

    fn verify_chain(&self) -> bool { /* recompute the chain forward from genesis */ }
}
```

Three properties of this code deserve comment. **Holds are rows, not fields**: a reservation is admitted as an appended row and resolved by another appended row, so the oracle has no update path at all and the available/ledger distinction is two folds rather than two stored numbers. **Conservation quantifies over currency**, which is what makes cross-currency imbalance detectable: a transaction moving −100 in one currency and +100 in another is rejected even though its arithmetic total is zero. **Bitemporality is two filters**, one per axis, which is the entire implementation of "what did we believe then."

## F.3 What the Suite Does With It

**Differential testing.** The harness generates seeded streams across all benchmark mixes, feeds identical admitted streams to 𝒪 and to Nilestream, and compares every observable: balances (ledger and available) at every rung's chosen anchor, bitemporal queries, chain digests, admission decisions including idempotency conflicts and their fingerprint-mismatch variant, and lineage.

**Invariant sweeps.** Per-currency conservation at every epoch of every run; the available-balance identity at every anchor; FX-atomicity.

**Algebra laws as properties.** For random evict/reconstruct schedules applied to Nilestream, answers must equal 𝒪's fold at the same anchor — the executable mirror of Theorem 4.1 — and integration/differentiation round-trips must agree as canonical Z-sets at every epoch (H-F2).

**Fault campaigns.** Crash-recovery, duplicate and reordered delivery, eviction storms, recovery mid-upquery. 𝒪, being a pure fold, is the fixed point against which the chaos is measured. The five partial-state anomalies of §4.2 are written as tests that *must fail to reproduce them*.

**Confidentiality extension.** Under the committed-amount profile, the suite checks that the sum of commitments opens to the oracle's sum of values per currency, which is the property the homomorphic scheme is chosen for.

## F.4 Current Test Coverage

Implemented and passing in the artifact today: balanced transfer conservation; rejection of unbalanced transactions; rejection of duplicate idempotency keys; FX legs balancing per currency separately; rejection of a transaction whose currencies net to zero only by mixing them; bitemporal answers across a backdated correction; hold placement, partial capture with remainder release, void, and expiry; the available-balance identity; and tamper detection via chain verification. These are unit tests of the *oracle*, not measurements of a system — the distinction Chapter 7 states plainly.

## F.5 What Acceptance Means

A green suite means Nilestream's observable behaviour is indistinguishable from a short, deliberately stupid fold, under the declared contracts, across every tested schedule and fault. That is the operational content of "correct" everywhere this thesis uses the word. It also means the suite is portable: any future implementation of this theory — anyone's, in any language — can claim conformance by passing the same suite against the same oracle, which is why the oracle and the suite are listed as an engineering contribution in their own right.
