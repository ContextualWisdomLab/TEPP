# Product and Technical Gap Baseline

**Status:** Active delivery recovery  
**Product:** Temporal Event Psychometrics Platform (TEPP)  
**Snapshot:** 2026-09-01T16:36:55Z  
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`  
**Workspace version:** `0.2.0`  
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)  
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md)

## Purpose

This document is the operator-facing authority for current product and technical gaps. Historical snapshots remain in Git history. A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Passing or queued Checks on an open PR never promote that PR to `implemented-main`.

## Live snapshot

| Signal | Current evidence | Delivery implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until protected main changes. |
| Open pull requests | **134** | WIP remains release-blocking, but queue consolidation has reduced the observed peak of 149. |
| Draft pull requests | **121** | Most remaining work is explicitly non-landable pending consolidation, repair, dependency or scientific evidence. |
| Non-draft pull requests | **13** | Ready metadata is not merge readiness without exact-head evidence and qualifying review. |
| Open issues | **14** | Includes #437 for repository-wide ADR identity normalization. |
| GitHub releases | **0** | No open PR head or local branch is a released TEPP product contract. |
| Effective organization ruleset | `18156473` | Requires one qualifying approval, resolved review threads, current required workflows and an allowed merge method. |

The classic branch-protection payload is not the sole policy source. Organization rulesets are the effective merge authority where applicable.

The queue rose to 149 before the recovery vehicle started folding strict linear stacks. Since then, GET/CLI predecessor pairs in export, interpretation, project-history and temporal-context adapters have been collapsed into surviving landing vehicles while preserving predecessor discussion, RED/GREEN lineage and review evidence. New one-operation or one-refusal PR creation while #435 remains open is a delivery regression unless an existing bounded-context vehicle cannot safely own the repair.

## Current priority open pull-request evidence

This table is an Exact current head **priority subset**, not a row-for-row copy of the 134-PR queue. PR #435 itself is omitted because writing its own exact head into a commit on that branch would immediately make the embedded head stale.

| PR | Exact current head | Draft | Base | Ownership / disposition |
| ---: | --- | :---: | --- | --- |
| #466 | `0c5efc3c9075115d0670b8438342c72069043dcd` | true | export-retrieval GET ancestor | Analysis Run / naruon export idempotency GET+CLI landing vehicle; fold compatible export mechanics here or a coherent successor. |
| #464 | `1b3a477242336634be2c7867b29d39979e9a6dca` | true | temporal-context retrieval GET ancestor | Analysis Run / LineageWeave temporal-context stored-request GET+CLI landing vehicle. |
| #462 | `c1b7d627167dd7636d2975cc41cec050a5e477ba` | true | main | Bounded `JsonLdExport.node_id` source-name compatibility repair; serialized v1 key remains `id`. |
| #441 | `23959d1199f84554f4a5090cea2b9e9d70b55dd8` | true | main | Longitudinal Modeling true two-marginal lagged-correlation repair; source findings resolved, security support gate remains external/configuration-blocked. |
| #417 | `1e468f62ec47f3476a7b4d18ed2980451dc425cf` | true | main | Analysis Run / naruon export retrieval GET+CLI landing vehicle. |
| #420 | `0dc8b48f66b367a90847fcfebd2c6453ff275a1d` | false | main | Project-history query CLI; must fold into the coherent Analysis Run / LineageWeave project-history vehicle rather than land as a one-operation boundary. |

## Strategic Domain-Driven Design baseline

Cargo crates, HTTP routes, CLI verbs, refusal rules and clocks are implementation units, not bounded contexts.

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
| Supporting | Persistence & Recovery | repositories and durable receipts | `persistence_postgres` and object-store adapters |
| Generic | Compute backend | execution receipt | CPU/GPU/MLX adapters; execution is not scientific authority |

### Canonical owner boundaries

- **fast-mlsirm** owns reusable static/generalized-mixed/dependence-aware psychometric model specification and numerical kernels, including reusable LSIRM/MLSIRM/DLSJM computation.
- **TEPP** owns temporal/event composition: event/valid/assertion/document/system/available time semantics, leakage-safe knowledge cutoff, irregular intervals, time-varying covariates/random effects/memberships, longitudinal invariance/drift, event ontology/graph, temporal alignment, state evolution and temporal recovery.
- **contextual-orchestrator** owns every model-provider call, routing/fallback, credential, verifier/adjudicator execution and LLM call provenance.
- **LineageWeave or the consuming product** owns source/item-generation lineage; lineage is evidence rather than numerical authority.
- **context-graph-contracts** is a contract-only Shared Kernel and **enterprise-architecture-core** is the authoritative EA Decision Plane. TEPP consumes only released/versioned contracts through ACLs and never writes cross-service SQL.

### Six-clock invariant

`event_time`, `assertion_time`, `document_time`, `system_time`, `available_time` and `knowledge_cutoff` remain distinct in code, schema and tests. A retrospective source may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective, citation, revision and provenance relations.

## Temporal dependence composition

TEPP composes time over the full released upstream candidate identity rather than hard-coded family names. Every temporal candidate records the released upstream version/digest, exact response and generalized-mixed formulation, dependence structure, `supported | research_candidate | unsupported`, temporal state/generative equations, clock roles, identification/alignment, time-varying covariates/random effects/memberships, estimator owner, required data support, primary citations and recovery status.

Auto-expansion is not auto-activation. Rasch remains distinct from generic 1PL. Formulation-qualified 2PLM–5PLM, confirmatory/exploratory MIRT and ideal-point/GGUM identities are preserved. Cross-classification and multiple membership remain distinct; membership weights are explicit, auditable, time-valid and observed-normalized or model-estimated according to the declared formulation.

LSIRM/MLSIRM temporal candidates preserve full base-family parameters plus person/item geometry and generalized-mixed structure. DLSJM retains distinct item- and person-dependence spaces. Dynamic geometry remains `research_candidate` until state equations, longitudinal identification/alignment and true-parameter recovery exist. Raw latent maps are never compared across occasions without alignment.

## Scientific validation invariants

Every supported temporal estimator requires realistic known-truth recovery evidence: RMSE, bias, interval coverage, convergence/uncertainty calibration, temporal ordering, leakage-safe rolling-origin evaluation, irregular gaps, delayed/retrospective reports, missing occasions, changing memberships, language/source drift and CPU/GPU parity where applicable. Monte Carlo decisions use uncertainty of the simulation study rather than arbitrary pass percentages. Scientific failure is never hidden with skip, xfail or source rewriting.

A numerical standardizer or adapter is not a DSEM/ctsem/LSIRM/MLSIRM/DLSJM estimator unless the exact generative/state equations, identification, estimator and recovery evidence exist.

## Current classifications and repairs

**#441 — Longitudinal Modeling root-cause repair.** The invalid predecessor that divided lagged covariance by only the earlier marginal variance is retired. `longitudinal_core::recover_event_time_lagged_correlation` requires lagged covariance plus both marginal variances and a positive event-time interval. Exact binary64 covariance-bound checking and scale-ordered division cover extreme finite inputs without using an invalid one-sided ratio. Current review threads are resolved. Semgrep is GREEN on the current head. The central Security Scan fails before dependency review because `GET /repos/ContextualWisdomLab/TEPP/dependency-graph/compare/<base>...<head>` returns HTTP 403 even though the repository is public and the job token has `contents: read`; OSV/Trivy evidence is independently GREEN. This is `blocked_external` on repository dependency-graph/security configuration or GitHub service behavior, not a TEPP vulnerability result. Do not weaken the fail-closed workflow to make the check green.

**Analysis Run adapter families.** Export, interpretation, project-history and temporal-context HTTP/CLI mechanics are application adapters inside one supporting bounded context. Strict predecessor/child stacks are folded only after exact ancestry is proven; siblings require real source/test consolidation before closure. #417/#466, #428/#420 and related surviving vehicles therefore remain separate until unique routes/refusals/tests are actually composed into coherent vehicles.

**Validation / Analysis Run profile proliferation.** One existing refusal or one output profile does not create architecture authority. Compatible profile registration, cutoff/digest/census mechanics, limit handling and inspect-payload refusal tests must be consolidated in the Validation/Analysis Run owner path. Existing branches remain evidence until their unique behavior is folded; they are not individually production candidates merely because they are mergeable.

**#462 — bounded source-identifier repair.** Rust source uses `node_id` while the versioned serialized v1 wire key remains `id`. This is a compatibility-scoped naming repair, not a new bounded context or JSON-LD `@id` semantic change.

**#437 — ADR identity repair.** The #435 branch now requires one repository-wide identity/target per indexed ADR and rejects duplicate index numbers, duplicate index targets and duplicate numbered ADR files. The normalized index preserves pre-normalization collision lineage under `docs/adr/archive/` rather than treating branch-local ADR numbers as architecture authority. Issue closure waits for protected-main integration.

## Dependency and integration status

TEPP must not copy reusable static psychometric kernels from fast-mlsirm or provider-selection logic from contextual-orchestrator. Upstream candidate/model contracts are consumed only through versioned ACLs. A checksum-pinned contextual-orchestrator dependency is not advanced until the exact replacement source archive digest is reproducibly acquired; the checksum is never removed merely to follow upstream.

`context-graph-contracts` and `enterprise-architecture-core` remain read-only to this writer while their dedicated Context Fabric writer is active. Open PR heads are not released contracts. With no released Context Graph artifact, TEPP may prepare conformance fixtures behind candidate/test boundaries but cannot bind deployable integration or authoritative EA projection to a stacked branch.

## Operator-gap register

| ID | Gap | Maturity | Authority | Closure evidence |
| --- | --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 134 open PRs | `release-blocking` | #175 / #435 | classified queue, coherent bounded-context landing vehicles, unique evidence preserved, safe reduction |
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
| GAP-014 | required dependency-review evidence unavailable | `blocked_external` | repository security configuration + central workflow | exact base/head compare endpoint returns HTTP 403; enable/repair dependency graph support, rerun exact-head Security Scan, retain fail-closed behavior |

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

Every open PR receives one of `landing_vehicle`, `stacked_dependency`, `fold_into_landing_vehicle`, `superseded`, `duplicate`, `research_lineage_only`, or `blocked_external`. Before closing/folding work, compare exact heads and preserve unique production behavior, compatibility, tests, review findings, primary research, doctoring and provenance. Strict linear ancestry permits retarget-and-close consolidation; diverged siblings require an actual source/test fold first.

## Release truth

TEPP has no GitHub release at this snapshot. A release is permitted only after a coherent buyer/scientific vertical reaches protected main with current recovery and security evidence, reproducible package/build provenance, SBOM, upgrade/rollback evidence, and released integration contracts where cross-product deployment depends on them.