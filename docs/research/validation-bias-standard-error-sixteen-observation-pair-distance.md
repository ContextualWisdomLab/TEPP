# Exact represented-input mean-bias standard error for sixteen observations

## Problem and represented-input evidence

TEPP's Validation Evidence layer treats the represented binary64 inputs as the numerical observation contract. The exact sixteen-observation residual sample

`[314_270_929, 327_661_307, 371_854_441, 398_522_837, 413_483_290, 416_184_956, 565_808_551, 682_627_163, 724_514_517, 731_058_943, 740_662_035, 970_233_120, 1_141_566_755, 1_320_628_283, 1_526_331_271, 1_992_574_092]`

has 120 pairwise distances and an exact squared-distance sum `N = 56_762_922_330_032_131_548`. With `n = 16`,

`SE(mean)^2 = N / [16^2(16-1)] = N / 3_840`.

`gcd(N,3_840)=12`, so the reduced exact radicand is `4_730_243_527_502_677_629 / 320`. The predecessor bounded pair-distance proof stopped at fifteen observations. Its translated floating second-moment/`sqrt` fallback returns `0x419c_fcbb_b78d_2ad4`; exact dyadic midpoint-square comparison returns correctly rounded `0x419c_fcbb_b78d_2ad5`.

The public RED lineage is `65d3965fd9ce8e4667f235fe33ce0f8b38ec5f6c` followed by fixture correction `5da82b2d651706c191ca191c6c077d916cbfda25`; the latter is the authoritative RED fixture. `crates/validation_core/tests/bias_standard_error_sixteen_observation_pair_distance_contract.rs` exercises original order, reverse order, an independent permutation, and sign mirrors without changing the represented multiset.

## Causal repair

Repair `a509ae9e46c8ffc2cc3ef4f0e904774ad2516e1f` changes only `crates/validation_core/src/bias_se.rs` admission from `n=4..=15` to `n=4..=16`, updates its rustdoc, admits exact-zero `n=16`, and moves the explicit fallback boundary to `n=17`.

The proof conditions are unchanged: residual subtraction and every pairwise residual difference must be error-free; dyadic coefficient construction and pair-square accumulation must fit checked `u128`; the scientific denominator remains `n^2(n-1)` and is GCD-reduced; binary64 ratio/`sqrt` is only a candidate seed; exact candidate-square and adjacent-midpoint comparisons authorize the result. Any proof failure remains on the established general path.

This is not a payload-specific branch and does not claim globally correctly rounded standard errors for arbitrary `n`. However, demonstrated one-ULP counterexamples now span every bounded sample size from `n=4` through `n=16`. Treating the next integer sample count as the scientific boundary is therefore no longer a satisfactory long-term design. The next owner work is to replace the staircase cutoff with an evidence-based proof budget: characterize O(n^2) cost, checked-`u128` overflow/refusal behavior, realistic Validation Evidence sample sizes, and whether a mathematically equivalent O(n) or wider-integer exact accumulator can preserve the same midpoint authority. Until that evidence exists, `n=17` remains fail-closed to the established translated path rather than being admitted speculatively.

## Alternatives rejected

A fixture-specific `n=16` formula is rejected because the defect is a repeated exact-rational double-rounding class. Removing the bound outright is rejected because it would silently impose quadratic work on arbitrary buyer inputs without a measured latency/resource envelope. Weakening subtraction exactness, checked integer construction, or midpoint proof would turn the reference path into an approximation and is rejected. Arbitrary-precision production arithmetic is not introduced without a DDD/performance/release decision because the current demonstrated case fits checked `u128` after GCD reduction.

## Traceability

| Item | Exact evidence |
|---|---|
| Domain owner | Validation Evidence |
| Authoritative public RED | `5da82b2d651706c191ca191c6c077d916cbfda25` |
| Causal repair | `a509ae9e46c8ffc2cc3ef4f0e904774ad2516e1f` |
| Production module/API | `crates/validation_core/src/bias_se.rs` / `validation_core::bias_standard_error` |
| Public test | `crates/validation_core/tests/bias_standard_error_sixteen_observation_pair_distance_contract.rs` |
| Predecessor doctoring | `docs/research/validation-bias-standard-error-fifteen-observation-pair-distance.md` |

## Methodological and standards basis

IEEE 754-2019 and ISO/IEC 60559:2020 remain the published floating-point arithmetic authorities for binary64 behavior. Exact midpoint comparison is used so a rounded ratio followed by `sqrt` cannot silently become authoritative when represented inputs admit an exact bounded rational proof.

Morris, White, and Crowther (2019) distinguish deterministic calculation of a performance measure from Monte Carlo uncertainty due to finite simulation replications. This defect is deterministic for a fixed represented input and belongs to the former layer. AERA, APA, and NCME's *Standards for Educational and Psychological Testing* provide the broader validity-evidence basis; an LLM judgment does not replace the numerical estimator contract or scientific acceptance evidence.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization, & International Electrotechnical Commission. (2020). *ISO/IEC 60559:2020 Information technology—Microprocessor systems—Floating-point arithmetic*.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
