# Product and Technical Gap Baseline

**Status:** Active delivery recovery

**Product:** Temporal Event Psychometrics Platform (TEPP)

**Snapshot:** 2026-09-02T23:07Z

**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`

**Workspace version:** `0.2.0`

**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md).

## Delivery truth

A planning document, mergeable branch, local/source inspection, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Only protected-main integration plus current required evidence establishes delivery.

| Signal | Fresh evidence | Implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until main advances. |
| Open pull requests | **133** | WIP circuit breaker remains active; consolidate into existing bounded-context vehicles. |
| Draft pull requests | **133** | Every current open PR is Draft. |
| Non-Draft pull requests | **0** | No current PR is eligible for normal merge without a deliberate Ready transition after exact-head evidence. |
| Open issues | **16** | ADR normalization, orchestration admission, evaluation drift, and scientific recovery remain open. |
| GitHub releases | **0** | No TEPP open head is a released contract. |
| Organization ruleset | `18156473` | One qualifying current-head approval, stale-review dismissal after push, resolved threads, extra approval for unattributed changes where applicable, and central required workflows. |

Ruleset `18156473` currently permits merge/squash and prohibits deletion/non-fast-forward updates on the default branch. Organization-admin bypass exists but is not normal delivery evidence and is not used by this writer.

## Current landing authority

#435 intentionally omits its own branch SHA from this file because embedding a mutable self-head would make the file stale on every edit.

| PR | Exact current head | Draft | Base | Disposition |
| ---: | --- | :---: | --- | --- |
| #486 | `c451587e288ba119aebda67addee382106daf670` | true | #310 `agent/psychometric-discrete-drift-std-clean` | Longitudinal fold child. Its Hamaker source/doctoring remains evidence to inherit, but its `psychometric_core` temporal implementation is non-canonical. Do not close until every unique delta is verified on the surviving #310 head. |
| #485 | `f71591864efc2beff336ced7ef35d5a013305c36` | true | #416 `feat/copy-identity-analysis-run-gap-004` | Analysis Run fold child; preserve support-edge refusal/source/tests/doctoring and historical-cutoff evidence. |
| #484 | `9a1be78b5342ff65e3cf2aac1e9331c68943f246` | true | #416 `feat/copy-identity-analysis-run-gap-004` | Analysis Run fold child; preserve profile-specific source/tests/doctoring. |
| #483 | `847d96f913bb261803ac0bd751ad7e4f51324cee` | true | #416 `feat/copy-identity-analysis-run-gap-004` | Analysis Run fold child; preserve unique evidence. |
| #482 | `506dbae236a4484301b704b6c6a05b20faf0fe69` | true | #416 `feat/copy-identity-analysis-run-gap-004` | Analysis Run fold child; preserve unique evidence. |
| #480 | `01f45a99392457334a4f6d3d659f992af739eeee` | true | `main` | LLM-consumer governance repair; correctly Draft while immutable contextual-orchestrator release/deployment/auth provenance is unavailable. |
| #460 | `dfab4eab5ff733731e565a9348072b8dab2e4912` | true | #416 `feat/copy-identity-analysis-run-gap-004` | Fold child; typed cutoff equality and terminal-validation separation preserved. |
| #458 | `08165e3b3c929b4ae77396689549f72723ff8ff5` | true | #416 `feat/copy-identity-analysis-run-gap-004` | Fold child; typed cutoff equality and terminal-validation separation preserved. |
| #416 | `0b7155cc238defb1e55129ff3000658f04b343cf` | true | `main` | Validation / Analysis Run landing candidate; availability cutoff precedes duplicate-identity admission. |
| #310 | `7baff4c99473b1de035386e4a5055a4fe71edca0` | true | `main` | Longitudinal Modeling vehicle; owner-correct occasion-mean composition, signed-zero identity, representable-mean overflow repair, and Hamaker trace are now on the surviving branch; exact-head hosted verification remains non-passing. |

Exact-head evidence becomes stale after any source push.

## Domain ownership

TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment, leakage-safe knowledge cutoff, temporal recovery, Validation Evidence, and Projection policy. `longitudinal_modeling` is the bounded context and `longitudinal_core` is its current Rust implementation path.

`psychometric_core` is not the authority for new temporal/state composition. It retains existing measurement/legacy compatibility surfaces while explicit adapters are formed. fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and arithmetic. TEPP consumes only immutable released/versioned Published Language through an ACL; source copying and mutable sibling-head dependencies are prohibited.

contextual-orchestrator owns provider/model routing and semantic LLM execution. Context Graph contracts are contract-only integration authority; EA Core owns enterprise-architecture decisions. No cross-service SQL.

The clock contract separates event/valid time, assertion time, document time, system time, available time, and knowledge cutoff. Retrospective evidence may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective/citation/revision/provenance relations.

## Scientific invariants

- Rasch remains distinct from generic 1PL; formulation-qualified 2PLM–5PLM, MIRT, ideal-point/GGUM, testlet/rater/facet/generalized-mixed identity is preserved.
- Cross-classification and multiple membership remain distinct; weights are explicit, auditable, time-valid, and normalized or model-estimated according to the formulation.
- A nominal unit identifier is not repeated-measures evidence; singleton units cannot satisfy a longitudinal multilevel floor.
- Occasion-mean deviations `p_it = x_it - μ_t` are not CWC residuals, sample-wide grand-mean residuals, or RI-CLPM within-person effects. Numeric event time defines occasion identity, so `-0.0` and `+0.0` are one occasion.
- A representable final scientific estimand is not rejected solely because an avoidable intermediate binary64 operation overflows/underflows. A final false 0/1/non-finite boundary remains fail-closed where the mathematical estimand is interior/nonzero.
- TEPP composes time over the full released upstream candidate identity; auto-expansion never means auto-activation.
- Supported temporal estimators require state/trajectory and claimed-structure recovery, bias/RMSE, interval coverage, convergence, uncertainty calibration, and leakage-safe rolling-origin evidence.
- CPU/GPU parity counts only when the relevant accelerator path actually runs. Monte Carlo decisions use simulation uncertainty rather than arbitrary pass percentages.
- Scientific failures are never hidden with skip/xfail/source rewriting/coverage exclusions.

## Current repairs and blockers

### #310 — Longitudinal Modeling

#310 remains the canonical Longitudinal Modeling landing vehicle. Existing repair lineage covers lagged Pearson correlation with both marginal variances, stationary-variance materialization versus algebraically cancelled standardized maps, discrete-diffusion endpoint/subnormal representability, CWC atomistic admission, one irregular-rate authority, known-truth recovery, and DDD relocation from `psychometric_core` to `longitudinal_core`.

The newest fold repairs #486's occasion-mean slice test-first:

- RED `75b0184d2f6341ef23cf14fc84398c68d8d95d22` requires numeric signed-zero occasion identity, rejects a duplicate unit hidden behind `-0.0`/`+0.0`, preserves the known exact scalar residual log-rate, and pins `[0.75·MAX, 0.75·MAX, -0.5·MAX]` as a representable occasion-mean case that naive same-sign partial summation would overflow.
- Repair `7fe9aaf2570ffb6ecff3d6a83b12a30865fc198b` implements occasion-mean event-time composition in `crates/longitudinal_core/src/occasion_mean.rs`. It canonicalizes numeric zero, requires at least two distinct units per occasion and at least two lag-contributing units, retains typed positive finite event intervals, cancels opposing magnitudes before bounded same-sign averaging, and reuses the existing Longitudinal exact-log-rate boundary.
- Export repair `b900e21301f1f5bb769464a4b76da9088cd669ab` publishes the boundary through `longitudinal_core`; `30771ff24cf85479eb5ed227789b59489ac7ead2` fixes the typed interval accessor in the regression contract.
- Research trace `7baff4c99473b1de035386e4a5055a4fe71edca0` adds `docs/research/occasion-mean-event-time-composition.md` with the Hamaker et al. (2015) estimand boundary and test trace, without the mixed `Z KST` provenance string present on #486.

At exact head `7baff4c99473b1de035386e4a5055a4fe71edca0`, CodeQL PR run `33693410413` is `startup_failure` with zero materialized jobs. Rust Foundation CI, Documentation Quality, Security Scan, SAST Semgrep, OSV-Scanner PR, and Scorecard PR are queued. No qualifying independent current-head approval exists. #310 stays Draft and is not merge-ready.

#486 remains open Draft and conflict-exposing beneath #310. Its wrong-owner implementation must not be independently landed. The child may be closed only after its remaining unique documentation/contract/TRACEABILITY delta is proven inherited by a surviving #310 head; the mixed `2026-09-03T06:07Z KST` provenance text remains a child repair finding rather than a reason for evidence-losing closure.

### #416 — Validation / Analysis Run consolidation

#416 `0b7155cc238defb1e55129ff3000658f04b343cf` centralizes the leakage-safe invariant that availability filtering occurs before duplicate-identity admission. #458/#460/#482/#483/#484/#485 remain fold children over shared Cargo/lib/lock/docs surfaces. Their unique evidence must reach a surviving #416 head before any child can be considered fully superseded.

### #480 — contextual-orchestrator boundary

#480 removes TEPP-owned provider discovery/ranking and requires the contextual-orchestrator owner contract. contextual-orchestrator currently has no immutable GitHub release for this consumer path, so #480 remains Draft/blocked-equivalent. Mutable owner `main` is not a released production contract.

### Owner handoffs

Reusable static/generalized-mixed/dependence psychometric arithmetic remains fast-mlsirm-owned. The currently released fast-mlsirm version must be compared with open owner work before any TEPP adoption; an open PR head is never dependency authority.

`context-graph-contracts` and `enterprise-architecture-core` remain read-only from this TEPP writer. Their open heads are candidate evidence only until immutable releases exist.

### #437 — ADR identity

Repository-wide ADR IDs are immutable authority. Duplicate index IDs/targets/numbered files, index/file mismatch, and repeated Decision-status or Implementation-maturity authority must fail deterministic documentation fitness. Branch-local micro-slice records do not mint independent architecture authority.

## Gap register

| ID | Gap | Maturity | Closure evidence |
| --- | --- | --- | --- |
| GAP-001 | PR authority fragmented across 133 open PRs | `release-blocking` | coherent landing vehicles, unique-evidence preservation, protected-main reduction |
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
| GAP-019 | Longitudinal scientific instructions contradicted stationary-overflow implementation | `verification-pending` | RED `9d8a82d...` + repair `9c962205...`; exact-head Documentation Quality/review GREEN and protected-main integration |
| GAP-020 | Nonzero lagged covariance can be misreported as exact-zero correlation when standardized magnitude is unrepresentable | `verification-pending` | RED `c345ee7b...` + repair `5785e07a...`; exact-head Rust/documentation/review GREEN and protected-main integration |
| GAP-021 | Longitudinal irregular-rate facade duplicated public wrapper identities over one canonical implementation | `verification-pending` | RED `464863860...` + repair `7f0bea084...`; exact-head Rust/documentation/review GREEN and protected-main integration |
| GAP-022 | Architecture assigned Longitudinal Modeling semantics to `psychometric_core` and duplicated implementation responsibility rows | `verification-pending` | RED `fe5eb745...` + repair `7fadc757...`; exact-head documentation/quality review GREEN and protected-main integration |
| GAP-023 | `discreteDIFFUSIONstd` rejected a representable subnormal final ratio because `aΔt` underflowed before the required factor two | `verification-pending` | RED `d5107b198...` + repair `7164c7ce4...`; exact-head Rust/documentation/review GREEN and protected-main integration |
| GAP-024 | Contributor guidance re-authorized a direct provider credential after LLM ownership moved to contextual-orchestrator | `verification-pending` | RED `4248b335...` + repair `01f45a993...`; released CO adoption, exact-head documentation/security/review GREEN, protected-main integration |
| GAP-025 | A singleton unit could satisfy the nominal CWC unit floor while all lag evidence came from one repeated unit | `verification-pending` | RED `671709bbc...` + repair `4784b370c...`; exact-head Rust/documentation/review GREEN and protected-main integration |
| GAP-026 | Scalar standardized longitudinal maps rejected representable finals when a cancelled stationary-variance intermediate lay outside binary64 range | `verification-pending` | RED `4a1f6c49...` / `96d8ed13...` + repairs `a4bc6230...` / `33f4b187...` / `26b03c32...`; exact-head GREEN and protected-main integration |
| GAP-027 | Finite-interval `discreteDIFFUSIONstd` could report exact unit diffusion after exponent overflow/saturation erased a nonzero remainder | `verification-pending` | RED `a8de3c9f...` + repair `c17e2ff8...`; exact-head GREEN and protected-main integration |
| GAP-028 | Actual stationary variance `p` could be misreported as exact zero when positive real `p` lies below binary64 range | `verification-pending` | RED `27d9fa39...` + repair `a0132b62...`; exact-head GREEN and protected-main integration |
| GAP-029 | Occasion-mean temporal composition arrived in the wrong bounded context with raw-bit event identity and naive mean summation | `active-fold` | RED `75b0184d...`; owner repair `7fe9aaf2...`; export/test repairs `b900e213...` / `30771ff2...`; research trace `7baff4c9...`; verify remaining #486 unique evidence inheritance; exact-head GREEN/review/main integration |

## Release gate

TEPP currently has no GitHub release. Release is permitted only after a coherent vertical reaches protected main with exact protected-head CI/security/recovery evidence, reproducible package + SBOM + provenance + rollback artifacts, current version/CHANGELOG, required migration/restore/load evidence, and released integration contracts where deployment depends on them. Queued/pending/startup-failed/skipped or predecessor-head evidence is not GREEN.
