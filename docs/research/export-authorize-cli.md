# Export-authorize CLI (doctoring)

## Scope

`tepp-exports authorize` is the operator-visible client of loopback
`POST /v1/exports` on `NaruonLiveService`. `tepp-naruon-live` is the packaged
listener. `tepp-loopback` is `AnalysisRunLiveService` and does not serve this
path. HTTP method, path, and header semantics follow current HTTP semantics
(Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of non-loopback
hosts, non-modular purposes, review/Copilot/GitHub credential flags, and
scientific-authority promotion is repository contract authority (ADR 0026;
ADR 0009; ADR 0011; ADR 0014), not an RFC inference rule.

CLI stdout is the purpose-bound `ExportAuthorizationDecision` JSON.
`tepp.scientific_acceptance.v1` never appears. Process exit 0 is not a
completed temporal model, calibrated score, theta estimate, uncertainty
statement, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing the enclosed
representation. TEPP maps that processing onto a bounded, purpose-gated
export authorization. The RFC does not define psychometric acceptance, RMSE,
or claim promotion.

### Internal contract evidence

- `docs/adr/0026-export-authorize-cli.md` — this client
- `docs/adr/0009-purpose-bound-pii-governance.md` — purpose-bound disclosure
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `crates/tepp_api/tests/export_authorize_cli_contract.rs` — fail-closed
  export CLI proofs

## Verification

- `tepp-exports authorize` of a modular export returns
  `purpose_bound_export_allowed` without RMSE/bias/coverage/SE-gate keys or
  `tepp.scientific_acceptance.v1`;
- operational-monitoring purpose, non-loopback hosts, credential flags, empty
  stdin, and unknown verbs fail closed.

## Non-claims

This slice does not implement export retrieval GET, analysis-run CLIs,
GET-by-id, wait CLI, lookup CLI, persistence, production TLS, Leiden
consensus, GAP-010 Figma/export, or an ADR 0014 scientific claim-promotion
package.
