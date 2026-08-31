# ADR 0030 — Analysis-run cancel consumer parity

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0029 and ADR 0018. Does not supersede ADR 0014. ADR 0026–0028 remain on other GAP-003A slices. This ADR number is unique on the cancel-HTTP lineage; other live PRs may reuse 0030 on unrelated stacks.

## Context

ADR 0029 added `POST /v1/analysis-runs/{run_id}/cancel` on `AnalysisRunLiveService` and a Naruon cancel-exchange builder. The Naruon compatibility listener (`NaruonLiveService`) still refused every cancel path. `LineageWeave` had a create-exchange builder but no cancel-exchange, so a published consumer would have to mint a Naruon-labelled cancel. The packaged `tepp-loopback` binary had no TCP proof that cancel works on the shared listener.

Duplicating the cancel DTO, GET status, lifecycle POST, collection GET, retry, or engine-library slices would not close this consumer-parity gap.

## Decision

- `lineageweave_analysis_run_cancel_exchange` reuses the Naruon cancel builder and replaces only `tepp-consumer`.
- `NaruonLiveService` serves the same metric-free cancel path for the Naruon-only compatibility listener. LineageWeave consumers remain refused there; they use `AnalysisRunLiveService`.
- `tepp-loopback` proves create-then-cancel over loopback TCP.
- Cancelled status stays metric-free. Succeeded, failed, and unknown runs still fail closed.

## Non-goals

- GET status, running/terminal POST, collection GET, retry, persistence, or production TLS.
- Opening `NaruonLiveService` to LineageWeave.
- An ADR 0014 scientific claim.

## Alternatives considered

1. **Leave cancel only on `AnalysisRunLiveService`** — rejected because the compatibility listener would silently refuse a documented path.
2. **Admit LineageWeave on `NaruonLiveService`** — rejected because that listener is Naruon-only (ADR 0011/0018).
3. **Mint a second cancel DTO** — rejected as a duplicate of ADR 0029.
4. **Consumer-parity cancel on the existing typed request** — accepted.

## Consequences

- Both published consumers can build a credential-free cancel exchange.
- Naruon local proofs can cancel on either listener.
- Operators can observe cancel through `tepp-loopback` without a second HTTP stack.

## Failure and recovery

Unknown runs, consumer mismatch, idempotency mismatch, metric keys, succeeded/failed runs, and oversized identities fail closed with a redacted envelope. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Cancel remains loopback-only and metric-free.
- HTTP `200` cancelled is not measurement or release evidence.

## Compatibility and migration

ADR 0029 create/cancel semantics are unchanged. Production adapters may replace loopback while preserving consumer identity, metric-free cancelled status, and Naruon-only compatibility-listener admission.

## Verification

- LineageWeave cancel exchange carries `tepp-consumer: lineageweave` and no credentials;
- NaruonLiveService cancels accepted/running Naruon runs and refuses LineageWeave, metrics, and terminal runs;
- `tepp-loopback` create-then-cancel over TCP returns metric-free `cancelled`;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the LineageWeave builder, compatibility-listener cancel, and binary TCP proof; ADR 0029 shared-listener cancel remains. A superseding ADR is required to persist cancel, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0029 owns the shared-listener cancel path and cancelled status.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
