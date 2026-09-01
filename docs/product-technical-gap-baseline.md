# Product and Technical Gap Baseline

**Status:** Active delivery recovery  
**Product:** Temporal Event Psychometrics Platform (TEPP)  
**Snapshot:** 2026-09-01T09:14:20Z  
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`  
**Workspace version:** `0.2.0`  
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)  
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md)

## Purpose

This is the operator-facing authority for the current product and technical gaps. Historical queue snapshots remain in Git history; stale counts and branch-local architecture claims are not carried forward as current facts.

A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Re-read live GitHub state before every merge, release, scientific claim, or customer-facing maturity claim.

## Live snapshot

| Signal | Current evidence | Delivery implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until protected main changes. |
| Open pull requests | **149** | The queue is release-blocking and grew from the initial 141 while consolidation remained open. |
| Draft pull requests | **100** | Draft state is not classification or ownership. |
| Non-draft pull requests | **49** | Ready metadata is not merge readiness without exact-head evidence. |
| Open issues | **14** | Includes #437 for repository-wide ADR identity normalization. |
| GitHub releases | **0** | No open PR head or local branch is a released product contract. |
| Effective ruleset | `18156473` | Current ruleset, exact-head workflows, resolved conversations, and qualifying review are the merge authority. |

The repository-level classic branch-protection payload is not the sole policy source. Do not infer that protection is absent from an empty classic required-check list and do not bypass the organization ruleset.

## Current priority open pull-request evidence

This table is an exact-head **priority subset**, not a row-for-row copy of the 149-PR queue. The live total and this operator inventory are deliberately different concepts.

PR #435 itself is deliberately omitted from the exact-head table. A commit that rewrites this document necessarily advances #435's head, so embedding that same branch's “exact current head” inside the commit would be self-invalidating. The delivery-authority line links #435, while its live head is re-read from GitHub immediately before any readiness or merge decision.

| PR | Exact current head | Draft | Base | Ownership / disposition |
| ---: | --- | :---: | --- | --- |
| #443 | `504793d88c6b754f5181f48dc7abde073ff9146a` | false | #411/export stack | Analysis Run export collection adapter; `fold_into_landing_vehicle` candidate |
| #441 | `c1aeed3bc2ca5f801f3baa748a4a3dde9f948338` | false | main | Longitudinal Modeling lagged-correlation repair; original invalid ratio removed; exact-head checks pending |
| #436 | `460503b6e787362b702509faa955c4730f6d8680` | false | #433 head | Analysis Run/contextual-orchestrator CLI stack; `stacked_dependency` |
| #434 | `c0fbaabd8c95e69407c3b9e50f8d1846bd949598` | false | main | membership-target refusal profile; `fold_into_landing_vehicle` candidate |
| #432 | `3e09ff29cc89ef97a859f3ae50e1297846dd2eeb` | false | main | Topic Measurement profile binding; preserve unique contract tests |
| #425 | `c11558313dd1b95d7528eb5fcb89ad296cf879c0` | false | main | Analysis Run/contextual-orchestrator create CLI stack root candidate |
| #389 | `035bfb087d47543fd7dd87cfdbc4edd778f4a6aa` | false | main | irregular event-time composition; Longitudinal Modeling fold candidate |
| #356 | `df33bfa3e61ae4de3dbfae16df0deac12d2f4003` | false | main | Validation Evidence candidate; scientifically blocked from global claim promotion |

Passing or queued Checks on an open PR never promote that PR to implemented-main.

## Strategic Domain-Driven Design baseline

Cargo crates are implementation units, not bounded contexts. Current product responsibility is organized as follows.

| Subdomain | Bounded context | Aggregate authority | Primary implementation nucleus |
| --- | --- | --- | --- |
| Core | Evidence & Semantic Measurement | `EvidenceCorpus`, `SemanticUnitSet`, `ConceptDictionaryRevision` | `evidence_core`, `semantic_core` |
| Core | Temporal Event Knowledge | `EventEpisode`, `TemporalRelationSet`, `MembershipAssignmentSet` | `temporal_core`, `event_core`, `relation_graph`, `membership_core` |
| Core | Topic Measurement | `TopicModelRun`, `TopicLineage` | `topic_measurement`, `topic_lineage`, `model_selection`, `network_analysis` |
| Core | Longitudinal Modeling / Temporal Psychometric Composition | `TemporalModelSpecification`, `TemporalModelRun` | `longitudinal_core` plus TEPP temporal composition around versioned fast-mlsirm contracts |
| Core | Analysis Run | `AnalysisRun` | `analysis_engine` application services; transport remains adapter-owned |
| Core | Scientific Validation & Claim Promotion | `ValidationStudy`, `ClaimPromotionDecision` | `validation_core`, `tepp_simulation`; claim promotion is distinct from evidence generation |
| Supporting | Interpretation | evidence-grounded interpretation workflow | contextual-orchestrator ACL only |
| Supporting | Persistence & Recovery | repository implementations and durable receipts | `persistence_postgres` and object-store adapters |
| Supporting | Runtime Security & Operations | authenticated tenancy/operations | runtime adapters |
| Generic | Compute backend | backend execution receipt | CPU/GPU/MLX adapters; receipt is not scientific authority |

### Canonical owner boundaries

- **fast-mlsirm** owns reusable static/generalized-mixed/dependence-aware psychometric model specification and numerical kernels, including LSIRM/MLSIRM/DLSJM.
- **TEPP** owns temporal/event composition: six-clock semantics, cutoff safety, irregular time, time-varying covariates/random effects/memberships, longitudinal invariance/drift, state evolution, event ontology/graph, temporal alignment and temporal recovery.
- **contextual-orchestrator** owns every LLM provider call, routing/fallback, credential, verifier/adjudicator execution, and model-call provenance.
- **LineageWeave or the consuming product** owns source/item-generation lineage; lineage is evidence, not numerical authority.

At this snapshot, fast-mlsirm PR #1714 at `cf538931199c4433a2c018c970d3609e17939505` proposes the non-numerical candidate compiler. It preserves full candidate identity and formulation-scoped evidence but remains an unreleased upstream dependency. A TEPP consumer-boundary review also requires an explicit published-manifest schema/version/digest and typed generalized-mixed membership/weight semantics before this can serve as TEPP's versioned Published Language. TEPP does not copy its implementation or bind deployable behavior to the open head.

### Dependency invariants

- Domain/application code does not depend on HTTP, CLI, PostgreSQL tables, provider SDK DTOs, or UI state.
- `tepp_api` is an adapter around Analysis Run and published read models; it does not own estimator mathematics, temporal truth, or scientific claim promotion.
- Persistence adapters implement repository contracts. Cross-context direct SQL is prohibited.
- External contexts are isolated behind versioned anti-corruption layers.
- A compute receipt proves execution of the named operation, not scientific validity.
- LLM output may propose or verify an interpretation but cannot satisfy a numerical recovery contract or promote a scientific claim.

## Temporal dependence composition

TEPP composes time over the **full upstream candidate identity**, not over hard-coded family names. A compatible new fast-mlsirm base family therefore inherits temporal-candidate compilation without a TEPP family-specific wrapper.

Every TEPP temporal candidate records the upstream contract version/digest, exact base formulation, generalized-mixed structure, dependence structure, clock roles, event/occasion semantics, state equation, temporal identification/alignment rules, time-varying membership/covariates, estimator owner, recovery contract, citations, and one status: `supported`, `research_candidate`, or `unsupported`.

Auto-expansion is not auto-activation. Unknown or novel couplings remain `research_candidate`; incoherent couplings are `unsupported`. TEPP never silently simplifies a dependence-aware temporal request to a static or local-independent model.

### LSIRM / MLSIRM

LSIRM residual person-item interaction keeps person/item interaction positions, distances and interaction strength separate from known hierarchy, testlets, raters, methods and covariates. MLSIRM is the **multidimensional-main-effect** latent-space extension; multilevel, cross-classified and multiple-membership operators are orthogonal generalized-mixed structure. Temporal maps require declared translation/rotation/reflection identification before coordinates or trajectories are compared across occasions.

### DLSJM

DLSJM follows Jin and Jeon (2019) as the baseline joint model for distinct local item-dependence and local person-dependence spaces. TEPP preserves distinct item-space and person-space temporal states, distances, clusters, uncertainty and alignment. Raw maps or raw cluster labels from separate occasions are not longitudinal evidence without alignment.

### Base families and generalized mixed structure

Rasch remains distinct from generic 1PL. 2PLM through formulation-qualified 5PLM retain exact parameter meanings. Confirmatory/exploratory MIRT and ideal-point/GGUM response processes remain distinct axes from hierarchy, dependence and time. Testlet, rater/facet, nested, crossed, cross-classified and multiple-membership structures remain explicit and cannot be hidden inside latent-space dependence.

Multiple-membership weights are explicit, auditable and time-valid. They are observed/normalized or estimated according to the declared formulation; equal weights are never invented as a fallback.

## Active product and scientific gaps

| ID | Gap | Maturity | Authority | Closure evidence |
| --- | --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 149 heads | `release-blocking` | #175 / #435 | exact-head classification, bounded-context landing vehicles, unique evidence preserved, safe queue reduction |
| GAP-002 | multilingual span-grounded semantic/concept admission incomplete | `partial` | Evidence & Semantic Measurement | immutable offsets/layout, language profiles, concept dictionary, unknown review, invariance/calibration, hostile-input tests |
| GAP-003 | shared-latent temporal topic estimator incomplete | `partial` | #167 | Rust CPU `f64` likelihood/estimands/uncertainty, time/relation/membership effects, multi-seed recovery, real candidate-K fits |
| GAP-004 | durable end-to-end Analysis Run incomplete | `partial` | #166 | idempotent lifecycle, persistence/recovery, estimator-bound artifacts, complete validation evidence, separate claim promotion, Compose E2E |
| GAP-005 | temporal psychometric composition fragmented and partly duplicated | `partial` | #169 + fast-mlsirm owner boundary | versioned upstream model/dependence contract, TEPP temporal ACL, invariance/alignment, irregular time, temporal recovery, duplicate static kernels removed after parity |
| GAP-006 | TDT/CHRONOS event workflow incomplete | `partial` | #170 | calibrated event evidence/detection/tracking/schema/interval consistency with recovery and durable artifacts |
| GAP-007 | real accelerator/memory evidence incomplete | `accepted-target` | #171 | real hardware execution, CPU f64 parity, bounded memory/OOM/fallback evidence |
| GAP-008 | posterior network/cluster buyer workflow incomplete | `partial` | #172 | known-truth network recovery, uncertainty/stability, repeated Leiden consensus, exact-value exports |
| GAP-009 | production interpreter/verifier incomplete | `partial` | #176 | contextual-orchestrator execution, evidence citations, independent verifier, ablations, abstention/fallback |
| GAP-010 | coordinated accessible buyer UI incomplete | `accepted-target` | #173 | Figma/Storybook/design tokens, keyboard/touch/error/empty states, exact-value and print/export provenance |
| GAP-011 | operable multi-tenant supported release incomplete | `accepted-target` | #174 | OIDC/RLS/purpose controls, durable queues/storage, OTel/SLO, restore/load/migration, signed release/SBOM/provenance |
| GAP-012 | directory/crate structure obscures domain ownership | `active-refactor` | #435 and landing vehicles | staged folds, compatibility ACLs, no cycles/cross-context persistence/shared-kernel creep |
| GAP-013 | ADR identity is branch-local/duplicated in parts of the queue | `release-integrity` | #437 | repository-wide unique ADR identity, duplicate detection, normalized index and supersession lineage |

## Delivery sequence

1. Restore queue, ADR and bounded-context authority (#175, #435, #437).
2. Consolidate Evidence & Semantic Measurement.
3. Complete the Rust CPU `f64` shared-latent topic estimator (#167).
4. Complete durable end-to-end Analysis Run and evidence/promotion separation (#166).
5. Compose temporal psychometrics through fast-mlsirm contracts (#169) and complete Event Intelligence (#170) instead of producing one-equation product slices.
6. Add real accelerator parity after CPU scientific authority is established (#171).
7. Complete network/cluster, interpretation and buyer visual workflows (#172/#176/#173).
8. Productionize tenancy, durability, observability, recovery, release and support (#174).

A bounded dependency repair may land ahead of this sequence when it directly unblocks a selected landing vehicle. It does not create a new product priority.

## Queue consolidation rules

Every open PR receives one of:

- `landing_vehicle`
- `stacked_dependency`
- `fold_into_landing_vehicle`
- `superseded`
- `duplicate`
- `research_lineage_only`
- `blocked_external`

Similarity of titles is insufficient for closure. Compare exact current heads and preserve unique production behavior, public compatibility, tests, review findings, primary research, doctoring and provenance before applying `duplicate`, `superseded`, or `fold_into_landing_vehicle`.

One-rule crates, one-clock crates and one-operation API/CLI PRs are not independent product boundaries by default.

### Current classifications

**#441 — Longitudinal Modeling lagged-correlation repair.** The invalid predecessor API that divided lagged covariance by only the earlier marginal variance was removed from the final diff after review showed it could produce impossible autocorrelations under nonstationary marginals. Exact repair head `c1aeed3bc2ca5f801f3baa748a4a3dde9f948338` instead places event-time association standardization in `longitudinal_core`, requires both marginal variances, enforces the covariance bound, and avoids the avoidable variance-product overflow. The predecessor commit remains RED/scientific-failure lineage. The branch is not merge-ready until exact-head hosted checks and fresh review complete.

**#443 — Analysis Run export adapter fold candidate.** The export-collection GET is useful operator behavior, but one route is not an independent bounded context. Preserve its pagination, authorization and refusal tests while folding the operation into a coherent Analysis Run/export landing vehicle.

**#356 — Validation Evidence candidate.** Keep useful run binding, but do not make it global scientific-acceptance authority. Its generic `RMSE <= k * SE(RMSE)` rule does not bound absolute recovery error when RMSE and its SE arise from the same residual vector; caller-provided truth/recovered vectors and an `authored_by_llm` boolean also do not establish estimator-owned provenance. Graph recovery, invariance, convergence and applicable CPU/GPU parity remain incomplete. Scientific Claim Promotion is a separate aggregate governed by ADR 0014.

**#352/#355 — Longitudinal Modeling fold candidates.** They share a Driver/ctsem scalar rewrite but retain different public names, refusal guards, tests and research documentation. Preserve the strongest evidence before closing either.

**#425/#433/#436 and similar route/CLI stacks — Analysis Run adapter dependencies.** They are transport/application behavior, not bounded contexts and not grounds for branch-local ADR identity.

**#434/#430/#427/#426/#422/#421/#419/#418/#416 and similar refusal profiles — owning-context fold candidates.** Evidence/method refusals belong to Evidence & Semantic Measurement; temporal relation/membership rules belong to Temporal Event Knowledge.

## Directory and crate repair register

| Current fragments | Owning context |
| --- | --- |
| `system_clock`, `event_clock`, `assertion_clock`, `cutoff_clock`, `available_clock`, `document_clocks`, `revision_order` | Temporal Event Knowledge |
| `summarizes_edge`, `retrospective_edge`, `support_edge`, `citation_edge`, `outcome_order`, `subevent_containment`, `prediction_contradiction`, `relation_absence`, `role_contradiction` | Temporal Event Knowledge |
| `location_membership`, `episode_membership`, `membership_target` | Temporal Event Knowledge |
| `prompt_source`, `style_source`, `modality_source`, `copied_text`, `copy_identity`, `corpus_background`, `stopword_deletion`, `payload_bound`, `derived_sensitivity` | Evidence & Semantic Measurement |
| reusable static/generalized-mixed/dependence psychometric arithmetic in TEPP | migrate to fast-mlsirm after parity/recovery; retain only TEPP temporal/event ACL and policy |
| event-time association standardization | `longitudinal_core`; require both marginals and keep model-specific covariance construction outside this generic standardizer |
| flat `analysis_engine` one-profile files | fold by owning domain/application module through landing vehicles |

Do not run a repository-wide rename across more than one hundred live heads. The target DDD architecture is mandatory, but path migration is staged through landing vehicles so concurrent-agent intent and review evidence survive.

## Scientific and data invariants

- Production mathematical/statistical/psychometric/vector/matrix arithmetic is Rust-owned; deterministic CPU `f64` is the scientific numerical reference.
- Event, valid, assertion, document, system, available and knowledge-cutoff semantics stay distinct; historical evidence requires availability at or before cutoff.
- Measurement occasion as a rater/method facet is distinct from substantive event time.
- Process/transition edges are forward-only. Citation, support, summary, revision and retrospective report do not become transitions.
- Cross-classification and weighted multiple membership are preserved rather than forced into one parent hierarchy.
- Local dependence is diagnosed after known factors/testlets/item families/raters/methods/hierarchy/membership/covariates are represented.
- Residual person-item interaction may motivate LSIRM/MLSIRM; joint local-item plus local-person dependence may motivate DLSJM only when its relational formulation matches the question.
- Exploratory factors/loadings and dependence geometry remain hypotheses until confirmatory/invariance/recovery evidence permits production use.
- Every temporal dependence candidate generates model-appropriate known-truth recovery for states/trajectories, fixed/random effects, covariance, membership, factors/loadings, response-family-specific parameters, dependence geometry, dynamics and uncertainty.
- Temporal recovery uses event-time/available-time separation, rolling-origin evaluation, irregular gaps, delayed/retrospective records, missing occasions, changing membership and language/source drift.
- Monte Carlo uncertainty is reported; arbitrary pass percentages or rule-of-thumb thresholds are not scientific promotion criteria.
- LLMs never replace estimation/validation and are called only through contextual-orchestrator.
- Database authority is normalized, tenant/time/provenance aware, uses descriptive multiword `snake_case`, explicit idempotency/UPSERT and measured hot-partition evidence.

## Merge gate

Before a landing vehicle merges:

1. Re-read exact head/base, live ruleset, review state, unresolved threads and current required workflows.
2. Repair valid findings and failures test-first; do not suppress deprecations, coverage or scientific failures.
3. Run relevant Rust unit/integration/property/fuzz/recovery tests on the exact head.
4. Require 100% production statement/branch coverage and public Rust documentation for the shipped scope.
5. Preserve unique evidence from every folded/closed PR.
6. Update PRD/TRD/ADR/context map/UML/traceability/doctoring/CHANGELOG when the protected product contract changes.
7. Use normal merge policy. Force-push and protection bypass are not queue-consolidation tools.

## Operator-gap register

| ID | Closure evidence |
| --- | --- |
| GAP-001 | Queue reaches a bounded, classified set of coherent landing vehicles and all superseded/duplicate closures retain replacement mapping. |
| GAP-003 | Real CPU `f64` estimator passes preregistered known-truth recovery and is invoked by the product workflow. |
| GAP-004 | One durable evidence-to-terminal-result run survives restart/restore and separates Validation Evidence from Claim Promotion. |
| GAP-005 | TEPP consumes a released/versioned fast-mlsirm candidate contract; temporal auto-expansion/recovery is generic; duplicated static kernels are removed after parity. |
| GAP-012 | Architecture fitness tests and landing vehicles demonstrate correct bounded-context paths/dependency direction. |
| GAP-013 | #437 closes with unique repository-wide ADR identities and duplicate-ID validation. |

## Research traceability for dependence ownership

- Jin, I. H., & Jeon, M. (2019). A doubly latent space joint model for local item and person dependence in the analysis of item response data. *Psychometrika, 84*(1), 236–260. https://doi.org/10.1007/s11336-018-9630-0
- Jeon, M., Jin, I. H., Schweinberger, M., & Baugh, S. (2021). Mapping unobserved item–respondent interactions: A latent space item response model with interaction map. *Psychometrika, 86*(2), 378–403. https://doi.org/10.1007/s11336-021-09762-5
- Kang, I., & Jeon, M. (2025). Multidimensional latent space item response models: A note on the relativity of conditional dependence. *Psychometrika, 90*(2), 799–826. https://doi.org/10.1017/psy.2025.5

The cited dependence families do not establish every novel base-family × generalized-mixed × dependence × temporal coupling. Novel compositions stay `research_candidate` until the exact combined model, identification, estimator and recovery are demonstrated.
