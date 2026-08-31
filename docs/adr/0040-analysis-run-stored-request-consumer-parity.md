# ADR 0040 — Analysis-run stored-request consumer parity

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0034 and ADR 0018. Does not supersede ADR 0014. ADR 0026–0039 remain on other live PRs (0039 is the two-group OLS profile). This ADR number is unique on the stored-request GET lineage.

## Context

ADR 0034 added `GET /v1/analysis-runs/{run_id}/request` on `AnalysisRunLiveService` and a Naruon stored-request exchange builder. The Naruon compatibility listener (`NaruonLiveService`) still refused every GET path. `LineageWeave` had a create-exchange builder but no stored-request exchange, so a published consumer would have to mint a Naruon-labelled inspect. The packaged `tepp-loopback` binary had no TCP proof that stored-request GET works on the shared listener.

Duplicating the stored-request DTO, GET status, lifecycle POST, collection GET, retry, retry-lineage, or engine-library slices would not close this consumer-parity gap.

## Decision

- `lineageweave_analysis_run_stored_request_exchange` reuses the Naruon stored-request builder and replaces only `tepp-consumer`.
- `NaruonLiveService` serves the same metric-free stored-request GET for the Naruon-only compatibility listener. LineageWeave consumers remain refused there; they use `AnalysisRunLiveService`.
- `tepp-loopback` proves create-then-inspect over loopback TCP.
- Stored-request payloads stay metric-free. Unknown runs, consumer mismatch, nonempty bodies, and metric keys still fail closed.

## Non-goals

- GET status, running/terminal POST, collection GET, retry, retry-lineage, persistence, or production TLS.
- Opening `NaruonLiveService` to LineageWeave.
- An ADR 0014 scientific claim.

## Alternatives considered

1. **Leave stored-request only on `AnalysisRunLiveService`** — rejected because the compatibility listener would silently refuse a documented path.
2. **Admit LineageWeave on `NaruonLiveService`** — rejected because that listener is Naruon-only (ADR 0011/0018).
3. **Mint a second stored-request DTO** — rejected as a duplicate of ADR 0034.
4. **Consumer-parity stored-request on the existing typed inspect** — accepted.

## Consequences

- Both published consumers can build a credential-free stored-request GET.
- Naruon local proofs can inspect stored create fields on either listener.
- Operators can observe stored-request inspect through `tepp-loopback` without a second HTTP stack.

## Failure and recovery

Unknown runs, consumer mismatch, nonempty bodies, metric keys, and oversized identities fail closed with a redacted envelope. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Stored-request remains loopback-only and metric-free.
- HTTP `200` inspect is not measurement or release evidence.

## Compatibility and migration

ADR 0034 create/inspect semantics are unchanged. Production adapters may replace loopback while preserving consumer identity, metric-free stored-request fields, and Naruon-only compatibility-listener admission.

## Verification

- LineageWeave stored-request exchange carries `tepp-consumer: lineageweave` and no credentials;
- NaruonLiveService inspects accepted Naruon runs and refuses LineageWeave, metrics, nonempty bodies, and unknown runs;
- `tepp-loopback` create-then-inspect over TCP returns metric-free stored create fields;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the LineageWeave builder, compatibility-listener inspect, and binary TCP proof; ADR 0034 shared-listener stored-request GET remains. A superseding ADR is required to persist inspect, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0034 owns the shared-listener stored-request GET path and metric-free inspect fields.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
