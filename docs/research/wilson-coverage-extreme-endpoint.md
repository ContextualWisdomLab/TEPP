# Wilson coverage endpoint stability

## Decision

`validation_core::wilson_coverage_interval` remains the Validation Evidence authority for a Wilson score interval around empirical interval-coverage proportions. The public estimand is unchanged. The numerical implementation must not turn a mathematically positive, binary64-representable Wilson endpoint into exact zero through cancellation in an avoidable intermediate expression.

For an all-covered sample (`p̂ = 1`), the ordinary Wilson lower endpoint simplifies algebraically to

`n / (n + z²)`.

Evaluating the generic center-minus-margin form first can subtract two nearly equal `O(z²)` quantities. With one covered replication and finite `z = 1e154`, `z² = 1e308` is still finite and the exact simplified lower endpoint is approximately `1e-308`, which is representable in binary64. The predecessor generic expression rounded the numerator cancellation to exact zero and therefore reported a stronger boundary statement than the represented inputs justify.

## RED → repair trace

- Public RED: `f84e5918acc81ca8bf3708f3cce2004c67675b78`, `crates/validation_core/tests/wilson_all_covered_extreme_z_contract.rs`.
- Causal repair: `fe9b9c8a5b94a01cd8416efd613503569b98ac1a`, `crates/validation_core/src/coverage.rs`.
- API: `validation_core::wilson_coverage_interval`.

The repair evaluates the exact all-covered endpoint directly as `n / (n + z²)` and returns the exact upper endpoint `1.0`. It does not change the Wilson estimand, the ordinary mixed-coverage path, interval-admission rules, or the configuration rejection for non-finite/non-positive `z` and overflowing `z²`.

## Scientific boundary

This is Validation Evidence execution arithmetic, not a psychometric estimator and not Longitudinal Modeling composition. A value of `z` is caller-supplied configuration; finite positive values remain admitted under the existing API contract. If future product policy restricts supported confidence levels, that is a separate configuration/PRD decision and must not be smuggled in as a numerical workaround.

Wilson's score construction is the methodological authority for the interval form. IEEE/ISO/IEC 60559 binary floating-point semantics explain why algebraically equivalent expressions can have different endpoint behavior in finite precision; TEPP therefore uses the algebraically reduced endpoint when it preserves a representable result.

## Reference

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953
