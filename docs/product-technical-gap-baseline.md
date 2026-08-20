# Product and Technical Gap Baseline

**Status:** Live delivery baseline
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-08-20 22:10 KST
**Protected-main evidence:** `7c29e7c971d7940e1fb3def1ed3aae2d1bc8ad4a`
**Workspace version on protected main:** `0.1.0`

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
| Protected-main SHA | `7c29e7c971d7940e1fb3def1ed3aae2d1bc8ad4a` | All as-built claims are bounded to this commit. |
| Workspace members | 10 Rust crates | The foundation is modular, but the approved target contains additional semantic, estimator, compute, psychometric, event-intelligence, network, interpretation, artifact, and visual boundaries. |
| Open pull requests | **92** | The queue is itself a release blocker and requires consolidation. |
| Draft pull requests | **55** | Most queued work is not independently review-ready. |
| Non-draft pull requests | **37** | A non-draft state is not a qualifying review or required-check result. |
| Open product issues | **12** | Issue #156 plus the product-completion program #166–#176. |
| Current package version | `0.1.0` | No supported product release is established by the repository version alone. |

The pull-request counts come from the live GitHub search at this snapshot. The
full exact-head classification is owned by
[issue #175](https://github.com/ContextualWisdomLab/TEPP/issues/175); this file
keeps the buyer-level summary rather than duplicating a 92-row volatile ledger.

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

| ID | Buyer-visible gap | Maturity | Delivery authority | Closure evidence |
|---|---|---|---|---|
| GAP-001 | Submission can produce a durable accepted receipt, but protected `main` lacks the separate deterministic terminal-result lifecycle. | `active-PR` | [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) / [PR #157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | Exact request/result/snapshot/cutoff/model/profile binding, typed terminal failures, deterministic retrieval, current-head checks, and qualifying review. |
| GAP-002 | LineageWeave and other modular consumers cannot yet rely on the complete protected-main HTTP evidence/result boundary. | `active-PR stack` | [#107](https://github.com/ContextualWisdomLab/TEPP/pull/107) → [#155](https://github.com/ContextualWisdomLab/TEPP/pull/155) → [#158](https://github.com/ContextualWisdomLab/TEPP/pull/158) / [#159](https://github.com/ContextualWisdomLab/TEPP/pull/159) | Merge in dependency order with live loopback, framing, credential, idempotency, cutoff, tenant, and result-evidence tests. |
| GAP-003 | A buyer cannot run immutable evidence through estimation, validation, persistence, and terminal artifact retrieval as one product workflow. | `accepted-target` | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | Realistic end-to-end Compose/CLI/API execution, durable idempotent job lifecycle, production scientific metrics, restart/recovery, and reproducibility manifest. |
| GAP-004 | The central shared-latent temporal/relational topic estimator is absent. | `accepted-target` | [#167](https://github.com/ContextualWisdomLab/TEPP/issues/167) | Rust CPU `f64` fitting, sparse bounded parallelism, posterior artifacts, convergence, true-parameter RMSE/bias/coverage, and real candidate-K fitting. |
| GAP-005 | Real multilingual documents are not yet transformed into validated exact-span semantic units and versioned shared concepts. | `accepted-target` | [#168](https://github.com/ContextualWisdomLab/TEPP/issues/168) | Unicode/layout/language-tailored processing, unknown-concept review, multilingual calibration/invariance, image-position evidence, and prompt-injection tests. |
| GAP-006 | Posterior topic measurements cannot yet be fitted through a complete cross-classified longitudinal ESEM/DSEM engine. | `accepted-target` | [#169](https://github.com/ContextualWisdomLab/TEPP/issues/169) | Plausible-value/joint uncertainty, invariance, irregular event time, within/between separation, multiple membership, true-parameter recovery, and causal-claim refusal. |
| GAP-007 | TDT detection/tracking and CHRONOS schema/forecast/temporal reasoning remain isolated bounded gates rather than one calibrated product workflow. | `accepted-target` | [#170](https://github.com/ContextualWisdomLab/TEPP/issues/170) | Span-grounded mentions, calibrated TDT metrics, schema/forecast hypothesis states, interval consistency, known-truth recovery, persistence, and exports. |
| GAP-008 | GPU support is policy-only; no production estimator kernel has real hardware parity or declared VRAM evidence. | `accepted-target` | [#171](https://github.com/ContextualWisdomLab/TEPP/issues/171) | Real CUDA/portable backend execution, CPU parity, streamed memory, bounded OOM/fallback, hardware profiles, telemetry, and no skipped-support claim. |
| GAP-009 | Topic association and cluster outputs lack posterior-valid estimation, uncertainty, edge stability, and consensus communities. | `accepted-target` | [#172](https://github.com/ContextualWisdomLab/TEPP/issues/172) | Valid log-ratio coordinates, interval/stability-bearing edges, repeated Leiden consensus, known-truth network/cluster recovery, and reproducible exports. |
| GAP-010 | Buyers lack coordinated accessible visual analytics and exact-value export workflows. | `accepted-target` | [#173](https://github.com/ContextualWisdomLab/TEPP/issues/173) | Real Figma File ID in ADR, Storybook/design tokens, ten PRD views, exact-value tables, accessible interaction/print/PDF states, provenance, and source-consistent exports. |
| GAP-011 | TEPP is not yet an operable multi-tenant service or supported release. | `accepted-target` | [#174](https://github.com/ContextualWisdomLab/TEPP/issues/174) | Durable queue/storage, OIDC/RLS/purpose controls, OpenTelemetry/SLOs, load/recovery, migrations, signed release/SBOM/provenance, assurance evidence, and support policy. |
| GAP-012 | The 92-PR queue obscures authority, repeatedly stales exact-head evidence, and fragments product boundaries. | `release-blocker` | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) | Every PR classified; unique landing vehicles; compatible slices folded; superseded work closed with provenance; scheduler prioritizes consolidation; queue reaches zero before GA. |
| GAP-013 | Evidence-grounded LLM interpretation is routed but not executed and validated as a production interpreter/verifier port. | `partial` / `accepted-target` | [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176), [PR #69](https://github.com/ContextualWisdomLab/TEPP/pull/69), [PR #165](https://github.com/ContextualWisdomLab/TEPP/pull/165) | Contextual-orchestrator execution, evidence citations, verifier refusals, comparable-budget ablations, provider eligibility/fallback, abstention, live/offline contract tests, and no numerical-authority escalation. |
| GAP-014 | README/TRD and some PR descriptions lag protected-main and live queue reality. | `partial` | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) | Reconcile README, TRD, traceability, ADR maturity, CHANGELOG, preferred-merge declarations, and exact protected-main evidence. |
| GAP-015 | There was no canonical live product/buyer-gap register tied to documentation validation. | `active-PR` | [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164) | Merge this document and its validator/map integration after exact-head checks and independent review. |

## Product-completion issue register

| Issue | Product vertical | Depends on / constrains |
|---:|---|---|
| [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) | Completed analysis-run result contract | Must land before consumers can call accepted work a measurement result. |
| [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | Executable end-to-end analysis run | Integrates all scientific/service verticals; cannot substitute placeholders. |
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
| [#164](https://github.com/ContextualWisdomLab/TEPP/pull/164) | This product/technical gap baseline | Re-run exact-head documentation/repository checks and obtain independent review. |
| [#165](https://github.com/ContextualWisdomLab/TEPP/pull/165) | Hourly agent routing through contextual-orchestrator | Ensure queue-consolidation policy from #175 prevents unrelated micro-PR growth. |
| [#157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | Terminal result contract for #156 | Complete exact-head review/check gates; keep #156 open until protected-main verification. |
| [#107](https://github.com/ContextualWisdomLab/TEPP/pull/107) | Loopback analysis-run service | Review/merge before dependent consumer work. |
| [#155](https://github.com/ContextualWisdomLab/TEPP/pull/155) | Modular LineageWeave consumer parent | Preserve current stack parent and obtain independent review. |
| [#158](https://github.com/ContextualWisdomLab/TEPP/pull/158) | Temporal evidence context | Revalidate after its parent lands. |
| [#159](https://github.com/ContextualWisdomLab/TEPP/pull/159) | Cutoff-safe history projection | Revalidate after its parent lands. |
| [#48](https://github.com/ContextualWisdomLab/TEPP/pull/48) | Logistic-normal/ILR coordinate slice | Resolve requested changes and fold into the estimator landing plan; do not call it fitting. |
| [#51](https://github.com/ContextualWisdomLab/TEPP/pull/51) | VRAM budget/fallback policy types | Resolve requested changes; connect only to a real production kernel under #171. |
| [#53](https://github.com/ContextualWisdomLab/TEPP/pull/53) | Concept/language-profile gates | Integrate into the semantic pipeline authority under #168. |
| [#67](https://github.com/ContextualWisdomLab/TEPP/pull/67) | Model-selection gates | Rebase/merge only with an explicit path to real candidate fitting under #167. |
| [#69](https://github.com/ContextualWisdomLab/TEPP/pull/69) | Interpretation grounding/refusal metrics | Fold into the product interpreter/verifier under #176. |
| [#71](https://github.com/ContextualWisdomLab/TEPP/pull/71) | Compositional geometry/network pair metrics | Fold into the network estimator under #172. |
| [#119](https://github.com/ContextualWisdomLab/TEPP/pull/119) | Bounded loading/lag recovery slice | Consolidate into the psychometric engine under #169. |
| [#144](https://github.com/ContextualWisdomLab/TEPP/pull/144) | Draft multilevel event-time scalar recovery stack | Compare exact unique evidence and consolidate rather than land many scalar crates independently. |

## Delivery sequence

The dependency-aware product order is:

1. **Consolidate delivery authority:** #175 and PR #164.
2. **Finish live result/consumer contracts:** #156/#157 and #107 → #155 → #158/#159.
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
  form where applicable, explicit tenant/temporal/provenance authority, and
  measured hot-partition mitigation.
- Documents, web/search results, connector data, and LLM output are untrusted.
- Purpose-bound access and protected identity mappings preserve PII utility
  without broadcasting or blanket masking.
- External products integrate through versioned API/event/artifact contracts,
  never direct application-table access.
- CSAP/SOC 2/ISO/NIST alignment is readiness evidence, not certification.
- Every method/standard decision updates APA 7 doctoring and source-to-test
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
