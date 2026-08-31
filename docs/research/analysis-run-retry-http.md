# Analysis-run retry HTTP (doctoring)

## Scope

`AnalysisRunLiveService` serves `POST /v1/analysis-runs/{run_id}/retry` on a
loopback-only HTTP/1.1 listener. HTTP method, path, and header semantics
follow current HTTP semantics (Fielding, Nottingham, & Reschke, 2022).
Fail-closed refusal of non-loopback binds, table-access hosts,
review/Copilot/GitHub credential headers, and scientific-authority promotion
is repository contract authority (ADR 0018; ADR 0011; ADR 0032), not an RFC
inference rule.

Retry responses are metric-free `AnalysisRunAccepted` JSON with a new `run_id`
and a new idempotency key. HTTP `202` is not a completed temporal model,
calibrated score, theta estimate, uncertainty statement, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing the request
according to the resource's own semantics. TEPP maps that processing onto a
clone of a failed or cancelled run into a new accepted receipt. The RFC does
not define psychometric acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0032-analysis-run-retry-http.md` — retry authority and
  metric-free new `202 Accepted`
- `docs/adr/0031-analysis-run-collection-get.md` — collection lists parent
  and child without scientific-acceptance keys
- `docs/adr/0029-analysis-run-cancel-http.md` — cancelled is retryable
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented retry resource
- `crates/tepp_api/tests/analysis_run_retry_http_contract.rs` — fail-closed
  retry exchange proofs

## Verification

- loopback `POST /v1/analysis-runs/{run_id}/retry` of a failed run returns
  `202` accepted JSON without RMSE/bias/coverage/SE-gate keys;
- cancelled runs retry to the same metric-free accepted receipt family;
- replaying the same new idempotency key returns the same child;
- accepted, running, succeeded, and unknown runs fail closed;
- GET `/v1/analysis-runs/{run_id}` remains `400` on this slice;
- review, Copilot, GitHub, and bearer headers remain `AuthorizationDenied`.

## Non-claims

This slice does not implement GET-by-id, running/terminal POST, cancel HTTP,
collection GET, loopback CLI, persistence, production TLS, Leiden consensus,
or an ADR 0014 scientific claim-promotion package.
