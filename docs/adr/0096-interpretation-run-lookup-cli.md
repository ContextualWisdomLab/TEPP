# ADR 0096 — Loopback interpretation-run lookup CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0095. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0095 occupied
including #467=0095, #466=0093+0094.
**Figma File ID:** N/A — this increment changes a Rust CLI binary and has no
user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0095 publishes `GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}`.
Operators still had no published binary that mints that GET onto spawned
`tepp-orchestrator-loopback` TCP. Reusing `tepp-interpretation-run-request`
(#454) or `tepp-interpretation-runs` (#425) would collide with stored-request
GET and create. Export lookup CLI (#466) is naruon-owned. Analysis-run lookup
CLI (#401) is a different stack. Project-history by-idempotency lookup would
duplicate GET-by-id (already keyed by `idempotency_key`). Cancel lineages stay
closed.

## Decision

Publish `tepp-interpretation-run-lookup lookup` which mints
`contextual_orchestrator_interpretation_run_lookup_exchange` onto spawned
loopback TCP. Empty stdin is admitted. Nonempty leftover stdin, public bind,
`localhost`, `http` origin, unpublished consumer, credential flags, reserved
`by-run-id` as an identity, slash/NUL, and metric keys fail closed. Stdout is
the metric-free identity. `claim_status` remains hypothetical.
`scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
appears. Naruon and LineageWeave are refused. `NaruonLiveService` stays
POST-only.

## Non-goals

- Production TLS, public bind, or durable interpretation-run storage.
- Leiden community detection, Driver p.16 std-family restoration, or
  Figma/export work (GAP-010).
- Promoting an ADR 0014 scientific claim from CLI success.
- Duplicating lookup GET (#467), GET-by-id (#438), retrieval CLI (#439),
  collection GET/CLI (#433/#436), stored-request GET/CLI (#453/#454), create
  CLI (#425), export lookup (#466), analysis-run lookup GET/CLI (#380/#401),
  or cancel lineages (closed).
- Adding GET to `NaruonLiveService`. Opening naruon or LineageWeave on this
  orchestrator-owned adapter.

## Alternatives considered

1. Re-open cancel CLI — rejected.
2. Reuse `tepp-interpretation-run-request` or `tepp-interpretation-runs` —
   rejected; those are ADR 0086 and ADR 0064.
3. Project-history by-idempotency lookup — rejected; GET-by-id already keys
   by `idempotency_key`.
4. Dedicated lookup binary — accepted.

## Consequences

CLI success is not measurement evidence and is not an ADR 0014 claim.
Operators who hold a 202 receipt or log `orch-run-N` can resolve identity
without scanning collection pages.

## Failure and recovery

Non-orchestrator consumers, nonempty leftover stdin, extra segments, slash/NUL,
reserved prefix, missing ids, 0 or >1 match, and metric keys fail closed.

## Verification

- `tepp-interpretation-run-lookup lookup` of an accepted `orch-run-N` prints
  the matching `idempotency_key` without RMSE/`tepp.scientific_acceptance.v1`;
- naruon, LineageWeave, public bind, `localhost`, `http` origin, leftover
  stdin, reserved prefix, and missing ids fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes the published binary; lookup GET remains valid. A superseding
ADR is required to persist the registry, bind a public address, re-open cancel,
emit scientific-acceptance, open naruon or LineageWeave, add GET to
`NaruonLiveService`, or treat CLI success as an ADR 0014 claim.

## Related authority

ADR 0095, ADR 0071, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
