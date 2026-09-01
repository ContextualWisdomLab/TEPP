# ADR 0099 — Loopback export idempotency-key lookup stored-request GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0093 and ADR 0089. Does not re-open
cancel lineages. Does not supersede ADR 0014. Unique versus protected main;
0026–0098 occupied including #470=0098, #469=0097, #466=0093+0094.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no
user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0093 publishes `GET /v1/exports/by-idempotency/{idempotency_key}` as the
metric-free identity. Stored-request GET (`GET /v1/exports/{export_id}/request`)
is the client-id extra-segment on a parallel stack. Operators who hold a 200
authorization receipt or log key still need two hops (lookup identity, then
stored-request by `export_id`) to recover the create. Reuse of
`{export_id}/request` with the idempotency key as the id would collide with
server-id stored-request. Cancel extra-segment stays refused.

## Decision

`AnalysisRunLiveService` serves
`GET /v1/exports/by-idempotency/{idempotency_key}/request` on loopback:

- The payload is the stored naruon export-authorization request.
  `tepp.scientific_acceptance.v1` never appears.
- Lookup stored-request is consumer-scoped to naruon. Zero matches and more
  than one match fail closed (no tenant oracle). LineageWeave is refused.
- Empty GET bodies only. Query strings, lookup without `/request`,
  `{export_id}/request`, GET-by-id, POST `/by-idempotency/.../request`,
  collection GET, reserved `by-idempotency` as a key, slash/NUL, cancel
  extra-segment, and nonempty bodies fail closed.
- Dispatch order: lookup stored-request `by-idempotency/{key}/request` →
  lookup by-idempotency → GET-by-id.
- `NaruonLiveService` stays POST-only. Unknown keys fail closed. Persistence
  remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable export storage.
- Leiden community detection, Driver p.16 std-family restoration, or
  Figma/export work (GAP-010).
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating lookup GET/CLI (#465/#466), stored-request GET/CLI (#457/#459),
  GET-by-id (#411), retrieval CLI (#417), collection GET/CLI (#443/#444),
  export-authorize CLI (#410), analysis-run lookup (#380), or cancel lineages
  (closed).
- Adding GET to `NaruonLiveService`. Opening LineageWeave on this naruon-owned
  adapter.

## Alternatives considered

1. Ask operators to hop lookup then `{export_id}/request` — rejected because a
   200 receipt already carries the client key.
2. Reuse `{export_id}/request` with the key as the id — rejected because ADR
   0089 owns server-id stored-request.
3. Return identity JSON on `/request` — rejected because that is ADR 0093.
4. Metric-free lookup stored-request GET on loopback — accepted.

## Consequences

Operators can recover the stored create from an authorization key without a
second hop. HTTP 200 is not measurement evidence.

## Failure and recovery

Unknown keys, extra path segments, lookup without `/request`, `{export_id}/request`,
query strings, nonempty bodies, POST, metric keys, LineageWeave, unpublished
consumers, consumer mismatch, ambiguous multi-match, reserved prefix-as-key,
slash/NUL, cancel extra-segment, and non-loopback hosts return a redacted `400`
envelope. Oversized keys return `413`. Credential headers remain `403`.

## Verification

- GET lookup stored-request JSON has no RMSE/scientific-acceptance keys;
- GET of an authorized key returns the matching stored create `artifact_id`;
- `{export_id}/request`, lookup without `/request`, LineageWeave, cancel
  extra-segment, reserved prefix, and unknown keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes lookup stored-request dispatch; lookup GET, GET-by-id, and
POST remain valid. A superseding ADR is required to persist the registry, bind
a public address, emit scientific-acceptance, open LineageWeave, add GET to
`NaruonLiveService`, re-open cancel lineages, or treat HTTP success as an
ADR 0014 claim.

## Related authority

ADR 0093, ADR 0089, ADR 0054, ADR 0014, RFC 9110 (Fielding, Nottingham, &
Reschke, 2022).
