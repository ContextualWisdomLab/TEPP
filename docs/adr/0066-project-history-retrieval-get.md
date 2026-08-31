# ADR 0066 — LineageWeave project-history GET-by-id

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0028 and ADR 0021 for retrieving one accepted projection. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on protected main; live vs-main PRs occupy 0026–0064 and stacked #428 occupies 0065.

## Context

ADR 0028 enumerates accepted project-history projections as metric-free identities, and #428 publishes a collection CLI. Operators who hold an `idempotency_key` from that page still had to replay `POST /v1/project-histories` to recover the stored cutoff-safe projection. Duplicating collection GET (#424), collection CLI (#428), project-history POST CLI (#420), temporal-context CLI (#414), export retrieval GET (#411), analysis-run GET-by-id (#359), Leiden, Driver p.16, or GAP-010 Figma/export would collide with live PRs.

## Decision

`tepp_api` publishes loopback-only `GET /v1/project-histories/{idempotency_key}` on `tepp-loopback`:

- Consumer is `lineageweave` only. Empty body. The identity travels in the path.
- The response is the stored cutoff-safe `ProjectHistoryProjection`. `inference_status` remains `temporal_association_only`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, and `causal_score` never appear.
- Collection GET (`GET /v1/project-histories` with no extra segment) is unchanged.
- Pagination headers, naruon, nonempty bodies, extra path segments, and unknown keys fail closed.
- The retrieval does not infer causality, mutate TEPP state, or return a completed psychometric result.
- `NaruonLiveService` stays POST-only. This slice does not implement a retrieval CLI or persistence.

## Alternatives considered

1. **Keep POST replay as the only retrieval path** — rejected because operators still resubmit evidence after ADR 0028.
2. **Return only the collection row** — rejected; that identity is already on the list. GET-by-id recovers the stored projection.
3. **Reuse analysis-run GET-by-id (#359) or export retrieval GET (#411)** — rejected; those are different live resources.
4. **Loopback `GET /v1/project-histories/{idempotency_key}`** — accepted.

## Consequences

- Operators can retrieve one accepted projection from a collection identity without writing a second POST.
- Retrieval stdout cannot be mistaken for a succeeded scientific-acceptance result or a causal score.
- Retrieval success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-`lineageweave` consumers, nonempty GET bodies, collection pagination headers, extra path segments, unknown keys, credential flags, and metric keys fail closed. The in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The retrieval remains loopback-only and size-bounded.
- Process 200 on GET-by-id is not measurement evidence and is not a causal claim.

## Compatibility and migration

Collection GET, POST `/v1/project-histories`, temporal-context, and analysis-run paths are unchanged. A retrieval CLI remains a later slice.

## Verification

Falsifiable evidence:

- GET of an accepted projection returns `temporal_association_only` without RMSE/bias/coverage/SE-gate/`tepp.scientific_acceptance.v1`/`causal_score` keys;
- collection GET, naruon consumer, nonempty body, extra segments, and unknown keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes GET-by-id; collection GET and POST remain valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on retrieval, infer causality, or treat retrieval success as an ADR 0014 claim.

## Related authority

- ADR 0028 owns loopback collection GET.
- ADR 0021 owns the LineageWeave project-history service boundary.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
