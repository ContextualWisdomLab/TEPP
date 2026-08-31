# ADR 0031 — Analysis-run collection GET path

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0018 and ADR 0029 for the operator-visible collection read. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0030 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel, and loopback-CLI slices.

## Context

Protected main and the live cancel slice accept analysis runs on loopback but refuse `GET /v1/analysis-runs`. Operators therefore cannot enumerate accepted, running, cancelled, or terminal runs without guessing run identities. Returning RMSE, bias, coverage, SE-gate, or `tepp.scientific_acceptance.v1` on the list would treat enumeration as measurement evidence. Only a succeeded single-run GET with output profile `scientific_acceptance_v1` may return that artifact (#359). Stacking this slice onto GET-by-id, lifecycle POST, or CLI would duplicate those heads.

## Decision

`AnalysisRunLiveService` serves `GET /v1/analysis-runs` on loopback:

- The collection lists the calling consumer's runs sorted by opaque `run_id`.
- Each row is metric-free: `run_id`, `run_state`, and `idempotency_key` only.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report, and `terminal_result` keys never appear on the list.
- Accepted, running, cancelled, succeeded, and failed states are listed. Succeeded rows still omit the artifact.
- Bounded cursor pagination uses `tepp-page-cursor` and `tepp-page-limit` headers because the shared request-line parser fails closed on query strings. Default limit is 32; maximum is 64. An unknown cursor fails closed.
- Empty collections return `200` with `runs: []`. GET-by-id, query strings, and nonempty GET bodies fail closed.
- Cancel POST, create POST, and consumer isolation are unchanged. Persistence remains GAP-003B.

## Alternatives considered

1. **Stack collection GET onto the live GET-by-id PR** — rejected because that head already owns single-run status and a parallel stack would duplicate it.
2. **Return `tepp.scientific_acceptance.v1` on succeeded collection rows** — rejected because collection bodies must stay metric-free; only a succeeded single-run GET with profile `scientific_acceptance_v1` may return the artifact.
3. **Query-string pagination** — rejected because `parse_request_line` fails closed on `?` to refuse hostile URLs.
4. **Header-paginated metric-free collection GET on loopback** — accepted.

## Consequences

- Operators can enumerate runs on the same loopback listener that created them.
- Collection pages cannot be mistaken for a succeeded scientific-acceptance result.
- GET-by-id may later return a digest-bound artifact without changing these collection gates.

## Failure and recovery

Unknown collection paths, GET-by-id, query strings, nonempty bodies, unknown cursors, zero or non-integer limits, metric keys, unpublished consumers, and non-loopback hosts return a redacted `400` envelope. Oversized page limits and cursors return `413`. Credential headers remain `403`. The in-memory registry is not durable; a restart requires re-POSTing the original metric-free create requests. Callers must not fabricate a succeeded scientific-acceptance artifact from a collection row.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Collection remains loopback-only, size-bounded, consumer-scoped, and content-redacting.
- HTTP `200` on a collection page is not measurement evidence and is not release evidence.

## Compatibility and migration

Create POST, cancel POST, temporal-context, and project-history paths are unchanged. GET-by-id remains refused on this slice. Production adapters may replace loopback while preserving metric-free collection rows and the artifact refusal.

## Verification

Falsifiable evidence:

- GET collection JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/`terminal_result` keys;
- GET lists accepted, running, cancelled, succeeded, and failed rows for one consumer;
- GET does not leak another consumer's runs;
- unknown cursor, GET-by-id, query strings, and nonempty bodies fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes collection GET dispatch; POST create receipts and cancel remain valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on the list, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0029 owns loopback cancel.
- ADR 0027 owns GET-by-id status (live on another PR).
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
