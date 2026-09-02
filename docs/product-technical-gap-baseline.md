# Product and Technical Gap Baseline

**Status:** Active delivery recovery
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-09-02T06:44Z
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`
**Workspace version:** `0.2.0`
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md)

## Delivery truth

A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Passing or queued checks never promote an open PR to `implemented-main`.

| Signal | Current evidence | Delivery implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until protected main changes. |
| Open pull requests | **128** | The queue is down from the observed peak of 149, but remains release-blocking. |
| Draft pull requests | **126** | Non-landable work is explicitly parked while consolidated, repaired, or supplied with missing evidence. |
| Non-draft pull requests | **2** | #310 is the scientific landing vehicle; #480 is an independently landable CI/provider-admission repair. |
| Open issues | **16** | Includes #437 ADR identity normalization, #479 hourly free-route admission, and #481 dynamic-evaluation drift design/evidence gathering. |
| GitHub releases | **0** | No open TEPP head is a released contract. |
| Effective organization ruleset | `18156473` | Current-head required workflows, resolved conversations, qualifying review, and an allowed merge method remain landing authority. |

Classic branch protection is not the sole policy source; organization rulesets are effective merge authority where applicable. Any increase in open PR count while #435 remains open is a WIP regression unless the new PR is a demonstrably independent root-cause repair that cannot safely belong to an existing landing vehicle. #480 is such an exception: it owns TEPP CI/provider-admission policy and cannot coherently belong to the Validation/Analysis Run, Longitudinal Modeling, or queue-documentation vehicles.

The live queue was **127** immediately before #480 was opened and is **128** after that bounded governance repair. This increase is intentional and must not be used as precedent for one-operation/model/route micro-PR creation. Issue #481 adds design/evidence work only; it does not authorize an implementation PR until immutable released upstream contracts exist.

## Current priority open pull-request evidence

This is a priority subset, not a row-for-row copy of the 128-PR queue. #435 intentionally omits its own SHA because embedding the branch head in a file changed by that same branch would make the evidence self-stale.

| PR | Exact current head | Draft | Base | Ownership / disposition |
| ---: | --- | :---: | --- | --- |
| #480 | `f8749d315bf65ad3ba0e3f790c6e00d90072de00` | false | main | TEPP CI/provider-admission governance repair for issue #479; explicit zero-cost discovered routes only, fail-closed when none exist; review-driven ADR/runbook/fixture/CHANGELOG repairs present. |
| #469 | `72a7755bcc91b1107560c980ce817eca153126e4` | true | interpretation-run retrieval ancestry | Analysis Run / contextual-orchestrator interpretation-run retrieval, lookup, and stored-request adapter landing vehicle. |
| #466 | `71f34b890bbd096eee152947c5e22d9778d323e8` | true | export-retrieval ancestry | Analysis Run / naruon export idempotency lookup and quarantine-parity landing vehicle. |
| #464 | `1b3a477242336634be2c7867b29d39979e9a6dca` | true | temporal-context retrieval ancestry | Analysis Run / LineageWeave temporal-context stored-request GET+CLI landing vehicle. |
| #462 | `c1b7d627167dd7636d2975cc41cec050a5e477ba` | true | main | Bounded source-name compatibility repair; Rust `node_id`, v1 serialized key remains `id`. |
| #456 | `f02436236a73824c87c6043fc5d1e0b08cb0d448` | true | project-history retrieval ancestry | Analysis Run / LineageWeave project-history landing vehicle; includes folded query CLI evidence. |
| #417 | `1e468f62ec47f3476a7b4d18ed2980451dc425cf` | true | main | Analysis Run / naruon export retrieval GET+CLI landing vehicle. |
| #416 | `b0d6cb8969aa0ddd386ab82a1755155fa13d18a8` | true | main | Provisional Validation / Analysis Run simple-refusal landing vehicle; now preserves copy identity, inferred status, location membership, episode membership, subevent containment, and membership-target evidence. |
| #315 | `538f9bd1c76422bc894836b65083c62544330c7c` | true | main | TEPP `TIPREDVARstd` adapter lineage only; blocked on a released fast-mlsirm covariance-standardisation contract and deletion of duplicate local arithmetic. |
| #310 | `3132a0818455c982f211a7f170fdf2b8db63fa7b` | false | main | Longitudinal Modeling landing vehicle; repaired lagged correlation, diffusion research candidates, and full-range/subnormal-safe CWC mean arithmetic. |

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

**#310 — Longitudinal Modeling landing vehicle.** Closed predecessor #441 is contained by #310. The invalid one-sided covariance/earlier-variance ratio remains retired; public lagged Pearson correlation requires lagged covariance and both occasion-specific marginal variances. `EventTimeInterval` is preserved end-to-end. Current head `3132a0818455c982f211a7f170fdf2b8db63fa7b` also preserves the full-range/subnormal-safe mixed-sign CWC mean repair. It is not landable until exact-head required workflows terminate successfully and a qualifying independent approval exists.

**#416 — Validation / Analysis Run consolidation.** Current head `b0d6cb8969aa0ddd386ab82a1755155fa13d18a8` contains non-force lineage from #473, #430, #461, #478, and #434. Shared admission/artifact invariants remain centralized while each refusal profile keeps its domain vocabulary. The #478 TRACEABILITY repair now explicitly keeps both `containment_recovery_rate` and `identity_recovery_rate` out of inspect payloads. Next compatible simple-refusal siblings still targeting `main` are #458 (`outcome_order_v1`) and #460 (`relation_absence_v1`); they should fold into this vehicle or a coherent successor rather than ship as independent bounded contexts.

**#480 / #479 — hourly free-route admission.** Protected main currently ranks all discovered chat candidates without a free-only admission step. #480 current exact head `f8749d315bf65ad3ba0e3f790c6e00d90072de00` repairs the TEPP-owned bootstrap ACL so both provider-reported token-price components must be explicitly zero before a route reaches ranking. Paid, partially priced, fully unpriced, and missing-price production rows are excluded; an empty explicit-free pool fails closed. Focused regressions cover paid-vs-free ranking, unknown Bytez-style prices, partial prices, missing price attributes, and empty-pool refusal. Review-driven runbook, ADR, legacy-fixture, and CHANGELOG findings are repaired and observed threads are resolved. Exact-head Rust Foundation CI, Documentation Quality, Security Scan, and SAST Semgrep remain queued; no queued result is GREEN and no qualifying independent approval has been observed.

**#481 — dynamic evaluation drift monitoring design/evidence lane.** TEPP owns temporal composition, availability/knowledge-cutoff semantics, longitudinal/multilevel/multiple-membership structure, drift/change-point evidence, and time-indexed invariance monitoring. It does not own item generation, provider/rater invocation, adjudication, anchor promotion, or reusable static psychometric kernels. Implementation remains blocked until canonical owners publish immutable released/digest-pinned run/item/rater/adjudication/linking contracts. The first eventual slice is a versioned ACL with no-anchor/no-linking fail-closed tests, not a drift score or dashboard. Current mutable sibling PR heads are evidence only and must not become production dependencies.

**#315 — static-standardisation ownership repair.** #315 remains Draft only to preserve TEPP-specific `TIPREDVARstd` naming/event-time/refusal semantics. It must not merge with duplicate reusable arithmetic. After the live fast-mlsirm owner contract is merged and released/versioned, #315 must consume that immutable contract through an ACL, prove parity, retain TEPP temporal/model admission, delete wrong-owner production arithmetic, and fold into a Measurement/Longitudinal Modeling landing vehicle.

**#437 — ADR identity.** Repository-wide ADR IDs are immutable authority. Duplicate index IDs, duplicate targets, duplicate numbered ADR files, repeated authority declarations, and index/file status or maturity drift must fail deterministic documentation fitness tests. Operation-specific ADRs on adapter/model micro-branches are implementation lineage pending normalization, not branch-local architecture authority. #435 contains the current deterministic fitness implementation; issue closure still requires exact-head hosted evidence and a coherent landing decision.

## Dependency and Context Fabric status

The live fast-mlsirm generalized-mixed/dependence compiler remains owner-side candidate work and is not a TEPP production dependency until merged and released/versioned. TEPP consumes its Published Language through an ACL and never copies the static kernel implementation.

TEPP protected main checksum-pins contextual-orchestrator commit `e226e1197bdfc890c9d8e5b9b648c78857d7e465` with SHA-256 `964b22ff577e3862b761af847ccad65489bb3f8fc750c8f84fcf8628df096673`. Upstream protected `main` is `8839081659df587b19642be17b9114f9dee8b666`, which contains the richer `orchestrator/free` route-mode and discovery work absent from the pinned revision. Advancing TEPP still requires a reproducible replacement archive digest and exact replacement review; checksum validation must not be removed or guessed. #480 therefore repairs free-only admission using the current pinned public discovery-price contract rather than silently upgrading the supply-chain pin.

`context-graph-contracts` and `enterprise-architecture-core` remain read-only to this writer while their dedicated Context Fabric owner loop is active. Open heads are candidate evidence, not released contracts. TEPP may maintain fail-closed conformance fixtures behind candidate/test boundaries, but deployable integration and authoritative EA projection require a released/versioned Context Graph artifact plus passing compatibility evidence. TEPP latent estimates, measurement scores, inferred event relations, and validity evidence do not become authoritative EA facts.

## Gap register

| ID | Gap | Maturity | Closure evidence |
| --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 128 open PRs | `release-blocking` | coherent landing vehicles, unique evidence preservation, protected-main reduction |
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
| GAP-016 | hourly LLM bootstrap can admit paid/unknown-price routes | `active-repair` | #480 exact-head GREEN, qualifying review, merge to protected main, issue #479 closure |
| GAP-017 | dynamic evaluation item/rater/anchor drift monitoring | `blocked-external-design` | released/digest-pinned owner contracts, ACL conformance, no-anchor/no-linking refusal, then evidence-gated temporal monitoring |

## Delivery and release order

Queue/ADR/domain authority precedes semantic admission, the Rust CPU f64 shared-latent temporal estimator, durable Analysis Run, released fast-mlsirm temporal composition, event intelligence, accelerator parity, buyer workflows, and finally tenancy/durability/observability/release support. A bounded security/dependency/provider-admission repair may land earlier when it directly closes a fail-closed control defect.

Every open PR is classified as `landing_vehicle`, `stacked_dependency`, `fold_into_landing_vehicle`, `superseded`, `duplicate`, `research_lineage_only`, `blocked_external`, or an explicitly justified independent root-cause repair. Exact heads must be compared before closure. Strict ancestry permits safe consolidation; diverged siblings require a real code/test fold first.

TEPP has no GitHub release at this snapshot. A release is permitted only after a clean integration state reaches protected main with exact protected-head CI/security evidence, claim-scoped scientific and recovery acceptance, reproducible artifacts with SBOM and provenance, validated migrations/upgrade/rollback/recovery where applicable, consistent version metadata and a current `CHANGELOG.md`, accessibility and operability evidence for user-facing components, no unresolved scientific/privacy/security/supply-chain blockers, and released integration contracts where cross-product deployment depends on them.
