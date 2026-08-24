# Product and Technical Gap Baseline

**Status:** Live delivery baseline
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-08-24T10:55:23Z
**Protected-main evidence:** `e65cd663fc0802d6b70ff88d895a8077a1e572ae` (full SHA fetched live before every mutation)
**Workspace version on protected main:** `0.1.0`
**Canonical gap-baseline authority:** [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164). [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164) merged; this file is now maintained by follow-up refresh PRs against protected main. Duplicate [PR #179] stays closed.

## Purpose

This document is the executable operator-gap register for TEPP. It separates:

- capabilities an operator can use from protected `main`;
- bounded work that exists only on open pull requests;
- product-completion issues with measurable acceptance evidence; and
- release claims that remain prohibited.

A planning document, local test, queued check, predecessor-head result, LLM
judgment, or mergeable branch does not make a capability shipped. Re-read live
GitHub state before any customer, release, certification, or valuation claim.

## Snapshot facts

| Signal | Snapshot evidence | Delivery implication |
|---|---:|---|
| Protected-main SHA | `e65cd663fc08…` (2026-08-24T10:11Z) | All as-built claims are bounded to this commit. |
| Workspace members | 47 unique Rust crates | The repository is modular, but the approved target still lacks complete semantic, estimator, compute, psychometric, event-intelligence, network, interpretation, artifact, and visual product boundaries. |
| Open pull requests | **71** | The queue is itself a release blocker and requires consolidation. |
| Draft pull requests | **48** | Most queued work is not independently review-ready. |
| Non-draft pull requests | **23** | A non-draft state is not a qualifying review or required-check result. |
| Open product issues | **11** | Issue #156 plus open product-completion issues #166–#169 and #171–#176. |
| Duplicate gap-baseline PRs | **#164** (authority); **#179** closed as superseded | Only one live register remains; queued Checks on #164 are not protected-main evidence. |
| Current package version | `0.1.0` | No supported product release is established by the repository version alone. |

The pull-request counts come from the live GitHub search at this snapshot. The
full exact-head classification is owned by
[issue #175](https://github.com/ContextualWisdomLab/TEPP/issues/175); this file
keeps the operator-level summary while the exact-head register below records the
volatile queue (live counts above). Passing or queued Checks on an open PR never
promote that PR to implemented-main.

### Queue-consolidation progress (GAP-012)

The hourly scheduler plus one batch integration vehicle are draining the
main-conflicting green queue:

- [PR #215](https://github.com/ContextualWisdomLab/TEPP/pull/215) folds 31 fully
  green but main-conflicting slices into one exact-head landing vehicle with
  per-slice merge-commit provenance and local `cargo test`/quality-suite
  verification at its head.
- The scheduler independently landed #110, #114, #115, #118–#128, #130,
  #133–#135, #137–#138, and #164 while #215 was in Checks; #215 was refreshed
  onto that main and re-verified.
- Transient Strix provider failures (`provider/backend unavailable`) on
  #48/#49/#63/#65/#137/#149/#157 were re-run rather than treated as
  code defects.

Remaining after consolidation lands: strix-rerun set (#48 #49 #63 #65 #149 #157),
pending-check slices (#58 #59 #62 #66), conflicted #131, toolchain bump #214, and
the 48 draft PRs retained per the stacking plan, including the psychometric
recovery stack and non-psychometric drafts.

## Snapshot open pull-request evidence

The following exact-head register was fetched live from GitHub at
2026-08-24T10:55:23Z. Review decisions, required Checks, and mergeability remain
volatile; the live GitHub API supersedes this snapshot. `draft=false` is not approval, mergeability, or a
passing-check claim. Re-read the full SHA, current review decision, required
Checks, and branch rules immediately before every mutation.

| PR | Exact current head | Draft | Base | Title |
|---:|---|:---:|---|---|
| #217 | `79df32c73b260838fd3d0bfed3e84ff3e58e4b28` | false | main | docs(gap): refresh baseline snapshot, consolidation progress; untrack codegraph artifacts |
| #216 | `91903db726f1da6b398e0a32c5a401126598154f` | true | agent/psychometric-standardised-discrete-intercept | feat(psychometric): recover Driver p.16 asymCINTstd after positive p |
| #215 | `f813ce51e3749a91958c1e075dc8d0824f811af0` | false | main | chore(delivery): consolidate 31 queued green slices into one landing vehicle (#175) |
| #214 | `5c5621c27bdf18d52f6f98b7ac373eb90768fbc5` | false | main | build(deps): bump rust-toolchain from 1.97.1 to 1.98.0 |
| #213 | `d32c069c5653434302dc2507e272ff916cb7b16a` | true | agent/psychometric-standardised-asymptotic-diffusion | feat(psychometric): recover Driver p.16 discreteCINTstd after positive p |
| #211 | `643b9beb76ea6bc6b3a468bdb0f08e1a820cae9f` | true | agent/psychometric-standardised-manifest-variance | feat(psychometric): recover Driver p.16 TIPREDVARstd after positive TIPREDVAR |
| #210 | `22202c57c0818d3efb28438e95bdde5d0e7143b0` | true | agent/psychometric-standardised-manifest-trait-variance | feat(psychometric): recover Driver p.16 MANIFESTVARstd after positive Θ |
| #209 | `5e3ef38256f42ce09a934eb0f3893e02a40ad6b7` | true | agent/psychometric-added-initial-tdpred-var | feat(psychometric): recover Eq. 5 of analog addedT0TIPREDVAR extra observed TD variance |
| #208 | `adcd68a582725130c046239753290d6cb5f3b8b7` | true | agent/psychometric-standardised-trait-variance | feat(psychometric): recover Driver p.16 MANIFESTTRAITVARstd after positive Ψ_τ |
| #207 | `5dd0c01549c93fa05f57df5658058244c7b675e4` | true | agent/psychometric-standardised-initial-latent-variance | feat(psychometric): recover Driver p.16 TRAITVARstd after positive TRAITVAR |
| #206 | `dcb339f3e2e5ac2dd2dda9b61b5d2c9b811c7636` | true | agent/psychometric-standardised-initial-tdpred-std | feat(psychometric): recover analog of addedT0TIPREDVAR for first-occasion TD extra |
| #205 | `db60fe4eb47abf1efb793ccef37597689d11d09a` | true | agent/psychometric-standardised-initial-tdpred-std | feat(psychometric): recover Driver p.16 T0VARstd after positive T0VAR |
| #204 | `ac757e5e4f3651cad98fa88d5a44db570eb1c3cf` | true | agent/psychometric-standardised-continuous-tdpred-effect | feat(psychometric): recover Driver Table 3 T0TDPREDEFFECTstd after positive T0VAR |
| #203 | `e5a63d0c7ffac91e9945f6c483152a0bdf267525` | true | agent/psychometric-asymptotic-tipred-observed-variance | feat(psychometric): recover Driver 2017-era addedTIPREDVARstd after addedTIPREDVAR |
| #202 | `2d2d3efe5fd2f742a335e071f22fc1738c51ab27` | true | agent/psychometric-asymptotic-tipred-observed-variance | feat(psychometric): recover Driver p.16 TDPREDEFFECTstd after positive variances |
| #200 | `8f429471e2b249e76c9b722eaf2443518b666c98` | true | agent/psychometric-initial-tipred-observed-variance | feat(psychometric): recover Driver Eq. 5 of addedTIPREDVAR extra observed TI variance |
| #199 | `7a476ed3c39e4ba14c9c51f5cbbafc33310979d2` | true | agent/psychometric-added-t0-tipred-var | feat(psychometric): recover Driver Eq. 5 of addedT0TIPREDVAR extra observed TI variance |
| #198 | `52a9b4ceb26bdd89e6e49c56fb473d96c68ca352` | true | agent/psychometric-standardised-initial-tipred-effect | feat(psychometric): recover Driver 2017-era addedT0TIPREDVAR after T0TIPREDEFFECT |
| #197 | `a19ea1ecba3d773470f12ed4dbbeb8a01b987f26` | true | agent/psychometric-standardised-initial-tipred-effect | feat(psychometric): recover Driver Table 3 T0TDPREDEFFECTstd after positive T0VAR |
| #196 | `1cb91b43002765bf5b855be51e55cb29db41a599` | true | agent/psychometric-standardised-continuous-tipred-effect | feat(psychometric): recover Driver Table 3 T0TIPREDEFFECTstd after positive T0VAR |
| #195 | `cf275b25611953bd405550f96df257c77df01751` | true | agent/psychometric-standardised-continuous-tipred-effect | feat(psychometric): recover Driver p.16 CINTstd after positive asymDIFFUSION |
| #194 | `471b115e891be11fb41eb75a5cc3073258ab0f34` | true | agent/psychometric-standardised-asymptotic-tipred-effect | feat(psychometric): recover Driver p.16 TIPREDEFFECTstd after positive variances |
| #193 | `6235a3fc6fd6899f85f74aed71053bd50529c34c` | true | agent/psychometric-standardised-asymptotic-tipred-effect | feat(psychometric): recover Driver p.16 finite-interval TIPREDEFFECTstd after positive variances |
| #192 | `1690c7b65693cf27c8f1730150d67084594aef07` | true | agent/psychometric-standardised-continuous-drift | feat(psychometric): recover Driver p.16 asymTIPREDEFFECTstd after positive variances |
| #190 | `adb0bc6f2a1b0684eb9242bcb0d6cddd1ff09ae3` | true | agent/psychometric-standardised-continuous-diffusion | feat(psychometric): recover Driver p.16 DRIFTstd after positive asymDIFFUSION |
| #189 | `0836c359fab7878084ac3a711920550eb6cacb0d` | true | agent/psychometric-standardised-discrete-diffusion | feat(psychometric): recover Driver p.16 DIFFUSIONstd after positive asymDIFFUSION |
| #188 | `444304d2d702cceb47cd4fdf695ba630d7e7c83a` | true | agent/psychometric-standardised-discrete-drift | feat(psychometric): recover Driver p.16 discreteDIFFUSIONstd after positive asymDIFFUSION |
| #187 | `6afd048d1a78bd7e1f68555182ec6a1f0db9ea8c` | true | agent/psychometric-predetermined-later-start-later-t0var | feat(psychometric): recover Driver p.16 discreteDRIFTstd after positive asymDIFFUSION |
| #185 | `69ffec6425863b2842094ff1b1d6c07d8eee6b5b` | true | agent/psychometric-predetermined-later-lagged-t0var | feat(psychometric): recover Driver later-start later-occasion variance of predetermined T0VAR |
| #184 | `6b93147e668ff414240f3ea94997f2937c0817f1` | true | agent/psychometric-predetermined-initial-t0var | feat(psychometric): recover Driver later-start lagged covariance of predetermined T0VAR |
| #183 | `c10097be0f6f4ff8dd8e6db21dce8818630fe17f` | true | agent/psychometric-predetermined-lagged-t0var | feat(psychometric): recover Driver first-occasion variance of predetermined T0VAR |
| #182 | `ee999e1ed23327a75d9cf4eb09a4652561f07cab` | true | agent/psychometric-predetermined-later-t0var | feat(psychometric): recover Driver lagged covariance of predetermined T0VAR |
| #181 | `96e8d1c21924124d144f6273893eda8ac6b41b2d` | true | agent/psychometric-posterior-esem-input | feat(psychometric): recover Driver later-occasion variance of predetermined T0VAR |
| #157 | `340087494b0a9653aede4eeb4bd27049e051222d` | false | main | feat(api): publish completed analysis-run result contract |
| #147 | `08c427f76098adcd5226d1e141d208896bfe58f8` | true | main | feat(method): refuse section boilerplate as unique content |
| #146 | `5419b0f76a737faf65d6b0c848c83f2f1336e326` | false | main | feat(membership): refuse episode membership outside the episode |
| #142 | `7ca6035c5d8b7c9a7761a0c5f26d1de4215c334b` | true | main | feat(evidence): refuse untrusted payloads as estimator authority |
| #136 | `7825c778f39bb8d1ab9f6fe18c227559f8e78fee` | true | main | feat(relation): refuse translation edges as state transitions |
| #134 | `7f4a5df9fd18be203c995f59f1e734f69eba56f9` | true | main | feat(psychometric): refuse unidentified association as causal language |
| #132 | `fe73209ea3407f0c8cd62fbf10ecea844ac15e50` | true | main | feat(relation): refuse unobserved pairs as no-relationship |
| #131 | `0651aeec72cd54c1119effa77e97e992bf265b3d` | false | main | feat(membership): refuse collapsing targets into entity or project |
| #129 | `8981696f58892a1ace9baeb6224ac55b436b8110` | true | main | feat(membership): refuse customer-competitor overlap |
| #113 | `bd800fc62a7cb59339177e34baf3237d6897fce7` | false | main | feat(persistence): fail-closed entity and project target SQL |
| #95 | `3c42b3b09e9a0501d7c7a1a42a322e58cba39165` | true | main | feat(privacy): refuse blanket-masked scientific field grants |
| #92 | `192e1eab459ea1c7e155d9fafe60d1b5912025f2` | true | main | feat(orchestrator): serve interpretation POSTs on a loopback HTTP listener |
| #86 | `47f6216aec217f15a6274d889bc507e59821096f` | true | main | feat(privacy): replay privileged-access decisions without source identity |
| #85 | `1ebb40644bb479ed8703f6df1b351f3e19d5fe13` | false | main | feat(event): score CHRONOS occurrence forecasts with a Brier rule |
| #84 | `e0ef1f869c46aa20969ce083f9ca31d622c31bb4` | true | main | feat(invariance): replay shared-meaning gate on current main |
| #83 | `18200b674cb29d84ab66853f50e1b5dc38096279` | true | main | docs(evidence): define language-agnostic semantic spans |
| #82 | `774a01bc63600c0c4703114bb01467fd65e57077` | true | main | feat(privacy): export identity maps only under re-identification purpose |
| #81 | `a7769d37ad1f950547f00e05ce2c03e5a2843b43` | true | main | feat(privacy): bind tenant roles to system-time lifetimes |
| #80 | `9539417f223bc6067f21e701986a49b7d3a37aba` | true | main | feat(temporal): space longitudinal lags on event time |
| #79 | `166e37c2cb540a575c690c813a21123a13d86376` | true | main | feat(privacy): bind authorization grants to one processing purpose |
| #76 | `ccffcab50ff2f1d8c0e6ee80269d0f7ee0075368` | true | main | feat(event): score TDT topic detections without promoting clusters |
| #75 | `90cc8b6d4848e4c7718dd4bea74c1aaaab6f3685` | true | main | feat(method): model template sources without inferential weights |
| #74 | `0ce7e6a923282d3533cf3b8c0592d5df244049f8` | true | main | feat(temporal): keep predicted Allen assertions hypothetical |
| #72 | `899656968cc9f68a3d8fc55dddc4fd257fb27050` | false | main | feat(event): score TDT story segments with WindowDiff and Pk |
| #70 | `7a1f33aa68c1c9be9e9da7ac7f7dadb1092ff9e4` | false | main | feat(event): score CHRONOS schema slots with precision and recall |
| #68 | `6a98c812ef8e621e1bc1c46a615296b39f0bae33` | false | main | feat(event): score TDT tracks with pair precision and switch rate |
| #66 | `0fd2e207c731909581469a1ea2c0c1b55bd99d9a` | false | main | feat(event): score TDT mention links with precision and recall |
| #65 | `74351f398d67df590019049ab85a7f649495b860` | false | main | feat(event): score first-story detections with FAR and miss rates |
| #63 | `0735ca67abcf5c6c2fd591a52d29067e5f16215a` | false | main | feat(corpus): refuse TF-IDF and BM25 as inferential weights |
| #62 | `7ba0f09df56ad2fc5601a93e581f6b3fa1af76c9` | false | main | feat(simulation): exclude delayed documents before they are available |
| #61 | `c7f094cf17b5d0be0aa9270b2d397da4d2db851d` | false | main | feat(temporal): refuse uncertain availability past knowledge cutoff |
| #60 | `40ede67614599ba21495fea024d4493d3af9fbb3` | false | main | feat(relation): refuse association and precedence as causation |
| #59 | `83731ec705bd4c29f8466e7990f1febb4f073120` | false | main | feat(corpus): treat NFC/NFD bodies as split leakage |
| #58 | `6a01fefca73fb19a25d082314dcdbc10d52ee383` | false | main | feat(evidence): keep embedded image URIs as positional non-lexical units |
| #57 | `b76e38393e932873ec2aed682ab539ee61828537` | false | main | feat(validation): exact-head claim promotion gates |
| #51 | `1801501c4d7c5be720d24aba954280fbc9068612` | false | main | feat(compute): VRAM budget types with CPU f64 fallback |
| #49 | `9516ef1384f5239096d6d13c52155f8f7dde7795` | false | main | feat(psychometric): posterior ESEM input gates with true-parameter RMSE |
| #48 | `6110d3660607ba46b312b4d76f048f1bcc4f3bc5` | false | main | feat(topic): logistic-normal ALR coordinates with true-parameter RMSE |

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

Protected `main` contains 47 unique Rust crate boundaries in the current
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
| GAP-012 | The 71-PR queue obscures authority, repeatedly stales exact-head evidence, and fragments product boundaries. | `partial` | release blocker | `e65cd66` (protected-main truth) | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) | `—` (live queue program) | Every PR classified; unique landing vehicles; compatible slices folded; superseded work closed with provenance; scheduler prioritizes consolidation; queue reaches zero before GA. |
| GAP-013 | Evidence-grounded LLM interpretation is routed but not executed and validated as a production interpreter/verifier port. | `partial` | active integration | `e65cd66` (routing and refusal contracts only) | [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176), [PR #69](https://github.com/ContextualWisdomLab/TEPP/pull/69), [PR #165](https://github.com/ContextualWisdomLab/TEPP/pull/165) | `8e4a3ca9cc80` / `34083c3f5d66` | Contextual-orchestrator execution, evidence citations, verifier refusals, comparable-budget ablations, provider eligibility/fallback, abstention, live/offline contract tests, and no numerical-authority escalation. |
| GAP-014 | README/TRD and some PR descriptions lag protected-main and live queue reality. | `partial` | documentation drift | `e65cd66` (documentation is not fully synchronized) | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) | `—` (queue-consolidation work) | Reconcile README, TRD, traceability, ADR maturity, CHANGELOG, preferred-merge declarations, and exact protected-main evidence. |
| GAP-015 | There was no canonical live product/operator-gap register tied to documentation validation. | `active-PR` | landing vehicle | `e65cd66` (the register and validator are implemented-main; this refresh keeps the volatile queue current) | [PR #217](https://github.com/ContextualWisdomLab/TEPP/pull/217); [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164) is the merged authority; closed [PR #179](https://github.com/ContextualWisdomLab/TEPP/pull/179) is not a second register | `79df32c73b260838fd3d0bfed3e84ff3e58e4b28` | Land this refresh after exact-head checks and independent review, then regenerate it whenever protected-main or the live queue changes. |
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
| [#179](https://github.com/ContextualWisdomLab/TEPP/pull/179) | Duplicate gap-baseline snapshot | Closed as superseded at 2026-08-23T12:39:22Z; do not reopen as a second register. |
| [#177](https://github.com/ContextualWisdomLab/TEPP/pull/177) | Central scheduler budget-hardening pin | Re-run exact-head Checks and obtain independent review before relying on the hourly caller. |
| [#165](https://github.com/ContextualWisdomLab/TEPP/pull/165) | Hourly agent routing through contextual-orchestrator | Ensure queue-consolidation policy from #175 prevents unrelated micro-PR growth. |
| [#157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | Terminal result contract for #156, including folded cutoff-safe execution from closed #178 | Complete exact-head review/check gates; keep #156 open until protected-main verification. |
| [#180](https://github.com/ContextualWisdomLab/TEPP/pull/180) | Hide raw provider exception details | Review/check gates; do not weaken fail-closed provider handling. |
| [#153](https://github.com/ContextualWisdomLab/TEPP/pull/153) | Refuse location as entity identity or language | Independent review after exact-head membership tests. |
| [#48](https://github.com/ContextualWisdomLab/TEPP/pull/48) | Logistic-normal/ILR coordinate slice, including folded stacked #191 TRSL lineage artifacts | Fold into the estimator landing plan; coordinates and lineage artifacts are not candidate-K fitting. |
| [#51](https://github.com/ContextualWisdomLab/TEPP/pull/51) | VRAM budget/fallback policy types | Resolve requested changes; connect only to a real production kernel under #171. |
| [#67](https://github.com/ContextualWisdomLab/TEPP/pull/67) | Model-selection gates | Rebase/merge only with an explicit path to real candidate fitting under #167. |
| [#69](https://github.com/ContextualWisdomLab/TEPP/pull/69) | Interpretation grounding/refusal metrics | Fold into the product interpreter/verifier under #176. |
| [#71](https://github.com/ContextualWisdomLab/TEPP/pull/71) | Compositional geometry/network pair metrics | Fold into the network estimator under #172. |
| [#119](https://github.com/ContextualWisdomLab/TEPP/pull/119) | Bounded loading/lag recovery slice | Consolidate into the psychometric engine under #169. |
| [#181](https://github.com/ContextualWisdomLab/TEPP/pull/181)–[#190](https://github.com/ContextualWisdomLab/TEPP/pull/190), [#192](https://github.com/ContextualWisdomLab/TEPP/pull/192)–[#204](https://github.com/ContextualWisdomLab/TEPP/pull/204) | Draft stacked psychometric T0VAR/drift/standardised recoveries | Keep stacked; do not promote drafts or predecessor heads as protected-main. Closed stacked #191 landed on #48 only. |

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
