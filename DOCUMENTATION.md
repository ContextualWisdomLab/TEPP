# TEPP Documentation Map

TEPP's approved PRD v0.4 and implementation plan are the primary product baseline. This index makes the technical, data, scientific, security/privacy, integration, quality, operating, and assurance contracts discoverable without duplicating that source material.

| Area | Canonical document |
|---|---|
| Approved product requirements | [`docs/product/prd-v0.4-approved.md`](docs/product/prd-v0.4-approved.md) |
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
| Actions workflow fleet audit | [`docs/operations/ACTIONS_WORKFLOW_FLEET.md`](docs/operations/ACTIONS_WORKFLOW_FLEET.md) |
| Actions fleet research doctoring | [`docs/research/actions-workflow-fleet.md`](docs/research/actions-workflow-fleet.md) |
| TDT story-segmentation `WindowDiff`/`Pk` doctoring | [`docs/research/tdt-story-segmentation.md`](docs/research/tdt-story-segmentation.md) |
| Hourly NIM OpenCode doctoring | [`docs/doctoring/hourly-nim-opencode-development.md`](docs/doctoring/hourly-nim-opencode-development.md) |
| Change history | [`CHANGELOG.md`](CHANGELOG.md) |

## Maturity vocabulary

The canonical implementation-maturity vocabulary is defined in [`docs/adr/ADR_POLICY.md`](docs/adr/ADR_POLICY.md) and promotion evidence in [`docs/TRACEABILITY.md`](docs/TRACEABILITY.md). In particular, **an ADR with decision status `Accepted` is not automatically implemented or shipped.**

- **implemented-main** — source is integrated on protected `main` and the relevant exact-current-head tests, scientific/recovery/validation evidence, security and supply-chain gates, and qualifying review required by live policy pass.
- **active-PR** — implementation exists only on an open PR and is not a protected-main claim.
- **partial** — an explicitly identified subset is implemented on protected main while the rest remains target work.
- **accepted-target** — accepted PRD/ADR architecture not yet integrated.
- **research-only** — evaluated research direction not accepted as production behavior.
- **out-of-scope** — explicitly outside TEPP ownership.
- **conceptual** — logical entity/service/model contract; not evidence of a migration or deployment.
- **deployment-owned** — evidence depends on a concrete deployed environment or organization and cannot be claimed by repository design alone.
- **external-assurance** — certification, attestation, legal opinion, or other independent assessment that TEPP cannot self-issue.

## Documentation fitness

The documentation graph is **design-sufficient** when a reviewer can reconstruct TEPP's product requirements, technical/scientific estimands, authority boundaries, temporal/event/membership semantics, data model, failure modes, security/privacy controls, validation strategy, API/integration contract, operability, research basis, ADR ownership/supersession, and release acceptance without chat history.

It is **protected-main-sufficient** only after the canonical documents are integrated on protected `main`, remain semantically current with live code, and their required exact-head documentation/security/review gates pass. An active documentation PR can therefore be design-sufficient while the protected branch remains documentation-insufficient.

At the time of this review, immutable evidence records/exact spans, the Rust workspace quality foundation, and typed six-clock values/uncertain intervals (PR #8) are implemented-main. PR #9 is the active-PR that replays Task 4 Allen interval algebra and bounded path-consistency reasoner work onto that protected-main temporal foundation. Superseded PRs #5 and #6 remain historical lineage only. Event ontology, PostgreSQL persistence, shared-latent topic estimation, GPU kernels, TDT/CHRONOS intelligence, longitudinal ESEM/DSEM, visual analytics, production HTTP services, and deployment assurance remain later accepted-target or deployment-owned work.
