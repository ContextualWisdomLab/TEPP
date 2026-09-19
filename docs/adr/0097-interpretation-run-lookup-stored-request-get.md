# ADR 0097 — Loopback interpretation-run lookup stored-request GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0095 and ADR 0085. Does not re-open
cancel lineages. Does not supersede ADR 0014. Unique versus protected main;
0026–0096 occupied including #468=0096, #467=0095.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no
user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0095 publishes `GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}`
as the metric-free identity. ADR 0085 publishes
`GET /v1/interpretation-runs/{idempotency_key}/request` as the stored create.
Operators who hold a 202 receipt or log `orch-run-N` still need two hops
(lookup identity, then stored-request by client key) to recover the create.
Reuse of `{idempotency_key}/request` with the run id as the key would collide
with client-key stored-request. Cancel extra-segment stays refused.

## Decision

`OrchestratorLiveService` serves
`GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}/request` on
loopback:

- The payload is the stored metric-free create request.
  `scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
  appears.
- Lookup stored-request is consumer-scoped to contextual-orchestrator. Zero
  matches and more than one match fail closed (no tenant oracle). Naruon and
  LineageWeave are refused.
- Empty GET bodies only. Query strings, lookup without `/request`,
  `{key}/request`, GET-by-id, POST `/by-run-id/.../request`, collection GET,
  reserved `by-run-id` as an identity, slash/NUL, cancel extra-segment, and
  nonempty bodies fail closed.
- Dispatch order: collection → stored-request `{key}/request` → lookup
  stored-request `by-run-id/{id}/request` → lookup by-run-id → GET-by-id.
  Stored-request `{key}/request` refuses reserved prefix `by-run-id` as a key.
- `NaruonLiveService` stays POST-only. Unknown ids fail closed. Persistence
  remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable interpretation-run storage.
- Leiden community detection, Driver p.16 std-family restoration, or
  Figma/export work (GAP-010).
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating lookup GET/CLI (#467/#468), stored-request GET/CLI (#453/#454),
  GET-by-id (#438), retrieval CLI (#439), collection GET/CLI (#433/#436),
  create CLI (#425), export lookup (#466), analysis-run lookup (#380/#401),
  or cancel lineages (closed).
- Adding GET to `NaruonLiveService`. Opening naruon or LineageWeave on this
  orchestrator-owned adapter.

## Alternatives considered

1. Ask operators to hop lookup then `{key}/request` — rejected because a 202
   receipt is already an addressable server-assigned identity.
2. Reuse `{idempotency_key}/request` with the run id as the key — rejected
   because ADR 0085 owns client-key stored-request.
3. Return identity JSON on `/request` — rejected because that is ADR 0095.
4. Metric-free lookup stored-request GET on loopback — accepted.

## Consequences

Operators can recover the stored create from `orch-run-N` without a second hop
and without scanning collection pages. HTTP 200 is not measurement evidence.

## Failure and recovery

Unknown ids, extra path segments, lookup without `/request`, `{key}/request`,
query strings, nonempty bodies, POST, metric keys, naruon, LineageWeave,
unpublished consumers, consumer mismatch, ambiguous multi-match, reserved
prefix-as-id, slash/NUL, cancel extra-segment, and non-loopback hosts return a
redacted `400` envelope. Oversized ids return `413`. Credential headers remain
`403`.

## Verification

- GET lookup stored-request JSON has no RMSE/scientific-acceptance keys and
  keeps `scientific_authority=false`;
- GET of an accepted `orch-run-N` returns the matching stored create
  `idempotency_key`;
- `{key}/request`, lookup without `/request`, naruon, cancel extra-segment,
  reserved prefix, slash/NUL, and unknown ids fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes lookup stored-request dispatch; lookup GET, `{key}/request`,
GET-by-id, collection GET, and POST remain valid. A superseding ADR is required
to persist the registry, bind a public address, emit scientific-acceptance,
open naruon or LineageWeave, add GET to `NaruonLiveService`, re-open cancel
lineages, or treat HTTP success as an ADR 0014 claim.

## Related authority

ADR 0095, ADR 0085, ADR 0071, ADR 0014, RFC 9110 (Fielding, Nottingham, &
Reschke, 2022).
