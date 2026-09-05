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

This proof narrows the remaining resource question. For the canonical `u128` coefficient domain, future refusal analysis should focus on correctness of the full-width multiply/subtract, exact restoration of the common dyadic unit, reduced scientific denominator and midpoint proof, and the upstream represented-residual admission contract. Searching for a pair-admitted coefficient-sum or square-sum overflow case is no longer a valid acceptance criterion.

The executable characterization is `crates/validation_core/tests/bias_standard_error_wide_linear_admission_bound_characterization.rs`. It covers deterministic compact fixtures, the power-of-two normalization boundary, the odd `2^58+1` / `n=65` narrow-refusal boundary, and every base-four composition for sample counts 2 through 7.

## Decision

Do not promote the wider O(n) reference into production solely from this arithmetic proof. Production `validation_core::bias_standard_error` remains bounded to `n=4..=16` until current-head Rust/rustdoc/coverage/security checks execute, release-mode resource evidence is recorded, and the wider path is compared against the present pairwise admission semantics. The proof removes an impossible research task; it does not waive verification, release, or independent-review gates.

## Traceability

- Issue: #491
- Landing PR: #488
- Characterization: `b7e4da353ac58069afd73ee7c0e8427d49993fdb`
- CHANGELOG fragment: `CHANGELOG.d/validation-bias-standard-error-wide-linear-admission-bound.md`
- Production module under decision: `crates/validation_core/src/bias_se.rs`
- Existing exact-proof budget harness: `crates/validation_core/examples/bias_se_exact_proof_budget.rs`
