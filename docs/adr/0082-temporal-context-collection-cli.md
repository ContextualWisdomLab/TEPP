# ADR 0082 — Loopback temporal-context collection CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0081 for operator-visible collection GET.
Does not supersede ADR 0014 claim-promotion authority. This ADR number is
unique versus protected main; live vs-main and sibling GAP-003A PRs already
occupy 0026–0081.

## Context

ADR 0081 enumerates accepted temporal-context identities on
`AnalysisRunLiveService`. Operators still had no published binary that mints
that GET onto spawned `tepp-loopback` TCP. Duplicating temporal-context
collection GET (#449), temporal-context CLI (#414), project-history collection
CLI (#428), export collection CLI (#444), interpretation-run collection CLI
(#436), Leiden, Driver p.16, or GAP-010 Figma/export would collide with live
PRs. Naruon is refused on this LineageWeave-owned adapter;
`NaruonLiveService` stays POST-only.

## Decision

Publish `tepp-temporal-contexts list`:

- Pattern: `from_args` + typed `lineageweave_temporal_context_collection_exchange`
  + `loopback_http1_from_temporal_context_collection_exchange` +
  `dispatch`/`execute`/`render` + published `[[bin]]`.
- Empty stdin is admitted. Nonempty leftover stdin fails closed.
- Public bind, `localhost` host, `http` origin, unpublished consumer, and
  credential flags fail closed.
- Stdout is one metric-free collection page with
  `inference_status=temporal_association_only`. Event labels, actor lists,
  timeline events, evidence text, findings, RMSE, bias, coverage, SE-gate,
  causal scores, and `tepp.scientific_acceptance.v1` never appear.
- Dedicated binary so it does not collide with `tepp-temporal-context` (#414).

## Alternatives considered

1. **Reuse `tepp-temporal-context`** — rejected; that CLI is POST.
2. **Add GET to `NaruonLiveService`** — rejected; POST-only.
3. **Published `tepp-temporal-contexts list`** — accepted.

## Consequences

- Operators can list accepted temporal-context identities without a second
  collection GET PR.
- Collection JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Collection success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-LineageWeave consumers, nonempty leftover stdin, present
`idempotency-key` as an HTTP header, credential flags, public bind, and metric
keys fail closed. TCP execute does not fall back to an empty in-process
listener.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Event labels, actor lists, and evidence stay off the collection receipt.
- HTTP 200 on collection is not measurement evidence and is not a causal claim.

## Compatibility and migration

Collection GET, POST `/v1/temporal-context`, and `NaruonLiveService` POST-only
remain unchanged. Persistence remains GAP-003B.

## Verification

Falsifiable evidence:

- `tepp-temporal-contexts list` of accepted identities returns metric-free
  rows without RMSE/bias/coverage/SE-gate/event-label/actor/evidence/findings/
  causal-score/`tepp.scientific_acceptance.v1` keys;
- naruon, nonempty leftover stdin, `localhost`, `http` origin, public bind,
  and unknown keys fail closed;
- `NaruonLiveService` still refuses GET;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the published binary; collection GET remains valid. A
superseding ADR is required to persist the registry, bind a public address,
emit scientific-acceptance on collection, open naruon on this adapter, add GET
to `NaruonLiveService`, or treat collection success as an ADR 0014 claim.

## Related authority

- ADR 0081 owns loopback temporal-context collection GET.
- ADR 0027 owns the temporal-context CLI (live #414).
- ADR 0076 owns the export collection CLI (live #444).
- ADR 0070 owns interpretation-run collection CLI (live #436).
- ADR 0065 owns project-history collection CLI (live #428).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
