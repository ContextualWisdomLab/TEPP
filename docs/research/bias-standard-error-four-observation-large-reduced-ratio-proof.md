# Bias standard error: large reduced four-observation ratio proof

## Finding

GAP-112 correctly reduced the exact four-observation pair-distance ratio before the bounded square-root proof, but it still required the reduced numerator itself to be an exactly representable binary64 integer. That restriction was stronger than the actual proof contract: `numerator as f64` is only an initial candidate seed, while the returned value is decided by exact `u128` candidate-square and adjacent-midpoint comparisons.

A represented-input counterexample is

`r = [19_274_968, 693_729_138, 711_353_557, 1_625_519_116]`.

Every residual and pairwise difference is exactly represented in binary64. The six squared pair distances are

`454888427430388900`, `478972773352230921`, `2580020262984245904`, `310620145087561`, `868232563101240484`, and `835698669261782481`.

They sum exactly to

`N = 5218123316274976251`.

For four observations,

`SE(mean)^2 = N / 48`.

`gcd(N, 48) = 3`, so the exact reduced radicand is

`1739374438758325417 / 16`.

The reduced numerator is larger than `2^53`. GAP-112 therefore rejected the bounded midpoint proof and returned to the translated floating ratio/square-root path, which yields adjacent-lower bits `0x41b3_a706_d408_9e31`. The exact represented-input target is `0x41b3_a706_d408_9e32`.

The defect is deterministic Validation Evidence arithmetic. The estimand, sampling design, and scientific denominator are unchanged.

## RED and causal repair

Public RED `50153a1c4452d780c23b58fd34c695db9048e603` adds `crates/validation_core/tests/bias_standard_error_four_observation_large_reduced_ratio_contract.rs`. It fixes multiple permutations and sign mirrors at `0x41b3_a706_d408_9e32`.

Causal repair `dbf6b40946c0940c8b088f376a6dc1750401350e` changes only the bounded ratio proof in `crates/validation_core/src/bias_se.rs`. The reduced numerator remains exact in `u128`; its conversion to binary64 is explicitly treated only as a seed for the initial square-root candidate. No result is admitted from that seed alone. The implementation still compares the exact rational radicand with the candidate square and with the exact dyadic midpoint between adjacent binary64 candidates. If those checked `u128` comparisons overflow or the candidate cannot be settled within the existing bounded neighbor walk, the function returns `None` and preserves the established fallback.

The public admission remains narrow:

- exactly four observations;
- finite, subtraction-error-free represented residuals;
- finite, subtraction-error-free pairwise residual differences;
- checked `u128` construction of the dyadic pair-square sum;
- exact GCD reduction against the scientific denominator `48`;
- a denominator exactly representable in the existing binary64 seed path;
- exact checked candidate-square and midpoint comparisons that complete without integer overflow.

No arbitrary-precision runtime, payload-specific branch, mutable sibling dependency, or reusable static psychometric estimator is introduced. The reusable arithmetic owner boundary with `fast-mlsirm` is unchanged.

## Alternatives rejected

Keeping the `2^53` numerator admission was rejected because it confuses the approximation used to seed the search with the exact arithmetic used to accept the final result. GAP-113 corrects that proof boundary rather than changing the estimator.

Replacing the bounded proof with the floating ratio/square-root formula was rejected because that path is the demonstrated one-ULP failure.

Special-casing this residual multiset was rejected because the defect is the seed-exactness precondition, not these values.

Arbitrary-precision production arithmetic was rejected as unnecessary: the exact numerator, denominator, candidate dyadics, and midpoint comparisons for this bounded case fit the existing checked `u128` machinery. Cases that do not fit still fail closed to the established path.

GAP-113 does not claim globally correctly rounded `bias_standard_error` for arbitrary sample sizes or for every four-observation geometry.

## Standards and methodological trace

IEEE 754-2019 remains the active published IEEE floating-point authority; IEEE P754 is an active revision project rather than a published replacement. ISO/IEC 60559:2020 remains the corresponding published International Standard. The engineering consequence here is that a rounded binary64 seed may be used as a search starting point only when the acceptance decision is made against the exact represented target and binary64 rounding boundaries.

The AERA/APA/NCME public testing authority remains the 2014 *Standards for Educational and Psychological Testing*. This repair concerns the numerical integrity of Validation Evidence rather than a change to the validity argument or score interpretation.

Morris, White, and Crowther (2019) distinguish deterministic performance-measure calculation from Monte Carlo uncertainty. A fixed-input one-ULP error caused by an unnecessary fallback belongs to the former and must not be reported as simulation uncertainty.

### References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019).

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

## Traceability

- bounded context: Validation Evidence;
- public API: `validation_core::bias_standard_error`;
- exact admission: `crates/validation_core/src/bias_se.rs`;
- fallback: `crates/validation_core/src/bias.rs`;
- public RED: `crates/validation_core/tests/bias_standard_error_four_observation_large_reduced_ratio_contract.rs` at `50153a1c4452d780c23b58fd34c695db9048e603`;
- causal source repair: `dbf6b40946c0940c8b088f376a6dc1750401350e`;
- CHANGELOG: `CHANGELOG.d/validation-bias-standard-error-four-observation-large-reduced-ratio.md` beginning at `ca85765f7fce56c238641bffa57f4eb19547efb7`;
- landing vehicle: PR #488;
- predecessor retained: GAP-112 and all inherited Validation Evidence lineages remain in ancestry.
