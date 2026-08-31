# Temporal-context CLI (doctoring)

## Scope

`tepp-temporal-context query` is the operator-visible client of loopback
`POST /v1/temporal-context` on `AnalysisRunLiveService` / `tepp-loopback`.
HTTP method, path, and header semantics follow current HTTP semantics
(Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of non-loopback
hosts, unpublished consumers, review/Copilot/GitHub credential flags, and
scientific-authority promotion is repository contract authority (ADR 0027;
ADR 0002; ADR 0011; ADR 0014), not an RFC inference rule.

CLI stdout is the cutoff-safe `TemporalContextResponse`.
`claim_boundary` remains `association_not_causal`.
`tepp.scientific_acceptance.v1` never appears. Process exit 0 is not a
completed temporal model, calibrated score, theta estimate, uncertainty
statement, causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing the enclosed
representation. TEPP maps that processing onto a bounded, cutoff-safe
temporal-context read. The RFC does not define psychometric acceptance, RMSE,
causality, or claim promotion.

### Internal contract evidence

- `docs/adr/0027-temporal-context-cli.md` — this client
- `docs/adr/0002-six-clock-temporal-semantics.md` — cutoff eligibility
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `crates/tepp_api/tests/temporal_context_cli_contract.rs` — fail-closed
  temporal-context CLI proofs

## Verification

- `tepp-temporal-context query` of a cutoff-safe LineageWeave body returns
  `association_not_causal` without RMSE/bias/coverage/SE-gate keys,
  `causal_score`, or `tepp.scientific_acceptance.v1`;
- non-loopback hosts, credential flags, empty stdin, and unknown verbs fail
  closed.

## Non-claims

This slice does not implement project-history CLI, export CLI, analysis-run
CLIs, GET-by-id, wait CLI, lookup CLI, persistence, production TLS, Leiden
consensus, GAP-010 Figma/export, causal inference, or an ADR 0014 scientific
claim-promotion package.
