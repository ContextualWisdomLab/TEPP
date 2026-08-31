# Analysis-run stored-request HTTP (doctoring)

## Scope

`AnalysisRunLiveService` serves `GET /v1/analysis-runs/{run_id}/request` on a
loopback-only HTTP/1.1 listener. HTTP method, path, and header semantics
follow current HTTP semantics (Fielding, Nottingham, & Reschke, 2022).
Fail-closed refusal of non-loopback binds, table-access hosts,
review/Copilot/GitHub credential headers, and scientific-authority promotion
is repository contract authority (ADR 0018; ADR 0011; ADR 0034), not an RFC
inference rule.

Stored-request responses are metric-free `AnalysisRunStoredRequest` JSON.
Each payload carries `run_id`, `run_state`, `idempotency_key`, `snapshot_id`,
`knowledge_cutoff`, `model_contract_version`, and `output_profile` only.
HTTP `200` is not a completed temporal model, calibrated score, theta
estimate, uncertainty statement, or scientific claim.
`tepp.scientific_acceptance.v1` never appears.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource's
current state. TEPP maps that retrieval onto a bounded, consumer-scoped
inspect of stored create fields. The RFC does not define psychometric
acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0034-analysis-run-stored-request-get.md` — stored-request
  authority and metric-free inspect fields
- `docs/adr/0032-analysis-run-retry-http.md` — retry clones after inspect
- `docs/adr/0031-analysis-run-collection-get.md` — collection lists identity
  only
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented stored-request resource
- `crates/tepp_api/tests/analysis_run_stored_request_http_contract.rs` —
  fail-closed stored-request exchange proofs

## Verification

- loopback `GET /v1/analysis-runs/{run_id}/request` of failed and cancelled
  runs returns snapshot, cutoff, model contract, and output profile without
  RMSE/bias/coverage/SE-gate keys or `tepp.scientific_acceptance.v1`;
- another consumer cannot read the first consumer's stored request;
- GET-by-id, query strings, nonempty GET bodies, and unknown identities fail
  closed;
- review, Copilot, GitHub, and bearer headers remain `AuthorizationDenied`.

## Non-claims

This slice does not implement GET-by-id, running/terminal POST, cancel HTTP,
collection GET, retry HTTP, loopback CLI, persistence, production TLS, Leiden
consensus, or an ADR 0014 scientific claim-promotion package.
