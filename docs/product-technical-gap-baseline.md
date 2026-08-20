# TEPP product and technical gap baseline

**Snapshot date:** 2026-08-21
**Authority:** approved PRD v0.4, ADRs, traceability ledger, validation ledger, and live GitHub state
**Purpose:** give the next developer a bounded, evidence-based queue of buyer-visible gaps and the acceptance proof required to close each one.

This baseline is a planning and evidence document. It does not promote an active pull request to protected-main implementation, and it does not claim CSAP, SOC 2, ISO, NIST, GPU, multilingual, psychometric, or production-service certification. Recheck the exact head, Checks, review, ruleset, and deployment evidence before promoting any row.

## Buyer promise and evidence rule

TEPP should let a research or risk team trace a measured change from an immutable source span through time, event, membership, latent measurement, uncertainty, and a reproducible export. A buyer can rely on that promise only when the numerical authority is deterministic Rust, availability cutoffs are enforced, multiple-membership structure is retained, LLM output remains a proposal, and exact-head scientific, security, and operational evidence is available.

Protected-main evidence is authoritative. An active PR is a delivery candidate only. A queued Check, predecessor-head result, bot comment, simulation that bypasses production code, or LLM agreement cannot close a gap.

## Gap register

| ID | Buyer-visible gap | Current evidence | Next bounded slice | Closure evidence and owner boundary |
|---|---|---|---|---|
| G-01 | A buyer can inspect contracts, but cannot yet run TEPP as a deployed modular service with `naruon` or `contextual-orchestrator`. | `tepp_api` DTO/export contracts and connector documents are on protected `main`; live HTTP service and deployed port remain open. | Ship one versioned HTTPS analysis-run/export port with health, authorization, timeout, provenance, and no cross-service table access. | Contract tests, live same-origin integration, failure/rollback proof, SBOM/provenance, and a consumer test in the owning service repositories. TEPP owns the contract; each service owns its deployment. |
| G-02 | The foundation validates recovery metrics but does not yet provide a production psychometric estimator vertical slice. | `validation_core`, truth simulation, temporal, relation, and membership foundations are on protected `main`; topic and psychometric estimator crates remain target work. | Implement one Rust estimator with a CPU `f64` reference, bounded multithreaded CPU path, explicit multilevel/multiple-membership inputs, time-varying state, and a realistic known-truth recovery test. | RMSE, bias, interval coverage, reproducibility, temporal leakage, and branch/line/docstring evidence at 100%; GPU parity is promoted only after real hardware or an explicit deployment-owned hardware gate. Owner: TEPP compute/measurement crates. |
| G-03 | Event users cannot yet move from separated mentions and hypothetical forecasts to a complete TDT/CHRONOS workflow. | Mention/instance separation and forward-transition protection are on protected `main`; occurrence forecast calibration is an active PR; detection, tracking, schema, and temporal-consistency stack remain incomplete. | Complete one event-intelligence vertical slice: evidence-bounded detection, schema proposal, calibrated occurrence forecast, and refusal to promote a forecast into an observed state. | Synthetic and realistic event fixtures, Brier calibration, temporal-order and contradiction tests, provenance spans, exact-head checks, and independent review. `event_core` owns semantics; LLMs only propose. |
| G-04 | Multilingual and semantic search value is specified but not measurable as shared-latent measurement invariance. | PRD and ADR 0004 define shared latent space and language-specific emissions; implementation remains a target. | Add paragraph/sentence/DOM/message semantic-unit contracts, shared-space alignment diagnostics, and a language-paired invariance recovery study without default TF-IDF/BM25 inferential weighting. | Cross-language parameter recovery, invariance thresholds, lexical/semantic drift separation, exact-span provenance, and adversarial segmentation tests. Owner: future semantic/topic boundary; do not hide this in `tepp_api`. |
| G-05 | Documents containing embedded base64 images cannot yet preserve image position, OCR/object/tag evidence, or searchable image identity. | The product requirement is recorded in the user brief and architecture direction; no protected-main image evidence schema or extraction contract is present. | Define a normalized image asset/region/observation contract that stores source position, digest, OCR, object/tag observations, model/version, confidence, and separate image-search references while preserving the original document. | 3NF schema review, hostile base64/size/depth tests, OCR/object provenance, position round-trip, deletion/retention authorization, and multimodal retrieval quality evidence. Owner: evidence/persistence boundary; provider adapters never receive identity mappings by default. |
| G-06 | A buyer sees architecture and contracts but not an accessible visual analysis product. | PRD requests bitemporal/network/drift/invariance views; `visual_analytics` is not implemented and no stable Figma contract exists. | First stabilize one user story and exact-value export contract; then create a Figma file and Storybook token/component inventory only for that approved interaction. | Accessibility, keyboard/interaction/i18n/edge tests, design-token tests, visual regression, and an ADR containing the real Figma File ID. Until then, no Figma ID is claimed and no UI dependency is added. |
| G-07 | Release operators do not yet receive a complete reproducible package/image evidence bundle. | CycloneDX SBOM, exact-head provenance, and checksums are generated in CI; full package/image SBOM, migration rollback, and deployment recovery bundle remain partial. | Produce one release candidate bundle with version consistency, signed/provenance-linked artifacts, migration/recovery rehearsal, and an operator-readable evidence index. | Exact protected-head CI/security evidence, SBOM, provenance, checksum, rollback/recovery logs, `CHANGELOG.md`, and no unsupported certification claim. Owner: release/operability boundary. |
| G-08 | Rust workflow changes can be reported as incomplete by the central Strix scope gate. | TEPP PR #154 exposed the issue; the root-cause repair is central `ContextualWisdomLab/.github#1173` at head `a6764368c634e9ea3a49d162a560d771d518e37e`. | Merge the central scanner repair under its own checks and review, then rerun PR #154 from its exact current head. | Central protected-main evidence, TEPP exact-head Strix success, and no TEPP-local workaround. The central repository owns scanner scope; TEPP owns Rust source correctness. |
| G-09 | The PR queue contains useful work but its evidence maturity is difficult for a buyer to audit. | The live queue includes active implementation, stale/dirty lineage, queued checks, and clean-but-unapproved candidates; current-head status changes continuously. | Keep this ledger synchronized after every merge, repair, close, or new gap; classify each PR by exact head, current checks, review, and merge state. | A machine-readable or deterministic inventory can reproduce the snapshot; stale reviews/checks are not reused; protected merge is never bypassed. Owner: repository governance. |

## Live PR delivery ledger

The following is the bounded queue snapshot used for the next loop. It is not a promise that these PRs are mergeable.

| PR | Current purpose | Snapshot evidence | Next action |
|---:|---|---|---|
| #50 | TDT/CHRONOS state-transition admission | head `1b12210af301098a18ad0cf729b947f20db02a2e`; open, blocked by stale review state and queued current-head Checks, no qualifying independent approval | Recheck reviews and exact-head Checks; merge only after policy gates pass |
| #85 | CHRONOS occurrence-prediction Brier calibration | head `af3060afdf3a7df2231dd04bfdd59253c83a0637`; refreshed from protected `main`; current Checks queued, no qualifying independent approval | Let exact-head Checks and review run; do not promote forecast calibration before protected merge |
| #84 | Measurement-invariance loading validation | head `e0ef1f869c46aa20969ce083f9ca31d622c31bb4`; dirty/queued current head, no qualifying independent approval | Resolve current-head merge state and rerun focused/full evidence |
| #154 | Rust toolchain/coverage baseline | head `aaadb7f`; required Strix run queued after root-cause diagnosis | Follow central #1173, then rerun exact-head evidence |
| #165 | Hourly contextual-orchestrator product-development scheduler | head `37f85fed18d78e992e5ec2de3ffcc44eb98bbca4`; current coverage evidence queued, no qualifying independent approval | Verify scheduler secrets/model pin/permissions and wait for exact-head Checks while working another bounded queue item |
| #159, #158, #155 | Clean candidates with focused repairs | heads `855c6c7153c2f66a1c14e842ad700f571592dd35`, `6d28d23c432288c4dbecbd74a25f093ed9d9ef61`, and `0e2910825a042d7fdeb6497a20975b538493c65c`; required Checks were green at snapshot, but no qualifying independent approval | Re-fetch live review/ruleset state; merge only when the protected policy supplies approval |

Closed or stale predecessor PRs are not used as evidence. For example, a closed event predecessor must be represented by its open successor and reacquire all current-head gates.

## Loop contract

For each open PR, the next developer performs these actions in order:

1. Fetch the current head, base, merge state, reviews, review threads, required Checks, ruleset, and changed-file responsibility boundary.
2. Reproduce every actionable review or failed Check with the smallest relevant test.
3. Apply the root-cause fix in the owning repository; preserve concurrent remote commits and never force-push.
4. Run focused and complete verification, including scientific, security, documentation, coverage, and operational evidence affected by the change.
5. Update the owning ADR, traceability, validation ledger, research citation, API/architecture document, and changelog when the contract changed.
6. Re-fetch exact-head evidence and merge only with required Checks and a qualifying independent approval. Never use admin, self-approval, or bypass.
7. Rebuild the queue. If no PR or issue remains, select the highest-impact bounded gap above, open a PR, and repeat.

Checks and review latency pause one branch only. The loop continues with another safe branch or with a bounded buyer gap.

## Release gate

No TEPP release is claimed until the selected capability is implemented on protected `main`, exact-head Checks and security evidence are green, scientific recovery and edge cases pass, public documentation and APA 7 research references are current, SBOM/provenance and rollback evidence exist, and the buyer can exercise the documented standalone and modular path.

## Evidence sources

- Product and measurement target: [`docs/product/prd-v0.4-approved.md`](prd-v0.4-approved.md)
- Technical and security contracts: [`DOCUMENTATION.md`](../../DOCUMENTATION.md), [`ARCHITECTURE.md`](../../ARCHITECTURE.md), [`docs/TRACEABILITY.md`](../TRACEABILITY.md)
- Scientific/release promotion: [`docs/validation/temporal-event-foundation.md`](../validation/temporal-event-foundation.md), [`docs/TEST_STRATEGY.md`](../TEST_STRATEGY.md), [`docs/research/standards-and-literature.md`](../research/standards-and-literature.md)
- Live repository evidence: GitHub PR/Checks/review/ruleset APIs fetched on 2026-08-21; re-fetch before acting.
