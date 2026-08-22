# ADR 0017 — Deterministic cutoff-safe analysis-run execution

**Decision status:** Accepted
**Implementation maturity:** active-PR — stacked on PR #157; not implemented-main
**Date:** 2026-08-21
**Supersedes:** None; complements ADR 0002, ADR 0003, ADR 0011, ADR 0013, and the terminal-result contract introduced by PR #157.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

TEPP already accepts an analysis request and can describe a completed result,
but a buyer needs a demonstrable path between those contracts. Without one
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

## Alternatives considered

1. Keep the API as contracts only — rejected because an accepted run would not
   produce a buyer-verifiable terminal outcome.
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

## Verification

The stacked PR includes Rust unit and integration tests for cutoff exclusion,
multiple-membership summation, snapshot binding, duplicate identities, empty
eligibility, receipt validation, and package identity. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

The supporting research and APA 7th citations are recorded in
`docs/doctoring/analysis-engine-v1.md` and the standards register.

## Rollback and supersession

Rollback removes the `analysis_engine` workspace member and stops publishing
the readiness artifact while preserving the request and terminal-result DTOs.
No persisted schema migration is introduced. Supersession requires a new ADR
if execution changes cutoff semantics, artifact authority, privacy fields, or
scientific estimands.
