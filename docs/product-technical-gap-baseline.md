# Product and Technical Gap Baseline

**Status:** Active delivery recovery  
**Product:** Temporal Event Psychometrics Platform (TEPP)  
**Snapshot:** 2026-09-19T00:11:00Z  
**Protected-main evidence:** `a243f18da4a4ca8a8d068c39922537f1f8ed6ad0`  
**Workspace version:** `0.2.0`

**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), PR [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435), and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md). Historical source/test/fixture/contract/ADR evidence remains in repository ancestry, `docs/research/`, `docs/TRACEABILITY.md`, and surviving landing vehicles. This file is the current operator register; volatile aggregate PR/issue counts are not promotion evidence and are intentionally omitted.

## Delivery truth

A planning document, mergeable branch, local/source inspection, predecessor-head result, queued/skipped check, ADR number, bot status, or LLM judgment does not make a capability shipped. Only protected-main integration plus current required evidence establishes delivery.

| Authority | Exact current evidence | Delivery implication |
| --- | --- | --- |
| Protected `main` | `a243f18da4a4ca8a8d068c39922537f1f8ed6ad0` | #490 remains the protected product authority until main advances. |
| TEPP releases | **0** | No open TEPP head is a released contract. |
| #435 documentation authority | open / Draft / mergeable; branch `chore/queue-authority-ddd-20260901` | Canonical operator/TRACEABILITY updates belong here; this file intentionally does not self-pin #435's mutable head. |
| #492 central admission consumer | `794ba9e6dda9f043aa499920fdf609b81b075d7e`, Draft, base `main` | Preserved schedule/parser repair remains valid, but immutable `.github` worker policy and contextual-orchestrator `orchestrator/free` release authority must exist before consumer migration and new exact-head evidence. |
| #538 foundation successor | `f335624cea977fea7d7fe6f36871868557fe2bb7`, Ready, base `main` | #594 removes the post-proposal writable cache surface found by exact-head Actions CodeQL. New exact-head checks must reacquire evidence; #498 Dependency Graph support, `.github#2276` authenticated GHAS identity reads, and qualifying independent approval remain promotion prerequisites. |
| #521 persistence successor | `0fa4b71c0e8cd88d95c70c806af648e853ea7b08`, Draft, base #538, ahead 314 / behind 0 | #538's three new ordinary-forward commits briefly left #521 behind and non-mergeable. Merge commit `0fa4b71c...` preserves all #521 history while inheriting the #594 workflow/test deltas without rebase or force push. #590 remains the last accepted persistence production repair; #591 remains retired. |
| #416 Analysis Run survivor / #372 CWC child | #416 `03f8de2ed0a0fb842d2022d411814e440df7cfb4`, Draft; #372 `2a05f4d38f5136d32ab0170249dd3c47cf95949e`, Draft, retargeted to #416 and currently non-mergeable | #372 carries #592/#593/#595 evidence-population integrity and #596's public same-evidence permutation RED. Shared lock/source/docs conflicts must be resolved ordinary-forward into #416 without replacing unrelated profile history or #435-owned shared documentation. #501 remains separate scientific recovery acceptance. |
| #488 Validation Evidence | `520df488fd86ba48008af8ee5a2e112b4587fc22`, Draft, base #492 | Current exact head remains short of owned-production 100% authored line/branch evidence and qualifying approval. |
| #310 Longitudinal Modeling | `ba10820e0d28cc33d1b91ef37f6f6d163b3d91e9`, Draft, base `main` | TEPP-owned temporal/longitudinal semantics remain mutable and unreleased; reusable static arithmetic must arrive only through immutable fast-mlsirm releases. |

Passing, queued, skipped, or predecessor-head checks on an open PR never promote that PR to implemented-main. Organization/admin bypass and self-approval are not normal delivery evidence.

## External scientific owner boundary

fast-mlsirm owns reusable static/generalized-mixed/dependence-aware psychometric specification and arithmetic. TEPP consumes only immutable released/versioned contracts through an ACL; source copying, cross-repository SQL, and mutable sibling-head dependencies are prohibited.

At this snapshot the latest immutable fast-mlsirm release is **v0.11.4**, published `2026-09-18T08:14:12Z`. It predates both current upstream candidate heads below and therefore is not authority for either candidate delta.

| Owner vehicle | Exact current evidence | TEPP implication |
| --- | --- | --- |
| fast-mlsirm #1717 | `4493f58ac02ba16e08e2d8dc6b0e2ef2b2407db8`, open / Ready / mergeable | Exact predecessor `81d3ed2275...` acquired a hosted runner and returned three unsuppressed Semgrep WARNING findings: two dynamic `globals()` namespace lookups in `python/fast_mlsirm/dif.py` and one discovered-module `importlib.import_module(modname)` execution in `tools/inventory_public_api.py`. Ordinary-forward repairs `b2f1cac98...` and `4493f58ac...` remove those dynamic execution surfaces without suppression or gate weakening. Fresh current-head CI/security/CodeQL/Semgrep/fuzz remain pending/queued, so the repair is not yet hosted GREEN. Organization runner-acquisition authority remains `.github#712` for the still-waiting current-head jobs. |
| fast-mlsirm #1816 | `7b853c2e8766c0dd58c3b01064c0dbcad1500493`, open / Draft / mergeable, protected-main compare ahead 19 / behind 0 | Correctly-rounded finite binary64 mean candidate has been ordinary-forward reconciled to `main@a712995b1c22230bc7fcc7f693ae4ad88cb363f4` while preserving exactly six owner deltas. It remains mutable/unreleased; new Security/CodeQL/Semgrep are queued and CI/fuzz are Draft-skipped, so no acceptance transfers. |
| fast-mlsirm release | `v0.11.4`, immutable | Current released owner authority only; it does **not** authorize TEPP to consume #1717/#1816 mutable deltas. |

The next owner-correct scientific sequence is #1717 exact-head hosted GREEN + qualifying review -> normal protected-main merge -> immutable fast-mlsirm release -> #1816 exact-head numerical/oracle/security/review acceptance -> immutable release -> TEPP released-contract bumps for #310 and the #596 CWC cluster-mean consumer. TEPP must not short-circuit that sequence by copying static arithmetic or pinning mutable Git heads.

contextual-orchestrator owns provider/model routing and semantic LLM execution. TEPP consumes only an immutable released owner contract. Model-backed Actions use the released organization gateway contract and `orchestrator/free`; leaf provider keys, provider/model selection, or unpublished owner source are not TEPP production authority.

## Domain ownership

TEPP owns Temporal Semantics, Event Ontology, Temporal Graph composition, irregular event time, time-varying multilevel/cross-classified/multiple-membership semantics, Longitudinal Modeling, leakage-safe knowledge cutoff, temporal recovery, Validation Evidence, and Projection policy. `longitudinal_core` is the current Rust implementation path for Longitudinal Modeling.

The clock contract separates event/valid time, assertion time, document time, system time, availability time, and knowledge cutoff. Retrospective evidence may describe an earlier event but cannot enter an earlier knowledge cutoff. Forward state/transition edges remain distinct from retrospective/citation/revision/provenance relations.

## Scientific and implementation invariants

- Rasch remains distinct from generic 1PL; formulation-qualified 2PLM–5PLM, MIRT, ideal-point/GGUM, testlet/rater/facet/generalized-mixed identity is preserved.
- A nominal unit identifier is not repeated-measures evidence. Stable `Between` known truth is unit-level with canonical `occasion_index = 0`; `Within` retains actual `(unit, occasion)` identity.
- Row arrival order is not scientific evidence. Fixed admitted observations must produce bit-identical results under permutation wherever the deterministic CPU `f64` contract claims it. #596 is a current public RED showing longitudinal CWC cluster means do not yet satisfy this invariant for mixed-sign finite binary64 evidence.
- Historical-cutoff admission occurs before snapshot/domain and duplicate-identity checks. A cutoff-visible foreign-snapshot row fails closed; future-unavailable evidence, including a foreign-snapshot or duplicate-identity row, cannot change an earlier run's conflicts, counts, artifacts, or terminal state.
- Supported temporal estimators require state/trajectory and claimed-structure recovery, bias/RMSE, interval coverage, convergence, uncertainty calibration, reproducibility, Monte Carlo uncertainty, and leakage-safe rolling-origin evidence. Synthetic fixtures are unit-level evidence only and do not replace realistic scientific acceptance.
- Mean signed bias and bias SE are Validation Evidence measures. The bounded exact route remains `neutral_zero_linear -> pairwise_reference -> generic_fallback`; pairwise is a fail-closed comparison path, not the represented-input admission definition.
- The `n=16` Validation Evidence cutoff is an implementation/resource boundary, not a scientific boundary. Wider fixtures are characterization unless a buyer-path cardinality contract and measured resource evidence promote them.
- The mixed-sign witness `[-2^53,0,1,2^53]` retains exact pair numerator `P=2^109+3`; the public route is pinned to `0x432a20bd700c2c3e`.
- `[f64::MAX,-f64::MAX,0,0]` has finite `SE(mean)=f64::MAX/sqrt(6)` and remains pinned to `0x7fda20bd700c2c3d`; a historical `None` expectation is not an accepted fail-closed contract.
- Scientific failures are never hidden through skip/xfail, source rewriting, sample shrinkage, coverage exclusion, or denominator manipulation.
- DDD owner boundaries remain authoritative: scientific/domain truth stays in its canonical owner; consumers use released contracts/ACLs rather than source copies.

## Current Validation Evidence boundary

#488 exact head is `520df488fd86ba48008af8ee5a2e112b4587fc22`. The head is documentation-only relative to its predecessor and does not change numerical arithmetic or coverage-classifier inputs. Exact-head documentation, Bias-SE proof-budget, Live PostgreSQL, Rust format/lint/test/rustdoc/dependency policy, and repository/Python contracts have evidence, but production authored-source coverage remains **12,344/12,346** lines and **4,450/4,452** branch outcomes. The two unresolved authored line/branch locations remain `bias.rs:68-69` and `bias.rs:591-592`.

The first branch is scientifically/source-domain reachable but resource-extreme; it must be closed through a typed buyer-path evidence-to-metric cardinality/resource contract rather than a giant fixture or arbitrary public-library cutoff. The second has exact-real dispersion separation `D >= n/2`, but implemented floating positivity still requires an implementation-matched forward-error proof or a compact caller-valid counterexample. No current-head full GREEN, protected merge, release, or qualifying independent approval is claimed.

Canonical `docs/TRACEABILITY.md` must index the current #491/#488 resource/cardinality obligation, the immutable fast-mlsirm #1717/#1816 -> TEPP #310/#596 owner chain, #595 cutoff-before-snapshot CWC historical-population contract, and #596 permutation-stability contract before this documentation lane can claim fully code-current traceability. `docs/traceability-current-owner-boundaries.md` preserves those four obligations until the safe canonical fold is complete.

## Longitudinal CWC historical-population and numerical boundary

#372 is retargeted directly to surviving Analysis Run vehicle #416. Current child head `2a05f4d38f5136d32ab0170249dd3c47cf95949e` remains Draft and non-mergeable because both branches changed shared lock/source/documentation surfaces; this is a conflict-resolution obligation, not grounds for closing the child or replacing survivor history.

#592 established opaque evidence identity and cutoff-visible duplicate refusal. #593 removed future-only census state from the digest-bound historical artifact. #595 found the provenance side channel where `snapshot_id` was checked before `AvailableTime`; public RED `0ea8f6573d8519762e3ef9038ec4c4892c8e3a8b` preserves visible foreign-snapshot refusal while requiring future-unavailable foreign-snapshot invariance, and causal repair `07360220cf8e7018107b271dc8e6e2c2f49b0c58` moves availability admission ahead of snapshot and identity admission while retaining the raw `MAX_EVIDENCE_UNITS` ceiling. ADR 0033 remains Proposed.

#596 adds a separate numerical owner finding. `psychometric_core::recover_cluster_mean_within_between_slopes` currently accumulates predictor/outcome cluster means in source order with naive `f64` addition. The same cutoff-visible six-row population can change between/contextual decomposition when cluster-1 enumeration changes from `[1e16, 1, -1e16]` to `[1e16, -1e16, 1]`. Public RED `2a05f4d38f5136d32ab0170249dd3c47cf95949e` adds `longitudinal_cwc_permutation_contract.rs` and requires bit-identical artifact/terminal result. TEPP must not repair this with a local summation kernel, sorting heuristic, compensation fork, mutable fast-mlsirm dependency, or tolerance weakening. The RED can turn GREEN only after immutable release of `fast_mlsirm.binary64_mean@1.0.0` and released-contract consumption for both predictor and outcome cluster means.

These integrity and numerical repairs do not satisfy scientific acceptance. #501 still requires repeated true-parameter recovery for within, between, and contextual slopes with bias/RMSE and Monte Carlo uncertainty; attempted/recovered/failed denominators; cluster-size/imbalance/noise/signal variation; unequal follow-up/time-varying availability; reproducibility; and leakage-safe rolling-origin evaluation. The child may close only after #416 or a verified successor inherits all valid #592/#593/#595/#596 source/test/ADR/doctoring/released-owner deltas, #435 folds shared traceability without losing unrelated history, and fresh surviving-head evidence is reacquired.

## Persistence / execution-context boundary

#521 remains a bounded PostgreSQL validator rather than a complete SQL/procedural parser. #564–#590 bind final RLS/policy/table/trigger/append-only/retention routine state and `session_replication_role` across direct `SET`, canonical `set_config`, writable `pg_settings.setting`, CTE/data-modifying CTE, row assignment, opaque immediate `DO`/`CALL`, Unicode-escaped identifiers, and persistent role/user/database/system defaults.

#590 is the last accepted production repair: whitespace around schema qualification could alias sibling relation RLS final state, so unsupported qualified identities fail closed. #591 then proposed rejecting `ALTER TABLE ... SET SCHEMA`; review showed TEPP has no schema-bound owner/ACL/search_path invariant and relation-attached RLS/policy/trigger state survives schema movement. That hypothesis was therefore retired ordinary-forward without a production fix. The architectural endpoint remains canonical relation identity, a shared PostgreSQL Unicode identifier decoder, and first-class final RLS/execution/default-state aggregates with explicit RESET/DEFAULT/FROM CURRENT precedence.

The foundation advance for #594 created an ordinary stack-maintenance obligation rather than a new persistence finding. Before repair #521 compared as ahead 313 / behind 3 and GitHub reported it non-mergeable. `0fa4b71c...` is a two-parent ordinary-forward merge of prior #521 head `2e3c57bc...` and current #538 `f335624...`; the resulting compare is ahead 314 / behind 0. This keeps the open stack coherent but is not the later post-#538 protected-main restack or acceptance evidence.

## Foundation / CI security boundary

#538's predecessor head exposed two exact CodeQL Actions findings in the hourly generated-proposal verifier. After the immutable proposal patch was applied, two `actions/cache@v5` steps restored Rust quality binaries into `~/.cargo/bin`; cache post-actions could persist proposal-mutated bytes under stable default-branch keys. #594 records RED `af3e0bc...`, corrected RED `24241780...`, and causal repair `f335624...`: the verifier now has a structural regression forbidding `actions/cache@` in that trust boundary and installs the same version-pinned `cargo-nextest`, `cargo-deny`, and `cargo-llvm-cov` directly before proposal code runs. The repair does not suppress CodeQL or weaken artifact identity, credential stripping, quality gates, coverage, or publication separation.

The same canonical dispatch proved a separate owner-path failure: Python SARIF was clean but `.github` could not read TEPP `code-scanning/analyses`, receiving HTTP 403 from the integration while checking GHAS base/head configuration identity. `.github#2276` owns that cross-repository permission repair and must preserve fail-closed semantics. #498 independently remains the repository-admin prerequisite for Dependency Graph/dependency-review support. Neither external prerequisite is grounds to bypass #538 gates.

Current required jobs also remain in the organization-level pre-checkout runner-acquisition class. TEPP #538 Rust Foundation jobs are queued with `runner_id=0`, empty runner identity and `steps=[]`; fast-mlsirm #1717's new exact-head gates are likewise waiting in pending/queued state. The predecessor #1717 Semgrep run did acquire a hosted runner and returned the three real source findings recorded above, so runner starvation and source-level RED are distinct diagnoses. Canonical queue diagnosis remains `.github#712`; no leaf no-op wake commit, blind rerun, label downgrade, or predecessor receipt is valid acceptance evidence.

## Gap register

| ID / vehicle | Gap / state | Current closure evidence |
| --- | --- | --- |
| GAP-001–044 | Existing product/domain/DDD/release and Longitudinal gaps; mixed/inherited | Exact histories remain in ancestry, surviving landing vehicles, ADRs/tests, and research/TRACEABILITY. |
| GAP-045–125 | Validation Evidence arithmetic/recovery lineage; verification pending | Exact source/tests/research remain inherited on #488; current authored line/branch evidence is 12,344/12,346 and 4,450/4,452. |
| #491 / #488 | Resource/cardinality and translated-dispersion floating-proof obligations remain open | Buyer-path metric cardinality must be typed/versioned before Analysis Run admission can narrow public-library resource proofs; exact-real separation alone does not remove the floating positivity guard. |
| #492 | Central admission consumer migration is incomplete | Wait for immutable `.github` worker and contextual-orchestrator `orchestrator/free` authority, then migrate leaf workflow/guidance and reacquire exact-head gates. |
| #538 / #594 | Foundation successor security repair is implemented but not yet promoted | Exact-head `f335624...` must reacquire Rust/docs/security/Semgrep/CodeQL evidence proving the cache-poisoning findings are gone. #498 Dependency Graph, `.github#2276` GHAS analyses-read authority, `.github#712` runner acquisition, and qualifying independent approval remain prerequisites. |
| #521 | Persistence successor verification pending | Open stack is coherent at `0fa4b71c...` (ahead 314 / behind 0). After #538 lands normally, non-force restack surviving persistence delta from protected main, then reacquire Rust/Python/docs/security/CodeQL, Live PostgreSQL, rustdoc and authored line/branch/edge evidence. |
| #416 / #372 / #592 / #593 / #595 / #596 | Longitudinal CWC Analysis Run population-integrity and numerical-owner deltas await conflict-resolving successor fold | #372 `2a05f4d3...` is retargeted to #416. Preserve cutoff-before-snapshot/identity admission, visible provenance refusal, future-only invariance, no future census field, public same-evidence permutation RED, tests, ADR Proposed state and doctoring while resolving shared survivor conflicts ordinary-forward. #596 remains RED until the released fast-mlsirm finite-mean owner is consumed. |
| #501 | Longitudinal CWC commercial scientific acceptance remains open | Add realistic repeated true-parameter trajectory/state recovery evidence with RMSE/bias/MC uncertainty, explicit failure denominators, cluster/follow-up/time variation, reproducibility and leakage-safe rolling-origin evaluation after the #596 numerical owner migration. Deterministic unit fixtures are insufficient. |
| #310 | Longitudinal consumer blocked on released static arithmetic | Consume only a new immutable fast-mlsirm release after #1717 then #1816 land with owner acceptance; the same released numerical owner also gates #596. |

## Release gate

TEPP currently has no GitHub release. Release is permitted only after a coherent vertical reaches protected main with exact protected-head CI/security evidence, scientific/recovery acceptance, reproducible package/build artifacts with SBOM and provenance, validated migrations/upgrade/rollback/recovery where applicable, consistent version metadata and current CHANGELOG/TRACEABILITY/operator baseline, accessibility/operability evidence for user-facing components, no unresolved scientific/privacy/security/supply-chain blockers, and released integration contracts where deployment depends on them. Queued/pending/startup-failed/skipped or predecessor-head evidence is not GREEN.