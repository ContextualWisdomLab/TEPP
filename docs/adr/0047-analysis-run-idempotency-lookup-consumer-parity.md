# ADR 0047 — Analysis-run idempotency-lookup consumer parity

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0037 and ADR 0018. Does not supersede ADR 0014. ADR 0026–0046 remain on other live PRs (0046 is retry-parent CLI; 0045 is retry-lineage consumer parity and nested ICC). This ADR number is unique on the idempotency-lookup GET lineage.

## Context

ADR 0037 added `GET /v1/analysis-runs/by-idempotency/{key}` on `AnalysisRunLiveService` and a Naruon lookup exchange builder. The Naruon compatibility listener (`NaruonLiveService`) still refused every GET path. `LineageWeave` had a create-exchange builder but no idempotency-lookup exchange, so a published consumer would have to mint a Naruon-labelled inspect. The packaged `tepp-loopback` binary had no TCP proof that lookup GET works on the shared listener after create.

Duplicating the lookup DTO, GET status, lifecycle POST, collection GET, retry POST, stored-request, retry-lineage, retry-parent, or engine-library slices would not close this consumer-parity gap.

## Decision

- `lineageweave_analysis_run_idempotency_lookup_exchange` reuses the Naruon lookup builder and replaces only `tepp-consumer`.
- `NaruonLiveService` serves the same metric-free lookup GET for the Naruon-only compatibility listener. Accepted creates return a real `run_id` because that listener already keys idempotency replay. LineageWeave consumers remain refused there; they use `AnalysisRunLiveService`.
- `tepp-loopback` proves create-then-inspect over loopback TCP so a durable `run_id` is observable from the 202 receipt key.
- Lookup payloads stay metric-free. Unknown keys, consumer mismatch, nonempty bodies, and metric keys still fail closed.

## Non-goals

- GET status, running/terminal POST, collection GET, retry POST, retry-parent, persistence, or production TLS.
- Opening `NaruonLiveService` to LineageWeave.
- An ADR 0014 scientific claim.

## Alternatives considered

1. **Leave lookup only on `AnalysisRunLiveService`** — rejected because the compatibility listener would silently refuse a documented path.
2. **Admit LineageWeave on `NaruonLiveService`** — rejected because that listener is Naruon-only (ADR 0011/0018).
3. **Mint a second lookup DTO** — rejected as a duplicate of ADR 0037.
4. **Consumer-parity lookup on the existing typed inspect** — accepted.

## Consequences

- Both published consumers can build a credential-free idempotency-lookup GET.
- Naruon local proofs can resolve an accepted create to a durable `run_id` on either listener.
- Operators can observe the same resolve through `tepp-loopback` after create without a second HTTP stack.

## Failure and recovery

Unknown keys, consumer mismatch, nonempty bodies, metric keys, and oversized identities fail closed with a redacted envelope. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Lookup remains loopback-only and metric-free.
- HTTP `200` inspect is not measurement or release evidence.

## Compatibility and migration

ADR 0037 create/inspect semantics are unchanged. Production adapters may replace loopback while preserving consumer identity, metric-free lookup fields, and Naruon-only compatibility-listener admission.

## Verification

- LineageWeave lookup exchange carries `tepp-consumer: lineageweave` and no credentials;
- NaruonLiveService resolves accepted Naruon runs to a real `run_id` and refuses LineageWeave, metrics, nonempty bodies, and unknown keys;
- `tepp-loopback` create-then-GET over TCP returns a metric-free identity;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the LineageWeave builder, compatibility-listener inspect, and binary TCP proof; ADR 0037 shared-listener lookup GET remains. A superseding ADR is required to persist inspect, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0037 owns the shared-listener idempotency-lookup GET path and metric-free inspect fields.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
