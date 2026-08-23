# Product and Technical Gap Baseline

**Status:** Live delivery baseline
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-08-23T13:57:34Z
**Protected-main evidence:** `c45be17a9dbce95ef81cee230e9d128abc7160ac`
**Workspace version on protected main:** `0.1.0`
**Canonical gap-baseline authority:** [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164). Duplicate [PR #179](https://github.com/ContextualWisdomLab/TEPP/pull/179) was closed at 2026-08-23T12:39:22Z and is not a second protected-main register.

## Purpose

This document is the executable buyer-gap register for TEPP. It separates:

- capabilities a buyer can use from protected `main`;
- bounded work that exists only on open pull requests;
- product-completion issues with measurable acceptance evidence; and
- release claims that remain prohibited.

A planning document, local test, queued check, predecessor-head result, LLM
judgment, or mergeable branch does not make a capability shipped. Re-read live
GitHub state before any customer, release, certification, or valuation claim.

## Snapshot facts

| Signal | Snapshot evidence | Delivery implication |
|---|---:|---|
| Protected-main SHA | `c45be17a9dbce95ef81cee230e9d128abc7160ac` | All as-built claims are bounded to this commit. |
| Workspace members | 10 Rust crates | The foundation is modular, but the approved target contains additional semantic, estimator, compute, psychometric, event-intelligence, network, interpretation, artifact, and visual boundaries. |
| Open pull requests | **96** | The queue is itself a release blocker and requires consolidation. |
| Draft pull requests | **27** | Most queued work is not independently review-ready. |
| Non-draft pull requests | **69** | A non-draft state is not a qualifying review or required-check result. |
| Open product issues | **12** | Issue #156 plus the product-completion program #166–#176. |
| Duplicate gap-baseline PRs | **#164** (authority); **#179** closed as superseded | Only one live register remains; queued Checks on #164 are not protected-main evidence. |
| Current package version | `0.1.0` | No supported product release is established by the repository version alone. |

The pull-request counts come from the live GitHub search at this snapshot. The
full exact-head classification is owned by
[issue #175](https://github.com/ContextualWisdomLab/TEPP/issues/175); this file
keeps the buyer-level summary while the exact-head register below records the
current 96-row volatile queue. Passing or queued Checks on an open PR never
promote that PR to implemented-main.

## Current open pull-request evidence

The following exact-head register was fetched from the live GitHub pull-request
API at the snapshot above. `draft=false` is not approval, mergeability, or a
passing-check claim. Re-read the full SHA, current review decision, required
Checks, and branch rules immediately before every mutation.

| PR | Exact current head | Draft | Base | Title |
|---:|---|:---:|---|---|
| #190 | `adb0bc6f2a1b0684eb9242bcb0d6cddd1ff09ae3` | true | agent/psychometric-standardised-continuous-diffusion | feat(psychometric): recover Driver p.16 DRIFTstd after positive asymDIFFUSION |
| #189 | `0836c359fab7878084ac3a711920550eb6cacb0d` | true | agent/psychometric-standardised-discrete-diffusion | feat(psychometric): recover Driver p.16 DIFFUSIONstd after positive asymDIFFUSION |
| #188 | `444304d2d702cceb47cd4fdf695ba630d7e7c83a` | true | agent/psychometric-standardised-discrete-drift | feat(psychometric): recover Driver p.16 discreteDIFFUSIONstd after positive asymDIFFUSION |
| #187 | `6afd048d1a78bd7e1f68555182ec6a1f0db9ea8c` | true | agent/psychometric-predetermined-later-start-later-t0var | feat(psychometric): recover Driver p.16 discreteDRIFTstd after positive asymDIFFUSION |
| #185 | `69ffec6425863b2842094ff1b1d6c07d8eee6b5b` | true | agent/psychometric-predetermined-later-lagged-t0var | feat(psychometric): recover Driver later-start later-occasion variance of predetermined T0VAR |
| #184 | `6b93147e668ff414240f3ea94997f2937c0817f1` | true | agent/psychometric-predetermined-initial-t0var | feat(psychometric): recover Driver later-start lagged covariance of predetermined T0VAR |
| #183 | `c10097be0f6f4ff8dd8e6db21dce8818630fe17f` | true | agent/psychometric-predetermined-lagged-t0var | feat(psychometric): recover Driver first-occasion variance of predetermined T0VAR |
| #182 | `2d4d6bf9fdfa1c55d2cc3b2f36ba6a1d0523c8a8` | true | agent/psychometric-predetermined-later-t0var | feat(psychometric): recover Driver lagged covariance of predetermined T0VAR |
| #181 | `542806b747eea55c9eab705c5dd3479a3e1c63b6` | true | agent/psychometric-posterior-esem-input | feat(psychometric): recover Driver later-occasion variance of predetermined T0VAR |
| #180 | `fa169e8e41f13d3bae6aff242db68ebf8281e112` | false | main | fix(transport): hide raw provider exception details |
| #177 | `580d45206536bf2f7deba327ffafb96a69ba7260` | false | main | fix(workflow): pin scheduler budget hardening |
| #165 | `34083c3f5d6637cf1665a29dac45fbf67a74d5dd` | false | main | feat: route hourly product agents through contextual orchestrator |
| #164 | `f5e2db0bc0268b3e898129fea1d80809822b4989` | false | main | docs: define TEPP product completion and technical gap baseline |
| #157 | `7402745cfb0c27964f286c38a2efd7042c2320ff` | false | main | feat(api): publish completed analysis-run result contract |
| #155 | `02c009c16e6a9c6fee3237224ed6a17cbc4169e1` | false | main | feat(api): admit LineageWeave as a modular analysis-run consumer |
| #154 | `a7a1bdf96a6759f11aed9284b73319ed94b82fd2` | false | main | ci(rust): refresh branch coverage compiler baseline |
| #153 | `e7c871bc2f5e6023002ceeea30217130561842e1` | false | main | feat(membership): refuse location as entity identity or language |
| #152 | `70a6c6608e39df757417b2674f0acf27aa09b612` | false | main | feat(method): refuse prompt boilerplate as unique content |
| #151 | `f87c812415280681c4243117fbd1302b204e4f68` | false | main | feat(method): refuse corpus-background wording as unique content |
| #150 | `f9110e0eec68d823778a433c4ccab3a1beca46f5` | false | main | feat(method): refuse non-lexical modality as unique content |
| #149 | `6077b9ce7f0961a3e287c4e8afe0faa5a0638b2e` | false | main | feat(method): refuse copied-text residue as unique content |
| #148 | `4c0612fe717a40cebfb9ba1e31daaa7317607c9f` | false | main | feat(method): refuse house-voice style as unique content |
| #147 | `08c427f76098adcd5226d1e141d208896bfe58f8` | true | main | feat(method): refuse section boilerplate as unique content |
| #146 | `5419b0f76a737faf65d6b0c848c83f2f1336e326` | false | main | feat(membership): refuse episode membership outside the episode |
| #145 | `696d56c5816815e22181a03c57a822f974d41706` | false | main | feat(method): refuse default stopword deletion of report language |
| #143 | `7d533c6198492f748303869f0795fac5c8764b50` | false | main | feat(relation): refuse a template copy as the source identity |
| #142 | `7ca6035c5d8b7c9a7761a0c5f26d1de4215c334b` | true | main | feat(evidence): refuse untrusted payloads as estimator authority |
| #141 | `d558a4b4b3518f4ae09fde6ac828602bb007cdec` | false | main | feat(privacy): refuse untrusted intake without a grant |
| #140 | `4feb427f6f895ebb53a151ae7c8483f69a3890f0` | false | main | feat(estimator): refuse a checkpoint as the CPU f64 estimator |
| #139 | `43ebbda0d35f9bf227c5764d53d429fff0f08c8c` | false | main | feat(relation): refuse a summary as the source identity |
| #138 | `f06879da364e3149aa2caf51158de60926d9846f` | false | main | feat(relation): refuse reverse input-process-outcome event time |
| #137 | `c8b3d494df100e57aab7fdbcdc58a798c874e6c0` | false | main | feat(relation): refuse retrospective reporting as a transition |
| #136 | `7825c778f39bb8d1ab9f6fe18c227559f8e78fee` | true | main | feat(relation): refuse translation edges as state transitions |
| #135 | `1576e7fe5f03eb1a678060091115dfc927518d49` | false | main | feat(evidence): refuse untrusted payloads without identity and bounds |
| #134 | `7f4a5df9fd18be203c995f59f1e734f69eba56f9` | true | main | feat(psychometric): refuse unidentified association as causal language |
| #133 | `dbe1c1f6f1e27403ee6301f3de220fff1f2d8b40` | false | main | feat(relation): refuse inferred status as observed evidence |
| #132 | `fe73209ea3407f0c8cd62fbf10ecea844ac15e50` | true | main | feat(relation): refuse unobserved pairs as no-relationship |
| #131 | `02c04db787ed6fc443102875740d97b7d2e2cbcc` | false | main | feat(membership): refuse collapsing targets into entity or project |
| #130 | `6f963c8527986c8290aff94ae898991adf03e4d6` | false | main | feat(relation): refuse support edges as state transitions |
| #129 | `8981696f58892a1ace9baeb6224ac55b436b8110` | true | main | feat(membership): refuse customer-competitor overlap |
| #128 | `30ff215365a18ccef7b22e03912c293532b8f7bc` | false | main | feat(temporal): refuse other clocks as system time |
| #127 | `bd34a2dda405fab264147f68daa2059dd61c30ca` | false | main | feat(temporal): refuse other clocks as event time |
| #126 | `28f7ed69785da71c38439c4241a132ab391a5c6f` | false | main | feat(temporal): refuse other clocks as assertion time |
| #125 | `c225f353f27647268d44a9e8156412a1b9de9fc2` | false | main | feat(temporal): refuse event, system, and available time as cutoff |
| #124 | `e89b90ce8d2792cb3f1fd61ad7dbd12556855df2` | false | main | feat(temporal): refuse event and system time as availability |
| #123 | `78e41907ff95067a8755e3ceb41de5f32299ba6e` | false | main | feat(temporal): refuse document rows that omit assertion or document time |
| #122 | `38542fd41196394587e024c7e824c840dcafb32c` | false | main | feat(temporal): refuse later revisions with earlier system time |
| #121 | `93f6aa5e28761e69f14093d2e341799219f867c6` | false | main | feat(privacy): seal identity mappings with purpose-bound HMAC |
| #120 | `34c4c119608b2aa46c84e9bef2b47711740ff9de` | false | main | feat(relation): refuse citation edges as state transitions |
| #119 | `47ab763d49b14ab1ba31f4828ad6e3fda274c007` | false | main | feat(psychometric): recover ESEM loadings and refuse reverse DSEM lags |
| #118 | `b8f4582f65852b0ac5ba9149de9d5d7783b1c282` | false | main | feat(event): refuse subevents that escape the parent interval |
| #117 | `8a5a3fa6d38703e6b83b03ad64959dd01c8f263d` | false | main | feat(membership): recover nested ICC and refuse cross-classified collapse |
| #116 | `08d557c2f772e92543e71d27ac41a108fe64a8bb` | false | main | docs(adr): wire verified method papers into temporal and psychometric ADRs |
| #115 | `2dbc0fd1d7200100a72990c01008e50ad5f9e641` | false | main | fix(quality): refuse inverted coverage-authority PR sentences |
| #114 | `df734d71293bf5bbdd10d28430c15626cdb1220e` | false | main | feat(privacy): record provider field codes without source text |
| #113 | `bd800fc62a7cb59339177e34baf3237d6897fce7` | false | main | feat(persistence): fail-closed entity and project target SQL |
| #110 | `4517b3f00d56d08a5a9bf47e6b7a681537765127` | false | main | feat(api): export corpus-split leakage-audit manifests |
| #106 | `60aef093e4f89638b896864342aa28d3a2d3ece7` | false | main | fix(privacy): inspect audit_event inserts through try_record |
| #100 | `b2a885b9b5418c79d5e828d64291c878f5ae5477` | false | main | test(tls): recover bind decisions from the live policy |
| #99 | `c4ed064c9e931f1772992526969fb4ae5fcd4537` | false | main | feat(persistence): persist exact text_segment byte spans without 0007 |
| #95 | `3c42b3b09e9a0501d7c7a1a42a322e58cba39165` | true | main | feat(privacy): refuse blanket-masked scientific field grants |
| #92 | `192e1eab459ea1c7e155d9fafe60d1b5912025f2` | true | main | feat(orchestrator): serve interpretation POSTs on a loopback HTTP listener |
| #91 | `808ae6690b35bb58f2152d74c7978e90de6b86a0` | false | main | feat(privacy): inherit source sensitivity onto derived artifacts |
| #86 | `47f6216aec217f15a6274d889bc507e59821096f` | true | main | feat(privacy): replay privileged-access decisions without source identity |
| #85 | `1ebb40644bb479ed8703f6df1b351f3e19d5fe13` | false | main | feat(event): score CHRONOS occurrence forecasts with a Brier rule |
| #84 | `e0ef1f869c46aa20969ce083f9ca31d622c31bb4` | true | main | feat(invariance): replay shared-meaning gate on current main |
| #83 | `18200b674cb29d84ab66853f50e1b5dc38096279` | true | main | docs(evidence): define language-agnostic semantic spans |
| #82 | `774a01bc63600c0c4703114bb01467fd65e57077` | true | main | feat(privacy): export identity maps only under re-identification purpose |
| #81 | `a7769d37ad1f950547f00e05ce2c03e5a2843b43` | true | main | feat(privacy): bind tenant roles to system-time lifetimes |
| #80 | `9539417f223bc6067f21e701986a49b7d3a37aba` | true | main | feat(temporal): space longitudinal lags on event time |
| #79 | `166e37c2cb540a575c690c813a21123a13d86376` | true | main | feat(privacy): bind authorization grants to one processing purpose |
| #78 | `b64c6799708ac1f6ac7c84e07fe3ae50750434e1` | false | main | feat(longitudinal): keep unit means out of within-unit change |
| #76 | `ccffcab50ff2f1d8c0e6ee80269d0f7ee0075368` | true | main | feat(event): score TDT topic detections without promoting clusters |
| #75 | `90cc8b6d4848e4c7718dd4bea74c1aaaab6f3685` | true | main | feat(method): model template sources without inferential weights |
| #74 | `0ce7e6a923282d3533cf3b8c0592d5df244049f8` | true | main | feat(temporal): keep predicted Allen assertions hypothetical |
| #73 | `11cc811a10b80a49e8a8ed6bc60fba3d5c753d28` | false | main | feat(topic): keep one identity across dormancy and reactivation |
| #72 | `899656968cc9f68a3d8fc55dddc4fd257fb27050` | false | main | feat(event): score TDT story segments with WindowDiff and Pk |
| #71 | `2588f38281b97fa03f552634d307e8b4c908899d` | false | main | feat(network): refuse raw simplex Euclidean cluster geometry |
| #70 | `7a1f33aa68c1c9be9e9da7ac7f7dadb1092ff9e4` | false | main | feat(event): score CHRONOS schema slots with precision and recall |
| #69 | `8e4a3ca9cc80f0f4e3bccf712fe6a725bfb1365e` | false | main | feat(interpretation): keep LLM claims hypothetical and evidence-cited |
| #68 | `6a98c812ef8e621e1bc1c46a615296b39f0bae33` | false | main | feat(event): score TDT tracks with pair precision and switch rate |
| #67 | `45b272d2498ecff128d75621c88c1e7d6eb1400b` | false | main | feat(model): statistical Pareto K gates refuse LLM numerical authority |
| #66 | `380be3f33f798cc8648a9dc9f24b85011f9e0468` | false | main | feat(event): score TDT mention links with precision and recall |
| #65 | `74351f398d67df590019049ab85a7f649495b860` | false | main | feat(event): score first-story detections with FAR and miss rates |
| #64 | `c11adce997d4f3e462f50711211af9f051e747b0` | false | main | feat(event): score mention confidence with a known-truth Brier rule |
| #63 | `0735ca67abcf5c6c2fd591a52d29067e5f16215a` | false | main | feat(corpus): refuse TF-IDF and BM25 as inferential weights |
| #62 | `7a16e579814ef06517f82fd2b41ebb27568eb960` | false | main | feat(simulation): exclude delayed documents before they are available |
| #61 | `c7f094cf17b5d0be0aa9270b2d397da4d2db851d` | false | main | feat(temporal): refuse uncertain availability past knowledge cutoff |
| #60 | `40ede67614599ba21495fea024d4493d3af9fbb3` | false | main | feat(relation): refuse association and precedence as causation |
| #59 | `b47efdc3cfdcec61f69c4219c49519a160bd5549` | false | main | feat(corpus): treat NFC/NFD bodies as split leakage |
| #58 | `431561a6c422660c618f82a241e5a4b320653378` | false | main | feat(evidence): keep embedded image URIs as positional non-lexical units |
| #57 | `b76e38393e932873ec2aed682ab539ee61828537` | false | main | feat(validation): exact-head claim promotion gates |
| #51 | `1801501c4d7c5be720d24aba954280fbc9068612` | false | main | feat(compute): VRAM budget types with CPU f64 fallback |
| #50 | `d4cd08341e8dca8475542cf9f3b46a1f5bae5ea3` | false | main | feat(event): refuse TDT/CHRONOS outputs as state transitions |
| #49 | `22b8e68813ad59a9a91689bacfa4cf033dfad158` | false | main | feat(psychometric): posterior ESEM input gates with true-parameter RMSE |
| #48 | `b7214a8e9dc140e622124fc046d9aa5147dd7b27` | false | main | feat(topic): logistic-normal ALR coordinates with true-parameter RMSE |

The exact-head register, counts, and protected-main SHA were fetched from the live GitHub REST API at `2026-08-23T13:57:34Z`. Review decisions, required Checks, and mergeability remain volatile; re-read them immediately before every mutation. This snapshot is not merge authorization and does not treat queued or passing Checks as shipped protected-main behavior.

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

Protected `main` contains the following workspace boundaries:

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

## Buyer-gap register

| ID | Buyer-visible gap | Maturity | Delivery status | Protected-main authority | Current delivery authority | Current head SHA | Closure evidence |
|---|---|---|---|---|---|---|---|
| GAP-001 | Submission can produce a durable accepted receipt, but protected `main` lacks the separate deterministic terminal-result lifecycle. | `active-PR` | landing vehicle | `c45be17` (accepted receipt only) | [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) / [PR #157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | `7402745cfb0c` | Exact request/result/snapshot/cutoff/model/profile binding, typed terminal failures, deterministic retrieval, current-head checks, and qualifying review. Cutoff-safe execution from closed stacked PR #178 is on this head only and is not implemented-main. |
| GAP-002 | LineageWeave and other modular consumers cannot yet rely on the complete protected-main HTTP evidence/result boundary. | `active-PR` | landing vehicle | `c45be17` (loopback naruon listener from merged PR #107; versioned API contracts) | [#155](https://github.com/ContextualWisdomLab/TEPP/pull/155) | `02c009c16e6a` | Merge the live consumer contract after exact-head review/Checks. Closed stacked PRs #158 and #159 were folded into this head; they are not protected-main evidence. |
| GAP-003A | Immutable evidence cannot yet be submitted to a durable validation run that produces buyer-usable scientific acceptance evidence. | `accepted-target` | product-completion | `c45be17` (validation metrics are library-level only) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | `—` (issue program; no current implementation PR) | Compose/CLI/API execution must bind immutable evidence, cutoffs, model configuration, validation metrics, and reproducibility manifests to one idempotent run. |
| GAP-003B | Scientific result artifacts cannot yet be persisted, restarted, and recovered as one supported buyer workflow. | `accepted-target` | product-completion | `c45be17` (persistence contracts lack E2E recovery) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | `—` (issue program; no current implementation PR) | Durable storage, migration/rollback, restart/recovery, artifact digest verification, and terminal retrieval must pass against a real Compose deployment. |
| GAP-003C | The persistence slice classifies concurrent-write SQLSTATEs, but has no measured hot-partition detection, routing, or mitigation for tenant/result workloads. | `accepted-target` | product-completion | `c45be17` (conflict classification only; no measured partition control) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | `—` (issue program; no current implementation PR) | A real Compose/PostgreSQL workload identifies hot keys and partition skew, applies bounded tenant/time or result routing without weakening 3NF or temporal authority, and proves conflict rate, latency, recovery, and migration/rollback behavior under load. |
| GAP-004 | The central shared-latent temporal/relational topic estimator is absent. | `accepted-target` | product vertical | `c45be17` (no production estimator) | [#167](https://github.com/ContextualWisdomLab/TEPP/issues/167) / [PR #48](https://github.com/ContextualWisdomLab/TEPP/pull/48) | `b7214a8e9dc1` | Rust CPU `f64` fitting, sparse bounded parallelism, posterior artifacts, convergence, true-parameter RMSE/bias/coverage, and real candidate-K fitting. |
| GAP-005 | Real multilingual documents are not yet transformed into validated exact-span semantic units and versioned shared concepts. | `accepted-target` | product vertical | `c45be17` (evidence spans exist without full semantic pipeline) | [#168](https://github.com/ContextualWisdomLab/TEPP/issues/168) | `—` (PR #53 merged only onto a non-main maintenance branch and is not implemented-main) | Unicode/layout/language-tailored processing, unknown-concept review, multilingual calibration/invariance, image-position evidence, and prompt-injection tests. |
| GAP-006 | Posterior topic measurements cannot yet be fitted through a complete cross-classified longitudinal ESEM/DSEM engine. | `accepted-target` | product vertical | `c45be17` (temporal and membership primitives only) | [#169](https://github.com/ContextualWisdomLab/TEPP/issues/169) / [PR #119](https://github.com/ContextualWisdomLab/TEPP/pull/119) | `47ab763d49b1` | Plausible-value/joint uncertainty, invariance, irregular event time, within/between separation, multiple membership, true-parameter recovery, and causal-claim refusal. |
| GAP-007 | TDT detection/tracking and CHRONOS schema/forecast/temporal reasoning remain isolated bounded gates rather than one calibrated product workflow. | `accepted-target` | product vertical | `c45be17` (event/time primitives only) | [#170](https://github.com/ContextualWisdomLab/TEPP/issues/170) / [PR #70](https://github.com/ContextualWisdomLab/TEPP/pull/70) | `7a1f33aa68c1` | Span-grounded mentions, calibrated TDT metrics, schema/forecast hypothesis states, interval consistency, known-truth recovery, persistence, and exports. |
| GAP-008 | GPU support is policy-only; no production estimator kernel has real hardware parity or declared VRAM evidence. | `accepted-target` | product vertical | `c45be17` (VRAM policy only) | [#171](https://github.com/ContextualWisdomLab/TEPP/issues/171) / [PR #51](https://github.com/ContextualWisdomLab/TEPP/pull/51) | `1801501c4d7c` | Real CUDA/portable backend execution, CPU parity, streamed memory, bounded OOM/fallback, hardware profiles, telemetry, and no skipped-support claim. |
| GAP-009 | Topic association and cluster outputs lack posterior-valid estimation, uncertainty, edge stability, and consensus communities. | `accepted-target` | product vertical | `c45be17` (network primitives only) | [#172](https://github.com/ContextualWisdomLab/TEPP/issues/172) / [PR #71](https://github.com/ContextualWisdomLab/TEPP/pull/71) | `2588f38281b9` | Valid log-ratio coordinates, interval/stability-bearing edges, repeated Leiden consensus, known-truth network/cluster recovery, and reproducible exports. |
| GAP-010 | Buyers lack coordinated accessible visual analytics and exact-value export workflows. | `accepted-target` | product vertical | `c45be17` (no visual workspace) | [#173](https://github.com/ContextualWisdomLab/TEPP/issues/173) | `—` (Figma work not started) | Real Figma File ID in ADR, Storybook/design tokens, ten PRD views, exact-value tables, accessible interaction/print/PDF states, provenance, and source-consistent exports. |
| GAP-011 | TEPP is not yet an operable multi-tenant service or supported release. | `accepted-target` | product vertical | `c45be17` (library contracts only) | [#174](https://github.com/ContextualWisdomLab/TEPP/issues/174) | `—` (issue program; no current implementation PR) | Durable queue/storage, OIDC/RLS/purpose controls, OpenTelemetry/SLOs, load/recovery, migrations, signed release/SBOM/provenance, assurance evidence, and support policy. |
| GAP-012 | The 96-PR queue obscures authority, repeatedly stales exact-head evidence, and fragments product boundaries. | `partial` | release blocker | `c45be17` (protected-main truth) | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) | `—` (live queue program) | Every PR classified; unique landing vehicles; compatible slices folded; superseded work closed with provenance; scheduler prioritizes consolidation; queue reaches zero before GA. |
| GAP-013 | Evidence-grounded LLM interpretation is routed but not executed and validated as a production interpreter/verifier port. | `partial` | active integration | `c45be17` (routing and refusal contracts only) | [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176), [PR #69](https://github.com/ContextualWisdomLab/TEPP/pull/69), [PR #165](https://github.com/ContextualWisdomLab/TEPP/pull/165) | `8e4a3ca9cc80` / `34083c3f5d66` | Contextual-orchestrator execution, evidence citations, verifier refusals, comparable-budget ablations, provider eligibility/fallback, abstention, live/offline contract tests, and no numerical-authority escalation. |
| GAP-014 | README/TRD and some PR descriptions lag protected-main and live queue reality. | `partial` | documentation drift | `c45be17` (documentation is not fully synchronized) | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) | `—` (queue-consolidation work) | Reconcile README, TRD, traceability, ADR maturity, CHANGELOG, preferred-merge declarations, and exact protected-main evidence. |
| GAP-015 | There was no canonical live product/buyer-gap register tied to documentation validation. | `active-PR` | landing vehicle | `c45be17` (no live gap register) | [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164) (authority); closed [PR #179](https://github.com/ContextualWisdomLab/TEPP/pull/179) is not a second register | `f5e2db0bc026` | Merge this document and its validator/map/quality-test integration after exact-head checks and independent review. Passing Checks on #164 are not implemented-main. |
| GAP-016 | Hourly PR maintenance used an older central scheduler revision whose per-repository sweep budgets could amplify the queued review workload. | `active-PR` | operability hardening | `c45be17` (caller pin before central budget hardening) | [PR #177](https://github.com/ContextualWisdomLab/TEPP/pull/177) | `580d45206536` | The change pins a verified central revision immutably; closure still requires exact-head hosted Checks, resolved threads, and independent review. |
| GAP-017 | Accepted analysis runs have a terminal DTO, and cutoff-safe execution exists only on the #157 head after stacked PR #178 merged into that branch. | `active-PR` | product vertical | `c45be17` (accepted receipt only) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) / [PR #157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | `7402745cfb0c` | Exact availability cutoff, snapshot binding, multiple-membership preservation, digest integrity, redacted no-eligible failure, and realistic end-to-end tests on protected main. Closed stacked PR #178 is not implemented-main. |

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
| [#164](https://github.com/ContextualWisdomLab/TEPP/pull/164) | Canonical product/technical gap baseline | Re-run exact-head documentation/repository checks and obtain independent review. |
| [#179](https://github.com/ContextualWisdomLab/TEPP/pull/179) | Duplicate gap-baseline snapshot | Closed as superseded at 2026-08-23T12:39:22Z; do not reopen as a second register. |
| [#177](https://github.com/ContextualWisdomLab/TEPP/pull/177) | Central scheduler budget-hardening pin | Re-run exact-head Checks and obtain independent review before relying on the hourly caller. |
| [#165](https://github.com/ContextualWisdomLab/TEPP/pull/165) | Hourly agent routing through contextual-orchestrator | Ensure queue-consolidation policy from #175 prevents unrelated micro-PR growth. |
| [#157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | Terminal result contract for #156, including folded cutoff-safe execution from closed #178 | Complete exact-head review/check gates; keep #156 open until protected-main verification. |
| [#155](https://github.com/ContextualWisdomLab/TEPP/pull/155) | Modular LineageWeave consumer parent, including folded #158/#159 | Obtain independent review; passing Checks are not implemented-main. |
| [#180](https://github.com/ContextualWisdomLab/TEPP/pull/180) | Hide raw provider exception details | Review/check gates; do not weaken fail-closed provider handling. |
| [#153](https://github.com/ContextualWisdomLab/TEPP/pull/153) | Refuse location as entity identity or language | Independent review after exact-head membership tests. |
| [#48](https://github.com/ContextualWisdomLab/TEPP/pull/48) | Logistic-normal/ILR coordinate slice | Resolve requested changes and fold into the estimator landing plan; do not call it fitting. |
| [#51](https://github.com/ContextualWisdomLab/TEPP/pull/51) | VRAM budget/fallback policy types | Resolve requested changes; connect only to a real production kernel under #171. |
| [#67](https://github.com/ContextualWisdomLab/TEPP/pull/67) | Model-selection gates | Rebase/merge only with an explicit path to real candidate fitting under #167. |
| [#69](https://github.com/ContextualWisdomLab/TEPP/pull/69) | Interpretation grounding/refusal metrics | Fold into the product interpreter/verifier under #176. |
| [#71](https://github.com/ContextualWisdomLab/TEPP/pull/71) | Compositional geometry/network pair metrics | Fold into the network estimator under #172. |
| [#119](https://github.com/ContextualWisdomLab/TEPP/pull/119) | Bounded loading/lag recovery slice | Consolidate into the psychometric engine under #169. |
| [#181](https://github.com/ContextualWisdomLab/TEPP/pull/181)–[#187](https://github.com/ContextualWisdomLab/TEPP/pull/187) | Draft stacked psychometric T0VAR/drift recoveries | Keep stacked; do not promote drafts or predecessor heads as protected-main. |

## Delivery sequence

The dependency-aware product order is:

1. **Consolidate delivery authority:** #175 and PR #164.
2. **Finish live result/consumer contracts:** #156/#157 and #155 (loopback PR #107 is already implemented-main; closed stacked #158/#159/#178 are not).
3. **Build validated multilingual evidence:** #168.
4. **Build the CPU topic estimator:** #167.
5. **Build event intelligence and posterior networks:** #170 and #172.
6. **Build the posterior-aware longitudinal psychometric engine:** #169.
7. **Accelerate real kernels with parity:** #171.
8. **Complete the durable end-to-end run:** #166.
9. **Execute and validate interpretation:** #176.
10. **Design and implement the buyer workspace:** #173.
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
result and cannot replace buyer adoption, predictive/construct validity,
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
- an issue or buyer-gap acceptance boundary;
- a capability's implementation maturity;
- the dependency/landing order;
- a release, deprecation, replacement, Figma file, or standards/research basis.

Keep this file buyer-oriented. Store the volatile per-PR classification in the
artifact required by issue #175, and link it here. Never rewrite an active-PR
capability as protected-main before merge and exact-head verification.
