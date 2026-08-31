# ADR 0032 — Analysis-run collection loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0031 for the operator-visible collection client. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0031 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel, scientific-acceptance CLI, and collection-GET slices.

## Context

ADR 0031 serves `GET /v1/analysis-runs` on the loopback listener, but operators still had to write raw HTTP/1.1 to enumerate accepted, running, cancelled, or terminal runs. Duplicating the collection GET listener, GET-by-id, lifecycle POST, cancel HTTP, or the scientific-acceptance CLI (`tepp-analysis-run` on live #362) would collide with live PRs.

## Decision

`tepp_api` publishes a loopback-only `tepp-analysis-runs` CLI:

- `list` GETs `/v1/analysis-runs` with optional `tepp-page-cursor` / `tepp-page-limit`.
- Stdout is the metric-free collection page: `run_id`, `run_state`, `idempotency_key`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report, and `terminal_result` never appear.
- Non-loopback hosts, unpublished consumers, credential-shaped flags, GET-by-id flags, nonempty stdin, unknown verbs, and hostile pagination fail closed.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep raw HTTP as the only collection path** — rejected because operators still guess framing after ADR 0031.
2. **Add `list` onto the live scientific-acceptance CLI (#362)** — rejected because that head already owns create/running/terminal/status and is stacked on GET-by-id, not collection GET.
3. **Persist listed rows in PostgreSQL** — rejected as GAP-003B / live draft #287.
4. **Loopback collection CLI with the same metric-free gates as ADR 0031** — accepted.

## Consequences

- Operators can enumerate runs on the same loopback listener that created them without writing HTTP.
- Collection pages cannot be mistaken for a succeeded scientific-acceptance result.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, metric keys, nonempty bodies, unknown cursors, zero or non-integer limits, unpublished consumers, and credential flags fail closed. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- Process exit 0 on a collection page is not measurement evidence and is not an ADR 0014 claim.

## Compatibility and migration

Collection GET, create POST, cancel POST, temporal-context, and project-history paths are unchanged. The scientific-acceptance CLI binary name `tepp-analysis-run` remains owned by ADR 0030 / #362. Production adapters may replace loopback while preserving metric-free collection rows.

## Verification

Falsifiable evidence:

- CLI list JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/`terminal_result` keys;
- CLI list returns accepted and cancelled rows for one consumer and does not leak another consumer's runs;
- non-loopback host, credential flags, GET-by-id flags, nonempty stdin, and unknown verbs fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the `tepp-analysis-runs` binary and client module; collection GET remains valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on the list, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0031 owns loopback collection GET.
- ADR 0030 owns the scientific-acceptance loopback CLI (live #362).
- ADR 0029 owns loopback cancel.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
