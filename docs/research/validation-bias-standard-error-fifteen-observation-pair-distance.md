# Exact represented-input mean-bias standard error for fifteen observations

## Problem and represented-input evidence

TEPP's Validation Evidence layer treats the represented binary64 inputs as the numerical observation contract. For an exact fifteen-observation residual sample

`[224_611_356, 291_740_781, 326_373_353, 461_196_594, 686_313_913, 812_468_689, 889_538_573, 1_089_098_727, 1_093_012_981, 1_094_199_400, 1_387_143_595, 1_412_604_591, 1_556_072_759, 1_847_457_618, 1_990_087_657]`,

the 105 exact squared pair distances sum to `N = 65_163_338_527_647_814_596`. With `n = 15`,

`SE(mean)^2 = N / [15^2(15-1)] = N / 3150`.

`gcd(N,3150)=18`, so the reduced exact radicand is `3_620_185_473_758_211_922 / 175`. The predecessor bounded pair-distance proof stopped at fourteen observations. Its translated floating second-moment/`sqrt` fallback returns `0x41a1_254f_de99_720c`; exact dyadic midpoint-square comparison places the target above the midpoint between that value and its upper neighbor and below the next midpoint, so the correctly rounded represented result is `0x41a1_254f_de99_720d`.

Public RED `1c8a7cedd4ff846d3f3ab226cb4fa25b79650c58` adds `crates/validation_core/tests/bias_standard_error_fifteen_observation_pair_distance_contract.rs` with original order, reverse order, an independent permutation, and sign mirrors.

## Causal repair

Repair `3cc041ee1aa5f9871619c483059f5930a056f41a` changes only `crates/validation_core/src/bias_se.rs` admission from `n=4..=14` to `n=4..=15`, updates its rustdoc, admits exact-zero `n=15`, and moves the explicit fallback boundary to `n=16`.

The proof conditions are unchanged: residual subtraction and every pairwise residual difference must be error-free; dyadic coefficient construction and pair-square accumulation must fit checked `u128`; the scientific denominator remains `n^2(n-1)` and is GCD-reduced; binary64 ratio/`sqrt` is only a candidate seed; exact candidate-square and adjacent-midpoint comparisons authorize the result. Any proof failure stays on the established general path.

This is not a payload-specific branch and does not claim a globally correctly rounded standard error for arbitrary `n`. An unconditional O(n^2) reference path, speculative admission beyond the demonstrated boundary, weakening exactness checks, and arbitrary-precision production arithmetic remain rejected. The repeated one-ULP sequence through `n=15` is evidence that the numerical class persists, but a broader production-bound change still requires explicit cost/overflow evidence rather than an unmeasured cutoff removal.

## Traceability

| Item | Exact evidence |
|---|---|
| Domain owner | Validation Evidence |
| Public RED | `1c8a7cedd4ff846d3f3ab226cb4fa25b79650c58` |
| Causal repair | `3cc041ee1aa5f9871619c483059f5930a056f41a` |
| Production module/API | `crates/validation_core/src/bias_se.rs` / `validation_core::bias_standard_error` |
| Public test | `crates/validation_core/tests/bias_standard_error_fifteen_observation_pair_distance_contract.rs` |
| Predecessor doctoring | `docs/research/validation-bias-standard-error-twelve-through-fourteen-observation-pair-distance.md` |

## Methodological and standards basis

IEEE 754-2019 and ISO/IEC 60559:2020 remain the published floating-point arithmetic authorities for binary64 behavior. Exact midpoint comparison is used so a rounded ratio followed by `sqrt` cannot silently become authoritative when the represented inputs admit an exact bounded rational proof.

Morris, White, and Crowther (2019) distinguish a performance measure's deterministic calculation from Monte Carlo uncertainty due to finite simulation replications. This defect is deterministic for a fixed represented input and belongs to the former layer. AERA, APA, and NCME's *Standards for Educational and Psychological Testing* provide the broader validity-evidence basis; an LLM judgment does not replace the numerical estimator contract or scientific acceptance evidence.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization, & International Electrotechnical Commission. (2020). *ISO/IEC 60559:2020 Information technology—Microprocessor systems—Floating-point arithmetic*.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
