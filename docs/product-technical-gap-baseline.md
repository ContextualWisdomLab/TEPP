# Product and Technical Gap Baseline

**Status:** Active delivery recovery  
**Product:** Temporal Event Psychometrics Platform (TEPP)  
**Snapshot:** 2026-09-01T20:30:01Z  
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`  
**Workspace version:** `0.2.0`  
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)  
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md)

## Delivery truth

A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Passing or queued Checks never promote an open PR to `implemented-main`.

| Signal | Current evidence | Delivery implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until protected main changes. |
| Open pull requests | **134** | WIP remains release-blocking, but the queue has been reduced from the observed peak of 149 and from the 136-PR regression at the start of this repair slice. |
| Draft pull requests | **133** | Non-landable work is explicitly parked while consolidated, repaired, or supplied with missing evidence. |
| Non-draft pull requests | **1** | #310 is the only current non-Draft landing vehicle. |
| Open issues | **15** | Includes #437 for repository-wide ADR identity normalization and #472 for cutoff-safe inferred-status Analysis Run projection. |
| GitHub releases | **0** | No open head is a released TEPP contract. |
| Effective organization ruleset | `18156473` | Current-head required workflows, resolved conversations, qualifying review, and an allowed merge method remain landing authority. |

Classic branch protection is not the sole policy source; organization rulesets are effective merge authority where applicable. Any increase in open PR count while #435 remains open is a WIP regression unless the new PR is a demonstrably independent root-cause repair that cannot safely belong to an existing landing vehicle.

The most recent regression reached 136 open PRs when #476 (`DIFFUSIONstd`) and #477 (`discreteDIFFUSIONstd`) were opened as one-map Drafts under the wrong technical owner `psychometric_core`. Their scientific evidence was not discarded. Exact source and tests were verified as folded into #310 under the Longitudinal Modeling bounded context, including typed event time, positive stationarity, scale invariance, subnormal/overflow regressions, interval ordering, refusal contracts, and explicit `research_candidate` status. #476 and #477 were then closed as superseded lineage. The live queue consequently fell to 134.

## Current priority open pull-request evidence

This is a priority subset, not a row-for-row copy of the 134-PR queue. #435 intentionally omits its own SHA because embedding the branch head in a file changed by that same branch would make the evidence self-stale.

| PR | Exact current head | Draft | Base | Ownership / disposition |
| ---: | --- | :---: | --- | --- |
| #469 | `72a7755bcc91b1107560c980ce817eca153126e4` | true | interpretation-run retrieval ancestry | Analysis Run / contextual-orchestrator stored-request + server-id lookup GET/CLI landing vehicle. |
| #466 | `71f34b890bbd096eee152947c5e22d9778d323e8` | true | export-retrieval ancestry | Analysis Run / naruon export idempotency lookup + quarantine-parity landing vehicle. |
| #464 | `1b3a477242336634be2c7867b29d39979e9a6dca` | true | temporal-context retrieval ancestry | Analysis Run / LineageWeave temporal-context stored-request GET+CLI landing vehicle; re-read before mutation. |
| #462 | `c1b7d627167dd7636d2975cc41cec050a5e477ba` | true | main | Bounded source-name compatibility repair; v1 serialized key remains `id`. |
| #420 | `0dc8b48f66b367a90847fcfebd2c6453ff275a1d` | true | main | Project-history query CLI; `fold_into_landing_vehicle` until its unique source/tests are composed into the surviving Analysis Run vehicle. |
| #417 | `1e468f62ec47f3476a7b4d18ed2980451dc425cf` | true | main | Analysis Run / naruon export retrieval GET+CLI landing vehicle; re-read before mutation. |
| #310 | `4e7435f6dd232ae8e1e019f1393e7285e32c6527` | false | main | Longitudinal Modeling landing vehicle; contains closed #441 lineage and verified #476/#477 diffusion folds. |

Exact current head evidence is authoritative only for the named PR and becomes stale after any source mutation.

## Domain ownership

Cargo crates, HTTP routes, CLI verbs, refusal rules, clocks, and individual statistical maps are implementation units, not bounded contexts.

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

**Canonical owners:** fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and LSIRM/MLSIRM/DLSJM kernels. TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment, and temporal recovery. contextual-orchestrator owns every LLM/provider call and routing decision. Context Graph contracts are a contract-only Shared Kernel; EA Core owns authoritative architecture decisions. No cross-service SQL.

`event_time`, `assertion_time`, `document_time`, `system_time`, `available_time`, and `knowledge_cutoff` remain distinct in code, schemas, and tests. A retrospective source may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective, citation, revision, and provenance relations.

## Temporal/dependence model policy

TEPP composes time over the full released upstream candidate identity, not hard-coded family names. Every candidate records exact response/generalized-mixed/dependence formulation, `supported | research_candidate | unsupported`, state/generative equations, clock roles, identification/alignment, time-varying covariates/random effects/memberships, estimator owner, required data support, primary citations, and recovery status.

Rasch remains distinct from generic 1PL. Formulation-qualified 2PLM–5PLM, confirmatory/exploratory MIRT, and ideal-point/GGUM identities are preserved. Cross-classification and multiple membership remain distinct; membership weights are explicit, time-valid, and observed-normalized or model-estimated according to the formulation. LSIRM/MLSIRM temporal candidates preserve base parameters plus person/item geometry. DLSJM keeps distinct item- and person-dependence spaces. Dynamic geometry stays `research_candidate` until state equations, temporal identification/alignment, and true-parameter recovery exist.

Auto-expansion is not auto-activation. A numerical standardizer or adapter is not a DSEM/ctsem/LSIRM/MLSIRM/DLSJM estimator without exact generative/state equations, identification, estimator, and recovery evidence.

## Scientific validation invariants

Supported temporal estimators require realistic known-truth recovery: RMSE, bias, interval coverage, convergence and uncertainty calibration, temporal ordering, leakage-safe rolling-origin evaluation, irregular gaps, delayed/retrospective reports, missing occasions, changing memberships, language/source drift, and CPU/GPU parity where applicable. Monte Carlo decisions use simulation uncertainty rather than arbitrary pass percentages. Scientific failure is never hidden with skip, xfail, source rewriting, or coverage exclusions.

## Current repairs and blockers

**#310 — Longitudinal Modeling landing vehicle.** Closed predecessor #441 is contained by #310. The invalid one-sided covariance/earlier-variance ratio remains retired; public lagged correlation requires lagged covariance and both marginal variances. `EventTimeInterval` is preserved end-to-end. The surviving branch also contains the source/test evidence from #476/#477 under `longitudinal_core`: continuous and discrete scalar diffusion-standardisation candidates, positive stationary-within admission, subnormal-cancellation repair, scale-invariance tests, signed-zero event-product refusal, interval ordering, and named-estimand refusals. Those maps remain `research_candidate` because the 2017 ctsem summary source does not emit named `DIFFUSIONstd` / `discreteDIFFUSIONstd` matrices. On exact head `4e7435f6dd232ae8e1e019f1393e7285e32c6527`, all review threads returned by the current review-thread query are resolved. Rust Foundation CI, Documentation Quality, SAST Semgrep, and Security Scan are currently queued, and no qualifying independent APPROVE is present; therefore #310 is not mergeable by policy yet.

**#476/#477 — superseded scientific micro-PRs.** Both Drafts are closed after source-level parity verification against #310. Their immutable discussions and branches remain research lineage; they must not be merged independently. This is the required queue repair pattern: fold unique evidence into the bounded-context owner first, verify parity, then retire the micro-PR.

**Dependency-review support.** Where Security Scan fails before Dependency Review because GitHub dependency-graph comparison is unavailable to the workflow token, keep the gate fail-closed. OSV, Trivy, and Scorecard are sibling evidence, not substitutes. Do not weaken TEPP source to manufacture missing control-plane evidence.

**Analysis Run adapter/profile proliferation.** Export, interpretation, project-history, and temporal-context HTTP/CLI mechanics are adapters inside one supporting context. Strict ancestry may be folded without force after exact comparison; diverged siblings require an actual source/test fold. One refusal/profile/route/CLI verb does not create architecture authority.

**#437 — ADR identity.** Repository-wide ADR IDs are immutable authority. Duplicate index IDs, duplicate targets, and duplicate numbered ADR files must fail deterministic documentation fitness tests. Operation-specific ADRs on adapter/model micro-branches are implementation lineage pending normalization, not branch-local architecture authority.

## Dependency and Context Fabric status

TEPP never copies fast-mlsirm numerical kernels or contextual-orchestrator provider logic. An upstream open PR head is not a released dependency. A checksum-pinned contextual-orchestrator source advances only after the exact replacement archive digest is reproducibly acquired and reviewed.

`context-graph-contracts` and `enterprise-architecture-core` remain read-only to this writer while their dedicated Context Fabric owner loop is active. Open heads are candidate evidence, not released contracts. TEPP may maintain fail-closed conformance fixtures behind candidate/test boundaries, but deployable integration and authoritative EA projection require a released/versioned Context Graph artifact plus passing compatibility evidence.

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
| GAP-014 | dependency-review evidence unavailable | `blocked_external` | authorized dependency-graph availability, pinned Dependency Review execution, exact-head Security Scan GREEN, no fail-open bypass |

## Delivery and release order

Queue/ADR/domain authority precedes semantic admission, the Rust CPU f64 shared-latent temporal estimator, durable Analysis Run, released fast-mlsirm temporal composition, event intelligence, accelerator parity, buyer workflows, and finally tenancy/durability/observability/release support. A bounded security/dependency repair may land earlier when it directly unblocks a selected vehicle.

Every open PR is classified as `landing_vehicle`, `stacked_dependency`, `fold_into_landing_vehicle`, `superseded`, `duplicate`, `research_lineage_only`, or `blocked_external`. Exact heads must be compared before closure. Strict ancestry permits safe consolidation; diverged siblings require a real code/test fold first.

TEPP has no GitHub release at this snapshot. Release requires a coherent buyer/scientific vertical on protected main, exact-head scientific and security evidence, reproducible package/build provenance and SBOM, upgrade/rollback evidence, and released integration contracts where deployment depends on them.
