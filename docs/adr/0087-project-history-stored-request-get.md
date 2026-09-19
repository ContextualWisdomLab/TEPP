# ADR 0087 — Loopback project-history stored-request GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0066. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0086 occupied.

## Context

ADR 0066 retrieves one accepted project-history projection. Operators still
had no extra-segment GET for the stored LineageWeave create request.
Interpretation-run stored-request GET (#453) is orchestrator-owned. Duplicating
GET-by-id (#429), retrieval CLI (#431), collection GET/CLI, POST CLI, Leiden,
or GAP-010 would collide with live PRs. Cancel lineages stay closed. Naruon is
refused on this LineageWeave-owned adapter.

## Decision

Publish `GET /v1/project-histories/{idempotency_key}/request` on
`AnalysisRunLiveService`. Extra-segment parse. Slash/NUL fail closed. Empty
body. LineageWeave-only. Tenant header required. `inference_status` on the
stored projection remains `temporal_association_only`.
`tepp.scientific_acceptance.v1` never appears. Cancel extra-segment stays
refused. `NaruonLiveService` stays POST-only.

## Alternatives considered

1. Re-open cancel HTTP — rejected.
2. Return GET-by-id projection — rejected (ADR 0066).
3. Loopback stored-request GET — accepted.

## Consequences

HTTP 200 is not measurement evidence and is not an ADR 0014 claim. Sequence
remains association, not causation.

## Failure and recovery

Naruon, nonempty bodies, extra segments, slash/NUL, missing keys, missing
tenant, and metric keys fail closed.

## Verification

- `GET /v1/project-histories/{idempotency_key}/request` of an accepted history
  returns the stored create request without RMSE/`tepp.scientific_acceptance.v1`;
- naruon, GET-by-id path, extra segments, slash/NUL, nonempty body, and missing
  keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the extra-segment GET; POST and GET-by-id remain valid. A
superseding ADR is required to persist the registry, bind a public address,
re-open cancel, emit scientific-acceptance, open naruon on this adapter, add
GET to `NaruonLiveService`, or treat retrieval success as an ADR 0014 claim.

## Related authority

ADR 0066, ADR 0028, ADR 0021, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
