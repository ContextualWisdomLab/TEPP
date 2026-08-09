# TEPP Documentation Map

TEPP's approved PRD v0.4 and implementation plan are already the primary product baseline. This index makes the remaining technical, data, quality, and operating contracts discoverable without duplicating that source material.

| Area | Canonical document |
|---|---|
| Approved product requirements | [`docs/product/prd-v0.4-approved.md`](docs/product/prd-v0.4-approved.md) |
| Technical requirements | [`docs/TRD.md`](docs/TRD.md) |
| Architecture | [`ARCHITECTURE.md`](ARCHITECTURE.md) |
| UML/runtime/scientific flows | [`docs/UML.md`](docs/UML.md) |
| Logical/physical ERD | [`docs/ERD.md`](docs/ERD.md) |
| Security policy | [`SECURITY.md`](SECURITY.md) |
| Test/scientific validation strategy | [`docs/TEST_STRATEGY.md`](docs/TEST_STRATEGY.md) |
| Operability/recovery/release | [`docs/OPERABILITY.md`](docs/OPERABILITY.md) |
| Requirement/research/evidence traceability | [`docs/TRACEABILITY.md`](docs/TRACEABILITY.md) |
| Architecture decisions | [`docs/adr/README.md`](docs/adr/README.md) |
| Delivery roadmap | [`docs/roadmaps/2026-08-05-tepp-delivery-roadmap.md`](docs/roadmaps/2026-08-05-tepp-delivery-roadmap.md) |
| Foundation implementation plan | [`docs/superpowers/plans/2026-08-05-temporal-event-foundation.md`](docs/superpowers/plans/2026-08-05-temporal-event-foundation.md) |
| Standards and APA 7 literature | [`docs/research/standards-and-literature.md`](docs/research/standards-and-literature.md) |
| Governance | [`GOVERNANCE.md`](GOVERNANCE.md) |
| Agent development rules | [`AGENTS.md`](AGENTS.md) |
| Agent context | [`CLAUDE.md`](CLAUDE.md) |
| Change history | [`CHANGELOG.md`](CHANGELOG.md) |

## Maturity vocabulary

- **implemented-main** — promotion follows the canonical rule in [`docs/TRACEABILITY.md`](docs/TRACEABILITY.md): source is integrated on protected `main` and the relevant exact-current-head tests, scientific/recovery/validation evidence, security and supply-chain gates, and qualifying independent review all pass.
- **active-PR** — implemented only on an open PR and not yet a protected-main claim.
- **accepted-target** — approved PRD/ADR architecture not yet implemented.
- **conceptual** — logical entity/service/model contract; not evidence of a migration or deployment.
- **research-only** — evaluated research direction not accepted as production behavior.

At the time of this review, immutable evidence records/exact spans and the Rust workspace quality foundation are implemented-main. PR #5 typed six-clock values/uncertain intervals and PR #6 Allen interval algebra/bounded closure are active-PR. Event ontology, PostgreSQL persistence, shared-latent topic estimation, GPU kernels, TDT/CHRONOS intelligence, longitudinal ESEM/DSEM, and visual analytics remain later accepted-target work.