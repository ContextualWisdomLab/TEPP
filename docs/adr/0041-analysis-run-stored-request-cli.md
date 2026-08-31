# ADR 0041 — Analysis-run stored-request loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0034 for the operator-visible stored-request client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on the stored-request lineage; other live PRs may reuse 0041 on unrelated stacks.

## Context

ADR 0034 serves `GET /v1/analysis-runs/{run_id}/request` and ADR 0040 gives LineageWeave a stored-request exchange, but operators still had to write raw HTTP/1.1 to inspect snapshot, cutoff, model contract, and output profile before retry. Duplicating stored-request HTTP, stored-request consumer-parity, retry HTTP, retry CLI (`tepp-retry` on live #394), collection GET/CLI, cancel/create/status CLIs, or GET-by-id would collide with live PRs.

## Decision

`tepp_api` publishes a loopback-only `tepp-analysis-runs` CLI on this stored-request lineage:

- `stored-request` GETs `/v1/analysis-runs/{run_id}/request` with `--run-id`.
- Empty stdin is required. A nonempty body fails closed.
- Stdout is metric-free `AnalysisRunStoredRequest` JSON. `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, `terminal_result`, and `tenant_workspace_id` never appear.
- Non-loopback hosts, unpublished consumers, credential-shaped flags, collection pagination flags, unknown verbs, hostile identities, and metric keys on stdout fail closed.
- This slice does not implement stored-request HTTP (ADR 0034) and does not open GET-by-id on this stack.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep raw HTTP as the only stored-request path** — rejected because operators still guess framing after ADR 0034.
2. **Add `stored-request` onto the live retry CLI (#394)** — rejected because that head already owns `tepp-retry` POST retry.
3. **Return scientific-acceptance on succeeded inspect** — rejected because stored-request bodies must stay metric-free.
4. **Loopback stored-request CLI stacked on ADR 0034/0040** — accepted.

## Consequences

- Operators can inspect stored create fields of a listed run before retry without writing HTTP.
- Inspect stdout cannot be mistaken for a succeeded scientific-acceptance result.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, nonempty stdin, unpublished consumers, credential flags, oversized run identities, and collection flags fail closed. Unknown runs and consumer mismatch remain refused by ADR 0034. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- Process exit 0 on a stored-request inspect is not measurement evidence and is not an ADR 0014 claim.

## Compatibility and migration

Stored-request GET HTTP, stored-request consumer-parity, create POST, cancel POST, retry POST, collection GET, temporal-context, and project-history paths are unchanged. The collection/cancel/create `tepp-analysis-runs` verbs live on a parallel stack and merge by combining verbs. Production adapters may replace loopback while preserving metric-free inspect fields.

## Verification

Falsifiable evidence:

- CLI stored-request of an accepted run returns snapshot/cutoff/model/profile with no RMSE/bias/coverage/SE-gate/`tepp.scientific_acceptance.v1` keys;
- another consumer cannot inspect the first consumer's stored request;
- non-loopback host, credential flags, collection flags, nonempty stdin, and unknown verbs fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the stored-request verb and `tepp-analysis-runs` binary from this lineage; stored-request GET HTTP remains valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on inspect, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0034 owns loopback stored-request GET.
- ADR 0040 owns LineageWeave stored-request-exchange parity.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
