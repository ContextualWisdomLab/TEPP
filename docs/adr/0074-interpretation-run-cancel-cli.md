# ADR 0074 — Contextual-orchestrator interpretation-run cancel CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0073 for operator-visible cancel. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on this interpretation stack versus protected main; live vs-main and sibling GAP-003A PRs already occupy 0026–0073.

## Context

ADR 0073 publishes loopback
`POST /v1/interpretation-runs/{idempotency_key}/cancel`. Operators still had
no published binary that mints that typed exchange onto spawned
`tepp-orchestrator-loopback` TCP. Duplicating interpretation-run CLI (#425),
collection GET (#433), collection CLI (#436), GET-by-id HTTP (#438),
retrieval CLI (#439), cancel HTTP (#440), analysis-run cancel CLI (#378),
Leiden, Driver p.16, or GAP-010 Figma/export would collide with live PRs.
Naruon and `LineageWeave` are refused on this orchestrator-owned adapter;
`NaruonLiveService` stays POST-only for analysis-run and export.

## Decision

`orchestrator_live` publishes `tepp-interpretation-run-cancel cancel`:

- `from_args` plus typed
  `contextual_orchestrator_interpretation_run_cancel_exchange`,
  `loopback_http1_from_interpretation_run_cancel_exchange`,
  `dispatch`/`execute`/`render`, and a published `[[bin]]`.
- Consumer is `contextual-orchestrator` only. Empty stdin is admitted;
  leftover nonempty stdin fails closed.
- Public bind, `localhost` as a hostname, `http` origins, unpublished
  consumers, pagination flags, and credential-shaped flags fail closed.
- Stdout is one metric-free cancelled identity: `claim_status=hypothetical`,
  `scientific_authority=false`, `cancelled=true`.
- `tepp.scientific_acceptance.v1` never appears.
- The CLI does not infer causality, persist, or return a completed
  psychometric result.
- A dedicated binary avoids colliding with collection CLI `list` on #436 and
  retrieval CLI `get` on #439.

## Alternatives considered

1. **Keep HTTP cancel without a CLI** — rejected; operators still write raw
   HTTP after ADR 0073.
2. **Add `cancel` onto `tepp-interpretation-runs`** — rejected for this
   slice; collection CLI (`list`) lives on a parallel stack (#436).
3. **Reuse analysis-run cancel CLI (#378)** — rejected; that is a different
   live resource.
4. **Published `tepp-interpretation-run-cancel cancel`** — accepted.

## Consequences

- Operators can drop one accepted hypothetical identity from a collection
  key without writing HTTP by hand.
- Cancel stdout cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Cancel success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`contextual-orchestrator` consumers, nonempty leftover stdin, present
pagination flags, public bind, `localhost`, `http` origins, credential flags,
slash/NUL keys, missing identities, a second cancel, and metric keys fail
closed. The in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Evidence spans, tenant, and budget stay off the cancel stdout.
- Process 0 / HTTP 200 on cancel is not measurement evidence and is not a
  causal claim.

## Compatibility and migration

Cancel HTTP, GET-by-id, collection GET, POST `/v1/interpretation-runs`, and
`tepp-interpretation-runs create` remain unchanged. Persistence remains
GAP-003B.

## Verification

Falsifiable evidence:

- `tepp-interpretation-run-cancel cancel` of an accepted identity returns
  `hypothetical` / `scientific_authority=false` / `cancelled=true` without
  RMSE/bias/coverage/SE-gate/evidence/`causal_score`/`tepp.scientific_acceptance.v1`
  keys;
- leftover stdin, naruon or LineageWeave, public bind, `localhost`, `http`
  origin, pagination flags, credential flags, and a second cancel fail
  closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes `tepp-interpretation-run-cancel`; cancel HTTP remains
valid. A superseding ADR is required to persist cancellations, bind a public
address, emit scientific-acceptance on cancel, open naruon or LineageWeave,
or treat cancel success as an ADR 0014 claim.

## Related authority

- ADR 0073 owns loopback interpretation-run cancel HTTP.
- ADR 0072 owns the retrieval CLI (live #439).
- ADR 0071 owns loopback interpretation-run GET-by-id.
- ADR 0069 owns loopback interpretation-run collection GET.
- ADR 0064 owns the interpretation-run POST CLI (live #425).
- ADR 0010 owns orchestration mode vocabulary and scientific-authority
  separation.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
