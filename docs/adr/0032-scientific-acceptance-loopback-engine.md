# ADR 0032 — Scientific-acceptance loopback engine execute

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0026 (engine library), ADR 0027 (GET), and ADR 0028 (lifecycle POST). Does not reuse ADR 0030 or ADR 0031. Does not supersede ADR 0014 claim-promotion authority.

## Context

ADR 0026 binds cutoff-safe evidence to a hash-stable validation run that emits
`tepp.scientific_acceptance.v1`. ADR 0027 serves GET status. ADR 0028 records
running and terminal status, but the terminal POST still requires
caller-supplied `scientific_acceptance_json`. Operators therefore cannot obtain
scientific-acceptance evidence from a `scientific_acceptance_v1` loopback run
without already possessing the artifact. Duplicating GET, lifecycle POST, the
engine library, collection GET, cancel HTTP, or the loopback CLI would collide
with live PRs. `analysis_engine` already depends on `tepp_api`; the reverse
dependency would cycle.

## Decision

`analysis_engine` owns `POST /v1/analysis-runs/{run_id}/execute` on a wrapper
around `AnalysisRunLiveService`:

- The execute body carries corpus, recovery vectors, seed, pre-registered
  SE-gate `k`, study label, completion time, and an explicit LLM-authorship
  flag. It must not carry `scientific_acceptance_json` or receipt metric keys.
- The wrapper calls `submit_validation_run` then `complete_validation_run`,
  records running then terminal through the public loopback recorder, and
  returns the same status body GET would return.
- Only an accepted run whose request profile is `scientific_acceptance_v1` and
  whose model is `validation_cpu_f64_v1` may execute. GET then returns
  `tepp.scientific_acceptance.v1` without a caller-supplied artifact.
- Wrong profile, LLM recovery, metric keys, unknown run, consumer mismatch,
  already-terminal status, digest mismatch, and unknown execute fields fail
  closed.
- `tepp_api` recognizes the `/execute` suffix and refuses it so the raw
  listener cannot pretend to execute. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Collection GET, cancel HTTP, or loopback CLI.

## Alternatives considered

1. **Keep caller-supplied terminal artifacts** — rejected because GAP-003A is
   the operator-visible product-completion gap and the engine already emits the
   schema.
2. **Add `analysis_engine` as a `tepp_api` dependency** — rejected as a crate
   cycle.
3. **Persist execute rows in PostgreSQL** — rejected as GAP-003B / live draft
   #287.
4. **Engine wrapper that records produced bytes on the existing lifecycle
   path** — accepted.

## Consequences

- Operators can POST create, POST execute, and GET `tepp.scientific_acceptance.v1`
  without supplying the artifact.
- Lifecycle POST remains the generic status-update path (ADR 0028). Execute is
  the engine-owned production of those bytes.
- HTTP 200 on execute is not release evidence.

## Failure and recovery

Unknown run identities, extra path segments, metric keys, LLM recovery, wrong
profile, already-terminal runs, and consumer mismatch return a redacted `400`
envelope. Unsupported execute contract versions return `422`. Credential
headers remain `403`. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Execute remains loopback-only, size-bounded, and content-redacting.
- SHA-256 digest agreement is a byte-identity check, not a validity claim.
- LLM-authored recovery cannot become scientific authority.

## Compatibility and migration

GET status, POST create, POST running/terminal, temporal-context, and
project-history paths are unchanged. Production adapters may replace loopback
while preserving metric-free receipts and engine-produced scientific
acceptance.

## Verification

Falsifiable evidence:

- POST create stays metric-free;
- POST execute without `scientific_acceptance_json` then GET returns
  `tepp.scientific_acceptance.v1`;
- wrong profile, LLM recovery, metric keys, unknown run, consumer mismatch,
  and a second execute on a terminal run fail closed;
- Clippy `-D warnings`, `analysis_engine` and `tepp_api` tests, rustdoc, and
  exact-head review remain required.

## Rollback and supersession

Rollback removes the execute wrapper; GET, lifecycle POST, and the engine
library remain valid. A superseding ADR is required to persist status, bind a
public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0026 owns the validation-run library bind.
- ADR 0027 owns the GET status read.
- ADR 0028 owns POST running/terminal.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0022 owns deterministic execution to a digest-bound terminal result.
- ADR 0014 owns scientific claim promotion.
- ADR 0008 owns SHA-256 identity.
- ADR 0011 owns standalone/modular HTTP boundaries.
