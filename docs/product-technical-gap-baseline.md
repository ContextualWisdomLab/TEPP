# Product and Technical Gap Baseline

**Status:** Active delivery recovery  
**Product:** Temporal Event Psychometrics Platform (TEPP)  
**Snapshot:** 2026-09-04T11:09:23Z  
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`  
**Workspace version:** `0.2.0`

**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md).

## Delivery truth

A planning document, mergeable branch, local/source inspection, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Only protected-main integration plus current required evidence establishes delivery.

| Signal | Fresh evidence | Implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until main advances. |
| Open pull requests | **134** | WIP circuit breaker remains active; consolidate into existing bounded-context vehicles. |
| Draft pull requests | **134** | Every current open PR is Draft. |
| Non-Draft pull requests | **0** | No PR is eligible for normal merge until deliberately made Ready after exact-head evidence. |
| Open issues | **16** | ADR normalization, orchestration admission, evaluation drift, and scientific recovery remain open. |
| GitHub releases | **0** | No TEPP open head is a released contract. |
| Organization ruleset | `18156473` | One qualifying current-head approval, stale-review dismissal after push, resolved threads, extra approval for unattributed changes where applicable, and central required workflows. |

Ruleset `18156473` permits merge/squash and prohibits deletion/non-fast-forward updates on the default branch. Organization-admin bypass is not normal delivery evidence and is not used by this writer.

## Current priority open pull-request evidence

#435 intentionally omits its own mutable branch SHA from this file.

| PR | Exact current head | Draft | Base | Disposition |
| ---: | --- | :---: | --- | --- |
| #488 | `daa097861342ac14a7f0553c8d565b75cb6131fd` | true | `main` | Validation Evidence landing vehicle. Wilson provenance/count and rounding repairs remain inherited. `accept_within_standard_errors` now preserves a finite represented `k · SE` bound before scale normalization so a large estimate/target scale cannot erase positive scientific tolerance by underflow; overflow-only normalization remains for the both-overflow case. Hosted exact-head gates and independent review remain required. |
| #487 | `e07b2ff9f78ef456ff911b8643710af20921fe54` | true | #416 | Validation / Analysis Run fold child. Unique evidence must be inherited by a conflict-resolving survivor; child-head CI does not transfer. |
| #485 | `f71591864efc2beff336ced7ef35d5a013305c36` | true | #416 | Analysis Run fold child; preserve support-edge source/tests/doctoring. |
| #484 | `9a1be78b5342ff65e3cf2aac1e9331c68943f246` | true | #416 | Analysis Run fold child; preserve summarizes-edge source/tests/doctoring. |
| #483 | `847d96f913bb261803ac0bd751ad7e4f51324cee` | true | #416 | Analysis Run fold child; preserve unique evidence. |
| #482 | `506dbae236a4484301b704b6c6a05b20faf0fe69` | true | #416 | Analysis Run fold child; preserve unique evidence. |
| #480 | `01f45a99392457334a4f6d3d659f992af739eeee` | true | `main` | contextual-orchestrator consumer repair; stays Draft while immutable CO release/deployment/auth provenance is unavailable. |
| #460 | `dfab4eab5ff733731e565a9348072b8dab2e4912` | true | #416 | Analysis Run fold child; typed cutoff equality and terminal-validation separation. |
| #458 | `08165e3b3c929b4ae77396689549f72723ff8ff5` | true | #416 | Analysis Run fold child; typed cutoff equality and terminal-validation separation. |
| #416 | `aa730c63563eb4a33048d822b581036c8487bd47` | true | `main` | Validation / Analysis Run landing vehicle. Generic cutoff-before-identity repair remains in ancestry. |
| #310 | `c6680450152b1e0a2c9abb553772d74a23923335` | true | `main` | Longitudinal Modeling vehicle. Stable `Between` recovery is unit-level with canonical occasion `0`; `Within` retains actual occasion identity. |

Exact-head evidence becomes stale after any source push.

## Domain ownership

TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment, leakage-safe knowledge cutoff, temporal recovery, Validation Evidence, and Projection policy. `longitudinal_modeling` is the bounded context and `longitudinal_core` is its current Rust implementation path.

`psychometric_core` is not authority for new temporal/state composition. fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and arithmetic. TEPP consumes only immutable released/versioned Published Language through an ACL; source copying and mutable sibling-head dependencies are prohibited. Fresh owner evidence is fast-mlsirm protected `main@b5a3a0c1057d4b53d7a4bb18e0de69f630c2b45c`; latest immutable release remains `v0.9.1`.

contextual-orchestrator owns provider/model routing and semantic LLM execution. Fresh owner evidence is protected `main@2e414d15ba58f28597751b625a8a2f00fc9fadcf`, while GitHub releases remain zero. Semantic LLM work and model-backed Actions therefore continue to require a released/versioned contextual-orchestrator contract; Actions use `orchestrator/free` through the gateway credential and do not select providers/models/groups or consume direct provider keys. Mutable owner main is evidence, not a TEPP production contract. Context Graph contracts are contract-only integration authority; EA Core owns enterprise-architecture decisions. No cross-service SQL.

The clock contract separates event/valid time, assertion time, document time, system time, availability time, and knowledge cutoff. Retrospective evidence may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective/citation/revision/provenance relations.

## Scientific invariants

- Rasch remains distinct from generic 1PL; formulation-qualified 2PLM–5PLM, MIRT, ideal-point/GGUM, testlet/rater/facet/generalized-mixed identity is preserved.
- A nominal unit identifier is not repeated-measures evidence; singleton units cannot satisfy a longitudinal multilevel floor.
- Occasion-mean deviations `p_it = x_it - μ_t` are not CWC residuals, sample-wide grand-mean residuals, or RI-CLPM within-person effects. Numeric event time defines occasion identity, so `-0.0` and `+0.0` are one occasion.
- Row arrival order is not scientific evidence. Fixed admitted observations must produce bit-identical means/centered results under permutation wherever the contract claims deterministic CPU `f64` reference behavior.
- Known-truth component recovery follows bounded-context identity: stable `Between` is unit-level and uses canonical `occasion_index = 0`; `Within` retains actual `(unit, occasion)` identity. Duplicate identities or stable-Between occasion aliases are not weights.
- A representable final scientific estimand is not rejected solely because an avoidable intermediate binary64 operation overflows/underflows. False exact 0/1/non-finite endpoints, erased finite bounds, and cancellation residues are not accepted when an equivalent operation order preserves the represented estimand or decision.
- Mean signed bias and bias SE remain Validation Evidence performance measures over admitted recovery units; representable cancellation/SEM must survive avoidable overflow/underflow while mathematically nonzero finals below binary64 range fail closed.
- Generic Validation Evidence RMSE and its delta-method SE normalize before squaring. Exact-zero point RMSE requires exact-zero point RMSE SE. For positive RMSE, the declared producer satisfies `SE(RMSE) <= RMSE / 2`, with a small relative binary64 admission tolerance.
- Generic `MonteCarloSummary` is sign-neutral. Zero SD requires zero SE and percentile endpoints equal to the represented mean. Positive SD requires positive SE coherent with represented `SD / sqrt(n)` within a small relative tolerance. Every inclusive nearest-rank endpoint is a retained observation and satisfies `|endpoint - mean| <= SD * sqrt(n - 1)`. Distinct lower/upper endpoint values share the same `(n - 1) * SD²` deviation budget. When `replication_count = 2` and endpoints are numerically distinct, those values exhaust the retained sample and the recorded represented mean/sample SD must match deterministic two-value reconstruction. Equal numeric endpoints do not prove two distinct ranks and are not subjected to that exhaustion rule.
- When the generic carrier occupies `ValidationReport::monte_carlo_rmse`, retained replications are nonnegative; positive mean implies `SE(mean) <= mean`, and inclusive nearest-rank endpoints cannot exceed nonnegative sample-sum support `n * mean`. These typed bounds are not imposed on signed metrics such as bias.
- SE-aware acceptance is `|estimate - target| <= k * SE`. For `k = 0` or exact-zero SE, exact recovery is evaluated before any scale reduction; `-0.0` and `+0.0` are one numeric zero-valued scientific state. When the represented residual and represented `k · SE` bound are finite, they are compared directly so `SE / scale` cannot underflow a positive admissible bound to zero. Scale normalization is reserved for overflow handling.
- `match_count` is a finite-threshold decision and does not require materializing an unrepresentable absolute residual; `absolute_residuals` remains fail closed when the magnitude itself is requested.
- Wilson interval coverage preserves represented-input endpoints through stable rationalized/complementary/compensated forms. Legacy `ValidationReport` endpoint pairs satisfy necessary Wilson coherence and boundary checks, but exact provenance is not inferred from them. `WilsonCoverageEvidenceV1` retains fixed-width `u64` `sample_count`/`covered_count`, `critical_value_kind=standard_normal_z`, `interval_sidedness=two_sided`, numeric `z`, represented coverage, and canonical endpoints and validates them by exact recomputation through the single crate-private Wilson authority. Exact all-covered status is integer count equality, not rounded `p == 1`. Count proportions are correctly rounded from exact integers. When covered counts dominate, the smaller uncovered complement is evaluated and reflected. When retained `sample_count` itself is not exactly representable in binary64, Wilson scale terms use correctly rounded reciprocal `1/n`; its all-covered path uses complementary miss mass for `z²/n <= 1` and direct reciprocal evaluation for `z²/n > 1`. For an exactly representable all-covered count, a false exact-one quotient retains the boundary-local complementary repair. Otherwise, if `n + z²` is inexact, the canonical writer recovers the TwoSum denominator residual and FMA quotient residual, identifies the adjacent binary64 candidate from residual direction, and changes the direct quotient only when the exact represented-input rational lies beyond that candidate's midpoint; exact midpoint selection is ties-to-even. This avoids both under-correction and residual double-rounding overcorrection without claiming globally correctly rounded Wilson rearrangements. Smaller miss mass genuinely below binary64 resolution remains exact one. `ValidationEvidenceV1` binds provenance to the legacy report projection without breaking existing report callers.
- Durable and human-facing Validation Evidence must preserve producing-metric invariants on explicit validation, serde ingress/egress, and human projection. Missing v1 provenance is never synthesized from endpoint algebra.
- Bias, RMSE, and Monte Carlo may share private deterministic scalar support inside `validation_core`, but metric-specific cancellation/normalization, denominator, uncertainty, and fail-closed semantics remain with their bounded modules.
- Historical-cutoff admission occurs before duplicate-identity checks. Future-unavailable evidence cannot change an earlier run's conflicts, counts, or terminal state.
- Supported temporal estimators require state/trajectory and claimed-structure recovery, bias/RMSE, interval coverage, convergence, uncertainty calibration, reproducibility, and leakage-safe rolling-origin evidence.
- CPU/GPU parity counts only when the relevant accelerator path actually runs. Scientific failures are never hidden with skip/xfail/source rewriting/coverage exclusions.

## Current repairs and blockers

### #310 — Longitudinal Modeling

#310 remains the canonical Longitudinal Modeling landing vehicle at `c6680450152b1e0a2c9abb553772d74a23923335`. Current-head Rust/documentation/security/review evidence remains required before Ready/merge; predecessor or child-head evidence does not transfer.

### #488 — Validation Evidence

#488 is the generic Validation Evidence landing vehicle at exact head `daa097861342ac14a7f0553c8d565b75cb6131fd`. It remains distinct from Longitudinal Modeling and does not consume mutable fast-mlsirm source.

GAP-070 and GAP-072 remain inherited: stored Wilson endpoints must be one coherent score interval, and exact all-covered evidence cannot use a zero lower endpoint. GAP-071 has advanced from design-only to source implementation. Corrected carrier RED `6f6e06d2...` requires a versioned denominator/critical-value carrier; `ca517ed3...` establishes one private covered-count/Wilson-from-counts authority; `31e1ab2b...` adds `WilsonCoverageEvidenceV1`; `e9c63926...`/`fdd24a1a...` bind standard-normal/two-sided semantics. Envelope RED `07766cb0...` and repair `a16f22e6...` add `ValidationEvidenceV1`, which cross-validates the legacy report projection against the recomputable carrier.

GAP-073 retains large-count identity/schema and exact-ratio projection. GAP-074 retains reciprocal-scale evaluation for sample counts that are not exactly representable in binary64. GAP-075 retains the inexact-count all-covered extreme-`z` scale switch and exact-oracle correction.

GAP-076 records the exact-count all-covered near-one denominator-absorption defect. Public RED `e0c4ec81bb455d230259489dc71e23fe33704b1d` fixes `n=1` and represented `z=0x1.0000000000001p-27`; causal repair `c9dcb9df363999bbcbb6fffdc8b6a6d9ae5e762c` performs the complementary miss calculation only after direct false-one collapse; boundary `6140080d2257d0550be479d71371d70e2255c3d0` preserves true exact one below binary64 resolution; changelog `09ebb482851fe7836e738a74395e6424621da9bf`; research predecessor `fe89c43803136ab979912fffa636d2a4f169a73e`.

GAP-077 records the ordinary partial-denominator state left after the near-one and complete-absorption repairs. With `n=1` and represented `z=3*2^-28`, represented `z²=9*2^-56`; `1+z²` rounds upward before division, so predecessor direct evaluation emits `0x1.ffffffffffffep-1` although the represented-input rational lower endpoint rounds to `0x1.fffffffffffffp-1`. Public RED `06e556538e171e675c4d8a8287d75052ffc2c4c3` → causal TwoSum/FMA residual repair `6c084dbe607e6c415288c77fa41a4270947cd51e` → research `25cc19436085881602356e7f2609b697b575540c` → dedicated changelog `5acc894b8a4d42cd7af8cdc22a02b61a063f16aa` → predecessor changelog correction `f89e36d1f2a048befb983327c83f5696baf530cc` → correct-direct-rounding control `80a0a0ade16f516bf63907f2a8d5105dbcd9c438`.

GAP-078 records the opposite residual-rounding state. With `n=3`, represented `z=0x1.6a09e667f3bcdp+492`, and represented `z²=0x1.0000000000001p+985`, the rounded denominator fully absorbs `n`, but the exact represented-input rational and the direct rounded-denominator quotient both round to `0x1.7ffffffffffffp-984`. The predecessor additive residual correction forced `0x1.7fffffffffffep-984`, one ULP too low. Public RED `d076d344cb3cd3a768e7a5c2d8d7bd9039c657e9` → midpoint-selection repair `32314239754204158f228ec67a0771abf4d39b45` → changelog `a75940c955633d5eb92f227d79e64acd0ef46ea8` → research predecessor `ef6441f786d64e6dca198d0d6140838c391f0659`.

GAP-079 records SE-aware acceptance scale-first underflow. For `estimate=1.0e308`, `target=next_down(estimate)`, `SE=2.2e-16`, and `k=1.0e308`, the represented residual (`1.99584030953472e292`) is within the represented finite bound (`2.2e292`), but the predecessor formed `SE / scale == 0.0` before multiplication and falsely rejected the recovery. Public RED `76067efce44419c27687b8673cb76092c90fb5a5` includes the admissible case and a nearby rejection at `SE=1.8e-16`; causal repair `4ffdf3665b3bbafd3b0bbf06b599fe71498169ab`; changelog `d822c0ba7fa548c3283e462323c56d8f5705de31`; research/current #488 `daa097861342ac14a7f0553c8d565b75cb6131fd`.

These repairs advance the source-contract portion of GAP-071/GAP-073/GAP-074/GAP-075/GAP-076/GAP-077/GAP-078/GAP-079 but not delivery: exact-head hosted gates, independent review, protected-main integration, and adoption by durable consumers that require v1 provenance remain outstanding. Bare legacy `ValidationReport` remains a backward-compatible compact projection and is not silently treated as v1 provenance.

Wilson (1927) remains the primary score-interval reference. AERA/APA/NCME (2014) remains the current published testing-standards edition while revision is underway. IEEE 754-2019 and ISO/IEC 60559:2020 remain the published floating-point authorities used for binary64 representation trace; active IEEE P754 is not treated as a published replacement.

### #416 — Validation / Analysis Run consolidation

#416 branch head is `aa730c63563eb4a33048d822b581036c8487bd47`; its body distinguishes current exact source repair from later metadata/docs commits. #458/#460/#482/#483/#484/#485/#487 remain fold children until every unique source/test/fixture/contract/doctoring delta is inherited by a conflict-resolving survivor. Child-head CI never transfers to #416.

### #480 — contextual-orchestrator boundary

#480 removes TEPP-owned provider discovery/ranking and requires a released contextual-orchestrator owner contract. Its exact source head remains `01f45a99392457334a4f6d3d659f992af739eeee`. contextual-orchestrator protected main is `2e414d15ba58f28597751b625a8a2f00fc9fadcf`, but GitHub releases remain zero, so #480 stays Draft until an immutable compatible release plus deployment/auth/schema/artifact provenance exists. Mutable owner main is evidence, not a production contract.

## Gap register

| ID | Gap | Maturity | Closure evidence |
| --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 134 open PRs | `release-blocking` | coherent landing vehicles, unique-evidence preservation, protected-main reduction |
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
| GAP-016 | hourly LLM path needs released owner-only routing/authentication | `active-repair` | #480 Draft + released CO adoption + exact-head GREEN/review/main merge |
| GAP-017 | dynamic evaluation item/rater/anchor drift monitoring | `owner-contract-active` | released/digest-pinned dynamic criterion/item/run contract, ACL conformance, no-anchor/no-linking refusal, evidence-gated temporal monitoring |
| GAP-018 | Longitudinal stable-mean logic had a separate decomposition implementation after CWC/occasion consolidation | `verification-pending` | RED `7dc87aa8...` + repair `97c8ad35...`; exact-head GREEN/review/main integration |
| GAP-019 | Longitudinal scientific instructions contradicted stationary-overflow implementation | `verification-pending` | RED `9d8a82d...` + repair `9c962205...`; exact-head documentation/review GREEN and protected-main integration |
| GAP-020 | Nonzero lagged covariance can be misreported as exact-zero correlation when standardized magnitude is unrepresentable | `verification-pending` | RED `c345ee7b...` + repair `5785e07a...`; exact-head integration |
| GAP-021 | Longitudinal irregular-rate facade duplicated public wrapper identities over one canonical implementation | `verification-pending` | RED `464863860...` + repair `7f0bea084...`; exact-head integration |
| GAP-022 | Architecture assigned Longitudinal Modeling semantics to `psychometric_core` and duplicated implementation responsibility rows | `verification-pending` | RED `fe5eb745...` + repair `7fadc757...`; exact-head integration |
| GAP-023 | `discreteDIFFUSIONstd` rejected a representable subnormal final ratio because `aΔt` underflowed before factor two | `verification-pending` | RED `d5107b198...` + repair `7164c7ce4...`; exact-head integration |
| GAP-024 | Contributor guidance re-authorized a direct provider credential after LLM ownership moved to contextual-orchestrator | `verification-pending` | RED `4248b335...` + repair `01f45a993...`; released CO adoption + exact-head integration |
| GAP-025 | A singleton unit could satisfy the nominal CWC unit floor while all lag evidence came from one repeated unit | `verification-pending` | RED `671709bbc...` + repair `4784b370c...`; exact-head integration |
| GAP-026 | Scalar standardized longitudinal maps rejected representable finals when a cancelled stationary-variance intermediate lay outside binary64 range | `verification-pending` | RED `4a1f6c49...` / `96d8ed13...` + repairs `a4bc6230...` / `33f4b187...` / `26b03c32...` |
| GAP-027 | Finite-interval `discreteDIFFUSIONstd` could report exact unit diffusion after exponent saturation erased a nonzero remainder | `verification-pending` | RED `a8de3c9f...` + repair `c17e2ff8...` |
| GAP-028 | Actual stationary variance `p` could be misreported as exact zero when positive real `p` lies below binary64 range | `verification-pending` | RED `27d9fa39...` + repair `a0132b62...` |
| GAP-029 | Occasion-mean temporal composition arrived in the wrong bounded context with raw-bit event identity, naive mean summation, and order-dependent averaging | `verification-pending` | verified-successor #486 closure into #310 + release fragment `db335d90...` |
| GAP-030 | Prediction-contradiction Analysis Run treated four observed relation classes as mandatory design strata | `active-fold` | RED `a2892b6...` + repair `a6402015...` + Proposed ADR repair `e07b2ff9...`; fold into #416 |
| GAP-031 | Occasion-mean same-sign averaging double-rounded a representable minimum-subnormal ties-to-even mean | `verification-pending` | RED `9aff817f...` + repair `40e057b8...` |
| GAP-032 | CWC/irregular-residual same-sign averaging retained the same minimum-subnormal double-rounding defect | `verification-pending` | RED `23476f45...` + repair `b14eb6e8...` |
| GAP-033 | Arbitrary max-magnitude normalization introduced a second rounding and misrounded a 7.5-ULP subnormal mean | `verification-pending` | RED `b073f03f...` + repair `350b8d4e...` + consolidation `dd53eff6...` |
| GAP-034 | Mixed-sign cancellation rounded a retained-only mean before restoring the original denominator | `verification-pending` | RED `ae5e61f9...` + repair `39469067...` |
| GAP-035 | Within/between decomposition shadow running mean misrounded `[1 ULP, 2 ULP]` | `verification-pending` | RED `7dc87aa8...` + repair `97c8ad35...` |
| GAP-036 | Known-truth component RMSE could underflow a nonzero recovery error to exact zero | `verification-pending` | RED `496583c6...` + repair `a82b383b...` |
| GAP-037 | Strict-interior lagged covariance could round to false exact `±1` correlation | `verification-pending` | RED `683b28ee...` + repair `9eeb373d...` |
| GAP-038 | Exact Cauchy–Schwarz boundary covariance could round one ULP below `±1` | `verification-pending` | RED `c2500090...` + repair `d06259ec...` |
| GAP-039 | Exact zero lagged covariance could leak IEEE `-0.0` through public projection | `verification-pending` | RED `e15d0531...` + repair `fc61f7bd...` |
| GAP-040 | Exact zero within-person deviation could leak IEEE `-0.0` through public decomposition | `verification-pending` | RED `aeb008a3...` + repair `a9a70baa...` |
| GAP-041 | One-sign irregular residual log-rate mean could underflow to exact zero | `verification-pending` | RED `96f1c334...` + repair `ae5081d8...` |
| GAP-042 | Ratio-first logarithm could nearly double an adjacent-float irregular residual growth rate | `verification-pending` | RED `766ddc7a...` + `ln_1p` repair `16f21d9a...` |
| GAP-043 | Duplicate known-truth component identities could silently reweight RMSE recovery evidence | `verification-pending` | RED `698f12f5...` + uniqueness repair `2fae4cb2...` |
| GAP-044 | Known-truth RMSE alignment/accumulation depended on row order instead of component identity | `verification-pending` | RED `8ad72ac9...` / `5fb93c40...` + repairs `2dd9537e...` / `025dce7f...`; #310 `c6680450...` |
| GAP-045 | Mean signed bias could reject a representable recovery result because finite residuals were summed before dividing | `verification-pending` | RED `c5ec42e4...` + repair `7499042f...`; #488 `daa09786...` |
| GAP-046 | Bias SE could reject a representable SEM because raw squared deviations/intermediates overflowed | `verification-pending` | RED `7de0ef90...` + repair `cad23162...`; #488 `daa09786...` |
| GAP-047 | Generic RMSE/RMSE-SE could reject representable extremes/subnormals or report false perfect recovery | `verification-pending` | RED `dd41ff53...` / `f4e19991...` + repair `6b182107...`; #488 `daa09786...` |
| GAP-048 | Mean signed bias could erase a representable subnormal residual during mixed-sign cancellation | `verification-pending` | RED `b6084750...` + repair `227921d9...`; #488 `daa09786...` |
| GAP-049 | Stable `Between` recovery could be aliased across occasion indices and reweight RMSE | `verification-pending` | RED `0a03041c...` + repair `ec2c1219...`; #310 `c6680450...` |
| GAP-050 | Zero-multiplier SE-aware acceptance could erase a nonzero residual during scale reduction | `verification-pending` | RED `bd8a7c8a...` + repair `00ef2d90...`; #488 `daa09786...` |
| GAP-051 | Exact-recovery acceptance distinguished IEEE `-0.0` and `+0.0` | `verification-pending` | RED `379e6525...` + repair `55876e60...`; #488 `daa09786...` |
| GAP-052 | All-covered Wilson lower endpoint could cancel a positive representable value to zero | `verification-pending` | RED `f84e5918...` + repair `fe9b9c8a...`; #488 `daa09786...` |
| GAP-053 | Strict-interior Wilson lower endpoint could cancel a positive representable value to zero | `verification-pending` | RED `9d45f482...` + repair `4f259f6e...`; #488 `daa09786...` |
| GAP-054 | Wilson upper endpoint could falsely round to exact `1.0` | `verification-pending` | RED `c070da26...` / `344081bf...` + repair `9a2fdd05...`; #488 `daa09786...` |
| GAP-055 | Strict-interior Wilson lower endpoint could accept a nonzero cancellation residue | `verification-pending` | RED `1a24fac7...` + repair `f7e20ddc...`; #488 `daa09786...` |
| GAP-056 | Durable/human-facing Validation Evidence could accept impossible finite relationships or bypass validation | `verification-pending` | report/serde/egress/projection repair lineage; #488 `daa09786...` |
| GAP-057 | RMSE-specific Monte Carlo slot could admit negative mean/percentiles from a generic signed carrier | `verification-pending` | RED `3cd6e41d...` + repair `0090259d...`; #488 `daa09786...` |
| GAP-058 | Generic Monte Carlo summary could admit impossible coarse SD/SE/count relationships | `verification-pending` | RED `e2d0c057...` + repair `0e973b56...`; #488 `daa09786...` |
| GAP-059 | Generic Monte Carlo summary could materially misstate positive SE relative to `SD / sqrt(n)` | `verification-pending` | RED `0a4c242f...` + repair `9b53076a...`; #488 `daa09786...` |
| GAP-060 | RMSE Monte Carlo exact-zero mean could coexist with positive spread/support | `verification-pending` | RED `a17dfe1b...` + repair `d17d8034...`; #488 `daa09786...` |
| GAP-061 | Generic Monte Carlo zero spread could retain non-degenerate empirical support | `verification-pending` | RED `ce21941a...` + repair `d0f5c145...`; #488 `daa09786...` |
| GAP-062 | Finite-tolerance matching could fail while the pair is deterministically outside every finite tolerance | `verification-pending` | RED `d023ecdb...` + repair `5040ff96...`; #488 `daa09786...` |
| GAP-063 | Exact-zero point RMSE could coexist with positive point RMSE SE | `verification-pending` | RED `f7b018c5...` + repair `4c599918...`; #488 `daa09786...` |
| GAP-064 | Positive point RMSE could carry SE above the declared squared-residual producer support | `verification-pending` | RED `a2aca5b0...` + repair `32f09402...`; #488 `daa09786...` |
| GAP-065 | Positive Monte Carlo RMSE could carry spread/SE impossible for nonnegative replications | `verification-pending` | RED `43a7dec1...` + repair `2f78954e...`; #488 `daa09786...` |
| GAP-066 | RMSE Monte Carlo nearest-rank percentile could exceed total nonnegative `n*mean` support | `verification-pending` | RED `84a200ee...` + repair `04c9cdd4...`; #488 `daa09786...` |
| GAP-067 | Generic nearest-rank percentile endpoint could be individually incompatible with represented mean/sample spread | `verification-pending` | RED `40acb4f6...`; rejected over-strong repair `2798e4f9...`; causal repair `c7151b49...`; edge `dbef285b...`; changelog `c38a320c...`; research `2bbcbb24...`; #488 `daa09786...` |
| GAP-068 | Distinct generic percentile endpoints could each pass individual radius checks while jointly exceeding the sample squared-deviation budget | `verification-pending` | RED `c4a13826...` + causal repair `cb3f80a2...` + changelog `f727450d...`; #488 `daa09786...` |
| GAP-069 | Two-replication distinct nearest-rank endpoints could exhaust the retained sample while the stored mean or SD remained impossible | `verification-pending` | RED `81bf0d9e...` + causal repair `d48f8fef...` + mean coverage `fb314d8a...` + branch cleanup `440b78d8...` + changelog `45116498...`; #488 `daa09786...` |
| GAP-070 | Stored Wilson bounds could contain empirical coverage while the lower/upper pair could not arise from one Wilson score interval for that same coverage | `verification-pending` | RED `a839c606...` + repair `38c5b8e8...` + complementary edge `c1cb16a7...`; #488 `daa09786...` |
| GAP-071 | Durable Wilson evidence lacked denominator and critical-value/sidedness provenance, preventing exact recomputation from a persisted artifact | `active-repair` | valid carrier RED `6f6e06d2...`; shared count/Wilson authority `ca517ed3...`; `WilsonCoverageEvidenceV1` `31e1ab2b...`; sidedness `e9c63926...`/`fdd24a1a...`; envelope RED `07766cb0...` + `ValidationEvidenceV1` repair `a16f22e6...`; docs/current #488 `daa09786...`; remaining: hosted GREEN/review/main plus durable-consumer adoption |
| GAP-072 | Exact all-covered Wilson evidence could retain a zero lower endpoint even though the canonical non-empty finite-`z²` producer always emits `n/(n+z²) > 0` | `verification-pending` | RED `ce714f07...` + non-force source-restore RCA `18499052...` + repair `72e9d954...` + changelog `e3a2f4a2...`; inherited by #488 `daa09786...` |
| GAP-073 | Durable Wilson count provenance could round one uncovered observation away above binary64's exact-integer range and expose a pointer-width-dependent count schema | `verification-pending` | RED `29d710a5...` + complement/count repair `29968c80...` + fixed-width carrier repair `91d9a3bb...` + exact ratio RED/repair `63ddfdd6...`/`11323c57...`; #488 `daa09786...` |
| GAP-074 | Durable Wilson evidence could correctly retain exact `u64` counts yet pre-round `sample_count` before endpoint projection, shifting strict-interior endpoints or erasing all-covered finite-sample uncertainty | `verification-pending` | RED `f89e2467...` + all-covered edge `1a5180b2...` + stale-oracle correction `6254c498...` + causal reciprocal-scale repair `73bbb5cf...` + changelog `f2b5768b...` + research predecessor `7fc78a59...`; current #488 `daa09786...` |
| GAP-075 | Inexact durable all-covered Wilson evidence could round the complementary miss mass to exact one at extreme finite `z` and erase a representable positive lower endpoint | `verification-pending` | RED `059ce70d...`; transient duplicate-owner implementation `46f91f3f...` excluded; canonical repair `0f478392...`; single-writer cleanup `8e2058f2...`; exact-oracle correction `93d5d208...`; changelog correction `43b6562c...`; research predecessor `b03a719d...`; current #488 `daa09786...` |
| GAP-076 | Exactly representable all-covered Wilson counts could absorb a small positive `z²` in `n + z²` and falsely emit exact `1.0` although a lower binary64 endpoint is representable | `verification-pending` | RED `e0c4ec81...` + causal boundary repair `c9dcb9df...` + below-resolution edge `6140080d...` + changelog `09ebb482...` + research predecessor `fe89c438...`; current #488 `daa09786...` |
| GAP-077 | Exactly representable all-covered Wilson counts could form an inexact partial denominator `n + z²` whose rounded sum shifts the represented-input lower endpoint by one ULP even though neither operand is fully absorbed | `verification-pending` | RED `06e55653...` + causal TwoSum/FMA repair `6c084dbe...` + research `25cc1943...` + changelog `5acc894b...` + predecessor changelog correction `f89e36d1...` + correct-direct-rounding control `80a0a0ad...`; current #488 `daa09786...` |
| GAP-078 | Exact-count all-covered Wilson residual compensation could move a direct quotient by one ULP even when the represented-input rational remains on the same side of the adjacent midpoint | `verification-pending` | RED `d076d344...` + midpoint-selection repair `32314239...` + changelog `a75940c9...` + research predecessor `ef6441f7...`; current #488 `daa09786...` |
| GAP-079 | SE-aware acceptance could underflow a finite positive `k · SE` tolerance to exact zero by dividing `SE` by a much larger estimate/target scale before multiplication | `verification-pending` | RED `76067efc...` + causal direct-finite comparison repair `4ffdf366...` + changelog `d822c0ba...` + research/current #488 `daa09786...` |

## Release gate

TEPP currently has no GitHub release. Release is permitted only after a clean coherent vertical reaches protected main with exact protected-head CI/security evidence, scientific/recovery acceptance, reproducible package/build artifacts with SBOM and provenance, validated migrations/upgrade/rollback/recovery where applicable, consistent version metadata and current `CHANGELOG.md`, accessibility/operability evidence for user-facing components, no unresolved scientific/privacy/security/supply-chain blockers, and released integration contracts where deployment depends on them. Queued/pending/startup-failed/skipped or predecessor-head evidence is not GREEN.
