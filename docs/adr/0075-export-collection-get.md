# ADR 0075 — Loopback export collection GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0054 for enumerating authorized export identities. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique versus protected main; live vs-main and sibling GAP-003A PRs already occupy 0026–0074.

## Context

ADR 0054 retrieves one authorized export by `export_id`. Operators who hold a
200 authorization receipt still had no loopback path to enumerate minted
identities without guessing UUIDs. Duplicating export retrieval GET (#411),
export-retrieval CLI (#417), export-authorize CLI (#410), interpretation-run
collection GET (#433), project-history collection GET (#424), Leiden, Driver
p.16, or GAP-010 Figma/export would collide with live PRs. LineageWeave is
refused on this naruon-owned adapter; `NaruonLiveService` stays POST-only.

## Decision

`AnalysisRunLiveService` publishes loopback-only `GET /v1/exports` on
`tepp-loopback`:

- Consumer is `naruon` only. Empty body. Identity does not travel in a header.
  `idempotency-key` is refused.
- Extra path segments fail closed as GET-by-id parsing, not as collection.
- Pagination uses `tepp-page-limit` (default 32, max 64) and exclusive
  `tepp-page-cursor` on `export_id`.
- Each row is the same metric-free `ExportRetrieval` identity as ADR 0054:
  `export_id`, `artifact_id`, `decision_code=purpose_bound_export_allowed`,
  `purpose`, `idempotency_key`. Tenant, principal, source text, RMSE, bias,
  coverage, SE-gate, and `tepp.scientific_acceptance.v1` never appear.
- Collection does not infer causality, persist, or return a completed
  psychometric result.
- This slice does not implement a collection CLI.

## Alternatives considered

1. **Keep GET-by-id without a collection** — rejected; operators still guess
   UUID v7 identities after ADR 0054.
2. **Reuse interpretation-run collection GET (#433)** — rejected; that is a
   different live resource and a contextual-orchestrator consumer.
3. **Add GET collection to `NaruonLiveService`** — rejected; that listener
   stays POST-only.
4. **Loopback `GET /v1/exports`** — accepted.

## Consequences

- Operators can enumerate authorized export identities without guessing
  `export_id`.
- Collection JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Collection success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`naruon` consumers, nonempty GET bodies, present `idempotency-key`, extra
path segments, slash/NUL cursors, credential flags, and metric keys fail
closed. The in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Tenant, principal, and source text stay off the collection page.
- HTTP 200 on collection is not measurement evidence and is not a causal
  claim.

## Compatibility and migration

GET-by-id, POST `/v1/exports` on `AnalysisRunLiveService`, and
`NaruonLiveService` POST-only remain unchanged. A collection CLI remains a
later slice. Persistence remains GAP-003B.

## Verification

Falsifiable evidence:

- GET collection of authorized exports returns metric-free identities without
  RMSE/bias/coverage/SE-gate/tenant/principal/source-text/
  `tepp.scientific_acceptance.v1` keys;
- LineageWeave, nonempty body, present `idempotency-key`, extra segments, and
  unknown keys fail closed;
- `NaruonLiveService` still refuses GET;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes collection GET; GET-by-id and POST remain valid. A
superseding ADR is required to persist the collection, bind a public address,
emit scientific-acceptance on collection, open LineageWeave, add GET to
`NaruonLiveService`, or treat collection success as an ADR 0014 claim.

## Related authority

- ADR 0054 owns loopback export retrieval GET.
- ADR 0055 owns the export-retrieval CLI (live #417).
- ADR 0026 owns the export-authorize CLI (live #410).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
