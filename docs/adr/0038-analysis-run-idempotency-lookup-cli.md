# ADR 0038 — Analysis-run idempotency-key lookup loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0037 for the operator-visible lookup client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on the lookup lineage; other live PRs may reuse 0038 on unrelated stacks (retry-parent GET).

## Context

ADR 0037 serves `GET /v1/analysis-runs/by-idempotency/{idempotency_key}`, but operators still had to write raw HTTP/1.1 to jump from a 202 receipt or retry child key to a durable `run_id`. Duplicating lookup HTTP, stored-request CLI (#395), retry CLI (#394), retry-parent CLI (#400), lifecycle CLI (#397), collection/cancel/create/status CLIs, or GET-by-id would collide with live PRs.

## Decision

`tepp_api` publishes a loopback-only `tepp-analysis-runs` CLI on this lookup lineage:

- `lookup` GETs `/v1/analysis-runs/by-idempotency/{idempotency_key}` with `--idempotency-key`.
- The key travels in the path. The CLI does not send an `idempotency-key` header.
- Empty stdin is required. A nonempty body fails closed.
- Stdout is metric-free `AnalysisRunIdempotencyLookup` JSON. `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, `terminal_result`, `tenant_workspace_id`, and `snapshot_id` never appear.
- Non-loopback hosts, unpublished consumers, credential-shaped flags, `--run-id`, collection pagination flags, unknown verbs, hostile identities, and metric keys on stdout fail closed.
- This slice does not implement lookup HTTP (ADR 0037).
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep raw HTTP as the only lookup path** — rejected because operators still guess framing after ADR 0037.
2. **Add `lookup` onto the live stored-request CLI (#395)** — rejected because that head already owns inspect-by-`run_id`.
3. **Return scientific-acceptance on succeeded lookup** — rejected because lookup bodies must stay metric-free.
4. **Loopback lookup CLI stacked on ADR 0037** — accepted.

## Consequences

- Operators can resolve a 202 receipt or retry child key to a durable `run_id` without writing HTTP.
- Lookup stdout cannot be mistaken for a succeeded scientific-acceptance result.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, nonempty stdin, unpublished consumers, credential flags, oversized keys, `--run-id`, and collection flags fail closed. Unknown keys and consumer mismatch remain refused by ADR 0037. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- Process exit 0 on a lookup is not measurement evidence and is not an ADR 0014 claim.

## Compatibility and migration

Lookup GET HTTP, create POST, cancel POST, retry POST, collection GET, stored-request GET, retry-lineage GET, temporal-context, and project-history paths are unchanged. The collection/cancel/create `tepp-analysis-runs` verbs live on a parallel stack and merge by combining verbs.

## Verification

Falsifiable evidence:

- CLI lookup of an accepted run returns `run_id`/`run_state`/`idempotency_key` with no RMSE/bias/coverage/SE-gate/`tepp.scientific_acceptance.v1` keys;
- another consumer cannot resolve the first consumer's key;
- non-loopback host, credential flags, `--run-id`, nonempty stdin, and unknown verbs fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the lookup verb and `tepp-analysis-runs` binary from this lineage; lookup GET HTTP remains valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on lookup, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0037 owns loopback idempotency-key lookup GET.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
