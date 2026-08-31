# ADR 0029 — Analysis-run lifecycle consumer parity

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0028 and ADR 0018. Does not supersede ADR 0014. ADR 0029 on the cancel-HTTP lineage is a different stack; this ADR number is unique on the lifecycle-POST lineage.

## Context

ADR 0028 added `POST /v1/analysis-runs/{run_id}/running` and
`POST /v1/analysis-runs/{run_id}/terminal` on `AnalysisRunLiveService` plus
Naruon running/terminal exchange builders. The Naruon compatibility listener
(`NaruonLiveService`) still refused every lifecycle path. `LineageWeave` had a
create-exchange builder but no running/terminal exchange, so a published
consumer would have to mint a Naruon-labelled lifecycle POST. The packaged
`tepp-loopback` binary had no TCP proof that lifecycle POST works for
LineageWeave on the shared listener.

Duplicating the GET status listener, the lifecycle POST listener, cancel,
collection GET, retry, or engine-library slices would not close this
consumer-parity gap. `NaruonLiveService` stays POST-only: lifecycle is POST, so
the compatibility listener can serve it without adding GET.

## Decision

- `lineageweave_analysis_run_running_exchange` and
  `lineageweave_analysis_run_terminal_exchange` reuse the Naruon builders and
  replace only `tepp-consumer`.
- `NaruonLiveService` serves the same metric-free running/terminal path for the
  Naruon-only compatibility listener. LineageWeave consumers remain refused
  there; they use `AnalysisRunLiveService`.
- `tepp-loopback` proves create-then-running over loopback TCP.
- Accepted and running responses stay metric-free. Reverse transitions,
  mutating a terminal run, unknown runs, consumer/idempotency mismatch, and
  receipt metric keys fail closed.

## Non-goals

- GET status on `NaruonLiveService`.
- Duplicating the shared-listener lifecycle POST (ADR 0028).
- Cancel, collection GET, retry, persistence, or production TLS.
- Opening `NaruonLiveService` to LineageWeave.
- An ADR 0014 scientific claim.

## Alternatives considered

1. **Leave lifecycle only on `AnalysisRunLiveService`** — rejected because the
   compatibility listener would silently refuse a documented POST path.
2. **Admit LineageWeave on `NaruonLiveService`** — rejected because that
   listener is Naruon-only (ADR 0011/0018).
3. **Mint a second lifecycle DTO** — rejected as a duplicate of ADR 0028.
4. **Consumer-parity lifecycle on the existing typed transition** — accepted.

## Consequences

- Both published consumers can build a credential-free running/terminal
  exchange.
- Naruon local proofs can record lifecycle on either listener.
- Operators can observe LineageWeave running through `tepp-loopback` without a
  second HTTP stack.

## Failure and recovery

Unknown runs, consumer mismatch, idempotency mismatch, metric keys, reverse
transitions, mutating a terminal run, GET, and oversized identities fail closed
with a redacted envelope. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Lifecycle remains loopback-only. Running stays metric-free.
- HTTP `200` running/terminal is not measurement or release evidence.

## Compatibility and migration

ADR 0028 create/running/terminal semantics are unchanged. Production adapters
may replace loopback while preserving consumer identity, metric-free running
status, and Naruon-only compatibility-listener admission.

## Verification

- LineageWeave running/terminal exchanges carry `tepp-consumer: lineageweave`
  and no credentials;
- NaruonLiveService records accepted→running→terminal for Naruon and refuses
  LineageWeave, GET, metrics, unknown runs, and reverse transitions;
- `tepp-loopback` create-then-running over TCP returns metric-free `running`;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain
  required.

## Rollback and supersession

Rollback removes the LineageWeave builders, compatibility-listener lifecycle,
and binary TCP proof; ADR 0028 shared-listener lifecycle remains. A superseding
ADR is required to persist lifecycle, bind a public address, add GET to
`NaruonLiveService`, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0028 owns the shared-listener lifecycle POST path.
- ADR 0027 owns GET status.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
