# ADR 0093 — Loopback export idempotency-key lookup GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0054 and ADR 0018 for the operator-visible
jump from an export idempotency key to a durable export identity. Does not
supersede ADR 0014. Unique versus protected main; 0026–0092 occupied including
#464=0092, #463=0091, #459=0090, #457=0089, #411=0054.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no
user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0054 publishes `GET /v1/exports/{export_id}`. Collection GET is a different
stack. Stored-request GET requires an `export_id`. Operators who hold a 200
authorization receipt or a log key therefore cannot jump to that export without
scanning identities. Returning RMSE, bias, coverage, SE-gate, source text, or
`tepp.scientific_acceptance.v1` on the lookup body would treat key resolution as
measurement evidence. Analysis-run lookup GET (#380) is a different adapter.
Reuse of GET-by-id with the key as `{export_id}` would collide with
server-assigned UUID v7 capabilities.

## Decision

`AnalysisRunLiveService` serves `GET /v1/exports/by-idempotency/{idempotency_key}`
on loopback:

- The payload is metric-free: `export_id`, `decision_code`, `idempotency_key`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report,
  `terminal_result`, `tenant_workspace_id`, `principal_id`, and
  `includes_source_text` never appear.
- Lookup is consumer-scoped to naruon. Zero matches and more than one match
  fail closed (no tenant oracle). LineageWeave is refused.
- Empty GET bodies only. Query strings, GET-by-id, POST `/by-idempotency`,
  GET `/request`, collection GET `/v1/exports`, reserved `by-idempotency` as a
  key, and nonempty bodies fail closed.
- The key travels in the path. The NARUON exchange does not send an
  `idempotency-key` header or credentials.
- `NaruonLiveService` stays POST-only. Unknown keys fail closed. Persistence
  remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable export storage.
- Leiden community detection, Driver p.16 std-family restoration, or
  Figma/export work (GAP-010).
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating GET `/v1/exports/{export_id}` (#411), retrieval CLI (#417),
  collection GET/CLI (#443/#444), stored-request GET/CLI (#457/#459),
  export-authorize CLI (#410), analysis-run lookup GET (#380), or cancel
  lineages (closed).
- Adding GET to `NaruonLiveService`.

## Alternatives considered

1. **Ask operators to scan collection pages or re-POST authorization** —
   rejected because collection GET is a different stack and a 200 decision is
   not an addressable identity.
2. **Return `tepp.scientific_acceptance.v1` on succeeded lookup** — rejected
   because lookup bodies must stay metric-free.
3. **Reuse GET-by-id with the key as `{export_id}`** — rejected because
   GET-by-id (#411) owns UUID v7 capabilities.
4. **Metric-free export idempotency-key lookup GET on loopback** — accepted.

## Consequences

- Operators can resolve a 200 export authorization receipt or log key to a
  durable `export_id` without scanning identities.
- Lookup pages cannot be mistaken for a succeeded scientific-acceptance result.
- GET-by-id remains the capability-bearing retrieval route.

## Failure and recovery

Unknown keys, extra path segments, GET-by-id, query strings, nonempty bodies,
POST `/by-idempotency`, metric keys, LineageWeave, unpublished consumers,
consumer mismatch, ambiguous multi-tenant matches, reserved prefix-as-key, and
non-loopback hosts return a redacted `400` envelope. Oversized keys return
`413`. Credential headers remain `403`. The in-memory registry is not durable;
a restart requires re-POSTing the original metric-free authorization. Callers
must not fabricate a succeeded scientific-acceptance artifact from a lookup
payload.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Idempotency-key lookup remains loopback-only, size-bounded, consumer-scoped,
  and content-redacting.
- HTTP `200` on a lookup payload is not measurement evidence and is not
  release evidence.
- Ambiguous matches fail closed so lookup cannot become a tenant-count oracle.

## Compatibility and migration

Create POST, retrieval GET, temporal-context, and project-history paths are
unchanged. GET-by-id remains the capability route. Production adapters may
replace loopback while preserving metric-free lookup fields and the artifact
refusal.

## Verification

Falsifiable evidence:

- GET lookup JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/
  `terminal_result`/`tenant_workspace_id`/`principal_id`/`includes_source_text`
  keys;
- GET of a create key returns the matching `export_id`;
- GET does not leak another consumer's export;
- GET-by-id, query strings, nonempty bodies, POST `/by-idempotency`, unknown
  keys, LineageWeave, `NaruonLiveService` GET, and reserved `by-idempotency` as
  a key fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes idempotency-lookup GET dispatch; POST authorize receipts and
retrieval GET remain valid. A superseding ADR is required to persist the
registry, bind a public address, emit scientific-acceptance on lookup, open
LineageWeave on this naruon-owned adapter, add GET to `NaruonLiveService`, or
treat HTTP success as an ADR 0014 claim.

## Related authority

ADR 0054, ADR 0018, ADR 0009, ADR 0011, ADR 0014, RFC 9110 (Fielding,
Nottingham, & Reschke, 2022).
