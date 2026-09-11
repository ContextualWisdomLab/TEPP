# Longitudinal binary64 mean owner handoff

## Decision status

**Proposed consumer boundary.** This record does not activate a new numerical algorithm. TEPP keeps the current Longitudinal Modeling RED until the reusable numerical owner has protected-integrated and immutably released a contract that satisfies the evidence below.

## Consumer finding

TEPP PR #310 exposes a compact public RED through `crates/longitudinal_core/tests/mixed_sign_mean_rounding_contract.rs`.

The represented rate values are `P = 0x1.8p+106` (`0x4698000000000000`), `-2^53`, and `-1`. The exact real sum is `P - 2^53 - 1`. The correctly rounded mean is `0x1.fffffffffffffp+104` (`0x467fffffffffffff`), while TEPP's current `scaled_compensated_mean` path returns `0x4680000000000000`, one ULP high because the same-side coalescing step can lose the final `-1` before the original-count division.

This is an ordinary finite three-value input through the public already-centered residual path. It is not a resource-extreme synthetic witness. The latest TEPP source-level RED authority remains Rust Foundation run `34595076660`: formatting and Clippy passed, 1,568/1,569 tests passed, and only `mixed_sign_mean_rounding_contract::half_ulp_tail_changes_the_final_mixed_sign_rounding` failed with the expected bit mismatch.

## Bounded-context ownership

TEPP owns the temporal estimand: event-time admission, Driver-style log-rate construction, CWC semantics, longitudinal evidence composition, and the decision to publish or refuse a result.

Reusable finite binary64 sum/mean arithmetic is domain-neutral numerical infrastructure and belongs to `ContextualWisdomLab/fast-mlsirm`. Canonical owner issue `fast-mlsirm#1814` and implementation PR `fast-mlsirm#1816` own this primitive. TEPP must not copy the owner source, pin a mutable PR head, introduce a second generic summation algorithm, or reinterpret an unreleased owner branch as dependency authority.

Repository-owned GPU/CI acquisition and parity evidence is separately owned by `fast-mlsirm#1717`. TEPP and the numerical PR must not duplicate that CI repair, lower its GPU capability invariant, substitute CPU fallback as GPU evidence, or treat a skipped parity test as acceptance.

The immutable fast-mlsirm release remains `v0.9.1`; it predates this contract.

## Fresh owner evidence — 2026-09-11

The live `fast-mlsirm#1816` head is `432765ccf633c9802e0f796ceeb4d6d572059acf` on protected `main@493326f2de49ea1704da0ded19868ed05d2fe00f`. The PR is open, **Draft**, mergeable, and unmerged. Its owner lane has now currentized the body to the same head and Draft-containment state; no predecessor review or hosted success is promoted across the head change.

The numerical implementation entered at `b5fec1d33c02e8ea969405e59747f0505e690d20`. Subsequent ordinary-forward commits added the independent exact oracle and directed cancellation/non-finite fixtures (`087df1d...`), Proposed ADR-0029 (`9a8614f...`), doctoring/changelog currentization, touched-function rustdoc coverage (`f00c31c...`), and finally official ISO/IEEE record links (`432765c...`). The `f00c31c... -> 432765c...` delta changes only `docs/doctoring/correctly-rounded-binary64-mean.md`; the production arithmetic is unchanged.

Current exact-head hosted evidence on `432765c...` is mixed and therefore non-mergeable:

- native CodeQL `34606295285`, Security Scan `34606295363`, SAST Semgrep `34606295323`, and ClusterFuzzLite `34606295428` are GREEN;
- repository CI `34606295259` is RED because `gpu-smoke` job `103285576158` failed while installing the software Vulkan adapter, before Vulkan availability or GPU parity executed. Package and fuzz jobs are GREEN. That run is historical evidence of a failed acquisition path, not the current root-cause authority for GPU readiness;
- required delegated CodeQL PR `34606295286` is RED. Python job `103285476466` and Actions job `103285476530` both successfully read the current-head dispatch verdict and then failed at `Release runner or enforce current-head CodeQL verdict`; only afterward did dispatch job `103286411396` succeed. This remains the central producer/consumer settlement class and must be repaired at the canonical `.github` owner rather than copied into TEPP or fast-mlsirm;
- current formal reviews are COMMENTED only. The CodeRabbit oracle/standards-link findings are resolved, but there is no qualifying submitted current-head `APPROVED` review;
- current Noema evidence admitted `orchestrator/free` but the verdict request ended in HTTP 502; the exact consumer evidence belongs to the contextual-orchestrator owner path rather than a leaf provider/model retry;
- current central coverage contexts still do not provide authoritative exact-head Rust owned line+branch evidence for this material Rust implementation; no denominator trick or Python line-touch substitute is acceptable;
- current Strix evidence failed before an authoritative scan during sandbox/bootstrap control, so absence of a scan is not a clean security result.

## Canonical GPU/CI owner evidence

The stronger current GPU diagnosis is `fast-mlsirm#1717`, not another patch inside #1816. Its live exact head is `0b31640928e07f4362ce27dad3d310e630ab1b5d`, open/Draft/mergeable on the same protected fast-mlsirm base.

#1717's controlled predecessor admission successfully acquired and initialized image-local SwiftShader, then measured the actual adapter contract. `SwiftShader Device (Subzero)` exposes `max_storage_buffers_per_shader_stage = 10`, while the current marginal GPU layout requires at least 18 storage buffers per shader stage. The adapter therefore cannot execute the governed kernel topology. Lowering the 18-buffer requirement without redesign evidence, accepting CPU fallback, or relabeling skipped GPU parity as success would weaken the product contract.

The #1717 forward repair also moved the environment-specific capacity probe out of ordinary workspace tests into `crates/mlsirm-core/examples/gpu_adapter_capacity.rs`, so ordinary `cargo test --workspace` no longer performs hardware acquisition outside the dedicated GPU lane. Its exact-head CI `34607995391` reached `gpu-smoke` job `103291084354`, configured image-local SwiftShader, proved the Vulkan loader, ran the isolated capacity probe, reproduced the 10-versus-18 buffer mismatch, and failed closed at that invariant. The workflow was subsequently cancelled when the PR returned to Draft; the completed capacity RED remains diagnostic evidence, not a GREEN run or release gate.

The next causal GPU repair therefore belongs to #1717 or its verified successor: either provide a reproducible adapter that satisfies the existing 18-buffer contract, or redesign the marginal resource topology and prove CPU-`f64` parity plus realistic performance/recovery without weakening scientific or GPU acceptance. #1816 must reacquire its own exact-head repository evidence only after that canonical CI path is protected-integrated; TEPP must wait for the resulting immutable numerical release.

## Proposed numerical contract

The owner candidate publishes the proposed identity `fast_mlsirm.binary64_mean@1.0.0`. Every finite binary64 value is represented exactly as an integer multiple of `q = 2^-1074`. Positive and negative totals are accumulated separately in fixed 34×`u64` magnitudes, the exact signed magnitude is divided by the original slice cardinality, and only the final rational result is projected to binary64 with round-to-nearest, ties-to-even. `exact_zero` distinguishes exact represented cancellation from a nonzero mean that rounds to signed zero.

For the largest finite binary64 value, the coefficient in `q` units is `(2^53 - 1) * 2^2045`, requiring 2,098 magnitude bits. On supported `usize::BITS <= 64` targets, any materializable same-sign slice total is `< 2^2162`; 34 `u64` limbs provide 2,176 magnitude bits. The fixed width is therefore a representation bound, not a psychometric sample ceiling.

The final mean must be rounded from the exact rational `(S / n) * 2^-1074`, not from a binary64-rounded sum. This also permits same-sign cases whose exact intermediate sum exceeds binary64 while their mean remains representable.

## Owner acceptance already encoded in tests

The current owner test surface includes the TEPP half-ULP counterexample and mirrored sign, `[1e16, -1, -1]`, `[f64::MAX, 1e-16, -f64::MAX]`, exact cancellation versus nonzero underflow, minimum-subnormal residue after MAX cancellation, subnormal/normal boundaries, normal and subnormal ties-to-even, binade carry, same-sign `f64::MAX`, permutation invariance, and empty/NaN/±infinity refusal. It also carries a deterministic 10,000-case subnormal-domain oracle using independent test-only exact integer/rational arithmetic rather than the production 34-limb accumulator.

Those tests are necessary but not sufficient. One unchanged owner head still needs repository Rust/rustdoc/Clippy, actual owned statement and branch/edge coverage, package/fuzz/security/supply-chain evidence, zero valid unresolved findings, qualifying independent review, and terminal central controls. The canonical GPU capacity RED, delegated CodeQL failure, missing authoritative Rust coverage evidence, Noema owner failure, and pre-scan Strix failure keep that bar open.

## Rejected TEPP-local repairs

The following are not causal acceptance:

- adding another swallowed-term or pair-order special case to `scaled_compensated_mean`;
- replacing it with plain Kahan, Neumaier, sorting/coalescing, or pre-scaling without a final-rounding proof;
- computing a rounded floating sum first and dividing by the sample count afterward;
- adding a product-specific sample ceiling to simplify the numerical proof;
- copying fast-mlsirm #1536 private partials or #1816 source into TEPP;
- pinning a mutable owner PR/head;
- copying or modifying #1717's GPU/CI acquisition logic in TEPP or #1816;
- weakening the 18-buffer GPU capability contract, accepting CPU fallback as GPU parity, or accepting a skipped GPU test;
- describing faithful or approximate behavior as correctly rounded;
- treating native CodeQL/security success as a substitute for failed repository/GPU evidence, delegated CodeQL receipt, authoritative Rust coverage, independent review, semantic-review evidence, or immutable release.

## Release and consumer acceptance

Before TEPP changes production arithmetic, the fast-mlsirm owner chain must close in order: #1717 or a verified successor must establish normal GPU/CI acceptance; `fast-mlsirm#1816` must then reacquire its exact-current repository evidence, land through the normal protected path, and publish the numerical contract in a new immutable versioned release with version/tag/package, SBOM/provenance, reproducibility and rollback evidence. ADR-0029 remains Proposed until that acceptance is complete.

After release, TEPP must pin the released contract through the approved dependency/ACL boundary, remove the local generic mean heuristic rather than retaining two numerical authorities, rerun the public longitudinal RED to GREEN, and reacquire exact-head formatting, Clippy, no-retry tests, rustdoc, 100% owned line/branch/edge coverage, dependency/security policy, SBOM/provenance, live PostgreSQL, OpenCode/Noema, independent review, protected merge, and TEPP release evidence. LLM review cannot substitute for numerical or scientific acceptance.

## Research and standards trace

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). https://www.iso.org/standard/80985.html

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://standards.ieee.org/ieee/315/6210/

Ogita, T., Rump, S. M., & Oishi, S. (2005). Accurate sum and dot product. *SIAM Journal on Scientific Computing, 26*(6), 1955–1988. https://doi.org/10.1137/030601818

Rump, S. M., Ogita, T., & Oishi, S. (2008a). Accurate floating-point summation part I: Faithful rounding. *SIAM Journal on Scientific Computing, 31*(1), 189–224. https://doi.org/10.1137/050645671

Rump, S. M., Ogita, T., & Oishi, S. (2008b). Accurate floating-point summation part II: Sign, K-fold faithful and rounding to nearest. *SIAM Journal on Scientific Computing, 31*(2), 1269–1302. https://doi.org/10.1137/07068816X

These sources motivate the numerical design and final-rounding requirements. They do not prove the concrete fast-mlsirm implementation or authorize TEPP production activation by themselves.
