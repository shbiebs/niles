#!/usr/bin/env bash
# Every acceptance grep of Work Order 2, in one script, comment-aware.
#
# The previous cycle recorded three acceptance greps that matched only comments — a check
# that passes because the thing it forbids is quoted in the explanation of why it is
# forbidden. So every pattern here is filtered: a line whose first non-space characters open
# a comment does not count, and a line that *retracts* the phrase it names does not count
# either, because retracting a claim requires saying it.
#
#   ./tools/wo2-greps.sh          # from the niles checkout, with gbs beside it
set -uo pipefail
cd "$(dirname "$0")/.."
GBS=../gbs
fails=0

# Prose lines only: drop comment openers and the retraction vocabulary.
prose() { grep -v ':[[:space:]]*\(//\|#\|<!--\|\*\)' | grep -iv 'withdrawn\|not claimed\|used to\|no longer\|which is now\|deleted'; }

expect_none() { # name, then a command whose output must be empty
  local name="$1"; shift
  local out; out=$("$@" 2>/dev/null | prose)
  if [ -n "$out" ]; then
    echo "FAIL $name"; echo "$out" | sed 's/^/       /'; fails=$((fails+1))
  else
    echo "ok   $name"
  fi
}

expect_at_least() { # name, count, then a command
  local name="$1" want="$2"; shift 2
  local n; n=$("$@" 2>/dev/null | wc -l)
  if [ "$n" -lt "$want" ]; then
    echo "FAIL $name: $n < $want"; fails=$((fails+1))
  else
    echo "ok   $name ($n)"
  fi
}

echo "== T-01: Theorem 4.1 names its fragment"
expect_none "the old unrestricted statement is gone" \
  grep -rn "Theorem 4.1 (Epoch-Anchored Reconstruction).\*\* For every REV with circuit Q over base B" thesis/
expect_at_least "Q_lin is named" 3 grep -rho "Q_lin" thesis/04-novel-contributions.md

echo "== T-02: the frontier claims a boundary, not an impossibility"
# The bibliography carries paper titles; one of them is about information-theoretically
# secure secret sharing, which is a subject and not a claim of this thesis.
# The work-order report's "claims weakened" table quotes the old wording beside the new,
# which is its entire job; excluding it is not a loophole, because the wording it quotes is in
# a column headed "Was".
expect_none "no impossibility, no policy quantifier, no competitive guarantee" \
  grep -rn --exclude=references.md --exclude=WORK-ORDER-2-REPORT.md "no policy escapes\|information-theoretic\|impossibility region\|competitive guarantee" thesis/ docs/
expect_at_least "two corollaries" 2 grep -rho "Corollary 4.2.[12]" thesis/04-novel-contributions.md

echo "== T-03/T-04: the soundness clauses and the ladder"
expect_none "the theorem no longer quantifies over LTS traces" \
  grep -rn "every trace of P's execution under the LTS" thesis/
expect_none "l3 is not defined without l2" grep -rn "ℓ₃ = EXACT ∧ X-CONSIST" thesis/
expect_at_least "the ladder is a chain" 1 grep -rn "ℓ₃ = ℓ₂ ∧ X-CONSIST" thesis/03-theoretical-framework.md

echo "== T-08: Theorem 3.7's quantifiers"
expect_none "no universal over anchors for the expectation" \
  grep -rn "for every key k and anchor a: (i)" thesis/
expect_at_least "the checkpoint lookup is charged" 1 grep -rn "log(n/C)" thesis/

echo "== T-09: SQL-Core is what the compiler accepts"
expect_none "no alpha-equivalence claim" grep -rn "α-equivalent circuits" thesis/
expect_at_least "SQL-Core is defined and bounded" 2 grep -rn "SQL-Core" thesis/appendix-h.md

echo "== T-10: lineage and submodularity are conditional"
expect_none "Corollary 4.1.2 is a remark now" grep -rn "Corollary 4.1.2" thesis/
expect_at_least "Remark 4.1.2 exists" 1 grep -rn "Remark 4.1.2" thesis/04-novel-contributions.md

echo "== T-16: the stub crates are gone"
expect_none "no crate imports them" \
  grep -rn "nilestream_storage\|niles_stdlib\|nilestream_lineage" --include=*.rs crates
expect_at_least "ROADMAP records the deletion" 3 grep -n "nilestream-storage\|niles-stdlib\|nilestream-lineage" docs/ROADMAP.md

echo "== T-18: no Elle checker is claimed"
expect_none "the programme does not 'add' one" grep -rn "adds black-box anomaly inference" thesis/

echo "== T-19: the interleaving claim is hedged"
expect_none "exhaustive interleaving checks are not claimed as done" \
  grep -rn "small enough for exhaustive interleaving checks" thesis/

echo "== T-20/T-21: citations and related work"
expect_none "the corrupted reference is fixed" grep -n "H-F1 Lightning" thesis/references.md
expect_none "QLDB is not called commercially viable" grep -n "commercially viable" thesis/10-related-work.md
expect_at_least "the three omitted neighbours are engaged" 3 \
  grep -rho "DynaMat\|Hyder\|Semantic caching\|semantic caching" thesis/10-related-work.md

echo "== T-22/T-23: the instruments and the cuts"
# Two files: the domain and the test that runs it. Counting files rather than mentions,
# because a mention in a comment is not an instrument.
expect_at_least "the inventory domain exists" 2 grep -rli "inventory" crates examples
expect_none "H-S9 has no status entry" grep -n '^id = "H-S9"' thesis/status.toml
expect_at_least "the two new statuses are used" 3 grep -n 'status = "specified"\|status = "argued"' thesis/status.toml

echo "== T-24: the counterproposal runs every defect it has"
expect_none "no swallowed failures" grep -n "|| true" crates/counterproposal/run.sh

echo "== T-14/T-15: GBS"
expect_none "no unwrap_or(false) in products" grep -rn "unwrap_or(false)" $GBS/crates/gbs-products/src
# The verdict may say the hit path is exercised only if it also says where, and what that
# sentence used to be worth: the test it cites measured zero hits until this cycle.
expect_at_least "the G3 verdict says what its hit-path claim is worth" 1 \
  grep -c "zero hits" $GBS/results/G3-verdict.md

echo
if [ "$fails" -eq 0 ]; then echo "all checks passed"; else echo "$fails check(s) failed"; fi
exit "$fails"
