# Wilson coverage endpoint stability

## Decision

`validation_core::wilson_coverage_interval` remains the Validation Evidence authority for a Wilson score interval around empirical interval-coverage proportions. The public estimand is unchanged. The numerical implementation must not turn a mathematically positive, binary64-representable Wilson endpoint into exact zero through cancellation in an avoidable intermediate expression.

For an all-covered sample (`p̂ = 1`), the ordinary Wilson lower endpoint simplifies algebraically to

`n / (n + z²)`.

Evaluating the generic center-minus-margin form first can subtract two nearly equal `O(z²)` quantities. With one covered replication and finite `z = 1e154`, `z² = 1e308` is still finite and the exact simplified lower endpoint is approximately `1e-308`, which is representable in binary64. The predecessor generic expression rounded the numerator cancellation to exact zero and therefore reported a stronger boundary statement than the represented inputs justify.

The same defect exists away from the endpoint. For strict-interior empirical coverage `0 < p̂ < 1`, write the Wilson lower root as

`(A - B) / (2(n + z²))`,

where `A = z² + 2np̂` and `B = z sqrt(z² + 4np̂(1-p̂))`. Rationalizing the numerator gives the exactly equivalent form

`2np̂² / (A + B)`.

Dividing numerator and denominator through by `z²` avoids both the `A - B` cancellation and an avoidable `A + B` overflow:

`(2np̂² / z²) / (1 + 2np̂ / z² + sqrt(1 + 4np̂(1-p̂) / z²))`.

With two replications, one covered (`p̂ = 0.5`), and finite `z = 1e154`, the generic predecessor path produces an exact-zero lower endpoint because its center and margin both round to `2.5e307`. The rationalized Wilson lower endpoint is approximately `5e-309`, still representable in binary64.

## RED → repair trace

- All-covered public RED: `f84e5918acc81ca8bf3708f3cce2004c67675b78`, `crates/validation_core/tests/wilson_all_covered_extreme_z_contract.rs`.
- All-covered causal repair: `fe9b9c8a5b94a01cd8416efd613503569b98ac1a`, `crates/validation_core/src/coverage.rs`.
- Strict-interior public RED: `9d45f482854037d96d5dff38964fd3844335a39b`, `crates/validation_core/tests/wilson_interior_extreme_z_contract.rs`.
- Strict-interior causal repair: `4f259f6e5c98ade2e4a34125430de872f32c1589`, `crates/validation_core/src/coverage.rs`.
- Release trace: `2875ac5fe28cccbe8aab65baf0ace0d247cc52d3`, `CHANGELOG.d/validation-wilson-extreme-endpoint.md`.
- API: `validation_core::wilson_coverage_interval`.

The implementation keeps the ordinary generic Wilson calculation for cases where the lower endpoint remains nonzero. It uses the rationalized strict-interior lower root only when the generic path has collapsed to exact zero, so ordinary mixed-coverage results and the upper endpoint are not needlessly perturbed. Interval-admission rules and rejection of non-finite/non-positive `z` or overflowing `z²` remain unchanged.

## Scientific boundary

This is Validation Evidence execution arithmetic, not a psychometric estimator and not Longitudinal Modeling composition. A value of `z` is caller-supplied configuration; finite positive values remain admitted under the existing API contract. If future product policy restricts supported confidence levels, that is a separate configuration/PRD decision and must not be smuggled in as a numerical workaround.

Wilson's score construction is the methodological authority for the interval form. IEEE/ISO/IEC 60559 binary floating-point semantics explain why algebraically equivalent expressions can have different endpoint behavior in finite precision; TEPP therefore uses algebraically equivalent forms that preserve representable scientific evidence across the admitted binary64 domain.

## Reference

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953
