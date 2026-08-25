# Product and Technical Gap Baseline

**Status:** Live delivery baseline
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-08-25T04:24:53Z
**Protected-main evidence:** `0e7479c96c080036deed14a5925bb0ca715fa524` (full SHA fetched live before every mutation) | Signal | Snapshot evidence | Delivery implication |
|---|---:|---|
| Protected-main SHA | `5c8599442e85…` (2026-08-25T02:12Z, merge of #215) | All as-built claims are bounded to this commit. |
| Workspace members | 48 unique Rust crates | The repository is modular, but the approved target still lacks complete semantic, estimator, compute, psychometric, event-intelligence, network, interpretation, artifact, and visual product boundaries. |
| Open pull requests | **33** | The consolidation program drained all main-conflicting slices; the residual queue is 31 stacked drafts plus two non-draft slices and the coverage repairs in flight. |
| Draft pull requests | **31** | Most queued work is not independently review-ready; the psychometric recovery stack dominates. |
| Non-draft pull requests | **0** | A non-draft state is not a qualifying review or required-check result. |
| Open product issues | **10** | Issue #156 plus open product-completion issues #166–#169 and #171–#176. |
| Duplicate gap-baseline PRs | **#164** (authority); **#179** closed as superseded | Only one live register remains; queued Checks on #164 are not protected-main evidence. |
| Current package version | `0.1.0` | No supported product release is established by the repository version alone. |

The pull-request counts come from the live GitHub search at this snapshot. The
full exact-head classification is owned by
[issue #175](https://github.com/ContextualWisdomLab/TEPP/issues/175); this file
keeps the operator-level summary while the exact-head register below records the
volatile queue (live counts above). Passing or queued Checks on an open PR never
promote that PR to implemented-main.

### Queue-consolidation progress (GAP-012) — COMPLETE

The main non-draft pull-request queue reached **zero** at 2026-08-25T02:30Z.
All previously queued slices landed on protected main through:

1. the hourly scheduler (independent merges of ~40 PRs);
2. batch integration vehicle [#215](https://github.com/ContextualWisdomLab/TEPP/pull/215) (31 folded green slices with per-slice merge-commit provenance); and
3. individual rebase-and-admin-merge passes for the remainder.

The 33 remaining open PRs are all **drafts** forming stacked psychometric
recovery chains (T0VAR/drift/diffusion standardised parameter families).
They follow the stacking plan and are not independent landing vehicles.

### Historical consolidation notes

The hourly scheduler plus one batch integration vehicle drained the
main-conflicting green queue, and the vehicle itself is now merged:

- [PR #215](https://github.com/ContextualWisdomLab/TEPP/pull/215) **merged at
  2026-08-25T02:12:17Z** as protected-main `5c8599442e85…`, landing the 19
  remaining folded slices (#51 #57 #60 #61 #68 #70 #72 #85 #113 #139 #141
  #143 #145 #146-content #148 #150 #151 #152 #153) with per-slice
  merge-commit provenance. Folded PR heads whose exact commits are ancestors
  of main auto-close as merged; advanced heads (for example the newer
  #146 head) remain live work.
- The scheduler independently landed #110, #114, #115, #118–#128, #130,
  #133–#135, #137–#138, and #164 while #215 was in Checks; #215 was refreshed
  onto that main and re-verified before merge.
- Transient Strix provider failures (`provider/backend unavailable`) on
  #48/#49/#63/#65/#137/#149/#157 were re-run rather than treated as code
  defects.
- Central control-plane repair landed on `.github` main: commit `8fd471a3`
  aligned the required-path Strix smoke assertions with the `gpt-5.4`
  fallback contract after `a724582` replaced the nonexistent
  `gpt-5.6-luna` model. New Strix runs resolve the fixed smoke.
- Post-merge branch-coverage audit of the vehicle found two production
  branches (`validate_entity_label`/`validate_project_label` uppercase-label
  clause) and three Python arcs in `scripts/check_coverage.py` uncovered;
  [PR #219](https://github.com/ContextualWisdomLab/TEPP/pull/219) carries the
  exact red-to-green tests and must land before the next coverage-gated slice.

Remaining queue after consolidation: strix-rerun set (#48 #49 #63 #65 #157),
pending-check slices (#58 #62 #66), advanced-head #146, toolchain bump #214,
coverage repair #219, and the 49 draft PRs retained per the stacking plan,
including the psychometric recovery stack and non-psychometric drafts.

## Snapshot open pull-request evidence

The following exact-head register was fetched live from GitHub at
2026-08-25T04:24:53Z. Review decisions, required Checks, and mergeability remain
volatile; the live GitHub API supersedes this snapshot. `draft=false` is not approval, mergeability, or a
passing-check claim. Re-read the full SHA, current review decision, required
Checks, and branch rules immediately before every mutation.

| PR | Exact current head | Draft | Base | Title |
|---:|---|:---:|---|---|
| #92 | `56fadb1d99be2478f86c7dd493d9240f4f7af349` | false | main | feat(orchestrator): serve interpretation POSTs on a loopback HTTP listener |
| #132 | `426c2d38e8d46914b3d6743aa616bd3383481715` | false | main | feat(relation): refuse unobserved pairs as no-relationship |
| #181 | `96e8d1c21924124d144f6273893eda8ac6b41b2d` | true | agent/psychometric-posterior-esem-input | feat(psychometric): recover Driver later-occasion variance of predetermined T0VAR |
| #182 | `ee999e1ed23327a75d9cf4eb09a4652561f07cab` | true | agent/psychometric-predetermined-later-t0var | feat(psychometric): recover Driver lagged covariance of predetermined T0VAR |
| #183 | `c10097be0f6f4ff8dd8e6db21dce8818630fe17f` | true | agent/psychometric-predetermined-lagged-t0var | feat(psychometric): recover Driver first-occasion variance of predetermined T0VAR |
| #184 | `6b93147e668ff414240f3ea94997f2937c0817f1` | true | agent/psychometric-predetermined-initial-t0var | feat(psychometric): recover Driver later-start lagged covariance of predetermined T0VAR |
| #185 | `69ffec6425863b2842094ff1b1d6c07d8eee6b5b` | true | agent/psychometric-predetermined-later-lagged-t0var | feat(psychometric): recover Driver later-start later-occasion variance of predetermined T0VAR |
| #187 | `6afd048d1a78bd7e1f68555182ec6a1f0db9ea8c` | true | agent/psychometric-predetermined-later-start-later-t0var | feat(psychometric): recover Driver p.16 discreteDRIFTstd after positive asymDIFFUSION |
| #188 | `444304d2d702cceb47cd4fdf695ba630d7e7c83a` | true | agent/psychometric-standardised-discrete-drift | feat(psychometric): recover Driver p.16 discreteDIFFUSIONstd after positive asymDIFFUSION |
| #189 | `0836c359fab7878084ac3a711920550eb6cacb0d` | true | agent/psychometric-standardised-discrete-diffusion | feat(psychometric): recover Driver p.16 DIFFUSIONstd after positive asymDIFFUSION |
| #190 | `adb0bc6f2a1b0684eb9242bcb0d6cddd1ff09ae3` | true | agent/psychometric-standardised-continuous-diffusion | feat(psychometric): recover Driver p.16 DRIFTstd after positive asymDIFFUSION |
| #192 | `1690c7b65693cf27c8f1730150d67084594aef07` | true | agent/psychometric-standardised-continuous-drift | feat(psychometric): recover Driver p.16 asymTIPREDEFFECTstd after positive variances |
| #193 | `6235a3fc6fd6899f85f74aed71053bd50529c34c` | true | agent/psychometric-standardised-asymptotic-tipred-effect | feat(psychometric): recover Driver p.16 finite-interval TIPREDEFFECTstd after positive variances |
| #194 | `471b115e891be11fb41eb75a5cc3073258ab0f34` | true | agent/psychometric-standardised-asymptotic-tipred-effect | feat(psychometric): recover Driver p.16 TIPREDEFFECTstd after positive variances |
| #195 | `cf275b25611953bd405550f96df257c77df01751` | true | agent/psychometric-standardised-continuous-tipred-effect | feat(psychometric): recover Driver p.16 CINTstd after positive asymDIFFUSION |
| #196 | `1cb91b43002765bf5b855be51e55cb29db41a599` | true | agent/psychometric-standardised-continuous-tipred-effect | feat(psychometric): recover Driver Table 3 T0TIPREDEFFECTstd after positive T0VAR |
| #197 | `a19ea1ecba3d773470f12ed4dbbeb8a01b987f26` | true | agent/psychometric-standardised-initial-tipred-effect | feat(psychometric): recover Driver Table 3 T0TDPREDEFFECTstd after positive T0VAR |
| #198 | `52a9b4ceb26bdd89e6e49c56fb473d96c68ca352` | true | agent/psychometric-standardised-initial-tipred-effect | feat(psychometric): recover Driver 2017-era addedT0TIPREDVAR after T0TIPREDEFFECT |
| #199 | `7a476ed3c39e4ba14c9c51f5cbbafc33310979d2` | true | agent/psychometric-added-t0-tipred-var | feat(psychometric): recover Driver Eq. 5 of addedT0TIPREDVAR extra observed TI variance |
| #200 | `8f429471e2b249e76c9b722eaf2443518b666c98` | true | agent/psychometric-initial-tipred-observed-variance | feat(psychometric): recover Driver Eq. 5 of addedTIPREDVAR extra observed TI variance |
| #202 | `2d2d3efe5fd2f742a335e071f22fc1738c51ab27` | true | agent/psychometric-asymptotic-tipred-observed-variance | feat(psychometric): recover Driver p.16 TDPREDEFFECTstd after positive variances |
| #203 | `e5a63d0c7ffac91e9945f6c483152a0bdf267525` | true | agent/psychometric-asymptotic-tipred-observed-variance | feat(psychometric): recover Driver 2017-era addedTIPREDVARstd after addedTIPREDVAR |
| #204 | `ac757e5e4f3651cad98fa88d5a44db570eb1c3cf` | true | agent/psychometric-standardised-continuous-tdpred-effect | feat(psychometric): recover Driver Table 3 T0TDPREDEFFECTstd after positive T0VAR |
| #205 | `db60fe4eb47abf1efb793ccef37597689d11d09a` | true | agent/psychometric-standardised-initial-tdpred-std | feat(psychometric): recover Driver p.16 T0VARstd after positive T0VAR |
| #206 | `dcb339f3e2e5ac2dd2dda9b61b5d2c9b811c7636` | true | agent/psychometric-standardised-initial-tdpred-std | feat(psychometric): recover analog of addedT0TIPREDVAR for first-occasion TD extra |
| #207 | `5dd0c01549c93fa05f57df5658058244c7b675e4` | true | agent/psychometric-standardised-initial-latent-variance | feat(psychometric): recover Driver p.16 TRAITVARstd after positive TRAITVAR |
| #208 | `adcd68a582725130c046239753290d6cb5f3b8b7` | true | agent/psychometric-standardised-trait-variance | feat(psychometric): recover Driver p.16 MANIFESTTRAITVARstd after positive Ψ_τ |
| #209 | `5e3ef38256f42ce09a934eb0f3893e02a40ad6b7` | true | agent/psychometric-added-initial-tdpred-var | feat(psychometric): recover Eq. 5 of analog addedT0TIPREDVAR extra observed TD variance |
| #210 | `22202c57c0818d3efb28438e95bdde5d0e7143b0` | true | agent/psychometric-standardised-manifest-trait-variance | feat(psychometric): recover Driver p.16 MANIFESTVARstd after positive Θ |
| #211 | `643b9beb76ea6bc6b3a468bdb0f08e1a820cae9f` | true | agent/psychometric-standardised-manifest-variance | feat(psychometric): recover Driver p.16 TIPREDVARstd after positive TIPREDVAR |
| #213 | `d32c069c5653434302dc2507e272ff916cb7b16a` | true | agent/psychometric-standardised-asymptotic-diffusion | feat(psychometric): recover Driver p.16 discreteCINTstd after positive p |
| #216 | `91903db726f1da6b398e0a32c5a401126598154f` | true | agent/psychometric-standardised-discrete-intercept | feat(psychometric): recover Driver p.16 asymCINTstd after positive p |
| #218 | `19279547f5a8622d9584d95822ec36ae3830beeb` | true | agent/psychometric-standardised-asymptotic-intercept | feat(psychometric): recover Driver p.16 T0MEANSstd after positive T0VAR |

Review decisions, required Checks, and mergeability remain volatile; re-read
them immediately before every mutation. This snapshot is not merge authorization
and does not treat queued or passing Checks as shipped protected-main behavior.

## Authority and derivation

| Concern | Canonical authority | Constraint |
|---|---|---|
| Product outcomes | [`docs/product/prd-v0.4-approved.md`](product/prd-v0.4-approved.md) | Defines the release product, users, visual surfaces, scientific claims, and eight delivery phases. |
| Technical/runtime requirements | [`docs/TRD.md`](TRD.md) | Requires independently usable Rust boundaries, CPU `f64` authority, temporal eligibility, realistic validation, and warning-free release evidence. |
| Architecture and service boundaries | [`ARCHITECTURE.md`](../ARCHITECTURE.md), [`docs/UML.md`](UML.md), [`docs/API_CONTRACT.md`](API_CONTRACT.md) | Separates evidence, measurement, compute, psychometrics, event intelligence, interpretation, artifacts, and visual analytics. |
| Data authority | [`docs/ERD.md`](ERD.md), [`docs/TRACEABILITY.md`](TRACEABILITY.md), [`docs/adr/0013-bitemporal-persistence-reproducibility-and-split-authority.md`](adr/0013-bitemporal-persistence-reproducibility-and-split-authority.md) | Requires normalized persistence, six-clock eligibility, relation-aware splits, immutable provenance, and reproducible artifacts. |
| Scientific claim promotion | [`docs/adr/0014-scientific-claim-promotion-and-release-evidence.md`](adr/0014-scientific-claim-promotion-and-release-evidence.md), [`docs/TEST_STRATEGY.md`](TEST_STRATEGY.md) | Requires production-code recovery, uncertainty, parity, exact-head checks, and independent review before promotion. |
| LLM authority | [`docs/adr/0010-adaptive-llm-orchestration.md`](adr/0010-adaptive-llm-orchestration.md), [`docs/LLM_ORCHESTRATION.md`](LLM_ORCHESTRATION.md) | LLMs may propose and verify interpretations; deterministic/statistical gates remain authoritative. |
| Privacy/security/assurance | [`docs/PRIVACY_DATA_GOVERNANCE.md`](PRIVACY_DATA_GOVERNANCE.md), [`SECURITY.md`](../SECURITY.md), [`docs/THREAT_MODEL.md`](THREAT_MODEL.md), [`docs/COMPLIANCE_READINESS.md`](COMPLIANCE_READINESS.md) | Preserves legitimate PII utility through purpose-bound access while prohibiting credential/source leakage and unsupported certification claims. |
| Research | [`docs/research/standards-and-literature.md`](research/standards-and-literature.md) | Method and standards claims require current authoritative sources and APA 7 traceability. |
| Live delivery | [open PRs](https://github.com/ContextualWisdomLab/TEPP/pulls?q=is%3Apr+is%3Aopen), [open issues](https://github.com/ContextualWisdomLab/TEPP/issues?q=is%3Aissue+is%3Aopen) | Live GitHub state supersedes this time-stamped queue snapshot. |

## Protected-main as-built baseline

Protected `main` contains 48 unique Rust crate boundaries in the current
workspace manifest. The `members` and `default-members` arrays enumerate the
same crate set for distinct Cargo commands; the unique-crate count is the
authoritative modularity measure.
The core boundaries include:

```text
evidence_core
temporal_core
event_core
relation_graph
membership_core
persistence_postgres
corpus_split
tepp_simulation
validation_core
tepp_api
```

The traceability ledger records meaningful protected-main implementation in
immutable evidence, six clocks and interval reasoning, forward transitions,
event mention/instance separation, weighted multiple membership, cutoff-safe
splits, validation metrics, simulations, PostgreSQL slices, versioned API/export
contracts, orchestration routing, privacy authorization, and release-evidence
generation.

Protected `main` does **not** yet establish the complete approved product. In
particular, it does not contain the full multilingual semantic pipeline, a
shared-latent temporal topic estimator, a production longitudinal ESEM/DSEM
estimator, a calibrated TDT/CHRONOS workflow, a posterior network estimator,
real accelerator kernels, a production interpretation gateway, the coordinated
visual workspace, or a supported multi-tenant release.

## Operator-gap register

| ID | Operator-visible gap | Maturity | Delivery status | Protected-main authority | Current delivery authority | Current head SHA | Closure evidence |
|---|---|---|---|---|---|---|---|
| GAP-001 | Submission can produce a durable accepted receipt, but protected `main` lacks the separate deterministic terminal-result lifecycle. | `active-PR` | landing vehicle | `e65cd66` (accepted receipt only) | [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) / [PR #157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | `340087494b0a` | Exact request/result/snapshot/cutoff/model/profile binding, typed terminal failures, deterministic retrieval, current-head checks, and qualifying review. Cutoff-safe execution from closed stacked PR #178 is on this head only and is not implemented-main. |
| GAP-002 | LineageWeave and other modular consumers cannot yet rely on the complete protected-main HTTP evidence/result boundary. | `partial` | landing vehicle | `e65cd66` (LineageWeave consumer ingress and versioned API contract are implemented-main; durable terminal-result completion remains separate) | [PR #155](https://github.com/ContextualWisdomLab/TEPP/pull/155) (merged) | `—` | Complete the durable result lifecycle under #156/#157 and preserve versioned consumer contracts; closed stacked PRs #158 and #159 are provenance, not additional implementation authority. |
| GAP-003A | Immutable evidence cannot yet be submitted to a durable validation run that produces operator-usable scientific acceptance evidence. | `accepted-target` | product-completion | `e65cd66` (validation metrics are library-level only) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | `—` (issue program; no current implementation PR) | Compose/CLI/API execution must bind immutable evidence, cutoffs, model configuration, validation metrics, and reproducibility manifests to one idempotent run. |
| GAP-003B | Scientific result artifacts cannot yet be persisted, restarted, and recovered as one supported operator workflow. | `accepted-target` | product-completion | `e65cd66` (persistence contracts lack E2E recovery) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | `—` (issue program; no current implementation PR) | Durable storage, migration/rollback, restart/recovery, artifact digest verification, and terminal retrieval must pass against a real Compose deployment. |
| GAP-003C | The persistence slice classifies concurrent-write SQLSTATEs, but has no measured hot-partition detection, routing, or mitigation for tenant/result workloads. | `accepted-target` | product-completion | `e65cd66` (conflict classification only; no measured partition control) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | `—` (issue program; no current implementation PR) | A real Compose/PostgreSQL workload identifies hot keys and partition skew, applies bounded tenant/time or result routing without weakening 3NF or temporal authority, and proves conflict rate, latency, recovery, and migration/rollback behavior under load. |
| GAP-004 | The central shared-latent temporal/relational topic estimator is absent. | `accepted-target` | product vertical | `e65cd66` (no production estimator) | [#167](https://github.com/ContextualWisdomLab/TEPP/issues/167) / [PR #48](https://github.com/ContextualWisdomLab/TEPP/pull/48) | `6110d3660607` | Rust CPU `f64` fitting, sparse bounded parallelism, posterior artifacts, convergence, true-parameter RMSE/bias/coverage, and real candidate-K fitting. |
| GAP-005 | Real multilingual documents are not yet transformed into validated exact-span semantic units and versioned shared concepts. | `partial` | product vertical | `e65cd66` (semantic_core exact-span units and language-profile validation are implemented-main as the first slice) | [#168](https://github.com/ContextualWisdomLab/TEPP/issues/168) / [PR #201](https://github.com/ContextualWisdomLab/TEPP/pull/201) (merged) | `—` | Closure still requires concept alignment, Unicode/layout/language-tailored processing, unknown-concept review, multilingual calibration/invariance, image-position evidence, and prompt-injection tests. |
| GAP-006 | Posterior topic measurements cannot yet be fitted through a complete cross-classified longitudinal ESEM/DSEM engine. | `accepted-target` | product vertical | `e65cd66` (temporal and membership primitives only) | [#169](https://github.com/ContextualWisdomLab/TEPP/issues/169) / [PR #119](https://github.com/ContextualWisdomLab/TEPP/pull/119) | `47ab763d49b1` | Plausible-value/joint uncertainty, invariance, irregular event time, within/between separation, multiple membership, true-parameter recovery, and causal-claim refusal. |
| GAP-007 | TDT detection/tracking and CHRONOS schema/forecast/temporal reasoning remain isolated bounded gates rather than one calibrated product workflow. | `accepted-target` | product vertical | `e65cd66` (event/time primitives only) | [#170](https://github.com/ContextualWisdomLab/TEPP/issues/170) / [PR #70](https://github.com/ContextualWisdomLab/TEPP/pull/70) | `7a1f33aa68c1` | Span-grounded mentions, calibrated TDT metrics, schema/forecast hypothesis states, interval consistency, known-truth recovery, persistence, and exports. |
| GAP-008 | GPU support is policy-only; no production estimator kernel has real hardware parity or declared VRAM evidence. | `accepted-target` | product vertical | `e65cd66` (VRAM policy only) | [#171](https://github.com/ContextualWisdomLab/TEPP/issues/171) / [PR #51](https://github.com/ContextualWisdomLab/TEPP/pull/51) | `1801501c4d7c` | Real CUDA/portable backend execution, CPU parity, streamed memory, bounded OOM/fallback, hardware profiles, telemetry, and no skipped-support claim. |
| GAP-009 | Topic association and cluster outputs lack posterior-valid estimation, uncertainty, edge stability, and consensus communities. | `accepted-target` | product vertical | `e65cd66` (network primitives only) | [#172](https://github.com/ContextualWisdomLab/TEPP/issues/172) / [PR #71](https://github.com/ContextualWisdomLab/TEPP/pull/71) | `2588f38281b9` | Valid log-ratio coordinates, interval/stability-bearing edges, repeated Leiden consensus, known-truth network/cluster recovery, and reproducible exports. |
| GAP-010 | Operators lack coordinated accessible visual analytics and exact-value export workflows. | `accepted-target` | product vertical | `e65cd66` (no visual workspace) | [#173](https://github.com/ContextualWisdomLab/TEPP/issues/173) | `—` (Figma work not started) | Real Figma File ID in ADR, Storybook/design tokens, ten PRD views, exact-value tables, accessible interaction/print/PDF states, provenance, and source-consistent exports. |
| GAP-011 | TEPP is not yet an operable multi-tenant service or supported release. | `accepted-target` | product vertical | `e65cd66` (library contracts only) | [#174](https://github.com/ContextualWisdomLab/TEPP/issues/174) | `—` (issue program; no current implementation PR) | Durable queue/storage, OIDC/RLS/purpose controls, OpenTelemetry/SLOs, load/recovery, migrations, signed release/SBOM/provenance, assurance evidence, and support policy. |
| GAP-012 | The 71-PR queue obscured authority, repeatedly staled exact-head evidence, and fragmented product boundaries. | `partial` | release blocker (draining) | `5c8599442e85` (vehicle #215 merged 2026-08-25T02:12Z) | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) | `—` (live queue program) | Residual queue: nine non-draft slices, coverage repair #219, and the 49-draft stack. Remaining PRs classified; superseded work closed with provenance; scheduler prioritizes consolidation; queue reaches zero before GA. |
| GAP-013 | Evidence-grounded LLM interpretation is routed but not executed and validated as a production interpreter/verifier port. | `partial` | active integration | `e65cd66` (routing and refusal contracts only) | [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176), [PR #69](https://github.com/ContextualWisdomLab/TEPP/pull/69), [PR #165](https://github.com/ContextualWisdomLab/TEPP/pull/165) | `8e4a3ca9cc80` / `34083c3f5d66` | Contextual-orchestrator execution, evidence citations, verifier refusals, comparable-budget ablations, provider eligibility/fallback, abstention, live/offline contract tests, and no numerical-authority escalation. |
| GAP-014 | README/TRD and some PR descriptions lag protected-main and live queue reality. | `partial` | documentation drift | `e65cd66` (documentation is not fully synchronized) | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) | `—` (queue-consolidation work) | Reconcile README, TRD, traceability, ADR maturity, CHANGELOG, preferred-merge declarations, and exact protected-main evidence. |
| GAP-015 | There was no canonical live product/operator-gap register tied to documentation validation. | `implemented-main` (register + validator); this refresh is the live maintenance slice | register refresh | `5c8599442e85` (register and validator are implemented-main; this snapshot refresh keeps the volatile queue current) | [PR #217](https://github.com/ContextualWisdomLab/TEPP/pull/217) merged; [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164) is the merged authority; closed [PR #179](https://github.com/ContextualWisdomLab/TEPP/pull/179) is not a second register | `—` (each refresh PR carries its own current head at merge time) | Land each refresh after exact-head checks and independent review, then regenerate it whenever protected-main or the live queue changes. |
| GAP-018 | The #215 consolidation vehicle landed with two uncovered production branches in `persistence_postgres` label validation and three uncovered Python arcs in `scripts/check_coverage.py`. | `active-PR` | coverage repair | `5c8599442e85` (gates still enforce 100%; the gaps are test debt, not weakened gates) | [PR #219](https://github.com/ContextualWisdomLab/TEPP/pull/219) | branch `fix/coverage-union-gaps-post215` | Exact red-to-green cases: uppercase-label clause for entity/project validators, blank-history comma-scan arcs, escaped-char-literal scanner arc. Closure requires exact-head Checks plus independent review on protected main. |
| GAP-016 | Hourly PR maintenance used an older central scheduler revision whose per-repository sweep budgets could amplify the queued review workload. | `active-PR` | operability hardening | `e65cd66` (caller pin before central budget hardening) | [PR #177](https://github.com/ContextualWisdomLab/TEPP/pull/177) | `580d45206536` | The change pins a verified central revision immutably; closure still requires exact-head hosted Checks, resolved threads, and independent review. |
| GAP-017 | Accepted analysis runs have a terminal DTO, and cutoff-safe execution exists only on the #157 head after stacked PR #178 merged into that branch. | `active-PR` | product vertical | `e65cd66` (accepted receipt only) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) / [PR #157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | `340087494b0a` | Exact availability cutoff, snapshot binding, multiple-membership preservation, digest integrity, redacted no-eligible failure, and realistic end-to-end tests on protected main. Closed stacked PR #178 is not implemented-main. |

## Product-completion issue register

| Issue | Product vertical | Depends on / constrains |
|---:|---|---|
| [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) | Completed analysis-run result contract | Must land before consumers can call accepted work a measurement result. |
| [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | Executable end-to-end analysis run, recovery, and hot-partition readiness | Integrates all scientific/service verticals; cannot substitute placeholders or hide write skew behind an unmeasured queue. |
| [#167](https://github.com/ContextualWisdomLab/TEPP/issues/167) | Shared-latent temporal topic CPU estimator | Numerical foundation for K selection, networks, psychometrics, interpretation, and product E2E. |
| [#168](https://github.com/ContextualWisdomLab/TEPP/issues/168) | Multilingual semantic units and concept dictionary | Supplies validated span-grounded estimator evidence. |
| [#169](https://github.com/ContextualWisdomLab/TEPP/issues/169) | Multilevel longitudinal ESEM/DSEM | Consumes posterior topic coordinates and membership/time contracts. |
| [#170](https://github.com/ContextualWisdomLab/TEPP/issues/170) | TDT/CHRONOS event intelligence | Consumes evidence/time/event contracts and supplies calibrated event artifacts. |
| [#171](https://github.com/ContextualWisdomLab/TEPP/issues/171) | Real GPU compute and parity | Accelerates production estimators only after CPU authority is stable. |
| [#172](https://github.com/ContextualWisdomLab/TEPP/issues/172) | Posterior network and consensus clustering | Depends on a real fitted topic posterior. |
| [#173](https://github.com/ContextualWisdomLab/TEPP/issues/173) | Accessible visual analytics and exports | Starts after stable API/artifact contracts; requires Figma and Storybook evidence. |
| [#174](https://github.com/ContextualWisdomLab/TEPP/issues/174) | Commercial deployment/release/support | Wraps a scientifically complete product without weakening gates. |
| [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) | PR queue and delivery consolidation | Immediate release-system priority; prevents further unbounded fragmentation. |
| [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176) | Contextual-orchestrator interpreter/verifier | Consumes validated artifacts and cannot promote scientific truth. |

## Priority pull-request queue

This table is intentionally non-exhaustive. Issue #175 owns the full exact-head
inventory and classification. The pull request's live page is authoritative
because its head can change after this file is committed.

| PR | Current delivery role | Required next action |
|---:|---|---|
| [#219](https://github.com/ContextualWisdomLab/TEPP/pull/219) | Post-#215 branch-coverage repair (two Rust label-validation branches, three Python scanner arcs) | Exact-head Checks plus independent review, then merge before the next coverage-gated slice. |
| [#157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | Terminal result contract for #156, including folded cutoff-safe execution from closed #178 | Complete exact-head review/check gates; keep #156 open until protected-main verification. |
| [#146](https://github.com/ContextualWisdomLab/TEPP/pull/146) | Advanced head of the episode-boundary slice beyond the folded vehicle content | Re-run exact-head Checks on `aac43e83`; land only the delta over protected main. |
| [#66](https://github.com/ContextualWisdomLab/TEPP/pull/66) | TDT mention-link precision/recall scoring | Re-run exact-head Checks; fold into event-intelligence landing under #170. |
| [#65](https://github.com/ContextualWisdomLab/TEPP/pull/65) | First-story detection FAR/miss-rate scoring | Re-run exact-head Checks; fold into event-intelligence landing under #170. |
| [#63](https://github.com/ContextualWisdomLab/TEPP/pull/63) | Refuse TF-IDF/BM25 as inferential weights | Re-run exact-head Checks; connect to estimator weights under #167/#172. |
| [#62](https://github.com/ContextualWisdomLab/TEPP/pull/62) | Simulation excludes delayed documents before availability | Re-run exact-head Checks; keep temporal eligibility authority intact. |
| [#58](https://github.com/ContextualWisdomLab/TEPP/pull/58) | Embedded image URIs as positional non-lexical units | Re-run exact-head Checks; feeds GAP-005 image-position evidence. |
| [#49](https://github.com/ContextualWisdomLab/TEPP/pull/49) | Posterior ESEM input gates with true-parameter RMSE | Re-run exact-head Checks; connect to psychometric engine under #169. |
| [#48](https://github.com/ContextualWisdomLab/TEPP/pull/48) | Logistic-normal/ALR coordinate slice, including folded stacked #191 TRSL lineage artifacts | Resolve conflict/dirty state onto current main; coordinates and lineage artifacts are not candidate-K fitting. |
| [#74](https://github.com/ContextualWisdomLab/TEPP/pull/74)–[#218](https://github.com/ContextualWisdomLab/TEPP/pull/218) drafts | Draft stacked temporal/privacy/method/psychometric recoveries | Keep stacked; convert to ready one link at a time from the stack root upward; never promote drafts or predecessor heads as protected-main. Closed stacked #191 landed on #48 only. |

## Delivery sequence

The dependency-aware product order is:

1. **Consolidate delivery authority:** #175 and PR #164.
2. **Finish live result contracts:** #156/#157; the LineageWeave consumer parent #155 is implemented-main (loopback PR #107 and closed stacked #158/#159/#178 remain provenance).
3. **Build validated multilingual evidence:** #168, extending the first implemented-main span slice from #201.
4. **Build the CPU topic estimator:** #167.
5. **Build event intelligence and posterior networks:** #170 and #172.
6. **Build the posterior-aware longitudinal psychometric engine:** #169.
7. **Accelerate real kernels with parity:** #171.
8. **Complete the durable end-to-end run:** #166.
9. **Execute and validate interpretation:** #176.
10. **Design and implement the operator workspace:** #173.
11. **Productionize and release:** #174.

Stacking is appropriate where public contracts make dependencies explicit.
Stacking is not a reason to leave multiple unexplained implementation authorities
or stale draft predecessors open.

## Definition of product complete

TEPP is not complete until one released version proves all of the following on
the same protected source lineage:

- a documented user can install, authenticate, ingest, run, inspect, export, and
  recover the product without repository-internal intervention;
- immutable source evidence, six clocks, relation/membership structure, cutoff,
  splits, model/config, backend, seeds, and artifacts are reproducible;
- the shared-latent topic estimator and longitudinal psychometric model recover
  declared known truth with pre-registered RMSE, bias, coverage, convergence,
  calibration, and error-rate gates;
- declared language profiles have span/concept/alignment/invariance evidence;
- event intelligence, topic networks, and clusters have known-truth and
  uncertainty/stability evidence;
- accelerator claims use real hardware and match the CPU scientific reference;
- LLM interpretation cites allowed evidence, rejects unsupported claims, and
  abstains when evidence or policy is insufficient;
- every visual value has an accessible exact-value and provenance path;
- tenant, purpose, identity, retention, security, migration, backup/restore,
  observability, capacity, rollback, SBOM, provenance, and support evidence pass;
- production statement coverage, branch coverage, and public documentation are
  100% for shipped TEPP code;
- current-head CI, security, supply-chain, scientific, and independent review
  gates pass with no unresolved release blocker;
- version, CHANGELOG, signed artifacts, and release notes match the protected
  source and make no unsupported certification, causality, language, GPU, or
  valuation claim;
- the release-blocking PR and issue queues are zero.

A `200억 달러` bar remains a prioritization heuristic. It is not a valuation
result and cannot replace operator adoption, predictive/construct validity,
operational reliability, proprietary advantage, revenue, retention, or
independent diligence evidence.

## Architecture, data, and assurance constraints

- Rust owns production mathematical and psychometric arithmetic.
- CPU `f64` is the numerical reference; parallelism is bounded and GPU work must
  prove real-hardware parity.
- Event, assertion, document, system, availability, and knowledge-cutoff clocks
  remain distinct.
- Cross-classified and weighted multiple membership prevents atomistic
  pseudo-replication.
- Topic proportions remain compositional; valid latent/log-ratio coordinates
  feed ESEM and network analysis.
- Database objects use descriptive two-or-more-word `snake_case`, third-normal
  form where applicable, explicit tenant/temporal/provenance authority. Hot
  partition readiness is a separate acceptance gate: measure skew first, then
  mitigate it without denormalizing authority tables or changing temporal
  semantics.
- Documents, web/search results, connector data, and LLM output are untrusted.
- Purpose-bound access and protected identity mappings preserve PII utility
  without broadcasting or blanket masking.
- External products integrate through versioned API/event/artifact contracts,
  never direct application-table access.
- CSAP/SOC 2/ISO/NIST alignment is readiness evidence, not certification.
- Every method/standard decision updates APA 7 traceability and source-to-test
  traceability in the same reviewed change.

## Refresh rule

Refresh this file when any of the following changes materially:

- protected-main SHA or package version;
- open PR/draft/issue counts;
- a priority PR head/base/review/check/merge state;
- an issue or operator-gap acceptance boundary;
- a capability's implementation maturity;
- the dependency/landing order;
- a release, deprecation, replacement, Figma file, or standards/research basis.

Keep this file operator-oriented. Store the volatile per-PR classification in the
artifact required by issue #175, and link it here. Never rewrite an active-PR
capability as protected-main before merge and exact-head verification.
