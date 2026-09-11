# Bias standard error must preserve dyadic scale through translated dispersion

## Finding

GAP-098 isolates a binary64 rounding defect in the exact translated-residual path introduced to avoid rounded-mean dispersion drift. The path proved that anchor-relative residual deltas were exactly representable, but then normalized them by the largest translated magnitude itself. When that magnitude is not a power of two, the final scale restoration can introduce a second rounding that changes `SE(mean)` even though the translated geometry was exact.

Let `u = 2^-52`, `a = 1 - 4u`, and `b = 1 + u`. With `truth = [0,0,0]` and `recovered = [a,a,b]`, every pairwise residual subtraction is exact. The represented residual gap is

`d = b - a = 5 * 2^-52`.

For a three-observation vector `[a,a,a+d]`, translation invariance gives centered deviations `[-d/3,-d/3,2d/3]`, therefore

`SE(mean) = sqrt(sum((r_i-r_bar)^2)/(3*2)) = d/3`.

The correctly rounded binary64 result is bits `0x3cba_aaaa_aaaa_aaab`. The predecessor translated to `[0,0,d]`, chose `scale=d`, evaluated the normalized geometry as `[0,0,1]`, rounded the square-root stage to the binary64 approximation of `1/3`, and then multiplied that rounded value by the non-power scale `d`. The result was one ULP low at `0x3cba_aaaa_aaaa_aaaa`. The sign-mirrored sample reproduces the same defect.

## Constraints

This remains TEPP Validation Evidence performance-measure arithmetic. It does not create a reusable psychometric estimator, does not move static psychometric ownership from `fast-mlsirm`, and does not consume mutable sibling-repository source. Production remains deterministic Rust `f64`, O(n), and fail closed when a mathematically nonzero requested result cannot be represented.

The repair must preserve GAP-092 through GAP-097: the exact two-observation identity, common-high subtraction-low recovery, exact translated residual admission, the direct normalized SE expression, and bounded fallback when translated deltas cannot be proved exact. It must not claim globally correctly rounded n>2 dispersion.

## Alternatives considered

Keeping the largest translated magnitude as the normalization scale was rejected because the RED proves that an otherwise exact translated geometry can be projected through a rounded normalized result and multiplied by a non-dyadic scale, introducing a second rounding at the scientific result boundary.

Adding arbitrary-precision rational arithmetic to production was rejected as disproportionate to the demonstrated cause. The input values are already binary64 dyadic rationals, and an exact power-of-two normalization is sufficient to preserve their represented geometry across scale reduction/restoration.

Special-casing the `[a,a,b]` three-observation identity was rejected because the defect belongs to the normalization policy, not to that sample shape. The existing translated-second-moment path should retain its general bounded admission contract.

## Selected repair

`numeric::exact_power_of_two_scale` is made crate-visible and reused by `bias::exact_translated_residual_standard_error`. The helper chooses the power-of-two binade scale already used by Validation Evidence's deterministic mean arithmetic. Dividing a represented dyadic residual delta by that scale changes only its exponent when representable, and multiplying the normalized result by the same scale restores magnitude without an additional non-dyadic rounding.

For the RED, the translated vector remains an exact dyadic geometry under the power scale. The normalized second moment therefore carries the factor `5` into the square-root ratio instead of collapsing it into a later multiplication by `d`; final power-of-two scale restoration yields the correctly rounded `d/3` result.

The existing nonzero-to-zero normalization guard remains in force. Exact translated-delta admission is unchanged, and samples that fail that proof still use the predecessor bounded fallback.

## Evidence and traceability

| Evidence | Exact reference | Role |
| --- | --- | --- |
| Public RED | `426fc5afd8aae8c2d5f81f53e5db074b480ac8b7` | Adds the exact `[a,a,b]` counterexample and sign mirror with expected bits `0x3cba_aaaa_aaaa_aaab`. |
| Shared numeric prerequisite | `d8b411f5dcf93ff21b18eaa4ff5ccd8d2863a6cd` | Makes the existing exact power-of-two scale helper crate-visible without changing its arithmetic. |
| Causal repair | `d536e3b1082c0f705a6ddd7722bf45c93ac594e3` | Uses the dyadic scale in the exact translated-residual standard-error path and updates the API contract. |
| CHANGELOG | `ebce979fdfb0ab2d94e3ba70145b0076a7ef07ca` | Records the buyer-visible one-ULP uncertainty correction and bounded scope. |
| Module/API | `crates/validation_core/src/bias.rs::bias_standard_error` | Canonical TEPP Validation Evidence producer. |
| Shared helper | `crates/validation_core/src/numeric.rs::exact_power_of_two_scale` | Existing crate-private dyadic normalization owner reused by the producer. |
| Public contract | `crates/validation_core/tests/bias_standard_error_power_scale_rounding_contract.rs` | Exact positive/sign-mirror regression contract. |
| Landing vehicle | PR `#488` | Surviving Validation Evidence head; hosted current-head evidence remains PR-authoritative. |

## Methodological authority

Morris, White, and Crowther (2019) treat bias and uncertainty as performance measures evaluated against known simulation truth and emphasize Monte Carlo uncertainty. The floating-point behavior in this repair is interpreted under IEEE 754-2019 / ISO/IEC 60559:2020. The AERA/APA/NCME *Standards for Educational and Psychological Testing* (2014) remains the published testing-standards edition while revision work continues; TEPP therefore ties validation claims to published authority and exact executable evidence rather than unpublished draft language.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
