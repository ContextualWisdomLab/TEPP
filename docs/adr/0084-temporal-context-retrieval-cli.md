# ADR 0084 — Loopback temporal-context retrieval CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0083 for operator-visible GET-by-id.
Does not re-open collection GET/CLI (#449/#450) or cancel lineages. Does not
supersede ADR 0014. Unique versus protected main; 0026–0083 occupied.

## Context

ADR 0083 retrieves one accepted temporal-context identity on
`AnalysisRunLiveService`. Operators still had no published binary that mints
that GET onto spawned `tepp-loopback` TCP. Duplicating GET-by-id HTTP (#451),
temporal-context CLI (#414), project-history retrieval CLI (#431), export
retrieval CLI (#417), interpretation-run retrieval CLI (#439), Leiden, Driver
p.16, or GAP-010 Figma/export would collide with live PRs. Naruon is refused;
`NaruonLiveService` stays POST-only.

## Decision

Publish `tepp-temporal-context-get get`:

- Pattern: `from_args` + typed `lineageweave_temporal_context_retrieval_exchange`
  + `loopback_http1_from_temporal_context_retrieval_exchange` +
  `dispatch`/`execute`/`render` + published `[[bin]]`.
- Empty stdin is admitted. Nonempty leftover stdin fails closed.
- Public bind, `localhost` host, `http` origin, unpublished consumer, slash/NUL
  identities, and credential flags fail closed.
- Identity travels as `--idempotency-key`. Present `idempotency-key` HTTP
  header on the minted GET fails closed.
- Stdout is one metric-free identity with
  `inference_status=temporal_association_only`. Event labels, actor lists,
  timeline events, evidence text, findings, RMSE, and
  `tepp.scientific_acceptance.v1` never appear.
- Dedicated binary so it does not collide with `tepp-temporal-context` (#414).

## Alternatives considered

1. **Reuse `tepp-temporal-context`** — rejected; that CLI is POST.
2. **Add GET to `NaruonLiveService`** — rejected; POST-only.
3. **Published `tepp-temporal-context-get get`** — accepted.

## Consequences

- Operators can retrieve one accepted identity without a second GET-by-id PR.
- HTTP 200 is not measurement evidence and is not a causal claim.

## Failure and recovery

Non-LineageWeave consumers, nonempty leftover stdin, slash/NUL identities,
credential flags, public bind, and metric keys fail closed. TCP execute does
not fall back to an empty in-process listener.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers. Event labels and actor lists stay off stdout.
- HTTP 200 is not an ADR 0014 claim.

## Compatibility and migration

GET-by-id HTTP, POST `/v1/temporal-context`, and `NaruonLiveService` POST-only
remain unchanged. Collection GET stays closed. Persistence remains GAP-003B.

## Verification

- `tepp-temporal-context-get get` of an accepted identity returns a metric-free
  row without RMSE/event-label/actor/`tepp.scientific_acceptance.v1`;
- naruon, nonempty leftover stdin, `localhost`, `http` origin, public bind,
  slash/NUL, and unknown keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the published binary; GET-by-id HTTP remains valid. A
superseding ADR is required to persist the registry, bind a public address,
re-open collection GET, emit scientific-acceptance, open naruon, add GET to
`NaruonLiveService`, or treat retrieval success as an ADR 0014 claim.

## Related authority

- ADR 0083 owns loopback temporal-context GET-by-id.
- ADR 0027 owns the temporal-context CLI (live #414).
- ADR 0067 owns project-history retrieval CLI (live #431).
- ADR 0072 owns interpretation-run retrieval CLI (live #439).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022).
