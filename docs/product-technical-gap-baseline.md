# Product and Technical Gap Baseline

**Status:** Active delivery recovery
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-09-02T00:15Z
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`
**Workspace version:** `0.2.0`
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md)

## Delivery truth

A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Passing or queued checks never promote an open PR to `implemented-main`.

| Signal | Current evidence | Delivery implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until protected main changes. |
| Open pull requests | **131** | WIP remains release-blocking, but the queue is down from the observed peak of 149. |
| Draft pull requests | **130** | Non-landable work is explicitly parked while consolidated, repaired, or supplied with missing evidence. |
| Non-draft pull requests | **1** | #310 is the only current non-Draft landing vehicle. |
| Open issues | **15** | Includes #437 for repository-wide ADR identity normalization and active Analysis Run/product gaps. |
| GitHub releases | **0** | No open head is a released TEPP contract. |
| Effective organization ruleset | `18156473` | Current-head required workflows, resolved conversations, qualifying review, and an allowed merge method remain landing authority. |

Classic branch protection is not the sole policy source; organization rulesets are effective merge authority where applicable. Any increase in open PR count while #435 remains open is a WIP regression unless the new PR is a demonstrably independent root-cause repair that cannot safely belong to an existing landing vehicle.

The queue previously regressed when one-map and one-route Drafts appeared as independently shippable work. Recovery preserves unique RED/GREEN, review, research, and contract evidence on coherent bounded-context landing vehicles before strict ancestors or superseded siblings are closed. The live TEPP queue is **131**.

## Current priority open pull-request evidence

This is a priority subset, not a row-for-row copy of the 131-PR queue. #435 intentionally omits its own SHA because embedding the branch head in a file changed by that same branch would make the evidence self-stale. Every row below was re-read before this snapshot.

| PR | Exact current head | Draft | Base | Ownership / disposition |
| ---: | --- | :---: | --- | --- |
| #473 | `9ae0488e44bc0775b2289c71d83d88c57e660b0d` | true | profile ancestry | Validation / Analysis Run inferred-status refusal profile; `fold_into_landing_vehicle`, not independent architecture authority. |
| #469 | `72a7755bcc91b1107560c980ce817eca153126e4` | true | interpretation-run retrieval ancestry | Analysis Run / contextual-orchestrator interpretation-run retrieval, lookup, and stored-request adapter landing vehicle. |
| #466 | `71f34b890bbd096eee152947c5e22d9778d323e8` | true | export-retrieval ancestry | Analysis Run / naruon export idempotency lookup and quarantine-parity landing vehicle. |
| #464 | `1b3a477242336634be2c7867b29d39979e9a6dca` | true | temporal-context retrieval ancestry | Analysis Run / LineageWeave temporal-context stored-request GET+CLI landing vehicle. |
| #462 | `c1b7d627167dd7636d2975cc41cec050a5e477ba` | true | main | Bounded source-name compatibility repair; Rust `node_id`, v1 serialized key remains `id`. |
| #456 | `f02436236a73824c87c6043fc5d1e0b08cb0d448` | true | project-history retrieval ancestry | Analysis Run / LineageWeave project-history landing vehicle; includes folded query CLI evidence. |
| #417 | `1e468f62ec47f3476a7b4d18ed2980451dc425cf` | true | main | Analysis Run / naruon export retrieval GET+CLI landing vehicle. |
| #416 | `a6ba5d79c8d9acc1e7b4a53b3be5ef3d70ab19e7` | true | main | Provisional Validation / Analysis Run simple-refusal landing vehicle; shared admission/artifact invariants plus profile-specific adapters. |
| #315 | `538f9bd1c76422bc894836b65083c62544330c7c` | true | main | TEPP `TIPREDVARstd` adapter lineage only; blocked on a released fast-mlsirm covariance-standardisation contract and removal of duplicate local arithmetic. |
| #310 | `3132a0818455c982f211a7f170fdf2b8db63fa7b` | false | main | Longitudinal Modeling landing vehicle; repaired lagged correlation, diffusion research candidates, and mixed-sign full-range/subnormal-safe CWC mean arithmetic. |

Exact current-head evidence is authoritative only for the named PR and becomes stale after any source mutation.

## Domain ownership

Cargo crates, HTTP routes, CLI verbs, refusal rules, clocks, and individual statistical maps are implementation units, not bounded contexts.

| Subdomain | Bounded context | Aggregate authority | Implementation nucleus |
| --- | --- | --- | --- |
| Core | Evidence & Semantic Measurement | `EvidenceCorpus`, `SemanticUnitSet`, `ConceptDictionaryRevision` | `evidence_core`, `semantic_core` |
| Core | Temporal Semantics | `TemporalEvidenceWindow`, `KnowledgeCutoffPolicy` | temporal primitives and cutoff policy |
| Core | Event Ontology & Temporal Graph | `EventEpisode`, `TemporalRelationSet` | `event_core`, `relation_graph` |
| Core | Measurement | `MeasurementSpecification`, `MeasurementRun` | measurement modules + released fast-mlsirm ACL |
| Core | Longitudinal Modeling | `TemporalModelSpecification`, `TemporalModelRun` | `longitudinal_core` + temporal/event composition |
| Core | Validation Evidence | `ValidationStudy`, `ValidationEvidence` | `validation_core`, `tepp_simulation` |
| Core | Scientific Claim Promotion | `ClaimPromotionDecision` | ADR 0014 policy; consumes validation evidence but does not own estimators or transport |
| Supporting | Projection / Analysis Run | `AnalysisRun`, published read models | application services; HTTP/CLI are adapters |
| Supporting | Interpretation | evidence-grounded interpretation workflow | contextual-orchestrator ACL only |
| Supporting | Persistence & Recovery | repositories and durable receipts | `persistence_postgres`, object-store adapters |
| Generic | Compute backend | execution receipt | CPU/GPU/MLX adapters; execution is not scientific authority |

**Canonical owners:** fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and numerical kernels, including reusable covariance standardisation and LSIRM/MLSIRM/DLSJM kernels. TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment, and temporal recovery. contextual-orchestrator owns every LLM/provider call and routing decision. Context Graph contracts are a contract-only Shared Kernel; EA Core owns authoritative architecture decisions. No cross-service SQL.

The six-clock contract keeps event/valid time as one role, followed by assertion time, document time, system time, available time, and knowledge cutoff. A retrospective source may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective, citation, revision, and provenance relations.

## Temporal/dependence model policy

TEPP composes time over the full released upstream candidate identity, not hard-coded family names. Every candidate records exact response/generalized-mixed/dependence formulation, `supported | research_candidate | unsupported`, state/generative equations, clock roles, identification/alignment, time-varying covariates/random effects/memberships, estimator owner, required data support, primary citations, and recovery status.

Rasch remains distinct from generic 1PL. Formulation-qualified 2PLM–5PLM, confirmatory/exploratory MIRT, and ideal-point/GGUM identities are preserved. Cross-classification and multiple membership remain distinct; membership weights are explicit, time-valid, and observed-normalized or model-estimated according to the formulation. LSIRM/MLSIRM temporal candidates preserve base parameters plus person/item geometry. DLSJM keeps distinct item- and person-dependence spaces. Dynamic geometry stays `research_candidate` until state equations, temporal identification/alignment, and true-parameter recovery exist.

Auto-expansion is not auto-activation. A numerical standardizer or adapter is not a DSEM/ctsem/LSIRM/MLSIRM/DLSJM estimator without exact generative/state equations, identification, estimator, and recovery evidence.

## Scientific validation invariants

Supported temporal estimators require realistic known-truth recovery for every claimed structure. Parameter claims require RMSE, bias, interval/credible-interval coverage, convergence, and uncertainty calibration. Temporal-order, transition, dependency-graph, branch-graph, cluster-transition, or cross-time comparison claims additionally require recovery of that structure and applicable longitudinal measurement-invariance evidence. Leakage-safe rolling-origin evaluation separates event/valid time from available time and tests irregular gaps, delayed or retrospective reports, missing occasions, changing memberships, and relevant language/source drift. CPU/GPU parity is evidence only when the corresponding accelerator path actually runs; skipped or ignored GPU tests do not qualify. Monte Carlo decisions use simulation uncertainty rather than arbitrary pass percentages. Scientific failure is never hidden with skip, xfail, source rewriting, or coverage exclusions.

## Current repairs and blockers

**#310 — Longitudinal Modeling landing vehicle.** Closed predecessor #441 is contained by #310. The invalid one-sided covariance/earlier-variance ratio remains retired; public lagged Pearson correlation requires lagged covariance and both occasion-specific marginal variances. `EventTimeInterval` is preserved end-to-end. Earlier RED/GREEN slices repaired event-order-sensitive CWC accumulation, stationary/diffusion edge arithmetic, and the `ln(0.5) / Δt` documentation boundary.

The current arithmetic repair progressed through two additional RED/GREEN slices. RED `0057b85a8467b9c904aaa20ffe79462aa3339786` showed that largest-magnitude normalization could erase a representable low-order contribution when near-maximum positive and negative rates cancelled. GREEN `af767e1bb9ab27c71023e0c3f4be1bb0918dd20f` removed that magnitude normalization, but fresh review found its count-derived power-of-two pre-scaling could still divide positive subnormal terms to zero before cancellation. RED `203a926680f52cf0a75de909247e6ad5db237403` reproduces the remaining defect with two positive subnormal rates bracketed by opposing extreme finite rates. GREEN `3132a0818455c982f211a7f170fdf2b8db63fa7b` removes mixed-sign pre-scaling entirely: values are partitioned by sign, ordered from largest magnitude down, opposite signs are cancelled first without overflow, and surviving same-sign residuals are averaged by the bounded convex path and weighted to the original sample count. The internal contract includes `[f64::MAX, 2*min_subnormal, 2*min_subnormal, -f64::MAX] -> min_subnormal`, plus previous full-range, same-sign-overflow, zero, empty, and non-finite cases. The two substantive cancellation review threads are resolved on the current source.

#310 is still not landable. Exact head `3132a0818455c982f211a7f170fdf2b8db63fa7b` has fresh Rust Foundation CI, Documentation Quality, Security Scan, and SAST Semgrep queued. Queued evidence is non-passing. There is no qualifying current-head independent `APPROVED` review. The exact-head verification conversation remains intentionally unresolved until the required workflows terminate successfully and review policy is satisfied.

The surviving branch also contains source/test evidence folded from #476/#477 under `longitudinal_core`: continuous and discrete scalar diffusion-standardisation candidates, positive stationary-within admission, subnormal-cancellation repair, scale-invariance tests, signed-zero event-product refusal, interval ordering, and named-estimand refusals. Those maps remain `research_candidate` because the 2017 ctsem summary source does not emit named `DIFFUSIONstd` / `discreteDIFFUSIONstd` matrices.

**#416 and sibling simple-refusal profiles.** #416 is the provisional Validation / Analysis Run landing vehicle. Its current head `a6ba5d79c8d9acc1e7b4a53b3be5ef3d70ab19e7` keeps the already-repaired `MAX_EVIDENCE_UNITS` execution/artifact bound before identity-set growth and adds a temporal-admission repair. RED `b7312eed81e25aba586323be058077dabc38a2df` proves that equivalent RFC 3339 spellings such as `2026-08-01T09:00:00+09:00` and `2026-08-01T00:00:00Z` were incorrectly rejected when request cutoff text was compared directly. GREEN `a6ba5d79c8d9acc1e7b4a53b3be5ef3d70ab19e7` parses the request cutoff into `KnowledgeCutoff` and compares `instant()` values, retaining genuine cutoff/profile/model mismatch refusal. The stale oversize-census thread and the equivalent-offset thread are resolved after current-source verification. Fresh exact-head Rust, Documentation, Security, and SAST workflows are queued, so #416 remains Draft and must not independently land ahead of consolidation.

Compatible style/prompt/modality/background/citation/copied-text/location/membership-target/outcome-order/relation-absence/episode-membership/inferred-status profiles must preserve unique refusal vocabulary, cutoff behavior, tests, and evidence while sharing execution/artifact-count/output-size invariants. #473 is explicitly `fold_into_landing_vehicle`; it is not independently shippable.

**#315 — static-standardisation ownership repair.** #315 remains Draft only to preserve TEPP-specific `TIPREDVARstd` naming/event-time/refusal semantics. It must not merge with duplicate reusable arithmetic. After fast-mlsirm #1722 or its live successor is merged and released/versioned, #315 must consume that immutable contract through an ACL, prove parity, retain TEPP temporal/model admission, delete wrong-owner production arithmetic, and fold into a Measurement/Longitudinal Modeling landing vehicle.

**Dependency-review support.** An earlier #310 head failed central Security Scan before Dependency Review because GitHub dependency-graph comparison was unavailable to the workflow token. That is predecessor evidence, not a permanent current-head exemption. The gate remains fail-closed; OSV, Trivy, and Scorecard are sibling evidence, not substitutes. Current head `3132a0818455c982f211a7f170fdf2b8db63fa7b` must obtain its own terminal Security Scan.

**Analysis Run adapter/profile proliferation.** Export, interpretation, project-history, inferred-status, and temporal-context HTTP/CLI mechanics are adapters inside supporting contexts. Strict ancestry may be folded without force after exact comparison; diverged siblings require an actual source/test fold. One refusal/profile/route/CLI verb does not create architecture authority.

**#437 — ADR identity.** Repository-wide ADR IDs are immutable authority. Duplicate index IDs, duplicate targets, duplicate numbered ADR files, repeated authority declarations, and index/file status or maturity drift must fail deterministic documentation fitness tests. Operation-specific ADRs on adapter/model micro-branches are implementation lineage pending normalization, not branch-local architecture authority.

## Dependency and Context Fabric status

The fast-mlsirm generalized-mixed/dependence compiler remains PR #1714 at exact head `6abdcc2acab7be463977e35191566119e384c906`. It publishes typed response/dimensional/generalized-mixed/dependence identities, typed membership topology and weight authority, formulation-scoped promotion evidence, and deterministic candidate manifest `fast_mlsirm.model_specification.candidate_manifest@1.0.0` with SHA-256 digest. All visible substantive review conversations are resolved, but exact-head CI is pending and Security/SAST/CodeQL are queued. It is not a released TEPP dependency.

TEPP protected main checksum-pins contextual-orchestrator commit `e226e1197bdfc890c9d8e5b9b648c78857d7e465` with SHA-256 `964b22ff577e3862b761af847ccad65489bb3f8fc750c8f84fcf8628df096673`. Upstream protected `main` is `4d143601c2904a28e95d091b261c0a15e9a4f283`, which merged the Bytez zero-price ranking repair. Advancing TEPP requires a reproducible replacement archive digest and review of the exact replacement; removing checksum validation or pinning an unverified digest is prohibited.

`context-graph-contracts` and `enterprise-architecture-core` remain read-only to this writer while their dedicated Context Fabric owner loop is active. Fresh state is:

- context-graph-contracts default/protected `develop@99cb5468ba3c15c5e79688f53dee74724fae2d13`, 13 open PRs, zero releases. Current Context Assertion/CloudEvent envelope work remains unreleased candidate work.
- enterprise-architecture-core default/protected `develop@1c0fa8b15ceb9e72186274aeb255d6777eb84ef4`, 24 open PRs, zero releases.

Open heads are candidate evidence, not released contracts. TEPP may maintain fail-closed conformance fixtures behind candidate/test boundaries, but deployable integration and authoritative EA projection require a released/versioned Context Graph artifact plus passing compatibility evidence. TEPP latent estimates, measurement scores, inferred event relations, and validity evidence do not become authoritative EA facts.

## Gap register

| ID | Gap | Maturity | Closure evidence |
| --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 131 open PRs | `release-blocking` | coherent landing vehicles, unique evidence preservation, protected-main reduction |
| GAP-002 | multilingual span-grounded semantic/concept admission | `partial` | immutable offsets/layout, language profiles, concept dictionary, invariance/calibration, hostile-input tests |
| GAP-003 | shared-latent temporal topic estimator | `partial` | Rust CPU f64 likelihood/uncertainty, relation/time/membership effects, true recovery, fitted candidate-K |
| GAP-004 | durable end-to-end Analysis Run | `partial` | idempotent lifecycle, persistence/recovery, estimator-bound artifacts, validation/promotion separation, Compose E2E |
| GAP-005 | temporal psychometric composition/duplication | `partial` | released fast-mlsirm contracts, TEPP ACLs, temporal recovery, wrong-owner static kernels removed after parity |
| GAP-006 | event intelligence | `partial` | calibrated detection/tracking/schema/interval recovery and durable artifacts |
| GAP-007 | accelerator/memory evidence | `accepted-target` | real hardware, CPU-f64 parity, bounded OOM/fallback evidence |
| GAP-008 | network/cluster buyer workflow | `partial` | known-truth recovery, uncertainty/stability, repeated consensus, exact-value export |
| GAP-009 | production interpreter/verifier | `partial` | contextual-orchestrator execution, evidence citations, independent verification, abstention/fallback |
| GAP-010 | accessible buyer UI | `accepted-target` | Figma/Storybook, keyboard/touch/error/empty states, exact-value provenance |
| GAP-011 | operable multi-tenant release | `accepted-target` | OIDC/RLS/purpose controls, durable queue/storage, OTel/SLO, restore/load/migration, signed SBOM/provenance |
| GAP-012 | paths obscure domain ownership | `active-refactor` | staged moves, ACLs, no cycles/cross-context persistence/shared-kernel creep |
| GAP-013 | ADR identity collisions | `release-integrity` | unique repository-wide identity, deterministic duplicate detection, supersession lineage |
| GAP-014 | dependency-review evidence unavailable on predecessor heads | `external-control-risk` | authorized dependency-graph availability, pinned Dependency Review execution, exact-current Security Scan GREEN, no fail-open bypass |
| GAP-015 | contextual-orchestrator pin behind protected upstream | `supply-chain-pinned` | reproducible replacement archive SHA-256, exact diff/review, pinned TEPP adoption |

## Delivery and release order

Queue/ADR/domain authority precedes semantic admission, the Rust CPU f64 shared-latent temporal estimator, durable Analysis Run, released fast-mlsirm temporal composition, event intelligence, accelerator parity, buyer workflows, and finally tenancy/durability/observability/release support. A bounded security/dependency repair may land earlier when it directly unblocks a selected vehicle.

Every open PR is classified as `landing_vehicle`, `stacked_dependency`, `fold_into_landing_vehicle`, `superseded`, `duplicate`, `research_lineage_only`, or `blocked_external`. Exact heads must be compared before closure. Strict ancestry permits safe consolidation; diverged siblings require a real code/test fold first.

TEPP has no GitHub release at this snapshot. A release is permitted only after a clean integration state reaches protected main with exact protected-head CI/security evidence, claim-scoped scientific and recovery acceptance, reproducible artifacts with SBOM and provenance, validated migrations/upgrade/rollback/recovery where applicable, consistent version metadata and a current `CHANGELOG.md`, accessibility and operability evidence for user-facing components, no unresolved scientific/privacy/security/supply-chain blockers, and released integration contracts where cross-product deployment depends on them.
