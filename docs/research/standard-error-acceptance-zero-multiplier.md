# Standard-error exact-recovery boundaries

## Scope

`validation_core::accept_within_standard_errors` evaluates the existing Validation Evidence rule

`|estimate - target| <= k * standard_error`.

These repairs do not change the estimand or introduce a new acceptance rule. They fix binary64 execution only at the exact-recovery boundary reached when `k = 0` or `standard_error = 0`.

## Zero-multiplier scale-collapse finding

For any finite nonnegative standard error, `k = 0` makes the mathematical acceptance bound exactly zero. The gate is therefore an exact-recovery comparison: a nonzero estimate-target residual must be rejected regardless of the magnitude of the supplied standard error.

The predecessor implementation scaled `estimate`, `target`, and `standard_error` by one shared magnitude before comparing them. That protects the ordinary positive-`k` path from opposite-sign overflow, but it is not sound for the zero-multiplier boundary. With

- `estimate = f64::from_bits(1)` (the minimum positive binary64 subnormal),
- `target = 0.0`,
- `standard_error = f64::MAX`, and
- `k = 0.0`,

the scientific residual is nonzero and the acceptance bound is exactly zero. After predecessor scale reduction, however, `estimate / f64::MAX` rounds to `0.0`; the scaled residual and scaled bound both become zero and the gate incorrectly accepts.

Public RED `bd8a7c8a6d93ec262f5634cec199265521308dfd` fixes this contract through `crates/validation_core/tests/standard_error_acceptance_zero_multiplier_contract.rs`, including positive and negative minimum-subnormal residuals and an exact-equality control.

Causal repair `00ef2d90580e01494370e48ef68afbe4d0819ba8` handles `k == 0.0` in the same exact-recovery branch already used for `standard_error == 0.0`, before any scale reduction. Positive-`k` scaling and non-finite/configuration refusal remain unchanged.

## Signed-zero identity finding

Fresh review of the exact-recovery branch found a separate semantic defect. The predecessor used `f64::total_cmp` for equality. `total_cmp` intentionally distinguishes IEEE `-0.0` from `+0.0`, but the acceptance rule is numerical: both zeros satisfy `estimate - target = 0` exactly and neither sign bit denotes a distinct recovery error.

Public RED `379e65258c5675ae9fee6d84d369803f1e8a1ae3` requires `-0.0` versus `+0.0` to be accepted in both exact-recovery entry paths: zero multiplier with finite positive SE and zero SE with positive multiplier. Causal repair `55876e60b5ae553bcc4c1c41a793861b3d7e9cc8` replaces bit-order equality with finite numeric equality (`estimate == target`) only in the exact-recovery branch. NaN and infinities remain rejected before that comparison, and every nonzero finite residual remains non-equal.

This is not blanket signed-zero canonicalization. The API returns a Boolean acceptance decision, so the relevant invariant is that mathematically equal finite zero-valued estimates and targets produce one decision. Private numerical paths remain free to retain signed zero where it carries diagnostic information.

## Scientific and DDD boundary

The repair belongs to Validation Evidence because it changes only execution fidelity of an acceptance predicate over already-computed estimate, target, and uncertainty values. It is not a psychometric estimator, longitudinal composition rule, or reusable static psychometric primitive, so no fast-mlsirm source or mutable sibling dependency is introduced.

Morris, White, and Crowther treat simulation performance measures and Monte Carlo uncertainty as quantities whose definitions and evaluation rules must be explicit. Here the rule already defines the exact-recovery endpoint; binary64 normalization or representational ordering must not silently change that endpoint.

IEEE/ISO/IEC 60559-2020 remains the floating-point authority for binary64 behavior exercised by the regressions. Minimum subnormal values are valid finite inputs, and IEEE signed zeros compare numerically equal even though total ordering distinguishes their encodings.

## Traceability

- Public API: `validation_core::accept_within_standard_errors`.
- Scale-collapse RED: `bd8a7c8a6d93ec262f5634cec199265521308dfd`.
- Scale-collapse repair: `00ef2d90580e01494370e48ef68afbe4d0819ba8`.
- Signed-zero RED: `379e65258c5675ae9fee6d84d369803f1e8a1ae3`.
- Signed-zero causal repair: `55876e60b5ae553bcc4c1c41a793861b3d7e9cc8`.
- Test: `crates/validation_core/tests/standard_error_acceptance_zero_multiplier_contract.rs`.
- Production module: `crates/validation_core/src/monte_carlo.rs`.
- Landing vehicle: PR #488.
- Required delivery evidence: exact-head Rust, documentation, security/supply-chain, owned line/branch coverage, and qualifying independent review; predecessor-head evidence does not transfer.

## References

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

IEEE Computer Society. (2020). *IEEE/ISO/IEC 60559-2020: ISO/IEC/IEEE international standard—Floating-point arithmetic*. IEEE Standards Association. https://standards.ieee.org/ieee/60559/10226/
