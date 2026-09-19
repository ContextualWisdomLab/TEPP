# ADR 0076 — Loopback export collection CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0075 for operator-visible collection GET.
Does not supersede ADR 0014 claim-promotion authority. This ADR number is
unique versus protected main; live vs-main and sibling GAP-003A PRs already
occupy 0026–0075.

## Context

ADR 0075 enumerates authorized export identities on
`AnalysisRunLiveService`. Operators still had no published binary that mints
that GET onto spawned `tepp-loopback` TCP. Duplicating collection GET (#443),
export-retrieval CLI (#417), export retrieval GET (#411), export-authorize CLI
(#410), interpretation-run collection CLI (#436), Leiden, Driver p.16, or
GAP-010 Figma/export would collide with live PRs. LineageWeave is refused on
this naruon-owned adapter; `NaruonLiveService` stays POST-only.

## Decision

Publish `tepp-export-list list`:

- Pattern: `from_args` + typed `naruon_export_collection_exchange` +
  `loopback_http1_from_export_collection_exchange` +
  `dispatch`/`execute`/`render` + published `[[bin]]`.
- Empty stdin is admitted. Nonempty leftover stdin fails closed.
- Public bind, `localhost` host, `http` origin, unpublished consumer, and
  credential flags fail closed.
- Stdout is one metric-free collection page. Tenant, principal, source text,
  RMSE, bias, coverage, SE-gate, and `tepp.scientific_acceptance.v1` never
  appear.
- Dedicated binary so it does not collide with `tepp-export-get` (#417) or
  `tepp-exports` (#410).

## Alternatives considered

1. **Reuse `tepp-export-get get`** — rejected; that CLI is GET-by-id (#417).
2. **Reuse `tepp-exports authorize`** — rejected; that CLI is POST (#410).
3. **Add GET collection to `NaruonLiveService`** — rejected; POST-only.
4. **Published `tepp-export-list list`** — accepted.

## Consequences

- Operators can enumerate authorized export identities without guessing
  `export_id` and without a second collection GET PR.
- Collection JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Collection success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`naruon` consumers, nonempty leftover stdin, present `idempotency-key`,
extra path segments, slash/NUL cursors, credential flags, public bind, and
metric keys fail closed. TCP execute does not fall back to an empty
in-process listener.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Tenant, principal, and source text stay off the collection page.
- HTTP 200 on collection is not measurement evidence and is not a causal
  claim.

## Compatibility and migration

GET-by-id, collection GET, POST `/v1/exports`, and `NaruonLiveService`
POST-only remain unchanged. Persistence remains GAP-003B.

## Verification

Falsifiable evidence:

- `tepp-export-list list` of authorized exports returns metric-free identities
  without RMSE/bias/coverage/SE-gate/tenant/principal/source-text/
  `tepp.scientific_acceptance.v1` keys;
- LineageWeave, nonempty leftover stdin, present `idempotency-key`, extra
  segments, public bind, `localhost`, `http` origin, and unknown keys fail
  closed;
- `NaruonLiveService` still refuses GET;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the published binary; collection GET remains valid. A
superseding ADR is required to persist the collection, bind a public address,
emit scientific-acceptance on collection, open LineageWeave, add GET to
`NaruonLiveService`, or treat collection success as an ADR 0014 claim.

## Related authority

- ADR 0075 owns loopback export collection GET.
- ADR 0054 owns loopback export retrieval GET.
- ADR 0055 owns the export-retrieval CLI (live #417).
- ADR 0026 owns the export-authorize CLI (live #410).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
