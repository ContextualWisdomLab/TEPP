# Product and Technical Gap Baseline

**Status:** Active delivery recovery  
**Product:** Temporal Event Psychometrics Platform (TEPP)  
**Snapshot:** 2026-09-01T12:31:35Z  
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`  
**Workspace version:** `0.2.0`  
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)  
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md)

## Purpose

This document is the operator-facing authority for current product and technical gaps. Historical queue snapshots remain in Git history. A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped.

Passing or queued Checks on an open PR never promote that PR to implemented-main.

## Live snapshot

| Signal | Current evidence | Delivery implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until protected main changes. |
| Open pull requests | **135** | Release-blocking WIP remains high despite falling from the observed peak of 149. |
| Draft pull requests | **89** | Draft status is workflow metadata, not scientific or domain classification. |
| Non-draft pull requests | **46** | Ready metadata is not merge readiness without exact-head evidence. |
| Open issues | **14** | Includes #437 for repository-wide ADR identity normalization. |
| GitHub releases | **0** | No open PR head or local branch is a released TEPP product contract. |
| Effective organization ruleset | `18156473` | Current ruleset, exact-head workflows, resolved conversations and qualifying review govern merge. |

The classic branch-protection payload is not the sole policy source. Organization rulesets are the effective merge authority where applicable.

The queue fell from 149 to 134, then returned to 135 when #454 appeared. New one-operation Analysis Run slices therefore still count as WIP regression while queue recovery is active even though overall predecessor closure is reducing the queue.

## Current priority open pull-request evidence

This table is an exact-head **priority subset**, not a row-for-row copy of the 135-PR queue. PR #435 itself is omitted because writing its own exact head into a commit on that branch would immediately make the embedded head stale.

| PR | Exact current head | Draft | Base | Ownership / disposition |
| ---: | --- | :---: | --- | --- |
| #454 | `c911fbcd6e13046358cc7f3692775ed4b008dadd` | false | #453 head | Analysis Run/contextual-orchestrator stored-request CLI; `fold_into_landing_vehicle` |
| #453 | `baee8854e99dc416b2b907a22101c53a6eca9eca` | false | #438 head | Analysis Run/contextual-orchestrator stored-request GET; `fold_into_landing_vehicle` |
| #452 | `cb97aad9f87283df4d94abe8c6df61a0a476c893` | false | #451 head | Analysis Run/LineageWeave temporal-context retrieval CLI; `fold_into_landing_vehicle` candidate |
| #451 | `7b117e8f69cd74e28eceaf1748c6a9210f5dffbc` | false | main | Analysis Run/LineageWeave temporal-context GET-by-id; `fold_into_landing_vehicle` candidate |
| #443 | `504793d88c6b754f5181f48dc7abde073ff9146a` | false | #411 head | Analysis Run/export collection adapter; `fold_into_landing_vehicle` |
| #441 | `6f483224b3a03e8237c6f4f098a8b0e85e0a91f5` | false | main | Longitudinal Modeling lagged-correlation root-cause repair; auto-merge enabled, exact-head hosted gates still pending |

## Strategic Domain-Driven Design baseline

Cargo crates and HTTP routes are implementation units, not bounded contexts.

| Subdomain | Bounded context | Aggregate authority | Primary implementation nucleus |
| --- | --- | --- | --- |
| Core | Evidence & Semantic Measurement | `EvidenceCorpus`, `SemanticUnitSet`, `ConceptDictionaryRevision` | `evidence_core`, `semantic_core` |
| Core | Temporal Semantics | `TemporalEvidenceWindow`, `KnowledgeCutoffPolicy` | temporal primitives and cutoff policy |
| Core | Event Ontology & Temporal Graph | `EventEpisode`, `TemporalRelationSet` | `event_core`, `relation_graph` |
| Core | Measurement | `MeasurementSpecification`, `MeasurementRun` | topic/measurement modules plus released fast-mlsirm ACL |
| Core | Longitudinal Modeling | `TemporalModelSpecification`, `TemporalModelRun` | `longitudinal_core` and temporal/event composition |
| Core | Validation | `ValidationStudy`, `ValidationEvidence` | `validation_core`, `tepp_simulation` |
| Supporting | Projection / Analysis Run | `AnalysisRun`, published read models | application services with HTTP/CLI as adapters |
| Supporting | Interpretation | evidence-grounded interpretation workflow | contextual-orchestrator ACL only |
| Supporting | Persistence & Recovery | repository implementations and durable receipts | `persistence_postgres` and object-store adapters |
| Generic | Compute backend | execution receipt | CPU/GPU/MLX adapters; execution is not scientific authority |

### Canonical owner boundaries

- **fast-mlsirm** owns reusable static/generalized-mixed/dependence-aware psychometric model specification and numerical kernels, including reusable LSIRM/MLSIRM/DLSJM computation.
- **TEPP** owns temporal/event composition: event/valid/assertion/document/system/available time semantics, leakage-safe knowledge cutoff, irregular intervals, time-varying covariates/random effects/memberships, longitudinal invariance/drift, event ontology/graph, temporal alignment, state evolution and temporal recovery.
- **contextual-orchestrator** owns every model-provider call, routing/fallback, credential, verifier/adjudicator execution and LLM call provenance.
- **LineageWeave or the consuming product** owns source/item-generation lineage; lineage is evidence rather than numerical authority.
- **context-graph-contracts** is a contract-only Shared Kernel and **enterprise-architecture-core** is the authoritative EA Decision Plane. TEPP consumes only released/versioned contracts through ACLs and never writes cross-service SQL.

### Six-clock invariant

The following meanings remain distinct in code, schema and tests:

- `event_time`: when a substantive event occurs;
- `assertion_time`: when a statement claims or records an event/state;
- `document_time`: when the source document is created/revised/published;
- `system_time`: when TEPP records the fact;
- `available_time`: when evidence became usable by an analysis;
- `knowledge_cutoff`: the latest available-time admitted to a run.

A valid retrospective document may point to an earlier event, but it cannot enter an earlier knowledge cutoff. Forward state/transition edges are distinct from retrospective, citation, revision and provenance relations.

## Temporal dependence composition

TEPP composes time over the full released upstream candidate identity rather than hard-coded family names. A compatible base family added to fast-mlsirm should inherit temporal-candidate compilation without a TEPP family-specific wrapper.

Each temporal candidate records:

- released upstream contract version and digest;
- exact base formulation and parameter meaning;
- generalized-mixed and dependence structures;
- status: `supported`, `research_candidate`, or `unsupported`;
- temporal state/generative equation;
- event/occasion clock roles;
- identification/alignment constraints;
- time-varying covariates, random effects and membership semantics;
- estimator owner;
- required data support;
- primary citations;
- recovery contract and current recovery status.

Auto-expansion is not auto-activation. Unknown or novel couplings remain `research_candidate`. Incoherent combinations are `unsupported`. No dependence-aware temporal request is silently simplified to a static/local-independent model.

### Base-family identity

Rasch remains distinct from generic 1PL. Formulation-qualified 2PLM–5PLM retain their parameter meanings. Confirmatory/exploratory MIRT and ideal-point/GGUM response processes are distinct axes from hierarchy, dependence and time. Testlet, rater/facet, nested, crossed, cross-classified and multiple-membership structure remain explicit.

Known hierarchy/testlet/rater/method/item-family effects are modeled before residual latent-space dependence. Cross-classification and multiple membership are distinct. Multiple-membership weights are explicit, auditable, time-valid and either observed/normalized or model-estimated according to the declared formulation; equal weights are never invented as a fallback.

### LSIRM / MLSIRM / DLSJM

LSIRM/MLSIRM temporal candidates preserve the complete base-family parameterization plus person/item positions, distances/interactions and the declared generalized-mixed structure. Dynamic latent-space evolution remains `research_candidate` until state equations, longitudinal identification/alignment and recovery exist.

DLSJM temporal candidates retain distinct item-dependence and person-dependence spaces. Jin and Jeon (2019) is the baseline authority for DLSJM itself; novel temporal couplings remain extensions. Translation/rotation/reflection and cluster-label alignment are required before maps or clusters are compared across occasions.

## Scientific validation invariants

Every supported temporal estimator requires realistic recovery evidence. Applicable contracts include true-state/true-parameter RMSE, bias, interval coverage, convergence, uncertainty calibration, temporal ordering, leakage-safe rolling-origin evaluation, irregular gaps, delayed and retrospective reports, missing occasions, changing memberships, language/source drift and CPU/GPU parity where an accelerator backend exists.

Monte Carlo decisions use uncertainty of the Monte Carlo study rather than arbitrary observed-pass percentages. Scientific failure is never hidden with skip, xfail or source rewriting.

A generic arithmetic standardizer is not itself a DSEM/ctsem estimator and must not be promoted as one without state equations, process-noise/marginal recovery, identification and validation evidence.

## Current classifications

**#441 — Longitudinal Modeling landing vehicle.** The invalid predecessor that divided lagged covariance by only the earlier marginal variance has been retired. `longitudinal_core::recover_event_time_lagged_correlation` now requires lagged covariance plus both occasion-specific marginal variances and a positive event-time lag. Exact binary64 covariance-bound checking avoids rounded-product acceptance/rejection errors and stable division order avoids avoidable overflow/underflow. All currently visible review threads are resolved. Auto-merge is enabled. Exact-head Documentation Quality, Rust Foundation CI, Security Scan and SAST Semgrep are still queued and no qualifying independent non-author APPROVE has been observed, so queued checks never constitute implemented-main evidence.

**#453/#454 — Analysis Run interpretation adapter fold candidates.** Their PR descriptions now explicitly classify them `fold_into_landing_vehicle`. Preserve stored-request path/CLI parsing, hostile-input, origin/credential/consumer refusal, metric-free result and `scientific_authority=false` tests. ADR 0085/0086 are implementation evidence pending #437 normalization, not branch-local architecture authority.

**#451/#452 — Analysis Run temporal-context adapter fold candidates.** Preserve LineageWeave-only identity/refusal/metric-free tests, then fold with the coherent temporal-context adapter vehicle instead of creating a bounded context per GET/CLI operation.

**#443/#444 and related export slices — Analysis Run export fold candidates.** Preserve pagination/auth/refusal/metric-free tests and fold into one export application-adapter landing vehicle.

**#356 — closed, not merged.** Its self-referential RMSE-SE acceptance gate, caller-declared recovery provenance and Validation/Claim-Promotion conflation remain prohibited. Useful cutoff/run-binding/metric evidence belongs in the coherent Validation/Analysis Run vehicle.

**#437 — ADR identity repair.** Repository-wide ADR IDs are immutable and unique. Duplicate numeric files/index rows must fail deterministic fitness tests. Implementation maturity does not create branch-local architecture authority.

## Operator-gap register

| ID | Gap | Maturity | Authority | Closure evidence |
| --- | --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 135 open PRs | `release-blocking` | #175 / #435 | classified queue, coherent bounded-context landing vehicles, unique evidence preserved, safe reduction |
| GAP-002 | multilingual span-grounded semantic/concept admission incomplete | `partial` | Evidence & Semantic Measurement | immutable offsets/layout, language profiles, concept dictionary, unknown review, invariance/calibration, hostile-input tests |
| GAP-003 | shared-latent temporal topic estimator incomplete | `partial` | #167 | Rust CPU `f64` likelihood/estimands/uncertainty, relation/time/membership effects, multi-seed recovery, real candidate-K fits |
| GAP-004 | durable end-to-end Analysis Run incomplete | `partial` | #166 | idempotent lifecycle, persistence/recovery, estimator-bound artifacts, validation evidence, claim-promotion separation, Compose E2E |
| GAP-005 | temporal psychometric composition fragmented/partly duplicated | `partial` | Longitudinal Modeling + fast-mlsirm ACL | released upstream contract, TEPP temporal ACL, invariance/alignment, irregular time, temporal recovery, duplicated static kernels removed after parity |
| GAP-006 | TDT/CHRONOS event workflow incomplete | `partial` | Event Ontology & Temporal Graph | calibrated detection/tracking/schema/interval consistency with recovery and durable artifacts |
| GAP-007 | real accelerator/memory evidence incomplete | `accepted-target` | Compute backend | real hardware, CPU-f64 parity, bounded memory/OOM/fallback evidence |
| GAP-008 | network/cluster buyer workflow incomplete | `partial` | Projection | known-truth recovery, uncertainty/stability, repeated consensus, exact-value exports |
| GAP-009 | production interpreter/verifier incomplete | `partial` | Interpretation | contextual-orchestrator execution, evidence citations, independent verifier, ablations and abstention/fallback |
| GAP-010 | coordinated accessible buyer UI incomplete | `accepted-target` | Projection/UI | Figma/Storybook/design tokens, keyboard/touch/error/empty states, exact-value and print/export provenance |
| GAP-011 | operable multi-tenant supported release incomplete | `accepted-target` | Operations | OIDC/RLS/purpose controls, durable queue/storage, OTel/SLO, restore/load/migration, signed release/SBOM/provenance |
| GAP-012 | directory/crate paths obscure domain ownership | `active-refactor` | #435 / landing vehicles | staged folds, compatibility ACLs, no cycles/cross-context persistence/shared-kernel creep |
| GAP-013 | ADR identity duplicated/branch-local in parts of queue | `release-integrity` | #437 | unique repository-wide identity, duplicate detection, normalized index and supersession lineage |

## Delivery order

1. Restore queue, ADR and bounded-context authority.
2. Consolidate Evidence & Semantic Measurement.
3. Complete the Rust CPU `f64` shared-latent temporal topic estimator.
4. Complete durable end-to-end Analysis Run and evidence/promotion separation.
5. Compose temporal psychometrics through released fast-mlsirm contracts and complete Event Intelligence.
6. Add real accelerator parity only after CPU scientific authority exists.
7. Complete network/cluster, interpretation and buyer visual workflows.
8. Productionize tenancy, durability, observability, recovery, release and support.

A bounded dependency/security repair may land earlier when it directly unblocks a selected landing vehicle. It does not create a new product priority.

## Queue consolidation rules

Every open PR receives one of:

- `landing_vehicle`
- `stacked_dependency`
- `fold_into_landing_vehicle`
- `superseded`
- `duplicate`
- `research_lineage_only`
- `blocked_external`

Before classifying a PR as duplicate/superseded/folded, compare exact heads and preserve unique production behavior, compatibility, tests, review findings, primary research, doctoring and provenance. One-rule crates, one-clock crates and one-operation API/CLI PRs are not independent product boundaries by default.

## Release truth

TEPP has no GitHub release at this snapshot. A release is permitted only after a coherent buyer/scientific vertical reaches protected main with required exact-head recovery and security evidence, reproducible package/build provenance, SBOM, upgrade/rollback evidence, and released integration contracts where cross-product deployment depends on them.
