# Interpretation-run retrieval CLI (doctoring)

## Scope

`tepp-interpretation-run-get get` is the operator-visible client of loopback
`GET /v1/interpretation-runs/{idempotency_key}` on
`tepp-orchestrator-loopback`. HTTP method, path, and header semantics follow
current HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed
refusal of unpublished consumers, leftover nonempty stdin, public bind,
`localhost`, `http` origins, pagination flags, review/Copilot/GitHub
credential flags, and scientific-authority promotion is repository contract
authority (ADR 0072; ADR 0071; ADR 0010; ADR 0011; ADR 0014), not an RFC
inference rule.

Stdout is metric-free. `claim_status` remains `hypothetical`.
`scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
appears. Process 0 is not a completed psychometric result, calibrated score,
theta estimate, uncertainty statement, causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource.
TEPP maps that retrieval onto a bounded, hypothetical interpretation-run
identity. The RFC does not define psychometric acceptance, RMSE, causality,
or claim promotion.

### Internal contract evidence

- `docs/adr/0072-interpretation-run-retrieval-cli.md` — this CLI
- `docs/adr/0071-interpretation-run-retrieval-get.md` — GET-by-id HTTP
- `docs/adr/0064-interpretation-run-cli.md` — create CLI
- `docs/adr/0010-adaptive-llm-orchestration.md` — mode vocabulary and
  scientific-authority separation
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — process
  0 is not a scientific claim
- `crates/orchestrator_live/tests/interpretation_run_retrieval_cli_contract.rs`
  — fail-closed retrieval CLI proofs

## Verification

- `tepp-interpretation-run-get get` of an accepted contextual-orchestrator
  identity returns `hypothetical` with `scientific_authority` false and
  without RMSE/bias/coverage/SE-gate keys, `evidence_span_ids`,
  `causal_score`, or `tepp.scientific_acceptance.v1`;
- leftover nonempty stdin, naruon or LineageWeave, public bind, `localhost`,
  `http` origin, pagination flags, and credential flags fail closed.

## Non-claims

This slice does not implement collection CLI, analysis-run GET-by-id, export
GET, project-history retrieval CLI, persistence, production TLS, Leiden
consensus, GAP-010 Figma/export, provider execution, causal inference, or an
ADR 0014 scientific claim-promotion package.
