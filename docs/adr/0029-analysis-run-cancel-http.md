# ADR 0029 — Analysis-run cancel HTTP path

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0018 for the operator-visible cancel path. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0028 remain on live GAP-003A engine-library, GET-status, and lifecycle-POST slices.

## Context

`docs/API_CONTRACT.md` documents `POST /v1/analysis-runs/{run_id}/cancel` and the lifecycle `accepted/running -> cancelling -> cancelled`. Protected main accepts analysis runs on loopback but refuses every non-create analysis-run path. Operators therefore cannot withdraw an accepted or running run. Returning RMSE, bias, coverage, SE-gate, or `tepp.scientific_acceptance.v1` on a cancel body would treat cancellation as measurement evidence. Stacking this slice onto the live GET-status or lifecycle-POST PRs would duplicate those heads.

## Decision

`AnalysisRunLiveService` serves `POST /v1/analysis-runs/{run_id}/cancel` on loopback:

- Accepted and running runs transition atomically to metric-free `cancelled` status.
- Already-cancelled runs are idempotent: the same `200` cancelled status is returned.
- Succeeded, failed, and unknown runs cannot be cancelled.
- Empty POST bodies are admitted and bind path `run_id` plus the `idempotency-key` header. A typed `AnalysisRunCancelRequest` body must match path identity and header key.
- Cancel bodies and cancelled status JSON refuse RMSE, bias, coverage, SE-gate, scientific-acceptance, and report keys.
- `Cancelling` is not an HTTP response state: the loopback proof has no worker, so cancel is atomic.
- GET status and running/terminal POST transitions remain later slices. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable cancel storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating GET `/v1/analysis-runs/{run_id}` or POST running/terminal.

## Alternatives considered

1. **Stack cancel onto the live GET-status PR** — rejected because that head is moving and cancel is independently operator-visible.
2. **Return `cancelling` then `cancelled`** — rejected because the loopback proof has no worker and a two-step HTTP state would be fiction.
3. **Carry scientific-acceptance metrics on the cancel receipt** — rejected because cancellation is not measurement evidence.
4. **Atomic accepted/running → cancelled with metric-free `200`** — accepted.

## Consequences

- Operators can withdraw an accepted or running run on the same loopback listener that created it.
- Cancelled status cannot be mistaken for a succeeded scientific-acceptance result.
- GET status may later report `cancelled` without changing these cancel gates.

## Failure and recovery

Unknown run identities, extra path segments, truncated percent-encoding, metric keys on cancel bodies, succeeded/failed cancel, consumer mismatch, and path/header/body identity mismatch return a redacted `400` envelope. Oversized run identities return `413`. Credential headers remain `403`. The in-memory registry is not durable; a restart requires re-POSTing the original metric-free create request. Callers must not fabricate a succeeded run from a cancelled status.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Cancel remains loopback-only, size-bounded, and content-redacting.
- HTTP `200` on a cancelled run is not measurement evidence and is not release evidence.

## Compatibility and migration

The existing POST analysis-run, temporal-context, and project-history paths are unchanged. GET remains refused on this slice. Production adapters may replace loopback while preserving metric-free cancelled status and the succeeded/failed cancel refusal.

## Verification

Falsifiable evidence:

- POST accepted JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance keys;
- POST cancel of accepted and running returns metric-free `cancelled`;
- POST cancel of already-cancelled is idempotent;
- POST cancel of succeeded, failed, and unknown runs fails closed;
- metric keys, consumer mismatch, and identity mismatch fail closed;
- GET `/v1/analysis-runs` remains `400` on this slice;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes cancel dispatch and the in-memory run-id index; POST create receipts remain valid. A superseding ADR is required to persist cancel, bind a public address, emit `cancelling` as an HTTP state, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0022 owns deterministic execution to a digest-bound terminal result.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
