# Architecture Decision Records

TEPP uses numbered ADRs for decisions that constrain latent-variable meaning, temporal semantics, event ontology, multilingual measurement, numerical backends, privacy/security, persistence, orchestration, automation authority, scientific claims, and modular service boundaries.

Read [`ADR_POLICY.md`](ADR_POLICY.md) first. **Decision status and implementation maturity are independent. `Accepted` means the architecture decision is authoritative; it does not mean the capability is implemented or released.**

| ADR | Decision | Decision status | Implementation maturity | Clarification / supersession |
|---|---|---|---|---|
| [0001](0001-rust-first-modular-msa.md) | Rust-first numerical core and CPU `f64` reference | Accepted | partial | ADR 0011 owns cross-service/MSA authority; 0001 retains numerical/backend authority. |
| [0002](0002-six-clock-temporal-semantics.md) | Six-clock temporal semantics and fail-closed historical leakage prevention | Accepted | active-PR | Unmerged PR #8 is the canonical Task 3 replacement implementing typed clocks/intervals against the current protected-main lineage; conflicted PR #5 is superseded lineage. Later graph/split enforcement remains target work. |
| [0003](0003-relational-event-multiple-membership.md) | Relational event ontology and time-varying cross-classified multiple membership | Accepted | partial | Weighted time-varying membership network/roles are active-PR (PR #12); full multilevel estimators, graph ontology, and persistence remain accepted-target. ADR 0016 owns event-intelligence tasks. |
| [0004](0004-shared-multilingual-latent-space.md) | One shared multilingual latent space with explicit invariance status | Accepted | accepted-target | ADR 0012 owns the full topic-estimator/backend/global-topic contract. |
| [0005](0005-posterior-esem-dsem.md) | Posterior-aware ESEM/DSEM and valid compositional coordinates | Accepted | accepted-target | Downstream psychometric authority; upstream topic/network model is clarified by ADR 0012. |
| [0006](0006-vram-gpu-nvidia-orchestration.md) | VRAM-adaptive GPU compute and model-credential boundary | Accepted | accepted-target | LLM orchestration policy superseded by ADR 0010; autonomous development authority governed by ADR 0015. |
| [0007](0007-rust-workspace-quality-gates.md) | Explicit Rust workspace, pinned toolchains, and exact quality gates | Accepted | implemented-main | ADR 0014 governs scientific/product claim promotion beyond repository-quality tooling. |
| [0008](0008-immutable-evidence-identities-digests-and-spans.md) | Immutable evidence identities, `SHA-256` digests, exact spans, and strict wire reconstruction | Accepted | implemented-main | ADR 0013 governs future persistence/reproducibility/split authority. |
| [0009](0009-purpose-bound-pii-governance.md) | Purpose-bound PII governance without blanket masking | Accepted | partial | Persistence retention/deletion/legal-hold (`0007`) and provider-payload minimization implemented-main; deployment evidence remains accepted-target. |
| [0010](0010-adaptive-llm-orchestration.md) | Adaptive LLM orchestration and test-time compute | Accepted | partial | `tepp_api` router/ablation/orchestrator binding on the active PR; live NIM execution and production ablation evidence remain accepted-target. |
| [0011](0011-standalone-modular-msa-boundary.md) | Standalone operation and modular CWL MSA boundary | Accepted | partial | Owns cross-service persistence/credential/API authority; no direct cross-service application-table coupling. |
| [0012](0012-temporal-relational-shared-latent-topic-measurement.md) | Temporal Relational Shared-Latent Topic Measurement (TRSL-TM) | Accepted | accepted-target | Owns topic backend compatibility, global topic identity, method effects, K/model-selection prerequisites, and compositional topic coordinates. |
| [0013](0013-bitemporal-persistence-reproducibility-and-split-authority.md) | Bitemporal persistence, reproducibility manifests, and relation-aware split authority | Accepted | partial | Owns PostgreSQL adapter semantics, immutable run/split manifests, leakage-safe partitions, and recovery identity; optional `live-sqlx` `PgPool`, live PG CI, tenant RLS, and `0006` membership implemented-main; `0007` retention/deletion/legal-hold on the active PR; remaining physical ERD/backup accepted-target. |
| [0014](0014-scientific-claim-promotion-and-release-evidence.md) | Scientific claim promotion and release evidence authority | Accepted | partial | Separates design, implementation, scientific/product claim, and release authority; repository SBOM/provenance generator implemented, full release bundle remaining. |
| [0015](0015-autonomous-development-review-and-merge-authority.md) | Autonomous development, review, and merge authority separation | Accepted | active-PR | Separates model proposal, deterministic verification, publication, independent review, and merge/release authority. |
| [0016](0016-tdt-chronos-event-intelligence-boundary.md) | TDT, CHRONOS, and Event Ontology intelligence boundary | Accepted | active-PR | TDT story-segmentation `WindowDiff`/`Pk` in existing `event_core`; remaining TDT/CHRONOS stack remains accepted-target. |

## Decision ownership summary

Use the narrowest owning ADR when decisions overlap:

- **numerical implementation / reference backend:** ADR 0001;
- **clock/time eligibility:** ADR 0002;
- **event ontology / relation / membership semantics:** ADR 0003;
- **multilingual semantic alignment:** ADR 0004;
- **ESEM/DSEM and psychometric interpretation:** ADR 0005;
- **GPU/VRAM and model-credential boundary:** ADR 0006;
- **repository quality tooling:** ADR 0007;
- **evidence identity / spans / wire reconstruction:** ADR 0008;
- **PII/privacy authority:** ADR 0009;
- **LLM test-time compute / orchestration:** ADR 0010;
- **standalone/CWL MSA service authority:** ADR 0011;
- **topic measurement / backend/global-K/method-effect contract:** ADR 0012;
- **persistence / manifests / leakage-safe split:** ADR 0013;
- **claim maturity / release evidence:** ADR 0014;
- **autonomous development/review/merge authority:** ADR 0015;
- **TDT/CHRONOS event intelligence:** ADR 0016.

## Change and supersession rule

ADR status changes require a pull request, source traceability, tests/evidence for affected invariants, and corresponding PRD/Architecture/TRD/Traceability updates where the approved measurement target changes. Decisions that materially alter privacy authority, orchestration authority, service ownership, temporal semantics, ontology, persistence identity, scientific estimands, release evidence, or automation authority require a superseding ADR rather than silent drift.

Partial supersession must identify the exact moved decision scope in both the older ADR and this index. Historical ADR text remains evidence of why a decision existed; it must not be silently rewritten to make a later architecture appear original.
