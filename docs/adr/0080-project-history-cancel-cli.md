# ADR 0080 — Loopback project-history cancel CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0079 for operator-visible cancel POST.
Does not supersede ADR 0014 claim-promotion authority. This ADR number is
unique versus protected main; live vs-main and sibling GAP-003A PRs already
occupy 0026–0079.

## Context

ADR 0079 removes an accepted project-history identity on
`AnalysisRunLiveService`. Operators still had no published binary that mints
that POST onto spawned `tepp-loopback` TCP. Duplicating project-history
cancel HTTP (#447), retrieval CLI (#431), GET-by-id (#429), collection CLI
(#428), collection GET (#424), POST CLI (#420), export cancel CLI (#446),
interpretation-run cancel CLI (#442), analysis-run cancel (#361), Leiden,
Driver p.16, or GAP-010 Figma/export would collide with live PRs. Naruon is
refused on this LineageWeave-owned adapter; `NaruonLiveService` stays
POST-only.

## Decision

Publish `tepp-project-history-cancel cancel`:

- Pattern: `from_args` + typed `lineageweave_project_history_cancel_exchange`
  + `loopback_http1_from_project_history_cancel_exchange` +
  `dispatch`/`execute`/`render` + published `[[bin]]`.
- Empty stdin is admitted. Nonempty leftover stdin fails closed.
- Public bind, `localhost` host, `http` origin, unpublished consumer, and
  credential flags fail closed.
- Stdout is one metric-free cancelled identity with `cancelled=true` and
  `inference_status=temporal_association_only`. Evidence text, findings,
  RMSE, bias, coverage, SE-gate, causal scores, and
  `tepp.scientific_acceptance.v1` never appear.
- Dedicated binary so it does not collide with `tepp-project-histories`
  (#428) or `tepp-project-history-get` (#431).

## Alternatives considered

1. **Reuse `tepp-project-histories list`** — rejected; that CLI is collection GET.
2. **Reuse `tepp-project-history-get get`** — rejected; that CLI is GET-by-id.
3. **Add GET to `NaruonLiveService`** — rejected; POST-only.
4. **Published `tepp-project-history-cancel cancel`** — accepted.

## Consequences

- Operators can retract an accepted project-history identity without a
  second cancel HTTP PR.
- Cancel JSON cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- Cancel success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-LineageWeave consumers, nonempty leftover stdin, present
`idempotency-key` as an HTTP header, slash/NUL identities, credential flags,
public bind, and metric keys fail closed. TCP execute does not fall back to
an empty in-process listener.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Evidence text, findings, and actor lists stay off the cancel receipt.
- HTTP 200 on cancel is not measurement evidence and is not a causal claim.

## Compatibility and migration

Cancel HTTP, collection GET, GET-by-id, POST `/v1/project-histories`, and
`NaruonLiveService` POST-only remain unchanged. Persistence remains
GAP-003B.

## Verification

Falsifiable evidence:

- `tepp-project-history-cancel cancel` of an accepted project-history returns
  metric-free `cancelled=true` without RMSE/bias/coverage/SE-gate/evidence/
  findings/causal-score/`tepp.scientific_acceptance.v1` keys;
- naruon, nonempty leftover stdin, `localhost`, `http` origin, slash/NUL
  identities, public bind, and unknown keys fail closed;
- `NaruonLiveService` still refuses GET;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the published binary; cancel HTTP remains valid. A
superseding ADR is required to persist cancel, bind a public address, emit
scientific-acceptance on cancel, open naruon on this adapter, add GET to
`NaruonLiveService`, or treat cancel success as an ADR 0014 claim.

## Related authority

- ADR 0079 owns loopback project-history cancel HTTP.
- ADR 0066 owns loopback project-history GET-by-id.
- ADR 0028 owns loopback project-history collection GET.
- ADR 0078 owns the export cancel CLI (live #446).
- ADR 0074 owns interpretation-run cancel CLI (live #442).
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
