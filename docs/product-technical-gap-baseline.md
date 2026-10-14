# Product and Technical Gap Baseline

## 2026-09-28 snapshot refresh

- Protected main is `a243f18da4a4` (#490). The dated delivery notes below are
  history bounded to their own dates; the as-built register was last audited
  at `b03cc378228d` and needs a full re-audit for the later landings.
- The open queue holds 151 pull requests, all draft. None is shipped
  protected-main behavior before merge and exact-head verification.
- Issue #175 (queue consolidation) is open again, so the GAP-012 completion
  note below describes the 2026-08-25 state only.

## 2026-08-28 active delivery queue

- Protected main now includes the Driver p.16 `std` restorations through
  `MANIFESTVARstd` (#271). `TIPREDVARstd` remains active on #272 and
  `discreteDRIFTstd` on #280; #296 adds the distinct finite-interval
  `discreteDIFFUSIONstd` map. None is implemented-main before protected merge
  at its exact reviewed head.
- The versioned TDT/CHRONOS composition landed through #269. PR #279 adds a
  bounded Allen/CHRONOS interval-consistency slice; persistence and exports
  remain product gaps under #170.
- PR #282 merged its fitted candidate-`K` topic-selection slice into #283's
  feature branch. PR #283 now carries that candidate plus the duplicate-ADR
  repair; neither capability is protected-main authority before #283 merges.
- PR #287 adds durable, tenant-isolated analysis-run request and state-event
  persistence for #166. It remains an unmerged branch candidate and does not
  establish an executable end-to-end analysis service on protected main.
- PR #288 merged typed durable-run reads, retained-session worker locking, and
  atomic artifact/terminal publication into #287's feature branch. PR #287 is
  still unmerged from protected main.
- PR #289 merged tenant-bound, evidence-digest-bound reproducibility-manifest
  materialization into the durable-worker feature branch. PR #290 carries the
  bounded one-shot executable worker; PR #292 adds its scheduler-facing exit
  classification; PR #294 adds real topic-lineage estimation and atomic
  artifact publication on that stack. Scheduler leases and protected object
  ingestion remain gaps.
- PR #291 merged durable interval-artifact persistence into #279's current
  feature head and closed; #293 remains the dependent authority candidate.

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
**Snapshot:** 2026-09-28T15:27:07Z
Facts fetched live from GitHub at 2026-09-28T15:27:07Z.
**Protected-main evidence:** `a243f18da4a4ca8a8d068c39922537f1f8ed6ad0` (merge of [PR #490](https://github.com/ContextualWisdomLab/TEPP/pull/490) central hourly development admission)
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
| Protected-main SHA | `a243f18da4a4ca8a8d068c39922537f1f8ed6ad0` (2026-09-04T17:14Z, merge of [#490](https://github.com/ContextualWisdomLab/TEPP/pull/490)) | The as-built register below was last audited at `b03cc378228d5e568fc34970fcb23dc2b452f535`; the 21 first-parent landings since then are not yet classified here. |
| Workspace members | 58 unique Rust crates | The repository is modular, but the approved target still lacks complete semantic, compute, psychometric-engine, event-intelligence, interpretation, artifact, and visual product boundaries. |
| Workspace version | `0.2.0` (aligned across every crate manifest) | A version number alone does not establish a supported product release; no signed artifact or support policy exists yet. |
| Open pull requests | **151** | Every open PR is listed in the snapshot-head register below. Per-PR delivery roles are not re-audited for this snapshot. |
| Draft pull requests | **151** | Every open PR is draft. Draft state is not approval or merge readiness. |
| Open product issues | **11** | Product-gap issues #166–#167, #169, #171–#176, #275, and #277 remain open (#170 closed; #175 is open again). The tracker also holds 245 other open issues, mostly generated repair tickets. |
| Current package version | `0.2.0` | No supported product release is established by the repository version alone; the tagged cut remains queued. |

The pull-request counts come from the live GitHub search at this snapshot. The
full exact-head classification lives in this register; re-read live GitHub
state immediately before every mutation. Passing or queued Checks on an open PR never
promote that PR to implemented-main.

### Post-#239/#266 state note

[#239](https://github.com/ContextualWisdomLab/TEPP/pull/239) (`c482ccea`) and
[#266](https://github.com/ContextualWisdomLab/TEPP/pull/266) (`c7cf34b8`) merged
as squash and landed things operators must know:

1. **network_analysis estimator repairs and provider-owned analysis-run status
   HTTP exchange:** exact two-sided Fisher z-transform p-values replace
   pseudo-p-values; fail-closed guard ordering for non-finite correlations and
   short samples; negative-effect edges excluded from the whole consensus
   perturbation pipeline; explicit validated `edge_drop_probability`;
   bounds-safe admission helpers; and the provider-owned status/read HTTP
   exchange for caller-scoped analysis-run probes. This advances GAP-009's
   estimator core beyond the #230 merge (`a69eb3e2`) it builds on.
2. **Workspace version alignment 0.1.0 → 0.2.0** across every crate manifest,
   matching the CHANGELOG `[0.2.0] - 2026-08-25` entry. The version bump is not
   itself a release: no tag, signed artifact, SBOM/provenance bundle, or support
   policy exists yet ([GAP-011](#operator-gap-register)).
3. **Driver p.16 `std`-family restorations continue on protected main:** the
   suite now includes #267/#268/#270/#271. `TIPREDVARstd` (#272) and
   `discreteDRIFTstd` (#280) remain active-PR candidates. Fitted candidate-`K`
   execution is also an unmerged candidate on #282, not protected-main behavior.

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

At the 2026-09-28 snapshot the open queue holds 151 draft PRs; see the
snapshot-head register below. Issue #175 is open again.

## Snapshot open pull-request evidence

The following snapshot-head register belongs to the canonical live snapshot
above. Review decisions,
required Checks, and mergeability remain volatile; the live GitHub API
supersedes this snapshot. `draft=false` is not approval, mergeability, or a
passing-check claim. Re-read the full SHA, current review decision, required
Checks, and branch rules immediately before every mutation.

For this baseline PR itself, the recorded SHA is its immediate publication
parent: the commit containing this table necessarily changes its own head. All
other rows record the exact live head observed at the snapshot.

| PR | Snapshot head evidence | Draft | Base | Title |
|---:|---|:---:|---|---|
| #281 | `3a1ebf42c8bcb2e4b0b6d53af748c61ad9e1f7f3` | true | main | docs(gap): refresh protected-main and eleven-PR queue |
| #283 | `8d825b5bbe23d054f33848a84615a3fa9177adbf` | true | main | fix(adr): enforce unique decision identities |
| #287 | `9fd492a3b3873d03460c1393308d3c8d1ca73f83` | true | main | feat(persistence): persist idempotent analysis runs |
| #290 | `cd911ace54a5fc075756abc4b4936a761c27596d` | true | feat/analysis-run-persistence | feat: execute durable analysis runs |
| #293 | `11a6a6ad645681789a544f74df0be29a30b53590` | true | feat/interval-consistency-export-persistence | fix(event): bind complete interval artifact authority |
| #294 | `308a62564803e04a711c88d89a9d3ce243d53653` | true | feat/analysis-worker-exit-contract | feat(worker): execute durable topic-lineage runs |
| #299 | `1906f3b3ec575c41db7d5f6ed54475987d5b1249` | true | main | feat(psychometric): restore Driver p.16 asymTIPREDEFFECTstd on main |
| #300 | `19a32037873f4e2000c942e883614881a96d59f9` | true | main | feat(psychometric): restore Driver p.16 TIPREDEFFECTstd on main |
| #301 | `9754dcfd829da7e4e02eddbf24a7885adfd82dc5` | true | main | feat(rater): add temporal monitoring bounded context |
| #302 | `1c69d0b6d12654c6f7328ff1a16a676bec471021` | true | main | feat(psychometric): restore Driver Table 3 T0TIPREDEFFECTstd on main |
| #303 | `a38776bed8703b47f315c531421fa77fe42ac399` | true | main | feat(psychometric): restore Driver Table 3 T0TDPREDEFFECTstd on main |
| #304 | `fad56b9040d00514350cb98232e792704323f839` | true | main | feat(psychometric): restore Driver p.16 TDPREDEFFECTstd on main |
| #305 | `eb6f925a360956c1f8334e2d3ae91677085bf5ee` | true | main | feat(psychometric): restore Driver p.16 discreteTIPREDEFFECTstd on main |
| #306 | `65364bdaaf2fe3d0918497f57f15d645799c3d25` | true | main | feat(psychometric): restore Driver p.16 addedTIPREDVARstd extra/extra=1 on main |
| #307 | `b984a2513b794a90e595dd36ad938d6baed0d113` | true | main | feat(psychometric): restore Driver 2017-era addedT0TIPREDVAR t0_b² v on main |
| #308 | `6f17d60877e1a7874e6fd57c19a39fd9646da1b2` | true | main | feat(psychometric): restore Driver Eq. 5 of addedT0TIPREDVAR λ² t0_b² v on main |
| #309 | `ad2e382ba8317046d4b2b25cf0628b0383ae3be6` | true | main | feat(psychometric): restore Driver Eq. 5 of addedTIPREDVAR λ² (B/a)² v on main |
| #310 | `d0693deb9ce870da36b774def33bcebd46ef89b2` | true | main | feat(longitudinal): consolidate lagged correlation and discreteDRIFTstd |
| #311 | `c73b2506f88b78a9addbb5b5088a080630450334` | true | main | feat(psychometric): restore Driver p.16 discreteDIFFUSIONstd Q_Δt/p on main |
| #312 | `7aeefc7d2518e76ae5f3b9766d5b0bd39c82bb60` | true | main | feat(psychometric): recover Kish-weighted CWC within/between |
| #313 | `256ed31f1c49a3e766213fb58af56999f1f36dc8` | true | main | feat(psychometric): restore Driver p.16 DIFFUSIONstd q/p=−2a on main |
| #314 | `272ee6cb46ba73184ac609c9edb6f187002d60f5` | true | main | feat(psychometric): restore Driver p.16 DRIFTstd on main |
| #315 | `538f9bd1c76422bc894836b65083c62544330c7c` | true | main | feat(psychometric): restore Driver p.16 TIPREDVARstd v/v=1 on main |
| #316 | `a4b31daf6abd0306e0558ed310c80610f4fb0a63` | true | main | Restore Driver p.16 LAMBDAstd as λ·√p/√θ |
| #317 | `13191e34c03f8a17828a50e5df650fa09c274b84` | true | main | Restore Driver Table 2 T0TDPREDCOV as t0_m·v |
| #318 | `67c476de500f9d7a70794d66bf37c4362dc46094` | true | main | Restore 2017-era T0TRAITEFFECT as t0_trait·trait |
| #319 | `ea72321c39feb84450ce013044046f9b77087113` | true | main | Restore 2017-era T0TRAITEFFECT Eq.3 carry as e^{aΔt} t0_trait·trait |
| #320 | `5869f78d7e115524f2f462df3df7a7d175c33aee` | true | main | feat(psychometric): restore Driver Eq. 5 of T0TRAITEFFECT carry on main |
| #321 | `92dd5f9bf782cbfb26ccd9360137f9cfda7fd9cf` | true | main | feat(psychometric): restore Driver 2017-era T0TRAITVAR t0_trait² · trait on main |
| #322 | `2e11fad5778c19eb73f49a9e38b115e985627b12` | true | main | feat(psychometric): restore Driver 2017-era T0TRAITVARstd extra/extra=1 on main |
| #324 | `a1b320891774ca28f15c39f1ae2f0ff044aa1772` | true | main | feat(psychometric): restore Driver 2017-era T0TOTALVAR extra+p_0 on main |
| #325 | `dea73f1d233ebf7291c8dd58b3b7792562135032` | true | main | feat(psychometric): recover Driver Eq. 5 of addedT0TIPREDVAR as λ² t0_b² v + θ |
| #326 | `a3f4c6366a010017622259687cb9aac82be3d5d4` | true | main | feat(psychometric): restore Driver 2017-era T0TOTALVARstd total/total=1 on main |
| #328 | `42ed97b6c761b62d44902136f4353ae98a6d1134` | true | main | feat(psychometric): recover Driver Eq. 5 of T0TOTALVAR as λ² (t0_trait² · trait + p_0) + θ |
| #329 | `e1593766877ce93a0e5c3cc3bf19a0d5cae2814a` | true | main | feat(psychometric): recover Driver Eq. 5 of T0TOTALVAR after addedT0TIPREDVAR as λ² (t0_trait² · trait + p_0 + t0_b² v) + θ |
| #330 | `0cbb221a9c13d3068dfea90ec85bb6b065e40396` | true | main | feat(psychometric): recover 2017-era commented asymTRAITVAR as trait/a² on main |
| #331 | `faf956a76bc77c01b8bd7291f98f0c2ce8326fd4` | true | main | feat(psychometric): recover Driver Eq. 5 of T0TOTALVAR after addedT0TIPREDVAR with ψ as λ² (t0_trait² · trait + p_0 + t0_b² v) + θ + ψ |
| #332 | `dabb7692685354aa252edaf61b2b381913e10e31` | true | agent/psychometric-cwc-irregular-event-lag-probe | feat(psychometric): expose grand-mean-centered event-time lag |
| #333 | `29c345b08ed7d654f256d32e101e6b15bc0e0522` | true | agent/psychometric-grand-mean-event-lag-probe | feat(psychometric): expose person-specific linear-detrend event-time lag |
| #334 | `7c4cb9adc6da5de890183cd7a42bc377ef1f8d9c` | true | main | feat(psychometric): name grand-mean pooled OLS slope and refuse as within |
| #335 | `e47b1476801906eb4c47e9cd2bed1d6623f117af` | true | main | feat(psychometric): restore Driver p.16 TDPREDVARstd v/v=1 on main |
| #336 | `153fa387eb26ef23d38bea5be1892ba75ee80fad` | true | main | feat(psychometric): recover 2017-era discreteTDPREDEFFECT e^{a Δt} m on main |
| #337 | `edc0551c145c4e7e73213202e8f3a1802e50a6b8` | true | main | feat(psychometric): recover Driver p.16 discreteTDPREDEFFECTstd on main |
| #338 | `82b02ccc3f75a092c4e0fee0715d5d3efac387d0` | true | main | feat(psychometric): recover Driver p.16 TDPREDMEANSstd on main |
| #339 | `3bc96e9041b783dcca2ea627d1d2334445dc522d` | true | main | feat(psychometric): recover Driver p.16 TIPREDMEANSstd on main |
| #340 | `80c9dc40b7c74078cf0eb63ff71ce31721e24536` | true | main | feat(psychometric): recover 2017-era commented asymTOTALVAR as p + trait/a² on main |
| #341 | `7e38950e783b0d31edb6a00daa9d6254b9c533b8` | true | main | feat(psychometric): recover 2017-era commented asymTOTALVAR after addedTIPREDVAR on main |
| #342 | `3b5bfad87aaf90b2a83fc7ca2cc37373c762ced4` | true | main | feat(psychometric): recover 2017-era commented asymTOTALVARstd on main |
| #343 | `72f57211fc41087c7eb77755b95e3714d543553c` | true | main | feat(psychometric): recover 2017-era commented discreteTRAITVAR on main |
| #344 | `786cfb71f62f6c99fcbd4266b92bc691a46a2845` | true | main | feat(psychometric): recover Eq. 5 of 2017-era commented asymTOTALVAR after addedTIPREDVAR on main |
| #345 | `98e31c7b8604e97727688288f031cecc609d4229` | true | main | feat(psychometric): recover Eq. 5 of 2017-era three-term asymTOTALVAR on main |
| #346 | `03a28e11bb33c7ffc1196a49ae64c4be142a34b6` | true | main | feat(psychometric): recover Eq. 5 of 2017-era three-term asymTOTALVAR with ψ on main |
| #347 | `aebe08784ce08fee7db0869b658368668ff702fa` | true | main | feat(psychometric): recover 2017-era commented asymptotes=TRUE TRAITVAR rewrite on main |
| #348 | `5a3f8fad061ade9d2e30687bf559ec70f9a6093b` | true | main | feat(psychometric): recover Eq. 5 of 2017-era commented discreteTRAITVAR on main |
| #349 | `721d30a643f2ca2e0f7617f1182472c0ede35eac` | true | main | feat(psychometric): recover 2017-era active asymptotes=TRUE CINT rewrite on main |
| #350 | `914a17ce375fcd4313c7b8a4a77d07cb2ebfcde6` | true | main | feat(psychometric): recover Eq. 5 of 2017-era discreteTRAITVAR with ψ on main |
| #351 | `b6aa35a08507a3623c757b37c7734a24796474f4` | true | main | feat(network): replace union-find stand-in with Leiden consensus |
| #352 | `53115b033142aa7e46217713e777c8b3862cdfca` | true | main | feat(psychometric): recover 2017-era active asymptotes=TRUE TIPREDEFFECT rewrite on main |
| #353 | `05f882d677a2bbd9922a4df003a0a5edd3c3f96d` | true | main | fix(psychometric): cover MANIFESTVARstd Display claim-boundary arms |
| #355 | `f0c8c87746f7b3826989d9afe9df26e989d0b335` | true | main | feat(psychometric): recover 2017-era active asymptotes=TRUE TIPREDEFFECT rewrite on main |
| #357 | `4996e971f7da3e768911c6546111b3588edd9d0e` | true | main | feat(psychometric): recover 2017-era active asymptotes=TRUE DIFFUSION rewrite on main |
| #363 | `c42a83cc85bfa8e54cfcdd60d9ece00d6b849926` | true | main | feat(psychometric): recover Driver later-occasion variance of predetermined T0VAR on main |
| #364 | `26d27ad30f75ecd5406b5fcd48b8c176ee155321` | true | main | feat(analysis): bind TDT/CHRONOS composition to an analysis-run profile |
| #365 | `bbeb2be303e3ab0cb0dabd36c51778dc13ed0814` | true | main | feat(psychometric): recover Eq. 5 of Driver later-occasion predetermined T0VAR on main |
| #366 | `6f90b20a714415b9ae8581bb9e1faba923026bc9` | true | main | feat(psychometric): recover Eq. 5 of 2017-era active asymptotes=TRUE DIFFUSION rewrite on main |
| #367 | `39cac61bc0adf4f0a66675091380d97196c7d548` | true | main | feat(psychometric): recover Driver lagged covariance of predetermined T0VAR on main |
| #369 | `58bfc40d1d577042ff4f3913867651c54b26feb8` | true | feat/analysis-run-collection-get-gap-003a | feat(api): retry failed and cancelled analysis runs on loopback |
| #371 | `f7d5e5a391eb2caa55bd5d337e90e719137b5041` | true | feat/analysis-run-collection-get-gap-003a | feat(api): enumerate analysis runs via loopback collection CLI |
| #372 | `47b6a6c85c69983bd843d8bb6d19001cfd3629d8` | true | feat/copy-identity-analysis-run-gap-004 | feat(analysis): bind CWC within/between slopes to an analysis-run profile |
| #374 | `7fb76d1d5b338ecfb652a11a136c984c092ada15` | true | main | feat(analysis): bind Rubin loading uncertainty to an analysis-run profile |
| #376 | `be5afa4ffc1f8539d96224813fe6460585c9fe08` | true | main | feat(analysis): bind longitudinal ESEM/DSEM composition to an analysis-run profile |
| #377 | `f80bd97c05641601d5158c12eef2aa48e7944892` | true | feat/analysis-run-retry-http-gap-003a | feat(api): inspect stored analysis-run create fields on loopback |
| #379 | `2040a763505aba35daca131a73924f268d60344d` | true | feat/analysis-run-stored-request-get-gap-003a | feat(api): inspect analysis-run retry children on loopback |
| #380 | `ed21a10579d69faf4182ef6c07004fd622983a1e` | true | feat/analysis-run-retry-lineage-get-gap-003a | feat(api): resolve analysis-run identity by idempotency key on loopback |
| #384 | `c12ea6005c808fad36e782f5d175f7d85a28170e` | true | feat/analysis-run-idempotency-lookup-get-gap-003a | feat(api): inspect analysis-run retry parent on loopback |
| #385 | `58a9644105044edd573eb4d314db926d4576b40c` | true | feat/analysis-run-cancel-cli-gap-003a | feat(api): create analysis runs via loopback CLI |
| #386 | `1801a97ab97246eb0f9ffbee52d29eae3b8e0425` | true | main | feat(analysis): bind two-group OLS invariance to an analysis-run profile |
| #387 | `e2ffc6d1cbb6c2f972efd69f236ac5cd918113e6` | true | feat/analysis-run-stored-request-get-gap-003a | feat(api): inspect stored analysis-run requests from LineageWeave and Naruon |
| #389 | `035bfb087d47543fd7dd87cfdbc4edd778f4a6aa` | true | main | feat(analysis): bind irregular event-time log-rate to an analysis-run profile |
| #391 | `061a6eaa407894ec4dc10f1d268b9147162a4e38` | true | feat/analysis-run-collection-get-gap-003a | feat(api): enumerate analysis runs from LineageWeave and Naruon |
| #393 | `520f1a4e9fc73c274201d89a9857b8d0b5d8353d` | true | feat/analysis-run-retry-http-gap-003a | feat(api): retry analysis runs from LineageWeave and Naruon compatibility |
| #394 | `deef33b68ef5a0571beec371ad20d625ce2caf58` | true | feat/analysis-run-retry-consumer-parity-gap-003a | feat(api): POST retry from published tepp-retry CLI |
| #395 | `a99cdc77d26488f01c2deb7501dc691bc820ad02` | true | feat/analysis-run-stored-request-consumer-parity-gap-003a | feat(api): inspect stored analysis-run requests via loopback CLI |
| #396 | `02eee9f07f58dd6dee5396c36cfcd0a0b228d199` | true | feat/analysis-run-retry-parent-get-gap-003a | feat(api): inspect analysis-run retry parent from LineageWeave and Naruon |
| #398 | `3070dafb9d1ba89298859271e885d1338431fc0d` | true | main | feat(analysis): bind nested ICC of posterior coordinates to an analysis-run profile |
| #399 | `0542042ca05ce1f5976d07ae568c62579a95ece7` | true | feat/analysis-run-retry-lineage-get-gap-003a | feat(api): inspect analysis-run retry children from LineageWeave and Naruon |
| #400 | `caceebb9376a6833817b70b624a559897b63b056` | true | feat/analysis-run-retry-parent-consumer-parity-gap-003a | feat(api): GET retry parent from published tepp-retry-parent CLI |
| #401 | `a3410d42d82c09dd3eb7978a54e6c46513179695` | true | feat/analysis-run-idempotency-lookup-get-gap-003a | feat(api): resolve analysis-run identity via loopback lookup CLI |
| #402 | `7034670efb5104479183fab92d913becec1db393` | true | feat/analysis-run-idempotency-lookup-get-gap-003a | feat(api): resolve analysis-run identity from LineageWeave and Naruon |
| #403 | `2c95104774c0e609d856ba8ecbde7f594703273d` | true | feat/analysis-run-retry-lineage-consumer-parity-gap-003a | feat(api): GET retry children from published tepp-retry-lineage CLI |
| #404 | `c8bcafa1c363f4dc42c63039747fe8dac82e376f` | true | main | feat(analysis): bind fitted candidate-K selection to an analysis-run profile |
| #405 | `9363372f9f77ff30d4054409579a0243b69f85d2` | true | main | feat(analysis): bind interpreter/verifier composition to an analysis-run profile |
| #406 | `04ea4c9431d23994c7649eae4ed69ba34a91bded` | true | feat/analysis-run-status-cli-gap-003a | feat(api): wait for analysis-run terminal status via loopback CLI |
| #407 | `cabfa0c58989e5d6fc316e36a36166a9621982e7` | true | main | feat(analysis): bind topic activity/dormancy/reactivation to an analysis-run profile |
| #408 | `3cf49c2bfb9a14222376b926c76b55dd55064a0f` | true | main | feat(analysis): bind joint posterior Laplace draws to an analysis-run profile |
| #409 | `ce546f4da50dbbf7bb7f8780a9c0637d932f2379` | true | main | feat(analysis): bind Pareto candidate-K selection to an analysis-run profile |
| #410 | `0c0341e39f0fb166c8fc37bb8368cf26652095f3` | true | main | feat(api): authorize purpose-bound exports via loopback CLI |
| #412 | `1286eba0b888c0e69744f852e9f85513e498fdd1` | true | main | feat(analysis): compose fitted candidate-K selection with topic lineage |
| #413 | `b38fdce0c2dc1eb244ad2abf0fc4bed42dfe656c` | true | main | feat(analysis): bind exhaustive case-deletion refit to an analysis-run profile |
| #414 | `a4409f96aa6123d12f26f8df59843aa92596a81e` | true | main | feat(api): query cutoff-safe temporal context via loopback CLI |
| #415 | `b20ed27f065ddec9c6f05cfe531ece546c3f4d2e` | true | main | feat(analysis): bind simulation method-effect labels to an analysis-run profile |
| #416 | `03f8de2ed0a0fb842d2022d411814e440df7cfb4` | true | main | feat(analysis): bind template-copy identity refusals to an analysis-run profile |
| #417 | `3678634ea698f4701f531455c8ba8c69cd651bd2` | true | main | feat(api): consolidate export retrieval GET and CLI |
| #418 | `a3cf12b3a689455cae30ba0f40c35caf650c839c` | true | main | feat(analysis): bind house-voice style refusals to an analysis-run profile |
| #419 | `8c4a7bac60cbe8ef02cc2f38e2d3017b2fc6d297` | true | main | feat(analysis): bind prompt-boilerplate refusals to an analysis-run profile |
| #421 | `067d308fb3096d2d02876230c55c71caddc32d59` | true | main | feat(analysis): bind non-lexical modality refusals to an analysis-run profile |
| #422 | `1743e87ecc6de4c92382e52a7740f12cbc2d9587` | true | main | feat(analysis): bind corpus-background refusals to an analysis-run profile |
| #423 | `2b666d94df1454157700dafe5edf5448de28d9e0` | true | main | feat(analysis): bind independent TDT link-criterion fitting to an analysis-run profile |
| #426 | `1e6b3859b771062da3d164aa2f6f183e2d77549b` | true | main | feat(analysis): bind provenance-is-not-transition refusals to an analysis-run profile |
| #427 | `b934c685e46093134ae745ab129fed93930997a6` | true | main | feat(analysis): bind copied-text residue refusals to an analysis-run profile |
| #428 | `922546166d87e76559edb30034b7231a6a679808` | true | main | feat(api): consolidate project-history collection GET and CLI |
| #431 | `6b2fd2f9a6202d1b6718298a1901516ccd6471be` | true | feat/project-history-collection-get-gap-003a | feat(api): consolidate project-history retrieval GET and CLI |
| #432 | `021843391c8b450e728703e4ec721b2f4a5b3927` | true | main | feat(analysis): bind posterior topic-context producer to an analysis-run profile |
| #435 | `eb98c3900e03b364cba63deae4076df10b3b5c3e` | true | main | docs(ddd): restore queue authority and bounded-context ownership |
| #436 | `460503b6e787362b702509faa955c4730f6d8680` | true | main | feat(api): consolidate interpretation-run create and collection adapters |
| #439 | `95dbb195943fb17ca79fc316796de738abcfc6de` | true | feat/interpretation-run-collection-get-gap-003a | feat(api): consolidate interpretation-run retrieval GET and CLI |
| #444 | `95ab519fdb39c66a574d1452e969b0b80b9e4ba9` | true | feat/export-retrieval-get-gap-003a | feat(api): consolidate export collection GET and CLI |
| #452 | `cb97aad9f87283df4d94abe8c6df61a0a476c893` | true | main | feat(api): consolidate temporal-context retrieval GET and CLI |
| #456 | `f02436236a73824c87c6043fc5d1e0b08cb0d448` | true | feat/project-history-retrieval-get-gap-003a | feat(api): consolidate project-history stored-request GET and CLI |
| #458 | `08165e3b3c929b4ae77396689549f72723ff8ff5` | true | feat/copy-identity-analysis-run-gap-004 | feat(analysis): bind outcome-order refusals to an analysis-run profile |
| #459 | `dee8b50e9b5a49b3154604ea9ccecc98626f1e33` | true | feat/export-retrieval-get-gap-003a | feat(api): consolidate export stored-request GET and CLI |
| #460 | `dfab4eab5ff733731e565a9348072b8dab2e4912` | true | feat/copy-identity-analysis-run-gap-004 | feat(analysis): bind relation-absence refusals to an analysis-run profile |
| #462 | `c1b7d627167dd7636d2975cc41cec050a5e477ba` | true | main | refactor(api): name JSON-LD node identity explicitly |
| #464 | `1b3a477242336634be2c7867b29d39979e9a6dca` | true | feat/temporal-context-retrieval-get-gap-003a | feat(api): consolidate temporal-context stored-request GET and CLI |
| #466 | `71f34b890bbd096eee152947c5e22d9778d323e8` | true | feat/export-retrieval-get-gap-003a | feat(api): consolidate export idempotency lookup and quarantine-parity adapters |
| #469 | `08cc7277cbea3b2d1d93b6663e60e8123b5cd4bb` | true | feat/interpretation-run-retrieval-get-gap-003a | feat(api): consolidate interpretation-run retrieval, lookup, and stored-request adapters |
| #480 | `0dcd198b8bfc5f92afc751a1a121624850a0f3c7` | true | main | fix(ci): delegate hourly LLM routing to released orchestrator |
| #482 | `506dbae236a4484301b704b6c6a05b20faf0fe69` | true | feat/copy-identity-analysis-run-gap-004 | feat(analysis): bind role-contradiction refusals to an analysis-run profile |
| #483 | `847d96f913bb261803ac0bd751ad7e4f51324cee` | true | feat/copy-identity-analysis-run-gap-004 | feat(analysis): bind retrospective-edge refusals to an analysis-run profile |
| #484 | `9a1be78b5342ff65e3cf2aac1e9331c68943f246` | true | feat/copy-identity-analysis-run-gap-004 | feat(analysis): bind summarizes-edge refusals to an analysis-run profile |
| #485 | `f71591864efc2beff336ced7ef35d5a013305c36` | true | feat/copy-identity-analysis-run-gap-004 | feat(analysis): bind support-edge refusals to an analysis-run profile |
| #487 | `8d89bd5f21d7188981bcd990ee5b3b1e2ab826ac` | true | feat/copy-identity-analysis-run-gap-004 | feat(analysis): bind prediction-contradiction refusals to an analysis-run profile |
| #488 | `31d55fdb29d8141031a8da1529b48fa96355b1c0` | true | main | fix(validation): preserve representable recovery metrics |
| #492 | `794ba9e6dda9f043aa499920fdf609b81b075d7e` | true | main | fix(actions): align central hourly admission contracts |
| #494 | `5431e7ca47d2a193f251a424f9b968aee95effde` | true | main | chore(deps): bump rust-toolchain from 1.98.0 to 1.98.1 |
| #497 | `6f6ee499e80b0537c363baf8d94fd4c46c91ce9d` | true | fix/contradictory-zero-count-records | test(quality): exercise Python CLI entrypoints instead of pragma suppression |
| #502 | `b2c16749fb3e9c2a067b409d2a3ae27c889f47d2` | true | main | fix(quality): keep rustdoc attached across multi-line attributes |
| #504 | `3d153b382e2382efb532f49bb1e2a8b93be0b73f` | true | feat/rubin-loading-uncertainty-analysis-run-gap-006 | test(science): add Rubin loading recovery and coverage acceptance evidence |
| #506 | `f4a7eb5c018530e57a197ab2da56532a73e41456` | true | codex/rubin-loading-scientific-acceptance | fix(validation): fail closed Rubin projection without draw provenance |
| #517 | `625237eb1834c0d4bc147c5d1078c6719da5c71d` | true | main | fix(quality): align the hourly admission contract with central dispatch |
| #520 | `13b6312b0a62788cf87011dddc47bf4e4e849a45` | true | main | fix(persistence): enforce the case half of the object naming contract |
| #521 | `0fa4b71c0e8cd88d95c70c806af648e853ea7b08` | true | fix/contradictory-zero-count-records | feat(persistence): enforce migration object naming contracts |
| #523 | `df35b316714121965cc7905bacd29a3e68380b8b` | true | main | fix(ci): stop draft churn and repair authored-line coverage gaps |
| #525 | `ca74260133988b6aa4b877add0d782fef1288ad9` | true | fix/hourly-central-admission-contract | fix(coverage): count match arms whose body is one string literal |
| #527 | `82d43a8d398863df3c2ec6facd858883f367f1b5` | true | main | feat(evidence): own immutable source snapshot receipts |
| #531 | `21e0e12691efc7eccd22ab832b83b978ee271e43` | true | fix/contradictory-zero-count-records | docs(readme): bind the crate count to the workspace |
| #538 | `f335624cea977fea7d7fe6f36871868557fe2bb7` | true | main | fix(coverage): reconcile contradictory LCOV opener counts |
| #605 | `f5670df0ca2cd1ef7626a939aa61ecef89852721` | true | main | feat(membership): add owner-issued single-membership admission |
| #615 | `10b07fcdf47f867e3c7b78ba1b8e8a2a668cc4b3` | true | feat/analysis-run-persistence | fix(persistence): enforce membership weight and share budgets |
| #639 | `40d5f72d2c03b3551593b00de891ae21f8d0e513` | true | main | fix(release): bind fail-closed reconstructable topic-lineage results (#638) |
| #648 | `0be133ae0e0a58c1a0f88a8c59df839eae971ead` | true | main | fix(actions): align hourly quality contract with central admission (#647) |

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

Protected `main` contains 58 unique Rust crate boundaries in the current
workspace manifest (as of `b03cc378`). The `members` and `default-members`
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
mlx_native_receipt
```

The traceability ledger records meaningful protected-main implementation in
immutable evidence, six clocks and interval reasoning, forward transitions,
event mention/instance separation, weighted multiple membership, cutoff-safe
splits, validation metrics, simulations, PostgreSQL slices, versioned API/export
contracts, orchestration routing, privacy authorization, release-evidence
generation, the CPU topic-measurement reference estimator, the repaired
posterior network estimator (#230 + #239), the Driver et al. (2017) SDE
recovery suite (#231/#232) and its `T0MEANSstd`/`T0VARstd` restorations
(#262/#265), followed by `asymDIFFUSIONstd`, `TRAITVARstd`,
`MANIFESTTRAITVARstd`, and `MANIFESTVARstd` (#267/#268/#270/#271), the
deterministic analysis-run execution engine, the loopback
interpretation gateway, the provider-owned analysis-run status/read HTTP
exchange (#266), the macOS-native MLX CPU receipt probe
(`mlx_native_receipt`), and VRAM-policy compute types.

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
| GAP-004 | The central shared-latent temporal/relational topic estimator is absent. | `partial` | product vertical | CPU `f64` TRSL-TM reference estimator with ALR/ILR coordinates and refusal gates is implemented-main (v0.2.0 `topic_measurement`); fitted candidate-`K`, digest-bound v2 topic-context artifact, draw-basis binding, and coverage repair from merged #282/#284–#286 are folded into open #283's feature head | [#167](https://github.com/ContextualWisdomLab/TEPP/issues/167) / [PR #283](https://github.com/ContextualWisdomLab/TEPP/pull/283) | `ada51518878f` | Revalidate and land #283 with exact-head checks and independent review; GPU, method effects, full Bayesian sampling, and topic birth/split/merge remain. This is not full #167 closure. |
| GAP-005 | Real multilingual documents are not yet transformed into validated exact-span semantic units and versioned shared concepts. | `partial` | product vertical | `e65cd66` lineage (semantic_core exact-span units and language-profile validation are implemented-main as the first slice from [PR #201](https://github.com/ContextualWisdomLab/TEPP/pull/201)) | [#168](https://github.com/ContextualWisdomLab/TEPP/issues/168) CLOSED COMPLETED 2026-08-24; residual evidence tracked under product completion (#166/#169) | `—` | Remaining evidence beyond the closed first slice: concept alignment, Unicode/layout/language-tailored processing, unknown-concept review, multilingual calibration/invariance, image-position evidence, and prompt-injection tests. |
| GAP-006 | Posterior topic measurements cannot yet be fitted through a complete cross-classified longitudinal ESEM/DSEM engine. | `partial` | product vertical | Psychometric recovery primitives, including Driver p.16 maps through `MANIFESTVARstd`, are implemented-main in `psychometric_core` through [#271](https://github.com/ContextualWisdomLab/TEPP/pull/271) | [#169](https://github.com/ContextualWisdomLab/TEPP/issues/169); bounded map slices [#272](https://github.com/ContextualWisdomLab/TEPP/pull/272), [#280](https://github.com/ContextualWisdomLab/TEPP/pull/280), and [#296](https://github.com/ContextualWisdomLab/TEPP/pull/296) are active PRs | `1f37cf4a154734a252b6bf8261748a289c0dd493` / `fe42aa19f70b398f66ee034b87284e33c0e7db2c` / `1907526a59e36ac46d24db51479a160210a42982` | Remaining: joint plausible-value uncertainty wiring, full invariance evidence, irregular event-time fitting at production scale, multiple-membership integration with posterior coordinates, and end-to-end composition under #166/#167. Recovery primitives alone are not the ESEM/DSEM engine. |
| GAP-007 | TDT detection/tracking and CHRONOS schema/forecast/temporal reasoning now compose on protected main, but interval consistency, persistence, and exports are not yet one calibrated operator workflow. | `partial` | product vertical | Versioned TDT/CHRONOS composition merged through [#269](https://github.com/ContextualWisdomLab/TEPP/pull/269) | [#170](https://github.com/ContextualWisdomLab/TEPP/issues/170) / [PR #279](https://github.com/ContextualWisdomLab/TEPP/pull/279) / [PR #293](https://github.com/ContextualWisdomLab/TEPP/pull/293); closed #291 is folded into #279's feature head | `907d3a7278592d063773cd79b34bf0ce1bd90cb0` / `11a6a6ad6456` | Land #279 with its folded persistence slice, then revalidate #293, which binds complete variable scope and cutoff provenance; versioned JSON-LD export and known-truth workflow recovery remain. |
| GAP-008 | GPU support is policy-only; no production estimator kernel has real hardware parity or declared VRAM evidence. | `accepted-target` | product vertical | `e65cd66` (VRAM policy only) | [#171](https://github.com/ContextualWisdomLab/TEPP/issues/171) / [PR #51](https://github.com/ContextualWisdomLab/TEPP/pull/51) | `1801501c4d7c` | Real CUDA/portable backend execution, CPU parity, streamed memory, bounded OOM/fallback, hardware profiles, telemetry, and no skipped-support claim. |
| GAP-009 | Topic association and cluster outputs lacked posterior-valid estimation, uncertainty, edge stability, and consensus communities. | `partial` (estimator core + repairs landed; Leiden consensus + buyer workflow remain) | product vertical | `a69eb3e2` (posterior log-ratio edge estimator merged from [PR #230](https://github.com/ContextualWisdomLab/TEPP/pull/230)) advanced by [#239](https://github.com/ContextualWisdomLab/TEPP/pull/239) (`c482ccea`): exact two-sided Fisher z-transform p-values driving Benjamini–Hochberg admission (Benjamini & Hochberg, 1995), percentile-bootstrap credible intervals and selection fractions (Efron, 1979), fail-closed guard ordering for non-finite correlations and short samples, negative-effect edges excluded from the whole consensus perturbation pipeline, explicit validated `edge_drop_probability`, bounds-safe admission helpers | [#172](https://github.com/ContextualWisdomLab/TEPP/issues/172) | — | Remaining closure evidence: repeated Leiden consensus replacing the union-find stand-in (Traag et al., 2019), known-truth network/cluster recovery at production scale, and reproducible exports wired into the end-to-end run (#166). |
| GAP-010 | Operators lack coordinated accessible visual analytics and exact-value export workflows. | `accepted-target` | product vertical | `e65cd66` (no visual workspace) | [#173](https://github.com/ContextualWisdomLab/TEPP/issues/173) | `—` (Figma work not started) | Real Figma File ID in ADR, Storybook/design tokens, ten PRD views, exact-value tables, accessible interaction/print/PDF states, provenance, and source-consistent exports. |
| GAP-011 | TEPP is not yet an operable multi-tenant service or supported release. | `partial` | product vertical | `e65cd66` (library contracts only); durable analysis-run persistence remains an unmerged candidate on #287, with typed worker transport and evidence-bound manifest loading folded through merged #288/#289; #290 adds the bounded executable worker, #292 its scheduler-facing exit classification, and #294 real topic-lineage estimation plus atomic artifact publication on the dependent stack | [#174](https://github.com/ContextualWisdomLab/TEPP/issues/174) / [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) / [PR #287](https://github.com/ContextualWisdomLab/TEPP/pull/287) / [PR #290](https://github.com/ContextualWisdomLab/TEPP/pull/290) / [PR #292](https://github.com/ContextualWisdomLab/TEPP/pull/292) / [PR #294](https://github.com/ContextualWisdomLab/TEPP/pull/294) | `36b25a9ac1cadc0ef694d19bd5b9c52516a5dfa4` | Land #287/#290, then revalidate #292/#294 through the stack; scheduler-owned durable input retention/backoff, protected object ingestion, OIDC/purpose enforcement, OpenTelemetry/SLOs, load/recovery, signed release/SBOM/provenance, assurance evidence, and support policy remain. |
| GAP-012 | The 71-PR queue obscured authority, repeatedly staled exact-head evidence, and fragmented product boundaries. | `implemented-main` (consolidation complete) | closed | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) CLOSED; queue drained through #215, the hourly scheduler, vehicles #231/#232, and individual passes | — | — | The eleven current PRs are bounded forward work; exact-head discipline stays enforced by this register's refresh rule. |
| GAP-013 | Evidence-grounded LLM interpretation is routed but not executed and validated as a production interpreter/verifier port. | `partial` | active integration | `e65cd66` lineage (routing and refusal contracts implemented-main; loopback interpretation POSTs landed via #92/#107) | [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176), [PR #69](https://github.com/ContextualWisdomLab/TEPP/pull/69), [PR #165](https://github.com/ContextualWisdomLab/TEPP/pull/165) | `8e4a3ca9cc80` / `34083c3f5d66` | Contextual-orchestrator execution, evidence citations, verifier refusals, comparable-budget ablations, provider eligibility/fallback, abstention, live/offline contract tests, and no numerical-authority escalation. |
| GAP-014 | README/TRD and some PR descriptions can lag protected-main and live queue reality. | `partial` | documentation drift | This register is synchronized to `b03cc378`; documentation validation enforces its structure | [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) (closed) | — | Reconcile any remaining README/TRD/CHANGELOG drift and keep ADR maturity current. |
| GAP-015 | There was no canonical live product/operator-gap register tied to documentation validation. | `implemented-main` | register refresh | Register and validator are implemented-main; [#278](https://github.com/ContextualWisdomLab/TEPP/pull/278) is the latest protected refresh before this snapshot | [PR #164](https://github.com/ContextualWisdomLab/TEPP/pull/164) is the merged authority | — | Regenerate after protected-main or queue changes and land each refresh only after exact-head checks and independent review. |
| GAP-016 | Hourly PR maintenance previously used an older central scheduler revision. | `implemented-main` | closed | The immutable central scheduler pin and bounded trust separation are implemented-main | — | — | Continue verifying the pinned reusable workflow and hourly caller; no open implementation PR exists for this closed slice. |
| GAP-017 | Accepted analysis runs have a terminal DTO and cutoff-safe execution on protected main after #157 merged. | `implemented-main` | closed on protected main | [PR #157](https://github.com/ContextualWisdomLab/TEPP/pull/157) merged 2026-08-25T02:53Z carrying the terminal result contract and folded cutoff-safe execution from closed stacked PR #178 | [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) (closed) / [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | — | Exact availability cutoff, snapshot binding, multiple-membership preservation, digest integrity, redacted no-eligible failure, and realistic end-to-end tests are protected-main behavior; remaining E2E composition work belongs to #166. |
| GAP-018 | Production statement and branch coverage gates are enforced at 100%. | `implemented-main` | closed | Coverage repairs through #241 are implemented-main; current PRs must continue to pass exact-head line and branch gates | — | — | Keep both gates required and add the smallest executable oracle whenever a production branch is introduced. |

## Product-completion issue register

| Issue | Product vertical | Depends on / constrains |
|---:|---|---|
| [#156](https://github.com/ContextualWisdomLab/TEPP/issues/156) **CLOSED** | Completed analysis-run result contract | Landed on protected main through PR #157 (merged 2026-08-25T02:53Z). |
| [#166](https://github.com/ContextualWisdomLab/TEPP/issues/166) | Executable end-to-end analysis run, recovery, and hot-partition readiness | Integrates all scientific/service verticals; cannot substitute placeholders or hide write skew behind an unmeasured queue. |
| [#167](https://github.com/ContextualWisdomLab/TEPP/issues/167) | Shared-latent temporal topic CPU estimator | Numerical foundation for K selection, networks, psychometrics, interpretation, and product E2E; CPU reference landed, full estimator remains. |
| [#168](https://github.com/ContextualWisdomLab/TEPP/issues/168) **CLOSED** | Multilingual semantic units and concept dictionary | Closed COMPLETED 2026-08-24; first-slice span units are implemented-main, remaining invariance/calibration evidence tracks product completion elsewhere. |
| [#169](https://github.com/ContextualWisdomLab/TEPP/issues/169) | Multilevel longitudinal ESEM/DSEM | Consumes posterior topic coordinates and membership/time contracts; recovery stack landed via #231/#232, engine composition remains. |
| [#170](https://github.com/ContextualWisdomLab/TEPP/issues/170) | TDT/CHRONOS event intelligence | Versioned composition is implemented-main; #279 carries bounded interval consistency, while persistence and exports remain. |
| [#171](https://github.com/ContextualWisdomLab/TEPP/issues/171) | Real GPU compute and parity | Accelerates production estimators only after CPU authority is stable. |
| [#172](https://github.com/ContextualWisdomLab/TEPP/issues/172) | Posterior network and consensus clustering | Estimator core plus #239 repairs landed; Leiden consensus and buyer workflow remain. |
| [#173](https://github.com/ContextualWisdomLab/TEPP/issues/173) | Accessible visual analytics and exports | Starts after stable API/artifact contracts; requires Figma and Storybook evidence. |
| [#174](https://github.com/ContextualWisdomLab/TEPP/issues/174) | Commercial deployment/release/support | Wraps a scientifically complete product without weakening gates; v0.2.0 version alignment (#239) is a prerequisite slice, not closure. |
| [#175](https://github.com/ContextualWisdomLab/TEPP/issues/175) **CLOSED** | PR queue and delivery consolidation | Queue consolidation completed at near-zero; issue closed after the residual queue drained through #239. |
| [#176](https://github.com/ContextualWisdomLab/TEPP/issues/176) | Contextual-orchestrator interpreter/verifier | Consumes validated artifacts and cannot promote scientific truth. |
| [#275](https://github.com/ContextualWisdomLab/TEPP/issues/275) | Leakage-safe longitudinal CEFR language-profile and drift analysis | Blocked on released interoperability, immutable result, psychometric, and temporal/persistence contracts; must not average ordinal CEFR labels. |
| [#277](https://github.com/ContextualWisdomLab/TEPP/issues/277) | Longitudinal CEFR development and drift from immutable result events | Consumes versioned result observations without re-owning assessment execution or base scoring; requires cutoff-safe recovery evidence. |

## Priority pull-request queue

The per-PR delivery roles recorded at the 2026-08-28 snapshot no longer
describe the queue. At 2026-09-28T15:27:07Z the queue holds 151 draft PRs, and their roles have
not been re-audited against this register. Treat the snapshot-head register
above as the inventory and each pull request's live page as authoritative
before any mutation.

## Delivery sequence

The dependency-aware product order is (✓ = landed on protected main):

1. ✓ **Consolidate delivery authority:** #175 closed; PR #164 merged; queue drained through #239.
2. ✓ **Finish live result contracts:** #156/#157 merged; the LineageWeave consumer parent #155 is implemented-main.
3. ✓ **Build validated multilingual evidence (first slice):** #168 closed COMPLETED 2026-08-24 with span units implemented-main from #201; remaining alignment/invariance evidence tracks product completion under #166/#169.
4. **Build the CPU topic estimator:** #167 — reference estimator landed; fitted candidate-`K`, topic-context artifact, draw-basis binding, and coverage repair are combined on open PR #282, while full Bayesian fitting remains.
5. **Build event intelligence and posterior networks:** #170 and #172 — the versioned TDT/CHRONOS composition is implemented-main; interval consistency is active on #279, while persistence/exports, Leiden consensus, and buyer workflow remain.
6. **Build the posterior-aware longitudinal psychometric engine:** #169 — recovery primitives through `MANIFESTVARstd` are implemented-main; #272, #280, and #296 are bounded active-PR restorations, and engine composition remains.
7. **Accelerate real kernels with parity:** #171.
8. **Complete the durable end-to-end run:** #166 — terminal-result lifecycle and analysis-run execution engine are implemented-main; full E2E validation remains.
9. **Execute and validate interpretation:** #176.
10. **Design and implement the operator workspace:** #173.
11. **Productionize and release:** #174 — v0.2.0 version alignment landed (#239); tagged cut remains pending.

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
