# ADR 0089 — Loopback export stored-request GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0054. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0088 occupied.

## Context

ADR 0054 retrieves one authorized export identity. Operators still had no
extra-segment GET for the stored naruon authorization request.
Project-history stored-request GET (#455) is LineageWeave-owned.
Interpretation-run stored-request GET (#453) is orchestrator-owned.
Duplicating GET-by-id (#411), retrieval CLI (#417), collection GET/CLI
(#443/#444), export-authorize CLI (#410), Leiden, or GAP-010 would collide
with live PRs. Cancel lineages stay closed. LineageWeave is refused on this
naruon-owned adapter. `NaruonLiveService` stays POST-only.

## Decision

Publish `GET /v1/exports/{export_id}/request` on `AnalysisRunLiveService`.
Extra-segment parse before GET-by-id. Slash/NUL fail closed. Empty body.
Naruon-only. Response is the stored authorization request. Scientific-metric
keys and `tepp.scientific_acceptance.v1` never appear. Cancel extra-segment
stays refused. `NaruonLiveService` stays POST-only.

## Alternatives considered

1. Re-open cancel HTTP — rejected.
2. Return GET-by-id retrieval identity — rejected (ADR 0054).
3. Loopback stored-request GET — accepted.

## Consequences

HTTP 200 is not measurement evidence and is not an ADR 0014 claim. Sequence
remains association, not causation.

## Failure and recovery

LineageWeave, nonempty bodies, extra segments, slash/NUL, missing keys, http
origins, unpublished consumers, credential flags, and metric keys fail closed.

## Verification

- `GET /v1/exports/{export_id}/request` of an authorized export returns the
  stored authorization request without RMSE/`tepp.scientific_acceptance.v1`;
- LineageWeave, GET-by-id path, extra segments, slash/NUL, nonempty body, and
  missing keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the extra-segment GET; POST and GET-by-id remain valid. A
superseding ADR is required to persist the registry, bind a public address,
re-open cancel, emit scientific-acceptance, open LineageWeave on this adapter,
add GET to `NaruonLiveService`, or treat retrieval success as an ADR 0014 claim.

## Related authority

ADR 0054, ADR 0009, ADR 0011, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
