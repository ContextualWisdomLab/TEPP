# ADR 0028 — Analysis-run status consumer parity

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0027 and ADR 0018. Does not supersede ADR 0014. ADR 0026 remains on other GAP-003A slices. This ADR number is unique on the GET-status lineage; other live PRs may reuse 0028 on unrelated stacks.

## Context

ADR 0027 added `GET /v1/analysis-runs/{run_id}` on `AnalysisRunLiveService` and a Naruon status-exchange builder. `LineageWeave` had a create-exchange builder but no status-exchange, so a published consumer would have to mint a Naruon-labelled GET. The packaged `tepp-loopback` binary had no TCP proof that status GET works on the shared listener.

Duplicating the GET listener, lifecycle POST, cancel HTTP, collection GET, retry, or engine-library slices would not close this consumer-parity gap. Opening GET on `NaruonLiveService` would violate that listener's POST-only Naruon compatibility contract.

## Decision

- `lineageweave_analysis_run_status_exchange` reuses the Naruon status builder and replaces only `tepp-consumer`.
- `AnalysisRunLiveService` remains the shared GET listener. LineageWeave consumers poll their own runs there; Naruon runs stay isolated.
- `NaruonLiveService` stays POST-only. It does not serve GET status.
- `tepp-loopback` proves create-then-GET over loopback TCP with the LineageWeave consumer.
- Accepted GET bodies stay metric-free. Scientific-acceptance attachment remains the ADR 0027 succeeded-profile gate.

## Non-goals

- A second GET listener, running/terminal POST, collection GET, retry, persistence, or production TLS.
- Opening `NaruonLiveService` to GET or to LineageWeave.
- An ADR 0014 scientific claim.

## Alternatives considered

1. **Leave status GET only on the Naruon builder** — rejected because a published LineageWeave consumer would have to mint a Naruon-labelled GET.
2. **Admit GET on `NaruonLiveService`** — rejected because that listener is Naruon-only POST (ADR 0011/0018).
3. **Mint a second status DTO** — rejected as a duplicate of ADR 0027.
4. **Consumer-parity status GET on the existing typed request** — accepted.

## Consequences

- Both published consumers can build a credential-free status GET exchange.
- Operators can observe LineageWeave status through `tepp-loopback` without a second HTTP stack.
- Naruon compatibility remains POST-only.

## Failure and recovery

Non-`https` origins, empty or oversized run identities, credential headers, and consumer/idempotency mismatch fail closed. The in-memory registry is not durable. HTTP `200` on an accepted GET is not an ADR 0014 claim.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- GET remains loopback-only, size-bounded, and content-redacting.
- Metric-free accepted/running status is unchanged from ADR 0027.

## Compatibility and migration

The existing POST analysis-run, temporal-context, project-history, and GET status paths are unchanged. Production adapters may replace loopback while preserving the LineageWeave consumer header on status GET.

## Verification

Falsifiable evidence:

- LineageWeave status exchange sets only the published consumer header and uses GET;
- `tepp-loopback` create-then-GET over TCP returns `200` accepted without RMSE/scientific-acceptance keys;
- LineageWeave GET of its own run succeeds; Naruon GET of that run fails closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the LineageWeave status builder and the TCP GET proof; the ADR 0027 GET listener remains valid. A superseding ADR is required to persist status, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0027 owns the loopback GET listener and Naruon status builder.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
