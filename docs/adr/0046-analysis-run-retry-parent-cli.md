# ADR 0046 — Loopback analysis-run retry-parent CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0038 (retry-parent GET) and ADR 0044 (retry-parent consumer parity). Does not reuse ADR 0039–0045 numbers from other stacks. Does not supersede ADR 0014 claim-promotion authority.

## Context

ADR 0038 owns `GET /v1/analysis-runs/{run_id}/parent`. ADR 0044 owns typed
naruon/`LineageWeave` retry-parent exchanges. Operators still have to hand-roll
HTTP/1.1 to inspect a retry child's parent on spawned `tepp-loopback`. Retry
CLI (#394), stored-request CLI (#395), status CLI (#392), collection CLI
(#371), cancel CLI (#378), and create CLI (#385) are different verbs or
different stacks. `tepp_api` owns retry-parent; the CLI belongs here.

## Decision

Publish `tepp-retry-parent`:

- `tepp-retry-parent parent` mints `naruon_analysis_run_retry_parent_exchange`
  or `lineageweave_analysis_run_retry_parent_exchange` and renders through
  `loopback_http1_from_retry_parent_exchange`.
- `--origin` stays the published HTTPS origin; only `--host` is the loopback
  bind address printed by `tepp-loopback`.
- Empty stdin is required; nonempty GET bodies fail closed.
- Success stdout is a metric-free `200 OK` inspect. `"parent": null` when the
  run was never retried.
- Public bind hosts, `localhost`, unpublished consumers, credential-shaped
  flags, and non-`https` origins fail closed.
- Persistence remains GAP-003B.
- This slice does not add GET to `NaruonLiveService` beyond the Naruon-only
  compatibility inspect already owned by ADR 0044. LineageWeave remains
  refused there.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Retry CLI, retry HTTP, retry consumer-parity, retry-parent GET, or
  retry-parent consumer-parity slices.

## Alternatives considered

1. **Keep hand-rolled retry-parent HTTP in each operator script** — rejected
   because GAP-003A is operator-visible and retry/stored-request already have
   CLIs.
2. **Add `parent` to `tepp-retry` on the retry-CLI stack** — rejected because
   that stack does not include retry-parent GET (#384).
3. **Open LineageWeave on `NaruonLiveService`** — rejected; that listener is
   Naruon-only (ADR 0011/0018).

## Consequences

- Operators can inspect a retry child's parent without embedding the library.
- HTTP 200 on inspect is not release evidence.

## Failure and recovery

Non-loopback hosts, `localhost`, non-`https` origins, unpublished consumers,
metric keys, unknown artifact fields, empty identities, nonempty GET bodies,
and unknown runs return a fail-closed API error. The in-memory registry is
not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Retry-parent remains loopback-served, size-bounded, and content-redacting.
- Inspect receipts stay metric-free.

## Compatibility and migration

Retry-parent GET and consumer exchanges are unchanged. Production adapters may
replace loopback while preserving metric-free inspect receipts.

## Verification

Falsifiable evidence:

- naruon retry-parent CLI is HTTPS GET `/parent` without credentials or RMSE keys;
- LineageWeave retry-parent CLI changes only `tepp-consumer`;
- public bind, `localhost`, `http://` origins, and nonempty bodies fail closed;
- create then cancel then retry then typed parent CLI stdout is a metric-free
  non-null parent for both consumers, and never-retried runs print `"parent": null`;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the retry-parent CLI; retry-parent GET and consumer exchanges
remain valid. A superseding ADR is required to persist inspect, bind a public
address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0038 owns retry-parent GET.
- ADR 0044 owns retry-parent consumer parity.
- ADR 0018 owns consumer-scoped ingress and metric-free receipts.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
