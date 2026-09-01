# Architecture Decision Records

TEPP uses repository-wide numbered ADR identities for decisions that constrain latent-variable meaning, temporal semantics, event ontology, multilingual measurement, numerical backends, privacy/security, persistence, orchestration, automation authority, scientific claims, and modular service boundaries.

Read [`ADR_POLICY.md`](ADR_POLICY.md) first. **Decision status and implementation maturity are independent. `Accepted` means the architecture decision is authoritative; it does not mean the capability is implemented or released.** One ADR identity appears exactly once in this index and exactly once in the numbered root of `docs/adr/`. Branch-local implementation slices update implementation evidence for the owning ADR rather than minting a second row for the same decision.

| ADR | Decision | Decision status | Implementation maturity | Clarification / supersession |
|---|---|---|---|---|
| [0001](0001-rust-first-modular-msa.md) | Rust-first numerical core and CPU `f64` reference | Accepted | partial | Owns production arithmetic and reference-backend authority; ADR 0011 owns cross-service boundaries. |
| [0002](0002-six-clock-temporal-semantics.md) | Six-clock temporal semantics and fail-closed historical eligibility | Accepted | partial | Owns event/valid, assertion, document, system, available, and knowledge-cutoff semantics plus temporal eligibility. |
| [0003](0003-relational-event-multiple-membership.md) | Relational event ontology and time-varying cross-classified multiple membership | Accepted | partial | Owns event/relation/membership semantics; full multilevel estimators and persistence remain separate implementation work. |
| [0004](0004-shared-multilingual-latent-space.md) | Shared multilingual latent semantic space | Accepted | accepted-target | Owns multilingual latent-space and invariance intent; ADR 0020 owns span-grounded semantic-unit identity. |
| [0005](0005-posterior-esem-dsem.md) | Posterior-aware ESEM/DSEM and compositional coordinates | Accepted | active-PR | Owns longitudinal psychometric interpretation; reusable static kernels migrate to their canonical owner when applicable. |
| [0006](0006-vram-gpu-nvidia-orchestration.md) | VRAM-adaptive GPU compute and credential boundary | Accepted | accepted-target | Owns compute-backend/VRAM policy; LLM orchestration belongs to ADR 0010. |
| [0007](0007-rust-workspace-quality-gates.md) | Rust workspace, toolchain, and quality gates | Accepted | implemented-main | Owns repository quality tooling; scientific claim promotion belongs to ADR 0014. |
| [0008](0008-immutable-evidence-identities-digests-and-spans.md) | Immutable evidence identities, digests, spans, and wire reconstruction | Accepted | partial | Owns source-evidence identity and bounded reconstruction; persistence authority belongs to ADR 0013. |
| [0009](0009-purpose-bound-pii-governance.md) | Purpose-bound PII governance without blanket masking | Accepted | partial | Owns purpose-bound disclosure and re-identification controls; implementation evidence does not imply certification. |
| [0010](0010-adaptive-llm-orchestration.md) | Adaptive LLM orchestration and test-time compute | Accepted | partial | All provider execution remains behind contextual-orchestrator; LLM output has no numerical authority. |
| [0011](0011-standalone-modular-msa-boundary.md) | Standalone operation and modular CWL MSA boundary | Accepted | partial | Owns cross-service API/credential/persistence boundaries and anti-corruption-layer direction. |
| [0012](0012-temporal-relational-shared-latent-topic-measurement.md) | Temporal relational shared-latent topic measurement | Accepted | partial | Owns TEPP topic-measurement contract and fitted artifact semantics; capability promotion remains evidence-gated. |
| [0013](0013-bitemporal-persistence-reproducibility-and-split-authority.md) | Bitemporal persistence, reproducibility, and split authority | Accepted | partial | Owns PostgreSQL adapter semantics, immutable manifests, and leakage-safe split identity. |
| [0014](0014-scientific-claim-promotion-and-release-evidence.md) | Scientific claim promotion and release evidence | Accepted | partial | Keeps Validation Evidence distinct from the decision that promotes a scientific/product claim. |
| [0015](0015-autonomous-development-review-and-merge-authority.md) | Autonomous development, review, and merge authority separation | Accepted | active-PR | Keeps proposal, verification, publication, independent review, and merge/release authority separate. |
| [0016](0016-tdt-chronos-event-intelligence-boundary.md) | TDT, CHRONOS, and Event Ontology intelligence boundary | Accepted | active-PR | Separates observed evidence, event detection/tracking, prediction/schema inference, consistency, and promoted transition authority. |
| [0017](0017-hourly-contextual-orchestrator-gateway.md) | Hourly contextual-orchestrator gateway and provider discovery | Accepted | active-PR | Owns proposal-model gateway/discovery boundary; deterministic verification and merge authority remain separate. |
| [0018](0018-consumer-scoped-analysis-run-ingress.md) | Consumer-scoped modular analysis-run ingress | Accepted | active-PR | Owns closed consumer admission and consumer-qualified idempotency semantics. |
| [0019](0019-project-history-wire-size-symmetry.md) | Symmetric project-history wire-size enforcement | Accepted | active-PR | Owns bounded request/result size symmetry for the LineageWeave project-history boundary. |
| [0020](0020-span-grounded-semantic-units.md) | Span-grounded semantic units; language tags are not identity | Accepted | active-PR | First production slice of ADR 0004; does not claim completed multilingual invariance. |
| [0021](0021-lineageweave-project-history-boundary.md) | LineageWeave project-history service boundary | Accepted | active-PR | Owns the credential-free bounded project-history API while LineageWeave retains its authorization/domain authority. |
| [0022](0022-deterministic-analysis-run-execution.md) | Deterministic cutoff-safe analysis-run execution | Accepted | active-PR | Canonical identity for the analysis-engine execution decision; the pre-normalization colliding 0021 text is preserved under `archive/`. |
| [0023](0023-lineage-criterion-anchor-contract.md) | TEPP-owned Event Lineage criterion anchor | Accepted | active-PR | Owns independent criterion-anchor contract; estimator execution remains separately evidence-gated. |
| [0024](0024-independent-topic-importance-anchor.md) | Posterior topic-context producer contract | Accepted | contract-only active-PR | Owns the topic-context producer DTO/schema; fast-mlsirm owns reusable case-deletion influence arithmetic. |
| [0025](0025-macos-native-rust-mlx-metal-boundary.md) | macOS-native Rust-owned MLX Metal execution | Accepted | accepted-target | Owns native Metal execution/parity receipt boundary; Linux must not claim Metal execution. |
| [0026](0026-lineage-pair-criterion-and-project-journey-posterior.md) | Independent lineage-pair criterion and posterior Project Journey | Proposed | active-PR | Proposed successor identity for the pre-normalization colliding 0024 decision; it is not architecture authority unless and until accepted. |

## Pre-normalization lineage

The historical file [`archive/pre-normalization-0021-deterministic-analysis-run-execution.md`](archive/pre-normalization-0021-deterministic-analysis-run-execution.md) preserves the earlier branch-era ADR 0021 text byte-for-byte after that number was found to collide with the LineageWeave project-history decision. The canonical analysis-run execution decision is ADR 0022. Historical branch references to the colliding identity remain provenance, not current architecture authority.

The former lineage-pair/Project-Journey ADR 0024 collision is represented by proposed ADR 0026; accepted ADR 0024 remains the Posterior topic-context producer contract. No historical collision authorizes reuse of a retired number for a new decision, and a proposed successor does not become architecture authority merely because implementation work exists.

## Decision ownership summary

Use the narrowest owning **Accepted** ADR when decisions overlap. Proposed ADRs are recorded separately and are not architecture authority until accepted:

- numerical implementation/reference backend: ADR 0001;
- clock/time eligibility: ADR 0002;
- event ontology, relation, and membership semantics: ADR 0003;
- multilingual semantic alignment: ADR 0004; span-grounded unit identity: ADR 0020;
- ESEM/DSEM and longitudinal psychometric interpretation: ADR 0005;
- GPU/VRAM compute boundary: ADR 0006;
- repository quality tooling: ADR 0007;
- evidence identity/spans/wire reconstruction: ADR 0008;
- privacy and purpose-bound disclosure: ADR 0009;
- LLM test-time compute/orchestration: ADR 0010;
- standalone/CWL MSA service authority: ADR 0011;
- topic measurement and model-artifact contract: ADR 0012;
- persistence/manifests/leakage-safe split: ADR 0013;
- scientific claim maturity/release evidence: ADR 0014;
- autonomous development/review/merge authority: ADR 0015;
- TDT/CHRONOS event intelligence: ADR 0016;
- hourly contextual-orchestrator proposal gateway: ADR 0017;
- modular analysis-run consumer admission: ADR 0018;
- project-history wire-size symmetry: ADR 0019;
- LineageWeave project-history boundary: ADR 0021;
- deterministic analysis-run execution: ADR 0022;
- Event Lineage criterion anchor: ADR 0023;
- Posterior topic-context producer contract: ADR 0024;
- macOS-native Rust-owned MLX/Metal boundary: ADR 0025;
- proposed independent lineage-pair criterion and posterior Project Journey: ADR 0026 (`Proposed`; not current architecture authority).

## Change and supersession rule

ADR status changes require a pull request, source traceability, tests/evidence for affected invariants, and corresponding PRD/Architecture/TRD/Traceability updates where the approved measurement target changes. Decisions that materially alter privacy authority, orchestration authority, service ownership, temporal semantics, ontology, persistence identity, scientific estimands, release evidence, or automation authority require a superseding ADR rather than silent drift.

Partial supersession must identify the exact moved decision scope in both the older ADR and this index. Historical ADR text remains evidence of why a decision existed; it must not be silently rewritten to make a later architecture appear original. A pre-normalization collision may be archived outside the numbered root after its bytes are preserved and the canonical successor is explicit.
