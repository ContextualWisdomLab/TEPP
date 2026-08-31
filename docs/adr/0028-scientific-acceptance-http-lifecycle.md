# ADR 0028 — Scientific-acceptance loopback HTTP lifecycle POST

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0027 for the operator-visible status update. Does not supersede ADR 0014 claim-promotion authority and does not reuse ADR 0026 or ADR 0027.

## Context

ADR 0027 serves `GET /v1/analysis-runs/{run_id}` on the loopback listener, but the only way to record running or terminal status was a test-only helper. Production runs therefore stayed accepted forever, and a succeeded scientific-acceptance GET could not be produced through the HTTP boundary. Duplicating the GET slice, the terminal-result DTO, or Compose persistence would collide with live PRs.

## Decision

`AnalysisRunLiveService` serves production lifecycle updates on loopback:

- `POST /v1/analysis-runs/{run_id}/running` records a metric-free running status.
- `POST /v1/analysis-runs/{run_id}/terminal` records a request-bound terminal status.
- Canonical scientific-acceptance bytes travel as `scientific_acceptance_json` on the transition body so `result_sha256` hashes those exact bytes.
- Accepted and running responses stay metric-free. Only a succeeded status whose request profile is `scientific_acceptance_v1` may return `tepp.scientific_acceptance.v1` on the subsequent GET.
- Reverse transitions, mutating a terminal run, a failed-plus-artifact emission, an unknown run, a consumer/idempotency mismatch, and receipt metric keys fail closed.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.

## Alternatives considered

1. **Keep the test-only recorder** — rejected because operators cannot reach terminal status over HTTP.
2. **Stack the write path onto the live GET PR** — rejected because that head is already under review as a GET-only slice.
3. **Persist running/terminal rows in PostgreSQL** — rejected as GAP-003B / live draft #287.
4. **Loopback POST running/terminal with HTTP-layer schema, profile, and digest gates** — accepted.

## Consequences

- A worker or operator can move an accepted loopback run to running and then to terminal without a test helper.
- GET remains the safe read (ADR 0027). POST remains the state change (RFC 9110 §9.3.3).
- Canonical artifact bytes are preserved for SHA-256 identity on the write path.

## Failure and recovery

Unknown run identities, extra path segments, metric keys on running bodies, failed-plus-artifact emission, reverse transitions, and consumer mismatch return a redacted `400` envelope. Credential headers remain `403`. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- POST remains loopback-only, size-bounded, and content-redacting.
- SHA-256 digest agreement is a byte-identity check, not a validity claim.
- HTTP `200` on a succeeded scientific-acceptance GET is not release evidence.

## Compatibility and migration

GET status, POST create, temporal-context, and project-history paths are unchanged. Production adapters may replace loopback while preserving metric-free receipts and the succeeded-only scientific-acceptance rule.

## Verification

Falsifiable evidence:

- POST create and POST running JSON have no RMSE/bias/coverage/SE-gate/scientific-acceptance keys;
- POST terminal succeeded with profile `scientific_acceptance_v1` then GET returns `tepp.scientific_acceptance.v1` only when the artifact digest matches;
- POST failed with an artifact, reverse transitions, unknown run, and consumer mismatch fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the running/terminal POST dispatch; GET status and POST receipts remain valid. A superseding ADR is required to persist status, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0027 owns the GET status read.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0022 owns deterministic execution to a digest-bound terminal result.
- ADR 0014 owns scientific claim promotion.
- ADR 0008 owns SHA-256 identity.
- ADR 0011 owns standalone/modular HTTP boundaries.
