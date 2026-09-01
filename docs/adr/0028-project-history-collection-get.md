# ADR 0028 — LineageWeave project-history collection GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0021 and ADR 0011 for the operator-visible project-history collection. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on protected main; other live PRs may reuse 0028 on unrelated GAP-003A stacks (lifecycle POST).

## Context

Protected main already stores accepted project-history projections on `AnalysisRunLiveService` after `POST /v1/project-histories`, and #420 publishes a POST CLI. Operators still cannot enumerate stored projections without guessing idempotency keys. Duplicating analysis-run collection GET (#368), GET-by-id (#359), project-history CLI (#420), temporal-context CLI (#414), export CLI (#410), export-retrieval GET (#411), Leiden, Driver p.16, or GAP-010 Figma/export would collide with live PRs.

## Decision

`tepp_api` publishes loopback-only `GET /v1/project-histories` on `tepp-loopback`:

- Consumer is `lineageweave` only. The required `tepp-tenant-workspace-id` scopes every page to one validated tenant. Empty body. Pagination uses `tepp-page-limit` and exclusive `tepp-page-cursor` headers because the request-line parser fails closed on query strings.
- Collection rows are metric-free identities: `project_key`, `idempotency_key`, `knowledge_cutoff`, `inference_status=temporal_association_only`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, `evidence_text`, `findings`, and `causal_score` never appear.
- The collection does not infer causality, mutate TEPP state, or return a completed psychometric result.
- GET `/v1/analysis-runs` and GET `/v1/temporal-context` stay fail-closed on this slice.
- This slice does not implement project-history collection CLI, GET-by-id, or persistence.

## Alternatives considered

1. **Keep POST replay as the only retrieval path** — rejected because operators still guess idempotency keys.
2. **Reuse analysis-run collection GET (#368)** — rejected; that slice is a different live PR and a different resource.
3. **Return evidence text and findings on the list** — rejected because collection bodies must stay metric-free and identity-opaque.
4. **Loopback `GET /v1/project-histories`** — accepted.

## Consequences

- Operators can enumerate accepted project-history projections without writing a second POST.
- Collection stdout cannot be mistaken for a succeeded scientific-acceptance result or a causal score.
- Collection success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`lineageweave` consumers, nonempty GET bodies, zero/oversized page limits, empty cursors, credential flags, and metric keys fail closed. The in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Evidence text and findings stay off the collection page.
- Process 200 on collection GET is not measurement evidence and is not a causal claim.

## Compatibility and migration

`POST /v1/project-histories`, `POST /v1/temporal-context`, `POST /v1/analysis-runs`, and `tepp-loopback` POST paths are unchanged. Project-history collection CLI remains a later slice.

## Verification

Falsifiable evidence:

- GET of two accepted projections returns a metric-free page sorted by idempotency key with `temporal_association_only` and no RMSE/bias/coverage/SE-gate/`tepp.scientific_acceptance.v1`/`evidence_text`/`findings`/`causal_score` keys;
- GET `/v1/analysis-runs`, naruon consumer, nonempty body, and metric keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes collection GET; POST `/v1/project-histories` remains valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on the list, infer causality, or treat collection success as an ADR 0014 claim.

## Related authority

- ADR 0021 owns the LineageWeave project-history service boundary.
- ADR 0002 owns six-clock temporal eligibility.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
