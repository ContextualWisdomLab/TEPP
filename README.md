# TEPP

TEPP is the **Temporal Event Psychometrics Platform**: a multilingual, temporal,
relational measurement system whose statistical and psychometric arithmetic is
implemented in Rust.

## Current implementation state

This branch preserves the protected-main Rust workspace and quality-gate
foundation. The workspace crates expose tested contracts only: immutable
evidence identities and exact spans, six-clock temporal values with Allen
algebra and cutoff eligibility, event mentions/instances with
evidence-layer intelligence gates, a forward-only relation graph,
cross-classified membership, bitemporal persistence, leakage-safe corpus
splits, simulation manifests, claim-promotion validation, API DTOs, the
purpose-bound privacy envelope, and longitudinal within/between
decomposition. It adds the independently usable `analysis_engine` vertical
slice: bounded cutoff-safe readiness work that emits a digest-bound terminal
artifact or a redacted no-eligible-evidence result. That slice is active-PR
evidence, not a psychometric estimator or a release claim.

```text
crates/evidence_core
crates/semantic_core
crates/temporal_core
crates/event_core
crates/relation_graph
crates/membership_core
crates/persistence_postgres
crates/corpus_split
crates/tepp_simulation
crates/validation_core
crates/tepp_api
crates/analysis_engine
crates/prompt_source
crates/corpus_background
crates/modality_source
crates/copied_text
crates/style_source
crates/stopword_deletion
crates/copy_identity
crates/provider_receipt
crates/intake_authorization
crates/summarizes_edge
crates/outcome_order
crates/retrospective_edge
crates/payload_bound
crates/inferred_status
crates/support_edge
crates/system_clock
crates/event_clock
crates/assertion_clock
crates/cutoff_clock
crates/available_clock
crates/document_clocks
crates/revision_order
crates/encrypted_mapping
crates/citation_edge
crates/psychometric_fit
crates/subevent_containment
crates/prediction_contradiction
crates/provider_receipt
crates/operational_log
crates/service_tls
crates/derived_sensitivity
crates/longitudinal_core
crates/topic_lineage
crates/network_analysis
crates/interpretation_gateway
crates/model_selection
crates/checkpoint_authority
crates/membership_target

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
