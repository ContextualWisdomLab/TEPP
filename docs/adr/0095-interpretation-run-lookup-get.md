# ADR 0095 — Loopback interpretation-run lookup GET by server-assigned id

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0071 and ADR 0018 for the operator-visible
jump from a server-assigned `interpretation_run_id` to the metric-free identity.
Does not supersede ADR 0014. Unique versus protected main; 0026–0094 occupied
including #466=0093+0094, #464=0092, #454=0086, #453=0085.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no
user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0071 publishes `GET /v1/interpretation-runs/{idempotency_key}`. Collection
GET is a different stack. Stored-request GET requires the client key. Operators
who hold a 202 acceptance receipt or a log `orch-run-N` therefore cannot jump
to that identity without scanning pages. Returning RMSE, evidence spans, or
`tepp.scientific_acceptance.v1` on the lookup body would treat identity
resolution as measurement evidence. Export lookup GET (#466) is naruon-owned
and resolves the inverse dual identity (key → `export_id`). Reuse of GET-by-id
with the run id as `{idempotency_key}` would collide with client-key retrieval.

## Decision

`OrchestratorLiveService` serves
`GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}` on loopback:

- The payload is the metric-free collection identity: `interpretation_run_id`,
  `idempotency_key`, `orchestration_mode`, `claim_status=hypothetical`,
  `scientific_authority=false`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, evidence
  spans, and causal-score keys never appear.
- Lookup is consumer-scoped to contextual-orchestrator. Zero matches and more
  than one match fail closed (no tenant oracle). Naruon and LineageWeave are
  refused.
- Empty GET bodies only. Query strings, GET-by-id, POST `/by-run-id`, GET
  `/request`, collection GET `/v1/interpretation-runs`, reserved `by-run-id`
  as an identity, slash/NUL, and nonempty bodies fail closed.
- Dispatch order: collection → stored-request extra-segment → lookup by-run-id
  → GET-by-id. GET-by-id refuses the reserved prefix as an idempotency key.
- `NaruonLiveService` stays POST-only. Unknown ids fail closed. Persistence
  remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable interpretation-run storage.
- Leiden community detection, Driver p.16 std-family restoration, or
  Figma/export work (GAP-010).
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating GET `/v1/interpretation-runs/{idempotency_key}` (#438), retrieval
  CLI (#439), collection GET/CLI (#433/#436), stored-request GET/CLI
  (#453/#454), create CLI (#425), export lookup (#466), analysis-run lookup
  GET (#380), or cancel lineages (closed).
- Adding GET to `NaruonLiveService`. Opening naruon or LineageWeave on this
  orchestrator-owned adapter.

## Alternatives considered

1. **Ask operators to scan collection pages or re-POST create** — rejected
   because collection GET is a different stack and a 202 receipt is not an
   addressable GET-by-id identity.
2. **Return `tepp.scientific_acceptance.v1` on succeeded lookup** — rejected
   because lookup bodies must stay metric-free.
3. **Reuse GET-by-id with the run id as `{idempotency_key}`** — rejected
   because GET-by-id (#438) owns client-key retrieval.
4. **Metric-free interpretation-run lookup GET on loopback** — accepted.

## Consequences

- Operators can resolve a 202 acceptance receipt or log `orch-run-N` to the
  metric-free identity without scanning pages.
- Lookup pages cannot be mistaken for a succeeded scientific-acceptance result.
- GET-by-id remains the client-key retrieval route.

## Failure and recovery

Unknown ids, extra path segments, GET-by-id, query strings, nonempty bodies,
POST `/by-run-id`, metric keys, naruon, LineageWeave, unpublished consumers,
consumer mismatch, ambiguous multi-match, reserved prefix-as-id, slash/NUL,
and non-loopback hosts return a redacted `400` envelope. Oversized ids return
`413`. Credential headers remain `403`. The in-memory registry is not durable;
a restart requires re-POSTing the original metric-free create. Callers must not
fabricate a succeeded scientific-acceptance artifact from a lookup payload.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Run-id lookup remains loopback-only, size-bounded, consumer-scoped, and
  content-redacting.
- HTTP `200` on a lookup payload is not measurement evidence and is not
  release evidence.
- Ambiguous matches fail closed so lookup cannot become a tenant-count oracle.

## Compatibility and migration

Create POST, collection GET, GET-by-id, and stored-request GET paths are
unchanged. GET-by-id remains the client-key route. Production adapters may
replace loopback while preserving metric-free lookup fields and the artifact
refusal.

## Verification

Falsifiable evidence:

- GET lookup JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/
  evidence-span keys and keeps `claim_status=hypothetical` with
  `scientific_authority=false`;
- GET of an accepted `orch-run-N` returns the matching `idempotency_key`;
- GET does not leak another consumer's run;
- GET-by-id, query strings, nonempty bodies, POST `/by-run-id`, unknown ids,
  naruon, `NaruonLiveService` GET, reserved `by-run-id` as an identity, and
  slash/NUL fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes lookup GET dispatch; POST, collection GET, GET-by-id, and
stored-request GET remain valid. A superseding ADR is required to persist the
registry, bind a public address, emit scientific-acceptance on lookup, open
naruon or LineageWeave on this orchestrator-owned adapter, add GET to
`NaruonLiveService`, re-open cancel lineages, or treat HTTP success as an
ADR 0014 claim.

## Related authority

ADR 0071, ADR 0069, ADR 0018, ADR 0010, ADR 0011, ADR 0014, RFC 9110 (Fielding,
Nottingham, & Reschke, 2022).
