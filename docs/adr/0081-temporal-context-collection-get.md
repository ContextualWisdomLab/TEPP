# ADR 0081 — Loopback temporal-context collection GET

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements the LineageWeave `POST /v1/temporal-context`
read contract. Does not supersede ADR 0014 claim-promotion authority. This ADR
number is unique versus protected main; live vs-main and sibling GAP-003A PRs
already occupy 0026–0080.

## Context

`POST /v1/temporal-context` returns one cutoff-safe association page. Operators
still had no loopback GET that enumerates accepted identities without guessing
idempotency keys. Duplicating temporal-context CLI (#414), project-history
collection GET (#424), interpretation-run collection GET (#433), export
collection GET (#443), Leiden, Driver p.16, or GAP-010 Figma/export would
collide with live PRs. Naruon is refused on this LineageWeave-owned adapter;
`NaruonLiveService` stays POST-only.

## Decision

Publish `GET /v1/temporal-context` on `AnalysisRunLiveService` / `tepp-loopback`:

- Empty body. Present `idempotency-key` header fails closed.
- Public bind, unpublished consumer, and credential headers fail closed.
- Collection rows are metric-free identities with
  `inference_status=temporal_association_only`. Event labels, actor lists,
  timeline events, evidence text, findings, RMSE, bias, coverage, SE-gate,
  causal scores, and `tepp.scientific_acceptance.v1` never appear.
- POST remains a compute-and-return read. An optional `idempotency-key` header
  mints the identity into the in-memory collection; POST without that header
  stays backward compatible and is not listed.

## Alternatives considered

1. **Reuse `GET /v1/project-histories`** — rejected; that collection is
   project-history owned.
2. **Add GET to `NaruonLiveService`** — rejected; POST-only.
3. **Loopback `GET /v1/temporal-context`** — accepted.

## Consequences

- Operators can list accepted temporal-context identities without a second
  POST replay.
- Collection JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Collection success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-LineageWeave consumers, nonempty GET bodies, present `idempotency-key` as
a GET header, slash/NUL identities, credential flags, public bind, and metric
keys fail closed.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Event labels, actor lists, and evidence stay off the collection receipt.
- HTTP 200 on collection is not measurement evidence and is not a causal claim.

## Compatibility and migration

POST `/v1/temporal-context` without an idempotency header remains valid.
`NaruonLiveService` POST-only remains unchanged. Persistence remains GAP-003B.

## Verification

Falsifiable evidence:

- `GET /v1/temporal-context` of accepted LineageWeave identities returns
  metric-free rows without RMSE/bias/coverage/SE-gate/event-label/actor/
  evidence/findings/causal-score/`tepp.scientific_acceptance.v1` keys;
- naruon, nonempty leftover body, GET `idempotency-key`, slash/NUL identities,
  public bind, and unknown keys fail closed;
- `NaruonLiveService` still refuses GET;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes collection GET; POST remains valid. A superseding ADR is
required to persist the registry, bind a public address, emit
scientific-acceptance on collection, open naruon on this adapter, add GET to
`NaruonLiveService`, or treat collection success as an ADR 0014 claim.

## Related authority

- ADR 0002 owns six-clock temporal semantics.
- ADR 0027 owns the temporal-context CLI (live #414).
- ADR 0075 owns export collection GET (live #443).
- ADR 0069 owns interpretation-run collection GET (live #433).
- ADR 0028 owns project-history collection GET (live #424).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
