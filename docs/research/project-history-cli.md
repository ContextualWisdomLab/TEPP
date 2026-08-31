# Project-history CLI (doctoring)

## Scope

`tepp-project-history query` is the operator-visible client of loopback
`POST /v1/project-histories` on `AnalysisRunLiveService` / `tepp-loopback`.
The CLI mints `lineageweave_project_history_exchange` and renders onto spawned
`tepp-loopback` TCP. HTTP method, path, and header semantics follow current
HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of
non-loopback hosts, unpublished consumers, naruon on this adapter,
review/Copilot/GitHub credential flags, and scientific-authority promotion is
repository contract authority (ADR 0061; ADR 0021; ADR 0011; ADR 0014), not an
RFC inference rule.

CLI stdout is the cutoff-safe `ProjectHistoryProjection`.
`inference_status` remains `temporal_association_only`.
`tepp.scientific_acceptance.v1` never appears. Process exit 0 is not a
completed temporal model, calibrated score, theta estimate, uncertainty
statement, causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing the enclosed
representation. TEPP maps that processing onto a bounded, cutoff-safe
project-history projection. The RFC does not define psychometric acceptance,
RMSE, causality, or claim promotion.

### Internal contract evidence

- `docs/adr/0061-project-history-cli.md` — this client
- `docs/adr/0021-lineageweave-project-history-boundary.md` — HTTP boundary
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `crates/tepp_api/tests/project_history_cli_contract.rs` — fail-closed
  project-history CLI proofs

## Verification

- `tepp-project-history query` of a cutoff-safe LineageWeave body returns
  `temporal_association_only` without RMSE/bias/coverage/SE-gate keys,
  `causal_score`, or `tepp.scientific_acceptance.v1`;
- non-loopback hosts, `localhost`, credential flags, empty stdin, naruon, and
  unknown verbs fail closed;
- `NaruonLiveService` still refuses `POST /v1/project-histories`.

## Non-claims

This slice does not implement temporal-context CLI, export CLI, analysis-run
CLIs, GET-by-id, wait CLI, lookup CLI, persistence, production TLS, Leiden
consensus, GAP-010 Figma/export, causal inference, or an ADR 0014 scientific
claim-promotion package.
