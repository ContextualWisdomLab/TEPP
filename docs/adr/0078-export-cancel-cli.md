# ADR 0078 — Loopback export cancel CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0077 for operator-visible cancel POST.
Does not supersede ADR 0014 claim-promotion authority. This ADR number is
unique versus protected main; live vs-main and sibling GAP-003A PRs already
occupy 0026–0077.

## Context

ADR 0077 removes an authorized export identity on
`AnalysisRunLiveService`. Operators still had no published binary that mints
that POST onto spawned `tepp-loopback` TCP. Duplicating export cancel HTTP
(#445), export collection CLI (#444), export collection GET (#443),
export-retrieval CLI (#417), export retrieval GET (#411), export-authorize
CLI (#410), interpretation-run cancel CLI (#442), interpretation-run cancel
HTTP (#440), analysis-run cancel (#361), Leiden, Driver p.16, or GAP-010
Figma/export would collide with live PRs. LineageWeave is refused on this
naruon-owned adapter; `NaruonLiveService` stays POST-only.

## Decision

Publish `tepp-export-cancel cancel`:

- Pattern: `from_args` + typed `naruon_export_cancel_exchange` +
  `loopback_http1_from_export_cancel_exchange` +
  `dispatch`/`execute`/`render` + published `[[bin]]`.
- Empty stdin is admitted. Nonempty leftover stdin fails closed.
- Public bind, `localhost` host, `http` origin, unpublished consumer, and
  credential flags fail closed.
- Stdout is one metric-free cancelled identity with `cancelled=true`.
  Tenant, principal, source text, RMSE, bias, coverage, SE-gate, and
  `tepp.scientific_acceptance.v1` never appear.
- Dedicated binary so it does not collide with `tepp-export-list` (#444) or
  `tepp-export-get` (#417).

## Alternatives considered

1. **Reuse `tepp-export-list list`** — rejected; that CLI is collection GET.
2. **Reuse `tepp-export-get get`** — rejected; that CLI is GET-by-id (#417).
3. **Add GET to `NaruonLiveService`** — rejected; POST-only.
4. **Published `tepp-export-cancel cancel`** — accepted.

## Consequences

- Operators can retract an authorized export identity without a second
  cancel HTTP PR.
- Cancel JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Cancel success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`naruon` consumers, nonempty leftover stdin, present `idempotency-key`,
slash/NUL identities, credential flags, public bind, and metric keys fail
closed. TCP execute does not fall back to an empty in-process listener.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Tenant, principal, and source text stay off the cancel receipt.
- HTTP 200 on cancel is not measurement evidence and is not a causal claim.

## Compatibility and migration

Cancel HTTP, collection GET, GET-by-id, POST `/v1/exports`, and
`NaruonLiveService` POST-only remain unchanged. Persistence remains
GAP-003B.

## Verification

Falsifiable evidence:

- `tepp-export-cancel cancel` of an authorized export returns metric-free
  `cancelled=true` without RMSE/bias/coverage/SE-gate/tenant/principal/
  source-text/`tepp.scientific_acceptance.v1` keys;
- LineageWeave, nonempty leftover stdin, present `idempotency-key`, slash/NUL
  identities, public bind, `localhost`, `http` origin, and unknown keys fail
  closed;
- `NaruonLiveService` still refuses GET;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the published binary; cancel HTTP remains valid. A
superseding ADR is required to persist cancel, bind a public address, emit
scientific-acceptance on cancel, open LineageWeave, add GET to
`NaruonLiveService`, or treat cancel success as an ADR 0014 claim.

## Related authority

- ADR 0077 owns loopback export cancel HTTP.
- ADR 0076 owns the export collection CLI (live #444).
- ADR 0075 owns loopback export collection GET.
- ADR 0054 owns loopback export retrieval GET.
- ADR 0029 owns analysis-run cancel HTTP (live #361).
- ADR 0074 owns interpretation-run cancel CLI (live #442).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
