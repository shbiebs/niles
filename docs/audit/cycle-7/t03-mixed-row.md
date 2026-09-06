# T-03 — the mixed row, and what checking it against the probe found

E19 measured each half of the mixed workload and never the whole: `point` is readers alone and
`durable` is writers alone. Isolation is exactly what hides the anchor-mismatch fallback — a keyed
read is discarded when a writer has moved the view's frontier past the reader's anchor, so with no
writer the rate is 0.0% whatever the engine does. That is why the audit had to build
`probes/mixed` outside the repository to ask, and why F-27 survived three cycles.

The level now runs in the harness: readers = the connection count, writers = ⌈readers/2⌉, both
roles together for `--mixed-seconds` (default 4). Writers are half because a durable append costs a
barrier; matching them one-for-one measures a queue at the sealer rather than contention at the
base. The row carries reads/s, read p50/p99, writes/s, write p99, **fallback rate**, `max_batch`,
lock wait p99 and base epochs.

## The renderer refuses a row it cannot vouch for

A `mixed` row exists to carry the fallback rate. Without it the throughput figures are the `point`
and `durable` rows again under a name that claims more, so a level whose rate is `None` renders as
**REFUSED** and its throughput is not printed at all — printing it beside an `n/a` invites the one
reading the column exists to block, that the level ran and the mechanism was fine. The count of
refusals is written into the document, so a refusal cannot be scrolled past.

The median is taken with `unwrap_or(0.0)` rather than `expect`, deliberately: deleting the refusal
must produce **the wrong publishable row** — 0.00% beside a full set of figures — and not a crash,
because that is what the guard has to be able to observe. Reverted, the guard fails on the
assertion and prints the row that would have shipped.

## Checking it against the probe found a property of the instrument

The first comparison had the E19 row 38% faster than `probes/mixed` at the same reader and writer
counts. The cause is not either harness being wrong. **A mixed level's read rate is a function of
accumulated history**, because a reconstruction folds an account's postings and the writers in the
level are lengthening them as it runs. Measured on this host, same engine, same counts, only the
phase length varying:

| epochs sealed | mixed reads/s | read p50 |
|--:|--:|--:|
| 6,925 | 23,232 | 143 µs |
| 20,436 | 18,998 | 177 µs |
| 38,466 | 12,208 | 288 µs |

A 1.9× spread in read throughput with nothing about the engine changed. The probe runs three
phases and reaches its mixed phase against a base an order of magnitude larger than the one E19's
level starts from, which is most of the 38%.

So the row now reports `base_epochs`. Without it two mixed rows are not comparable, and the
temptation to compare them anyway is strong — they carry the same column names.

## What reproduces, and what does not

At matched base size (E19's level ended at 10,293 epochs; the probe's own curve interpolated to
that point):

| | probe | E19 | ratio |
|---|--:|--:|--:|
| reads/s | 22,177 | 26,872 | 1.21 |
| read p50 | 151 µs | 116 µs | 0.77 |
| writes/s | ~3,086 | 2,663 | 0.86 |
| write p99 | ~1,668 µs | 1,794 µs | 1.08 |
| fallback | 0.00% | 0.00% | — |

**The shape reproduces**: reads fall to ~0.4× of readers-alone once writers are present, writes
fall, and the fallback rate is 0% on both since T-02. **The absolute rates do not agree within
MAD**, and that is stated rather than rounded: E19's own run-to-run MAD is 260 reads/s (1%), so a
21% gap is far outside it.

The residual is warm state, not engine behaviour. The probe pre-reads every account before its
first phase, so its view starts fully resident; E19's engine arrives at the mixed level warmed by
whatever the `point`, `fold` and `durable` levels touched. Seeding is identical (one round each),
so that is not it. Making the two agree in absolute terms would mean giving the E19 level the
probe's warm-up, which would change what the other E19 levels measure — a worse trade than
reporting the base size and comparing at matched history.

**This is a limitation of the comparison, not a result about the engine.** The row is the
instrument the work order asked for; the probe remains the more controlled measurement, and the two
agree on the only thing they were built to disagree about.

## Not yet measured on a host

`results/E19-scaling/mixed.csv` carries one `NOT RUN` row: the level was added after the last
published host run, and container numbers are not published into the committed tree. The first
`--publish` run on Host C fills it.
