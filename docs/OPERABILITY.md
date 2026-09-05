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

Issue #491 owns the current bias-standard-error exact-proof budget. Production exact pair-distance admission stays bounded to `n<=16` while the current implementation is O(n²) in pair enumeration and stores `n(n-1)/2` pair records. The characterization on PR #488 records a seventeen-observation counterexample and an algebraically equivalent O(n) checked-integer numerator, but arithmetic representability alone does not authorize a wider production budget.

The current characterization distinguishes three bounds that must not be collapsed into one cutoff. At aligned diameter `D=2^53`, the minimum-shifted O(n) intermediate bound `n^2D^2` fits `u128` through `n=2_047`, while the exact pair-square numerator extremal bound `floor(n^2/4)D^2` fits through `n=4_095`. Separately, the unreduced scientific denominator `n^2(n-1)` exceeds `2^53` after `n=208_064`; production uses the reduced denominator after GCD, so that threshold is only an envelope marker. None of these values is a latency or memory budget.

Before changing the production boundary, retain:

- release-mode raw timing samples and p95 from `crates/validation_core/examples/bias_se_exact_proof_budget.rs`, with exact commit, CPU, OS, Rust toolchain, build flags, timing sample count, and cold/warm procedure;
- side-by-side buffered O(n²), allocation-free two-pass O(n²), and O(n) kernel evidence, with exact equality of restored pair-square numerators before timing;
- target `size_of::<Option<(u128, i32)>>()`, actual scratch `Vec` capacity, scratch payload bytes, and allocator/RSS evidence rather than byte estimates inferred from field widths;
- admitted/refused-set comparison between the existing pairwise proof and any stronger sufficient O(n) dyadic-grid proof, with proof refusal falling back rather than altering scientific meaning;
- checked-`u128` overflow/refusal evidence across sample count and represented exponent spread;
- a wider-integer/reference alternative assessment kept separate from production authority unless its dependency/security/performance cost is explicitly accepted;
- full service/API p95 when a buyer-facing path is affected, preserving the TEPP `p95 <= 20 ms` target without shrinking input, omitting proof work, or using an unrealistic cache-only setup.

The exact pair-record count is `n(n-1)/2`; the current characterization locks 120 records at `n=16`, 136 at `n=17`, 2,096,128 at `n=2,048`, and 4,997,541 at `n=3,162`. A two-pass O(n²) implementation may remove pair-record storage while preserving pair-enumeration proof shape, but that optimization still requires exact-head Rust/rustdoc/coverage evidence before it replaces the current implementation. A stronger O(n) admission is not accepted merely because its checked arithmetic fits `u128`.

## Model release/cutover

A model artifact is promoted only after convergence, posterior diagnostics, true-parameter/recovery benchmarks, invariance/fairness/language evidence, uncertainty/calibration, security/privacy, and reproducibility gates meet the versioned policy. Model-selection or LLM review disagreement can require human scientific review.

## Incident RCA

Trace the first failing boundary: evidence, temporal typing/reasoning, event/relation, membership, preprocessing/concept, topic estimator, psychometric estimator, compute backend, network/cluster, LLM interpretation, persistence, export/UI, or delivery pipeline. Fix the owning layer and add a realistic regression rather than compensating downstream.

## Actions workflow fleet

GitHub Actions registry identities survive YAML deletion. After any bootstrap, diagnosis, or repair workflow is removed from the tree, run `scripts/actions_workflow_fleet.py audit` and retain the JSON inventory (workflow ID, path, state, classification, default-branch SHA, timestamp, pagination receipts). Disable only re-fetched active orphans with `disable-orphans --apply`. Never disable the protected CI, documentation, hourly NIM, or hourly PR-maintenance paths, and never recreate deleted bootstrap/repair YAML. The auditor uses only `GITHUB_TOKEN`/`GH_TOKEN`. Product-development automation continues to use the owner-approved model route and must not receive unrelated provider credentials. Operator procedure: `docs/operations/ACTIONS_WORKFLOW_FLEET.md`.

## Release gate

A software release requires exact protected-head CI/security/review, 100% production coverage/docs, validated migrations/rollback where present, scientific benchmark artifacts, SBOM/provenance, reproducible packages/images, operator runbooks, accessibility for product UI, CHANGELOG/version/tag consistency, and post-publish verification. TEPP has not reached that integrated release state merely because individual foundation PRs merge. Unexecuted timing harnesses and branch-only resource characterization are not release evidence.
