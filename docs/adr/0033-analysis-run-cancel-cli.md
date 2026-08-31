# ADR 0033 — Analysis-run cancel loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0029 for the operator-visible cancel client. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0032 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel-HTTP, scientific-acceptance CLI, collection-GET, and collection-CLI slices.

## Context

ADR 0029 serves `POST /v1/analysis-runs/{run_id}/cancel` on the loopback listener, but operators still had to write raw HTTP/1.1 to withdraw an accepted or running run. Duplicating cancel HTTP, GET-by-id, lifecycle POST, the scientific-acceptance CLI (`tepp-analysis-run` on live #362), collection GET, collection CLI `list`, or consumer-parity cancel would collide with live PRs.

## Decision

`tepp_api` extends the loopback-only `tepp-analysis-runs` CLI:

- `cancel` POSTs `/v1/analysis-runs/{run_id}/cancel` with `--run-id` and `--idempotency-key`.
- Empty stdin is admitted (header-and-path cancel). A typed `AnalysisRunCancelRequest` body must match those flags.
- Stdout is metric-free cancelled status: `run_id`, `run_state`, `idempotency_key`, and `terminal_result: null`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report, and a non-null `terminal_result` never appear.
- Non-loopback hosts, unpublished consumers, credential-shaped flags, collection pagination flags, unknown verbs, hostile identities, and metric bodies fail closed.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep raw HTTP as the only cancel path** — rejected because operators still guess framing after ADR 0029.
2. **Add `cancel` onto the live scientific-acceptance CLI (#362)** — rejected because that head already owns create/running/terminal/status and is stacked on GET-by-id, not cancel HTTP.
3. **Open a second `tepp-analysis-runs` binary beside collection CLI list (#371)** — rejected because the operator-visible command is the same binary.
4. **Persist cancelled rows in PostgreSQL** — rejected as GAP-003B / live draft #287.
5. **Loopback cancel CLI with the same metric-free gates as ADR 0029** — accepted.

## Consequences

- Operators can withdraw accepted or running runs on the same loopback listener that created them without writing HTTP.
- Cancelled status cannot be mistaken for a succeeded scientific-acceptance result.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, metric keys, mismatched typed bodies, unpublished consumers, credential flags, oversized run identities, and collection pagination flags fail closed. Succeeded, failed, and unknown runs remain refused by ADR 0029. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- Process exit 0 on a cancelled status is not measurement evidence and is not an ADR 0014 claim.

## Compatibility and migration

Cancel HTTP, collection GET, collection CLI `list`, create POST, temporal-context, and project-history paths are unchanged. The scientific-acceptance CLI binary name `tepp-analysis-run` remains owned by ADR 0030 / #362. Production adapters may replace loopback while preserving metric-free cancelled status.

## Verification

Falsifiable evidence:

- CLI cancel JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance keys and no non-null `terminal_result`;
- CLI cancel of accepted is metric-free `cancelled` and replay is idempotent;
- another consumer cannot cancel the first consumer's run;
- non-loopback host, credential flags, collection flags, metric stdin, and unknown verbs fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the cancel verb from `tepp-analysis-runs`; collection `list` and cancel HTTP remain valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on cancel, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0029 owns loopback cancel HTTP.
- ADR 0032 owns `tepp-analysis-runs list`.
- ADR 0030 owns the scientific-acceptance loopback CLI (live #362).
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
