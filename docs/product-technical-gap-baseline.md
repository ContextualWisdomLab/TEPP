# Product and Technical Gap Baseline

**Status:** Active delivery recovery  
**Product:** Temporal Event Psychometrics Platform (TEPP)  
**Snapshot:** 2026-09-02T15:10Z  
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`  
**Workspace version:** `0.2.0`  
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)  
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md)

## Delivery truth

A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Only protected-main integration plus current required evidence can establish delivery.

| Signal | Fresh evidence | Implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until main advances. |
| Open pull requests | **131** | Queue increased from 130 when #484 appeared while #435 remained open; WIP circuit breaker is active. |
| Draft pull requests | **130** | Draft work must be consolidated/repaired rather than independently landed. |
| Non-Draft pull requests | **1** | #480 remains the only non-Draft PR and is still non-deployable without a compatible immutable contextual-orchestrator release. |
| Open issues | **16** | Includes ADR normalization, orchestrator admission, and dynamic-evaluation drift design/evidence work. |
| GitHub releases | **0** | No TEPP open head is a released contract. |
| Organization ruleset | `18156473` | One qualifying current-head approval, stale-review dismissal after push, resolved threads, unattributed-change approval where applicable, and central required workflows. |

The queue rose from **130 to 131** because #484 `summarizes_edge_v1` was opened as another Analysis Run profile against `main`. Its changed files overlap the existing Analysis Run landing surfaces (`Cargo.lock`, `crates/analysis_engine/Cargo.toml`, `crates/analysis_engine/src/lib.rs`, TRACEABILITY/ADR/doctoring). It has therefore been non-destructively retargeted to #416 rather than treated as an independent landing authority. The resulting conflict is real shared-file fold work; unique source/tests/doctoring and RED/repair evidence must survive into the eventual #416 head. The PR is not closed merely to reduce the count.

## Priority landing evidence

This is a priority subset, not a row-for-row copy of the 131-PR queue. #435 intentionally omits its own SHA from this file because embedding a branch head inside a file changed by the same branch would make the file self-stale.

| PR | Exact head | Draft | Base | Ownership / disposition |
| ---: | --- | :---: | --- | --- |
| #484 | `9a1be78b5342ff65e3cf2aac1e9331c68943f246` | true | #416 branch @ `0b7155cc238defb1e55129ff3000658f04b343cf` | `summarizes_edge_v1` fold child; future-unavailable evidence is filtered before duplicate-identity admission. |
| #483 | `847d96f913bb261803ac0bd751ad7e4f51324cee` | true | #416 branch @ `0b7155cc238defb1e55129ff3000658f04b343cf` | `retrospective_edge_v1` fold child; unique tests/doctoring must survive. |
| #482 | `506dbae236a4484301b704b6c6a05b20faf0fe69` | true | #416 branch @ `0b7155cc238defb1e55129ff3000658f04b343cf` | `role_contradiction_v1` fold child; unique tests/doctoring must survive. |
| #480 | `4475542750eda01afad0cf9ea8d563f508f63fd3` | false | main | Independent consumer-side LLM-governance repair; requires released CO + HTTPS `orchestrator/free`, deployment provenance and safe gateway auth. |
| #460 | `dfab4eab5ff733731e565a9348072b8dab2e4912` | true | #416 branch @ `0b7155cc238defb1e55129ff3000658f04b343cf` | `relation_absence_v1` fold child; typed cutoff equality and terminal validation separation preserved. |
| #458 | `08165e3b3c929b4ae77396689549f72723ff8ff5` | true | #416 branch @ `0b7155cc238defb1e55129ff3000658f04b343cf` | `outcome_order_v1` fold child; typed cutoff equality and terminal validation separation preserved. |
| #416 | `0b7155cc238defb1e55129ff3000658f04b343cf` | true | main | Validation / Analysis Run landing candidate; availability cutoff precedes duplicate-identity admission. |
| #310 | `406e6ae2b2a8fd99494e9bc82e61ced9b81bffe0` | true | main | Longitudinal Modeling vehicle; CWC stable-mean plus known-truth RMSE intermediate-overflow repairs preserve scientific fail-closed boundaries. |

Exact-head evidence becomes stale after source mutation.

## Domain ownership

Cargo crates, HTTP routes, CLI verbs, refusal rules, clocks, and individual statistical maps are implementation units rather than bounded contexts.

| Subdomain | Bounded context | Aggregate authority | Implementation nucleus |
| --- | --- | --- | --- |
| Core | Evidence & Semantic Measurement | `EvidenceCorpus`, `SemanticUnitSet`, `ConceptDictionaryRevision` | `evidence_core`, `semantic_core` |
| Core | Temporal Semantics | `TemporalEvidenceWindow`, `KnowledgeCutoffPolicy` | temporal primitives and cutoff policy |
| Core | Event Ontology & Temporal Graph | `EventEpisode`, `TemporalRelationSet` | `event_core`, `relation_graph` |
| Core | Measurement | `MeasurementSpecification`, `MeasurementRun` | measurement modules + released fast-mlsirm ACL |
| Core | Longitudinal Modeling | `TemporalModelSpecification`, `TemporalModelRun` | `longitudinal_core` + temporal/event composition |
| Core | Validation Evidence | `ValidationStudy`, `ValidationEvidence` | `validation_core`, `tepp_simulation` |
| Core | Scientific Claim Promotion | `ClaimPromotionDecision` | validation evidence + ADR 0014 policy |
| Supporting | Projection / Analysis Run | `AnalysisRun`, published read models | application services; HTTP/CLI adapters |
| Supporting | Interpretation | evidence-grounded interpretation workflow | contextual-orchestrator ACL only |
| Supporting | Persistence & Recovery | repositories and durable receipts | persistence/object-store adapters |
| Generic | Compute backend | execution receipt | CPU/GPU/MLX adapters; execution is not scientific authority |

fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and arithmetic, including LSIRM/MLSIRM/DLSJM kernels. TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment and temporal recovery. contextual-orchestrator owns provider/model routing and semantic LLM execution. Context Graph contracts are contract-only integration authority; EA Core owns enterprise-architecture decisions. No source copying, mutable sibling dependency, or cross-service SQL.

The clock contract separates event/valid time, assertion time, document time, system time, available time, and knowledge cutoff. Retrospective evidence may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective/citation/revision/provenance relations.

## Scientific invariants

- Rasch remains distinct from generic 1PL; formulation-qualified 2PLM–5PLM, MIRT, ideal-point/GGUM, testlet/rater/facet/generalized-mixed identity is preserved.
- Cross-classification and multiple membership remain distinct; weights are explicit, auditable, time-valid and observed-normalized or model-estimated according to formulation.
- TEPP composes time over the full released upstream candidate identity; auto-expansion never means auto-activation.
- Supported temporal estimators require state/trajectory and claimed-structure recovery, bias/RMSE, interval coverage, convergence, uncertainty calibration and leakage-safe rolling-origin evidence.
- CPU/GPU parity counts only when the relevant accelerator path actually runs. Monte Carlo decisions use simulation uncertainty rather than arbitrary pass percentages.
- Scientific failures are never hidden with skip/xfail/source rewriting/coverage exclusions.

## Current repairs and blockers

### #310 — Longitudinal Modeling

Closed predecessor #441 is contained by #310. The invalid covariance/earlier-variance quantity is not exposed as autocorrelation; lagged Pearson correlation requires lagged covariance and both occasion-specific marginal variances. Temporal/state composition is owned by `longitudinal_core`.

Current head `406e6ae2b2a8fd99494e9bc82e61ced9b81bffe0` preserves prior stationary-variance, full-range cancellation, log-domain same-sign ratio, one-numerical-authority, and CWC stable-mean repairs. CWC RED `9a706c3c0e9e0db68e88f89b94c64c13ea7fafd0` fixed the counterexample `[0.75·MAX, 0.75·MAX, -0.5·MAX]`; repair `260413efb9d95039b5fbba41919cba8097fcf8b5` reuses the Longitudinal Modeling `scaled_compensated_mean` so representable means/residuals are not rejected by a raw partial-sum overflow.

Fresh review then found the same class of defect one stage later in known-truth recovery. `component_root_mean_square_error` scaled the sum of squared residuals only after computing each raw `decided - truth`. For four matched components with one `truth=-MAX, decided=MAX` pair and three zero residuals, that individual subtraction overflows even though the exact aggregate RMSE is `MAX` and therefore representable.

RED `d72ba2b6e28909c6def73a2638ebd63258dec500` pins that four-row recovery case. Causal repair `406e6ae2b2a8fd99494e9bc82e61ced9b81bffe0` validates finite matched identities first, normalizes truth and decided values by the largest absolute component magnitude before subtraction, computes the bounded normalized mean square/root, and scales only the final RMSE back to component units. The representable four-row case returns `MAX`; the one-row `-MAX → MAX` case remains fail-closed because its final RMSE is genuinely non-representable. Component identity, failure denominator, and scientific acceptance semantics are unchanged.

Hosted evidence for this exact head is not GREEN. CodeQL PR run `33646689205` completed as `startup_failure`; OSV-Scanner PR, Rust Foundation CI, Scorecard PR, Security Scan, SAST Semgrep and Documentation Quality are queued. No qualifying current-head independent approval exists. Predecessor-head results do not transfer and the central CodeQL startup failure does not authorize bypass.

### #416 — Validation / Analysis Run consolidation

Current head `0b7155cc238defb1e55129ff3000658f04b343cf` centralizes the leakage-safe invariant established by RED `ffee655404716bf8d33c898a3c1a87a543abe701`: availability filtering occurs before duplicate-identity admission. #458/#460/#482/#483/#484 are comparison/fold children over shared Cargo/lib/lock/docs surfaces. Their unique evidence must be merged into a surviving #416 head before any child can be considered fully superseded.

### #480 / #479 — LLM owner boundary

#480 removes TEPP-owned provider discovery/ranking and requires HTTPS `contextual-orchestrator/orchestrator/free` from an immutable owner release. contextual-orchestrator protected `main@464da4715b495b5eaaa593eba3796e2d976ee0c9` still has **zero GitHub releases**, so the consumer remains deliberately non-deployable. Do not fall back to mutable source, direct provider calls or guessed checksums.

### #315 / fast-mlsirm owner handoff

TEPP-specific static-standardisation lineage must not become a second reusable arithmetic authority. fast-mlsirm protected `main@b5a3a0c1057d4b53d7a4bb18e0de69f630c2b45c`; generalized-mixed/dependence compiler #1714 is Ready/mergeable at `92a3f2152033b61ca89661b5ba8a584842e8c3a9`. Latest immutable release is `v0.9.1` (2026-08-26), which predates #1714. TEPP may adopt the new Published Language only after a compatible immutable release and ACL/parity evidence. Current #1714 exact-head evidence is non-GREEN: CI is pending; CodeQL PR is startup-failed; OSV, Scorecard, Semgrep, CodeQL and Security Scan are queued.

### #437 — ADR identity

Repository-wide ADR IDs are immutable authority. Duplicate index IDs/targets/numbered files, index/file mismatch and repeated Decision-status or Implementation-maturity authority must fail deterministic documentation fitness. Adapter/model micro-slice ADR numbers remain implementation lineage pending normalization through #435, not branch-local architecture authority.

## External contract state

`context-graph-contracts` protected/default `develop@99cb5468ba3c15c5e79688f53dee74724fae2d13` has **14 open PRs, 2 open issues, and zero GitHub releases**. Its new release-source/provenance work remains Draft/unreleased evidence. `enterprise-architecture-core` protected/default `develop@1c0fa8b15ceb9e72186274aeb255d6777eb84ef4` has **24 open PRs, 2 open issues, and zero releases**. Both remain read-only from this TEPP writer; open heads are candidate evidence rather than production contracts. TEPP deployable integration and authoritative EA projection require released/versioned contract artifacts plus compatibility/provenance evidence.

TEPP latent estimates, measurement scores, inferred event relations and validity evidence are not authoritative EA facts.

## Gap register

| ID | Gap | Maturity | Closure evidence |
| --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 131 open PRs | `release-blocking` | coherent landing vehicles, unique-evidence preservation, protected-main reduction |
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
| GAP-014 | current required-workflow startup/runner evidence unavailable | `external-control-risk` | central workflow repair, exact-current required workflows GREEN, no bypass |
| GAP-015 | contextual-orchestrator has no immutable released contract for current owner behavior | `release-blocking` | compatible immutable CO release, deployment provenance, safe gateway auth, exact TEPP ACL adoption |
| GAP-016 | hourly LLM path needs released owner-only routing/authentication | `active-repair` | #480 + released CO adoption + exact-head GREEN/review/main merge |
| GAP-017 | dynamic evaluation item/rater/anchor drift monitoring | `blocked-external-design` | released/digest-pinned owner contracts, ACL conformance, no-anchor/no-linking refusal, evidence-gated temporal monitoring |

## Release gate

TEPP currently has no GitHub release. Release is permitted only after a coherent vertical reaches protected main with exact protected-head CI/security/recovery evidence, reproducible package + SBOM + provenance + rollback artifacts, current version/CHANGELOG, required migration/restore/load evidence, and released integration contracts where deployment depends on them. Queued/pending/startup-failed/skipped or predecessor-head evidence is not GREEN.