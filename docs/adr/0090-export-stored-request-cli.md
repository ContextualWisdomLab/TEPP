# ADR 0090 — Loopback export stored-request CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0089. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0089 occupied.

## Context

ADR 0089 publishes `GET /v1/exports/{export_id}/request`. Operators still had
no published binary that mints that GET onto spawned `tepp-loopback` TCP.
Duplicating stored-request GET (#457), GET-by-id (#411), retrieval CLI (#417),
collection GET/CLI (#443/#444), export-authorize CLI (#410), project-history
stored-request CLI (#456), interpretation-run stored-request CLI (#454),
Leiden, or GAP-010 would collide with live PRs. Cancel lineages stay closed.
LineageWeave is refused on this naruon-owned adapter. `NaruonLiveService`
stays POST-only.

## Decision

Publish `tepp-export-request get` which mints
`naruon_export_stored_request_exchange` onto spawned `tepp-loopback` TCP.
Empty stdin is admitted. Nonempty leftover stdin, public bind, `localhost`,
`http` origin, unpublished consumer, LineageWeave, and credential flags fail
closed. Dedicated binary so it does not collide with `tepp-export-list`
(#444), `tepp-export-get` (#417), or export-authorize (#410). Response is the
stored authorization request. `tepp.scientific_acceptance.v1` never appears.

## Alternatives considered

1. Re-open cancel CLI — rejected.
2. Reuse `tepp-export-get` — rejected; that is ADR 0055.
3. Dedicated stored-request binary — accepted.

## Consequences

CLI success is not measurement evidence and is not an ADR 0014 claim.
Sequence remains association, not causation.

## Failure and recovery

LineageWeave, nonempty leftover stdin, extra segments, slash/NUL, missing
keys, public bind, `localhost`, and metric keys fail closed.

## Verification

- `tepp-export-request get` of an authorized export prints the stored
  authorization request without RMSE/`tepp.scientific_acceptance.v1`;
- LineageWeave, public bind, `localhost`, `http` origin, leftover stdin,
  slash/NUL, and missing keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the published binary; stored-request GET remains valid. A
superseding ADR is required to persist the registry, bind a public address,
re-open cancel, emit scientific-acceptance, open LineageWeave on this adapter,
add GET to `NaruonLiveService`, or treat CLI success as an ADR 0014 claim.

## Related authority

ADR 0089, ADR 0054, ADR 0009, ADR 0011, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
