# TEPP

TEPP is the **Temporal Event Psychometrics Platform**: a multilingual, temporal,
relational measurement system whose statistical and psychometric arithmetic is
implemented in Rust.

## Current implementation state

This branch establishes the Task 1 Rust workspace and quality-gate foundation.
The eleven bounded crates compile independently. Domain crates expose only
validated production APIs; placeholder surfaces are prohibited.

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
crates/psychometric_core  # input gates, CWC/event-time/contextual, irregular residual lag, Rubin T, strong means, Driver Eq. 3 TDPRED/TIPRED maps including Eq. 5 of TIPREDEFFECT; not a full ESEM/DSEM estimator
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
any crate whose lane still ships no executable behavior; it must never conceal
uncovered production behavior.

## Normative documents

- `AGENTS.md`
- `ARCHITECTURE.md`
- `docs/product/prd-v0.4-approved.md`
- `docs/superpowers/plans/2026-08-05-temporal-event-foundation.md`
- `docs/research/standards-and-literature.md`

Validated statistical-recovery APIs exist only inside `psychometric_core`: OLS
loading recovery on already-mapped coordinates, posterior-draw point estimates,
the Rubin total-variance identity `T = U_bar + (1 + 1/m) B`, CWC/event-time/
contextual recovery maps, and two-group OLS latent-mean comparison gated behind
typed strong/strict invariance evidence (`LatentMeanComparisonEvidence`; metric,
weak, or configural status cannot reduce to a passing flag). No release,
production-readiness, GPU, or database claim is made by this foundation slice,
and no crate yet implements a full ESEM/DSEM estimator (the two-group OLS
invariance gate is not MGCFA).
