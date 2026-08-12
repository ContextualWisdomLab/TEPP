# TEPP Documentation Map

TEPP's concrete PRD v0.5 is the canonical product-requirements contract. The
approved v0.4 design remains historical evidence. This map makes the product,
technical, data, scientific, security/privacy, integration, quality, operating,
and assurance contracts discoverable without chat reconstruction.

| Area | Canonical document |
|---|---|
| Current product requirements | [`docs/product/prd-v0.5.md`](docs/product/prd-v0.5.md) |
| Historical approved v0.4 design | [`docs/product/prd-v0.4-approved.md`](docs/product/prd-v0.4-approved.md) |
| Whole-conversation documentation fitness | [`docs/DOCUMENTATION_ASSESSMENT.md`](docs/DOCUMENTATION_ASSESSMENT.md) |
| Technical requirements | [`docs/TRD.md`](docs/TRD.md) |
| Architecture | [`ARCHITECTURE.md`](ARCHITECTURE.md) |
| Modular/API integration contract | [`docs/API_CONTRACT.md`](docs/API_CONTRACT.md) |
| naruon modular consumer contract | [`docs/connectors/naruon-artifact-consumer.md`](docs/connectors/naruon-artifact-consumer.md) |
| contextual-orchestrator interpretation port | [`docs/connectors/contextual-orchestrator-interpretation-port.md`](docs/connectors/contextual-orchestrator-interpretation-port.md) |
| UML/runtime/scientific flows | [`docs/UML.md`](docs/UML.md) |
| Logical/physical ERD | [`docs/ERD.md`](docs/ERD.md) |
| Security policy | [`SECURITY.md`](SECURITY.md) |
| Threat model | [`docs/THREAT_MODEL.md`](docs/THREAT_MODEL.md) |
| Privacy and data governance | [`docs/PRIVACY_DATA_GOVERNANCE.md`](docs/PRIVACY_DATA_GOVERNANCE.md) |
| Compliance/assurance readiness | [`docs/COMPLIANCE_READINESS.md`](docs/COMPLIANCE_READINESS.md) |
| LLM orchestration/test-time compute | [`docs/LLM_ORCHESTRATION.md`](docs/LLM_ORCHESTRATION.md) |
| Test/scientific validation strategy | [`docs/TEST_STRATEGY.md`](docs/TEST_STRATEGY.md) |
| Operability/recovery/release | [`docs/OPERABILITY.md`](docs/OPERABILITY.md) |
| Requirement/research/evidence traceability | [`docs/TRACEABILITY.md`](docs/TRACEABILITY.md) |
| Architecture decision index / ownership map | [`docs/adr/README.md`](docs/adr/README.md) |
| ADR status, maturity, and supersession policy | [`docs/adr/ADR_POLICY.md`](docs/adr/ADR_POLICY.md) |
| Delivery roadmap | [`docs/roadmaps/2026-08-05-tepp-delivery-roadmap.md`](docs/roadmaps/2026-08-05-tepp-delivery-roadmap.md) |
| Foundation implementation plan | [`docs/superpowers/plans/2026-08-05-temporal-event-foundation.md`](docs/superpowers/plans/2026-08-05-temporal-event-foundation.md) |
| Foundation validation ledger | [`docs/validation/temporal-event-foundation.md`](docs/validation/temporal-event-foundation.md) |
| Standards and APA 7 literature | [`docs/research/standards-and-literature.md`](docs/research/standards-and-literature.md) |
| Governance | [`GOVERNANCE.md`](GOVERNANCE.md) |
| Agent development rules | [`AGENTS.md`](AGENTS.md) |
| Agent context | [`CLAUDE.md`](CLAUDE.md) |
| Hourly NIM product-development operations | [`docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md`](docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md) |
| Hourly NIM OpenCode doctoring | [`docs/doctoring/hourly-nim-opencode-development.md`](docs/doctoring/hourly-nim-opencode-development.md) |
| Change history | [`CHANGELOG.md`](CHANGELOG.md) |

## PRD v0.5 concreteness

The current PRD adds stable requirement identifiers and acceptance evidence for:

- immutable evidence, six clocks, relations, events, and multiple membership;
- multilingual semantic units, concept governance, method effects, and language
  profile promotion;
- TRSL-TM posterior estimation, candidate-K hard gates/Pareto review, valid
  compositional networks, and consensus clusters;
- ESEM/DSEM construct-role, uncertainty, invariance, within/between, irregular
  time, and causal-language gates;
- LLM interpreter/verifier separation and test-time-compute ablation;
- CPU/GPU/VRAM profiles and resource admission;
- APIs, exports, purpose-bound PII, tenant isolation, audit, operability, and
  release evidence;
- user workflows, lifecycle state machines, error taxonomy, product surfaces,
  scale tiers, commercial metrics, and phased exit criteria.

## Maturity vocabulary

The canonical implementation-maturity vocabulary is defined in
[`docs/adr/ADR_POLICY.md`](docs/adr/ADR_POLICY.md), with promotion evidence in
[`docs/TRACEABILITY.md`](docs/TRACEABILITY.md). In particular, **an ADR or PRD
requirement being accepted does not mean the capability is implemented or
released.**

- **implemented-main** — source is integrated on protected `main` and the
  relevant exact-current-head tests, scientific/recovery/validation evidence,
  security and supply-chain gates, and qualifying review required by live
  policy pass.
- **active-PR** — implementation exists only on an open PR and is not a
  protected-main claim.
- **partial** — an explicitly identified subset is implemented on protected
  main while the rest remains target work.
- **accepted-target** — accepted PRD/ADR architecture not yet integrated.
- **research-only** — evaluated research direction not accepted as production
  behavior.
- **out-of-scope** — explicitly outside TEPP ownership.
- **conceptual** — logical entity/service/model contract; not evidence of a
  migration or deployment.
- **deployment-owned** — evidence depends on a concrete deployed environment or
  organization and cannot be claimed by repository design alone.
- **external-assurance** — certification, attestation, legal opinion, or other
  independent assessment that TEPP cannot self-issue.

## Documentation fitness

The documentation graph is **design-sufficient** when a reviewer can reconstruct
TEPP's product workflows, requirements, technical/scientific estimands,
authority boundaries, temporal/event/membership semantics, data model, failure
modes, security/privacy controls, validation strategy, API/integration contract,
operability, research basis, ADR ownership/supersession, and release acceptance
without chat history.

It is **protected-main-sufficient** only after the canonical documents are
integrated on protected `main`, remain semantically current with live code, and
their required exact-head documentation/security/review gates pass. A reviewable
documentation PR may therefore be design-sufficient while protected main remains
product-documentation-insufficient.

At the inspected baseline, immutable evidence/exact spans, typed six-clock
values, bounded interval reasoning, forward-only relation graphs, leakage-safe
corpus splitting, deterministic simulations, recovery metrics, and selected API
and export contracts are implemented-main. Event/membership/persistence/API
surfaces are partial. FORCE RLS and runtime-role isolation are active on PR #30
until merged. Multilingual semantic measurement, TRSL-TM estimation,
candidate-K selection, topic networks/clusters, GPU kernels, TDT/CHRONOS,
longitudinal ESEM/DSEM, evidence-bounded interpretation, and visual analytics
remain separately gated product slices.
