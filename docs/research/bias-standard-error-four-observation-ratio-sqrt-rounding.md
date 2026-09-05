# Bias standard error: four-observation exact ratio/square-root rounding

## Finding

The Validation Evidence CPU reference could still move an exactly defined represented-input bias standard error by one binary64 ULP after GAP-110. The remaining defect was not residual subtraction, translation-anchor selection, overflow/underflow, or Monte Carlo uncertainty. It occurred in the general translated path when an exact finite dispersion numerator was first divided by the scientific denominator in binary64 and only then square-rooted.

The public reproducer uses four exactly represented residuals

`r = [0, 1, 2, 7]`.

For `n` observations,

`Σ_{i<j}(r_i-r_j)^2 = n Σ_i (r_i-r̄)^2`,

so

`SE(mean)^2 = Σ_{i<j}(r_i-r_j)^2 / [n^2(n-1)]`.

For this four-observation sample the six squared pair distances sum to

`1 + 4 + 49 + 1 + 36 + 25 = 116`,

hence

`SE(mean)^2 = 116 / 48 = 29 / 12`.

The exact represented-input target is therefore `sqrt(29/12)`. Rounding the exact real value to binary64 gives bits `0x3ff8_df7d_a2e6_6e88`. The GAP-110 predecessor instead evaluates the ratio in binary64 before `sqrt`; that intermediate rounding moves the final result to adjacent lower bits `0x3ff8_df7d_a2e6_6e87`.

This is a deterministic performance-measure arithmetic error. It must not be reported as finite-replication Monte Carlo uncertainty.

## RED and causal repair

Public RED `978c422cbdccff02605b5d220bd2564900a830d5` adds `crates/validation_core/tests/bias_standard_error_four_observation_ratio_sqrt_rounding_contract.rs`. The contract checks several permutations of `[0,1,2,7]` and their sign mirrors, fixing the represented-input expectation at `0x3ff8_df7d_a2e6_6e88`.

The repair remains inside TEPP's Validation Evidence bounded context. `crates/validation_core/src/bias_se.rs` is a narrow admission service in front of the established `bias.rs` fallback. It is used only when all of the following are proved:

- the sample has exactly four observations;
- every `recovered - truth` residual is finite and error-free in binary64;
- every pairwise residual difference is finite and error-free;
- each nonzero pair distance can be represented as an exact dyadic coefficient times a common power-of-two unit;
- the sum of squared integer coefficients fits the bounded `u128` proof;
- exact comparison with the adjacent binary64 square-root midpoint can itself be completed without integer overflow.

The pair-distance identity then supplies the exact rational radicand without a rounded `numerator / 48` becoming authoritative. A binary64 `sqrt` of the rounded ratio is used only as a candidate. The implementation compares the exact rational radicand with the square of the adjacent-float midpoint and chooses the correctly rounded neighbor, including ties-to-even. Failure to prove any precondition returns to the predecessor `bias.rs` implementation rather than widening admission.

Source lineage for the repair is `3de7a73781576b3ad2b58d0c5bd5341ebf2300c2` → `ea609474a102dbf0ed3cd544c200f542a59e2760` → role-naming/refinement `2c68909557a62d09d9719df20e8ffd1e644ea7a1` → `622d1681a10006d2a67ed9c615626014c7253b92` → `170f77eec2f9d72dedea1f932f9be687e7849f8b`. CHANGELOG lineage starts at `49c46aa4ec80a006a5d265a2a9c7c2281e989697`.

## Alternatives rejected

Changing every bias-SE evaluation from `sqrt(numerator / denominator)` to `sqrt(numerator) / sqrt(denominator)` was rejected. The two evaluation orders have different rounding surfaces; neither is generally correctly rounded for an exact rational square root. A broad formula replacement would therefore exchange one class of one-ULP errors for another.

A payload-specific branch for `[0,1,2,7]` was rejected because the scientific invariant is the exact four-observation pair-distance identity, not one fixture. Arbitrary-precision production arithmetic was also rejected: it would create a new reusable numerical authority in TEPP and exceed the causal scope. Static reusable psychometric arithmetic remains owned by fast-mlsirm.

The repair deliberately does not assert globally correctly rounded `bias_standard_error` for arbitrary `n > 2`. Larger samples and four-observation samples that cannot satisfy the bounded exact proof remain on the existing deterministic fallback and are candidates for later findings only when a realistic represented-input counterexample is established.

## Standards and methodological trace

IEEE P754 is currently an Active PAR, approved 2024-06-06, to revise/supersede IEEE 754-2019; it is not yet a published replacement. ISO/IEC 60559:2020 remains a published International Standard (stage 60.60) specifying floating-point formats, arithmetic, exceptions, and uniquely determined results for specified operations and destination formats. These standards support treating the sequence of rounding operations as part of the numerical contract rather than assuming algebraically equivalent source expressions are representation-equivalent.

The AERA/APA/NCME public testing authority remains the 2014 *Standards for Educational and Psychological Testing* while a Joint Committee is revising that edition. No unpublished revision is treated as normative authority here.

Morris, White, and Crowther (2019) frame simulation evaluation around known truth, estimands, methods, and explicit performance measures, and separately require Monte Carlo standard errors for uncertainty caused by finite simulation repetitions. That distinction is material here: `bias_standard_error` is itself a deterministic performance-measure calculation on represented inputs, so a reproducible one-ULP arithmetic error is not Monte Carlo error.

### References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019).

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

## Traceability

- bounded context: Validation Evidence;
- public API: `validation_core::bias_standard_error`;
- implementation admission: `crates/validation_core/src/bias_se.rs`;
- established fallback: `crates/validation_core/src/bias.rs`;
- public RED: `crates/validation_core/tests/bias_standard_error_four_observation_ratio_sqrt_rounding_contract.rs` at `978c422cbdccff02605b5d220bd2564900a830d5`;
- repair lineage: `3de7a737...` through `170f77ee...`;
- CHANGELOG: `CHANGELOG.d/validation-bias-standard-error-four-observation-ratio-sqrt-rounding.md`;
- landing vehicle: PR #488;
- predecessor retained: GAP-110 and all inherited Validation Evidence lineages remain in ancestry.
