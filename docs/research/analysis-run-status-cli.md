# Analysis-run status CLI (doctoring)

## Scope

`tepp-analysis-runs status` is the operator-visible client of loopback
`GET /v1/analysis-runs/{run_id}`. HTTP method, path, and header semantics
follow current HTTP semantics (Fielding, Nottingham, & Reschke, 2022).
Fail-closed refusal of non-loopback hosts, unpublished consumers,
review/Copilot/GitHub credential flags, and scientific-authority promotion is
repository contract authority (ADR 0029; ADR 0027; ADR 0018; ADR 0011), not an
RFC inference rule.

CLI stdout for accepted, running, and failed status is metric-free
`AnalysisRunStatus` JSON. `tepp.scientific_acceptance.v1` appears only on a
succeeded GET whose request profile is `scientific_acceptance_v1`. Process
exit 0 is not a completed temporal model, calibrated score, theta estimate,
uncertainty statement, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving a representation of
the target resource. TEPP maps that retrieval onto a bounded, consumer-scoped
status read of one analysis run. The RFC does not define psychometric
acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0029-analysis-run-status-cli.md` — this client
- `docs/adr/0027-scientific-acceptance-http-status.md` — status GET listener
- `docs/adr/0028-analysis-run-status-consumer-parity.md` — LineageWeave
  status-exchange builder
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented status resource
- `crates/tepp_api/tests/analysis_run_status_cli_contract.rs` —
  fail-closed status CLI proofs

## Verification

- `tepp-analysis-runs status` of an accepted run returns metric-free status
  without RMSE/bias/coverage/SE-gate keys or
  `tepp.scientific_acceptance.v1`;
- succeeded `scientific_acceptance_v1` status may print
  `tepp.scientific_acceptance.v1`;
- another consumer cannot read the first consumer's run;
- non-loopback hosts, credential flags, collection pagination flags, nonempty
  stdin, and unknown verbs fail closed;
- review, Copilot, GitHub, and bearer flags remain `AuthorizationDenied`.

## Non-claims

This slice does not implement GET-by-id HTTP, running/terminal POST, collection
GET, collection CLI list, cancel HTTP, cancel CLI, create CLI,
scientific-acceptance CLI verbs, status consumer-parity, persistence,
production TLS, Leiden consensus, or an ADR 0014 scientific claim-promotion
package.
