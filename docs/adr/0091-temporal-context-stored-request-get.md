# ADR 0091 — Loopback temporal-context stored-request GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0083. Does not re-open cancel lineages
or collection GET. Does not supersede ADR 0014. Unique versus protected main;
0026–0090 occupied including #459=0090, #457=0089, #456=0088, #455=0087,
#454=0086, #453=0085, #452=0084, #451=0083.

## Context

ADR 0083 retrieves one accepted temporal-context identity. Operators still had
no extra-segment GET for the stored LineageWeave create request.
Project-history stored-request GET (#455) is LineageWeave-owned on a different
path. Interpretation-run stored-request GET (#453) is orchestrator-owned.
Export stored-request GET (#457) is naruon-owned. Duplicating GET-by-id (#451),
retrieval CLI (#452), temporal-context CLI (#414), Leiden, or GAP-010 would
collide with live PRs. Cancel and collection lineages stay closed. Naruon is
refused on this LineageWeave-owned adapter. `NaruonLiveService` stays POST-only.

## Decision

Publish `GET /v1/temporal-context/{idempotency_key}/request` on
`AnalysisRunLiveService`. Extra-segment parse before GET-by-id. Slash/NUL fail
closed. Empty body. LineageWeave-only. Response is the stored create request.
Scientific-metric keys and `tepp.scientific_acceptance.v1` never appear.
`inference_status` on the live projection remains `temporal_association_only`.
Cancel extra-segment stays refused. `NaruonLiveService` stays POST-only.

## Alternatives considered

1. Re-open cancel HTTP — rejected.
2. Return GET-by-id retrieval identity — rejected (ADR 0083).
3. Loopback stored-request GET — accepted.

## Consequences

HTTP 200 is not measurement evidence and is not an ADR 0014 claim. Sequence
remains association, not causation.

## Failure and recovery

Naruon, nonempty bodies, extra segments, slash/NUL, missing keys, http
origins, unpublished consumers, credential flags, and metric keys fail closed.

## Verification

- `GET /v1/temporal-context/{idempotency_key}/request` of an accepted identity
  returns the stored create request without RMSE/`tepp.scientific_acceptance.v1`;
- naruon, GET-by-id path, extra segments, slash/NUL, nonempty body, and
  missing keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the extra-segment GET; POST and GET-by-id remain valid. A
superseding ADR is required to persist the registry, bind a public address,
re-open cancel or collection, emit scientific-acceptance, open naruon on this
adapter, add GET to `NaruonLiveService`, or treat retrieval success as an
ADR 0014 claim.

## Related authority

ADR 0083, ADR 0002, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
