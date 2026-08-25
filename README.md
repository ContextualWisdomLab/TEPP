# TEPP

TEPP is the **Temporal Event Psychometrics Platform**: a multilingual, temporal,
relational measurement system whose statistical and psychometric arithmetic is
implemented in Rust.

## Current implementation state

This branch establishes the Rust workspace and quality-gate foundation. The
bounded crates compile independently; domain behavior includes immutable
evidence records, topic measurement, and cutoff-safe analysis execution.

```text
crates/analysis_engine
crates/assertion_clock
crates/available_clock
crates/checkpoint_authority
crates/citation_edge
crates/copied_text
crates/copy_identity
crates/corpus_background
crates/corpus_split
crates/cutoff_clock
crates/derived_sensitivity
crates/document_clocks
crates/encrypted_mapping
crates/event_clock
crates/event_core
crates/evidence_core
crates/inferred_status
crates/intake_authorization
crates/interpretation_gateway
crates/location_membership
crates/longitudinal_core
crates/membership_core
crates/membership_target
crates/modality_source
crates/model_selection
crates/network_analysis
crates/operational_log
crates/outcome_order
crates/payload_bound
crates/persistence_postgres
crates/prediction_contradiction
crates/prompt_source
crates/provider_receipt
crates/psychometric_fit
crates/relation_graph
crates/retrospective_edge
crates/revision_order
crates/semantic_core
crates/service_tls
crates/stopword_deletion
crates/style_source
crates/subevent_containment
crates/summarizes_edge
crates/support_edge
crates/system_clock
crates/temporal_core
crates/tepp_api
crates/tepp_simulation
crates/topic_lineage
crates/topic_measurement
crates/validation_core
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

The active stacked analysis-engine slice adds a bounded executable readiness path
from an accepted run to a digest-bound terminal artifact. It is not yet
implemented-main and does not replace scientific estimator contracts.
