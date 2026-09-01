# ADR 0069 — Contextual-orchestrator interpretation-run collection GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0010, ADR 0011, and ADR 0064 for the operator-visible interpretation-run collection. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on this stack versus protected main; live vs-main and sibling GAP-003A PRs already occupy 0026–0068.

## Context

Protected main already serves `POST /v1/interpretation-runs` on
`OrchestratorLiveService`, and #425 publishes `tepp-interpretation-runs create`.
Operators still cannot enumerate accepted hypothetical runs without guessing
idempotency keys. Duplicating interpretation-run CLI (#425), project-history
collection GET (#424), collection CLI (#428), GET-by-id (#429), retrieval CLI
(#431), analysis-run collection GET (#368), Leiden, Driver p.16, or GAP-010
Figma/export would collide with live PRs. Naruon and `LineageWeave` are refused
on this orchestrator-owned adapter; `NaruonLiveService` stays POST-only.

## Decision

`orchestrator_live` publishes loopback-only `GET /v1/interpretation-runs` on
`tepp-orchestrator-loopback`:

- Consumer is `contextual-orchestrator` only. Empty body. Pagination uses
  `tepp-page-limit` and exclusive `tepp-page-cursor` headers because the
  request-line parser fails closed on query strings.
- `idempotency-key` is refused on collection GET. Extra path segments
  (GET-by-id) fail closed on this slice.
- Collection rows are metric-free identities: `interpretation_run_id`,
  `idempotency_key`, `orchestration_mode`, `claim_status=hypothetical`,
  `scientific_authority=false`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate,
  `evidence_span_ids`, `tenant_workspace_id`, `compute_budget_tokens`,
  `evidence_text`, `findings`, and `causal_score` never appear.
- The collection does not infer causality, call a model provider, mutate TEPP
  state, or return a completed psychometric result.
- This slice does not implement interpretation-run collection CLI, GET-by-id,
  or persistence.

## Alternatives considered

1. **Keep POST replay as the only retrieval path** — rejected because
   operators still guess idempotency keys.
2. **Reuse analysis-run or project-history collection GET** — rejected; those
   slices are different live PRs and different resources.
3. **Return evidence spans, tenant, or budget on the list** — rejected because
   collection bodies must stay metric-free identities.
4. **Open naruon or LineageWeave on this adapter** — rejected; the listener
   admits `contextual-orchestrator` only.
5. **Loopback `GET /v1/interpretation-runs`** — accepted.

## Consequences

- Operators can enumerate accepted hypothetical interpretation runs without
  writing a second POST.
- Collection JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Collection success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`contextual-orchestrator` consumers, nonempty GET bodies, present
`idempotency-key`, extra path segments, zero/oversized page limits, empty or
slash/NUL cursors, credential flags, and metric keys fail closed. The
in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Evidence spans, tenant, and budget stay off the collection page.
- HTTP 200 on collection GET is not measurement evidence and is not a causal
  claim.

## Compatibility and migration

`POST /v1/interpretation-runs` and `tepp-interpretation-runs create` remain
unchanged. Interpretation-run collection CLI remains a later slice.

## Verification

Falsifiable evidence:

- GET of two accepted runs returns a metric-free page sorted by idempotency
  key with `claim_status=hypothetical`, `scientific_authority=false`, and no
  RMSE/bias/coverage/SE-gate/`tepp.scientific_acceptance.v1`/`evidence_span_ids`/
  `causal_score` keys;
- GET extra segments, naruon or LineageWeave consumer, nonempty body, present
  `idempotency-key`, and metric keys fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes collection GET; `POST /v1/interpretation-runs` remains valid.
A superseding ADR is required to persist the registry, bind a public address,
emit scientific-acceptance on the list, infer causality, open naruon or
`LineageWeave`, or treat collection success as an ADR 0014 claim.

## Related authority

- ADR 0010 owns adaptive LLM orchestration and scientific-authority
  separation.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0064 owns the interpretation-run create CLI.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does
  not authorize scientific claims.
