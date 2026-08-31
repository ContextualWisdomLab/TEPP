# ADR 0034 — Analysis-run create loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0018 for the operator-visible create client. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0033 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel-HTTP, scientific-acceptance CLI, collection-GET, collection-CLI, and cancel-CLI slices.

## Context

ADR 0018 accepts `POST /v1/analysis-runs` on the loopback listener, but operators still had to write raw HTTP/1.1 to submit a metric-free analysis run. Duplicating create HTTP, GET-by-id, lifecycle POST, cancel HTTP, the scientific-acceptance CLI (`tepp-analysis-run` on live #362), collection GET, collection CLI `list`, or cancel CLI would collide with live PRs.

## Decision

`tepp_api` extends the loopback-only `tepp-analysis-runs` CLI:

- `create` POSTs `/v1/analysis-runs` with `--idempotency-key` and a typed `AnalysisRunRequest` stdin body.
- Empty stdin is refused. The body's idempotency key must match the flag.
- Stdout is metric-free `202 Accepted`: `run_id`, `run_state=accepted`, and `idempotency_key`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report, and `terminal_result` never appear.
- Non-loopback hosts, unpublished consumers, credential-shaped flags, collection pagination flags, cancel `--run-id`, unknown verbs, mismatched keys, and metric bodies fail closed.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep raw HTTP as the only create path** — rejected because operators still guess framing after ADR 0018.
2. **Add `create` onto the live scientific-acceptance CLI (#362)** — rejected because that head already owns create/running/terminal/status for the scientific-acceptance profile and is stacked on GET-by-id, not the collection/cancel operator binary.
3. **Open a second `tepp-analysis-runs` binary beside collection CLI list (#371) and cancel CLI (#378)** — rejected because the operator-visible command is the same binary.
4. **Persist created rows in PostgreSQL** — rejected as GAP-003B / live draft #287.
5. **Loopback create CLI with the same metric-free gates as ADR 0018** — accepted.

## Consequences

- Operators can submit metric-free analysis runs on the same loopback listener that lists and cancels them without writing HTTP.
- Accepted receipts cannot be mistaken for a succeeded scientific-acceptance result.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, metric keys, mismatched idempotency keys, unpublished consumers, credential flags, empty stdin, and collection/cancel flags fail closed. Conflicting idempotent bodies remain refused by ADR 0018. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- Process exit 0 on an accepted receipt is not measurement evidence and is not an ADR 0014 claim.

## Compatibility and migration

Cancel HTTP, collection GET, collection CLI `list`, cancel CLI, create POST, temporal-context, and project-history paths are unchanged. The scientific-acceptance CLI binary name `tepp-analysis-run` remains owned by ADR 0030 / #362. Production adapters may replace loopback while preserving metric-free accepted receipts.

## Verification

Falsifiable evidence:

- CLI create JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance keys and no `terminal_result`;
- CLI create of a valid request is metric-free `accepted` and replay is idempotent;
- another consumer cannot collide with the first consumer's idempotency key;
- non-loopback host, credential flags, collection/cancel flags, metric stdin, empty stdin, and unknown verbs fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the create verb from `tepp-analysis-runs`; collection `list`, cancel, and create HTTP remain valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on create, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0032 owns `tepp-analysis-runs list`.
- ADR 0033 owns `tepp-analysis-runs cancel`.
- ADR 0030 owns the scientific-acceptance loopback CLI (live #362).
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
