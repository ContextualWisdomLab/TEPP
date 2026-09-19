# ADR 0093 — Loopback export idempotency-key lookup GET

**Decision status:** Accepted  
**Implementation maturity:** active-PR  
**Date:** 2026-09-01  
**Supersedes:** None; complements ADR 0054 and ADR 0018 for the operator-visible jump from an export idempotency key to a durable export identity. Does not supersede ADR 0014.  
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.  
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0054 publishes `GET /v1/exports/{export_id}`. Operators who hold a 200 authorization receipt or log key still need a metric-free way to resolve the server-assigned export identity without scanning a collection. Reusing GET-by-id with the key as `{export_id}` would collide with server-assigned UUID v7 capability identity.

Export authorization already accepts opaque idempotency keys. Review found that the first lookup adapter imposed narrower client/path rules: the CLI rejected slash-containing keys and the HTTP/DTO/CLI rejected the literal key `by-idempotency`. Those restrictions made valid authorization receipts unresolvable. The lookup contract therefore has to preserve accepted opaque key identity rather than retrospectively inventing a smaller key domain.

## Decision

`AnalysisRunLiveService` serves `GET /v1/exports/by-idempotency/{idempotency_key}` on loopback:

- The payload is metric-free: `export_id`, `decision_code`, `idempotency_key`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report, `terminal_result`, `tenant_workspace_id`, `principal_id`, and `includes_source_text` never appear.
- Lookup is consumer-scoped to naruon. Zero matches and more than one match fail closed without disclosing tenant counts. LineageWeave is refused.
- Empty GET bodies only. Query strings, GET-by-id, POST `/by-idempotency`, stored-request suffixes, collection GET, and nonempty bodies fail closed.
- Client idempotency keys are opaque accepted request data. A `/` inside a key is percent-encoded into one path segment and decoded after route segmentation. The literal value `by-idempotency` remains addressable at `/v1/exports/by-idempotency/by-idempotency`; the first occurrence is routing syntax and the second is data.
- Raw extra path segments are never treated as part of a key. NUL and oversized keys fail closed.
- The Naruon exchange does not send an `idempotency-key` header or credentials.
- `NaruonLiveService` stays POST-only. Persistence remains GAP-003B.

The separate stored-request-by-idempotency convenience route is governed by ADR 0099 and is currently quarantined; success of the metric-free identity lookup does not authorize disclosure of the original request.

## Non-goals

- Production TLS, public bind, or durable export storage.
- Treating an idempotency key as authorization to retrieve a stored create request.
- Leiden, longitudinal-model repair, or GAP-010 UI/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Adding GET to `NaruonLiveService`.

## Alternatives considered

1. Ask operators to scan collection pages or re-POST authorization — rejected because a valid receipt should remain addressable without changing request identity.
2. Restrict new lookup clients to a narrower key grammar than authorization — rejected because it strands already-valid receipts.
3. Reuse GET-by-id with the client key as `{export_id}` — rejected because GET-by-id owns server-assigned export capabilities.
4. Preserve opaque accepted key identity with one-segment percent encoding — accepted.

## Consequences

- A valid authorization key remains lookup-addressable even when it contains `/` or equals `by-idempotency`.
- Route parsing remains unambiguous because segmentation occurs before percent decoding and raw additional `/` segments are rejected.
- Lookup payloads cannot be mistaken for scientific results or stored authorization requests.

## Failure and recovery

Unknown keys, extra raw path segments, GET-by-id, query strings, nonempty bodies, POST `/by-idempotency`, metric keys, LineageWeave, unpublished consumers, consumer mismatch, ambiguous multi-tenant matches, NUL, and non-loopback hosts return a redacted failure. Oversized keys return the bounded limit failure. Credential headers remain forbidden. The in-memory registry is not durable; a restart requires reconstruction through the authorized create path.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross this consumer boundary.
- The response exposes no tenant/principal/source-text or scientific fields.
- Ambiguous matches fail closed so the lookup is not a tenant-count oracle.
- An idempotency key identifies a replay domain; it is not a bearer credential for the ADR 0099 stored-request resource.

## Compatibility and migration

Create POST, retrieval GET, temporal-context, and project-history paths are unchanged. Existing accepted slash-containing or prefix-looking keys need no migration: lookup preserves their exact decoded identity. Production adapters may replace loopback only while retaining this opaque-key and metric-free contract.

## Verification

Falsifiable evidence includes:

- GET lookup JSON has no scientific, tenant, principal, source-text, report, or terminal-result fields;
- POST with `scope/key` followed by lookup through `scope%2Fkey` returns the same opaque key and matching `export_id`;
- POST with the literal key `by-idempotency` remains resolvable through the nested lookup path;
- CLI and HTTP builders admit the same accepted key domain;
- raw extra segments, NUL, oversized keys, unknown keys, LineageWeave and forbidden credentials fail closed;
- exact-head Clippy, `tepp_api` tests, rustdoc, line/branch coverage, security workflows and qualifying review remain required.

## Rollback and supersession

Rollback removes idempotency-lookup GET dispatch; POST authorization receipts and retrieval GET remain valid. A superseding ADR is required to change accepted idempotency-key identity, persist the registry, expose a public address, open LineageWeave, add GET to `NaruonLiveService`, or promote HTTP success to scientific authority.

## Related authority

ADR 0054, ADR 0018, ADR 0009, ADR 0011, ADR 0014, ADR 0099, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
