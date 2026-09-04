# Bias standard error with distinct rounded residual highs

## Finding

`bias_standard_error` previously preserved subtraction low terms only when every rounded residual high part was identical. That condition is sufficient for the false-zero case repaired by GAP-093, but it is not necessary for discarded pairwise-subtraction mass to change a nonzero uncertainty estimate.

Let `q = 2^-54`, `truth = [q, 3q, 0]`, and `recovered = [1, 1, 1]`. The represented inputs define exact residuals

- `r1 = 1 - q`,
- `r2 = 1 - 3q`,
- `r3 = 1`.

Their exact mean is `1 - 4q/3`, so the centered deviations are `[q/3, -5q/3, 4q/3]`. Therefore

`SE = sqrt(sum((ri-r̄)^2) / (3*2)) = q * sqrt(7) / 3`,

which rounds to binary64 bits `0x3c8c_38aa_37c3_f68d`.

The predecessor first rounded each pairwise subtraction. Ties-to-even yields `[1, 1-2^-52, 1]`, whose rounded-residual path reports `0x3c96_a09e_667f_3bcd`. The difference is not a cosmetic ULP boundary: the predecessor is about 60% larger because it turns three nearby exact represented residuals into a different dispersion geometry before computing the standard error.

## Constraints

This is TEPP Validation Evidence performance-measure semantics. It does not create a reusable psychometric estimator and does not move static psychometric arithmetic away from `fast-mlsirm`. The repair must remain deterministic CPU `f64`, O(n), overflow-aware, and must not introduce arbitrary-precision runtime arithmetic or a second standard-error definition.

The branch already has specialized exact handling for n=2 and for n>2 when every rounded residual high part collapses to one value. Those contracts remain authoritative. A wider repair must not replace them or claim global correct rounding for every finite input.

## Alternatives considered

Computing the standard error from rounded residuals was rejected because the RED proves that the rounded residual vector is not the represented-input scientific target. Computing all pairwise exact residual differences was also rejected: the identity is valid, but an O(n²) implementation is unnecessary for a production validation path.

Reconstructing an arbitrary-precision residual vector in production was rejected because the represented inputs are binary64 and the owner contract requires a deterministic Rust `f64` reference. It would add runtime complexity without evidence that every path needs arbitrary precision.

The selected repair uses the existing error-free subtraction decomposition `ri = high_i + low_i`. It chooses one residual as a translation anchor and admits the refinement only when every anchor-relative `high` difference, `low` difference, and their final sum are each exactly representable in binary64. Translation leaves variance unchanged. With the anchor translated to exact zero, the second-moment identity

`SE² = (n * sum(di²) - sum(di)²) / (n² * (n-1))`

can be evaluated in O(n). The zero anchor also bounds cancellation in the numerator because `sum(di)² <= (n-1) * sum(di²)`. If exact translated deltas cannot be established, the predecessor rounded-residual path remains in force rather than silently widening the numerical claim.

## Evidence and traceability

| Evidence | Exact reference | Role |
| --- | --- | --- |
| Public RED | `ae906ad609aa4eb948b8311c1cebd975f55eb2f6` | Adds the distinct-high counterexample, sign mirror, and a no-subtraction-roundoff control. |
| Causal repair | `d59b9c30bf810a6dc6fdceaba6d20e048fad985a` | Retains error-free low terms through exact anchor-relative deltas and an O(n) translation-invariant second moment. |
| CHANGELOG | `89c018630d5baadef04a8167034951b7e378a4b2` | Records the buyer-visible uncertainty correction and bounded scope. |
| Module/API | `crates/validation_core/src/bias.rs::bias_standard_error` | Canonical TEPP Validation Evidence producer. |
| Public contract | `crates/validation_core/tests/bias_standard_error_distinct_high_roundoff_contract.rs` | Exact positive case, sign symmetry, and unchanged exact-residual control. |
| Landing vehicle | PR `#488` | Exact surviving head and hosted evidence remain PR-authoritative. |

The fix does **not** claim globally correctly rounded n>2 bias standard errors. In particular, if an anchor-relative exact residual delta itself needs rounding, this repair intentionally declines the exact-delta path until an independent represented-input counterexample justifies a further widening.

## Methodological authority

Bias and Monte Carlo uncertainty are performance measures that should be evaluated against known truth rather than implementation artifacts (Morris et al., 2019). The binary64 behavior described here is interpreted under IEEE 754-2019 / ISO/IEC 60559:2020 round-to-nearest, ties-to-even semantics. The broader validity claim remains evidence-oriented rather than numerical-method substitution: the *Standards for Educational and Psychological Testing* require validation evidence appropriate to the proposed interpretation and use of scores (AERA, APA, & NCME, 2014).

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
