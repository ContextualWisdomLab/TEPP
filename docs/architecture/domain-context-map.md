# TEPP Domain Context Map

**Status:** Delivery-refactoring authority for the 2026-09-01 queue-consolidation cycle.
**Cross-service authority:** ADR 0011.

This document applies Domain-Driven Design to the protected-main product. Cargo crates are implementation units; they are not automatically bounded contexts. A crate, ADR number, refusal helper, transport operation, clock type, or equation earns a separate boundary only when it has an independently meaningful domain lifecycle, ubiquitous language, invariants, and reuse boundary.

## Strategic design

### Core subdomains

| Bounded context | Product responsibility | Aggregate / authority | Current implementation nucleus |
| --- | --- | --- | --- |
| Evidence & Semantic Measurement | preserve source evidence and derive span-grounded semantic/concept observations without replacing source truth | `EvidenceCorpus`, `SemanticUnitSet`, `ConceptDictionaryRevision` | `evidence_core`, `semantic_core` |
| Temporal Event Knowledge | represent the six-clock contract, event identity, interval relations, typed provenance/transition edges, and time-varying memberships | `EventEpisode`, `TemporalRelationSet`, `MembershipAssignmentSet` | `temporal_core`, `event_core`, `relation_graph`, `membership_core` |
| Topic Measurement | estimate shared-latent temporal topic coordinates and uncertainty; preserve topic identity through activity/dormancy/reactivation | `TopicModelRun`, `TopicLineage` | `topic_measurement`, `topic_lineage`, `model_selection`, `network_analysis` |
| Longitudinal Modeling | compose TEPP-owned temporal/event state evolution, longitudinal invariance/drift, irregular time and time-varying generalized-mixed structure around released static psychometric contracts | `TemporalModelSpecification`, `TemporalModelRun` | `longitudinal_core`; reusable static kernels migrate to fast-mlsirm |
| Analysis Run | orchestrate cutoff-safe accepted work into durable execution and typed terminal artifacts without owning scientific formulas | `AnalysisRun` | `analysis_engine` |
| Scientific Validation | produce method-specific recovery/coverage/parity/invariance/convergence evidence | `ValidationStudy` | `validation_core`, `tepp_simulation` |
| Scientific Claim Promotion | decide whether a claim may be promoted under a preregistered complete evidence contract | `ClaimPromotionDecision` | ADR 0014 policy; implementation incomplete |
| Projection | publish authorized read models/exports without changing domain or scientific authority | projection-specific read models | `tepp_api`, artifact/export adapters |

### Supporting subdomains

| Bounded context | Responsibility | Implementation nucleus |
| --- | --- | --- |
| Interpretation | evidence-grounded interpretation and independent verification; never numerical authority | `interpretation_gateway` plus contextual-orchestrator ACL |
| Persistence & Recovery | durable run/artifact storage, restart/recovery, outbox/checkpoint semantics | `persistence_postgres`, `checkpoint_authority` |
| Runtime Security & Operations | authenticated intake, TLS, operational audit, provider receipts, encrypted mappings | runtime/security adapters |

### Generic subdomains

`compute_backend`, accelerator adapters, serialization, hashing, and transport framing are generic infrastructure. They may serve several bounded contexts but may not define domain truth or scientific validity.

## Context map

```mermaid
flowchart LR
    ES[Evidence & Semantic Measurement] --> TE[Temporal Event Knowledge]
    ES --> TM[Topic Measurement]
    TE --> TM
    TE --> LM[Longitudinal Modeling]
    TM --> LM
    TM --> AR[Analysis Run]
    LM --> AR
    AR --> VE[Scientific Validation]
    VE --> CP[Scientific Claim Promotion]
    AR --> PJ[Projection]
    CP --> PJ
    PJ --> IN[Interpretation]
    AR --> PR[Persistence & Recovery]
    AR --> RO[Runtime Security & Operations]
    CB[Compute Backend] -. generic service .-> TM
    CB -. generic service .-> LM
    CB -. generic service .-> VE
    FM[fast-mlsirm Model Specification / Numerical Kernel] -- released versioned contract --> LM
    CO[contextual-orchestrator] -- ACL --> IN
```

Dependency direction follows the arrows. Transport, persistence, UI, and provider adapters may depend on domain/application contracts; domain code must not depend on HTTP, PostgreSQL, CLI, or provider-specific representations.

## Anti-corruption layers

- `tepp_api` is an HTTP/CLI/projection adapter around Analysis Run and published read models. It must not own scientific acceptance, estimator formulas, event semantics, or persistence truth.
- `contextual-orchestrator` provider vocabulary is translated into TEPP interpretation/application contracts before crossing the boundary. Direct provider calls are prohibited.
- `fast-mlsirm` is consumed through a released/versioned candidate and numerical-kernel contract. TEPP does not copy open-PR source as a dependency.
- `persistence_postgres` implements repositories owned by domain/application contexts. Other contexts must not query its tables directly.
- compute backends expose execution receipts; a backend receipt is not a scientific result.
- Naruon, LineageWeave, Context Graph, and EA Core are external contexts. Their identifiers and transport vocabulary remain behind explicit adapters rather than leaking into core aggregates.

## Ubiquitous language and invariants

- **Evidence** is immutable source-backed observation, not an inferred fact.
- **Semantic unit** is a span-grounded measured unit tied to source offsets and a versioned concept dictionary.
- **Event or valid time** is the first of six temporal roles; an event instant and validity interval are alternative representations of when a state/event holds, not separate analysis clocks.
- **Available time** is when evidence becomes usable; **knowledge cutoff** is an analysis eligibility boundary. They are never aliases for event/valid time.
- **Transition edge** is forward-only state/process change; citation, summary, retrospective report, and support are provenance/evidence relations and never become transitions by coercion.
- **Membership assignment** may be cross-classified or multiple-membership and must not collapse language, template, department, project, location, or role into one entity identity.
- **Temporal model specification** composes time around an exact released upstream candidate identity; it does not rewrite the base family or dependence semantics.
- **Analysis run** owns lifecycle, idempotency, execution identity, and terminal artifact binding; it does not redefine estimators.
- **Validation evidence** records method-specific recovery/coverage/parity/invariance/convergence evidence. It is not a global scientific acceptance decision.
- **Claim promotion decision** may be made only from a preregistered, versioned evidence contract whose required dimensions are complete. LLM output cannot satisfy a numerical evidence requirement.

## Crate-boundary repair register

The following protected-main crates look like rule fragments rather than independent bounded contexts. They are retained temporarily to preserve remote-head compatibility, but new work treats them as modules/value objects/invariants of the owning context and folds them when the corresponding product-vertical landing vehicle is replayed onto current main.

| Current crate fragments | Owning bounded context |
| --- | --- |
| `system_clock`, `event_clock`, `assertion_clock`, `cutoff_clock`, `available_clock`, `document_clocks`, `revision_order` | Temporal Event Knowledge |
| `summarizes_edge`, `retrospective_edge`, `support_edge`, `citation_edge`, `outcome_order`, `subevent_containment`, `prediction_contradiction`, `relation_absence`, `role_contradiction` | Temporal Event Knowledge |
| `location_membership`, `episode_membership`, `membership_target` | Temporal Event Knowledge |
| `prompt_source`, `style_source`, `modality_source`, `copied_text`, `copy_identity`, `corpus_background`, `stopword_deletion`, `payload_bound`, `derived_sensitivity` | Evidence & Semantic Measurement |
| event-time correlation standardization | Longitudinal Modeling / `longitudinal_core` |
| reusable static/generalized-mixed/dependence psychometric arithmetic | `fast-mlsirm` owner path after parity/recovery |
| flat `analysis_engine` one-profile files | owning Analysis Run capability module |

A fold is permitted only after comparing the exact remote head and preserving unique tests, doctoring, research citations, and public compatibility. Public compatibility, when required, is provided by explicit adapters/re-exports with a removal plan; legacy paths do not remain canonical merely to avoid a refactor.

## Analysis Run directory rule

New `analysis_engine` profiles are organized by the owning domain capability, not as an indefinitely growing flat list of one-file refusals or one-route features:

```text
analysis_engine/src/
  runs/
  evidence_measurement/
  topic_measurement/
  longitudinal_modeling/
  event_intelligence/
  validation/
```

Transport-only features stay in `tepp_api`; scientific claim promotion remains a separate policy boundary instead of being hidden inside one validation-run helper.

## Migration rule

Do not perform a repository-wide path rename while a large remote PR fleet is open. Apply path repairs incrementally inside the selected landing vehicle for each bounded context, then close or retarget superseded micro-PRs with exact-head replacement mappings. The target architecture is mandatory; the migration is staged only to preserve concurrent-agent intent and review evidence.