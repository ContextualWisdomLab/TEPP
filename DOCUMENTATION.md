# TEPP Documentation Map

TEPP's approved PRD v0.4 remains the product/measurement baseline. This map identifies the canonical technical, domain, scientific, security, operating, and delivery authorities without duplicating their content. An open PR, queued check, local test, or accepted ADR is not evidence of protected-main implementation unless the maturity and exact-head gates say so.

| Area | Canonical document |
|---|---|
| Approved product requirements | [`docs/product/prd-v0.4-approved.md`](docs/product/prd-v0.4-approved.md) |
| Live product and technical gap baseline | [`docs/product-technical-gap-baseline.md`](docs/product-technical-gap-baseline.md) |
| Whole-repository documentation assessment | [`docs/DOCUMENTATION_ASSESSMENT.md`](docs/DOCUMENTATION_ASSESSMENT.md) |
| Technical requirements | [`docs/TRD.md`](docs/TRD.md) |
| Architecture | [`ARCHITECTURE.md`](ARCHITECTURE.md) |
| DDD bounded-context and ownership map | [`docs/architecture/domain-context-map.md`](docs/architecture/domain-context-map.md) |
| Temporal/dependence composition boundary | [`docs/architecture/temporal-dependence-composition.md`](docs/architecture/temporal-dependence-composition.md) |
| Modular/API integration contract | [`docs/API_CONTRACT.md`](docs/API_CONTRACT.md) |
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
| LSIRM/MLSIRM/DLSJM primary research authority | [`docs/research/temporal-dependence-models.md`](docs/research/temporal-dependence-models.md) |
| Multilevel/event-time recovery doctoring | [`docs/research/multilevel-event-time-recovery.md`](docs/research/multilevel-event-time-recovery.md) |
| Posterior ESEM/DSEM input-gate doctoring | [`docs/research/posterior-esem-input-gates.md`](docs/research/posterior-esem-input-gates.md) |
| Interval cutoff eligibility | [`docs/research/interval-cutoff-eligibility.md`](docs/research/interval-cutoff-eligibility.md) |
| Scientific claim-promotion gates | [`docs/research/scientific-claim-promotion-gates.md`](docs/research/scientific-claim-promotion-gates.md) |
| Causal-identification gate | [`docs/research/causal-identification-gate.md`](docs/research/causal-identification-gate.md) |
| Topic log-ratio coordinates | [`docs/research/topic-logratio-coordinates.md`](docs/research/topic-logratio-coordinates.md) |
| VRAM budget / GPU fallback | [`docs/research/vram-budget-types.md`](docs/research/vram-budget-types.md) |
| TDT story segmentation | [`docs/research/tdt-story-segmentation.md`](docs/research/tdt-story-segmentation.md) |
| TDT link-detection calibration | [`docs/research/event-link-detection-calibration.md`](docs/research/event-link-detection-calibration.md) |
| First-story detection calibration | [`docs/research/first-story-detection-calibration.md`](docs/research/first-story-detection-calibration.md) |
| CHRONOS prediction calibration | [`docs/research/chronos-prediction-calibration.md`](docs/research/chronos-prediction-calibration.md) |
| CHRONOS schema-slot calibration | [`docs/research/chronos-schema-slot-calibration.md`](docs/research/chronos-schema-slot-calibration.md) |
| Event-tracking calibration | [`docs/research/event-tracking-calibration.md`](docs/research/event-tracking-calibration.md) |
| Provider payload minimization | [`docs/research/provider-payload-minimization.md`](docs/research/provider-payload-minimization.md) |
| Operational log/source separation | [`docs/research/operational-log-source-separation.md`](docs/research/operational-log-source-separation.md) |
| Adaptive orchestration | [`docs/research/adaptive-orchestration-router.md`](docs/research/adaptive-orchestration-router.md) |
| Governance | [`GOVERNANCE.md`](GOVERNANCE.md) |
| Agent development rules | [`AGENTS.md`](AGENTS.md) |
| Agent context | [`CLAUDE.md`](CLAUDE.md) |
| Hourly product-development operations | [`docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md`](docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md) |
| Actions workflow fleet audit | [`docs/operations/ACTIONS_WORKFLOW_FLEET.md`](docs/operations/ACTIONS_WORKFLOW_FLEET.md) |
| Change history | [`CHANGELOG.md`](CHANGELOG.md) |

## Authority and maturity rules

The implementation-maturity vocabulary is defined by [`docs/adr/ADR_POLICY.md`](docs/adr/ADR_POLICY.md), and scientific promotion evidence is governed by [`docs/TRACEABILITY.md`](docs/TRACEABILITY.md) and ADR 0014. In particular, an Accepted ADR is a decision, not proof of a shipped implementation.

`docs/architecture/domain-context-map.md` owns the strategic DDD boundary vocabulary for the delivery-recovery cycle. `docs/architecture/temporal-dependence-composition.md` refines the cross-repository dependence/temporal composition boundary under ADR 0011; it cannot override the PRD, ADR 0011, or protected-main source. The dependence-family primary citations are maintained once in `docs/research/temporal-dependence-models.md`.

## Six-clock contract

The canonical six temporal roles are event **or** valid time, assertion time, document time, system time, available time, and knowledge cutoff. Event instants and validity intervals are representations of the first role, not two independent analysis clocks. Historical eligibility requires available time at or before knowledge cutoff. Forward transition/state relations and retrospective/provenance relations remain distinct.

## Documentation fitness

The graph is design-sufficient when a reviewer can reconstruct product requirements, technical/scientific estimands, DDD/service authority, temporal/event/membership semantics, data model, failure modes, security/privacy controls, validation strategy, API/integration contract, operability, research basis, ADR ownership/supersession, and release acceptance without chat history.

It is protected-main-sufficient only after the relevant canonical documents are integrated on protected `main`, remain semantically current with live code, and their current-head documentation/security/review gates pass. Queued, stale, skipped, predecessor-head, or branch-local evidence is non-passing.