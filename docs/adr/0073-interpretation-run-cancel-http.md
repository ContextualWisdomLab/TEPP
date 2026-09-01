# ADR 0073 — Contextual-orchestrator interpretation-run cancel HTTP

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0071 for removing one accepted identity. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on this interpretation stack versus protected main; live vs-main and sibling GAP-003A PRs already occupy 0026–0072.

## Context

ADR 0071 retrieves one accepted hypothetical interpretation-run identity, and
#439 publishes a retrieval CLI. Operators who held an `idempotency_key` still
had no loopback path to drop that in-memory identity without restarting the
listener. Duplicating interpretation-run CLI (#425), collection GET (#433),
collection CLI (#436), GET-by-id HTTP (#438), retrieval CLI (#439),
analysis-run cancel HTTP (#361), Leiden, Driver p.16, or GAP-010 Figma/export
would collide with live PRs. Naruon and `LineageWeave` are refused on this
orchestrator-owned adapter; `NaruonLiveService` stays POST-only for
analysis-run and export.

## Decision

`orchestrator_live` publishes loopback-only
`POST /v1/interpretation-runs/{idempotency_key}/cancel` on
`tepp-orchestrator-loopback`:

- Consumer is `contextual-orchestrator` only. Empty body. Identity travels in
  the path. `idempotency-key` and collection pagination headers are refused.
- Extra extra-segments, slash, NUL, and oversized identities fail closed.
- A successful cancel removes the identity from the in-memory registry and
  returns a metric-free receipt: `interpretation_run_id`, `idempotency_key`,
  `orchestration_mode`, `claim_status=hypothetical`,
  `scientific_authority=false`, `cancelled=true`.
- Subsequent GET-by-id and collection rows for that key fail closed as
  missing. A second cancel of the same key fails closed.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate,
  `evidence_span_ids`, `findings`, and `causal_score` never appear.
- Cancel does not infer causality, persist, or return a completed
  psychometric result.
- This slice does not implement a cancel CLI.

## Alternatives considered

1. **Restart the listener to drop identities** — rejected; operators still
   cannot target one key after ADR 0071.
2. **Reuse analysis-run cancel HTTP (#361)** — rejected; that is a different
   live resource and a naruon consumer.
3. **Keep identities until process exit** — rejected; the in-memory registry
   would retain hypothetic identities with no operator-visible drop path.
4. **Loopback `POST /v1/interpretation-runs/{idempotency_key}/cancel`** —
   accepted.

## Consequences

- Operators can drop one accepted hypothetical identity without restarting
  `tepp-orchestrator-loopback`.
- Cancel JSON cannot be mistaken for a succeeded scientific-acceptance result
  or a causal score.
- Cancel success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`contextual-orchestrator` consumers, nonempty POST bodies, present
`idempotency-key`, pagination headers, extra path segments, slash/NUL keys,
credential flags, missing identities, and metric keys fail closed. The
in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Evidence spans, tenant, and budget stay off the cancel receipt.
- HTTP 200 on cancel is not measurement evidence and is not a causal claim.

## Compatibility and migration

Collection GET, GET-by-id, POST `/v1/interpretation-runs`, and
`tepp-interpretation-runs create` remain unchanged. A cancel CLI remains a
later slice. Persistence remains GAP-003B.

## Verification

Falsifiable evidence:

- POST cancel of an accepted identity returns `hypothetical` /
  `scientific_authority=false` / `cancelled=true` without RMSE/bias/coverage/
  SE-gate/evidence/`causal_score`/`tepp.scientific_acceptance.v1` keys;
- subsequent GET-by-id, a second cancel, naruon or LineageWeave, nonempty
  body, and unknown keys fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes cancel HTTP; collection GET, GET-by-id, and POST remain
valid. A superseding ADR is required to persist cancellations, bind a public
address, emit scientific-acceptance on cancel, open naruon or LineageWeave,
or treat cancel success as an ADR 0014 claim.

## Related authority

- ADR 0071 owns loopback interpretation-run GET-by-id.
- ADR 0072 owns the retrieval CLI (live #439).
- ADR 0069 owns loopback interpretation-run collection GET.
- ADR 0064 owns the interpretation-run POST CLI (live #425).
- ADR 0029 owns analysis-run cancel HTTP (live #361) as a different resource.
- ADR 0010 owns orchestration mode vocabulary and scientific-authority
  separation.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
