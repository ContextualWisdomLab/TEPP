# Product and Technical Gap Baseline

**Status:** Active delivery recovery

**Product:** Temporal Event Psychometrics Platform (TEPP)

**Snapshot:** 2026-09-02T16:13Z

**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`

**Workspace version:** `0.2.0`

**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md).

**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md) and [`docs/architecture/temporal-dependence-composition.md`](architecture/temporal-dependence-composition.md).

## Delivery truth

A planning document, mergeable branch, local test, predecessor-head result, queued/skipped check, ADR number, or LLM judgment does not make a capability shipped. Only protected-main integration plus current required evidence establishes delivery.

| Signal | Fresh evidence | Implication |
| --- | ---: | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | Capability claims remain bounded to this commit until main advances. |
| Open pull requests | **131** | WIP circuit breaker remains active. |
| Draft pull requests | **130** | Draft work must consolidate/repair rather than independently land. |
| Non-Draft pull requests | **1** | #480 is the only non-Draft PR and is not deployable without a compatible immutable contextual-orchestrator release. |
| Open issues | **16** | Includes ADR normalization, orchestrator admission, and dynamic-evaluation drift evidence work. |
| GitHub releases | **0** | No TEPP open head is a released contract. |
| Organization ruleset | `18156473` | One qualifying current-head approval, stale-review dismissal after push, resolved threads, unattributed-change approval where applicable, and central required workflows. |

#484 `summarizes_edge_v1` is a #416 Analysis Run fold child, not an independent bounded-context landing authority. Its unique source/tests/doctoring must survive the eventual shared-file fold; it is not closed merely to reduce queue count.

## Current priority open pull-request evidence

#435 intentionally omits its own SHA from this file because embedding a branch head inside a file changed by that branch makes the file self-stale.

| PR | Exact current head | Draft | Base | Disposition |
| ---: | --- | :---: | --- | --- |
| #484 | `9a1be78b5342ff65e3cf2aac1e9331c68943f246` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Analysis Run fold child; preserve profile-specific source/tests/doctoring. |
| #483 | `847d96f913bb261803ac0bd751ad7e4f51324cee` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Analysis Run fold child; preserve unique evidence. |
| #482 | `506dbae236a4484301b704b6c6a05b20faf0fe69` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Analysis Run fold child; preserve unique evidence. |
| #480 | `4475542750eda01afad0cf9ea8d563f508f63fd3` | false | `main@1bc02f580cf48e1d39da239f0e818453437c31c3` | Independent consumer-side LLM-governance repair; released CO plus deployment/auth provenance required. |
| #460 | `dfab4eab5ff733731e565a9348072b8dab2e4912` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Fold child; typed cutoff equality and terminal validation separation preserved. |
| #458 | `08165e3b3c929b4ae77396689549f72723ff8ff5` | true | #416 `feat/copy-identity-analysis-run-gap-004@0b7155cc238defb1e55129ff3000658f04b343cf` | Fold child; typed cutoff equality and terminal validation separation preserved. |
| #416 | `0b7155cc238defb1e55129ff3000658f04b343cf` | true | `main@1bc02f580cf48e1d39da239f0e818453437c31c3` | Validation / Analysis Run landing candidate; availability cutoff precedes duplicate-identity admission. |
| #310 | `440ac6902d86dce48fedd1229ac71ab8c133bdf0` | true | `main@1bc02f580cf48e1d39da239f0e818453437c31c3` | Longitudinal Modeling vehicle; nonzero irregular-rate underflow fails closed while exact no-change remains zero. |

Exact-current-head evidence becomes stale after source mutation or any new commit.

## Domain ownership

TEPP owns temporal/event composition, irregular time, time-varying multilevel/cross-classified/multiple-membership semantics, longitudinal invariance/drift/alignment, leakage-safe knowledge cutoff, temporal recovery and projection policy. fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and arithmetic, including LSIRM/MLSIRM/DLSJM kernels. contextual-orchestrator owns provider/model routing and semantic LLM execution. Context Graph contracts are contract-only integration authority; EA Core owns enterprise-architecture decisions. No source copying, mutable sibling dependency, or cross-service SQL.

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

Current head `440ac6902d86dce48fedd1229ac71ab8c133bdf0` adds a fail-closed contract for an irregular residual log-rate that is mathematically nonzero but smaller than binary64 can represent after division by an extreme positive event interval. RED `4d1031092388f6237d99fe29149c9fbccf453d1c` uses `earlier=1.0`, the next representable `later`, and `Δt=f64::MAX`; predecessor source returned `Ok(0.0)`, conflating nonrepresentable change with exact no-change. Repair `021130aa09584c56d77c91a2b090c34da4a484b9` rejects represented zero when residual magnitudes differ. Edge coverage `440ac6902d86dce48fedd1229ac71ab8c133bdf0` proves equal residual magnitudes still return exact zero.

Earlier numerical lineages remain active: CWC RED `9a706c3c0e9e0db68e88f89b94c64c13ea7fafd0` / repair `260413efb9d95039b5fbba41919cba8097fcf8b5`; known-truth RMSE overflow/precision repair ending at `312dbcd25a7246683d5596385571068d475fc4c3`; within/between stable-mean repair `6bf3661bf390e0de00e1bb83539e6d62ee06b85f`; stationary-variance fallback `(q * 0.5) / |a|`; and two-marginal lagged Pearson correlation with covariance-bound admission.

The current head is mergeable but remains Draft. CodeQL PR ends as `startup_failure` before any job is materialized; Rust Foundation CI, Security Scan, Documentation Quality, SAST Semgrep, Scorecard and OSV are queued/pending. There is no qualifying current-head independent approval. No merge is authorized until current ruleset evidence passes.

### #416 — Validation / Analysis Run consolidation

Current head `0b7155cc238defb1e55129ff3000658f04b343cf` centralizes the leakage-safe invariant established by RED `ffee655404716bf8d33c898a3c1a87a543abe701`: availability filtering occurs before duplicate-identity admission. #458/#460/#482/#483/#484 remain fold children over shared Cargo/lib/lock/docs surfaces. Their unique evidence must reach a surviving #416 head before any child can be considered fully superseded.

### #480 / #479 — contextual-orchestrator boundary

#480 removes TEPP-owned provider discovery/ranking and requires HTTPS `contextual-orchestrator/orchestrator/free` from an immutable owner release. contextual-orchestrator protected `main@212ff437dc297613289dba2e6064ade9942e07d8` has **zero GitHub releases**. Mutable branch state is not a released contract. The consumer remains deliberately fail-closed.

### fast-mlsirm owner handoff

fast-mlsirm protected `main@b5a3a0c1057d4b53d7a4bb18e0de69f630c2b45c`; generalized-mixed/dependence compiler #1714 remains Ready/mergeable at `92a3f2152033b61ca89661b5ba8a584842e8c3a9`. Immutable `v0.9.1` predates #1714. Exact #1714 CI is pending/queued and CodeQL PR is startup-failed. TEPP does not consume the new Published Language until a compatible immutable release and ACL/parity evidence exist.

### #437 — ADR identity

Repository-wide ADR IDs are immutable authority. Duplicate index IDs/targets/numbered files, index/file mismatch and repeated Decision-status or Implementation-maturity authority must fail deterministic documentation fitness. Adapter/model micro-slice ADR numbers remain implementation lineage pending normalization through #435.

## External contract state

`context-graph-contracts` protected/default `develop@99cb5468ba3c15c5e79688f53dee74724fae2d13` has **14 open PRs, 2 open issues, and zero releases**. `enterprise-architecture-core` protected/default `develop@1c0fa8b15ceb9e72186274aeb255d6777eb84ef4` has **24 open PRs, 2 open issues, and zero releases**. Both remain read-only from this TEPP writer; open heads are candidate evidence rather than production contracts.

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
