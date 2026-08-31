# ADR 0033 — Analysis-run retry consumer parity

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0032 and ADR 0018. Does not supersede ADR 0014. ADR 0026–0032 remain on other GAP-003A slices. This ADR number is unique on the retry-HTTP lineage; other live PRs may reuse 0033 on unrelated stacks.

## Context

ADR 0032 added `POST /v1/analysis-runs/{run_id}/retry` on `AnalysisRunLiveService` and a Naruon retry-exchange builder. The Naruon compatibility listener (`NaruonLiveService`) still refused every retry path. `LineageWeave` had a create-exchange builder but no retry-exchange, so a published consumer would have to mint a Naruon-labelled retry. The packaged `tepp-loopback` binary had no TCP proof that retry works on the shared listener.

Duplicating the retry DTO, GET status, lifecycle POST, cancel, collection GET, or engine-library slices would not close this consumer-parity gap.

## Decision

- `lineageweave_analysis_run_retry_exchange` reuses the Naruon retry builder and replaces only `tepp-consumer`.
- `NaruonLiveService` serves the same metric-free retry path for the Naruon-only compatibility listener. LineageWeave consumers remain refused there; they use `AnalysisRunLiveService`.
- `NaruonLiveService` stays POST-only. GET remains refused.
- `tepp-loopback` proves create-then-cancel-then-retry over loopback TCP for LineageWeave.
- Retry receipts stay metric-free `202 Accepted` with a new `run_id` and a new idempotency key. Accepted, running, succeeded, and unknown runs still fail closed.

## Non-goals

- GET status, running/terminal POST, collection GET, cancel DTO changes, persistence, or production TLS.
- Opening `NaruonLiveService` to LineageWeave.
- Adding GET to `NaruonLiveService`.
- An ADR 0014 scientific claim.

## Alternatives considered

1. **Leave retry only on `AnalysisRunLiveService`** — rejected because the compatibility listener would silently refuse a documented path.
2. **Admit LineageWeave on `NaruonLiveService`** — rejected because that listener is Naruon-only (ADR 0011/0018).
3. **Mint a second retry DTO** — rejected as a duplicate of ADR 0032.
4. **Consumer-parity retry on the existing typed request** — accepted.

## Consequences

- Both published consumers can build a credential-free retry exchange.
- Naruon local proofs can retry failed or cancelled runs on either listener.
- Operators can observe LineageWeave retry through `tepp-loopback` without a second HTTP stack.

## Failure and recovery

Unknown runs, consumer mismatch, parent-key reuse, metric keys, accepted/running/succeeded retry, identity mismatch, and oversized identities fail closed with a redacted envelope. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Retry remains loopback-only and metric-free.
- HTTP `202` on a retried run is not measurement or release evidence.

## Compatibility and migration

ADR 0032 create/retry semantics are unchanged. Production adapters may replace loopback while preserving consumer identity, metric-free retry receipts, and Naruon-only compatibility-listener admission.

## Verification

- LineageWeave retry exchange carries `tepp-consumer: lineageweave` and no credentials;
- NaruonLiveService retries failed/cancelled Naruon runs and refuses LineageWeave, GET, metrics, and accepted/running/succeeded runs;
- `tepp-loopback` create-then-cancel-then-retry over TCP returns metric-free `202 Accepted`;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the LineageWeave builder, compatibility-listener retry, and binary TCP proof; ADR 0032 shared-listener retry remains. A superseding ADR is required to persist retry, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0032 owns the shared-listener retry path and metric-free child `202`.
- ADR 0029 owns loopback cancel used by the TCP proof.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
