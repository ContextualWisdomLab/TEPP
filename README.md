# TEPP

TEPP is the **Temporal Event Psychometrics Platform**: a multilingual, temporal,
relational measurement system whose statistical and psychometric arithmetic is
implemented in Rust.

## Current implementation state

The repository currently implements 54 independently documented crates rather
than a full commercial release. The implemented crates include topic
measurement, the analysis engine, and psychometric input/recovery gates; they
do not claim a complete commercial estimator, operator workspace, or supported
release.

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
- `psychometric_core`: validated statistical-recovery APIs on already-mapped
  coordinates. It does not implement a full ESEM/DSEM estimator.

```text
crates/analysis_engine
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
