# Product and Technical Gap Baseline

**Status:** Active delivery recovery

**Product:** Temporal Event Psychometrics Platform (TEPP)

**Snapshot:** 2026-09-03T13:18Z

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
| Non-Draft pull requests | **0** | No PR is eligible for normal merge until it is deliberately made Ready after exact-head evidence. |
| Open issues | **16** | ADR normalization, orchestration admission, evaluation drift, and scientific recovery remain open. |
| GitHub releases | **0** | No TEPP open head is a released contract. |
| Organization ruleset | `18156473` | One qualifying current-head approval, stale-review dismissal after push, resolved threads, extra approval for unattributed changes where applicable, and central required workflows. |

Ruleset `18156473` permits merge/squash and prohibits deletion/non-fast-forward updates on the default branch. Organization-admin bypass is not normal delivery evidence and is not used by this writer.

## Current landing authority

#435 intentionally omits its own mutable branch SHA from this file.

| PR | Exact current head | Draft | Base | Disposition |
| ---: | --- | :---: | --- | --- |
| #488 | `956c9c98931cb22a445f22ec674eb48d598c7d5c` | true | `main` | Validation Evidence numerical repair. Mean-bias RED `c5ec42e4...` / repair `7499042f...` removes avoidable raw residual-sum overflow; bias-SE RED `7de0ef90...` / repair `cad23162...` removes avoidable raw squared-deviation/variance overflow and supplies a scaled direct SEM path. Exact-head hosted gates and independent review remain required. |
| #487 | `e07b2ff9f78ef456ff911b8643710af20921fe54` | true | #416 `feat/copy-identity-analysis-run-gap-004` | Validation / Analysis Run fold child. Sparse observed relation classes are valid evidence; RED `a2892b6...`, repair `a6402015...`, predecessor-test correction `6b0c8de6...`; ADR 0079 is repaired to `Proposed` at `e07b2ff...`. No child-head CI transfers to #416. |
| #485 | `f71591864efc2beff336ced7ef35d5a013305c36` | true | #416 | Analysis Run fold child; preserve support-edge source/tests/doctoring. |
| #484 | `9a1be78b5342ff65e3cf2aac1e9331c68943f246` | true | #416 | Analysis Run fold child; preserve summarizes-edge source/tests/doctoring. |
| #483 | `847d96f913bb261803ac0bd751ad7e4f51324cee` | true | #416 | Analysis Run fold child; preserve unique evidence. |
| #482 | `506dbae236a4484301b704b6c6a05b20faf0fe69` | true | #416 | Analysis Run fold child; preserve unique evidence. |
| #480 | `01f45a99392457334a4f6d3d659f992af739eeee` | true | `main` | contextual-orchestrator consumer repair; stays Draft while immutable CO release/deployment/auth provenance is unavailable. |
| #460 | `dfab4eab5ff733731e565a9348072b8dab2e4912` | true | #416 | Analysis Run fold child; typed cutoff equality and terminal-validation separation. |
| #458 | `08165e3b3c929b4ae77396689549f72723ff8ff5` | true | #416 | Analysis Run fold child; typed cutoff equality and terminal-validation separation. |
| #416 | `aa730c63563eb4a33048d822b581036c8487bd47` | true | `main` | Validation / Analysis Run landing vehicle. Generic cutoff-before-identity repair remains in ancestry; six current-head follow-up commits canonicalize episode/cutoff fixtures and harden bounded artifact/profile tests. |
| #310 | `464df01e9268f13ba8e135f72d0fb62beead3e04` | true | `main` | Longitudinal Modeling vehicle; known-truth RMSE now aligns and accumulates by canonical component identity so serialization order cannot change admission or deterministic CPU `f64` recovery. Exact-head verification remains non-passing until current gates and review complete. |

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
- Row arrival order is not scientific evidence. Fixed admitted observations must produce bit-identical means/centered results under permutation wherever the contract claims deterministic f64 reference behavior.
- Known-truth component recovery aligns truth and recovered values by unique `(unit, occasion, level)` identity and accumulates in canonical identity order. A permutation of either slice must preserve the same deterministic CPU `f64` result.
- A representable final scientific estimand is not rejected solely because an avoidable intermediate binary64 operation overflows/underflows. False exact 0/1/non-finite endpoints remain fail-closed when the mathematical estimand is interior/nonzero.
- Mean signed bias is a Validation Evidence performance measure over admitted recovery units. A representable bias must not fail solely because a raw residual sum overflows; exact cancellation remains zero, while a mathematically nonzero bias below binary64 range fails closed rather than becoming false zero.
- Bias standard error is formed from the same admitted signed differences. A representable SEM must not fail solely because raw squared deviations, their sum, or an intermediate sample variance overflows; scaling is applied before squaring and the SEM is formed directly.
- A mathematically nonzero scientific error metric must not be promoted to exact perfect recovery merely because its final binary64 representation underflows to zero.
- Known-truth recovery denominators are defined by unique scientific component identity. Duplicate `(unit, occasion, level)` rows are not implicit weights and must fail closed unless a separately named weighted contract explicitly owns the weighting rule.
- A mathematically nonzero one-sign temporal-rate mean must not be promoted to exact no-change merely because its final binary64 representation underflows to zero. All-zero inputs and exact mixed-sign cancellation remain exact zero.
- A temporal log-rate over represented endpoint magnitudes must not be materially changed by avoidable ratio-first rounding. Nearby values use exact-difference/`ln_1p` evaluation before event-time division; extreme-scale fallbacks remain fail-closed or log-domain as appropriate.
- A strict-interior exact binary64 covariance must not be promoted to exact `±1` correlation merely because rounded square roots/divisions reach the endpoint. Conversely, an exact represented Cauchy–Schwarz boundary must not be weakened to an interior coefficient merely because rounded square roots/divisions fall one ULP below `1`.
- Exact zero covariance has one public no-association encoding. IEEE `-0.0` is not directional scientific evidence and is canonicalized to `+0.0` at the Temporal Association projection boundary after validation.
- Exact zero within-person deviation has one public decomposition encoding. IEEE `-0.0` does not denote negative within-person change and is canonicalized to `+0.0` only at the validated public residual projection boundary; private numerical intermediates may retain signed zero.
- Binary64 minimum-subnormal means follow IEEE ties-to-even at the public composition boundary. Overflow-safe normalization must not add an avoidable earlier rounding step, and mixed-sign cancellation must apply the original sample-count denominator before rounding a retained-only mean.
- Observed Allen support classes are data, not required design strata. A historical prediction census may truthfully have zero covered, partial-overlap, adjacent, or contradictory rows; absent classes remain explicit zero counts rather than invalidating the run.
- Historical-cutoff admission occurs before duplicate-identity checks. Future-unavailable evidence cannot change an earlier run's conflicts, counts, or terminal state.
- Supported temporal estimators require state/trajectory and claimed-structure recovery, bias/RMSE, interval coverage, convergence, uncertainty calibration, and leakage-safe rolling-origin evidence.
- CPU/GPU parity counts only when the relevant accelerator path actually runs. Monte Carlo decisions use simulation uncertainty rather than arbitrary pass percentages.
- Scientific failures are never hidden with skip/xfail/source rewriting/coverage exclusions.

## Current repairs and blockers

### #310 — Longitudinal Modeling

#310 remains the canonical Longitudinal Modeling landing vehicle. Its lineage covers lagged Pearson correlation with both marginal variances, stationary-variance materialization versus algebraically cancelled standardized maps, discrete-diffusion endpoint/subnormal representability, CWC atomistic admission, one irregular-rate authority, occasion-mean composition, known-truth recovery, within/between decomposition, and DDD relocation from `psychometric_core` to `longitudinal_core`.

The #486 occasion-mean fold is owner-correct on #310: RED `75b0184d...`, owner repair `7fe9aaf2...`, export/test repairs `b900e213...` / `30771ff2...`, determinism RED `8a59019e...`, same-sign ordering repair `465d139d...`, same-panel/sparse successor regressions `b9e952bb...` / `aad56b50...`, ADR `04b3c26e...`, PRD `a221f494...`, research trace `27aa78ee...`, subnormal RED `9aff817f...`, source repair `40e057b8...`, and trace `a2e6ace7...`. Commit `db335d904ccc43dd0af7982c34df83008c968ef6` preserves the child's valid operational release delta in `CHANGELOG.d/longitudinal-occasion-mean-event-time.md` under the correct owner. Repository precedent verifies `CHANGELOG.d` as a valid non-destructive stacked-consolidation release-note carrier, and #486 was closed as a verified successor after all valid scientific/test/contract/docs/release-note deltas were checked on #310; invalid wrong-owner/weaker deltas were intentionally not copied.

The CWC/occasion/decomposition/irregular-rate mean lineage remains one Longitudinal-local authority. CWC RED `23476f45...` / repair `b14eb6e8...` closed the `[1 ULP, 2 ULP]` ties-to-even defect; `b073f03f...` / `350b8d4e...` changed arbitrary normalization to an exact power-of-two scale; `dd53eff6...` consolidated CWC and occasion means; mixed-sign RED `ae5e61f9...` / repair `39469067...` plus `201f3900...` and trace `34fd059a...` removed retained-mean double rounding; decomposition RED `7dc87aa8...` / repair `97c8ad35...` and trace `a1aef11e...` removed the remaining shadow running mean. Mean-level underflow RED `96f1c334...` proves that individually representable temporal rates can still have a mathematically nonzero aggregate mean below binary64's range; repair `ae5081d8...` rejects only that one-sign false-zero endpoint, while exact all-zero input and exact mixed-sign cancellation remain zero.

Individual-rate review also found a quotient-rounding defect. For the represented value immediately below `2.0` followed by `2.0`, the previous finite-ratio-first path rounded `later / earlier` before `ln`, making the recovered log-rate almost twice the correctly rounded logarithm of the represented endpoint ratio. Public RED `766ddc7a3d95102450d663f06d2c577306964d79` fixes the adjacent-float power-of-two fixture. Causal repair `16f21d9a223fb34d52f24d148c7cad9b385e1d91` uses exact represented differences plus `ln_1p` within the factor-two neighborhood and preserves direct-ratio/log-domain fallbacks for extreme scales.

Known-truth component RMSE now has four linked identity/numerical boundaries. First, for minimum positive subnormal ULP `u`, five identity-matched component rows with one residual `u` and four exact-zero residuals have mathematical RMSE `u / sqrt(5) > 0`, but the final binary64 product rounds to exact zero. RED `496583c6b62cbe0ad1be0e65b51f01d7f72acd5a` drives the public API; causal repair `a82b383b5940126a0139180d66729d2e6aa4baf7` preserves exact perfect recovery only on the pre-existing `scale == 0` path and rejects a later rounded zero after nonzero residual scale.

Second, repeating an already-matched component silently changed `n` in the RMSE denominator and therefore acted as an undocumented weight. Public RED `698f12f5b2f7a3c194e9d1d3f00c5aeaf10591f8` requires duplicate scientific identity to fail closed; uniqueness repair `2fae4cb2e7df2845270bd27192000ca370fb05ad` prevents duplicate denominator weighting.

Third, the predecessor still used pairwise slice position to align truth and recovered values. Public RED `8ad72ac91cbddd5ce6432fc70630bad6ce7072ce` permutes recovered rows while preserving the same unique identity-value pairs and requires the same perfect-recovery result. Initial identity-alignment repair `2dd9537e04dd2048559ba707fecd2404db5a2a31` indexes recovered values by `(unit, occasion, level)` rather than serialization position.

Fourth, that initial repair still accumulated residuals in caller-provided truth order. Because scaled binary64 sum-of-squares addition is not associative, the same scientific identity-value set could differ by one last-bit RMSE under a truth-row permutation. RED `5fb93c40eddbd9e7920196ef09594457b8ac72d3` fixes a residual set containing `1`, `1e-100`, `3`, and the representable value immediately below `1`. Causal repair `025dce7fd98cfb4f94ea790cacd555b744095377` admits both slices into unique identity maps, sorts the scientific identity tuple, and accumulates only in canonical order. Edge coverage `976ce7d710125717ff2f8daeb943d54278c4acde` separately exercises truth-side duplication while recovered identities remain unique. Research trace `1106c005060c667e892f62ae8763e6b0cc478f0a` records the full denominator/alignment/deterministic-order lineage, and current release fragment/head `464df01e9268f13ba8e135f72d0fb62beead3e04` records the buyer-visible recovery correction. Overflow-safe accumulation, finite-input checks, exact-zero perfect recovery, and nonzero-underflow refusal remain intact.

Exact-binary association review found two endpoint defects in opposite directions. First, with earlier variance `f64::from_bits(4_607_182_418_800_016_408)`, later variance `f64::from_bits(4_607_182_418_800_016_427)`, and covariance magnitude `f64::from_bits(4_607_182_418_800_016_417)`, exact `Cov²` is strictly below the exact product of the two represented marginal variances, yet rounded square roots/divisions produce exact `±1.0`. RED `683b28eeeda3ad72ac11f5317c5aea54f34e0692` fixes both signs through the public `recover_event_time_lagged_correlation` API. Causal repair `9eeb373df2cd333fe7543df2197ea0cc0c492780` extends the exact integer-significand/power-of-two covariance comparison to distinguish strict interior from exact boundary and rejects rounded `±1` for strict-interior inputs.

Second, exact-boundary inputs can round the other way. `Var_t = 2`, `Var_{t+Δ} = 8`, and `|Cov| = 4` satisfy exact represented `Cov² = Var_t Var_{t+Δ}`, but the predecessor square-root/division path returns `0x1.fffffffffffffp-1` for the positive coefficient. RED `c25000901eb429a43817552f8b76cf4aae04e522` fixes both signs; causal repair `d06259ec1e036558d8d2f775c266b2b9db4e42c4` returns the exact signed endpoint when the exact integer relation is on the boundary and uses rounded standardization only for strict-interior coefficients.

A third endpoint identity defect existed at exact zero. A supplied `-0.0` lagged covariance is the same exact no-association estimand as `+0.0`, but the private numeric path could preserve the negative IEEE sign bit into the public return value. RED `e15d053100c17ac46b48cc04ad61440d0185e729` requires both signs to project to public `+0.0`; causal repair `fc61f7bdb2fd8c67719c41a9ce9ef6436beab38a` canonicalizes only the validated zero result at `temporal_association.rs`, preserving private numerical validation/error precedence. Research trace `b56e1d160debde3b656f3ec71780fadb03d2fe85` synchronizes the API/test/module boundary.

Fresh decomposition review found the same semantic class at a different public estimand. A `-0.0` score minus a canonical `+0.0` unit mean can produce a bit-distinct `-0.0` within residual even though exact zero deviation has no direction. RED `aeb008a38bc333ba0f1bec0651739e361426e66d` drives public `decompose_within_between`; causal repair `a9a70baa5c2a40ec8bf3fc77748bc3a5eaf92cd8` canonicalizes only validated exact-zero within residuals at `decompose.rs`, and research trace `ecb909143145b2021a89827cb071315f99c03fb5` records the API/test/module/IEEE-754 boundary in `docs/research/longitudinal-within-between.md`.

At exact #310 head `464df01e9268f13ba8e135f72d0fb62beead3e04`, predecessor workflow/review evidence is stale by definition. Current-head Rust/documentation/security/coverage workflows and a qualifying independent current-head `APPROVED` submission are required before Ready/merge. Queued, pending, startup-failed, skipped, or predecessor-head evidence is not GREEN.

#486 is closed as a verified-successor consolidation, not a PR-count-only close. No child-head CI/review evidence transfers to #310.

### #488 — Validation Evidence bias arithmetic

Protected-main `validation_core::mean_bias` defines bias correctly as `mean(recovered − truth)` but implemented the mean as a raw residual sum followed by division. Two finite `f64::MAX` residuals therefore overflowed the intermediate sum even though the final mean bias is representable as `f64::MAX`; `bias_standard_error` inherited the same mean failure and rejected a constant extreme bias whose sampling variance is exactly zero.

Public RED `c5ec42e40307f3645c18b0d73114b73e01745a20` fixes that recovery contract. Causal repair `7499042f7451b2e3d5e9f83843aeea82c4f5ff06` validates each signed residual, normalizes by the maximum magnitude, sums normalized residuals deterministically with compensated arithmetic, divides by the recovery count, and restores scale once. Exact cancellation remains canonical zero; a represented nonzero normalized mean that becomes zero only at final scale-back fails closed.

Review of the repaired SE path exposed a second avoidable overflow. Public RED `7de0ef90944925ae7b232a8280f5bf9096df6502` uses `[1e154, -1e154, 0]`, whose raw squared-deviation sum overflows even though the final SEM is finite. Repair `cad231620679d8f912bded36c654446032b45e57` scales deviations before squaring and forms the SEM directly; if direct subtraction from the finite mean overflows it re-expresses the same deviations in a normalized scale. `8a6cc346...` and `28d96c23...` harden the public/unit oracle without requiring incidental last-bit equality. Research trace `1f22ef675d632b15378da982ace2182de3fdbab0` and release fragment/current head `956c9c98931cb22a445f22ec674eb48d598c7d5c` keep the complete scientific and buyer-visible lineage attached to the source.

This belongs to TEPP Validation Evidence because it repairs the generic recovery metrics already owned by `validation_core`. It does not move Longitudinal composition or create a second psychometric estimator authority. Generic RMSE numerical limitations remain a separate future finding.

### #416 — Validation / Analysis Run consolidation

#416 exact head is `aa730c63563eb4a33048d822b581036c8487bd47`. The generic cutoff-before-identity repair `0b7155cc...` remains in its ancestry; six newer commits canonicalize episode/cutoff fixtures and harden bounded artifact/profile tests without transferring child-head CI. #458/#460/#482/#483/#484/#485/#487 remain current fold children over shared Cargo/lib/lock/docs surfaces. Their unique evidence must reach a surviving #416 exact head before any child can be considered fully superseded.

#487 exposed a scientific acceptance bug: the first implementation required all four Allen support classes and at least four assignments merely because the test fixture exercised those branches. RED `a2892b6...` requires covered-only and contradiction-only historical censuses to succeed with truthful zero counts. Repair `a6402015...` keeps only real invariants: nonempty admitted evidence, size bound, exact class sums, refusal-plus-covered conservation, cutoff-safe admission, digest/schema validation, and claim boundary. `6b0c8de6...` removes predecessor tests that encoded the rejected fixture-as-production rule. Fresh docs-to-code repair `e07b2ff9...` makes branch-local ADR 0079 `Proposed` under the existing canonical ADR lineage and removes the contradictory mixed-four-class prerequisite. The child remains conflicted beneath #416 and does not transfer child-head evidence to #416.

### #480 — contextual-orchestrator boundary

#480 removes TEPP-owned provider discovery/ranking and requires the contextual-orchestrator owner contract. Its branch-local `AGENTS.md` already replaces the protected-main direct-NVIDIA-key guidance with released contextual-orchestrator-only routing and `orchestrator/free` for model-backed Actions. The consumer remains Draft until the owner provides an immutable compatible release plus authenticated deployment identity/schema/artifact provenance and safe scoped/brokered gateway authentication. Mutable owner main is evidence, not a production contract.

### Owner handoffs

fast-mlsirm protected main and latest immutable release must be revalidated before any dependency bump. Open owner heads newer than the latest release are candidate evidence, not TEPP dependency authority.

Context Graph Contracts and Enterprise Architecture Core remain read-only owner dependencies until their relevant contracts are immutable releases. No sibling mutable head is promoted to TEPP production authority.

### #437 — ADR identity

Repository-wide ADR IDs are immutable authority. Duplicate index IDs/targets/numbered files, index/file mismatch, and repeated Decision-status or Implementation-maturity authority must fail deterministic documentation fitness. Branch-local micro-slice records do not mint independent architecture authority.

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
| GAP-018 | Longitudinal stable-mean logic had a separate decomposition implementation after CWC/occasion consolidation | `verification-pending` | RED `7dc87aa8...` + causal repair `97c8ad35...` + trace `a1aef11e...`; scan remaining Longitudinal means, exact-head GREEN/review/main integration |
| GAP-019 | Longitudinal scientific instructions contradicted stationary-overflow implementation | `verification-pending` | RED `9d8a82d...` + repair `9c962205...`; exact-head documentation/review GREEN and protected-main integration |
| GAP-020 | Nonzero lagged covariance can be misreported as exact-zero correlation when standardized magnitude is unrepresentable | `verification-pending` | RED `c345ee7b...` + repair `5785e07a...`; exact-head Rust/documentation/review GREEN and protected-main integration |
| GAP-021 | Longitudinal irregular-rate facade duplicated public wrapper identities over one canonical implementation | `verification-pending` | RED `464863860...` + repair `7f0bea084...`; exact-head Rust/documentation/review GREEN and protected-main integration |
| GAP-022 | Architecture assigned Longitudinal Modeling semantics to `psychometric_core` and duplicated implementation responsibility rows | `verification-pending` | RED `fe5eb745...` + repair `7fadc757...`; exact-head documentation/review GREEN and protected-main integration |
| GAP-023 | `discreteDIFFUSIONstd` rejected a representable subnormal final ratio because `aΔt` underflowed before the required factor two | `verification-pending` | RED `d5107b198...` + repair `7164c7ce4...`; exact-head GREEN and protected-main integration |
| GAP-024 | Contributor guidance re-authorized a direct provider credential after LLM ownership moved to contextual-orchestrator | `verification-pending` | RED `4248b335...` + repair `01f45a993...`; released CO adoption and exact-head GREEN/review/main merge |
| GAP-025 | A singleton unit could satisfy the nominal CWC unit floor while all lag evidence came from one repeated unit | `verification-pending` | RED `671709bbc...` + repair `4784b370c...`; exact-head GREEN and protected-main integration |
| GAP-026 | Scalar standardized longitudinal maps rejected representable finals when a cancelled stationary-variance intermediate lay outside binary64 range | `verification-pending` | RED `4a1f6c49...` / `96d8ed13...` + repairs `a4bc6230...` / `33f4b187...` / `26b03c32...`; exact-head GREEN and protected-main integration |
| GAP-027 | Finite-interval `discreteDIFFUSIONstd` could report exact unit diffusion after exponent saturation erased a nonzero remainder | `verification-pending` | RED `a8de3c9f...` + repair `c17e2ff8...`; exact-head GREEN and protected-main integration |
| GAP-028 | Actual stationary variance `p` could be misreported as exact zero when positive real `p` lies below binary64 range | `verification-pending` | RED `27d9fa39...` + repair `a0132b62...`; exact-head GREEN and protected-main integration |
| GAP-029 | Occasion-mean temporal composition arrived in the wrong bounded context with raw-bit event identity, naive mean summation, and order-dependent averaging | `verification-pending` | verified-successor #486 closure into owner-correct #310 lineage + release fragment `db335d90...`; exact-head GREEN/review/main integration |
| GAP-030 | Prediction-contradiction Analysis Run treated four observed relation classes as mandatory design strata and documented that branch-local rule as accepted architecture | `active-fold` | RED `a2892b6...` + repair `a6402015...` + predecessor-test correction `6b0c8de6...` + Proposed-ADR repair `e07b2ff9...`; fold unique evidence into #416; exact-survivor GREEN/review/main integration |
| GAP-031 | Occasion-mean same-sign averaging double-rounded a representable minimum-subnormal ties-to-even mean | `verification-pending` | RED `9aff817f...` + repair `40e057b8...` + trace `a2e6ace7...`; exact-head GREEN and protected-main integration |
| GAP-032 | CWC/irregular-residual same-sign averaging retained the same minimum-subnormal double-rounding defect | `verification-pending` | RED `23476f45...` + repair `b14eb6e8...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-033 | Arbitrary max-magnitude normalization introduced a second rounding and misrounded a 7.5-ULP subnormal mean to 7 ULPs | `verification-pending` | RED `b073f03f...` + exact-power-of-two repair `350b8d4e...` + Longitudinal mean-authority consolidation `dd53eff6...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-034 | Mixed-sign cancellation rounded a retained-only mean before restoring the original denominator, moving a representable subnormal mean by one ULP | `verification-pending` | RED `ae5e61f9...` + causal repair `39469067...` + occasion public-contract hardening `201f3900...` + research trace `34fd059a...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-035 | Within/between decomposition shadow running mean misrounded the representable `[1 ULP, 2 ULP]` unit mean and shifted residuals by one ULP | `verification-pending` | RED `7dc87aa8...` + causal repair `97c8ad35...` + research trace `a1aef11e...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-036 | Known-truth component RMSE could underflow a mathematically nonzero recovery error to exact zero and falsely report perfect recovery | `verification-pending` | RED `496583c6...` + causal repair `a82b383b...` + research trace `bfaa69c4...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-037 | Strict-interior lagged covariance could round to false exact `±1` correlation after exact covariance-bound admission | `verification-pending` | RED `683b28ee...` + causal repair `9eeb373d...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-038 | Exact Cauchy–Schwarz boundary covariance could round one ULP below `±1` and weaken a perfect represented association | `verification-pending` | RED `c2500090...` + causal repair `d06259ec...` + research trace `be5123d5...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-039 | Exact zero lagged covariance could leak IEEE `-0.0` through the public Temporal Association projection and create a bit-distinct directional-looking no-association value | `verification-pending` | RED `e15d0531...` + causal repair `fc61f7bd...` + research trace `b56e1d16...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-040 | Exact zero within-person deviation could leak IEEE `-0.0` through public within/between decomposition and create two bit-distinct identities for one no-deviation estimand | `verification-pending` | RED `aeb008a3...` + causal repair `a9a70baa...` + research trace `ecb90914...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-041 | Individually representable irregular residual log-rates could have a mathematically nonzero one-sign mean that underflowed to exact zero and falsely reported no temporal change | `verification-pending` | RED `96f1c334...` + causal repair `ae5081d8...` + research trace `0280aba0...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-042 | Ratio-first logarithm could nearly double an adjacent-float power-of-two irregular residual growth rate before event-time scaling | `verification-pending` | public RED `766ddc7a...` + causal `ln_1p` repair `16f21d9a...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-043 | Duplicate known-truth component identities could silently reweight RMSE recovery evidence by changing the denominator without adding a new scientific target | `verification-pending` | public RED `698f12f5...` + causal uniqueness repair `2fae4cb2...` + research trace `1106c005...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-044 | Known-truth component RMSE alignment and deterministic binary64 accumulation depended on serialization row order rather than scientific component identity | `verification-pending` | cross-slice RED `8ad72ac9...` + identity-alignment repair `2dd9537e...` + truth-order rounding RED `5fb93c40...` + canonical-order repair `025dce7f...` + edge coverage `976ce7d7...` + research trace/current #310 `1106c005...` / `464df01e...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-045 | Mean signed bias could reject a representable recovery result because finite residuals were summed before dividing | `verification-pending` | public RED `c5ec42e4...` + scaled compensated-mean repair `7499042f...` + research trace `1f22ef67...` + current #488 `956c9c98...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |
| GAP-046 | Bias standard error could reject a representable SEM because raw squared deviations, their sum, or an intermediate sample variance overflowed | `verification-pending` | public RED `7de0ef90...` + scaled direct-SEM repair `cad23162...` + oracle/edge reinforcement `8a6cc346...` / `28d96c23...` + research trace/current #488 `1f22ef67...` / `956c9c98...`; exact-head Rust/documentation/security/review GREEN and protected-main integration |

## Release gate

TEPP currently has no GitHub release. Release is permitted only after a coherent vertical reaches protected main with exact protected-head CI/security/recovery evidence, reproducible package + SBOM + provenance + rollback artifacts, current version/CHANGELOG, required migration/restore/load evidence, and released integration contracts where deployment depends on them. Queued/pending/startup-failed/skipped or predecessor-head evidence is not GREEN.