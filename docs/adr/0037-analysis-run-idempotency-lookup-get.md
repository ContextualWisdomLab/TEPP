# ADR 0037 — Analysis-run idempotency-key lookup GET path

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0018, ADR 0031, ADR 0032, ADR 0034, and ADR 0035 for the operator-visible jump from an idempotency key to a durable run identity. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0036 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel, loopback-CLI, collection-GET, retry, collection-CLI, engine-execute, loopback-binary, CWC, cancel-consumer-parity, Rubin, stored-request, retry-lineage, ESEM/DSEM, and cancel-CLI slices.

## Context

Collection GET enumerates runs as cursor-paginated identity rows. Stored-request GET and retry-lineage GET require a `run_id`. Retry HTTP clones a failed or cancelled run under a **new** idempotency key. Operators who hold a 202 receipt or a log key therefore cannot jump to that run without scanning pages. Returning RMSE, bias, coverage, SE-gate, or `tepp.scientific_acceptance.v1` on the lookup body would treat key resolution as measurement evidence. GET-by-id (#359) is status/terminal by `run_id` on another stack and remains 400 here.

## Decision

`AnalysisRunLiveService` serves `GET /v1/analysis-runs/by-idempotency/{idempotency_key}` on loopback:

- The payload is metric-free: `run_id`, `run_state`, `idempotency_key`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report, `terminal_result`, `tenant_workspace_id`, and `snapshot_id` never appear.
- Lookup is consumer-scoped. Zero matches and more than one match fail closed (no tenant oracle).
- Empty GET bodies only. Query strings, GET-by-id, POST `/by-idempotency`, GET `/request`, GET `/retries`, and nonempty bodies fail closed.
- The key travels in the path. The NARUON exchange does not send an `idempotency-key` header or credentials.
- Unknown keys fail closed. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable request storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating GET `/v1/analysis-runs/{run_id}`, POST running/terminal, POST cancel, GET collection, POST retry, GET stored-request, GET retry-lineage, loopback CLI, or cancel CLI.
- LineageWeave/Naruon stored-request consumer-parity (mirrors #373; remains a later unique slice).

## Alternatives considered

1. **Ask operators to scan collection pages** — rejected because collection GET (#368) is cursor-bounded and retry mints a new key the operator already holds.
2. **Return `tepp.scientific_acceptance.v1` on succeeded lookup** — rejected because lookup bodies must stay metric-free.
3. **Reuse GET-by-id with the key as `{run_id}`** — rejected because GET-by-id (#359) owns status/terminal by server-assigned identity on another stack.
4. **Metric-free idempotency-key lookup GET on loopback** — accepted.

## Consequences

- Operators can resolve a 202 receipt or retry child key to a durable `run_id` without scanning the collection.
- Lookup pages cannot be mistaken for a succeeded scientific-acceptance result.
- GET-by-id may later return a digest-bound artifact without changing these lookup gates.

## Failure and recovery

Unknown keys, extra path segments, GET-by-id, query strings, nonempty bodies, metric keys, unpublished consumers, consumer mismatch, ambiguous multi-tenant matches, and non-loopback hosts return a redacted `400` envelope. Oversized keys return `413`. Credential headers remain `403`. The in-memory registry is not durable; a restart requires re-POSTing the original metric-free create and retry requests. Callers must not fabricate a succeeded scientific-acceptance artifact from a lookup payload.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Idempotency-key lookup remains loopback-only, size-bounded, consumer-scoped, and content-redacting.
- HTTP `200` on a lookup payload is not measurement evidence and is not release evidence.
- Ambiguous matches fail closed so lookup cannot become a tenant-count oracle.

## Compatibility and migration

Create POST, cancel POST, retry POST, collection GET, stored-request GET, retry-lineage GET, temporal-context, and project-history paths are unchanged. GET-by-id remains refused on this slice. Production adapters may replace loopback while preserving metric-free lookup fields and the artifact refusal.

## Verification

Falsifiable evidence:

- GET lookup JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/`terminal_result`/`tenant_workspace_id`/`snapshot_id` keys;
- GET of a create key and of a retry child key each return the matching `run_id`;
- GET does not leak another consumer's run;
- GET-by-id, query strings, nonempty bodies, POST `/by-idempotency`, unknown keys, and reserved `by-idempotency` as a key fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes idempotency-lookup GET dispatch; POST create receipts, cancel, collection GET, retry, stored-request GET, and retry-lineage GET remain valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on lookup, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0031 owns loopback collection GET.
- ADR 0032 owns loopback retry HTTP on this stack.
- ADR 0034 owns loopback stored-request GET on this stack.
- ADR 0035 owns loopback retry-lineage GET on this stack.
- ADR 0027 owns GET-by-id status (live on another PR).
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
