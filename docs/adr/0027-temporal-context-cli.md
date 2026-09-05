# ADR 0027 — LineageWeave temporal-context loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0002 and ADR 0011 for the operator-visible temporal-context client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on protected main; other live PRs may reuse 0027 on unrelated GAP-003A stacks (GET-by-id status).

## Context

Protected main already serves `POST /v1/temporal-context` on `AnalysisRunLiveService` / `tepp-loopback`, but LineageWeave operators still had to write raw HTTP/1.1. Duplicating analysis-run CLIs (#362/#371/#378/#385/#392/#394/#395/#397/#400/#401/#403/#406), export CLI (#410), export retrieval GET (#411), GET-by-id, Leiden, Driver p.16, or GAP-010 Figma/export would collide with live PRs.

## Decision

`tepp_api` publishes a loopback-only `tepp-temporal-context query` verb:

- `query` POSTs `/v1/temporal-context` to `tepp-loopback` with `--host`. Stdin is `TemporalContextRequest` JSON. Consumer is `lineageweave` only.
- No `idempotency-key` header. Temporal-context is a bounded read, not a durable analysis-run create.
- Stdout is the cutoff-safe `TemporalContextResponse`. `claim_boundary` remains `association_not_causal`. `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, and `causal_score` keys never appear.
- The CLI does not infer causality, mutate TEPP state, or return a completed psychometric result.
- Non-loopback hosts, credential-shaped flags, unknown verbs, empty stdin, unpublished consumers, and metric keys fail closed.
- This slice does not implement project-history CLI, export CLI, or analysis-run HTTP.

## Alternatives considered

1. **Keep raw HTTP as the only temporal-context path** — rejected because operators still guess framing after ADR 0011.
2. **Add `query` onto `tepp-analysis-runs`** — rejected because temporal-context is a LineageWeave read, not an analysis-run verb.
3. **Return scientific-acceptance on ordered events** — rejected because temporal-context bodies must stay metric-free.
4. **Loopback temporal-context CLI against `tepp-loopback`** — accepted.

## Consequences

- Operators can request a cutoff-safe temporal context without writing HTTP.
- Temporal-context stdout cannot be mistaken for a succeeded scientific-acceptance result or a causal score.
- CLI success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, empty stdin, credential flags, naruon consumer codes, and events unavailable at cutoff fail closed. The in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only. Event identities stay opaque; free-text PII is not introduced.
- Process exit 0 on query is not measurement evidence and is not a causal claim.

## Compatibility and migration

`POST /v1/analysis-runs`, `POST /v1/exports`, `POST /v1/project-histories`, and `tepp-loopback` paths are unchanged. Project-history CLI remains a later slice.

## Verification

Falsifiable evidence:

- CLI query of a cutoff-safe LineageWeave body returns `association_not_causal` with no RMSE/bias/coverage/SE-gate/`tepp.scientific_acceptance.v1`/`causal_score` keys;
- non-loopback host, credential flags, empty stdin, unknown verbs, and metric keys fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes `tepp-temporal-context`; `POST /v1/temporal-context` remains valid. A superseding ADR is required to persist the registry, bind a public address, emit scientific-acceptance on temporal-context, infer causality, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0002 owns six-clock temporal eligibility.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
