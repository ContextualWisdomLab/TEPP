# ADR 0032 — Analysis-run retry HTTP path

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0018 for the operator-visible retry path. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0031 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel, loopback-CLI, and collection-GET slices.

## Context

`docs/API_CONTRACT.md` documents the lifecycle `failed → retryable`. Protected main and the live collection GET slice let operators list a failed or cancelled run, but `POST /v1/analysis-runs` with the original idempotency key returns the original failed or cancelled receipt. Operators therefore cannot start a new attempt without reconstructing the original body and inventing a new key. Returning RMSE, bias, coverage, SE-gate, or `tepp.scientific_acceptance.v1` on a retry body would treat a new attempt as measurement evidence. Stacking this slice onto GET-by-id, lifecycle POST, cancel, CLI, or collection GET would duplicate those heads.

## Decision

`AnalysisRunLiveService` serves `POST /v1/analysis-runs/{run_id}/retry` on loopback:

- Failed and cancelled runs clone the stored request into a new metric-free `202 Accepted` receipt with a new `run_id` and a **new** idempotency key.
- The new key comes from the `idempotency-key` header (and matching body field when a body is present). Reusing the parent's key fails closed.
- Already-accepted retry receipts with the same new key are idempotent: the same `202` child is returned.
- Accepted, running, succeeded, and unknown runs cannot be retried.
- Empty POST bodies are admitted and bind path `run_id` plus the new idempotency header. A typed `AnalysisRunRetryRequest` body must match path identity and header key.
- Retry bodies and accepted receipts refuse RMSE, bias, coverage, SE-gate, scientific-acceptance, and report keys.
- The parent remains failed or cancelled. Collection GET lists both parent and child.
- GET-by-id, lifecycle POST, cancel, and CLI remain other live slices. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable retry storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating GET `/v1/analysis-runs/{run_id}`, POST running/terminal, POST cancel, GET collection, or loopback CLI.

## Alternatives considered

1. **Ask operators to POST create with a reconstructed body** — rejected because collection GET already identified the failed run and reconstructing the original snapshot/cutoff/profile is not operator-visible.
2. **Reuse the parent's idempotency key** — rejected because create replay returns the original failed or cancelled receipt.
3. **Carry scientific-acceptance metrics on the retry receipt** — rejected because a new attempt is not measurement evidence until a later succeeded GET-by-id with profile `scientific_acceptance_v1`.
4. **Clone failed/cancelled runs into a new metric-free `202` with a distinct key** — accepted.

## Consequences

- Operators can start a new attempt from a listed failed or cancelled run on the same loopback listener.
- Retry receipts cannot be mistaken for a succeeded scientific-acceptance result.
- GET-by-id may later report the child without changing these retry gates.

## Failure and recovery

Unknown run identities, extra path segments, truncated percent-encoding, metric keys on retry bodies, accepted/running/succeeded retry, parent-key reuse, consumer mismatch, and path/header/body identity mismatch return a redacted `400` envelope. Oversized run identities return `413`. Credential headers remain `403`. The in-memory registry is not durable; a restart requires re-POSTing the original metric-free create request. Callers must not fabricate a succeeded run from a retry `202`.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Retry remains loopback-only, size-bounded, and content-redacting.
- HTTP `202` on a retried run is not measurement evidence and is not release evidence.

## Compatibility and migration

The existing POST analysis-run, cancel, collection GET, temporal-context, and project-history paths are unchanged. GET-by-id remains refused on this slice. Production adapters may replace loopback while preserving metric-free retry receipts and the accepted/running/succeeded retry refusal.

## Verification

Falsifiable evidence:

- POST retry of failed and cancelled returns metric-free `202 Accepted` with a new `run_id` and new idempotency key;
- POST retry of accepted, running, succeeded, and unknown runs fails closed;
- reusing the parent idempotency key fails closed;
- replaying the same new key is idempotent;
- metric keys, consumer mismatch, and identity mismatch fail closed;
- GET `/v1/analysis-runs/{run_id}` remains `400` on this slice;
- collection GET lists both the parent and the child;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes retry dispatch; POST create receipts, cancel, and collection GET remain valid. A superseding ADR is required to persist retry, bind a public address, retry succeeded runs, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0029 owns loopback cancel.
- ADR 0031 owns loopback collection GET.
- ADR 0022 owns deterministic execution to a digest-bound terminal result.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
