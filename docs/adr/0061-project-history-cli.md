# ADR 0061 — LineageWeave project-history loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0021 and ADR 0011 for the operator-visible project-history client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on protected main; live vs-main PRs already occupy 0026–0060.

## Context

Protected main already serves `POST /v1/project-histories` on
`AnalysisRunLiveService` / `tepp-loopback`, but LineageWeave operators still
had to write raw HTTP/1.1. Duplicating temporal-context CLI (#414), export
CLIs (#410/#417), analysis-run CLIs, GET-by-id, Leiden, Driver p.16, or
GAP-010 Figma/export would collide with live PRs. Naruon is refused on this
adapter; `NaruonLiveService` stays POST-only for analysis-run and export.

## Decision

`tepp_api` publishes a loopback-only `tepp-project-history query` verb:

- `query` mints `lineageweave_project_history_exchange` and renders through
  `loopback_http1_from_project_history_exchange` onto spawned `tepp-loopback`
  TCP. `--origin` stays the published HTTPS origin; only `--host` is the
  loopback bind address.
- Stdin is `ProjectHistoryRequest` JSON. Consumer is `lineageweave` only.
- The idempotency key travels in the typed exchange header from the request
  body. HTTP control characters fail closed, and the raw serializer revalidates
  the exact four-header set against the typed body. Operators do not pass a
  separate credential flag.
- Stdout is the cutoff-safe `ProjectHistoryProjection`.
  `inference_status` remains `temporal_association_only`.
  `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, and
  `causal_score` keys never appear.
- The CLI does not infer causality, mutate TEPP state, or return a completed
  psychometric result.
- Non-loopback hosts, `localhost`, credential-shaped flags, unknown verbs,
  empty stdin, unpublished consumers, naruon, non-`https` origins, and metric
  keys fail closed.
- Stdin and response bodies are bounded by the existing 256 KiB project-history
  wire limit. Response headers use the existing loopback header limits;
  duplicate framing, transfer encoding, and non-2xx bodies never reach stdout.
- This slice does not implement temporal-context CLI, export CLI, or
  analysis-run HTTP.

## Alternatives considered

1. **Keep raw HTTP as the only project-history path** — rejected because
   operators still guess framing after ADR 0021.
2. **Add `query` onto `tepp-temporal-context`** — rejected because
   project-history is a distinct LineageWeave projection, not a temporal-context
   verb, and #414 is a live PR.
3. **Open naruon on this adapter** — rejected; `AnalysisRunLiveService`
   project-history is LineageWeave-only (ADR 0021).
4. **Return scientific-acceptance on ordered events** — rejected because
   project-history bodies must stay metric-free.
5. **Loopback project-history CLI against `tepp-loopback`** — accepted.

## Consequences

- Operators can request a cutoff-safe project-history projection without
  writing HTTP.
- Project-history stdout cannot be mistaken for a succeeded
  scientific-acceptance result or a causal score.
- CLI success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, empty stdin,
credential flags, naruon consumer codes, and events unavailable at cutoff fail
closed. The in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Header injection, duplicate headers, unbounded reads, and ambiguous HTTP/1.1
  response framing fail closed before operator output.
- The CLI remains loopback-only. Event identities stay opaque; free-text PII
  is not introduced by this client.
- Process exit 0 on query is not measurement evidence and is not a causal
  claim.

## Compatibility and migration

`POST /v1/analysis-runs`, `POST /v1/exports`, `POST /v1/temporal-context`, and
`tepp-loopback` paths are unchanged. Temporal-context CLI remains #414.

## Verification

Falsifiable evidence:

- CLI query of a cutoff-safe LineageWeave body returns
  `temporal_association_only` with no RMSE/bias/coverage/SE-gate/
  `tepp.scientific_acceptance.v1`/`causal_score` keys;
- non-loopback host, `localhost`, credential flags, empty stdin, naruon,
  unknown verbs, and metric keys fail closed;
- `NaruonLiveService` still refuses `POST /v1/project-histories`;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes `tepp-project-history`; `POST /v1/project-histories` remains
valid. A superseding ADR is required to persist the registry, bind a public
address, emit scientific-acceptance on project-history, infer causality, open
naruon on this adapter, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0021 owns the LineageWeave project-history service boundary.
- ADR 0019 owns symmetric project-history wire-size enforcement.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
