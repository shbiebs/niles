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
	cargo run -q -p niles-lang --bin gen-sql-surface -- --check

bootstrap:
	@echo "bootstrap: three-stage self-hosting build (Appendix E) — not yet implemented"

# Regenerate every results file produced by an ignored measurement test or a --render
# step, then fail if a committed file differs. Deliberately does NOT re-run the durable
# E16 measurement (`bench --run`): that is wall-clock and machine-dependent.
reproduce:
	cargo run -q -p niles-lang --bin gen-sql-surface
	cargo test -p niles-lang --test solver_verdicts -- --ignored
	cargo test -p nilestream-optimizer --test unnest_corpus -- --ignored
	cargo run --release -p bank-bench --bin bench -- --render
	cargo run --release -p experiments -- e1 e8
	git diff --exit-code -- results/
