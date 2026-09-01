# ADR 0079 — Loopback project-history cancel HTTP

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0066/0028 for operator-visible cancel.
Does not supersede ADR 0014 claim-promotion authority. This ADR number is
unique versus protected main; live vs-main and sibling GAP-003A PRs already
occupy 0026–0078.

## Context

ADR 0028 and ADR 0066 mint and retrieve accepted project-history identities
on `AnalysisRunLiveService`. Operators had no loopback POST that removes one
identity without guessing a second collection GET. Duplicating
project-history POST CLI (#420), collection GET (#424), collection CLI
(#428), GET-by-id (#429), retrieval CLI (#431), export cancel HTTP (#445),
interpretation-run cancel HTTP (#440), analysis-run cancel (#361), Leiden,
Driver p.16, or GAP-010 Figma/export would collide with live PRs. Naruon is
refused on this LineageWeave-owned adapter; `NaruonLiveService` stays
POST-only.

## Decision

Publish `POST /v1/project-histories/{idempotency_key}/cancel` on
`AnalysisRunLiveService` / `tepp-loopback`:

- Empty body is admitted. Nonempty leftover body fails closed.
- Public bind, unpublished consumer, present `idempotency-key`, extra path
  segments, slash/NUL identities, and credential headers fail closed.
- Receipt is metric-free with `inference_status=temporal_association_only`
  and `cancelled=true`. Evidence text, findings, RMSE, bias, coverage,
  SE-gate, causal scores, and `tepp.scientific_acceptance.v1` never appear.
- Cancelled identities drop from collection GET and GET-by-id.

## Alternatives considered

1. **Reuse analysis-run cancel (#361)** — rejected; different resource.
2. **Reuse export cancel (#445)** — rejected; naruon-owned export adapter.
3. **Reuse interpretation-run cancel (#440)** — rejected; orchestrator-owned.
4. **Add GET to `NaruonLiveService`** — rejected; POST-only.
5. **Published loopback cancel POST** — accepted.

## Consequences

- Operators can retract an accepted project-history identity without
  persistence.
- Cancel JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Cancel success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Missing identity, second cancel, naruon, nonempty body, present
`idempotency-key`, extra segments, public bind, and metric keys fail closed.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Evidence text, findings, and actor lists stay off the cancel receipt.
- HTTP 200 on cancel is not measurement evidence and is not a causal claim.

## Compatibility and migration

GET-by-id, collection GET, POST `/v1/project-histories`, and
`NaruonLiveService` POST-only remain unchanged. Persistence remains
GAP-003B.

## Verification

Falsifiable evidence:

- cancel of an accepted project-history returns metric-free `cancelled=true`
  without RMSE/bias/coverage/SE-gate/evidence/findings/causal-score/
  `tepp.scientific_acceptance.v1`;
- subsequent GET-by-id and collection GET omit the identity;
- naruon, nonempty body, present `idempotency-key`, extra segments, public
  bind, and unknown keys fail closed;
- `NaruonLiveService` still refuses GET and refuses this cancel path as a
  200;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the cancel route; collection and retrieval remain valid. A
superseding ADR is required to persist cancel, bind a public address, emit
scientific-acceptance on cancel, open naruon on this adapter, add GET to
`NaruonLiveService`, or treat cancel success as an ADR 0014 claim.

## Related authority

- ADR 0066 owns loopback project-history GET-by-id.
- ADR 0028 owns loopback project-history collection GET.
- ADR 0077 owns loopback export cancel HTTP (live #445).
- ADR 0073 owns interpretation-run cancel HTTP (live #440).
- ADR 0029 owns analysis-run cancel HTTP (live #361).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
