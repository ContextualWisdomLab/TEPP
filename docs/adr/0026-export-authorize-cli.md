# ADR 0026 — Purpose-bound export-authorize loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0009 and ADR 0011 for the operator-visible export client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on protected main; other live PRs may reuse 0026 on unrelated GAP-003A stacks.

## Context

Protected main already serves `POST /v1/exports` on `NaruonLiveService`, but operators still had to write raw HTTP/1.1. `tepp-loopback` is `AnalysisRunLiveService` and does not serve `/v1/exports`. Duplicating analysis-run CLIs (#362/#371/#378/#385/#392/#394/#395/#397/#400/#401/#403/#406), GET-by-id, Leiden, Driver p.16, or GAP-010 Figma/export would collide with live PRs.

## Decision

`tepp_api` publishes a loopback-only export client and the naruon live listener it targets:

- `tepp-naruon-live` binds `NaruonLiveService` on `127.0.0.1:18082` by default. It is not `tepp-loopback`.
- `tepp-exports authorize` POSTs `/v1/exports` with `--host` and `--idempotency-key`. Stdin is `ExportAuthorizationRequest` JSON.
- Only `modular_service_consumer` is accepted. Other purposes fail closed before the wire.
- The idempotency key must not equal `principal_id`.
- Stdout is the purpose-bound decision JSON. `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, and SE-gate keys never appear.
- Non-loopback hosts, credential-shaped flags, unknown verbs, empty stdin, and metric keys fail closed.
- This slice does not implement export retrieval `GET /v1/exports/{export_id}`, analysis-run HTTP, or persistence.

## Alternatives considered

1. **Keep raw HTTP as the only export path** — rejected because operators still guess framing after ADR 0011.
2. **Add export onto `tepp-loopback`** — rejected because that binary is `AnalysisRunLiveService` and must not pretend to serve `/v1/exports`.
3. **Return scientific-acceptance on allowed export** — rejected because export bodies must stay purpose-bound and metric-free.
4. **Loopback export CLI against `NaruonLiveService`** — accepted.

## Consequences

- Operators can authorize a purpose-bound export without writing HTTP.
- Export stdout cannot be mistaken for a succeeded scientific-acceptance result.
- CLI success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, empty stdin, credential flags, non-modular purposes, and an idempotency key equal to `principal_id` fail closed. Denied source-text purposes remain refused by ADR 0009. The in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only. Free-text source bodies stay purpose-gated (ADR 0009).
- Process exit 0 on authorize is not measurement evidence.

## Compatibility and migration

`POST /v1/analysis-runs`, `tepp-loopback`, temporal-context, and project-history paths are unchanged. `GET /v1/exports/{export_id}` remains a later slice.

## Verification

Falsifiable evidence:

- CLI authorize of a modular export returns `allowed`/`purpose_bound_export_allowed` with no RMSE/bias/coverage/SE-gate/`tepp.scientific_acceptance.v1` keys;
- operational-monitoring purpose, non-loopback host, credential flags, empty stdin, and unknown verbs fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes `tepp-exports` and `tepp-naruon-live`; `NaruonLiveService` HTTP remains valid. A superseding ADR is required to persist exports, bind a public address, emit scientific-acceptance on export, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0009 owns purpose-bound PII governance.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
