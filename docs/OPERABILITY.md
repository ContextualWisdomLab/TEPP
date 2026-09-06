# TEPP Operability, Recovery, and Release Guide

**Status:** Accepted target operating baseline with current maturity explicit.  
**Last reviewed:** 2026-09-06

TEPP is still an implementation-stage research/product platform. Protected main currently contains the Rust workspace/evidence foundation plus implemented-main temporal primitives (merged PRs #8 and #9). Superseded PRs #5 and #6 are historical lineage only. Database adapters are partial; model fitting, GPU, services, visual analytics, and production deployment are later targets. This guide defines the operating evidence those stages must satisfy rather than claiming they already exist. Unmerged or draft PRs are not implemented-main claims.

## Operating principles

- immutable source evidence is never overwritten to repair a downstream model;
- analyses are reproducible from source hashes, configuration, code, dependency lock, seeds, and knowledge cutoff;
- availability-time leakage gates precede model fitting;
- statistical uncertainty and abstention/failure are observable states;
- GPU/LLM/provider failure must have bounded degraded/fallback behavior where the product contract permits it;
- current-main versus active-PR versus target architecture is explicit in operator-facing evidence.

## Current foundation operation

The Rust domain packages are embedded/library boundaries. They require no production database or service deployment yet. Current recovery is reconstructing validated evidence objects from immutable authorized source artifacts and versioned wire records.

## Planned service SLIs

When corresponding services exist, track:

- source ingest success/rejection and exact error class;
- evidence/span count and lineage completeness;
- future-evidence exclusion count at each knowledge cutoff;
- temporal contradiction/path-consistency and budget exhaustion counts;
- event/link/tracking confidence and calibration;
- semantic-unit unknown/abstention rate by language;
- model convergence/ELBO/objective and posterior diagnostics;
- true-recovery/validation drift against release benchmark;
- CPU/GPU parity and fallback count;
- VRAM/RSS/transfer/kernel time;
- model/LLM provider failures and evidence-verifier rejection;
- artifact/export provenance completeness;
- tenant authorization/audit anomalies.

Do not expose raw PII/source text in ordinary metrics/logs merely to gain observability.

## Data snapshot and replay

A model run is pinned to an immutable corpus/evidence snapshot and `knowledge_cutoff`. Re-run/recovery must not silently include documents that became available later. Snapshot manifests record source hashes and relation-aware split identity sufficient to reproduce inclusion/exclusion.

## GPU degradation

Before admitting a GPU job, estimate budget and reserve margin. On OOM: classify, release transient allocations, shrink micro-batch a bounded number of times, then use the CPU reference or fail with an explicit resource state. Never silently change numerical precision/model specification to obtain success.

## LLM degradation

LLM-backed semantic/interpreter functions consume only immutable released compatible contextual-orchestrator contracts. Provider failure may retry or route only through that owner contract and its policy, or return deferred/unresolved evidence. It must not corrupt deterministic/statistical results, silently substitute a mutable owner head, or expose credentials/source beyond approved policy. Model-backed Actions use `orchestrator/free` through the approved gateway route; provider/model hard-coding and LLM numerical authority remain prohibited.

## Database target recovery

Migration `0007` (active PR) contracts policy-driven retention, legal-hold blocked deletion completion, evidence tombstones without raw-source restore, deletion requests bound to the cited policy, and analysis exclusion only for `logical_revocation`/`identity_tombstone`; live PostgreSQL evidence remains pending exact-head CI.

Before PostgreSQL becomes production state, prove migrations and rollback, tenant isolation/RLS, temporal/lineage constraints, idempotency/concurrency, backup/restore, retention/deletion, and reconstruction from immutable artifacts. Concurrent document first-insert and revise stress is implemented-main. `persistence_postgres::mark_restored_state_usable` and `assert_restore_integrity` are the current fail-closed restore gate (active PR): they revalidate tenant identity, canonical digests, same-tenant knowledge-cutoff eligibility, temporal window order, and enabled append-only triggers. They do not yet revalidate relation-aware splits or full lineage graphs; those remain separate post-restore scientific steps. The gate does not replace operator `pg_dump`/`pg_restore` runbooks.

## Numerical proof resource budgets

A numerical proof boundary is an operational resource contract when it changes asymptotic work, allocation, or buyer-path latency. It is not determined by the next sample count that happens to expose a rounding defect.

Issue #491 owns the current bias-standard-error exact-proof budget. Production exact pair-distance admission stays bounded to `n<=16` while the current implementation is O(n²) in pair enumeration and stores `n(n-1)/2` pair records. The characterization on PR #488 records a seventeen-observation counterexample and algebraically equivalent O(n) checked-integer numerators, but arithmetic representability alone does not authorize a wider production budget.

The current characterization distinguishes three bounds that must not be collapsed into one cutoff. For a canonical aligned coefficient diameter `D=2^53` **after removing the largest common power-of-two dyadic unit**, the normalized narrow O(n) distribution-independent intermediate envelope `n^2D^2` fits `u128` through `n=2_047`, while the exact aligned pair-square numerator extremal bound `floor(n^2/4)D^2` fits through `n=4_095`. Separately, the unreduced scientific denominator `n^2(n-1)` exceeds `2^53` after `n=208_064`; production uses the reduced denominator after GCD, so that threshold is only an envelope marker. None of these values is a latency or memory budget.

The shared dyadic unit is a proof obligation. The predecessor characterization judged the O(n) candidate on raw values with one zero and the rest at `D=2^58`, and therefore reported a refusal at `n=65`. That refusal was representation-dependent: canonical normalization divides every nonzero coefficient by the exact common unit `2^58`, leaving one zero and sixty-four ones. The normalized intermediates are only `4_160` and `4_096`; their difference `64` restores to the exact pair numerator `2^122`. RED `4f1bd2c343cf2d54905a07c257a570a89dc575d3` fixes this requirement, and characterization repair `d423b57797b6f7f127e61e0679f9ee9841525c77` evaluates checked O(n) admission on the normalized dyadic grid.

The corrected narrow O(n) accumulator is still a strict sufficient subset of the current pair reference under `u128`; the valid boundary uses odd diameter `D=2^58+1`, whose common dyadic unit is one. Both kernels fit at `n=64`. At `n=65`, the exact pair numerator `64D^2` is a 123-bit `u128`, while both O(n) products require 129 bits before cancellation. A normalized narrow O(n) refusal therefore still cannot become a scientific refusal.

Wider-intermediate characterization `081000289f5a52e94863026d55696ee2a4daf923` makes that distinction executable without introducing a production dependency. A test-only two-limb `Wide256` performs exact `u128 × u128` products and checked cancellation. On odd `D=2^58+1,n=65`, it recovers the same exact 123-bit pair numerator after the two 129-bit products cancel, while the narrow checked-`u128` O(n) path refuses. The characterization also fixes `(2^128-1)^2` as high limb `2^128-2`, low limb `1`.

Accumulator-bound characterization `b7e4da353ac58069afd73ee7c0e8427d49993fdb` removes a further false resource boundary. After canonical minimum anchoring, every coefficient `c_i` is a nonnegative integer and at least one coefficient is zero. Therefore `Σc_i <= Σc_i²`; the zero-anchor pair terms contain every `c_i²`, while all other pair-square terms are nonnegative, so `Σc_i² <= Σ(i<j)(c_i-c_j)²`. Any canonical pair numerator that fits `u128` consequently bounds both O(n) accumulators. A pair-admitted coefficient-sum or square-sum overflow fixture is impossible in this domain and is no longer an acceptance target.

RED `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993` → repair `e9a7dee29afb97542bfe2965f850c8ab5a34368e` closes the numerator-product-width theorem. For a canonical pair-admitted case, `S1`, `S2`, and supported `n` are all `u128` operands, so both `n*S2` and `S1²` are at most 256-bit products. Product width alone cannot justify a pair fallback after a correctly implemented `Wide256` numerator cancellation. This theorem does not automatically cover later exact-rounding products.

RED `3136739460ef0c8e13c044a7e5b04891e4f4e23d` makes the resource route executable by requiring a `narrow O(n) -> Wide256 O(n) -> pairwise fail-closed fallback` kernel before it existed. Repair `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5` implements that candidate in `crates/validation_core/examples/bias_se_exact_proof_budget.rs` without changing production arithmetic. The predecessor narrow→buffered-pair hybrid remains in the same harness so the cost of unnecessary pair allocation is measurable rather than inferred.

Represented-input characterization `5a19b6334487b43fb630abba7e487d7cf4c49960` proves that wider numerator recovery is reachable through the same binary64 subtraction gates rather than only on synthetic integer coefficients. At `n=4096`, residual classes `{0,1,2^53}` with represented zero truth make residual construction exact and limit distinct pairwise subtraction magnitudes to exact values `{1,2^53,2^53-1}`. Coefficient `1` forces the canonical common dyadic shift to zero. Both narrow products overflow, while the exact pair numerator remains the 119-bit value `664_289_479_338_799_435_974_172_876_300_357_631`; `Wide256` recovers it exactly. The unreduced denominator `68_702_699_520` is below `2^53`.

Exact-rounding-width characterization `a8423173188fa53a26a16d3afdafeb76e114cc1d` exposes the next causal boundary at represented `n=2050`. The exact pair numerator `332_306_998_946_228_931_332_463_617_650_984_961` is only 118 bits and is recovered by `Wide256`; denominator `8_610_922_500` remains below `2^53`. The binary64 ratio/square-root seed is `0x4296998e1aff78de`. The existing exact candidate-square comparison would need 136-bit scaled operands, and the upward-adjacent midpoint comparison needs 140-bit operands. Test-only `Wide256` comparisons show the exact target lies above the candidate square but below the midpoint square, proving that seed is nearest. A production route that widens only the O(n) numerator would therefore still falsely refuse this represented exact-proof case at the midpoint-comparison layer.

Operationally, the wider resource path is not one arithmetic substitution. Before production admission changes, the exact candidate-square and adjacent-midpoint comparisons must become width-safe or receive a separate proved bound while preserving exact tie-to-even behavior. The pairwise path remains a fail-closed comparison authority until represented residual conversion, canonical normalization, full-width cancellation, dyadic restoration, rational reduction, candidate/midpoint proof, current-head verification, and measured resource behavior are demonstrated together.

The corrected harness compares six numerator-resource shapes: buffered O(n²), allocation-free two-pass O(n²), normalized narrow checked O(n), the two-limb wider-product O(n) reference, the predecessor narrow→pair hybrid, and the new narrow→Wide256→pair hybrid. Before timing it requires exact restored-numerator equality. It records `used_wide_product` and `used_pairwise_fallback` independently. `D=2^58,n=65` must normalize and remain on the narrow route; odd `D=2^58+1,n=64` must remain narrow; odd `D=2^58+1,n=65` makes the predecessor hybrid use pair fallback while the new candidate must recover through `Wide256` with no pair allocation. This is route characterization, not proof that the new hybrid is production-ready.

Before changing the production boundary, retain:

- release-mode raw timing samples and p95 from `crates/validation_core/examples/bias_se_exact_proof_budget.rs`, with exact commit, CPU, OS, Rust 1.98.0 toolchain, build flags, timing sample count, and cold/warm procedure;
- side-by-side buffered O(n²), allocation-free two-pass O(n²), normalized narrow O(n), two-limb wider-product O(n), predecessor narrow→pair hybrid, and narrow→Wide256→pair candidate evidence, with exact equality of restored pair-square numerators before timing;
- route-aware timing covering common-power normalized admission and odd-diameter admitted/refused boundaries, including normalized unit exponent, `used_wide_product`, and `used_pairwise_fallback`;
- target `size_of::<Option<(u128, i32)>>()`, actual scratch `Vec` capacity, scratch payload bytes, and allocator/RSS evidence rather than byte estimates inferred from field widths;
- admitted/refused-set comparison at the actual represented-input boundary across sample count, canonical aligned coefficient diameter/exponent spread, and coefficient distribution; verify the accumulator and product-width theorems against conversion into canonical coefficients and classify any remaining refusal by scale restoration, denominator handling, exact candidate/midpoint comparison, or upstream exact-residual admission;
- width-safe exact candidate-square and both adjacent-midpoint comparisons over represented cases including ordinary nearest-neighbor selection and exact midpoint/tie-to-even cases;
- full service/API p95 when a buyer-facing path is affected, preserving the TEPP `p95 <= 20 ms` target without shrinking input, omitting proof work, or using an unrealistic cache-only setup.

The exact pair-record count is `n(n-1)/2`; the current characterization locks 120 records at `n=16`, 136 at `n=17`, 2,096,128 at `n=2,048`, and 4,997,541 at `n=3,162`. A two-pass O(n²) implementation may remove pair-record storage while preserving pair-enumeration proof shape, but that optimization still requires exact-head Rust/rustdoc/coverage evidence before it replaces the current implementation. A stronger O(n) admission is not accepted merely because its checked arithmetic fits a wider intermediate, and a narrow checked O(n) refusal is not allowed to narrow the current exact pairwise admission set.

No release-mode timing numbers are currently authoritative. The current execution environment does not provide the required Rust 1.98.0 toolchain, and hosted exact-head jobs have not produced a benchmark artifact. Unexecuted timing harnesses and branch-only resource characterization remain supporting evidence only.

## Model release/cutover

A model artifact is promoted only after convergence, posterior diagnostics, true-parameter/recovery benchmarks, invariance/fairness/language evidence, uncertainty/calibration, security/privacy, and reproducibility gates meet the versioned policy. Model-selection or LLM review disagreement can require human scientific review.

## Incident RCA

Trace the first failing boundary: evidence, temporal typing/reasoning, event/relation, membership, preprocessing/concept, topic estimator, psychometric estimator, compute backend, network/cluster, LLM interpretation, persistence, export/UI, or delivery pipeline. Fix the owning layer and add a realistic regression rather than compensating downstream.

## Actions workflow fleet

GitHub Actions registry identities survive YAML deletion. After any bootstrap, diagnosis, or repair workflow is removed from the tree, run `scripts/actions_workflow_fleet.py audit` and retain the JSON inventory (workflow ID, path, state, classification, default-branch SHA, timestamp, pagination receipts). Disable only re-fetched active orphans with `disable-orphans --apply`. Never disable the protected CI, documentation, hourly NIM, or hourly PR-maintenance paths, and never recreate deleted bootstrap/repair YAML. The auditor uses only `GITHUB_TOKEN`/`GH_TOKEN`. Product-development automation continues to use the owner-approved model route and must not receive unrelated provider credentials. Operator procedure: `docs/operations/ACTIONS_WORKFLOW_FLEET.md`.

## Release gate

A software release requires exact protected-head CI/security/review, 100% production coverage/docs, validated migrations/rollback where present, scientific benchmark artifacts, SBOM/provenance, reproducible packages/images, operator runbooks, accessibility for product UI, CHANGELOG/version/tag consistency, and post-publish verification. TEPP has not reached that integrated release state merely because individual foundation PRs merge. Unexecuted timing harnesses and branch-only resource characterization are not release evidence.
