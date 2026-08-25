# ADR 0021 — Deterministic cutoff-safe analysis-run execution

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on the active product branch; not implemented-main
**Date:** 2026-08-21
**Supersedes:** None; complements ADR 0002, ADR 0003, ADR 0011, ADR 0013, and the terminal-result contract.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

TEPP already accepts an analysis request and can describe a completed result,
but a consumer needs a demonstrable path between those contracts. Without one
bounded execution slice, an accepted run is only a receipt and consumers cannot
verify cutoff safety, multiple-membership preservation, or artifact identity.

## Decision

Add the standalone `analysis_engine` Rust crate as the first executable vertical
slice. It consumes a request, an accepted receipt, and a bounded identity-free
evidence snapshot. It:

- excludes evidence whose `available_time` is later than the request's
  `knowledge_cutoff`;
- preserves multiple-membership assignments by summing their counts rather than
  reducing an evidence unit to one group;
- binds the result to the accepted run and source snapshot;
- verifies request/receipt idempotency identity before scanning the corpus;
- emits a canonical SHA-256-digested `AnalysisArtifact` and the versioned
  `AnalysisRunTerminalResult` from `tepp_api`;
- returns a content-redacted failed terminal result when no evidence is
  eligible; and
- remains a readiness/counting slice, not latent-variable, topic, or
  psychometric estimator authority.

The engine is deterministic, synchronous, bounded to `100_000` evidence units,
and CPU-only. Scientific estimators and their Rust CPU `f64`/GPU parity
contracts remain separate boundaries under ADR 0001 and ADR 0006.

For the `trsl_topic_lineage_v1` output profile, the engine may invoke the
ADR-0012 `topic_measurement` CPU `f64` reference estimator through its validated
`ReferenceTopicInput`. The engine does not reimplement or reinterpret the
estimator. It binds the request snapshot and cutoff, then emits a canonical
`tepp.trsl_topic_lineage.v1` artifact containing only the selected seed,
iteration/objective evidence, topic count, evidence count, fitted
predecessor/successor topic edges, connectable-post count, and lineage count.
The artifact is bounded, digest-bound, and self-validating; invalid or
non-converged estimation returns no partial artifact. Production selection of
`K` remains governed by ADR 0012 and `model_selection`, outside this executor.

## Alternatives considered

1. Keep the API as contracts only — rejected because an accepted run would not
produce a consumer-verifiable terminal outcome.
2. Put execution into `tepp_api` — rejected because transport contracts and
   scientific execution would become one service boundary.
3. Add a bounded standalone engine behind the existing contracts — accepted
   because it is independently testable and composable without shared tables.

## Consequences

Consumers can run a reproducible readiness check while seeing only opaque
identifiers, bounded counts, temporal extrema, and a digest. The engine does
not expose source text or identity mappings and does not claim a psychometric
measurement. The initial linear scan is intentionally simple; a production
large-corpus adapter must stream snapshots and preserve the same artifact
semantics before raising the bound.

LineageWeave may consume the topic-lineage artifact as completed model evidence
beside, but never inside, the project-history temporal-association claim. The
two contracts keep separate schema identities and inference-status copy.

## Verification

The stacked PR includes Rust unit and integration tests for cutoff exclusion,
multiple-membership summation, snapshot binding, duplicate identities, empty
eligibility, receipt validation, and package identity. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

The topic-lineage execution contract additionally verifies a synthetic
known-topic corpus, exact request/snapshot/cutoff binding, canonical artifact
round-trip and digest stability, predecessor/successor count consistency, and
fail-closed tamper/non-convergence paths.

The supporting research and APA 7th citations are recorded in
`docs/doctoring/analysis-engine-v1.md` and the standards register.

## Rollback and supersession

Rollback removes the `analysis_engine` workspace member and stops publishing
the readiness artifact while preserving the request and terminal-result DTOs.
No persisted schema migration is introduced. Supersession requires a new ADR
if execution changes cutoff semantics, artifact authority, privacy fields, or
scientific estimands.
