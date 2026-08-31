# Analysis-run retry-lineage HTTP (doctoring)

## Scope

`AnalysisRunLiveService` serves `GET /v1/analysis-runs/{run_id}/retries` on a
loopback-only HTTP/1.1 listener. HTTP method, path, and header semantics
follow current HTTP semantics (Fielding, Nottingham, & Reschke, 2022).
Fail-closed refusal of non-loopback binds, table-access hosts,
review/Copilot/GitHub credential headers, and scientific-authority promotion
is repository contract authority (ADR 0018; ADR 0011; ADR 0035), not an RFC
inference rule.

Retry-lineage responses are metric-free `AnalysisRunRetryLineage` JSON.
Each payload carries parent `run_id`, `run_state`, `idempotency_key`, and a
`retries` array of direct children (`run_id`, `run_state`, `idempotency_key`)
only. HTTP `200` is not a completed temporal model, calibrated score, theta
estimate, uncertainty statement, or scientific claim.
`tepp.scientific_acceptance.v1` never appears.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource's
current state. TEPP maps that retrieval onto a bounded, consumer-scoped
inspect of direct retry children. The RFC does not define psychometric
acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0035-analysis-run-retry-lineage-get.md` — retry-lineage
  authority and metric-free parent/child fields
- `docs/adr/0034-analysis-run-stored-request-get.md` — stored-request inspect
- `docs/adr/0032-analysis-run-retry-http.md` — retry clones without linkage
- `docs/adr/0031-analysis-run-collection-get.md` — collection lists identity
  only
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/API_CONTRACT.md` — documented retry-lineage resource
- `crates/tepp_api/tests/analysis_run_retry_lineage_http_contract.rs` —
  fail-closed retry-lineage exchange proofs

## Operator-visible behaviour

- loopback `GET /v1/analysis-runs/{run_id}/retries` of a failed or cancelled
  parent returns metric-free direct children after retry
- a never-retried parent returns `200` with an empty `retries` array
- collection GET still lists parent and child independently and does not
  leak `retried_from`
- stored-request GET still inspects snapshot/cutoff/model/profile and does
  not list children
- GET-by-id remains refused on this stack
- consumer mismatch, unknown identities, nonempty bodies, and metric keys
  fail closed
