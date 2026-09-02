# Appendix F. A Runnable Conservation-of-Money Reference Model

This appendix specifies the reference oracle 𝒪 (Section 3.11) as executable code. **This is the one component of the system that is implemented and passing tests today.** The full program is `crates/conservation-suite`; reproduced here is its semantic core, extracted from the source, and it is short enough to audit by reading — and that shortness *is* the point. Correctness of the large system is defined as agreement with a small program anyone can check.

## F.1 Design Rules

1. **No optimization of any kind.** The base is a vector of epochs; every read folds a prefix. Any cleverness here would make the oracle a second system that could itself be wrong.
2. **No shared mutable state.** Single-threaded, message in, answer out.
3. **Exact integer money at the currency's own scale.** Never floating point; never a hard-coded scale of two, since real currency scales range from zero to three.
4. **It implements the LTS of Section 3.11, transition by transition, and nothing else.**

## F.2 Semantic Core

**Extracted from the source, not transcribed from it.** What follows is the region of
`crates/conservation-suite/src/oracle.rs` between its `BEGIN:appendix-f` and `END:appendix-f`
markers, copied here by `thesis/gen-appendix-f.py`, with a test that fails if the two differ.
The version this replaces was a paraphrase that had drifted: its fold signature no longer
matched the oracle's and it declared a type nothing in it used, in an appendix whose opening
sentence calls this the one component implemented and passing tests today.

<!-- BEGIN:appendix-f-core thesis/appendix-f-core.md#verbatim -->

*Generated from `thesis/appendix-f-core.md`. Do not edit by hand.*

*Extracted from `crates/conservation-suite/src/oracle.rs`. Do not edit by hand.*

```rust
    /// Admission: idempotency, then the commit rule (per (txn, currency) balance), then
    /// seal immediately. The oracle's epoch is one batch; the epoch period is a
    /// performance knob in the real engine and is irrelevant to the semantics here.
    pub fn submit(&mut self, key: &str, rows: Vec<Row>) -> Result<u64, Reject> {
        if self.idem.contains(key) {
            return Err(Reject::Duplicate);
        }

        // The commit rule: every (txn, currency) group sums to zero. Note the
        // quantification over currency — a transaction moving -100 in one currency and
        // +100 in another nets to zero arithmetically and is still rejected.
        let mut sums: HashMap<(u64, Cur), Minor> = HashMap::new();
        for r in &rows {
            if let Row::Post(p) = r {
                *sums.entry((p.txn, p.cur)).or_insert(0) += p.amt;
            }
        }
        if sums.values().any(|s| *s != 0) {
            return Err(Reject::Unbalanced);
        }

        // A resolution must refer to a hold that exists and is not already resolved.
        for r in &rows {
            if let Row::Resolve { hold, .. } = r {
                if !self.hold_exists(*hold) || self.hold_is_resolved(*hold) {
                    return Err(Reject::BadResolution);
                }
            }
        }

        let parent = self.epochs.last().map(|e| e.hash).unwrap_or([0; 32]);
        let hash = chain(&parent, &rows);
        let id = self.epochs.len() as u64;
        self.epochs.push(EpochRec {
            id,
            parent,
            hash,
            rows,
        });
        self.idem.insert(key.to_string());
        Ok(id)
    }

    fn rows_upto(&self, anchor: u64) -> impl Iterator<Item = &Row> {
        // `saturating_add`: an anchor of `u64::MAX` means "everything retained", and
        // panicking on it would make the oracle refuse the one question it exists to
        // answer. The oracle is the definition of correctness; it does not get to abort.
        self.epochs
            .iter()
            .take((anchor as usize).saturating_add(1))
            .flat_map(|e| &e.rows)
    }

    fn hold_exists(&self, id: u64) -> bool {
        self.epochs
            .iter()
            .flat_map(|e| &e.rows)
            .any(|r| matches!(r, Row::Hold { id: h, .. } if *h == id))
    }

    fn hold_is_resolved(&self, id: u64) -> bool {
        self.epochs
            .iter()
            .flat_map(|e| &e.rows)
            .any(|r| matches!(r, Row::Resolve { hold, .. } if *hold == id))
    }

    /// The ledger balance: settled postings only. This is the regulator's "ledger
    /// balance" — computed from settled transactions, taking no account of holds.
    ///
    /// Every read is a fold of a prefix. No caches, no indexes, no cleverness.
    pub fn ledger_balance(&self, a: Acct, c: Cur, anchor: u64) -> Minor {
        self.rows_upto(anchor)
            .filter_map(|r| match r {
                Row::Post(p) if p.acct == a && p.cur == c => Some(p.amt),
                _ => None,
            })
            .sum()
    }
```

<!-- END:appendix-f-core -->

Three properties of this code deserve comment. **Holds are rows, not fields**: a reservation
is admitted as an appended row and resolved by another appended row, so the oracle has no
update path at all and the available/ledger distinction is two folds rather than two stored
numbers. **Conservation quantifies over currency**, which is what makes cross-currency
imbalance detectable: a transaction moving −100 in one currency and +100 in another is
rejected even though its arithmetic total is zero. **Every read is a fold of a prefix** —
`rows_upto` and nothing else — which is what makes the oracle worth being the definition.

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
