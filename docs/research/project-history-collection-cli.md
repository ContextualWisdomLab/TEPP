# Project-history collection CLI (doctoring)

## Scope

`tepp-project-histories list` is the operator-visible client of loopback
`GET /v1/project-histories`. HTTP method, path, and header semantics follow
current HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed
refusal of non-loopback hosts, unpublished consumers, naruon, review/Copilot/
GitHub credential flags, and scientific-authority promotion is repository
contract authority (ADR 0065; ADR 0028; ADR 0021; ADR 0011), not an RFC
inference rule.

CLI stdout is metric-free `ProjectHistoryCollection` JSON. Each row carries
`project_key`, `idempotency_key`, `knowledge_cutoff`, and
`inference_status=temporal_association_only` only. Process exit 0 is not a
completed temporal model, calibrated score, theta estimate, uncertainty
statement, causal inference, or scientific claim.
`tepp.scientific_acceptance.v1` never appears.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource's
current state. TEPP maps that retrieval onto a bounded, LineageWeave-owned
collection of metric-free project-history identities. The RFC does not define
psychometric acceptance, RMSE, causality, or claim promotion.

### Internal contract evidence

- `docs/adr/0065-project-history-collection-cli.md` — this client
- `docs/adr/0028-project-history-collection-get.md` — collection GET listener
- `docs/adr/0061-project-history-cli.md` — distinct `tepp-project-history`
  POST CLI on live #420
- `docs/adr/0021-lineageweave-project-history-boundary.md` — LineageWeave
  POST boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — CLI
  success is not a scientific claim
- `docs/API_CONTRACT.md` — documented collection resource
- `crates/tepp_api/tests/project_history_collection_cli_contract.rs` —
  fail-closed collection CLI proofs

## Verification

- `tepp-project-histories list` of accepted LineageWeave projections returns
  metric-free `temporal_association_only` rows without RMSE/bias/coverage/
  SE-gate keys, `evidence_text`, `findings`, `causal_score`, or
  `tepp.scientific_acceptance.v1`;
- naruon consumer, non-loopback hosts, credential flags, nonempty stdin, and
  unknown verbs fail closed;
- review, Copilot, GitHub, and bearer flags remain `AuthorizationDenied`.

## Non-claims

This slice does not implement project-history POST CLI, GET-by-id, analysis-run
collection CLI, temporal-context CLI, export CLI, persistence, production TLS,
Leiden consensus, or an ADR 0014 scientific claim-promotion package. It does
not infer causality.
