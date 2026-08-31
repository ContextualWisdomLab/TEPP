# ADR 0033 — Scientific-acceptance published loopback binary

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0032 (engine-on-loopback execute). Does not reuse ADR 0030, ADR 0031, or ADR 0032. Does not supersede ADR 0014 claim-promotion authority.

## Context

ADR 0032 wraps `AnalysisRunLiveService` so `POST /v1/analysis-runs/{run_id}/execute`
produces `tepp.scientific_acceptance.v1` without a caller-supplied artifact.
The published `tepp-loopback` binary still lived in `tepp_api` and bound the
raw listener, which refuses `/execute`. Operators therefore could not reach
engine execute on the packaged listener without embedding `analysis_engine`.
`tepp_api` cannot depend on `analysis_engine` (crate cycle). Duplicating the
engine-execute library (#370), loopback CLI (#362), collection CLI (#371),
GET, lifecycle POST, cancel, collection GET, retry, DTO, or engine-library
slices would collide with live PRs. Same-name binaries in two crates would
make `cargo --bin tepp-loopback` ambiguous.

## Decision

Move the published `tepp-loopback` binary into `analysis_engine`:

- The binary binds [`ScientificAcceptanceLoopbackService`](../../crates/analysis_engine/src/loopback_execute.rs)
  on the same CLI (`127.0.0.1:18081` by default; optional loopback address and
  request limit).
- The Dockerfile builds `-p analysis_engine --bin tepp-loopback`.
- `tepp_api` no longer ships that binary. Create, GET, running, terminal,
  temporal-context, and project-history stay delegated to the live service.
- `/execute` is reachable on the published listener without embedding the
  library. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Collection GET, cancel HTTP, loopback CLI, collection CLI, retry HTTP, or
  another engine-execute library wrapper.

## Alternatives considered

1. **Keep `tepp-loopback` in `tepp_api` and document library embedding** —
   rejected because GAP-003A is operator-visible and the packaged listener
   would still refuse `/execute`.
2. **Add `analysis_engine` as a `tepp_api` dependency** — rejected as a crate
   cycle.
3. **Ship a second binary name** — rejected because operators and the
   Dockerfile already call `tepp-loopback`.
4. **Move the same binary name onto the ADR 0032 wrapper** — accepted.

## Consequences

- Operators can run the published binary, POST create, POST execute, and GET
  `tepp.scientific_acceptance.v1` without supplying the artifact and without
  embedding `analysis_engine`.
- Temporal-context health checks remain valid.
- HTTP 200 on execute is not release evidence.

## Failure and recovery

Unknown run identities, extra path segments, metric keys, LLM recovery, wrong
profile, already-terminal runs, and consumer mismatch return a redacted `400`
envelope. Unsupported execute contract versions return `422`. Credential
headers remain `403`. Non-loopback bind remains fail-closed. The in-memory
registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The published listener remains loopback-only, size-bounded, and content-redacting.
- SHA-256 digest agreement is a byte-identity check, not a validity claim.
- LLM-authored recovery cannot become scientific authority.

## Compatibility and migration

CLI arguments, default bind `127.0.0.1:18081`, and the container entrypoint
name are unchanged. Callers that embedded `AnalysisRunLiveService` still see
`/execute` refused; they must use the wrapper or the published binary.
Production adapters may replace loopback while preserving metric-free receipts
and engine-produced scientific acceptance.

## Verification

Falsifiable evidence:

- the packaged binary still returns `200` for bounded temporal-context;
- POST create then POST execute without `scientific_acceptance_json` then GET
  through the spawned binary returns `tepp.scientific_acceptance.v1`;
- Clippy `-D warnings`, `analysis_engine` and `tepp_api` tests, rustdoc, and
  exact-head review remain required.

## Rollback and supersession

Rollback restores the `tepp_api` binary; ADR 0032 library execute remains
valid. A superseding ADR is required to persist status, bind a public address,
or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0032 owns engine-on-loopback execute.
- ADR 0026 owns the validation-run library bind.
- ADR 0027 owns the GET status read.
- ADR 0028 owns POST running/terminal.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
