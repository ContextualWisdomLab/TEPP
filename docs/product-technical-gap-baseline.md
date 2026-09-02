# Product and Technical Gap Baseline

**Status:** Active delivery recovery

**Product:** Temporal Event Psychometrics Platform (TEPP)

**Snapshot:** 2026-09-02T19:02:00Z

**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`

**Workspace version:** `0.2.0`

**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md).

**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md).

## Delivery truth

A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Only protected-main integration plus current required evidence establishes delivery.

| Signal | Fresh evidence | Implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until main advances. |
| Open pull requests | **132** | WIP circuit breaker remains active; #485 caused a fresh regression and was retargeted into #416 rather than accepted as an independent landing lane. |
| Draft pull requests | **131** | Draft work must consolidate/repair rather than independently land. |
| Non-Draft pull requests | **1** | #480 is the only non-Draft PR and is not deployable without a compatible immutable contextual-orchestrator release. |
| Open issues | **16** | ADR normalization, orchestration admission, evaluation drift and scientific recovery work remain open. |
| GitHub releases | **0** | No TEPP open head is a released contract. |
| Organization ruleset | `18156473` | One qualifying current-head approval, stale-review dismissal after push, resolved threads, unattributed-change approval where applicable, and central required workflows. |

#484 `summarizes_edge_v1` and #485 `support_edge_v1` remain #416 Analysis Run fold children. Their unique source/tests/doctoring must survive the shared-file fold; neither is closed merely to reduce queue count.

## Current priority open pull-request evidence

#435 intentionally omits its own SHA from this file because embedding a branch head inside a file changed by that branch makes the file self-stale.

| PR | Exact current head | Draft | Base | Disposition |
| ---: | --- | :---: | --- | --- |
| #485 | `f71591864efc2beff336ced7ef35d5a013305c36` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Analysis Run fold child after fresh WIP regression; preserve support-edge refusal/source/tests/doctoring and historical-cutoff evidence. |
| #484 | `9a1be78b5342ff65e3cf2aac1e9331c68943f246` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Analysis Run fold child; preserve profile-specific source/tests/doctoring. |
| #483 | `847d96f913bb261803ac0bd751ad7e4f51324cee` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Analysis Run fold child; preserve unique evidence. |
| #482 | `506dbae236a4484301b704b6c6a05b20faf0fe69` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Analysis Run fold child; preserve unique evidence. |
| #480 | `4475542750eda01afad0cf9ea8d563f508f63fd3` | false | `main@1bc02f580cf48e1d39da239f0e818453437c31c3` | Independent consumer-side LLM-governance repair; released CO plus deployment/auth provenance required. |
| #460 | `dfab4eab5ff733731e565a9348072b8dab2e4912` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Fold child; typed cutoff equality and terminal-validation separation preserved. |
| #458 | `08165e3b3c929b4ae77396689549f72723ff8ff5` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Fold child; typed cutoff equality and terminal-validation separation preserved. |
| #416 | `0b7155cc238defb1e55129ff3000658f04b343cf` | true | `main@1bc02f580cf48e1d39da239f0e818453437c31c3` | Validation / Analysis Run landing candidate; availability cutoff precedes duplicate-identity admission. |
| #310 | `7f0bea0841fb89a6ce9bdd5b9f10c0e4612f4270` | true | `main@1bc02f580cf48e1d39da239f0e818453437c31c3` | Longitudinal Modeling vehicle; one canonical irregular-rate function identity is re-exported through the stable facade; exact-head hosted verification is pending. |

Exact-current-head evidence becomes stale after source mutation or any new commit.

## Domain ownership

TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment, leakage-safe knowledge cutoff, temporal recovery and projection policy. fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and arithmetic, including covariance-to-correlation standardization and LSIRM/MLSIRM/DLSJM kernels. contextual-orchestrator owns provider/model routing and semantic LLM execution. Context Graph contracts are contract-only integration authority; EA Core owns enterprise-architecture decisions. No source copying, mutable sibling dependency, or cross-service SQL.

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

The lagged-correlation representability repair remains intact. RED `c345ee7b8bdf642430669b7b0e1d7fc6873a84af` pins finite inputs `cov=min_subnormal`, `Var_t=Var_t+Δ=f64::MAX`: the covariance satisfies the exact binary64 Cauchy–Schwarz bound, but the true standardized magnitude is below the smallest representable binary64 and the predecessor returned `Ok(0.0)`, falsely converting a nonzero association into exact no-association. Repair `5785e07a352801c193d92dde03863d0697a2853a` rejects that representability collapse as `InvalidTemporalAssociationInput` while preserving genuine zero covariance as `Ok(0.0)`.

The current exact head `7f0bea0841fb89a6ce9bdd5b9f10c0e4612f4270` also removes a redundant public wrapper layer around the canonical irregular-rate implementation. An initial test commit `5697221872650f0327cbdf6db0a5020a07fe39e4` over-specified canonical visibility as `pub(crate)`; before production mutation, RED refinement `4648638608436fb6c04315d96f59a6404e2e790b` corrected the contract to preserve re-exportable canonical `pub fn` identities while forbidding facade wrapper `pub fn` declarations. Repair `7f0bea0841fb89a6ce9bdd5b9f10c0e4612f4270` replaces the wrappers with direct `pub use` re-exports. `lib.rs` retains the same crate-public names through `stable_irregular_rate`, so the API remains compatible while one function identity owns the numerical behavior.

The earlier stationary-overflow documentation repair remains intact. RED `9d8a82d78443cafc9b5064fc3bb35aa3f2052722` rejects the retired `(q / a) * -0.5` overflow instruction and requires `(q * 0.5) / |a|`; repair `9c962205dca26925c2e60d1e15ec4ce15681bbee` synchronizes `CLAUDE.md` with production `recover_stationary_within_variance` behavior. Earlier CWC, within/between, known-truth RMSE, irregular-rate zero-underflow, stationary-subnormal and exact covariance-bound lineages also remain on the same vehicle.

The current head is mergeable but remains Draft. Exact-head CodeQL PR run `33670670642` is `startup_failure`; Documentation Quality `33670668067`, SAST Semgrep `33670668039`, OSV-Scanner PR `33670669157`, Security Scan `33670667891`, Scorecard PR `33670667957`, and Rust Foundation CI `33670667997` are queued. There is no qualifying current-head independent approval. Source repair is complete, but protected-main integration waits for fresh exact-head GREEN evidence and review.

The CWC and within/between unit-mean helpers now agree on the material extreme-cancellation behavior but are still separate implementations. Consolidation remains a maintainability target only after their full error/estimand semantics are shown equivalent; a reusable domain-neutral arithmetic primitive belongs in fast-mlsirm rather than being copied across TEPP contexts.

### #416 — Validation / Analysis Run consolidation

Current head `0b7155cc238defb1e55129ff3000658f04b343cf` centralizes the leakage-safe invariant established by RED `ffee655404716bf8d33c898a3c1a87a543abe701`: availability filtering occurs before duplicate-identity admission. #458/#460/#482/#483/#484/#485 remain fold children over shared Cargo/lib/lock/docs surfaces. Their unique evidence must reach a surviving #416 head before any child can be considered fully superseded. #485 was opened as an independent main-targeting profile despite the active WIP circuit breaker; fresh file-surface verification showed it touches the same Analysis Run integration files, so it was repaired by non-force retargeting to #416.

### #480 / #479 — contextual-orchestrator boundary

#480 removes TEPP-owned provider discovery/ranking and requires HTTPS `contextual-orchestrator/orchestrator/free` from an immutable owner release. contextual-orchestrator protected `main@212ff437dc297613289dba2e6064ade9942e07d8` has **zero GitHub releases** at this snapshot. Mutable branch state is not a released contract. The consumer remains deliberately fail-closed. Owner issue #1023 remains open for authenticated release/deployment provenance and scoped/brokered authentication that does not expose a reusable long-lived gateway bearer token to model-controlled execution.

### fast-mlsirm owner handoff

fast-mlsirm protected `main@b5a3a0c1057d4b53d7a4bb18e0de69f630c2b45c`. Immutable `v0.9.1` predates the current owner work.

Generalized-mixed/dependence Published Language #1714 is open/Ready at `92a3f2152033b61ca89661b5ba8a584842e8c3a9`. Reusable static covariance-to-correlation standardization #1722 is open/Ready at `338dbb2d25f32b0e201102e7bf73076846fb57b3`. Dynamic evaluation criterion/item/run snapshots #1727 remain Draft at `17ac218a5c0a31f0082839603a8ffc64b4432534`; zero-anchor within-run work is explicit, while unsupported cross-version linking fails closed. None of these mutable owner heads is a TEPP production dependency. Temporal/EventTime admission and state composition remain TEPP-owned.

### #437 — ADR identity

Repository-wide ADR IDs are immutable authority. Duplicate index IDs/targets/numbered files, index/file mismatch and repeated Decision-status or Implementation-maturity authority must fail deterministic documentation fitness. Adapter/model micro-slice ADR numbers, including #485's ADR 0078, remain implementation lineage pending normalization through #435.

## External contract state

`context-graph-contracts` and `enterprise-architecture-core` remain read-only from this TEPP writer. Fresh state is CGC `develop@99cb5468ba3c15c5e79688f53dee74724fae2d13`, 14 open PR / 2 open issue / release 0, and EA Core `develop@1c0fa8b15ceb9e72186274aeb255d6777eb84ef4`, 24 open PR / 2 open issue / release 0. CGC #25 is still a Draft source-bound release-provenance prerequisite; EA #40 remains Draft/fail-closed against unreleased CGC evidence. Open heads are candidate evidence rather than production contracts.

## Gap register

| ID | Gap | Maturity | Closure evidence |
| --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 132 open PRs | `release-blocking` | coherent landing vehicles, unique-evidence preservation, protected-main reduction |
| GAP-002 | multilingual span-grounded semantic/concept admission | `partial` | immutable offsets/layout, KO/EN/JA/ZH/VI/ES/DE/FR profiles, concept dictionary, invariance/calibration, hostile-input tests |
| GAP-003 | shared-latent temporal topic estimator | `partial` | Rust CPU f64 likelihood/uncertainty, relation/time/membership effects, true recovery, fitted candidate-K |
| GAP-004 | durable end-to-end Analysis Run | `partial` | idempotent lifecycle, persistence/recovery, estimator-bound artifacts, validation/promotion separation, Compose E2E |
| GAP-005 | temporal psychometric composition/duplication | `partial` | released fast-mlsirm contracts, TEPP ACLs, temporal recovery, wrong-owner static kernels removed after parity |
| GAP-006 | event intelligence | `partial` | calibrated detection/tracking/schema/interval recovery and durable artifacts |
| GAP-007 | accelerator/memory evidence | `accepted-target` | real hardware, CPU-f64 parity, bounded OOM/fallback evidence |
| GAP-008 | network/cluster buyer workflow | `partial` | known-truth recovery, uncertainty/stability, repeated consensus, exact-value export |
| GAP-009 | production interpreter/verifier | `partial` | released contextual-orchestrator execution, evidence citations, independent verification, abstention/fallback |
| GAP-010 | accessible buyer UI | `accepted-target` | Figma/Storybook, locale-specific CJK/text expansion/font fallback, keyboard/touch/loading/empty/error/permission states, exact-value provenance |
| GAP-011 | operable multi-tenant release | `accepted-target` | OIDC/RLS/purpose controls, durable queue/storage, OTel/SLO, restore/load/migration, signed SBOM/provenance |
| GAP-012 | paths obscure domain ownership | `active-refactor` | staged moves, ACLs, no cycles/cross-context persistence/shared-kernel creep |
| GAP-013 | ADR identity collisions | `release-integrity` | unique repository-wide identity, deterministic duplicate detection, supersession lineage |
| GAP-014 | current required-workflow startup/runner evidence unavailable | `external-control-risk` | central workflow repair, exact-current required workflows GREEN, no bypass |
| GAP-015 | contextual-orchestrator lacks immutable released contract for current owner behavior | `release-blocking` | compatible immutable CO release, deployment provenance, safe gateway auth, exact TEPP ACL adoption |
| GAP-016 | hourly LLM path needs released owner-only routing/authentication | `active-repair` | #480 + released CO adoption + exact-head GREEN/review/main merge |
| GAP-017 | dynamic evaluation item/rater/anchor drift monitoring | `owner-contract-active` | released/digest-pinned dynamic criterion/item/run contract, ACL conformance, no-anchor/no-linking refusal, evidence-gated temporal monitoring |
| GAP-018 | Longitudinal stable-mean logic remains duplicated | `active-refactor` | semantic-equivalence proof, one TEPP Longitudinal primitive or released fast-mlsirm generic owner contract, recovery parity |
| GAP-019 | Longitudinal scientific instructions contradict current stationary-overflow implementation | `verification-pending` | RED `9d8a82d...` + repair `9c962205...`; exact-head Documentation Quality/review GREEN and protected-main integration |
| GAP-020 | Nonzero lagged covariance can be misreported as exact-zero correlation when the standardized magnitude is unrepresentable | `verification-pending` | RED `c345ee7b...` + repair `5785e07a...`; exact-head Rust/documentation/review GREEN and protected-main integration |
| GAP-021 | Longitudinal irregular-rate facade duplicated public wrapper function identities over one canonical implementation | `verification-pending` | RED refinement `464863860...` + repair `7f0bea084...`; exact-head Rust/documentation/review GREEN and protected-main integration |

## Release gate

TEPP currently has no GitHub release. Release is permitted only after a coherent vertical reaches protected main with exact protected-head CI/security/recovery evidence, reproducible package + SBOM + provenance + rollback artifacts, current version/CHANGELOG, required migration/restore/load evidence, and released integration contracts where deployment depends on them. Queued/pending/startup-failed/skipped or predecessor-head evidence is not GREEN.