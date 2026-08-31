# ADR 0038 — Analysis-run retry-parent GET path

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0018, ADR 0031, ADR 0032, ADR 0034, ADR 0035, and ADR 0037 for the operator-visible jump from a retry child to its parent. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0037 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel, loopback-CLI, collection-GET, retry, collection-CLI, engine-execute, loopback-binary, CWC, cancel-consumer-parity, Rubin, stored-request, retry-lineage, ESEM/DSEM, cancel-CLI, execute-exchange, execute-TCP, and idempotency-lookup slices.

## Context

Retry-lineage GET lists direct children of a parent. Collection GET lists parent and child as independent rows and does not leak `retried_from`. Stored-request GET returns snapshot/cutoff/model/profile of one run. Idempotency-key lookup resolves a 202 receipt or retry child key to a `run_id` without linkage. Operators who land on a retry child (from collection or lookup) therefore cannot see which parent it was cloned from. Returning RMSE, bias, coverage, SE-gate, or `tepp.scientific_acceptance.v1` on the parent body would treat parent inspect as measurement evidence. GET-by-id (#359) is status/terminal by `run_id` on another stack and remains 400 here.

## Decision

`AnalysisRunLiveService` serves `GET /v1/analysis-runs/{run_id}/parent` on loopback:

- The payload is metric-free: child `run_id`, `run_state`, `idempotency_key`, and a `parent` object (`run_id`, `run_state`, `idempotency_key`) or JSON `null`.
- The `parent` key is always present. Original (never-retried) runs return `"parent": null`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report, `terminal_result`, `tenant_workspace_id`, `snapshot_id`, and `retried_from` never appear.
- Empty GET bodies only. Query strings, GET-by-id, POST `/parent`, GET `/retries`, GET `/request`, GET `/by-idempotency/{key}`, and nonempty bodies fail closed.
- Consumer isolation: another consumer cannot read the first consumer's parent. Missing parent identity fails closed.
- Unknown identities fail closed. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable request storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating GET `/v1/analysis-runs/{run_id}`, POST running/terminal, POST cancel, GET collection, POST retry, GET stored-request, GET retry-lineage, GET idempotency-lookup, loopback CLI, or cancel CLI.
- LineageWeave/Naruon stored-request/retry-lineage/idempotency/parent consumer-parity (mirrors #373; remains a later unique slice).

## Alternatives considered

1. **Add `retried_from` to collection GET rows** — rejected because collection GET (#368) already owns identity-only enumeration and a parallel field would duplicate that head.
2. **Ask operators to scan retry-lineage of every listed run** — rejected because retry-lineage GET (#379) is parent→children and operators often hold only the child `run_id`.
3. **Return `tepp.scientific_acceptance.v1` on a succeeded parent** — rejected because parent bodies must stay metric-free.
4. **Metric-free retry-parent GET on loopback** — accepted.

## Consequences

- Operators can inspect the parent of a listed retry child after retry or idempotency-key lookup.
- Parent pages cannot be mistaken for a succeeded scientific-acceptance result.
- GET-by-id may later return a digest-bound artifact without changing these parent gates.

## Failure and recovery

Unknown identities, extra path segments, GET-by-id, query strings, nonempty bodies, metric keys, unpublished consumers, consumer mismatch, missing parent identity, and non-loopback hosts return a redacted `400` envelope. Oversized run identities return `413`. Credential headers remain `403`. The in-memory registry is not durable; a restart requires re-POSTing the original metric-free create and retry requests. Callers must not fabricate a succeeded scientific-acceptance artifact from a retry-parent payload.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Retry-parent GET remains loopback-only, size-bounded, consumer-scoped, and content-redacting.
- HTTP `200` on a retry-parent payload is not measurement evidence and is not release evidence.

## Compatibility and migration

Create POST, cancel POST, retry POST, collection GET, stored-request GET, retry-lineage GET, idempotency-key lookup GET, temporal-context, and project-history paths are unchanged. GET-by-id remains refused on this slice. Production adapters may replace loopback while preserving metric-free parent fields and the artifact refusal.

## Verification

Falsifiable evidence:

- GET retry-parent JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/`terminal_result`/`tenant_workspace_id`/`snapshot_id`/`retried_from` keys;
- GET of an original run returns `"parent": null`;
- GET of a retry child returns the parent identity;
- GET of the parent after retry still returns `"parent": null`;
- GET does not leak another consumer's parent;
- GET-by-id, query strings, nonempty bodies, POST `/parent`, and unknown identities fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes retry-parent GET dispatch; POST create receipts, cancel, collection GET, retry, stored-request GET, retry-lineage GET, and idempotency-key lookup GET remain valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on parent inspect, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0031 owns loopback collection GET.
- ADR 0032 owns loopback retry HTTP on this stack.
- ADR 0034 owns loopback stored-request GET on this stack.
- ADR 0035 owns loopback retry-lineage GET on this stack.
- ADR 0037 owns loopback idempotency-key lookup GET on this stack.
- ADR 0027 owns GET-by-id status (live on another PR).
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
