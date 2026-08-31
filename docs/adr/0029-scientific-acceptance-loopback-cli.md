# ADR 0029 — Scientific-acceptance loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0027 and ADR 0028 for the operator-visible client. Does not supersede ADR 0014 claim-promotion authority and does not reuse ADR 0026, ADR 0027, or ADR 0028.

## Context

ADR 0027 serves `GET /v1/analysis-runs/{run_id}` and ADR 0028 serves production `POST /running` and `POST /terminal` on the loopback listener. Operators still had to write raw HTTP/1.1 to create a run, record running or terminal status, and read the result. Duplicating the GET listener, the lifecycle POST listener, the terminal-result DTO, or the `analysis_engine` library bind would collide with live PRs.

## Decision

`tepp_api` publishes a loopback-only `tepp-analysis-run` CLI:

- `create` POSTs a metric-free analysis-run request.
- `running` POSTs a metric-free running transition.
- `terminal` POSTs a request-bound terminal transition.
- `status` GETs the current status.
- Accepted and running stdout stay metric-free. Only a succeeded status whose request profile is `scientific_acceptance_v1` may print `tepp.scientific_acceptance.v1`.
- Non-loopback hosts, unpublished consumers, credential-shaped flags, reverse transitions, failed-plus-artifact emission, and receipt RMSE/bias/coverage/SE-gate keys fail closed.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from CLI success.

## Alternatives considered

1. **Keep raw HTTP as the only operator path** — rejected because the GET and POST listeners are not operator-usable without a client.
2. **Stack the client onto the live lifecycle POST PR as extra HTTP routes** — rejected because that head is already under review as a write-path slice.
3. **Persist CLI transcripts in PostgreSQL** — rejected as GAP-003B / live draft #287.
4. **Loopback CLI with the same metric-free and succeeded-only gates as ADR 0027/0028** — accepted.

## Consequences

- Operators can create, run, terminate, and read a scientific-acceptance analysis run on loopback without writing HTTP.
- GET remains the safe read (ADR 0027). POST remains the state change (ADR 0028 / RFC 9110 §9.3.3). The CLI is a client of those paths.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, metric keys on create/running, failed-plus-artifact emission, consumer mismatch, and a scientific-acceptance profile without an artifact fail closed. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- SHA-256 digest agreement is a byte-identity check, not a validity claim.
- Process exit 0 on a succeeded scientific-acceptance GET is not release evidence.

## Compatibility and migration

GET status, POST create, POST running/terminal, temporal-context, and project-history paths are unchanged. Production adapters may replace loopback while preserving metric-free receipts and the succeeded-only scientific-acceptance rule.

## Verification

Falsifiable evidence:

- CLI create and CLI running JSON have no RMSE/bias/coverage/SE-gate/scientific-acceptance keys;
- CLI terminal succeeded with profile `scientific_acceptance_v1` then CLI status prints `tepp.scientific_acceptance.v1` only when the artifact digest matches;
- CLI failed with an artifact, non-loopback host, credential flags, and consumer mismatch fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the CLI binary and client module; GET status and POST lifecycle remain valid. A superseding ADR is required to persist status, bind a public address, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0028 owns the HTTP write path.
- ADR 0027 owns the GET status read.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0022 owns deterministic execution to a digest-bound terminal result.
- ADR 0014 owns scientific claim promotion.
- ADR 0008 owns SHA-256 identity.
- ADR 0011 owns standalone/modular HTTP boundaries.
