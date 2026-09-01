# Interpretation-run cancel CLI (doctoring)

## Scope

`tepp-interpretation-run-cancel cancel` is the operator-visible client of
loopback `POST /v1/interpretation-runs/{idempotency_key}/cancel` on
`tepp-orchestrator-loopback`. HTTP method, path, and header semantics follow
current HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed
refusal of unpublished consumers, leftover nonempty stdin, public bind,
`localhost`, `http` origins, pagination flags, review/Copilot/GitHub
credential flags, and scientific-authority promotion is repository contract
authority (ADR 0074; ADR 0073; ADR 0010; ADR 0011; ADR 0014), not an RFC
inference rule.

Stdout is metric-free. `claim_status` remains `hypothetical`.
`scientific_authority` remains false. `cancelled` is `true`.
`tepp.scientific_acceptance.v1` never appears. Process 0 is not a completed
psychometric result, calibrated score, theta estimate, uncertainty statement,
causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing the representation
enclosed in the request. TEPP maps that processing onto a bounded, in-memory
drop of one hypothetical interpretation-run identity. The RFC does not define
psychometric acceptance, RMSE, causality, or claim promotion.

### Internal contract evidence

- `docs/adr/0074-interpretation-run-cancel-cli.md` — this CLI
- `docs/adr/0073-interpretation-run-cancel-http.md` — cancel HTTP
- `docs/adr/0064-interpretation-run-cli.md` — create CLI
- `docs/adr/0010-adaptive-llm-orchestration.md` — mode vocabulary and
  scientific-authority separation
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — process
  0 is not a scientific claim
- `crates/orchestrator_live/tests/interpretation_run_cancel_cli_contract.rs`
  — fail-closed cancel CLI proofs

## Verification

- `tepp-interpretation-run-cancel cancel` of an accepted contextual-orchestrator
  identity returns `hypothetical` with `scientific_authority` false,
  `cancelled` true, and without RMSE/bias/coverage/SE-gate keys,
  `evidence_span_ids`, `causal_score`, or `tepp.scientific_acceptance.v1`;
- leftover nonempty stdin, naruon or LineageWeave, public bind, `localhost`,
  `http` origin, pagination flags, and credential flags fail closed.

## Non-claims

This slice does not implement collection CLI, analysis-run cancel CLI, export
GET, project-history retrieval CLI, persistence, production TLS, Leiden
consensus, GAP-010 Figma/export, provider execution, causal inference, or an
ADR 0014 scientific claim-promotion package.
