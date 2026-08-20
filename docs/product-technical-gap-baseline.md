# Product and Technical Gap Baseline

**Status:** Live delivery baseline
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-08-20 20:15 KST
**Protected-main evidence:** `7c29e7c971d7940e1fb3def1ed3aae2d1bc8ad4a`

## Purpose

This document turns the approved product, technical, scientific, security, and
operability documents into an executable buyer-gap register. It records what a
buyer can use on protected `main`, what is being delivered on open pull
requests, and the evidence required before a capability can be described as
shipped. Re-read the live pull request and check links before making a release
or customer commitment; the table below is a time-stamped snapshot, not a
replacement for current-head GitHub evidence.

The immediate buyer action is to use the priority order and acceptance gates in
this document to choose which integration or measurement outcome to validate
next. A capability marked `active-PR` is not available from protected `main`.

## Authority and derivation

The gap register is derived from these canonical sources:

| Concern | Authority | How it constrains the gap register |
|---|---|---|
| Product outcomes and user decisions | [`docs/product/prd-v0.4-approved.md`](product/prd-v0.4-approved.md) | Defines temporal evidence, relational measurement, scientific users, visual outputs, and release outcomes. |
| Technical and runtime contracts | [`docs/TRD.md`](TRD.md) | Separates protected-main implementation from accepted-target architecture and requires Rust, temporal eligibility, modular APIs, and realistic validation. |
| Runtime and service boundaries | [`docs/UML.md`](UML.md), [`ARCHITECTURE.md`](../ARCHITECTURE.md), [`docs/API_CONTRACT.md`](API_CONTRACT.md) | Defines standalone crates, versioned interchange, temporal flows, and no cross-service application-table coupling. |
| Data authority and persistence | [`docs/ERD.md`](ERD.md), [`docs/TRACEABILITY.md`](TRACEABILITY.md), [`docs/adr/0013-bitemporal-persistence-reproducibility-and-split-authority.md`](adr/0013-bitemporal-persistence-reproducibility-and-split-authority.md) | Requires immutable provenance, six-clock eligibility, relation-aware splits, third-normal-form persistence, and reproducible artifacts. |
| Scientific and release promotion | [`docs/validation/temporal-event-foundation.md`](validation/temporal-event-foundation.md), [`docs/adr/0014-scientific-claim-promotion-and-release-evidence.md`](adr/0014-scientific-claim-promotion-and-release-evidence.md) | Prevents a design document, simulation, queued check, or LLM judgment from becoming a product claim. |
| Automation and merge authority | [`docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md`](operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md), [`docs/adr/0015-autonomous-development-review-and-merge-authority.md`](adr/0015-autonomous-development-review-and-merge-authority.md) | Keeps proposal, verification, publication, independent review, and merge authority separate. |
| Scientific, privacy, and assurance constraints | [`docs/TEST_STRATEGY.md`](TEST_STRATEGY.md), [`docs/PRIVACY_DATA_GOVERNANCE.md`](PRIVACY_DATA_GOVERNANCE.md), [`docs/COMPLIANCE_READINESS.md`](COMPLIANCE_READINESS.md) | Requires recovery/parity tests, purpose-bound identity handling, and CSAP/SOC 2 readiness evidence without false certification claims. |
| Live delivery state | [TEPP open pull requests](https://github.com/ContextualWisdomLab/TEPP/pulls?q=is%3Apr+is%3Aopen), [TEPP open issues](https://github.com/ContextualWisdomLab/TEPP/issues?q=is%3Aissue+is%3Aopen) | Supplies current-head SHA, base branch, review, check, dependency, and issue evidence. |

## Buyer-gap register

| ID | Buyer-visible gap | Current maturity | Delivery authority | Closure evidence |
|---|---|---|---|---|
| GAP-001 | A consumer can submit an analysis run and receive a durable receipt, but protected `main` does not yet publish the separate deterministic completed-result contract needed to distinguish `accepted`, `running`, success, and typed terminal failure. | `active-PR` | PR #157; linked issue #156 | Current-head checks and qualifying review; request/result binding, cutoff/snapshot/model/output-profile identity, canonical digest, bounded summary, deterministic retrieval, and fail-closed mismatch tests. |
| GAP-002 | LineageWeave and other modular consumers cannot yet rely on a protected-main live HTTP consumer boundary for analysis-run submission, temporal evidence context, and cutoff-safe history projection. | `active-PR` | Stacked PRs #107 → #155 → #158/#159 | Merge the stack in base order after current-head review/check evidence; preserve loopback/credential/cutoff/idempotency refusal and live TCP tests. |
| GAP-003 | The platform foundation has strong temporal, evidence, membership, privacy, event, topic-coordinate, and compute contracts, but a buyer cannot yet run the complete multilingual temporal psychometrics product from protected `main`. | `partial` / `accepted-target` by capability | PRs #48, #50, #51, #91, #100, #106, #110, plus later estimator/service work | Each capability must independently pass realistic scientific recovery, numerical parity, security, documentation, and exact-head merge gates; no aggregate claim is allowed from planning documents. |
| GAP-004 | Research and operational evidence is discoverable, but there was no single live register mapping buyer outcomes to current PR/issue state, protected-main maturity, and closure evidence. | `active-PR` in this change | This document, linked from the documentation map | The baseline is required by the documentation validator and refreshed whenever the protected head, PR queue, issue queue, or capability maturity changes materially. |
| GAP-005 | Coordinated visual analytics, accessible exact-value exports, design tokens, Storybook inventory, and Figma interaction contracts are not yet needed for the current headless Rust/API slice, so a buyer cannot yet validate a visual analytics workflow. | `accepted-target` | PRD phase 7 and the visual-analytics roadmap phase | Start a separate UI/product-design PR only after the stable data/API contract exists; that PR's ADR must record the real Figma File ID and Storybook inventory before implementation. |

## Current pull-request queue

The following table was read from GitHub at the snapshot time. Check evidence is
reported qualitatively so a queued run cannot be mistaken for a completed pass.
An empty review decision means that no qualifying decision was reported by the
API at snapshot time. These values do not authorize a merge by themselves.

| PR | Head | Base | Merge state | Review decision | Current check evidence | Delivery role |
|---:|---|---|---|---|---|---|
| [164](https://github.com/ContextualWisdomLab/TEPP/pull/164) | `83eb9bb` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | required Checks queued; no completed failure observed | Product/technical gap baseline |
| [159](https://github.com/ContextualWisdomLab/TEPP/pull/159) | `586cda5` | `feat/lineageweave-live-consumer-contract` | `MERGEABLE/UNSTABLE` | required | required Checks queued; no completed failure observed | LineageWeave cutoff-safe history projection |
| [158](https://github.com/ContextualWisdomLab/TEPP/pull/158) | `99c8d4c` | `feat/lineageweave-live-consumer-contract` | `MERGEABLE/CLEAN` | required | current Checks green; independent review still required | LineageWeave temporal evidence context |
| [157](https://github.com/ContextualWisdomLab/TEPP/pull/157) | `f1c94f7` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | required Checks queued; no completed failure observed | Completed analysis-run result contract |
| [155](https://github.com/ContextualWisdomLab/TEPP/pull/155) | `0e29108` | `cursor/bc-422aba2a-86ab-45e3-9911-95cff5c28a87-5627` | `MERGEABLE/CLEAN` | required | current Checks green; independent review still required | Modular LineageWeave consumer parent |
| [144](https://github.com/ContextualWisdomLab/TEPP/pull/144) | `980b62b` | `agent/psychometric-posterior-esem-input` | `MERGEABLE/UNSTABLE` | draft; review required after ready-for-review | required Checks queued; scientific tests pass locally | Multilevel event-time psychometric recovery |
| [116](https://github.com/ContextualWisdomLab/TEPP/pull/116) | `5f9d7bf` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | current Checks green; independent review still required | APA 7 method-paper doctoring |
| [115](https://github.com/ContextualWisdomLab/TEPP/pull/115) | `2106170` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | required Checks queued; no completed failure observed | Coverage-authority quality validator |
| [110](https://github.com/ContextualWisdomLab/TEPP/pull/110) | `c63e1cf` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | current Checks green; independent review still required | Corpus-split leakage-audit manifest |
| [107](https://github.com/ContextualWisdomLab/TEPP/pull/107) | `542fa0a` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | current Checks green; independent review still required | Loopback Naruon analysis-run service |
| [106](https://github.com/ContextualWisdomLab/TEPP/pull/106) | `d01892e` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | current Checks green; independent review still required | Privacy audit-event inspection |
| [100](https://github.com/ContextualWisdomLab/TEPP/pull/100) | `3e928bc` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | current Checks green; independent review still required | Live service-TLS bind policy |
| [91](https://github.com/ContextualWisdomLab/TEPP/pull/91) | `53a5a31` | `main` | `MERGEABLE/BLOCKED` | `CHANGES_REQUESTED` | no completed failure; review changes remain | Derived-artifact sensitivity inheritance |
| [62](https://github.com/ContextualWisdomLab/TEPP/pull/62) | `1951ab4` | `main` | `MERGEABLE/BLOCKED` | required | required Checks queued; no completed failure observed | Simulation cutoff eligibility |
| [61](https://github.com/ContextualWisdomLab/TEPP/pull/61) | `8ebbc04` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | required Checks queued; no completed failure observed | Interval-aware uncertain availability cutoff |
| [58](https://github.com/ContextualWisdomLab/TEPP/pull/58) | `e98ada4` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | required Checks queued; no completed failure observed | Positional embedded image source units |
| [59](https://github.com/ContextualWisdomLab/TEPP/pull/59) | `43bf6cf` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | required Checks queued; no completed failure observed | Unicode NFC/NFD canonical identity for split leakage |
| [57](https://github.com/ContextualWisdomLab/TEPP/pull/57) | `14014b7` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | required Checks queued; no completed failure observed | Exact-head scientific claim promotion gates |
| [51](https://github.com/ContextualWisdomLab/TEPP/pull/51) | `0111a1a` | `main` | `MERGEABLE/BLOCKED` | `CHANGES_REQUESTED` | current Checks green; review changes remain | VRAM budget and CPU fallback types |
| [50](https://github.com/ContextualWisdomLab/TEPP/pull/50) | `2a29e24` | `main` | `MERGEABLE/BLOCKED` | `CHANGES_REQUESTED` | required Checks queued after branch-coverage fix; no completed failure observed on this head | TDT/CHRONOS transition boundary |
| [49](https://github.com/ContextualWisdomLab/TEPP/pull/49) | `199ed6c` | `main` | `MERGEABLE/BLOCKED` | `REVIEW_REQUIRED` | required Checks queued; no completed failure observed | Posterior ESEM input gates and true-parameter RMSE |
| [48](https://github.com/ContextualWisdomLab/TEPP/pull/48) | `8e88b33` | `main` | `MERGEABLE/BLOCKED` | `CHANGES_REQUESTED` | current Checks green; review changes remain | Logistic-normal topic coordinates |

### Merge and review order

1. Process independent current-main PRs by exact-head review and required-check
   evidence; no stale `CHANGES_REQUESTED` or queued check is sufficient.
2. Process the LineageWeave stack in dependency order: **#107 → #155 →
   (#158 and #159)**. Re-fetch each remote head immediately before any commit,
   push, or merge and preserve remote-agent commits.
3. Process #157 as the completion of issue #156. Keep the issue open until the
   PR is actually merged and the protected-main result contract is verified.
4. After each merge or closure, enumerate the queue again. A clean merge state
   without a qualifying independent review is not merge-ready.

## Issue queue and hygiene

At the snapshot, issue #156 is the only open issue with product acceptance
criteria. It remains open because PR #157 is not merged. Issues #161 and #162
were closed as queue-hygiene placeholders: they contained no buyer outcome,
acceptance criteria, or evidence requirement. Reopen a placeholder only with a
concrete customer action and verifiable acceptance test.

## Product and technical delivery specification

The next buyer-facing delivery slice is the completed-result interchange:

- Keep the accepted receipt and completed result as separate DTO families.
- Bind result identity to tenant/workspace, accepted run, immutable snapshot,
  knowledge cutoff, model contract, output profile, idempotency key, and result
  digest.
- Permit only terminal success or typed terminal failure to carry completion
  metadata; `accepted` and `running` remain non-measurement states.
- Reject unknown fields, unsupported versions, malformed timestamps/digests,
  oversized payloads, cross-tenant/snapshot/cutoff/model/profile mismatches,
  source text, credentials, and direct identity.
- Keep deterministic/statistical scientific authority outside the LLM; an LLM
  interpretation can annotate evidence but cannot promote a result to truth.

This specification is already owned by issue #156 and PR #157. Do not create a
duplicate implementation while that PR is in the review/check loop.

## Architecture, data, and assurance constraints

- Rust owns production mathematical and psychometric arithmetic; every future
  estimator needs a CPU `f64` reference, bounded parallel execution, GPU parity,
  and safe VRAM-to-CPU fallback evidence.
- Temporal analyses must preserve event, assertion, document, system,
  availability, and knowledge-cutoff clocks and must reject future-available
  evidence.
- Multiple-membership and cross-classified structures remain explicit so a
  document is not treated as an independent atom when it belongs to several
  authors, teams, projects, episodes, or roles.
- Database objects use descriptive two-or-more-word `snake_case` names and
  normalized ownership/temporal/provenance tables; hot-partition mitigation
  must be designed with measured access patterns before a physical migration.
- Purpose-bound authorization, opaque analytical identifiers, protected identity
  mappings, encryption, retention/deletion, selective disclosure, and audit
  preserve PII utility without blanket masking.
- CSAP and SOC 2 are readiness targets only. Certification or attestation needs
  independent evidence outside this repository.
- The present slice has no frontend interaction contract. Figma and Storybook
  are therefore deferred; when visual analytics begins, create the Figma file,
  record its actual File ID in the owning ADR, and test the shared design tokens
  and accessible interaction states.

## Refresh rule

Refresh this file when a PR head/base changes materially, a review or required
check changes, an issue opens/closes, a capability crosses a maturity boundary,
or a buyer-visible gap is added/removed. Use the current protected-main SHA and
current PR head SHA in every claim. Keep the document small: update the table
and evidence links rather than creating a second planning register.

## References

The authoritative APA 7 research register is [`docs/research/standards-and-literature.md`](research/standards-and-literature.md).
Method-specific sources remain in the linked doctoring documents and ADRs; this
baseline records delivery implications and does not replace those sources.
