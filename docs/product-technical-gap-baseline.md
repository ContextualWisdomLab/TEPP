# TEPP Product–Technical Gap Baseline

**Snapshot:** 2026-08-20  
**Evidence baseline:** protected-main candidate `7c29e7c` (`origin/main`)  
**Purpose:** turn the approved product intent, ADR decisions, research register, implementation ledger, and live PR queue into one executable feature specification. This document is a planning and claim-discipline artifact; it does not promote an active PR to protected-main capability.

## 1. How to read this baseline

The product contract is the approved PRD. Technical and scientific constraints are owned by the TRD, Architecture, ERD, UML, Test Strategy, Operability guide, privacy/security documents, and the owning ADR. Research notes in `docs/research/` are the doctoring register and use APA 7th references. The live GitHub ruleset, current PR head, exact-head Checks, qualifying review, and protected merge are the only evidence that can promote a capability to `implemented-main`.

The maturity words below are deliberate:

- `implemented-main`: integrated on protected `main` with current required verification and qualifying review;
- `active-PR`: implemented only on an open PR;
- `partial`: a bounded foundation exists, while the customer capability is incomplete;
- `accepted-target`: approved architecture or product target without protected-main implementation;
- `deployment-owned`: requires deployed infrastructure, operating history, or independent assurance.

## 2. Buyer workflow and derived feature specification

The buyer should be able to move through this sequence without losing provenance or scientific meaning:

| Feature slice | Customer action | Required contract | Acceptance evidence | Current state |
|---|---|---|---|---|
| Evidence intake | Submit a document or artifact and see exactly what was admitted | immutable source bytes, digest, exact spans, bounded layout, opaque analytical identity, purpose-bound authorization | wire round-trip, hostile payload refusal, digest mismatch refusal, audit record | `implemented-main` foundation |
| Temporal evidence | Ask what was knowable at a historical cutoff | event, assertion, document, system, availability, and cutoff clocks; uncertain intervals; no future-available evidence | interval/partial-order tests and cutoff-safe replay | `implemented-main` primitives; persistence enforcement is `partial` |
| Relational context | Preserve authorship, teams, projects, episodes, and overlapping roles | typed, time-varying, cross-classified multiple membership; no atomistic collapse | known-truth membership recovery and weighted ESS tests | `partial` |
| Event analysis | Separate observed mentions, event instances, relations, tracking, and predictions | forward transition DAG; provenance may point backward but cannot become a reverse transition | graph recovery, temporal ordering, contradiction, TDT/CHRONOS calibration | ontology foundation `partial`; intelligence stack `accepted-target` |
| Measurement | Estimate topics and latent traits with uncertainty | shared multilingual latent space, method effects, multilevel/cross-classified/multiple-membership and longitudinal estimands | true-parameter bias/RMSE, interval coverage, invariance, convergence, irregular-time recovery | estimator products `accepted-target` |
| Compute | Run the same scientific job safely on CPU or accelerator | Rust CPU `f64` reference, bounded fixed-pool CPU, streamed VRAM budget, real GPU parity, OOM fallback | objective/parameter parity, memory profile, device-loss/OOM recovery, deterministic artifacts | `accepted-target` |
| Interpretation | Request evidence-bounded narrative assistance | contextual-orchestrator boundary, auto-discovered approved providers, access list, recursion/decomposition/role effort, deterministic authority outside LLM | live provider test, comparable-budget ablation, refusal on untrusted instructions | router `partial`; live execution `accepted-target` |
| Integration | Consume versioned results from `naruon` or another CWL service | standalone library and versioned API/artifact contracts; no cross-service table reads | contract fixtures, compatibility tests, live HTTPS service checks | contracts `partial`; live HTTP `accepted-target` |
| Governance | Retain, disclose, delete, and investigate without destroying valid longitudinal signal | opaque IDs, protected identity map, purpose-bound disclosure, retention, legal hold, deletion, audit, selective disclosure | tenant/RLS, grant expiry, hold refusal, tombstone/recovery and access audit proofs | privacy foundation `implemented-main`; physical retention lifecycle `active-PR` |
| Operations | Release and operate a reproducible product | exact-head checks, SBOM/provenance, migration rollback, backup/restore, SLOs, bounded degraded modes | protected release gate and recovery drill | repository readiness `partial`; deployment assurance `deployment-owned` |

## 3. Current implementation and missing customer value

### Implemented foundation on protected main

- Rust workspace, pinned quality tooling, public rustdoc checks, release evidence generation, and repository security contracts.
- Immutable evidence records with UUIDv7 identity, canonical SHA-256 content digests, UTF-8/exact-span validation, bounded page coordinates, and strict wire reconstruction.
- Typed six-clock values, uncertain intervals, bounded Allen reasoning, forward-only transition graph, cutoff-safe corpus splitting, truth-corpus generation, and recovery metrics.
- PostgreSQL contract foundation: tenant RLS, append-only manifests, temporal window checks, typed membership SQL, event/source/audit SQL, concurrent document-write proof, and live SQL gates where the ledger marks them implemented.
- Purpose-bound provider payload minimization and opaque analytical identifiers. The product does not blanket-mask PII when that would destroy authorship, temporal, role, or longitudinal measurement; re-identification is separately authorized and audited.
- Versioned standalone/API connector contracts for `naruon` and `contextual-orchestrator`; these are contracts, not claims of deployed HTTP services.

### Active PR value and exact evidence still required

The current non-draft queue at this snapshot is:

| PR | Base → head | Exact head | Review state | Merge state | Next loop |
|---:|---|---|---|---|---|
| #159 | `feat/lineageweave-live-consumer-contract` → `feat/lineageweave-project-history-projection` | `5295f5e` | no qualifying decision | `UNSTABLE` | recheck required Checks, then independent review |
| #158 | `feat/lineageweave-live-consumer-contract` → `feat/lineageweave-temporal-context-contract` | `99c8d4c` | no qualifying decision | `UNSTABLE` | recheck required Checks, then independent review |
| #157 | `main` → `feat/completed-analysis-result-contract-v1` | `30918e5` | review required | `BLOCKED` | exact-head Checks and qualifying approval |
| #155 | `cursor/bc-422aba2a-86ab-45e3-9911-95cff5c28a87-5627` → `feat/lineageweave-live-consumer-contract` | `0e29108` | no qualifying decision | `CLEAN` | verify exact-head required review before merge |
| #116 | `main` → `cursor/adr-method-references-1e2d` | `5f9d7bf` | review required | `BLOCKED` | review/Checks and source-link validation |
| #115 | `main` → `cursor/bc-2f412b9a-13bf-46ec-8a1c-690b987d943c-29fa` | `2525b8e` | no qualifying decision | `BLOCKED` | exact-head quality Checks and review |
| #110 | `main` → `cursor/bc-1385ba5f-a881-48aa-accb-830007840892-6509` | `c63e1cf` | review required | `BLOCKED` | exact-head leakage-audit Checks and review |
| #107 | `main` → `cursor/bc-422aba2a-86ab-45e3-9911-95cff5c28a87-5627` | `a6cea38` | review required | `BLOCKED` | live HTTP Checks and review |
| #106 | `main` → `cursor/bc-30f8155d-8e86-4195-9ed2-26e399488fbc-e48d` | `d01892e` | review required | `BLOCKED` | privacy/audit Checks and review |
| #100 | `main` → `cursor/bc-5f161840-8f3e-409e-bf2b-95a6ffa3a68f-fe36` | `3e928bc` | review required | `BLOCKED` | live TLS policy Checks and review |
| #91 | `main` → `agent/derived-sensitivity-inheritance` | `53a5a31` | changes requested | `BLOCKED` | resolve current-head review findings and rerun Checks |
| #51 | `main` → `agent/compute-backend-vram-budget` | `0111a1a` | changes requested | `BLOCKED` | resolve review findings; do not claim GPU implementation before real parity |
| #50 | `main` → `agent/event-intelligence-status-gates` | `af67f14` | changes requested | `BLOCKED` | resolve review findings and rerun event-gate tests |
| #48 | `main` → `agent/topic-logratio-coordinates` | `8e88b33` | changes requested | `BLOCKED` | resolve review findings and rerun true-parameter tests |

Draft PRs remain candidates, not merge-ready work. Their branch state must be revalidated against current `main` before promotion. No stale approval, predecessor head, queued check, or local green run promotes a PR.

## 4. Ranked product–technical gaps

| Rank | Gap / buyer-visible consequence | Smallest valuable slice | Proof required |
|---:|---|---|---|
| 1 | A buyer cannot yet complete the full ingest → analysis-run → versioned result flow through a deployed service | merge the current analysis-run consumer/result/history contracts, then add one live HTTPS loopback service backed only by versioned DTOs | exact-head protected checks, contract compatibility, real request/response, timeout and error envelope tests |
| 2 | No protected-main production psychometric estimator yet; simulations cannot be used as product evidence | implement one Rust CPU `f64` multilevel/multiple-membership longitudinal estimator with deterministic small truth corpus | parameter bias/RMSE, interval coverage, within/between recovery, irregular-time ordering, reproducibility |
| 3 | GPU/VRAM architecture is accepted but no real accelerator lane is proven | add one backend-neutral operation with CPU reference, bounded batch controller, real CUDA/WGPU execution when available, and CPU fallback | parity, VRAM profiles, OOM/device loss recovery, artifact metadata; skipped GPU is not green evidence |
| 4 | Physical persistence is not yet a production lineage store, and hot-partition behavior is unproven | validate 3NF tables and introduce a measured tenant/time partition strategy only for a demonstrated hot append-only workload; keep immutable identities and relation-aware split boundaries | catalog/DDL test, no duplicated facts, query plan/load evidence, migration rollback, tenant isolation, backup/restore |
| 5 | Semantic retrieval and embedded-image evidence are designed but not implemented as a buyer workflow | add semantic-unit records for paragraph/syntax/DOM/sender-receiver boundaries plus positional image metadata, OCR/object/tag outputs and separate image-search references | chunk boundary tests, source-position round trip, base64 size/type validation, image/text retrieval precision, provenance |
| 6 | LLM orchestration has a governed selector but no live provider execution or all-key auto-discovery proof | implement the provider-neutral contextual-orchestrator adapter with allowlisted secrets, auto-discovery, role/access manifests, and deterministic abstention | live NIM/OpenCode test, no `COPILOT_GITHUB_TOKEN`, comparable-budget Fugu/Conductor/TRINITY ablation, audit record |
| 7 | Multilingual measurement and longitudinal ESEM/DSEM remain target architecture | deliver one two-language profile with shared latent coordinates and time-varying invariance status | configural/metric/scalar or partial invariance, shared-space alignment error, temporal recovery |
| 8 | Buyer-facing visual analytics and accessible interaction are not in this backend repository | keep UI in a separately owned consumer repository until a stable interaction contract exists; then use Figma and Storybook there | accessibility, keyboard/touch/screen-reader, design-token and interaction tests; record the Figma file ID in that UI ADR |
| 9 | CSAP/SOC 2/ISO/NIST readiness is evidence-oriented but not an assurance claim | close repository evidence gaps, then collect deployment/operating-history evidence with the responsible assessor | control mapping, incident/recovery drills, SBOM/provenance, independent assessment; never self-claim certification |

## 5. Boundaries that are intentional, not gaps

- Smart crawling is outside this repository request and is not being added.
- No direct application-table access is allowed between TEPP, `naruon`, `contextual-orchestrator`, or `.github`; integration uses versioned API/artifact contracts.
- PII is not indiscriminately masked. Purpose-bound grants, opaque IDs, encrypted/separately protected identity mapping, selective disclosure, retention/deletion, and auditable privileged access preserve valid measurement while controlling disclosure.
- There is no frontend in this repository. Figma, Storybook, and design-token implementation are deferred to the UI consumer boundary; no Figma file ID is fabricated in a backend ADR.
- Synthetic data is for controlled truth and unit/acceptance tests only; production ingestion must require authorized real evidence and provenance.

## 6. Execution loop and release gate

The next safe action is selected by this order: inspect the exact current PR head and unresolved review threads; reproduce actionable defects; apply the smallest owning-layer fix; rerun focused and complete checks; update affected doctoring/ADR/PRD/technical docs; wait for qualifying independent approval without treating queue time as a blocker; merge only through the live ruleset; then rebuild the queue. Stacked PRs are evaluated base-first, and a remote agent’s new commit is fetched and respected rather than force-pushed over.

When no mergeable PR or issue remains, select the highest-ranked bounded gap above, open one focused PR, and return to the same loop. A release is allowed only after clean protected integration, exact-head required Checks, qualifying review, scientific recovery evidence, migration/rollback and recovery evidence, SBOM/provenance, version consistency, updated `CHANGELOG.md`, and no unresolved scientific, privacy, security, or supply-chain blocker.

## 7. Research and standards authority

The active authority register is [`docs/research/standards-and-literature.md`](research/standards-and-literature.md). Relevant APA 7 entries include the *Standards for Educational and Psychological Testing* (American Educational Research Association et al., 2014), dynamic structural equation models (Asparouhov et al., 2018), multiple-membership multilevel modeling (Browne et al., 2001), dynamic topic models (Blei & Lafferty, 2006), SemAF-Time/ISO-TimeML (International Organization for Standardization, 2012), W3C OWL-Time (Hobbs & Pan, 2017), NIST AI RMF (Tabassi, 2023), ISO/IEC 42001:2023, ISO/IEC 23894:2023, AICPA Trust Services Criteria (American Institute of Certified Public Accountants, 2023), and the Fugu, Conductor, and TRINITY research notes recorded in `docs/research/adaptive-orchestration-router.md`. New implementation claims must add a primary-source note before promotion.

## 8. Traceability actions

The next implementation PR should update this baseline’s relevant row, the owning traceability entry, the validation ledger, and `CHANGELOG.md` in the same reviewed change. If the change alters an estimand, clock meaning, event relation, membership meaning, backend authority, privacy authority, service ownership, or release claim, update the owning ADR and PRD version according to `docs/adr/ADR_POLICY.md` before calling the slice complete.
