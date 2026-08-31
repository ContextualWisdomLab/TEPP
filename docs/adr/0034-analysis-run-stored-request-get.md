# ADR 0034 — Analysis-run stored-request GET path

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0018, ADR 0031, and ADR 0032 for the operator-visible stored-request inspect. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0033 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel, loopback-CLI, collection-GET, retry, collection-CLI, engine-execute, and loopback-binary slices.

## Context

Collection GET lists `run_id`, `run_state`, and `idempotency_key` only. Retry HTTP clones a failed or cancelled run blindly. Operators therefore cannot inspect `snapshot_id`, `knowledge_cutoff`, `model_contract_version`, or `output_profile` of a listed run before retry. GET-by-id (#359) returns status/terminal on a different stack and would duplicate that head if stacked here. Returning RMSE, bias, coverage, SE-gate, or `tepp.scientific_acceptance.v1` on the inspect body would treat enumeration of stored create fields as measurement evidence.

## Decision

`AnalysisRunLiveService` serves `GET /v1/analysis-runs/{run_id}/request` on loopback:

- The payload is metric-free: `run_id`, `run_state`, `idempotency_key`, `snapshot_id`, `knowledge_cutoff`, `model_contract_version`, and `output_profile`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report, `terminal_result`, and `tenant_workspace_id` never appear.
- Accepted, running, cancelled, succeeded, and failed runs are readable. Succeeded rows still omit the artifact.
- Empty GET bodies only. Query strings, GET-by-id, POST `/request`, and nonempty bodies fail closed.
- Consumer isolation: another consumer cannot read the first consumer's stored request.
- Unknown identities fail closed. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable request storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating GET `/v1/analysis-runs/{run_id}`, POST running/terminal, POST cancel, GET collection, POST retry, or loopback CLI.

## Alternatives considered

1. **Stack inspect onto the live GET-by-id PR** — rejected because that head already owns single-run status and a parallel stack would duplicate it.
2. **Return `tepp.scientific_acceptance.v1` on succeeded inspect** — rejected because stored-request bodies must stay metric-free.
3. **Ask operators to reconstruct snapshot/cutoff/profile from local notes** — rejected because collection GET already identified the run and retry clones blindly.
4. **Metric-free stored-request GET on loopback** — accepted.

## Consequences

- Operators can inspect stored create fields of a listed run before retry.
- Inspect pages cannot be mistaken for a succeeded scientific-acceptance result.
- GET-by-id may later return a digest-bound artifact without changing these inspect gates.

## Failure and recovery

Unknown identities, extra path segments, GET-by-id, query strings, nonempty bodies, metric keys, unpublished consumers, consumer mismatch, and non-loopback hosts return a redacted `400` envelope. Oversized run identities return `413`. Credential headers remain `403`. The in-memory registry is not durable; a restart requires re-POSTing the original metric-free create requests. Callers must not fabricate a succeeded scientific-acceptance artifact from a stored-request payload.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Stored-request GET remains loopback-only, size-bounded, consumer-scoped, and content-redacting.
- HTTP `200` on a stored-request payload is not measurement evidence and is not release evidence.

## Compatibility and migration

Create POST, cancel POST, retry POST, collection GET, temporal-context, and project-history paths are unchanged. GET-by-id remains refused on this slice. Production adapters may replace loopback while preserving metric-free stored-request fields and the artifact refusal.

## Verification

Falsifiable evidence:

- GET stored-request JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/`terminal_result`/`tenant_workspace_id` keys;
- GET returns snapshot, cutoff, model contract, and output profile for failed and cancelled runs;
- GET does not leak another consumer's stored request;
- GET-by-id, query strings, nonempty bodies, and unknown identities fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes stored-request GET dispatch; POST create receipts, cancel, collection GET, and retry remain valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on inspect, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0031 owns loopback collection GET.
- ADR 0032 owns loopback retry HTTP on this stack.
- ADR 0027 owns GET-by-id status (live on another PR).
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
