# ADR 0037 — Scientific-acceptance execute exchange on loopback TCP

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0034 (typed execute exchanges) and ADR 0033 (published binary). Does not reuse ADR 0035 or ADR 0036. Does not supersede ADR 0014 claim-promotion authority.

## Context

ADR 0034 mints typed naruon and `LineageWeave` execute exchanges. ADR 0033 publishes `tepp-loopback` so `/execute` is reachable on the packaged listener. The typed exchanges were still proven only through in-memory `handle_http_request`. The published binary test still hand-rolled HTTP. Operators therefore could not send the typed consumer contract over the spawned TCP listener without inventing HTTP/1.1. Public bind hosts must fail closed. `localhost` is not a loopback exception. Duplicating the execute consumer-exchange builders (#381), published binary (#375), engine-execute library (#370), cancel consumer parity (#373), loopback CLI (#362), collection CLI (#371), retry (#369), GET, lifecycle POST, cancel HTTP, collection GET, DTO, or engine-library slices would collide with live PRs.

## Decision

`analysis_engine` owns HTTP/1.1 rendering of typed execute exchanges onto the spawned `tepp-loopback` TCP listener:

- `loopback_http1_from_naruon_exchange` renders a typed HTTPS exchange onto a bound loopback `Host`.
- `loopback_http1_from_execute_exchange` requires POST `/execute` with a naruon or `LineageWeave` consumer identity, then renders that exchange.
- The exchange keeps its HTTPS origin contract. Only `Host` is the loopback bind address printed by `tepp-loopback`.
- Public bind hosts, unparseable hosts including `localhost`, credential headers, empty methods, non-`https` targets, and non-execute exchanges fail closed before any socket is opened.
- Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable status storage.
- Leiden community detection, Driver p.16 std-family restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Another execute consumer-exchange builder, published-binary move, engine-execute library, cancel consumer parity, loopback CLI, collection CLI, or retry HTTP.

## Alternatives considered

1. **Keep hand-rolled HTTP in the binary test** — rejected because GAP-003A is operator-visible and the typed exchange would remain an in-memory-only contract.
2. **Add the renderer to `tepp_api`** — rejected for this slice; execute body ownership stays in `analysis_engine` and the crate cycle remains forbidden.
3. **Treat `localhost` as loopback** — rejected; `localhost` is a name, not a loopback bind address.

## Consequences

- Naruon and `LineageWeave` can POST `/execute` to the spawned listener from the typed exchange without embedding the library or inventing HTTP/1.1.
- HTTP 200 on execute is not release evidence.

## Failure and recovery

Public bind hosts return authorization denied. `localhost`, missing paths, GET status exchanges, and non-execute verbs return a fail-closed API error before any socket is opened. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The published listener remains loopback-only, size-bounded, and content-redacting.
- LLM-authored recovery cannot become scientific authority.

## Compatibility and migration

Create, GET, running, terminal, temporal-context, project-history, and typed execute builders are unchanged. Production adapters may replace loopback while preserving metric-free receipts and engine-produced scientific acceptance.

## Verification

Falsifiable evidence:

- public bind hosts and `localhost` fail closed without opening a socket;
- GET status exchanges are refused by the execute renderer;
- naruon and `LineageWeave` typed execute exchanges over spawned `tepp-loopback` TCP then GET return `tepp.scientific_acceptance.v1`;
- Clippy `-D warnings`, `analysis_engine` and `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the HTTP/1.1 renderer; the typed builders and published binary remain valid. A superseding ADR is required to persist status, bind a public address, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0034 owns typed execute consumer exchanges.
- ADR 0033 owns the published `tepp-loopback` binary.
- ADR 0032 owns engine-on-loopback execute.
- ADR 0018 owns consumer-scoped ingress and metric-free `202 Accepted`.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
