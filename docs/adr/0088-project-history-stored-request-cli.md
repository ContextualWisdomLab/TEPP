# ADR 0088 — Loopback project-history stored-request CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0087. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0087 occupied.

## Context

ADR 0087 publishes `GET /v1/project-histories/{idempotency_key}/request`.
Operators still had no published binary that mints that GET onto spawned
`tepp-loopback` TCP. Duplicating stored-request GET (#455), GET-by-id (#429),
retrieval CLI (#431), collection GET/CLI (#424/#428), POST CLI (#420),
interpretation-run stored-request CLI (#454), Leiden, or GAP-010 would collide
with live PRs. Cancel lineages stay closed. Naruon is refused on this
LineageWeave-owned adapter.

## Decision

Publish `tepp-project-history-request get` which mints
`lineageweave_project_history_stored_request_exchange` onto spawned
`tepp-loopback` TCP. Empty stdin is admitted. Nonempty leftover stdin, public
bind, `localhost`, `http` origin, unpublished consumer, naruon, and credential
flags fail closed. Dedicated binary so it does not collide with
`tepp-project-histories` (#428), `tepp-project-history-get` (#431), or
`tepp-project-history` (#420). Stored projection `inference_status` remains
`temporal_association_only`. `tepp.scientific_acceptance.v1` never appears.

## Alternatives considered

1. Re-open cancel CLI — rejected.
2. Reuse `tepp-project-history-get` — rejected; that is ADR 0067.
3. Dedicated stored-request binary — accepted.

## Consequences

CLI success is not measurement evidence and is not an ADR 0014 claim.
Sequence remains association, not causation.

## Failure and recovery

Naruon, nonempty leftover stdin, extra segments, slash/NUL, missing keys,
public bind, `localhost`, and metric keys fail closed.

## Verification

- `tepp-project-history-request get` of an accepted history prints the stored
  create request without RMSE/`tepp.scientific_acceptance.v1`;
- naruon, public bind, `localhost`, `http` origin, leftover stdin, slash/NUL,
  and missing keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the published binary; stored-request GET remains valid. A
superseding ADR is required to persist the registry, bind a public address,
re-open cancel, emit scientific-acceptance, open naruon on this adapter, add
GET to `NaruonLiveService`, or treat CLI success as an ADR 0014 claim.

## Related authority

ADR 0087, ADR 0066, ADR 0021, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
