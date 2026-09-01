# Product and Technical Gap Baseline

**Status:** Active delivery recovery
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-09-01 13:50 KST
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`
**Workspace version:** `0.2.0`
**Delivery authority:** issue [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), this register, and [`docs/delivery/pr-queue-authority-2026-09-01.md`](delivery/pr-queue-authority-2026-09-01.md)
**DDD authority:** [`docs/architecture/domain-context-map.md`](architecture/domain-context-map.md)

## Purpose

This document is the current operator-facing authority for product and technical gaps. Historical queue snapshots remain available in Git history; they are not copied forward when their facts are no longer true.

A planning document, local test, queued or skipped check, predecessor-head result, mergeable branch, separate crate, ADR number, or LLM judgment does not make a capability shipped. Re-read live GitHub state before every merge, customer claim, release claim, certification claim, or closure decision.

## Live snapshot

| Signal | Current evidence | Delivery implication |
| --- | --- | --- |
| Protected `main` | `1bc02f580cf48e1d39da239f0e818453437c31c3` | All protected-main capability claims are bounded to this commit until `main` changes. |
| Workspace | 58 Rust crates | Cargo modularity is implementation structure, not proof of 58 bounded contexts. Several crates are one-clock/one-edge/one-rule fragments that belong inside larger domain boundaries. |
| Workspace version | `0.2.0` | Version metadata is not a supported release. |
| Open pull requests | **142** | The delivery queue is again a release blocker. This count includes the queue-recovery PR created from this snapshot. |
| Draft pull requests | **100** | Draft state is not a substitute for classification; every remote head still needs an owning bounded context and a replacement/landing decision. |
| Non-draft pull requests | **42** | A non-draft PR is not merge-ready without exact-head ruleset evidence and resolved scientific/product blockers. |
| Open issues | **13** | Product-completion and newly discovered equation/recovery work coexist; issue count alone does not define product priority. |
| Required ruleset | `18156473` — `CWL Central required workflows` | Organization ruleset is the effective merge authority: one approving review, stale-approval dismissal, resolved conversations, unattributed-change approval, and central required workflows. |

The classic branch-protection payload reports no status-check contexts, but that is not the effective policy source. The active organization ruleset applies to the default branch and supplies the review/workflow gates. Do not bypass it merely because repository-level classic protection looks empty.

## Strategic Domain-Driven Design baseline

Cargo crates are not automatically bounded contexts. The current product responsibilities are:

### Core subdomains

| Bounded context | Aggregate authority | Primary implementation nucleus |
| --- | --- | --- |
| Evidence & Semantic Measurement | `EvidenceCorpus`, `SemanticUnitSet`, `ConceptDictionaryRevision` | `evidence_core`, `semantic_core` |
| Temporal Event Knowledge | `EventEpisode`, `TemporalRelationSet`, `MembershipAssignmentSet` | `temporal_core`, `event_core`, `relation_graph`, `membership_core` |
| Topic Measurement | `TopicModelRun`, `TopicLineage` | `topic_measurement`, `topic_lineage`, `model_selection`, `network_analysis` |
| Longitudinal Psychometrics | `PsychometricStudy`, `LongitudinalModelRun` | `psychometric_core`, `longitudinal_core`, `psychometric_fit` |
| Analysis Run | `AnalysisRun` | `analysis_engine` |
| Scientific Validation & Claim Promotion | `ValidationStudy`, `ClaimPromotionDecision` | `validation_core`, `tepp_simulation`; claim-promotion policy remains incomplete |

### Supporting subdomains

- Interpretation
- Persistence & Recovery
- Runtime Security & Operations

### Generic subdomains

- compute backend execution
- MLX/native execution receipts
- serialization, hashing, transport framing

Detailed context relationships, ubiquitous language, anti-corruption layers, and staged path repairs are normative in [`domain-context-map.md`](architecture/domain-context-map.md).

### Dependency invariants

- Transport/UI/persistence adapters depend on domain/application contracts; domain code does not depend on HTTP, PostgreSQL, CLI, or provider DTOs.
- `tepp_api` is an adapter around Analysis Run and published read models. It does not own estimator mathematics, scientific acceptance, temporal truth, or persistence truth.
- `persistence_postgres` implements repositories; other bounded contexts do not read its tables directly.
- contextual-orchestrator, Naruon, and LineageWeave remain external contexts behind anti-corruption layers.
- A compute/backend receipt proves execution of the named backend operation, not scientific validity.
- LLM output can propose or verify interpretation; it cannot satisfy numerical evidence or promote a scientific claim.

## Active delivery gaps

| ID | Gap | Current maturity | Delivery authority | Required closure evidence |
| --- | --- | --- | --- | --- |
| GAP-001 | PR authority is fragmented across 142 open heads and the former zero-queue baseline became stale. | `release-blocking` | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175), [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435) | Every open PR classified by exact head and bounded-context ownership; coherent landing vehicles selected; duplicates/superseded slices closed only after unique evidence is preserved; queue reduced without protection bypass. |
| GAP-002 | Real multilingual evidence is only partially transformed into validated span-grounded semantic/concept observations with complete language/profile/invariance evidence. | `partial` | Evidence & Semantic Measurement vertical; historical #168 first slice is closed | Immutable offsets, Unicode/layout-aware segmentation, versioned concept dictionary, shared-concept/native-lexical channels, unknown-concept review, language-profile calibration/invariance, prompt-injection and image-position evidence. |
| GAP-003 | Shared-latent temporal topic measurement is not yet a complete production scientific estimator. | `partial` | [#167](https://github.com/ContextualWisdomLab/TEPP/issues/167) | Rust CPU `f64` estimator over admitted evidence; explicit likelihood/estimands; uncertainty; time/relation/multiple-membership effects; deterministic reduction; multiple seeds/initialisations; known-truth RMSE/bias/coverage/convergence; real candidate-`K` fits. |
| GAP-004 | Analysis Run is not yet one durable buyer workflow from immutable evidence through scientifically promotable result artifacts and restart/recovery. | `partial` | [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | Idempotent lifecycle, persistence/recovery, terminal artifacts, estimator-bound provenance, complete validation evidence, separate claim-promotion decision, Compose E2E, hot-partition evidence. |
| GAP-005 | Longitudinal/multilevel psychometrics remains fragmented across scalar recovery slices rather than one composed ESEM/DSEM product boundary. | `partial` | [#169](https://github.com/ContextualWisdomLab/TEPP/issues/169) | Coherent Rust longitudinal model, plausible-value uncertainty, irregular time, multiple membership, invariance, known-truth recovery, bounded CPU/GPU parity, and one public product contract. |
| GAP-006 | TDT/CHRONOS event intelligence remains bounded components rather than one calibrated temporal-event workflow. | `partial` | [#170](https://github.com/ContextualWisdomLab/TEPP/issues/170) | Span-grounded event evidence, calibrated TDT tasks, CHRONOS schema/forecast state, interval consistency, known-truth recovery, durable artifacts and exports. |
| GAP-007 | Real accelerator support and memory control are incomplete. | `accepted-target` | [#171](https://github.com/ContextualWisdomLab/TEPP/issues/171) | Real hardware execution, CPU `f64` parity, streamed/bounded memory, OOM/fallback evidence, no skipped-hardware claim. |
| GAP-008 | Posterior network/cluster workflow is incomplete as a buyer-visible stable product vertical. | `partial` | [#172](https://github.com/ContextualWisdomLab/TEPP/issues/172) | Known-truth network recovery, uncertainty/stability, repeated Leiden consensus, accessible exact-value exports and reproducible run binding. |
| GAP-009 | Evidence-grounded interpretation is not yet a validated production interpreter/verifier workflow. | `partial` | [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176) | contextual-orchestrator execution, citations, independent verifier, comparable-budget ablations, fallback/abstention, no numerical-authority escalation. |
| GAP-010 | Coordinated buyer UI and exact-value accessible visual analytics are incomplete. | `accepted-target` | [#173](https://github.com/ContextualWisdomLab/TEPP/issues/173) | Real Figma file ID, Storybook inventory, design tokens, keyboard/touch/accessibility states, exact-value tables, print/PDF/JSON/SVG provenance. |
| GAP-011 | TEPP is not yet an operable supported multi-tenant release. | `accepted-target` | [#174](https://github.com/ContextualWisdomLab/TEPP/issues/174) | OIDC/RLS/purpose controls, durable queue/storage, OpenTelemetry/SLOs, backup/restore, load/recovery, migrations, signed release/SBOM/provenance, support policy. |
| GAP-012 | Directory/crate structure contains technical fragments that obscure bounded-context ownership. | `active-refactor` | [#435](https://github.com/ContextualWisdomLab/TEPP/pull/435) and each subsequent landing vehicle | Incremental path/crate folds by domain responsibility, explicit compatibility adapters where required, no cross-context direct DB access, no cyclic dependency or Shared-Kernel expansion. |

## Delivery sequence

The current priority is dependency-driven rather than PR-number-driven:

1. Restore queue and bounded-context authority (#175 / #435).
2. Consolidate Evidence & Semantic Measurement so real multilingual source evidence has one admission/measurement boundary.
3. Complete the real Rust CPU `f64` shared-latent topic estimator (#167).
4. Complete the durable end-to-end Analysis Run and scientific evidence/promotion boundary (#166).
5. Compose Longitudinal Psychometrics (#169) and Event Intelligence (#170) instead of continuing one-equation product slices.
6. Add real accelerator parity only after the CPU scientific authority is complete (#171).
7. Complete posterior network/cluster, interpretation, and buyer visual workflows (#172/#176/#173).
8. Productionize tenancy, durability, observability, recovery, release, and support (#174).

A small dependency repair may land ahead of this list when it directly unblocks a selected landing vehicle. It does not create a new product priority.

## Queue consolidation rules

Every open PR receives one classification:

- `landing_vehicle`
- `stacked_dependency`
- `fold_into_landing_vehicle`
- `superseded`
- `duplicate`
- `research_lineage_only`
- `blocked_external`

No PR is closed because its title looks similar to another. Before `superseded`, `duplicate`, or `fold_into_landing_vehicle` is applied, compare the exact current remote head and preserve unique production behavior, tests, public compatibility, research citations, doctoring, and provenance.

One-rule crates and one-operation PRs are not independent product boundaries by default. New work must map to the owning bounded context first.

### Current classifications established in this recovery slice

#### PR #356 — Analysis Run validation evidence

Current head: `df33bfa3e61ae4de3dbfae16df0deac12d2f4003`.

Classification: `landing_vehicle` **candidate only; scientifically blocked**.

The useful run-binding work should be preserved, but the current branch must not establish global scientific acceptance because:

- its RMSE acceptance uses a caller-selected `k × SE(RMSE)` threshold where the SE is derived from the same residual vector; this does not bound absolute recovery error and is not an evidence-derived scientific threshold;
- recovery truth/recovered vectors are caller-provided rather than bound to known-truth and estimator-owned Rust CPU `f64` artifacts by identity/digest;
- a boolean stating that input was not LLM-authored is not provenance;
- graph recovery, invariance, convergence, and active-backend CPU/GPU parity applicability are incomplete for a global claim-promotion artifact.

DDD correction: this branch may produce **Validation Evidence**. A separate **Scientific Claim Promotion Decision** aggregate governed by ADR 0014 decides promotability from a preregistered, method-specific complete evidence contract. `analysis_engine` does not own psychometric acceptance mathematics.

Exact-head Product workflow evidence is also not green: coverage-diagnostic jobs failed on the current head. A mergeable GitHub state therefore does not make #356 merge-ready.

#### PRs #352 and #355 — Driver/ctsem TIPREDEFFECT rewrite

Classification: both are `fold_into_landing_vehicle` candidates under Longitudinal Psychometrics; neither is superseded yet.

They share the same core scalar rewrite but differ in public naming, refusal guards, tests, doctoring, and documentation. The eventual landing vehicle must preserve the stronger domain guards and published-example/recovery evidence before either source PR is closed.

#### Analysis-run transport slices

Per-operation GET/POST/CLI/status/cancel/retry/export/history slices belong to the Analysis Run application context and `tepp_api` adapter. They remain traceable as stacked dependencies until a current-main landing vehicle preserves each unique contract test and consumer behavior.

#### Evidence/method refusal slices

Prompt/style/modality/copied-text/corpus-background/template-copy/location/membership/citation refusal helpers belong under Evidence & Semantic Measurement or Temporal Event Knowledge. A separate crate/PR is retained only when an independently versioned public reuse boundary exists.

## Directory and crate repair register

These protected-main paths are staged fold targets; current remote PR compatibility is preserved until the owning landing vehicle is ready:

| Current fragments | Owning bounded context |
| --- | --- |
| `system_clock`, `event_clock`, `assertion_clock`, `cutoff_clock`, `available_clock`, `document_clocks`, `revision_order` | Temporal Event Knowledge |
| `summarizes_edge`, `retrospective_edge`, `support_edge`, `citation_edge`, `outcome_order`, `subevent_containment`, `prediction_contradiction`, `relation_absence`, `role_contradiction` | Temporal Event Knowledge |
| `location_membership`, `episode_membership`, `membership_target` | Temporal Event Knowledge |
| `prompt_source`, `style_source`, `modality_source`, `copied_text`, `copy_identity`, `corpus_background`, `stopword_deletion`, `payload_bound`, `derived_sensitivity` | Evidence & Semantic Measurement |
| `psychometric_fit` | Longitudinal Psychometrics |

Do not perform one repository-wide rename while more than one hundred remote heads are active. The target DDD architecture is mandatory, but migration is staged through bounded-context landing vehicles to avoid destroying concurrent-agent intent.

For `analysis_engine`, new/replayed profiles should converge toward domain-owned modules rather than an indefinitely flat one-file-refusal directory:

```text
analysis_engine/src/
  runs/
  evidence_measurement/
  topic_measurement/
  psychometrics/
  event_intelligence/
  validation/
```

Scientific claim-promotion policy remains a distinct boundary rather than being hidden inside a validation-run transport helper.

## Scientific and data invariants

- Production mathematical, statistical, psychometric, vector, linear/matrix algebra, and token-size arithmetic is Rust-owned.
- CPU `f64` is the scientific numerical reference. Parallel reduction is bounded and deterministic for declared reproducibility modes.
- Event, assertion, document, system, availability, and knowledge-cutoff clocks remain distinct.
- `available_time <= knowledge_cutoff` is enforced for cutoff-safe evidence.
- Transition/process edges are forward-only; citation, support, summary, revision, and retrospective report do not become transitions.
- Cross-classified and weighted multiple membership is preserved; entity role, language, template, project, department, and location are not collapsed into one hierarchy.
- Topic proportions are compositional; raw simplex Pearson correlation is not a scientific network estimator.
- Synthetic data may establish known truth in tests; production output never substitutes synthetic evidence for customer/source evidence.
- LLMs may propose semantic units or interpretations only behind deterministic span/schema/security checks and never replace estimation/validation.
- Database objects use descriptive two-or-more-word `snake_case`, normalized authority tables, explicit temporal/provenance/tenant ownership, measured hot-partition evidence, and explicit UPSERT/idempotency contracts.
- External products integrate through versioned API/event/artifact contracts, never direct application-table access.
- Compliance mappings are readiness evidence, not certification.

## Merge gate

Before any landing vehicle merges:

1. Re-read exact head/base, live ruleset, review decision, unresolved threads, and current required workflow results.
2. Resolve valid review findings at the exact head; do not suppress deprecations, coverage gaps, or scientific failures.
3. Re-run the relevant Rust unit/integration/property/fuzz/known-truth tests and exact-head hosted gates.
4. Require 100% production statement coverage, production branch coverage, and public Rust documentation for the shipped scope.
5. Preserve unique evidence from every PR classified for folding or closure.
6. Update PRD/TRD/ADR/architecture/context map/UML/traceability/doctoring/CHANGELOG when the protected product contract changes.
7. Merge through normal policy. Force push and protection bypass are not queue-consolidation tools.

## Definition of product complete

TEPP is not complete until one supported release proves on the same protected source lineage that a documented user can authenticate, ingest real evidence, execute the approved scientific models, inspect uncertainty and provenance, export results, recover from restart/failure, and operate the service without repository-internal intervention.

The same release must prove real multilingual measurement evidence, true-parameter recovery, temporal/multiple-membership correctness, real hardware parity for claimed accelerators, interpretation abstention/citation behavior, accessible exact-value visual workflows, tenant/purpose/security controls, migrations, backup/restore, SLO/capacity evidence, signed artifacts/SBOM/provenance, support policy, current-head required workflows, independent review, and zero release-blocking PR/issue queue.

A valuation target is a prioritization bar, not evidence of market value. Product, scientific, operational, security, adoption, revenue, retention, and independent diligence evidence remain required.

## Refresh rule

Refresh this baseline whenever any of these materially changes:

- protected-main SHA or version;
- open PR/draft/issue counts;
- a priority PR head/base/review/check/merge state;
- queue classification or replacement mapping;
- bounded-context ownership or a canonical path/module boundary;
- issue acceptance criteria or implementation maturity;
- release sequence, deprecation, Figma file, standard, or primary research basis.

Live GitHub state always supersedes the snapshot. Historical snapshots stay in Git history rather than being carried forward as contradictory current facts.
