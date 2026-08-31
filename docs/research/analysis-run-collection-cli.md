# Analysis-run collection CLI (doctoring)

## Scope

`tepp-analysis-runs list` is the operator-visible client of loopback
`GET /v1/analysis-runs`. HTTP method, path, and header semantics follow current
HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of
non-loopback hosts, unpublished consumers, review/Copilot/GitHub credential
flags, and scientific-authority promotion is repository contract authority
(ADR 0032; ADR 0031; ADR 0018; ADR 0011), not an RFC inference rule.

CLI stdout is metric-free `AnalysisRunCollection` JSON. Each row carries
`run_id`, `run_state`, and `idempotency_key` only. Process exit 0 is not a
completed temporal model, calibrated score, theta estimate, uncertainty
statement, or scientific claim. `tepp.scientific_acceptance.v1` never appears.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource's
current state. TEPP maps that retrieval onto a bounded, consumer-scoped
collection of metric-free run rows. The RFC does not define psychometric
acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0032-analysis-run-collection-cli.md` — this client
- `docs/adr/0031-analysis-run-collection-get.md` — collection GET listener
- `docs/adr/0030-scientific-acceptance-loopback-cli.md` — distinct
  `tepp-analysis-run` scientific-acceptance CLI on live #362
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented collection resource
- `crates/tepp_api/tests/analysis_run_collection_cli_contract.rs` —
  fail-closed collection CLI proofs

## Verification

- `tepp-analysis-runs list` of accepted and cancelled runs returns metric-free
  rows without RMSE/bias/coverage/SE-gate keys or `tepp.scientific_acceptance.v1`;
- another consumer cannot read the first consumer's rows;
- non-loopback hosts, credential flags, GET-by-id flags, nonempty stdin, and
  unknown verbs fail closed;
- review, Copilot, GitHub, and bearer flags remain `AuthorizationDenied`.

## Non-claims

This slice does not implement GET-by-id, running/terminal POST, cancel CLI,
scientific-acceptance CLI verbs, persistence, production TLS, Leiden consensus,
or an ADR 0014 scientific claim-promotion package.
