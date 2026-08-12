# TEPP

TEPP is the **Temporal Event Psychometrics Platform**: a multilingual, temporal,
relational measurement system whose statistical and psychometric arithmetic is
implemented in Rust.

TEPP is designed to preserve the difference between source evidence, inferred
events, latent topics, psychometric constructs, structural paths, and published
claims. It does not substitute keyword matching, ordinary embedding clusters,
or unbounded LLM summaries for a fitted and validated measurement model.

## Product contract

The canonical product contract is
[`docs/product/prd-v0.5.md`](docs/product/prd-v0.5.md). It defines stable
requirement identifiers, evidence and temporal workflows, multilingual and
psychometric claim gates, candidate-topic-count selection, CPU/GPU and VRAM
profiles, privacy controls, accessible product surfaces, release slices, and
acceptance evidence. The approved v0.4 design is retained as historical
architecture evidence.

Core invariants include:

- immutable source evidence and exact source spans;
- six distinct clocks and leakage-safe historical cutoffs;
- forward-only state transitions with retrospective provenance separated;
- time-varying cross-classified multiple membership;
- one shared multilingual latent topic identity with native lexical channels;
- no default stopword deletion or TF-IDF/BM25 inferential weighting;
- posterior uncertainty, valid compositional coordinates, and longitudinal
  ESEM/DSEM claim discipline;
- Rust CPU `f64` numerical authority and parity-qualified GPU acceleration;
- evidence-bounded LLM proposals and interpretation;
- purpose-bound PII authorization rather than destructive blanket masking.

## Current implementation state

Protected `main` contains the modular Rust workspace and substantial parts of
the temporal-event foundation:

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
```

Implemented or partially implemented capabilities include immutable evidence,
exact spans, typed six-clock values, bounded interval reasoning, forward-only
relation graphs, event mention/instance separation, weighted multiple
membership, leakage-safe corpus snapshots and splits, deterministic known-truth
simulation, recovery metrics, bitemporal persistence contracts, versioned API
artifacts, modular consumer contracts, and release-evidence generation.

The authoritative capability-maturity ledger is
[`docs/TRACEABILITY.md`](docs/TRACEABILITY.md). Multilingual semantic
measurement, the TRSL-TM estimator, candidate-K selection, topic networks and
clusters, GPU kernels, TDT/CHRONOS intelligence, longitudinal ESEM/DSEM,
evidence-bounded interpretation, and coordinated visual analytics remain
separately gated product slices until protected-main evidence promotes them.

## Local verification

```bash
python3 scripts/check_workspace_contract.py
python3 scripts/check_docstrings.py
python3 -m coverage run --branch -m unittest discover -s tests/quality -p 'test_*.py'
python3 -m coverage report --fail-under=100 --show-missing
python3 scripts/validate_documentation.py

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
an unstable compiler capability. Owned production line and branch coverage must
remain exactly 100%; complete coverage is necessary but does not by itself prove
scientific validity or release readiness.

## Documentation

Start with [`DOCUMENTATION.md`](DOCUMENTATION.md). The normative set includes:

- `docs/product/prd-v0.5.md`;
- `docs/TRD.md`;
- `ARCHITECTURE.md`;
- `docs/UML.md` and `docs/ERD.md`;
- `docs/adr/README.md` and `docs/adr/ADR_POLICY.md`;
- `docs/TEST_STRATEGY.md` and `docs/TRACEABILITY.md`;
- `SECURITY.md`, `docs/THREAT_MODEL.md`, and
  `docs/PRIVACY_DATA_GOVERNANCE.md`;
- `docs/OPERABILITY.md` and `docs/COMPLIANCE_READINESS.md`;
- `docs/research/standards-and-literature.md`.

No release, certification, universal language validity, causal effect, GPU
parity, production SLO, or commercial valuation claim is implied by repository
architecture alone.
