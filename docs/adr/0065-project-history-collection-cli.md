# ADR 0065 — LineageWeave project-history collection loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0028 for the operator-visible collection client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on protected main; live vs-main PRs already occupy 0026–0064.

## Context

ADR 0028 serves `GET /v1/project-histories` on `AnalysisRunLiveService` /
`tepp-loopback`, but operators still had to write raw HTTP/1.1 to enumerate
accepted cutoff-safe projections. Duplicating project-history POST CLI (#420),
collection GET (#424), temporal-context CLI (#414), export CLIs (#410/#417),
analysis-run collection CLI (#371), GET-by-id, Leiden, Driver p.16, or GAP-010
Figma/export would collide with live PRs. Naruon is refused on this adapter;
`NaruonLiveService` stays POST-only.

## Decision

`tepp_api` publishes a loopback-only `tepp-project-histories` CLI:

- `list` mints `lineageweave_project_history_collection_exchange` and renders
  through `loopback_http1_from_project_history_collection_exchange` onto
  spawned `tepp-loopback` TCP. `--origin` stays the published HTTPS origin;
  only `--host` is the loopback bind address.
- Empty stdin is admitted. Consumer is `lineageweave` only.
- Optional `--page-cursor` / `--page-limit` become `tepp-page-cursor` /
  `tepp-page-limit` headers because the shared request-line parser fails
  closed on query strings.
- Stdout is the metric-free collection page: `project_key`,
  `idempotency_key`, `knowledge_cutoff`,
  `inference_status=temporal_association_only`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, evidence
  text, findings, and `causal_score` never appear.
- The CLI does not infer causality, mutate TEPP state, or return a completed
  psychometric result.
- Non-loopback hosts, `localhost`, credential-shaped flags, unknown verbs,
  nonempty stdin, unpublished consumers, naruon, non-`https` origins, and
  hostile pagination fail closed.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep raw HTTP as the only collection path** — rejected because operators
   still guess framing after ADR 0028.
2. **Add `list` onto the live project-history POST CLI (#420)** — rejected
   because that head owns POST query against a different live PR and is not
   stacked on collection GET.
3. **Open naruon on this adapter** — rejected; project-history collection GET
   is LineageWeave-only (ADR 0028 / ADR 0021).
4. **Persist listed rows in PostgreSQL** — rejected as GAP-003B / live draft
   #287.
5. **Loopback collection CLI with the same metric-free gates as ADR 0028** —
   accepted.

## Consequences

- Operators can enumerate accepted project-history projections on the same
  loopback listener that created them without writing HTTP.
- Collection pages cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, metric keys,
nonempty bodies, unknown cursors, zero or non-integer limits, unpublished
consumers, naruon, and credential flags fail closed. The in-memory registry
is not durable. Non-200 bodies never reach stdout; failures emit only the
stable redacted API error on stderr. Successful pages must remain strictly
ordered, respect the requested exclusive cursor and limit, and bind any next
cursor to the page's last row.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- Process exit 0 on a collection page is not measurement evidence and is not
  an ADR 0014 claim.

## Compatibility and migration

Collection GET, project-history POST, temporal-context, and analysis-run
paths are unchanged. The project-history POST CLI binary name
`tepp-project-history` remains owned by ADR 0061 / #420. Production adapters
may replace loopback while preserving metric-free collection rows.

## Verification

Falsifiable evidence:

- CLI list JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/
  evidence/`findings`/`causal_score` keys;
- CLI list returns accepted LineageWeave rows and refuses naruon;
- non-loopback host, credential flags, nonempty stdin, and unknown verbs fail
  closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the `tepp-project-histories` binary and client module;
collection GET remains valid. A superseding ADR is required to persist the
registry, bind a public address, emit scientific-acceptance on the list, open
naruon, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0028 owns loopback project-history collection GET.
- ADR 0061 owns the project-history POST CLI (live #420).
- ADR 0021 owns the LineageWeave project-history POST boundary.
- ADR 0018 owns consumer-scoped ingress and metric-free receipts.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
