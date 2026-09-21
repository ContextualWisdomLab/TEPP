### Fixed

- Preserve exact translated two-level bias-standard-error count geometry when the reduced count factor is any exact rational square, not only a reciprocal integer square. A 2/8 split of 10 exact represented residuals now evaluates the algebraic scale `2*|gap|/15` through the deterministic represented-sum divisor path instead of re-rounding it through sum/square/FMA/square-root reconstruction; mathematically nonzero results that cannot be represented still fail closed.
