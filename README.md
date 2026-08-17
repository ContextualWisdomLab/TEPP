# TEPP

TEPP is the **Temporal Event Psychometrics Platform**: a multilingual, temporal,
relational measurement system whose statistical and psychometric arithmetic is
implemented in Rust.

## Current implementation state

This branch establishes the Task 1 Rust workspace and quality-gate foundation.
The eleven bounded crates compile independently but intentionally expose no
placeholder production APIs. Domain behavior begins in Task 2 with immutable
evidence identifiers and source records.

```text
crates/evidence_core
crates/temporal_core
crates/event_core
crates/relation_graph
crates/membership_core
crates/persistence_postgres
crates/corpus_split
crates/tepp_simulation
crates/validation_core
crates/tepp_api
crates/copy_identity
```

## Local verification

```bash
python3 scripts/check_workspace_contract.py
python3 scripts/check_docstrings.py
python3 -m coverage run --branch -m unittest discover -s tests/quality -p 'test_*.py'
python3 -m coverage report --fail-under=100 --show-missing

cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace --all-features
cargo doc --workspace --all-features --no-deps
cargo deny check
```

Stable Rust line coverage is measured with `cargo-llvm-cov`. Branch coverage is
measured in a separately pinned nightly lane because Rust branch coverage remains
an unstable compiler capability. A zero denominator is reported explicitly for
this skeleton-only slice; it must never conceal uncovered production behavior.

## Normative documents

- `AGENTS.md`
- `ARCHITECTURE.md`
- `docs/product/prd-v0.4-approved.md`
- `docs/superpowers/plans/2026-08-05-temporal-event-foundation.md`
- `docs/research/standards-and-literature.md`

No release, production-readiness, GPU, database, or statistical-recovery claim is
made by this foundation slice.
