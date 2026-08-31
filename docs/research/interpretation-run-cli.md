# Interpretation-run CLI (doctoring)

## Scope

`tepp-interpretation-runs create` is the operator-visible client of loopback
`POST /v1/interpretation-runs` on `OrchestratorLiveService` /
`tepp-orchestrator-loopback`. The CLI mints
`contextual_orchestrator_interpretation_run_exchange` and renders onto spawned
`tepp-orchestrator-loopback` TCP. HTTP method, path, and header semantics
follow current HTTP semantics (Fielding, Nottingham, & Reschke, 2022).
Fail-closed refusal of non-loopback hosts, unpublished consumers, naruon,
`LineageWeave`, review/Copilot/GitHub credential flags, and
scientific-authority promotion is repository contract authority (ADR 0064;
ADR 0010; ADR 0011; ADR 0014), not an RFC inference rule.

CLI stdout is the accepted hypothetical run. `claim_status` remains
`hypothetical`. `scientific_authority` remains false.
`tepp.scientific_acceptance.v1` never appears. Process exit 0 is not a
completed temporal model, calibrated score, theta estimate, uncertainty
statement, causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing the enclosed
representation. TEPP maps that processing onto a bounded, hypothetical
interpretation-run acknowledgement. The RFC does not define psychometric
acceptance, RMSE, causality, or claim promotion.

### Internal contract evidence

- `docs/adr/0064-interpretation-run-cli.md` — this client
- `docs/adr/0010-adaptive-llm-orchestration.md` — mode vocabulary and
  scientific-authority separation
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `crates/orchestrator_live/tests/interpretation_run_cli_contract.rs` —
  fail-closed interpretation-run CLI proofs

## Verification

- `tepp-interpretation-runs create` of a hypothetical body returns
  `claim_status` `hypothetical` with `scientific_authority` false and without
  RMSE/bias/coverage/SE-gate keys, `causal_score`, or
  `tepp.scientific_acceptance.v1`;
- non-loopback hosts, `localhost`, credential flags, empty stdin, naruon,
  LineageWeave, and unknown verbs fail closed;
- `tepp-orchestrator-loopback` serves one bounded POST on loopback only.

## Non-claims

This slice does not implement analysis-run CLIs, export CLI, temporal-context
CLI, project-history CLI, GET-by-id, wait CLI, lookup CLI, persistence,
production TLS, Leiden consensus, GAP-010 Figma/export, provider execution,
causal inference, or an ADR 0014 scientific claim-promotion package.
