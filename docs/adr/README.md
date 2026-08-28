# Architecture Decision Records

TEPP uses numbered ADRs for decisions that constrain latent-variable meaning, temporal semantics, event ontology, multilingual measurement, numerical backends, privacy/security, persistence, orchestration, automation authority, scientific claims, and modular service boundaries.

Read [`ADR_POLICY.md`](ADR_POLICY.md) first. **Decision status and implementation maturity are independent. `Accepted` means the architecture decision is authoritative; it does not mean the capability is implemented or released.**

| ADR | Decision | Decision status | Implementation maturity | Clarification / supersession |
|---|---|---|---|---|
| [0001](0001-rust-first-modular-msa.md) | Rust-first numerical core and CPU `f64` reference | Accepted | partial | Rust owns production arithmetic and the CPU reference; GPU and estimator completion remain target work. |
| [0002](0002-six-clock-temporal-semantics.md) | Six-clock temporal semantics and fail-closed historical eligibility | Accepted | partial | Protected main owns typed clocks and Allen algebra; the active consolidation PR carries clock identity, availability/cutoff, revision, provenance, and ordering gates. |
| [0003](0003-relational-event-multiple-membership.md) | Relational event ontology and time-varying multiple membership | Accepted | partial | Protected main owns the membership network and forward-transition foundation; the active consolidation PR carries typed target and identity slices. Full multilevel/MMMC estimators and persistence remain target work. |
| [0004](0004-shared-multilingual-latent-space.md) | Shared multilingual latent semantic space | Accepted | partial | Shared-space estimation and measurement invariance remain the scientific target; ADR 0020 owns span-grounded units. |
| [0005](0005-posterior-esem-dsem.md) | Posterior-aware ESEM/DSEM and compositional coordinates | Accepted | partial | CPU `f64` fit and longitudinal within/between slices are active; invariance and multilevel estimators remain target work. |
| [0006](0006-vram-gpu-nvidia-orchestration.md) | VRAM-adaptive GPU compute and credential boundary | Accepted | active-PR | GPU streaming/parity and backend completion remain target work; orchestration policy belongs to ADR 0010. |
| [0007](0007-rust-workspace-quality-gates.md) | Rust workspace, toolchain, and quality gates | Accepted | implemented-main | Repository quality contracts are implemented; scientific claim promotion belongs to ADR 0014. |
| [0008](0008-immutable-evidence-identities-digests-and-spans.md) | Immutable evidence identities, digests, spans, and wire reconstruction | Accepted | implemented-main | Identity and span contracts are implemented-main; untrusted payload bounds are active in the consolidation PR. |
| [0009](0009-purpose-bound-pii-governance.md) | Purpose-bound PII governance without blanket masking | Accepted | partial | Opaque analytical IDs, purpose grants, provider minimization, retention, and encrypted mapping are covered; deployment evidence and persistent access storage remain target work. |
| [0010](0010-adaptive-llm-orchestration.md) | Adaptive LLM orchestration and test-time compute | Accepted | partial | Direct/verify/committee routing and ablation contracts exist; live provider execution and production calibration remain target work. |
| [0011](0011-standalone-modular-msa-boundary.md) | Standalone operation and modular CWL MSA boundary | Accepted | partial | Versioned service boundaries and credential separation are authoritative; production TLS and live ports remain target work. |
| [0012](0012-temporal-relational-shared-latent-topic-measurement.md) | Temporal relational shared-latent topic measurement | Accepted | partial | Coordinates and the CPU `f64` reference estimator are implemented-main; fitted candidate-`K` scoring is this PR; method effects and GPU remain accepted-target. |
| [0013](0013-bitemporal-persistence-reproducibility-and-split-authority.md) | Bitemporal persistence, reproducibility, and split authority | Accepted | partial | Migration, tenant, append-only, interval, and live SQL contracts are present; physical ERD and recovery depth remain target work. |
| [0014](0014-scientific-claim-promotion-and-release-evidence.md) | Scientific claim promotion and release evidence | Accepted | partial | Exact-head promotion authority and repository evidence exist; the complete release bundle remains target work. |
| [0015](0015-autonomous-development-review-and-merge-authority.md) | Autonomous development, review, and merge authority separation | Accepted | active-PR | Proposal, deterministic verification, publication, independent review, and merge/release authority remain separate. |
| [0016](0016-tdt-chronos-event-intelligence-boundary.md) | TDT, CHRONOS, and event-intelligence boundary | Accepted | active-PR | Span-grounded mentions with exact-extent recovery are active on this PR; unified TDT/CHRONOS workflow, interval consistency, persistence, and exports remain target work. |
| [0017](0017-hourly-contextual-orchestrator-gateway.md) | Hourly contextual-orchestrator gateway and provider discovery | Accepted | active-PR | Proposal-model execution is pinned behind a loopback gateway and remains separate from verification and merge authority. |
| [0018](0018-consumer-scoped-analysis-run-ingress.md) | Consumer-scoped modular analysis-run ingress | Accepted | active-PR | Closed consumer registry, credential-free exchange, and consumer-qualified idempotency are active. |
| [0019](0019-project-history-wire-size-symmetry.md) | Symmetric LineageWeave project-history wire-size enforcement | Accepted | active-PR | Request serialization and generated project-history projections share bounded size rules. |
| [0020](0020-span-grounded-semantic-units.md) | Span-grounded semantic units; language tags are not identity | Accepted | active-PR | First ADR 0004 production slice; concept alignment, invariance, and topic estimation are not claimed. |
| [0021](0021-lineageweave-project-history-boundary.md) | LineageWeave project-history service boundary | Accepted | active-PR | Credential-free bounded project-history API preserves LineageWeave authorization ownership. |
| [0022](0022-deterministic-analysis-run-execution.md) | Deterministic cutoff-safe analysis-run execution | Accepted | active-PR | Closes the first executable product path from accepted run to digest-bound terminal result without claiming estimator authority. |
| [0023](0023-lineage-criterion-anchor-contract.md) | TEPP-owned Event Lineage criterion anchor | Accepted | active-PR | PR #237 publishes the strict accepted/rejected artifact and identities; estimator execution remains fail-closed future work. |
| [0024](0024-independent-topic-importance-anchor.md) | Posterior topic-context producer contract | Accepted | active-PR | The CPU reference path emits joint draws and plausible values; fast-mlsirm owns case-deletion influence. |
| [0025](0025-macos-native-rust-mlx-metal-boundary.md) | macOS-native Rust-owned MLX Metal execution | Accepted | accepted-target | Compose authenticates to a native host service; Linux never claims Metal, and actual backend/parity receipts fail closed. |
| [0026](0026-lineage-pair-criterion-and-project-journey-posterior.md) | Independent Event Lineage pair criterion and posterior Project Journey | Proposed | active-PR | Strict artifacts preserve criterion/event-time draws, branches, ties, and CPU/GPU receipts without claiming the scientific estimator is complete. |

## Decision ownership summary

Use the narrowest owning ADR when decisions overlap:

- **numerical implementation / reference backend:** ADR 0001;
- **clock/time eligibility:** ADR 0002;
- **event ontology / relation / membership semantics:** ADR 0003;
- **multilingual semantic alignment:** ADR 0004; span-grounded unit identity: ADR 0020;
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
- **TDT/CHRONOS event intelligence:** ADR 0016;
- **hourly proposal gateway and provider discovery:** ADR 0017;
- **modular consumer admission / replay identity:** ADR 0018;
- **project-history wire-size symmetry:** ADR 0019;
- **LineageWeave project-history service boundary:** ADR 0021;
- **accepted-run execution and terminal artifact production:** ADR 0022;
- **TEPP-owned accepted/rejected lineage criterion anchor:** ADR 0023;
- **posterior topic-context producer contract:** ADR 0024;
- **macOS-native Rust-owned MLX Metal execution:** ADR 0025;
- **independent lineage-pair criterion and posterior Project Journey:** ADR 0026.

## Change and supersession rule

ADR status changes require a pull request, source traceability, tests/evidence for affected invariants, and corresponding PRD/Architecture/TRD/Traceability updates where the approved measurement target changes. Decisions that materially alter privacy authority, orchestration authority, service ownership, temporal semantics, ontology, persistence identity, scientific estimands, release evidence, or automation authority require a superseding ADR rather than silent drift.

Partial supersession must identify the exact moved decision scope in both the older ADR and this index. Historical ADR text remains evidence of why a decision existed; it must not be silently rewritten to make a later architecture appear original.
