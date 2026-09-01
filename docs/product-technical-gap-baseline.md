# Product and Technical Gap Baseline

**Status:** Active delivery recovery  
**Product:** Temporal Event Psychometrics Platform (TEPP)  
**Snapshot:** 2026-09-01T16:43:00Z  
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`  
**Workspace version:** `0.2.0`  
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)  
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md)

## Delivery truth

A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Passing or queued Checks on an open PR never promote that PR to `implemented-main`.

| Signal | Current evidence | Delivery implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until protected main changes. |
| Open pull requests | **134** | WIP remains release-blocking, though verified consolidation reduced the observed peak of 149. |
| Draft pull requests | **133** | Non-landable work is now explicitly parked while it is consolidated, repaired or supplied with missing evidence. |
| Non-draft pull requests | **1** | #310 is the sole Ready PR and is not independently mergeable under the current DDD disposition. |
| Open issues | **14** | Includes #437 for repository-wide ADR identity normalization. |
| GitHub releases | **0** | No open head is a released TEPP contract. |
| Effective organization ruleset | `18156473` | One qualifying approval, resolved conversations, exact-head required workflows and an allowed merge method are required. |

Classic branch-protection status is not the sole policy source; organization rulesets are the effective merge authority where applicable. Any increase in open PR count while #435 remains open is a WIP regression unless the new PR is a demonstrably independent root-cause repair that cannot safely belong to an existing landing vehicle.

## Priority exact-head inventory

This is a priority subset, not a row-for-row copy of the 134-PR queue. #435 omits its own SHA because embedding it in the same branch would make the value stale on commit.

| PR | Exact current head | Draft | Base | Ownership / disposition |
| ---: | --- | :---: | --- | --- |
| #466 | `0c5efc3c9075115d0670b8438342c72069043dcd` | true | export-retrieval ancestor | Analysis Run / naruon export idempotency GET+CLI landing vehicle. |
| #464 | `1b3a477242336634be2c7867b29d39979e9a6dca` | true | temporal-context retrieval ancestor | Analysis Run / LineageWeave temporal-context stored-request GET+CLI landing vehicle. |
| #462 | `c1b7d627167dd7636d2975cc41cec050a5e477ba` | true | main | Bounded source-name compatibility repair; v1 serialized key remains `id`. |
| #441 | `23959d1199f84554f4a5090cea2b9e9d70b55dd8` | true | main | Longitudinal Modeling two-marginal lagged-correlation repair; source findings resolved, dependency-review support blocked externally. |
| #420 | `0dc8b48f66b367a90847fcfebd2c6453ff275a1d` | true | main | Project-history query CLI; `fold_into_landing_vehicle`. |
| #417 | `1e468f62ec47f3476a7b4d18ed2980451dc425cf` | true | main | Analysis Run / naruon export retrieval GET+CLI landing vehicle. |
| #310 | `cc7fad4d0bdffe7b524636737581227663f930f4` | false | main | Sole Ready PR; temporal `discreteDRIFTstd` evidence must migrate from `psychometric_core` into Longitudinal Modeling before landing. |

## Domain ownership

Cargo crates, HTTP routes, CLI verbs, refusal rules and clocks are implementation units, not bounded contexts.

| Subdomain | Bounded context | Aggregate authority | Implementation nucleus |
| --- | --- | --- | --- |
| Core | Evidence & Semantic Measurement | `EvidenceCorpus`, `SemanticUnitSet`, `ConceptDictionaryRevision` | `evidence_core`, `semantic_core` |
| Core | Temporal Semantics | `TemporalEvidenceWindow`, `KnowledgeCutoffPolicy` | temporal primitives and cutoff policy |
| Core | Event Ontology & Temporal Graph | `EventEpisode`, `TemporalRelationSet` | `event_core`, `relation_graph` |
| Core | Measurement | `MeasurementSpecification`, `MeasurementRun` | measurement modules + released fast-mlsirm ACL |
| Core | Longitudinal Modeling | `TemporalModelSpecification`, `TemporalModelRun` | `longitudinal_core` + temporal/event composition |
| Core | Validation | `ValidationStudy`, `ValidationEvidence` | `validation_core`, `tepp_simulation` |
| Supporting | Projection / Analysis Run | `AnalysisRun`, published read models | application services; HTTP/CLI are adapters |
| Supporting | Interpretation | evidence-grounded interpretation workflow | contextual-orchestrator ACL only |
| Supporting | Persistence & Recovery | repositories and durable receipts | `persistence_postgres`, object-store adapters |
| Generic | Compute backend | execution receipt | CPU/GPU/MLX adapters; execution is not scientific authority |

**Canonical owners:** fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and LSIRM/MLSIRM/DLSJM kernels. TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment and temporal recovery. contextual-orchestrator owns every LLM/provider call and routing decision. Context Graph contracts are a contract-only Shared Kernel; EA Core owns authoritative architecture decisions. No cross-service SQL.

### Clock and relation invariants

`event_time`, `assertion_time`, `document_time`, `system_time`, `available_time` and `knowledge_cutoff` remain distinct in code, schemas and tests. A retrospective source may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective, citation, revision and provenance relations.

## Temporal/dependence model policy

TEPP composes time over the full released upstream candidate identity, not hard-coded model names. Every candidate records exact response/generalized-mixed/dependence formulation, `supported | research_candidate | unsupported`, state/generative equations, clock roles, identification/alignment, time-varying covariates/random effects/memberships, estimator owner, required data support, primary citations and recovery status.

Rasch remains distinct from generic 1PL. Formulation-qualified 2PLM–5PLM, confirmatory/exploratory MIRT and ideal-point/GGUM identities are preserved. Cross-classification and multiple membership remain distinct; membership weights are explicit, time-valid and observed-normalized or model-estimated according to the formulation. LSIRM/MLSIRM temporal candidates preserve base parameters plus person/item geometry. DLSJM keeps distinct item- and person-dependence spaces. Dynamic geometry stays `research_candidate` until state equations, temporal identification/alignment and true-parameter recovery exist.

Auto-expansion is not auto-activation. A numerical standardizer or adapter is not a DSEM/ctsem/LSIRM/MLSIRM/DLSJM estimator without the exact generative/state equations, identification, estimator and recovery evidence.

## Scientific validation invariants

Supported temporal estimators require realistic known-truth recovery: RMSE, bias, interval coverage, convergence and uncertainty calibration, temporal ordering, leakage-safe rolling-origin evaluation, irregular gaps, delayed/retrospective reports, missing occasions, changing memberships, language/source drift and CPU/GPU parity where applicable. Monte Carlo decisions use simulation uncertainty rather than arbitrary observed-pass percentages. Scientific failure is never hidden with skip, xfail, source rewriting or coverage exclusions.

## Current repairs and blockers

**#441 — Longitudinal Modeling.** The invalid one-sided covariance/earlier-variance ratio is retired. `longitudinal_core::recover_event_time_lagged_correlation` requires lagged covariance, both marginal variances and a positive event-time interval. Exact binary64 covariance-bound checking and scale-ordered division cover extreme finite inputs. All visible review threads are resolved; Semgrep is GREEN. The central Security Scan fails before dependency review because the exact dependency-graph compare endpoint returns HTTP 403 for this public repository even with `contents: read`; OSV and Trivy are separately GREEN. Treat this as `blocked_external` repository dependency-graph/security support, not as a TEPP vulnerability. Never weaken the fail-closed central workflow.

**#310 — wrong-owner temporal mapping.** Its scalar Driver/Oud/Voelkle p.16 `discreteDRIFTstd` mapping is event-interval composition but currently lives in `psychometric_core::event_time`. Preserve its scientific distinction from unstandardised `discreteDRIFT`, trait-plus-state association and DSEM/ctsem estimation, but move the temporal transform/refusals/tests into Longitudinal Modeling before landing. Reusable static psychometric/dependence arithmetic remains fast-mlsirm-owned.

**Analysis Run adapter/profile proliferation.** Export, interpretation, project-history and temporal-context HTTP/CLI mechanics are adapters inside one supporting context. Strict predecessor/child stacks may be folded only after exact ancestry is proven; diverged siblings require an actual source/test fold. One refusal/profile does not create architecture authority. Preserve unique routes, refusals, tests and review evidence before predecessor closure.

**#462 — bounded naming repair.** Rust source uses `node_id` while the v1 serialized key remains `id`; this is not a JSON-LD `@id` semantic change or a new bounded context.

**#437 — ADR identity.** The #435 branch rejects duplicate index IDs, duplicate index targets and duplicate numbered ADR files. The normalized index preserves pre-normalization collision lineage under `docs/adr/archive/`. Closure waits for protected-main integration.

## Dependency and Context Fabric status

TEPP never copies fast-mlsirm numerical kernels or contextual-orchestrator provider logic. An upstream open PR head is not a released dependency. A checksum-pinned contextual-orchestrator source is advanced only after the exact replacement archive digest is reproducibly acquired and reviewed.

`context-graph-contracts` and `enterprise-architecture-core` remain read-only to this writer while their dedicated owner loop is active. Open heads are candidate evidence, not released contracts. TEPP may maintain fail-closed conformance fixtures behind candidate/test boundaries, but deployable integration and authoritative EA projection require a released/versioned Context Graph artifact and passing compatibility evidence.

## Gap register

| ID | Gap | Maturity | Closure evidence |
| --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 134 open PRs | `release-blocking` | coherent landing vehicles, unique evidence preservation, protected-main reduction |
| GAP-002 | multilingual span-grounded semantic/concept admission | `partial` | immutable offsets/layout, language profiles, concept dictionary, invariance/calibration, hostile-input tests |
| GAP-003 | shared-latent temporal topic estimator | `partial` | Rust CPU f64 likelihood/uncertainty, relation/time/membership effects, true recovery, fitted candidate-K |
| GAP-004 | durable end-to-end Analysis Run | `partial` | idempotent lifecycle, persistence/recovery, estimator-bound artifacts, validation/promotion separation, Compose E2E |
| GAP-005 | temporal psychometric composition/duplication | `partial` | released fast-mlsirm contract, TEPP ACL, temporal recovery, wrong-owner static kernels removed after parity |
| GAP-006 | event intelligence | `partial` | calibrated detection/tracking/schema/interval recovery and durable artifacts |
| GAP-007 | accelerator/memory evidence | `accepted-target` | real hardware, CPU-f64 parity, bounded OOM/fallback evidence |
| GAP-008 | network/cluster buyer workflow | `partial` | known-truth recovery, uncertainty/stability, repeated consensus, exact-value export |
| GAP-009 | production interpreter/verifier | `partial` | contextual-orchestrator execution, evidence citations, independent verification, abstention/fallback |
| GAP-010 | accessible buyer UI | `accepted-target` | Figma/Storybook, keyboard/touch/error/empty states, exact-value provenance |
| GAP-011 | operable multi-tenant release | `accepted-target` | OIDC/RLS/purpose controls, durable queue/storage, OTel/SLO, restore/load/migration, signed SBOM/provenance |
| GAP-012 | paths obscure domain ownership | `active-refactor` | staged moves, ACLs, no cycles/cross-context persistence/shared-kernel creep |
| GAP-013 | ADR identity collisions | `release-integrity` | unique repository-wide identity, deterministic duplicate detection, supersession lineage |
| GAP-014 | dependency-review evidence unavailable | `blocked_external` | repair/enable dependency-graph support, exact-head Security Scan GREEN, no fail-open bypass |

## Delivery and release order

Queue/ADR/domain authority precedes semantic admission, the Rust CPU f64 shared-latent temporal estimator, durable Analysis Run, released fast-mlsirm temporal composition, event intelligence, accelerator parity, buyer workflows and finally tenancy/durability/observability/release support. A bounded security/dependency repair may land earlier when it directly unblocks a selected vehicle.

Every open PR is classified as `landing_vehicle`, `stacked_dependency`, `fold_into_landing_vehicle`, `superseded`, `duplicate`, `research_lineage_only`, or `blocked_external`. Exact heads must be compared before closure. Strict ancestry permits retarget-and-close folding; diverged siblings require a real code/test fold first.

TEPP has no GitHub release at this snapshot. Release requires a coherent buyer/scientific vertical on protected main, exact-head scientific and security evidence, reproducible package/build provenance and SBOM, upgrade/rollback evidence, and released integration contracts where deployment depends on them.