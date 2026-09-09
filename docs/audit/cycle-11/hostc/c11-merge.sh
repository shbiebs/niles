#!/usr/bin/env bash
#
# T11-02 — the deferred merge against the pinned control, on Host C.
#
# WHAT THIS RUNS
#
#   Two arms of the *same build*, differing in one value: NILESTREAM_MERGE_CAPS.
#
#     merge   caps 32 epochs / 4096 rows   the preregistered bounds
#     pinned  merging off                  the policy every late landing takes today
#
#   Both working points, warm-ups discarded but checked, measured replicates interleaved and
#   AB/BA order-balanced, one daemon per replicate on its own port with its own storage.
#
# WHY IT IS SEVEN LINES
#
#   Every refusal this measurement needs already exists in `c11-baselock.sh` and is already
#   proved by that script's `--self-test`: the dirty-repository refusal, the stale-worktree
#   refusal, the occupied-port refusal, the per-replicate deadline, the missing-column
#   refusal, the writer-progress refusal, the toolchain report, the pooled-MAD statistics and
#   the order balancing. A second script would either duplicate all of that — a second source
#   of truth for questions that have exactly one answer, which is the mistake this cycle has
#   already paid for twice — or quietly do without it.
#
#   So the arms are a mode of that script and this file is the name the work order gives the
#   run. The one thing `--merge-arms` adds is a refusal of its own, because the two arms are
#   one binary and nothing about the *build* can tell them apart: every replicate's
#   transcript must carry the merge line, and the caps in it must be the caps that replicate
#   was launched with. A run whose arm rests on the flag having been typed correctly is not
#   an ablation.
#
# WHAT TO DO WITH THE OUTPUT
#
#   Paste it. The gate is the preregistered one — a claim needs BOTH a >=10% relative
#   difference AND >=3 pooled MADs, against the A=A noise floor already measured on this host
#   (worst separation 0.83%, about one pooled MAD). If it does not pass, that is the result:
#   the merge's correctness targets stand on their own and the performance claim is simply
#   not made. Do not re-run until it passes.
#
#   Estimate: the same cost as `--baseline-only`, roughly 25-40 minutes at the defaults.
#
set -u
here="$(cd "$(dirname "$0")" && pwd)"
exec bash "$here/c11-baselock.sh" --merge-arms "$@"
