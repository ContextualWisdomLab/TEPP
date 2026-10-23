# ADR 0083 — Loopback temporal-context GET-by-id

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements `POST /v1/temporal-context`. Does not
re-open collection GET (#449 closed as fold-into-landing-vehicle) or cancel
lineages closed as unsafe mutation. Does not supersede ADR 0014. Unique versus
protected main; 0026–0082 were assigned on live or closed sibling GAP-003A
PRs.

## Context

`POST /v1/temporal-context` returns one cutoff-safe association page. Operators
who hold an idempotency key still had no loopback GET-by-id. Collection GET
(#449) was closed as a standalone list route. Cancel HTTP/CLI lineages were
closed as unauthenticated destructive operations. Duplicating temporal-context
CLI (#414), project-history GET-by-id (#429), interpretation-run GET-by-id
(#438), export retrieval GET (#411), Leiden, or GAP-010 Figma/export would
collide with live PRs. Naruon is refused; `NaruonLiveService` stays POST-only.

## Decision

Publish `GET /v1/temporal-context/{idempotency_key}` on
`AnalysisRunLiveService` / `tepp-loopback`:

- Extra-segment path parse. Slash/NUL/control identities fail closed.
- Empty body. Present `idempotency-key` header fails closed (identity is in
  the path).
- Collection path `GET /v1/temporal-context` stays refused.
- Retrieval JSON is a metric-free identity with
  `inference_status=temporal_association_only`. Event labels, actor lists,
  timeline events, evidence text, findings, RMSE, and
  `tepp.scientific_acceptance.v1` never appear.
- POST remains compute-and-return. An optional `idempotency-key` header mints
  the identity for later GET-by-id.

## Alternatives considered

1. **Re-open collection GET (#449)** — rejected; closed as
   fold-into-landing-vehicle.
2. **Re-open cancel HTTP** — rejected; closed as unsafe mutation.
3. **Loopback GET-by-id** — accepted.

## Consequences

- Operators can retrieve one accepted identity without POST replay.
- HTTP 200 is not measurement evidence and is not a causal claim.

## Failure and recovery

Non-LineageWeave consumers, nonempty GET bodies, collection path, extra
segments, slash/NUL identities, missing keys, credential flags, and metric
keys fail closed.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers. Event labels and actor lists stay off the retrieval.
- HTTP 200 is not an ADR 0014 claim.

## Compatibility and migration

POST without an idempotency header remains valid. Collection GET stays closed.
`NaruonLiveService` POST-only remains unchanged. Persistence remains GAP-003B.

## Verification

- `GET /v1/temporal-context/{idempotency_key}` of an accepted identity returns
  a metric-free row without RMSE/event-label/actor/`tepp.scientific_acceptance.v1`;
- naruon, collection path, extra segments, slash/NUL, nonempty body, and
  missing keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes GET-by-id; POST remains valid. A superseding ADR is required
to persist the registry, bind a public address, re-open collection GET as a
standalone route, emit scientific-acceptance, open naruon, add GET to
`NaruonLiveService`, or treat retrieval success as an ADR 0014 claim.

## Related authority

- ADR 0002 owns six-clock temporal semantics.
- ADR 0027 owns the temporal-context CLI (live #414).
- ADR 0066 owns project-history GET-by-id (live #429).
- ADR 0071 owns interpretation-run GET-by-id (live #438).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022).
