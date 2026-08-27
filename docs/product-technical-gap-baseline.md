# Product and Technical Gap Baseline

## 2026-08-27 Driver p.16 standardised-map recovery queue

- Protected `main` `c7cf34b84d087904bdcb4604479dda2ed8cfcf77` carries analysis-run status HTTP (#266) and prior Driver p.16 standardised families through `T0VARstd` / `T0MEANSstd` / `asymCINTstd` / `discreteCINTstd`.
- Open independent psychometric recovery slices on this base (do not merge without independent non-author APPROVE and exact-head required Checks):
  - #267 `asymDIFFUSIONstd` head `a7c805af22ca9a1b908ae8c699e9ee920b552541`
  - #268 `TRAITVARstd`
  - #270 `MANIFESTTRAITVARstd`
  - #271 `MANIFESTVARstd`
  - #272 `TIPREDVARstd` head `3234f5034ee6677aa324eddfbde1adbb431dad57`
- Draft #269 composes TDT/CHRONOS as one versioned workflow (GAP-007); independent of the psychometric maps.
- Org collaborator set is single-author; ruleset 18156473 still requires two independent approvals. Author will not self-approve.

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

**Status:** Live delivery baseline
**Product:** Temporal Event Psychometrics Platform (TEPP)
**Snapshot:** 2026-08-27T02:05:00Z
**Protected-main evidence:** `c7cf34b84d087904bdcb4604479dda2ed8cfcf77` (merge of [PR #266](https://github.com/ContextualWisdomLab/TEPP/pull/266) analysis-run status HTTP)
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
| Protected-main SHA | `c7cf34b84d087904bdcb4604479dda2ed8cfcf77` (2026-08-26T12:40Z, merge of [#266](https://github.com/ContextualWisdomLab/TEPP/pull/266)) | All as-built claims are bounded to this commit. |
| Workspace members | 57 unique Rust crates | The repository is modular, but the approved target still lacks complete semantic, compute, psychometric-engine, event-intelligence, interpretation, artifact, and visual product boundaries. |
| Workspace version | `0.2.0` | A version number alone does not establish a supported product release; no signed artifact or support policy exists yet. |
| Open pull requests | **6** | Active queue: #267 asymDIFFUSIONstd, #268 TRAITVARstd, #269 (draft) TDT/CHRONOS composition, #270 MANIFESTTRAITVARstd, #271 MANIFESTVARstd, #272 TIPREDVARstd. |
| Draft pull requests | **1** | #269 event composition only. |
| Open product issues | **9** | Issues #166–#167 and #169–#174 plus #176 remain open. Result-contract issue #156, semantic-units issue #168, and queue-consolidation issue #175 are all CLOSED. |
| Current package version | `0.2.0` | No supported product release is established by the repository version alone. |

The pull-request counts come from the live GitHub search at this snapshot. The
full exact-head classification lives in this register; re-read live GitHub
state immediately before every mutation. Passing or queued Checks on an open PR never
promote that PR to implemented-main.

## Snapshot open pull-request evidence

The following exact-head register was fetched live from GitHub at
2026-08-27T02:05:00Z against protected main `c7cf34b`. Review decisions,
required Checks, and mergeability remain volatile; the live GitHub API
supersedes this snapshot. `draft=false` is not approval, mergeability, or a
passing-check claim. Re-read the full SHA, current review decision, required
Checks, and branch rules immediately before every mutation.

| PR | Exact current head | Draft | Base | Title |
|---:|---|:---:|---|---|
| #267 | `a7c805af22ca9a1b908ae8c699e9ee920b552541` | false | main | feat(psychometric): restore Driver p.16 asymDIFFUSIONstd p/p=1 on main |
| #268 | (live) | false | main | feat(psychometric): restore Driver p.16 TRAITVARstd trait/trait=1 on main |
| #269 | (live) | true | main | feat(event): compose TDT and CHRONOS as one versioned workflow |
| #270 | (live) | false | main | feat(psychometric): restore Driver p.16 MANIFESTTRAITVARstd ψ/ψ=1 on main |
| #271 | (live) | false | main | feat(psychometric): restore Driver p.16 MANIFESTVARstd θ/θ=1 on main |
| #272 | `3234f5034ee6677aa324eddfbde1adbb431dad57` | false | main | feat(psychometric): restore Driver p.16 TIPREDVARstd v/v=1 on main |

Review decisions, required Checks, and mergeability remain volatile; re-read
them immediately before every mutation. This snapshot is not merge authorization
and does not treat queued or passing Checks as shipped protected-main behavior.

## Refresh rule

Refresh this file when any of the following changes materially:

- protected-main SHA or package version;
- open PR/draft/issue counts;
- a priority PR head/base/review/check/merge state;
- an issue or operator-gap acceptance boundary;
- a capability's implementation maturity;
- the dependency/landing order;
- a release, deprecation, replacement, Figma file, or standards/research basis.

Keep this file operator-oriented. Never rewrite an active-PR capability as
protected-main before merge and exact-head verification. TEPP is not a purchase
catalog; this register names operator-visible gaps only.
