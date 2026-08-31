# ADR 0029 — Analysis-run status loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0027 for the operator-visible status client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on the GET-status lineage; other live PRs may reuse 0029 on unrelated stacks (cancel HTTP).

## Context

ADR 0027 serves `GET /v1/analysis-runs/{run_id}` on the loopback listener, and ADR 0028 gives LineageWeave a status-exchange builder, but operators still had to write raw HTTP/1.1 to inspect one run. Duplicating GET-by-id HTTP, status consumer-parity, lifecycle POST, cancel HTTP, the scientific-acceptance CLI (`tepp-analysis-run` on live #362), collection GET/CLI, cancel CLI, or create CLI would collide with live PRs.

## Decision

`tepp_api` publishes a loopback-only `tepp-analysis-runs` CLI on this GET-status lineage:

- `status` GETs `/v1/analysis-runs/{run_id}` with `--run-id` and `--idempotency-key`.
- Empty stdin is required. A nonempty body fails closed.
- Accepted, running, and failed stdout is metric-free `AnalysisRunStatus` JSON.
- `tepp.scientific_acceptance.v1` appears only on a succeeded GET whose request profile is `scientific_acceptance_v1`.
- Non-loopback hosts, unpublished consumers, credential-shaped flags, collection pagination flags, unknown verbs, hostile identities, and metric keys on non-succeeded stdout fail closed.
- This slice does not implement GET-by-id HTTP (ADR 0027) and does not open `NaruonLiveService` to GET.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep raw HTTP as the only status path** — rejected because operators still guess framing after ADR 0027.
2. **Add `status` onto the live scientific-acceptance CLI (#362)** — rejected because that head already owns create/running/terminal/status for the scientific-acceptance profile on the lifecycle stack.
3. **Implement GET-by-id HTTP on the collection/cancel/create CLI stack (#385)** — rejected because GET-by-id is live #359; that listener treats GET as collection-only.
4. **Loopback status CLI stacked on ADR 0027/0028 with the same scientific-acceptance gate** — accepted.

## Consequences

- Operators can inspect one run on the same loopback listener that created it without writing HTTP.
- Accepted/running/failed status cannot be mistaken for a succeeded scientific-acceptance result.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, nonempty stdin, unpublished consumers, credential flags, oversized run identities, and collection flags fail closed. Unknown runs and consumer/idempotency mismatch remain refused by ADR 0027. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- Process exit 0 on an accepted or succeeded status is not measurement evidence and is not an ADR 0014 claim.

## Compatibility and migration

GET-by-id HTTP, status consumer-parity, create POST, temporal-context, and project-history paths are unchanged. The scientific-acceptance CLI binary name `tepp-analysis-run` remains owned by the lifecycle stack. The collection/cancel/create `tepp-analysis-runs` verbs live on a parallel stack and merge by combining verbs. Production adapters may replace loopback while preserving the succeeded-only scientific-acceptance rule.

## Verification

Falsifiable evidence:

- CLI status of accepted/running is metric-free and has no `tepp.scientific_acceptance.v1`;
- CLI status of succeeded `scientific_acceptance_v1` may print `tepp.scientific_acceptance.v1`;
- another consumer cannot read the first consumer's run;
- non-loopback host, credential flags, collection flags, nonempty stdin, and unknown verbs fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the status verb and `tepp-analysis-runs` binary from this lineage; GET-by-id HTTP remains valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on accepted/running status, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0027 owns loopback status GET.
- ADR 0028 owns LineageWeave status-exchange parity.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
