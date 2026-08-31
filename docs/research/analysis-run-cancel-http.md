# Analysis-run cancel HTTP (doctoring)

## Scope

`AnalysisRunLiveService` serves `POST /v1/analysis-runs/{run_id}/cancel` on a
loopback-only HTTP/1.1 listener. HTTP method, path, and header semantics
follow current HTTP semantics (Fielding, Nottingham, & Reschke, 2022).
Fail-closed refusal of non-loopback binds, table-access hosts,
review/Copilot/GitHub credential headers, and scientific-authority promotion
is repository contract authority (ADR 0018; ADR 0011; ADR 0029), not an RFC
inference rule.

Cancelled responses are metric-free `AnalysisRunStatus` JSON with
`run_state = cancelled` and no `terminal_result`. HTTP `200` is not a
completed temporal model, calibrated score, theta estimate, uncertainty
statement, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing the request
according to the resource's own semantics. TEPP maps that processing onto an
atomic accepted/running → cancelled transition. The RFC does not define
psychometric acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0029-analysis-run-cancel-http.md` — cancel authority and
  metric-free cancelled status
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented cancel resource
- `crates/tepp_api/tests/analysis_run_cancel_http_contract.rs` — fail-closed
  cancel exchange proofs

## Verification

- loopback `POST /v1/analysis-runs/{run_id}/cancel` of an accepted run
  returns `200` cancelled status without RMSE/bias/coverage/SE-gate keys;
- running runs cancel to the same metric-free status;
- already-cancelled runs replay the same body;
- succeeded, failed, and unknown runs fail closed;
- GET `/v1/analysis-runs` remains `400` on this slice;
- review, Copilot, GitHub, and bearer headers remain `AuthorizationDenied`.

## Non-claims

This slice does not implement GET status, running/terminal POST, persistence,
production TLS, Leiden consensus, or an ADR 0014 scientific claim-promotion
package.
