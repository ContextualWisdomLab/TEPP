# Product and Technical Gap Baseline

**Status:** Active delivery recovery

**Product:** Temporal Event Psychometrics Platform (TEPP)

**Snapshot:** 2026-09-03T01:15Z

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
| #487 | `6b0c8de64f41bc11f8bf908e0f9cbe854c1e213c` | true | #416 `feat/copy-identity-analysis-run-gap-004` | Validation / Analysis Run fold child. Sparse observed relation classes are valid evidence; RED `a2892b6...`, repair `a6402015...`, predecessor-test correction `6b0c8de6...`. No child-head CI transfers to #416. |
| #486 | `c451587e288ba119aebda67addee382106daf670` | true | #310 `agent/psychometric-discrete-drift-std-clean@9def784a...` | Longitudinal fold child, non-force base refreshed to the current survivor. Preserve unique Hamaker source/doctoring; reject wrong-owner `psychometric_core` temporal implementation and mixed-timezone provenance. |
| #485 | `f71591864efc2beff336ced7ef35d5a013305c36` | true | #416 | Analysis Run fold child; preserve support-edge source/tests/doctoring. |
| #484 | `9a1be78b5342ff65e3cf2aac1e9331c68943f246` | true | #416 | Analysis Run fold child; preserve summarizes-edge source/tests/doctoring. |
| #483 | `847d96f913bb261803ac0bd751ad7e4f51324cee` | true | #416 | Analysis Run fold child; preserve unique evidence. |
| #482 | `506dbae236a4484301b704b6c6a05b20faf0fe69` | true | #416 | Analysis Run fold child; preserve unique evidence. |
| #480 | `01f45a99392457334a4f6d3d659f992af739eeee` | true | `main` | contextual-orchestrator consumer repair; stays Draft while immutable CO release/deployment/auth provenance is unavailable. |
| #460 | `dfab4eab5ff733731e565a9348072b8dab2e4912` | true | #416 | Analysis Run fold child; typed cutoff equality and terminal-validation separation. |
| #458 | `08165e3b3c929b4ae77396689549f72723ff8ff5` | true | #416 | Analysis Run fold child; typed cutoff equality and terminal-validation separation. |
| #416 | `0b7155cc238defb1e55129ff3000658f04b343cf` | true | `main` | Validation / Analysis Run landing vehicle; availability cutoff precedes duplicate-identity admission. |
| #310 | `9def784a78bed0c3990f8366f1a7f64d9c64043b` | true | `main` | Longitudinal Modeling vehicle; owner-correct occasion composition and deterministic stable-mean arithmetic. Hosted exact-head verification remains non-passing. |

Exact-head evidence becomes stale after any source push.

## Domain ownership

TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment, leakage-safe knowledge cutoff, temporal recovery, Validation Evidence, and Projection policy. `longitudinal_modeling` is the bounded context and `longitudinal_core` is its current Rust implementation path.

`psychometric_core` is not authority for new temporal/state composition. fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and arithmetic. TEPP consumes only immutable released/versioned Published Language through an ACL; source copying and mutable sibling-head dependencies are prohibited.

contextual-orchestrator owns provider/model routing and semantic LLM execution. Context Graph contracts are contract-only integration authority; EA Core owns enterprise-architecture decisions. No cross-service SQL.

The clock contract separates event/valid time, assertion time, document time, system time, available time, and knowledge cutoff. Retrospective evidence may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective/citation/revision/provenance relations.

## Scientific invariants

- Rasch remains distinct from generic 1PL; formulation-qualified 2PLM–5PLM, MIRT, ideal-point/GGUM, testlet/rater/facet/generalized-mixed identity is preserved.
- Cross-classification and multiple membership remain distinct; weights are explicit, auditable, time-valid, and normalized or model-estimated according to the formulation.
- A nominal unit identifier is not repeated-measures evidence; singleton units cannot satisfy a longitudinal multilevel floor.
- Occasion-mean deviations `p_it = x_it - μ_t` are not CWC residuals, sample-wide grand-mean residuals, or RI-CLPM within-person effects. Numeric event time defines occasion identity, so `-0.0` and `+0.0` are one occasion.
- Row arrival order is not scientific evidence. Fixed admitted observations must produce bit-identical means/centered results under permutation wherever the contract claims deterministic f64 reference behavior.
- A representable final scientific estimand is not rejected solely because an avoidable intermediate binary64 operation overflows/underflows. False exact 0/1/non-finite endpoints remain fail-closed when the mathematical estimand is interior/nonzero.
- Observed Allen support classes are data, not required design strata. A historical prediction census may truthfully have zero covered, partial-overlap, adjacent, or contradictory rows; absent classes remain explicit zero counts rather than invalidating the run.
- Historical-cutoff admission occurs before duplicate-identity checks. Future-unavailable evidence cannot change an earlier run's conflicts, counts, or terminal state.
- Supported temporal estimators require state/trajectory and claimed-structure recovery, bias/RMSE, interval coverage, convergence, uncertainty calibration, and leakage-safe rolling-origin evidence.
- CPU/GPU parity counts only when the relevant accelerator path actually runs. Monte Carlo decisions use simulation uncertainty rather than arbitrary pass percentages.
- Scientific failures are never hidden with skip/xfail/source rewriting/coverage exclusions.

## Current repairs and blockers

### #310 — Longitudinal Modeling

#310 remains the canonical Longitudinal Modeling landing vehicle. Its lineage covers lagged Pearson correlation with both marginal variances, stationary-variance materialization versus algebraically cancelled standardized maps, discrete-diffusion endpoint/subnormal representability, CWC atomistic admission, one irregular-rate authority, known-truth recovery, and DDD relocation from `psychometric_core` to `longitudinal_core`.

The #486 occasion-mean fold is owner-correct on #310: RED `75b0184d...`, owner repair `7fe9aaf2...`, export/test repairs `b900e213...` / `30771ff2...`, determinism RED `8a59019e...`, same-sign ordering repair `465d139d...`, same-panel/sparse successor regressions `b9e952bb...` / `aad56b50...`, ADR `04b3c26e...`, PRD `a221f494...`, and research trace `27aa78ee...`. Exact head `9def784a...` additionally canonicalizes the older CWC/irregular-residual same-sign mean path with `f64::total_cmp`.

At `9def784a...`, CodeQL PR run `33699010954` is `startup_failure` with zero materialized jobs. Rust Foundation CI, Documentation Quality, Security Scan, SAST Semgrep, OSV-Scanner PR, and Scorecard PR are queued. #310 remains Draft and non-passing.

#486 remains open Draft beneath the refreshed #310 base. It can close only after its remaining valid CHANGELOG/operational wording is either inherited owner-correctly or verified redundant; wrong-owner source and mixed `Z KST` provenance are intentionally rejected rather than copied.

### #416 — Validation / Analysis Run consolidation

#416 `0b7155cc...` centralizes the leakage-safe invariant that availability filtering occurs before duplicate-identity admission. #458/#460/#482/#483/#484/#485/#487 are current fold children over shared Cargo/lib/lock/docs surfaces. Their unique evidence must reach a surviving #416 exact head before any child can be considered fully superseded.

#487 exposed a scientific acceptance bug: the first implementation required all four Allen support classes and at least four assignments merely because the test fixture exercised those branches. RED `a2892b6...` requires covered-only and contradiction-only historical censuses to succeed with truthful zero counts. Repair `a6402015...` keeps only real invariants: nonempty admitted evidence, size bound, exact class sums, refusal-plus-covered conservation, cutoff-safe admission, digest/schema validation, and claim boundary. `6b0c8de6...` removes predecessor tests that encoded the rejected fixture-as-production rule. The child remains conflicted beneath #416 and has no materialized PR workflow run at this head, so it is not GREEN and does not transfer evidence to #416.

### #480 — contextual-orchestrator boundary

#480 removes TEPP-owned provider discovery/ranking and requires the contextual-orchestrator owner contract. contextual-orchestrator protected `main@212ff437dc297613289dba2e6064ade9942e07d8` still has no GitHub release. #480 remains Draft/blocked-equivalent; mutable owner `main` is not a released production contract.

### Owner handoffs

fast-mlsirm protected `main@b5a3a0c1057d4b53d7a4bb18e0de69f630c2b45c`; latest immutable release remains `v0.9.1` (2026-08-26). Open owner heads newer than that release are candidate evidence, not TEPP dependency authority.

`context-graph-contracts` remains read-only at protected `develop@99cb5468ba3c15c5e79688f53dee74724fae2d13` with no GitHub release. `enterprise-architecture-core` remains read-only at protected `develop@1c0fa8b15ceb9e72186274aeb255d6777eb84ef4` with no GitHub release.

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
| GAP-018 | Longitudinal stable-mean logic remains duplicated | `active-refactor` | semantic-equivalence proof, one TEPP Longitudinal primitive or released fast-mlsirm generic owner contract, recovery parity |
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
| GAP-029 | Occasion-mean temporal composition arrived in the wrong bounded context with raw-bit event identity, naive mean summation, and order-dependent averaging | `active-fold` | owner-correct #310 lineage through `9def784a...`; verify remaining #486 unique evidence; exact-head GREEN/review/main integration |
| GAP-030 | Prediction-contradiction Analysis Run treated four observed relation classes as mandatory design strata | `active-fold` | RED `a2892b6...` + repair `a6402015...` + predecessor-test correction `6b0c8de6...`; fold unique evidence into #416; exact-survivor GREEN/review/main integration |

## Release gate

TEPP currently has no GitHub release. Release is permitted only after a coherent vertical reaches protected main with exact protected-head CI/security/recovery evidence, reproducible package + SBOM + provenance + rollback artifacts, current version/CHANGELOG, required migration/restore/load evidence, and released integration contracts where deployment depends on them. Queued/pending/startup-failed/skipped or predecessor-head evidence is not GREEN.
