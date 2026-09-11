### Fixed

- `validation_core::bias_standard_error` now preserves nonzero sampling uncertainty when three or more distinct represented-input bias residuals collapse to one rounded binary64 `recovered - truth` value. When every rounded residual shares the same high part, the common high part is dispersion-invariant and the standard error is evaluated from the error-free subtraction low terms instead of reporting false zero.
- The public contract covers a three-observation represented-input boundary, its sign mirror, and an equal-residual control that remains exactly zero. The existing two-observation exact-difference path and the general non-collapsed estimator path remain unchanged.
