# Analysis-run idempotency-lookup CLI (doctoring)

## Scope

`tepp-analysis-runs lookup` is the operator-visible client of loopback
`GET /v1/analysis-runs/by-idempotency/{idempotency_key}`. HTTP method, path,
and header semantics follow current HTTP semantics (Fielding, Nottingham, &
Reschke, 2022). Fail-closed refusal of non-loopback hosts, unpublished
consumers, review/Copilot/GitHub credential flags, and scientific-authority
promotion is repository contract authority (ADR 0038; ADR 0037; ADR 0018;
ADR 0011), not an RFC inference rule.

CLI stdout is metric-free `AnalysisRunIdempotencyLookup` JSON. `run_id`,
`run_state`, and `idempotency_key` are inspectable.
`tepp.scientific_acceptance.v1` never appears. Process exit 0 is not a
completed temporal model, calibrated score, theta estimate, uncertainty
statement, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving a representation of
the target resource. TEPP maps that retrieval onto a bounded, consumer-scoped
resolve of an idempotency key to a durable run identity. The RFC does not
define psychometric acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0038-analysis-run-idempotency-lookup-cli.md` — this client
- `docs/adr/0037-analysis-run-idempotency-lookup-get.md` — lookup GET listener
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumer
  registry and metric-free `202 Accepted`
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented lookup resource
- `crates/tepp_api/tests/analysis_run_idempotency_lookup_cli_contract.rs` —
  fail-closed lookup CLI proofs

## Verification

- `tepp-analysis-runs lookup` of an accepted run returns metric-free
  `run_id`/`run_state`/`idempotency_key` without RMSE/bias/coverage/SE-gate
  keys or `tepp.scientific_acceptance.v1`;
- another consumer cannot resolve the first consumer's key;
- non-loopback hosts, credential flags, `--run-id`, collection pagination
  flags, nonempty stdin, and unknown verbs fail closed;
- review, Copilot, GitHub, and bearer flags remain `AuthorizationDenied`.

## Non-claims

This slice does not implement lookup GET HTTP, stored-request CLI, retry CLI,
retry-parent CLI, collection GET, collection CLI list, cancel HTTP, cancel
CLI, create CLI, status CLI, GET-by-id, persistence, production TLS, Leiden
consensus, or an ADR 0014 scientific claim-promotion package.
