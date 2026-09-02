# ADR 0086 — Loopback interpretation-run stored-request CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0085. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0085 occupied.

## Context

ADR 0085 publishes `GET /v1/interpretation-runs/{idempotency_key}/request`.
Operators still had no published binary that mints that GET onto spawned
`tepp-orchestrator-loopback` TCP. Duplicating stored-request GET (#453),
GET-by-id (#438), retrieval CLI (#439), collection GET/CLI, create CLI,
analysis-run stored-request CLI (#395), Leiden, or GAP-010 would collide with
live PRs. Cancel lineages stay closed.

## Decision

Publish `tepp-interpretation-run-request get` which mints
`contextual_orchestrator_interpretation_run_stored_request_exchange` onto
spawned loopback TCP. Empty stdin is admitted. Nonempty leftover stdin, public
bind, `localhost`, `http` origin, unpublished consumer, and credential flags
fail closed. `scientific_authority` remains false.
`tepp.scientific_acceptance.v1` never appears.

## Alternatives considered

1. Re-open cancel CLI — rejected.
2. Reuse `tepp-interpretation-run-get` — rejected; that is ADR 0072.
3. Dedicated stored-request binary — accepted.

## Consequences

CLI success is not measurement evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-orchestrator consumers, nonempty leftover stdin, extra segments, slash/NUL,
missing keys, and metric keys fail closed.

## Verification

- `tepp-interpretation-run-request get` of an accepted run prints the stored
  create request without RMSE/`tepp.scientific_acceptance.v1`;
- naruon, LineageWeave, public bind, `localhost`, `http` origin, leftover
  stdin, and missing keys fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes the published binary; stored-request GET remains valid. A
superseding ADR is required to persist the registry, bind a public address,
re-open cancel, emit scientific-acceptance, open naruon or LineageWeave, add
GET to `NaruonLiveService`, or treat CLI success as an ADR 0014 claim.

## Related authority

ADR 0085, ADR 0071, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
