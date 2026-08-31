# ADR 0027 — Scientific-acceptance loopback HTTP status path

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0018 and ADR 0022 for the operator-visible status read. Does not supersede ADR 0014 claim-promotion authority and does not reuse ADR 0026.

## Context

Modular consumers can POST an analysis-run receipt on the loopback listener and can build a `GET /v1/analysis-runs/{run_id}` exchange, but the listener refused GET. Scientific-acceptance metrics therefore could not appear on an operator-visible status/terminal HTTP path. Putting RMSE, bias, coverage, or SE-gate keys on the create receipt would treat acknowledgement as measurement evidence. Duplicating the terminal-result DTO on this slice would collide with the live API wire PR.

## Decision

`AnalysisRunLiveService` serves `GET /v1/analysis-runs/{run_id}` on loopback:

- `POST /v1/analysis-runs` remains a metric-free `202 Accepted` receipt.
- Accepted and running GET bodies are metric-free `AnalysisRunStatus` JSON.
- Only a succeeded status whose request profile is `scientific_acceptance_v1` may return `tepp.scientific_acceptance.v1`.
- A failed status cannot carry the scientific-acceptance object.
- An all-zero binding or result digest, a digest mismatch, a profile mismatch, a GET body, an unknown run, or a consumer/idempotency mismatch fails closed.
- This slice does not introduce a `ScientificAcceptanceArtifact` DTO. Persistence, Compose recovery, and worker execution remain GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.

## Alternatives considered

1. **Stack GET onto the live terminal-result DTO PR** — rejected because that head is moving and the HTTP gap is independently operator-visible.
2. **Copy the terminal-result scientific-acceptance DTO into this crate module** — rejected as a duplicate API wire slice.
3. **Return metrics on the accepted receipt** — rejected because acknowledgement is not measurement evidence.
4. **Loopback GET with HTTP-layer schema, profile, and digest gates** — accepted.

## Consequences

- Operators can poll an accepted run on the same loopback listener that created it.
- Scientific-acceptance bytes appear only after a succeeded, profile-matched, digest-bound terminal status.
- The typed terminal-result DTO may later nest the same object without changing these HTTP gates.

## Failure and recovery

Unknown run identities, extra path segments, truncated percent-encoding, non-empty GET bodies, metric keys on receipts, failed-plus-artifact emission, all-zero digests, and digest mismatch return a redacted `400` envelope. Credential headers remain `403`. The in-memory registry is not durable; a restart requires re-POSTing the original metric-free request. Callers must not fabricate a succeeded GET.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- GET remains loopback-only, size-bounded, and content-redacting.
- SHA-256 digest agreement is a byte-identity check, not a validity claim.
- HTTP `200` on a succeeded scientific-acceptance GET is not release evidence.

## Compatibility and migration

The existing POST analysis-run, temporal-context, and project-history paths are unchanged. The client GET builder already targets `/v1/analysis-runs/{run_id}`. Production adapters may replace loopback while preserving metric-free receipts and the succeeded-only scientific-acceptance rule.

## Verification

Falsifiable evidence:

- POST accepted JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance keys;
- GET accepted and GET running stay metric-free;
- GET succeeded with profile `scientific_acceptance_v1` returns `tepp.scientific_acceptance.v1` only when the artifact digest matches;
- GET failed with an artifact, all-zero digest, digest mismatch, GET body, unknown run, and consumer mismatch fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes GET dispatch and the in-memory status index; POST receipts remain valid. A superseding ADR is required to persist status, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0022 owns deterministic execution to a digest-bound terminal result.
- ADR 0014 owns scientific claim promotion.
- ADR 0008 owns SHA-256 identity.
- ADR 0011 owns standalone/modular HTTP boundaries.
