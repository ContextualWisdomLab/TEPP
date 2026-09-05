# Bias standard-error wide-linear accumulator admission bound

## Problem

Issue #491 previously required a fixture where the two-limb O(n) exact pair-numerator reference would refuse because the normalized coefficient sum or normalized square sum overflowed `u128` while the O(n²) pair numerator still fit. That search target is inconsistent with the canonical anchor-relative integer representation used by the characterization.

Let `c_i` be canonical nonnegative integer coefficients after subtracting the minimum represented value and removing the greatest common power-of-two unit. At least one `c_i` is zero. Define

`P = sum_{i<j} (c_i - c_j)^2`,
`S1 = sum_i c_i`, and
`S2 = sum_i c_i^2`.

For every nonnegative integer `c_i`, `c_i <= c_i^2`. Therefore `S1 <= S2`. Because at least one anchor coefficient is zero, the pair terms against that anchor contain every `c_i^2`, and all remaining pair terms are nonnegative. Therefore `S2 <= P`. Hence

`S1 <= S2 <= P`.

If the exact canonical pair numerator `P` fits `u128`, both O(n) accumulators necessarily fit `u128`. A pair-admitted fixture cannot fail the wider O(n) reference solely because `S1` or `S2` overflowed.

## Consequence for #491

The normalized narrow O(n) implementation can still refuse while `P` fits because the products `n*S2` and `S1^2` may require more than 128 bits before cancellation. The existing odd-diameter fixture `D = 2^58 + 1`, `n = 65` demonstrates that case: both products require 129 bits while the exact pair numerator is 123 bits. The dependency-free `Wide256` reference exists specifically to preserve those full-width products and their exact subtraction.

RED `3136739460ef0c8e13c044a7e5b04891e4f4e23d` made the next missing delivery artifact executable by requiring a narrow-to-wide-to-pair route in the resource harness before that route existed. Repair `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5` adds the candidate route `narrow O(n) -> Wide256 O(n) -> buffered pair fail-closed fallback` without changing production admission. The harness retains the predecessor `narrow O(n) -> buffered pair` hybrid as a comparison baseline.

For odd `D = 2^58 + 1`, `n = 65`, the predecessor hybrid must still select the pair fallback because its narrow products overflow. The new candidate must select the wider-product route, recover the same exact restored numerator as both O(n²) references, and report `used_wide_product=true` with `used_pairwise_fallback=false`. The power-of-two-normalized `D=2^58,n=65` geometry and odd `n=64` geometry remain narrow-path admissions. Separate route observability matters because a correct exact numerator does not by itself show whether the candidate avoided O(n²) allocation.

This proof and route repair narrow the remaining resource question. For the canonical `u128` coefficient domain, future refusal analysis should focus on correctness of the full-width multiply/subtract, exact restoration of the common dyadic unit, reduced scientific denominator and midpoint proof, and the upstream represented-residual admission contract. Searching for a pair-admitted coefficient-sum or square-sum overflow case is no longer a valid acceptance criterion.

The executable accumulator characterization is `crates/validation_core/tests/bias_standard_error_wide_linear_admission_bound_characterization.rs`. The executable timing/layout vehicle is `crates/validation_core/examples/bias_se_exact_proof_budget.rs`; it now records `used_wide_product` and `used_pairwise_fallback` independently so route selection can be audited from raw CSV.

## Decision

Do not promote the wider O(n) route into production solely from this arithmetic proof or the new characterization path. Production `validation_core::bias_standard_error` remains bounded to `n=4..=16`. A production change requires recorded Rust 1.98.0 `--release` raw CSV, CPU/OS/build metadata, allocator/RSS evidence, represented-input admission comparison, applicable full buyer-path p95 evidence, exact-head Rust/rustdoc/100% line+branch coverage/security/documentation GREEN, and qualifying independent current-head review. Pairwise proof remains the fail-closed reference until those gates are satisfied.

## Traceability

- Issue: #491
- Landing PR: #488
- Accumulator-bound characterization: `b7e4da353ac58069afd73ee7c0e8427d49993fdb`
- Narrow-wide-pair RED: `3136739460ef0c8e13c044a7e5b04891e4f4e23d`
- Narrow-wide-pair repair: `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5`
- CHANGELOG fragment: `CHANGELOG.d/validation-bias-exact-proof-budget-characterization.md`
- Production module under decision: `crates/validation_core/src/bias_se.rs`
- Exact-proof budget harness: `crates/validation_core/examples/bias_se_exact_proof_budget.rs`
