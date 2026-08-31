# ADR 0035 — Analysis-run retry-lineage GET path

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0018, ADR 0031, ADR 0032, and ADR 0034 for the operator-visible retry parent/child inspect. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0034 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel, loopback-CLI, collection-GET, retry, collection-CLI, engine-execute, loopback-binary, CWC, cancel-consumer-parity, Rubin, stored-request, ESEM/DSEM, and cancel-CLI slices.

## Context

Retry HTTP clones a failed or cancelled run into a new metric-free `202 Accepted`. Collection GET lists parent and child as independent rows. Stored-request GET returns snapshot/cutoff/model/profile of one run. Operators therefore cannot see which cloned attempts belong to a listed failed or cancelled parent. Returning RMSE, bias, coverage, SE-gate, or `tepp.scientific_acceptance.v1` on the lineage body would treat parent/child enumeration as measurement evidence.

## Decision

`AnalysisRunLiveService` serves `GET /v1/analysis-runs/{run_id}/retries` on loopback:

- The payload is metric-free: parent `run_id`, `run_state`, `idempotency_key`, and a `retries` array of direct children (`run_id`, `run_state`, `idempotency_key`).
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report, `terminal_result`, `tenant_workspace_id`, and `snapshot_id` never appear.
- Direct children only. Grandchildren are listed on their own parent. An empty `retries` array is `200` when the parent exists and was never retried.
- Empty GET bodies only. Query strings, GET-by-id, POST `/retries`, POST `/retry`, GET `/request`, and nonempty bodies fail closed.
- Consumer isolation: another consumer cannot read the first consumer's retry lineage.
- Unknown identities fail closed. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable request storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating GET `/v1/analysis-runs/{run_id}`, POST running/terminal, POST cancel, GET collection, POST retry, GET stored-request, loopback CLI, or cancel CLI.

## Alternatives considered

1. **Add `retried_from` to collection GET rows** — rejected because collection GET (#368) already owns identity-only enumeration and a parallel field would duplicate that head.
2. **Return `tepp.scientific_acceptance.v1` on succeeded children** — rejected because lineage bodies must stay metric-free.
3. **Ask operators to correlate parent and child from local notes** — rejected because retry already cloned the parent and collection lists both without linkage.
4. **Metric-free retry-lineage GET on loopback** — accepted.

## Consequences

- Operators can inspect direct retry children of a listed run after retry.
- Lineage pages cannot be mistaken for a succeeded scientific-acceptance result.
- GET-by-id may later return a digest-bound artifact without changing these lineage gates.

## Failure and recovery

Unknown identities, extra path segments, GET-by-id, query strings, nonempty bodies, metric keys, unpublished consumers, consumer mismatch, and non-loopback hosts return a redacted `400` envelope. Oversized run identities and more than 64 direct children return `413`. Credential headers remain `403`. The in-memory registry is not durable; a restart requires re-POSTing the original metric-free create and retry requests. Callers must not fabricate a succeeded scientific-acceptance artifact from a retry-lineage payload.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Retry-lineage GET remains loopback-only, size-bounded, consumer-scoped, and content-redacting.
- HTTP `200` on a retry-lineage payload is not measurement evidence and is not release evidence.

## Compatibility and migration

Create POST, cancel POST, retry POST, collection GET, stored-request GET, temporal-context, and project-history paths are unchanged. GET-by-id remains refused on this slice. Production adapters may replace loopback while preserving metric-free lineage fields and the artifact refusal.

## Verification

Falsifiable evidence:

- GET retry-lineage JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/`terminal_result`/`tenant_workspace_id`/`snapshot_id` keys;
- GET returns direct children after retry of failed and cancelled parents;
- GET of a never-retried parent returns an empty `retries` array;
- GET does not leak another consumer's retry lineage;
- GET-by-id, query strings, nonempty bodies, POST `/retries`, and unknown identities fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes retry-lineage GET dispatch; POST create receipts, cancel, collection GET, retry, and stored-request GET remain valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on lineage, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0031 owns loopback collection GET.
- ADR 0032 owns loopback retry HTTP on this stack.
- ADR 0034 owns loopback stored-request GET on this stack.
- ADR 0027 owns GET-by-id status (live on another PR).
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
