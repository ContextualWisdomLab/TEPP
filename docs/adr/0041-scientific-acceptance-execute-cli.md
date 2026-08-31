# ADR 0041 — Scientific-acceptance execute loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0037 (typed execute exchange on loopback TCP), ADR 0034 (typed execute exchanges), and ADR 0033 (published binary). Does not reuse ADR 0030–0040. Does not supersede ADR 0014 claim-promotion authority.

## Context

ADR 0034 mints typed naruon and `LineageWeave` execute exchanges. ADR 0037 renders those exchanges onto spawned `tepp-loopback` TCP. Operators still had to write a test or embed the library to POST `/execute`. Duplicating the TCP renderer (#382), execute builders (#381), published binary (#375), engine-execute (#370), lifecycle CLI (#362), create CLI (#385), cancel consumer-parity (#373), cancel CLI (#378), stored-request GET (#377), stored-request consumer-parity (#387), retry-children (#379), idempotency (#380), retry-parent (#384), collection GET (#368), GET (#359), lifecycle POST (#360), cancel HTTP (#361), collection CLI (#371), retry (#369), engine-library (#356), DTO (#358), persistence (#287), Leiden (#351), Driver p.16, CWC/Rubin/ESEM/OLS, GAP-010, or GAP-003C would collide with live PRs.

## Decision

`analysis_engine` publishes a loopback-only `tepp-execute` CLI:

- `execute` POSTs `/v1/analysis-runs/{run_id}/execute` by minting `naruon_analysis_run_execute_exchange` or `lineageweave_analysis_run_execute_exchange` and rendering through `loopback_http1_from_execute_exchange`.
- Stdin is the typed execute body. Empty stdin is refused. Optional `--run-id` and `--idempotency-key` must match the body when present.
- `--origin` remains the published HTTPS origin on the typed exchange. Only `--host` is the loopback bind address printed by `tepp-loopback`.
- Success stdout is the engine-produced `tepp.scientific_acceptance.v1` status. Non-success stdout must not carry that schema.
- Public bind hosts, `localhost`, unpublished consumers, credential-shaped flags, LLM recovery, receipt metric keys, `http://` origins, and unknown verbs fail closed.
- Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from CLI success.
- Another TCP renderer, execute-builder, published-binary, engine-execute, lifecycle CLI, create CLI, cancel, collection, retry, GET, or persistence slice.

## Alternatives considered

1. **Keep embedding the library or hand-rolling HTTP as the only operator path** — rejected because GAP-003A is operator-visible and create/cancel already have published CLIs.
2. **Add `execute` onto the live lifecycle CLI (#362) or collection CLI (#371/#385)** — rejected because those heads live in `tepp_api`, cannot depend on `analysis_engine`, and do not own the execute body.
3. **Treat `localhost` as loopback** — rejected; `localhost` is a name, not a loopback bind address.
4. **Persist CLI transcripts in PostgreSQL** — rejected as GAP-003B / live draft #287.

## Consequences

- Operators can POST `/execute` from a published CLI using the typed naruon/`LineageWeave` execute exchange against spawned `tepp-loopback` TCP.
- CLI success is not release evidence.

## Failure and recovery

Public bind hosts return authorization denied. `localhost`, empty stdin, LLM recovery, metric keys, unpublished consumers, credential flags, `http://` origins, and unknown verbs fail closed. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- LLM-authored recovery cannot become scientific authority.
- Process exit 0 on a succeeded execute is not an ADR 0014 claim.

## Compatibility and migration

Typed execute builders, the TCP renderer, and the published `tepp-loopback` binary are unchanged. Production adapters may replace loopback while preserving metric-free receipts and engine-produced scientific acceptance.

## Verification

Falsifiable evidence:

- public bind hosts and `localhost` fail closed without opening a socket;
- credential-shaped flags, empty stdin, LLM recovery, metric keys, and `http://` origins fail closed;
- naruon and `LineageWeave` `tepp-execute execute` against spawned `tepp-loopback` TCP print `tepp.scientific_acceptance.v1`;
- Clippy `-D warnings`, `analysis_engine` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the `tepp-execute` binary; typed builders, the TCP renderer, and `tepp-loopback` remain valid. A superseding ADR is required to persist status, bind a public address, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0037 owns typed execute exchanges on loopback TCP.
- ADR 0034 owns typed execute consumer exchanges.
- ADR 0033 owns the published `tepp-loopback` binary.
- ADR 0032 owns engine-on-loopback execute.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
