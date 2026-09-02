.PHONY: gate check test fmt lint reproduce bootstrap

check:
	cargo check --workspace

test:
	cargo test --workspace

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

# The gate every task must pass before it is done.
gate: fmt lint generated test

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
reproduce:
	python3 thesis/gen-appendix-d.py map > thesis/appendix-d-map.md
	python3 thesis/gen-appendix-d.py api > thesis/appendix-d-api.md
	cargo run -q -p niles-lang --bin gen-sql-surface
	cargo run -q -p niles-lang --bin gen-spec-conformance -- docs/SPEC-LANGUAGE.md
	cargo run -q -p niles-lang --bin gen-keyword-ref -- docs/keywords.md
	cargo test -p niles-lang --test solver_verdicts -- --ignored
	cargo test -p nilestream-optimizer --test unnest_corpus -- --ignored
	cargo test -p nilestream-server --test psql_conformance -- --ignored transcript
	cargo run --release -p bank-bench --bin bench -- --render
	cargo run --release -p experiments -- e1 e4 e8
	./target/release/nilestream sweep examples/demo_bank.niles ledger_balance > results/e12_phase_compiled.csv
	python3 thesis/include-results.py
	git diff --exit-code -- results/ thesis/ docs/SPEC-LANGUAGE.md docs/keywords.md
