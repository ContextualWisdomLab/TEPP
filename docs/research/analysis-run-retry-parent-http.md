# Analysis-run retry-parent HTTP (doctoring)

## Scope

`AnalysisRunLiveService` serves
`GET /v1/analysis-runs/{run_id}/parent` on a loopback-only HTTP/1.1 listener.
HTTP method, path, and header semantics follow current HTTP semantics
(Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of non-loopback
binds, table-access hosts, review/Copilot/GitHub credential headers, and
scientific-authority promotion is repository contract authority (ADR 0018;
ADR 0011; ADR 0038), not an RFC inference rule.

Parent responses are metric-free `AnalysisRunRetryParent` JSON. Each payload
carries the inspected run's `run_id`, `run_state`, and `idempotency_key`,
plus a `parent` object or JSON `null`. HTTP `200` is not a completed temporal
model, calibrated score, theta estimate, uncertainty statement, or scientific
claim. `tepp.scientific_acceptance.v1` never appears.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource's
current state. TEPP maps that retrieval onto a bounded, consumer-scoped
inspect of a retry child's parent identity. The RFC does not define
psychometric acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0038-analysis-run-retry-parent-get.md` — parent inspect
  authority and metric-free identity fields
- `docs/adr/0037-analysis-run-idempotency-lookup-get.md` — key-to-identity
  resolve without linkage
- `docs/adr/0035-analysis-run-retry-lineage-get.md` — parent→children inspect
- `docs/adr/0034-analysis-run-stored-request-get.md` — stored-request inspect
- `docs/adr/0032-analysis-run-retry-http.md` — retry clones without exposing
  parent/child linkage
- `docs/adr/0031-analysis-run-collection-get.md` — collection is
  identity-only and does not leak `retried_from`
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/API_CONTRACT.md` — documented retry-parent resource
- `crates/tepp_api/tests/analysis_run_retry_parent_http_contract.rs` —
  fail-closed parent exchange proofs

## Operator-visible behaviour

- loopback `GET /v1/analysis-runs/{run_id}/parent` of an original run
  returns `"parent": null`
- the same path of a retry child returns the parent's metric-free identity
- GET of the parent after retry still returns `"parent": null`
- collection GET still lists identity rows and does not leak `retried_from`
- stored-request GET still inspects snapshot/cutoff/model/profile
- retry-lineage GET still lists direct children of a parent `run_id`
- idempotency-key lookup still resolves a key to a durable `run_id`
- GET-by-id remains refused on this stack
- consumer mismatch, unknown identities, nonempty bodies, and metric keys
  fail closed
