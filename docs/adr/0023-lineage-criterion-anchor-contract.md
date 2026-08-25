# ADR 0023 — TEPP-owned Event Lineage criterion anchor

**Decision status:** Accepted  
**Implementation maturity:** active-PR — the transport contract is implemented on this PR; the registered TEPP analysis remains accepted-target  
**Date:** 2026-08-25
**Supersedes:** None; complements ADR 0014's scientific promotion boundary.

## Context

LineageWeave uses fast-mlsirm to estimate relative Event Lineage channel
information. Internal response structure is not independent criterion
validity, and a consumer-authored validity flag would not make it independent.

## Decision

TEPP owns the `tepp.lineage_criterion_anchor.v1` result artifact and the
`tepp-lineage-criterion-v1` / `lineage_pair_criterion_anchor` analysis-run
request identity. The artifact binds TEPP's accepted or rejected criterion
decision to one opaque estimation run, immutable snapshot SHA-256, knowledge
cutoff, and positive validated-pair count.
The estimation-run identity uses the canonical lowercase, hyphenated UUID
form in both the executable DTO and the published JSON Schema.

The wire contract does not define an arbitrary correlation threshold, invent a
theta, or allow LineageWeave to reinterpret a rejection. The registered TEPP
analysis implementation and its scientific validation evidence own the
criterion design and acceptance procedure. Until that implementation emits a
digest-bound artifact through the terminal-result contract, consumers must
treat channel weighting as unavailable.

This separation follows the Standards' requirement that an intended score
interpretation and use be stated and supported by appropriate validity
evidence; internal model fit is not silently promoted to evidence for the
Event Lineage use.

## Alternatives considered

1. Let the consumer author a validity flag — rejected because the evidence
   would not be independent of the proposed weights.
2. Treat an accepted transport receipt as validity evidence — rejected because
   transport acceptance does not establish the intended score interpretation.
3. Publish a TEPP-owned, identity-bound outcome — accepted because it keeps
   criterion authority and provenance at the measurement boundary.

## Consequences

- An accepted transport receipt is never a criterion anchor.
- Unknown fields and malformed provenance fail closed.
- Both accepted and rejected outcomes are preserved; only TEPP can author the
  outcome, and a consumer may activate weights only for an exact accepted
  artifact.
- The executable estimator remains a separately gated delivery slice.

## Verification

The Rust contract tests cover accepted and rejected round trips, canonical UUID
identity, malformed provenance, unknown fields, and payload limits. The JSON
Schema is checked against the same canonical UUID examples by the API schema
test suite.

## Rollback and supersession

Rollback stops publishing the result profile while preserving previously issued
versioned artifacts. Supersession requires a new ADR that preserves independent
criterion authority, exact run/snapshot/cutoff binding, and fail-closed consumer
activation.

## Reference

American Educational Research Association, American Psychological
Association, & National Council on Measurement in Education. (2014).
*Standards for educational and psychological testing*. American Educational
Research Association. https://www.testingstandards.net/open-access-files.html
