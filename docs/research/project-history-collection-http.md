# Project-history collection GET (doctoring)

## Scope

`GET /v1/project-histories` is the operator-visible collection of accepted
cutoff-safe project-history projections on `AnalysisRunLiveService` /
`tepp-loopback`. HTTP method, path, and header semantics follow current HTTP
semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of
unpublished consumers, nonempty GET bodies, review/Copilot/GitHub credential
flags, evidence text, and scientific-authority promotion is repository
contract authority (ADR 0028; ADR 0021; ADR 0011; ADR 0014), not an RFC
inference rule.

Collection JSON is metric-free. `inference_status` remains
`temporal_association_only`. `tepp.scientific_acceptance.v1` never appears.
A 200 collection page is not a completed temporal model, calibrated score,
theta estimate, uncertainty statement, causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource.
TEPP maps that retrieval onto a bounded, cutoff-safe project-history
collection. The RFC does not define psychometric acceptance, RMSE, causality,
or claim promotion.

### Internal contract evidence

- `docs/adr/0028-project-history-collection-get.md` — this collection
- `docs/adr/0021-lineageweave-project-history-boundary.md` — POST boundary
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/project_history_collection_http_contract.rs` —
  fail-closed collection proofs

## Verification

- `GET /v1/project-histories` of accepted LineageWeave projections returns
  `temporal_association_only` rows without RMSE/bias/coverage/SE-gate keys,
  `evidence_text`, `findings`, `causal_score`, or
  `tepp.scientific_acceptance.v1`;
- GET `/v1/analysis-runs`, naruon consumer, nonempty body, and unknown verbs
  fail closed.

## Non-claims

This slice does not implement project-history collection CLI, GET-by-id,
export CLI, analysis-run collection GET, wait CLI, lookup CLI, persistence,
production TLS, Leiden consensus, GAP-010 Figma/export, causal inference, or
an ADR 0014 scientific claim-promotion package.
