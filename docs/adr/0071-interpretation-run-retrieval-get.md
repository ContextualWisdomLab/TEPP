# ADR 0071 — Contextual-orchestrator interpretation-run GET-by-id

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0069 for retrieving one accepted identity. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on this interpretation stack versus protected main; live vs-main and sibling GAP-003A PRs already occupy 0026–0070.

## Context

ADR 0069 enumerates accepted hypothetical interpretation runs as metric-free
identities, and #436 publishes a collection CLI. Operators who hold an
`idempotency_key` from that page still had to replay
`POST /v1/interpretation-runs` to recover the stored identity. Duplicating
interpretation-run CLI (#425), collection GET (#433), collection CLI (#436),
project-history GET-by-id (#429), retrieval CLI (#431), analysis-run GET-by-id
(#359), Leiden, Driver p.16, or GAP-010 Figma/export would collide with live
PRs. Naruon and `LineageWeave` are refused on this orchestrator-owned adapter;
`NaruonLiveService` stays POST-only.

## Decision

`orchestrator_live` publishes loopback-only
`GET /v1/interpretation-runs/{idempotency_key}` on `tepp-orchestrator-loopback`:

- Consumer is `contextual-orchestrator` only. Empty body. The identity travels
  in the path. `idempotency-key` and collection pagination headers are refused.
- Extra extra-segments, slash, NUL, and oversized identities fail closed.
- The response is the stored metric-free identity: `interpretation_run_id`,
  `idempotency_key`, `orchestration_mode`, `claim_status=hypothetical`,
  `scientific_authority=false`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate,
  `evidence_span_ids`, `tenant_workspace_id`, `compute_budget_tokens`,
  `findings`, and `causal_score` never appear.
- Collection GET (`GET /v1/interpretation-runs` with no extra segment) is
  unchanged.
- The retrieval does not infer causality, mutate TEPP state, or return a
  completed psychometric result.
- This slice does not implement a retrieval CLI or persistence.

## Alternatives considered

1. **Keep POST replay as the only retrieval path** — rejected because operators
   still resubmit evidence after ADR 0069.
2. **Return the full accepted POST body** — rejected; GET-by-id stays a
   metric-free identity without tenant, budget, or evidence spans.
3. **Reuse analysis-run GET-by-id (#359) or project-history GET-by-id (#429)** —
   rejected; those are different live resources.
4. **Loopback `GET /v1/interpretation-runs/{idempotency_key}`** — accepted.

## Consequences

- Operators can retrieve one accepted hypothetical identity from a collection
  key without writing a second POST.
- Retrieval JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Retrieval success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`contextual-orchestrator` consumers, nonempty GET bodies, present
`idempotency-key`, pagination headers, extra path segments, slash/NUL keys,
credential flags, missing identities, and metric keys fail closed. The
in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Evidence spans, tenant, and budget stay off the retrieval body.
- HTTP 200 on GET-by-id is not measurement evidence and is not a causal claim.

## Compatibility and migration

Collection GET, POST `/v1/interpretation-runs`, and
`tepp-interpretation-runs create`/`list` remain unchanged. A retrieval CLI
remains a later slice.

## Verification

Falsifiable evidence:

- GET of an accepted identity returns `hypothetical` /
  `scientific_authority=false` without RMSE/bias/coverage/SE-gate/
  evidence/`causal_score`/`tepp.scientific_acceptance.v1` keys;
- missing identity, extra segments, naruon or LineageWeave, nonempty body,
  pagination headers, and unknown keys fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes GET-by-id; collection GET and POST remain valid. A
superseding ADR is required to persist the registry, bind a public address,
emit scientific-acceptance on retrieval, open naruon or LineageWeave, or treat
retrieval success as an ADR 0014 claim.

## Related authority

- ADR 0069 owns loopback interpretation-run collection GET.
- ADR 0070 owns the collection CLI (live #436).
- ADR 0064 owns the interpretation-run POST CLI (live #425).
- ADR 0010 owns orchestration mode vocabulary and scientific-authority
  separation.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
