# Analysis-run collection HTTP (doctoring)

## Scope

`AnalysisRunLiveService` serves `GET /v1/analysis-runs` on a loopback-only
HTTP/1.1 listener. HTTP method, path, and header semantics follow current HTTP
semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of
non-loopback binds, table-access hosts, review/Copilot/GitHub credential
headers, and scientific-authority promotion is repository contract authority
(ADR 0018; ADR 0011; ADR 0031), not an RFC inference rule.

Collection responses are metric-free `AnalysisRunCollection` JSON. Each row
carries `run_id`, `run_state`, and `idempotency_key` only. HTTP `200` is not a
completed temporal model, calibrated score, theta estimate, uncertainty
statement, or scientific claim. `tepp.scientific_acceptance.v1` never appears
on the list.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource's
current state. TEPP maps that retrieval onto a bounded, consumer-scoped
collection of metric-free run rows. The RFC does not define psychometric
acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0031-analysis-run-collection-get.md` — collection authority and
  metric-free list rows
- `docs/adr/0029-analysis-run-cancel-http.md` — cancelled is a listable
  metric-free state
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented collection resource
- `crates/tepp_api/tests/analysis_run_collection_http_contract.rs` —
  fail-closed collection exchange proofs

## Verification

- loopback `GET /v1/analysis-runs` of accepted, running, cancelled, succeeded,
  and failed runs returns metric-free rows without RMSE/bias/coverage/SE-gate
  keys or `tepp.scientific_acceptance.v1`;
- another consumer cannot read the first consumer's rows;
- unknown cursor, GET-by-id, query strings, and nonempty GET bodies fail
  closed;
- review, Copilot, GitHub, and bearer headers remain `AuthorizationDenied`.

## Non-claims

This slice does not implement GET-by-id, running/terminal POST, loopback CLI,
persistence, production TLS, Leiden consensus, or an ADR 0014 scientific
claim-promotion package.
