# ADR 0067 — LineageWeave project-history GET-by-id loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0066 for the operator-visible retrieval client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on protected main; live vs-main PRs occupy 0026–0066 (#430 occupies 0066 vs-main) and stacked #428 occupies 0065. Stacked #429 occupies 0066 on this lineage.

## Context

ADR 0066 / #429 serves `GET /v1/project-histories/{idempotency_key}` on
`AnalysisRunLiveService` / `tepp-loopback`, but operators still had to write
raw HTTP/1.1 to recover one stored cutoff-safe projection. Duplicating
collection GET (#424), collection CLI (#428), GET-by-id HTTP (#429),
project-history POST CLI (#420), temporal-context CLI (#414), export retrieval
CLI (#417), stored-request CLI (#395), analysis-run GET-by-id, Leiden, Driver
p.16, or GAP-010 Figma/export would collide with live PRs. Naruon is refused
on this adapter; `NaruonLiveService` stays POST-only.

## Decision

`tepp_api` publishes a loopback-only `tepp-project-history-get` CLI:

- `get` mints `lineageweave_project_history_retrieval_exchange` and renders
  through `loopback_http1_from_project_history_retrieval_exchange` onto
  spawned `tepp-loopback` TCP. `--origin` stays the published HTTPS origin;
  only `--host` is the loopback bind address.
- Empty stdin is admitted. Consumer is `lineageweave` only.
- `--idempotency-key` is the path identity and required `--tenant-workspace-id` selects its authorized tenant registry. Pagination flags fail closed.
- A successful GET returns an identity-bound receipt; the CLI compares both tenant and idempotency key before printing only the validated projection.
- Stdout is the stored `ProjectHistoryProjection`. `inference_status` remains
  `temporal_association_only`. Evidence text and findings belong to the stored
  projection.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, and
  `causal_score` never appear.
- The CLI does not infer causality, mutate TEPP state, or return a completed
  psychometric result.
- Non-loopback hosts, `localhost`, credential-shaped flags, unknown verbs,
  nonempty stdin, unpublished consumers, naruon, non-`https` origins, control
  identities, and pagination headers fail closed. Percent-encoded `/` keys
  round-trip because the POST contract admits them.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep raw HTTP as the only retrieval path** — rejected because operators
   still guess framing after ADR 0066.
2. **Add `get` onto the live project-history POST CLI (#420)** — rejected
   because that head owns POST query against a different live PR.
3. **Add `get` onto collection CLI (#428)** — rejected; that sibling stack is
   not this GET-by-id lineage.
4. **Open naruon on this adapter** — rejected; project-history GET-by-id is
   LineageWeave-only (ADR 0066 / ADR 0021).
5. **Loopback retrieval CLI with the same fail-closed gates as ADR 0066** —
   accepted.

## Consequences

- Operators can recover one accepted projection from a collection identity
  without writing HTTP or replaying POST.
- Retrieval stdout cannot be mistaken for a succeeded scientific-acceptance
  result or a causal score.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, metric keys,
nonempty bodies, unpublished consumers, naruon, and credential flags fail
closed. The in-memory registry is not durable.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and size-bounded.
- Process exit 0 on retrieval is not measurement evidence and is not an
  ADR 0014 claim.

## Compatibility and migration

GET-by-id HTTP, collection GET, project-history POST, and analysis-run paths
are unchanged. The POST CLI binary name `tepp-project-history` remains owned
by ADR 0061 / #420. The collection CLI binary `tepp-project-histories` remains
owned by ADR 0065 / #428.

## Verification

Falsifiable evidence:

- CLI get JSON is a stored `temporal_association_only` projection without
  RMSE/bias/coverage/SE-gate/`tepp.scientific_acceptance.v1`/`causal_score`;
- CLI get returns an accepted LineageWeave projection and refuses naruon;
- non-loopback host, credential flags, nonempty stdin, pagination flags, and
  unknown verbs fail closed;
- `NaruonLiveService` still refuses the composed GET;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the `tepp-project-history-get` binary and client module;
GET-by-id HTTP remains valid. A superseding ADR is required to persist the
registry, bind a public address, emit scientific-acceptance on retrieval, open
naruon, or treat CLI success as an ADR 0014 claim.

## Related authority

- ADR 0066 owns loopback project-history GET-by-id.
- ADR 0028 owns loopback project-history collection GET.
- ADR 0061 owns the project-history POST CLI (live #420).
- ADR 0021 owns the LineageWeave project-history POST boundary.
- ADR 0018 owns consumer-scoped ingress and metric-free receipts.
- ADR 0014 owns scientific claim promotion.
- ADR 0011 owns standalone/modular HTTP boundaries.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It
  does not authorize scientific claims.
