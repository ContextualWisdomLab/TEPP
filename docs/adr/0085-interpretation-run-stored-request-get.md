# ADR 0085 — Loopback interpretation-run stored-request GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0071. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0084 occupied.

## Context

ADR 0071 retrieves one accepted interpretation-run identity. Operators still
had no extra-segment GET for the stored create request. Analysis-run
stored-request GET (#377) is naruon-owned. Duplicating GET-by-id (#438),
retrieval CLI (#439), collection GET/CLI, create CLI, Leiden, or GAP-010
would collide with live PRs.

## Decision

Publish `GET /v1/interpretation-runs/{idempotency_key}/request` on
`OrchestratorLiveService`. Extra-segment parse. Slash/NUL fail closed. Empty
body. `scientific_authority` remains false. `tepp.scientific_acceptance.v1`
never appears. Cancel extra-segment stays refused.

## Alternatives considered

1. Re-open cancel HTTP — rejected.
2. Return GET-by-id identity — rejected (ADR 0071).
3. Loopback stored-request GET — accepted.

## Consequences

HTTP 200 is not measurement evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-orchestrator consumers, nonempty bodies, extra segments, slash/NUL,
missing keys, and metric keys fail closed.

## Verification

- `GET /v1/interpretation-runs/{idempotency_key}/request` of an accepted run
  returns the stored create request without RMSE/`tepp.scientific_acceptance.v1`;
- naruon, LineageWeave, GET-by-id path, extra segments, slash/NUL, nonempty
  body, and missing keys fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes the extra-segment GET; POST and GET-by-id remain valid. A
superseding ADR is required to persist the registry, bind a public address,
re-open cancel, emit scientific-acceptance, open naruon or LineageWeave, add
GET to `NaruonLiveService`, or treat retrieval success as an ADR 0014 claim.

## Related authority

ADR 0071, ADR 0069, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
