# ADR 0094 — Loopback export idempotency-key lookup CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0093. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0093 occupied
including #465=0093, #464=0092, #463=0091, #459=0090, #457=0089, #411=0054.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no
user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0093 publishes `GET /v1/exports/by-idempotency/{idempotency_key}`. Operators
still had no published binary that mints that GET onto spawned `tepp-loopback`
TCP. Duplicating lookup GET (#465), GET-by-id (#411), retrieval CLI (#417),
collection GET/CLI (#443/#444), stored-request GET/CLI (#457/#459),
export-authorize CLI (#410), analysis-run lookup CLI (#401), Leiden, or GAP-010
would collide with live PRs. Cancel lineages stay closed. LineageWeave is
refused on this naruon-owned adapter. `NaruonLiveService` stays POST-only.

## Decision

Publish `tepp-export-lookup lookup` which mints
`naruon_export_idempotency_lookup_exchange` onto spawned `tepp-loopback` TCP.
Empty stdin is admitted. Nonempty leftover stdin, public bind, `localhost`,
`http` origin, unpublished consumer, LineageWeave, reserved prefix-as-key, and
credential flags fail closed. Dedicated binary so it does not collide with
`tepp-export-list` (#444), `tepp-export-get` (#417), `tepp-export-request`
(#459), or export-authorize (#410). Response is the metric-free
`ExportIdempotencyLookup`. `tepp.scientific_acceptance.v1` never appears.

## Alternatives considered

1. Re-open cancel CLI — rejected.
2. Reuse `tepp-export-get` — rejected; that is ADR 0055.
3. Reuse `tepp-export-request` — rejected; that is stored-request GET.
4. Dedicated lookup binary — accepted.

## Consequences

CLI success is not measurement evidence and is not an ADR 0014 claim.
Sequence remains association, not causation.

## Failure and recovery

LineageWeave, nonempty leftover stdin, extra segments, slash/NUL, missing
keys, public bind, `localhost`, reserved prefix-as-key, and metric keys fail
closed.

## Verification

- `tepp-export-lookup lookup` of an authorized export prints
  `export_id`/`decision_code`/`idempotency_key` without RMSE or
  `tepp.scientific_acceptance.v1`;
- LineageWeave, public bind, `localhost`, `http` origin, leftover stdin,
  slash/NUL, reserved prefix, and missing keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the published binary; lookup GET remains valid. A superseding
ADR is required to persist the registry, bind a public address, re-open cancel,
emit scientific-acceptance, open LineageWeave on this adapter, add GET to
`NaruonLiveService`, or treat CLI success as an ADR 0014 claim.

## Related authority

ADR 0093, ADR 0054, ADR 0009, ADR 0011, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
