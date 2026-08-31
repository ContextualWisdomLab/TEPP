# ADR 0044 — Analysis-run retry-parent consumer parity

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0038 and ADR 0018. Does not supersede ADR 0014. ADR 0026–0043 remain on other live PRs (0043 is the retry CLI). This ADR number is unique on the retry-parent GET lineage.

## Context

ADR 0038 added `GET /v1/analysis-runs/{run_id}/parent` on `AnalysisRunLiveService` and a Naruon retry-parent exchange builder. The Naruon compatibility listener (`NaruonLiveService`) still refused every GET path. `LineageWeave` had a create-exchange builder but no retry-parent exchange, so a published consumer would have to mint a Naruon-labelled inspect. The packaged `tepp-loopback` binary had no TCP proof that retry-parent GET works on the shared listener after cancel and retry.

Duplicating the retry-parent DTO, GET status, lifecycle POST, collection GET, retry POST, retry-lineage, stored-request, or engine-library slices would not close this consumer-parity gap. Adding retry to `NaruonLiveService` would duplicate ADR 0032 / #369.

## Decision

- `lineageweave_analysis_run_retry_parent_exchange` reuses the Naruon retry-parent builder and replaces only `tepp-consumer`.
- `NaruonLiveService` serves the same metric-free retry-parent GET for the Naruon-only compatibility listener. Accepted creates return `"parent": null` because that listener does not retry. LineageWeave consumers remain refused there; they use `AnalysisRunLiveService`.
- `tepp-loopback` proves create-then-cancel-then-retry-then-inspect over loopback TCP so a non-null parent is observable.
- Retry-parent payloads stay metric-free. Unknown runs, consumer mismatch, nonempty bodies, and metric keys still fail closed.

## Non-goals

- GET status, running/terminal POST, collection GET, retry POST, retry-lineage, persistence, or production TLS.
- Opening `NaruonLiveService` to LineageWeave.
- Adding retry to `NaruonLiveService`.
- An ADR 0014 scientific claim.

## Alternatives considered

1. **Leave retry-parent only on `AnalysisRunLiveService`** — rejected because the compatibility listener would silently refuse a documented path.
2. **Admit LineageWeave on `NaruonLiveService`** — rejected because that listener is Naruon-only (ADR 0011/0018).
3. **Mint a second retry-parent DTO** — rejected as a duplicate of ADR 0038.
4. **Add retry to `NaruonLiveService` so parent can be non-null there** — rejected as a duplicate of ADR 0032.
5. **Consumer-parity retry-parent on the existing typed inspect** — accepted.

## Consequences

- Both published consumers can build a credential-free retry-parent GET.
- Naruon local proofs can inspect `"parent": null` on accepted creates on either listener.
- Operators can observe a non-null parent through `tepp-loopback` after cancel and retry without a second HTTP stack.

## Failure and recovery

Unknown runs, consumer mismatch, nonempty bodies, metric keys, and oversized identities fail closed with a redacted envelope. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Retry-parent remains loopback-only and metric-free.
- HTTP `200` inspect is not measurement or release evidence.

## Compatibility and migration

ADR 0038 create/inspect semantics are unchanged. Production adapters may replace loopback while preserving consumer identity, metric-free retry-parent fields, and Naruon-only compatibility-listener admission.

## Verification

- LineageWeave retry-parent exchange carries `tepp-consumer: lineageweave` and no credentials;
- NaruonLiveService inspects accepted Naruon runs as `"parent": null` and refuses LineageWeave, metrics, nonempty bodies, and unknown runs;
- `tepp-loopback` create-cancel-retry-inspect over TCP returns a metric-free non-null parent;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the LineageWeave builder, compatibility-listener inspect, and binary TCP proof; ADR 0038 shared-listener retry-parent GET remains. A superseding ADR is required to persist inspect, bind a public address, add retry to `NaruonLiveService`, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0038 owns the shared-listener retry-parent GET path and metric-free inspect fields.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
