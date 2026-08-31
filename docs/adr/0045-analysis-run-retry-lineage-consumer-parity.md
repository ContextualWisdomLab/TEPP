# ADR 0045 — Analysis-run retry-lineage consumer parity

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0035 and ADR 0018. Does not supersede ADR 0014. ADR 0026–0044 remain on other live PRs (0044 is retry-parent consumer parity). This ADR number is unique on the retry-lineage GET lineage.

## Context

ADR 0035 added `GET /v1/analysis-runs/{run_id}/retries` on `AnalysisRunLiveService` and a Naruon retry-lineage exchange builder. The Naruon compatibility listener (`NaruonLiveService`) still refused every GET path. `LineageWeave` had a create-exchange builder but no retry-lineage exchange, so a published consumer would have to mint a Naruon-labelled inspect. The packaged `tepp-loopback` binary had no TCP proof that retry-lineage GET works on the shared listener after cancel and retry.

Duplicating the retry-lineage DTO, GET status, lifecycle POST, collection GET, retry POST, stored-request, retry-parent, or engine-library slices would not close this consumer-parity gap. Adding retry to `NaruonLiveService` would duplicate ADR 0032 / #369.

## Decision

- `lineageweave_analysis_run_retry_lineage_exchange` reuses the Naruon retry-lineage builder and replaces only `tepp-consumer`.
- `NaruonLiveService` serves the same metric-free retry-lineage GET for the Naruon-only compatibility listener. Accepted creates return an empty `retries` array because that listener does not retry. LineageWeave consumers remain refused there; they use `AnalysisRunLiveService`.
- `tepp-loopback` proves create-then-cancel-then-retry-then-inspect over loopback TCP so a non-empty child list is observable.
- Retry-lineage payloads stay metric-free. Unknown runs, consumer mismatch, nonempty bodies, and metric keys still fail closed.

## Non-goals

- GET status, running/terminal POST, collection GET, retry POST, retry-parent, persistence, or production TLS.
- Opening `NaruonLiveService` to LineageWeave.
- Adding retry to `NaruonLiveService`.
- An ADR 0014 scientific claim.

## Alternatives considered

1. **Leave retry-lineage only on `AnalysisRunLiveService`** — rejected because the compatibility listener would silently refuse a documented path.
2. **Admit LineageWeave on `NaruonLiveService`** — rejected because that listener is Naruon-only (ADR 0011/0018).
3. **Mint a second retry-lineage DTO** — rejected as a duplicate of ADR 0035.
4. **Add retry to `NaruonLiveService` so children can be non-empty there** — rejected as a duplicate of ADR 0032.
5. **Consumer-parity retry-lineage on the existing typed inspect** — accepted.

## Consequences

- Both published consumers can build a credential-free retry-lineage GET.
- Naruon local proofs can inspect an empty `retries` array on accepted creates on either listener.
- Operators can observe a non-empty child list through `tepp-loopback` after cancel and retry without a second HTTP stack.

## Failure and recovery

Unknown runs, consumer mismatch, nonempty bodies, metric keys, and oversized identities fail closed with a redacted envelope. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Retry-lineage remains loopback-only and metric-free.
- HTTP `200` inspect is not measurement or release evidence.

## Compatibility and migration

ADR 0035 create/inspect semantics are unchanged. Production adapters may replace loopback while preserving consumer identity, metric-free retry-lineage fields, and Naruon-only compatibility-listener admission.

## Verification

- LineageWeave retry-lineage exchange carries `tepp-consumer: lineageweave` and no credentials;
- NaruonLiveService inspects accepted Naruon runs as an empty `retries` array and refuses LineageWeave, metrics, nonempty bodies, and unknown runs;
- `tepp-loopback` create-cancel-retry-inspect over TCP returns a metric-free non-empty child list;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the LineageWeave builder, compatibility-listener inspect, and binary TCP proof; ADR 0035 shared-listener retry-lineage GET remains. A superseding ADR is required to persist inspect, bind a public address, add retry to `NaruonLiveService`, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0035 owns the shared-listener retry-lineage GET path and metric-free inspect fields.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
