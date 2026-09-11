### Fixed

- Preserve exact translated-residual `bias_standard_error` conditioning when several exact anchors exist. The Validation Evidence path now chooses the exact anchor with the smallest maximum translated magnitude and uses canonical represented `(high, low)` ordering only as a tie-breaker, preventing an avoidable one-ULP square/square-root drift while retaining permutation invariance and the existing exactness admission boundary.
