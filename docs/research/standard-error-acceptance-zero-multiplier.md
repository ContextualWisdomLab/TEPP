# Zero-multiplier standard-error acceptance

## Scope

`validation_core::accept_within_standard_errors` evaluates the existing Validation Evidence rule

`|estimate - target| <= k * standard_error`.

This repair does not change the estimand, introduce a new acceptance rule, or move arithmetic into Longitudinal Modeling. It fixes the binary64 execution of the already-public rule when `k = 0`.

## Finding

For any finite nonnegative standard error, `k = 0` makes the mathematical acceptance bound exactly zero. The gate is therefore an exact-recovery comparison: a nonzero estimate-target residual must be rejected regardless of the magnitude of the supplied standard error.

The predecessor implementation scaled `estimate`, `target`, and `standard_error` by one shared magnitude before comparing them. That protects the ordinary positive-`k` path from opposite-sign overflow, but it is not sound for the zero-multiplier boundary. With

- `estimate = f64::from_bits(1)` (the minimum positive binary64 subnormal),
- `target = 0.0`,
- `standard_error = f64::MAX`, and
- `k = 0.0`,

the scientific residual is nonzero and the acceptance bound is exactly zero. After predecessor scale reduction, however, `estimate / f64::MAX` rounds to `0.0`; the scaled residual and scaled bound both become zero and the gate incorrectly accepts.

Public RED `bd8a7c8a6d93ec262f5634cec199265521308dfd` fixes this contract through `crates/validation_core/tests/standard_error_acceptance_zero_multiplier_contract.rs`, including positive and negative minimum-subnormal residuals and an exact-equality control.

Causal repair `00ef2d90580e01494370e48ef68afbe4d0819ba8` handles `k == 0.0` in the same exact-recovery branch already used for `standard_error == 0.0`, before any scale reduction. Positive-`k` scaling, non-finite/configuration refusal, and the existing exact-comparison semantics remain unchanged.

## Scientific and DDD boundary

The repair belongs to Validation Evidence because it changes only execution fidelity of an acceptance predicate over already-computed estimate, target, and uncertainty values. It is not a psychometric estimator, longitudinal composition rule, or reusable static psychometric primitive, so no fast-mlsirm source or mutable sibling dependency is introduced.

Morris, White, and Crowther treat simulation performance measures and Monte Carlo uncertainty as quantities whose definitions and evaluation rules must be explicit. Here the rule itself already defines the zero-multiplier endpoint; binary64 normalization must not silently widen that endpoint into a nonzero tolerance.

IEEE/ISO/IEC 60559-2020 remains the floating-point authority for the binary64 projection behavior exercised by the regression. The failure is not subnormal input invalidity: it is avoidable loss of a representable nonzero input during an unnecessary scale operation after the mathematical bound is already known to be zero.

## Traceability

- Public API: `validation_core::accept_within_standard_errors`.
- RED: `bd8a7c8a6d93ec262f5634cec199265521308dfd`.
- Test: `crates/validation_core/tests/standard_error_acceptance_zero_multiplier_contract.rs`.
- Causal source repair: `00ef2d90580e01494370e48ef68afbe4d0819ba8`.
- Production module: `crates/validation_core/src/monte_carlo.rs`.
- Landing vehicle: PR #488.
- Required delivery evidence: exact-head Rust, documentation, security/supply-chain, owned line/branch coverage, and qualifying independent review; predecessor-head evidence does not transfer.

## References

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

IEEE Computer Society. (2020). *IEEE/ISO/IEC 60559-2020: ISO/IEC/IEEE international standard—Floating-point arithmetic*. IEEE Standards Association. https://standards.ieee.org/ieee/60559/10226/
