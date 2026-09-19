# ADR 0098 — Loopback interpretation-run lookup stored-request CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0097. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0097 occupied
including #469=0097, #468=0096.
**Figma File ID:** N/A — this increment changes a Rust CLI binary and has no
user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0097 publishes
`GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}/request`.
Operators still had no published binary that mints that GET onto spawned
`tepp-orchestrator-loopback` TCP. Reusing `tepp-interpretation-run-lookup`
(#468) or `tepp-interpretation-run-request` (#454) would collide with identity
lookup and client-key stored-request. Temporal-context stored-request is
already #464. Cancel lineages stay closed.

## Decision

Publish `tepp-interpretation-run-lookup-request get` which mints
`contextual_orchestrator_interpretation_run_lookup_stored_request_exchange`
onto spawned loopback TCP. Empty stdin is admitted. Nonempty leftover stdin,
public bind, `localhost`, `http` origin, unpublished consumer, credential
flags, reserved `by-run-id` as an identity, slash/NUL, and metric keys fail
closed. Stdout is the stored metric-free create.
`scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
appears. Naruon and LineageWeave are refused. `NaruonLiveService` stays
POST-only.

## Non-goals

- Production TLS, public bind, or durable interpretation-run storage.
- Leiden community detection, Driver p.16 std-family restoration, or
  Figma/export work (GAP-010).
- Promoting an ADR 0014 scientific claim from CLI success.
- Duplicating lookup stored-request GET (#469), lookup GET/CLI (#467/#468),
  stored-request GET/CLI (#453/#454), GET-by-id (#438), retrieval CLI (#439),
  collection GET/CLI (#433/#436), create CLI (#425), export lookup (#466),
  analysis-run lookup (#380/#401), or cancel lineages (closed).
- Adding GET to `NaruonLiveService`. Opening naruon or LineageWeave on this
  orchestrator-owned adapter.

## Alternatives considered

1. Re-open cancel CLI — rejected.
2. Reuse `tepp-interpretation-run-lookup` or `tepp-interpretation-run-request`
   — rejected; those are ADR 0096 and ADR 0086.
3. Temporal-context stored-request GET — rejected; already #464.
4. Dedicated lookup stored-request binary — accepted.

## Consequences

CLI success is not measurement evidence and is not an ADR 0014 claim.
Operators who hold a 202 receipt or log `orch-run-N` can recover the stored
create without a second hop.

## Failure and recovery

Non-orchestrator consumers, nonempty leftover stdin, extra segments, slash/NUL,
reserved prefix, missing ids, 0 or >1 match, and metric keys fail closed.

## Verification

- `tepp-interpretation-run-lookup-request get` of an accepted `orch-run-N`
  prints the matching stored create without RMSE/`tepp.scientific_acceptance.v1`;
- naruon, LineageWeave, public bind, `localhost`, `http` origin, leftover
  stdin, reserved prefix, and missing ids fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes the published binary; lookup stored-request GET remains valid.
A superseding ADR is required to persist the registry, bind a public address,
emit scientific-acceptance, open naruon or LineageWeave, add GET to
`NaruonLiveService`, re-open cancel lineages, or treat CLI success as an
ADR 0014 claim.

## Related authority

ADR 0097, ADR 0096, ADR 0085, ADR 0014, RFC 9110 (Fielding, Nottingham, &
Reschke, 2022).
