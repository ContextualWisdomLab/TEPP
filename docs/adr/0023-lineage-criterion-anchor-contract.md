# ADR 0023 — TEPP-owned Event Lineage criterion anchor

**Status:** Accepted  
**Date:** 2026-08-25

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

## Consequences

- An accepted transport receipt is never a criterion anchor.
- Unknown fields and malformed provenance fail closed.
- Both accepted and rejected outcomes are preserved; only TEPP can author the
  outcome, and a consumer may activate weights only for an exact accepted
  artifact.
- The executable estimator remains a separately gated delivery slice.

## Reference

American Educational Research Association, American Psychological
Association, & National Council on Measurement in Education. (2014).
*Standards for educational and psychological testing*. American Educational
Research Association. https://www.testingstandards.net/open-access-files.html
