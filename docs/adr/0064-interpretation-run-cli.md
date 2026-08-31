# ADR 0064 — Contextual-orchestrator interpretation-run loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0010 and ADR 0011 for the operator-visible interpretation-run client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on protected main; live vs-main PRs already occupy 0026–0063.

## Context

Protected main already serves `POST /v1/interpretation-runs` on
`OrchestratorLiveService`, but operators still had to write a custom binary
and raw HTTP/1.1. Duplicating analysis-run CLIs, export CLIs, temporal-context
CLI, project-history CLI, project-history collection GET, Leiden, Driver p.16,
or GAP-010 Figma/export would collide with live PRs. Naruon and `LineageWeave`
are refused on this orchestrator-owned adapter; `NaruonLiveService` stays
POST-only for analysis-run and export.

## Decision

`orchestrator_live` publishes loopback-only `tepp-orchestrator-loopback` and
`tepp-interpretation-runs create`:

- `tepp-orchestrator-loopback` binds `OrchestratorLiveService` on
  `127.0.0.1:18082` by default. Public bind fails closed.
- `create` mints `contextual_orchestrator_interpretation_run_exchange` and
  renders through `loopback_http1_from_interpretation_run_exchange` onto
  spawned `tepp-orchestrator-loopback` TCP. `--origin` stays the published
  HTTPS origin; only `--host` is the loopback bind address.
- Stdin is `InterpretationRunRequest` JSON. Consumer is
  `contextual-orchestrator` only.
- Stdout is the accepted hypothetical run. `claim_status` remains
  `hypothetical`. `scientific_authority` remains false.
  `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, and
  `causal_score` keys never appear.
- The CLI does not call a model provider, infer causality, or return a
  completed psychometric result.
- Non-loopback hosts, `localhost`, credential-shaped flags, unknown verbs,
  empty stdin, unpublished consumers, naruon, LineageWeave, non-`https`
  origins, and metric keys fail closed.

## Alternatives considered

1. **Keep raw HTTP as the only interpretation-run path** — rejected because
   operators still guess framing after the live listener shipped.
2. **Add `create` onto `tepp-analysis-runs`** — rejected because
   interpretation-run is a distinct orchestrator projection, and #385 is live.
3. **Open naruon or LineageWeave on this adapter** — rejected; the listener
   admits `contextual-orchestrator` only.
4. **Return scientific-acceptance on hypothetical runs** — rejected because
   ADR 0010 forbids treating orchestrator output as measurement truth.
5. **Loopback interpretation-run CLI against `tepp-orchestrator-loopback`** —
   accepted.

## Consequences

- Operators can request a hypothetical interpretation-run acknowledgement
  without writing HTTP.
- Interpretation-run stdout cannot be mistaken for a succeeded
  scientific-acceptance result or a causal score.
- CLI success is not release evidence and is not an ADR 0014 claim.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, empty stdin,
credential flags, naruon or LineageWeave consumer codes, and
`scientific_authority: true` fail closed. The in-memory listener is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only.
- Process exit 0 on create is not measurement evidence and is not a causal
  claim.

## Compatibility and migration

`POST /v1/analysis-runs`, `POST /v1/exports`, `POST /v1/temporal-context`,
`POST /v1/project-histories`, and `tepp-loopback` paths are unchanged.

## Verification

Falsifiable evidence:

- CLI create of a hypothetical body returns `claim_status` `hypothetical`
  with `scientific_authority` false and no RMSE/bias/coverage/SE-gate/
  `tepp.scientific_acceptance.v1`/`causal_score` keys;
- non-loopback host, `localhost`, credential flags, empty stdin, naruon,
  LineageWeave, unknown verbs, and metric keys fail closed;
- `tepp-orchestrator-loopback` serves one bounded POST and refuses public bind;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes `tepp-interpretation-runs` and `tepp-orchestrator-loopback`;
`POST /v1/interpretation-runs` remains valid as a library. A superseding ADR
is required to bind a public address, emit scientific-acceptance on
interpretation-run, infer causality, open naruon or LineageWeave on this
adapter, call a model provider, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0010 owns adaptive LLM orchestration and scientific-authority
  separation.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- ADR 0017 owns the hourly proposal gateway.
- RFC 9110 owns POST semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
