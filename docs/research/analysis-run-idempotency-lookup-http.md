# Analysis-run idempotency-key lookup HTTP (doctoring)

## Scope

`AnalysisRunLiveService` serves
`GET /v1/analysis-runs/by-idempotency/{idempotency_key}` on a loopback-only
HTTP/1.1 listener. HTTP method, path, and header semantics follow current
HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal
of non-loopback binds, table-access hosts, review/Copilot/GitHub credential
headers, and scientific-authority promotion is repository contract authority
(ADR 0018; ADR 0011; ADR 0037), not an RFC inference rule.

Lookup responses are metric-free `AnalysisRunIdempotencyLookup` JSON. Each
payload carries `run_id`, `run_state`, and `idempotency_key` only. HTTP `200`
is not a completed temporal model, calibrated score, theta estimate,
uncertainty statement, or scientific claim.
`tepp.scientific_acceptance.v1` never appears.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource's
current state. TEPP maps that retrieval onto a bounded, consumer-scoped
resolution of an idempotency key to a durable run identity. The RFC does not
define psychometric acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0037-analysis-run-idempotency-lookup-get.md` — lookup
  authority and metric-free identity fields
- `docs/adr/0035-analysis-run-retry-lineage-get.md` — retry-lineage inspect
- `docs/adr/0034-analysis-run-stored-request-get.md` — stored-request inspect
- `docs/adr/0032-analysis-run-retry-http.md` — retry mints a new key
- `docs/adr/0031-analysis-run-collection-get.md` — collection is
  cursor-paginated identity only
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/API_CONTRACT.md` — documented idempotency-lookup resource
- `crates/tepp_api/tests/analysis_run_idempotency_lookup_http_contract.rs` —
  fail-closed lookup exchange proofs

## Operator-visible behaviour

- loopback `GET /v1/analysis-runs/by-idempotency/{key}` of a create key
  returns the metric-free `run_id` of that accepted, failed, or cancelled run
- the same path of a retry child key returns the cloned attempt
- collection GET still lists identity rows and does not become a key index
- stored-request GET still inspects snapshot/cutoff/model/profile
- retry-lineage GET still lists direct children of a parent `run_id`
- GET-by-id remains refused on this stack
- consumer mismatch, unknown keys, nonempty bodies, and metric keys fail
  closed
