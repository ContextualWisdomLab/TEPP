# Validation bias standard error retains distinct residual roundoff

- Preserve error-free subtraction low terms when represented bias residual high parts differ but their anchor-relative exact deltas are representable.
- Evaluate the translation-invariant second moment of those exact deltas in O(n), avoiding a rounded-residual standard error that can materially overstate uncertainty.
- Keep the predecessor rounded-residual path when exact translated deltas cannot be established without another rounding step; this change does not claim globally correctly rounded standard errors.
