# ADR 0094 — Loopback export idempotency-key lookup CLI

**Decision status:** Accepted  
**Implementation maturity:** active-PR  
**Date:** 2026-09-01  
**Supersedes:** None; complements ADR 0093. Does not re-open cancel lineages or supersede ADR 0014.  
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.  
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0093 publishes `GET /v1/exports/by-idempotency/{idempotency_key}`. Operators need a published binary that mints that GET onto spawned `tepp-loopback` TCP without hand-writing HTTP. The CLI must accept the same opaque idempotency-key domain as export authorization and ADR 0093. Review found the first CLI narrowed that domain by rejecting slash-containing keys and the literal value `by-idempotency`, even though those values could already have been accepted by export authorization.

## Decision

Publish `tepp-export-lookup lookup`, backed by `naruon_export_idempotency_lookup_exchange`:

- Empty stdin is admitted; nonempty leftover stdin fails closed.
- Public bind, `localhost`, non-HTTPS origin, unpublished consumers, LineageWeave and credential-shaped flags fail closed.
- Idempotency keys are opaque data. Slash-containing keys are accepted by the CLI and percent-encoded by the typed HTTP exchange into one route segment. The literal key `by-idempotency` is accepted as data after the route prefix.
- NUL and oversized keys fail closed. Raw additional URL segments are never accepted by the HTTP parser as part of a key.
- Response stdout is only the metric-free `ExportIdempotencyLookup`. `tepp.scientific_acceptance.v1` and tenant/principal/source-text data never appear.
- The CLI does not authorize ADR 0099 stored-request disclosure; that separate convenience route remains quarantined until authenticated tenant/principal scope exists.
- `NaruonLiveService` stays POST-only.

## Alternatives considered

1. Re-open a cancel CLI — rejected; unrelated lifecycle responsibility.
2. Reuse `tepp-export-get` — rejected because that command resolves server-assigned `export_id` capabilities.
3. Keep a stricter CLI key grammar than the create contract — rejected because accepted receipts would become operationally unreachable.
4. Preserve the exact opaque accepted key domain through the typed exchange — accepted.

## Consequences

The CLI is compatible with the create contract for key identity instead of imposing a second, narrower schema. URL routing remains safe because encoding happens inside a single path segment and parsing segments precedes percent decoding.

## Failure and recovery

LineageWeave, nonempty stdin, extra raw URL segments, NUL, oversized keys, missing keys, public bind, `localhost`, invalid origin, credentials and metric-bearing responses fail closed. A key containing `/` or equal to `by-idempotency` is not itself an error; it must resolve exactly as the create contract stored it.

## Verification

- lookup of an authorized export prints `export_id`/`decision_code`/`idempotency_key` without RMSE or scientific-acceptance data;
- POST then CLI lookup round-trips `scope/key` through `%2F` path encoding;
- POST then CLI lookup round-trips the literal key `by-idempotency` through `/by-idempotency/by-idempotency`;
- LineageWeave, public bind, `localhost`, non-HTTPS origin, leftover stdin, NUL, oversized keys, missing keys and credentials fail closed;
- exact-head Clippy, `tepp_api` tests, rustdoc, line/branch coverage, security workflows and qualifying review remain required.

## Rollback and supersession

Rollback removes the published binary; ADR 0093 lookup GET remains valid. A superseding ADR is required to change key identity semantics, persist the registry, bind a public address, open LineageWeave, add GET to `NaruonLiveService`, or treat CLI success as an ADR 0014 claim.

## Related authority

ADR 0093, ADR 0099, ADR 0054, ADR 0009, ADR 0011, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
