# Analysis-run stored-request CLI (doctoring)

## Scope

`tepp-analysis-runs stored-request` is the operator-visible client of loopback
`GET /v1/analysis-runs/{run_id}/request`. HTTP method, path, and header
semantics follow current HTTP semantics (Fielding, Nottingham, & Reschke,
2022). Fail-closed refusal of non-loopback hosts, unpublished consumers,
review/Copilot/GitHub credential flags, and scientific-authority promotion is
repository contract authority (ADR 0041; ADR 0034; ADR 0018; ADR 0011), not an
RFC inference rule.

CLI stdout is metric-free `AnalysisRunStoredRequest` JSON. Snapshot, cutoff,
model contract, and output profile are inspectable. `tepp.scientific_acceptance.v1`
never appears. Process exit 0 is not a completed temporal model, calibrated
score, theta estimate, uncertainty statement, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving a representation of
the target resource. TEPP maps that retrieval onto a bounded, consumer-scoped
inspect of stored create fields. The RFC does not define psychometric
acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0041-analysis-run-stored-request-cli.md` — this client
- `docs/adr/0034-analysis-run-stored-request-get.md` — stored-request GET listener
- `docs/adr/0040-analysis-run-stored-request-consumer-parity.md` — LineageWeave
  stored-request-exchange builder
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented stored-request resource
- `crates/tepp_api/tests/analysis_run_stored_request_cli_contract.rs` —
  fail-closed stored-request CLI proofs

## Verification

- `tepp-analysis-runs stored-request` of an accepted run returns metric-free
  snapshot/cutoff/model/profile without RMSE/bias/coverage/SE-gate keys or
  `tepp.scientific_acceptance.v1`;
- another consumer cannot inspect the first consumer's stored request;
- non-loopback hosts, credential flags, collection pagination flags, nonempty
  stdin, and unknown verbs fail closed;
- review, Copilot, GitHub, and bearer flags remain `AuthorizationDenied`.

## Non-claims

This slice does not implement stored-request GET HTTP, retry HTTP, retry CLI,
collection GET, collection CLI list, cancel HTTP, cancel CLI, create CLI,
status CLI, GET-by-id, persistence, production TLS, Leiden consensus, or an
ADR 0014 scientific claim-promotion package.
