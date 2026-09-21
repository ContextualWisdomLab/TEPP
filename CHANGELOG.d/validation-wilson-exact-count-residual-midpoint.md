### Fixed

- Exact-count all-covered Wilson evidence now compares the represented-input quotient residual against the adjacent binary64 midpoint before changing the direct quotient. This prevents residual compensation from forcing a one-ULP move when the finite-count correction is real but below the final endpoint's rounding resolution, while preserving earlier cases where denominator rounding genuinely changes the correctly rounded endpoint.
