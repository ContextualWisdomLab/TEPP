# Analysis-run wait CLI (doctoring)

## Scope

`tepp-analysis-runs wait` is the operator-visible poll client of loopback
`GET /v1/analysis-runs/{run_id}`. HTTP method, path, and header semantics
follow current HTTP semantics (Fielding, Nottingham, & Reschke, 2022).
Fail-closed refusal of non-loopback hosts, unpublished consumers,
review/Copilot/GitHub credential flags, unbounded waits, and
scientific-authority promotion is repository contract authority (ADR 0030;
ADR 0029; ADR 0027; ADR 0018; ADR 0011), not an RFC inference rule.

CLI stdout reuses status gates. Accepted, running, and failed remain
metric-free. `tepp.scientific_acceptance.v1` appears only on succeeded
`scientific_acceptance_v1`. Process exit 0 is not a completed temporal model,
calibrated score, theta estimate, uncertainty statement, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving a representation of
the target resource. TEPP maps repeated retrieval onto a bounded wait for
terminal status. The RFC does not define psychometric acceptance, RMSE, or
claim promotion.

### Internal contract evidence

- `docs/adr/0030-analysis-run-wait-cli.md` — this client
- `docs/adr/0029-analysis-run-status-cli.md` — single-shot status client
- `docs/adr/0027-scientific-acceptance-http-status.md` — GET-by-id listener
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `crates/tepp_api/tests/analysis_run_wait_cli_contract.rs` — fail-closed wait
  CLI proofs

## Verification

- `tepp-analysis-runs wait` of a failed run returns metric-free failed status
  without RMSE/bias/coverage/SE-gate keys or `tepp.scientific_acceptance.v1`;
- wait of an accepted run with `--timeout-ms 0` fails closed;
- non-loopback hosts, credential flags, nonempty stdin, and unknown verbs fail
  closed.

## Non-claims

This slice does not implement GET-by-id HTTP, status CLI, lifecycle POST,
cancel/create/retry/lookup/retry-lineage CLIs, persistence, production TLS,
Leiden consensus, or an ADR 0014 scientific claim-promotion package.
