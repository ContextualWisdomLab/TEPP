# TEPP

TEPP is the **Temporal Event Psychometrics Platform**: a multilingual, temporal,
relational measurement system whose statistical and psychometric arithmetic is
implemented in Rust.

## Current implementation state

The current workspace contains 50 independently documented Rust crates. Each
crate exposes a bounded, tested contract for evidence, temporal semantics,
event and relation reasoning, membership, persistence, simulation, validation,
API exchange, compute planning, or evidence-grounded interpretation. Numerical
and psychometric authority remains on the CPU `f64` reference path; streamed
accelerator plans must preserve the full observation set and fail closed to the
reference path when resources or validation are insufficient.

These are production contracts, not a claim that the complete commercial
estimator, operator workspace, or supported release already exists. Read the
[product and technical gap baseline](docs/product-technical-gap-baseline.md)
before treating a crate as a shipped product capability.
This branch keeps the Rust workspace quality foundation and the bounded
foundation crates. Domain crates expose only tested contracts: immutable
evidence, six-clock temporal values, event mentions/instances, relations,
membership, persistence, splits, simulation, validation, API DTOs, and the
predicted-versus-observed promotion gate.
This branch establishes the Rust workspace, quality-gate foundation, and the
longitudinal within/between decomposition capability. The eleven bounded crates
compile independently. `longitudinal_core` exposes within/between decomposition
and component RMSE APIs; the remaining crates expose no placeholder production
APIs, and domain behavior for them begins in Task 2 with immutable evidence
identifiers and source records.
This branch establishes the Rust workspace and quality-gate foundation. The
bounded crates compile independently. Domain crates expose only validated
production APIs; placeholder surfaces are prohibited.

```text
crates/assertion_clock
crates/available_clock
crates/checkpoint_authority
crates/citation_edge
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
crates/location_membership
crates/prompt_source
crates/corpus_background
crates/modality_source
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
crates/psychometric_core
crates/psychometric_fit
crates/relation_graph
crates/retrospective_edge
crates/revision_order
crates/semantic_core
crates/operational_log
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
crates/validation_core
crates/network_analysis
crates/interpretation_gateway
crates/model_selection
crates/checkpoint_authority
crates/compute_backend
crates/episode_membership
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
