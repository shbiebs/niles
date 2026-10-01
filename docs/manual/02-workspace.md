# 2. The workspace: from a fresh clone to a passing gate

*The construction manual, chapter 2 (cycle 16, C16-12; cycle 15's C15-09). Every `sh` block
below is executed, in order, by `tools/replay.sh docs/manual/02-workspace.md` in a fresh clone,
and a block followed by an `expected` block must print exactly that. `console` blocks are
illustrations and are not run. If a step here does not work as written, the chapter is wrong,
not the reader.*

## 2.1 What the host needs

| need | why | how it is checked |
|---|---|---|
| Linux on x86-64 or macOS on ARM64, 2 cores and 8 GB or more | the hosts every figure in `results/` was produced on | recorded by the replay's log, not enforced |
| Rust **1.95.0** | `rust-toolchain.toml` pins it, so a new Rust release cannot break the gate for reasons unrelated to this code | `make preflight` |
| PostgreSQL **16**, running, with a stock `pg_hba.conf` | the comparisons and the conformance tests drive a real PostgreSQL (decision D-1, cycle 14) | `make preflight` |
| `python3`, `git`, `psql` | the thesis renderer, the reproduction diff, and an independent wire client | the replay's log |

> **Decision card — PostgreSQL is a precondition of the gate.** *Alternatives:* (a) a gate that
> needs only `cargo`; (b) a gate that needs a provisioned PostgreSQL. *Criterion:* the gate must
> run every comparison the thesis reports, against the real system it compares with. *Evidence:*
> before D-1, a fresh container failed the gate one test at a time on a precondition written in
> no file, and a harness that spoke only `trust` authentication passed only on a machine whose
> `pg_hba.conf` had been weakened by hand. *Chosen:* (b), with SCRAM-SHA-256 in the harness's own
> wire client and a checked-in provisioning script. *What would reverse it:* a release in which
> no result depends on PostgreSQL.

## 2.2 The clone

The replay starts inside a fresh clone, so this step is shown rather than run:

```console
$ git clone https://github.com/shbiebs/niles.git
$ cd niles
$ tools/replay.sh docs/manual/02-workspace.md
```

Where the network cannot reach `static.rust-lang.org` to install a pinned channel, build with
`RUSTUP_TOOLCHAIN=stable`, which overrides the pin and is acceptable only where `stable` *is*
1.95.0. The first block checks exactly that.

```sh
cargo --version | cut -d' ' -f1,2
```

```expected
cargo 1.95.0
```

## 2.3 PostgreSQL, provisioned

Start the cluster the way the platform does (on Debian and Ubuntu as root,
`pg_ctlcluster 16 main start`; with Homebrew, `brew services start postgresql@16`), then
provision the role the harness uses. `tools/pg-provision.sh` creates the role `bench` and the
database `bank`, generates a random password into `.pg-bench-password` (mode 600, ignored by
git, never printed), and sets it on the role. Nothing in this repository ever asks for a password
to be typed or pasted.

```sh
sh tools/pg-provision.sh > /dev/null
test -s .pg-bench-password && echo "password file present"
git check-ignore -q .pg-bench-password && echo "and ignored by git"
```

```expected
password file present
and ignored by git
```

`make preflight` names every missing precondition at once, rather than failing one test at a
time. On a ready workspace it prints four lines: the toolchain, the server, and a SCRAM login as
`bench` to each of the two databases the tests use (`bank` and `postgres`):

```sh
mkdir -p target
make -s preflight > target/preflight.out 2>&1 || { cat target/preflight.out; exit 1; }
grep -c '^preflight:' target/preflight.out
```

```expected
4
```

## 2.4 Build and test

```sh
cargo build --release --workspace -q > target/build.out 2>&1 || { tail -n 20 target/build.out; exit 1; }
cargo test --workspace -q > target/test.out 2>&1 || { grep -E 'FAILED|panicked' target/test.out | head -n 20; exit 1; }
grep -c '^test result: ok' target/test.out > /dev/null && echo "every test binary passed"
```

```expected
every test binary passed
```

## 2.5 The checkpoint: one slice of the whole system

Five steps, each the smallest instance of something the thesis claims: **one query, one durable
two-leg event, one derived answer, one refused program, one restart.** They run against
`nilestreamd`, the instrument (decision DB-1: the product engine is PostgreSQL; Nilestream is how
the theory is tested), over the PostgreSQL wire with `psql` as the client.

**Start the daemon on a durable segment.** It refuses to start without `--durable` or an
explicit `--volatile`: an acknowledged write must be on stable storage, and a client cannot tell
the difference from outside.

```sh
mkdir -p target/manual
rm -f target/manual/ledger.seg
nohup target/release/nilestreamd --durable target/manual/ledger.seg --accounts 100 --rounds 1 \
    --port 5433 > target/manual/daemon.log 2>&1 &
echo $! > target/manual/daemon.pid
for _ in $(seq 1 100); do
    psql -h 127.0.0.1 -p 5433 -U anyone -At bank -c "select nilestream_frontier" > /dev/null 2>&1 && break
    sleep 0.2
done
psql -h 127.0.0.1 -p 5433 -U anyone -At bank -c "select nilestream_frontier" > /dev/null && echo up
```

```expected
up
```

**One query**, a point lookup by key. Every answer carries its anchor as its last column; the
lookup is printed without it, because the anchor's value depends on how the base was seeded.

```sh
psql -h 127.0.0.1 -p 5433 -U anyone -At bank \
    -c "select acct, count(amt) from postings where acct = 7 group by acct" | cut -d'|' -f1
```

```expected
7
```

**One durable two-leg event.** A posting set conserves or it does not commit: the one-legged
insert is refused, and the balanced pair commits as one epoch. (The refusal makes `psql` exit
non-zero, which is the point; `|| true` keeps a script that stops on errors going.)

```sh
{ psql -h 127.0.0.1 -p 5433 -U anyone -At bank \
    -c "insert into postings values (900001, 4242, 0, 100)" 2>&1 || true; } | head -n 1
psql -h 127.0.0.1 -p 5433 -U anyone -At bank \
    -c "insert into postings values (900001, 4242, 0, 100), (900001, 4243, 0, -100)"
```

```expected
ERROR:  the insert did not commit
INSERT 0 2
```

**One derived answer**, from the read model rather than from the insert's acknowledgement:

```sh
psql -h 127.0.0.1 -p 5433 -U anyone -At bank \
    -c "select acct, sum(amt) from postings where acct = 4242 group by acct" | cut -d'|' -f1,2
```

```expected
4242|100
```

**One refused program.** The checker refuses, at check time, a transaction that posts one half
of a movement through a helper function — conservation across a call, the obligation that used
to disappear rather than be refused (`crates/niles-lang/tests/mutants/`).

```sh
cat crates/niles-lang/tests/mutants/schema.niles \
    crates/niles-lang/tests/mutants/interprocedural_single_half_post.niles > target/manual/refused.niles
if target/release/nilesc check target/manual/refused.niles > target/manual/refused.out 2>&1; then
    echo accepted
else
    echo refused
fi
grep -o 'NL0300' target/manual/refused.out | head -n 1
```

```expected
refused
NL0300
```

**One restart.** Stop the daemon, start it again on the same segment with the same seed, and ask
the same question. The answer is recovered from the segment, not remembered.

```sh
kill "$(cat target/manual/daemon.pid)"
for _ in $(seq 1 50); do kill -0 "$(cat target/manual/daemon.pid)" 2>/dev/null || break; sleep 0.2; done
nohup target/release/nilestreamd --durable target/manual/ledger.seg --accounts 100 --rounds 1 \
    --port 5433 > target/manual/daemon2.log 2>&1 &
echo $! > target/manual/daemon.pid
for _ in $(seq 1 100); do
    psql -h 127.0.0.1 -p 5433 -U anyone -At bank -c "select nilestream_frontier" > /dev/null 2>&1 && break
    sleep 0.2
done
psql -h 127.0.0.1 -p 5433 -U anyone -At bank \
    -c "select acct, sum(amt) from postings where acct = 4242 group by acct" | cut -d'|' -f1,2
kill "$(cat target/manual/daemon.pid)"
```

```expected
4242|100
```

## 2.6 The gate

`make gate` is what every change must pass before it is called done: the preflight, formatting,
lints, the generated documents, every test, the measurements, the memory budgets, and
**`make reproduce`**, which regenerates every results file the code derives and fails if any
committed copy differs.

> **Decision card — reproduce is inside the gate.** *Alternatives:* (a) a fast gate, with
> reproduce in a separate `gate-full`; (b) one gate that ends with reproduce. *Criterion:* "the
> gate exits 0" must be true of everything the gate claims. *Evidence:* with reproduce outside
> the gate, `make reproduce` exited 2 for a whole round and a results file went stale, while every
> report said "gate exit 0" (cycle 15, DA-11). *Chosen:* (b). *What would reverse it:* a reproduce
> so slow that it stopped being run, which would be measured and reported, not assumed.

```sh
make -s gate > target/gate.out 2>&1 || { tail -n 40 target/gate.out; exit 1; }
echo "gate passed"
```

```expected
gate passed
```

## 2.7 What this chapter does not cover

GBS, which lives in its own repository and is built by its own `make gate`; the comparisons
(chapter 5, `docs/manual/05-counterexamples.md`, written after E27b closes); and Host C, the
author's Mac, which runs the same replay with `PSQL_ADMIN` and `PG_BIN` set for its PostgreSQL.
