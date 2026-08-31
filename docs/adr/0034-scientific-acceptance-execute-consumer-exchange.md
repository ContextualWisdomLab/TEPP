# ADR 0034 — Scientific-acceptance execute consumer exchange

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0032 (engine execute) and ADR 0033 (published binary). Does not reuse ADR 0030 or ADR 0031. Does not supersede ADR 0014 claim-promotion authority.

## Context

ADR 0032 owns `POST /v1/analysis-runs/{run_id}/execute` on
`ScientificAcceptanceLoopbackService`. ADR 0033 publishes that wrapper as
`tepp-loopback`. Naruon and `LineageWeave` still have no typed execute
exchange: they would have to hand-roll HTTPS, path encoding, consumer
headers, and the metric-free execute body. Cancel consumer parity (#373) is a
different stack and a different verb. `tepp_api` cannot depend on
`analysis_engine` (crate cycle), so the execute body and exchange builders
belong in `analysis_engine`. Duplicating the published binary (#375),
engine-execute library (#370), cancel consumer parity (#373), loopback CLI
(#362), collection CLI (#371), retry (#369), GET, lifecycle POST, cancel
HTTP, collection GET, DTO, or engine-library slices would collide with live
PRs.

## Decision

`analysis_engine` owns typed execute consumer exchanges:

- `ScientificAcceptanceExecuteRequest` is the deny-unknown-fields execute
  body. It carries corpus, recovery, seed, and pre-registered SE-gate `k`.
  It refuses `scientific_acceptance_json`, receipt metric keys, and
  LLM-authored recovery.
- `naruon_analysis_run_execute_exchange` builds a credential-free HTTPS
  `POST /v1/analysis-runs/{run_id}/execute`.
- `lineageweave_analysis_run_execute_exchange` reuses that builder and
  replaces only the published `tepp-consumer` identity.
- Non-`https` origins, table-access hosts, empty or oversized run
  identities, unsupported contract versions, and hostile bodies fail closed.
- Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Cancel consumer parity, collection GET, loopback CLI, collection CLI, retry HTTP, or another published-binary move.

## Alternatives considered

1. **Keep hand-rolled execute HTTP in each consumer** — rejected because
   GAP-003A is operator-visible and create/GET/lifecycle already have typed
   exchanges.
2. **Add the builders to `tepp_api`** — rejected as a crate cycle once the
   body is the engine execute contract.
3. **Reuse the cancel consumer-parity PR** — rejected; cancel is a different
   verb on a different stack.

## Consequences

- Naruon and `LineageWeave` can mint `/execute` without embedding the
  library or inventing routes.
- HTTP 200 on execute is not release evidence.

## Failure and recovery

Non-`https` origins, table-access hosts, LLM recovery, metric keys, unknown
artifact fields, empty identities, and oversized run identifiers return a
fail-closed API error before any socket is opened. The in-memory registry is
not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Execute remains loopback-served, size-bounded, and content-redacting.
- LLM-authored recovery cannot become scientific authority.

## Compatibility and migration

Create, GET, running, terminal, temporal-context, and project-history
exchanges are unchanged. Production adapters may replace loopback while
preserving metric-free receipts and engine-produced scientific acceptance.

## Verification

Falsifiable evidence:

- naruon execute exchange is HTTPS POST `/execute` without credentials or
  `scientific_acceptance_json`;
- LineageWeave execute exchange changes only `tepp-consumer`;
- LLM recovery, metric keys, unknown artifact fields, and `http://` origins
  fail closed;
- POST create then the typed execute exchange then GET returns
  `tepp.scientific_acceptance.v1` for both consumers;
- Clippy `-D warnings`, `analysis_engine` and `tepp_api` tests, rustdoc, and
  exact-head review remain required.

## Rollback and supersession

Rollback removes the execute exchange builders; the engine wrapper and
published binary remain valid. A superseding ADR is required to persist
status, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0032 owns engine-on-loopback execute.
- ADR 0033 owns the published `tepp-loopback` binary.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
