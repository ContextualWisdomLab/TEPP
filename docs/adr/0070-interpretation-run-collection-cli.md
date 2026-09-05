# ADR 0070 — Contextual-orchestrator interpretation-run collection loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0069 for the operator-visible collection client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on this interpretation stack versus protected main; live vs-main and sibling GAP-003A PRs already occupy 0026–0069.

## Context

ADR 0069 serves `GET /v1/interpretation-runs` on `OrchestratorLiveService` /
`tepp-orchestrator-loopback`, but operators still had to write raw HTTP/1.1 to
enumerate accepted hypothetical runs. Duplicating interpretation-run CLI
(#425), collection GET (#433), project-history collection CLI (#428), GET-by-id
(#429), retrieval CLI (#431), analysis-run collection CLI (#371), Leiden,
Driver p.16, or GAP-010 Figma/export would collide with live PRs. Naruon and
`LineageWeave` are refused on this orchestrator-owned adapter;
`NaruonLiveService` stays POST-only.

## Decision

`orchestrator_live` publishes a loopback-only `tepp-interpretation-runs list`
verb:

- `list` mints `contextual_orchestrator_interpretation_run_collection_exchange`
  and renders through
  `loopback_http1_from_interpretation_run_collection_exchange` onto spawned
  `tepp-orchestrator-loopback` TCP. `--origin` stays the published HTTPS origin;
  only `--host` is the loopback bind address.
- Empty stdin is admitted. Consumer is `contextual-orchestrator` only.
- Optional `--page-cursor` / `--page-limit` become `tepp-page-cursor` /
  `tepp-page-limit` headers because the shared request-line parser fails
  closed on query strings.
- Stdout is the metric-free collection page: `interpretation_run_id`,
  `idempotency_key`, `orchestration_mode`, `claim_status=hypothetical`,
  `scientific_authority=false`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, evidence
  spans, tenant, budget, findings, and `causal_score` never appear.
- The CLI does not infer causality, mutate TEPP state, or return a completed
  psychometric result.
- Non-loopback hosts, `localhost`, credential-shaped flags, unknown verbs,
  nonempty stdin, unpublished consumers, naruon, LineageWeave, non-`https`
  origins, and hostile pagination fail closed.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep raw HTTP as the only collection path** — rejected because operators
   still guess framing after ADR 0069.
2. **Open naruon or LineageWeave on this adapter** — rejected; interpretation
   collection GET is contextual-orchestrator only (ADR 0069 / ADR 0010).
3. **Add GET to NaruonLiveService** — rejected; Naruon stays POST-only.
4. **Persist listed rows in PostgreSQL** — rejected as GAP-003B / live draft
   #287.
5. **Loopback collection CLI with the same metric-free gates as ADR 0069** —
   accepted.

## Consequences

- Operators can enumerate accepted hypothetical interpretation runs on the same
  loopback listener that created them without writing HTTP.
- Collection pages cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, metric keys,
nonempty bodies, slash/NUL cursors, zero or non-integer limits, unpublished
consumers, naruon, LineageWeave, and credential flags fail closed. The
in-memory registry is not durable. Non-200 bodies never reach stdout.
Successful pages must remain strictly ordered, respect the requested exclusive
cursor and limit, and bind any next cursor to the page's last row.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- Process exit 0 on a collection page is not measurement evidence and is not
  an ADR 0014 claim.

## Compatibility and migration

Collection GET, interpretation-run POST, and `tepp-interpretation-runs create`
remain unchanged. Production adapters may replace loopback while preserving
metric-free collection rows.

## Verification

Falsifiable evidence:

- CLI list JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/
  evidence/`findings`/`causal_score` keys;
- CLI list returns accepted hypothetical rows and refuses naruon and
  LineageWeave;
- non-loopback host, `localhost`, credential flags, nonempty stdin, and unknown
  verbs fail closed;
- Clippy `-D warnings`, `orchestrator_live` tests, rustdoc, and exact-head
  review remain required.

## Rollback and supersession

Rollback removes the `list` verb and client module; collection GET remains
valid. A superseding ADR is required to persist the registry, bind a public
address, emit scientific-acceptance on the list, open naruon or LineageWeave,
or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0069 owns loopback interpretation-run collection GET.
- ADR 0064 owns the interpretation-run POST CLI (live #425).
- ADR 0010 owns orchestration mode vocabulary and scientific-authority
  separation.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
