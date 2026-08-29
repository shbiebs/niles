.PHONY: gate check test reproduce bootstrap

check:
	cargo check --workspace

test:
	cargo test --workspace

gate: check test
	@echo "gate: determinism + conservation gates are stubs until Phase 2"

bootstrap:
	@echo "bootstrap: three-stage self-hosting build (Appendix E) — not yet implemented"

reproduce:
	@echo "reproduce FIGURE=<id> (Appendix G.10) — not yet implemented"
