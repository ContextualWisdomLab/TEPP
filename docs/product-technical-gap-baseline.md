# Product and Technical Gap Baseline

## 2026-08-26 Pair criterion and Project Journey posterior slice

- Active branch publishes strict Rust artifacts for
  `tepp.lineage_pair_criterion_posterior.v2` and
  `tepp.project_journey_posterior.v1`.
- The contracts preserve continuous criterion/event-time draws, distinct
  record time, multiple predecessors, branches, transitions, exact ties,
  TDT/CHRONOS provenance, unique anchor alignment, and method-derived CPU/MLX
  parity receipts. They reject fixed starts, nearest-date substitution,
  unsupported certainty, and consumer repair.
- Remaining release gap: no protected-main scientific estimator with
  CHRONOS event-time draw generation and real macOS-native MLX Metal parity
  produces these artifacts yet. The Rust CPU independent binary TDT-link
  criterion posterior now has deterministic synthetic parameter-recovery tests,
  and Rust qualitative relation draws have exact-recovery tests, but those
  bounded estimators are not evidence that calibrated Project Journey or
  channel-weight results are available.
- ADR 0025 is the normative Apple Silicon boundary: Rust-owned native MLX
  Metal behind authenticated local transport, exact backend receipts, Linux
  `rust_cpu`/`mlx_cpu`/`mlx_cuda`/`rust_opencl` portability, and fail-closed
  parity. The native service and hardware E2E remain a release gap.
- `mlx_native_receipt` provides a macOS-only, Rust-owned MLX CPU execution
  probe. Its receipt proves only the stated matrix objective and cannot be
  reused as an Event Lineage estimator or Metal receipt.
- `event_core` now materializes producer-identified discrete event-time mass
  into canonical complete draws and recovers synthetic mass exactly. Inferring
  the event-time atoms/mass from admitted evidence and binding the estimator's
  own MLX receipt remain open; record time and nearest-date substitution stay
  prohibited.
- `analysis_engine` now executes exhaustive actual `D \ {i}` fitter calls and
  retains full/deleted seed-domain and corpus identities. The remaining gap is
  the scientific temporal topic fitter plus unique anchor alignment, incident
  relation/membership deletion, artifact assembly, and estimator-bound backend
  parity; the runner alone does not publish case-deletion influence.

## 2026-08-25 Event Lineage anchor contract slice

- Exact base: protected `main` `cf0e0ad74d23c5d2e0e33d389bb0bb4d37067c31`.
- This branch publishes TEPP's strict request identity and
  `tepp.lineage_criterion_anchor.v1` accepted/rejected artifact contract.
- The buyer-visible integrity gain is fail-closed: LineageWeave cannot promote
  fast-mlsirm's internal response structure into calibrated Event Lineage
  weights without an exact TEPP-authored criterion result.
- Remaining product gap: the registered TEPP criterion estimator and terminal
  artifact delivery are not implemented by this contract slice. Until they
  exist and pass scientific recovery/validity gates, production activation
  remains unavailable; the consumer must not invent a substitute.
- Acceptance evidence for this slice: complete `tepp_api` tests, warning-free
  clippy, strict unknown-field/provenance rejection, schema and ADR/API
  traceability, followed by exact-head protected checks and independent review.

**Status:** Live delivery baseline
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-08-26T07:00:00Z
**Protected-main evidence:** `2f2119dffa26f4823026445bfec8e249804dc93c` (merge of [PR #250](https://github.com/ContextualWisdomLab/TEPP/pull/250) at 2026-08-26T06:30:15Z; restores Driver p.16 `asymCINTstd`)
**Workspace version on protected main:** `0.2.0`
**Canonical gap-baseline authority:** [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164). [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164) merged; this file is now maintained by follow-up refresh PRs against protected main.

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
| Protected-main SHA | `c482ccea…` (2026-08-25T09:23Z, merge of [#239](https://github.com/ContextualWisdomLab/TEPP/pull/239)) | All as-built claims are bounded to this commit. |
| Workspace members | 57 unique Rust crates | The repository is modular, but the approved target still lacks complete semantic, compute, psychometric-engine, event-intelligence, interpretation, artifact, and visual product boundaries. |
| Workspace version | `0.2.0` (aligned across every crate manifest by #239) | A version number alone does not establish a supported product release; no signed artifact or support policy exists yet. |
| Open pull requests | **5** | Active queue: #257 lineage criterion + mlx crate, #256 exact deletion refit, #262 T0MEANSstd (rebased), #264 gap-baseline refresh (this PR), #265 T0VARstd. |
| Draft pull requests | **3** | The 31-draft psychometric recovery stack fully drained into integration vehicles #231/#232 (both merged 2026-08-25T06:2xZ); remaining drafts are repair and event-intelligence slices, not a stacked chain. |
| Open product issues | **9** | Issues #166–#167 and #169–#174 plus #176 remain open. Result-contract issue #156, semantic-units issue #168, and queue-consolidation issue #175 are all CLOSED. |
| Current package version | `0.2.0` | No supported product release is established by the repository version alone; #235 proposes the tagged cut but has not merged. |

The pull-request counts come from the live GitHub search at this snapshot. The
full exact-head classification lives in this register; re-read live GitHub
state immediately before every mutation. Passing or queued Checks on an open PR never
promote that PR to implemented-main.

### Post-#239 state note

[#239](https://github.com/ContextualWisdomLab/TEPP/pull/239) merged as squash
`c482ccea` and landed two things operators must know:

1. **network_analysis estimator repairs before consumer dependence:** exact
   two-sided Fisher z-transform p-values replace pseudo-p-values; fail-closed
   guard ordering for non-finite correlations and short samples; negative-effect
   edges excluded from the whole consensus perturbation pipeline; explicit
   validated `edge_drop_probability`; bounds-safe admission helpers; and exact
   red-to-green branch cases. This advances GAP-009's estimator core beyond the
   #230 merge (`a69eb3e2`) it builds on.
2. **Workspace version alignment 0.1.0 → 0.2.0** across every crate manifest,
   matching the CHANGELOG `[0.2.0] - 2026-08-25` entry. The version bump is not
   itself a release: no tag, signed artifact, SBOM/provenance bundle, or support
   policy exists yet ([GAP-011](#operator-gap-register)).

### Queue-consolidation progress (GAP-012) — COMPLETE (issue #175 closed)

The main non-draft pull-request queue reached **zero** at 2026-08-25T02:30Z and
issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) is CLOSED.
All previously queued slices landed on protected main through:

1. the hourly scheduler (independent merges of ~40 PRs);
2. batch integration vehicle [#215](https://github.com/ContextualWisdomLab/TEPP/pull/215) (31 folded green slices with per-slice merge-commit provenance); and
3. individual rebase-and-admin-merge passes for the remainder, including the
   psychometric recovery stack drained through vehicles
   [#231](https://github.com/ContextualWisdomLab/TEPP/pull/231)/[#232](https://github.com/ContextualWisdomLab/TEPP/pull/232),
   coverage repair [#219](https://github.com/ContextualWisdomLab/TEPP/pull/219)
   (merged 2026-08-25T03:17Z), terminal-result contract
   [#157](https://github.com/ContextualWisdomLab/TEPP/pull/157) (merged
   2026-08-25T02:53Z), posterior network estimator
   [#230](https://github.com/ContextualWisdomLab/TEPP/pull/230) (merged
   2026-08-25T06:24Z), and network-repair/version-alignment
   [#239](https://github.com/ContextualWisdomLab/TEPP/pull/239).

The residual seven open PRs are new forward work (release cut, register refresh,
API anchor contract, coverage completion, event-intelligence fold), not backlog.

## Snapshot open pull-request evidence

The following exact-head register was fetched live from GitHub at
2026-08-25T10:12:54Z against protected main `c482ccea`. Review decisions,
required Checks, and mergeability remain volatile; the live GitHub API
supersedes this snapshot. `draft=false` is not approval, mergeability, or a
passing-check claim. Re-read the full SHA, current review decision, required
Checks, and branch rules immediately before every mutation.

| PR | Exact current head | Draft | Base | Title |
| #256 | `7ad9d9e3a149f03e6a9edb111e35412d5ae24763` | false | feat/topic-exact-loo-refit-cpu | feat(topic): add exact deletion refit prerequisite |
| #257 | `e616c06a20824f2cdbd425e5c4e39b28fb805f5a` | false | feat/lineage-pair-criterion-producer | feat(api): publish lineage criterion and Project Journey posteriors |
| #262 | `63e0695e6d83910bb475984126d7c400aac7c109` | false | agent/psychometric-t0-means-std | feat(psychometric): restore Driver p.16 T0MEANSstd on main |
| #264 | `2439d72a57b4df91bca6f774a84576fb21b8a5c0` | false | docs/gap-baseline-refresh | docs(gap): refresh baseline snapshot for post-#250 state |
| #265 | `6c9e2cd7de402d0f60a3abed9e78f7aa4da1b863` | false | agent/psychometric-t0-var-std | feat(psychometric): restore Driver p.16 T0VARstd on main |
|---:|---|:---:|---|---|







Note on #241: the #239 squash-merge landed through its pre-fix head, so #241
cherry-picks the final coverage-and-semantics commit (dead singleton-guard
removal in `network_analysis::consensus`, unreachable let-else replacement in
`evidence_core::image_unit`) onto current main. Until #241 merges, protected
main carries those two dead-branch semantics points; they are test debt, not a
weakened 100% gate.

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

Protected `main` contains 57 unique Rust crate boundaries in the current
workspace manifest (as of `c482ccea`). The `members` and `default-members`
arrays enumerate the same crate set for distinct Cargo commands; the
unique-crate count is the authoritative modularity measure.
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
topic_measurement
network_analysis
psychometric_core
analysis_engine
interpretation_gateway
compute_backend
```

The traceability ledger records meaningful protected-main implementation in
immutable evidence, six clocks and interval reasoning, forward transitions,
event mention/instance separation, weighted multiple membership, cutoff-safe
splits, validation metrics, simulations, PostgreSQL slices, versioned API/export
contracts, orchestration routing, privacy authorization, release-evidence
generation, the CPU topic-measurement reference estimator, the repaired
posterior network estimator (#230 + #239), the Driver et al. (2017) SDE
recovery suite (#231/#232), the deterministic analysis-run execution engine,
the loopback interpretation gateway, and VRAM-policy compute types.

Protected `main` does **not** yet establish the complete approved product. In
particular, it does not contain the full multilingual semantic pipeline beyond
the first span slice, full Bayesian candidate-`K` topic fitting, a composed
longitudinal ESEM/DSEM estimation engine, a calibrated TDT/CHRONOS workflow,
repeated Leiden consensus clustering with buyer-facing exports, real accelerator
kernels with hardware parity, an executed contextual-orchestrator interpreter,
the coordinated visual workspace, or a supported multi-tenant release.

## Operator-gap register

| ID | Operator-visible gap | Maturity | Delivery status | Protected-main authority | Current delivery authority | Current head SHA | Closure evidence |
|---|---|---|---|---|---|---|---|
| GAP-001 | Submission produces a durable accepted receipt, and the deterministic terminal-result lifecycle is now implemented-main. | `implemented-main` | closed on protected main | `340087494b0a` lineage merged through [PR #157](https://github.com/ContextualWisdomLab/TEPP/pull/157) (merged 2026-08-25T02:53Z); [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) CLOSED | — | — | Exact request/result/snapshot/cutoff/model/profile binding, typed terminal failures, deterministic retrieval, and cutoff-safe execution are protected-main behavior as of the #157 merge. |
| GAP-002 | LineageWeave and other modular consumers can rely on the complete protected-main HTTP evidence/result boundary. | `partial` | consumer hardening remains | Terminal-result lifecycle implemented-main via #157; versioned API contract intact | [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) (closed) / [PR #155](https://github.com/ContextualWisdomLab/TEPP/pull/155) (merged) | — | Remaining work is consumer-side adoption evidence and any versioned-contract drift discovered during integration; core boundary is no longer the gap it was. |
| GAP-003A | Immutable evidence cannot yet be submitted to a durable validation run that produces operator-usable scientific acceptance evidence. | `accepted-target` | product-completion | `e65cd66` (validation metrics are library-level only) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | `—` (issue program; no current implementation PR) | Compose/CLI/API execution must bind immutable evidence, cutoffs, model configuration, validation metrics, and reproducibility manifests to one idempotent run. |
| GAP-003B | Scientific result artifacts cannot yet be persisted, restarted, and recovered as one supported operator workflow. | `accepted-target` | product-completion | `e65cd66` (persistence contracts lack E2E recovery) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | `—` (issue program; no current implementation PR) | Durable storage, migration/rollback, restart/recovery, artifact digest verification, and terminal retrieval must pass against a real Compose deployment. |
| GAP-003C | The persistence slice classifies concurrent-write SQLSTATEs, but has no measured hot-partition detection, routing, or mitigation for tenant/result workloads. | `accepted-target` | product-completion | `e65cd66` (conflict classification only; no measured partition control) | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | `—` (issue program; no current implementation PR) | A real Compose/PostgreSQL workload identifies hot keys and partition skew, applies bounded tenant/time or result routing without weakening 3NF or temporal authority, and proves conflict rate, latency, recovery, and migration/rollback behavior under load. |
| GAP-004 | The central shared-latent temporal/relational topic estimator is absent. | `partial` | product vertical | CPU `f64` TRSL-TM reference estimator with ALR/ILR coordinates and refusal gates is implemented-main (v0.2.0 `topic_measurement`); fitted candidate-`K` scoring present | [#167](https://github.com/ContextualWisdomLab/TEPP/issues/167) | — | GPU, method effects, full Bayesian sampling, and topic birth/split/merge remain. This is not full #167 closure. |
| GAP-005 | Real multilingual documents are not yet transformed into validated exact-span semantic units and versioned shared concepts. | `partial` | product vertical | `e65cd66` lineage (semantic_core exact-span units and language-profile validation are implemented-main as the first slice from [PR #201](https://github.com/ContextualWisdomLab/TEPP/pull/201)) | [#168](https://github.com/ContextualWisdomLab/TEPP/issues/168) CLOSED COMPLETED 2026-08-24; residual evidence tracked under product completion (#166/#169) | `—` | Remaining evidence beyond the closed first slice: concept alignment, Unicode/layout/language-tailored processing, unknown-concept review, multilingual calibration/invariance, image-position evidence, and prompt-injection tests. |
| GAP-006 | Posterior topic measurements cannot yet be fitted through a complete cross-classified longitudinal ESEM/DSEM engine. | `partial` | product vertical | Psychometric recovery stack drained onto protected main through integration vehicles [#231](https://github.com/ContextualWisdomLab/TEPP/pull/231)/[#232](https://github.com/ContextualWisdomLab/TEPP/pull/232) (both merged 2026-08-25T06:2xZ): Driver et al. (2017) SDE discrete-time recovery suite with standardised-parameter families and true-parameter RMSE tests is implemented-main (`psychometric_core`) | [#169](https://github.com/ContextualWisdomLab/TEPP/issues/169) | — | Remaining: joint plausible-value uncertainty wiring, full invariance evidence, irregular event-time fitting at production scale, multiple-membership integration with posterior coordinates, and end-to-end composition under #166/#167. Recovery primitives alone are not the ESEM/DSEM engine. |
| GAP-007 | TDT detection/tracking and CHRONOS schema/forecast/temporal reasoning remain isolated bounded gates rather than one calibrated product workflow. | `accepted-target` | product vertical | `e65cd66` (event/time primitives only) | [#170](https://github.com/ContextualWisdomLab/TEPP/issues/170) / [PR #70](https://github.com/ContextualWisdomLab/TEPP/pull/70) | `7a1f33aa68c1` | Span-grounded mentions, calibrated TDT metrics, schema/forecast hypothesis states, interval consistency, known-truth recovery, persistence, and exports. |
| GAP-008 | GPU support is policy-only; no production estimator kernel has real hardware parity or declared VRAM evidence. | `accepted-target` | product vertical | `e65cd66` (VRAM policy only) | [#171](https://github.com/ContextualWisdomLab/TEPP/issues/171) / [PR #51](https://github.com/ContextualWisdomLab/TEPP/pull/51) | `1801501c4d7c` | Real CUDA/portable backend execution, CPU parity, streamed memory, bounded OOM/fallback, hardware profiles, telemetry, and no skipped-support claim. |
| GAP-009 | Topic association and cluster outputs lacked posterior-valid estimation, uncertainty, edge stability, and consensus communities. | `partial` (estimator core + repairs landed; Leiden consensus + buyer workflow remain) | product vertical | `a69eb3e2` (posterior log-ratio edge estimator merged from [PR #230](https://github.com/ContextualWisdomLab/TEPP/pull/230)) advanced by [#239](https://github.com/ContextualWisdomLab/TEPP/pull/239) (`c482ccea`): exact two-sided Fisher z-transform p-values driving Benjamini–Hochberg admission (Benjamini & Hochberg, 1995), percentile-bootstrap credible intervals and selection fractions (Efron, 1979), fail-closed guard ordering for non-finite correlations and short samples, negative-effect edges excluded from the whole consensus perturbation pipeline, explicit validated `edge_drop_probability`, bounds-safe admission helpers | [#172](https://github.com/ContextualWisdomLab/TEPP/issues/172) | — (#241 carries two dead-guard removals) | Remaining closure evidence: repeated Leiden consensus replacing the union-find stand-in (Traag et al., 2019), known-truth network/cluster recovery at production scale, and reproducible exports wired into the end-to-end run (#166). |
| GAP-010 | Operators lack coordinated accessible visual analytics and exact-value export workflows. | `accepted-target` | product vertical | `e65cd66` (no visual workspace) | [#173](https://github.com/ContextualWisdomLab/TEPP/issues/173) | `—` (Figma work not started) | Real Figma File ID in ADR, Storybook/design tokens, ten PRD views, exact-value tables, accessible interaction/print/PDF states, provenance, and source-consistent exports. |
| GAP-011 | TEPP is not yet an operable multi-tenant service or supported release. | `accepted-target` | product vertical | `e65cd66` (library contracts only) | [#174](https://github.com/ContextualWisdomLab/TEPP/issues/174) | `—` (issue program; no current implementation PR) | Durable queue/storage, OIDC/RLS/purpose controls, OpenTelemetry/SLOs, load/recovery, migrations, signed release/SBOM/provenance, assurance evidence, and support policy. |
| GAP-012 | The 71-PR queue obscured authority, repeatedly staled exact-head evidence, and fragmented product boundaries. | `implemented-main` (consolidation complete) | closed | `c482ccea` (#239 merged 2026-08-25T09:23Z); [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) CLOSED; queue drained from 71 through #215, the hourly scheduler, vehicles #231/#232, and individual passes | — | — | Residual queue is seven forward-work PRs (release cut, register refresh, API anchor contract, coverage completion, event-intelligence fold); no backlog remains. Exact-head discipline stays enforced by this register's refresh rule. |
| GAP-013 | Evidence-grounded LLM interpretation is routed but not executed and validated as a production interpreter/verifier port. | `partial` | active integration | `e65cd66` lineage (routing and refusal contracts implemented-main; loopback interpretation POSTs landed via #92/#107) | [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176), [PR #69](https://github.com/ContextualWisdomLab/TEPP/pull/69), [PR #165](https://github.com/ContextualWisdomLab/TEPP/pull/165) | `8e4a3ca9cc80` / `34083c3f5d66` | Contextual-orchestrator execution, evidence citations, verifier refusals, comparable-budget ablations, provider eligibility/fallback, abstention, live/offline contract tests, and no numerical-authority escalation. |
| GAP-014 | README/TRD and some PR descriptions lag protected-main and live queue reality. | `partial` | documentation drift | Documentation synchronized to `c482ccea` through register refreshes including this file and [#236](https://github.com/ContextualWisdomLab/TEPP/pull/236) | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) (closed) | — | Remaining: reconcile README/TRD crate counts to 57, retire CHANGELOG `[Unreleased]` bullets superseded by the v0.2.0 entry (duplicated gate-integrity bullets and the now-landed network_analysis repair bullet), and keep ADR maturity current. |
| GAP-015 | There was no canonical live product/operator-gap register tied to documentation validation. | `implemented-main` (register + validator); this refresh is the live maintenance slice | register refresh | `c482ccea` (register and validator are implemented-main; this snapshot refresh keeps the volatile queue current) | [PR #217](https://github.com/ContextualWisdomLab/TEPP/pull/217) merged; [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164) is the merged authority; open [#236](https://github.com/ContextualWisdomLab/TEPP/pull/236) is a parallel refresh whose base predates #239 | `—` (each refresh PR carries its own current head at merge time) | Land each refresh after exact-head checks and independent review, then regenerate it whenever protected-main or the live queue changes. Rebase or supersede #236 against `c482ccea` so only one refresh authority lands per queue state. |
| GAP-016 | Hourly PR maintenance used an older central scheduler revision whose per-repository sweep budgets could amplify the queued review workload. | `active-PR` | operability hardening | `e65cd66` (caller pin before central budget hardening) | [PR #177](https://github.com/ContextualWisdomLab/TEPP/pull/177) | `580d45206536` | The change pins a verified central revision immutably; closure still requires exact-head hosted Checks, resolved threads, and independent review. With the queue near zero the urgency is low but the slice remains unlanded. |
| GAP-017 | Accepted analysis runs have a terminal DTO and cutoff-safe execution on protected main after #157 merged. | `implemented-main` | closed on protected main | [PR #157](https://github.com/ContextualWisdomLab/TEPP/pull/157) merged 2026-08-25T02:53Z carrying the terminal result contract and folded cutoff-safe execution from closed stacked PR #178 | [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) (closed) / [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | — | Exact availability cutoff, snapshot binding, multiple-membership preservation, digest integrity, redacted no-eligible failure, and realistic end-to-end tests are protected-main behavior; remaining E2E composition work belongs to #166. |
| GAP-018 | Coverage debt: the #219 repair landed, and #239 added 27 exact branch cases, leaving two dead-guard semantics points queued on [#241](https://github.com/ContextualWisdomLab/TEPP/pull/241). | `active-PR` | coverage completion | `5c8599442e85` lineage + #219 (merged 2026-08-25T03:17Z) + #239 (`c482ccea`): gates enforce 100% unique production arms; remaining items are dead-branch removals in `network_analysis::consensus` and `evidence_core::image_unit` | [PR #241](https://github.com/ContextualWisdomLab/TEPP/pull/241) | `a12e48c1e61b` | Merge #241 after exact-head Checks plus independent review so protected main carries the final semantics cleanup; two provably-dead singleton guards and an unreachable let-else are the last known items. |

## Product-completion issue register

| Issue | Product vertical | Depends on / constrains |
|---:|---|---|
| [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) **CLOSED** | Completed analysis-run result contract | Landed on protected main through PR #157 (merged 2026-08-25T02:53Z). |
| [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | Executable end-to-end analysis run, recovery, and hot-partition readiness | Integrates all scientific/service verticals; cannot substitute placeholders or hide write skew behind an unmeasured queue. |
| [#167](https://github.com/ContextualWisdomLab/TEPP/issues/167) | Shared-latent temporal topic CPU estimator | Numerical foundation for K selection, networks, psychometrics, interpretation, and product E2E; CPU reference landed, full estimator remains. |
| [#168](https://github.com/ContextualWisdomLab/TEPP/issues/168) **CLOSED** | Multilingual semantic units and concept dictionary | Closed COMPLETED 2026-08-24; first-slice span units are implemented-main, remaining invariance/calibration evidence tracks product completion elsewhere. |
| [#169](https://github.com/ContextualWisdomLab/TEPP/issues/169) | Multilevel longitudinal ESEM/DSEM | Consumes posterior topic coordinates and membership/time contracts; recovery stack landed via #231/#232, engine composition remains. |
| [#170](https://github.com/ContextualWisdomLab/TEPP/issues/170) | TDT/CHRONOS event intelligence | Consumes evidence/time/event contracts and supplies calibrated event artifacts; span-grounding fold queued as draft #233. |
| [#171](https://github.com/ContextualWisdomLab/TEPP/issues/171) | Real GPU compute and parity | Accelerates production estimators only after CPU authority is stable. |
| [#172](https://github.com/ContextualWisdomLab/TEPP/issues/172) | Posterior network and consensus clustering | Estimator core plus #239 repairs landed; Leiden consensus and buyer workflow remain. |
| [#173](https://github.com/ContextualWisdomLab/TEPP/issues/173) | Accessible visual analytics and exports | Starts after stable API/artifact contracts; requires Figma and Storybook evidence. |
| [#174](https://github.com/ContextualWisdomLab/TEPP/issues/174) | Commercial deployment/release/support | Wraps a scientifically complete product without weakening gates; v0.2.0 version alignment (#239) is a prerequisite slice, not closure. |
| [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) **CLOSED** | PR queue and delivery consolidation | Queue consolidation completed at near-zero; issue closed after the residual queue drained through #239. |
| [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176) | Contextual-orchestrator interpreter/verifier | Consumes validated artifacts and cannot promote scientific truth. |

## Priority pull-request queue

This table lists every open pull request at snapshot time. The pull request's
live page is authoritative because its head can change after this file is
committed.

| PR | Current delivery role | Required next action |
|---:|---|---|
| [#235](https://github.com/ContextualWisdomLab/TEPP/pull/235) | Tagged v0.2.0 release cut over the aligned workspace (#239 already aligned versions on main) | Exact-head Checks plus independent review; verify tag/artifact/SBOM evidence before treating 0.2.0 as a supported release (#174). |
| [#236](https://github.com/ContextualWisdomLab/TEPP/pull/236) | Register refresh against pre-#239 base `cf0e0ad7` | Rebase onto `c482ccea` or supersede against this file so only one post-#239 refresh authority lands. |
| [#237](https://github.com/ContextualWisdomLab/TEPP/pull/237) | TEPP lineage criterion anchor contract for API consumers | Exact-head Checks plus independent review; keep versioned consumer contracts intact (GAP-002). |
| [#241](https://github.com/ContextualWisdomLab/TEPP/pull/241) | Final branch-gap coverage cherry-pick after the #239 squash-merge (dead singleton guards, unreachable let-else) | Merge before further network/evidence coverage-gated slices; exact-head Checks plus review (GAP-018). |
| [#240](https://github.com/ContextualWisdomLab/TEPP/pull/240) | Draft repair of #232 rustfmt and psychometric test-arity fallout | Convert from draft once the repair is complete; land before or with the next psychometric slice. |
| [#238](https://github.com/ContextualWisdomLab/TEPP/pull/238) | Draft coverage of `is_image_media_type_token` non-image prefix arm | Fold into #241 or land independently after exact-head Checks; do not let it stale behind #241. |
| [#233](https://github.com/ContextualWisdomLab/TEPP/pull/233) | Draft fold of span grounding into EventMention (#170 / GAP-007) | Keep as event-intelligence forward work; convert when #170 landing sequence resumes. |

## Delivery sequence

The dependency-aware product order is (✓ = landed on protected main):

1. ✓ **Consolidate delivery authority:** #175 closed; PR #164 merged; queue drained through #239.
2. ✓ **Finish live result contracts:** #156/#157 merged; the LineageWeave consumer parent #155 is implemented-main.
3. ✓ **Build validated multilingual evidence (first slice):** #168 closed COMPLETED 2026-08-24 with span units implemented-main from #201; remaining alignment/invariance evidence tracks product completion under #166/#169.
4. **Build the CPU topic estimator:** #167 — reference estimator landed, full Bayesian/candidate-K fitting remains.
5. **Build event intelligence and posterior networks:** #170 and #172 — network estimator core plus #239 repairs landed; Leiden consensus, buyer workflow, and event-intelligence calibration remain (#233 queued).
6. **Build the posterior-aware longitudinal psychometric engine:** #169 — recovery stack landed via #231/#232; engine composition remains.
7. **Accelerate real kernels with parity:** #171.
8. **Complete the durable end-to-end run:** #166 — terminal-result lifecycle and analysis-run execution engine are implemented-main; full E2E validation remains.
9. **Execute and validate interpretation:** #176.
10. **Design and implement the operator workspace:** #173.
11. **Productionize and release:** #174 — v0.2.0 version alignment landed (#239); tagged cut pending on #235.

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

Keep this file operator-oriented. The volatile per-PR classification lives in
this register's snapshot tables (issue #175 is closed; no separate artifact is
required). Never rewrite an active-PR
capability as protected-main before merge and exact-head verification.
