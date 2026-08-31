# ADR 0042 — Analysis-run collection GET consumer parity

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0031 and ADR 0018. Does not supersede ADR 0014. ADR 0026–0041 remain on other live PRs (0041 is the scientific-acceptance execute CLI). This ADR number is unique on the collection GET lineage.

## Context

ADR 0031 added `GET /v1/analysis-runs` on `AnalysisRunLiveService` and a Naruon collection exchange builder. The Naruon compatibility listener (`NaruonLiveService`) still refused every GET path. `LineageWeave` had a create-exchange builder but no collection exchange, so a published consumer would have to mint a Naruon-labelled list. The packaged `tepp-loopback` binary had no TCP proof that collection GET works on the shared listener.

Duplicating the collection DTO, GET status, lifecycle POST, stored-request GET, cancel consumer-parity, retry, or engine-library slices would not close this consumer-parity gap.

## Decision

- `lineageweave_analysis_run_collection_exchange` reuses the Naruon collection builder and replaces only `tepp-consumer`.
- `NaruonLiveService` serves the same metric-free collection GET for the Naruon-only compatibility listener. LineageWeave consumers remain refused there; they use `AnalysisRunLiveService`.
- `tepp-loopback` proves create-then-list over loopback TCP.
- Collection payloads stay metric-free. Unknown cursors, consumer mismatch, nonempty bodies, GET-by-id, and metric keys still fail closed.

## Non-goals

- GET status, running/terminal POST, stored-request GET, retry, cancel consumer-parity, persistence, or production TLS.
- Opening `NaruonLiveService` to LineageWeave.
- An ADR 0014 scientific claim.

## Alternatives considered

1. **Leave collection GET only on `AnalysisRunLiveService`** — rejected because the compatibility listener would silently refuse a documented path.
2. **Admit LineageWeave on `NaruonLiveService`** — rejected because that listener is Naruon-only (ADR 0011/0018).
3. **Mint a second collection DTO** — rejected as a duplicate of ADR 0031.
4. **Consumer-parity collection GET on the existing typed list** — accepted.

## Consequences

- Both published consumers can build a credential-free collection GET.
- Naruon local proofs can enumerate accepted runs on either listener.
- Operators can observe collection GET through `tepp-loopback` without a second HTTP stack.

## Failure and recovery

Unknown cursors, consumer mismatch, nonempty bodies, metric keys, GET-by-id, and oversized page limits fail closed with a redacted envelope. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Collection remains loopback-only and metric-free.
- HTTP `200` list is not measurement or release evidence.

## Compatibility and migration

ADR 0031 create/list semantics are unchanged. Production adapters may replace loopback while preserving consumer identity, metric-free collection rows, and Naruon-only compatibility-listener admission.

## Verification

- LineageWeave collection exchange carries `tepp-consumer: lineageweave` and no credentials;
- NaruonLiveService lists accepted Naruon runs and refuses LineageWeave, metrics, nonempty bodies, GET-by-id, and unknown cursors;
- `tepp-loopback` create-then-list over TCP returns metric-free collection rows;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the LineageWeave builder, compatibility-listener list, and binary TCP proof; ADR 0031 shared-listener collection GET remains. A superseding ADR is required to persist the registry, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0031 owns the shared-listener collection GET path and metric-free list rows.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
