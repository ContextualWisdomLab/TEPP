# Validation bias standard error preserves common-high low-term dispersion

- When every rounded signed residual collapses to the same binary64 high part, evaluate the subtraction low-term dispersion with the exact translated-residual second moment whenever its anchor-relative deltas are exactly representable.
- Prevent low terms such as `[-2^-54, 0, 0]` from being re-centered on a rounded low-term mean and moving the final standard error one ULP upward.
- Retain the predecessor scaled low-term path only when exact translation cannot be established; this is a bounded represented-input repair, not a global correct-rounding claim.
