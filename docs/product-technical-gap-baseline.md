# Product and Technical Gap Baseline

**Status:** Active delivery recovery
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-09-02T14:34Z
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`
**Workspace version:** `0.2.0`
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md)

## Delivery truth

A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Passing or queued checks never promote an open PR to `implemented-main`.

| Signal | Current evidence | Delivery implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until protected main changes. |
| Open pull requests | **130** | The queue is down from the observed peak of 149 but remains above the prior fresh 128-PR snapshot; #482/#483 are WIP-regression fold children, not independent landing authority. |
| Draft pull requests | **129** | Non-landable work is explicitly parked while consolidated, repaired, or supplied with missing evidence. |
| Non-draft pull requests | **1** | #480 is the only non-Draft PR, but it is not deployable while contextual-orchestrator has no compatible immutable release and released gateway identity/authentication contracts remain incomplete. |
| Open issues | **16** | Includes #437 ADR identity normalization, #479 hourly orchestrator admission, and #481 dynamic-evaluation drift design/evidence gathering. |
| GitHub releases | **0** | No open TEPP head is a released contract. |
| Effective organization ruleset | `18156473` | One qualifying approval, stale-review dismissal after push, resolved review threads, unattributed-change approval where applicable, and central required workflows remain landing authority. |

Classic branch protection is not the sole policy source; organization rulesets are effective merge authority where applicable. Any increase in open PR count while #435 remains open is a WIP regression unless the new PR is a demonstrably independent root-cause repair that cannot safely belong to an existing landing vehicle. #480 is such an exception because it owns TEPP's consumer-side CI/orchestrator admission boundary and cannot coherently belong to the Validation/Analysis Run, Longitudinal Modeling, or queue-documentation vehicles.

The queue was **128** at the previous fresh snapshot and rose to **130** when #482 `role_contradiction_v1` and #483 `retrospective_edge_v1` were opened as one-profile slices. Fresh changed-file classification showed both modify the same Analysis Run/Validation shared Cargo/lib/docs surfaces as #416. #458, #460, #482, and #483 now all target the #416 branch with comparison base SHA `0b7155cc238defb1e55129ff3000658f04b343cf`; GitHub still reports all four as non-mergeable. The remaining conflicts are therefore real shared-file fold work rather than stale base metadata. Unique source, tests, doctoring, RED/repair lineage, and temporal-refusal semantics must survive into the eventual #416 landing head. Issue #481 remains design/evidence work only and does not authorize implementation against mutable upstream PR heads.

## Current priority open pull-request evidence

This is a freshly verified priority subset, not a row-for-row copy of the 130-PR queue. #435 intentionally omits its own SHA because embedding a branch head inside a file changed by that same branch would make the file self-stale.

| PR | Exact current head | Draft | Base | Ownership / disposition |
| ---: | --- | :---: | --- | --- |
| #483 | `847d96f913bb261803ac0bd751ad7e4f51324cee` | true | #416 branch @ `0b7155cc238defb1e55129ff3000658f04b343cf` | `retrospective_edge_v1` fold child; cutoff-before-identity admission repair and unique tests/doctoring must survive the conflict-resolving fold. |
| #482 | `506dbae236a4484301b704b6c6a05b20faf0fe69` | true | #416 branch @ `0b7155cc238defb1e55129ff3000658f04b343cf` | `role_contradiction_v1` fold child; cutoff-before-identity admission repair and unique tests/doctoring must survive the conflict-resolving fold. |
| #480 | `4475542750eda01afad0cf9ea8d563f508f63fd3` | false | main | TEPP consumer-side LLM governance repair; removes provider discovery/ranking and requires released `orchestrator/free` through HTTPS. Still blocked on immutable owner release, authenticated deployment provenance, and non-reusable/brokered gateway authentication. |
| #460 | `dfab4eab5ff733731e565a9348072b8dab2e4912` | true | #416 branch @ `0b7155cc238defb1e55129ff3000658f04b343cf` | `relation_absence_v1` fold child; typed cutoff equality and terminal validation-status separation preserved. |
| #458 | `08165e3b3c929b4ae77396689549f72723ff8ff5` | true | #416 branch @ `0b7155cc238defb1e55129ff3000658f04b343cf` | `outcome_order_v1` fold child; typed cutoff equality and terminal validation-status separation preserved. |
| #416 | `0b7155cc238defb1e55129ff3000658f04b343cf` | true | main | Provisional Validation / Analysis Run simple-refusal landing vehicle; generic cutoff-before-identity admission is the surviving leakage-safe invariant. |
| #310 | `260413efb9d95039b5fbba41919cba8097fcf8b5` | true | main | Longitudinal Modeling scientific landing work; CWC unit-mean accumulation now survives overflowing raw partial sums when the mean/residuals remain representable, reusing the canonical compensated mean. Current hosted merge evidence is non-GREEN. |

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

fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and numerical kernels, including reusable covariance standardisation and LSIRM/MLSIRM/DLSJM kernels. TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment, and temporal recovery. contextual-orchestrator owns every LLM/provider execution and routing decision. Context Graph contracts are a contract-only Shared Kernel; EA Core owns authoritative architecture decisions. No cross-service SQL.

The six-clock contract keeps event/valid time as one role, followed by assertion time, document time, system time, available time, and knowledge cutoff. A retrospective source may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective, citation, revision, and provenance relations.

## Temporal/dependence model policy

TEPP composes time over the full released upstream candidate identity, not hard-coded family names. Every candidate records exact response/generalized-mixed/dependence formulation, `supported | research_candidate | unsupported`, state/generative equations, clock roles, identification/alignment, time-varying covariates/random effects/memberships, estimator owner, required data support, primary citations, and recovery status.

Rasch remains distinct from generic 1PL. Formulation-qualified 2PLM–5PLM, confirmatory/exploratory MIRT, and ideal-point/GGUM identities are preserved. Cross-classification and multiple membership remain distinct; membership weights are explicit, time-valid, and observed-normalized or model-estimated according to the formulation. LSIRM/MLSIRM temporal candidates preserve base parameters plus person/item geometry. DLSJM keeps distinct item- and person-dependence spaces. Dynamic geometry stays `research_candidate` until state equations, temporal identification/alignment, and true-parameter recovery exist.

Auto-expansion is not auto-activation. A numerical standardizer or adapter is not a DSEM/ctsem/LSIRM/MLSIRM/DLSJM estimator without exact generative/state equations, identification, estimator, and recovery evidence.

## Scientific validation invariants

Supported temporal estimators require realistic known-truth recovery for every claimed structure. Parameter claims require RMSE, bias, interval/credible-interval coverage, convergence, and uncertainty calibration. Temporal-order, transition, dependency-graph, branch-graph, cluster-transition, or cross-time comparison claims additionally require recovery of that structure and applicable longitudinal measurement-invariance evidence. Leakage-safe rolling-origin evaluation separates event/valid time from available time and tests irregular gaps, delayed or retrospective reports, missing occasions, changing memberships, and relevant language/source drift. CPU/GPU parity is evidence only when the corresponding accelerator path actually runs; skipped or ignored GPU tests do not qualify. Monte Carlo decisions use simulation uncertainty rather than arbitrary pass percentages. Scientific failure is never hidden with skip, xfail, source rewriting, or coverage exclusions.

## Current repairs and blockers

**#310 — Longitudinal Modeling scientific landing work.** Closed predecessor #441 is contained by #310. The invalid one-sided covariance/earlier-variance ratio remains retired; public lagged Pearson correlation requires lagged covariance and both occasion-specific marginal variances. `EventTimeInterval` is preserved end-to-end. Current head `260413efb9d95039b5fbba41919cba8097fcf8b5` preserves the prior stationary-variance/cancellation/log-domain repairs and adds RED `9a706c3c0e9e0db68e88f89b94c64c13ea7fafd0` plus causal repair `260413efb9d95039b5fbba41919cba8097fcf8b5` for a distinct CWC centering defect. The predecessor formed the unit mean with a naive raw sum, so finite `[0.75·MAX, 0.75·MAX, -0.5·MAX]` observations were rejected when the first partial sum overflowed even though the final mean and centered residuals were representable. The repair reuses the canonical Longitudinal Modeling `scaled_compensated_mean`; it does not mint a second numerical implementation. The historical `[MAX, MAX]` input now forms the valid finite mean `MAX`, then correctly fails later because zero residuals provide no admissible real log-rate, rather than being mislabeled as an observation-payload overflow. The PR remains Draft. Current exact-head hosted merge evidence is non-GREEN: the first materialized PR workflow observed after the repair is Scorecard run `33642604895`, still queued, and predecessor-head CI/review results do not transfer. A qualifying current-head independent approval is still required. This is not permission to bypass the gate.

**#416 — Validation / Analysis Run consolidation.** Current head `0b7155cc238defb1e55129ff3000658f04b343cf` contains non-force lineage from #473, #430, #461, #478, and #434. Shared admission/artifact invariants remain centralized while each refusal profile keeps its domain vocabulary. Generic RED `ffee655404716bf8d33c898a3c1a87a543abe701` and repair `0b7155cc238defb1e55129ff3000658f04b343cf` enforce availability cutoff before duplicate-identity admission so future-unavailable rows cannot change historical replay. #458, #460, #482, and #483 are now compared against this exact head and still conflict on shared Cargo/lib/lock/docs surfaces. Their unique evidence must be folded, not replaced or independently landed.

**#480 / #479 — hourly orchestrator admission.** Current exact head `4475542750eda01afad0cf9ea8d563f508f63fd3` removes TEPP's second provider-routing authority. Scheduled execution now requires an immutable contextual-orchestrator release, an HTTPS gateway, gateway authentication, and only `contextual-orchestrator/orchestrator/free`; provider discovery, provider/model/group ranking, free/paid admission, and provider credentials remain owner-side. RED `6d756d02409d0eb11a35146b9abfe41369efd2ad` → repair `f1da3f29ee1c9d3de6923a52d6cf26b71b96d257` restricts gateway redirects to HTTPS, and RED `1f0d2ddfb3ac5d8e6c8e1c1c5c40d47c46a017c9` → repair `4475542750eda01afad0cf9ea8d563f508f63fd3` applies the same downgrade protection to the checksum-pinned OpenCode archive path. Two owner gaps remain: the running gateway cannot yet prove authenticated identity bound to the selected immutable release/schema/artifact, and the model-controlled process still receives a reusable gateway bearer. Both are tracked in contextual-orchestrator #1023 and require owner-side release/provenance plus scoped/ephemeral or brokered authentication. contextual-orchestrator protected `main@464da4715b495b5eaaa593eba3796e2d976ee0c9` still has zero GitHub releases, so #480 remains deliberately non-deployable rather than falling back to mutable source or direct provider execution.

**#481 — dynamic evaluation drift monitoring design/evidence lane.** TEPP owns temporal composition, availability/knowledge-cutoff semantics, longitudinal/multilevel/multiple-membership structure, drift/change-point evidence, and time-indexed invariance monitoring. It does not own item generation, provider/rater invocation, adjudication, anchor promotion, or reusable static psychometric kernels. Implementation remains blocked until canonical owners publish immutable released/digest-pinned run/item/rater/adjudication/linking contracts. The first eventual slice is a versioned ACL with no-anchor/no-linking fail-closed tests, not a drift score or dashboard. Current mutable sibling PR heads are evidence only and must not become production dependencies.

**#315 — static-standardisation ownership repair.** #315 remains lineage only to preserve TEPP-specific `TIPREDVARstd` naming/event-time/refusal semantics. It must not merge with duplicate reusable arithmetic. The current canonical owner candidate is fast-mlsirm #1722 at exact head `338dbb2d25f32b0e201102e7bf73076846fb57b3`, exposing `fast_mlsirm.covariance_standardization@1.0.0` from protected-base `main@b5a3a0c1057d4b53d7a4bb18e0de69f630c2b45c`. Immutable `v0.9.1` predates #1722. TEPP therefore keeps #315 as research/compatibility lineage only until #1722 or its verified successor lands and is immutably released, after which TEPP must consume that released contract through an ACL, prove parity, retain temporal/model admission, delete wrong-owner production arithmetic, and fold TEPP-specific composition into a Measurement/Longitudinal Modeling landing vehicle.

**#437 — ADR identity.** Repository-wide ADR IDs are immutable authority. Duplicate index IDs, duplicate targets, duplicate numbered ADR files, repeated authority declarations, and index/file status or maturity drift must fail deterministic documentation fitness tests. Operation-specific ADRs on adapter/model micro-branches are implementation lineage pending normalization, not branch-local architecture authority. #435 contains the current deterministic fitness implementation; issue closure still requires exact-head hosted evidence and a coherent landing decision.

## Dependency and Context Fabric status

fast-mlsirm protected `main` is `b5a3a0c1057d4b53d7a4bb18e0de69f630c2b45c`. Generalized-mixed/dependence compiler #1714 is open, Ready, mergeable, and currently at `92a3f2152033b61ca89661b5ba8a584842e8c3a9` on that exact base. It preserves typed membership topology/weight authority and deterministic `fast_mlsirm.model_specification.candidate_manifest@1.0.0`. The latest immutable fast-mlsirm release is `v0.9.1` from 2026-08-26 and predates #1714 and #1722, so TEPP must not consume either new Published Language until an immutable compatible owner release exists.

TEPP protected main checksum-pins contextual-orchestrator commit `e226e1197bdfc890c9d8e5b9b648c78857d7e465` with SHA-256 `964b22ff577e3862b761af847ccad65489bb3f8fc750c8f84fcf8628df096673`. Fresh owner state is protected `main@464da4715b495b5eaaa593eba3796e2d976ee0c9`, but contextual-orchestrator still has zero GitHub releases. Advancing TEPP requires a compatible immutable release, authenticated deployment/release provenance, a safe gateway-authentication contract, and an exact reviewed ACL/dependency bump; checksum validation must not be removed or guessed.

`context-graph-contracts` and `enterprise-architecture-core` remain read-only to this writer. Their open heads are candidate evidence, not released contracts. TEPP may maintain fail-closed conformance fixtures behind candidate/test boundaries, but deployable integration and authoritative EA projection require released/versioned Context Graph artifacts plus passing compatibility and provenance evidence. TEPP latent estimates, measurement scores, inferred event relations, and validity evidence do not become authoritative EA facts.

## Gap register

| ID | Gap | Maturity | Closure evidence |
| --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 130 open PRs | `release-blocking` | coherent landing vehicles, unique evidence preservation, protected-main reduction |
| GAP-002 | multilingual span-grounded semantic/concept admission | `partial` | immutable offsets/layout, language profiles, concept dictionary, invariance/calibration, hostile-input tests |
| GAP-003 | shared-latent temporal topic estimator | `partial` | Rust CPU f64 likelihood/uncertainty, relation/time/membership effects, true recovery, fitted candidate-K |
| GAP-004 | durable end-to-end Analysis Run | `partial` | idempotent lifecycle, persistence/recovery, estimator-bound artifacts, validation/promotion separation, Compose E2E |
| GAP-005 | temporal psychometric composition/duplication | `partial` | released fast-mlsirm contracts, TEPP ACLs, temporal recovery, wrong-owner static kernels removed after parity |
| GAP-006 | event intelligence | `partial` | calibrated detection/tracking/schema/interval recovery and durable artifacts |
| GAP-007 | accelerator/memory evidence | `accepted-target` | real hardware, CPU-f64 parity, bounded OOM/fallback evidence |
| GAP-008 | network/cluster buyer workflow | `partial` | known-truth recovery, uncertainty/stability, repeated consensus, exact-value export |
| GAP-009 | production interpreter/verifier | `partial` | released contextual-orchestrator execution, evidence citations, independent verification, abstention/fallback |
| GAP-010 | accessible buyer UI | `accepted-target` | Figma/Storybook, keyboard/touch/error/empty states, exact-value provenance |
| GAP-011 | operable multi-tenant release | `accepted-target` | OIDC/RLS/purpose controls, durable queue/storage, OTel/SLO, restore/load/migration, signed SBOM/provenance |
| GAP-012 | paths obscure domain ownership | `active-refactor` | staged moves, ACLs, no cycles/cross-context persistence/shared-kernel creep |
| GAP-013 | ADR identity collisions | `release-integrity` | unique repository-wide identity, deterministic duplicate detection, supersession lineage |
| GAP-014 | required workflow startup/runner evidence unavailable on current heads | `external-control-risk` | central runner/workflow repair, exact-current required workflows GREEN, no fail-open bypass |
| GAP-015 | contextual-orchestrator lacks immutable released contract for current owner behavior | `release-blocking` | compatible immutable CO release, deployment identity/provenance, safe gateway authentication, exact TEPP ACL/dependency adoption |
| GAP-016 | hourly LLM path needs released owner-only routing/authentication | `active-repair` | #480 consumer repair plus released CO adoption, authenticated deployment identity, brokered/scoped auth, exact-head GREEN, qualifying review, protected-main merge |
| GAP-017 | dynamic evaluation item/rater/anchor drift monitoring | `blocked-external-design` | released/digest-pinned owner contracts, ACL conformance, no-anchor/no-linking refusal, then evidence-gated temporal monitoring |

## Delivery and release order

Queue/ADR/domain authority precedes semantic admission, the Rust CPU f64 shared-latent temporal estimator, durable Analysis Run, released fast-mlsirm static psychometric owner contracts plus TEPP-owned temporal composition, event intelligence, accelerator parity, buyer workflows, and finally tenancy/durability/observability/release support. A bounded security/dependency/provider-admission repair may land earlier when it directly closes a fail-closed control defect and all immutable-owner prerequisites exist.

Every open PR is classified as `landing_vehicle`, `stacked_dependency`, `fold_into_landing_vehicle`, `superseded`, `duplicate`, `research_lineage_only`, `blocked_external`, or an explicitly justified independent root-cause repair. Exact heads must be compared before closure. Strict ancestry permits safe consolidation; diverged siblings require a real code/test fold first.

TEPP has no GitHub release at this snapshot. A release is permitted only after a clean integration state reaches protected main with exact protected-head CI/security evidence, claim-scoped scientific and recovery acceptance, reproducible artifacts with SBOM and provenance, validated migrations/upgrade/rollback/recovery where applicable, consistent version metadata and a current `CHANGELOG.md`, accessibility and operability evidence for user-facing components, no unresolved scientific/privacy/security/supply-chain blockers, and released integration contracts where cross-product deployment depends on them.