# Product and Technical Gap Baseline

**Status:** Active delivery recovery

**Product:** Temporal Event Psychometrics Platform (TEPP)

**Snapshot:** 2026-09-04T04:10Z

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

## Current landing authority

#435 intentionally omits its own mutable branch SHA from this file.

| PR | Exact current head | Draft | Base | Disposition |
| ---: | --- | :---: | --- | --- |
| #488 | `4f1fdc52c1857072f79a3f91f80b5c4f9af8966d` | true | `main` | Validation Evidence numerical/artifact repair. Generic `MonteCarloSummary` binds SE to represented `SD / sqrt(n)`, zero spread to degenerate support, nearest-rank endpoints to represented moment support, distinct endpoints jointly to one squared-deviation budget, and—when `n = 2` and the endpoints are numerically distinct—the exposed endpoints exhaust the retained sample, so recorded mean/SD must match those two values. Typed `monte_carlo_rmse` additionally enforces nonnegative RMSE support. Exact-head hosted gates and independent review remain required. |
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

`psychometric_core` is not authority for new temporal/state composition. fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and arithmetic. TEPP consumes only immutable released/versioned Published Language through an ACL; source copying and mutable sibling-head dependencies are prohibited.

contextual-orchestrator owns provider/model routing and semantic LLM execution. Semantic LLM work and model-backed Actions must use a released/versioned contextual-orchestrator contract; Actions use `orchestrator/free` through the gateway credential and must not select providers/models/groups or consume direct provider keys. Context Graph contracts are contract-only integration authority; EA Core owns enterprise-architecture decisions. No cross-service SQL.

The clock contract separates event/valid time, assertion time, document time, system time, available time, and knowledge cutoff. Retrospective evidence may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective/citation/revision/provenance relations.

## Scientific invariants

- Rasch remains distinct from generic 1PL; formulation-qualified 2PLM–5PLM, MIRT, ideal-point/GGUM, testlet/rater/facet/generalized-mixed identity is preserved.
- Cross-classification and multiple membership remain distinct; weights are explicit, auditable, time-valid, and normalized or model-estimated according to the formulation.
- A nominal unit identifier is not repeated-measures evidence; singleton units cannot satisfy a longitudinal multilevel floor.
- Occasion-mean deviations `p_it = x_it - μ_t` are not CWC residuals, sample-wide grand-mean residuals, or RI-CLPM within-person effects. Numeric event time defines occasion identity, so `-0.0` and `+0.0` are one occasion.
- Row arrival order is not scientific evidence. Fixed admitted observations must produce bit-identical means/centered results under permutation wherever the contract claims deterministic CPU `f64` reference behavior.
- Known-truth component recovery follows bounded-context identity: stable `Between` is unit-level and uses canonical `occasion_index = 0`; `Within` retains actual `(unit, occasion)` identity. Duplicate identities or stable-Between occasion aliases are not weights.
- A representable final scientific estimand is not rejected solely because an avoidable intermediate binary64 operation overflows/underflows. False exact 0/1/non-finite endpoints and cancellation residues are not accepted when an algebraically equivalent stable form preserves the represented estimand.
- Mean signed bias and bias SE remain Validation Evidence performance measures over admitted recovery units; representable cancellation/SEM must survive avoidable overflow/underflow while mathematically nonzero finals below binary64 range fail closed.
- Generic Validation Evidence RMSE and its delta-method SE normalize before squaring. Exact-zero point RMSE requires exact-zero point RMSE SE. For positive RMSE, the declared producer satisfies `SE(RMSE) <= RMSE / 2`, with a small relative binary64 admission tolerance.
- Generic `MonteCarloSummary` is sign-neutral. Zero SD requires zero SE and percentile endpoints equal to the represented mean. Positive SD requires positive SE coherent with represented `SD / sqrt(n)` within a small relative tolerance. Every inclusive nearest-rank endpoint is a retained observation and satisfies `|endpoint - mean| <= SD * sqrt(n - 1)` under the producer's represented-mean squared-deviation identity. Distinct lower/upper endpoint values share the same `(n - 1) * SD²` deviation budget. When `replication_count = 2` and the endpoints are numerically distinct, those two values exhaust the retained sample, so the recorded represented mean and sample SD must agree with the deterministic two-value reconstruction. Equal numeric endpoints do not prove two distinct ranks and are therefore not subjected to that exhaustion rule.
- When the generic carrier occupies `ValidationReport::monte_carlo_rmse`, retained replications are nonnegative; positive mean implies `SE(mean) <= mean`, and inclusive nearest-rank endpoints cannot exceed the nonnegative sample-sum support `n * mean`. These typed bounds are not imposed on signed metrics such as bias.
- SE-aware acceptance is `|estimate - target| <= k * SE`. For `k = 0` or exact-zero SE, exact recovery is evaluated before scale reduction; `-0.0` and `+0.0` are one numeric zero-valued scientific state.
- `match_count` is a finite-threshold decision and does not require materializing an unrepresentable absolute residual; `absolute_residuals` remains fail closed when the magnitude itself is requested.
- Wilson interval coverage evidence preserves representable endpoints through stable rationalized/complementary forms rather than cancellation-prone center±margin evaluation.
- Durable and human-facing Validation Evidence must preserve producing-metric invariants on explicit validation, serde ingress/egress, and human projection. Finiteness alone is not evidence validity.
- Bias, RMSE, and Monte Carlo may share private deterministic scalar support inside `validation_core`, but metric-specific cancellation/normalization, denominator, uncertainty, and fail-closed semantics remain with their bounded modules.
- Historical-cutoff admission occurs before duplicate-identity checks. Future-unavailable evidence cannot change an earlier run's conflicts, counts, or terminal state.
- Supported temporal estimators require state/trajectory and claimed-structure recovery, bias/RMSE, interval coverage, convergence, uncertainty calibration, reproducibility, and leakage-safe rolling-origin evidence.
- CPU/GPU parity counts only when the relevant accelerator path actually runs. Scientific failures are never hidden with skip/xfail/source rewriting/coverage exclusions.

## Current repairs and blockers

### #310 — Longitudinal Modeling

#310 remains the canonical Longitudinal Modeling landing vehicle at `c6680450152b1e0a2c9abb553772d74a23923335`. Current-head Rust/documentation/security/review evidence remains required before Ready/merge; predecessor or child-head evidence does not transfer.

### #488 — Validation Evidence

#488 is the generic Validation Evidence landing vehicle at exact head `4f1fdc52c1857072f79a3f91f80b5c4f9af8966d`. It remains distinct from Longitudinal Modeling and does not consume mutable fast-mlsirm source.

The preceding RMSE, bias, Wilson, Monte Carlo SE, durable-artifact, typed-RMSE, matching, individual percentile moment-support, and joint percentile moment-support lineages remain in branch ancestry and GAP-045–GAP-068.

Fresh review found one more finite-sample contract that the stored artifact itself determines. With exactly two retained replications and two numerically distinct nearest-rank endpoint values, the endpoints are the only two retained observations. A payload with `mean=0`, `SD=1`, `SE=1/sqrt(2)`, lower `-0.5`, upper `0.5` passes the predecessor's SE, individual endpoint, and joint deviation-budget checks, but the only possible two-observation sample has represented mean `0` and sample SD `sqrt(0.5)`. Public RED `81bf0d9e2f1a28947b1343244002d6762b703f8a`; corrected causal implementation `d48f8fef08e77b8fa654f2852814c25c5d1baa79`; represented-mean coverage `fb314d8afccdf19f0074c64f6277d2edb290e907`; branch-coverable cleanup `440b78d86908fe1464e65bdbc4ceb5f9f6606c9f`; changelog `45116498f29c0d3421192d452e26182975b114ae`; research/current exact head `4f1fdc52c1857072f79a3f91f80b5c4f9af8966d`. Equal numeric endpoints remain under the conservative generic rules because the artifact does not preserve percentile probabilities or rank multiplicity.

Morris, White, and Crowther (2019) remain the simulation performance-measure/Monte Carlo uncertainty methodology trace; Wilson (1927) remains the primary score-interval reference. AERA/APA/NCME (2014) remains the current published testing-standards edition while revision is underway. Exact-head hosted Rust/documentation/security/supply-chain/100%-coverage gates and a qualifying independent current-head review remain required.

### #416 — Validation / Analysis Run consolidation

#416 exact head is `aa730c63563eb4a33048d822b581036c8487bd47`. #458/#460/#482/#483/#484/#485/#487 remain fold children until every unique source/test/fixture/contract/doctoring delta is inherited by a conflict-resolving survivor. Child-head CI never transfers to #416.

### #480 — contextual-orchestrator boundary

#480 removes TEPP-owned provider discovery/ranking and requires a released contextual-orchestrator owner contract. The consumer remains Draft until an immutable compatible release plus deployment/auth/schema/artifact provenance exists. Mutable owner main is evidence, not a production contract.

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
| GAP-045 | Mean signed bias could reject a representable recovery result because finite residuals were summed before dividing | `verification-pending` | RED `c5ec42e4...` + repair `7499042f...`; #488 `4f1fdc52...` |
| GAP-046 | Bias SE could reject a representable SEM because raw squared deviations/intermediates overflowed | `verification-pending` | RED `7de0ef90...` + repair `cad23162...`; #488 `4f1fdc52...` |
| GAP-047 | Generic RMSE/RMSE-SE could reject representable extremes/subnormals or report false perfect recovery | `verification-pending` | RED `dd41ff53...` / `f4e19991...` + repair `6b182107...`; #488 `4f1fdc52...` |
| GAP-048 | Mean signed bias could erase a representable subnormal residual during mixed-sign cancellation | `verification-pending` | RED `b6084750...` + repair `227921d9...`; #488 `4f1fdc52...` |
| GAP-049 | Stable `Between` recovery could be aliased across occasion indices and reweight RMSE | `verification-pending` | RED `0a03041c...` + repair `ec2c1219...`; #310 `c6680450...` |
| GAP-050 | Zero-multiplier SE-aware acceptance could erase a nonzero residual during scale reduction | `verification-pending` | RED `bd8a7c8a...` + repair `00ef2d90...`; #488 `4f1fdc52...` |
| GAP-051 | Exact-recovery acceptance distinguished IEEE `-0.0` and `+0.0` | `verification-pending` | RED `379e6525...` + repair `55876e60...`; #488 `4f1fdc52...` |
| GAP-052 | All-covered Wilson lower endpoint could cancel a positive representable value to zero | `verification-pending` | RED `f84e5918...` + repair `fe9b9c8a...`; #488 `4f1fdc52...` |
| GAP-053 | Strict-interior Wilson lower endpoint could cancel a positive representable value to zero | `verification-pending` | RED `9d45f482...` + repair `4f259f6e...`; #488 `4f1fdc52...` |
| GAP-054 | Wilson upper endpoint could falsely round to exact `1.0` | `verification-pending` | RED `c070da26...` / `344081bf...` + repair `9a2fdd05...`; #488 `4f1fdc52...` |
| GAP-055 | Strict-interior Wilson lower endpoint could accept a nonzero cancellation residue | `verification-pending` | RED `1a24fac7...` + repair `f7e20ddc...`; #488 `4f1fdc52...` |
| GAP-056 | Durable/human-facing Validation Evidence could accept impossible finite relationships or bypass validation | `verification-pending` | report/serde/egress/projection repair lineage; #488 `4f1fdc52...` |
| GAP-057 | RMSE-specific Monte Carlo slot could admit negative mean/percentiles from a generic signed carrier | `verification-pending` | RED `3cd6e41d...` + repair `0090259d...`; #488 `4f1fdc52...` |
| GAP-058 | Generic Monte Carlo summary could admit impossible coarse SD/SE/count relationships | `verification-pending` | RED `e2d0c057...` + repair `0e973b56...`; #488 `4f1fdc52...` |
| GAP-059 | Generic Monte Carlo summary could materially misstate positive SE relative to `SD / sqrt(n)` | `verification-pending` | RED `0a4c242f...` + repair `9b53076a...`; #488 `4f1fdc52...` |
| GAP-060 | RMSE Monte Carlo exact-zero mean could coexist with positive spread/support | `verification-pending` | RED `a17dfe1b...` + repair `d17d8034...`; #488 `4f1fdc52...` |
| GAP-061 | Generic Monte Carlo zero spread could retain non-degenerate empirical support | `verification-pending` | RED `ce21941a...` + repair `d0f5c145...`; #488 `4f1fdc52...` |
| GAP-062 | Finite-tolerance matching could fail while the pair is deterministically outside every finite tolerance | `verification-pending` | RED `d023ecdb...` + repair `5040ff96...`; #488 `4f1fdc52...` |
| GAP-063 | Exact-zero point RMSE could coexist with positive point RMSE SE | `verification-pending` | RED `f7b018c5...` + repair `4c599918...`; #488 `4f1fdc52...` |
| GAP-064 | Positive point RMSE could carry SE above the declared squared-residual producer support | `verification-pending` | RED `a2aca5b0...` + repair `32f09402...`; #488 `4f1fdc52...` |
| GAP-065 | Positive Monte Carlo RMSE could carry spread/SE impossible for nonnegative replications | `verification-pending` | RED `43a7dec1...` + repair `2f78954e...`; #488 `4f1fdc52...` |
| GAP-066 | RMSE Monte Carlo nearest-rank percentile could exceed total nonnegative `n*mean` support | `verification-pending` | RED `84a200ee...` + repair `04c9cdd4...`; #488 `4f1fdc52...` |
| GAP-067 | Generic nearest-rank percentile endpoint could be individually incompatible with represented mean/sample spread | `verification-pending` | RED `40acb4f6...`; rejected over-strong repair `2798e4f9...`; causal repair `c7151b49...`; edge `dbef285b...`; changelog `c38a320c...`; research `2bbcbb24...`; #488 `4f1fdc52...` |
| GAP-068 | Distinct generic percentile endpoints could each pass individual radius checks while jointly exceeding the sample squared-deviation budget | `verification-pending` | RED `c4a13826...` + causal repair `cb3f80a2...` + changelog `f727450d...`; #488 `4f1fdc52...` |
| GAP-069 | Two-replication distinct nearest-rank endpoints could exhaust the retained sample while the stored mean or SD remained impossible | `verification-pending` | RED `81bf0d9e...` + causal repair `d48f8fef...` + mean coverage `fb314d8a...` + branch cleanup `440b78d8...` + changelog `45116498...` + research/current #488 `4f1fdc52...`; exact-head GREEN/review/main integration |

## Release gate

TEPP currently has no GitHub release. Release is permitted only after a coherent vertical reaches protected main with exact protected-head CI/security/recovery evidence, reproducible package + SBOM + provenance + rollback artifacts, current version/CHANGELOG, required migration/restore/load evidence, and released integration contracts where deployment depends on them. Queued/pending/startup-failed/skipped or predecessor-head evidence is not GREEN.
