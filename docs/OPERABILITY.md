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

When corresponding services exist, track source ingest success/rejection and exact error class; evidence/span count and lineage completeness; future-evidence exclusion at each knowledge cutoff; temporal contradiction/path-consistency and budget exhaustion; event/link/tracking confidence and calibration; semantic-unit unknown/abstention by language; model convergence/objective/posterior diagnostics; true-recovery drift; CPU/GPU parity and fallback; VRAM/RSS/transfer/kernel time; model/LLM provider failures and evidence-verifier rejection; artifact/export provenance completeness; and tenant authorization/audit anomalies. Do not expose raw PII/source text in ordinary metrics/logs merely to gain observability.

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

Issue #491 owns the current bias-standard-error exact-proof budget. Production exact admission remains `n<=16`; larger counts are characterization evidence only. The work separates represented-input exactness, arithmetic width, exact-rounding width, and measured resource cost rather than treating one integer cutoff as all four.

The old nonnegative minimum-anchor characterization established `Σc_i <= Σc_i² <= P` and showed why raw-scale `D=2^58,n=65` refusal disappears after common dyadic-unit normalization. Odd `D=2^58+1,n=65` remains a narrow-width witness: pair numerator fits in 123 bits while cancellation products require 129 bits. `Wide256` characterization `081000289f5a52e94863026d55696ee2a4daf923` and product-width RED `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993` → repair `e9a7dee29afb97542bfe2965f850c8ab5a34368e` show the characterized cancellation products need no more than two `u128` limbs.

Represented-input reachability `5a19b6334487b43fb630abba7e487d7cf4c49960` reaches the wider numerator route at `n=4096` on `{0,1,2^53}`. Exact-rounding characterization `a8423173188fa53a26a16d3afdafeb76e114cc1d` shows represented `n=2050` needs 136-bit candidate-square and 140-bit adjacent-midpoint comparison operands even though its exact pair numerator is 118 bits. `aab9fe9115cee97225f2aa81e54a55ceafb23336` shows those comparisons can remain bounded as `Wide256` mantissa plus signed dyadic exponent. RED `f7717361ad8c5f0592688c1514c104cc1b4adabe` → repair `e4a85f53a611922be7492fe906d62ce65787c18e` integrates that comparison into the production exact rounder, and `1240ace8eb41a01fa72a4bb99df842fd550a1288` fixes both tie-to-even parity directions.

`7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943` proves represented pair/Wide256 exact-ratio equivalence for one family where the represented minimum is an exact anchor. That is not a universal anchor policy. Characterization `2bc1d2284d75154e020640adb573c1cfadf005fb` demonstrates `[0,1,2^-54,2]`, where non-anchor pair subtraction rounds but anchor `0` remains exact.

Source RED `fd9f9ff2c5c395e4cc13042232f4deef018adb48` establishes the first public anchor defect with `[0,1,2,-2^53]`: minimum `-2^53` cannot exactly translate `1`, represented anchor `0` preserves every coordinate, exact `P=243388915243820099130562543878155`, denominator `48`, and exact result `0x4320000000000001` while the predecessor translated floating-moment path returns `0x4320000000000000`. Initial repair `81ba770cc4812c8fbeb4b3529f0a73b41abbed0f` searched represented residual anchors.

Follow-up RED `9f403194a2ec1636531c2dfe9229cfb34b73d747` shows that observed anchors alone are operationally incomplete. Residuals `[1,2^-54,2,3]` have no observed universal exact anchor, but neutral dyadic anchor `0` preserves every residual exactly. On unit `2^-54`, exact `P=6490371073168534319490338297741315`; exact result `0x3fe4a7e9cb8a3491` differs by one ULP from the predecessor fallback `0x3fe4a7e9cb8a3492`.

Correctness repair `6cf30eeb549c0df0377bda1111cf46396e8282a3` established neutral zero as an admissible exact translation origin, but its production order still paid pairwise O(n²) first and then O(n²) anchor search. Source-level route-order RED `e0b324864e48a503e2aba0d2a487a0b95f5276ed` requires `neutral_zero_linear -> conditioned_observed_anchor -> pairwise_reference`. Repair `2b62bd46eb0c391327d2285c2244a76f5a1e0449` implements that sequence.

The neutral-zero kernel scans residuals once to determine the common dyadic exponent and once more to accumulate positive/negative coefficient mass and `Σc_i²`; it then computes `n*Σc_i²-(Σc_i)²` with exact `Wide256` products/subtraction. This is O(n) time with O(1) proof storage after the already-required residual vector and allocates no pair records. If the bounded zero-origin integer representation refuses, observed-anchor search remains an O(n²) conditioned fallback that may reduce coordinate dynamic range. Pairwise O(n²) now runs last as comparison/fail-closed authority while broader represented-input equivalence remains under validation.

The superseded route-order RED did not finish a hosted failing run, so it is source-level TDD evidence only. The current source adds a common-domain neutral-zero/pairwise equality unit and preserves the anchor-only public regressions; that is not yet full bounded-domain equivalence.

Operator implications:

- attempt the neutral-zero linear proof before quadratic proof work for the current bounded production route;
- do not diagnose pairwise-f64, minimum-anchor, or observed-anchor refusal as scientific invalidity when neutral-zero or a conditioned exact anchor admits the bounded dyadic proof;
- keep observed anchors only as a demonstrated dynamic-range recovery fallback and require a fixture that proves such recovery; do not retain O(n²) search merely by assumption;
- do not make row order part of proof semantics; forward/reversed/permuted fixtures must be bit-identical;
- treat signed-coordinate accumulation, Wide256 subtraction/downcast, denominator reduction, or exact-rounding failure as a fail-closed proof refusal and use later proof/fallback routes rather than weakening checks;
- keep pairwise proof as the comparison/fail-closed authority, not as unconditional first work;
- separate route observability from numerical equality. The existing resource harness records `used_wide_product` and `used_pairwise_fallback`, but production evidence still must distinguish `neutral_zero_linear`, `conditioned_observed_anchor`, `pairwise_reference`, and `generic_fallback`;
- describe storage precisely: the new kernel uses O(1) proof storage after the residual vector, while the public exact path still materializes O(n) residual storage;
- treat `n=2_047`, `4_095`, and `208_064` only as arithmetic envelope markers from older aligned characterizations, not service limits;
- before widening beyond 16, retain raw Rust 1.98.0 `--release` timing CSV, CPU/OS/build flags, p95, actual scratch capacity/payload, allocator/RSS, and any applicable buyer-path `p95<=20 ms` evidence without sample shrinkage or omitted proof work.

Exact pair-record counts remain 120 at `n=16`, 136 at `n=17`, 2,096,128 at `n=2048`, and 4,997,541 at `n=3162`. A two-pass O(n²) implementation may remove pair-record storage but still requires exact-head scientific/resource evidence. The neutral-zero Wide256 O(n) implementation removes pair enumeration for admitted geometries but still requires broad pair-equivalence, conditioned-anchor admission, exact-rounding, refusal, route, and resource evidence on one surviving production head.

No release-mode resource numbers are authoritative yet. Source-level RED runs that were superseded or cancelled are not counted as hosted RED. Predecessor Rustfmt artifact `9982621569` from `1f765a...` is not current-head formatting evidence.

## Model release/cutover

A model artifact is promoted only after convergence, posterior diagnostics, true-parameter/recovery benchmarks, invariance/fairness/language evidence, uncertainty/calibration, security/privacy, and reproducibility gates meet the versioned policy. Model-selection or LLM review disagreement can require human scientific review.

## Incident RCA

Trace the first failing boundary: evidence, temporal typing/reasoning, event/relation, membership, preprocessing/concept, topic estimator, psychometric estimator, compute backend, network/cluster, LLM interpretation, persistence, export/UI, or delivery pipeline. Fix the owning layer and add a realistic regression rather than compensating downstream.

## Actions workflow fleet

GitHub Actions registry identities survive YAML deletion. After any bootstrap, diagnosis, or repair workflow is removed from the tree, run `scripts/actions_workflow_fleet.py audit` and retain the JSON inventory (workflow ID, path, state, classification, default-branch SHA, timestamp, pagination receipts). Disable only re-fetched active orphans with `disable-orphans --apply`. Never disable the protected CI, documentation, hourly NIM, or hourly PR-maintenance paths, and never recreate deleted bootstrap/repair YAML. The auditor uses only `GITHUB_TOKEN`/`GH_TOKEN`. Product-development automation continues to use the owner-approved model route and must not receive unrelated provider credentials. Operator procedure: `docs/operations/ACTIONS_WORKFLOW_FLEET.md`.

## Release gate

A software release requires exact protected-head CI/security/review, 100% production coverage/docs, validated migrations/rollback where present, scientific benchmark artifacts, SBOM/provenance, reproducible packages/images, operator runbooks, accessibility for product UI, CHANGELOG/version/tag consistency, and post-publish verification. TEPP has not reached that integrated release state merely because individual foundation PRs merge. Unexecuted timing harnesses and branch-only resource characterization are not release evidence.