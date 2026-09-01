# ADR 0072 — Contextual-orchestrator interpretation-run retrieval CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0071 for operator-visible GET-by-id. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on this interpretation stack versus protected main; live vs-main and sibling GAP-003A PRs already occupy 0026–0071.

## Context

ADR 0071 publishes loopback `GET /v1/interpretation-runs/{idempotency_key}`.
Operators still had no published binary that mints that typed exchange onto
spawned `tepp-orchestrator-loopback` TCP. Duplicating interpretation-run CLI
(#425), collection GET (#433), collection CLI (#436), GET-by-id HTTP (#438),
project-history retrieval CLI (#431), analysis-run GET-by-id (#359), Leiden,
Driver p.16, or GAP-010 Figma/export would collide with live PRs. Naruon and
`LineageWeave` are refused on this orchestrator-owned adapter;
`NaruonLiveService` stays POST-only.

## Decision

`orchestrator_live` publishes `tepp-interpretation-run-get get`:

- `from_args` plus typed
  `contextual_orchestrator_interpretation_run_retrieval_exchange`,
  `loopback_http1_from_interpretation_run_retrieval_exchange`,
  `dispatch`/`execute`/`render`, and a published `[[bin]]`.
- Consumer is `contextual-orchestrator` only. Empty stdin is admitted;
  leftover nonempty stdin fails closed.
- Public bind, `localhost` as a hostname, `http` origins, unpublished
  consumers, pagination flags, and credential-shaped flags fail closed.
- Stdout is one metric-free identity: `claim_status=hypothetical`,
  `scientific_authority=false`. `tepp.scientific_acceptance.v1` never appears.
- The CLI does not infer causality, mutate TEPP state, or return a completed
  psychometric result.
- This slice does not implement persistence.

## Alternatives considered

1. **Keep HTTP GET-by-id without a CLI** — rejected; operators still write raw
   HTTP after ADR 0071.
2. **Add `get` onto `tepp-interpretation-runs`** — rejected for this slice;
   collection CLI (`list`) lives on a parallel stack (#436). A dedicated
   retrieval binary avoids colliding with that live PR.
3. **Reuse project-history retrieval CLI (#431)** — rejected; that is a
   different live resource.
4. **Published `tepp-interpretation-run-get get`** — accepted.

## Consequences

- Operators can retrieve one accepted hypothetical identity from a collection
  key without writing a second POST or crafting HTTP by hand.
- Retrieval stdout cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Retrieval success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`contextual-orchestrator` consumers, nonempty leftover stdin, present
pagination flags, public bind, `localhost`, `http` origins, credential flags,
slash/NUL keys, and metric keys fail closed. The in-memory listener is not
durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Evidence spans, tenant, and budget stay off the retrieval stdout.
- Process 0 / HTTP 200 on GET-by-id is not measurement evidence and is not a
  causal claim.

## Compatibility and migration

Collection GET, GET-by-id HTTP, POST `/v1/interpretation-runs`, and
`tepp-interpretation-runs create` remain unchanged. Persistence remains a
later slice.

## Verification

Falsifiable evidence:

- `tepp-interpretation-run-get get` of an accepted identity returns
  `hypothetical` / `scientific_authority=false` without RMSE/bias/coverage/
  SE-gate/evidence/`causal_score`/`tepp.scientific_acceptance.v1` keys;
- leftover stdin, naruon or LineageWeave, public bind, `localhost`, `http`
  origin, pagination flags, and credential flags fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes `tepp-interpretation-run-get`; GET-by-id HTTP remains valid.
A superseding ADR is required to persist the registry, bind a public address,
emit scientific-acceptance on retrieval, open naruon or LineageWeave, or treat
retrieval success as an ADR 0014 claim.

## Related authority

- ADR 0071 owns loopback interpretation-run GET-by-id.
- ADR 0070 owns the collection CLI (live #436).
- ADR 0069 owns loopback interpretation-run collection GET.
- ADR 0064 owns the interpretation-run POST CLI (live #425).
- ADR 0010 owns orchestration mode vocabulary and scientific-authority
  separation.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
