# ADR 0077 — Loopback export cancel HTTP

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0075/0054 for operator-visible cancel.
Does not supersede ADR 0014 claim-promotion authority. This ADR number is
unique versus protected main; live vs-main and sibling GAP-003A PRs already
occupy 0026–0076.

## Context

ADR 0054 and ADR 0075 mint and enumerate authorized export identities on
`AnalysisRunLiveService`. Operators had no loopback POST that removes one
identity without guessing a second collection GET. Duplicating analysis-run
cancel (#361), interpretation-run cancel HTTP (#440), interpretation-run
cancel CLI (#442), export collection GET (#443), export collection CLI
(#444), export-retrieval CLI (#417), export retrieval GET (#411),
export-authorize CLI (#410), Leiden, Driver p.16, or GAP-010 Figma/export
would collide with live PRs. LineageWeave is refused on this naruon-owned
adapter; `NaruonLiveService` stays POST-only.

## Decision

Publish `POST /v1/exports/{export_id}/cancel` on `AnalysisRunLiveService` /
`tepp-loopback`:

- Empty body is admitted. Nonempty leftover body fails closed.
- Public bind, unpublished consumer, present `idempotency-key`, extra path
  segments, slash/NUL identities, and credential headers fail closed.
- Receipt is metric-free with `cancelled=true`. Tenant, principal, source
  text, RMSE, bias, coverage, SE-gate, and `tepp.scientific_acceptance.v1`
  never appear.
- Cancelled identities drop from collection GET and GET-by-id.

## Alternatives considered

1. **Reuse analysis-run cancel (#361)** — rejected; different resource.
2. **Reuse interpretation-run cancel (#440)** — rejected; orchestrator-owned.
3. **Add cancel to `NaruonLiveService`** — rejected; POST-only for authorize.
4. **Published loopback cancel POST** — accepted.

## Consequences

- Operators can retract an authorized export identity without persistence.
- Cancel JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Cancel success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Missing identity, second cancel, LineageWeave, nonempty body, present
`idempotency-key`, extra segments, public bind, and metric keys fail closed.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Tenant, principal, and source text stay off the cancel receipt.
- HTTP 200 on cancel is not measurement evidence and is not a causal claim.

## Compatibility and migration

GET-by-id, collection GET, POST `/v1/exports`, and `NaruonLiveService`
POST-only remain unchanged. Persistence remains GAP-003B.

## Verification

Falsifiable evidence:

- cancel of an authorized export returns metric-free `cancelled=true`
  without RMSE/bias/coverage/SE-gate/tenant/principal/source-text/
  `tepp.scientific_acceptance.v1`;
- subsequent GET-by-id and collection GET omit the identity;
- LineageWeave, nonempty body, present `idempotency-key`, extra segments,
  public bind, and unknown keys fail closed;
- `NaruonLiveService` still refuses GET and refuses this cancel path as a
  200;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the cancel route; collection and retrieval remain valid. A
superseding ADR is required to persist cancel, bind a public address, emit
scientific-acceptance on cancel, open LineageWeave, add GET to
`NaruonLiveService`, or treat cancel success as an ADR 0014 claim.

## Related authority

- ADR 0075 owns loopback export collection GET.
- ADR 0076 owns the export collection CLI (live #444).
- ADR 0054 owns loopback export retrieval GET.
- ADR 0029 owns analysis-run cancel HTTP (live #361).
- ADR 0073 owns interpretation-run cancel HTTP (live #440).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
