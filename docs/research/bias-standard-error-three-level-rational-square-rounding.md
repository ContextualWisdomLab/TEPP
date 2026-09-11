# Three-level rational-square bias standard-error rounding

## Problem

`validation_core::bias_standard_error` already preserves exact translated residual geometry when the represented `recovered - truth` values admit an error-free anchor translation. GAP-102 through GAP-105 added exact algebraic handling for two-level samples, and GAP-106/GAP-107 removed observation-order and anchor-conditioning effects. A remaining three-level case still reconstructed an exact rational square through normalized sums, products, division, and `sqrt`, which introduced one avoidable binary64 rounding boundary.

The public represented-input counterexample uses

- `truth = [0, 0, 0]`;
- `recovered = [0, 5/1024, 21/1024]`;
- binary64 values `0x0000_0000_0000_0000`, `0x3f74_0000_0000_0000`, and `0x3f95_0000_0000_0000`.

The minimax exact translation uses the middle level as anchor, giving `[−5/1024, 0, 16/1024]`. For a translated three-observation sample `[0, x, y]`,

`SE(mean)^2 = (x^2 + y^2 - xy) / 9`.

Here the dispersion numerator is

`25/2^20 + 256/2^20 + 80/2^20 = 361/2^20 = (19/1024)^2`,

so the represented-input target is exactly `19/3072`. Its correctly rounded binary64 value is `0x1.9555555555555p-8`, bits `0x3f79_5555_5555_5555`.

The predecessor general translated-moment path normalized to `[-0.3125, 0, 1]`, formed the squared-ratio value `0x1.40e38e38e38e4p-3`, then took `sqrt`. That produces adjacent upper `0x1.9555555555556p-8`, bits `0x3f79_5555_5555_5556`. The one-ULP displacement is deterministic representation error, not Monte Carlo uncertainty.

## RED and causal repair

Public RED `32dcab8434a9676854f3a470094aadfc4f3f417d` added `crates/validation_core/tests/bias_standard_error_three_level_rational_scale_rounding_contract.rs`. The contract requires `0x3f79_5555_5555_5555` rather than predecessor `...5556`.

Causal source repair `bee85e3df044e13a2df6c077cc87706b6cd78402` adds a bounded three-level identity path after the existing exact-translation and two-level admissions. It does not infer exactness from a rounded result. The path proceeds only when:

1. the two non-zero anchor-relative offsets have finite binary64 squares and cross-product;
2. fused multiply-add residuals prove those three products error-free;
3. the sum of squares and subtraction of the cross-product are both error-free under the existing subtraction-roundoff proof; and
4. the resulting dispersion numerator has an exactly represented binary64 square root, verified by a zero fused multiply-add residual.

Only then is that exact root divided by the scientific denominator `3` through the existing representable sum-over-count primitive, with the existing minimum-subnormal rational projection used where applicable. Any failed proof returns to the predecessor translated second-moment path. The repair therefore does not claim globally correctly rounded three-level or `n > 2` standard errors.

Contract completion `8260bc0bff11abae2b05e0a85b5c1c374b8cbd49` covers all six permutations and each sign mirror. CHANGELOG evidence is `5757dda1dc2699618d12c0f8a33913aedde67ad4`.

## Constraints and rejected alternatives

A payload-specific branch for the `5/16/19` integer triple was rejected because it would encode the counterexample rather than the scientific invariant. Replacing TEPP Validation Evidence arithmetic with arbitrary-precision production arithmetic was rejected because the defect has a narrower proof boundary and reusable static arithmetic remains owned by `fast-mlsirm`. Applying the closed form to every three-level sample was rejected because inexact products, additions, or irrational roots would create a new unverified rounding surface. Reusing the generic normalized moment merely with a different exact anchor was also rejected: GAP-107 already minimizes anchor dynamic range, but this counterexample remains one ULP wrong even under its best exact anchor.

## Scientific and standards trace

IEEE 754-2019 remains the published floating-point arithmetic authority for the binary64 operations used here. IEEE P754 is an active revision project approved on 2024-06-06 and has not replaced the published 2019 standard. ISO/IEC 60559:2020 remains the published international adoption. These sources distinguish the defined rounding of individual operations from the separate numerical-analysis question of whether an algorithm introduces avoidable intermediate rounding boundaries.

Morris, White, and Crowther (2019) distinguish performance-measure estimation from Monte Carlo uncertainty. The present defect changes the deterministic computation of the standard error of represented signed-bias observations; it must therefore be repaired before Monte Carlo uncertainty can be interpreted as simulation uncertainty rather than arithmetic error.

The AERA/APA/NCME *Standards for Educational and Psychological Testing* published edition remains the 2014 edition; the sponsoring organizations announced the Joint Committee for revision in 2024. This repair changes numerical fidelity inside Validation Evidence and does not change the construct, intended score interpretation, or validation-policy authority, so no PRD/ADR target change is required.

## Traceability

- Bounded context: Validation Evidence.
- Module/API: `crates/validation_core/src/bias.rs` → `validation_core::bias_standard_error`.
- Public contract: `crates/validation_core/tests/bias_standard_error_three_level_rational_scale_rounding_contract.rs`.
- RED: `32dcab8434a9676854f3a470094aadfc4f3f417d`.
- Causal source repair: `bee85e3df044e13a2df6c077cc87706b6cd78402`.
- Permutation/sign-mirror completion: `8260bc0bff11abae2b05e0a85b5c1c374b8cbd49`.
- CHANGELOG: `5757dda1dc2699618d12c0f8a33913aedde67ad4`.
- Owner boundary: no mutable `fast-mlsirm` or unreleased `contextual-orchestrator` source is consumed.
- Promotion boundary: exact-head hosted Rust/coverage/security/documentation evidence and qualifying independent review remain mandatory before merge or release.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE Computer Society. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE. https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
