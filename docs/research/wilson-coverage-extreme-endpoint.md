# Wilson coverage endpoint stability

## Decision

`validation_core::wilson_coverage_interval` remains the Validation Evidence authority for a Wilson score interval around empirical interval-coverage proportions. The public estimand is unchanged. The numerical implementation must not turn a mathematically interior, binary64-representable Wilson endpoint into exact `0.0` or `1.0` through avoidable cancellation or rounded equality in intermediate expressions.

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

The upper endpoint has the complementary identity

`U(p̂) = 1 - L(1 - p̂)`.

That identity matters when the generic `center + margin` numerator and denominator round to the same binary64 value. With two uncovered replications and `z = 2^27`, `z² / n = 2^53` exactly. The predecessor evaluates the all-uncovered upper endpoint as exact `1.0`, although the represented Wilson value correctly rounds to `next_down(1.0)`. The same false-one collapse occurs for strict-interior coverage; with one covered replication out of eight and `z = 2^28`, the represented upper endpoint also rounds to `next_down(1.0)`, not `1.0`.

The repair therefore keeps the ordinary upper calculation when it remains below one. Only when that direct path reaches exact `1.0` while uncovered mass is nonzero does TEPP evaluate the positive lower endpoint of the complementary uncovered proportion with the same rationalized form and subtract it from one. This preserves a representable nonunit endpoint without perturbing ordinary cases or redefining true boundary coverage.

## RED → repair trace

- All-covered lower public RED: `f84e5918acc81ca8bf3708f3cce2004c67675b78`, `crates/validation_core/tests/wilson_all_covered_extreme_z_contract.rs`.
- All-covered lower causal repair: `fe9b9c8a5b94a01cd8416efd613503569b98ac1a`, `crates/validation_core/src/coverage.rs`.
- Strict-interior lower public RED: `9d45f482854037d96d5dff38964fd3844335a39b`, `crates/validation_core/tests/wilson_interior_extreme_z_contract.rs`.
- Strict-interior lower causal repair: `4f259f6e5c98ade2e4a34125430de872f32c1589`, `crates/validation_core/src/coverage.rs`.
- All-uncovered upper public RED: `c070da269aa257fc9c9fa9eae17231a51ec63b74`, `crates/validation_core/tests/wilson_all_uncovered_upper_endpoint_contract.rs`.
- Strict-interior upper RED expansion: `344081bfd98ee9bc70a3bf8fdebc795a9090e692`, same public contract file.
- Complementary rationalized upper repair: `9a2fdd05c2994f51f6c72030fe39e695ba5a876d`, `crates/validation_core/src/coverage.rs`.
- Release trace: `c68076f565822ef9dd1e7c540d966cc29bf54191`, `CHANGELOG.d/validation-wilson-extreme-endpoint.md`.
- API: `validation_core::wilson_coverage_interval`.

Interval-admission rules and rejection of non-finite/non-positive `z` or overflowing `z²` remain unchanged. The new path is endpoint-representation repair, not a change to nominal coverage policy.

## Scientific boundary

This is Validation Evidence execution arithmetic, not a psychometric estimator and not Longitudinal Modeling composition. A value of `z` is caller-supplied configuration; finite positive values remain admitted under the existing API contract. If future product policy restricts supported confidence levels, that is a separate configuration/PRD decision and must not be smuggled in as a numerical workaround.

Wilson's score construction is the methodological authority for the interval form. IEEE/ISO/IEC 60559 binary floating-point semantics explain why algebraically equivalent expressions can have different endpoint behavior in finite precision; TEPP therefore uses algebraically equivalent forms that preserve representable scientific evidence across the admitted binary64 domain. As of 2026-09-04, IEEE/ISO/IEC 60559-2020 remains an active published floating-point standard, while IEEE P754 is an active revision project superseding IEEE 754-2019; an unpublished revision is not treated as current normative text.

## References

IEEE. (2020). *IEEE/ISO/IEC 60559-2020: ISO/IEC/IEEE International Standard—Floating-point arithmetic*. https://standards.ieee.org/ieee/60559/10226/

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953
