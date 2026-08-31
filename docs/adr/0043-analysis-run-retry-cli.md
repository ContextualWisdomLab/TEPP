# ADR 0043 — Loopback analysis-run retry CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0032 (retry HTTP) and ADR 0033 (retry consumer parity). Does not reuse ADR 0030–0042 numbers from other stacks. Does not supersede ADR 0014 claim-promotion authority.

## Context

ADR 0032 owns `POST /v1/analysis-runs/{run_id}/retry`. ADR 0033 owns typed
naruon/`LineageWeave` retry exchanges. Operators still have to hand-roll
HTTP/1.1 to clone a failed or cancelled run on spawned `tepp-loopback`.
Cancel CLI (#378), create CLI (#385), status CLI (#392), collection CLI
(#371), lifecycle CLI (#362), and execute CLI (#390) are different verbs or
different stacks. `tepp_api` owns retry; the CLI belongs here.

## Decision

Publish `tepp-retry`:

- `tepp-retry retry` mints `naruon_analysis_run_retry_exchange` or
  `lineageweave_analysis_run_retry_exchange` and renders through
  `loopback_http1_from_retry_exchange`.
- `--origin` stays the published HTTPS origin; only `--host` is the loopback
  bind address printed by `tepp-loopback`.
- Empty stdin is admitted; typed retry JSON must match `--run-id` and the
  **new** `--idempotency-key`.
- Success stdout is a metric-free child `202 Accepted` with a new `run_id`.
- Public bind hosts, `localhost`, unpublished consumers, credential-shaped
  flags, and non-`https` origins fail closed.
- Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Execute CLI, cancel CLI, create CLI, status CLI, collection CLI, or another
  retry HTTP/consumer-parity slice.

## Alternatives considered

1. **Keep hand-rolled retry HTTP in each operator script** — rejected because
   GAP-003A is operator-visible and create/cancel already have CLIs.
2. **Add `retry` to `tepp-analysis-runs` on the create-CLI stack** — rejected
   because that stack does not include retry HTTP (#369).
3. **Reuse execute CLI** — rejected; execute is a different verb on
   `analysis_engine`.

## Consequences

- Operators can clone failed or cancelled runs without embedding the library.
- HTTP 202 on retry is not release evidence.

## Failure and recovery

Non-loopback hosts, `localhost`, non-`https` origins, unpublished consumers,
metric keys, unknown artifact fields, empty identities, and retry of
accepted/running/succeeded/unknown parents return a fail-closed API error.
The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Retry remains loopback-served, size-bounded, and content-redacting.
- Child receipts stay metric-free.

## Compatibility and migration

Create, cancel, collection, and retry HTTP exchanges are unchanged. Production
adapters may replace loopback while preserving metric-free child receipts.

## Verification

Falsifiable evidence:

- naruon retry CLI is HTTPS POST `/retry` without credentials or RMSE keys;
- LineageWeave retry CLI changes only `tepp-consumer`;
- public bind, `localhost`, `http://` origins, and accepted parents fail closed;
- create then cancel then typed retry CLI then stdout is a new metric-free
  `202 Accepted` for both consumers;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the retry CLI; retry HTTP and consumer exchanges remain
valid. A superseding ADR is required to persist status, bind a public
address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0032 owns retry HTTP.
- ADR 0033 owns retry consumer parity.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
