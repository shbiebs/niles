.PHONY: gate check test fmt lint reproduce bootstrap

check:
	cargo check --workspace

test:
	cargo test --workspace

# The memory gate (E18). Separate from `test` for two reasons that are both about the
# instrument rather than about convenience.
#
# `--release`: an allocation count is exact *within* a build profile and not across one —
# the same served query allocates 173,363 times in release and 341,918 in debug. The
# published table is a release build, so the gate that holds it must be one.
#
# `--test-threads=1`: a `#[global_allocator]` is process-wide, so a test allocating on
# another thread lands in whatever region is being measured. The test checks a known-quiet
# region first and says so rather than reporting a wrong number.
#
# Two invocations, not one. `--ignored` *filters out* every non-ignored test, so a single
# `-- --ignored` run never executed `the_counting_allocator_is_actually_installed` or
# `every_measured_scenario_has_a_budget_and_the_lists_agree` — and `memprobe` is outside the
# workspace, so `make test` never ran them either. Two of this gate's own checks were
# therefore running nowhere. The first line runs them; the second runs the measurement.
# Both lines are `--test-threads=1` for the same reason: the counters are process-global, so
# a second test allocating on another thread lands in whichever region is being measured.
# `a_reading_reports_what_a_region_kept_apart_from_what_it_touched` measured 58 bytes of
# another test's allocations the first time it was ever actually run.
memory:
	cargo test --release --manifest-path tools/memprobe/Cargo.toml -- --test-threads=1
	cargo test --release --manifest-path tools/memprobe/Cargo.toml \
	      -- --ignored --test-threads=1

fmt:
	cargo fmt --all -- --check
	cargo fmt --all --manifest-path tools/memprobe/Cargo.toml -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings
	# The measurement tool is outside the workspace and is held to the same lints.
	cargo clippy --manifest-path tools/memprobe/Cargo.toml --all-targets -- -D warnings

# The gate every task must pass before it is done.
gate: fmt lint generated test memory

# Every table, status line and results file that is derived from code. Each target
# regenerates from the single source and fails if the committed copy differs, so a
# document that drifts from the artifact fails a build rather than being noticed in
# review — which is the failure mode a status line has when nothing produces it.
generated:
	python3 thesis/gen-appendix-d.py map | diff -u thesis/appendix-d-map.md - 
	python3 thesis/gen-appendix-d.py api | diff -u thesis/appendix-d-api.md -
	cargo run -q -p niles-lang --bin gen-sql-surface -- --check
	cargo run -q -p niles-lang --bin gen-spec-conformance -- docs/SPEC-LANGUAGE.md --check
	cargo run -q -p niles-lang --bin gen-keyword-ref -- docs/keywords.md
	python3 thesis/check-citations.py
	python3 thesis/include-results.py --check

bootstrap:
	@echo "bootstrap: three-stage self-hosting build (Appendix E) — not yet implemented"

# Regenerate every results file produced by an ignored measurement test or a --render step,
# then fail if a committed file differs.
#
# The point is that a results file nobody can regenerate is a results file nobody can check,
# and three of these were produced by `#[ignore]`d tests with no guard at all: they could
# drift from the code that made them and no build would notice.
#
# Deliberately does NOT re-run the durable E16 measurement (`bench --run`): that is
# wall-clock, needs a live PostgreSQL, and is machine-dependent, so a diff against a
# committed CSV would fail on any machine but the one that produced it. `--render` re-derives
# the table from the committed CSVs, which is the part that must not drift.
#
# `nilestream sweep` needs the release binary, so `make reproduce` after `cargo build
# --release -p nilestream`; the E12 sweep is deterministic and its diff is meaningful.
#
# E21 (`--example wire_cost`) is excluded for the same reason: it is wall-clock, it times two
# write paths against each other over a socket, and its ratios move a few percent between
# runs on the same machine. Re-run it when the reply path changes.
reproduce:
	python3 thesis/gen-appendix-d.py map > thesis/appendix-d-map.md
	python3 thesis/gen-appendix-d.py api > thesis/appendix-d-api.md
	cargo run -q -p niles-lang --bin gen-sql-surface
	cargo run -q -p niles-lang --bin gen-spec-conformance -- docs/SPEC-LANGUAGE.md
	cargo run -q -p niles-lang --bin gen-keyword-ref -- docs/keywords.md
	cargo test -p niles-lang --test corpus_obligations
	cargo test -p niles-lang --test solver_verdicts -- --ignored
	cargo test -p nilestream-optimizer --test unnest_corpus -- --ignored
	cargo test -p nilestream-server --test psql_conformance -- --ignored transcript
	cargo run --release -p bank-bench --bin bench -- --render
	cargo run --release -p experiments -- e1 e4 e8
	cargo run -q --release --manifest-path tools/memprobe/Cargo.toml
	./target/release/nilestream sweep examples/demo_bank.niles ledger_balance > results/e12_phase_compiled.csv
	python3 thesis/include-results.py
	git diff --exit-code -- results/ thesis/ docs/SPEC-LANGUAGE.md docs/keywords.md
