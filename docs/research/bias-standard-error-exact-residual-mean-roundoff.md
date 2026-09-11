# Bias standard error must not round an exact residual mean before dispersion

## Finding

GAP-095 established that pairwise subtraction roundoff can materially change a nonzero bias standard error even when rounded residual high parts differ. Its first regression control exposed an independent boundary: the same scientific error can arise even when every `recovered - truth` subtraction is exact.

Let `a = 2^-52`, `truth = [0,0,0]`, and `recovered = [1, 1-a, 1]`. The represented residuals are already exact binary64 values. Their exact mean is `1-a/3`; their centered deviations are `[a/3, -2a/3, a/3]`, so

`SE = sqrt(sum((r_i-r_bar)^2)/(3*2)) = a/3`.

The correctly rounded binary64 result is `0x3c95_5555_5555_5555`.

The predecessor first rounded the residual mean to `next_down(1.0)`. Centering against that rounded mean created the different represented deviation vector `[2^-53, -2^-53, 2^-53]`, producing `0x3c96_a09e_667f_3bcd`. This is not a pairwise-subtraction defect: the residuals themselves are exact. The premature mean projection changes the dispersion geometry before the scientific denominator is applied.

## Constraints

This remains TEPP Validation Evidence performance-measure arithmetic. It does not create a reusable psychometric estimator and does not move static psychometric arithmetic from `fast-mlsirm`. Production remains deterministic Rust `f64`, O(n), bounded-allocation, and fail closed when a mathematically nonzero final standard error is outside binary64 range.

The repair must retain GAP-092's exact two-observation subtraction-roundoff path and GAP-093's common-high/error-free-low path. It must also retain GAP-095's exact translated-residual path for distinct rounded highs. No arbitrary-precision production runtime or second standard-error definition is introduced.

## Alternatives considered

Keeping `mean = deterministic_representable_mean(residuals)` and centering each residual on that rounded scalar was rejected because this RED proves the rounded mean itself can become an implementation artifact that changes the target dispersion.

An O(n^2) sum of all exact pairwise residual differences was rejected. The variance identity is translation invariant, so a single exact anchor is sufficient when anchor-relative residual deltas are representable.

Globally reconstructing every exact binary64 rational with arbitrary-precision integers was rejected as disproportionate to the proven boundary. If an anchor-relative delta itself cannot be represented exactly, the existing fallback remains until an independent counterexample justifies a wider numerical contract.

## Selected repair

The GAP-095 helper already expresses each represented residual as an error-free `high + low` pair and admits a translated second-moment calculation only when every anchor-relative high delta, low delta, and combined residual delta is exactly representable. GAP-096 removes the unnecessary prerequisite that at least one pairwise subtraction must have roundoff. For `n > 2`, exact residual vectors can therefore use the same bounded translation-invariant path before any rounded residual mean is made authoritative.

For the RED, anchor translation yields `[0, -2^-52, 0]`. After scale reduction the second-moment numerator is evaluated from `[0,-1,0]`, giving the exact normalized ratio `1/9` and the final `2^-52/3` result. A symmetric exact-residual control remains on the same translated identity and preserves its established result.

This does not claim globally correctly rounded n>2 bias standard errors. The admitted path proves only that the translated residual deltas themselves are representable; later binary64 multiplication, compensated summation, division, square root, and scale restoration remain bounded operations whose independent counterexamples must be demonstrated before further widening.

## Evidence and traceability

| Evidence | Exact reference | Role |
| --- | --- | --- |
| Public RED | `9265b34a2163f45bd232d628c20f725d2844f50a` | Adds the exact-residual rounded-mean counterexample and its sign mirror; replaces GAP-095's invalid predecessor-preservation control with a symmetric scientific control. |
| Causal repair | `2fa266b21069460370f30243cfb498e2022888bf` | Admits the exact translated-residual second-moment path for all larger samples when its exact-delta preconditions hold, not only when subtraction roundoff is nonzero. |
| CHANGELOG | `ab361b25ec4296165f89e4eff26e6d3521c00571` | Records the buyer-visible uncertainty correction and bounded scope. |
| Module/API | `crates/validation_core/src/bias.rs::bias_standard_error` | Canonical TEPP Validation Evidence producer. |
| Public contract | `crates/validation_core/tests/bias_standard_error_exact_residual_mean_roundoff_contract.rs` | Exact positive/sign-mirror contract with expected bits `0x3c95_5555_5555_5555`. |
| GAP-095 regression | `crates/validation_core/tests/bias_standard_error_distinct_high_roundoff_contract.rs` | Keeps the distinct-high subtraction-roundoff repair and replaces the scientifically invalid control. |
| Landing vehicle | PR `#488` | Surviving Validation Evidence head; hosted/current-head evidence remains PR-authoritative. |

## Methodological authority

Morris, White, and Crowther (2019) treat bias and uncertainty as performance measures evaluated against known simulation truth and explicitly recommend reporting Monte Carlo uncertainty. The binary64 behavior here is interpreted under IEEE 754-2019 / ISO/IEC 60559:2020 arithmetic. The AERA/APA/NCME *Standards for Educational and Psychological Testing* (2014) remains the published edition while its sponsoring organizations revise that edition; validation claims therefore remain tied to published evidence rather than an unpublished revision.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
