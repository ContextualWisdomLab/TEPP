# ADR 0092 — Loopback temporal-context stored-request CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0091. Does not re-open cancel lineages
or collection GET. Does not supersede ADR 0014. Unique versus protected main;
0026–0091 occupied including #463=0091, #459=0090, #457=0089.

## Context

ADR 0091 publishes `GET /v1/temporal-context/{idempotency_key}/request`.
Operators still had no published binary that mints that GET onto spawned
`tepp-loopback` TCP. Duplicating stored-request GET (#463), GET-by-id (#451),
retrieval CLI (#452), temporal-context CLI (#414), project-history
stored-request CLI (#456), interpretation-run stored-request CLI (#454),
export stored-request CLI (#459), Leiden, or GAP-010 would collide with live
PRs. Cancel and collection lineages stay closed. Naruon is refused on this
LineageWeave-owned adapter. `NaruonLiveService` stays POST-only.

## Decision

Publish `tepp-temporal-context-request get` which mints
`lineageweave_temporal_context_stored_request_exchange` onto spawned
`tepp-loopback` TCP. Empty stdin is admitted. Nonempty leftover stdin, public
bind, `localhost`, `http` origin, unpublished consumer, naruon, and credential
flags fail closed. Dedicated binary so it does not collide with
`tepp-temporal-context-get` (#452) or `tepp-temporal-context` (#414). Response
is the stored create request. `inference_status` on the live projection remains
`temporal_association_only`. `tepp.scientific_acceptance.v1` never appears.

## Alternatives considered

1. Re-open cancel CLI — rejected.
2. Reuse `tepp-temporal-context-get` — rejected; that is ADR 0084.
3. Dedicated stored-request binary — accepted.

## Consequences

CLI success is not measurement evidence and is not an ADR 0014 claim.
Sequence remains association, not causation.

## Failure and recovery

Naruon, nonempty leftover stdin, extra segments, slash/NUL, missing keys,
public bind, `localhost`, and metric keys fail closed.

## Verification

- `tepp-temporal-context-request get` of an accepted identity prints the stored
  create request without RMSE/`tepp.scientific_acceptance.v1`;
- naruon, public bind, `localhost`, `http` origin, leftover stdin, slash/NUL,
  and missing keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the published binary; stored-request GET remains valid. A
superseding ADR is required to persist the registry, bind a public address,
re-open cancel or collection, emit scientific-acceptance, open naruon on this
adapter, add GET to `NaruonLiveService`, or treat CLI success as an ADR 0014
claim.

## Related authority

ADR 0091, ADR 0083, ADR 0002, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
