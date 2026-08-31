# ADR 0030 — Loopback analysis-run lifecycle CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0028 (lifecycle POST) and ADR 0029 (lifecycle consumer parity). Does not reuse ADR 0030 numbers from other stacks (scientific-acceptance execute CLI on a sibling lineage). Does not supersede ADR 0014 claim-promotion authority.

## Context

ADR 0028 owns `POST /v1/analysis-runs/{run_id}/running` and
`POST /v1/analysis-runs/{run_id}/terminal`. ADR 0029 owns typed
naruon/`LineageWeave` lifecycle exchanges and Naruon compatibility-listener
POST. Operators still have to hand-roll HTTP/1.1 to record running or terminal
status on spawned `tepp-loopback`. Scientific-acceptance execute CLI (#362),
retry CLI (#394), status CLI (#392), cancel CLI (#378), create CLI (#385), and
collection CLI (#371) are different verbs or different stacks. `tepp_api` owns
lifecycle; the CLI belongs here.

## Decision

Publish `tepp-lifecycle`:

- `tepp-lifecycle running` and `tepp-lifecycle terminal` mint
  `naruon_analysis_run_running_exchange` /
  `naruon_analysis_run_terminal_exchange` or the LineageWeave equivalents and
  render through `loopback_http1_from_lifecycle_exchange`.
- `--origin` stays the published HTTPS origin; only `--host` is the loopback
  bind address printed by `tepp-loopback`.
- Empty stdin is admitted for `running`; terminal requires typed JSON matching
  `--run-id`, `--idempotency-key`, and a terminal `run_state`.
- Success stdout is a metric-free `200` status. `tepp.scientific_acceptance.v1`
  never appears.
- Public bind hosts, `localhost`, unpublished consumers, credential-shaped
  flags, and non-`https` origins fail closed.
- `NaruonLiveService` stays POST-only. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- GET on `NaruonLiveService`.
- Leiden community detection, Driver p.16 restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Execute CLI, retry CLI, status CLI, cancel CLI, create CLI, collection CLI,
  or another lifecycle POST/consumer-parity slice.

## Alternatives considered

1. **Keep hand-rolled lifecycle HTTP in each operator script** — rejected
   because GAP-003A is operator-visible and create/retry already have CLIs.
2. **Add `running`/`terminal` to `tepp-retry` or execute CLI** — rejected
   because those stacks do not include lifecycle POST (#360/#388).
3. **Reuse scientific-acceptance execute CLI (#362)** — rejected; that CLI
   drives engine execute, not running/terminal status.

## Consequences

- Operators can record running and terminal status without embedding the
  library.
- HTTP 200 on lifecycle is not release evidence.

## Failure and recovery

Non-loopback hosts, `localhost`, non-`https` origins, unpublished consumers,
metric keys, empty terminal stdin, unknown artifact fields, empty identities,
consumer mismatch, reverse transitions, and mutating a terminal run return a
fail-closed API error. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Lifecycle remains loopback-served, size-bounded, and content-redacting.
- Running and failed receipts stay metric-free.

## Compatibility and migration

Create and lifecycle HTTP exchanges are unchanged. Production adapters may
replace loopback while preserving metric-free running/terminal receipts.

## Verification

Falsifiable evidence:

- naruon running CLI is HTTPS POST `/running` without credentials or RMSE keys;
- LineageWeave running CLI changes only `tepp-consumer`;
- public bind, `localhost`, `http://` origins, unpublished consumers, and empty
  terminal stdin fail closed;
- create then typed running CLI then stdout is metric-free `200` for both
  consumers; consumer mismatch is `400`;
- create then failed terminal CLI records `Failed` without scientific
  acceptance;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the lifecycle CLI; lifecycle HTTP and consumer exchanges
remain valid. A superseding ADR is required to persist status, bind a public
address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0028 owns lifecycle POST.
- ADR 0029 owns lifecycle consumer parity.
- ADR 0018 owns consumer-scoped ingress and metric-free receipts.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
