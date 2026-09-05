# Validation bias standard error: exact four-observation ratio/sqrt rounding

- For four observations whose represented residuals and all pairwise residual differences are exact, `bias_standard_error` now uses the identity `SE(mean)^2 = Σ_{i<j}(r_i-r_j)^2 / 48` when the dyadic pair-distance numerator fits the bounded integer proof.
- The exact rational square root is rounded against adjacent binary64 midpoints instead of rounding the ratio under the square root first. This removes the deterministic one-ULP error exposed by residuals `[0, 1, 2, 7]`, whose exact represented-input target is `sqrt(29/12)` and rounds to bits `0x3ff8_df7d_a2e6_6e88`.
- Inputs that cannot prove exact represented residuals, exact pairwise differences, or bounded dyadic arithmetic retain the established Validation Evidence fallback. This does not claim globally correctly rounded bias standard errors for arbitrary sample sizes.
