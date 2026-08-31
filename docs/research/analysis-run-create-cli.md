# Analysis-run create CLI (doctoring)

## Scope

`tepp-analysis-runs create` is the operator-visible client of loopback
`POST /v1/analysis-runs`. HTTP method, path, and header semantics follow
current HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed
refusal of non-loopback hosts, unpublished consumers, review/Copilot/GitHub
credential flags, and scientific-authority promotion is repository contract
authority (ADR 0034; ADR 0018; ADR 0011), not an RFC inference rule.

CLI stdout is metric-free `AnalysisRunAccepted` JSON with `run_state=accepted`.
The receipt carries `run_id`, `run_state`, and `idempotency_key` only.
Process exit 0 is not a completed temporal model, calibrated score, theta
estimate, uncertainty statement, or scientific claim.
`tepp.scientific_acceptance.v1` never appears.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing the enclosed
representation according to the resource's own semantics. TEPP maps that
processing onto a bounded, consumer-scoped create of a metric-free analysis
run. The RFC does not define psychometric acceptance, RMSE, or claim
promotion.

### Internal contract evidence

- `docs/adr/0034-analysis-run-create-cli.md` — this client
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/adr/0032-analysis-run-collection-cli.md` — distinct `list` verb on the
  same `tepp-analysis-runs` binary
- `docs/adr/0033-analysis-run-cancel-cli.md` — distinct `cancel` verb on the
  same `tepp-analysis-runs` binary
- `docs/adr/0030-scientific-acceptance-loopback-cli.md` — distinct
  `tepp-analysis-run` scientific-acceptance CLI on live #362
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented create resource
- `crates/tepp_api/tests/analysis_run_create_cli_contract.rs` —
  fail-closed create CLI proofs

## Verification

- `tepp-analysis-runs create` of a valid request returns metric-free accepted
  status without RMSE/bias/coverage/SE-gate keys or
  `tepp.scientific_acceptance.v1`;
- replay of the same create is idempotent;
- another consumer cannot collide with the first consumer's idempotency key;
- non-loopback hosts, credential flags, collection pagination flags, cancel
  `--run-id`, metric stdin, empty stdin, and unknown verbs fail closed;
- review, Copilot, GitHub, and bearer flags remain `AuthorizationDenied`.

## Non-claims

This slice does not implement GET-by-id, running/terminal POST, collection GET,
collection CLI list, cancel HTTP, cancel CLI, scientific-acceptance CLI verbs,
consumer-parity cancel, persistence, production TLS, Leiden consensus, or an
ADR 0014 scientific claim-promotion package.
