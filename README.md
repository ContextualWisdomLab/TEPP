# TEPP

TEPP is the **Temporal Event Psychometrics Platform**: a multilingual, temporal,
relational measurement system whose statistical and psychometric arithmetic is
implemented in Rust.

## Current implementation state

The repository currently implements 53 independently documented crates rather
than a full commercial release. The implemented crates include topic measurement
and the analysis engine; they do not claim a complete commercial estimator,
operator workspace, or supported release.

- `topic_measurement`: the first production topic-measurement crate. It
  estimates topic proportions from observed counts, maps those proportions into
  additive log-ratio coordinates, and keeps posterior uncertainty attached so
  later psychometric models do not treat raw topic proportions as ordinary
  Euclidean indicators.
- `analysis_engine`: the first production analysis-run crate. It assembles one
  cutoff-safe run from a validated design, documented evidence graph, and
  estimator contract; persists the run with the six TEPP clocks; and emits a
  typed terminal result. The crate does not claim buyer-visible product
  completeness.

```text
crates/analysis_engine
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
longitudinal within/between decomposition capability. The workspace bounded crates
compile independently. `longitudinal_core` exposes within/between decomposition
and component RMSE APIs; the remaining crates expose no placeholder production
APIs, and domain behavior for them begins in Task 2 with immutable evidence
identifiers and source records.
This branch establishes the Task 1 Rust workspace and quality-gate foundation.
The workspace bounded crates compile independently but intentionally expose no
The workspace bounded crates compile independently; Task 1 includes the
The twelve bounded crates compile independently but intentionally expose no
The eleven bounded crates compile independently; Task 1 includes the
implemented `encrypted_mapping` crate with AES-256-GCM sealing and
purpose-bound opening, while the remaining domain behavior begins in Task 2
with immutable evidence identifiers and source records.
The workspace bounded crates compile independently. `derived_sensitivity` inherits
source Restricted/Internal classes onto topic, factor, and relation artifacts
and fails closed on unknown kinds; derivation and blanket PII masking are not
declassification. Other crates still begin domain behavior in Task 2 with
immutable evidence identifiers and source records.
The workspace bounded crates compile independently but intentionally expose no

The eleven bounded crates compile independently but intentionally expose no
placeholder production APIs. Domain behavior begins in Task 2 with immutable
evidence identifiers and source records.
This branch establishes the Rust workspace and quality-gate foundation. The
bounded crates compile independently. Domain crates expose only validated
production APIs; placeholder surfaces are prohibited.

```text
crates/assertion_clock
crates/available_clock
crates/checkpoint_authority
crates/citation_edge
crates/compute_backend
crates/copied_text
crates/copy_identity
crates/corpus_background
crates/corpus_split
crates/cutoff_clock
crates/derived_sensitivity
crates/document_clocks
crates/encrypted_mapping
crates/episode_membership
crates/event_clock
crates/evidence_core
crates/semantic_core
crates/temporal_core
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
crates/corpus_split
crates/tepp_simulation
crates/validation_core
crates/tepp_api
crates/episode_membership
crates/location_membership
crates/prediction_contradiction
crates/prompt_source
crates/provider_receipt
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
crates/topic_measurement
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

No release, production-readiness, GPU, database, or statistical-recovery claim is
made by this foundation slice.

The active stacked analysis-engine slice adds a bounded executable readiness path
from an accepted run to a digest-bound terminal artifact. It is not yet
implemented-main and does not replace scientific estimator contracts.
Validated statistical-recovery APIs exist only inside `psychometric_core`: OLS
loading recovery on already-mapped coordinates, posterior-draw point estimates,
the Rubin total-variance identity `T = U_bar + (1 + 1/m) B`, CWC/event-time/
contextual recovery maps, and two-group OLS latent-mean comparison gated behind
typed strong/strict invariance evidence (`LatentMeanComparisonEvidence`; metric,
weak, or configural status cannot reduce to a passing flag). No release,
production-readiness, GPU, or database claim is made by this foundation slice,
and no crate yet implements a full ESEM/DSEM estimator (the two-group OLS
invariance gate is not MGCFA).
